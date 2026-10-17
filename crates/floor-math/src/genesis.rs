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
use carpenter_clmm_math::tick_math::get_tick_at_sqrt_price;

use crate::constants::{BPS, SUPPLY};
use crate::error::{FloorMathError, FloorMathResult};
use crate::price::sqrt_price_from_ratio;

///
///
///
///
///
///
pub const GENESIS_SALE: u64 = GENESIS_CORE_TOKENS + GENESIS_FF_TOKENS;
///
///
pub const GENESIS_CORE_TOKENS: u64 = 20_000_000_000_000;
///
///
///
pub const GENESIS_FF_TOKENS: u64 = 22_500_000_000_000;
///
pub const GENESIS_PRICE_UNIT_TOKENS: u64 = 1_000_000_000_000;
///
///
///
///
///
///
pub const GENESIS_SALE_PRICE_LAMPORTS_PER_MILLION: u64 = 4_000_000_000;
///
pub const GENESIS_CORE_CAP_LAMPORTS: u64 = 80_000_000_000;
///
pub const GENESIS_FF_CAP_LAMPORTS: u64 = 90_000_000_000;
///
pub const GENESIS_OPEN_MCAP_LAMPORTS: u64 = 4_000_000_000_000;
const _: () = assert!(GENESIS_CORE_CAP_LAMPORTS as u128 * GENESIS_PRICE_UNIT_TOKENS as u128 == GENESIS_CORE_TOKENS as u128 * GENESIS_SALE_PRICE_LAMPORTS_PER_MILLION as u128);
const _: () = assert!(GENESIS_FF_CAP_LAMPORTS as u128 * GENESIS_PRICE_UNIT_TOKENS as u128 == GENESIS_FF_TOKENS as u128 * GENESIS_SALE_PRICE_LAMPORTS_PER_MILLION as u128);
const _: () = assert!(GENESIS_OPEN_MCAP_LAMPORTS as u128 * GENESIS_PRICE_UNIT_TOKENS as u128 == SUPPLY as u128 * GENESIS_SALE_PRICE_LAMPORTS_PER_MILLION as u128);

///
///
///
pub const GENESIS_OPEN_TICK_MIN: i32 = -189_081;
pub const GENESIS_OPEN_TICK_MAX: i32 = 443_515;

///
///
///
///
pub fn genesis_open_tick() -> FloorMathResult<(i32, bool)> {
    let raw = get_tick_at_sqrt_price(sqrt_price_from_ratio(
        GENESIS_SALE_PRICE_LAMPORTS_PER_MILLION as u128,
        GENESIS_PRICE_UNIT_TOKENS as u128,
    )?)?;
    let tick = raw.clamp(GENESIS_OPEN_TICK_MIN, GENESIS_OPEN_TICK_MAX);
    Ok((tick, tick != raw))
}

///
pub fn genesis_tokens_at_price(lamports: u64) -> FloorMathResult<u64> {
    u64::try_from(lamports as u128 * GENESIS_PRICE_UNIT_TOKENS as u128 / GENESIS_SALE_PRICE_LAMPORTS_PER_MILLION as u128)
        .map_err(|_| FloorMathError::Overflow)
}

///
///
///
pub fn genesis_allocation(core: u64, ff: u64) -> FloorMathResult<u64> {
    genesis_tokens_at_price(core)?.checked_add(genesis_tokens_at_price(ff)?).ok_or(FloorMathError::Overflow)
}

///
///
///
///
pub fn genesis_sold(raised_core: u64, raised_ff: u64) -> FloorMathResult<(u64, u64)> {
    let core = genesis_tokens_at_price(raised_core)?.min(GENESIS_CORE_TOKENS);
    let ff = genesis_tokens_at_price(raised_ff)?.min(GENESIS_FF_TOKENS);
    Ok((core, ff))
}

///
///
///
///
///
pub const GENESIS_BUYER_VEST_SECS: u64 = 30 * 86_400;
pub const GENESIS_BUYER_CLIFF_SECS: u64 = 14 * 86_400;
pub fn genesis_buyer_vested(allocation: u64, elapsed: i64) -> u64 {
    if elapsed < GENESIS_BUYER_CLIFF_SECS as i64 {
        return 0;
    }
    let e = (elapsed as u64).min(GENESIS_BUYER_VEST_SECS);
    (allocation as u128 * e as u128 / GENESIS_BUYER_VEST_SECS as u128) as u64
}

///
///
///
pub fn genesis_split(raised: u64, floor_share_bps: u16) -> FloorMathResult<(u64, u64)> {
    if floor_share_bps as u64 > BPS {
        return Err(FloorMathError::Overflow);
    }
    let initial = u64::try_from(raised as u128 * floor_share_bps as u128 / BPS as u128).map_err(|_| FloorMathError::Overflow)?;
    Ok((initial, raised - initial))
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
pub fn genesis_stair_steps(top_mcap: u64, open_tick: i32, spacing: i32) -> FloorMathResult<u8> {
    let max = crate::stair::MAX_STAIR_STEPS as u32;
    let open = GENESIS_OPEN_MCAP_LAMPORTS as u128;
    let top = top_mcap as u128;
    let mut n: u32 = if top <= open {
        0
    } else {

        let mut k = 0u32;
        while (open << (k + 1)) <= top {
            k += 1;
        }
        let base = open << k;

        let up = U256::from(top) * U256::from(top) >= U256::from(base) * U256::from(base) * U256::from(2u8);
        k + up as u32
    };
    n = n.clamp(1, max);
    let open_edge = crate::spacing::floor_to_spacing(open_tick, spacing)?;
    while n > 1 && !crate::stair::stair_fits(open_edge, n as u8, spacing)? {
        n -= 1;
    }
    Ok(n as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use carpenter_clmm_math::tick_math::{get_sqrt_price_at_tick, MAX_SQRT_PRICE_X64, MAX_TICK};

///
///
///
    #[test]
    fn sgr_m1_the_open_is_the_fixed_price() {
        let (t, clamped) = genesis_open_tick().unwrap();
        assert!(!clamped && (55_210..=55_222).contains(&t), "{t}");
        assert_eq!(GENESIS_OPEN_TICK_MAX, MAX_TICK - 120 - 1, "`create_launch`'s own bound on an open");
        assert!(get_sqrt_price_at_tick(t).unwrap() < MAX_SQRT_PRICE_X64);
        assert!(t > GENESIS_OPEN_TICK_MIN && t < GENESIS_OPEN_TICK_MAX);

        let ratio = get_tick_at_sqrt_price(sqrt_price_from_ratio(40_000_000_000, 10_000_000_000_000).unwrap()).unwrap();
        assert_eq!(t, ratio);
    }

///
///
///
    #[test]
    fn sgr_m2_one_fixed_price() {
        const SOL: u64 = 1_000_000_000;
        assert_eq!(GENESIS_SALE_PRICE_LAMPORTS_PER_MILLION, 4 * SOL);
        assert_eq!((GENESIS_CORE_TOKENS, GENESIS_FF_TOKENS, GENESIS_SALE), (20_000_000_000_000, 22_500_000_000_000, 42_500_000_000_000));
        assert_eq!((GENESIS_CORE_CAP_LAMPORTS, GENESIS_FF_CAP_LAMPORTS), (80 * SOL, 90 * SOL));
        assert_eq!(GENESIS_OPEN_MCAP_LAMPORTS, 4_000 * SOL);
        assert_eq!(GENESIS_OPEN_MCAP_LAMPORTS as u128 * 150 / SOL as u128, 600_000, "$600k at $150");
        assert_eq!(genesis_tokens_at_price(SOL).unwrap(), 250_000_000_000, "1 SOL buys 250,000 tokens");
        assert_eq!(genesis_tokens_at_price(GENESIS_CORE_CAP_LAMPORTS).unwrap(), GENESIS_CORE_TOKENS);
        assert_eq!(genesis_tokens_at_price(GENESIS_FF_CAP_LAMPORTS).unwrap(), GENESIS_FF_TOKENS);
        assert_eq!(genesis_tokens_at_price(1).unwrap(), 250);
        assert_eq!(genesis_tokens_at_price(5 * SOL).unwrap(), 1_250_000_000_000, "5 SOL buys 1.25M in Phase 2");
    }

///
///
///
    #[test]
    fn sgr_m3_both_phases_buy_at_the_one_price() {
        const SOL: u64 = 1_000_000_000;
        const M: u64 = 1_000_000_000_000;
        assert_eq!(genesis_allocation(20 * SOL, 5 * SOL).unwrap(), 5 * M + 1_250_000_000_000, "a Core wallet in Phase 2 too");
        assert_eq!(genesis_allocation(0, 5 * SOL).unwrap(), 1_250_000_000_000);
        assert_eq!(genesis_allocation(7, 3).unwrap(), 2_500);
        assert_eq!(genesis_allocation(0, 0).unwrap(), 0);
        assert!(genesis_allocation(u64::MAX, 1).is_err(), "an overflow is refused, never wrapped");
    }

///
///
    #[test]
    fn sgr_m4_sold_and_unsold() {
        const SOL: u64 = 1_000_000_000;
        const M: u64 = 1_000_000_000_000;
        for ((c, f), (sc, sf)) in [((80, 90), (20 * M, 22_500_000_000_000)), ((80, 45), (20 * M, 11_250_000_000_000)), ((80, 0), (20 * M, 0)), ((40, 20), (10 * M, 5 * M))] {
            assert_eq!(genesis_sold(c * SOL, f * SOL).unwrap(), (sc, sf), "{c} + {f}");
        }
        assert_eq!(GENESIS_SALE - 10 * M - 5 * M, 27_500_000_000_000, "40 + 20 leaves 27.5M to the protocol's steps");
        assert_eq!(genesis_sold(0, 0).unwrap(), (0, 0));

        assert_eq!(genesis_sold(500 * SOL, 500 * SOL).unwrap(), (20 * M, 22_500_000_000_000));
    }

///
    #[test]
    fn sgr_m5_allocations_never_pass_what_was_sold() {
        const SOL: u64 = 1_000_000_000;
        let core = [20 * SOL, 20 * SOL, 7_333_333_333, 1];
        let ff = [5 * SOL, 3_333_333_333, 4_999_999_999, 1, 7];
        let (rc, rf) = (core.iter().sum::<u64>(), ff.iter().sum::<u64>());
        let (sc, sf) = genesis_sold(rc, rf).unwrap();
        let got: u64 = core.iter().chain(&ff).map(|c| genesis_tokens_at_price(*c).unwrap()).sum();
        assert_eq!(got, sc + sf, "250 base units a lamport: no dust");
    }

///
    #[test]
    fn sgr_m6_split() {
        assert_eq!(genesis_split(3_000, 10_000).unwrap(), (3_000, 0));
        assert_eq!(genesis_split(3_000, 0).unwrap(), (0, 3_000));
        assert_eq!(genesis_split(3, 5_000).unwrap(), (1, 2));
        assert!(genesis_split(3, 10_001).is_err());
    }

///
///
///
    #[test]
    fn sgr_m7_the_step_count_is_the_nearest_doubling_to_the_top() {
        let s = 60;
        let (t, _) = genesis_open_tick().unwrap();
        let steps = |top: u64| genesis_stair_steps(top, t, s).unwrap();
        let open = GENESIS_OPEN_MCAP_LAMPORTS;

        assert_eq!(steps(17_066_666_666_666_667), 12, "the suite's top");

        assert_eq!(steps(open << 9), 9);
        assert_eq!(steps((open << 9) + (open << 9) / 100_000 * 41_421), 9, "just under 2^9.5 rounds down");
        assert_eq!(steps((open << 9) + (open << 9) / 100_000 * 41_422), 10, "just over 2^9.5 rounds up");
        assert_eq!(steps(u64::MAX), 16, "u64::MAX lamports is 2^22.1 of the open: clamped at 16");

        assert_eq!(steps(1), 1);
        assert_eq!(steps(open), 1);
        assert_eq!(steps(open * 2), 1, "one doubling");

        for t in [GENESIS_OPEN_TICK_MIN, 0, GENESIS_OPEN_TICK_MAX] {
            let e = crate::spacing::floor_to_spacing(t, s).unwrap();
            assert!(crate::stair::stair_fits(e, 16, s).unwrap(), "{t}");
        }
    }

///
///
    #[test]
    fn sgr_m8_the_top_alone_sets_the_count() {
        let (t, _) = genesis_open_tick().unwrap();
        let at = |usd: u128| genesis_stair_steps((usd * 1_000_000_000 / 150) as u64, t, 60).unwrap();
        assert_eq!(at(2_560_000_000), 12);
        assert_eq!(at(1_000_000_000), 11);
        assert_eq!(at(5_000_000_000), 13);
        assert_eq!(at(500_000_000), 10);
    }

///
///
///
    #[test]
    fn sgr_m9_the_buyers_vest_over_thirty_days_from_a_fourteen_day_cliff() {
        const D: i64 = 86_400;
        let a = 1_000_000_000_000u64;
        assert_eq!(genesis_buyer_vested(a, -1), 0);
        assert_eq!(genesis_buyer_vested(a, 0), 0);
        assert_eq!(genesis_buyer_vested(a, 14 * D - 1), 0, "a second before the cliff");
        assert_eq!(genesis_buyer_vested(a, 14 * D), 466_666_666_666, "14/30 at the cliff");
        assert_eq!(genesis_buyer_vested(a, 22 * D), 733_333_333_333);
        assert_eq!(genesis_buyer_vested(a, 30 * D - 1), a - 385_803, "a second before the end: floored");
        assert_eq!(genesis_buyer_vested(a, 30 * D), a);
        assert_eq!(genesis_buyer_vested(a, 365 * D), a, "never more than the allocation");
        assert_eq!(genesis_buyer_vested(u64::MAX, 20 * D), (u64::MAX as u128 * 20 / 30) as u64, "u128: no overflow");
        let mut last = 0;
        for t in (14 * D..=31 * D).step_by(3_607) {
            let v = genesis_buyer_vested(a, t);
            assert!(v >= last && v <= a);
            last = v;
        }
    }
}
