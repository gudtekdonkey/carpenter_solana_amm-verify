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

use floor_types::constants::{
    AFTER_SWAP_FLAG, BEFORE_SWAP_FLAG, DYNAMIC_FEE_FLAG, MAX_SWAP_FEE,
};
use floor_types::hook::{
    self as hk, ActionResult, BalanceDelta, FeeSplit, HookAction, HookDeltaAssembly, HookRecordV1, RecordExpect,
    HOOK_INTERFACE_VERSION, PHASE_AFTER_ACTIONS, PHASE_AFTER_SWAP, PHASE_BEFORE_SWAP,
};
use floor_types::ids::FLOOR_HOOK_ID;
use floor_types::layout::{FloorPool, Pool, TickArray};
use floor_types::U128;

use crate::error::{QuoteError, QuoteResult};
use crate::floor::{self, AfterSwapIn, FeesTaken, ReplaceBooking, SwapCtx};
use crate::modify;
use crate::state::TickArraySet;
use crate::walk::{self, WalkKind, WalkParams};

///
///
#[derive(Clone, Copy, Debug)]
pub struct PoolAccount {
    pub key: [u8; 32],
    pub pool: Pool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SwapQuoteParams {
    pub zero_for_one: bool,
///
    pub amount_specified: i128,
    pub sqrt_price_limit_x64: u128,
///
    pub supply0: u64,
///
///
    pub supply1: u64,
///
///
///
///
    pub cushion_top: Option<i32>,
}

///
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct QuoteFees {
///
///
    pub lp: u64,
///
    pub skim: u64,
///
    pub creator: u64,
///
///
///
///
///
///
///
    pub holder: u64,
///
    pub platform_cut: u64,
///
///
    pub team: u64,
///
///
///
///
///
///
    pub protocol: u64,
///
    pub protocol_currency: u8,
}

impl QuoteFees {
///
///
///
    pub fn hook_total(&self) -> u64 {
        self.skim + self.creator + self.holder + self.team
    }
}

///
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplaceQuote {
    pub booking: ReplaceBooking,
///
    pub from_tick: i32,
///
    pub to_tick: i32,
}

#[derive(Clone, Debug)]
pub struct SwapQuote {
///
    pub amount_in: u64,
///
    pub amount_out: u64,
    pub fees: QuoteFees,
///
    pub sqrt_after: u128,
    pub tick_after: i32,
///
    pub sqrt_after_trade: u128,
    pub tick_after_trade: i32,
///
    pub arrays_touched: Vec<i32>,
///
    pub replace_actions: Vec<HookAction>,
///
    pub action_arrays: Vec<i32>,
    pub replace: Option<ReplaceQuote>,
    pub signals: u8,
///
    pub swap_delta: BalanceDelta,
///
    pub hook_delta: BalanceDelta,
///
///
///
///
///
///
///
    pub fee_split0: FeeSplit,
    pub fee_split1: FeeSplit,
///
    pub trader_delta: BalanceDelta,
    pub pool_after: Pool,
///
    pub arrays_after: Vec<TickArray>,
    pub floor_after: Option<FloorPool>,
}

impl SwapQuote {
///
///
    pub fn all_arrays(&self) -> Vec<i32> {
        let mut v = self.arrays_touched.clone();
        for s in &self.action_arrays {
            if !v.contains(s) {
                v.push(*s);
            }
        }
        v
    }
}

fn snapshot_tick_sqrt(pool: &Pool) -> (u128, i32) {
    (pool.sqrt_price_x64.get(), pool.tick_current)
}

fn expect(pool: &Pool, phase: u8, round: u8) -> RecordExpect {
    RecordExpect { version: HOOK_INTERFACE_VERSION, phase, round, flags: pool.hook_flags, max_rounds: pool.hook_max_rounds }
}

fn credit_claims(pool: &mut Pool, delta: BalanceDelta) -> QuoteResult<()> {
    if delta.amount0 < 0 || delta.amount1 < 0 {
        return Err(QuoteError::HookDeltaNegative);
    }
    let d0 = u64::try_from(delta.amount0).map_err(|_| QuoteError::Overflow)?;
    let d1 = u64::try_from(delta.amount1).map_err(|_| QuoteError::Overflow)?;
    pool.hook_claims0 = pool.hook_claims0.checked_add(d0).ok_or(QuoteError::Overflow)?;
    pool.hook_claims1 = pool.hook_claims1.checked_add(d1).ok_or(QuoteError::Overflow)?;
    pool.hook_fees0 = U128::new(pool.hook_fees0.get().checked_add(d0 as u128).ok_or(QuoteError::Overflow)?);
    pool.hook_fees1 = U128::new(pool.hook_fees1.get().checked_add(d1 as u128).ok_or(QuoteError::Overflow)?);
    Ok(())
}

///
///
///
struct Executed {
    results: Vec<ActionResult>,
    results_carry_fees: bool,
}

///
///
///
fn execute_actions(
    pool: &mut Pool,
    set: &mut TickArraySet,
    actions: &[HookAction],
    action_arrays: &mut Vec<i32>,
) -> QuoteResult<Executed> {
    let mut results = Vec::with_capacity(actions.len());
    let mut results_carry_fees = true;
    for (index, action) in actions.iter().enumerate() {
        match *action {
            HookAction::ModifyPosition { salt: _, tick_lower, tick_upper, liquidity_delta } => {
                let mut result = ActionResult { index: index as u8, kind: action.kind(), ..Default::default() };
                let m = modify::modify_position(pool, set, tick_lower, tick_upper, liquidity_delta, WalkKind::Action, action_arrays)?;
                results_carry_fees &= m.fees_replayed;
                let (paid0, paid1) = ((-m.delta.amount0).max(0), (-m.delta.amount1).max(0));
                let (got0, got1) = (m.delta.amount0.max(0), m.delta.amount1.max(0));
                let c0 = (pool.hook_claims0 as i128).checked_sub(paid0).ok_or(QuoteError::Overflow)?;
                let c1 = (pool.hook_claims1 as i128).checked_sub(paid1).ok_or(QuoteError::Overflow)?;
                if c0 < 0 || c1 < 0 {
                    return Err(QuoteError::HookClaimsInsufficient);
                }
                pool.hook_claims0 = u64::try_from(c0 + got0).map_err(|_| QuoteError::Overflow)?;
                pool.hook_claims1 = u64::try_from(c1 + got1).map_err(|_| QuoteError::Overflow)?;
                result.delta = m.delta;
                result.post_sqrt_price_x64 = pool.sqrt_price_x64.get();
                result.post_tick = pool.tick_current;
                results.push(result);
            }
            HookAction::Swap { zero_for_one, amount_specified, sqrt_price_limit_x64 } => {
                let mut result = ActionResult { index: index as u8, kind: action.kind(), ..Default::default() };
                if amount_specified != 0 {
                    if amount_specified.unsigned_abs() > u64::MAX as u128 {
                        return Err(QuoteError::AmountTooLarge);
                    }
                    walk::check_price_limit(pool, zero_for_one, sqrt_price_limit_x64)?;
                    let p = WalkParams { zero_for_one, amount_specified, sqrt_price_limit_x64, fee_pips: pool.lp_fee };
                    let out = walk::walk(pool, set, &p, WalkKind::Action)?;
                    for s in out.arrays_read {
                        if !action_arrays.contains(&s) {
                            action_arrays.push(s);
                        }
                    }

                    let d = out.swap_delta;
                    let (paid0, paid1) = ((-d.amount0).max(0), (-d.amount1).max(0));
                    let (got0, got1) = (d.amount0.max(0), d.amount1.max(0));
                    let c0 = (pool.hook_claims0 as i128).checked_sub(paid0).ok_or(QuoteError::Overflow)?;
                    let c1 = (pool.hook_claims1 as i128).checked_sub(paid1).ok_or(QuoteError::Overflow)?;
                    if c0 < 0 || c1 < 0 {
                        return Err(QuoteError::HookClaimsInsufficient);
                    }
                    pool.hook_claims0 = u64::try_from(c0 + got0).map_err(|_| QuoteError::Overflow)?;
                    pool.hook_claims1 = u64::try_from(c1 + got1).map_err(|_| QuoteError::Overflow)?;
                    result.delta = d;
                    if zero_for_one {
                        result.fees0 = out.fee_total;
                    } else {
                        result.fees1 = out.fee_total;
                    }
                }
                result.post_sqrt_price_x64 = pool.sqrt_price_x64.get();
                result.post_tick = pool.tick_current;
                results.push(result);
            }
        }
    }
    Ok(Executed { results, results_carry_fees })
}

///
///
///
///
///
pub fn quote_swap(
    pool: &PoolAccount,
    arrays: &TickArraySet,
    hook: Option<&FloorPool>,
    params: &SwapQuoteParams,
) -> QuoteResult<SwapQuote> {
    let mut p = pool.pool;
    let mut set = arrays.clone();
    if set.tick_spacing != p.tick_spacing {
        return Err(QuoteError::QuoteBadTickArray);
    }
    let flags = p.hook_flags;
    let hooked = p.hook != [0u8; 32] && flags & (BEFORE_SWAP_FLAG | AFTER_SWAP_FLAG) != 0;
    let mut fp: Option<FloorPool> = if hooked {
        if p.hook != FLOOR_HOOK_ID {
            return Err(QuoteError::QuoteHookNotQuotable);
        }
        Some(*hook.ok_or(QuoteError::QuoteHookStateMissing)?)
    } else {
        None
    };


    let zfo = params.zero_for_one;
    let amount_specified = params.amount_specified;
    if amount_specified == 0 {
        return Err(QuoteError::AmountZero);
    }
    if amount_specified.unsigned_abs() > u64::MAX as u128 {
        return Err(QuoteError::AmountTooLarge);
    }
    let (pre_sqrt, pre_tick) = snapshot_tick_sqrt(&p);
    let _ = pre_sqrt;
    let ctx = SwapCtx {
        pool_key: pool.key,
        zero_for_one: zfo,
        amount_specified,
        pre_tick,
        supply1: params.supply1,
        claims0: p.hook_claims0,
        claims1: p.hook_claims1,
        mint0: p.mint0,
        mint1: p.mint1,
        tick_spacing: p.tick_spacing,
        fee: p.fee,
        cushion_top: params.cushion_top,
    };
    let mut taken = FeesTaken::default();


    let mut spec: i128 = 0;
    let mut unspec: i128 = 0;
    let mut fee_override: Option<u32> = None;
    let mut fees_before: FeeSplit = [0; 5];
    if let Some(state) = fp.as_mut() {
        if flags & BEFORE_SWAP_FLAG != 0 {
            let rec0 = floor::before_swap(state, &ctx, &mut taken)?;
            hk::validate_record(&rec0, &expect(&p, PHASE_BEFORE_SWAP, 0))?;
            spec = rec0.delta_specified;
            unspec = rec0.delta_unspecified;
            fee_override = hk::effective_fee_override(&rec0, flags, p.fee == DYNAMIC_FEE_FLAG);
            fees_before = rec0.fees();
        }
    }

    let amount_to_swap = hk::apply_specified_delta(amount_specified, spec)?;
    let fee = fee_override.unwrap_or(p.lp_fee);


    let walk_fee = walk::fee_for(&p, zfo, fee);
    if walk_fee.swap_fee >= MAX_SWAP_FEE && amount_to_swap > 0 {
        return Err(QuoteError::InvalidFeeForExactOut);
    }


    let (swap_delta, lp_fee_total, protocol_fee_total, arrays_touched) = if amount_to_swap == 0 {
        (BalanceDelta::default(), 0u64, 0u64, Vec::new())
    } else {
        walk::check_price_limit(&p, zfo, params.sqrt_price_limit_x64)?;
        let wp = WalkParams { zero_for_one: zfo, amount_specified: amount_to_swap, sqrt_price_limit_x64: params.sqrt_price_limit_x64, fee_pips: fee };
        let out = walk::walk(&mut p, &mut set, &wp, WalkKind::Trader)?;
        (out.swap_delta, out.fee_total, out.protocol_fee, out.arrays_read)
    };
    let (sqrt_after_trade, tick_after_trade) = snapshot_tick_sqrt(&p);


    let mut rec1: Option<HookRecordV1> = None;
    if let Some(state) = fp.as_mut() {
        if flags & AFTER_SWAP_FLAG != 0 {
            let a = AfterSwapIn {
                post_sqrt_price_x64: sqrt_after_trade,
                post_tick: tick_after_trade,
                swap_delta,
                before_delta_specified: spec,
                before_delta_unspecified: unspec,
            };
            let r = floor::after_swap(state, &ctx, &a, &mut taken)?;
            hk::validate_record(&r, &expect(&p, PHASE_AFTER_SWAP, 1))?;
            unspec = unspec.checked_add(r.delta_unspecified).ok_or(QuoteError::Overflow)?;
            rec1 = Some(r);
        }
    }


    let hook_delta = HookDeltaAssembly::assemble(amount_specified, zfo, spec, unspec);


    let fees_after = rec1.as_ref().map(HookRecordV1::fees).unwrap_or_default();
    let (fee_split0, fee_split1) = HookDeltaAssembly::assemble_fees(amount_specified, zfo, fees_before, fees_after);
    let trader_delta = BalanceDelta {
        amount0: swap_delta.amount0.checked_sub(hook_delta.amount0).ok_or(QuoteError::Overflow)?,
        amount1: swap_delta.amount1.checked_sub(hook_delta.amount1).ok_or(QuoteError::Overflow)?,
    };
    credit_claims(&mut p, hook_delta)?;


    let mut replace_actions = Vec::new();
    let mut action_arrays = Vec::new();
    let mut replace: Option<ReplaceQuote> = None;
    let mut signals = 0u8;
    if let Some(r1) = rec1 {
        signals = r1.signals;
        let from_tick = tick_after_trade;
        replace_actions.extend_from_slice(&r1.actions);
        let mut done = execute_actions(&mut p, &mut set, &r1.actions, &mut action_arrays)?;
        let mut rec = r1;
        let mut round: u8 = 1;
        while rec.want_callback {
            if u16::from(round) + 1 >= u16::from(p.hook_max_rounds) {
                return Err(QuoteError::HookRoundsExceeded);
            }
            round += 1;
            let state = fp.as_mut().expect("rounds only run on a hooked pool");
            let (next, booking) =
                floor::after_actions(state, &ctx, round, &done.results, done.results_carry_fees, p.hook_claims0, p.hook_claims1)?;
            hk::validate_record(&next, &expect(&p, PHASE_AFTER_ACTIONS, round))?;
            if round == 2 && !done.results.is_empty() {
                replace = Some(ReplaceQuote { booking, from_tick, to_tick: p.tick_current });
            }
            replace_actions.extend_from_slice(&next.actions);
            done = execute_actions(&mut p, &mut set, &next.actions, &mut action_arrays)?;
            rec = next;
        }
    }


    let (inp, outp) = if zfo { (trader_delta.amount0, trader_delta.amount1) } else { (trader_delta.amount1, trader_delta.amount0) };
    if !(inp <= 0 && outp >= 0) {
        return Err(QuoteError::HookDeltaExceedsSwapOutput);
    }
    let amount_in = u64::try_from(-inp).map_err(|_| QuoteError::AmountOverflow)?;
    let amount_out = u64::try_from(outp).map_err(|_| QuoteError::AmountOverflow)?;

    if zfo {
        p.volume0 = U128::new(p.volume0.get().checked_add(amount_in as u128).ok_or(QuoteError::Overflow)?);
        p.lp_fees0 = U128::new(p.lp_fees0.get().checked_add(lp_fee_total as u128).ok_or(QuoteError::Overflow)?);
    } else {
        p.volume1 = U128::new(p.volume1.get().checked_add(amount_in as u128).ok_or(QuoteError::Overflow)?);
        p.lp_fees1 = U128::new(p.lp_fees1.get().checked_add(lp_fee_total as u128).ok_or(QuoteError::Overflow)?);
    }
    p.swap_count = p.swap_count.checked_add(1).ok_or(QuoteError::Overflow)?;

    let mut read: Vec<i32> = arrays_touched.clone();
    for s in &action_arrays {
        if !read.contains(s) {
            read.push(*s);
        }
    }
    let arrays_after = read.iter().filter_map(|s| set.arrays.get(s).copied()).collect();

    Ok(SwapQuote {
        amount_in,
        amount_out,
        fees: QuoteFees {
            lp: lp_fee_total,
            skim: taken.skim,
            creator: taken.creator,
            holder: taken.holder,
            platform_cut: taken.platform_cut,
            team: taken.team,
            protocol: protocol_fee_total,
            protocol_currency: walk_fee.currency,
        },
        sqrt_after: p.sqrt_price_x64.get(),
        tick_after: p.tick_current,
        sqrt_after_trade,
        tick_after_trade,
        arrays_touched,
        replace_actions,
        action_arrays,
        replace,
        signals,
        swap_delta,
        hook_delta,
        fee_split0,
        fee_split1,
        trader_delta,
        pool_after: p,
        arrays_after,
        floor_after: fp,
    })
}
