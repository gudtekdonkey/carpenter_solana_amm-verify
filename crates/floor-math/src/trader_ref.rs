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

///
pub const TRADER_REF_T1_BPS: u64 = 3_000;
///
pub const TRADER_REF_T2_BPS: u64 = 500;
///
pub const TRADER_REF_T3_BPS: u64 = 200;
///
pub const TRADER_REF_TIER_BPS: [u64; 3] = [TRADER_REF_T1_BPS, TRADER_REF_T2_BPS, TRADER_REF_T3_BPS];
///
pub const TRADER_REF_BPS_DENOM: u64 = 10_000;


const _: () = assert!(TRADER_REF_T1_BPS + TRADER_REF_T2_BPS + TRADER_REF_T3_BPS == 3_700);
const _: () = assert!(TRADER_REF_T1_BPS + TRADER_REF_T2_BPS + TRADER_REF_T3_BPS < TRADER_REF_BPS_DENOM);

///
pub fn trader_ref_tiers_counting(set: [bool; 3]) -> [bool; 3] {
    let t1 = set[0];
    let t2 = t1 && set[1];
    let t3 = t2 && set[2];
    [t1, t2, t3]
}

///
///
///
pub fn trader_ref_slices(tfee: u64, set: [bool; 3]) -> [u64; 3] {
    let counting = trader_ref_tiers_counting(set);
    let mut out = [0u64; 3];
    for i in 0..3 {
        if counting[i] {

            out[i] = ((tfee as u128 * TRADER_REF_TIER_BPS[i] as u128) / TRADER_REF_BPS_DENOM as u128) as u64;
        }
    }
    out
}

///
///
pub fn trader_ref_floor_keeps(tfee: u64, slices: [u64; 3]) -> Option<u64> {
    tfee.checked_sub(slices[0])?.checked_sub(slices[1])?.checked_sub(slices[2])
}

///
///
pub fn trader_link_tiers_well_formed(trader: &[u8; 32], tiers: [&[u8; 32]; 3]) -> bool {
    const ZERO: [u8; 32] = [0u8; 32];
    let set = [tiers[0] != &ZERO, tiers[1] != &ZERO, tiers[2] != &ZERO];
    if !set[0] || (set[1] && !set[0]) || (set[2] && !set[1]) {
        return false;
    }
    if trader == &ZERO {
        return false;
    }
    for i in 0..3 {
        if set[i] && tiers[i] == trader {
            return false;
        }
        for j in (i + 1)..3 {
            if set[i] && set[j] && tiers[i] == tiers[j] {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [bool; 3] = [true, true, true];

///
    #[test]
    fn the_rates_are_the_owners_thirty_five_two() {
        assert_eq!(TRADER_REF_T1_BPS * 100 / TRADER_REF_BPS_DENOM, 30);
        assert_eq!(TRADER_REF_T2_BPS * 100 / TRADER_REF_BPS_DENOM, 5);
        assert_eq!(TRADER_REF_T3_BPS * 100 / TRADER_REF_BPS_DENOM, 2);

        let s = trader_ref_slices(1_000_000, ALL);
        assert_eq!(s, [300_000, 50_000, 20_000]);
        assert_eq!(trader_ref_floor_keeps(1_000_000, s), Some(630_000));
    }

    #[test]
    fn each_slice_is_floored_on_its_own_and_floor_keeps_the_dust() {

        let s = trader_ref_slices(99, ALL);
        assert_eq!(s, [29, 4, 1]);
        assert_eq!(trader_ref_floor_keeps(99, s), Some(65));

        assert_eq!(trader_ref_slices(1, ALL), [0, 0, 0]);
        assert_eq!(trader_ref_slices(0, ALL), [0, 0, 0]);

        assert_eq!(trader_ref_slices(34, ALL), [10, 1, 0]);
    }

    #[test]
    fn an_absent_tier_stays_with_floor_and_a_gap_stops_the_chain() {
        assert_eq!(trader_ref_slices(10_000, [true, false, false]), [3_000, 0, 0]);
        assert_eq!(trader_ref_slices(10_000, [true, true, false]), [3_000, 500, 0]);

        assert_eq!(trader_ref_slices(10_000, [true, false, true]), [3_000, 0, 0]);
        assert_eq!(trader_ref_slices(10_000, [false, true, true]), [0, 0, 0]);
        assert_eq!(trader_ref_slices(10_000, [false, false, false]), [0, 0, 0]);
    }

    #[test]
    fn the_slices_never_exceed_the_fee_and_never_overflow() {
        for tfee in [u64::MAX, u64::MAX - 1, u64::MAX / 3, 1 << 63, 12_345_678_901_234_567] {
            let s = trader_ref_slices(tfee, ALL);
            let sum = s[0] as u128 + s[1] as u128 + s[2] as u128;
            assert!(sum <= tfee as u128 * 37 / 100, "≤ 37 % of the fee");
            assert!(trader_ref_floor_keeps(tfee, s).is_some());
            assert_eq!(s[0] as u128, tfee as u128 * 3_000 / 10_000);
        }

        let mut prev = [0u64; 3];
        for tfee in 0..20_000u64 {
            let s = trader_ref_slices(tfee, ALL);
            for i in 0..3 {
                assert!(s[i] >= prev[i]);
            }
            prev = s;
        }
    }

    #[test]
    fn the_link_shape_refuses_gaps_self_and_repeats() {
        let t = [9u8; 32];
        let (a, b, c, z) = ([1u8; 32], [2u8; 32], [3u8; 32], [0u8; 32]);
        assert!(trader_link_tiers_well_formed(&t, [&a, &z, &z]));
        assert!(trader_link_tiers_well_formed(&t, [&a, &b, &z]));
        assert!(trader_link_tiers_well_formed(&t, [&a, &b, &c]));

        assert!(!trader_link_tiers_well_formed(&t, [&z, &z, &z]));
        assert!(!trader_link_tiers_well_formed(&t, [&z, &b, &z]));
        assert!(!trader_link_tiers_well_formed(&t, [&a, &z, &c]));

        assert!(!trader_link_tiers_well_formed(&t, [&t, &z, &z]));
        assert!(!trader_link_tiers_well_formed(&t, [&a, &t, &z]));
        assert!(!trader_link_tiers_well_formed(&t, [&a, &b, &t]));

        assert!(!trader_link_tiers_well_formed(&t, [&a, &a, &z]));
        assert!(!trader_link_tiers_well_formed(&t, [&a, &b, &a]));
        assert!(!trader_link_tiers_well_formed(&t, [&a, &b, &b]));

        assert!(!trader_link_tiers_well_formed(&z, [&a, &z, &z]));
    }
}
