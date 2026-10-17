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

use carpenter_clmm_math::big_num::U256;
use carpenter_clmm_math::full_math::MulDiv;
use carpenter_clmm_math::tick_math::{get_sqrt_price_at_tick, MAX_TICK, MIN_TICK};

use crate::error::{FloorMathError, FloorMathResult};
use crate::modes::MAX_TRANCHE_PRICE_X64;

///
///
///
pub const SMART_CAP_TWAP_WINDOW_SECS: u32 = 1_800;

///
///
const POW10_U128_MAX_EXP: u8 = 38;

///
///
///
///
pub fn min_bid_for(decimals: u8) -> u64 {
    if decimals > POW10_U128_MAX_EXP {
        return u64::MAX;
    }
    let unit = 10u128.pow(decimals as u32);
    let v = unit.div_ceil(1_000).max(1);
    u64::try_from(v).unwrap_or(u64::MAX)
}

///
///
///
pub fn quote_amount_at_sqrt(amount: u64, sqrt_price_x64: u128) -> u64 {
    if amount == 0 || sqrt_price_x64 == 0 {
        return 0;
    }
    let s = U256::from(sqrt_price_x64);

    let a = (U256::from(amount) * s) >> 64;
    let v = (a * s) >> 64;
    if v > U256::from(u64::MAX) {
        u64::MAX
    } else {
        v.as_u64()
    }
}

///
pub fn quote_amount_at_tick(amount: u64, tick: i32) -> u64 {
    match get_sqrt_price_at_tick(tick) {
        Ok(s) => quote_amount_at_sqrt(amount, s),
        Err(_) => 0,
    }
}

///
///
///
///
///
///
pub const CHAIN_PRICE_TWAP_WINDOW_SECS: u32 = 1_800;

///
///
///
pub fn chain_hop_tick(twap_tick: i32, live_tick: i32) -> i32 {
    twap_tick.max(live_tick)
}

///
///
///
///
///
///
///
pub fn chained_price_x64(root_price_x64: u128, chain_ticks: i64) -> FloorMathResult<u128> {
    if chain_ticks == 0 {
        return Ok(root_price_x64);
    }
    let t = i32::try_from(chain_ticks).map_err(|_| FloorMathError::TranchePriceOutOfRange)?;
    if !(MIN_TICK..=MAX_TICK).contains(&t) {
        return Err(FloorMathError::TranchePriceOutOfRange);
    }
    let s = U256::from(get_sqrt_price_at_tick(t).map_err(|_| FloorMathError::TranchePriceOutOfRange)?);

    let a = U256::from(root_price_x64) * s;
    let p = a.mul_div_floor(s, U256::one() << 128).ok_or(FloorMathError::TranchePriceOutOfRange)?;
    if p.is_zero() || p >= U256::from(MAX_TRANCHE_PRICE_X64) {
        return Err(FloorMathError::TranchePriceOutOfRange);
    }
    Ok(p.as_u128())
}

#[cfg(test)]
mod tests {
    use super::*;

///
///
    #[test]
    fn the_chained_price_is_the_root_price_times_the_summed_hops() {
        let root = crate::tranche_price_x64(50_000, 15_000, 9).unwrap();
        assert_eq!(chained_price_x64(root, 0), Ok(root), "no hop: unchanged");
        let one = chained_price_x64(root, 6_932).unwrap();
        let ratio = one as f64 / root as f64;
        assert!((ratio - 2.0).abs() < 1e-3, "6,932 ticks ~ 2x, got {ratio}");
        let two = chained_price_x64(root, 6_932 + 23_027).unwrap();
        let r2 = two as f64 / root as f64;
        assert!((r2 - 20.0).abs() / 20.0 < 1e-3, "2x then 10x = 20x, got {r2}");
        let down = chained_price_x64(root, -6_932).unwrap();
        assert!(((down as f64 / root as f64) - 0.5).abs() < 1e-3);
        assert_eq!(chain_hop_tick(100, 50), 100, "the cheaper parent: the higher tick");
        assert_eq!(chain_hop_tick(-5, 7), 7);
        assert_eq!(chained_price_x64(root, MAX_TICK as i64 + 1), Err(FloorMathError::TranchePriceOutOfRange));
        assert_eq!(chained_price_x64(root, i64::MIN), Err(FloorMathError::TranchePriceOutOfRange));
        assert_eq!(chained_price_x64(root, 400_000), Err(FloorMathError::TranchePriceOutOfRange), "past 2^77");
        assert_eq!(chained_price_x64(1, -400_000), Err(FloorMathError::TranchePriceOutOfRange), "rounds to zero");
        assert_eq!(CHAIN_PRICE_TWAP_WINDOW_SECS, 1_800, "the EVM's FLOOR_QUOTE_PRICE_TWAP_WINDOW");
    }

///
    #[test]
    fn min_bid_is_a_thousandth_of_a_token_rounded_up_and_never_zero() {
        assert_eq!(min_bid_for(0), 1, "0 decimals: 1/1000 of one unit rounds UP to one raw unit");
        assert_eq!(min_bid_for(1), 1);
        assert_eq!(min_bid_for(2), 1);
        assert_eq!(min_bid_for(3), 1);
        assert_eq!(min_bid_for(4), 10);
        assert_eq!(min_bid_for(6), 1_000, "a 6-dp FLOOR token: 0.001 = 1,000 raw");
        assert_eq!(min_bid_for(8), 100_000);
        assert_eq!(min_bid_for(9), 1_000_000, "wSOL-shaped: 0.001 SOL = 1,000,000 lamports");
        assert_eq!(min_bid_for(18), 1_000_000_000_000_000);
        assert_eq!(min_bid_for(22), 10_000_000_000_000_000_000u128.min(u64::MAX as u128) as u64);
        assert_eq!(min_bid_for(36), u64::MAX, "saturates, never overflows");
        assert_eq!(min_bid_for(255), u64::MAX);
        for d in 0..=40u8 {
            assert!(min_bid_for(d) >= 1, "d = {d}");
        }
    }

///
///
    #[test]
    fn the_cap_converts_at_the_raw_unit_ratio_and_is_monotone() {
        let at0 = quote_amount_at_tick(1_000_000, 0);
        assert!((999_999..=1_000_000).contains(&at0), "tick 0 ~ 1:1, got {at0}");

        let at_double = quote_amount_at_tick(1_000_000, 6_932);
        assert!((1_999_000..=2_000_100).contains(&at_double), "6,932 ticks ~ 2x, got {at_double}");
        let at_half = quote_amount_at_tick(1_000_000, -6_932);
        assert!((499_900..=500_100).contains(&at_half), "-6,932 ticks ~ 0.5x, got {at_half}");
        let mut prev = 0;
        for t in (-100_000..=100_000).step_by(997) {
            let v = quote_amount_at_tick(10_000_000_000, t);
            assert!(v >= prev, "monotone in the tick at {t}");
            prev = v;
        }
        assert!(quote_amount_at_tick(2, 1_000) >= quote_amount_at_tick(1, 1_000));
        assert_eq!(quote_amount_at_tick(0, 50_000), 0);
    }

///
///
///
///
///
    #[test]
    fn the_cap_rides_two_levels_through_different_decimals() {
        let sol_cap = 50_000_000u64;
        let f_cap = quote_amount_at_tick(sol_cap, 23_027);
        assert!((499_000_000..=500_100_000).contains(&f_cap), "500 F (6 dp), got {f_cap}");
        let g_cap = quote_amount_at_tick(f_cap, 6_932);
        assert!((998_000_000..=1_000_100_000).contains(&g_cap), "1,000 G (6 dp), got {g_cap}");
    }

    #[test]
    fn the_smart_cap_reads_the_evms_thirty_minute_twap() {
        assert_eq!(SMART_CAP_TWAP_WINDOW_SECS, 1_800);
    }

///
    #[test]
    fn the_cap_saturates_at_the_extremes() {
        use carpenter_clmm_math::tick_math::{MAX_TICK, MIN_TICK};
        assert_eq!(quote_amount_at_tick(u64::MAX, MAX_TICK), u64::MAX);
        assert_eq!(quote_amount_at_sqrt(u64::MAX, u128::MAX), u64::MAX);
        assert_eq!(quote_amount_at_tick(1, MIN_TICK), 0);
        assert_eq!(quote_amount_at_tick(1_000, MAX_TICK + 1), 0, "an out-of-range tick is the conservative 0");
        assert_eq!(quote_amount_at_sqrt(1_000, 0), 0);
    }
}
