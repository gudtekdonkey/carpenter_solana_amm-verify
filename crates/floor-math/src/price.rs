//!
//!
//!
//!
//!

use carpenter_clmm_math::big_num::U256;
use carpenter_clmm_math::full_math::MulDiv;
use carpenter_clmm_math::tick_math::{get_tick_at_sqrt_price, MAX_SQRT_PRICE_X64, MIN_SQRT_PRICE_X64};

use crate::error::{FloorMathError, FloorMathResult};
use crate::spacing::floor_to_spacing;

///
///
const Q96: u128 = 1u128 << 96;

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
pub fn sqrt_price_from_ratio(eth: u128, tok: u128) -> FloorMathResult<u128> {
    if tok == 0 {
        return Err(FloorMathError::ZeroDenominator);
    }
    if eth == 0 {
        return Ok(MAX_SQRT_PRICE_X64 - 1);
    }


    let ratio_x96 = U256::from(tok)
        .mul_div_floor(U256::from(Q96), U256::from(eth))
        .ok_or(FloorMathError::Overflow)?;
    let sqrt = ratio_x96.integer_sqrt() << 16usize;
    if sqrt < U256::from(MIN_SQRT_PRICE_X64) {
        return Ok(MIN_SQRT_PRICE_X64);
    }
    if sqrt >= U256::from(MAX_SQRT_PRICE_X64) {
        return Ok(MAX_SQRT_PRICE_X64 - 1);
    }
    Ok(sqrt.as_u128())
}

///
///
///
///
///
///
pub fn ask_tick(
    cost: u64,
    tokens: u64,
    multiple_bps: u64,
    floor_tick: i32,
    min_spread: i32,
    spacing: i32,
) -> FloorMathResult<i32> {
    let raw_ceiling = floor_tick.checked_sub(min_spread).ok_or(FloorMathError::Overflow)?;
    let ceiling = floor_to_spacing(raw_ceiling, spacing)?;
    if tokens == 0 || cost == 0 {
        return Ok(ceiling);
    }

    let target = (cost as u128 * multiple_bps as u128) / 10_000;
    let sqrt = sqrt_price_from_ratio(target, tokens as u128)?;
    let by_basis = floor_to_spacing(get_tick_at_sqrt_price(sqrt)?, spacing)?;
    Ok(by_basis.min(ceiling))
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
pub fn open_tick(price_x64: u128) -> FloorMathResult<i32> {
    if price_x64 == 0 {
        return Err(FloorMathError::ZeroDenominator);
    }
    let sqrt = ((U256::one() << 192usize) / U256::from(price_x64)).integer_sqrt();
    if sqrt >= U256::from(MAX_SQRT_PRICE_X64) {
        return Err(carpenter_clmm_math::MathError::SqrtPriceOutOfRange.into());
    }
    Ok(get_tick_at_sqrt_price(sqrt.as_u128())?)
}






#[cfg(test)]
mod tests {
    use super::*;

    use crate::constants::TICKS_PER_DOUBLING;
    use carpenter_clmm_math::fixed_point_64::Q64;
    use carpenter_clmm_math::tick_math::{get_sqrt_price_at_tick, MAX_TICK};


    #[test]
    fn ratio_edges() {
        assert_eq!(sqrt_price_from_ratio(1, 0), Err(FloorMathError::ZeroDenominator));
        assert_eq!(sqrt_price_from_ratio(0, 1).unwrap(), MAX_SQRT_PRICE_X64 - 1);
        assert_eq!(sqrt_price_from_ratio(0, 0), Err(FloorMathError::ZeroDenominator));

        assert_eq!(sqrt_price_from_ratio(7, 7).unwrap(), Q64);

        assert_eq!(sqrt_price_from_ratio(1, 4).unwrap(), 2 * Q64);

        assert_eq!(sqrt_price_from_ratio(u128::MAX, 1).unwrap(), MIN_SQRT_PRICE_X64);
        assert_eq!(sqrt_price_from_ratio(1, u128::MAX).unwrap(), MAX_SQRT_PRICE_X64 - 1);
    }



    #[test]
    fn ratio_rounds_down_within_2_pow_16() {
        let cases: [(u128, u128); 8] = [
            (1_000_000_000, 1_000_000_000_000_000),
            (3, 10),
            (123_456_789, 987_654_321),
            (50_000_000_000, 550_000_000_000_000),
            (999_999_937, 1),
            (1, 999_999_937),
            (1_275_002_924_882_050_770, 2_196),
            (426_441_616_129, 376),
        ];
        for (eth, tok) in cases {
            let s = sqrt_price_from_ratio(eth, tok).unwrap();

            let lhs = U256::from(s) * U256::from(s) * U256::from(eth);
            assert!(lhs <= U256::from(tok) * (U256::one() << 128usize), "{eth}/{tok}");
            let up = U256::from(s) + (U256::one() << 16usize);
            let lhs_up = up * up * U256::from(eth);
            assert!(lhs_up > U256::from(tok) * (U256::one() << 128usize), "{eth}/{tok}");
        }
    }








    #[test]
    fn matches_the_evm_x96_root_bit_for_bit() {
        let cases: [(u128, u128, Option<i32>); 9] = [
            (1_275_002_924_882_050_770, 2_196, Some(-339_968)),
            (426_441_616_129, 376, Some(-208_502)),
            (1_249_717_930_458, 1_000_000_000_000_000, Some(66_851)),
            (56, 4, Some(-26_392)),
            (1_007_062_920_101_364_418, 11_156_033_366_948, Some(-114_112)),
            (3, 10, None),
            (123_456_789, 987_654_321, None),
            (u64::MAX as u128, 1, None),
            (1, u64::MAX as u128, None),
        ];
        for (eth, tok, evm_tick) in cases {
            let ratio_x96 = (U256::from(tok) * (U256::one() << 96usize)) / U256::from(eth);
            let evm_x96 = ratio_x96.integer_sqrt() << 48usize;
            assert!(evm_x96 & ((U256::one() << 48usize) - U256::one()) == U256::zero());
            let mut evm_q64 = (evm_x96 >> 32usize).as_u128();
            evm_q64 = evm_q64.clamp(MIN_SQRT_PRICE_X64, MAX_SQRT_PRICE_X64 - 1);
            let ours = sqrt_price_from_ratio(eth, tok).unwrap();
            assert_eq!(ours, evm_q64, "{eth}/{tok}");
            if let Some(t) = evm_tick {
                assert_eq!(get_tick_at_sqrt_price(ours).unwrap(), t, "{eth}/{tok}");
            }
        }
    }


    #[test]
    fn ask_tick_takes_the_higher_ask_of_basis_and_floor() {
        let s = 60;

        assert_eq!(ask_tick(0, 5, 20_000, 0, TICKS_PER_DOUBLING, s).unwrap(), -6960);
        assert_eq!(ask_tick(5, 0, 20_000, 0, TICKS_PER_DOUBLING, s).unwrap(), -6960);



        assert_eq!(ask_tick(1_000_000, 1_000_000, 20_000, 100_000, TICKS_PER_DOUBLING, s).unwrap(), -6960);

        let at = ask_tick(1, 1_000_000_000, 20_000, -50_000, TICKS_PER_DOUBLING, s).unwrap();
        assert_eq!(at, floor_to_spacing(-50_000 - 6932, s).unwrap());

        assert_eq!(ask_tick(1, 1, 1, i32::MIN, 1, s), Err(FloorMathError::Overflow));
    }




    #[test]
    fn evm_open_tick_is_never_higher() {
        for price_x64 in [
            922_337_203_685_477u128,
            (1u128 << 64) / 3,
            (1u128 << 64) / 7 + 1,
            1u128 << 64,
            (1u128 << 64) + 12_345,
            (1u128 << 77) - 1,
            (1u128 << 77) / 3,
            1u128 << 20,
            (1u128 << 20) + 1,
            999_999_999_999,
        ] {
            let ratio_x96 = (U256::one() << 160usize) / U256::from(price_x64);
            let evm_x96 = ratio_x96.integer_sqrt() << 48usize;
            let evm_tick = get_tick_at_sqrt_price((evm_x96 >> 32usize).as_u128()).unwrap();
            let ours = open_tick(price_x64).unwrap();
            assert!(evm_tick <= ours, "price {price_x64}: evm {evm_tick} > ours {ours}");
            assert!(ours - evm_tick <= 1, "price {price_x64}: evm {evm_tick}, ours {ours}");
        }
    }


    #[test]
    fn open_tick_brackets_the_sale_price() {
        for price_x64 in [
            922_337_203_685_477u128,
            (1u128 << 64) / 3,
            1u128 << 64,
            (1u128 << 77) - 1,
            1u128 << 20,
        ] {
            let t = open_tick(price_x64).unwrap();
            let exact = ((U256::one() << 192usize) / U256::from(price_x64)).integer_sqrt().as_u128();
            assert!(get_sqrt_price_at_tick(t).unwrap() <= exact);
            if t < MAX_TICK {
                assert!(get_sqrt_price_at_tick(t + 1).unwrap() > exact);
            }
        }
        assert_eq!(open_tick(0), Err(FloorMathError::ZeroDenominator));

        assert!(open_tick(1).is_err());
    }


}
