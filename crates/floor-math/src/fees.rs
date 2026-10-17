//!
//!
//!
//!
//!
//!
//!
//!
//!

use crate::band::dear_edge;
use crate::constants::{BPS, SKIM_RAMP_TICKS};
use crate::error::{FloorMathError, FloorMathResult};
use crate::ramp::skim_bps;
use crate::state::FloorState;

fn add(a: u64, b: u64) -> FloorMathResult<u64> {
    a.checked_add(b).ok_or(FloorMathError::Overflow)
}

///
///
///
///
///
///
///
pub fn part(leg: u64, bps: u64) -> FloorMathResult<u64> {
    u64::try_from(leg as u128 * bps as u128 / BPS as u128).map_err(|_| FloorMathError::Overflow)
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
pub fn intake(state: &mut FloorState, x: u64) -> FloorMathResult<u64> {
    if x == 0 || state.platform_bps == 0 {
        return Ok(x);
    }
    let cut = part(x, state.platform_bps as u64)?;
    if cut == 0 {
        return Ok(x);
    }
    let owed = add(state.platform_owed, cut)?;
    let paid = add(state.platform_paid_eth, cut)?;
    state.platform_owed = owed;
    state.platform_paid_eth = paid;
    Ok(x - cut)
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
///
///
///
///
///
///
pub const STAIR_GAP_SKIM: u8 = 2;
///
pub const STAIR_GAP_SKIM_TICKS: i32 = 5_109;
///
pub const STAIR_GAP_SKIM_FROM_TICKS: i32 = 2_232;
///
pub const STAIR_GAP_SKIM_MULT_BPS: u64 = 15_000;
///
pub const STAIR_GAP_SKIM_MAX_BPS: u64 = 1_500;

const _: () = assert!(STAIR_GAP_SKIM <= 2 && 0 < STAIR_GAP_SKIM_FROM_TICKS && STAIR_GAP_SKIM_FROM_TICKS < STAIR_GAP_SKIM_TICKS);

///
pub const fn cushion_gap_skim_on() -> bool {
    STAIR_GAP_SKIM != 0
}

///
///
pub fn gap_buy_skim_bps(bps: u64, is_buy: bool, tick: i32, cushion_top: Option<i32>) -> u64 {
    if !cushion_gap_skim_on() || !is_buy {
        return bps;
    }
    gap_skim_with(bps, tick, cushion_top, STAIR_GAP_SKIM, STAIR_GAP_SKIM_FROM_TICKS, STAIR_GAP_SKIM_TICKS)
}

///
pub fn gap_skim_with(bps: u64, tick: i32, cushion_top: Option<i32>, mode: u8, from: i32, to: i32) -> u64 {
    let Some(top) = cushion_top else { return bps };
    let gap = top as i64 - tick as i64;
    let (from, to) = (from as i64, to as i64);
    let m = if gap >= to {
        STAIR_GAP_SKIM_MULT_BPS
    } else if mode == 2 && gap > from {
        BPS + (STAIR_GAP_SKIM_MULT_BPS - BPS) * (gap - from) as u64 / (to - from) as u64
    } else {
        return bps;
    };
    let bps = bps * m / BPS;
    bps.min(STAIR_GAP_SKIM_MAX_BPS)
}

///
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FeeTake {
///
///
    pub total: u64,
///
    pub skim_bps: u64,
///
    pub skim_fee: u64,
///
    pub kept: u64,
///
    pub cut: u64,
///
///
///
///
    pub creator_fee: u64,
///
///
///
///
///
///
///
    pub referral_fee: u64,
///
///
///
///
///
///
///
///
    pub holder_fee: u64,
///
///
    pub team_fee: u64,
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
pub fn take_fees(
    state: &mut FloorState,
    leg: u64,
    is_buy: bool,
    tick: i32,
    cushion_top: Option<i32>,
) -> FloorMathResult<FeeTake> {
    let edge = dear_edge(state.floor_tick, state.tick_spacing)?;
    let distance = edge.checked_sub(tick).ok_or(FloorMathError::Overflow)?;
    let bps = skim_bps(
        distance,
        SKIM_RAMP_TICKS,
        is_buy,
        state.skim_buy_far_bps,
        state.skim_sell_far_bps,
        state.skim_buy_at_floor_bps,
        state.skim_sell_at_floor_bps,
    );
    let bps = gap_buy_skim_bps(bps, is_buy, tick, cushion_top);
    let (creator_bps, holder_bps) = if is_buy {
        (state.creator_buy_bps, state.holder_buy_bps)
    } else {
        (state.creator_sell_bps, state.holder_sell_bps)
    };
    let fee = part(leg, bps)?;
    let cfee = part(leg, creator_bps as u64)?;
    let hfee = part(leg, holder_bps as u64)?;

    let tfee = part(leg, state.team_fee_bps as u64)?;
    let total = add(add(add(fee, cfee)?, hfee)?, tfee)?;





    let mut take = FeeTake {
        total,
        skim_bps: bps,
        skim_fee: fee,
        kept: 0,
        cut: 0,
        creator_fee: cfee,
        referral_fee: 0,
        holder_fee: hfee,
        team_fee: tfee,
    };
    if total == 0 {
        return Ok(take);
    }
    let mut next = *state;
    if fee > 0 {
        let kept = intake(&mut next, fee)?;
        next.vault_eth = add(next.vault_eth, kept)?;
        next.unpaid_skim = add(next.unpaid_skim, kept)?;
        take.kept = kept;
        take.cut = fee - kept;
    }
















    let mut creator_net = cfee;
    if cfee > 0 && next.referrer != [0u8; 32] {
        let share = next.referral_share_bps as u64;


        if share > BPS {
            return Err(FloorMathError::ReferralShareOutOfRange);
        }
        let referral = part(cfee, share)?;

        if referral != 0 {
            next.referral_owed = add(next.referral_owed, referral)?;
            next.referral_fees_paid = add(next.referral_fees_paid, referral)?;
            creator_net = cfee - referral;
            take.referral_fee = referral;
        }
    }
    next.creator_fees_owed = add(next.creator_fees_owed, creator_net)?;
    next.holder_owed = add(next.holder_owed, hfee)?;
    next.team_owed = add(next.team_owed, tfee)?;
    *state = next;
    Ok(take)
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use crate::band::band_ticks;
    use crate::constants::{DEFAULT_SKIM_BUY_AT_FLOOR_BPS, DEFAULT_SKIM_SELL_AT_FLOOR_BPS};
    use crate::modes::{preset, Mode};
    use crate::policy::PLATFORM_BPS;

///
///
///
///
///
///
    fn meme(edge: i32) -> FloorState {
        let mut s = FloorState::default();
        s.tick_spacing = 60;
        s.floor_tick = edge + band_ticks(60).unwrap();
        s.skim_buy_at_floor_bps = DEFAULT_SKIM_BUY_AT_FLOOR_BPS as u16;
        s.skim_sell_at_floor_bps = DEFAULT_SKIM_SELL_AT_FLOOR_BPS as u16;
        s.skim_buy_far_bps = 200;
        s.skim_sell_far_bps = 200;
        s.creator_buy_bps = 100;
        s.creator_sell_bps = 100;
        s.platform_bps = PLATFORM_BPS;
        s
    }

///
///
///
    fn preset_pool(m: Mode, edge: i32) -> FloorState {
        let p = preset(m);
        let mut s = meme(edge);
        (s.skim_buy_far_bps, s.skim_sell_far_bps) = (p.skim_buy_far_bps, p.skim_sell_far_bps);
        (s.creator_buy_bps, s.creator_sell_bps) = (p.creator_bps, p.creator_bps);
        (s.holder_buy_bps, s.holder_sell_bps) = (p.holder_bps, p.holder_bps);
        s.skim_buy_at_floor_bps = 0;
        s.skim_sell_at_floor_bps =
            crate::constants::skim_sell_at_floor_bps_at(p.skim_sell_far_bps, p.skim_sell_at_floor_multiple_bps as u64);
        s
    }

///
///
    #[test]
    fn the_team_fee_is_a_fifth_part_on_top_and_books_team_owed() {
        let mut s = meme(1200);
        s.team_fee_bps = crate::policy::TEAM_POOL_FEE_BPS_DEFAULT;
        let t = take_fees(&mut s, 10_000_000_000, false, 1200, None).unwrap();
        assert_eq!((t.team_fee, t.skim_fee, t.creator_fee, t.holder_fee, t.cut), (25_000_000, 1_000_000_000, 100_000_000, 0, 25_000_000));
        assert_eq!(t.total, 1_125_000_000);
        assert_eq!((s.team_owed, s.team_fees_paid), (25_000_000, 0));
        assert_eq!((s.vault_eth, s.unpaid_skim, s.platform_owed), (975_000_000, 975_000_000, 25_000_000));
    }

///
///
///
///
    #[test]
    fn the_team_fee_is_the_pools_own_rate_the_cap_a_lower_one_or_zero() {
        use crate::policy::TEAM_POOL_FEE_BPS_MAX;
        for (bps, fee) in [(TEAM_POOL_FEE_BPS_MAX, 50_000_000u64), (10, 10_000_000), (0, 0)] {
            let mut s = meme(1200);
            s.team_fee_bps = bps;
            let t = take_fees(&mut s, 10_000_000_000, false, 1200, None).unwrap();
            assert_eq!((t.team_fee, s.team_owed), (fee, fee), "{bps} bps");
            assert_eq!((t.skim_fee, t.creator_fee, t.cut), (1_000_000_000, 100_000_000, 25_000_000), "{bps} bps: the rest of the split is unchanged");
            assert_eq!(t.total, 1_100_000_000 + fee, "{bps} bps: on top");
            assert_eq!((s.vault_eth, s.platform_owed), (975_000_000, 25_000_000), "{bps} bps: never backing, never the cut");
        }
    }

///
    #[test]
    fn a_tiny_leg_pays_no_team_fee_and_books_nothing() {
        let mut s = meme(0);
        (s.team_fee_bps, s.skim_buy_far_bps, s.skim_buy_at_floor_bps, s.creator_buy_bps) = (25, 0, 0, 0);
        let before = s;
        let t = take_fees(&mut s, 399, true, -100_000, None).unwrap();
        assert_eq!((t.team_fee, t.total), (0, 0));
        assert_eq!(s, before);
        let t = take_fees(&mut s, 400, true, -100_000, None).unwrap();
        assert_eq!((t.team_fee, t.total, s.team_owed), (1, 1, 1));
    }

    #[test]
    fn a_pool_with_no_team_rate_pays_no_team_fee() {
        let mut s = meme(1200);
        let t = take_fees(&mut s, 10_000_000_000, false, 1200, None).unwrap();
        assert_eq!((t.team_fee, s.team_owed, t.total), (0, 0, 1_100_000_000));
    }

    #[test]
    fn team_overflow_is_atomic() {
        let mut s = meme(0);
        (s.team_fee_bps, s.team_owed) = (25, u64::MAX);
        let before = s;
        assert_eq!(take_fees(&mut s, 10_000_000, false, 0, None), Err(FloorMathError::Overflow));
        assert_eq!(s, before);
    }






    #[test]
    fn todays_presets_ramp_only_the_skim() {
        let leg = 1_000_000u64;
        let far = -20_000;
        for (m, far_bps, at_bps, creator) in
            [(Mode::Cook, 200, 300, 75u64), (Mode::Bluechip, 200, 300, 100), (Mode::Moonshot, 200, 300, 50)]
        {
            let mut s = preset_pool(m, 0);
            let sell_far = take_fees(&mut s, leg, false, far, None).unwrap();
            let sell_at = take_fees(&mut s, leg, false, 0, None).unwrap();
            let buy_far = take_fees(&mut s, leg, true, far, None).unwrap();
            let buy_at = take_fees(&mut s, leg, true, 0, None).unwrap();
            assert_eq!((sell_far.skim_bps, sell_at.skim_bps), (far_bps, at_bps), "{m:?} sell far / at the floor");
            assert_eq!((buy_far.skim_bps, buy_at.skim_bps), (far_bps, 0), "{m:?} buy far / at the floor");
            for t in [sell_far, sell_at, buy_far, buy_at] {
                assert_eq!(t.creator_fee, leg * creator / 10_000, "{m:?}: the creator fee never ramps");
                assert_eq!(t.holder_fee, 0, "{m:?}: no preset carries a holder rate");
            }
        }
    }


    #[test]
    fn intake_cuts_2_5_percent_and_floors_dust_to_the_vault() {
        let mut s = meme(0);
        assert_eq!(intake(&mut s, 1_000_000).unwrap(), 975_000);
        assert_eq!((s.platform_owed, s.platform_paid_eth), (25_000, 25_000));

        assert_eq!(intake(&mut s, 39).unwrap(), 39);
        assert_eq!(s.platform_owed, 25_000);
        assert_eq!(intake(&mut s, 40).unwrap(), 39);
        assert_eq!(s.platform_owed, 25_001);
        assert_eq!(intake(&mut s, 0).unwrap(), 0);

        s.platform_bps = 0;
        assert_eq!(intake(&mut s, 1_000_000).unwrap(), 1_000_000);
        assert_eq!(s.platform_owed, 25_001);
    }

    #[test]
    fn intake_overflow_leaves_state_untouched() {
        let mut s = meme(0);
        s.platform_owed = u64::MAX;
        let before = s;
        assert_eq!(intake(&mut s, 1_000_000), Err(FloorMathError::Overflow));
        assert_eq!(s, before);
    }


    #[test]
    fn take_fees_books_skim_cut_and_creator() {
        let mut s = meme(1200);
        assert_eq!(s.floor_tick, 1320);
        let t = take_fees(&mut s, 10_000_000_000, false, 1200, None).unwrap();
        assert_eq!(t.skim_bps, 1000);
        assert_eq!(t.skim_fee, 1_000_000_000);
        assert_eq!(t.cut, 25_000_000);
        assert_eq!(t.kept, 975_000_000);
        assert_eq!(t.creator_fee, 100_000_000);
        assert_eq!(t.total, 1_100_000_000);
        assert_eq!((s.vault_eth, s.unpaid_skim), (975_000_000, 975_000_000));
        assert_eq!((s.platform_owed, s.creator_fees_owed, s.holder_owed), (25_000_000, 100_000_000, 0));

        let mut b = meme(1200);
        let t = take_fees(&mut b, 10_000_000_000, true, 1300, None).unwrap();
        assert_eq!((t.skim_bps, t.skim_fee, t.creator_fee, t.total), (0, 0, 100_000_000, 100_000_000));
        assert_eq!((b.vault_eth, b.unpaid_skim, b.platform_owed), (0, 0, 0));
        assert_eq!(b.creator_fees_owed, 100_000_000);
    }


    #[test]
    fn zero_total_books_nothing() {
        let mut s = meme(0);
        let before = s;
        let t = take_fees(&mut s, 9, false, 0, None).unwrap();
        assert_eq!((t.total, t.skim_bps), (0, 1000));
        assert_eq!(s, before);
    }

    #[test]
    fn creator_and_holder_sides_follow_is_buy() {
        let mut s = meme(0);
        s.creator_buy_bps = 10;
        s.creator_sell_bps = 90;
        s.holder_buy_bps = 20;
        s.holder_sell_bps = 30;
        let buy = take_fees(&mut s, 1_000_000, true, -100_000, None).unwrap();
        assert_eq!((buy.creator_fee, buy.holder_fee), (1_000, 2_000));
        let sell = take_fees(&mut s, 1_000_000, false, -100_000, None).unwrap();
        assert_eq!((sell.creator_fee, sell.holder_fee), (9_000, 3_000));
        assert_eq!(s.holder_owed, 5_000);
    }

    #[test]
    fn take_fees_overflow_is_atomic() {
        let mut s = meme(0);
        s.creator_fees_owed = u64::MAX;
        let before = s;
        assert_eq!(take_fees(&mut s, 10_000_000, false, 0, None), Err(FloorMathError::Overflow));
        assert_eq!(s, before);
        let mut t = meme(0);
        t.floor_tick = i32::MIN + 120;
        assert_eq!(take_fees(&mut t, 1, true, 1, None), Err(FloorMathError::Overflow));


        let mut v = meme(0);
        v.floor_tick = i32::MIN;
        let before = v;
        assert_eq!(take_fees(&mut v, 1, true, 0, None), Err(FloorMathError::Overflow));
        assert_eq!(v, before);

        let mut u = meme(0);
        assert!(take_fees(&mut u, u64::MAX, false, 0, None).is_ok());
    }




    #[test]
    fn distance_is_measured_to_the_dear_edge() {
        let mut s = meme(0);
        let inside = take_fees(&mut s, 1_000_000, false, 60, None).unwrap();
        assert_eq!(inside.skim_bps, 1000, "inside the band is AT the floor");
        let at_cheap = take_fees(&mut s, 1_000_000, false, 120, None).unwrap();
        assert_eq!(at_cheap.skim_bps, 1000);
        let at_dear = take_fees(&mut s, 1_000_000, false, 0, None).unwrap();
        assert_eq!(at_dear.skim_bps, 1000);
        let above = take_fees(&mut s, 1_000_000, false, -60, None).unwrap();

        assert_eq!(above.skim_bps, 997);

        assert_eq!(skim_bps(180, SKIM_RAMP_TICKS, false, 200, 200, 0, 1000), 991);
    }



    #[test]
    fn at_floor_endpoints_are_read_off_the_state() {
        let mut s = meme(0);
        s.skim_sell_at_floor_bps = 400;
        s.skim_buy_at_floor_bps = 50;
        assert_eq!(take_fees(&mut s, 1_000_000, false, 0, None).unwrap().skim_bps, 400);
        assert_eq!(take_fees(&mut s, 1_000_000, true, 0, None).unwrap().skim_bps, 50);
        let mut flat = meme(0);
        flat.skim_sell_far_bps = 500;
        flat.skim_sell_at_floor_bps = 400;
        assert_eq!(take_fees(&mut flat, 1_000_000, false, 0, None).unwrap().skim_bps, 500);


        let mut zero = meme(0);
        zero.skim_sell_at_floor_bps = 0;
        assert_eq!(take_fees(&mut zero, 1_000_000, false, 0, None).unwrap().skim_bps, 200);
    }






///
///
    fn referred(edge: i32) -> FloorState {
        let mut s = meme(edge);

        s.referrer = [7u8; 32];
        s.referral_share_bps = crate::constants::REFERRAL_CREATOR_SHARE_BPS as u16;
        s
    }

///
///
///
///
///
    #[test]
    fn a_referred_pool_splits_the_floored_share_and_the_creator_keeps_the_remainder() {
        let mut s = referred(0);
        s.creator_buy_bps = 100;

        let t = take_fees(&mut s, 10_500, true, -100_000, None).unwrap();
        assert_eq!(t.creator_fee, 105, "creator_fee stays GROSS -- O-4 measures it against the delta");
        assert_eq!(t.referral_fee, 10, "the referral FLOORS");
        assert_eq!(s.creator_fees_owed, 95, "the creator keeps the remainder, dust included");
        assert_eq!(s.referral_owed, 10);
        assert_eq!(s.referral_fees_paid, 10);

        assert_eq!(s.creator_fees_owed + s.referral_owed, t.creator_fee);
    }

///
///
///
///
///
    #[test]
    fn an_unreferred_pool_splits_nothing_even_with_a_share_set() {
        let mut s = meme(0);
        s.creator_buy_bps = 100;

        s.referral_share_bps = crate::constants::REFERRAL_CREATOR_SHARE_BPS as u16;
        assert_eq!(s.referrer, [0u8; 32]);
        let t = take_fees(&mut s, 10_500, true, -100_000, None).unwrap();
        assert_eq!(t.referral_fee, 0);
        assert_eq!(s.referral_owed, 0);
        assert_eq!(s.referral_fees_paid, 0);
        assert_eq!(s.creator_fees_owed, 105, "the creator keeps 100 %");
    }

///
///
///
///
///
///
    #[test]
    fn a_legacy_distributor_pool_splits_nothing_after_the_upgrade() {
        let mut s = meme(0);
        s.creator_buy_bps = 100;
        s.trader_ref_owed = 0xD1D1_D1D1_D1D1_D1D1;
        s.pad7 = [0xD1u8; 16];
        s.referral_share_bps = crate::constants::REFERRAL_CREATOR_SHARE_BPS as u16;
        assert_eq!(s.referrer, [0u8; 32]);
        let t = take_fees(&mut s, 10_500, true, -100_000, None).unwrap();
        assert_eq!(t.referral_fee, 0);
        assert_eq!((s.referral_owed, s.referral_fees_paid), (0, 0));
        assert_eq!(s.creator_fees_owed, 105, "the creator keeps 100 % going forward");

        let mut r = s;
        r.creator_fees_owed = 0;
        r.referrer = [0x5Eu8; 32];
        let t = take_fees(&mut r, 10_500, true, -100_000, None).unwrap();
        assert_eq!(t.referral_fee, 10);
        assert_eq!(r.referral_owed, 10);
    }

///
///
///
///
///
///
    #[test]
    fn the_split_never_changes_what_the_trader_pays() {
        let leg = 10_000_000_000u64;
        let (mut plain, mut refd) = (meme(1200), referred(1200));
        let a = take_fees(&mut plain, leg, false, 1200, None).unwrap();
        let b = take_fees(&mut refd, leg, false, 1200, None).unwrap();
        assert_eq!(a.total, b.total, "the hook delta is unchanged");
        assert_eq!(a.creator_fee, b.creator_fee, "the record's creator figure is GROSS on both");
        assert_eq!((a.skim_fee, a.cut, a.kept), (b.skim_fee, b.cut, b.kept));
        assert_eq!(a.referral_fee, 0);
        assert_eq!(b.referral_fee, 10_000_000);


        assert_eq!(
            plain.quote_liabilities(),
            refd.quote_liabilities(),
            "I-3's quote sum is invariant under the split"
        );
    }

///
///
///
///
///
    #[test]
    fn a_referral_share_past_ten_thousand_is_refused_atomically() {
        let mut s = referred(0);
        s.creator_buy_bps = 100;
        s.referral_share_bps = 10_001;
        let before = s;
        assert_eq!(
            take_fees(&mut s, 10_500, true, -100_000, None),
            Err(FloorMathError::ReferralShareOutOfRange)
        );
        assert_eq!(s, before);

        let mut all = referred(0);
        all.creator_buy_bps = 100;
        all.referral_share_bps = 10_000;
        let t = take_fees(&mut all, 10_500, true, -100_000, None).unwrap();
        assert_eq!((t.creator_fee, t.referral_fee), (105, 105));
        assert_eq!((all.creator_fees_owed, all.referral_owed), (0, 105));
    }

///
///
///
///
///
    #[test]
    fn a_share_that_rounds_to_zero_books_nothing() {
        let mut s = referred(0);
        s.creator_buy_bps = 100;

        let t = take_fees(&mut s, 900, true, -100_000, None).unwrap();
        assert_eq!((t.creator_fee, t.referral_fee), (9, 0));
        assert_eq!((s.creator_fees_owed, s.referral_owed, s.referral_fees_paid), (9, 0, 0));

        let mut z = referred(0);
        z.creator_buy_bps = 100;
        z.referral_share_bps = 0;
        let t = take_fees(&mut z, 10_500, true, -100_000, None).unwrap();
        assert_eq!((t.referral_fee, z.referral_owed, z.creator_fees_owed), (0, 0, 105));
    }

///
///
///
///
    #[test]
    fn referral_overflow_is_atomic() {
        let mut s = referred(0);
        s.creator_buy_bps = 100;
        s.referral_owed = u64::MAX;
        let before = s;
        assert_eq!(take_fees(&mut s, 10_500, true, -100_000, None), Err(FloorMathError::Overflow));
        assert_eq!(s, before, "an overflowing referral leaves the skim booking behind too");

        let mut p = referred(0);
        p.creator_buy_bps = 100;
        p.referral_fees_paid = u64::MAX;
        let before = p;
        assert_eq!(take_fees(&mut p, 10_500, true, -100_000, None), Err(FloorMathError::Overflow));
        assert_eq!(p, before);
    }

///
///
///
///
    #[test]
    fn the_floors_free_buys_win_over_the_gap_multiplier() {
        let ramp = |distance| crate::ramp::skim_bps(distance, SKIM_RAMP_TICKS, true, 100, 100, 0, 1_500);
        for distance in [-500, -120, 0] {
            let off = ramp(distance);
            assert_eq!(off, 0, "today's ramp: a buy at the bid is free");
            for (mode, from, to) in [(1, 152, 305), (1, 1_054, 2_232), (2, 1_054, 2_232), (2, 1_804, 4_005)] {
                assert_eq!(gap_skim_with(off, -50_000, Some(0), mode, from, to), off, "ON = OFF at the bid, gap 50,000 t");
            }
        }
        for distance in [1, 4_000, 8_000, SKIM_RAMP_TICKS - 1] {
            let off = ramp(distance);
            let on = gap_skim_with(off, -50_000, Some(0), 1, 1_054, 2_232);
            assert!(on >= off && on <= off * 3 / 2, "partway down the ramp: at most 1.5 x the ramp ({off} -> {on})");
        }
    }

///
///
    #[test]
    fn the_gap_buy_skim_boosts_only_buys_past_the_gap() {

        let g = |bps, gap: i32, mode, from, to| gap_skim_with(bps, -gap, Some(0), mode, from, to);

        assert_eq!(g(75, 1_053, 1, 513, 1_054), 75);
        assert_eq!(g(75, 1_054, 1, 513, 1_054), 112, "75 x 1.5 floored");
        assert_eq!(g(75, 1_000, 1, 513, 1_054), 75, "mode 1 has no ramp");
        assert_eq!(g(100, 50_000, 1, 1_804, 4_005), 150);
        assert_eq!(g(0, 50_000, 1, 513, 1_054), 0, "the at-floor ramp to 0 stays");
        assert_eq!(gap_skim_with(75, -50_000, None, 1, 513, 1_054), 75, "no cushion standing");
        assert_eq!(g(75, -10, 1, 513, 1_054), 75, "the price under the cushion top");

        assert_eq!(g(1_000, 50_000, 1, 513, 1_054), 1_500);
        assert_eq!(g(1_200, 50_000, 1, 513, 1_054), 1_500);

        assert_eq!(g(100, 513, 2, 513, 1_054), 100);
        assert_eq!(g(100, 1_054, 2, 513, 1_054), 150);
        assert_eq!(g(100, 783, 2, 513, 1_054), 124, "270/541 of the way: m = 12,495");
        let mut last = 100;
        for d in 513..=1_100 {
            let b = g(100, d, 2, 513, 1_054);
            assert!(b >= last && b <= 150, "monotone in the gap");
            last = b;
        }
    }

///
///
///
    #[test]
    fn the_gap_buy_skim_ships_on_at_40_percent() {
        assert_eq!(
            (STAIR_GAP_SKIM, STAIR_GAP_SKIM_TICKS, STAIR_GAP_SKIM_FROM_TICKS, STAIR_GAP_SKIM_MULT_BPS, STAIR_GAP_SKIM_MAX_BPS),
            (2, 5_109, 2_232, 15_000, 1_500)
        );
        assert!(cushion_gap_skim_on());

        for d in [-5_000, 0, 2_000, 2_232] {
            assert_eq!(gap_buy_skim_bps(75, true, -d, Some(0)), 75, "at or under G/2: unchanged at gap {d}");
        }

        for (d, want) in [(2_233, 100), (2_808, 110), (3_670, 124), (4_000, 130), (5_108, 149)] {
            assert_eq!(gap_buy_skim_bps(100, true, -d, Some(0)), want, "inside the ramp: gap {d}");
            assert_eq!(gap_buy_skim_bps(100, false, -d, Some(0)), 100, "a sell is never boosted");
        }
        let mut last = 100;
        for d in 2_232..=5_200 {
            let b = gap_buy_skim_bps(100, true, -d, Some(0));
            assert!(b >= last && b <= 150, "monotone in the gap, at most 1.5 x");
            last = b;
        }
        for d in [5_109, 10_000] {
            assert_eq!(gap_buy_skim_bps(75, true, -d, Some(0)), 112, "past G: 1.5 x at gap {d}");
            assert_eq!(gap_buy_skim_bps(75, false, -d, Some(0)), 75, "a sell is never boosted");
        }
        assert_eq!(gap_buy_skim_bps(75, true, -10_000, None), 75, "no cushion: no boost");
        assert_eq!(gap_buy_skim_bps(0, true, -10_000, Some(0)), 0, "the floor's free buy stays free");
        let mut s = FloorState::default();
        s.tick_spacing = 60;
        s.floor_tick = 200_000;
        s.skim_buy_far_bps = 100;
        s.skim_sell_far_bps = 100;
        s.creator_buy_bps = 50;
        let t = take_fees(&mut s, 1_000_000, true, 0, Some(20_000)).unwrap();
        assert_eq!((t.skim_bps, t.creator_fee), (150, 5_000), "ON: a 20,000-tick gap pays 1.5 x the skim, the creator fee unchanged");
        let mut s2 = s;
        let t2 = take_fees(&mut s2, 1_000_000, true, 0, Some(1_000)).unwrap();
        assert_eq!(t2.skim_bps, 100, "ON: under G/2 the skim is unchanged");
        let mut s3 = s;
        let t3 = take_fees(&mut s3, 1_000_000, true, 0, Some(4_000)).unwrap();
        assert_eq!((t3.skim_bps, t3.creator_fee), (130, 5_000), "ON: a 4,000-tick gap pays the ramp's 1.30 x, the creator fee unchanged");
    }


}
