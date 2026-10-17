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

use carpenter_clmm_math::{bitmap, liquidity_math, tick_math};
use floor_types::constants::{MAX_TICK, MIN_TICK, TICK_ARRAY_SIZE};
use floor_types::hook::BalanceDelta;
use floor_types::layout::Pool;
use floor_types::{I128, U128};

use crate::error::{QuoteError, QuoteResult};
use crate::liquidity::amounts_for_liquidity;
use crate::state::TickArraySet;
use crate::walk::WalkKind;

///
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ModifyOutcome {
///
    pub delta: BalanceDelta,
///
    pub amount0: u64,
    pub amount1: u64,
///
///
///
///
    pub fees_replayed: bool,
}

///
///
///
pub fn modify_position(
    pool: &mut Pool,
    set: &mut TickArraySet,
    tick_lower: i32,
    tick_upper: i32,
    liquidity_delta: i128,
    kind: WalkKind,
    arrays: &mut Vec<i32>,
) -> QuoteResult<ModifyOutcome> {
    let spacing = set.tick_spacing;
    let s = i32::from(spacing);
    if tick_lower >= tick_upper
        || tick_lower < MIN_TICK
        || tick_upper > MAX_TICK
        || tick_lower % s != 0
        || tick_upper % s != 0
    {
        return Err(QuoteError::Math(carpenter_clmm_math::MathError::TickOutOfRange));
    }
    if liquidity_delta == 0 {
        return Ok(ModifyOutcome { fees_replayed: false, ..ModifyOutcome::default() });
    }
    let sqrt = pool.sqrt_price_x64.get();
    let tick_current = pool.tick_current;
    let g0 = pool.fee_growth_global0_x64.get();
    let g1 = pool.fee_growth_global1_x64.get();



    let (amount0, amount1) = amounts_for_liquidity(sqrt, tick_current, tick_lower, tick_upper, liquidity_delta)?;
    let new_liquidity = if tick_current >= tick_lower && tick_current < tick_upper {
        liquidity_math::add_delta(pool.liquidity.get(), liquidity_delta)?
    } else {
        pool.liquidity.get()
    };




    let lower = plan_tick(set, tick_lower, liquidity_delta, false, tick_current, kind, None)?;
    let upper = plan_tick(set, tick_upper, liquidity_delta, true, tick_current, kind, Some(&lower))?;

    apply_tick(set, &lower, g0, g1, arrays);
    apply_tick(set, &upper, g0, g1, arrays);
    pool.liquidity = U128::new(new_liquidity);

    let delta = if liquidity_delta > 0 {
        BalanceDelta { amount0: -(amount0 as i128), amount1: -(amount1 as i128) }
    } else {
        BalanceDelta { amount0: amount0 as i128, amount1: amount1 as i128 }
    };

    Ok(ModifyOutcome { delta, amount0, amount1, fees_replayed: false })
}

///
///
///
#[derive(Clone, Copy, Debug)]
struct TickUpdate {
///
///
    start: i32,
    index: usize,
    word: usize,
    mask: u64,
    gross_after: u128,
    net: i128,
///
///
    count_after: u16,
    change: Change,
///
    seed_outside: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Change {
///
    Initialise,
///
    Clear,
    Adjust,
}

///
///
///
#[allow(clippy::too_many_arguments)]
fn plan_tick(
    set: &TickArraySet,
    tick: i32,
    liquidity_delta: i128,
    upper: bool,
    tick_current: i32,
    kind: WalkKind,
    prior: Option<&TickUpdate>,
) -> QuoteResult<TickUpdate> {
    let spacing = set.tick_spacing;
    let start = bitmap::array_start_index(tick, spacing)?;
    let bit = bitmap::array_bit(start, spacing)?;
    let array = set.arrays.get(&start).ok_or_else(|| kind.missing(start))?;
    let i = (tick - start) / i32::from(spacing);
    debug_assert!((0..TICK_ARRAY_SIZE).contains(&i));
    let t = &array.ticks[i as usize];

    let gross_before = t.liquidity_gross.get();
    let gross_after = liquidity_math::add_delta(gross_before, liquidity_delta)?;
    let net = if upper {
        t.liquidity_net.get().checked_sub(liquidity_delta).ok_or(QuoteError::Overflow)?
    } else {
        t.liquidity_net.get().checked_add(liquidity_delta).ok_or(QuoteError::Overflow)?
    };
    let change = if gross_before == 0 && gross_after != 0 {
        Change::Initialise
    } else if gross_before != 0 && gross_after == 0 {
        Change::Clear
    } else {
        Change::Adjust
    };


    let count_before = match prior {
        Some(p) if p.start == start => p.count_after,
        _ => array.initialized_tick_count,
    };
    let count_after = match change {
        Change::Initialise => count_before.checked_add(1).ok_or(QuoteError::Overflow)?,
        Change::Clear => count_before.checked_sub(1).ok_or(QuoteError::Overflow)?,
        Change::Adjust => count_before,
    };



    let word = (bit / 64) as usize;
    let writes_bit = change == Change::Initialise || (change == Change::Clear && count_after == 0);
    if writes_bit && word >= set.words.len() {
        return Err(QuoteError::Math(carpenter_clmm_math::MathError::InvalidTickArrayBoundary));
    }

    Ok(TickUpdate {
        start,
        index: i as usize,
        word,
        mask: 1u64 << (bit % 64),
        gross_after,
        net,
        count_after,
        change,
        seed_outside: tick <= tick_current,
    })
}

///
///
///
///
///
///
///
fn apply_tick(set: &mut TickArraySet, u: &TickUpdate, g0: u128, g1: u128, arrays: &mut Vec<i32>) {
    let Some(array) = set.arrays.get_mut(&u.start) else {
        debug_assert!(false, "plan_tick proved the array is in the set");
        return;
    };
    if !arrays.contains(&u.start) {
        arrays.push(u.start);
    }
    let t = &mut array.ticks[u.index];
    match u.change {
        Change::Initialise => {
            if u.seed_outside {
                t.fee_growth_outside0_x64 = U128::new(g0);
                t.fee_growth_outside1_x64 = U128::new(g1);
            }
            t.liquidity_gross = U128::new(u.gross_after);
            t.liquidity_net = I128::new(u.net);
        }
        Change::Clear => *t = Default::default(),
        Change::Adjust => {
            t.liquidity_gross = U128::new(u.gross_after);
            t.liquidity_net = I128::new(u.net);
        }
    }
    array.initialized_tick_count = u.count_after;

    let set_to = match u.change {
        Change::Initialise => true,
        Change::Clear if u.count_after == 0 => false,
        _ => return,
    };
    let Some(w) = set.words.get_mut(u.word) else {
        debug_assert!(false, "plan_tick proved the bitmap holds this array's bit");
        return;
    };
    if set_to {
        *w |= u.mask;
    } else {
        *w &= !u.mask;
    }
}

///
///
pub fn arrays_for_range(tick_lower: i32, tick_upper: i32, tick_spacing: u16) -> QuoteResult<Vec<i32>> {
    let a = bitmap::array_start_index(tick_lower, tick_spacing)?;
    let b = bitmap::array_start_index(tick_upper, tick_spacing)?;
    Ok(if a == b { vec![a] } else { vec![a, b] })
}

///
pub fn sqrt_at(tick: i32) -> QuoteResult<u128> {
    Ok(tick_math::get_sqrt_price_at_tick(tick)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synth::{build_pool, PoolKeys, SynthKind, SynthPosition};

    fn keys() -> PoolKeys {
        PoolKeys { pool: [7; 32], mint0: [1; 32], mint1: [2; 32], vault0: [3; 32], vault1: [4; 32], token_program0: [5; 32], token_program1: [6; 32], ..Default::default() }
    }



    #[test]
    fn remove_then_add_round_trips_the_arrays() {
        let sqrt = tick_math::get_sqrt_price_at_tick(100_020).unwrap();
        let l = 5_000_000_000_000u128;
        let sp = build_pool(&keys(), SynthKind::Floor, 60, sqrt, 9, 6, &[SynthPosition { tick_lower: 99_000, tick_upper: 100_320, liquidity: l }]).unwrap();
        let mut set = sp.tick_array_set().unwrap();
        let mut pool = sp.account.pool;
        let before_arrays: Vec<_> = set.arrays.values().map(|a| bytemuck::bytes_of(a).to_vec()).collect();
        let before_words = set.words.clone();
        assert_eq!(pool.liquidity.get(), l);

        let mut arrays = Vec::new();
        let out = modify_position(&mut pool, &mut set, 99_000, 100_320, -(l as i128), WalkKind::Action, &mut arrays).unwrap();
        assert_eq!(pool.liquidity.get(), 0);
        assert!(out.delta.amount0 > 0 && out.delta.amount1 > 0, "in range: both legs come back");
        assert_eq!((out.amount0 as i128, out.amount1 as i128), (out.delta.amount0, out.delta.amount1));
        assert_eq!(arrays, vec![97_200], "both ticks sit in one array");
        let a = set.arrays.get(&97_200).unwrap();
        assert_eq!(a.initialized_tick_count, 0);
        assert!(!set.is_marked(97_200).unwrap());
        assert_eq!(a.ticks[((99_000 - 97_200) / 60) as usize].liquidity_gross.get(), 0);

        let back = modify_position(&mut pool, &mut set, 99_000, 100_320, l as i128, WalkKind::Action, &mut arrays).unwrap();
        assert!(back.delta.amount0 < 0 && back.delta.amount1 < 0, "an add debits");
        assert!(back.amount0 >= out.amount0 && back.amount1 >= out.amount1, "adds round up, removes down");
        assert_eq!(pool.liquidity.get(), l);
        let after_arrays: Vec<_> = set.arrays.values().map(|a| bytemuck::bytes_of(a).to_vec()).collect();
        assert_eq!(after_arrays, before_arrays);
        assert_eq!(set.words, before_words);
    }



    #[test]
    fn a_band_above_the_market_is_all_quote_and_leaves_liquidity_alone() {
        let sqrt = tick_math::get_sqrt_price_at_tick(100_020).unwrap();
        let sp = build_pool(&keys(), SynthKind::Floor, 60, sqrt, 9, 6, &[]).unwrap();
        let mut set = sp.tick_array_set().unwrap();
        let mut pool = sp.account.pool;
        pool.fee_growth_global0_x64 = U128::new(777);

        let mut arrays = Vec::new();
        assert_eq!(
            modify_position(&mut pool, &mut set, 100_200, 100_320, 1_000, WalkKind::Action, &mut arrays).unwrap_err(),
            QuoteError::HookActionAccountMissing { start_index: 97_200 }
        );
        let mut a: floor_types::layout::TickArray = bytemuck::Zeroable::zeroed();
        a.pool = [7; 32];
        a.start_index = 97_200;
        set.arrays.insert(97_200, a);
        let out = modify_position(&mut pool, &mut set, 100_200, 100_320, 1_000_000_000_000, WalkKind::Action, &mut arrays).unwrap();
        assert_eq!(out.delta.amount1, 0);
        assert!(out.delta.amount0 < 0);
        assert_eq!(pool.liquidity.get(), 0, "above the market: not active");
        assert!(set.is_marked(97_200).unwrap());
        let t = &set.arrays[&97_200].ticks[((100_200 - 97_200) / 60) as usize];
        assert_eq!(t.fee_growth_outside0_x64.get(), 0, "above the current tick: no outside growth");
        assert_eq!(t.liquidity_net.get(), 1_000_000_000_000);

        assert!(modify_position(&mut pool, &mut set, 100_210, 100_320, 1, WalkKind::Action, &mut arrays).is_err());
        assert!(modify_position(&mut pool, &mut set, 100_320, 100_200, 1, WalkKind::Action, &mut arrays).is_err());
        assert_eq!(arrays_for_range(100_200, 100_320, 60).unwrap(), vec![97_200]);
        assert_eq!(arrays_for_range(97_100, 100_320, 60).unwrap(), vec![93_600, 97_200]);
    }






    #[test]
    fn an_err_on_the_upper_tick_leaves_the_lower_tick_untouched() {
        let sqrt = tick_math::get_sqrt_price_at_tick(100_020).unwrap();
        let blank = |start: i32| {
            let mut a: floor_types::layout::TickArray = bytemuck::Zeroable::zeroed();
            a.pool = [7; 32];
            a.start_index = start;
            a
        };

        assert_eq!(arrays_for_range(97_140, 97_260, 60).unwrap(), vec![93_600, 97_200]);

        let snap = |set: &crate::state::TickArraySet, pool: &floor_types::layout::Pool| {
            let mut v: Vec<u8> = Vec::new();
            for (start, a) in &set.arrays {
                v.extend_from_slice(&start.to_le_bytes());
                v.extend_from_slice(bytemuck::bytes_of(a));
            }
            for w in &set.words {
                v.extend_from_slice(&w.to_le_bytes());
            }
            v.extend_from_slice(bytemuck::bytes_of(pool));
            v
        };

        let unchanged = |now: &[u8], before: &[u8]| match now.iter().zip(before).position(|(a, b)| a != b) {
            None => assert_eq!(now.len(), before.len(), "length moved"),
            Some(i) => panic!("byte {i} was written: {} → {}", before[i], now[i]),
        };


        let sp = build_pool(&keys(), SynthKind::Floor, 60, sqrt, 9, 6, &[]).unwrap();
        let mut set = sp.tick_array_set().unwrap();
        set.arrays.insert(93_600, blank(93_600));
        let mut pool = sp.account.pool;
        let before = snap(&set, &pool);
        let mut arrays = Vec::new();
        assert_eq!(
            modify_position(&mut pool, &mut set, 97_140, 97_260, 1_000_000_000_000, WalkKind::Action, &mut arrays).unwrap_err(),
            QuoteError::HookActionAccountMissing { start_index: 97_200 }
        );
        unchanged(&snap(&set, &pool), &before);
        assert!(arrays.is_empty(), "a refused modify names no array it did not touch");


        let mut set = sp.tick_array_set().unwrap();
        set.arrays.insert(93_600, blank(93_600));
        let mut hostile = blank(97_200);
        let t = &mut hostile.ticks[((97_260 - 97_200) / 60) as usize];
        t.liquidity_gross = U128::new(1);
        t.liquidity_net = I128::new(i128::MIN);
        hostile.initialized_tick_count = 1;
        set.arrays.insert(97_200, hostile);
        let mut pool = sp.account.pool;
        let before = snap(&set, &pool);
        let mut arrays = Vec::new();
        assert_eq!(
            modify_position(&mut pool, &mut set, 97_140, 97_260, 1_000_000_000_000, WalkKind::Action, &mut arrays).unwrap_err(),
            QuoteError::Overflow
        );
        unchanged(&snap(&set, &pool), &before);
        assert!(arrays.is_empty());
    }




    #[test]
    fn two_ticks_in_one_array_count_in_sequence() {
        let sqrt = tick_math::get_sqrt_price_at_tick(100_020).unwrap();
        let l = 5_000_000_000_000u128;
        let sp = build_pool(&keys(), SynthKind::Floor, 60, sqrt, 9, 6, &[SynthPosition { tick_lower: 99_000, tick_upper: 100_320, liquidity: l }]).unwrap();
        let mut set = sp.tick_array_set().unwrap();
        let mut pool = sp.account.pool;
        assert_eq!(set.arrays[&97_200].initialized_tick_count, 2, "both ticks are in array 97_200");
        assert!(set.is_marked(97_200).unwrap());
        let mut arrays = Vec::new();

        modify_position(&mut pool, &mut set, 99_000, 100_320, -((l / 2) as i128), WalkKind::Action, &mut arrays).unwrap();
        assert_eq!(set.arrays[&97_200].initialized_tick_count, 2);
        assert!(set.is_marked(97_200).unwrap());

        modify_position(&mut pool, &mut set, 99_000, 100_320, -((l - l / 2) as i128), WalkKind::Action, &mut arrays).unwrap();
        assert_eq!(set.arrays[&97_200].initialized_tick_count, 0);
        assert!(!set.is_marked(97_200).unwrap());
        assert_eq!(arrays, vec![97_200]);
    }
}
