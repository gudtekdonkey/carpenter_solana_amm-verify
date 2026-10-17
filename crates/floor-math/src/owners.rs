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

use crate::constants::{BPS, SUPPLY};
use crate::error::{FloorMathError, FloorMathResult};
use crate::stair::MAX_STAIR_STEPS;

const N: usize = MAX_STAIR_STEPS as usize;

///
///
///
pub const OWNER_BOOKS_DUST: u64 = 64;

///
///
///
pub fn owner_tokens_top(planned: &[u64; N], steps: u8, owner_total: u64) -> [u64; N] {
    let mut out = [0u64; N];
    let mut left = owner_total;
    for i in (0..steps.min(MAX_STAIR_STEPS) as usize).rev() {
        let take = left.min(planned[i]);
        out[i] = take;
        left -= take;
        if left == 0 {
            break;
        }
    }
    out
}

///
///
///
pub fn owner_tokens_blended(planned: &[u64; N], steps: u8, owner_total: u64) -> FloorMathResult<[u64; N]> {
    let n = steps.min(MAX_STAIR_STEPS) as usize;
    let total: u128 = planned[..n].iter().map(|&x| x as u128).sum();
    let mut out = [0u64; N];
    if total == 0 || owner_total == 0 {
        return Ok(out);
    }
    if owner_total as u128 > total {
        return Err(FloorMathError::Overflow);
    }
    for i in 0..n {
        out[i] = (planned[i] as u128 * owner_total as u128 / total) as u64;
    }
    Ok(out)
}

///
pub fn first_owner_step(tokens: &[u64; N], steps: u8) -> u8 {
    (0..steps.min(MAX_STAIR_STEPS)).find(|&i| tokens[i as usize] != 0).unwrap_or(steps)
}

///
///
///
///
///
pub fn owner_part_of_close(before: u64, after: u64, owner: u64, principal: u64) -> (u64, u64) {
    if before == 0 || after >= before || owner == 0 {
        return (0, 0);
    }
    let owner = owner.min(before);
    let sold = (before - after) as u128;
    let owner_sold = ((sold * owner as u128).div_ceil(before as u128) as u64).min(owner);
    let owner_quote = (principal as u128 * owner as u128 / before as u128) as u64;
    (owner_sold, owner_quote)
}

///
///
pub fn owner_fee_part(fee: u64, before: u64, owner: u64) -> u64 {
    if before == 0 || owner == 0 {
        return 0;
    }
    (fee as u128 * owner.min(before) as u128 / before as u128) as u64
}

///
///
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReserveClose {
///
///
    pub left: [u64; N],
///
    pub owner_sold: [u64; N],
///
    pub owner_total: u64,
///
    pub protocol_sold: u64,
///
    pub protocol_left: u64,
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
pub fn reserve_allot(real: u64, planned: &[u64; N], cut: &[u64; N], _fed: u64, j: u8, steps: u8) -> ([u64; N], [u64; N], u64) {
    let mut remaining = [0u64; N];
    let mut gone = [0u64; N];
    if j >= steps {
        return (remaining, gone, real);
    }
    let n = steps.min(MAX_STAIR_STEPS) as usize;
    let mut planned_sum = 0u64;
    for k in j as usize..n {
        planned_sum = planned_sum.saturating_add(planned[k].saturating_sub(cut[k]));
    }

    let steps_real = real.min(planned_sum);
    let protocol = real - steps_real;
    let mut deficit = planned_sum - steps_real;
    for k in j as usize..n {
        let p = planned[k].saturating_sub(cut[k]);
        let g = deficit.min(p);
        deficit -= g;
        gone[k] = g;
        remaining[k] = p - g;
    }
    (remaining, gone, protocol)
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
#[allow(clippy::too_many_arguments)]
pub fn reserve_close(before: u64, after: u64, planned: &[u64; N], cut: &[u64; N], owner: &[u64; N], fed: u64, j: u8, steps: u8) -> ReserveClose {
    let mut out = ReserveClose { left: [0; N], owner_sold: [0; N], owner_total: 0, protocol_sold: 0, protocol_left: 0 };
    let (remaining, gone, protocol) = reserve_allot(before, planned, cut, fed, j, steps);
    let mut sold = before.saturating_sub(after);
    out.protocol_sold = sold.min(protocol);
    out.protocol_left = protocol - out.protocol_sold;
    sold -= out.protocol_sold;
    if j >= steps {
        return out;
    }
    for k in j as usize..steps.min(MAX_STAIR_STEPS) as usize {
        let p_left = remaining[k];
        let here = sold.min(p_left);
        sold -= here;
        out.left[k] = gone[k] + here;
        if here == 0 || owner[k] == 0 {
            continue;
        }
        let o = ((here as u128 * owner[k].min(p_left) as u128).div_ceil(p_left as u128) as u64).min(owner[k]);
        out.owner_sold[k] = o;
        out.owner_total += o;
    }
    out
}

///
///
///
///
pub fn record_reserve_shortfall(planned: &[u64; N], cut: &mut [u64; N], fed: u64, j: u8, steps: u8, shortfall: u64) -> u64 {
    let from_fed = shortfall.min(fed);
    let mut left = shortfall - from_fed;
    for k in j.min(MAX_STAIR_STEPS) as usize..steps.min(MAX_STAIR_STEPS) as usize {
        if left == 0 {
            break;
        }
        let p = planned[k].saturating_sub(cut[k]);
        let g = left.min(p);
        cut[k] += g;
        left -= g;
    }
    fed - from_fed
}

///
///
///
///
pub fn reserve_owner_sold(before: u64, after: u64, planned: &[u64; N], cut: &[u64; N], owner: &[u64; N], j: u8, steps: u8) -> ([u64; N], u64) {
    let c = reserve_close(before, after, planned, cut, owner, 0, j, steps);
    (c.owner_sold, c.owner_total)
}

///
///
///
pub fn cap_owner_cheapest_first(owner: &mut [u64; N], from: u8, to: u8, held: u64) -> u64 {
    let (a, b) = (from.min(MAX_STAIR_STEPS) as usize, to.min(MAX_STAIR_STEPS) as usize);
    let total: u64 = owner[a.min(b)..b].iter().fold(0u64, |s, &t| s.saturating_add(t));
    let mut over = total.saturating_sub(held);
    let booked = over;
    for t in owner[a.min(b)..b].iter_mut() {
        if over == 0 {
            break;
        }
        let d = over.min(*t);
        *t -= d;
        over -= d;
    }
    booked
}

///
///
pub fn owner_planned(planned: &[u64; N], cut: &[u64; N]) -> [u64; N] {
    let mut out = [0u64; N];
    for k in 0..N {
        out[k] = planned[k].saturating_sub(cut[k]);
    }
    out
}

///
///
pub fn payee_due(owner_quote: u64, payee_bps: u16) -> u64 {
    (owner_quote as u128 * (payee_bps as u128).min(BPS as u128) / BPS as u128) as u64
}

///
///
///
///
///
pub fn payee_after_floor_first(due: u64, collected: u64, keep: u64) -> (u64, u64) {
    let room = collected.saturating_sub(keep);
    let paid = due.min(room);
    (paid, due - paid)
}

///
///
///
///
pub fn unlocked_steps(now: i64, anchor: i64, first_unlock_secs: u32, every_secs: u32) -> u8 {
    if anchor == 0 {
        return 0;
    }
    let at = anchor as i128 + first_unlock_secs as i128;
    if (now as i128) < at {
        return 0;
    }
    if every_secs == 0 {
        return MAX_STAIR_STEPS;
    }
    let more = (now as i128 - at) / every_secs as i128;
    (1 + more).min(MAX_STAIR_STEPS as i128) as u8
}

///
pub fn step_unlocked(i: u8, first_step: u8, now: i64, anchor: i64, first_unlock_secs: u32, every_secs: u32) -> bool {
    let u = unlocked_steps(now, anchor, first_unlock_secs, every_secs) as u16;
    i >= first_step && (i as u16) < first_step as u16 + u
}



///
///
///
pub fn team_buy_ok(mode: u8, total: u64, steps: u64) -> bool {
    team_buy_ok_max(team_buy_max_bps(mode), total, steps)
}

///
///
pub fn team_buy_ok_max(max_bps: u64, total: u64, steps: u64) -> bool {
    let max = SUPPLY / BPS * max_bps;
    total > 0
        && total <= max
        && steps <= total
        && (steps as u128) * BPS as u128 >= (total as u128) * floor_types_team_buy::TEAM_STEPS_MIN_BPS as u128
}

///
///
pub fn team_buy_max_bps(mode: u8) -> u64 {
    if mode == crate::modes::Mode::Bluechip as u8 {
        floor_types_team_buy::BLUECHIP_TEAM_BUY_MAX_BPS
    } else {
        floor_types_team_buy::TEAM_BUY_MAX_BPS
    }
}

///
///
///
pub fn team_step_share_bps(team_steps: u64, staircase: u64) -> Option<u64> {
    if staircase == 0 {
        return None;
    }
    Some((team_steps as u128 * BPS as u128 / staircase as u128) as u64)
}

///
///
///
pub fn team_steps_fit(team_steps: u64, staircase: u64) -> bool {
    staircase != 0
        && team_steps <= staircase
        && team_steps as u128 * BPS as u128 <= staircase as u128 * floor_types_team_buy::TEAM_STEPS_MAX_SHARE_BPS as u128
}

///
///
pub fn team_vest_days_ok(days: &[u16; 4], max_days: u16) -> bool {
    days[0] >= 1 && days.windows(2).all(|w| w[0] < w[1]) && days[3] <= max_days
}

///
///
pub fn team_vested(total: u64, days: &[u16; 4], anchor: i64, now: i64) -> u64 {
    if anchor == 0 {
        return 0;
    }
    let passed = days.iter().filter(|&&d| now as i128 >= anchor as i128 + d as i128 * 86_400).count() as u64;
    if passed >= 4 {
        total
    } else {
        total / 4 * passed
    }
}

///
pub mod floor_types_team_buy {
///
    pub const TEAM_BUY_MAX_BPS: u64 = 500;
///
    pub const BLUECHIP_TEAM_BUY_MAX_BPS: u64 = 1_000;
///
    pub const TEAM_STEPS_MIN_BPS: u64 = 8_000;
///
    pub const TEAM_STEPS_MAX_SHARE_BPS: u64 = 2_500;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stair::step_tokens;

    const M: u64 = 1_000_000_000_000;

///
///
    #[test]
    fn top_layout_is_the_specs_300k_table() {
        let planned = step_tokens(930 * M, 9, 9_500).unwrap();
        let team = owner_tokens_top(&planned, 9, 800 * M);
        assert_eq!(team.iter().sum::<u64>(), 800 * M);
        assert_eq!(team[0], 0, "step 1 is the protocol's");
        let shared_protocol = planned[1] - team[1];
        assert_eq!(((team[1] + M / 20) / (M / 10), (shared_protocol + M / 20) / (M / 10)), (1_152, 42), "115.2M / 4.2M");
        for i in 2..9 {
            assert_eq!(team[i], planned[i], "step {} all team", i + 1);
        }
        assert_eq!(first_owner_step(&team, 9), 1, "the first team step is step 2");

        let p6 = step_tokens(930 * M, 6, 9_500).unwrap();
        let t6 = owner_tokens_top(&p6, 6, 800 * M);
        assert_eq!(t6.iter().sum::<u64>(), 800 * M);
        assert!(t6[0] == p6[0] - 130 * M && t6[5] == p6[5]);
        assert_eq!(first_owner_step(&t6, 6), 0);
    }

    #[test]
    fn blended_is_an_equal_share_of_every_step_never_more() {
        let planned = step_tokens(730 * M, 12, 9_880).unwrap();
        let deployer = owner_tokens_blended(&planned, 12, 40 * M).unwrap();
        let sum: u64 = deployer.iter().sum();
        assert!(sum <= 40 * M && 40 * M - sum < 12);
        for i in 0..12 {
            assert!(deployer[i] <= planned[i]);
            let share = deployer[i] as f64 / planned[i] as f64;
            assert!((share - 40.0 / 730.0).abs() < 1e-9, "step {i}: {share}");
        }
        assert_eq!(first_owner_step(&deployer, 12), 0);
        assert!(owner_tokens_blended(&planned, 12, 731 * M).is_err());
    }

    #[test]
    fn a_close_rounds_against_the_owner() {

        assert_eq!(owner_part_of_close(100, 75, 40, 1_000), (10, 400));
        assert_eq!(owner_part_of_close(3, 2, 1, 10), (1, 3), "ceil(1/3) sold, floor(10/3) paid");
        assert_eq!(owner_part_of_close(100, 100, 40, 0), (0, 0), "nothing sold");
        assert_eq!(owner_part_of_close(0, 0, 0, 5), (0, 0));
        assert_eq!(owner_fee_part(99, 100, 40), 39);
    }

///
    #[test]
    fn the_reserve_attributes_bottom_up_after_the_excess() {
        let mut planned = [0u64; N];
        let mut owner = [0u64; N];
        let cut = [0u64; N];
        planned[1] = 100;
        planned[2] = 100;
        planned[3] = 100;
        owner[2] = 100;
        owner[3] = 50;

        let (per, total) = reserve_owner_sold(320, 250, &planned, &cut, &owner, 1, 4);
        assert_eq!((per[2], total), (0, 0));

        let (per, total) = reserve_owner_sold(320, 140, &planned, &cut, &owner, 1, 4);
        assert_eq!((per[2], per[3], total), (60, 0, 60));

        let (per, total) = reserve_owner_sold(320, 0, &planned, &cut, &owner, 1, 4);
        assert_eq!((per[2], per[3], total), (100, 50, 150));

        let mut cut2 = [0u64; N];
        cut2[3] = 40;
        let mut owner2 = owner;
        owner2[3] = 10;
        let (per, _) = reserve_owner_sold(280, 0, &planned, &cut2, &owner2, 1, 4);
        assert_eq!(per[3], 10);
        assert_eq!(owner_planned(&planned, &cut2)[3], 60);
    }

///
///
///
///
///
///
///
    #[test]
    fn a_second_close_never_books_capacity_an_earlier_close_sold() {
        let mut planned = [0u64; N];
        planned[1] = 100;
        planned[2] = 100;
        planned[3] = 100;
        let mut owner = planned;
        let mut cut = [0u64; N];
        let book = |owner: &mut [u64; N], cut: &mut [u64; N], before: u64, after: u64, j: u8| -> u64 {
            let c = reserve_close(before, after, &planned, cut, owner, 0, j, 4);
            for k in 0..N {
                owner[k] -= c.owner_sold[k];
                cut[k] += c.left[k];
            }
            c.owner_total
        };

        assert_eq!(book(&mut owner, &mut cut, 300, 150, 1), 150, "every token sold out of an all-owner reserve is the owner's");
        let (to_step, rest) = crate::stair::reserve_split(150, &owner_planned(&planned, &cut), 1, 4).unwrap();
        assert_eq!((to_step, rest), (0, 150), "step 1 sold through: the reserve keeps the 150 for steps 2..3");
        cap_owner_cheapest_first(&mut owner, 2, 4, rest);
        assert_eq!(owner[2..4].iter().sum::<u64>(), 150);

        assert_eq!(book(&mut owner, &mut cut, 150, 50, 2), 100, "the second close books all 100 sold tokens to the owner (was 50)");
        let (to_step, rest) = crate::stair::reserve_split(50, &owner_planned(&planned, &cut), 2, 4).unwrap();
        assert_eq!((to_step, rest), (0, 50));
        assert_eq!(owner[2..4].iter().sum::<u64>(), 50, "the owner's books stand at exactly what the reserve holds");
        assert_eq!((owner[2], owner[3]), (0, 50), "the cheapest step sold first");
    }

///
///
///
///
    #[test]
    fn a_short_reserve_loses_its_deficit_off_the_cheapest_steps() {
        let mut planned = [0u64; N];
        planned[2] = 100;
        planned[3] = 100;
        let mut owner = [0u64; N];
        owner[2] = 10;
        owner[3] = 100;
        let cut = [0u64; N];

        let c = reserve_close(110, 50, &planned, &cut, &owner, 0, 2, 4);
        assert_eq!((c.left[2], c.left[3]), (100, 50), "the deficit and step 2's last 10 left; 50 of step 3");
        assert_eq!((c.owner_sold[2], c.owner_sold[3], c.owner_total), (10, 50, 60));

        let mut o = [0u64; N];
        o[2] = 40;
        o[3] = 80;
        assert_eq!(cap_owner_cheapest_first(&mut o, 2, 4, 50), 70);
        assert_eq!((o[2], o[3]), (0, 50));
        assert_eq!(cap_owner_cheapest_first(&mut o, 2, 4, 50), 0, "already within: nothing moves");
    }

///
///
///
///
///
///
    #[test]
    fn a_reserve_hole_is_charged_to_the_fed_excess_first() {
        let mut planned = [0u64; N];
        planned[2] = 100;
        planned[3] = 100;
        let owner = planned;
        let cut = [0u64; N];
        let (remaining, gone, protocol) = reserve_allot(210, &planned, &cut, 60, 2, 4);
        assert_eq!((remaining[2], remaining[3], gone[2], protocol), (100, 100, 0, 10), "the hole is the fed record's; the steps whole");
        let c = reserve_close(210, 100, &planned, &cut, &owner, 60, 2, 4);
        assert_eq!((c.protocol_sold, c.protocol_left), (10, 0));
        assert_eq!((c.owner_sold[2], c.owner_sold[3], c.owner_total), (100, 0, 100));
        assert_eq!((c.left[2], c.left[3]), (100, 0), "step 2 sold through: no hole on it");

        let (remaining, gone, protocol) = reserve_allot(130, &planned, &cut, 60, 2, 4);
        assert_eq!((remaining[2], remaining[3], gone[2], protocol), (30, 100, 70, 0));

        let c = reserve_close(270, 240, &planned, &cut, &owner, 60, 2, 4);
        assert_eq!((c.protocol_sold, c.protocol_left, c.owner_total), (30, 40, 0));
    }

///
///
///
///
    #[test]
    fn row_889_the_cook_hole_books_the_evms_figures() {
        const M: u64 = 1_000_000_000_000;
        let (fed, hole, sold) = (7_500_000 * M / 1_000_000, 9_480_000 * M / 1_000_000, 410_000 * M / 1_000_000);
        let mut planned = [0u64; N];
        planned[2] = 60 * M;
        planned[3] = 62 * M;
        let mut owner = [0u64; N];
        owner[2] = 12 * M;
        owner[3] = 12 * M;
        let cut = [0u64; N];
        let before = planned[2] + planned[3] + fed - hole;
        let c = reserve_close(before, before - sold, &planned, &cut, &owner, fed, 2, 4);
        assert_eq!(c.protocol_left, 0, "the excess is 0, as the EVM's");
        assert_eq!(c.left[2], hole + sold - fed, "the step booked 2.39M gone, as the EVM's");
        assert_eq!(c.left[2], 2_390_000 * M / 1_000_000);
        assert_eq!(c.left[3], 0);
    }

///
///
    #[test]
    fn a_reopens_rounding_is_the_protocols_then_the_cheapest_steps() {
        let mut planned = [0u64; N];
        planned[2] = 100;
        planned[3] = 100;
        let mut cut = [0u64; N];
        assert_eq!(record_reserve_shortfall(&planned, &mut cut, 5, 2, 4, 3), 2, "the fed record pays");
        assert_eq!(cut, [0u64; N]);
        assert_eq!(record_reserve_shortfall(&planned, &mut cut, 2, 2, 4, 105), 0);
        assert_eq!((cut[2], cut[3]), (100, 3), "then step 2's whole record, then step 3's");
        let (remaining, _, protocol) = reserve_allot(97, &planned, &cut, 0, 2, 4);
        assert_eq!((remaining[2], remaining[3], protocol), (0, 97, 0), "the records equal the position");
    }

    #[test]
    fn floor_first_then_the_payee_and_the_rest_is_forfeited() {
        assert_eq!(payee_due(1_000, 4_000), 400);
        assert_eq!(payee_after_floor_first(400, 1_000, 0), (400, 0));
        assert_eq!(payee_after_floor_first(400, 1_000, 700), (300, 100), "the floor needed 700: 100 forfeited");
        assert_eq!(payee_after_floor_first(400, 1_000, 1_000), (0, 400));
        assert_eq!(payee_after_floor_first(400, 1_000, 2_000), (0, 400), "never a debt");
    }

///
    #[test]
    fn the_cliffs_are_every_30_days_from_day_30() {
        const D: i64 = 86_400;
        let a = 1_000_000;
        let u = |day: i64| unlocked_steps(a + day * D, a, 30 * D as u32, 30 * D as u32);
        assert_eq!((u(0), u(29), u(30), u(59), u(60), u(240), u(10_000)), (0, 0, 1, 1, 2, 8, 16));
        assert!(!step_unlocked(1, 1, a + 29 * D, a, 30 * D as u32, 30 * D as u32));
        assert!(step_unlocked(1, 1, a + 30 * D, a, 30 * D as u32, 30 * D as u32), "the lowest team step at day 30");
        assert!(!step_unlocked(2, 1, a + 59 * D, a, 30 * D as u32, 30 * D as u32));
        assert!(step_unlocked(2, 1, a + 60 * D, a, 30 * D as u32, 30 * D as u32));
        assert!(!step_unlocked(0, 1, a + 900 * D, a, 30 * D as u32, 30 * D as u32), "a protocol step never");
        let deployer = |day: i64| unlocked_steps(a + day * D, a, 90 * D as u32, 0);
        assert_eq!((deployer(89), deployer(90)), (0, 16));
        assert_eq!(unlocked_steps(i64::MAX, 0, 0, 0), 0, "no anchor, nothing");
    }

///
    #[test]
    fn bluechips_team_buy_is_up_to_ten_percent_the_rest_five() {
        use crate::modes::Mode;
        let (five, ten) = (SUPPLY / 20, SUPPLY / 10);
        let bc = Mode::Bluechip as u8;
        assert_eq!(team_buy_max_bps(bc), 1_000);
        for m in [Mode::Cook, Mode::Moonshot, Mode::Custom] {
            assert_eq!(team_buy_max_bps(m as u8), 500, "{m:?} keeps 5 %");
            assert!(!team_buy_ok(m as u8, five + 1, five + 1), "{m:?} over 5 %");
        }
        assert!(team_buy_ok(bc, ten, ten * 4 / 5), "≥8% steps, ≤2% wallet");
        assert!(team_buy_ok(bc, ten, ten), "all on steps");
        assert!(!team_buy_ok(bc, ten + 1, ten + 1), "over 10 %");
        assert!(!team_buy_ok(bc, ten, ten * 4 / 5 - 1), "under 80 % on steps");
    }

    #[test]
    fn the_team_buy_is_up_to_five_percent_eighty_on_steps() {
        let five = SUPPLY / 20;
        let cook = crate::modes::Mode::Cook as u8;
        assert!(team_buy_ok(cook, five, five * 4 / 5), "≥4% steps, ≤1% wallet");
        assert!(team_buy_ok(cook, five, five), "he may choose to put 100 percent here");
        assert!(!team_buy_ok(cook, five, five * 4 / 5 - 1), "under 80 % on steps");
        assert!(!team_buy_ok(cook, five + 1, five), "over 5 %");
        assert!(!team_buy_ok(cook, 0, 0), "more than 0");
        assert!(team_vest_days_ok(&[10, 20, 40, 80], 3_650) && team_vest_days_ok(&[30, 60, 90, 120], 3_650));
        assert!(!team_vest_days_ok(&[0, 20, 40, 80], 3_650) && !team_vest_days_ok(&[10, 10, 40, 80], 3_650));
        let a = 5_000;
        assert_eq!(team_vested(1_003, &[10, 20, 40, 80], a, a + 9 * 86_400), 0);
        assert_eq!(team_vested(1_003, &[10, 20, 40, 80], a, a + 10 * 86_400), 250);
        assert_eq!(team_vested(1_003, &[10, 20, 40, 80], a, a + 79 * 86_400), 750);
        assert_eq!(team_vested(1_003, &[10, 20, 40, 80], a, a + 80 * 86_400), 1_003);
        assert_eq!(team_vested(1_003, &[10, 20, 40, 80], 0, i64::MAX), 0);
    }

///
///
///
    #[test]
    fn team_steps_are_at_most_a_quarter_of_the_staircase_and_one_step_blends() {
        assert_eq!(floor_types_team_buy::TEAM_STEPS_MAX_SHARE_BPS, 2_500, "owner 2026-10-04: under 25 percent");
        assert!(team_steps_fit(250, 1_000), "exactly a quarter");
        assert!(!team_steps_fit(251, 1_000), "one over a quarter");
        assert!(!team_steps_fit(500, 1_000), "the old half is over");
        assert!(!team_steps_fit(1, 0), "no staircase");
        assert!(!team_steps_fit(0, 0), "no staircase, even for nothing");
        assert!(team_steps_fit(0, 1));
        assert!(!team_steps_fit(u64::MAX, u64::MAX - 1));
        assert!(team_steps_fit(u64::MAX / 4, u64::MAX), "u128, no overflow");
        assert!(!team_steps_fit(u64::MAX / 4 + 1, u64::MAX));
        assert_eq!(team_step_share_bps(500, 1_000), Some(5_000));
        assert_eq!(team_step_share_bps(1, 3), Some(3_333), "floored");
        assert_eq!(team_step_share_bps(5, 0), None);

        let staircase = 690_000_000_000_000 + 50_000_000_000_000;
        assert!(team_steps_fit(50_000_000_000_000, staircase));
        assert_eq!(team_step_share_bps(50_000_000_000_000, staircase), Some(675));

        let mut planned = [0u64; MAX_STAIR_STEPS as usize];
        planned[0] = 1_000;
        let one = owner_tokens_blended(&planned, 1, 500).unwrap();
        assert_eq!(one[0], 500);
        assert_eq!(one[1..].iter().sum::<u64>(), 0);
        assert_eq!(first_owner_step(&one, 1), 0);
    }
}
