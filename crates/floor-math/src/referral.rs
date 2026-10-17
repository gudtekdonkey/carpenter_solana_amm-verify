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
//!
//!
//!
//!
//!

use crate::state::FloorState;

///
///
pub const TIER_PASS_UP_DIVISOR: u64 = 10;

///
///
pub const REFERRAL_TIERS: u8 = 3;

const ZERO: [u8; 32] = [0u8; 32];

///
///
pub fn referral_tier_earned(lifetime: u64, t2: bool, t3: bool) -> [u64; 3] {
    let up2 = if t2 { lifetime / TIER_PASS_UP_DIVISOR } else { 0 };


    let up3 = if t2 && t3 { up2 / TIER_PASS_UP_DIVISOR } else { 0 };

    [lifetime - up2, up2 - up3, up3]
}

///
///
///
///
///
///
///
///
pub fn referral_tier_dues(s: &FloorState) -> [u64; 3] {
    let lifetime = s.referral_fees_paid as u128;
    let owed = s.referral_owed as u128;
    let paid2 = s.referral_paid_t2 as u128;
    let paid3 = s.referral_paid_t3 as u128;
    let Some(paid1) = owed
        .checked_add(paid2)
        .and_then(|x| x.checked_add(paid3))
        .and_then(|taken| lifetime.checked_sub(taken))
    else {
        return [0; 3];
    };
    let earned = referral_tier_earned(s.referral_fees_paid, s.referrer_t2 != ZERO, s.referrer_t3 != ZERO);
    let paid = [paid1, paid2, paid3];
    let mut due = [0u64; 3];
    for i in 0..3 {
        let d = (earned[i] as u128).saturating_sub(paid[i]).min(owed);

        due[i] = d as u64;
    }
    due
}

///
pub fn referral_tier_due(s: &FloorState, tier: u8) -> Option<u64> {
    match tier {
        1..=REFERRAL_TIERS => Some(referral_tier_dues(s)[(tier - 1) as usize]),
        _ => None,
    }
}

///
///
pub fn referral_tier_wallet(s: &FloorState, tier: u8) -> Option<[u8; 32]> {
    match tier {
        1 => Some(s.referrer),
        2 => Some(s.referrer_t2),
        3 => Some(s.referrer_t3),
        _ => None,
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    const W1: [u8; 32] = [1; 32];
    const W2: [u8; 32] = [2; 32];
    const W3: [u8; 32] = [3; 32];

    fn pool(tiers: u8, booked: u64) -> FloorState {
        let mut s = FloorState::default();
        s.referrer = W1;
        if tiers >= 2 {
            s.referrer_t2 = W2;
        }
        if tiers >= 3 {
            s.referrer_t3 = W3;
        }
        s.referral_fees_paid = booked;
        s.referral_owed = booked;
        s
    }

///
///
    fn claim(s: &mut FloorState, tier: u8) -> u64 {
        let due = referral_tier_due(s, tier).unwrap();
        s.referral_owed = s.referral_owed.checked_sub(due).unwrap();
        match tier {
            2 => s.referral_paid_t2 += due,
            3 => s.referral_paid_t3 += due,
            _ => {}
        }
        due
    }

///
///
///
///
    #[test]
    fn the_spec_table_one_two_and_three_tiers() {
        assert_eq!(referral_tier_dues(&pool(1, 500)), [500, 0, 0]);
        assert_eq!(referral_tier_dues(&pool(2, 500)), [450, 50, 0]);
        assert_eq!(referral_tier_dues(&pool(3, 500)), [450, 45, 5]);

        assert_eq!(referral_tier_dues(&pool(3, 1_000)), [900, 90, 10]);

        assert_eq!(referral_tier_dues(&FloorState::default()), [0, 0, 0]);
    }

///
///
///
///
///
    #[test]
    fn exact_and_monotone_over_a_range() {
        for (t2, t3) in [(false, false), (true, false), (true, true)] {
            let mut prev = [0u64; 3];
            for l in 0..200_000u64 {
                let e = referral_tier_earned(l, t2, t3);
                assert_eq!(e[0] + e[1] + e[2], l, "exact at L={l}");
                for i in 0..3 {
                    assert!(e[i] >= prev[i], "earned[{i}] fell at L={l} ({t2},{t3})");
                }
                prev = e;
            }
        }

        let e = referral_tier_earned(u64::MAX, true, true);
        assert_eq!(e[0] as u128 + e[1] as u128 + e[2] as u128, u64::MAX as u128);
    }

///
    #[test]
    fn a_gap_pays_tier_three_nothing() {
        let mut s = pool(1, 500);
        s.referrer_t3 = W3;
        assert_eq!(referral_tier_dues(&s), [500, 0, 0]);
    }

///
///
///
///
    #[test]
    fn a_pre_upgrade_pool_that_already_claimed_pays_exactly_owed() {
        let mut s = FloorState::default();
        s.referrer = W1;
        s.referral_fees_paid = 98_235 + 1_234;
        s.referral_owed = 1_234;
        assert_eq!(referral_tier_dues(&s), [1_234, 0, 0]);
        assert_eq!(claim(&mut s, 1), 1_234);
        assert_eq!(s.referral_owed, 0);
        assert_eq!(referral_tier_dues(&s), [0, 0, 0]);
    }

///
///
///
///
///
    #[test]
    fn claim_order_does_not_matter_and_the_meters_conserve() {
        let orders: [[u8; 3]; 6] = [[1, 2, 3], [1, 3, 2], [2, 1, 3], [2, 3, 1], [3, 1, 2], [3, 2, 1]];
        let accruals = [7u64, 93, 1, 400, 0, 12_345, 9, 10, 11];
        let mut totals: Vec<[u64; 3]> = Vec::new();
        for order in orders {
            let mut s = pool(3, 0);
            let mut got = [0u64; 3];
            for (k, a) in accruals.iter().enumerate() {

                s.referral_fees_paid += a;
                s.referral_owed += a;

                for &t in &order[..(k % 3) + 1] {
                    got[(t - 1) as usize] += claim(&mut s, t);
                    let paid1 = s.referral_fees_paid - s.referral_owed - s.referral_paid_t2 - s.referral_paid_t3;
                    assert_eq!(paid1 + s.referral_paid_t2 + s.referral_paid_t3, s.referral_fees_paid - s.referral_owed);

                    assert_eq!(referral_tier_due(&s, t), Some(0));
                }
            }
            for t in 1..=3 {
                got[(t - 1) as usize] += claim(&mut s, t);
            }
            assert_eq!(s.referral_owed, 0);
            assert_eq!(got.iter().sum::<u64>(), s.referral_fees_paid);
            assert_eq!(got, referral_tier_earned(s.referral_fees_paid, true, true));
            totals.push(got);
        }
        assert!(totals.windows(2).all(|w| w[0] == w[1]), "{totals:?}");
    }

///
    #[test]
    fn dues_are_capped_and_a_corrupt_pool_pays_nothing() {
        let mut s = pool(3, 1_000);
        let d = referral_tier_dues(&s);
        assert!(d.iter().all(|&x| x <= s.referral_owed));
        assert_eq!(d.iter().sum::<u64>(), s.referral_owed);
        s.referral_paid_t2 = 2_000;
        assert_eq!(referral_tier_dues(&s), [0, 0, 0]);
        let mut s = pool(3, u64::MAX);
        s.referral_paid_t2 = u64::MAX;
        s.referral_paid_t3 = u64::MAX;
        assert_eq!(referral_tier_dues(&s), [0, 0, 0], "no overflow in the u128 sum");
    }

    #[test]
    fn tier_numbers_outside_one_to_three_are_none() {
        let s = pool(3, 500);
        assert_eq!(referral_tier_due(&s, 0), None);
        assert_eq!(referral_tier_due(&s, 4), None);
        assert_eq!(referral_tier_wallet(&s, 0), None);
        assert_eq!(referral_tier_wallet(&s, 2), Some(W2));
        assert_eq!(referral_tier_wallet(&s, 3), Some(W3));
    }
}
