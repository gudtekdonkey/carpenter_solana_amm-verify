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
use carpenter_types::hook::{ActionResult, BalanceDelta, HookAction};
use carpenter_types::layout::tick_array_start_index;

use crate::amm_core::bitmap::flip_for;
use crate::amm_core::fees::clamp_paid_fee;
use crate::amm_core::positions::{modify_position, set_range, TickArrays};
use crate::amm_core::ticks::check_range;
use crate::amm_core::walk::{pool_mut, walk, BitmapWords, PoolWork, TickArraySet, WalkParams, WalkResult};
use crate::constants::POSITION_SEED;
use crate::errors::AmmError;
use crate::state::{Position, TickArray};

///
///
pub struct ActionEnv<'a, 'r, 'info> {
///
    pub pool: &'a AccountInfo<'info>,
    pub pool_key: Pubkey,
///
    pub hook_authority: Pubkey,
    pub bitmap_info: &'a AccountInfo<'info>,
    pub bitmap: &'a mut BitmapWords,
    pub set: &'a mut TickArraySet<'r, 'info>,
///
    pub vault0: u64,
///
    pub vault1: u64,
}

///
///
///
///
///
///
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ActionTotals {
///
    pub nudge_spent: u64,
///
    pub absorbed: u64,
///
    pub bid_replaced: bool,
///
    emptied: Vec<u64>,
}

///
///
pub fn execute(
    env: &mut ActionEnv<'_, '_, '_>,
    st: &mut PoolWork,
    actions: &[HookAction],
    totals: &mut ActionTotals,
) -> Result<Vec<ActionResult>> {
    let mut results = Vec::with_capacity(actions.len());
    for (i, action) in actions.iter().enumerate() {
        let index = u8::try_from(i).map_err(|_| AmmError::HookFieldNotAllowed)?;
        let r = match *action {
            HookAction::Swap { zero_for_one, amount_specified, sqrt_price_limit_x64 } => {
                swap_action(env, st, index, zero_for_one, amount_specified, sqrt_price_limit_x64, totals)?
            }
            HookAction::ModifyPosition { salt, tick_lower, tick_upper, liquidity_delta } => {
                modify_action(env, st, index, salt, tick_lower, tick_upper, liquidity_delta, totals)?
            }
        };
        results.push(r);
    }
    Ok(results)
}

fn swap_action(
    env: &mut ActionEnv<'_, '_, '_>,
    st: &mut PoolWork,
    index: u8,
    zero_for_one: bool,
    amount_specified: i128,
    sqrt_price_limit_x64: u128,
    totals: &mut ActionTotals,
) -> Result<ActionResult> {






    let at_or_past_limit = if zero_for_one {
        st.sqrt_price_x64 <= sqrt_price_limit_x64
    } else {
        st.sqrt_price_x64 >= sqrt_price_limit_x64
    };
    let res = if at_or_past_limit || amount_specified == 0 {
        WalkResult { swap_fee: st.lp_fee, ..WalkResult::default() }
    } else {
        let params = WalkParams { zero_for_one, amount_specified, sqrt_price_limit_x64, lp_fee: st.lp_fee };
        walk(st, &env.pool_key, &*env.bitmap, &mut *env.set, &params, AmmError::HookActionAccountMissing)?
    };

    st.apply_claims_delta(&res.delta)?;





    st.count_input(zero_for_one, res.amount_in, res.lp_fees)?;
    totals.nudge_spent = totals.nudge_spent.checked_add(res.amount_in).ok_or(AmmError::MathOverflow)?;
    let (fees0, fees1) = if zero_for_one { (res.lp_fees, 0) } else { (0, res.lp_fees) };
    Ok(ActionResult {
        index,
        kind: HookAction::KIND_SWAP,
        delta: res.delta,
        fees0,
        fees1,
        post_sqrt_price_x64: st.sqrt_price_x64,
        post_tick: st.tick,
    })
}

#[allow(clippy::too_many_arguments)]
fn modify_action(
    env: &mut ActionEnv<'_, '_, '_>,
    st: &mut PoolWork,
    index: u8,
    salt: u64,
    tick_lower: i32,
    tick_upper: i32,
    liquidity_delta: i128,
    totals: &mut ActionTotals,
) -> Result<ActionResult> {
    let spacing = st.tick_spacing;
    check_range(tick_lower, tick_upper, spacing)?;




    let (position_key, _) = Pubkey::find_program_address(
        &[POSITION_SEED, env.pool_key.as_ref(), env.hook_authority.as_ref(), &salt.to_le_bytes()],
        &crate::ID,
    );
    let position_info = env.set.find_key(&position_key).ok_or(AmmError::HookActionAccountMissing)?;
    require!(position_info.is_writable, AmmError::HookActionAccountMissing);
    require_keys_eq!(*position_info.owner, crate::ID, AmmError::SeedsMismatch);
    {
        let data = position_info.try_borrow_data()?;
        require!(
            data.len() == Position::LEN && data[..8] == carpenter_types::disc::account::POSITION,
            AmmError::SeedsMismatch
        );
        let p = bytemuck::from_bytes::<Position>(&data[8..Position::LEN]);
        require_keys_eq!(p.pool, env.pool_key, AmmError::SeedsMismatch);
        require_keys_eq!(p.owner, env.hook_authority, AmmError::SeedsMismatch);
        require_keys_eq!(p.position_id, env.hook_authority, AmmError::SeedsMismatch);
        require!(p.salt == salt, AmmError::SeedsMismatch);
    }


    let lower_start = tick_array_start_index(tick_lower, spacing);
    let upper_start = tick_array_start_index(tick_upper, spacing);
    let lower = env.set.find(&env.pool_key, lower_start, AmmError::HookActionAccountMissing)?;
    let upper = env.set.find(&env.pool_key, upper_start, AmmError::HookActionAccountMissing)?;
    require!(env.set.info(lower).is_writable && env.set.info(upper).is_writable, AmmError::HookActionAccountMissing);


    st.flush(env.pool)?;
    let (principal, owed0, owed1, flips, liquidity_after) = {
        let mut pool = pool_mut(env.pool)?;
        let mut position_data = position_info.try_borrow_mut_data()?;
        let position = bytemuck::from_bytes_mut::<Position>(&mut position_data[8..Position::LEN]);






        set_range(position, tick_lower, tick_upper)?;
        let mut lower_data = env.set.info(lower).try_borrow_mut_data()?;
        if lower == upper {
            let arr = bytemuck::from_bytes_mut::<TickArray>(&mut lower_data[8..TickArray::LEN]);
            let before = arr.initialized_tick_count;
            let (p, f0, f1) = modify_position(&mut pool, position, TickArrays::Same(&mut *arr), liquidity_delta)?;
            let flip = flip_for(before, arr.initialized_tick_count);
            (p, f0, f1, [(lower_start, flip), (upper_start, None)], position.liquidity.get())
        } else {
            let mut upper_data = env.set.info(upper).try_borrow_mut_data()?;
            let arr_lower = bytemuck::from_bytes_mut::<TickArray>(&mut lower_data[8..TickArray::LEN]);
            let arr_upper = bytemuck::from_bytes_mut::<TickArray>(&mut upper_data[8..TickArray::LEN]);
            let (bl, bu) = (arr_lower.initialized_tick_count, arr_upper.initialized_tick_count);
            let (p, f0, f1) = modify_position(
                &mut pool,
                position,
                TickArrays::Split { lower: &mut *arr_lower, upper: &mut *arr_upper },
                liquidity_delta,
            )?;
            let fl = flip_for(bl, arr_lower.initialized_tick_count);
            let fu = flip_for(bu, arr_upper.initialized_tick_count);
            (p, f0, f1, [(lower_start, fl), (upper_start, fu)], position.liquidity.get())
        }
    };
    *st = PoolWork::reload(env.pool)?;




    if liquidity_delta < 0 {
        totals.absorbed = totals
            .absorbed
            .checked_add(u64::try_from(principal.amount1.max(0)).map_err(|_| AmmError::MathOverflow)?)
            .ok_or(AmmError::MathOverflow)?;
        if liquidity_after == 0 && !totals.emptied.contains(&salt) {
            totals.emptied.push(salt);
        }
    } else if liquidity_delta > 0 && totals.emptied.contains(&salt) {
        totals.bid_replaced = true;
    }


    for (start, flip) in flips {
        if let Some(initialized) = flip {
            env.bitmap.set(env.bitmap_info, start, initialized)?;
        }
    }


    let out0 = u64::try_from(principal.amount0.max(0)).map_err(|_| AmmError::MathOverflow)?;
    let out1 = u64::try_from(principal.amount1.max(0)).map_err(|_| AmmError::MathOverflow)?;
    let (paid0, _truncated0) = clamp_paid_fee(owed0, env.vault0, st.hook_claims0, st.protocol_fees_owed0, out0);
    let (paid1, _truncated1) = clamp_paid_fee(owed1, env.vault1, st.hook_claims1, st.protocol_fees_owed1, out1);
    let delta = BalanceDelta {
        amount0: principal.amount0.checked_add(paid0 as i128).ok_or(AmmError::MathOverflow)?,
        amount1: principal.amount1.checked_add(paid1 as i128).ok_or(AmmError::MathOverflow)?,
    };


    st.apply_claims_delta(&delta)?;
    Ok(ActionResult {
        index,
        kind: HookAction::KIND_MODIFY_POSITION,
        delta,
        fees0: paid0,
        fees1: paid1,
        post_sqrt_price_x64: st.sqrt_price_x64,
        post_tick: st.tick,
    })
}
