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

use carpenter_clmm_math::tick_math::{get_sqrt_price_at_tick, MAX_TICK, MIN_TICK};

use crate::band::dear_edge;
use crate::error::{FloorMathError, FloorMathResult};
use crate::position::{pol_base, quote_cost};
use crate::spacing::ceil_to_spacing;
use crate::state::FloorState;

///
///
///
///
///
///
///
///
pub fn circulating_supply(state: &FloorState, supply: u64) -> FloorMathResult<u64> {
    supply
        .checked_sub(state.inventory_tokens)
        .and_then(|c| c.checked_sub(state.ladder_tokens_remaining))
        .and_then(|c| c.checked_sub(state.wall_held_tokens))
        .ok_or(FloorMathError::SupplyUnderflow)
}

///
///
pub fn sellable_at(state: &FloorState, supply: u64, tick: i32, extra: u64) -> FloorMathResult<u64> {
    sellable_at_with(state, supply, tick, extra, &[])
}

///
///
///
///
///
pub type MorePosition = (i32, i32, u128);

///
pub fn sellable_at_with(state: &FloorState, supply: u64, tick: i32, extra: u64, more: &[MorePosition]) -> FloorMathResult<u64> {
    let c = circulating_supply(state, supply)?;
    let mut locked = pol_base(tick, state.seed_lower, state.seed_upper, state.seed_liq.get())? as u128
        + extra as u128;
    for &(lower, upper, liq) in more {
        locked += pol_base(tick, lower, upper, liq)? as u128;
    }
    Ok(if locked >= c as u128 { 0 } else { c - locked as u64 })
}

fn cost_at(state: &FloorState, supply: u64, tick: i32, extra: u64, more: &[MorePosition]) -> FloorMathResult<u128> {
    quote_cost(sellable_at_with(state, supply, tick, extra, more)?, tick)
}

///
///
///
///
///
#[derive(Clone)]
pub(crate) struct Steps {
    pub(crate) circulating: u64,
    pub(crate) extra: u64,
///
    pub(crate) locked: [Option<(u128, u128, u128)>; 3],
}

impl Steps {
    pub(crate) fn of(state: &FloorState, supply: u64, extra: u64, more: &[MorePosition]) -> FloorMathResult<Option<Self>> {
        if more.len() > 2 {
            return Ok(None);
        }
        let circulating = circulating_supply(state, supply)?;
        let mut locked = [None; 3];
        locked[0] = crate::position::edges(state.seed_lower, state.seed_upper, state.seed_liq.get())?;
        for (i, &(lower, upper, liq)) in more.iter().enumerate() {
            locked[i + 1] = crate::position::edges(lower, upper, liq)?;
        }
        Ok(Some(Steps { circulating, extra, locked }))
    }

    pub(crate) fn sellable(&self, p: u128) -> FloorMathResult<u64> {
        let mut locked = self.extra as u128;
        for &(a, b, liq) in self.locked.iter().flatten() {
            locked += crate::position::pol_base_at_sqrt(p, a, b, liq)? as u128;
        }
        let c = self.circulating;
        Ok(if locked >= c as u128 { 0 } else { c - locked as u64 })
    }

    fn sellable_at(&self, tick: i32) -> FloorMathResult<u64> {
        self.sellable(get_sqrt_price_at_tick(tick)?)
    }

///
///
///
///
    fn aim(&self, q: u128, lo: i32, hi: i32, s: i32) -> Option<i32> {
        if q == 0 {
            return None;
        }
        const Q64F: f64 = 18_446_744_073_709_551_616.0;
        let sp_lo = get_sqrt_price_at_tick(lo).ok()?;
        let sp_hi = get_sqrt_price_at_tick(hi).ok()?;
        let (p_lo, p_hi) = (sp_lo as f64 / Q64F, sp_hi as f64 / Q64F);
        let c0 = self.circulating as f64 - self.extra as f64;
        let qf = q as f64;

        let mut pos = [(0.0f64, 0.0f64, 0.0f64); 3];
        let mut np = 0;
        let mut cuts = [p_hi; 6];
        let mut n = 0;
        for &(a, b, l) in self.locked.iter().flatten() {
            let (af, bf) = (a as f64 / Q64F, b as f64 / Q64F);
            pos[np] = (af, bf, l as f64);
            np += 1;
            for e in [af, bf] {
                if e > p_lo && e < p_hi {
                    cuts[n] = e;
                    n += 1;
                }
            }
        }
        for i in 1..n {
            let mut j = i;
            while j > 0 && cuts[j - 1] > cuts[j] {
                cuts.swap(j - 1, j);
                j -= 1;
            }
        }
        let mut from = p_lo;
        for k in 0..=n {
            let to = if k < n { cuts[k] } else { p_hi };
            let mid = 0.5 * (from + to);

            let (mut a_, mut b_) = (c0, 0.0f64);
            for &(af, bf, lf) in &pos[..np] {
                if mid >= bf {
                    a_ -= lf * (bf - af);
                } else if mid > af {
                    a_ += lf * af;
                    b_ += lf;
                }
            }

            let sell_to = a_ - b_ * to;
            if k < n && sell_to > qf * to * to {
                from = to;
                continue;
            }
            if k == n && sell_to > qf * to * to {
                return Some(hi);
            }
            if a_ <= 0.0 {
                return None;
            }

            let root = 2.0 * a_ / (b_ + sqrt_f64(b_ * b_ + 4.0 * qf * a_));
            let sp = root * Q64F;
            let t = if sp <= sp_lo as f64 {
                lo
            } else if sp >= sp_hi as f64 {
                hi
            } else {
                carpenter_clmm_math::tick_math::get_tick_at_sqrt_price(sp as u128).ok()? + 1
            };
            return ceil_to_spacing(t, s).ok().map(|g| g.clamp(lo, hi));
        }
        None
    }

    fn cost_at(&self, tick: i32) -> FloorMathResult<u128> {
        let p = get_sqrt_price_at_tick(tick)?;
        crate::position::quote_cost_at_sqrt(self.sellable(p)?, p)
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
pub fn solve_floor_tick_with(
    q: u128,
    extra: u64,
    state: &FloorState,
    supply: u64,
) -> FloorMathResult<i32> {
    solve_counting(q, extra, state, supply).map(|(t, _)| t)
}

///
///
pub fn solve_floor_tick_with_more(
    q: u128,
    extra: u64,
    state: &FloorState,
    supply: u64,
    more: &[MorePosition],
) -> FloorMathResult<i32> {
    solve_counting_with(q, extra, state, supply, more).map(|(t, _)| t)
}

///
pub fn solve_counting(
    q: u128,
    extra: u64,
    state: &FloorState,
    supply: u64,
) -> FloorMathResult<(i32, u32)> {
    solve_counting_with(q, extra, state, supply, &[])
}

fn solve_counting_with(
    q: u128,
    extra: u64,
    state: &FloorState,
    supply: u64,
    more: &[MorePosition],
) -> FloorMathResult<(i32, u32)> {
    let s = state.tick_spacing;
    let mut hi = ceil_to_spacing(MAX_TICK, s)?;
    let mut lo = ceil_to_spacing(MIN_TICK, s)?;
    let Some(steps) = Steps::of(state, supply, extra, more)? else {
        return solve_counting_general(q, extra, state, supply, more, lo, hi);
    };
    if steps.sellable_at(lo)? < state.min_sellable {
        return Ok((dear_edge(state.floor_tick, s)?, 0));
    }
    if steps.cost_at(hi)? > q {
        return Ok((hi, 0));
    }
    let mut n = 0u32;
    while hi - lo > s {
        let mut mid = ceil_to_spacing(lo + (hi - lo) / 2, s)?;
        if mid >= hi {
            mid = hi - s;
        }
        if mid <= lo {
            mid = lo + s;
        }
        if steps.cost_at(mid)? <= q {
            hi = mid;
        } else {
            lo = mid;
        }
        n += 1;
    }
    Ok((hi, n))
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
pub fn solve_floor_tick_within(
    q: u128,
    state: &FloorState,
    supply: u64,
    more: &[MorePosition],
    lo: i32,
    hi: i32,
) -> FloorMathResult<i32> {
    let s = state.tick_spacing;
    let grid_lo = ceil_to_spacing(MIN_TICK, s)?;
    let grid_hi = ceil_to_spacing(MAX_TICK, s)?;

    let (lo, hi) = (lo.max(grid_lo), hi.min(grid_hi));
    if lo >= hi || lo % s != 0 || hi % s != 0 {
        return Err(FloorMathError::InvalidRange);
    }
    let Some(steps) = Steps::of(state, supply, 0, more)? else {

        if sellable_at_with(state, supply, grid_lo, 0, more)? < state.min_sellable {
            return dear_edge(state.floor_tick, s);
        }
        return solve_floor_tick_with_more(q, 0, state, supply, more).map(|t| t.clamp(lo, hi));
    };
    if steps.sellable_at(grid_lo)? < state.min_sellable {
        return dear_edge(state.floor_tick, s);
    }
    let probes = core::cell::Cell::new(0u32);
    within_guided(q, lo, hi, s, grid_lo, &steps, &probes)
}

///
#[doc(hidden)]
pub fn solve_floor_tick_within_probes(
    q: u128,
    state: &FloorState,
    supply: u64,
    more: &[MorePosition],
    lo: i32,
    hi: i32,
) -> FloorMathResult<(i32, u32)> {
    let s = state.tick_spacing;
    let grid_lo = ceil_to_spacing(MIN_TICK, s)?;
    let grid_hi = ceil_to_spacing(MAX_TICK, s)?;
    let (lo, hi) = (lo.max(grid_lo), hi.min(grid_hi));
    if lo >= hi || lo % s != 0 || hi % s != 0 {
        return Err(FloorMathError::InvalidRange);
    }
    let steps = Steps::of(state, supply, 0, more)?.ok_or(FloorMathError::InvalidRange)?;
    if steps.sellable_at(grid_lo)? < state.min_sellable {
        return Ok((dear_edge(state.floor_tick, s)?, 0));
    }
    let probes = core::cell::Cell::new(0u32);
    let t = within_guided(q, lo, hi, s, grid_lo, &steps, &probes)?;
    Ok((t, probes.get()))
}

///
///
///
///
///
///
///
///
fn within_guided(q: u128, lo: i32, hi: i32, s: i32, grid_lo: i32, steps: &Steps, probes: &core::cell::Cell<u32>) -> FloorMathResult<i32> {
    let aim = steps.aim(q, lo, hi, s);
    within_aimed(q, lo, hi, s, grid_lo, aim, |t| {
        probes.set(probes.get() + 1);
        steps.cost_at(t)
    })
}

///
///
pub(crate) fn within_aimed(
    q: u128,
    mut lo: i32,
    mut hi: i32,
    s: i32,
    grid_lo: i32,
    aim: Option<i32>,
    mut cost: impl FnMut(i32) -> FloorMathResult<u128>,
) -> FloorMathResult<i32> {

    let mut lo_out = lo == grid_lo;
    if let Some(g) = aim.map(|g| ceil_to_spacing(g, s)).transpose()?.map(|g| g.clamp(lo, hi)) {

        let down = if g >= hi {
            if cost(hi)? > q {
                return Ok(hi);
            }
            true
        } else if g <= lo {
            if lo_out {
                false
            } else if cost(lo)? <= q {
                return Ok(lo);
            } else {
                lo_out = true;
                false
            }
        } else if cost(g)? <= q {
            hi = g;
            true
        } else {
            lo = g;
            lo_out = true;
            false
        };

        let mut step = s;

        for _ in 0..GALLOP {
            let t = if down { hi - step } else { lo + step };
            if t <= lo || t >= hi {
                break;
            }
            if cost(t)? <= q {
                hi = t;
                if !down {
                    break;
                }
            } else {
                lo = t;
                lo_out = true;
                if down {
                    break;
                }
            }
            step = step.saturating_mul(2);
        }
    }
    while hi - lo > s {
        let mut mid = ceil_to_spacing(lo + (hi - lo) / 2, s)?;
        if mid >= hi {
            mid = hi - s;
        }
        if mid <= lo {
            mid = lo + s;
        }
        if cost(mid)? <= q {
            hi = mid;
        } else {
            lo = mid;
            lo_out = true;
        }
    }
    if !lo_out && cost(lo)? <= q {
        return Ok(lo);
    }
    Ok(hi)
}

///
pub(crate) const GALLOP: u32 = 3;

///
pub(crate) fn sqrt_f64(x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    let mut r = f64::from_bits((x.to_bits() >> 1) + (0x3ff_u64 << 51));
    for _ in 0..5 {
        r = 0.5 * (r + x / r);
    }
    r
}

///
///
fn solve_counting_general(
    q: u128,
    extra: u64,
    state: &FloorState,
    supply: u64,
    more: &[MorePosition],
    mut lo: i32,
    mut hi: i32,
) -> FloorMathResult<(i32, u32)> {
    let s = state.tick_spacing;
    if sellable_at_with(state, supply, lo, extra, more)? < state.min_sellable {
        return Ok((dear_edge(state.floor_tick, s)?, 0));
    }
    if cost_at(state, supply, hi, extra, more)? > q {
        return Ok((hi, 0));
    }
    let mut steps = 0u32;
    while hi - lo > s {

        let mut mid = ceil_to_spacing(lo + (hi - lo) / 2, s)?;
        if mid >= hi {
            mid = hi - s;
        }
        if mid <= lo {
            mid = lo + s;
        }
        if cost_at(state, supply, mid, extra, more)? <= q {
            hi = mid;
        } else {
            lo = mid;
        }
        steps += 1;
    }
    Ok((hi, steps))
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use crate::band::publish;
    use crate::constants::{LADDER, MIN_SELLABLE, POL, SUPPLY, TICKS_PER_DOUBLING};
    use crate::modes::tranche_price_x64;
    use crate::position::position_liquidity;
    use crate::price::open_tick;
    use crate::spacing::floor_to_spacing;
    use carpenter_clmm_math::tick_math::get_sqrt_price_at_tick;

    fn state() -> FloorState {
        let mut s = FloorState::default();
        s.tick_spacing = 60;
        s.floor_tick = ceil_to_spacing(MAX_TICK, 60).unwrap();
        s.min_sellable = MIN_SELLABLE;
        s
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
    fn low_cap_open() -> i32 {
        let t = open_tick(tranche_price_x64(5_000, 100 * 100, 9).unwrap()).unwrap();
        assert_eq!(t, 99_039);
        t
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
    fn graduated(open: i32) -> FloorState {
        let mut s = state();
        s.seed_lower = ceil_to_spacing(MIN_TICK, 60).unwrap();
        s.seed_upper = floor_to_spacing(open, 60).unwrap();
        let p = get_sqrt_price_at_tick(open).unwrap();
        assert!(p >= get_sqrt_price_at_tick(s.seed_upper).unwrap());
        s.seed_liq.set(position_liquidity(p, s.seed_lower, s.seed_upper, 0, POL).unwrap());
        assert!(s.seed_liq.get() > 0, "the POL fixture must lock supply");
        s.ladder_tokens_remaining = LADDER;
        s.inventory_tokens = 5_000_000_000_000;
        s
    }







    #[test]
    fn fixture_pol_locks_supply_above_its_range() {
        let s = graduated(low_cap_open());
        assert_eq!((s.seed_lower, s.seed_upper), (-443_580, 99_000));
        let c = circulating_supply(&s, SUPPLY).unwrap();
        assert_eq!(c, 395_000_000_000_000);
        let above = sellable_at(&s, SUPPLY, s.seed_upper + 60, 0).unwrap();
        let at_bottom = sellable_at(&s, SUPPLY, s.seed_lower, 0).unwrap();
        assert!(above < at_bottom, "{above} !< {at_bottom}");
        assert_eq!(at_bottom, c, "at the tail's bottom edge the POL is all quote: nothing locked");
        assert_eq!(above, SELLABLE_ABOVE);
        assert_eq!(c - above, LOCKED_ABOVE, "the tail locks POL minus the seed's rounding");
        assert!(POL - LOCKED_ABOVE < 1_000, "the rounding loss is dust: 114 units on a 100e12 seed");
        assert_eq!(sellable_at(&s, SUPPLY, s.seed_upper + 60, 1_000).unwrap(), above - 1_000);


        let mut last = u64::MAX;
        for t in (s.seed_lower..=s.seed_upper + 60).step_by(6000) {
            let v = sellable_at(&s, SUPPLY, t, 0).unwrap();
            assert!(v <= last, "tick {t}: {v} > {last}");
            last = v;
        }
    }

///
///
///
///
///
///
///
    const SELLABLE_ABOVE: u64 = 295_000_000_000_114;
    const LOCKED_ABOVE: u64 = 99_999_999_999_886;

///
///
///
    #[test]
    fn the_full_range_pol_locks_at_every_reachable_tick_and_the_solve_still_bisects() {
        let open = low_cap_open();
        let s = graduated(open);
        let lo = ceil_to_spacing(MIN_TICK, 60).unwrap();
        assert_eq!((s.seed_lower, s.seed_upper), (lo, 99_000));
        assert_eq!(lo, -443_580, "the bottom of the usable grid at spacing 60");





        let c = circulating_supply(&s, SUPPLY).unwrap();
        assert_eq!(pol_base(lo, s.seed_lower, s.seed_upper, s.seed_liq.get()).unwrap(), 0);
        assert_eq!(sellable_at(&s, SUPPLY, lo, 0).unwrap(), c);



        let at_upper = sellable_at(&s, SUPPLY, s.seed_upper + 60, 0).unwrap();
        assert!(at_upper < c, "the POL locks above its range ({at_upper} !< {c})");




        let bounded_liq = {


            let mut b = state();
            b.seed_lower = floor_to_spacing(open - TICKS_PER_DOUBLING, 60).unwrap();
            b.seed_upper = floor_to_spacing(open, 60).unwrap();
            let p = get_sqrt_price_at_tick(open).unwrap();
            position_liquidity(p, b.seed_lower, b.seed_upper, 0, POL).unwrap()
        };
        let full_liq = s.seed_liq.get();
        assert!(full_liq * 100 < bounded_liq * 30 && full_liq * 100 > bounded_liq * 29, "{full_liq} vs {bounded_liq}");



        let mut last = u64::MAX;
        for t in (lo..=s.seed_upper + 60).step_by(60 * 97) {
            let v = sellable_at(&s, SUPPLY, t, 0).unwrap();
            assert!(v <= last, "tick {t}: {v} > {last}");
            last = v;
        }



        assert!(cost_at(&s, SUPPLY, lo, 0, &[]).unwrap() > u64::MAX as u128);
        for q in [1u128, 1_000, 27_500_000_000, 10u128.pow(20)] {
            let (t, steps) = solve_counting(q, 0, &s, SUPPLY).unwrap();
            assert!(steps <= 14, "DV-6: {steps} steps at q={q}");
            assert_eq!(t, brute(q, 0, &s, SUPPLY), "q={q}");
        }
    }



    #[test]
    fn solve_from_lo_with_full_supply_and_one_sol_does_not_overflow() {
        let s = state();
        let lo = ceil_to_spacing(MIN_TICK, 60).unwrap();
        assert!(cost_at(&s, SUPPLY, lo, 0, &[]).unwrap() > u64::MAX as u128);
        let (t, steps) = solve_counting(1_000_000_000, 0, &s, SUPPLY).unwrap();
        assert!(steps <= 14, "DV-6: at most 14 steps at spacing 60, took {steps}");
        assert!(quote_cost(SUPPLY, t).unwrap() <= 1_000_000_000);
        assert!(quote_cost(SUPPLY, t - 60).unwrap() > 1_000_000_000);
        for q in [0u128, 1, u64::MAX as u128, u128::MAX] {
            solve_floor_tick_with(q, 0, &s, SUPPLY).unwrap();
        }
    }

///
    fn brute(q: u128, extra: u64, s: &FloorState, supply: u64) -> i32 {
        let hi = ceil_to_spacing(MAX_TICK, s.tick_spacing).unwrap();
        let lo = ceil_to_spacing(MIN_TICK, s.tick_spacing).unwrap();
        if sellable_at(s, supply, lo, extra).unwrap() < s.min_sellable {
            return s.floor_tick;
        }
        let mut t = hi;
        while t - s.tick_spacing > lo && cost_at(s, supply, t - s.tick_spacing, extra, &[]).unwrap() <= q {
            t -= s.tick_spacing;
        }
        t
    }





    #[test]
    fn bisection_matches_a_grid_scan() {
        let s = graduated(low_cap_open());
        for q in [1u128, 1_000, 50_000_000, 27_500_000_000, 1_000_000_000_000, 10u128.pow(20)] {
            for extra in [0u64, 1_000_000_000_000] {
                assert_eq!(
                    solve_floor_tick_with(q, extra, &s, SUPPLY).unwrap(),
                    brute(q, extra, &s, SUPPLY),
                    "q={q} extra={extra}"
                );
            }
        }
    }



    #[test]
    fn undefined_floor_returns_the_dear_edge() {
        let mut s = state();
        s.floor_tick = -12_000;
        s.inventory_tokens = SUPPLY - (MIN_SELLABLE - 1);
        assert_eq!(solve_counting(u128::MAX, 0, &s, SUPPLY).unwrap(), (-12_120, 0));
        assert_eq!(publish(-12_120, 60).unwrap(), -12_000);
        let mut t = state();
        t.floor_tick = -6_000;
        assert_eq!(solve_counting(u128::MAX, SUPPLY - 5, &t, SUPPLY).unwrap(), (-6_120, 0));
        assert_eq!(publish(-6_120, 60).unwrap(), -6_000);

        let mut u = state();
        u.tick_spacing = 200;
        u.floor_tick = -6_000;
        u.inventory_tokens = SUPPLY;
        assert_eq!(solve_counting(u128::MAX, 0, &u, SUPPLY).unwrap(), (-6_200, 0));
    }




    #[test]
    fn publish_of_the_solve_round_trips() {
        let s = graduated(low_cap_open());
        for q in [1_000u128, 50_000_000, 27_500_000_000, 1_000_000_000_000] {
            let solved = solve_floor_tick_with(q, 0, &s, SUPPLY).unwrap();
            let published = publish(solved, 60).unwrap();
            assert_eq!(published, solved + 120, "q={q}: the published floor is one band above the solve");
            assert_eq!(dear_edge(published, 60).unwrap(), solved);
            assert_eq!(publish(dear_edge(published, 60).unwrap(), 60).unwrap(), published);
        }

        let mut u = state();
        u.inventory_tokens = SUPPLY;
        for f in [-12_000, 0, 6_000, 443_520] {
            u.floor_tick = f;
            let solved = solve_floor_tick_with(u128::MAX, 0, &u, SUPPLY).unwrap();
            assert_eq!(publish(solved, 60).unwrap(), f, "floor {f}");
        }
    }




    #[test]
    fn nothing_affordable_returns_hi() {
        let s = state();
        let hi = ceil_to_spacing(MAX_TICK, 60).unwrap();
        assert_eq!(hi, 443_520);
        assert_eq!(cost_at(&s, SUPPLY, hi, 0, &[]).unwrap(), 0);
        assert_eq!(cost_at(&s, u64::MAX, hi, 0, &[]).unwrap(), 1);
        assert_eq!(solve_counting(0, 0, &s, u64::MAX).unwrap(), (hi, 0));
        let (t, steps) = solve_counting(0, 0, &s, SUPPLY).unwrap();
        assert!(steps > 0 && t < hi);
    }


    #[test]
    fn more_backing_never_lowers_the_floor() {
        let s = graduated(low_cap_open());
        let mut last = i32::MAX;
        for e in 0..40 {
            let q = 1u128 << e;
            let t = solve_floor_tick_with(q, 0, &s, SUPPLY).unwrap();
            assert!(t <= last, "q=2^{e}: {t} > {last}");
            last = t;
        }
    }



    #[test]
    fn step_counts_by_spacing() {
        for (spacing, min, max) in [(1, 19, 20), (10, 16, 17), (60, 13, 14), (200, 12, 13)] {
            let mut s = graduated(low_cap_open());
            s.tick_spacing = spacing;
            let mut seen = (u32::MAX, 0);
            for e in 0..64 {
                let (_, n) = solve_counting(1u128 << e, 0, &s, SUPPLY).unwrap();
                if n == 0 {
                    continue;
                }
                seen = (seen.0.min(n), seen.1.max(n));
            }
            assert!(seen.0 >= min && seen.1 <= max, "spacing {spacing}: {seen:?}");
        }
    }

    #[test]
    fn supply_underflow_is_an_error() {
        let mut s = state();
        s.inventory_tokens = 10;
        s.ladder_tokens_remaining = 10;
        assert_eq!(solve_floor_tick_with(1, 0, &s, 19), Err(FloorMathError::SupplyUnderflow));
        assert_eq!(circulating_supply(&s, 20).unwrap(), 0);
    }

///
///
///
///
///
    #[test]
    fn held_tokens_are_outside_circulating_supply() {
        let mut s = FloorState::default();
        s.inventory_tokens = 10;
        s.ladder_tokens_remaining = 600;
        let all_listed = circulating_supply(&s, 1_000).unwrap();
        assert_eq!(all_listed, 390);
        s.ladder_tokens_remaining = 450;
        s.wall_held_tokens = 150;
        assert_eq!(circulating_supply(&s, 1_000).unwrap(), all_listed, "standing + held is the whole deposit");
        s.wall_held_tokens = 151;
        assert_eq!(circulating_supply(&s, 1_000).unwrap(), all_listed - 1, "C falls by held");
        s.wall_held_tokens = 541;
        assert_eq!(circulating_supply(&s, 1_000), Err(FloorMathError::SupplyUnderflow));
    }

///
///
///
///
    #[test]
    fn the_cushion_locks_what_it_would_hold_and_only_raises_the_floor() {
        let open = low_cap_open();
        let s = graduated(open);
        let q: u128 = 20_000_000_000;
        let plain = solve_floor_tick_with(q, 0, &s, SUPPLY).unwrap();
        assert_eq!(solve_floor_tick_with_more(q, 0, &s, SUPPLY, &[]).unwrap(), plain, "no cushion: the old solve");

        let market = open - 2 * TICKS_PER_DOUBLING;
        let (lo, hi) = crate::stair::cushion_range(market, 60).unwrap().unwrap();
        let p = get_sqrt_price_at_tick(market).unwrap();
        let liq = position_liquidity(p, lo, hi, 5_000_000_000, 0).unwrap();
        assert!(liq > 0);

        let at = hi + 60;
        let with = sellable_at_with(&s, SUPPLY, at, 0, &[(lo, hi, liq)]).unwrap();
        let without = sellable_at(&s, SUPPLY, at, 0).unwrap();
        assert!(with < without, "{with} !< {without}");
        assert_eq!(without - with, pol_base(at, lo, hi, liq).unwrap());

        assert_eq!(sellable_at_with(&s, SUPPLY, lo - 60, 0, &[(lo, hi, liq)]).unwrap(), sellable_at(&s, SUPPLY, lo - 60, 0).unwrap());
        let more = solve_floor_tick_with_more(q, 0, &s, SUPPLY, &[(lo, hi, liq)]).unwrap();
        assert!(more <= plain, "the cushion never lowers the floor (a lower tick is a dearer floor): {more} vs {plain}");

        let mut last = i32::MAX;
        for q in [1_000_000_000u128, 5_000_000_000, 20_000_000_000, 80_000_000_000, 400_000_000_000] {
            let t = solve_floor_tick_with_more(q, 0, &s, SUPPLY, &[(lo, hi, liq)]).unwrap();
            assert!(t <= last, "q {q}: {t} > {last}");
            last = t;
        }
    }

///
///
///
///
///
///
    #[test]
    fn a_burn_raises_the_floor_and_never_lowers_it() {
        let s = graduated(low_cap_open());
        let q: u128 = 20_000_000_000;
        let base = solve_floor_tick_with(q, 0, &s, SUPPLY).unwrap();
        let tenth = solve_floor_tick_with(q, 0, &s, SUPPLY - SUPPLY / 10).unwrap();
        assert!(tenth < base, "a 10 % burn backs a strictly dearer floor: {tenth} vs {base}");
        let mut last = base;
        for pct in [1u64, 2, 5, 10, 20, 30] {
            let t = solve_floor_tick_with(q, 0, &s, SUPPLY - SUPPLY / 100 * pct).unwrap();
            assert!(t <= last, "{pct} % burned: {t} > {last}");
            last = t;
        }
    }

///
///
///
///
    #[test]
    fn the_backing_need_is_the_solves_own_line_at_the_published_floor() {
        let open = low_cap_open();
        let s0 = graduated(open);
        let market = open - 2 * TICKS_PER_DOUBLING;
        let p = get_sqrt_price_at_tick(market).unwrap();
        let cushion_at = |lower: i32| {
            let (lo, hi) = crate::stair::cushion_range_from(lower, 60).unwrap().unwrap();
            (lo, hi, position_liquidity(p, lo, hi, 5_000_000_000, 0).unwrap())
        };
        let (lo, _) = crate::stair::cushion_range(market, 60).unwrap().unwrap();
        for floor_tick in [open + 600, open + 3_000, open + 9_000].map(|t| ceil_to_spacing(t, 60).unwrap()) {
            let mut s = s0;
            s.floor_tick = floor_tick;
            for c in [cushion_at(lo), cushion_at(lo + 3_000)] {
                let need = crate::stair::backing_need(&s, SUPPLY, floor_tick, &[c]).unwrap();
                assert!(need > 0);
                let at = publish(solve_floor_tick_with_more(need, 0, &s, SUPPLY, &[c]).unwrap(), 60).unwrap();
                assert!(at <= floor_tick, "need {need} publishes {at}, the floor stands at {floor_tick}");
                let under = publish(solve_floor_tick_with_more(need - 1, 0, &s, SUPPLY, &[c]).unwrap(), 60).unwrap();
                assert!(under > floor_tick, "one lamport less publishes cheaper: {under} vs {floor_tick}");
            }

            let dear = crate::stair::backing_need(&s, SUPPLY, floor_tick, &[cushion_at(lo)]).unwrap();
            let cheap = crate::stair::backing_need(&s, SUPPLY, floor_tick, &[cushion_at(lo + 3_000)]).unwrap();
            assert!(dear > cheap, "{dear} !> {cheap}");
        }
    }

///
///
///
    #[test]
    fn the_fast_step_is_the_step() {
        let s = graduated(low_cap_open());
        let open = low_cap_open();
        let cushion = |lo: i32, w: i32, quote: u64| -> MorePosition {
            let (lo, hi) = (ceil_to_spacing(lo, 60).unwrap(), ceil_to_spacing(lo + w, 60).unwrap());
            let p = get_sqrt_price_at_tick(lo - 60).unwrap();
            (lo, hi, position_liquidity(p, lo, hi, quote, 0).unwrap())
        };
        let sets: [&[MorePosition]; 6] = [
            &[],
            &[(open, open + 600, 0)],
            &[cushion(open + 600, 2_100, 5_000_000_000)],
            &[cushion(open - 1_200, 2_100, 5_000_000_000)],
            &[cushion(open + 9_000, 6_000, 50_000_000_000)],
            &[cushion(open + 600, 2_100, 5_000_000_000), cushion(open + 4_000, 1_200, 1_000_000_000)],
        ];
        let mut checked = 0;
        for more in sets {
            for q in [0u128, 1, 1_000, 50_000_000, 2_750_000_000, 27_500_000_000, 1_000_000_000_000, 10u128.pow(20)] {
                for extra in [0u64, 1_000_000_000_000, 300_000_000_000_000] {
                    let (lo, hi) = (ceil_to_spacing(MIN_TICK, 60).unwrap(), ceil_to_spacing(MAX_TICK, 60).unwrap());
                    let fast = solve_counting_with(q, extra, &s, SUPPLY, more);
                    let general = solve_counting_general(q, extra, &s, SUPPLY, more, lo, hi);
                    assert_eq!(fast, general, "q={q} extra={extra} more={more:?}");
                    checked += 1;
                }
            }
        }
        assert_eq!(checked, 6 * 8 * 3);

        let mut bad = s;
        bad.inventory_tokens = SUPPLY;
        let (lo, hi) = (ceil_to_spacing(MIN_TICK, 60).unwrap(), ceil_to_spacing(MAX_TICK, 60).unwrap());
        assert_eq!(solve_counting_with(1, 0, &bad, SUPPLY, &[]), solve_counting_general(1, 0, &bad, SUPPLY, &[], lo, hi));
    }

///
///
    #[test]
    fn backing_need_at_is_backing_need() {
        let s = graduated(low_cap_open());
        let open = low_cap_open();
        let mut checked = 0;
        for floor_tick in [open + 1_200, open + 6_000, open + 30_000] {
            for range in [None, Some((open + 600, open + 2_700)), Some((open - 600, open + 1_500)), Some((open + 9_000, open + 9_060))] {
                let at = crate::stair::BackingNeed::new(&s, SUPPLY, floor_tick, range).unwrap();
                for price_tick in [open - 3_000, open, open + 1_000] {
                    let p = get_sqrt_price_at_tick(price_tick).unwrap();
                    for quote in [0u64, 1, 1_000_000, 5_000_000_000, 500_000_000_000] {
                        let more: Vec<MorePosition> = match range {
                            Some((lo, hi)) if quote > 0 => {
                                let l = position_liquidity(p, lo, hi, quote, 0).unwrap();
                                if l != 0 { vec![(lo, hi, l)] } else { vec![] }
                            }
                            _ => vec![],
                        };
                        let want = crate::stair::backing_need(&s, SUPPLY, floor_tick, &more).unwrap();
                        assert_eq!(at.with_quote(p, quote).unwrap(), want, "floor {floor_tick} range {range:?} price {price_tick} quote {quote}");

                        if let Some((lo, hi)) = range {
                            let l = position_liquidity(p, lo, hi, quote, 0).unwrap();
                            let want = crate::stair::backing_need(&s, SUPPLY, floor_tick, &[(lo, hi, l)]).unwrap();
                            let none = crate::stair::BackingNeed::new(&s, SUPPLY, floor_tick, None).unwrap();
                            assert_eq!(none.with_range(p, lo, hi, quote).unwrap(), want, "the climb's form");
                        }
                        checked += 1;
                    }
                }
            }
        }
        assert_eq!(checked, 3 * 4 * 3 * 5);
    }

///
///
///
    #[test]
    fn m1_any_aim_is_the_bisection() {

        let s = 60;
        let grid_lo = ceil_to_spacing(MIN_TICK, s).unwrap();
        let mut n = 0;
        for (lo, hi) in [(6_000, 6_600), (6_000, 12_000), (-60_000, 60_000), (grid_lo, grid_lo + 1_200)] {
            for knee in (lo - 120..=hi + 120).step_by(60) {
                for qd in [0u128, 1] {

                    let q = 1_000u128 + qd;
                    let cost = |t: i32| -> FloorMathResult<u128> { Ok(if t < knee { 1_000 + (knee - t) as u128 + 1 } else { 1_000 - ((t - knee) as u128).min(999) }) };
                    let want = {
                        let (mut l, mut h) = (lo, hi);
                        if cost(h).unwrap() > q { h } else if l != grid_lo && cost(l).unwrap() <= q { l } else {
                            while h - l > s { let mut m = ceil_to_spacing(l + (h - l) / 2, s).unwrap(); if m >= h { m = h - s; } if m <= l { m = l + s; } if cost(m).unwrap() <= q { h = m } else { l = m } }
                            h
                        }
                    };
                    for aim in [None, Some(lo - 600), Some(hi + 600)].into_iter().chain((lo..=hi).step_by(60).map(Some)) {
                        assert_eq!(within_aimed(q, lo, hi, s, grid_lo, aim, cost).unwrap(), want, "lo {lo} hi {hi} knee {knee} aim {aim:?}");
                        n += 1;
                    }
                }
            }
        }
        assert!(n > 10_000);
    }

    #[test]
    fn m1_guided_probe_counts() {
        let s = graduated(low_cap_open());
        let open = low_cap_open();
        let c = {
            let (lo, hi) = (ceil_to_spacing(open + 600, 60).unwrap(), ceil_to_spacing(open + 2_700, 60).unwrap());
            (lo, hi, position_liquidity(get_sqrt_price_at_tick(lo - 60).unwrap(), lo, hi, 5_000_000_000, 0).unwrap())
        };
        let sets: [&[MorePosition]; 2] = [&[], &[c]];
        let (mut n, mut tot, mut bis, mut worst) = (0u32, 0u32, 0u32, 0u32);
        for more in sets {
            for q in [50_000_000u128, 2_750_000_000, 27_500_000_000, 1_000_000_000_000, 30_000_000_000_000] {
                let full = solve_floor_tick_with_more(q, 0, &s, SUPPLY, more).unwrap();
                for (dl, dh) in [(-20_000, 600), (-6_000, 6_000), (-600, 18_000), (-40_000, 60), (-120, 30_000)] {
                    let lo = ceil_to_spacing(full + dl, 60).unwrap();
                    let hi = ceil_to_spacing(full + dh, 60).unwrap();
                    let (got, p) = solve_floor_tick_within_probes(q, &s, SUPPLY, more, lo, hi).unwrap();
                    assert_eq!(got, full.clamp(lo, hi));
                    let b = 2 + (((hi - lo) / 60) as f64).log2().ceil() as u32;
                    n += 1; tot += p; bis += b; worst = worst.max(p);
                }
            }
        }
        println!("[m1-guided] {n} solves: {tot} probes guided vs ~{bis} bisection; worst {worst}");
    }

    #[test]
    fn the_bracket_is_the_clamp() {
        let s = graduated(low_cap_open());
        let open = low_cap_open();
        let c = {
            let (lo, hi) = (ceil_to_spacing(open + 600, 60).unwrap(), ceil_to_spacing(open + 2_700, 60).unwrap());
            (lo, hi, position_liquidity(get_sqrt_price_at_tick(lo - 60).unwrap(), lo, hi, 5_000_000_000, 0).unwrap())
        };
        let sets: [&[MorePosition]; 2] = [&[], &[c]];
        let mut checked = 0;
        for more in sets {
            for q in [0u128, 1_000, 50_000_000, 2_750_000_000, 27_500_000_000, 1_000_000_000_000, 10u128.pow(20)] {
                let full = solve_floor_tick_with_more(q, 0, &s, SUPPLY, more).unwrap();
                for (dl, dh) in [(-6_000, -600), (-600, 600), (-60, 60), (0, 60), (-60, 0), (600, 6_000), (-200_000, 200_000), (-1_200, 0), (0, 1_200)] {
                    let lo = ceil_to_spacing(full + dl, 60).unwrap();
                    let hi = ceil_to_spacing(full + dh, 60).unwrap();
                    if lo >= hi {
                        continue;
                    }
                    let got = solve_floor_tick_within(q, &s, SUPPLY, more, lo, hi).unwrap();
                    let gl = ceil_to_spacing(MIN_TICK, 60).unwrap();
                    let gh = ceil_to_spacing(MAX_TICK, 60).unwrap();
                    assert_eq!(got, full.clamp(lo.max(gl), hi.min(gh)), "q={q} bracket [{lo}, {hi}] full {full}");
                    checked += 1;
                }
            }
        }
        assert!(checked > 100, "{checked}");

        let mut none = s;
        none.min_sellable = SUPPLY;
        let full = solve_floor_tick_with_more(1, 0, &none, SUPPLY, &[]).unwrap();
        let (lo, hi) = (ceil_to_spacing(open, 60).unwrap(), ceil_to_spacing(open + 6_000, 60).unwrap());
        assert_eq!(solve_floor_tick_within(1, &none, SUPPLY, &[], lo, hi).unwrap(), full);
        assert!(solve_floor_tick_within(1, &s, SUPPLY, &[], open, open + 6_000).is_err(), "off the grid");
        assert!(solve_floor_tick_within(1, &s, SUPPLY, &[], hi, lo).is_err(), "empty");
    }
}
