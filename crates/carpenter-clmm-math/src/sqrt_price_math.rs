// Derived from raydium-io/raydium-clmm @ ed7c84a5, programs/amm/src/libraries/sqrt_price_math.rs, Apache-2.0.
// Modified by FLOOR, 2026: Anchor's `Result` / `ErrorCode` replaced by `MathResult` / `MathError`
// (`require!` / `require_gt!` spelled out); import paths; the token_0-removal branch of
// `get_next_sqrt_price_from_amount_0_rounding_up` returns `CalculateOverflow` where upstream's `as_u128()`
// panicked on a price above u128 (v4 reverts there too); `get_next_sqrt_price_from_amount_1_rounding_down`
// returns `ZeroLiquidity` instead of dividing by zero. The formulas and rounding are unchanged.

use crate::big_num::{U128, U256};
use crate::error::{MathError, MathResult};
use crate::fixed_point_64;
use crate::full_math::MulDiv;
use crate::tick_math;
use crate::unsafe_math::UnsafeMathTrait;

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
pub fn get_next_sqrt_price_from_amount_0_rounding_up(
    sqrt_price_x64: u128,
    liquidity: u128,
    amount: u64,
    add: bool,
) -> MathResult<u128> {
    if amount == 0 {
        return Ok(sqrt_price_x64);
    };
    let numerator_1 = U256::from(liquidity) << fixed_point_64::RESOLUTION;

    if add {
        if let Some(product) = U256::from(amount).checked_mul(U256::from(sqrt_price_x64)) {
            let denominator = numerator_1 + product;
            if denominator >= numerator_1 {
                return Ok(numerator_1
                    .mul_div_ceil(U256::from(sqrt_price_x64), denominator)
                    .ok_or(MathError::CalculateOverflow)?
                    .as_u128());
            };
        }

        Ok(U256::div_rounding_up(
            numerator_1,
            (numerator_1 / U256::from(sqrt_price_x64))
                .checked_add(U256::from(amount))
                .ok_or(MathError::CalculateOverflow)?,
        )
        .as_u128())
    } else {
        let product = U256::from(amount)
            .checked_mul(U256::from(sqrt_price_x64))
            .ok_or(MathError::CalculateOverflow)?;
        let denominator = numerator_1
            .checked_sub(product)
            .ok_or(MathError::CalculateOverflow)?;
        let next = numerator_1
            .mul_div_ceil(U256::from(sqrt_price_x64), denominator)
            .ok_or(MathError::CalculateOverflow)?;



        if next > U256::from(u128::MAX) {
            return Err(MathError::CalculateOverflow);
        }
        Ok(next.as_u128())
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
pub fn get_next_sqrt_price_from_amount_1_rounding_down(
    sqrt_price_x64: u128,
    liquidity: u128,
    amount: u64,
    add: bool,
) -> MathResult<u128> {
    if amount == 0 {
        return Ok(sqrt_price_x64);
    }


    if liquidity == 0 {
        return Err(MathError::ZeroLiquidity);
    }

    if add {
        let quotient = (U128::from(amount) << fixed_point_64::RESOLUTION) / U128::from(liquidity);
        sqrt_price_x64
            .checked_add(quotient.as_u128())
            .ok_or(MathError::CalculateOverflow)
    } else {
        let quotient = U128::div_rounding_up(
            U128::from(amount) << fixed_point_64::RESOLUTION,
            U128::from(liquidity),
        );
        sqrt_price_x64
            .checked_sub(quotient.as_u128())
            .ok_or(MathError::CalculateOverflow)
    }
}

///
///
///
pub fn get_next_sqrt_price_from_input(
    sqrt_price_x64: u128,
    liquidity: u128,
    amount_in: u64,
    zero_for_one: bool,
) -> MathResult<u128> {
    if !(sqrt_price_x64 >= tick_math::MIN_SQRT_PRICE_X64
        && sqrt_price_x64 < tick_math::MAX_SQRT_PRICE_X64)
    {
        return Err(MathError::SqrtPriceOutOfRange);
    }
    if !(liquidity > 0) {
        return Err(MathError::ZeroLiquidity);
    }


    if zero_for_one {
        get_next_sqrt_price_from_amount_0_rounding_up(sqrt_price_x64, liquidity, amount_in, true)
    } else {
        get_next_sqrt_price_from_amount_1_rounding_down(sqrt_price_x64, liquidity, amount_in, true)
    }
}

///
///
///
///
///
pub fn get_next_sqrt_price_from_output(
    sqrt_price_x64: u128,
    liquidity: u128,
    amount_out: u64,
    zero_for_one: bool,
) -> MathResult<u128> {
    if !(sqrt_price_x64 >= tick_math::MIN_SQRT_PRICE_X64
        && sqrt_price_x64 < tick_math::MAX_SQRT_PRICE_X64)
    {
        return Err(MathError::SqrtPriceOutOfRange);
    }
    if !(liquidity > 0) {
        return Err(MathError::ZeroLiquidity);
    }

    if zero_for_one {
        get_next_sqrt_price_from_amount_1_rounding_down(
            sqrt_price_x64,
            liquidity,
            amount_out,
            false,
        )
    } else {
        get_next_sqrt_price_from_amount_0_rounding_up(sqrt_price_x64, liquidity, amount_out, false)
    }
}
