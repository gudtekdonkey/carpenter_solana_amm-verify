//!
//!
//!
//!
//!
//!
//!
//!
//!

///
pub const SKIM_RAMP_TICKS: i32 = 16094;

///
///
///
///
///
///
pub const BID_BAND_TICKS: i32 = 120;

///
pub const TICKS_PER_DOUBLING: i32 = 6932;



///
pub const LP_FEE_PIPS: u32 = 2500;
///
///
///
///
///
///
///
pub const DEFAULT_SKIM_BUY_AT_FLOOR_BPS: u64 = 0;
pub const DEFAULT_SKIM_SELL_AT_FLOOR_BPS: u64 = 1000;
///
///
///
///
pub const SKIM_SELL_AT_FLOOR_MULTIPLE_BPS: u64 = 15_000;
///
///
///
pub const fn skim_sell_at_floor_bps(sell_far: u16) -> u16 {
    (sell_far as u64 * SKIM_SELL_AT_FLOOR_MULTIPLE_BPS / 10_000) as u16
}
const _: () = assert!(skim_sell_at_floor_bps(crate::policy::MAX_SKIM_FAR_BPS) == 1500);
///
///
///
///
///
pub const PRESET_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS: u64 = 15_000;
pub const MIN_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS: u64 = 10_000;
pub const MAX_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS: u64 = 50_000;
///
///
pub const MAX_SKIM_SELL_AT_FLOOR_BPS: u64 = 1_500;
const _: () = assert!(skim_sell_at_floor_bps(crate::policy::MAX_SKIM_FAR_BPS) as u64 == MAX_SKIM_SELL_AT_FLOOR_BPS);
///
///
pub const fn skim_sell_at_floor_bps_at(sell_far: u16, multiple_bps: u64) -> u16 {
    let v = (sell_far as u128 * multiple_bps as u128) / 10_000;
    if v > u16::MAX as u128 { u16::MAX } else { v as u16 }
}
const _: () = assert!(skim_sell_at_floor_bps_at(crate::policy::MAX_SKIM_FAR_BPS, MAX_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS) == 5_000);

///
///
///
///
pub const SUPPLY: u64 = 1_000_000_000_000_000;
pub const SALE: u64 = 300_000_000_000_000;
pub const POL: u64 = 100_000_000_000_000;
pub const LADDER: u64 = 600_000_000_000_000;
const _: () = assert!(SALE + POL + LADDER == SUPPLY);

///
pub const MIN_SELLABLE: u64 = 1_000_000_000;

///
pub const BID_SALT: u64 = 1;
pub const SEED_SALT: u64 = 3;




///
pub const BPS: u64 = 10_000;

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
pub const REFERRAL_CREATOR_SHARE_BPS: u64 = 1_000;

///
///
///
///
pub const MAX_REFERRAL_SHARE_BPS: u64 = 10_000;

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
pub const MIN_QUOTE_DECIMALS: u8 = 0;
pub const MAX_QUOTE_DECIMALS: u8 = 36;

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
pub const fn some_launch_prices_at(decimals: u8) -> bool {
    const RHS: u128 = 8192 * (u64::MAX as u128) * (SUPPLY as u128);
    match 10u128.checked_pow(decimals as u32) {
        Some(unit) => match unit.checked_mul(100) {
            Some(lhs) => lhs < RHS,
            None => false,
        },
        None => false,
    }
}



const _: () = assert!(some_launch_prices_at(MAX_QUOTE_DECIMALS));
const _: () = assert!(!some_launch_prices_at(MAX_QUOTE_DECIMALS + 1));
const _: () = assert!(MIN_QUOTE_DECIMALS == 0, "u8's own floor; 10^0 = 1 raw unit");

const _: () = assert!(quote_unit(MAX_QUOTE_DECIMALS).is_some());
const _: () = assert!(quote_unit(MAX_QUOTE_DECIMALS + 1).is_none());

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
pub const fn quote_unit(decimals: u8) -> Option<u128> {
    if decimals > MAX_QUOTE_DECIMALS {
        return None;
    }
    10u128.checked_pow(decimals as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    #[test]
    fn the_quote_decimals_window_is_where_some_launch_still_prices_under_the_cap() {
        use carpenter_clmm_math::big_num::U256;
        use carpenter_clmm_math::fixed_point_64::Q64;
        use carpenter_clmm_math::full_math::MulDiv;
        use crate::modes::MAX_TRANCHE_PRICE_X64;

        assert_eq!((MIN_QUOTE_DECIMALS, MAX_QUOTE_DECIMALS), (0, 36));


        let p_min = |d: u8| -> U256 {
            let unit = (0..d).fold(U256::from(1u8), |a, _| a * U256::from(10u8));
            (U256::from(100u8) * unit)
                .mul_div_floor(U256::from(Q64), U256::from(u64::MAX) * U256::from(SUPPLY))
                .expect("fits U512")
        };
        let cap = U256::from(MAX_TRANCHE_PRICE_X64);
        assert!(p_min(36) < cap, "d=36: {} prices", p_min(36));
        assert!(p_min(37) >= cap, "d=37: {} is past the cap for EVERY input", p_min(37));



        let ten23 = U256::from(10u128.pow(23));
        assert_eq!(p_min(36), ten23 + U256::from(5_421u32), "10^23 · 2^64 / (2^64 − 1)");
        assert!(p_min(36) * U256::from(2u8) > cap, "36 is not a near miss: p_min is over half the cap");
        assert!(p_min(37) > cap * U256::from(6u8), "37 is not a near miss: p_min is over six times the cap");

        for d in 0..=255u8 {
            let some = some_launch_prices_at(d);
            assert_eq!(some, d <= 36, "some_launch_prices_at({d})");
            if d <= 40 {


                assert_eq!(some, p_min(d) < cap, "d={d}: the const derivation vs the oracle");
            }
            let got = quote_unit(d);
            if (MIN_QUOTE_DECIMALS..=MAX_QUOTE_DECIMALS).contains(&d) {
                assert_eq!(got, Some(10u128.pow(d as u32)), "10^{d}");
            } else {
                assert_eq!(got, None, "{d} is outside the window and must carry no unit");
            }
        }


        assert_eq!(quote_unit(0), Some(1), "a 0-decimal quote's whole unit IS one raw unit");
        assert_eq!(quote_unit(2), Some(100));


        assert_eq!(quote_unit(20), Some(10u128.pow(20)), "a whole unit past u64 is still a unit");
    }


    #[test]
    fn band_multiple_and_skim_defaults_are_the_engine_values() {
        assert_eq!(BID_BAND_TICKS, 120);
        assert_eq!((DEFAULT_SKIM_BUY_AT_FLOOR_BPS, DEFAULT_SKIM_SELL_AT_FLOOR_BPS), (0, 1000));
        assert!(DEFAULT_SKIM_SELL_AT_FLOOR_BPS <= crate::policy::MAX_SKIM_FAR_BPS as u64);
    }


    #[test]
    fn at_floor_sell_is_one_and_a_half_times_far_uncapped() {
        assert_eq!(SKIM_SELL_AT_FLOOR_MULTIPLE_BPS, 15_000);
        assert_eq!(skim_sell_at_floor_bps(0), 0);
        assert_eq!(skim_sell_at_floor_bps(150), 225);
        assert_eq!(skim_sell_at_floor_bps(200), 300);
        assert_eq!(skim_sell_at_floor_bps(crate::policy::MAX_SKIM_FAR_BPS), 1500);
        assert_eq!(skim_sell_at_floor_bps(1), 1);
        assert_eq!(skim_sell_at_floor_bps(333), 499);
    }
}
