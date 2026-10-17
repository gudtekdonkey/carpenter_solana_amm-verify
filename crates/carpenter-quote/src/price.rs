//!
//!
//!
//!
//!
//!
//!
//!
//!

use carpenter_clmm_math::tick_math;
use floor_types::constants::{MAX_TICK, MIN_TICK};

use crate::error::{QuoteError, QuoteResult};

pub fn sqrt_price_at_tick(tick: i32) -> QuoteResult<u128> {
    Ok(tick_math::get_sqrt_price_at_tick(tick)?)
}

pub fn tick_at_sqrt_price(sqrt_price_x64: u128) -> QuoteResult<i32> {
    Ok(tick_math::get_tick_at_sqrt_price(sqrt_price_x64)?)
}

///
///
///
///
///
///
///
///
pub fn floor_to_spacing(tick: i32, spacing: u16) -> QuoteResult<i32> {
    let s = spacing_i32(spacing)?;
    crate::mech::floor_to_spacing(in_range(tick)?, s)
}

///
///
pub fn ceil_to_spacing(tick: i32, spacing: u16) -> QuoteResult<i32> {
    let s = spacing_i32(spacing)?;
    crate::mech::ceil_to_spacing(in_range(tick)?, s)
}

///
///
fn in_range(tick: i32) -> QuoteResult<i32> {
    if !(MIN_TICK..=MAX_TICK).contains(&tick) {
        return Err(QuoteError::Math(carpenter_clmm_math::MathError::TickOutOfRange));
    }
    Ok(tick)
}

///
///
pub fn min_usable_tick(spacing: u16) -> QuoteResult<i32> {
    let s = spacing_i32(spacing)?;
    Ok(-((-MIN_TICK).div_euclid(s) * s))
}

///
pub fn max_usable_tick(spacing: u16) -> QuoteResult<i32> {
    let s = spacing_i32(spacing)?;
    Ok(MAX_TICK.div_euclid(s) * s)
}

fn spacing_i32(spacing: u16) -> QuoteResult<i32> {
    if spacing == 0 {
        return Err(QuoteError::Math(carpenter_clmm_math::MathError::InvalidTickSpacing));
    }
    Ok(i32::from(spacing))
}

///
pub fn price_ui_at_sqrt(sqrt_price_x64: u128, decimals0: u8, decimals1: u8) -> f64 {
    let s = sqrt_price_x64 as f64 / 18_446_744_073_709_551_616.0;
    s * s * 10f64.powi(i32::from(decimals0) - i32::from(decimals1))
}

///
pub fn price_ui_at_tick(tick: i32, decimals0: u8, decimals1: u8) -> QuoteResult<f64> {
    Ok(price_ui_at_sqrt(sqrt_price_at_tick(tick)?, decimals0, decimals1))
}

///
///
///
pub fn tick_at_price_ui(price: f64, decimals0: u8, decimals1: u8) -> QuoteResult<i32> {
    if !(price.is_finite() && price > 0.0) {
        return Err(QuoteError::Math(carpenter_clmm_math::MathError::SqrtPriceOutOfRange));
    }
    let raw = price / 10f64.powi(i32::from(decimals0) - i32::from(decimals1));
    let guess = (raw.ln() / 1.0001f64.ln()).floor();
    let mut t = if guess.is_finite() { (guess as i64).clamp(MIN_TICK as i64, MAX_TICK as i64) as i32 } else { MIN_TICK };

    while t < MAX_TICK && price_ui_at_tick(t + 1, decimals0, decimals1)? <= price {
        t += 1;
    }
    while t > MIN_TICK && price_ui_at_tick(t, decimals0, decimals1)? > price {
        t -= 1;
    }
    Ok(t)
}
