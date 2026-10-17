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
//!
//!
//!
//!
//!
//!
//!
//!
//!

use crate::constants::{
    MAX_PROTOCOL_FEE, PIPS_DENOMINATOR, PROTOCOL_FEE_ON_CURRENCY0,
    PROTOCOL_FEE_ON_CURRENCY1, PROTOCOL_FEE_ON_INPUT,
};

///
///
pub const fn directional(packed: u32, zero_for_one: bool) -> u32 {
    if zero_for_one {
        packed & 0xfff
    } else {
        (packed >> 12) & 0xfff
    }
}

///
///
pub const fn pack(zero_for_one: u32, one_for_zero: u32) -> u32 {
    (zero_for_one & 0xfff) | ((one_for_zero & 0xfff) << 12)
}

///
///
pub const fn is_valid(packed: u32) -> bool {
    packed >> 24 == 0 && directional(packed, true) <= MAX_PROTOCOL_FEE && directional(packed, false) <= MAX_PROTOCOL_FEE
}

///
///
///
///
pub const fn compose_swap_fee(protocol_fee: u32, lp_fee: u32) -> u32 {
    let p = (protocol_fee & 0xfff) as u64;
    let l = (lp_fee & 0x00ff_ffff) as u64;
    (p + l - (p * l) / PIPS_DENOMINATOR as u64) as u32
}

///
///
///
pub const fn fee_currency(protocol_fee_currency: u8, zero_for_one: bool) -> u8 {
    if protocol_fee_currency == PROTOCOL_FEE_ON_CURRENCY0 {
        0
    } else if protocol_fee_currency == PROTOCOL_FEE_ON_CURRENCY1 {
        1
    } else if zero_for_one {
        0
    } else {
        1
    }
}

///
pub const fn on_output(protocol_fee_currency: u8, zero_for_one: bool) -> bool {
    let input = if zero_for_one { 0 } else { 1 };
    fee_currency(protocol_fee_currency, zero_for_one) != input
}

///
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WalkFee {
///
///
    pub swap_fee: u32,
///
    pub input_pips: u32,
///
///
    pub output_pips: u32,
///
    pub currency: u8,
}

///
///
pub const fn walk_fee(packed: u32, protocol_fee_currency: u8, zero_for_one: bool, lp_fee: u32) -> WalkFee {
    let p = directional(packed, zero_for_one);
    let currency = fee_currency(protocol_fee_currency, zero_for_one);
    if p == 0 {
        WalkFee { swap_fee: lp_fee, input_pips: 0, output_pips: 0, currency }
    } else if on_output(protocol_fee_currency, zero_for_one) {
        WalkFee { swap_fee: lp_fee, input_pips: 0, output_pips: p, currency }
    } else {
        WalkFee { swap_fee: compose_swap_fee(p, lp_fee), input_pips: p, output_pips: 0, currency }
    }
}

///
///
///
///
///
///
pub const fn input_share(step_fee: u64, gross_in: u64, swap_fee: u32, protocol_pips: u32) -> Option<u64> {
    if protocol_pips == 0 {
        return Some(0);
    }
    let share = if swap_fee == protocol_pips {
        step_fee
    } else {
        ((gross_in as u128) * (protocol_pips as u128) / PIPS_DENOMINATOR as u128) as u64
    };
    if share <= step_fee {
        Some(share)
    } else {
        None
    }
}

///
///
pub const fn output_share(gross_out: u64, protocol_pips: u32) -> u64 {
    ((gross_out as u128) * (protocol_pips as u128) / PIPS_DENOMINATOR as u128) as u64
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
pub const fn gross_for_net(net: u64, protocol_pips: u32) -> Option<u64> {
    if protocol_pips == 0 || net == 0 {
        return Some(net);
    }
    let p = protocol_pips as u128;
    if p >= PIPS_DENOMINATOR as u128 {
        return None;
    }
    let d = PIPS_DENOMINATOR as u128;
    let n = net as u128;

    let mut g = (n * d + (d - p) - 1) / (d - p);
    let mut i = 0;
    while i < 4 && g > n {
        let below = g - 1;
        if below - below * p / d >= n {
            g = below;
        } else {
            break;
        }
        i += 1;
    }
    if g > u64::MAX as u128 {
        None
    } else {
        Some(g as u64)
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
pub const fn currency_for_roles(listed_quote0: bool, listed_quote1: bool, wsol0: bool, wsol1: bool) -> u8 {
    match (wsol0, wsol1) {
        (true, false) => return PROTOCOL_FEE_ON_CURRENCY0,
        (false, true) => return PROTOCOL_FEE_ON_CURRENCY1,
        _ => {}
    }
    match (listed_quote0, listed_quote1) {
        (true, false) => PROTOCOL_FEE_ON_CURRENCY0,
        (false, true) => PROTOCOL_FEE_ON_CURRENCY1,
        _ => PROTOCOL_FEE_ON_INPUT,
    }
}

///
///
///
///
///
///
///
pub const fn rate_for_new_pool(current: u32, recorded0: Option<u32>, recorded1: Option<u32>) -> u32 {
    match (recorded0, recorded1) {
        (None, None) => current,
        (Some(r), None) | (None, Some(r)) => r,
        (Some(a), Some(b)) => {
            let (a0, b0) = (directional(a, true), directional(b, true));
            let (a1, b1) = (directional(a, false), directional(b, false));
            pack(if a0 < b0 { a0 } else { b0 }, if a1 < b1 { a1 } else { b1 })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_recorded_rate_wins_over_the_current_setting() {
        let now = pack(1_000, 1_000);
        assert_eq!(rate_for_new_pool(now, None, None), now, "no record: today's setting (a direct pool)");
        assert_eq!(rate_for_new_pool(now, None, Some(pack(500, 500))), pack(500, 500), "a launch's mint: its creation's rate");
        assert_eq!(rate_for_new_pool(now, Some(0), None), 0, "a recorded 0 is a rate, not an absence");
        assert_eq!(rate_for_new_pool(0, None, Some(pack(700, 300))), pack(700, 300), "a fall is not taken either");
        assert_eq!(rate_for_new_pool(now, Some(pack(200, 900)), Some(pack(600, 100))), pack(200, 100), "both: the lower per direction");
    }
    use crate::constants::{PROTOCOL_FEE_DEFAULT, PROTOCOL_FEE_DEFAULT_PACKED, PROTOCOL_FEE_ON_CURRENCY0, PROTOCOL_FEE_ON_CURRENCY1};

    #[test]
    fn the_default_is_500_both_ways_and_within_v4s_max() {
        assert_eq!(PROTOCOL_FEE_DEFAULT, 500);
        assert_eq!(MAX_PROTOCOL_FEE, 1_000, "v4 ProtocolFeeLibrary.MAX_PROTOCOL_FEE");
        assert_eq!(PROTOCOL_FEE_DEFAULT_PACKED, 2_048_500);
        assert_eq!(pack(500, 500), PROTOCOL_FEE_DEFAULT_PACKED);
        assert_eq!(directional(PROTOCOL_FEE_DEFAULT_PACKED, true), 500);
        assert_eq!(directional(PROTOCOL_FEE_DEFAULT_PACKED, false), 500);
        assert!(is_valid(PROTOCOL_FEE_DEFAULT_PACKED));

        assert_eq!(compose_swap_fee(500, 2_500), 2_999);
        assert_eq!(compose_swap_fee(500, 0), 500, "no LP fee: the composed rate IS the protocol's");
        assert_eq!(compose_swap_fee(0, 2_500), 2_500);
    }

    #[test]
    fn the_cap_is_inclusive_and_nothing_above_the_nibbles_is_valid() {
        assert!(is_valid(pack(1_000, 1_000)));
        assert!(is_valid(pack(0, 0)), "0 is a setting");
        assert!(is_valid(pack(1_000, 0)));
        assert!(!is_valid(pack(1_001, 0)));
        assert!(!is_valid(pack(0, 1_001)));
        assert!(!is_valid(pack(4_095, 4_095)));
        assert!(!is_valid(1 << 24), "a bit above the two nibbles");
    }

    #[test]
    fn v4s_nibble_order() {
        let packed = pack(1_000, 2_000);
        assert_eq!(packed & 0xfff, 1_000, "getZeroForOneFee = self & 0xfff");
        assert_eq!((packed >> 12) & 0xfff, 2_000, "getOneForZeroFee = self >> 12");
        assert_eq!(directional(packed, true), 1_000);
        assert_eq!(directional(packed, false), 2_000);
    }

    #[test]
    fn the_quote_side_rule_and_the_input_fallback() {

        assert_eq!(currency_for_roles(true, false, false, false), PROTOCOL_FEE_ON_CURRENCY0);
        assert_eq!(fee_currency(PROTOCOL_FEE_ON_CURRENCY0, true), 0, "a buy pays in the quote it puts in");
        assert_eq!(fee_currency(PROTOCOL_FEE_ON_CURRENCY0, false), 0, "a sell pays in the quote it receives");
        assert!(!on_output(PROTOCOL_FEE_ON_CURRENCY0, true));
        assert!(on_output(PROTOCOL_FEE_ON_CURRENCY0, false));

        assert_eq!(currency_for_roles(false, true, false, false), PROTOCOL_FEE_ON_CURRENCY1);
        assert_eq!(fee_currency(PROTOCOL_FEE_ON_CURRENCY1, true), 1);
        assert_eq!(fee_currency(PROTOCOL_FEE_ON_CURRENCY1, false), 1);
        assert!(on_output(PROTOCOL_FEE_ON_CURRENCY1, true));
        assert!(!on_output(PROTOCOL_FEE_ON_CURRENCY1, false));

        for code in [currency_for_roles(false, false, false, false), currency_for_roles(true, true, false, false), PROTOCOL_FEE_ON_INPUT, 7] {
            assert_eq!(fee_currency(code, true), 0);
            assert_eq!(fee_currency(code, false), 1);
            assert!(!on_output(code, true) && !on_output(code, false));
        }
    }

///
///
    #[test]
    fn a_pool_with_a_wsol_side_takes_the_fee_in_sol_even_when_both_sides_are_listed() {

        let c = currency_for_roles(true, true, true, false);
        assert_eq!(c, PROTOCOL_FEE_ON_CURRENCY0);
        assert_eq!(fee_currency(c, true), 0, "a route buy (SOL in) pays in SOL, off the input");
        assert_eq!(fee_currency(c, false), 0, "a route sell (stock in) pays in SOL, off the output");
        assert!(!on_output(c, true) && on_output(c, false));

        let c = currency_for_roles(true, true, false, true);
        assert_eq!(c, PROTOCOL_FEE_ON_CURRENCY1);
        assert_eq!((fee_currency(c, true), fee_currency(c, false)), (1, 1));

        assert_eq!(currency_for_roles(true, false, true, false), currency_for_roles(true, false, false, false));
        assert_eq!(currency_for_roles(false, true, false, true), currency_for_roles(false, true, false, false));

        for bits in 0u8..16 {
            let (l0, l1, w0, w1) = (bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
            let want = if w0 != w1 {
                if w0 {
                    PROTOCOL_FEE_ON_CURRENCY0
                } else {
                    PROTOCOL_FEE_ON_CURRENCY1
                }
            } else if l0 != l1 {
                if l0 {
                    PROTOCOL_FEE_ON_CURRENCY0
                } else {
                    PROTOCOL_FEE_ON_CURRENCY1
                }
            } else {
                PROTOCOL_FEE_ON_INPUT
            };
            assert_eq!(currency_for_roles(l0, l1, w0, w1), want, "listed ({l0}, {l1}) wsol ({w0}, {w1})");
        }
    }

    #[test]
    fn walk_fee_composes_only_on_the_input() {
        let d = PROTOCOL_FEE_DEFAULT_PACKED;

        let buy = walk_fee(d, PROTOCOL_FEE_ON_CURRENCY0, true, 2_500);
        assert_eq!(buy, WalkFee { swap_fee: 2_999, input_pips: 500, output_pips: 0, currency: 0 });

        let sell = walk_fee(d, PROTOCOL_FEE_ON_CURRENCY0, false, 2_500);
        assert_eq!(sell, WalkFee { swap_fee: 2_500, input_pips: 0, output_pips: 500, currency: 0 });

        let old = walk_fee(0, PROTOCOL_FEE_ON_INPUT, false, 3_000);
        assert_eq!(old, WalkFee { swap_fee: 3_000, input_pips: 0, output_pips: 0, currency: 1 });

        assert_eq!(walk_fee(d, PROTOCOL_FEE_ON_INPUT, false, 3_000).input_pips, 500);
        assert_eq!(walk_fee(d, PROTOCOL_FEE_ON_INPUT, false, 3_000).currency, 1);
    }

    #[test]
    fn input_share_is_v4s_split() {

        assert_eq!(input_share(37, 1_000_000, 500, 500), Some(37));

        assert_eq!(input_share(2_999, 1_000_000, 2_999, 500), Some(500));
        assert_eq!(input_share(5, 1_999, 2_999, 500), Some(0), "under 2,000 units in, the share floors to 0");

        assert_eq!(input_share(2_500, 1_000_000, 2_500, 0), Some(0));

        assert_eq!(input_share(1, 1_000_000, 2_999, 500), None);
    }

    #[test]
    fn output_share_floors() {
        assert_eq!(output_share(1_000_000, 500), 500);
        assert_eq!(output_share(1_999, 500), 0);
        assert_eq!(output_share(2_000, 500), 1);
        assert_eq!(output_share(u64::MAX, 1_000), ((u64::MAX as u128) * 1_000 / 1_000_000) as u64);
        assert_eq!(output_share(123, 0), 0);
    }

///
///
    #[test]
    fn gross_for_net_is_the_least_gross_and_nets_exactly() {
        let f = |g: u64, p: u32| g - output_share(g, p);
        for p in (0..=0xfffu32).step_by(7).chain([500, 1_000, 0xfff]) {
            for net in (0..3_000u64).chain([999_999, 1_000_000, 1_000_001, 123_456_789]) {
                let g = gross_for_net(net, p).expect("fits");
                assert_eq!(f(g, p), net, "p={p} net={net}: nets exactly");
                if g > 0 {
                    assert!(f(g - 1, p) < net, "p={p} net={net}: the least such gross");
                }
            }
        }

        assert_eq!(gross_for_net(u64::MAX, 500), None);
        assert_eq!(gross_for_net(u64::MAX, 0), Some(u64::MAX));
    }

}
