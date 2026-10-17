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
use carpenter_clmm_math::tick_math::{get_tick_at_sqrt_price, MAX_TICK, MIN_TICK};

use crate::constants::{BPS, SUPPLY};
use crate::error::{FloorMathError, FloorMathResult};
use crate::price::sqrt_price_from_ratio;
use crate::spacing::{ceil_to_spacing, floor_to_spacing};



///
pub const STAIR_SALE_BPS: u16 = 2_000;
///
///
pub const STAIR_POL_BPS: u16 = 1_000;
///
///
pub const STAIR_STEPS_BPS: u16 = 6_900;
///
pub const STAIR_TAIL_BPS: u16 = 100;
const _: () = assert!(STAIR_SALE_BPS as u64 + STAIR_POL_BPS as u64 + STAIR_STEPS_BPS as u64 + STAIR_TAIL_BPS as u64 == BPS);

///
pub const MIN_STAIR_STEPS: u8 = 1;
pub const MAX_STAIR_STEPS: u8 = 16;
///
///
///
pub const MIN_STAIR_GROWTH_BPS: u16 = 5_000;
pub const MAX_STAIR_GROWTH_BPS: u16 = 20_000;

///
pub const FLOOR_TO_PRICE_BPS: u64 = 2_000;
///
///
///
///
pub const POL_TO_MCAP_BPS: u64 = 1_500;
///
pub const CUSHION_BOTTOM_BPS: u64 = 9_000;
///
pub const POL_RELIST_MULTIPLE: u128 = 2;



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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StairParams {
///
///
    pub steps_bps: u16,
///
    pub tail_bps: u16,
///
    pub floor_to_price_bps: u64,
///
    pub pol_to_mcap_bps: u64,
///
    pub cushion_bottom_bps: u64,
///
///
    pub cushion_ticks: i32,
}

///
pub const STAIR_KIND_PRESET: u8 = 0;
///
pub const STAIR_KIND_GENESIS: u8 = 1;

///
pub const PRESET_STAIR: StairParams = StairParams {
    steps_bps: STAIR_STEPS_BPS,
    tail_bps: STAIR_TAIL_BPS,
    floor_to_price_bps: FLOOR_TO_PRICE_BPS,
    pol_to_mcap_bps: POL_TO_MCAP_BPS,
    cushion_bottom_bps: CUSHION_BOTTOM_BPS,
    cushion_ticks: CUSHION_TICKS,
};

///
///
///
///
pub const GENESIS_STAIR_STEPS_BPS: u16 = 8_975;
///
pub const GENESIS_STAIR_TAIL_BPS: u16 = 100;
///
pub const GENESIS_POL_TO_MCAP_BPS: u64 = 2_000;
///
pub const GENESIS_CUSHION_BOTTOM_BPS: u64 = 7_000;
///
pub const GENESIS_CUSHION_TICKS: i32 = 3_567;
///
///
pub const GENESIS_STAIR_GROWTH_BPS: u16 = 9_500;

///
pub const GENESIS_STAIR: StairParams = StairParams {
    steps_bps: GENESIS_STAIR_STEPS_BPS,
    tail_bps: GENESIS_STAIR_TAIL_BPS,
    floor_to_price_bps: FLOOR_TO_PRICE_BPS,
    pol_to_mcap_bps: GENESIS_POL_TO_MCAP_BPS,
    cushion_bottom_bps: GENESIS_CUSHION_BOTTOM_BPS,
    cushion_ticks: GENESIS_CUSHION_TICKS,
};

///
pub fn stair_params(kind: u8) -> Option<StairParams> {
    match kind {
        STAIR_KIND_PRESET => Some(PRESET_STAIR),
        STAIR_KIND_GENESIS => Some(GENESIS_STAIR),
        _ => None,
    }
}

///
///
///
pub fn stored_stair_params(cushion_bottom_bps: u16, pol_to_mcap_bps: u16, cushion_ticks: i32) -> StairParams {
    StairParams {
        cushion_bottom_bps: cushion_bottom_bps as u64,
        pol_to_mcap_bps: pol_to_mcap_bps as u64,
        cushion_ticks,
        ..PRESET_STAIR
    }
}

///
pub const SPLIT_SCALE: u128 = 1_000_000_000_000_000_000;
///
pub const STEP_WEIGHT_ONE: u128 = 1_000_000_000_000_000_000;



///
///
///
pub fn stair_tail_split(credited: u64) -> (u64, u64) {
    stair_tail_split_for(credited, &PRESET_STAIR)
}

///
///
pub fn stair_tail_split_for(credited: u64, p: &StairParams) -> (u64, u64) {
    let whole = p.steps_bps as u128 + p.tail_bps as u128;
    let tail = if whole == 0 { 0 } else { (credited as u128 * p.tail_bps as u128 / whole) as u64 };
    (credited - tail, tail)
}



///
///
///
///
///
///
pub const CUSTOM_STAIR_TOP_MCAP_USD: u64 = 1_000_000_000;
///
pub const CUSTOM_STAIR_GROWTH_BPS: u16 = 10_000;

///
///
///
///
///
///
///
///
pub fn custom_stair_steps(cap_usd: u64) -> u8 {
    let top = CUSTOM_STAIR_TOP_MCAP_USD as u128;
    let max = MAX_STAIR_STEPS as u32;
    let cap = cap_usd as u128;
    if cap == 0 {
        return MAX_STAIR_STEPS;
    }
    if cap >= top {
        return 1;
    }
    let mut n = 0u32;

    while n < max && (cap << (n + 1)) <= top {
        n += 1;
    }

    if n < max && top * top >= (cap * cap) << (2 * n + 1) {
        n += 1;
    }
    if n == 0 {
        1
    } else {
        n as u8
    }
}

///
///
pub fn custom_stair(cap_usd: u64) -> (u8, u16) {
    (custom_stair_steps(cap_usd), CUSTOM_STAIR_GROWTH_BPS)
}

///
///
///
///
///
///
///
///
pub fn stair_tail_split_supply(credited: u64, p: &StairParams) -> (u64, u64) {
    let tail = (SUPPLY / BPS * p.tail_bps as u64).min(credited);
    (credited - tail, tail)
}

///
pub fn stair_shape_ok(steps: u8, growth_bps: u16) -> bool {
    (MIN_STAIR_STEPS..=MAX_STAIR_STEPS).contains(&steps) && (MIN_STAIR_GROWTH_BPS..=MAX_STAIR_GROWTH_BPS).contains(&growth_bps)
}

///
///
///
pub fn step_tokens(total: u64, steps: u8, growth_bps: u16) -> FloorMathResult<[u64; MAX_STAIR_STEPS as usize]> {
    if !stair_shape_ok(steps, growth_bps) {
        return Err(FloorMathError::BadSplit);
    }
    let n = steps as usize;
    let mut w = [0u128; MAX_STAIR_STEPS as usize];
    w[0] = STEP_WEIGHT_ONE;
    for i in 1..n {
        w[i] = w[i - 1] * growth_bps as u128 / BPS as u128;
    }
    let sum: u128 = w[..n].iter().sum();
    let mut out = [0u64; MAX_STAIR_STEPS as usize];
    let mut given = 0u64;
    for i in 0..n {


        out[i] = match (total as u128).checked_mul(w[i]) {
            Some(p) => (p / sum) as u64,
            None => U256::from(total).mul_div_floor(U256::from(w[i]), U256::from(sum)).ok_or(FloorMathError::Overflow)?.as_u64(),
        };
        given += out[i];
    }
    out[0] += total - given;
    Ok(out)
}



///
///
///
///
///
///
///
pub fn doubling_ticks(i: u8) -> FloorMathResult<i32> {
    if (i as usize) < DOUBLING_TICKS.len() {
        return Ok(DOUBLING_TICKS[i as usize]);
    }
    doubling_ticks_computed(i)
}

///
pub const DOUBLING_TICKS: [i32; MAX_STAIR_STEPS as usize + 1] =
    [0, 6_932, 13_864, 20_796, 27_728, 34_660, 41_591, 48_523, 55_455, 62_387, 69_319, 76_251, 83_182, 90_114, 97_046, 103_978, 110_910];

///
pub fn doubling_ticks_computed(i: u8) -> FloorMathResult<i32> {
    if i == 0 {
        return Ok(0);
    }
    if i > 64 {
        return Err(FloorMathError::Overflow);
    }
    let t = get_tick_at_sqrt_price(sqrt_price_from_ratio(1, 1u128 << i)?)?;
    t.checked_add(1).ok_or(FloorMathError::Overflow)
}

///
///
///
pub fn stair_edge(open_edge: i32, i: u8, spacing: i32) -> FloorMathResult<i32> {
    let off = ceil_to_spacing(doubling_ticks(i)?, spacing)?;
    (open_edge as i64 - off as i64).try_into().map_err(|_| FloorMathError::Overflow)
}

///
pub fn grid_bottom(spacing: i32) -> FloorMathResult<i32> {
    ceil_to_spacing(MIN_TICK, spacing)
}

///
///
pub fn stair_fits(open_edge: i32, steps: u8, spacing: i32) -> FloorMathResult<bool> {
    let top = stair_edge(open_edge, steps, spacing)?;
    Ok(top as i64 - spacing as i64 >= grid_bottom(spacing)? as i64)
}

///
pub fn step_range(open_edge: i32, i: u8, steps: u8, spacing: i32) -> FloorMathResult<(i32, i32)> {
    if i >= steps {
        return Err(FloorMathError::InvalidRange);
    }
    let upper = stair_edge(open_edge, i, spacing)?;
    let lower = stair_edge(open_edge, i + 1, spacing)?;
    if lower >= upper || lower < grid_bottom(spacing)? {
        return Err(FloorMathError::InvalidRange);
    }
    Ok((lower, upper))
}

///
pub fn tail_range(open_edge: i32, steps: u8, spacing: i32) -> FloorMathResult<(i32, i32)> {
    let upper = stair_edge(open_edge, steps, spacing)?;
    let lower = grid_bottom(spacing)?;
    if lower >= upper {
        return Err(FloorMathError::InvalidRange);
    }
    Ok((lower, upper))
}

///
///
///
///
pub fn unsold_range(lower: i32, upper: i32, current: i32, spacing: i32) -> FloorMathResult<Option<(i32, i32)>> {


    let at = floor_to_spacing(current, spacing)?.min(upper);
    Ok(if at > lower && at <= current { Some((lower, at)) } else { None })
}



///
///
pub fn cushion_ticks() -> FloorMathResult<i32> {
    Ok(CUSHION_TICKS)
}

///
pub const CUSHION_TICKS: i32 = 1_054;

///
pub fn cushion_ticks_computed() -> FloorMathResult<i32> {
    cushion_ticks_for(CUSHION_BOTTOM_BPS)
}

///
///
///
pub fn cushion_ticks_for(bottom_bps: u64) -> FloorMathResult<i32> {
    if bottom_bps == 0 || bottom_bps >= BPS {
        return Err(FloorMathError::InvalidRange);
    }
    let t = get_tick_at_sqrt_price(sqrt_price_from_ratio(bottom_bps as u128, BPS as u128)?)?;
    t.checked_add(1).ok_or(FloorMathError::Overflow)
}

///
///
///
///
pub fn cushion_range(current: i32, spacing: i32) -> FloorMathResult<Option<(i32, i32)>> {
    cushion_range_w(current, spacing, CUSHION_TICKS)
}

///
pub fn cushion_range_w(current: i32, spacing: i32, width: i32) -> FloorMathResult<Option<(i32, i32)>> {
    let top = floor_to_spacing(MAX_TICK, spacing)?;
    let lower = ceil_to_spacing(current.checked_add(1).ok_or(FloorMathError::Overflow)?, spacing)?;
    if lower <= current || lower >= top {
        return Ok(None);
    }
    let upper = (lower as i64 + ceil_to_spacing(width, spacing)? as i64).min(top as i64) as i32;
    Ok(if upper > lower { Some((lower, upper)) } else { None })
}

///
///
///
///
pub fn cushion_top_tick(spot_tick: i32, twap_tick: i32) -> i32 {
    spot_tick.max(twap_tick)
}

///
///
pub fn cushion_range_at(spot_tick: i32, twap_tick: i32, spacing: i32) -> FloorMathResult<Option<(i32, i32)>> {
    cushion_range_at_w(spot_tick, twap_tick, spacing, CUSHION_TICKS)
}

///
pub fn cushion_range_at_w(spot_tick: i32, twap_tick: i32, spacing: i32, width: i32) -> FloorMathResult<Option<(i32, i32)>> {
    cushion_range_w(cushion_top_tick(spot_tick, twap_tick), spacing, width)
}

///
///
///
///
///
pub fn cushion_clip(range: Option<(i32, i32)>, dear_edge: i32) -> Option<(i32, i32)> {
    match range {
        Some((lo, hi)) if lo < dear_edge => Some((lo, hi.min(dear_edge))),
        _ => None,
    }
}

///
///
pub fn cushion_range_above(spot_tick: i32, twap_tick: i32, spacing: i32, floor_tick: i32) -> FloorMathResult<Option<(i32, i32)>> {
    cushion_range_above_w(spot_tick, twap_tick, spacing, floor_tick, CUSHION_TICKS)
}

///
pub fn cushion_range_above_w(
    spot_tick: i32,
    twap_tick: i32,
    spacing: i32,
    floor_tick: i32,
    width: i32,
) -> FloorMathResult<Option<(i32, i32)>> {
    Ok(cushion_clip(cushion_range_at_w(spot_tick, twap_tick, spacing, width)?, crate::band::dear_edge(floor_tick, spacing)?))
}

///
///
///
///
///
///
///
pub fn cushion_home_top(spot_tick: i32, floor_tick: i32, spacing: i32, width: i32) -> FloorMathResult<i32> {
    let dear = crate::band::dear_edge(floor_tick, spacing)?;
    let home = (dear as i64 - ceil_to_spacing(width, spacing)? as i64 - 1).max(i32::MIN as i64) as i32;
    Ok(spot_tick.max(home))
}

///
///
///
pub fn relist_range(current: i32, spacing: i32) -> FloorMathResult<Option<(i32, i32)>> {
    let upper = floor_to_spacing(current, spacing)?;
    if upper > current {
        return Ok(None);
    }
    let lower = (upper as i64 - ceil_to_spacing(doubling_ticks(1)?, spacing)? as i64).max(grid_bottom(spacing)? as i64) as i32;
    Ok(if upper > lower { Some((lower, upper)) } else { None })
}



///
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StairSplit {
///
    pub to_pol: u64,
///
    pub to_floor: u64,
///
    pub short_floor: u128,
///
    pub short_pol: u128,
}

///
///
///
pub fn floor_to_price(sqrt_price_x64: u128, floor_sqrt_x64: u128) -> FloorMathResult<u128> {
    if floor_sqrt_x64 == 0 {
        return Ok(0);
    }
    let p2 = U256::from(sqrt_price_x64) * U256::from(sqrt_price_x64);
    let f2 = U256::from(floor_sqrt_x64) * U256::from(floor_sqrt_x64);
    let r = p2.mul_div_floor(U256::from(SPLIT_SCALE), f2).ok_or(FloorMathError::Overflow)?;
    Ok(if r > U256::from(SPLIT_SCALE) { SPLIT_SCALE } else { r.as_u128() })
}

///
///
pub fn quote_to_mcap(quote: u64, sqrt_price_x64: u128, supply: u64) -> FloorMathResult<u128> {
    if supply == 0 {
        return Ok(SPLIT_SCALE);
    }
    let p2 = U256::from(sqrt_price_x64) * U256::from(sqrt_price_x64);
    let num = U256::from(quote) * U256::from(SPLIT_SCALE);
    let den = U256::from(supply) << 128usize;
    let r = p2.mul_div_floor(num, den).ok_or(FloorMathError::Overflow)?;
    Ok(if r > U256::from(SPLIT_SCALE) { SPLIT_SCALE } else { r.as_u128() })
}

///
///
pub fn shortfall(have: u128, target_bps: u64) -> u128 {
    if target_bps == 0 {
        return 0;
    }
    let reached = have.saturating_mul(BPS as u128) / target_bps as u128;
    SPLIT_SCALE.saturating_sub(reached)
}

///
///
///
///
///
pub fn stair_split(
    collected: u64,
    sqrt_price_x64: u128,
    floor_sqrt_x64: u128,
    pol_quote: u64,
    supply: u64,
) -> FloorMathResult<StairSplit> {
    stair_split_for(collected, sqrt_price_x64, floor_sqrt_x64, pol_quote, supply, &PRESET_STAIR)
}

///
///
pub fn stair_split_for(
    collected: u64,
    sqrt_price_x64: u128,
    floor_sqrt_x64: u128,
    pol_quote: u64,
    supply: u64,
    p: &StairParams,
) -> FloorMathResult<StairSplit> {
    let short_floor = shortfall(floor_to_price(sqrt_price_x64, floor_sqrt_x64)?, p.floor_to_price_bps);
    let short_pol = shortfall(quote_to_mcap(pol_quote, sqrt_price_x64, supply)?, p.pol_to_mcap_bps);
    let both = short_floor + short_pol;
    let to_pol = if both == 0 { 0 } else { (collected as u128 * short_pol / both) as u64 };
    Ok(StairSplit { to_pol, to_floor: collected - to_pol, short_floor, short_pol })
}



///
///
///
///
///
///
///
///
pub fn reserve_range(open_edge: i32, from: u8, steps: u8, spacing: i32) -> FloorMathResult<(i32, i32)> {
    if from >= steps {
        return Err(FloorMathError::InvalidRange);
    }
    let (lower, _) = step_range(open_edge, steps - 1, steps, spacing)?;
    let (_, upper) = step_range(open_edge, from, steps, spacing)?;
    Ok((lower, upper))
}

///
///
///
pub fn keeps_reserve(placed: u8, steps: u8) -> bool {
    steps > placed && steps - placed >= 2
}

///
///
///
///
///
///
pub fn reserve_split(left: u64, planned: &[u64; MAX_STAIR_STEPS as usize], j: u8, steps: u8) -> FloorMathResult<(u64, u64)> {
    if j >= steps || steps > MAX_STAIR_STEPS {
        return Err(FloorMathError::InvalidRange);
    }
    let later: u128 = planned[j as usize + 1..steps as usize].iter().map(|&x| x as u128).sum();
    let spare = (left as u128).saturating_sub(later);
    let to_step = spare.min(planned[j as usize] as u128) as u64;
    Ok((to_step, left - to_step))
}



///
///
///
///
pub fn vault_relist_cap_tick(tokens: u64, cost: u64) -> FloorMathResult<Option<i32>> {
    if tokens == 0 || cost == 0 {
        return Ok(None);
    }
    let two_cost = (cost as u128).checked_mul(2).ok_or(FloorMathError::Overflow)?;
    let sqrt = sqrt_price_from_ratio(two_cost, tokens as u128)?;
    Ok(Some(get_tick_at_sqrt_price(sqrt)?))
}

///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VaultListTarget {
///
    Step(u8),
///
    Reserve,
///
    Tail,
///
///
    Hold,
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
pub fn vault_list_target(
    cap: Option<i32>,
    first_open: u8,
    placed: u8,
    own_upper: &[Option<i32>; MAX_STAIR_STEPS as usize],
    reserve_upper: Option<i32>,
    tail_upper: Option<i32>,
) -> VaultListTarget {
    let ok = |upper: i32| cap.map_or(true, |c| upper <= c);
    for i in first_open..placed.min(MAX_STAIR_STEPS) {
        if let Some(u) = own_upper[i as usize] {
            if ok(u) {
                return VaultListTarget::Step(i);
            }
        }
    }
    if let Some(u) = reserve_upper {
        if ok(u) {
            return VaultListTarget::Reserve;
        }
    }
    if let Some(u) = tail_upper {
        if ok(u) {
            return VaultListTarget::Tail;
        }
    }
    VaultListTarget::Hold
}

///
pub const STAIR_SUPPLY_TOKENS: (u64, u64, u64, u64) = (
    SUPPLY / BPS * STAIR_SALE_BPS as u64,
    SUPPLY / BPS * STAIR_POL_BPS as u64,
    SUPPLY / BPS * STAIR_STEPS_BPS as u64,
    SUPPLY / BPS * STAIR_TAIL_BPS as u64,
);



///
///
///
///
///
///
///
///
pub fn backing_need(
    state: &crate::state::FloorState,
    supply: u64,
    floor_tick: i32,
    more: &[crate::solve::MorePosition],
) -> FloorMathResult<u128> {
    let t = crate::band::dear_edge(floor_tick, state.tick_spacing)?;
    crate::position::quote_cost(crate::solve::sellable_at_with(state, supply, t, 0, more)?, t)
}

///
///
///
///
///
pub struct BackingNeed {
    t: u128,
    steps: crate::solve::Steps,
    cushion: Option<(u128, u128)>,
///
///
    fixed: u128,
}

impl BackingNeed {
///
    pub fn new(
        state: &crate::state::FloorState,
        supply: u64,
        floor_tick: i32,
        range: Option<(i32, i32)>,
    ) -> FloorMathResult<Self> {
        let t = crate::band::dear_edge(floor_tick, state.tick_spacing)?;
        let steps = crate::solve::Steps::of(state, supply, 0, &[])?.ok_or(FloorMathError::Overflow)?;
        let cushion = match range {
            Some((lo, hi)) => Some((
                carpenter_clmm_math::tick_math::get_sqrt_price_at_tick(lo)?,
                carpenter_clmm_math::tick_math::get_sqrt_price_at_tick(hi)?,
            )),
            None => None,
        };
        let t = carpenter_clmm_math::tick_math::get_sqrt_price_at_tick(t)?;
        let mut fixed = steps.extra as u128;
        if let Some((a, b, liq)) = steps.locked[0] {
            fixed += crate::position::pol_base_at_sqrt(t, a, b, liq)? as u128;
        }
        Ok(BackingNeed { t, steps, cushion, fixed })
    }

///
    fn need(&self, cushion: Option<(u128, u128, u128)>) -> FloorMathResult<u128> {
        let mut locked = self.fixed;
        if let Some((a, b, liq)) = cushion {
            locked += crate::position::pol_base_at_sqrt(self.t, a, b, liq)? as u128;
        }
        let c = self.steps.circulating;
        let sellable = if locked >= c as u128 { 0 } else { c - locked as u64 };
        crate::position::quote_cost_at_sqrt(sellable, self.t)
    }

///
    pub fn with_quote(&self, price: u128, quote: u64) -> FloorMathResult<u128> {
        let mut c = None;
        if let Some((a, b)) = self.cushion {
            if quote > 0 {
                let liq = crate::position::position_liquidity_at_sqrt(price, a, b, quote, 0)?;
                if liq != 0 {
                    c = Some((a, b, liq));
                }
            }
        }
        self.need(c)
    }

///
///
    pub fn with_range(&self, price: u128, lower: i32, upper: i32, quote: u64) -> FloorMathResult<u128> {
        let a = carpenter_clmm_math::tick_math::get_sqrt_price_at_tick(lower)?;
        let b = carpenter_clmm_math::tick_math::get_sqrt_price_at_tick(upper)?;
        let liq = crate::position::position_liquidity_at_sqrt(price, a, b, quote, 0)?;
        self.need(if liq != 0 { Some((a, b, liq)) } else { None })
    }
}

///
///
///
///
pub fn backing_have(state: &crate::state::FloorState, pending: u64) -> u128 {
    let unpaid = state.unpaid_skim as u128 + pending as u128;
    let bounty = (unpaid * state.bounty_bps as u128 / BPS as u128).min(state.max_bounty as u128);
    (state.vault_eth as u128).saturating_sub(bounty).saturating_sub(state.min_bid as u128)
}

///
///
///
///
pub fn floor_first(collected: u64, need: u128, have: u128, platform_bps: u16) -> u64 {
    if need <= have || collected == 0 {
        return 0;
    }
    let short = need - have;
    let keep = BPS as u128 - (platform_bps as u128).min(BPS as u128 - 1);

    let x = (short * BPS as u128).div_ceil(keep);
    if x >= collected as u128 { collected } else { x as u64 }
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
pub fn floor_first_least(collected: u64, mut surplus: impl FnMut(u64) -> FloorMathResult<i128>) -> FloorMathResult<u64> {
    if collected == 0 {
        return Ok(0);
    }
    let f0 = surplus(0)?;
    if f0 >= 0 {
        return Ok(0);
    }
    let fc = surplus(collected)?;
    if fc <= f0 {
        return Ok(0);
    }
    if fc < 0 {
        return Ok(collected);
    }

    let (mut lo, mut flo, mut hi, mut fhi) = (0u64, f0, collected, fc);
    let mut kept_side = 0i8;
    let mut bisect = false;
    for _ in 0..FLOOR_FIRST_PROBES {
        if hi - lo <= 1 {
            break;
        }
        let width = (hi - lo) as i128;
        let span = fhi - flo;

        let step = if bisect { width / 2 } else { (-flo).checked_mul(width).map(|n| (n + span - 1) / span).unwrap_or(width / 2) };
        let k = lo + (step.clamp(1, width - 1) as u64);
        let f = surplus(k)?;
        if f >= 0 {
            hi = k;
            fhi = f;
            if kept_side == -1 {
                flo = (flo / 2).min(-1);
            }
            kept_side = -1;
        } else {
            lo = k;
            flo = f;
            if kept_side == 1 {
                fhi /= 2;
            }
            kept_side = 1;
        }
        bisect = !bisect && ((hi - lo) as i128) * 2 > width;
    }
    Ok(hi)
}

///
pub const FLOOR_FIRST_PROBES: u32 = 10;

///
///
///
///
///
///
///
pub fn cushion_climb(
    target_lower: i32,
    current_lower: i32,
    spacing: i32,
    mut backed: impl FnMut(i32) -> FloorMathResult<bool>,
) -> FloorMathResult<i32> {
    if target_lower >= current_lower || backed(target_lower)? {
        return Ok(target_lower);
    }

    let (mut lo, mut hi) = (target_lower, current_lower);
    while hi - lo > spacing {
        let mid = floor_to_spacing(lo + (hi - lo) / 2, spacing)?;
        let mid = if mid <= lo { lo + spacing } else { mid };
        if backed(mid)? {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Ok(hi)
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
pub fn open_fixpoint_floor(
    dearest: i32,
    cheapest: i32,
    spacing: i32,
    mut range: impl FnMut(i32) -> FloorMathResult<Option<(i32, i32)>>,
    mut backed: impl FnMut(i32, (i32, i32)) -> FloorMathResult<bool>,
) -> FloorMathResult<Option<i32>> {
    let mut ok = |f: i32| -> FloorMathResult<bool> {
        match range(f)? {
            Some(r) => backed(f, r),
            None => Ok(false),
        }
    };
    if dearest > cheapest || !ok(cheapest)? {
        return Ok(None);
    }
    if ok(dearest)? {
        return Ok(Some(dearest));
    }

    let (mut lo, mut hi) = (dearest, cheapest);
    while hi - lo > spacing {
        let mid = floor_to_spacing(lo + (hi - lo) / 2, spacing)?;
        let mid = if mid <= lo { lo + spacing } else { mid };
        if ok(mid)? {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Ok(Some(hi))
}

///
///
///
///
pub fn cushion_climb_aimed(
    target_lower: i32,
    current_lower: i32,
    spacing: i32,
    aim: Option<i32>,
    mut backed: impl FnMut(i32) -> FloorMathResult<bool>,
) -> FloorMathResult<i32> {
    if target_lower >= current_lower {
        return Ok(target_lower);
    }
    let Some(g) = aim else {
        return cushion_climb(target_lower, current_lower, spacing, backed);
    };

    let (mut lo, mut hi) = (target_lower, current_lower);
    let mut lo_out = false;
    let g = floor_to_spacing(g, spacing)?.clamp(target_lower, current_lower);
    let down = if g <= lo {
        if backed(lo)? {
            return Ok(lo);
        }
        lo_out = true;
        false
    } else if g >= hi {
        true
    } else if backed(g)? {
        hi = g;
        true
    } else {
        lo = g;
        lo_out = true;
        false
    };
    let mut step = spacing;

    for _ in 0..crate::solve::GALLOP {
        let t = if down { hi - step } else { lo + step };
        if t <= lo || t >= hi {
            break;
        }
        if backed(t)? {
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
    if !lo_out {

        if backed(lo)? {
            return Ok(lo);
        }
    }
    while hi - lo > spacing {
        let mid = floor_to_spacing(lo + (hi - lo) / 2, spacing)?;
        let mid = if mid <= lo { lo + spacing } else { mid };
        if backed(mid)? {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Ok(hi)
}

impl BackingNeed {
///
///
///
///
///
    pub fn climb_aim(&self, have: u128, quote: u64, dear: i32, width: i32, spacing: i32) -> Option<i32> {
        const Q64F: f64 = 18_446_744_073_709_551_616.0;
        let tf = self.t as f64 / Q64F;
        let k = self.steps.circulating as f64 - self.fixed as f64 - have as f64 * tf * tf;
        if k <= 0.0 {
            return Some(MIN_TICK);
        }
        if quote == 0 {
            return None;
        }
        let qf = quote as f64;
        let wv = ceil_to_spacing(width, spacing).ok()?;
        let r = carpenter_clmm_math::tick_math::get_sqrt_price_at_tick(wv).ok()? as f64 / Q64F;
        let tick_of = |p: f64| -> Option<i32> {
            let sp = p * Q64F;
            if !(sp >= carpenter_clmm_math::tick_math::MIN_SQRT_PRICE_X64 as f64) {
                return Some(MIN_TICK);
            }
            if sp >= carpenter_clmm_math::tick_math::MAX_SQRT_PRICE_X64 as f64 {
                return Some(MAX_TICK);
            }
            get_tick_at_sqrt_price(sp as u128).ok().map(|t| t + 1)
        };
        let l = tick_of(crate::solve::sqrt_f64(k / (qf * r)))?;
        if l.saturating_add(wv) <= dear {
            return Some(l);
        }
        tick_of(k / (qf * tf))
    }
}

///
///
///
///
///
pub const STAIR_CLEAR_TICKS: i32 = 254;

///
///
pub fn clear_of_ask(ask: i32, band_ticks: i32, spacing: i32) -> FloorMathResult<i32> {
    let t = (ask as i64 + STAIR_CLEAR_TICKS as i64 + band_ticks as i64).min(i32::MAX as i64) as i32;
    ceil_to_spacing(t, spacing)
}

///
///
///
///
///
///
///
pub const RELIST_COST_MARKUP_TICKS: i32 = 296;

///
///
///
pub fn cushion_cost_tick(tokens: u64, cost: u128) -> FloorMathResult<Option<i32>> {
    if tokens == 0 || cost == 0 {
        return Ok(None);
    }
    let sqrt = sqrt_price_from_ratio(cost, tokens as u128)?;
    Ok(Some(get_tick_at_sqrt_price(sqrt)?))
}

///
///
pub fn relist_cost_line(cost_tick: i32, spacing: i32) -> FloorMathResult<i32> {
    floor_to_spacing(cost_tick.saturating_sub(RELIST_COST_MARKUP_TICKS), spacing)
}

///
///
pub fn cushion_range_from(lower: i32, spacing: i32) -> FloorMathResult<Option<(i32, i32)>> {
    cushion_range_from_w(lower, spacing, CUSHION_TICKS)
}

///
pub fn cushion_range_from_w(lower: i32, spacing: i32, width: i32) -> FloorMathResult<Option<(i32, i32)>> {
    cushion_range_w(lower.checked_sub(1).ok_or(FloorMathError::Overflow)?, spacing, width)
}

#[cfg(test)]
mod tests {
///
///
    #[test]
    fn the_open_fixpoint_is_the_dearest_backed_floor() {
        let range = |f: i32| -> FloorMathResult<Option<(i32, i32)>> { Ok(if f > 1_000 { Some((0, f - 120)) } else { None }) };
        for line in [1_060, 1_120, 2_000, 3_000, 4_980, 5_000] {
            let mut probes = 0;
            let got = open_fixpoint_floor(1_000, 5_000, 60, range, |f, _| {
                probes += 1;
                Ok(f >= line)
            })
            .unwrap();
            let want = floor_to_spacing(line + 59, 60).unwrap().max(1_020);
            assert_eq!(got, Some(want.min(5_000)), "line {line}");
            assert!(probes <= 9, "{probes} probes");
        }
        assert_eq!(open_fixpoint_floor(1_000, 5_000, 60, range, |_, _| Ok(false)).unwrap(), None);
        assert_eq!(open_fixpoint_floor(1_000, 5_000, 60, range, |_, _| Ok(true)).unwrap(), Some(1_020));
    }

///
///
///
///
///
    #[test]
    fn the_payee_never_takes_the_solves_reserve() {
        let mut st = crate::state::FloorState::default();
        st.vault_eth = 5_000_000_000;
        st.min_bid = 50_000_000;
        st.bounty_bps = 100;
        st.max_bounty = 50_000_000;
        assert_eq!(backing_have(&st, 0), (st.vault_eth - st.min_bid) as u128, "no skim: the vault less the reserve");
        st.unpaid_skim = 1_000_000_000;
        assert_eq!(backing_have(&st, 0), (st.vault_eth - st.min_bid - 10_000_000) as u128, "…and less the bounty");
        st.unpaid_skim = 0;

        let need = st.vault_eth as u128;
        for collected in [10_000_000u64, 51_282_052, 200_000_000, 1_000_000_000] {
            let keep = floor_first(collected, need, backing_have(&st, 0), crate::policy::PLATFORM_BPS);
            let kept_net = keep - (keep as u128 * crate::policy::PLATFORM_BPS as u128 / BPS as u128) as u64;
            assert!(kept_net >= st.min_bid.min(collected - collected / 40), "{collected}: keep {keep} backs the reserve");
            let (paid, forfeited) = crate::owners::payee_after_floor_first(collected, collected, keep);
            assert!(paid <= collected - keep, "{collected}: the payee is paid only out of what floor first left");
            assert_eq!(paid + forfeited, collected);
        }
    }

///
///
    #[test]
    fn h1_the_cost_line_and_the_relists_line_at_one_point_zero_three_x() {
        assert_eq!(cushion_cost_tick(0, 5).unwrap(), None);
        assert_eq!(cushion_cost_tick(5, 0).unwrap(), None);
        for (tokens, cost) in [(1_000_000_000_000u64, 3_000_000u128), (7_777_777_777, 123_456_789), (10u64.pow(15), 10u128.pow(12))] {
            let c = cushion_cost_tick(tokens, cost).unwrap().unwrap();

            let paid = crate::position::quote_cost(tokens, c).unwrap();
            assert!(paid + 2 >= cost, "the cost line never prices them under what was paid: {paid} < {cost}");
            let line = relist_cost_line(c, 60).unwrap();
            assert_eq!(line % 60, 0);
            let at_line = crate::position::quote_cost(tokens, line).unwrap();
            assert!(at_line * 100 >= cost * 103, "the re-list's line sells at >= 1.03 × the cost: {at_line} vs {cost}");
        }
    }

    use super::*;
    use crate::constants::TICKS_PER_DOUBLING;
    use crate::modes::{preset, tranche_price_x64, Mode};
    use crate::price::open_tick;
    use carpenter_clmm_math::tick_math::get_sqrt_price_at_tick;

    const S: i32 = 60;

    #[test]
    fn m1_any_climb_aim_is_the_climb() {

        let s = 60;
        let (target, current) = (12_000, 18_000);
        let mut n = 0;
        for knee in (target - 120..=current + 120).step_by(60) {
            let backed = |l: i32| -> FloorMathResult<bool> { Ok(l >= knee) };
            let want = cushion_climb(target, current, s, backed).unwrap();
            for aim in [None, Some(target - 600), Some(current + 600), Some(MIN_TICK)].into_iter().chain((target..=current).step_by(60).map(Some)) {
                assert_eq!(cushion_climb_aimed(target, current, s, aim, backed).unwrap(), want, "knee {knee} aim {aim:?}");
                n += 1;
            }
        }
        assert!(n > 5_000);
    }

    #[test]
    fn the_supply_is_twenty_ten_sixty_nine_one() {
        let (sale, pol, steps, tail) = STAIR_SUPPLY_TOKENS;
        assert_eq!(sale + pol + steps + tail, SUPPLY);
        assert_eq!((sale, pol, steps, tail), (200_000_000_000_000, 100_000_000_000_000, 690_000_000_000_000, 10_000_000_000_000));

        assert_eq!(stair_tail_split(steps + tail), (steps, tail), "the exact supply split, to the unit");
        let (s, t) = stair_tail_split(699_999_999_999_997);
        assert_eq!(s + t, 699_999_999_999_997, "conserved at any deposit");
    }

    #[test]
    fn step_tokens_sum_exactly_per_preset() {
        let total = STAIR_SUPPLY_TOKENS.2;
        for (m, steps, growth) in [(Mode::Moonshot, 15u8, 10_000u16), (Mode::Cook, 15, 10_400), (Mode::Bluechip, 14, 10_400)] {
            let p = preset(m);
            assert_eq!((p.stair_steps, p.stair_growth_bps), (steps, growth), "{m:?}'s row");
            let t = step_tokens(total, steps, growth).unwrap();
            assert_eq!(t.iter().sum::<u64>(), total, "{m:?}: the steps hold the staircase exactly");
            assert!(t[steps as usize..].iter().all(|&x| x == 0), "{m:?}: nothing past the last step");
            if growth == BPS as u16 {
                let each = total / steps as u64;
                for (i, &x) in t[..steps as usize].iter().enumerate() {
                    assert!(x.abs_diff(each) < steps as u64, "{m:?} step {i}: {x} vs {each}");
                }
            } else {
                let g = growth as f64 / BPS as f64;
                for i in 1..steps as usize {
                    assert_eq!(t[i] > t[i - 1], g > 1.0, "{m:?}: heavier up (Cook) or down (Bluechip)");
                    let ratio = t[i] as f64 / t[i - 1] as f64;
                    assert!((ratio - g).abs() < 1e-9, "{m:?} step {i}: ratio {ratio}");
                }
            }
        }

        let eq = step_tokens(total, 13, 10_000).unwrap();
        assert_eq!(eq[1], 53_076_923_076_923);
        assert_eq!(eq[0], 53_076_923_076_923 + 1, "the residue on step 0");

        let cook = step_tokens(total, 13, 10_400).unwrap();
        let sum: f64 = (0..13).map(|j| 1.04f64.powi(j)).sum();
        assert!((cook[0] as f64 - total as f64 / sum).abs() < 20.0);
        assert!((cook[12] as f64 / cook[0] as f64 - 1.04f64.powi(12)).abs() < 1e-9);

        let blue = step_tokens(total, 11, 9_880).unwrap();
        let sum: f64 = (0..11).map(|j| 0.988f64.powi(j)).sum();
        assert!((blue[0] as f64 - total as f64 / sum).abs() < 20.0);
        assert!(step_tokens(total, 0, 10_000).is_err() && step_tokens(total, 17, 10_000).is_err());
        assert!(step_tokens(total, 8, 4_999).is_err() && step_tokens(total, 8, 20_001).is_err());
        assert_eq!(step_tokens(0, 8, 9_500).unwrap(), [0; 16]);
        assert_eq!(step_tokens(u64::MAX, 16, 20_000).unwrap().iter().map(|&x| x as u128).sum::<u128>(), u64::MAX as u128);
    }

///
    #[test]
    fn the_doubling_table_is_the_formula() {
        for i in 0..=MAX_STAIR_STEPS {
            assert_eq!(DOUBLING_TICKS[i as usize], doubling_ticks_computed(i).unwrap(), "2^{i}");
            assert_eq!(doubling_ticks(i).unwrap(), doubling_ticks_computed(i).unwrap());
        }
        assert_eq!(doubling_ticks(17).unwrap(), doubling_ticks_computed(17).unwrap());
        assert_eq!(CUSHION_TICKS, cushion_ticks_computed().unwrap());
    }

    #[test]
    fn doubling_ticks_are_the_ceiled_log() {
        assert_eq!(doubling_ticks(0).unwrap(), 0);
        assert_eq!(doubling_ticks(1).unwrap(), TICKS_PER_DOUBLING);
        for i in 1u8..=16 {
            let want = (i as f64 * 2f64.ln() / 1.0001f64.ln()).ceil() as i32;
            assert_eq!(doubling_ticks(i).unwrap(), want, "2^{i}");
        }
    }

    #[test]
    fn the_steps_double_the_price_and_tile_without_gaps() {


        use crate::modes as m;
        let open = open_tick(tranche_price_x64(m::COOK_GRADUATION_MCAP_USD, 200 * 100, 9).unwrap()).unwrap();
        let open_edge = floor_to_spacing(open, S).unwrap();
        assert_eq!((m::MOONSHOT_STAIR_STEPS, m::COOK_STAIR_STEPS, m::BLUECHIP_STAIR_STEPS), (15, 15, 14));
        for (steps, name) in [(m::MOONSHOT_STAIR_STEPS, "Moonshot"), (m::COOK_STAIR_STEPS, "Cook"), (m::BLUECHIP_STAIR_STEPS, "Bluechip")] {
            assert!(stair_fits(open_edge, steps, S).unwrap(), "{name} fits");
            let mut prev_lower = open_edge;
            for i in 0..steps {
                let (lo, hi) = step_range(open_edge, i, steps, S).unwrap();
                assert_eq!(hi, prev_lower, "{name} step {i} starts where the last ended");
                assert!(lo < hi && lo % S == 0 && hi % S == 0);

                let ratio = 1.0001f64.powi(hi - lo);
                assert!((ratio - 2.0).abs() < 0.02, "{name} step {i}: ×{ratio}");

                let exact = open_edge as f64 - i as f64 * 2f64.ln() / 1.0001f64.ln();
                assert!((hi as f64 - exact).abs() <= S as f64, "{name} step {i}: {hi} vs {exact}");
                prev_lower = lo;
            }
            let (tl, tu) = tail_range(open_edge, steps, S).unwrap();
            assert_eq!(tu, prev_lower, "{name}: the tail starts at the staircase's top");
            assert_eq!(tl, grid_bottom(S).unwrap());
            assert!(step_range(open_edge, steps, steps, S).is_err());
        }

        let near_top = grid_bottom(S).unwrap() + ceil_to_spacing(doubling_ticks(11).unwrap(), S).unwrap();
        assert!(!stair_fits(near_top, 11, S).unwrap());
        assert!(stair_fits(near_top + S, 11, S).unwrap());
    }

    #[test]
    fn a_partly_bought_step_keeps_only_what_is_dearer_than_the_market() {
        let (lo, hi) = (60_000, 66_960);
        assert_eq!(unsold_range(lo, hi, 70_000, S).unwrap(), Some((lo, hi)), "not entered: whole");
        assert_eq!(unsold_range(lo, hi, 63_001, S).unwrap(), Some((lo, 63_000)), "entered: the grid line at or dearer than the market");
        assert_eq!(unsold_range(lo, hi, 62_999, S).unwrap(), Some((lo, 62_940)));
        assert_eq!(unsold_range(lo, hi, 62_940, S).unwrap(), Some((lo, 62_940)));
        assert_eq!(unsold_range(lo, hi, 60_059, S).unwrap(), None, "sold through: nothing left");
        assert_eq!(unsold_range(lo, hi, 50_000, S).unwrap(), None);
    }

    #[test]
    fn the_cushion_sits_from_nine_tenths_of_the_price_to_the_price() {
        assert_eq!(cushion_ticks().unwrap(), 1_054);
        for cur in [-200_000, -61, -60, 0, 1, 59, 60, 82_946, 99_039] {
            let (lo, hi) = cushion_range(cur, S).unwrap().unwrap();
            assert!(lo > cur && lo - cur <= S, "{cur}: the first grid line cheaper than the price");
            assert_eq!(hi - lo, 1_080, "ceil_to_spacing(1,054)");

            let p = get_sqrt_price_at_tick(cur).unwrap();
            let (q, b) = crate::position::position_amounts(p, lo, hi, 1_000_000_000_000).unwrap();
            assert!(q > 0 && b == 0, "{cur}: all quote");

            assert!(1.0001f64.powi(-(hi - cur)) <= 0.9 + 1e-9);
        }
        let top = floor_to_spacing(MAX_TICK, S).unwrap();
        assert_eq!(cushion_range(top, S).unwrap(), None, "no room above the top of the grid");
    }

///
///
    #[test]
    fn the_cushion_is_re_placed_under_each_new_price() {
        let mut last: Option<(i32, i32)> = None;
        for cur in (40_000..=90_000).rev().step_by(3_337) {
            let (lo, hi) = cushion_range(cur, S).unwrap().unwrap();
            assert!(lo > cur, "{cur}: under the price");
            assert!(1.0001f64.powi(-(hi - cur)) <= 0.9 + 1e-9 && 1.0001f64.powi(-(hi - cur)) > 0.88, "{cur}: to about 0.9 x");
            if let Some((plo, phi)) = last {
                assert!(lo < plo && hi < phi, "a dearer market moves the cushion dearer");
            }
            last = Some((lo, hi));
        }

        let (lo, _) = cushion_range(95_000, S).unwrap().unwrap();
        assert!(lo > 95_000);
    }

///
///
///
///
///
    #[test]
    fn a_pump_ratchet_dump_in_one_transaction_does_not_move_the_cushion_up() {
        let settled = 80_000;
        let before = cushion_range_at(settled, settled, S).unwrap().unwrap();
        for pumped in [79_000, 73_068, 60_000, 40_000] {

            let naive = cushion_range(pumped, S).unwrap().unwrap();
            assert!(naive.0 < before.0, "the old rule would have lifted it (the hole)");
            let fixed = cushion_range_at(pumped, settled, S).unwrap().unwrap();
            assert_eq!(fixed, before, "pumped to {pumped}: the cushion does not move up");

            assert!(fixed.0 > pumped);
        }

        let climbed = cushion_range_at(70_000, 70_060, S).unwrap().unwrap();
        assert!(climbed.0 < before.0 && climbed.0 > 70_060);

        let fell = cushion_range_at(85_000, settled, S).unwrap().unwrap();
        assert_eq!(fell, cushion_range(85_000, S).unwrap().unwrap());
        assert_eq!(cushion_top_tick(85_000, settled), 85_000);
    }

    #[test]
    fn the_relist_sits_from_the_price_to_twice_the_price() {
        for cur in [-100_000, -1, 0, 59, 82_946] {
            let (lo, hi) = relist_range(cur, S).unwrap().unwrap();
            assert!(hi <= cur && cur - hi < S);
            assert_eq!(hi - lo, 6_960);
            let p = get_sqrt_price_at_tick(cur).unwrap();
            let (q, b) = crate::position::position_amounts(p, lo, hi, 1_000_000_000_000).unwrap();
            assert!(q == 0 && b > 0, "{cur}: all base");
        }
        let bottom = grid_bottom(S).unwrap();
        assert_eq!(relist_range(bottom + 10, S).unwrap(), None);
        assert_eq!(relist_range(bottom + 100, S).unwrap(), Some((bottom, bottom + 60)), "clamped at the top of the range");
    }

    fn sqrt_at_price_multiple(base: i32, m: f64) -> u128 {

        get_sqrt_price_at_tick(base - (m.ln() / 1.0001f64.ln()).round() as i32).unwrap()
    }

    #[test]
    fn the_split_follows_the_two_shortfalls() {
        let floor_tick = 90_000;
        let f = get_sqrt_price_at_tick(floor_tick).unwrap();
        let supply = SUPPLY;

        let p = sqrt_at_price_multiple(floor_tick, 10.0);
        let r = floor_to_price(p, f).unwrap();
        assert!((r as f64 / SPLIT_SCALE as f64 - 0.1).abs() < 1e-4, "{r}");
        let sf = shortfall(r, FLOOR_TO_PRICE_BPS);
        assert!((sf as f64 / SPLIT_SCALE as f64 - 0.5).abs() < 1e-3);

        let mcap = {
            let p2 = U256::from(p) * U256::from(p);
            ((U256::from(supply) << 128usize) / p2).as_u128() as f64
        };

        let pol_q = (mcap * 0.075) as u64;
        let sp = stair_split(1_000_000, p, f, pol_q, supply).unwrap();
        assert!((sp.short_pol as f64 / SPLIT_SCALE as f64 - 0.5).abs() < 1e-3);
        assert!((sp.to_pol as i64 - 500_000).abs() < 1_000, "{sp:?}");
        assert_eq!(sp.to_pol + sp.to_floor, 1_000_000, "never over-allocates, never loses a unit");

        let sp = stair_split(1_000_000, p, f, (mcap * 0.151) as u64, supply).unwrap();
        assert_eq!((sp.short_pol, sp.to_pol, sp.to_floor), (0, 0, 1_000_000));

        let sp = stair_split(1_000_000, p, f, (mcap * 0.12) as u64, supply).unwrap();
        assert!((sp.short_pol as f64 / SPLIT_SCALE as f64 - 0.2).abs() < 1e-3, "{sp:?}");

        let near = sqrt_at_price_multiple(floor_tick, 4.0);
        let sp = stair_split(777, near, f, u64::MAX / 2, supply).unwrap();
        assert_eq!((sp.short_floor, sp.short_pol, sp.to_pol, sp.to_floor), (0, 0, 0, 777), "both met: all to the floor");

        let sp = stair_split(777, near, f, 0, supply).unwrap();
        assert_eq!((sp.short_floor, sp.short_pol, sp.to_pol, sp.to_floor), (0, SPLIT_SCALE, 777, 0));

        let sp = stair_split(1_000, p, 0, 0, supply).unwrap();
        assert_eq!((sp.short_floor, sp.short_pol), (SPLIT_SCALE, SPLIT_SCALE));
        assert_eq!(sp.to_pol, 500, "both fully short: half each, rounded down to the POL");

        for c in [0u64, 1, 2, 3, 999, u64::MAX] {
            let sp = stair_split(c, p, f, pol_q, supply).unwrap();
            assert_eq!(sp.to_pol + sp.to_floor, c);
            let exact = c as u128 * sp.short_pol / (sp.short_floor + sp.short_pol);
            assert_eq!(sp.to_pol as u128, exact);
        }
    }

    #[test]
    fn the_ratios_are_bounded_and_monotone() {
        let f = get_sqrt_price_at_tick(90_000).unwrap();
        assert_eq!(floor_to_price(f, f).unwrap(), SPLIT_SCALE, "price on the floor: 100 %");
        assert_eq!(floor_to_price(get_sqrt_price_at_tick(95_000).unwrap(), f).unwrap(), SPLIT_SCALE, "under the floor: capped");
        let mut last = SPLIT_SCALE;
        for t in (0..90_000).rev().step_by(7_000) {
            let r = floor_to_price(get_sqrt_price_at_tick(t).unwrap(), f).unwrap();
            assert!(r <= last);
            last = r;
        }
        assert_eq!(quote_to_mcap(0, f, SUPPLY).unwrap(), 0);
        assert_eq!(quote_to_mcap(u64::MAX, f, 1).unwrap(), SPLIT_SCALE);
        assert_eq!(shortfall(0, 500), SPLIT_SCALE);
        assert_eq!(shortfall(SPLIT_SCALE / 20, 500), 0, "exactly at the 5 % target: met");
        assert_eq!(shortfall(SPLIT_SCALE / 40, 500), SPLIT_SCALE / 2);
    }

///
///
///
///
///
    #[test]
    fn the_staircase_conserves_every_token_across_ratchets() {
        use crate::position::{position_amounts, position_liquidity};
        let open = open_tick(tranche_price_x64(50_000, 200 * 100, 9).unwrap()).unwrap();
        let open_edge = floor_to_spacing(open, S).unwrap();
        let steps = 11u8;
        let (stair_total, tail_total) = stair_tail_split(STAIR_SUPPLY_TOKENS.2 + STAIR_SUPPLY_TOKENS.3);
        let want = step_tokens(stair_total, steps, 10_000).unwrap();
        let p0 = get_sqrt_price_at_tick(open).unwrap();

        let mut pos: Vec<(i32, i32, u128)> = Vec::new();
        let mut placed = 0u64;
        for i in 0..steps {
            let (lo, hi) = step_range(open_edge, i, steps, S).unwrap();
            let liq = position_liquidity(p0, lo, hi, 0, want[i as usize]).unwrap();
            let (q, b) = position_amounts(p0, lo, hi, liq).unwrap();
            assert_eq!(q, 0, "step {i} is token-only at the open");
            assert!(b <= want[i as usize] && want[i as usize] - b < 100, "step {i}: placed {b} of {} (liquidity rounds down: dust, held)", want[i as usize]);
            placed += b;
            pos.push((lo, hi, liq));
        }
        let (tl, tu) = tail_range(open_edge, steps, S).unwrap();
        let tail_liq = position_liquidity(p0, tl, tu, 0, tail_total).unwrap();
        let held0 = stair_total - placed;
        let mut held = held0;
        let mut collected_total = 0u64;
        let mut first_open = 0usize;

        for jump in [3_000, 9_100, 400, 20_000, 7_777, 13_000, 25_000] {
            let cur = pos[first_open.min(pos.len() - 1)].1.min(open) - jump;
            let p = get_sqrt_price_at_tick(cur).unwrap();

            let mut i = first_open;
            while i < pos.len() {
                let (lo, hi, liq) = pos[i];
                if cur >= hi {
                    break;
                }
                let (q, b) = position_amounts(p, lo, hi, liq).unwrap();
                collected_total += q;
                match unsold_range(lo, hi, cur, S).unwrap() {
                    Some((nlo, nhi)) => {
                        let nliq = position_liquidity(p, nlo, nhi, 0, b).unwrap();
                        let (q2, b2) = position_amounts(p, nlo, nhi, nliq).unwrap();
                        assert_eq!(q2, 0, "a re-opened step is token-only");
                        assert!(b2 <= b);
                        held += b - b2;
                        pos[i] = (nlo, nhi, nliq);
                        break;
                    }
                    None => {
                        held += b;
                        pos[i] = (lo, hi, 0);
                        first_open = i + 1;
                        i += 1;
                    }
                }
            }

            let standing: u64 = pos.iter().map(|&(lo, hi, liq)| position_amounts(p, lo, hi, liq).unwrap().1).sum();
            let sold: u64 = stair_total - held - standing;


            assert!(standing + held <= stair_total);
            if sold > 0 {
                assert!(collected_total > 0, "sold {sold} tokens for no quote");
            }
            assert!(held - held0 < 1_000, "re-opening loses only dust: {}", held - held0);
        }
        assert!(first_open >= 1, "the climb sold through at least one step");

        assert_eq!(position_amounts(get_sqrt_price_at_tick(tu + 60).unwrap(), tl, tu, tail_liq).unwrap().0, 0);
    }

///
///
    #[test]
    fn the_reserve_spans_every_later_step_exactly() {
        let open = open_tick(tranche_price_x64(200_000, 200 * 100, 9).unwrap()).unwrap();
        let open_edge = floor_to_spacing(open, S).unwrap();
        for n in [2u8, 8, 9, 11] {
            let (lo, hi) = reserve_range(open_edge, 1, n, S).unwrap();
            assert_eq!(hi, step_range(open_edge, 0, n, S).unwrap().0, "{n}: starts where step 0 ends");
            assert_eq!(lo, tail_range(open_edge, n, S).unwrap().1, "{n}: ends where the tail starts");
            for from in 1..n {
                let (l, h) = reserve_range(open_edge, from, n, S).unwrap();
                assert_eq!((l, h), (step_range(open_edge, n - 1, n, S).unwrap().0, step_range(open_edge, from, n, S).unwrap().1));
            }
            assert!(reserve_range(open_edge, n, n, S).is_err());
        }
        assert!(keeps_reserve(1, 3) && !keeps_reserve(1, 2) && !keeps_reserve(2, 2) && keeps_reserve(0, 2));
    }

///
///
///
    #[test]
    fn a_single_liquidity_over_the_reserve_is_thicker_low() {
        use crate::position::{position_amounts, position_liquidity};
        let open = open_tick(tranche_price_x64(200_000, 200 * 100, 9).unwrap()).unwrap();
        let open_edge = floor_to_spacing(open, S).unwrap();
        let n = 9u8;
        let (lo, hi) = reserve_range(open_edge, 1, n, S).unwrap();
        let p0 = get_sqrt_price_at_tick(open).unwrap();
        let liq = position_liquidity(p0, lo, hi, 0, 1_000_000_000_000).unwrap();
        let mut last = u64::MAX;
        for j in 1..n {
            let (l, h) = step_range(open_edge, j, n, S).unwrap();
            let b = position_amounts(p0, l, h, liq).unwrap().1;
            assert!(b < last, "step {j}: {b} < {last}");
            if last != u64::MAX {
                let r = b as f64 / last as f64;
                assert!((r - 0.5f64.sqrt()).abs() < 0.01, "step {j}: ratio {r}");
            }
            last = b;
        }
    }

///
///
    #[test]
    fn the_split_conserves_tokens_and_keeps_the_later_steps_planned() {
        let total = STAIR_SUPPLY_TOKENS.2;
        for (n, g) in [(9u8, 10_000u16), (8, 9_500), (11, 10_000), (2, 10_000), (13, 10_400), (11, 9_880), (14, 10_000)] {
            let planned = step_tokens(total, n, g).unwrap();
            let later: u64 = planned[2..n as usize].iter().sum();
            let whole = planned[1] + later;

            assert_eq!(reserve_split(whole, &planned, 1, n).unwrap(), (planned[1], later), "{n}");

            let sold = planned[1] / 3;
            assert_eq!(reserve_split(whole - sold, &planned, 1, n).unwrap(), (planned[1] - sold, later));

            if n > 2 {
                let (a, b) = reserve_split(later - 5, &planned, 1, n).unwrap();
                assert_eq!((a, b), (0, later - 5));
            }

            let (a, b) = reserve_split(whole + 777, &planned, 1, n).unwrap();
            assert_eq!((a, b), (planned[1], later + 777));
            for left in [0u64, 1, whole / 2, whole, u64::MAX] {
                let (a, b) = reserve_split(left, &planned, 1, n).unwrap();
                assert_eq!(a + b, left, "conserved");
                assert!(a <= planned[1]);
            }

            assert_eq!(reserve_split(planned[n as usize - 1] + 9, &planned, n - 1, n).unwrap(), (planned[n as usize - 1], 9));
            assert!(reserve_split(1, &planned, n, n).is_err());
        }
    }

///
///
    #[test]
    fn the_vaults_tokens_list_at_no_less_than_twice_their_cost() {

        let cap = vault_relist_cap_tick(1_000, 500).unwrap().unwrap();
        assert!(cap <= 0 && cap > -2, "{cap}");

        for t in [cap, cap - 1, cap - 6_932] {
            assert!(1.0001f64.powi(-t) >= 1.0 - 1e-9, "{t}");
        }
        assert!(1.0001f64.powi(-(cap + 1)) < 1.0, "one tick cheaper is under the line");

        assert!(vault_relist_cap_tick(1_000, 5_000).unwrap().unwrap() < cap);
        assert_eq!(vault_relist_cap_tick(1_000, 0).unwrap(), None, "free tokens: no line");
        assert_eq!(vault_relist_cap_tick(0, 5).unwrap(), None);

        let mut own = [None; MAX_STAIR_STEPS as usize];
        own[2] = Some(80_000);
        own[3] = Some(73_080);
        let (reserve, tail) = (Some(66_120), Some(10_000));
        use VaultListTarget::*;

        assert_eq!(vault_list_target(Some(85_000), 2, 4, &own, reserve, tail), Step(2));
        assert_eq!(vault_list_target(Some(80_000), 2, 4, &own, reserve, tail), Step(2), "its cheap edge exactly at 2 × cost");
        assert_eq!(vault_list_target(None, 2, 4, &own, reserve, tail), Step(2), "free tokens: the step above the price");

        assert_eq!(vault_list_target(Some(79_999), 2, 4, &own, reserve, tail), Step(3));
        assert_eq!(vault_list_target(Some(70_000), 2, 4, &own, reserve, tail), Reserve);
        assert_eq!(vault_list_target(Some(60_000), 2, 4, &own, reserve, tail), Tail, "inside the reserve past its first step: the tail");
        assert_eq!(vault_list_target(Some(9_999), 2, 4, &own, reserve, tail), Hold);

        own[2] = None;
        assert_eq!(vault_list_target(Some(85_000), 2, 4, &own, reserve, tail), Step(3));
        assert_eq!(vault_list_target(Some(85_000), 4, 4, &own, None, tail), Tail);
        assert_eq!(vault_list_target(Some(85_000), 4, 4, &own, None, None), Hold);
    }

///
///
///
///
    #[test]
    fn graduation_in_the_buy_then_splits_conserve_every_token() {
        use crate::position::{position_amounts, position_liquidity};
        let open = open_tick(tranche_price_x64(200_000, 200 * 100, 9).unwrap()).unwrap();
        let open_edge = floor_to_spacing(open, S).unwrap();
        let n = 9u8;
        let (stair_total, _) = stair_tail_split(STAIR_SUPPLY_TOKENS.2 + STAIR_SUPPLY_TOKENS.3);
        let planned = step_tokens(stair_total, n, 10_000).unwrap();
        let p0 = get_sqrt_price_at_tick(open).unwrap();
        let place = |p: u128, lo: i32, hi: i32, tokens: u64| -> (u128, u64) {
            let liq = position_liquidity(p, lo, hi, 0, tokens).unwrap();
            let (q, b) = position_amounts(p, lo, hi, liq).unwrap();
            assert_eq!(q, 0, "token-only");
            (liq, b)
        };

        let (s0lo, s0hi) = step_range(open_edge, 0, n, S).unwrap();
        let (l0, b0) = place(p0, s0lo, s0hi, planned[0]);
        let (rlo, rhi) = reserve_range(open_edge, 1, n, S).unwrap();
        let rtok: u64 = planned[1..n as usize].iter().sum();
        let (lr, br) = place(p0, rlo, rhi, rtok);
        let mut held = (planned[0] - b0) + (rtok - br);

        let mut own: Vec<(u8, i32, i32, u128)> = vec![(0, s0lo, s0hi, l0)];
        let mut reserve = Some((rlo, rhi, lr));
        let mut placed = 1u8;
        let mut sold = 0u64;
        let mut tick = open;
        for jump in [4_000, 5_000, 900, 12_000, 3_000, 9_000, 20_000] {
            tick -= jump;
            let p = get_sqrt_price_at_tick(tick).unwrap();

            if let Some(k) = own.iter().position(|o| o.3 != 0 && tick < o.2) {
                let (i, lo, hi, liq) = own[k];
                let before = position_amounts(p0.min(get_sqrt_price_at_tick(hi).unwrap()), lo, hi, liq).unwrap().1;
                let left = position_amounts(p, lo, hi, liq).unwrap().1;
                sold += before - left;
                match unsold_range(lo, hi, tick, S).unwrap() {
                    Some((nl, nu)) => {
                        let (nliq, used) = place(p, nl, nu, left);
                        held += left - used;
                        own[k] = (i, nl, nu, nliq);
                    }
                    None => {
                        held += left;
                        own[k].3 = 0;
                    }
                }
            } else if let Some((lo, hi, liq)) = reserve.filter(|r| tick < r.1) {
                let before = position_amounts(get_sqrt_price_at_tick(hi).unwrap(), lo, hi, liq).unwrap().1;
                let left = position_amounts(p, lo, hi, liq).unwrap().1;
                sold += before - left;
                let j = placed;
                let (to_step, rest) = reserve_split(left, &planned, j, n).unwrap();
                let (sl, su) = step_range(open_edge, j, n, S).unwrap();
                let mut rest = rest;
                match unsold_range(sl, su, tick, S).unwrap() {
                    Some((nl, nu)) if to_step > 0 => {
                        let (nliq, used) = place(p, nl, nu, to_step);
                        rest += to_step - used;
                        own.push((j, nl, nu, nliq));
                    }
                    _ => rest += to_step,
                }
                placed = j + 1;
                let (nlo, nhi) = (step_range(open_edge, n - 1, n, S).unwrap().0, step_range(open_edge, placed, n, S).unwrap().1);
                match unsold_range(nlo, nhi, tick, S).unwrap() {
                    Some((nl, nu)) if rest > 0 && placed < n => {
                        let (nliq, used) = place(p, nl, nu, rest);
                        held += rest - used;
                        reserve = Some((nl, nu, nliq));
                    }
                    _ => {
                        held += rest;
                        reserve = None;
                    }
                }
            }


            let full = |lo: i32, hi: i32, liq: u128| position_amounts(get_sqrt_price_at_tick(hi).unwrap(), lo, hi, liq).unwrap().1;
            let mut standing: u64 = own.iter().filter(|o| o.3 != 0).map(|o| full(o.1, o.2, o.3)).sum();
            if let Some((lo, hi, liq)) = reserve {
                standing += full(lo, hi, liq);
            }
            let _ = p;
            assert!(standing + sold + held <= stair_total, "never more than was deposited");
            assert!(stair_total - (standing + sold + held) < 1_000, "conserved to rounding: {}", stair_total - (standing + sold + held));
        }
        assert!(placed >= 2, "the climb split at least one step off the reserve: placed {placed}");
    }

///
///
///
    #[test]
    fn floor_first_takes_the_least_that_backs_the_floor() {
        assert_eq!(floor_first(1_000, 500, 500, 250), 0, "backed already");
        assert_eq!(floor_first(1_000, 500, 900, 250), 0);
        assert_eq!(floor_first(0, 900, 500, 250), 0, "nothing collected");
        assert_eq!(floor_first(1_000, 1_000_000, 0, 250), 1_000, "capped at the collection (the pump window)");
        for (short, bps) in [(1u128, 250u16), (39, 250), (40, 250), (1_234_567, 250), (987_654_321_987, 250), (77, 0), (10_000, 9_999)] {
            let x = floor_first(u64::MAX, short, 0, bps);
            let kept = |x: u64| x - crate::fees::part(x, bps as u64).unwrap();
            assert!(kept(x) as u128 >= short, "short {short} bps {bps}: x {x} keeps {}", kept(x));

            let keep = (BPS - bps as u64) as u128;
            assert!((x as u128) * keep <= short * BPS as u128 + keep, "short {short} bps {bps}: {x} overshoots");
        }
    }

///
///
///
    #[test]
    fn the_cushion_climbs_to_the_dearest_backed_line() {
        let cur = 60_000;

        let mut asked = 0;
        assert_eq!(cushion_climb(cur + 600, cur, S, |_| { asked += 1; Ok(false) }).unwrap(), cur + 600);
        assert_eq!(cushion_climb(cur, cur, S, |_| { asked += 1; Ok(false) }).unwrap(), cur);
        assert_eq!(asked, 0);

        let mut asked = 0;
        assert_eq!(cushion_climb(cur - 6_000, cur, S, |_| { asked += 1; Ok(true) }).unwrap(), cur - 6_000);
        assert_eq!(asked, 1);

        assert_eq!(cushion_climb(cur - 6_000, cur, S, |_| Ok(false)).unwrap(), cur);

        for k in [cur - 5_999, cur - 3_000, cur - 2_970, cur - 60, cur - 1] {
            let mut asked = 0;
            let got = cushion_climb(cur - 6_000, cur, S, |l| {
                asked += 1;
                assert_eq!(l.rem_euclid(S), 0, "every question on the grid");
                Ok(l >= k)
            })
            .unwrap();
            assert_eq!(got, ceil_to_spacing(k, S).unwrap(), "k {k}");
            assert!(asked <= 9, "k {k}: {asked} questions");
        }
        assert_eq!(cushion_range_from(cur, S).unwrap(), cushion_range(cur - 1, S).unwrap());
        assert_eq!(cushion_range_from(cur, S).unwrap().unwrap().0, cur);
    }

///
///
    #[test]
    fn floor_first_least_finds_the_root_in_a_few_questions() {
        assert_eq!(floor_first_least(0, |_| Ok(-5)).unwrap(), 0);
        assert_eq!(floor_first_least(1_000, |_| Ok(3)).unwrap(), 0, "the split alone backs it");
        assert_eq!(floor_first_least(1_000, |k| Ok(-100 - k as i128)).unwrap(), 0, "keeping more cannot help");
        assert_eq!(floor_first_least(1_000, |k| Ok(-1_000 + k as i128 / 2)).unwrap(), 1_000, "the pump window");
        for (c, root, num, den) in [(1_000u64, 377u64, 1i128, 1i128), (10_000_000_000, 6_123_456_789, 3, 7), (5_000, 1, 9, 10), (5_000, 4_999, 1, 3)] {
            for noise in [0i128, 1, 3] {
                let mut asked = 0;
                let f = |k: u64| -> i128 { (k as i128 - root as i128) * num / den - if (k % 2) == 1 { noise } else { 0 } };
                let k = floor_first_least(c, |k| {
                    asked += 1;
                    Ok(f(k))
                })
                .unwrap();
                assert!(f(k) >= 0, "c {c} root {root} noise {noise}: k {k} is not backed");
                assert!(k <= root + (noise * den / num.max(1)) as u64 + 8, "c {c} root {root} noise {noise}: k {k} over-takes");
                assert!(asked as u32 <= 2 + FLOOR_FIRST_PROBES, "{asked} questions");
            }
        }
    }

///
///
    #[test]
    fn the_cushion_never_starts_inside_the_bid_band() {
        let floor = 60_000;
        let dear = crate::band::dear_edge(floor, S).unwrap();
        assert_eq!(dear, floor - 120);

        let far = cushion_range(40_000, S).unwrap();
        assert_eq!(cushion_range_above(40_000, 40_000, S, floor).unwrap(), far);

        let near = cushion_range(dear - 600, S).unwrap().unwrap();
        assert!(near.1 > dear, "the fixture straddles the band");
        assert_eq!(cushion_range_above(dear - 600, dear - 600, S, floor).unwrap(), Some((near.0, dear)));

        assert_eq!(cushion_range_above(40_000, dear - 600, S, floor).unwrap(), Some((near.0, dear)));

        assert_eq!(cushion_range_above(dear - 1, dear - 1, S, floor).unwrap(), None);
        assert_eq!(cushion_range_above(floor + 5_000, floor + 5_000, S, floor).unwrap(), None);
        assert_eq!(cushion_clip(Some((dear - 60, dear + 900)), dear), Some((dear - 60, dear)), "one spacing wide at the least");
        assert_eq!(cushion_clip(None, dear), None);
    }

///
///
///
    #[test]
    fn floor_first_least_holds_on_a_bent_surplus() {

        let c: u64 = 140_166_068_720;
        let bent = |k: u64| -> i128 {
            let k = k as i128;
            let have = 92_384_416_155i128 + k * 975 / 1_000 - (k * 100 / 1_000).min(50_000_000);
            let need = 94_696_710_012i128 - k * 27 / 100;
            have - need
        };
        let concave = |k: u64| -> i128 {
            let x = k as f64 / c as f64;
            (((1.0 - (1.0 - x).powi(3)) * 3.0e9) as i128) - 2_300_000_000
        };
        let least = |f: &dyn Fn(u64) -> i128| -> u64 {
            let (mut lo, mut hi) = (0u64, c);
            while hi - lo > 1 {
                let m = lo + (hi - lo) / 2;
                if f(m) >= 0 { hi = m } else { lo = m }
            }
            hi
        };



        let flat = |k: u64| -> i128 { if k >= 3_393_803_600 { 0 } else { -1 } };
        for (name, f, tol) in [("bent", &bent as &dyn Fn(u64) -> i128, 4u64), ("concave", &concave, 64), ("flat", &flat, c)] {
            let truth = least(f);
            let mut asked = 0u32;
            let k = floor_first_least(c, |k| {
                asked += 1;
                Ok(f(k))
            })
            .unwrap();
            assert!(f(k) >= 0, "{name}: {k} is not backed");
            assert!(k != c, "{name}: never the whole collection when less backs it (truth {truth})");
            assert!(k >= truth && k - truth <= tol, "{name}: {k} vs the least {truth}");
            assert!(asked <= 2 + FLOOR_FIRST_PROBES, "{name}: {asked} questions");
        }
    }

///
///
///
///
///
    #[test]
    fn floor_runs_the_presets_staircase_with_its_own_four_numbers() {

        let (sale, pol, steps, tail, vest) = (42_500_000_000_000u64, 10_000_000_000_000u64, 897_500_000_000_000u64, 10_000_000_000_000u64, 40_000_000_000_000u64);
        assert_eq!(sale + pol + steps + tail + vest, SUPPLY);
        assert_eq!(SUPPLY / BPS * GENESIS_STAIR_STEPS_BPS as u64, steps);
        assert_eq!(SUPPLY / BPS * GENESIS_STAIR_TAIL_BPS as u64, tail);
        assert_eq!(stair_tail_split_for(steps + tail, &GENESIS_STAIR), (steps, tail), "897.5M / 10M to the unit");
        let (a, b) = stair_tail_split_for(989_999_999_999_997, &GENESIS_STAIR);
        assert_eq!(a + b, 989_999_999_999_997, "conserved at any deposit");
        assert_eq!(stair_tail_split(STAIR_SUPPLY_TOKENS.2 + STAIR_SUPPLY_TOKENS.3), (STAIR_SUPPLY_TOKENS.2, STAIR_SUPPLY_TOKENS.3), "presets unchanged");

        for n in 1..=MAX_STAIR_STEPS {
            let t = step_tokens(steps, n, GENESIS_STAIR_GROWTH_BPS).unwrap();
            assert_eq!(t.iter().sum::<u64>(), steps, "{n} steps hold the 897.5M exactly");
            for i in 1..n as usize {
                assert!(t[i] < t[i - 1], "front-loaded: step {i} lighter than {}", i - 1);
                assert!((t[i] as f64 / t[i - 1] as f64 - 0.95).abs() < 1e-9, "{n} steps, step {i}");
            }
        }

        let nine = step_tokens(steps, 9, GENESIS_STAIR_GROWTH_BPS).unwrap();
        let sum: f64 = (0..9).map(|j| 0.95f64.powi(j)).sum();
        assert!((nine[0] as f64 - steps as f64 / sum).abs() < 20.0, "{}", nine[0]);
        assert_eq!((nine[0] + 50_000_000_000) / 100_000_000_000, 1_214, "121.4M");

        assert_eq!(stair_params(STAIR_KIND_PRESET), Some(PRESET_STAIR));
        assert_eq!(stair_params(STAIR_KIND_GENESIS), Some(GENESIS_STAIR));
        assert_eq!(stair_params(2), None);
        assert_eq!((PRESET_STAIR.pol_to_mcap_bps, PRESET_STAIR.cushion_bottom_bps, PRESET_STAIR.floor_to_price_bps), (1_500, 9_000, 2_000));
        assert_eq!((GENESIS_STAIR.pol_to_mcap_bps, GENESIS_STAIR.cushion_bottom_bps, GENESIS_STAIR.floor_to_price_bps), (2_000, 7_000, 2_000));
        assert_eq!(GENESIS_CUSHION_TICKS, cushion_ticks_for(GENESIS_CUSHION_BOTTOM_BPS).unwrap(), "0.7 ×: the formula's own value");
        assert_eq!(CUSHION_TICKS, cushion_ticks_for(CUSHION_BOTTOM_BPS).unwrap());
        assert_eq!(stored_stair_params(CUSHION_BOTTOM_BPS as u16, POL_TO_MCAP_BPS as u16, CUSHION_TICKS), PRESET_STAIR);
        let g = stored_stair_params(7_000, 2_000, GENESIS_CUSHION_TICKS);
        assert_eq!((g.pol_to_mcap_bps, g.cushion_bottom_bps, g.cushion_ticks, g.floor_to_price_bps), (2_000, 7_000, 3_567, 2_000));
        assert!(cushion_ticks_for(0).is_err() && cushion_ticks_for(10_000).is_err());
    }

///
    #[test]
    fn the_cushion_depth_is_per_pool_seventy_or_ninety_percent() {
        let current = 100_000;
        for (width, share) in [(CUSHION_TICKS, 0.9), (GENESIS_CUSHION_TICKS, 0.7)] {
            let (lo, hi) = cushion_range_w(current, S, width).unwrap().unwrap();
            assert_eq!(lo, ceil_to_spacing(current + 1, S).unwrap());
            assert_eq!(hi - lo, ceil_to_spacing(width, S).unwrap());

            let r = 1.0001f64.powi(-(hi - lo));
            assert!(r <= share && r > share - 0.01, "{width}: {r}");
            assert_eq!(cushion_range_from_w(lo, S, width).unwrap(), Some((lo, hi)));
            assert_eq!(cushion_range_at_w(current, current + 600, S, width).unwrap(), cushion_range_w(current + 600, S, width).unwrap());
        }
        assert_eq!(cushion_range(current, S).unwrap(), cushion_range_w(current, S, CUSHION_TICKS).unwrap(), "the presets' default");

        let floor_tick = 100_000 + 2_400;
        let dear = crate::band::dear_edge(floor_tick, S).unwrap();
        let (_, hi) = cushion_range_above_w(current, current, S, floor_tick, GENESIS_CUSHION_TICKS).unwrap().unwrap();
        assert!(hi <= dear, "never inside the band");
    }

///
///
///
///
    #[test]
    fn the_cushion_home_is_the_cheapest_full_depth_range_above_the_band() {
        for spacing in [1, 10, S, 200] {
            for width in [CUSHION_TICKS, GENESIS_CUSHION_TICKS] {
                let w = ceil_to_spacing(width, spacing).unwrap();
                let floor_tick = ceil_to_spacing(200_000, spacing).unwrap();
                let dear = crate::band::dear_edge(floor_tick, spacing).unwrap();

                for spot in [dear - 50 * w, dear - 3 * w, dear - w - 2 * spacing] {
                    let home = cushion_home_top(spot, floor_tick, spacing, width).unwrap();
                    assert_eq!(home, dear - w - 1, "spacing {spacing} width {width} spot {spot}");
                    let range = cushion_range_above_w(spot, home, spacing, floor_tick, width).unwrap();
                    assert_eq!(range, Some((dear - w, dear)), "the full depth, ending at the dear edge");
                    assert!(range.unwrap().0 > spot, "strictly cheaper than the spot: all quote");
                }

                for spot in [dear - w + spacing, dear - 2 * spacing, dear - spacing - 1] {
                    let home = cushion_home_top(spot, floor_tick, spacing, width).unwrap();
                    assert_eq!(home, spot, "spacing {spacing} width {width} spot {spot}");
                    let (lo, hi) = cushion_range_above_w(spot, home, spacing, floor_tick, width).unwrap().unwrap();
                    assert_eq!((lo, hi), (ceil_to_spacing(spot + 1, spacing).unwrap(), dear));
                }

                for spot in [dear - spacing, dear] {
                    let home = cushion_home_top(spot, floor_tick, spacing, width).unwrap();
                    assert_eq!(home, spot);
                    assert_eq!(cushion_range_above_w(spot, home, spacing, floor_tick, width).unwrap(), None);
                }
            }
        }
    }

///
///
    #[test]
    fn the_split_reads_each_pools_own_pol_target() {
        let floor_tick = 90_000;
        let f = get_sqrt_price_at_tick(floor_tick).unwrap();
        let p = sqrt_at_price_multiple(floor_tick, 4.0);
        let mcap = {
            let p2 = U256::from(p) * U256::from(p);
            ((U256::from(SUPPLY) << 128usize) / p2).as_u128() as f64
        };
        let pol_q = (mcap * 0.10) as u64;
        let preset = stair_split_for(1_000_000, p, f, pol_q, SUPPLY, &PRESET_STAIR).unwrap();
        assert_eq!(preset, stair_split(1_000_000, p, f, pol_q, SUPPLY).unwrap());
        assert_eq!(preset.short_floor, 0);
        assert!((preset.short_pol as f64 / SPLIT_SCALE as f64 - 1.0 / 3.0).abs() < 1e-3, "a preset's is a third short at 10 % of 15 %");
        assert_eq!((preset.to_pol, preset.to_floor), (1_000_000, 0), "floor met, POL short: all to the POL");
        let met = stair_split_for(1_000_000, p, f, (mcap * 0.16) as u64, SUPPLY, &PRESET_STAIR).unwrap();
        assert_eq!((met.short_pol, met.to_pol), (0, 0), "a preset's POL is met at 16 % (> 15 %): all to the floor");
        let floor = stair_split_for(1_000_000, p, f, pol_q, SUPPLY, &GENESIS_STAIR).unwrap();
        assert_eq!(floor.short_floor, 0);
        assert!((floor.short_pol as f64 / SPLIT_SCALE as f64 - 0.5).abs() < 1e-3, "$FLOOR's is half short at 10 % of 20 %");
        assert_eq!((floor.to_pol, floor.to_floor), (1_000_000, 0), "floor met, POL short: all to the POL");

        let p10 = sqrt_at_price_multiple(floor_tick, 10.0);
        let g = stair_split_for(1_000_000, p10, f, 0, SUPPLY, &GENESIS_STAIR).unwrap();
        assert!((g.to_pol as i64 - 666_666).abs() < 1_000, "{g:?}: 1 / (0.5 + 1)");
    }

///
///
///
///
///
    #[test]
    fn custom_steps_are_derived_from_the_cap_exactly_as_the_evm() {
        for (cap, want) in [
            (25_000u64, 15u8),
            (50_000, 14),
            (100_000, 13),
            (200_000, 12),
            (250_000, 12),
            (1_000_000, 10),
            (10_000_000, 7),
            (1_000, 16),
            (600_000_000, 1),
            (800_000_000, 1),
            (690_533, 11),
            (690_534, 10),
            (1_000_000_000, 1),
            (5_000_000_000, 1),
            (1, 16),
            (0, 16),
        ] {
            assert_eq!(custom_stair_steps(cap), want, "${cap}");
        }

        let mut last = MAX_STAIR_STEPS;
        let mut cap = 1u64;
        while cap < 4_000_000_000 {
            let n = custom_stair_steps(cap);
            assert!((MIN_STAIR_STEPS..=MAX_STAIR_STEPS).contains(&n), "${cap}: {n}");
            assert!(n <= last, "${cap}: {n} after {last}");
            last = n;
            cap = cap * 11 / 10 + 1;
        }

        for cap in [21_000u64, 33_333, 77_000, 123_456, 3_000_000, 42_000_000, 300_000_000] {
            let r = (CUSTOM_STAIR_TOP_MCAP_USD as f64 / cap as f64).log2().round().clamp(1.0, 16.0) as u8;
            assert_eq!(custom_stair_steps(cap), r, "${cap}");
        }
        assert_eq!(custom_stair(200_000), (12, 10_000), "equal steps");
        assert!(stair_shape_ok(custom_stair(25_000).0, CUSTOM_STAIR_GROWTH_BPS));
    }

///
///
///
///
    #[test]
    fn the_tail_is_one_percent_of_supply_and_the_presets_are_unchanged() {
        let (_, _, steps, tail) = STAIR_SUPPLY_TOKENS;
        assert_eq!(stair_tail_split_supply(steps + tail, &PRESET_STAIR), stair_tail_split_for(steps + tail, &PRESET_STAIR));
        let g = SUPPLY / BPS * (GENESIS_STAIR_STEPS_BPS + GENESIS_STAIR_TAIL_BPS) as u64;
        assert_eq!(stair_tail_split_supply(g, &GENESIS_STAIR), stair_tail_split_for(g, &GENESIS_STAIR));

        let custom = SUPPLY / BPS * 6_000;
        assert_eq!(stair_tail_split_supply(custom, &PRESET_STAIR), (590_000_000_000_000, 10_000_000_000_000));

        assert_eq!(stair_tail_split_supply(SUPPLY / 10, &PRESET_STAIR), (90_000_000_000_000, 10_000_000_000_000));
        assert_eq!(stair_tail_split_supply(1, &PRESET_STAIR), (0, 1), "never more than the deposit");
        assert_ne!(stair_tail_split_for(custom, &PRESET_STAIR).1, 10_000_000_000_000, "the ratio would have differed");
    }
}
