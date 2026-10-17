//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!

use crate::cu::mark;
use anchor_lang::prelude::*;
use floor_types::constants::{BPS, TEAM_TIP_CAP_DIVISOR, TEAM_TIP_INTERVAL_SECS};

use crate::errors::HookError;
use crate::mech::positions::{
    add, deploy_bid, intake, math, nudge, preview_raise, raise_pol, sub, sweep_seed_fees, teardown_bid, Mech,
};
use crate::mech::stair::StairOutcome;
use crate::state::FloorPool;

///
///
///
///
///
///
///
pub fn circulating(s: &FloorPool, supply: u64) -> Result<u64> {
    floor_math::circulating_supply(math::state(s), supply).map_err(|_| error!(HookError::InvariantViolated))
}

///
///
///
///
pub fn is_noop(s: &FloorPool, current_tick: i32) -> Result<bool> {
    if s.needs_ratchet != 0 || s.unpaid_skim != 0 || s.vault_eth >= s.min_bid {
        return Ok(false);
    }
    let spacing = s.tick_spacing;
    let bid_can_rise = s.bid_liquidity.get() != 0
        && math::dear_edge(s)? < s.bid_lower
        && math::ceil_to_spacing(current_tick + 1, spacing)? < s.bid_lower;
    Ok(!bid_can_rise)
}

///
///
///
///
///
///
///
///
///
///
///
///
pub fn claims_surplus(s: &FloorPool, claims0: u64) -> u64 {
    let liabilities = floor_math::FloorState::quote_liabilities(bytemuck::cast_ref(s));
    let claims0 = claims0 as u128;
    if claims0 <= liabilities {
        return 0;
    }
    (claims0 - liabilities) as u64
}

///
///
///
///
pub fn book_claims_surplus(m: &mut Mech, s: &mut FloorPool) -> Result<()> {
    let surplus = claims_surplus(s, m.market.hook_claims0);
    if surplus == 0 {
        return Ok(());
    }
    let kept = intake(s, surplus, &mut m.events)?;
    s.vault_eth = add(s.vault_eth, kept)?;
    Ok(())
}

///
///
///
///
///
///
///
///
///
///
///
///
pub fn keeper_tip(s: &FloorPool, amount: u64) -> u64 {
    let rate = ((amount as u128) * (s.bounty_bps as u128) / (BPS as u128)) as u64;
    rate.min(s.max_bounty).min(amount)
}

///
///
///
///
///
///
///
pub fn team_tip(s: &FloorPool, owed: u64, last_tip_at: i64, now: i64) -> u64 {
    if last_tip_at != 0 && now.saturating_sub(last_tip_at) < TEAM_TIP_INTERVAL_SECS {
        return 0;
    }
    let rate = ((owed as u128) * (s.bounty_bps as u128) / (BPS as u128)) as u64;
    rate.min(s.max_bounty / TEAM_TIP_CAP_DIVISOR).min(owed)
}

pub fn bounty_for(s: &FloorPool) -> u64 {
    let rate = ((s.unpaid_skim as u128) * (s.bounty_bps as u128) / (BPS as u128)) as u64;
    rate.min(s.max_bounty).min(s.vault_eth)
}

///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
pub fn run(m: &mut Mech, s: &mut FloorPool, supply: u64) -> Result<(i32, u64)> {








    require!(s.quote_is_token1 == 0, HookError::InvariantViolated);
    s.needs_ratchet = 0;
    let floor0 = s.floor_tick;

    mark("cu run.start");
    sweep_seed_fees(m, s)?;
    mark("cu run.sweep_seed");





    require!(s.stair_steps != 0, HookError::InvariantViolated);
    teardown_bid(m, s)?;
    mark("cu run.teardown_bid");


    let out = stair_positions(m, s, supply)?;
    mark("cu run.stair_half");
    let cushion = out.cushion;
    book_claims_surplus(m, s)?;
    mark("cu run.claims_surplus");







    let mut bounty = bounty_for(s);
    s.unpaid_skim = 0;
    if bounty > 0 {
        s.vault_eth = sub(s.vault_eth, bounty)?;

        if !m.pay_bounty(s, bounty)? {
            s.vault_eth = add(s.vault_eth, bounty)?;
            bounty = 0;
        }
    }
    let reserve = s.vault_eth.min(s.min_bid);
    mark("cu run.bounty");







    let stand0 = math::ceil_to_spacing(m.market.tick + 1 + math::band_ticks(s)?, s.tick_spacing)?;
    let hold = if out.parked {
        out.hold
    } else {
        match crate::mech::netflow::climb_line(m, s, supply, reserve, cushion, out.cushion_ticks, stand0)? {
            Some(c) => Some(out.hold.map_or(c, |h| h.max(c))),
            None => out.hold,
        }
    };
    mark("cu run.climb_hold");









    let lift_pub = math::ceil_to_spacing(m.market.tick + 1 + math::band_ticks(s)?, s.tick_spacing)?;


    let lift_pub = match hold {
        Some(h) if h > lift_pub => h,
        _ => lift_pub,
    };







    let mut pend = crate::mech::positions::PendingRaise::default();







    let (cut, lift_settled) = {
        let avail = s.vault_eth.saturating_sub(reserve);
        let old = avail.saturating_sub(out.inflow) as u128;
        let more: Vec<floor_math::MorePosition> = cushion.into_iter().collect();
        let need0 = floor_math::stair::backing_need(math::state(s), supply, floor0, &more)
            .map_err(|_| error!(HookError::MathOverflow))?;
        let cut = if old > need0 { (old - need0).min(u64::MAX as u128) as u64 } else { 0 };
        let settled = crate::mech::stair::settled_tick(m)?.max(m.market.tick);
        let line = math::ceil_to_spacing(settled + 1 + math::band_ticks(s)?, s.tick_spacing)?;
        let line = match hold {
            Some(h) if h > line => h,
            _ => line,
        };
        (cut, line)
    };
    passes(m, s, supply, cushion, reserve, cut, lift_pub, &mut pend)?;
    if cut > 0 && lift_settled < s.floor_tick {

        passes(m, s, supply, cushion, reserve, 0, lift_settled, &mut pend)?;
    }


    raise_pol(m, s, &mut pend)?;
    mark("cu run.raise_pol");




    mark("cu run.passes_end");
    nudge(m, s, reserve)?;
    mark("cu run.nudge");



    {
        let stand = math::ceil_to_spacing(m.market.tick + 1 + math::band_ticks(s)?, s.tick_spacing)?;
        if s.floor_tick < stand {
            s.floor_tick = if stand < floor0 { stand } else { floor0 };
        }
    }




    trim_cushion(m, s, floor0)?;


    let held = cut > 0 && lift_pub < s.floor_tick && s.floor_tick <= lift_settled;
    mark_older_surplus(m, held)?;

    mark_cushion_top(m, s)?;
    mark("cu run.trim");





    let at = s.ladder_upper;
    deploy_bid(m, s)?;
    mark("cu run.deploy_bid");
    Ok((at, bounty))
}

///
#[inline(never)]
fn mark_older_surplus(m: &Mech, held: bool) -> Result<()> {
    let mut data = m.cpi.floor_pool.try_borrow_mut_data()?;
    if let Some(mut o) = crate::state::twap::TwapOracle::from_account(&data) {
        if o.older_surplus_held != held as u8 {
            o.older_surplus_held = held as u8;
            o.store(&mut data);
        }
    }
    Ok(())
}

///
///
///
///
///
///
#[inline(never)]
fn mark_cushion_top(m: &Mech, s: &FloorPool) -> Result<()> {
    if !floor_math::fees::cushion_gap_skim_on() || s.stair_ready == 0 {
        return Ok(());
    }
    let (_, st) = crate::mech::stair::load(&m.accts.tick_arrays, m.cpi.floor_pool.key)?;
    let (set, top) = if st.cushion_liq.get() != 0 { (1u8, st.cushion_lower) } else { (0, 0) };
    let mut data = m.cpi.floor_pool.try_borrow_mut_data()?;
    if let Some(mut o) = crate::state::twap::TwapOracle::from_account(&data) {
        if (o.cushion_top_set, o.cushion_top_tick) != (set, top) {
            o.cushion_top_set = set;
            o.cushion_top_tick = top;
            o.store(&mut data);
        }
    }
    Ok(())
}

///
///
///
///
pub fn older_surplus_due(floor_pool: &AccountInfo, s: &FloorPool, tick: i32) -> Result<bool> {
    if s.stair_steps == 0 {
        return Ok(false);
    }
    let data = floor_pool.try_borrow_data()?;
    let Some(o) = crate::state::twap::TwapOracle::from_account(&data) else { return Ok(false) };
    if o.older_surplus_held == 0 {
        return Ok(false);
    }
    let now = crate::state::twap::now_secs()?;
    let settled = o
        .settled_tick(now, tick, floor_types::twap::CUSHION_TWAP_WINDOW_SECS)
        .map_err(|_| error!(HookError::TwapUnavailable))?
        .max(tick);
    let line = math::ceil_to_spacing(settled + 1 + math::band_ticks(s)?, s.tick_spacing)?;
    Ok(line < s.floor_tick)
}

///
///
///
#[allow(clippy::too_many_arguments)]
fn passes(
    m: &mut Mech,
    s: &mut FloorPool,
    supply: u64,
    cushion: Option<(i32, i32, u128)>,
    reserve: u64,
    cut: u64,
    cap: i32,
    pend: &mut crate::mech::positions::PendingRaise,
) -> Result<()> {
    let start = s.floor_tick;
    for i in 0..2 {
        let q = s.vault_eth.saturating_sub(reserve).saturating_sub(cut);
        if q == 0 || circulating(s, supply)? == 0 {
            break;
        }
        mark("cu run.pass_head");


        let hi = s.floor_tick;
        let lo = math::floor_to_spacing(cap.min(s.floor_tick) - math::band_ticks(s)? - 2 * s.tick_spacing, s.tick_spacing)?;
        let solved = if lo < hi && hi % s.tick_spacing == 0 {
            math::solve_floor_tick_within(q, s, supply, cushion, lo, hi)?
        } else {
            math::solve_floor_tick_with_cushion(q, s, supply, cushion)?
        };
        mark("cu run.solve");
        let candidate = math::publish(s, solved)?;
        mark("cu run.publish");
        if candidate < s.floor_tick {
            s.floor_tick = candidate;
        } else if i != 0 {
            break;
        }
        if s.floor_tick < cap {
            s.floor_tick = if cap < start { cap } else { start };
        }
        let raised = preview_raise(m, s, pend)?;
        mark("cu run.preview_raise");
        if !raised {
            break;
        }
    }
    Ok(())
}

///
///
#[inline(never)]
fn stair_positions(m: &mut Mech, s: &mut FloorPool, supply: u64) -> Result<StairOutcome> {
    crate::mech::stair::ratchet_positions(m, s, supply)
}

///
#[inline(never)]
fn trim_cushion(m: &mut Mech, s: &mut FloorPool, floor0: i32) -> Result<()> {
    crate::mech::stair::trim_cushion(m, s, floor0)
}

pub fn claims_cover_liabilities(s: &FloorPool, claims0: u64, claims1: u64) -> bool {





    let quote_owed = floor_math::FloorState::quote_liabilities(bytemuck::cast_ref(s));


    let base_owed = floor_math::FloorState::base_liabilities(bytemuck::cast_ref(s));
    (claims0 as u128) >= quote_owed && (claims1 as u128) >= base_owed
}

///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub floor_tick: i32,
    pub seed_liq: u128,
    pub seed_upper: i32,
}

impl Entry {
    pub fn of(s: &FloorPool) -> Self {
        Self { floor_tick: s.floor_tick, seed_liq: s.seed_liq.get(), seed_upper: s.seed_upper }
    }
}

///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
pub fn epilogue(s: &FloorPool, entry: &Entry, claims0: u64, claims1: u64) -> Result<()> {
    require!(s.floor_tick <= entry.floor_tick, HookError::InvariantViolated);
    require!(claims_cover_liabilities(s, claims0, claims1), HookError::InvariantViolated);
    if entry.seed_liq > 0 {
        require!(s.seed_liq.get() > 0, HookError::InvariantViolated);
    }
    require!(s.seed_upper <= entry.seed_upper, HookError::InvariantViolated);


    if s.bid_liquidity.get() > 0 {
        let width = math::bid_width(s);
        require!(s.bid_lower + width >= s.floor_tick, HookError::InvariantViolated);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
//!
//!
//!
    use super::*;
    use bytemuck::Zeroable;

    fn at(floor_tick: i32, seed_liq: u128, seed_upper: i32) -> Entry {
        Entry { floor_tick, seed_liq, seed_upper }
    }

    fn pool() -> FloorPool {
        let mut s = FloorPool::zeroed();
        s.tick_spacing = 60;
        s.floor_tick = 82_440;
        s.min_bid = 50_000_000;
        s
    }

///
///
///
///
    #[test]
    fn the_team_tip_is_capped_at_a_twentieth_and_paid_once_a_day() {
        let mut s = pool();
        s.bounty_bps = 100;
        s.max_bounty = 50_000_000;
        const T: i64 = 1_790_000_000;

        assert_eq!(team_tip(&s, 10_000_000_000, 0, T), 2_500_000, "max_bounty / 20");
        assert_eq!(s.max_bounty / TEAM_TIP_CAP_DIVISOR, 2_500_000);
        assert!(team_tip(&s, u64::MAX, 0, T) <= s.max_bounty / 20);
        assert_eq!(keeper_tip(&s, 10_000_000_000), 50_000_000, "the fee sweep's tip keeps the ratchet's cap");

        assert_eq!(team_tip(&s, 1_000_000, 0, T), 10_000);
        assert_eq!(team_tip(&s, 99, 0, T), 0);
        s.bounty_bps = 10_000;
        assert_eq!(team_tip(&s, 7, 0, T), 7);
        s.bounty_bps = 100;

        assert_eq!(team_tip(&s, 10_000_000_000, T, T), 0, "the same second");
        assert_eq!(team_tip(&s, 10_000_000_000, T, T + TEAM_TIP_INTERVAL_SECS - 1), 0, "one second short of a day");
        assert_eq!(team_tip(&s, 10_000_000_000, T, T + TEAM_TIP_INTERVAL_SECS), 2_500_000, "a day after");
        assert_eq!(TEAM_TIP_INTERVAL_SECS, 86_400);

        assert_eq!(team_tip(&s, 10_000_000_000, T, T - 5), 0);

        s.max_bounty = 0;
        assert_eq!(team_tip(&s, 10_000_000_000, 0, T), 0);

        s.max_bounty = 50_000_000;
        let (mut last, mut total) = (0i64, 0u64);
        for h in 0..(365 * 24) {
            let now = T + h * 3_600;
            let t = team_tip(&s, 10_000_000_000, last, now);
            if t > 0 {
                last = now;
                total += t;
            }
        }
        assert_eq!(total, 365 * 2_500_000, "hourly sweeps for a year tip once a day");
    }

    #[test]
    fn i6_rejects_a_bid_whose_band_ends_under_the_floor() {
        let mut s = pool();
        s.bid_liquidity.set(1);
        s.bid_lower = s.floor_tick - 120;
        s.bid_width = 120;
        assert!(epilogue(&s, &at(s.floor_tick, 0, 0), 0, 0).is_ok(), "the band form: lower + width == floor");
        s.bid_width = 60;
        assert!(epilogue(&s, &at(s.floor_tick, 0, 0), 0, 0).is_err(), "a band ending one spacing under the floor");
        s.bid_lower = s.floor_tick;
        s.bid_width = 60;
        assert!(epilogue(&s, &at(s.floor_tick, 0, 0), 0, 0).is_ok(), "the last-resort form: one spacing above a market past the band");
        s.bid_lower = s.floor_tick - 60;
        s.bid_width = 0;
        assert!(epilogue(&s, &at(s.floor_tick, 0, 0), 0, 0).is_ok(), "a pre-band bid reads its width as the spacing (E:522-525)");
        s.bid_lower = s.floor_tick - 120;
        assert!(epilogue(&s, &at(s.floor_tick, 0, 0), 0, 0).is_err(), "and one spacing lower is under the floor");
    }

    #[test]
    fn i4_rejects_a_ratchet_that_lowers_the_pols_edge_price_or_empties_it() {
        let mut s = pool();
        s.seed_upper = 79_920;
        s.seed_liq.set(7);
        assert!(epilogue(&s, &at(s.floor_tick, 7, 80_040), 0, 0).is_ok(), "seed_upper moved up in price (down in tick)");
        assert!(epilogue(&s, &at(s.floor_tick, 7, 79_920), 0, 0).is_ok(), "or stayed");
        assert!(epilogue(&s, &at(s.floor_tick, 7, 79_860), 0, 0).is_err(), "never down in price (up in tick)");
        s.seed_liq.set(0);
        assert!(epilogue(&s, &at(s.floor_tick, 7, 80_040), 0, 0).is_err(), "and never emptied once seeded");
        assert!(epilogue(&s, &at(s.floor_tick, 0, 80_040), 0, 0).is_ok(), "an unseeded pool stays unseeded");
    }

///
///
///
///
///
///
    #[test]
    fn the_sweep_tip_is_the_three_way_min_and_an_unset_cap_pays_nothing() {
        let mut s = pool();
        s.bounty_bps = 100;

        s.max_bounty = 1;
        assert_eq!(keeper_tip(&s, 10_000_000_000), 1, "a 1-unit cap pays exactly 1");

        s.max_bounty = 50_000_000;
        assert_eq!(keeper_tip(&s, 1_000_000_000), 10_000_000, "1 % of 1 SOL");
        assert_eq!(keeper_tip(&s, 10_000_000_000), 50_000_000, "…and the cap above it");

        assert_eq!(keeper_tip(&s, 99), 0, "dust pays nothing");
        assert_eq!(keeper_tip(&s, 100), 1, "the first whole unit");
        assert_eq!(keeper_tip(&s, 0), 0, "a no-op sweep pays nothing");

        s.bounty_bps = BPS as u16;
        assert_eq!(keeper_tip(&s, 7), 7, "never above the amount");

        s.max_bounty = 0;
        assert_eq!(keeper_tip(&s, u64::MAX), 0, "cap 0: the sweep pays nothing");
        s.bounty_bps = 100;
        s.vault_eth = 100_000_000_000;
        s.unpaid_skim = 10_000_000_000;
        assert_eq!(bounty_for(&s), 0, "cap 0: the ratchet pays no bounty (and `run` still runs: the bounty is not its guard)");
    }

///
///
///
    #[test]
    fn the_bounty_is_the_three_way_min_and_rounds_down() {
        let mut s = pool();
        s.bounty_bps = 100;
        s.max_bounty = 50_000_000;
        s.vault_eth = 100_000_000_000;


        s.unpaid_skim = 1_000_000_000;
        assert_eq!(bounty_for(&s), 10_000_000);


        s.unpaid_skim = 10_000_000_000;
        assert_eq!(bounty_for(&s), 50_000_000, "clamped at max_bounty");


        s.vault_eth = 7;
        assert_eq!(bounty_for(&s), 7, "clamped at vault_eth");


        s.vault_eth = 100_000_000_000;
        s.unpaid_skim = 10_001;
        s.bounty_bps = 99;
        assert_eq!(bounty_for(&s), 99, "floor((10_001 * 99) / 10_000)");


        s.bounty_bps = 0;
        s.unpaid_skim = u64::MAX;
        assert_eq!(bounty_for(&s), 0, "rate 0 pays nothing at any skim");


        s.bounty_bps = floor_types::constants::MAX_BOUNTY_BPS;
        s.max_bounty = floor_types::constants::MAX_BOUNTY_LAMPORTS_CEIL;
        assert_eq!(bounty_for(&s), floor_types::constants::MAX_BOUNTY_LAMPORTS_CEIL);




        s.unpaid_skim = 0;
        assert_eq!(bounty_for(&s), 0);
    }

    #[test]
    fn the_no_op_guard_has_no_gap_clause_and_the_band_helpers_measure_to_the_dear_edge() {
        let mut s = pool();



        assert!(is_noop(&s, s.floor_tick + 10_000).unwrap());

        s.bid_liquidity.set(1);
        s.bid_lower = s.floor_tick - 60;
        assert!(!is_noop(&s, s.floor_tick - 1_000).unwrap());
        s.bid_lower = s.floor_tick - 120;
        assert!(is_noop(&s, s.floor_tick - 1_000).unwrap(), "already over the whole band");

        assert_eq!(math::band_ticks(&s).unwrap(), 120);
        assert_eq!(math::dear_edge(&s).unwrap(), s.floor_tick - 120);
        assert_eq!(math::publish(&s, s.floor_tick - 120).unwrap(), s.floor_tick, "publish(dear edge) is the cheap edge");
        assert_eq!(math::publish(&s, 82_301).unwrap(), 82_440, "ceil'd to the grid");
        s.tick_spacing = 100;
        assert_eq!(math::band_ticks(&s).unwrap(), 200, "BID_BAND_TICKS rounds UP to the spacing");
        assert_eq!(math::bid_width(&s), 100, "a pre-band bid reads the spacing");
        s.bid_width = 200;
        assert_eq!(math::bid_width(&s), 200);
    }

///
///
///
///
///
///
///
///
///
    #[test]
    fn a_referred_pools_accrual_is_a_liability_not_a_donation() {
        let mut s = pool();
        s.vault_eth = 1;
        s.creator_fees_owed = 2;
        s.platform_owed = 4;
        s.holder_owed = 8;
        s.referral_owed = 16;
        let owed = floor_math::FloorState::quote_liabilities(bytemuck::cast_ref(&s));
        assert_eq!(owed, 31, "the one spelling counts all five");


        assert_eq!(claims_surplus(&s, 31), 0, "referral_owed is owed, not donated");

        assert_eq!(claims_surplus(&s, 30), 0);
        assert_eq!(claims_surplus(&s, 0), 0);

        assert_eq!(claims_surplus(&s, 31 + 5), 5);




        assert!(claims_cover_liabilities(&s, 31, 0));
        assert!(!claims_cover_liabilities(&s, 30, 0));
    }

///
///
///
///
///
///
    #[test]
    fn the_ratchet_counts_the_held_wall_tokens() {
        let mut s = pool();
        s.stair_steps = 3;
        s.inventory_tokens = 10;
        s.ladder_tokens_remaining = 450;
        s.wall_held_tokens = 150;
        assert_eq!(circulating(&s, 1_000).unwrap(), 390, "C = supply − inventory − listed − held");
        s.wall_held_tokens = 541;
        assert!(circulating(&s, 1_000).is_err(), "an underflow is refused, never saturated");
        s.wall_held_tokens = 150;
        assert!(claims_cover_liabilities(&s, 0, 160));
        assert!(!claims_cover_liabilities(&s, 0, 159), "held is owed");
        s.stair_steps = 0;
        assert!(!claims_cover_liabilities(&s, 0, 609), "a non-staircase state: inventory + listed + held");
        assert!(claims_cover_liabilities(&s, 0, 610));




        s.stair_steps = 3;
        s.ask_tokens_deployed = 100;
        assert_eq!(circulating(&s, 1_000).unwrap(), 390, "the team's held tokens count against the floor");
        assert!(!claims_cover_liabilities(&s, 0, 259), "the team's held tokens are owed in claims1");
        assert!(claims_cover_liabilities(&s, 0, 260));
    }
}
