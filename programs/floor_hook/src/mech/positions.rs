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

use anchor_lang::prelude::*;
use floor_types::constants::{
    MAX_STAIR_STEPS, MAX_TICK, SALT_BID, SALT_CUSHION, SALT_POL_RELIST, SALT_SEED, SALT_STAIR_STEP0, SALT_STAIR_RESERVE,
    SALT_STAIR_TAIL,
};
use floor_types::hook::HookOpResultV1;

use crate::errors::HookError;
use crate::events::{PlatformCut, PolRaised};
use crate::mech::amm_cpi::{self, AmmCpi, PoolAccounts};
use crate::state::FloorPool;





///
///
///
pub mod math {
    use crate::errors::HookError;
    use crate::state::FloorPool;
    use anchor_lang::prelude::*;
    use floor_math::FloorState;

    const _: () = assert!(core::mem::size_of::<FloorPool>() == floor_math::FLOOR_STATE_SIZE);

    fn map<T>(r: floor_math::FloorMathResult<T>) -> Result<T> {
        r.map_err(|e| {
            msg!("floor-math: {}", e);
            error!(HookError::MathOverflow)
        })
    }

///
    pub fn state(s: &FloorPool) -> &FloorState {
        bytemuck::cast_ref(s)
    }

///
    pub fn state_mut(s: &mut FloorPool) -> &mut FloorState {
        bytemuck::cast_mut(s)
    }

///
    pub fn ceil_to_spacing(tick: i32, spacing: i32) -> Result<i32> {
        map(floor_math::ceil_to_spacing(tick, spacing))
    }

///
    pub fn floor_to_spacing(tick: i32, spacing: i32) -> Result<i32> {
        map(floor_math::floor_to_spacing(tick, spacing))
    }

///
///
    pub fn position_liquidity(sqrt_price_x64: u128, lower: i32, upper: i32, quote: u64, base: u64) -> Result<u128> {
        map(floor_math::position_liquidity(sqrt_price_x64, lower, upper, quote, base))
    }

///
///
    pub fn position_amounts(sqrt_price_x64: u128, lower: i32, upper: i32, liquidity: u128) -> Result<(u64, u64)> {
        map(floor_math::position_amounts(sqrt_price_x64, lower, upper, liquidity))
    }

///
///
///
    pub fn position_amounts_up(sqrt_price_x64: u128, tick: i32, lower: i32, upper: i32, liquidity: u128) -> Result<(u64, u64)> {
        use carpenter_clmm_math::liquidity_math::{get_delta_amount_0_unsigned as d0, get_delta_amount_1_unsigned as d1};
        if liquidity == 0 {
            return Ok((0, 0));
        }
        let e = |_| error!(HookError::MathOverflow);
        let sa = sqrt_price_at_tick(lower)?;
        let sb = sqrt_price_at_tick(upper)?;
        Ok(if tick < lower {
            (d0(sa, sb, liquidity, true).map_err(e)?, 0)
        } else if tick < upper {
            (d0(sqrt_price_x64, sb, liquidity, true).map_err(e)?, d1(sa, sqrt_price_x64, liquidity, true).map_err(e)?)
        } else {
            (0, d1(sa, sb, liquidity, true).map_err(e)?)
        })
    }






///
    pub fn band_ticks(s: &FloorPool) -> Result<i32> {
        ceil_to_spacing(crate::constants::BID_BAND_TICKS, s.tick_spacing)
    }

///
///
///
///
    pub fn dear_edge(s: &FloorPool) -> Result<i32> {
        Ok(s.floor_tick - band_ticks(s)?)
    }

///
///
    pub fn publish(s: &FloorPool, solved: i32) -> Result<i32> {
        ceil_to_spacing(solved + band_ticks(s)?, s.tick_spacing)
    }

///
///
    pub fn bid_width(s: &FloorPool) -> i32 {
        if s.bid_width == 0 {
            s.tick_spacing
        } else {
            s.bid_width
        }
    }

///
///
    pub fn solve_floor_tick_with(q: u64, extra: u64, s: &FloorPool, supply: u64) -> Result<i32> {
        map(floor_math::solve_floor_tick_with(q as u128, extra, state(s), supply))
    }

///
///
    pub fn solve_floor_tick_with_cushion(q: u64, s: &FloorPool, supply: u64, cushion: Option<(i32, i32, u128)>) -> Result<i32> {
        match cushion {
            None => solve_floor_tick_with(q, 0, s, supply),
            Some(c) => map(floor_math::solve_floor_tick_with_more(q as u128, 0, state(s), supply, &[c])),
        }
    }

///
///
///
    pub fn solve_floor_tick_within(q: u64, s: &FloorPool, supply: u64, cushion: Option<(i32, i32, u128)>, lo: i32, hi: i32) -> Result<i32> {
        match cushion {
            None => map(floor_math::solve_floor_tick_within(q as u128, state(s), supply, &[], lo, hi)),
            Some(c) => map(floor_math::solve_floor_tick_within(q as u128, state(s), supply, &[c], lo, hi)),
        }
    }

///
///
    pub fn intake(s: &mut FloorPool, x: u64) -> Result<u64> {
        map(floor_math::intake(state_mut(s), x))
    }

///
    pub fn sqrt_price_at_tick(tick: i32) -> Result<u128> {
        carpenter_clmm_math::tick_math::get_sqrt_price_at_tick(tick).map_err(|_| error!(HookError::MathOverflow))
    }
}





///
///
pub enum MechEvent {
    StairRatcheted(crate::events::StairRatcheted),
    StairOwnerBooked(crate::events::StairOwnerBooked),
    StairOwnersPlaced(crate::events::StairOwnersPlaced),
    VaultTokensFed(crate::events::VaultTokensFed),
    PlatformCut(PlatformCut),
    PolRaised(PolRaised),
}

///
///
///
pub fn to_wall(s: &mut FloorPool, tokens: u64) -> Result<()> {
    s.wall_held_tokens = add(s.wall_held_tokens, tokens)?;
    Ok(())
}

///
///
///
///
pub fn take_held(s: &mut FloorPool) -> Result<u64> {
    require!(s.stair_steps != 0, HookError::InvariantViolated);
    let held = s.wall_held_tokens;
    s.wall_held_tokens = 0;
    Ok(held)
}

///
///
///
///
///
///
///
///
pub fn vault_bought(s: &mut FloorPool, tokens: u64, cost: u64) -> Result<()> {

    require!(s.stair_steps != 0, HookError::InvariantViolated);
    if tokens == 0 {
        return Ok(());
    }
    s.inventory_tokens = add(s.inventory_tokens, tokens)?;
    s.inventory_cost_eth = add(s.inventory_cost_eth, cost)?;
    Ok(())
}

pub fn add(a: u64, b: u64) -> Result<u64> {
    a.checked_add(b).ok_or_else(|| error!(HookError::MathOverflow))
}

pub fn sub(a: u64, b: u64) -> Result<u64> {
    a.checked_sub(b).ok_or_else(|| error!(HookError::MathOverflow))
}

///
///
///
///
///
pub fn intake(s: &mut FloorPool, amount: u64, events: &mut Vec<MechEvent>) -> Result<u64> {
    if amount == 0 {
        return Ok(0);
    }
    let kept = math::intake(s, amount)?;
    let cut = sub(amount, kept)?;
    if cut > 0 {
        events.push(MechEvent::PlatformCut(PlatformCut { pool: s.amm_pool, amount: cut }));
    }
    Ok(kept)
}








///
///
///
///
///
///
///
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Market {
    pub sqrt_price_x64: u128,
    pub tick: i32,
    pub hook_claims0: u64,
    pub hook_claims1: u64,
}

impl Market {
///
    pub fn from_pool(p: &floor_types::layout::Pool) -> Self {
        Self {
            sqrt_price_x64: p.sqrt_price_x64.get(),
            tick: p.tick_current,
            hook_claims0: p.hook_claims0,
            hook_claims1: p.hook_claims1,
        }
    }

///
    pub fn update(&mut self, r: &HookOpResultV1) {
        self.sqrt_price_x64 = r.post.sqrt_price_x64;
        self.tick = r.post.tick;
        self.hook_claims0 = r.hook_claims0;
        self.hook_claims1 = r.hook_claims1;
    }
}

///
pub struct Mech<'info> {
    pub cpi: AmmCpi<'info>,
    pub accts: PoolAccounts<'info>,
    pub position_seed: Option<AccountInfo<'info>>,
    pub position_bid: Option<AccountInfo<'info>>,


///
///
///
    pub bounty_destination: Option<AccountInfo<'info>>,
///
    pub spacing: u16,
    pub market: Market,
    pub events: Vec<MechEvent>,
///
///
    pub modifies: u8,
}

impl<'info> Mech<'info> {
///
    pub fn flush(&self, s: &FloorPool) -> Result<()> {
        let n = core::mem::size_of::<FloorPool>();
        let mut data = self.cpi.floor_pool.try_borrow_mut_data()?;
        require!(data.len() >= 8 + n, HookError::InvariantViolated);
        data[8..8 + n].copy_from_slice(bytemuck::bytes_of(s));
        Ok(())
    }

    fn position(&self, salt: u64) -> Result<AccountInfo<'info>> {
        let p = match salt {
            SALT_SEED => &self.position_seed,
            SALT_BID => &self.position_bid,

            SALT_STAIR_TAIL | SALT_CUSHION | SALT_POL_RELIST | SALT_STAIR_RESERVE => return self.remaining_position(salt),
            x if (SALT_STAIR_STEP0..SALT_STAIR_STEP0 + MAX_STAIR_STEPS as u64).contains(&x) => {
                return self.remaining_position(salt)
            }
            _ => return err!(HookError::InvariantViolated),
        };
        p.clone().ok_or_else(|| error!(anchor_lang::error::ErrorCode::AccountNotEnoughKeys))
    }

///
///
///
///
///
///
///
///
    #[inline(never)]
    pub(crate) fn remaining_position(&self, salt: u64) -> Result<AccountInfo<'info>> {


        let (pool, id) = (self.cpi.pool.key, self.cpi.hook_authority.key);
        if let Some(a) = self.accts.tick_arrays.iter().find(|a| amm_cpi::created_position_is(a, pool, id, salt)) {
            return Ok(a.clone());
        }
        let key = amm_cpi::position_key(self.cpi.pool.key, self.cpi.hook_authority.key, salt);
        self.accts
            .tick_arrays
            .iter()
            .find(|a| *a.key == key)
            .cloned()
            .ok_or_else(|| error!(anchor_lang::error::ErrorCode::AccountNotEnoughKeys))
    }

///
///
    fn after(&mut self, r: HookOpResultV1) -> Result<HookOpResultV1> {
        let fresh = amm_cpi::read_pool(&self.cpi.pool)?;
        require!(
            fresh.sqrt_price_x64.get() == r.post.sqrt_price_x64 && fresh.tick_current == r.post.tick,
            HookError::InvariantViolated
        );
        self.market.update(&r);
        Ok(r)
    }

    pub fn open(&mut self, s: &FloorPool, salt: u64, lower: i32, upper: i32, liquidity: u128) -> Result<HookOpResultV1> {
        self.modifies = self.modifies.saturating_add(1);
        self.flush(s)?;
        let pos = self.position(salt)?;
        let r = amm_cpi::open(&self.cpi, &self.accts, &pos, self.spacing, salt, lower, upper, liquidity)?;
        self.after(r)
    }

    pub fn close(&mut self, s: &FloorPool, salt: u64, lower: i32, upper: i32, liquidity: u128) -> Result<HookOpResultV1> {
        self.modifies = self.modifies.saturating_add(1);
        self.flush(s)?;
        let pos = self.position(salt)?;
        let r = amm_cpi::close(&self.cpi, &self.accts, &pos, self.spacing, salt, lower, upper, liquidity)?;
        self.after(r)
    }

    pub fn poke(&mut self, s: &FloorPool, salt: u64, lower: i32, upper: i32) -> Result<HookOpResultV1> {
        self.modifies = self.modifies.saturating_add(1);
        self.flush(s)?;
        let pos = self.position(salt)?;
        let r = amm_cpi::poke(&self.cpi, &self.accts, &pos, self.spacing, salt, lower, upper)?;
        self.after(r)
    }

///
///
///
    fn modify_signed(&mut self, s: &FloorPool, salt: u64, lower: i32, upper: i32, delta: i128) -> Result<HookOpResultV1> {
        self.modifies = self.modifies.saturating_add(1);
        self.flush(s)?;
        let pos = self.position(salt)?;
        let r = amm_cpi::modify(&self.cpi, &self.accts, &pos, self.spacing, salt, lower, upper, delta)?;
        self.after(r)
    }

    pub fn swap_exact_in(&mut self, s: &FloorPool, zero_for_one: bool, amount_in: u64, sqrt_limit: u128) -> Result<HookOpResultV1> {
        self.flush(s)?;
        let r = amm_cpi::swap_exact_in(&self.cpi, &self.accts, zero_for_one, amount_in, sqrt_limit)?;
        self.after(r)
    }

///
///
///
///
///
///
///
///
    pub fn pay_bounty(&mut self, s: &FloorPool, amount: u64) -> Result<bool> {
        let Some(dest) = self.bounty_destination.clone() else {
            return Ok(false);
        };
        self.flush(s)?;
        let r = amm_cpi::withdraw(
            &self.cpi,
            &self.accts.vault0,
            &dest,
            &self.accts.mint0,
            &self.accts.token_program0,
            0,
            amount,
        )?;
        self.after(r)?;
        Ok(true)
    }

    fn s_i32(&self) -> i32 {
        self.spacing as i32
    }
}





///
///
pub fn sweep_seed_fees(m: &mut Mech, s: &mut FloorPool) -> Result<()> {
    if s.seed_liq.get() == 0 {
        return Ok(());
    }
    let r = m.poke(s, SALT_SEED, s.seed_lower, s.seed_upper)?;
    let (eth_fees, tok_fees) = amm_cpi::back(&r)?;
    if eth_fees == 0 && tok_fees == 0 {
        return Ok(());
    }
    let eth = intake(s, eth_fees, &mut m.events)?;
    s.vault_eth = add(s.vault_eth, eth)?;
    to_wall(s, tok_fees)?;
    s.seed_fees_swept_eth = add(s.seed_fees_swept_eth, eth)?;
    s.seed_fees_swept_tokens = add(s.seed_fees_swept_tokens, tok_fees)?;
    Ok(())
}







///
///
///
pub fn teardown_bid(m: &mut Mech, s: &mut FloorPool) -> Result<()> {
    let liq = s.bid_liquidity.get();
    if liq == 0 {
        return Ok(());
    }
    let upper = s.bid_lower + math::bid_width(s);
    let r = m.close(s, SALT_BID, s.bid_lower, upper, liq)?;
    let (quote_back, tok_back) = amm_cpi::back(&r)?;
    let fees0 = r.fees0.min(quote_back);
    let principal0 = quote_back - fees0;
    let fee_kept = intake(s, fees0, &mut m.events)?;
    s.vault_eth = add(add(s.vault_eth, principal0)?, fee_kept)?;


    let cost = s.bid_eth_deployed.saturating_sub(principal0);
    vault_bought(s, tok_back, cost)?;
    s.bid_liquidity.set(0);
    s.bid_eth_deployed = 0;
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
pub fn deploy_bid(m: &mut Mech, s: &mut FloorPool) -> Result<()> {
    if s.vault_eth < s.min_bid {
        return Ok(());
    }
    let (p, current) = (m.market.sqrt_price_x64, m.market.tick);
    let spacing = m.s_i32();
    let mut upper = s.floor_tick;
    let mut lower = upper - math::band_ticks(s)?;
    let min_lower = math::ceil_to_spacing(current + 1, spacing)?;
    if lower < min_lower {
        lower = min_lower;
    }
    if lower >= upper {
        upper = lower + spacing;
    }
    if upper > MAX_TICK {
        return Ok(());
    }
    let liq = math::position_liquidity(p, lower, upper, s.vault_eth, 0)?;
    if liq == 0 {
        return Ok(());
    }
    let r = m.open(s, SALT_BID, lower, upper, liq)?;
    let (owed, _) = amm_cpi::used(&r)?;
    s.vault_eth = sub(s.vault_eth, owed)?;
    s.bid_eth_deployed = owed;
    s.bid_liquidity.set(liq);
    s.bid_lower = lower;
    s.bid_width = upper - lower;
    Ok(())
}

















///
///
///
///
///
pub fn buy_up_to(m: &mut Mech, s: &mut FloorPool, budget: u64, limit_tick: i32) -> Result<(u64, u64)> {
    let limit = math::sqrt_price_at_tick(limit_tick)?;

    let r = m.swap_exact_in(s, s.quote_is_token1 == 0, budget, limit)?;
    let spent = amm_cpi::neg_to_u64(r.delta.amount0)?;
    let got = amm_cpi::pos_to_u64(r.delta.amount1)?;
    s.vault_eth = sub(s.vault_eth, spent)?;
    vault_bought(s, got, spent)?;
    Ok((spent, got))
}

///
///
///
///
///
///
///
pub fn nudge(m: &mut Mech, s: &mut FloorPool, budget: u64) -> Result<()> {
    let budget = budget.min(s.vault_eth);
    let edge = math::dear_edge(s)?;
    let limit = math::sqrt_price_at_tick(edge)?;
    if budget == 0 || m.market.sqrt_price_x64 <= limit {
        return Ok(());
    }
    buy_up_to(m, s, budget, edge)?;
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
#[derive(Clone, Copy, Default)]
pub struct PendingRaise {
    pub active: bool,
    pub from: i32,
    pub liq0: u128,
    pub vq: u64,
    pub vt: u64,
}

///
pub fn preview_raise(m: &Mech, s: &mut FloorPool, pend: &mut PendingRaise) -> Result<bool> {
    let edge = math::dear_edge(s)?;
    let from = s.seed_upper;
    let seed_liq = s.seed_liq.get();
    if seed_liq == 0 || edge >= from || edge <= s.seed_lower {
        return Ok(false);
    }
    let p = m.market.sqrt_price_x64;



    let (q0, b0) = math::position_amounts(p, s.seed_lower, from, seed_liq)?;
    let liq = pol_size_of(p, s.seed_lower, edge, q0, b0)?;
    if liq == 0 {
        return Ok(false);
    }

    let (q1, b1) = math::position_amounts_up(p, m.market.tick, s.seed_lower, edge, liq)?;
    let (quote_moved, leftover, _) = pol_net(q0, q1, b0, b1)?;
    if !pend.active {
        *pend = PendingRaise { active: true, from, liq0: seed_liq, vq: 0, vt: 0 };
    }
    s.seed_upper = edge;
    s.seed_liq.set(liq);

    let kept = if quote_moved == 0 || s.platform_bps == 0 {
        quote_moved
    } else {
        quote_moved - floor_math::fees::part(quote_moved, s.platform_bps as u64).map_err(|_| error!(HookError::MathOverflow))?
    };
    s.vault_eth = add(s.vault_eth, kept)?;
    s.wall_held_tokens = add(s.wall_held_tokens, leftover)?;
    pend.vq = add(pend.vq, kept)?;
    pend.vt = add(pend.vt, leftover)?;
    Ok(true)
}

///
///
pub fn raise_pol(m: &mut Mech, s: &mut FloorPool, pend: &mut PendingRaise) -> Result<()> {
    if !pend.active {
        return Ok(());
    }
    pend.active = false;
    s.vault_eth = sub(s.vault_eth, pend.vq)?;
    s.wall_held_tokens = sub(s.wall_held_tokens, pend.vt)?;
    let (from, edge, liq) = (pend.from, s.seed_upper, s.seed_liq.get());
    let delta = i128::try_from(pend.liq0).map_err(|_| error!(HookError::MathOverflow))?;
    let r0 = m.modify_signed(s, SALT_SEED, s.seed_lower, from, -delta)?;
    let (q0, b0) = amm_cpi::back(&r0)?;
    let r1 = m.open(s, SALT_SEED, s.seed_lower, edge, liq)?;
    let (q1, b1) = amm_cpi::used(&r1)?;
    let (quote_moved, leftover, tokens_readded) = pol_net(q0, q1, b0, b1)?;
    if quote_moved > 0 {
        let kept = intake(s, quote_moved, &mut m.events)?;
        s.vault_eth = add(s.vault_eth, kept)?;
    }
    to_wall(s, leftover)?;
    m.events.push(MechEvent::PolRaised(PolRaised {
        pool: s.amm_pool,
        from_edge: from,
        to_edge: edge,
        quote_moved,
        tokens_readded,
    }));
    Ok(())
}

///
///
///
///
///
///
pub fn pol_size(sqrt_price_x64: u128, seed_lower: i32, from: i32, edge: i32, seed_liq: u128) -> Result<u128> {
    let (q, b) = math::position_amounts(sqrt_price_x64, seed_lower, from, seed_liq)?;
    pol_size_of(sqrt_price_x64, seed_lower, edge, q, b)
}

///
pub fn pol_size_of(sqrt_price_x64: u128, seed_lower: i32, edge: i32, q: u64, b: u64) -> Result<u128> {
    math::position_liquidity(sqrt_price_x64, seed_lower, edge, q.saturating_sub(1), b.saturating_sub(1))
}

///
///
///
///
///
///
///
pub fn pol_net(q0: u64, q1: u64, b0: u64, b1: u64) -> Result<(u64, u64, u64)> {
    let quote_moved = q0.checked_sub(q1).ok_or_else(|| error!(HookError::InvariantViolated))?;
    let leftover = b0.checked_sub(b1).ok_or_else(|| error!(HookError::InvariantViolated))?;
    let tokens_readded = b0.checked_sub(leftover).ok_or_else(|| error!(HookError::InvariantViolated))?;
    Ok((quote_moved, leftover, tokens_readded))
}



#[cfg(test)]
mod tests {
//!
//!
//!
//!
//!
//!
//!
//!
//!
    use super::*;

///
///
    const LOWER: i32 = -443_580;
    const FROM: i32 = 80_040;
    const SEED_LIQ: u128 = 2_700_000_000_000;

    fn price_at(tick: i32) -> u128 {
        math::sqrt_price_at_tick(tick).unwrap()
    }

///
///
    #[test]
    fn pol_size_of_the_removals_amounts_is_pol_size() {
        for (tick, edge) in [(82_500, 79_920), (60_000, 79_920), (-440_000, 79_920), (80_000, 79_980), (0, 40_020)] {
            let p = price_at(tick);
            let Ok((q, b)) = math::position_amounts(p, LOWER, FROM, SEED_LIQ) else {
                assert!(pol_size(p, LOWER, FROM, edge, SEED_LIQ).is_err(), "at {tick}: the removal does not fit, nor does the size");
                continue;
            };

            assert_eq!(pol_size_of(p, LOWER, edge, q, b).ok(), pol_size(p, LOWER, FROM, edge, SEED_LIQ).ok(), "at {tick}");
        }
    }

    #[test]
    fn pol_size_never_asks_for_more_than_the_removal_returns() {


        let p = price_at(82_500);
        let edge = 79_920;
        let (q, b) = math::position_amounts(p, LOWER, FROM, SEED_LIQ).unwrap();
        assert_eq!(q, 0, "a tail entirely below the market is token-only");
        assert!(b > 0);

        let liq = pol_size(p, LOWER, FROM, edge, SEED_LIQ).unwrap();
        assert!(liq > 0, "the raise is possible");


        let (q1, b1) = math::position_amounts(p, LOWER, edge, liq).unwrap();
        assert!(q1 <= q, "quote asked {q1} > quote returned {q}");
        assert!(b1 <= b, "base asked {b1} > base returned {b}");

        let (quote_moved, leftover, readded) = pol_net(q, q1, b, b1).unwrap();
        assert_eq!(quote_moved, q - q1);
        assert_eq!(leftover, b - b1);
        assert_eq!(readded, b1, "`uint256(b0) − leftover` IS the re-open's debit");
    }

    #[test]
    fn pol_size_is_zero_where_the_evm_returns_false() {
        let p = price_at(82_500);

        assert_eq!(pol_size(p, LOWER, FROM, 79_920, 0).unwrap(), 0, "seed_liq == 0");


        let (q, b) = math::position_amounts(price_at(80_000), 79_800, 79_860, 1).unwrap();
        assert_eq!((q, b), (0, 0), "L = 1 over one spacing holds under one unit of either asset");
        assert_eq!(pol_size(price_at(80_000), 79_800, 79_860, 79_820, 1).unwrap(), 0);
    }

    #[test]
    fn pol_net_refuses_a_net_debit_on_either_leg() {

        assert_eq!(pol_net(1_000, 0, 5_000, 4_997).unwrap(), (1_000, 3, 4_997));


        assert!(pol_net(10, 11, 5, 5).is_err(), "the quote leg");
        assert!(pol_net(10, 10, 5, 6).is_err(), "the base leg");

        assert_eq!(pol_net(0, 0, 7, 7).unwrap(), (0, 0, 7));
    }
}
