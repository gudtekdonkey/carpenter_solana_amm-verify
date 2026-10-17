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
use carpenter_clmm_math::big_num::U256;
use carpenter_clmm_math::fixed_point_64::Q64;
use carpenter_clmm_math::full_math::MulDiv;
use carpenter_types::constants::*;

use crate::errors::AmmError;
use crate::state::Tick;

///
pub const fn is_dynamic_fee(fee: u32) -> bool {
    fee == DYNAMIC_FEE_FLAG
}

///
pub fn check_key_fee(fee: u32) -> Result<()> {
    require!(is_dynamic_fee(fee) || fee <= MAX_LP_FEE, AmmError::FeeInvalid);
    Ok(())
}

///
pub const fn initial_lp_fee(fee: u32) -> u32 {
    if is_dynamic_fee(fee) {
        0
    } else {
        fee
    }
}

const fn has(flags: u16, flag: u16) -> bool {
    flags & flag != 0
}

///
pub const fn hook_flags_valid(flags: u16) -> bool {
    if flags & !ALL_HOOK_FLAGS != 0 {
        return false;
    }
    if !has(flags, BEFORE_SWAP_FLAG) && has(flags, BEFORE_SWAP_RETURNS_DELTA_FLAG) {
        return false;
    }
    if !has(flags, AFTER_SWAP_FLAG) && has(flags, AFTER_SWAP_RETURNS_DELTA_FLAG) {
        return false;
    }
    if !has(flags, AFTER_ADD_LIQUIDITY_FLAG) && has(flags, AFTER_ADD_LIQUIDITY_RETURNS_DELTA_FLAG) {
        return false;
    }
    if !has(flags, AFTER_REMOVE_LIQUIDITY_FLAG) && has(flags, AFTER_REMOVE_LIQUIDITY_RETURNS_DELTA_FLAG) {
        return false;
    }

    if has(flags, REQUESTS_ACTIONS_FLAG) && !has(flags, AFTER_SWAP_FLAG) {
        return false;
    }


    if has(flags, SETTLES_FLAG)
        && !(has(flags, BEFORE_SWAP_FLAG) && has(flags, AFTER_SWAP_FLAG) && has(flags, AFTER_SWAP_RETURNS_DELTA_FLAG))
    {
        return false;
    }

    if has(flags, AFTER_ADD_LIQUIDITY_RETURNS_DELTA_FLAG) || has(flags, AFTER_REMOVE_LIQUIDITY_RETURNS_DELTA_FLAG) {
        return false;
    }
    true
}

///
///
pub const fn hook_fee_valid(has_hook: bool, flags: u16, fee: u32) -> bool {
    if has_hook {
        flags & ALL_HOOK_FLAGS != 0 || is_dynamic_fee(fee)
    } else {
        !is_dynamic_fee(fee)
    }
}

///
pub fn fee_growth_inside(
    lower: &Tick,
    upper: &Tick,
    tick_lower: i32,
    tick_upper: i32,
    tick_current: i32,
    fee_growth_global0_x64: u128,
    fee_growth_global1_x64: u128,
) -> (u128, u128) {
    let (l0, l1) = (lower.fee_growth_outside0_x64.get(), lower.fee_growth_outside1_x64.get());
    let (u0, u1) = (upper.fee_growth_outside0_x64.get(), upper.fee_growth_outside1_x64.get());
    if tick_current < tick_lower {
        (l0.wrapping_sub(u0), l1.wrapping_sub(u1))
    } else if tick_current >= tick_upper {
        (u0.wrapping_sub(l0), u1.wrapping_sub(l1))
    } else {
        (
            fee_growth_global0_x64.wrapping_sub(l0).wrapping_sub(u0),
            fee_growth_global1_x64.wrapping_sub(l1).wrapping_sub(u1),
        )
    }
}

///
pub fn fees_owed(inside_x64: u128, last_x64: u128, liquidity: u128) -> u64 {
    let growth = U256::from(inside_x64.wrapping_sub(last_x64));
    match growth.mul_div_floor(U256::from(liquidity), U256::from(Q64)) {
        Some(v) if v <= U256::from(u64::MAX) => v.as_u64(),
        _ => u64::MAX,
    }
}

///
///
pub const fn fee_saturated(fee: u64) -> bool {
    fee == u64::MAX
}

///
///
pub fn clamp_paid_fee(owed: u64, vault_amount: u64, claims: u64, protocol_fees_owed: u64, principal_out: u64) -> (u64, bool) {
    let available = vault_amount
        .saturating_sub(claims)
        .saturating_sub(protocol_fees_owed)
        .saturating_sub(principal_out);
    if owed > available {
        (available, true)
    } else {
        (owed, fee_saturated(owed))
    }
}

///
///
pub fn donate_growth(amount: u64, liquidity: u128) -> Result<u128> {
    require!(liquidity != 0, AmmError::NoLiquidity);
    let v = U256::from(amount)
        .mul_div_floor(U256::from(Q64), U256::from(liquidity))
        .ok_or(AmmError::MathOverflow)?;
    require!(v <= U256::from(u128::MAX), AmmError::MathOverflow);
    Ok(v.as_u128())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_fee_rules() {
        assert!(check_key_fee(DYNAMIC_FEE_FLAG).is_ok());
        assert!(check_key_fee(MAX_LP_FEE).is_ok());
        assert!(check_key_fee(MAX_LP_FEE + 1).is_err());
        assert!(check_key_fee(DYNAMIC_FEE_FLAG | 1).is_err());
        assert_eq!(initial_lp_fee(DYNAMIC_FEE_FLAG), 0);
        assert_eq!(initial_lp_fee(3000), 3000);
    }

///
///
    const FLOOR_HOOK_FLAGS: u16 = 0x60CC;

    #[test]
    fn flag_rules() {
        assert!(hook_flags_valid(FLOOR_HOOK_FLAGS));
        assert!(hook_flags_valid(0));
        assert!(!hook_flags_valid(BEFORE_SWAP_RETURNS_DELTA_FLAG));
        assert!(!hook_flags_valid(AFTER_SWAP_RETURNS_DELTA_FLAG));
        assert!(!hook_flags_valid(REQUESTS_ACTIONS_FLAG));
        assert!(!hook_flags_valid(1 << 15));
        assert!(!hook_flags_valid(AFTER_ADD_LIQUIDITY_FLAG | AFTER_ADD_LIQUIDITY_RETURNS_DELTA_FLAG));
        assert!(hook_fee_valid(false, 0, 3000));
        assert!(!hook_fee_valid(false, 0, DYNAMIC_FEE_FLAG));
        assert!(hook_fee_valid(true, 0, DYNAMIC_FEE_FLAG));
        assert!(!hook_fee_valid(true, 0, 3000));
        assert!(hook_fee_valid(true, BEFORE_INITIALIZE_FLAG, 3000));
    }

    #[test]
    fn owed_and_clamp() {
        assert_eq!(fees_owed(3 << 64, 1 << 64, 5), 10);

        assert_eq!(fees_owed(0, u128::MAX, 1 << 64), 1);
        assert_eq!(fees_owed(u128::MAX, 0, u128::MAX), u64::MAX);
        assert_eq!(clamp_paid_fee(10, 100, 50, 0, 45), (5, true));
        assert_eq!(clamp_paid_fee(4, 100, 50, 0, 45), (4, false));
        assert_eq!(clamp_paid_fee(4, 10, 50, 0, 0), (0, true));
    }

    #[test]
    fn donation_growth_fits_at_the_minimum() {
        let g = donate_growth(u64::MAX / 2, MIN_DONATE_LIQUIDITY).unwrap();
        assert!(g > 0);
        assert!(donate_growth(1, 0).is_err());
    }
}
