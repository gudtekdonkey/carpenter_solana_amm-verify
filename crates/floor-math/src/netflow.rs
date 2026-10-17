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
//!
//!
//!
//!

use carpenter_clmm_math::big_num::U256;
use carpenter_clmm_math::full_math::MulDiv;

use crate::constants::BPS;

///
///
///
pub const STAIR_CUSHION_NETFLOW_BPS: u64 = 130;
///
///
pub const STAIR_CUSHION_NETFLOW_RELAX_BPS: u64 = 0;

///
///
///
///
///
///
pub const STAIR_CLIMB_HOLD_REACH_TICKS: i32 = 954;

const _: () = assert!(STAIR_CLIMB_HOLD_REACH_TICKS >= 0);

const _: () = assert!(STAIR_CUSHION_NETFLOW_BPS <= BPS && (STAIR_CUSHION_NETFLOW_BPS != 0 || STAIR_CUSHION_NETFLOW_RELAX_BPS == 0));

///
pub const fn cushion_netflow_on() -> bool {
    STAIR_CUSHION_NETFLOW_BPS != 0
}

///
///
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NetFlow {
    pub a: i128,
    pub b: u64,
}

impl NetFlow {
    pub fn is_empty(&self) -> bool {
        self.b == 0
    }

///
    pub fn mean(&self) -> Option<i128> {
        (self.b != 0).then(|| self.a / self.b as i128)
    }
}

///
///
pub fn mul_div_trunc(x: i128, m: u128, d: u128) -> Option<i128> {
    if d == 0 {
        return None;
    }
    let mag = U256::from(x.unsigned_abs()).mul_div_floor(U256::from(m), U256::from(d))?;
    if mag > U256::from(i128::MAX as u128) {
        return None;
    }
    let mag = mag.as_u128() as i128;
    Some(if x < 0 { -mag } else { mag })
}

///
pub fn share_of_mcap(mcap: u128, bps: u64) -> u64 {
    let x = U256::from(mcap) * U256::from(bps) / U256::from(BPS) + U256::from(1u8);
    if x > U256::from(u64::MAX) {
        u64::MAX
    } else {
        x.as_u64()
    }
}

///
///
pub fn note_flow(f: NetFlow, buy: bool, v: u64, pre: i32, post: i32, mcap: u128) -> NetFlow {
    if !cushion_netflow_on() {
        return f;
    }
    note_flow_with(f, buy, v, pre, post, mcap, STAIR_CUSHION_NETFLOW_BPS, STAIR_CUSHION_NETFLOW_RELAX_BPS)
}

///
///
///
///
#[allow(clippy::too_many_arguments)]
pub fn note_flow_with(f: NetFlow, buy: bool, v: u64, pre: i32, post: i32, mcap: u128, h_bps: u64, k_bps: u64) -> NetFlow {
    if v == 0 {
        return f;
    }
    step(f, buy, v, pre, post, mcap, h_bps, k_bps).unwrap_or_default()
}

#[allow(clippy::too_many_arguments)]
fn step(f: NetFlow, buy: bool, v: u64, pre: i32, post: i32, mcap: u128, h_bps: u64, k_bps: u64) -> Option<NetFlow> {
    let (mut a, mut b) = (f.a, f.b);

    let mid = (pre as i128 + post as i128) / 2;
    if buy {
        let h = share_of_mcap(mcap, h_bps);
        if STAIR_CUSHION_NETFLOW_HALF_RULE && b > 0 && 2 * v as u128 > h as u128 {



            let w = h - h / 2;
            let m = slice_tick(pre, post, w, v)?;
            let keep = h - w;
            if b > keep {
                a = mul_div_trunc(a, keep as u128, b as u128)?;
                b = keep;
            }
            a = a.checked_add((w as i128).checked_mul(m)?)?;
            b = b.checked_add(w)?;
        } else if v >= h {


            if STAIR_CUSHION_NETFLOW_HALF_RULE {
                let m = slice_tick(pre, post, h, v)?;
                (a, b) = ((h as i128).checked_mul(m)?, h);
            } else {
                (a, b) = ((v as i128).checked_mul(mid)?, v);
            }
        } else if b as u128 + v as u128 > h as u128 {

            a = mul_div_trunc(a, (h - v) as u128, b as u128)?;
            b = h - v;
            a = a.checked_add((v as i128).checked_mul(mid)?)?;
            b = b.checked_add(v)?;
        } else {
            a = a.checked_add((v as i128).checked_mul(mid)?)?;
            b = b.checked_add(v)?;
        }
    } else if b > 0 {

        let mut l = a / b as i128;
        if (pre as i128) < l {
            l = pre as i128;
        }
        if v >= b {
            (a, b) = (0, 0);
        } else {
            a = a.checked_sub((v as i128).checked_mul(l)?)?;
            b -= v;
        }
    }
    if k_bps != 0 && b > 0 {

        let k = share_of_mcap(mcap, k_bps);
        let gap = mid.checked_mul(b as i128)?.checked_sub(a)?;
        a = a.checked_add(mul_div_trunc(gap, v as u128, k as u128 + v as u128)?)?;
    }
    Some(NetFlow { a, b })
}

///
///
///
///
pub const STAIR_CUSHION_NETFLOW_HALF_RULE: bool = true;

///
///
///
///
///
pub fn slice_tick(pre: i32, post: i32, w: u64, v: u64) -> Option<i128> {
    if v == 0 {
        return None;
    }
    let w = w.min(v);
    let d = pre as i128 - post as i128;
    let num = d.checked_mul(w as i128)?;
    Some(post as i128 + num / (2 * v as i128))
}

///
///
///
///
///
pub fn flow_cap(top: i32, standing_lower: Option<i32>, f: NetFlow) -> i32 {
    let Some(lower) = standing_lower else { return top };
    let mut l = lower as i128 - 1;
    if f.b > 0 {
        let b = f.b as i128;
        let mut m = f.a / b;
        if f.a % b != 0 && f.a > 0 {
            m += 1;
        }
        if m < l {
            l = m;
        }
    }
    if l > top as i128 {
        l.min(i32::MAX as i128) as i32
    } else {
        top
    }
}

///
///
///
///
///
///
pub fn climb_line_parts(
    sqrt_price_x64: u128,
    tick: i32,
    state: &crate::state::FloorState,
    supply: u64,
    vault_q: u64,
    cushion: Option<(i32, i32, u128)>,
    width: i32,
) -> crate::FloorMathResult<ClimbLine> {
    climb_line_parts_reach(sqrt_price_x64, tick, state, supply, vault_q, cushion, width, STAIR_CLIMB_HOLD_REACH_TICKS)
}

///
///
///
pub fn climb_hold_target(tick: i32, cushion: Option<(i32, i32, u128)>, reach: i32) -> i32 {
    match cushion {
        Some((lower, _, liq)) if reach != 0 && liq != 0 => {
            let r = (lower as i64 - 1 - reach as i64).max(i32::MIN as i64) as i32;
            r.max(tick)
        }
        _ => tick,
    }
}

///
///
#[allow(clippy::too_many_arguments)]
pub fn climb_line_parts_reach(
    sqrt_price_x64: u128,
    tick: i32,
    state: &crate::state::FloorState,
    supply: u64,
    vault_q: u64,
    cushion: Option<(i32, i32, u128)>,
    width: i32,
    reach: i32,
) -> crate::FloorMathResult<ClimbLine> {
    climb_line_core(sqrt_price_x64, tick, state, supply, vault_q, cushion, width, reach, None)
}

///
///
///
///
///
///
///
#[allow(clippy::too_many_arguments)]
pub fn climb_line_within(
    sqrt_price_x64: u128,
    tick: i32,
    state: &crate::state::FloorState,
    supply: u64,
    vault_q: u64,
    cushion: Option<(i32, i32, u128)>,
    width: i32,
    lo: i32,
    hi: i32,
) -> crate::FloorMathResult<Option<i32>> {
    Ok(climb_line_core(sqrt_price_x64, tick, state, supply, vault_q, cushion, width, STAIR_CLIMB_HOLD_REACH_TICKS, Some((lo, hi)))?
        .line)
}

#[allow(clippy::too_many_arguments)]
fn climb_line_core(
    sqrt_price_x64: u128,
    tick: i32,
    state: &crate::state::FloorState,
    supply: u64,
    vault_q: u64,
    cushion: Option<(i32, i32, u128)>,
    width: i32,
    reach: i32,
    bracket: Option<(i32, i32)>,
) -> crate::FloorMathResult<ClimbLine> {
    let mut out = ClimbLine::default();
    let Some((lower, upper, liq)) = cushion else { return Ok(out) };
    let s = state.tick_spacing;

    let to = climb_hold_target(tick, cushion, reach);

    let Some((lo, hi)) = crate::stair::cushion_range_above_w(to, to, s, state.floor_tick, width)? else { return Ok(out) };
    out.spot_range = Some((lo, hi));

    if lower <= lo {
        return Ok(out);
    }

    let q = crate::position::position_amounts(sqrt_price_x64, lower, upper, liq)?.0;
    out.cushion_quote = q;
    if q == 0 {
        return Ok(out);
    }
    let l = crate::position::position_liquidity(sqrt_price_x64, lo, hi, q, 0)?;
    out.liquidity = l;
    if vault_q == 0 {

        out.line = Some(state.floor_tick);
        return Ok(out);
    }
    let more = [(lo, hi, l)];



    let solved = match bracket {
        Some((blo, bhi)) if blo < bhi && blo % s == 0 && bhi % s == 0 => {
            crate::solve::solve_floor_tick_within(vault_q as u128, state, supply, &more, blo, bhi)?
        }
        _ => crate::solve::solve_floor_tick_with_more(vault_q as u128, 0, state, supply, &more)?,
    };
    out.solved = Some(solved);
    out.line = Some(crate::band::publish(solved, s)?);
    Ok(out)
}

///
pub fn climb_line(
    sqrt_price_x64: u128,
    tick: i32,
    state: &crate::state::FloorState,
    supply: u64,
    vault_q: u64,
    cushion: Option<(i32, i32, u128)>,
    width: i32,
) -> crate::FloorMathResult<Option<i32>> {
    Ok(climb_line_parts(sqrt_price_x64, tick, state, supply, vault_q, cushion, width)?.line)
}

///
///
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClimbLine {
    pub spot_range: Option<(i32, i32)>,
    pub cushion_quote: u64,
    pub liquidity: u128,
    pub solved: Option<i32>,
    pub line: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const H: u64 = 130;
///
    const MCAP: u128 = 1_000_000_000_000;

    fn note(f: NetFlow, buy: bool, v: u64, pre: i32, post: i32) -> NetFlow {
        note_flow_with(f, buy, v, pre, post, MCAP, H, 0)
    }

///
///
///
    #[test]
    fn the_netflow_switch_ships_at_130() {
        assert_eq!(STAIR_CUSHION_NETFLOW_BPS, 130, "the net-flow cushion cap ships ON at 130");
        assert_eq!(STAIR_CUSHION_NETFLOW_RELAX_BPS, 0, "its volume catch-up ships at none");
        assert!(STAIR_CUSHION_NETFLOW_HALF_RULE, "the half rule and the last slice ship ON (owner 2026-10-07)");
        assert!(cushion_netflow_on());
        let f = NetFlow { a: -7, b: 3 };
        let out = note_flow(f, true, 1_000, -10, -20, MCAP);
        assert_ne!(out, f, "a buy writes the meter");
        assert_eq!(out, note_flow_with(f, true, 1_000, -10, -20, MCAP, 130, 0));
    }

///
///
    #[test]
    fn vectors_note_flow() {
        let h = share_of_mcap(MCAP, H);
        assert_eq!(h, 13_000_000_001);
        let cases: &[(NetFlow, bool, u64, i32, i32, u128, u64, u64, NetFlow)] = &[

            (NetFlow::default(), true, 1_000, -1_000, -1_100, MCAP, H, 0, NetFlow { a: -1_050_000, b: 1_000 }),

            (NetFlow::default(), true, 10, -1_001, -1_000, MCAP, H, 0, NetFlow { a: -10_000, b: 10 }),

            (NetFlow::default(), true, 10, 7, 8, MCAP, H, 0, NetFlow { a: 70, b: 10 }),

            (NetFlow { a: -1_050_000, b: 1_000 }, true, 3_000, -1_150, -1_250, MCAP, H, 0, NetFlow { a: -4_650_000, b: 4_000 }),

            (NetFlow { a: -4_650_000, b: 4_000 }, false, 1_000, -1_100, -1_050, MCAP, H, 0, NetFlow { a: -3_488_000, b: 3_000 }),

            (NetFlow { a: -4_650_000, b: 4_000 }, false, 1_000, -1_300, -1_250, MCAP, H, 0, NetFlow { a: -3_350_000, b: 3_000 }),

            (NetFlow { a: -4_650_000, b: 4_000 }, false, 4_000, -1_300, -1_000, MCAP, H, 0, NetFlow::default()),

            (NetFlow::default(), false, 4_000, -1_300, -1_000, MCAP, H, 0, NetFlow::default()),


            (
                NetFlow { a: -4_650_000, b: 4_000 },
                true,
                13_000_000_001,
                -2_000,
                -2_100,
                MCAP,
                H,
                0,
                NetFlow { a: -4_650_000 - 6_500_000_001i128 * 2_075, b: 4_000 + 6_500_000_001 },
            ),

            (
                NetFlow { a: -12_000_000_000 * 1_000 - 7, b: 12_000_000_000 },
                true,
                2_000_000_000,
                -1_400,
                -1_600,
                MCAP,
                H,
                0,
                NetFlow { a: (-12_000_000_000i128 * 1_000 - 7) * 11_000_000_001 / 12_000_000_000 - 2_000_000_000 * 1_500, b: 13_000_000_001 },
            ),

            (
                NetFlow { a: -4_650_000, b: 4_000 },
                false,
                1_000,
                -1_100,
                -1_050,
                MCAP,
                H,
                500,
                NetFlow { a: -3_488_000 + (-1_075i128 * 3_000 - -3_488_000) * 1_000 / 50_000_001_001, b: 3_000 },
            ),

            (NetFlow { a: -4_650_000, b: 4_000 }, true, 0, -1, -2, MCAP, H, 500, NetFlow { a: -4_650_000, b: 4_000 }),


            (NetFlow::default(), true, 52_000_000_004, -2_000, -3_000, MCAP, H, 0, NetFlow { a: 13_000_000_001i128 * -2_875, b: 13_000_000_001 }),

            (
                NetFlow { a: -1_000 * 13_000_000_001i128, b: 13_000_000_001 },
                true,
                26_000_000_002,
                -2_000,
                -3_000,
                MCAP,
                H,
                0,
                NetFlow { a: -1_000i128 * 6_500_000_000 + 6_500_000_001i128 * (-3_000 + 1_000 * 6_500_000_001 / 52_000_000_004), b: 13_000_000_001 },
            ),

            (NetFlow { a: -4_650_000, b: 4_000 }, true, 7_000_000_000, -1_000, -1_100, MCAP, H, 0, NetFlow { a: -4_650_000 + 6_500_000_001i128 * -1_054, b: 4_000 + 6_500_000_001 }),
        ];
        for (i, (f, buy, v, pre, post, mcap, h, k, want)) in cases.iter().enumerate() {
            assert_eq!(note_flow_with(*f, *buy, *v, *pre, *post, *mcap, *h, *k), *want, "vector {}", i + 1);
        }
    }

///
///
    #[test]
    fn vectors_flow_cap() {
        let m = |a: i128, b: u64| NetFlow { a, b };
        let cases: &[(i32, Option<i32>, NetFlow, i32)] = &[

            (-2_000, None, m(-5_000_000, 1_000), -2_000),

            (-2_000, Some(-1_500), NetFlow::default(), -1_501),
            (-1_000, Some(-1_500), NetFlow::default(), -1_000),

            (-2_000, Some(-1_500), m(-1_750_500, 1_000), -1_750),
            (-2_000, Some(-1_500), m(-1_750_000, 1_000), -1_750),

            (1_000, Some(2_000), m(1_750_500, 1_000), 1_751),

            (-2_000, Some(-1_500), m(-2_500_000, 1_000), -2_000),

            (-2_000, Some(-1_500), m(-1_000_000, 1_000), -1_501),

            (-500, Some(-1_500), m(-1_000_000, 1_000), -500),
        ];
        for (i, (top, lower, f, want)) in cases.iter().enumerate() {
            assert_eq!(flow_cap(*top, *lower, *f), *want, "vector {}", i + 1);
        }
    }

///
///
///
    #[test]
    fn a_buy_sits_between_its_prices_and_a_sell_never_lifts_the_mean() {
        let f1 = note(NetFlow::default(), true, 1_000_000, -5_000, -5_200);
        let m1 = f1.mean().unwrap();
        assert!((-5_200..=-5_000).contains(&m1));
        for pre in [-6_000, -5_100, -4_000] {
            let f2 = note(f1, false, 300_000, pre, pre + 50);
            assert!(f2.mean().unwrap() >= m1, "a sell from {pre} lifted the mean");
            assert!(f2.b <= f1.b);
        }

        let top = note(f1, true, 500_000, -5_200, -5_300);
        let back = note(top, false, 500_000, -5_300, -5_200);
        assert!(back.mean().unwrap() >= m1, "a round trip lifted the mean");
    }

///
    #[test]
    fn the_meter_remembers_only_the_last_h() {
        let h = share_of_mcap(MCAP, H);
        let mut f = NetFlow::default();
        for i in 0..200 {
            f = note(f, true, h / 30, -10 * i, -10 * i - 10);
            assert!(f.b <= h);
        }
        assert_eq!(f.b, h);

        let mean = f.mean().unwrap();
        assert!((-2_000..=-1_700).contains(&mean), "{mean}");
    }

///
    #[test]
    fn an_overflowing_step_empties_the_meter() {
        let f = NetFlow { a: i128::MAX - 10, b: 2 };
        assert_eq!(note_flow_with(f, true, 1, 400_000, 400_000, MCAP, H, 0), NetFlow::default());
        assert_eq!(mul_div_trunc(-7, 3, 2), Some(-10));
        assert_eq!(mul_div_trunc(7, 3, 2), Some(10));
        assert_eq!(mul_div_trunc(7, 3, 0), None);
        assert_eq!(share_of_mcap(u128::MAX, 10_000), u64::MAX);
    }

    fn climb_state() -> crate::FloorState {
        let mut s: crate::FloorState = bytemuck::Zeroable::zeroed();
        s.tick_spacing = 60;
        s.floor_tick = 60_000;
        s.seed_lower = -443_580;
        s.seed_upper = 58_200;
        s.seed_liq = crate::PodU128::new(2_000_000_000_000_000);
        s.ladder_tokens_remaining = 690_000_000_000_000;
        s.min_sellable = 1_000;
        s
    }

///
///
    #[test]
    fn the_climb_hold_reach_ships_at_954() {
        assert_eq!(STAIR_CLIMB_HOLD_REACH_TICKS, 954);
        assert_eq!(climb_hold_target(40_000, Some((43_200, 44_280, 1)), STAIR_CLIMB_HOLD_REACH_TICKS), 43_200 - 1 - 954);
    }

///
///
    #[test]
    fn the_reach_clamps_the_hold_target() {
        let c = Some((43_200, 44_280, 3u128));
        assert_eq!(climb_hold_target(40_000, c, 0), 40_000, "0 = the full hold");
        assert_eq!(climb_hold_target(40_000, c, 954), 43_200 - 1 - 954);
        assert_eq!(climb_hold_target(40_000, c, 100), 43_099);
        assert_eq!(climb_hold_target(40_000, c, 5_000), 40_000, "a reach past the spot holds for the whole climb");
        assert_eq!(climb_hold_target(40_000, None, 954), 40_000, "no cushion standing");
        assert_eq!(climb_hold_target(40_000, Some((43_200, 44_280, 0)), 954), 40_000, "a cushion with no liquidity");
        assert_eq!(climb_hold_target(i32::MIN + 5, Some((i32::MIN + 2, 0, 1)), 954), i32::MIN + 5, "no underflow");
    }

///
///
///
    #[test]
    fn the_reach_softens_the_climb_line() {
        let s = climb_state();
        let tick = 40_000;
        let sqrt = carpenter_clmm_math::tick_math::get_sqrt_price_at_tick(tick).unwrap();
        let c = Some((43_200, 44_280, 3_000_000_000_000_000u128));
        let full = climb_line_parts_reach(sqrt, tick, &s, 1_000_000_000_000_000, 50_000_000_000, c, 1_080, 0).unwrap();
        assert_eq!(full.spot_range, Some((40_020, 41_100)));
        let reach = climb_line_parts_reach(sqrt, tick, &s, 1_000_000_000_000_000, 50_000_000_000, c, 1_080, 954).unwrap();
        assert_eq!(reach, climb_line_parts(sqrt, tick, &s, 1_000_000_000_000_000, 50_000_000_000, c, 1_080).unwrap(), "the shipped reach is 954");

        assert_eq!(reach.spot_range, Some((42_300, 43_380)));
        assert_eq!(reach.cushion_quote, full.cushion_quote, "the same quote, placed nearer");
        let (fl, rl) = (full.line.unwrap(), reach.line.unwrap());
        assert!(rl <= fl, "the reach holds no more than the full hold: {rl} vs {fl}");
        assert_ne!(reach.liquidity, full.liquidity, "the same quote over a different range");
        let whole = climb_line_parts_reach(sqrt, tick, &s, 1_000_000_000_000_000, 50_000_000_000, c, 1_080, 10_000).unwrap();
        assert_eq!(whole, full, "a reach past the spot is the full hold");
    }

///
///
///
///
///
    #[test]
    fn the_bracketed_climb_line_moves_the_ratchet_as_the_full_one_does() {
        let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut next = |n: u64| -> u64 {
            x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = x;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            (z ^ (z >> 31)) % n
        };
        let sp = 60;
        let band = crate::band::band_ticks(sp).unwrap();
        let (mut held, mut inside, mut both) = (0u32, 0u32, 0u32);
        for _ in 0..4_000 {
            let mut s = climb_state();
            s.floor_tick = (20_000 + next(80_000) as i32) / sp * sp;
            s.seed_upper = s.floor_tick - band - sp * (1 + next(200) as i32);
            s.seed_liq = crate::PodU128::new(1 + next(5_000_000_000_000_000) as u128);
            s.ladder_tokens_remaining = next(900_000_000_000_000);
            let supply = 1_000_000_000_000_000u64;
            let tick = s.floor_tick - band - 1 - next(40_000) as i32;
            let sqrt = carpenter_clmm_math::tick_math::get_sqrt_price_at_tick(tick).unwrap();
            let lower = (tick + 1 + next(20_000) as i32) / sp * sp + sp;
            let width = 1_080;
            let c = Some((lower, lower + width, 1 + next(5_000_000_000_000_000) as u128));
            let vq = (1u64 << next(50)) + next(1 << 20);
            let stand = crate::spacing::ceil_to_spacing(tick + 1 + band, sp).unwrap();
            let hi = s.floor_tick;
            let lo = crate::spacing::floor_to_spacing(stand.min(hi) - band - 2 * sp, sp).unwrap();
            let Ok(full) = climb_line_core(sqrt, tick, &s, supply, vq, c, width, STAIR_CLIMB_HOLD_REACH_TICKS, None) else { continue };
            let within = climb_line_core(sqrt, tick, &s, supply, vq, c, width, STAIR_CLIMB_HOLD_REACH_TICKS, Some((lo, hi))).unwrap();
            assert_eq!(
                within.line,
                climb_line_within(sqrt, tick, &s, supply, vq, c, width, lo, hi).unwrap(),
                "the public entry is the core"
            );
            assert_eq!(full.line.is_some(), within.line.is_some(), "a line in one is a line in the other");
            let (Some(fl), Some(wl)) = (full.line, within.line) else { continue };
            both += 1;
            if let (Some(fs), Some(ws)) = (full.solved, within.solved) {
                assert_eq!(ws, fs.clamp(lo, hi), "the bracketed solve is the full solve clamped");
            }
            let adopt = |l: i32| l.max(stand).min(hi);
            assert_eq!(adopt(wl), adopt(fl), "the ratchet adopts the same: full {fl} vs bracketed {wl}, stand {stand}, floor {hi}");
            if fl > stand && fl < hi {
                inside += 1;
                assert_eq!(wl, fl, "a line strictly inside the bracket is the full one");
            }
            held += (fl > stand) as u32;
        }
        assert!(both > 1_000 && held > 80 && inside > 50, "the states reach the bracket: {both} lines, {held} binding, {inside} inside");
    }
}
