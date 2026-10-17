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
use floor_types::constants::{MIN_TICK, SALT_SEED};

use crate::errors::HookError;
use crate::mech::amm_cpi;
use crate::mech::positions::{add, intake, math, Mech, MechEvent};
use crate::state::FloorPool;

///
///
///
///
pub fn book_seed_floor(s: &mut FloorPool, credited: u64, team: u64, events: &mut Vec<MechEvent>) -> Result<()> {
    let backing = credited.checked_sub(team).ok_or(HookError::BadRange)?;
    s.team_owed = add(s.team_owed, team)?;
    let kept = intake(s, backing, events)?;
    s.vault_eth = add(s.vault_eth, kept)?;
    s.needs_ratchet = 1;
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
pub fn fix_seed_range(s: &mut FloorPool, lower: i32, upper: i32, signer: &Pubkey) -> Result<()> {
    require!(lower < upper, HookError::BadRange);
    require!(lower % s.tick_spacing == 0 && upper % s.tick_spacing == 0, HookError::BadRange);
    if s.seed_liq.get() == 0 {
        require_keys_eq!(*signer, s.pool_initializer, HookError::NotInitializer);
        s.seed_lower = lower;
        s.seed_upper = upper;
    } else {
        require!(lower == s.seed_lower && upper == s.seed_upper, HookError::SeedRangeFixed);
    }
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
pub fn open_seed(m: &mut Mech, s: &mut FloorPool, credited: u64) -> Result<u64> {
    let liq = math::position_liquidity(m.market.sqrt_price_x64, s.seed_lower, s.seed_upper, 0, credited)?;
    require!(liq > 0, HookError::BadRange);
    let r = m.open(s, SALT_SEED, s.seed_lower, s.seed_upper, liq)?;
    let (quote_used, token_used) = amm_cpi::used(&r)?;


    require!(quote_used == 0, HookError::InvariantViolated);
    require!(token_used <= credited, HookError::InvariantViolated);
    let (fee_eth, fee_tok) = amm_cpi::fees(&r);
    if fee_eth > 0 || fee_tok > 0 {
        let eth = intake(s, fee_eth, &mut m.events)?;
        s.vault_eth = add(s.vault_eth, eth)?;
        crate::mech::positions::to_wall(s, fee_tok)?;
        s.seed_fees_swept_eth = add(s.seed_fees_swept_eth, eth)?;
        s.seed_fees_swept_tokens = add(s.seed_fees_swept_tokens, fee_tok)?;
    }
    let new_liq = s.seed_liq.get().checked_add(liq).ok_or(HookError::MathOverflow)?;
    s.seed_liq.set(new_liq);
    s.needs_ratchet = 1;
    Ok(token_used)
}

///
///
///
///
///
pub fn seed_inventory(
    m: &mut Mech,
    s: &mut FloorPool,
    credited: u64,
    open_tick: i32,
    owner: Option<&floor_types::owners::StairOwnerArg>,
) -> Result<()> {
    let spacing = s.tick_spacing;
    s.ladder_upper = math::floor_to_spacing(open_tick, spacing)?;
    s.ladder_lower = math::ceil_to_spacing(MIN_TICK, spacing)?;




    require!(s.stair_steps != 0, HookError::StairShapeOutOfRange);
    crate::mech::stair::place_at_graduation(m, s, credited, owner)
}
