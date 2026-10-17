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

use carpenter_clmm_math::{bitmap, liquidity_math, swap_math, tick_math};
use floor_types::constants::{MAX_SQRT_PRICE_X64, MAX_TICK, MIN_SQRT_PRICE_X64, MIN_TICK, TICK_ARRAY_SIZE};
use floor_types::hook::BalanceDelta;
use floor_types::layout::{Pool, TickArray};
use floor_types::protocol_fee::{self, WalkFee};
use floor_types::U128;

use crate::error::{QuoteError, QuoteResult};
use crate::state::TickArraySet;

///
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WalkKind {
    Trader,
    Action,
}

impl WalkKind {
    pub(crate) fn missing(self, start_index: i32) -> QuoteError {
        match self {
            WalkKind::Trader => QuoteError::TickArrayMissing { start_index },
            WalkKind::Action => QuoteError::HookActionAccountMissing { start_index },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WalkParams {
    pub zero_for_one: bool,
///
    pub amount_specified: i128,
    pub sqrt_price_limit_x64: u128,
///
///
///
    pub fee_pips: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WalkOutcome {
///
    pub swap_delta: BalanceDelta,
///
    pub fee_total: u64,
///
///
    pub protocol_fee: u64,
///
    pub protocol_currency: u8,
///
    pub arrays_read: Vec<i32>,
    pub steps: u32,
}

///
///
const MAX_STEPS: u32 = 1_000_000;

///
pub fn check_price_limit(pool: &Pool, zero_for_one: bool, limit: u128) -> QuoteResult<()> {
    let sqrt = pool.sqrt_price_x64.get();
    let ok = if zero_for_one {
        limit < sqrt && limit > MIN_SQRT_PRICE_X64
    } else {
        limit > sqrt && limit < MAX_SQRT_PRICE_X64
    };
    if ok {
        Ok(())
    } else {
        Err(QuoteError::PriceLimitInvalid)
    }
}

///
///
///
pub fn fee_for(pool: &Pool, zero_for_one: bool, lp_fee_pips: u32) -> WalkFee {
    protocol_fee::walk_fee(pool.protocol_fee, pool.protocol_fee_currency, zero_for_one, lp_fee_pips)
}

///
///
pub fn walk(pool: &mut Pool, set: &mut TickArraySet, p: &WalkParams, kind: WalkKind) -> QuoteResult<WalkOutcome> {
    if p.amount_specified == 0 {
        return Err(QuoteError::AmountZero);
    }
    let zfo = p.zero_for_one;
    let exact_in = p.amount_specified < 0;
    let limit = p.sqrt_price_limit_x64;
    let wf = fee_for(pool, zfo, p.fee_pips);



    let target: i128 = if !exact_in && wf.output_pips > 0 {
        let named = u64::try_from(p.amount_specified).map_err(|_| QuoteError::AmountTooLarge)?;
        i128::from(protocol_fee::gross_for_net(named, wf.output_pips).ok_or(QuoteError::Overflow)?)
    } else {
        p.amount_specified
    };
    let mut remaining: i128 = target;
    let mut calculated: i128 = 0;
    let mut sqrt = pool.sqrt_price_x64.get();
    let mut tick = pool.tick_current;
    let mut liquidity = pool.liquidity.get();
    let mut fg_in = if zfo { pool.fee_growth_global0_x64.get() } else { pool.fee_growth_global1_x64.get() };
    let fg_other = if zfo { pool.fee_growth_global1_x64.get() } else { pool.fee_growth_global0_x64.get() };

    let mut out = WalkOutcome::default();

    while !(remaining == 0 || sqrt == limit) {
        out.steps += 1;
        if out.steps > MAX_STEPS {
            return Err(QuoteError::Overflow);
        }
        let sqrt_start = sqrt;
        let (mut tick_next, initialized) = next_initialized_tick(set, tick, zfo, kind, &mut out.arrays_read)?;
        tick_next = tick_next.clamp(MIN_TICK, MAX_TICK);
        let sqrt_next = tick_math::get_sqrt_price_at_tick(tick_next)?;
        let target = if (zfo && sqrt_next < limit) || (!zfo && sqrt_next > limit) { limit } else { sqrt_next };

        let amount_remaining = u64::try_from(remaining.unsigned_abs()).map_err(|_| QuoteError::AmountTooLarge)?;
        let step = swap_math::compute_swap(sqrt, target, liquidity, amount_remaining, wf.swap_fee, exact_in, zfo, true)?;
        sqrt = step.sqrt_price_next_x64;

        let in_plus_fee = (step.amount_in as i128) + (step.fee_amount as i128);
        if exact_in {
            remaining += in_plus_fee;
            calculated += step.amount_out as i128;
        } else {
            remaining -= step.amount_out as i128;
            calculated -= in_plus_fee;
        }


        let gross_in = step.amount_in.checked_add(step.fee_amount).ok_or(QuoteError::Overflow)?;
        let share = protocol_fee::input_share(step.fee_amount, gross_in, wf.swap_fee, wf.input_pips).ok_or(QuoteError::Overflow)?;
        let lp_fee = step.fee_amount - share;
        out.protocol_fee = out.protocol_fee.checked_add(share).ok_or(QuoteError::Overflow)?;
        out.fee_total = out.fee_total.checked_add(lp_fee).ok_or(QuoteError::Overflow)?;


        if liquidity > 0 {
            fg_in = fg_in.wrapping_add(((lp_fee as u128) << 64) / liquidity);
        }

        if sqrt == sqrt_next {
            if initialized {
                let (g0, g1) = if zfo { (fg_in, fg_other) } else { (fg_other, fg_in) };
                let mut net = cross_tick(set, tick_next, g0, g1, kind)?;
                if zfo {
                    net = net.checked_neg().ok_or(QuoteError::Overflow)?;
                }
                liquidity = liquidity_math::add_delta(liquidity, net)?;
            }
            tick = if zfo { tick_next - 1 } else { tick_next };
        } else if sqrt != sqrt_start {
            tick = tick_math::get_tick_at_sqrt_price(sqrt)?;
        }
    }

    pool.sqrt_price_x64 = U128::new(sqrt);
    pool.tick_current = tick;
    pool.liquidity = U128::new(liquidity);
    if zfo {
        pool.fee_growth_global0_x64 = U128::new(fg_in);
    } else {
        pool.fee_growth_global1_x64 = U128::new(fg_in);
    }


    let specified_used = target - remaining;
    out.swap_delta = if zfo != exact_in {
        BalanceDelta { amount0: calculated, amount1: specified_used }
    } else {
        BalanceDelta { amount0: specified_used, amount1: calculated }
    };



    if wf.output_pips > 0 {
        let leg = if zfo { &mut out.swap_delta.amount1 } else { &mut out.swap_delta.amount0 };
        let gross_out = u64::try_from(*leg).map_err(|_| QuoteError::Overflow)?;
        let share = protocol_fee::output_share(gross_out, wf.output_pips);
        *leg -= i128::from(share);
        out.protocol_fee = out.protocol_fee.checked_add(share).ok_or(QuoteError::Overflow)?;
    }
    out.protocol_currency = wf.currency;
    if wf.currency == 0 {
        pool.protocol_fees_owed0 = pool.protocol_fees_owed0.checked_add(out.protocol_fee).ok_or(QuoteError::Overflow)?;
    } else {
        pool.protocol_fees_owed1 = pool.protocol_fees_owed1.checked_add(out.protocol_fee).ok_or(QuoteError::Overflow)?;
    }
    Ok(out)
}

///
fn next_initialized_tick(
    set: &TickArraySet,
    tick: i32,
    zfo: bool,
    kind: WalkKind,
    read: &mut Vec<i32>,
) -> QuoteResult<(i32, bool)> {
    let bound = if zfo { MIN_TICK } else { MAX_TICK };
    if !(MIN_TICK..=MAX_TICK).contains(&tick) {


        return Ok((bound, false));
    }
    let spacing = set.tick_spacing;
    let s = i32::from(spacing);
    let start = bitmap::array_start_index(tick, spacing)?;

    if set.is_marked(start)? {
        let array = load(set, start, kind, read)?;
        let offset = (tick - start).div_euclid(s);
        let found = if zfo {
            let top = offset.min(TICK_ARRAY_SIZE - 1);
            (0..=top).rev().find(|&i| is_init(array, i))
        } else {
            ((offset + 1)..TICK_ARRAY_SIZE).find(|&i| is_init(array, i))
        };
        if let Some(i) = found {
            return Ok((start + i * s, true));
        }
    }

    let mut last = start;
    loop {
        match bitmap::next_initialized_array_start_index(&set.words, spacing, last, zfo)? {
            None => return Ok((bound, false)),
            Some(next) => {
                let array = load(set, next, kind, read)?;
                let found = if zfo {
                    (0..TICK_ARRAY_SIZE).rev().find(|&i| is_init(array, i))
                } else {
                    (0..TICK_ARRAY_SIZE).find(|&i| is_init(array, i))
                };
                if let Some(i) = found {
                    return Ok((next + i * s, true));
                }

                last = next;
            }
        }
    }
}

fn is_init(array: &TickArray, i: i32) -> bool {
    array.ticks[i as usize].liquidity_gross.get() != 0
}

fn load<'a>(set: &'a TickArraySet, start: i32, kind: WalkKind, read: &mut Vec<i32>) -> QuoteResult<&'a TickArray> {
    let array = set.arrays.get(&start).ok_or_else(|| kind.missing(start))?;
    if !read.contains(&start) {
        read.push(start);
    }
    Ok(array)
}

///
///
fn cross_tick(set: &mut TickArraySet, tick: i32, g0: u128, g1: u128, kind: WalkKind) -> QuoteResult<i128> {
    let spacing = set.tick_spacing;
    let start = bitmap::array_start_index(tick, spacing)?;
    let i = ((tick - start) / i32::from(spacing)) as usize;
    let array = set.arrays.get_mut(&start).ok_or_else(|| kind.missing(start))?;
    let t = &mut array.ticks[i];
    t.fee_growth_outside0_x64 = U128::new(g0.wrapping_sub(t.fee_growth_outside0_x64.get()));
    t.fee_growth_outside1_x64 = U128::new(g1.wrapping_sub(t.fee_growth_outside1_x64.get()));
    Ok(t.liquidity_net.get())
}
