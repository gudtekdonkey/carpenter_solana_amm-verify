//!
//!
//!
//!
//!

use anchor_lang::prelude::*;

use crate::errors::HookError;

///
pub fn sqrt_at_tick(tick: i32) -> Result<u128> {
    carpenter_clmm_math::tick_math::get_sqrt_price_at_tick(tick).map_err(|_| error!(HookError::MathOverflow))
}
