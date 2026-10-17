//!

use carpenter_clmm_math::big_num::U256;

use crate::error::{FloorMathError, FloorMathResult};

///
///
///
///
///
///
///
pub fn sale_tokens_for(lamports: u64, price_x64: u128, remaining: u64) -> FloorMathResult<(u64, u64)> {
    if price_x64 == 0 {
        return Err(FloorMathError::ZeroDenominator);
    }
    let price = U256::from(price_x64);
    let affordable = (U256::from(lamports) << 64usize) / price;
    let tokens = if affordable < U256::from(remaining) { affordable.as_u64() } else { remaining };
    let prod = U256::from(tokens) * price;
    let mut cost = prod >> 64usize;
    if !(prod & U256::from(u64::MAX)).is_zero() {
        cost += U256::one();
    }
    if cost > U256::from(u64::MAX) {
        return Err(FloorMathError::Overflow);
    }
    Ok((tokens, cost.as_u64()))
}



use crate::constants::BPS;
use crate::fees::part;

///
pub fn team_net_of(gross: u64, team_bps: u16) -> u64 {
    (gross as u128 * BPS as u128 / (BPS as u128 + team_bps as u128)) as u64
}
///
pub fn team_fee_of(amount: u64, team_bps: u16) -> FloorMathResult<u64> {
    part(amount, team_bps as u64)
}
///
pub fn team_gross_for(net: u64, team_bps: u16) -> FloorMathResult<u64> {
    u64::try_from((net as u128 * (BPS as u128 + team_bps as u128)).div_ceil(BPS as u128)).map_err(|_| FloorMathError::Overflow)
}
///
pub fn sale_buy_with_team(gross: u64, price_x64: u128, remaining: u64, team_bps: u16) -> FloorMathResult<(u64, u64, u64)> {
    let (tokens, cost) = sale_tokens_for(team_net_of(gross, team_bps), price_x64, remaining)?;
    Ok((tokens, cost, team_fee_of(cost, team_bps)?))
}
///
///
///
///
///
///
pub fn sell_back_payout_at(tokens: u64, price_x64: u128) -> FloorMathResult<u64> {
    if price_x64 == 0 {
        return Err(FloorMathError::ZeroDenominator);
    }
    let p = (U256::from(tokens) * U256::from(price_x64)) >> 64usize;
    if p > U256::from(u64::MAX) {
        return Err(FloorMathError::Overflow);
    }
    Ok(p.as_u64())
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
pub fn sell_back_settle(contributed: u64, allocation: u64, tokens: u64, price_x64: u128) -> FloorMathResult<(u64, u64, u64)> {
    let allocation_after = allocation.checked_sub(tokens).ok_or(FloorMathError::Overflow)?;
    let payout = sell_back_payout_at(tokens, price_x64)?;
    let contributed_after = contributed.checked_sub(payout).ok_or(FloorMathError::Overflow)?;
    Ok((payout, contributed_after, allocation_after))
}

///
pub fn sell_back_with_team(payout: u64, team_bps: u16) -> FloorMathResult<(u64, u64)> {
    let fee = team_fee_of(payout, team_bps)?;
    Ok((fee, payout.checked_sub(fee).ok_or(FloorMathError::Overflow)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::SALE;
    use crate::modes::tranche_price_x64;

    #[test]
    fn cost_rounds_up_and_never_exceeds_the_payment() {
        let p = tranche_price_x64(5_000, 100 * 100, 9).unwrap();
        let (t, c) = sale_tokens_for(1_000_000_000, p, SALE).unwrap();
        assert_eq!(t as u128, (1_000_000_000u128 << 64) / p);
        assert!(c <= 1_000_000_000);
        assert!(c as u128 * (1 << 64) >= t as u128 * p, "ceil");
        assert!((c as u128 - 1) * (1 << 64) < t as u128 * p, "tight");

        let (t, c) = sale_tokens_for(u64::MAX, p, 7).unwrap();
        assert_eq!((t, c), (7, 1));

        assert_eq!(sale_tokens_for(0, p, SALE).unwrap(), (0, 0));
        assert_eq!(sale_tokens_for(1, 1u128 << 76, SALE).unwrap(), (0, 0));
        assert_eq!(sale_tokens_for(1, 0, SALE), Err(FloorMathError::ZeroDenominator));
    }


    #[test]
    fn paying_the_ceiling_of_the_remainder_completes_the_sale() {
        for (mcap, sol) in [(5_000u64, 100u64), (20_000, 250), (100_000, 17), (100_000, 1)] {
            let p = tranche_price_x64(mcap, sol * 100, 9).unwrap();
            for remaining in [1u64, 19_999, 12_345_678_901, SALE / 3, SALE] {
                let need = ((remaining as u128 * p) + (1u128 << 64) - 1) >> 64;
                let (t, c) = sale_tokens_for(need as u64, p, remaining).unwrap();
                assert_eq!(t, remaining, "{mcap}@{sol} remaining {remaining}");
                assert_eq!(c as u128, need);
            }
        }
    }

///
    #[test]
    fn the_team_gross_up_and_its_inverse_at_25_bps() {
        assert_eq!([team_net_of(1_002_500_000, 25), team_net_of(401, 25), team_net_of(400, 25), team_net_of(1, 25)], [1_000_000_000, 400, 399, 0]);
        assert_eq!((team_fee_of(399, 25).unwrap(), team_fee_of(400, 25).unwrap()), (0, 1));
        assert_eq!([team_gross_for(1_000_000_000, 25).unwrap(), team_gross_for(399, 25).unwrap(), team_gross_for(1, 25).unwrap()], [1_002_500_000, 400, 2]);
        for c in [0u64, 1, 399, 400, 1_000_000_007, 5_000_000_000_000] {
            let g = team_gross_for(c, 25).unwrap();
            assert_eq!(team_net_of(g, 25), c, "net_of(gross_for(c)) == c");
            assert!(g - c - team_fee_of(c, 25).unwrap() <= 1);
            assert_eq!((team_net_of(c, 0), team_fee_of(c, 0).unwrap()), (c, 0));
        }
        assert_eq!(team_net_of(u64::MAX, 25), (u64::MAX as u128 * 10_000 / 10_025) as u64);
    }

    #[test]
    fn a_sale_buy_never_spends_more_than_the_gross_cap() {
        for (mcap, sol) in [(5_000u64, 100u64), (20_000, 150), (100_000, 17)] {
            let p = tranche_price_x64(mcap, sol * 100, 9).unwrap();
            for gross in [1u64, 2, 399, 400, 401, 1_000_000_007, 7_777_777_777, 50_000_000_000] {
                let (t, cost, fee) = sale_buy_with_team(gross, p, SALE, 25).unwrap();
                assert!(cost + fee <= gross);
                assert_eq!(fee as u128, cost as u128 * 25 / 10_000);
                assert_eq!((t, cost), sale_tokens_for(team_net_of(gross, 25), p, SALE).unwrap(), "the walk runs on the NET");
            }
        }
    }

    #[test]
    fn paying_the_gross_of_the_remainders_ceiling_completes_the_sale() {
        for (mcap, sol) in [(5_000u64, 100u64), (20_000, 250), (100_000, 17)] {
            let p = tranche_price_x64(mcap, sol * 100, 9).unwrap();
            for remaining in [1u64, 19_999, 12_345_678_901, SALE / 3, SALE] {
                let need = (((remaining as u128 * p) + (1u128 << 64) - 1) >> 64) as u64;
                let (t, cost, fee) = sale_buy_with_team(team_gross_for(need, 25).unwrap(), p, remaining, 25).unwrap();
                assert_eq!((t, cost, fee), (remaining, need, team_fee_of(need, 25).unwrap()));
            }
        }
    }

///
///
    #[test]
    fn the_stateless_payout_is_the_floored_cost_at_the_sale_price() {
        for (mcap, sol) in [(5_000u64, 100u64), (25_000, 150), (100_000, 17), (100_000, 1)] {
            let p = tranche_price_x64(mcap, sol * 100, 9).unwrap();
            for t in [0u64, 1, 2, 7, 19_999, 12_345_678_901, SALE / 3, SALE] {
                let exact = (t as u128 * p) >> 64;
                assert_eq!(sell_back_payout_at(t, p).unwrap() as u128, exact);
                let (bt, cost) = sale_tokens_for(u64::MAX, p, t).unwrap();
                assert_eq!(bt, t);
                assert!(cost - sell_back_payout_at(t, p).unwrap() <= 1, "a round trip at the sale price loses <= 1 lamport");
            }
        }
        assert_eq!(sell_back_payout_at(5, 0), Err(FloorMathError::ZeroDenominator));
        assert_eq!(sell_back_payout_at(u64::MAX, u128::MAX), Err(FloorMathError::Overflow));
        assert_eq!(sell_back_settle(10, 5, 6, 1u128 << 64), Err(FloorMathError::Overflow), "more than the allocation");
        assert_eq!(sell_back_settle(4, 5, 5, 1u128 << 64), Err(FloorMathError::Overflow), "a position breaking (I)");
        assert_eq!(sell_back_settle(7, 5, 2, 1u128 << 64).unwrap(), (2, 5, 3));
    }

///
///
///
///
///
    #[test]
    fn the_stateless_sell_back_keeps_the_escrow_solvent_over_random_sequences() {
        let mut seed = 0x9E37_79B9_7F4A_7C15u64;
        let mut rnd = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        let mut checked = 0u64;
        for round in 0..400 {
            let p = tranche_price_x64(5_000 + rnd() % 400_000, 100 + rnd() % 30_000, 9).unwrap();
            let sale = SALE / (1 + rnd() % 8);
            let wallets = 1 + (rnd() % 6) as usize;
            let mut pos = vec![(0u64, 0u64); wallets];
            let (mut sold, mut raised, mut ops) = (0u64, 0u64, 0u64);
            for _ in 0..(5 + rnd() % 60) {
                let w = (rnd() % wallets as u64) as usize;
                if rnd() % 2 == 0 {
                    let lamports = match rnd() % 4 { 0 => rnd() % 1_000, 1 => rnd() % 1_000_000, _ => rnd() % 50_000_000_000 };
                    let (t, cost) = sale_tokens_for(lamports, p, sale - sold).unwrap();
                    if t == 0 || sold + t == sale {
                        continue;
                    }
                    pos[w] = (pos[w].0 + cost, pos[w].1 + t);
                    (sold, raised, ops) = (sold + t, raised + cost, ops + 1);
                } else if pos[w].1 > 0 {
                    let (c, a) = pos[w];
                    let t = match rnd() % 3 { 0 => a, 1 => 1 + rnd() % a, _ => (a / 2).max(1) };
                    let old = (c as u128 * t as u128 / a as u128) as u64;
                    let (payout, c2, a2) = sell_back_settle(c, a, t, p).unwrap();
                    assert!(payout <= old, "round {round}: stateless {payout} above the old rule's {old}");
                    assert!(old - payout <= ops, "the old rule's excess is bounded by the position's history");
                    pos[w] = (c2, a2);
                    (sold, raised, ops) = (sold - t, raised - payout, ops + 1);
                    checked += 1;
                }
                for &(c, a) in &pos {
                    assert!((c as u128) << 64 >= a as u128 * p, "round {round}: (I) broken");
                }
                assert_eq!(pos.iter().map(|x| x.0).sum::<u64>(), raised, "Σ contributed == raised (no position closed here)");
                assert_eq!(pos.iter().map(|x| x.1).sum::<u64>(), sold);
            }

            for w in 0..wallets {
                let (c, a) = pos[w];
                if a == 0 {
                    continue;
                }
                let (payout, c2, _) = sell_back_settle(c, a, a, p).unwrap();
                assert!(payout <= raised);
                raised -= payout;
                pos[w] = (c2, 0);
            }
            assert_eq!(raised, pos.iter().map(|x| x.0).sum::<u64>(), "the escrow keeps exactly the residue");
            assert!(raised <= ops, "the residue is under one lamport an operation");
        }
        println!("[solvency] {checked} sell-backs checked");
        assert!(checked > 3_000, "the sequences reached the sell-back ({checked})");
    }

    #[test]
    fn a_sell_back_pays_the_share_less_the_floored_fee() {
        assert_eq!(sell_back_with_team(1_000_000_000, 25).unwrap(), (2_500_000, 997_500_000));
        assert_eq!(sell_back_with_team(399, 25).unwrap(), (0, 399));
        assert_eq!(sell_back_with_team(123, 0).unwrap(), (0, 123));
    }

///
///
///
///
///
///
    #[test]
    fn the_team_gross_up_and_its_inverse_at_the_100_bps_sale_default() {
        const D: u16 = crate::policy::TEAM_SALE_FEE_BPS_DEFAULT;
        assert_eq!(D, 100);
        assert_eq!([team_net_of(1_010_000_000, D), team_net_of(101, D), team_net_of(100, D), team_net_of(1, D)], [1_000_000_000, 100, 99, 0]);
        assert_eq!((team_fee_of(99, D).unwrap(), team_fee_of(100, D).unwrap(), team_fee_of(1_000_000_000, D).unwrap()), (0, 1, 10_000_000));
        assert_eq!([team_gross_for(1_000_000_000, D).unwrap(), team_gross_for(99, D).unwrap(), team_gross_for(1, D).unwrap()], [1_010_000_000, 100, 2]);
        for c in [0u64, 1, 99, 100, 101, 1_000_000_007, 5_000_000_000_000] {
            let g = team_gross_for(c, D).unwrap();
            assert_eq!(team_net_of(g, D), c, "net_of(gross_for(c)) == c");
            assert_eq!(g - c - team_fee_of(c, D).unwrap(), u64::from(c % 100 != 0), "the slack is 0 iff c % 100 == 0");
        }
        assert_eq!(sell_back_with_team(1_000_000_000, D).unwrap(), (10_000_000, 990_000_000));
        assert_eq!(sell_back_with_team(99, D).unwrap(), (0, 99));

        let (fee_in, (fee_out, back)) = (team_fee_of(1_000_000_000, D).unwrap(), sell_back_with_team(1_000_000_000, D).unwrap());
        assert_eq!((fee_in + fee_out, 1_010_000_000 - back), (20_000_000, 20_000_000));
    }
}
