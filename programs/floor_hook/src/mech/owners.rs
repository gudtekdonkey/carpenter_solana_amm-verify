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
use floor_types::constants::STAIR_OWNER_TEAM;
use floor_types::owners::{StairOwnerArg, StairOwners, STAIR_OWNERS_VERSION, STAIR_STATE_ACCOUNT_LEN_WITH_OWNERS};

use crate::errors::HookError;
use crate::mech::positions::math;
use crate::state::{FloorPool, StairState};

///
pub fn stair_account_len(stair_owner: u8) -> usize {
    if stair_owner != 0 {
        STAIR_STATE_ACCOUNT_LEN_WITH_OWNERS
    } else {
        StairState::LEN
    }
}

///
///
#[inline(never)]
pub fn read(acct: &AccountInfo, s: &FloorPool) -> Result<Option<Box<StairOwners>>> {
    if s.stair_owner == 0 {
        return Ok(None);
    }
    let data = acct.try_borrow_data()?;
    let o = StairOwners::from_account(&data).ok_or(error!(HookError::StairStateMissing))?;
    Ok(Some(Box::new(o)))
}

///
#[inline(never)]
pub fn write(acct: &AccountInfo, o: &StairOwners) -> Result<()> {
    let mut data = acct.try_borrow_mut_data()?;
    require!(o.store(&mut data), HookError::StairStateMissing);
    Ok(())
}

///
pub fn counted(o: Option<&StairOwners>) -> u64 {
    match o {
        Some(o) if o.counted != 0 => o.standing(),
        _ => 0,
    }
}

///
///
///
///
///
///
pub fn ladder_less_counted(standing: u64, counted: u64) -> Result<u64> {
    standing.checked_sub(counted).ok_or(error!(HookError::OwnerBooksExceedHoldings))
}

///
///
pub fn book_clamp(o: &mut StairOwners, tokens: u64) {
    o.add_clamped(tokens);
}

///
#[inline(never)]
pub fn token_only(lower: i32, upper: i32, liq: u128) -> Result<u64> {
    if liq == 0 {
        return Ok(0);
    }
    let p = math::sqrt_price_at_tick(upper)?;
    Ok(math::position_amounts(p, lower, upper, liq)?.1)
}

///
///
pub fn validate(arg: &StairOwnerArg, s: &FloorPool) -> Result<()> {
    use floor_types::constants::*;
    let ok = match arg.kind {
        STAIR_OWNER_TEAM => {
            s.stair_owner == STAIR_OWNER_TEAM
                && arg.counted == 1
                && arg.payee_bps == GENESIS_TEAM_PAYEE_BPS
                && arg.first_unlock_secs == GENESIS_TEAM_FIRST_CLIFF_SECS
                && arg.every_secs == GENESIS_TEAM_CLIFF_EVERY_SECS
                && arg.anchor > 0

                && arg.owner_tokens == GENESIS_TEAM_STAIR_TOKENS
        }
        STAIR_OWNER_TEAM_BUY => {
            s.stair_owner == STAIR_OWNER_TEAM_BUY
                && arg.counted == 1
                && arg.payee_bps == TEAM_BUY_PAYEE_BPS
                && arg.first_unlock_secs as u64 >= TEAM_LOCK_MIN_DAYS as u64 * 86_400
                && arg.first_unlock_secs as u64 <= TEAM_DAYS_MAX as u64 * 86_400
                && arg.every_secs == 0
        }
        _ => false,
    };
    require!(ok && arg.owner_tokens > 0, HookError::StairOwnerBadConfig);



    let named = (arg.payee != [0u8; 32]) as u8 + (arg.payee_pending_id != [0u8; 32]) as u8;
    let want = if arg.kind == STAIR_OWNER_TEAM { 1 } else { 0 };
    require!(named == want, HookError::StairOwnerBadConfig);


    let pending = arg.kind == STAIR_OWNER_TEAM && arg.payee_pending_id != [0u8; 32];
    let platform_ok = if pending {
        (1..=floor_types::slots::MAX_SLOT_PLATFORM).contains(&arg.payee_platform)
    } else {
        arg.payee_platform == 0
    };
    require!(platform_ok, HookError::StairOwnerBadConfig);
    Ok(())
}

///
///
pub fn team_slot0(arg: &StairOwnerArg) -> crate::state::slots::PayeeSlot {
    use crate::state::slots;
    let pending = arg.payee == [0u8; 32];
    slots::PayeeSlot {
        key: if pending { arg.payee_pending_id } else { arg.payee },
        bps: floor_math::slots::TOTAL_BPS as u16,
        platform: if pending { arg.payee_platform } else { 0 },
        state: if pending { slots::SLOT_STATE_PENDING } else { slots::SLOT_STATE_WALLET },
        ..Default::default()
    }
}

///
///
///
///
pub fn fill_team_slots(fp: &AccountInfo, arg: &StairOwnerArg) -> Result<()> {
    use crate::state::slots;
    let mut t = slots::read(fp)?;
    if arg.kind != STAIR_OWNER_TEAM {
        require!(t.team_count != 0, HookError::StairOwnerBadConfig);
        return Ok(());
    }
    require!(t.team_count == 0, HookError::StairOwnerBadConfig);
    let pending = arg.payee == [0u8; 32];
    t.team[0] = team_slot0(arg);
    t.team_count = 1;
    t.team_pending_mask = pending as u8;
    slots::write(fp, &t)
}

///
///
///
///
#[inline(never)]
pub fn fill(o: &mut StairOwners, arg: &StairOwnerArg, planned: &[u64; 16], steps: u8, now: i64) -> Result<()> {
    require!(o.version == 0, HookError::StairOwnerBadConfig);
    let tokens = match arg.kind {



        STAIR_OWNER_TEAM => {
            let total: u64 = planned[..steps.min(16) as usize].iter().fold(0u64, |a, &x| a.saturating_add(x));
            floor_math::genesis_layout::genesis_layout(total, steps).map_err(|_| error!(HookError::StairOwnerBadConfig))?.team
        }
        _ => {


            let staircase: u64 = planned[..steps.min(16) as usize].iter().fold(0u64, |a, &x| a.saturating_add(x));
            require!(floor_math::owners::team_steps_fit(arg.owner_tokens, staircase), HookError::StairOwnerBadConfig);
            floor_math::owners::owner_tokens_blended(planned, steps, arg.owner_tokens)
                .map_err(|_| error!(HookError::StairOwnerBadConfig))?
        }
    };
    o.version = STAIR_OWNERS_VERSION;
    o.kind = arg.kind;
    o.counted = arg.counted;
    o.first_step = floor_math::owners::first_owner_step(&tokens, steps);
    o.payee_bps = arg.payee_bps;
    o.first_unlock_secs = arg.first_unlock_secs;
    o.every_secs = arg.every_secs;
    o.anchor = if arg.anchor != 0 { arg.anchor } else { now };
    o.tokens = tokens;
    o.owner_total = tokens.iter().sum();
    Ok(())
}

///
///
///
pub fn book_close(o: &mut StairOwners, i: usize, before: u64, after: u64, principal: u64, fee: u64) -> (u64, u64) {
    let owner = o.tokens[i];
    if owner == 0 {
        return (0, 0);
    }
    let (sold, quote) = floor_math::owners::owner_part_of_close(before, after, owner, principal);
    let fee_part = floor_math::owners::owner_fee_part(fee, before, owner);
    o.tokens[i] = owner - sold;
    o.owner_sold = o.owner_sold.saturating_add(sold);
    o.owner_proceeds = o.owner_proceeds.saturating_add(quote);
    o.fees_to_floor = o.fees_to_floor.saturating_add(fee_part);
    (quote, fee_part)
}

///
///
///
///
///
///
///
pub fn hold_step(o: &mut StairOwners, i: usize, avail: u64) -> u64 {
    let books = core::mem::take(&mut o.tokens[i]);
    let kept = books.min(avail);
    o.set_held(o.held().saturating_add(kept));
    o.add_clamped(books - kept);
    kept
}

///
///
///
///
///
#[allow(clippy::too_many_arguments)]
pub fn book_reserve(
    o: &mut StairOwners,
    before: u64,
    after: u64,
    principal: u64,
    fee: u64,
    planned: &[u64; 16],
    j: u8,
    steps: u8,
) -> (u64, u64) {
    let c = floor_math::owners::reserve_close(before, after, planned, &o.cut, &o.tokens, o.reserve_fed(), j, steps);

    let mut owner_in = 0u64;
    for k in j as usize..steps as usize {
        owner_in = owner_in.saturating_add(o.tokens[k]);
    }
    for k in j as usize..steps.min(16) as usize {
        o.tokens[k] = o.tokens[k].saturating_sub(c.owner_sold[k]);
        o.cut[k] = o.cut[k].saturating_add(c.left[k]);
    }

    o.set_reserve_fed(c.protocol_left);
    let total = c.owner_total;
    let sold = before.saturating_sub(after);
    let quote = if sold == 0 { 0 } else { (principal as u128 * total.min(sold) as u128 / sold as u128) as u64 };
    let fee_part = floor_math::owners::owner_fee_part(fee, before, owner_in);
    o.owner_sold = o.owner_sold.saturating_add(total);
    o.owner_proceeds = o.owner_proceeds.saturating_add(quote);
    o.fees_to_floor = o.fees_to_floor.saturating_add(fee_part);
    (quote, fee_part)
}

///
///
///
pub fn cap_reserve(o: &mut StairOwners, placed: u8, steps: u8, held: u64) -> u64 {
    let booked = floor_math::owners::cap_owner_cheapest_first(&mut o.tokens, placed, steps, held);
    o.add_clamped(booked);
    booked
}

///
pub fn planned_less_cut(planned: &[u64; 16], o: Option<&StairOwners>) -> [u64; 16] {
    match o {
        Some(o) => floor_math::owners::owner_planned(planned, &o.cut),
        None => *planned,
    }
}

///
///
pub fn fee_share(o: &mut StairOwners, fee: u64, before: u64, owner: u64) -> u64 {
    let f = floor_math::owners::owner_fee_part(fee, before, owner);
    o.fees_to_floor = o.fees_to_floor.saturating_add(f);
    f
}

///
pub fn in_reserve(o: &StairOwners, placed: u8, steps: u8) -> u64 {
    (placed as usize..steps.min(16) as usize).fold(0u64, |a, k| a.saturating_add(o.tokens[k]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck::Zeroable;

    #[test]
    fn a_close_books_the_owner_down_and_its_fees_to_the_floor() {
        let mut o = StairOwners::zeroed();
        o.tokens[3] = 40;
        let (q, f) = book_close(&mut o, 3, 100, 75, 1_000, 99);
        assert_eq!((q, f, o.tokens[3], o.owner_sold, o.fees_to_floor), (400, 39, 30, 10, 39));
        assert_eq!(book_close(&mut o, 2, 100, 0, 1_000, 5), (0, 0), "a protocol step owes the owner nothing");
        assert_eq!(hold_step(&mut o, 3, 1_000), 30, "the step left unsold: the owner's 30 are HELD, not the protocol's");
        assert_eq!((o.tokens[3], o.held(), o.clamped()), (0, 30, 0));
    }

///
///
///
///
    #[test]
    fn a_step_left_unsold_holds_the_owners_tokens_and_conserves() {
        let mut o = StairOwners::zeroed();
        o.owner_total = 100;
        o.tokens[2] = 60;
        o.tokens[3] = 40;
        assert!(o.conserved());
        assert_eq!(hold_step(&mut o, 2, 1_000), 60);
        assert!(o.conserved(), "standing 40 + held 60");
        assert_eq!(hold_step(&mut o, 3, 35), 35, "only what came back is held");
        assert_eq!((o.held(), o.clamped(), o.standing()), (95, 5, 0));
        assert!(o.conserved(), "held 95 + clamped 5");
        assert_eq!(hold_step(&mut o, 3, 1_000), 0, "nothing left on the step");
    }

///
///
///
///
///
    #[test]
    fn a_second_reserve_close_books_every_owner_token_and_its_whole_principal() {
        let mut o = StairOwners::zeroed();
        let mut planned = [0u64; 16];
        planned[1] = 100;
        planned[2] = 100;
        planned[3] = 100;
        o.tokens = planned;

        let (q1, _) = book_reserve(&mut o, 300, 150, 1_500, 0, &planned, 1, 4);
        let (to_step, rest) = floor_math::stair::reserve_split(150, &planned_less_cut(&planned, Some(&o)), 1, 4).unwrap();
        o.tokens[1] = o.tokens[1].min(to_step);

        let (q2, _) = book_reserve(&mut o, rest, rest - 100, 1_000, 0, &planned, 2, 4);
        assert_eq!((q1, q2), (1_500, 1_000), "every principal unit of an all-owner reserve is the owner's");
        assert_eq!(o.owner_sold, 250);
        assert!(o.tokens[2] + o.tokens[3] <= rest - 100, "books {} over the reserve's {}", o.tokens[2] + o.tokens[3], rest - 100);
    }

///
///
///
///
    #[test]
    fn a_reserve_close_sells_the_fed_record_first() {
        let mut o = StairOwners::zeroed();
        let mut planned = [0u64; 16];
        planned[2] = 100;
        planned[3] = 100;
        o.tokens[2] = 50;
        o.tokens[3] = 100;
        o.cut[2] = 50;
        o.owner_total = 200;
        o.owner_sold = 50;
        o.set_reserve_fed(60);
        assert!(o.conserved());
        let (q, _) = book_reserve(&mut o, 210, 100, 1_100, 0, &planned, 2, 4);
        assert_eq!((q, o.owner_sold, o.tokens[2], o.tokens[3], o.reserve_fed()), (500, 100, 0, 100, 0));
        assert!(o.conserved());

        assert_eq!(cap_reserve(&mut o, 2, 4, 90), 10);
        assert_eq!((o.tokens[3], o.clamped(), o.owner_sold), (90, 10, 100));
        assert!(o.conserved());
    }

    #[test]
    fn counted_is_the_team_buys_standing_only() {
        let mut o = StairOwners::zeroed();
        o.tokens[0] = 5;
        o.tokens[7] = 6;
        assert_eq!(counted(Some(&o)), 0, "counted 0 counts nothing (`validate` refuses 0 for both kinds since ruling A)");
        o.counted = 1;
        assert_eq!(counted(Some(&o)), 11);
        assert_eq!(ladder_less_counted(100, 11).unwrap(), 89);
        assert!(ladder_less_counted(3, 11).is_err(), "books over the positions are refused, never saturated");
        assert_eq!(counted(None), 0);
    }

///
///
///
///
    #[test]
    fn lf_h1_layout_f_books_the_teams_part_of_every_step() {
        use floor_types::constants::*;
        let mut s = FloorPool::zeroed();
        s.stair_kind = STAIR_KIND_GENESIS;
        s.stair_owner = STAIR_OWNER_TEAM;
        assert!(crate::mech::stair::layout_f(&s));
        let n = 12u8;
        let planned = crate::mech::stair::planned_of(&s, GENESIS_STAIR_TOKENS, n, GENESIS_STAIR_GROWTH_BPS).unwrap();
        let arg = StairOwnerArg {
            kind: STAIR_OWNER_TEAM,
            counted: 1,
            payee_bps: GENESIS_TEAM_PAYEE_BPS,
            first_unlock_secs: GENESIS_TEAM_FIRST_CLIFF_SECS,
            every_secs: GENESIS_TEAM_CLIFF_EVERY_SECS,
            anchor: 1_800_000_000,
            owner_tokens: GENESIS_TEAM_STAIR_TOKENS,
            payee: [7u8; 32],
            payee_pending_id: [0u8; 32],
            payee_platform: 0,
        };
        validate(&arg, &s).unwrap();
        assert!(validate(&StairOwnerArg { owner_tokens: 800_000_000_000_000, ..arg }, &s).is_err(), "the old top 800M is refused");
        let mut o = StairOwners::zeroed();
        fill(&mut o, &arg, &planned, n, 0).unwrap();
        let want = floor_math::genesis_layout::genesis_layout(GENESIS_STAIR_TOKENS, n).unwrap();
        assert_eq!(o.tokens, want.team);
        assert_eq!((o.tokens[0], o.tokens[1], o.tokens[2]), (0, 0, GENESIS_TEAM_STEP3_TOKENS));
        assert_eq!((o.owner_total, o.first_step, o.counted), (GENESIS_TEAM_STAIR_TOKENS, 2, 1), "step 3 unlocks first");
        for i in 0..n as usize {
            assert_eq!(planned[i] - o.tokens[i], want.protocol[i], "step {}: the protocol's part", i + 1);
            assert!(planned[i] > o.tokens[i], "step {}: the protocol sells every step", i + 1);
        }
        assert_eq!(counted(Some(&o)), GENESIS_TEAM_STAIR_TOKENS, "every team token counts from day one (ruling A)");

        let mut p = FloorPool::zeroed();
        p.stair_owner = STAIR_OWNER_TEAM_BUY;
        assert_eq!(
            crate::mech::stair::planned_of(&p, GENESIS_STAIR_TOKENS, n, 9_500).unwrap(),
            floor_math::stair::step_tokens(GENESIS_STAIR_TOKENS, n, 9_500).unwrap()
        );
    }

///
///
///
///
    #[test]
    fn lf_h2_the_payee_is_paid_on_the_teams_part_of_a_mixed_step_only() {
        use floor_types::constants::*;
        let l = floor_math::genesis_layout::genesis_layout(GENESIS_STAIR_TOKENS, 12).unwrap();
        let mut o = StairOwners::zeroed();
        o.tokens = l.team;
        let before = l.protocol[3] + l.team[3];
        let principal = 50_000_000_000;
        let (q, _f) = book_close(&mut o, 3, before, before / 2, principal, 0);
        let want = (principal as u128 * l.team[3] as u128 / before as u128) as u64;
        assert_eq!(q, want, "the team's share of the principal is its token share of the step");
        assert!(q < principal && q > principal * 8 / 10, "93.04 / 103.69 of it");
        let sold = l.team[3] - o.tokens[3];
        assert_eq!(sold, ((before - before / 2) as u128 * l.team[3] as u128).div_ceil(before as u128) as u64, "ceiled against the team");
        let due = floor_math::owners::payee_due(q, GENESIS_TEAM_PAYEE_BPS);
        assert_eq!(due, q * 4 / 10);

        assert_eq!(floor_math::owners::payee_after_floor_first(due, principal, 0), (due, 0));
        assert_eq!(floor_math::owners::payee_after_floor_first(due, principal, principal - 1), (1, due - 1));
        assert_eq!(book_close(&mut o, 1, l.protocol[1], 0, principal, 0), (0, 0), "step 2 is the protocol's alone");
    }

///
///
///
    #[test]
    fn mr_s_genesis_pending_team_slot_carries_its_platform() {
        use floor_types::constants::*;
        use floor_types::slots::{SLOT_STATE_PENDING, SLOT_STATE_WALLET};
        let mut s = FloorPool::zeroed();
        s.stair_kind = STAIR_KIND_GENESIS;
        s.stair_owner = STAIR_OWNER_TEAM;
        let pend = StairOwnerArg {
            kind: STAIR_OWNER_TEAM,
            counted: 1,
            payee_bps: GENESIS_TEAM_PAYEE_BPS,
            first_unlock_secs: GENESIS_TEAM_FIRST_CLIFF_SECS,
            every_secs: GENESIS_TEAM_CLIFF_EVERY_SECS,
            anchor: 1_800_000_000,
            owner_tokens: GENESIS_TEAM_STAIR_TOKENS,
            payee: [0u8; 32],
            payee_pending_id: [6u8; 32],
            payee_platform: 2,
        };
        validate(&pend, &s).unwrap();
        let slot = team_slot0(&pend);
        assert_eq!((slot.key, slot.platform, slot.state, slot.bps), ([6u8; 32], 2, SLOT_STATE_PENDING, 10_000));
        for bad in [0u8, 5, 255] {
            assert!(validate(&StairOwnerArg { payee_platform: bad, ..pend }, &s).is_err(), "pending platform {bad}");
        }
        let wallet = StairOwnerArg { payee: [7u8; 32], payee_pending_id: [0u8; 32], payee_platform: 0, ..pend };
        validate(&wallet, &s).unwrap();
        let slot = team_slot0(&wallet);
        assert_eq!((slot.key, slot.platform, slot.state), ([7u8; 32], 0, SLOT_STATE_WALLET));
        assert!(validate(&StairOwnerArg { payee_platform: 1, ..wallet }, &s).is_err(), "a wallet carries platform 0");
    }

}
