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

use carpenter_clmm_math::big_num::U256;
use carpenter_clmm_math::fixed_point_64::Q64;
use carpenter_clmm_math::full_math::MulDiv;
use carpenter_clmm_math::liquidity_math::{get_delta_amount_0_unsigned, get_delta_amount_1_unsigned};
use carpenter_clmm_math::tick_math::get_sqrt_price_at_tick;

use crate::error::{FloorMathError, FloorMathResult};

fn bounds(lower: i32, upper: i32) -> FloorMathResult<(u128, u128)> {
    Ok((get_sqrt_price_at_tick(lower)?, get_sqrt_price_at_tick(upper)?))
}

///
fn liq0(a: u128, b: u128, amount: u64) -> FloorMathResult<U256> {
    if b <= a {
        return Err(FloorMathError::InvalidRange);
    }
    let ab = U256::from(a)
        .mul_div_floor(U256::from(b), U256::from(Q64))
        .ok_or(FloorMathError::Overflow)?;
    U256::from(amount)
        .mul_div_floor(ab, U256::from(b - a))
        .ok_or(FloorMathError::Overflow)
}

///
fn liq1(a: u128, b: u128, amount: u64) -> FloorMathResult<U256> {
    if b <= a {
        return Err(FloorMathError::InvalidRange);
    }
    U256::from(amount)
        .mul_div_floor(U256::from(Q64), U256::from(b - a))
        .ok_or(FloorMathError::Overflow)
}

///
///
///
pub fn position_liquidity(
    price: u128,
    lower: i32,
    upper: i32,
    quote: u64,
    base: u64,
) -> FloorMathResult<u128> {
    let (a, b) = bounds(lower, upper)?;
    position_liquidity_at_sqrt(price, a, b, quote, base)
}

///
///
pub(crate) fn position_liquidity_at_sqrt(price: u128, a: u128, b: u128, quote: u64, base: u64) -> FloorMathResult<u128> {
    let l = if price <= a {
        liq0(a, b, quote)?
    } else if price >= b {
        liq1(a, b, base)?
    } else {
        let l0 = liq0(price, b, quote)?;
        let l1 = liq1(a, price, base)?;
        if l0 < l1 {
            l0
        } else {
            l1
        }
    };
    Ok(if l > U256::from(u128::MAX) { 0 } else { l.as_u128() })
}

///
///
pub fn position_amounts(
    price: u128,
    lower: i32,
    upper: i32,
    liquidity: u128,
) -> FloorMathResult<(u64, u64)> {
    if liquidity == 0 {
        return Ok((0, 0));
    }
    let (a, b) = bounds(lower, upper)?;
    let mut p = price;
    if p < a {
        p = a;
    }
    if p > b {
        p = b;
    }
    let amount0 = get_delta_amount_0_unsigned(p, b, liquidity, false)?;
    let amount1 = get_delta_amount_1_unsigned(a, p, liquidity, false)?;
    Ok((amount0, amount1))
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
pub fn bid_amounts(
    price: u128,
    lower: i32,
    width: i32,
    liquidity: u128,
    deployed: u64,
) -> FloorMathResult<(u64, u64)> {
    if liquidity == 0 {
        return Ok((0, 0));
    }
    let untouched = get_sqrt_price_at_tick(lower)?;
    if price <= untouched {
        return Ok((deployed, 0));
    }
    let upper = lower.checked_add(width).ok_or(FloorMathError::Overflow)?;
    position_amounts(price, lower, upper, liquidity)
}

///
///
///
///
///
///
///
///
pub fn pol_base(tick: i32, lower: i32, upper: i32, liquidity: u128) -> FloorMathResult<u64> {
    let p = get_sqrt_price_at_tick(tick)?;
    if liquidity == 0 {
        return Ok(0);
    }
    let (a, b) = bounds(lower, upper)?;
    pol_base_at_sqrt(p, a, b, liquidity)
}

///
///
pub(crate) fn pol_base_at_sqrt(mut p: u128, a: u128, b: u128, liquidity: u128) -> FloorMathResult<u64> {
    if p < a {
        p = a;
    }
    if p > b {
        p = b;
    }
    Ok(get_delta_amount_1_unsigned(a, p, liquidity, false)?)
}

///
///
///
pub fn base_at_sqrt(p: u128, a: u128, b: u128, liquidity: u128) -> FloorMathResult<u64> {
    if liquidity == 0 {
        return Ok(0);
    }
    pol_base_at_sqrt(p, a, b, liquidity)
}

///
pub(crate) fn edges(lower: i32, upper: i32, liquidity: u128) -> FloorMathResult<Option<(u128, u128, u128)>> {
    if liquidity == 0 {
        return Ok(None);
    }
    let (a, b) = bounds(lower, upper)?;
    Ok(Some((a, b, liquidity)))
}

///
///
///
///
///
pub fn quote_cost(base: u64, tick: i32) -> FloorMathResult<u128> {
    quote_cost_at_sqrt(base, get_sqrt_price_at_tick(tick)?)
}

///
pub(crate) fn quote_cost_at_sqrt(base: u64, sqrt: u128) -> FloorMathResult<u128> {
    let sp = U256::from(sqrt);
    let q = U256::from(Q64);
    let x = U256::from(base).mul_div_floor(q, sp).ok_or(FloorMathError::Overflow)?;
    let y = x.mul_div_floor(q, sp).ok_or(FloorMathError::Overflow)?;
    Ok(if y > U256::from(u128::MAX) { u128::MAX } else { y.as_u128() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use carpenter_clmm_math::tick_math::{MAX_TICK, MIN_TICK};


    #[test]
    fn pol_base_is_bounded_by_the_seed() {
        let (lower, upper) = (-6960, 0);
        let seed = 150_000_000_000_000u64;
        let p_top = get_sqrt_price_at_tick(upper).unwrap();
        let l = position_liquidity(p_top, lower, upper, 0, seed).unwrap();
        assert!(l > 0);
        assert_eq!(pol_base(lower - 60, lower, upper, l).unwrap(), 0);
        let at_top = pol_base(upper + 6000, lower, upper, l).unwrap();
        assert!(at_top <= seed && seed - at_top < 2, "seed {seed} vs {at_top}");
        let mut last = 0;
        for t in (lower..=upper).step_by(60) {
            let b = pol_base(t, lower, upper, l).unwrap();
            assert!(b >= last, "base must grow with tick");
            last = b;
        }
        assert_eq!(pol_base(0, lower, upper, 0).unwrap(), 0);
        assert!(pol_base(MAX_TICK + 1, lower, upper, 0).is_err());
    }















    #[test]
    fn pol_base_below_a_dear_range_is_zero_not_an_overflow() {
        let open = -240_000;
        let (lower, upper) = (-246_960, -240_000);
        let p = get_sqrt_price_at_tick(open).unwrap();
        let l = position_liquidity(p, lower, upper, 0, 150_000_000_000_000).unwrap();
        assert_eq!(l, 83_019_744_082_906_835_908);

        assert_eq!(
            position_amounts(get_sqrt_price_at_tick(lower - 60).unwrap(), lower, upper, l),
            Err(FloorMathError::Math(carpenter_clmm_math::MathError::MaxTokenOverflow))
        );
        assert_eq!(pol_base(lower - 60, lower, upper, l), Ok(0));
        assert_eq!(pol_base(lower, lower, upper, l), Ok(0));

        let p_in = get_sqrt_price_at_tick(open - 3_000).unwrap();
        assert!(position_amounts(p_in, lower, upper, l).is_err());
        let mid = pol_base(open - 3_000, lower, upper, l).unwrap();
        assert!(mid > 0 && mid < 150_000_000_000_000, "{mid}");

        let top = pol_base(upper + 60, lower, upper, l).unwrap();
        assert_eq!(top, position_amounts(get_sqrt_price_at_tick(upper + 60).unwrap(), lower, upper, l).unwrap().1);
        assert!(150_000_000_000_000 - top < 2);



        let (tail_lower, tail_upper) = (-443_580, 99_000);
        let p_open = get_sqrt_price_at_tick(99_039).unwrap();
        let l_tail = position_liquidity(p_open, tail_lower, tail_upper, 0, 150_000_000_000_000).unwrap();
        assert!(l_tail > 0);
        assert!(position_amounts(get_sqrt_price_at_tick(tail_lower).unwrap(), tail_lower, tail_upper, l_tail).is_err());
        assert_eq!(pol_base(tail_lower, tail_lower, tail_upper, l_tail), Ok(0));
    }


    #[test]
    fn bid_untouched_reports_the_recorded_deployment() {
        for width in [60, 120] {
            let (lower, s) = (1200, width);
            let p_low = get_sqrt_price_at_tick(lower).unwrap();
            let l = position_liquidity(p_low, lower, lower + s, 1_000_000_000, 0).unwrap();
            assert_eq!(bid_amounts(p_low, lower, s, l, 999_999_999).unwrap(), (999_999_999, 0));
            let p_high = get_sqrt_price_at_tick(lower + s).unwrap();
            let (q, b) = bid_amounts(p_high + 1, lower, s, l, 999_999_999).unwrap();
            assert_eq!(q, 0);
            assert!(b > 0);


            let p_mid = get_sqrt_price_at_tick(lower + 30).unwrap();
            let (q, b) = bid_amounts(p_mid, lower, s, l, 1_000_000_000).unwrap();
            assert!(q > 0 && q < 1_000_000_000 && b > 0, "width {width}");
        }
        let (lower, s) = (1200, 60);
        let p_low = get_sqrt_price_at_tick(lower).unwrap();
        let l = position_liquidity(p_low, lower, lower + s, 1_000_000_000, 0).unwrap();
        assert_eq!(bid_amounts(p_low, lower, s, l, 999_999_999).unwrap(), (999_999_999, 0));
        assert_eq!(bid_amounts(p_low - 1, lower, s, l, 7).unwrap(), (7, 0));
        assert_eq!(bid_amounts(p_low, lower, s, 0, 7).unwrap(), (0, 0));

        let p_high = get_sqrt_price_at_tick(lower + s).unwrap();
        let (q, b) = bid_amounts(p_high + 1, lower, s, l, 999_999_999).unwrap();
        assert_eq!(q, 0);
        assert!(b > 0);

        let p_mid = get_sqrt_price_at_tick(lower + 30).unwrap();
        let (q, b) = bid_amounts(p_mid, lower, s, l, 1_000_000_000).unwrap();
        assert!(q > 0 && q < 1_000_000_000 && b > 0);
    }






    #[test]
    fn no_liquidity_is_empty_whatever_was_deployed() {
        let p = get_sqrt_price_at_tick(-46_015).unwrap();
        assert_eq!(bid_amounts(p, 160_260, 60, 0, 2_247_591_574).unwrap(), (0, 0));
        assert_eq!(bid_amounts(p, 160_260, 60, 0, u64::MAX).unwrap(), (0, 0));

        let p_low = get_sqrt_price_at_tick(160_260).unwrap();
        assert_eq!(bid_amounts(p_low, 160_260, 60, 0, 7).unwrap(), (0, 0));
        assert_eq!(bid_amounts(p_low + 1, 160_260, 60, 0, 7).unwrap(), (0, 0));


        assert_eq!(bid_amounts(p, MAX_TICK + 1, 60, 0, 7).unwrap(), (0, 0));
        assert!(bid_amounts(p, MAX_TICK + 1, 60, 1, 7).is_err());

        assert_eq!(bid_amounts(p, 160_260, 60, 1, 2_247_591_574).unwrap(), (2_247_591_574, 0));
    }


    #[test]
    fn liquidity_then_amounts_never_exceeds_the_deposit() {
        for (lower, upper) in [(-6960, 0), (60, 120), (-443_520, -443_460), (443_400, 443_520)] {
            let a = get_sqrt_price_at_tick(lower).unwrap();
            let b = get_sqrt_price_at_tick(upper).unwrap();
            let mid = a + (b - a) / 3;
            for (price, quote, base) in [(a, 5_000_000_000u64, 0u64), (b, 0, 7_000_000_000), (mid, 3_000_000_000, 4_000_000_000)] {
                let l = position_liquidity(price, lower, upper, quote, base).unwrap();
                if l == 0 {
                    continue;
                }
                let (q, bb) = position_amounts(price, lower, upper, l).unwrap();
                assert!(q <= quote && bb <= base, "[{lower},{upper}] {q}/{quote} {bb}/{base}");
            }
        }
    }

    #[test]
    fn empty_range_is_refused() {
        let p = get_sqrt_price_at_tick(0).unwrap();
        assert_eq!(position_liquidity(p, 60, 60, 1, 1), Err(FloorMathError::InvalidRange));
        assert_eq!(position_liquidity(p, 120, 60, 1, 1), Err(FloorMathError::InvalidRange));
    }



    #[test]
    fn u64_amounts_cannot_reach_the_u128_liquidity_cap() {
        for (lo, hi) in [(MAX_TICK - 1, MAX_TICK), (MIN_TICK, MIN_TICK + 1)] {
            let a = get_sqrt_price_at_tick(lo).unwrap();
            let b = get_sqrt_price_at_tick(hi).unwrap();
            let l0 = position_liquidity(a, lo, hi, u64::MAX, 0).unwrap();
            let l1 = position_liquidity(b, lo, hi, 0, u64::MAX).unwrap();
            assert!(l0 > 0 && l0 < 1u128 << 112, "{l0}");
            assert!(l1 > 0 && l1 < 1u128 << 112, "{l1}");
        }
    }


    #[test]
    fn quote_cost_is_floor_and_never_narrowed() {
        assert_eq!(quote_cost(1_000, 0).unwrap(), 1_000);


        assert_eq!(quote_cost(1_000_000, -6932).unwrap(), 2_000_035);

        let big = quote_cost(1_000_000_000_000_000, -443_580).unwrap();
        assert!(big > u64::MAX as u128 && big < u128::MAX);

        assert!(quote_cost(u64::MAX, MIN_TICK).unwrap() > u64::MAX as u128);

        let sp = U256::from(get_sqrt_price_at_tick(-1234).unwrap());
        let exact = (U256::from(987_654_321u64) << 128usize) / (sp * sp);
        assert!(U256::from(quote_cost(987_654_321, -1234).unwrap()) <= exact);
    }
}
