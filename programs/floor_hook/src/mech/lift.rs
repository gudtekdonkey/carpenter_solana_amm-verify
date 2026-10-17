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

use anchor_lang::prelude::*;
use floor_types::constants::{MAX_TICK, MIN_TICK, SALT_BID};
use floor_types::hook::HookAction as Kinds;

use crate::errors::HookError;
use crate::events::{BidReplaced, FloorUnbacked, PlatformCut};
use crate::instructions::CallbackEvent;
use crate::mech::positions::math;
use crate::mech::signals::sqrt_at_tick;
use crate::state::{ActionResult, FloorPool, HookAction, PoolSnapshot};

///
///
const _: () = assert!(core::mem::size_of::<floor_math::FloorState>() == core::mem::size_of::<FloorPool>());
const _: () = assert!(core::mem::align_of::<floor_math::FloorState>() <= core::mem::align_of::<FloorPool>());

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
pub fn books_open_from_round_3(round: u8, max_rounds: Option<u8>) -> bool {
    match max_rounds {
        Some(m) => (round as u16) + 1 < m as u16,
        None => false,
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
pub fn amm_max_rounds(pool_signer: &AccountInfo) -> Option<u8> {
    const OFF: usize = 8 + core::mem::offset_of!(floor_types::layout::Pool, hook_max_rounds);
    let data = pool_signer.try_borrow_data().ok()?;
    if data.len() < OFF + 1 || data[..8] != floor_types::disc::account::POOL {
        return None;
    }
    Some(data[OFF])
}

fn math_err<T>(r: floor_math::FloorMathResult<T>) -> Result<T> {
    r.map_err(|e| {
        msg!("floor-math: {}", e);
        error!(HookError::MathOverflow)
    })
}







///
pub fn dear_edge(fp: &FloorPool) -> Result<i32> {
    math::dear_edge(fp)
}

///
pub fn bid_width(fp: &FloorPool) -> i32 {
    math::bid_width(fp)
}

///
///
pub fn bid_live(fp: &FloorPool, sqrt_price_x64: u128) -> Result<(u64, u64)> {
    math_err(floor_math::bid_amounts(
        sqrt_price_x64,
        fp.bid_lower,
        bid_width(fp),
        fp.bid_liquidity.get(),
        fp.bid_eth_deployed,
    ))
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
fn assert_d05(fp: &FloorPool) -> Result<()> {
    require!(fp.quote_is_token1 == 0, HookError::InvariantViolated);
    Ok(())
}

///
fn split(delta: &crate::state::BalanceDelta) -> (i128, i128) {
    (delta.amount0, delta.amount1)
}

fn pos_u64(v: i128) -> Result<u64> {
    require!(v >= 0, HookError::InvariantViolated);
    u64::try_from(v).map_err(|_| error!(HookError::MathOverflow))
}

fn add(a: u64, b: u64) -> Result<u64> {
    a.checked_add(b).ok_or_else(|| error!(HookError::MathOverflow))
}



///
///
pub fn bid_holds_tokens(fp: &FloorPool, sqrt_price_x64: u128) -> Result<bool> {
    if fp.bid_liquidity.get() == 0 {
        return Ok(false);
    }
    Ok(bid_live(fp, sqrt_price_x64)?.1 > 0)
}

///
///
///
///
///
pub fn plan_replace(fp: &FloorPool, post: &PoolSnapshot) -> Result<Option<Vec<HookAction>>> {
    assert_d05(fp)?;
    if !bid_holds_tokens(fp, post.sqrt_price_x64)? {
        return Ok(None);
    }
    let edge = dear_edge(fp)?;

    if edge <= MIN_TICK {
        return Ok(None);
    }
    let width = bid_width(fp);
    let liquidity = fp.bid_liquidity.get();
    let (bid_quote, _) = bid_live(fp, post.sqrt_price_x64)?;




    let budget = add(fp.vault_eth, bid_quote)?.min(fp.min_bid);
    let mut actions = vec![HookAction::ModifyPosition {
        salt: SALT_BID,
        tick_lower: fp.bid_lower,
        tick_upper: fp.bid_lower.checked_add(width).ok_or(HookError::MathOverflow)?,
        liquidity_delta: -(liquidity as i128),
    }];
    let limit = sqrt_at_tick(edge)?;



    let at_or_past = post.sqrt_price_x64 <= limit;
    if budget > 0 && !at_or_past {
        actions.push(HookAction::Swap {
            zero_for_one: true,
            amount_specified: -(budget as i128),
            sqrt_price_limit_x64: limit,
        });
    }
    Ok(Some(actions))
}



///
///
///
///
pub fn book_teardown(fp: &mut FloorPool, r: &ActionResult, events: &mut Vec<CallbackEvent>) -> Result<(u64, u64)> {
    assert_d05(fp)?;
    require!(r.index == 0 && r.kind == Kinds::KIND_MODIFY_POSITION, HookError::InvariantViolated);
    let (quote, base) = split(&r.delta);
    let quote_back = pos_u64(quote)?;
    let tok_back = pos_u64(base)?;

    let fees0 = r.fees0.min(quote_back);
    let principal0 = quote_back - fees0;





    let fee_kept = math::intake(fp, fees0)?;
    let cut = crate::mech::positions::sub(fees0, fee_kept)?;
    if cut > 0 {
        events.push(CallbackEvent::PlatformCut(PlatformCut { pool: fp.amm_pool, amount: cut }));
    }
    let eth_back = add(principal0, fee_kept)?;
    fp.vault_eth = add(fp.vault_eth, eth_back)?;



    let cost = fp.bid_eth_deployed.saturating_sub(principal0);
    crate::mech::positions::vault_bought(fp, tok_back, cost)?;
    fp.bid_liquidity.set(0);
    fp.bid_eth_deployed = 0;
    Ok((tok_back, eth_back))
}

///
///
///
pub fn book_nudge(fp: &mut FloorPool, r: &ActionResult) -> Result<(u64, u64)> {
    assert_d05(fp)?;
    require!(r.index == 1 && r.kind == Kinds::KIND_SWAP, HookError::InvariantViolated);
    let (quote, base) = split(&r.delta);
    require!(quote <= 0 && base >= 0, HookError::InvariantViolated);
    let spent = u64::try_from(quote.unsigned_abs()).map_err(|_| error!(HookError::MathOverflow))?;
    let got = pos_u64(base)?;







    require!(spent <= fp.min_bid, HookError::InvariantViolated);

    fp.vault_eth = fp.vault_eth.checked_sub(spent).ok_or(HookError::InvariantViolated)?;
    crate::mech::positions::vault_bought(fp, got, spent)?;
    Ok((spent, got))
}

///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BidPlan {
    pub lower: i32,
    pub upper: i32,
    pub liquidity: u128,
}

///
///
///
///
///
pub fn plan_deploy(fp: &FloorPool, sqrt_price_x64: u128, current: i32) -> Result<Option<BidPlan>> {

    assert_d05(fp)?;
    if fp.vault_eth < fp.min_bid {
        return Ok(None);
    }
    let s = fp.tick_spacing;
    let mut upper = fp.floor_tick;
    let mut lower = dear_edge(fp)?;
    let min_lower = math::ceil_to_spacing(current.checked_add(1).ok_or(HookError::MathOverflow)?, s)?;
    if lower < min_lower {
        lower = min_lower;
    }
    if lower >= upper {
        upper = lower.checked_add(s).ok_or(HookError::MathOverflow)?;
    }
    if upper > MAX_TICK {
        return Ok(None);
    }
    let liquidity = math::position_liquidity(sqrt_price_x64, lower, upper, fp.vault_eth, 0)?;
    if liquidity == 0 {
        return Ok(None);
    }
    Ok(Some(BidPlan { lower, upper, liquidity }))
}

///
pub fn open_action(plan: &BidPlan) -> HookAction {
    HookAction::ModifyPosition {
        salt: SALT_BID,
        tick_lower: plan.lower,
        tick_upper: plan.upper,
        liquidity_delta: plan.liquidity as i128,
    }
}

///
///
///
///
///
pub fn replica_owed(plan: &BidPlan) -> Result<u64> {
    let sa = sqrt_at_tick(plan.lower)?;
    let sb = sqrt_at_tick(plan.upper)?;
    carpenter_clmm_math::liquidity_math::get_delta_amount_0_unsigned(sa, sb, plan.liquidity, true)
        .map_err(|_| error!(HookError::MathOverflow))
}

///
///
pub fn book_open_plan(fp: &mut FloorPool, plan: &BidPlan) -> Result<()> {
    fp.bid_liquidity.set(plan.liquidity);
    fp.bid_lower = plan.lower;
    fp.bid_width = plan.upper.checked_sub(plan.lower).ok_or(HookError::MathOverflow)?;
    Ok(())
}

///
pub fn book_open_owed(fp: &mut FloorPool, owed: u64) -> Result<()> {
    fp.vault_eth = fp.vault_eth.checked_sub(owed).ok_or(HookError::InvariantViolated)?;
    fp.bid_eth_deployed = owed;
    Ok(())
}

///
///
pub fn owed_from_result(fp: &FloorPool, r: &ActionResult) -> Result<u64> {
    assert_d05(fp)?;
    require!(r.index == 0 && r.kind == Kinds::KIND_MODIFY_POSITION, HookError::InvariantViolated);
    let (quote, base) = split(&r.delta);
    require!(quote <= 0 && base == 0, HookError::InvariantViolated);
    u64::try_from(quote.unsigned_abs()).map_err(|_| error!(HookError::MathOverflow))
}

///
pub fn replaced_event(fp: &FloorPool, tokens_absorbed: u64, eth_back: u64, from_lower: i32, to_lower: i32) -> BidReplaced {
    BidReplaced { pool: fp.amm_pool, tokens_absorbed, eth_back, from_lower, to_lower }
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
pub fn unbacked_event(fp: &FloorPool) -> FloorUnbacked {
    FloorUnbacked { pool: fp.amm_pool, floor_tick: fp.floor_tick, vault_eth: fp.vault_eth, min_bid: fp.min_bid }
}
