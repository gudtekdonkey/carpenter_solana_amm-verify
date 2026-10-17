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

#![allow(clippy::field_reassign_with_default)]

use floor_math::constants::{
    DEFAULT_SKIM_BUY_AT_FLOOR_BPS, DEFAULT_SKIM_SELL_AT_FLOOR_BPS, LP_FEE_PIPS, SKIM_RAMP_TICKS, TICKS_PER_DOUBLING,
};
use floor_math::{band_ticks, take_fees, FloorState};

///
///
///
///
fn skim_bps(distance: i32, ramp: i32, buy: bool, buy_far: u16, sell_far: u16) -> u64 {
    floor_math::skim_bps(distance, ramp, buy, buy_far, sell_far, DEFAULT_SKIM_BUY_AT_FLOOR_BPS as u16, DEFAULT_SKIM_SELL_AT_FLOOR_BPS as u16)
}

///
///
///
const TEAM: u64 = floor_math::policy::TEAM_POOL_FEE_BPS_DEFAULT as u64;

///
fn take_1e10(h: u64) -> u64 {
    10_000_000_000 - (10_000 - h) * (1_000_000 - LP_FEE_PIPS as u64)
}

fn pct(h: u64) -> f64 {
    take_1e10(h) as f64 / 1e8
}

fn near(a: f64, b: f64, eps: f64) {
    assert!((a - b).abs() < eps, "expected {b}, got {a} (Δ {})", (a - b).abs());
}

///
///
fn hook(buy: bool, distance: i32, far: u16, creator: u64, holder: u64) -> u64 {
    skim_bps(distance, SKIM_RAMP_TICKS, buy, far, far) + creator + holder + TEAM
}

const EPS: f64 = 5e-4;

///
///
fn pool(far: u16, creator: u16, holder: u16) -> FloorState {
    let mut s = FloorState::default();
    s.tick_spacing = 60;
    s.floor_tick = band_ticks(60).unwrap();
    s.skim_buy_at_floor_bps = DEFAULT_SKIM_BUY_AT_FLOOR_BPS as u16;
    s.skim_sell_at_floor_bps = DEFAULT_SKIM_SELL_AT_FLOOR_BPS as u16;
    s.skim_buy_far_bps = far;
    s.skim_sell_far_bps = far;
    s.creator_buy_bps = creator;
    s.creator_sell_bps = creator;
    s.holder_buy_bps = holder;
    s.holder_sell_bps = holder;
    s.platform_bps = 250;
    s.team_fee_bps = TEAM as u16;
    s
}


#[test]
fn f01_ramp_is_16094_ticks_not_a_round_5x() {
    assert_eq!(SKIM_RAMP_TICKS, 16094);
    let r = (SKIM_RAMP_TICKS as f64 * 1.0001f64.ln()).exp();
    near(r, 4.999408, 1e-5);

    assert_ne!(SKIM_RAMP_TICKS, (5f64.ln() / 1.0001f64.ln()).round() as i32);
}


#[test]
fn f02_constants_match_the_hook() {
    assert_eq!(LP_FEE_PIPS, 2500);
    assert_eq!(DEFAULT_SKIM_SELL_AT_FLOOR_BPS, 1000);
}


#[test]
fn f03_at_the_bid_closeness_is_full() {
    assert_eq!(skim_bps(0, SKIM_RAMP_TICKS, true, 200, 200), 0);
    assert_eq!(skim_bps(0, SKIM_RAMP_TICKS, false, 200, 200), 1000);
    assert_eq!(skim_bps(-60, SKIM_RAMP_TICKS, false, 200, 200), 1000);
}


#[test]
fn f04_far_rate_at_the_ramp_and_clamped_past_it() {
    for d in [SKIM_RAMP_TICKS, SKIM_RAMP_TICKS + 1, 100_000, i32::MAX] {
        assert_eq!(skim_bps(d, SKIM_RAMP_TICKS, true, 200, 200), 200);
        assert_eq!(skim_bps(d, SKIM_RAMP_TICKS, false, 200, 200), 200);
    }


    assert_eq!(skim_bps(SKIM_RAMP_TICKS - 1, SKIM_RAMP_TICKS, true, 200, 200), 200);
    assert_eq!(skim_bps(SKIM_RAMP_TICKS - 100, SKIM_RAMP_TICKS, true, 200, 200), 199);
}



#[test]
fn f05_linear_in_ticks_which_is_log_price() {

    assert_eq!(skim_bps(8047, SKIM_RAMP_TICKS, true, 200, 200), 100);
    assert_eq!(skim_bps(8047, SKIM_RAMP_TICKS, false, 200, 200), 600);
    let three_x = (3f64.ln() / 1.0001f64.ln()).round() as i32;
    assert!(skim_bps(three_x, SKIM_RAMP_TICKS, true, 200, 200) > 100, "3x is past halfway");
}





#[test]
fn f06_never_the_far_rate_at_the_bid() {
    for far in [100u16, 200, 500, 999] {
        assert_ne!(skim_bps(0, SKIM_RAMP_TICKS, true, far, far), far as u64);
        assert_ne!(skim_bps(0, SKIM_RAMP_TICKS, false, far, far), far as u64);
    }
}


#[test]
fn f07_buy_zero_at_bid_far_at_top() {
    assert_eq!(skim_bps(0, SKIM_RAMP_TICKS, true, 200, 200), 0);
    assert_eq!(skim_bps(SKIM_RAMP_TICKS, SKIM_RAMP_TICKS, true, 200, 200), 200);
}


#[test]
fn f08_sell_ten_at_bid_far_at_top() {
    assert_eq!(skim_bps(0, SKIM_RAMP_TICKS, false, 200, 200), 1000);
    assert_eq!(skim_bps(SKIM_RAMP_TICKS, SKIM_RAMP_TICKS, false, 200, 200), 200);
}


#[test]
fn f09_sides_move_opposite_ways() {
    let quarter = SKIM_RAMP_TICKS / 4;
    let three_q = SKIM_RAMP_TICKS * 3 / 4;
    assert!(skim_bps(quarter, SKIM_RAMP_TICKS, true, 200, 200) < skim_bps(three_q, SKIM_RAMP_TICKS, true, 200, 200));
    assert!(skim_bps(quarter, SKIM_RAMP_TICKS, false, 200, 200) > skim_bps(three_q, SKIM_RAMP_TICKS, false, 200, 200));
}


#[test]
fn f10_ten_percent_far_sell_is_flat() {
    for d in [0, 8047, SKIM_RAMP_TICKS] {
        assert_eq!(skim_bps(d, SKIM_RAMP_TICKS, false, 1000, 1000), 1000);
    }
}


#[test]
fn f11_lp_fee_composes_multiplicatively() {
    let h = hook(true, SKIM_RAMP_TICKS, 200, 100, 0);
    assert_eq!(take_1e10(h), 349_187_500);
    near(pct(h), 3.491875, EPS);
    assert!(pct(h) < 0.25 + 2.0 + 1.0 + 0.25);
}



#[test]
fn f12_meme_at_the_bid() {
    assert_eq!(take_1e10(hook(true, 0, 200, 100, 0)), 149_687_500);
    assert_eq!(take_1e10(hook(false, 0, 200, 100, 0)), 1_147_187_500);
    near(pct(hook(true, 0, 200, 100, 0)), 1.496875, EPS);
    near(pct(hook(false, 0, 200, 100, 0)), 11.471875, EPS);
}


#[test]
fn f13_meme_far_converges() {
    for d in [SKIM_RAMP_TICKS, 50_000] {
        near(pct(hook(true, d, 200, 100, 0)), 3.491875, EPS);
        near(pct(hook(false, d, 200, 100, 0)), 3.491875, EPS);
    }
}








#[test]
fn f14_meme_at_two_x() {
    assert_eq!(skim_bps(TICKS_PER_DOUBLING, SKIM_RAMP_TICKS, true, 200, 200), 87);
    assert_eq!(skim_bps(TICKS_PER_DOUBLING, SKIM_RAMP_TICKS, false, 200, 200), 655);
    assert_eq!(take_1e10(87 + 100 + TEAM), 236_470_000);
    assert_eq!(take_1e10(655 + 100 + TEAM), 803_050_000);
    near(pct(212), 2.35614, 0.01);
    near(pct(780), 8.03483, 0.01);
    assert!(pct(780) - pct(212) > 5.6, "the asymmetry is the mechanism");
}













#[test]
fn f15_bluechip_and_degen() {
    near(pct(hook(true, 0, 500, 50, 0)), 0.998125, EPS);
    near(pct(hook(true, SKIM_RAMP_TICKS, 500, 50, 0)), 5.985625, EPS);
    near(pct(hook(false, 0, 500, 50, 0)), 10.973125, EPS);
    near(pct(hook(true, SKIM_RAMP_TICKS, 100, 100, 0)), 2.494375, EPS);
    near(pct(hook(false, 0, 100, 100, 0)), 11.471875, EPS);
}













#[test]
fn f16_worst_legal_evm_launch() {
    use floor_math::constants::skim_sell_at_floor_bps;
    use floor_math::policy::{MAX_CREATOR_BPS, MAX_HOLDER_BPS_EVM, MAX_SKIM_FAR_BPS};
    assert_eq!((MAX_CREATOR_BPS, MAX_HOLDER_BPS_EVM, MAX_SKIM_FAR_BPS), (500, 1000, 1000));
    let h = hook(false, SKIM_RAMP_TICKS, MAX_SKIM_FAR_BPS, MAX_CREATOR_BPS as u64, MAX_HOLDER_BPS_EVM as u64);

    assert_eq!(h, 2525);
    assert_eq!(take_1e10(h), 2_543_687_500);
    near(pct(h), 25.436875, EPS);


    let at = floor_math::skim_bps(0, SKIM_RAMP_TICKS, false, MAX_SKIM_FAR_BPS, MAX_SKIM_FAR_BPS, 0, skim_sell_at_floor_bps(MAX_SKIM_FAR_BPS))
        + MAX_CREATOR_BPS as u64
        + MAX_HOLDER_BPS_EVM as u64
        + TEAM;
    assert_eq!(at, 3025);
    assert_eq!(take_1e10(at), 3_042_437_500);
    near(pct(at), 30.424375, EPS);
}




#[test]
fn f17_platform_cut_is_a_split_not_a_take() {
    let mut s = pool(200, 100, 0);
    let t = take_fees(&mut s, 10_000_000_000, true, -SKIM_RAMP_TICKS, None).unwrap();
    assert_eq!(t.total, 325_000_000);
    assert_eq!(t.kept + t.cut, t.skim_fee);
    assert_eq!(t.cut, 5_000_000);
    near(pct(t.total * 10_000 / 10_000_000_000), 3.491875, EPS);
}


#[test]
fn f18_holder_tax_is_counted() {
    let off = pct(hook(true, SKIM_RAMP_TICKS, 200, 100, 0));
    let on = pct(hook(true, SKIM_RAMP_TICKS, 200, 100, 200));
    assert!(on > off + 1.9);
    let mut s = pool(200, 100, 200);
    let t = take_fees(&mut s, 1_000_000, true, -SKIM_RAMP_TICKS, None).unwrap();
    assert_eq!((t.holder_fee, t.team_fee, t.total), (20_000, 2_500, 52_500));
}



#[test]
fn f19_zero_leg_is_zero_take() {
    let mut s = pool(200, 100, 0);
    let before = s;
    assert_eq!(take_fees(&mut s, 0, false, 0, None).unwrap().total, 0);
    assert_eq!(s, before);
}



#[test]
fn f20_ten_sol_sell_at_the_bid_nets_8_8528125() {
    let mut s = pool(200, 100, 0);
    let gross = 10_000_000_000u64;
    let t = take_fees(&mut s, gross, false, 0, None).unwrap();
    let after_hook = gross - t.total;
    let lp = after_hook * LP_FEE_PIPS as u64 / 1_000_000;
    assert_eq!(after_hook - lp, 8_852_812_500);
}


#[test]
fn f21_skim_rate_reported_separately() {
    let mut s = pool(200, 100, 0);
    assert_eq!(take_fees(&mut s, 1_000_000, false, 0, None).unwrap().skim_bps, 1000);
    assert_eq!(take_fees(&mut s, 1_000_000, false, -SKIM_RAMP_TICKS, None).unwrap().skim_bps, 200);
}
