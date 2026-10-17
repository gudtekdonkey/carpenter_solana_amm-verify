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
use carpenter_clmm_math::{liquidity_math, MathError};
use carpenter_types::constants::{MAX_TICK, MIN_TICK, TICK_ARRAY_SIZE};

use crate::errors::AmmError;
use crate::state::Tick;

///
///
///
///
pub fn math_err(e: MathError) -> Error {
    let code = match e {
        MathError::TickOutOfRange => AmmError::TickInvalid,
        MathError::SqrtPriceOutOfRange => AmmError::PriceLimitInvalid,
        MathError::LiquidityUnderflow => AmmError::InsufficientLiquidity,
        MathError::ZeroLiquidity => AmmError::NoLiquidity,
        MathError::InvalidTickArrayBoundary => AmmError::TickArrayStartInvalid,
        MathError::InvalidTickSpacing => AmmError::TickSpacingInvalid,
        MathError::InvalidFeeForExactOut => AmmError::InvalidFeeForExactOut,
        MathError::FeeRateTooHigh => AmmError::FeeInvalid,
        _ => AmmError::MathOverflow,
    };
    code.into()
}

///
///
pub fn check_tick(tick: i32, tick_spacing: u16) -> Result<()> {
    require!(tick_spacing != 0, AmmError::TickSpacingInvalid);
    require!(
        (MIN_TICK..=MAX_TICK).contains(&tick) && tick % tick_spacing as i32 == 0,
        AmmError::TickInvalid
    );
    Ok(())
}

///
pub fn check_range(tick_lower: i32, tick_upper: i32, tick_spacing: u16) -> Result<()> {
    check_tick(tick_lower, tick_spacing)?;
    check_tick(tick_upper, tick_spacing)?;
    require!(tick_lower < tick_upper, AmmError::TickInvalid);
    Ok(())
}

///
pub fn tick_offset(array_start: i32, tick: i32, tick_spacing: u16) -> Result<usize> {
    let s = tick_spacing as i32;
    require!(s != 0, AmmError::TickSpacingInvalid);
    let rel = tick.checked_sub(array_start).ok_or(AmmError::TickInvalid)?;
    require!(rel >= 0 && rel % s == 0 && rel / s < TICK_ARRAY_SIZE, AmmError::TickInvalid);
    Ok((rel / s) as usize)
}

///
pub fn max_liquidity_per_tick(tick_spacing: u16) -> u128 {
    let s = tick_spacing.max(1) as i32;

    let min_tick = (MIN_TICK / s) * s;
    let max_tick = (MAX_TICK / s) * s;
    let num_ticks = ((max_tick - min_tick) / s) as u128 + 1;
    u128::MAX / num_ticks
}

///
///
///
///
///
///
#[allow(clippy::too_many_arguments)]
pub fn update_tick(
    tick: &mut Tick,
    tick_index: i32,
    tick_current: i32,
    liquidity_delta: i128,
    fee_growth_global0_x64: u128,
    fee_growth_global1_x64: u128,
    upper: bool,
    max_liquidity: u128,
) -> Result<bool> {
    let gross_before = tick.liquidity_gross.get();
    let gross_after = liquidity_math::add_delta(gross_before, liquidity_delta).map_err(math_err)?;
    if liquidity_delta > 0 {
        require!(gross_after <= max_liquidity, AmmError::MathOverflow);
    }
    let flipped = (gross_after == 0) != (gross_before == 0);

    if gross_before == 0 && tick_index <= tick_current {

        tick.fee_growth_outside0_x64.set(fee_growth_global0_x64);
        tick.fee_growth_outside1_x64.set(fee_growth_global1_x64);
    }

    let net = tick.liquidity_net.get();
    let net_after = if upper { net.checked_sub(liquidity_delta) } else { net.checked_add(liquidity_delta) }
        .ok_or(AmmError::MathOverflow)?;

    tick.liquidity_gross.set(gross_after);
    tick.liquidity_net.set(net_after);
    Ok(flipped)
}

///
pub fn cross_tick(tick: &mut Tick, fee_growth_global0_x64: u128, fee_growth_global1_x64: u128) -> i128 {
    let o0 = tick.fee_growth_outside0_x64.get();
    let o1 = tick.fee_growth_outside1_x64.get();
    tick.fee_growth_outside0_x64.set(fee_growth_global0_x64.wrapping_sub(o0));
    tick.fee_growth_outside1_x64.set(fee_growth_global1_x64.wrapping_sub(o1));
    tick.liquidity_net.get()
}

///
///
pub fn clear_tick(tick: &mut Tick) {
    *tick = Tick::default();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_liquidity_matches_v4_formula() {
        let s = 60i32;
        let n = (((MAX_TICK / s) * s - (MIN_TICK / s) * s) / s) as u128 + 1;
        assert_eq!(max_liquidity_per_tick(60), u128::MAX / n);
        assert!(max_liquidity_per_tick(1) < max_liquidity_per_tick(60));
    }

    #[test]
    fn update_then_cross_follows_v4() {
        let mut t = Tick::default();
        let max = max_liquidity_per_tick(1);

        assert!(update_tick(&mut t, -10, 0, 100, 7, 9, false, max).unwrap());
        assert_eq!(t.fee_growth_outside0_x64.get(), 7);
        assert_eq!(t.liquidity_net.get(), 100);

        assert!(!update_tick(&mut t, -10, 0, 50, 70, 90, true, max).unwrap());
        assert_eq!(t.fee_growth_outside0_x64.get(), 7);
        assert_eq!(t.liquidity_net.get(), 50);
        assert_eq!(cross_tick(&mut t, 10, 20), 50);
        assert_eq!(t.fee_growth_outside0_x64.get(), 3);
        assert_eq!(t.fee_growth_outside1_x64.get(), 11);

        assert!(update_tick(&mut t, -10, 0, -151, 0, 0, false, max).is_err());
        assert!(update_tick(&mut t, -10, 0, -150, 0, 0, false, max).unwrap());
    }

    #[test]
    fn cross_wraps() {
        let mut t = Tick::default();
        t.fee_growth_outside0_x64.set(5);
        cross_tick(&mut t, 3, 0);
        assert_eq!(t.fee_growth_outside0_x64.get(), 3u128.wrapping_sub(5));
    }

    #[test]
    fn offsets_and_ranges() {
        assert_eq!(tick_offset(-3600, -3600, 60).unwrap(), 0);
        assert_eq!(tick_offset(-3600, -60, 60).unwrap(), 59);
        assert!(tick_offset(-3600, 0, 60).is_err());
        assert!(tick_offset(0, 30, 60).is_err());
        assert!(check_range(60, 60, 60).is_err());
        assert!(check_range(-60, 60, 60).is_ok());
        assert!(check_tick(MAX_TICK, 1).is_ok());
        assert!(check_tick(MAX_TICK + 1, 1).is_err());
    }
}
