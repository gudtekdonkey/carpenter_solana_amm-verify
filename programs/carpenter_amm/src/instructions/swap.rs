//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use carpenter_types::constants::SETTLES_FLAG;
use carpenter_types::hook::{
    ActionResult, AfterActionsArgs, AfterSwapArgs, BeforeSwapArgs, SwapParams, HOOK_INTERFACE_VERSION,
    MAX_HOOK_DATA, PHASE_AFTER_ACTIONS, PHASE_AFTER_SWAP, PHASE_BEFORE_SWAP,
};

use crate::amm_core::actions::{execute, ActionEnv, ActionTotals};
use crate::amm_core::hook_dispatch::{is_self_call, should_call, HookCallContext};
use crate::amm_core::hook_record::{
    accumulate_unspecified, amount_to_swap, balance_above_keep, call_record, check_full_fill, check_limit, check_rate,
    hook_delta, resolved_fee_override, settle_specified, swap_fees, taken, trader_amounts, trader_delta,
};
use crate::amm_core::settle::{check_vault_conservation, pay_in, pay_out, projected_vault};
use crate::amm_core::vaults::{admitted_hook, token_amount};
use crate::amm_core::walk::{pool_ref, walk, BitmapWords, PoolWork, TickArraySet, WalkParams, WalkResult};
use crate::constants::POOL_SEED;
use crate::errors::AmmError;
use crate::events::Swapped;
use crate::instructions::set_pool_paused::require_not_paused;
use crate::state::{BalanceDelta, HookOpResultV1, Pool, PoolSnapshot, TickArrayBitmap};

#[event_cpi]
#[derive(Accounts)]
pub struct Swap<'info> {
    pub trader: Signer<'info>,
    #[account(mut)]
    pub pool: AccountLoader<'info, Pool>,
///
///
///
///
    #[account(mut, constraint = bitmap.load()?.pool == pool.key() @ AmmError::SeedsMismatch)]
    pub bitmap: AccountLoader<'info, TickArrayBitmap>,
    #[account(mut)]
    pub trader_token_in: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
    #[account(mut)]
    pub trader_token_out: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
    #[account(mut, address = pool.load()?.vault0 @ AmmError::VaultMismatch)]
    pub vault0: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, address = pool.load()?.vault1 @ AmmError::VaultMismatch)]
    pub vault1: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(address = pool.load()?.mint0 @ AmmError::MintMismatch)]
    pub mint0: Box<InterfaceAccount<'info, Mint>>,
    #[account(address = pool.load()?.mint1 @ AmmError::MintMismatch)]
    pub mint1: Box<InterfaceAccount<'info, Mint>>,
    #[account(address = pool.load()?.token_program0 @ AmmError::TokenProgramMismatch)]
    pub token_program0: Interface<'info, TokenInterface>,
    #[account(address = pool.load()?.token_program1 @ AmmError::TokenProgramMismatch)]
    pub token_program1: Interface<'info, TokenInterface>,
///
///
    pub hook_program: Option<UncheckedAccount<'info>>,
}

///
struct PoolFixed {
    bump: u8,
    fee: u32,
    tick_spacing: u16,
    flags: u16,
    max_rounds: u8,
    callback_accounts: u8,
    mint0: Pubkey,
    mint1: Pubkey,
    hook: Pubkey,
    hook_authority: Pubkey,
///
    admitted: [Option<Pubkey>; 2],
}

impl PoolFixed {
    fn read(p: &Pool) -> Self {
        Self {
            bump: p.bump,
            fee: p.fee,
            tick_spacing: p.tick_spacing,
            flags: p.hook_flags,
            max_rounds: p.hook_max_rounds,
            callback_accounts: p.hook_callback_accounts,
            mint0: p.mint0,
            mint1: p.mint1,
            hook: p.hook,
            hook_authority: p.hook_authority,
            admitted: [admitted_hook(p, 0), admitted_hook(p, 1)],
        }
    }
}

///
///
pub enum SwapAmount {
///
    Specified(i128),
///
///
    FromBalance { keep: u64, limit_in_basis: u64 },
}

///
pub fn handler<'info>(
    ctx: Context<'info, Swap<'info>>,
    zero_for_one: bool,
    amount_specified: i128,
    sqrt_price_limit_x64: u128,
    amount_limit: u64,
    hook_data: Vec<u8>,
) -> Result<HookOpResultV1> {
    run(ctx, zero_for_one, SwapAmount::Specified(amount_specified), sqrt_price_limit_x64, amount_limit, hook_data)
}

///
///
///
pub fn from_balance_handler<'info>(
    ctx: Context<'info, Swap<'info>>,
    zero_for_one: bool,
    keep: u64,
    sqrt_price_limit_x64: u128,
    amount_limit: u64,
    limit_in_basis: u64,
    hook_data: Vec<u8>,
) -> Result<HookOpResultV1> {
    run(ctx, zero_for_one, SwapAmount::FromBalance { keep, limit_in_basis }, sqrt_price_limit_x64, amount_limit, hook_data)
}

///
///
///
///
fn resolve_input_balance(a: &Swap<'_>, zero_for_one: bool, keep: u64) -> Result<u64> {
    let (hook, hook_authority, mint_in) = {
        let p = a.pool.load()?;
        (p.hook, p.hook_authority, if zero_for_one { p.mint0 } else { p.mint1 })
    };
    require!(!is_self_call(&hook, &hook_authority, &a.trader.key()), AmmError::BalanceSwapSelfCall);
    let tin = a.trader_token_in.as_ref().ok_or(ErrorCode::AccountNotEnoughKeys)?;
    require_keys_eq!(tin.mint, mint_in, AmmError::MintMismatch);

    require_keys_eq!(tin.owner, a.trader.key(), AmmError::BalanceSwapNotOwner);
    balance_above_keep(tin.amount, keep)
}

fn run<'info>(
    ctx: Context<'info, Swap<'info>>,
    zero_for_one: bool,
    amount: SwapAmount,
    sqrt_price_limit_x64: u128,
    amount_limit: u64,
    hook_data: Vec<u8>,
) -> Result<HookOpResultV1> {


    require!(hook_data.len() <= MAX_HOOK_DATA, AmmError::HookDataTooLong);



    require_not_paused(&*ctx.accounts.pool.load()?)?;


    let (amount_specified, balance_mode): (i128, Option<(u64, u64)>) = match amount {
        SwapAmount::Specified(a) => (a, None),
        SwapAmount::FromBalance { keep, limit_in_basis } => {
            let resolved = resolve_input_balance(&*ctx.accounts, zero_for_one, keep)?;
            (-(resolved as i128), Some((resolved, limit_in_basis)))
        }
    };

    require!(amount_specified != 0, AmmError::AmountZero);
    require!(amount_specified.unsigned_abs() <= u64::MAX as u128, AmmError::MathOverflow);

    let pool_info = ctx.accounts.pool.to_account_info();
    let pool_key = *pool_info.key;
    let trader_info = ctx.accounts.trader.to_account_info();
    let trader_key = *trader_info.key;
    let fixed = {
        let p = pool_ref(&pool_info)?;
        PoolFixed::read(&p)
    };
    let mut st = PoolWork::reload(&pool_info)?;


    let self_call = is_self_call(&fixed.hook, &fixed.hook_authority, &trader_key);
    let has_hook = fixed.hook != Pubkey::default();

    let calls_before = should_call(fixed.flags, PHASE_BEFORE_SWAP, self_call);
    let calls_after = should_call(fixed.flags, PHASE_AFTER_SWAP, self_call);



    let remaining = ctx.remaining_accounts;
    let slice_len = if has_hook && (calls_before || calls_after) { fixed.callback_accounts as usize } else { 0 };
    require!(remaining.len() >= slice_len, ErrorCode::AccountNotEnoughKeys);
    let (slice, set_accounts) = remaining.split_at(slice_len);
    let mut set = TickArraySet::new(set_accounts)?;
    let bitmap_info = ctx.accounts.bitmap.to_account_info();
    let mut bitmap = BitmapWords::read(&bitmap_info, fixed.tick_spacing)?;

    let supply0 = ctx.accounts.mint0.supply;
    let supply1 = ctx.accounts.mint1.supply;
    let decimals0 = ctx.accounts.mint0.decimals;
    let decimals1 = ctx.accounts.mint1.decimals;
    let vault0_before = ctx.accounts.vault0.amount;
    let vault1_before = ctx.accounts.vault1.amount;


    let (mint_in, mint_out) = if zero_for_one { (fixed.mint0, fixed.mint1) } else { (fixed.mint1, fixed.mint0) };
    let trader_accounts = if self_call {
        None
    } else {
        let tin = ctx.accounts.trader_token_in.as_ref().ok_or(ErrorCode::AccountNotEnoughKeys)?;
        let tout = ctx.accounts.trader_token_out.as_ref().ok_or(ErrorCode::AccountNotEnoughKeys)?;
        require_keys_eq!(tin.mint, mint_in, AmmError::MintMismatch);
        require_keys_eq!(tout.mint, mint_out, AmmError::MintMismatch);
        Some((tin.to_account_info(), tout.to_account_info()))
    };



    let mut amm_keys: Vec<Pubkey> = vec![
        pool_key,
        *bitmap_info.key,
        ctx.accounts.vault0.key(),
        ctx.accounts.vault1.key(),
        fixed.mint0,
        fixed.mint1,
        trader_key,
        ctx.accounts.event_authority.key(),
    ];
    if let Some((tin, tout)) = &trader_accounts {
        amm_keys.push(*tin.key);
        amm_keys.push(*tout.key);
    }
    amm_keys.extend(set.keys().copied());


    let fee_le = fixed.fee.to_le_bytes();
    let spacing_le = fixed.tick_spacing.to_le_bytes();
    let bump = [fixed.bump];
    let pool_seeds: [&[u8]; 7] = [
        POOL_SEED,
        fixed.mint0.as_ref(),
        fixed.mint1.as_ref(),
        &fee_le,
        &spacing_le,
        fixed.hook.as_ref(),
        &bump,
    ];




    let settles = has_hook && fixed.flags & SETTLES_FLAG != 0 && (calls_before || calls_after);
    let settle_infos: Vec<AccountInfo<'info>> = if settles {
        vec![
            ctx.accounts.vault0.to_account_info(),
            ctx.accounts.vault1.to_account_info(),
            ctx.accounts.mint0.to_account_info(),
            ctx.accounts.mint1.to_account_info(),
            ctx.accounts.token_program0.to_account_info(),
            ctx.accounts.token_program1.to_account_info(),
        ]
    } else {
        Vec::new()
    };

    let mut settled: [u64; 2] = [0, 0];
    let mut vault_seen: [u64; 2] = [vault0_before, vault1_before];

    let hook_info = if calls_before || calls_after {
        let h = ctx.accounts.hook_program.as_ref().ok_or(ErrorCode::AccountNotEnoughKeys)?;
        Some(h.to_account_info())
    } else {
        None
    };

    let hook_ref: &AccountInfo<'info> = hook_info.as_ref().unwrap_or(&pool_info);
    let hctx = HookCallContext {
        hook_program: hook_ref,
        pool: &pool_info,
        pool_signer_seeds: &pool_seeds,
        slice,
        amm_accounts: &amm_keys,
        pool_hook: fixed.hook,
        flags: fixed.flags,
        max_rounds: fixed.max_rounds,
        callback_accounts: fixed.callback_accounts,
        settle_accounts: &settle_infos,
    };

    let params = SwapParams { zero_for_one, amount_specified, sqrt_price_limit_x64 };
    let pre = st.snapshot();
    let mut spec: i128 = 0;
    let mut unspec: i128 = 0;
    let mut fee_override: Option<u32> = None;
    let mut signals: u8 = 0;









    let mut rec0 = None;


    if calls_before {
        st.flush(&pool_info)?;
        let args = BeforeSwapArgs {
            version: HOOK_INTERFACE_VERSION,
            phase: PHASE_BEFORE_SWAP,
            round: 0,
            pool: pool_key.to_bytes(),
            sender: trader_key.to_bytes(),
            params,
            pre,
            supply0,
            supply1,
            hook_claims0: st.hook_claims0,
            hook_claims1: st.hook_claims1,
            hook_data: hook_data.clone(),
        };
        let r = call_record(&hctx, PHASE_BEFORE_SWAP, &args)?;
        st = PoolWork::reload(&pool_info)?;
        if settles {
            credit_settled(&settle_infos, &mut st, &mut vault_seen, &mut settled)?;
        }
        spec = r.delta_specified;
        unspec = r.delta_unspecified;
        fee_override = resolved_fee_override(&r, fixed.flags, fixed.fee);
        signals |= r.signals;
        rec0 = Some(r);
    }


    let to_swap = amount_to_swap(amount_specified, spec, settles)?;
    let lp_fee = fee_override.unwrap_or(st.lp_fee);




    let hook_limit = rec0.as_ref().map(|r| r.sqrt_price_limit_x64).unwrap_or(0);
    let mut walk_limit = sqrt_price_limit_x64;
    let mut skip_walk = false;
    if hook_limit != 0 {
        if zero_for_one {
            if hook_limit >= st.sqrt_price_x64 {
                skip_walk = true;
            } else {
                walk_limit = walk_limit.max(hook_limit);
            }
        } else if hook_limit <= st.sqrt_price_x64 {
            skip_walk = true;
        } else {
            walk_limit = walk_limit.min(hook_limit);
        }
    }
    let walked: WalkResult = if skip_walk {
        WalkResult::default()
    } else {
        walk(
            &mut st,
            &pool_key,
            &bitmap,
            &mut set,
            &WalkParams { zero_for_one, amount_specified: to_swap, sqrt_price_limit_x64: walk_limit, lp_fee },
            AmmError::TickArrayMissing,
        )?
    };
    let swap_delta = walked.delta;
    let post = st.snapshot();


    let mut rec1 = None;
    if calls_after {
        st.flush(&pool_info)?;
        let args = AfterSwapArgs {
            version: HOOK_INTERFACE_VERSION,
            phase: PHASE_AFTER_SWAP,
            round: 1,
            pool: pool_key.to_bytes(),
            sender: trader_key.to_bytes(),
            params,
            pre,
            post,
            swap_delta,
            before_delta_specified: spec,
            before_delta_unspecified: unspec,
            supply0,
            supply1,

            hook_claims0: st.hook_claims0,
            hook_claims1: st.hook_claims1,
            hook_data,
        };
        let r = call_record(&hctx, PHASE_AFTER_SWAP, &args)?;
        st = PoolWork::reload(&pool_info)?;
        if settles {
            credit_settled(&settle_infos, &mut st, &mut vault_seen, &mut settled)?;

            let fill = settle_specified(amount_specified, to_swap, zero_for_one, &swap_delta, r.delta_specified)?;
            spec = spec.checked_add(fill).ok_or(AmmError::MathOverflow)?;
        }
        unspec = accumulate_unspecified(unspec, &r)?;
        signals |= r.signals;
        rec1 = Some(r);
    }


    let hdelta = hook_delta(amount_specified, zero_for_one, spec, unspec, settles)?;
    let tdelta = trader_delta(&swap_delta, &hdelta)?;
    st.apply_claims_delta(&hdelta)?;
    st.hook_fees0 = st.hook_fees0.checked_add(taken(hdelta.amount0)).ok_or(AmmError::MathOverflow)?;
    st.hook_fees1 = st.hook_fees1.checked_add(taken(hdelta.amount1)).ok_or(AmmError::MathOverflow)?;






    let (fee_split0, fee_split1) = swap_fees(amount_specified, zero_for_one, rec0.as_ref(), rec1.as_ref());


    let mut totals = ActionTotals::default();
    if let Some(r1) = rec1 {




        let v0 = vault0_before.checked_add(settled[0]).ok_or(AmmError::MathOverflow)?;
        let v1 = vault1_before.checked_add(settled[1]).ok_or(AmmError::MathOverflow)?;
        let (vault0_after, vault1_after) = projected_vault(v0, v1, &tdelta, self_call)?;
        let mut env = ActionEnv {
            pool: &pool_info,
            pool_key,
            hook_authority: fixed.hook_authority,
            bitmap_info: &bitmap_info,
            bitmap: &mut bitmap,
            set: &mut set,
            vault0: vault0_after,
            vault1: vault1_after,
        };
        let mut round: u8 = 1;
        let mut results: Vec<ActionResult> = execute(&mut env, &mut st, &r1.actions, &mut totals)?;
        let mut want = r1.want_callback;
        while want {

            require!((round as u16) + 1 < fixed.max_rounds as u16, AmmError::HookRoundsExceeded);
            round += 1;
            st.flush(&pool_info)?;
            let args = AfterActionsArgs {
                version: HOOK_INTERFACE_VERSION,
                phase: PHASE_AFTER_ACTIONS,
                round,
                pool: pool_key.to_bytes(),
                post: st.snapshot(),
                supply0,
                supply1,

                hook_claims0: st.hook_claims0,
                hook_claims1: st.hook_claims1,
                results: core::mem::take(&mut results),
            };
            let rec = call_record(&hctx, PHASE_AFTER_ACTIONS, &args)?;
            st = PoolWork::reload(&pool_info)?;
            if settles {
                credit_settled(&settle_infos, &mut st, &mut vault_seen, &mut settled)?;
            }
            signals |= rec.signals;
            results = execute(&mut env, &mut st, &rec.actions, &mut totals)?;
            want = rec.want_callback;
        }
    }


    let amounts = trader_amounts(&tdelta, zero_for_one)?;
    check_limit(amount_specified, &amounts, amount_limit)?;


    if let Some((resolved, limit_in_basis)) = balance_mode {
        check_full_fill(&amounts, resolved)?;
        check_rate(&amounts, amount_limit, limit_in_basis)?;
    }
    st.count_input(zero_for_one, amounts.amount_in, walked.lp_fees)?;
    st.swap_count = st.swap_count.checked_add(1).ok_or(AmmError::MathOverflow)?;


    let (in0, out0, in1, out1) = if self_call {
        (0, 0, 0, 0)
    } else if zero_for_one {
        (amounts.amount_in, 0, 0, amounts.amount_out)
    } else {
        (0, amounts.amount_out, amounts.amount_in, 0)
    };
    match &trader_accounts {
        None => {

            st.apply_claims_delta(&tdelta)?;
            st.flush(&pool_info)?;
        }
        Some((tin, tout)) => {
            st.flush(&pool_info)?;



            let (side_in, side_out) = if zero_for_one { (0usize, 1usize) } else { (1, 0) };
            let (tp_in, vault_in, m_in, dec_in, tp_out, vault_out, m_out, dec_out) = if zero_for_one {
                (
                    ctx.accounts.token_program0.to_account_info(),
                    ctx.accounts.vault0.to_account_info(),
                    ctx.accounts.mint0.to_account_info(),
                    decimals0,
                    ctx.accounts.token_program1.to_account_info(),
                    ctx.accounts.vault1.to_account_info(),
                    ctx.accounts.mint1.to_account_info(),
                    decimals1,
                )
            } else {
                (
                    ctx.accounts.token_program1.to_account_info(),
                    ctx.accounts.vault1.to_account_info(),
                    ctx.accounts.mint1.to_account_info(),
                    decimals1,
                    ctx.accounts.token_program0.to_account_info(),
                    ctx.accounts.vault0.to_account_info(),
                    ctx.accounts.mint0.to_account_info(),
                    decimals0,
                )
            };
            pay_in(
                &tp_in,
                tin,
                &m_in,
                &vault_in,
                &trader_info,
                &pool_info,
                &pool_seeds,
                amounts.amount_in,
                dec_in,
                fixed.admitted[side_in],
                remaining,
            )?;
            pay_out(
                &tp_out,
                &vault_out,
                &m_out,
                tout,
                &pool_info,
                &pool_seeds,
                amounts.amount_out,
                dec_out,
                fixed.admitted[side_out],
                remaining,
            )?;
        }
    }


    ctx.accounts.vault0.reload()?;
    ctx.accounts.vault1.reload()?;
    check_vault_conservation(
        vault0_before.checked_add(settled[0]).ok_or(AmmError::MathOverflow)?,
        ctx.accounts.vault0.amount,
        in0,
        out0,
        st.hook_claims0,
        st.protocol_fees_owed0,
    )?;
    check_vault_conservation(
        vault1_before.checked_add(settled[1]).ok_or(AmmError::MathOverflow)?,
        ctx.accounts.vault1.amount,
        in1,
        out1,
        st.hook_claims1,
        st.protocol_fees_owed1,
    )?;


    emit_cpi!(Swapped {
        pool: pool_key,
        trader: trader_key,
        zero_for_one,
        amount_in: amounts.amount_in,
        amount_out: amounts.amount_out,
        hook_delta0: hdelta.amount0,
        hook_delta1: hdelta.amount1,
        lp_fee_total: walked.lp_fees,
        fee_override,
        pre_sqrt_price_x64: pre.sqrt_price_x64,
        pre_tick: pre.tick,
        post_sqrt_price_x64: st.sqrt_price_x64,
        post_tick: st.tick,
        signals,
        bid_replaced: totals.bid_replaced,
        absorbed: totals.absorbed,
        nudge_spent: totals.nudge_spent,


        protocol_fee_total: walked.protocol_fees,
        protocol_fee_currency: walked.protocol_currency,
        fee_split0,
        fee_split1,

        walk_sqrt_price_x64: post.sqrt_price_x64,
        walk_tick: post.tick,
    });




    let (fees0, fees1) = if zero_for_one { (walked.lp_fees, 0) } else { (0, walked.lp_fees) };
    Ok(HookOpResultV1 {
        version: HOOK_INTERFACE_VERSION,
        delta: BalanceDelta { amount0: tdelta.amount0, amount1: tdelta.amount1 },
        fees0,
        fees1,
        post: PoolSnapshot::from(st.snapshot()),
        hook_claims0: st.hook_claims0,
        hook_claims1: st.hook_claims1,
    })
}

///
///
///
///
fn credit_settled(vaults: &[AccountInfo], st: &mut PoolWork, seen: &mut [u64; 2], settled: &mut [u64; 2]) -> Result<()> {
    for c in 0..2 {
        let now = token_amount(&vaults[c])?;
        require!(now >= seen[c], AmmError::HookSettleVaultDecreased);
        let inc = now - seen[c];
        if inc > 0 {
            if c == 0 {
                st.hook_claims0 = st.hook_claims0.checked_add(inc).ok_or(AmmError::MathOverflow)?;
            } else {
                st.hook_claims1 = st.hook_claims1.checked_add(inc).ok_or(AmmError::MathOverflow)?;
            }
            settled[c] = settled[c].checked_add(inc).ok_or(AmmError::MathOverflow)?;
            seen[c] = now;
        }
    }
    Ok(())
}
