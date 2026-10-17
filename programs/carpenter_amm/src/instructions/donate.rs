//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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
use carpenter_types::hook::{DonateArgs, HOOK_INTERFACE_VERSION, MAX_HOOK_DATA, PHASE_AFTER_DONATE, PHASE_BEFORE_DONATE};

use crate::amm_core::fees::donate_growth;
use crate::amm_core::hook_dispatch::{call_simple, is_self_call, should_call, HookCallContext, PoolSeeds};
use crate::amm_core::vaults::{admitted_hook, transfer_checked_in, TransferIn};
use crate::constants::{DONATE_MAX_AMOUNT, MIN_DONATE_LIQUIDITY};
use crate::errors::AmmError;
use crate::events::Donated;
use crate::instructions::set_pool_paused::require_not_paused;
use crate::state::Pool;

#[event_cpi]
#[derive(Accounts)]
pub struct Donate<'info> {
    pub donor: Signer<'info>,
    #[account(mut)]
    pub pool: AccountLoader<'info, Pool>,
    #[account(mut)]
    pub token0: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub token1: Box<InterfaceAccount<'info, TokenAccount>>,
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
    pub hook_program: Option<UncheckedAccount<'info>>,
}

pub fn handler<'info>(ctx: Context<'info, Donate<'info>>, amount0: u64, amount1: u64, hook_data: Vec<u8>) -> Result<()> {
    require!(hook_data.len() <= MAX_HOOK_DATA, AmmError::HookDataTooLong);
    require!(amount0 <= DONATE_MAX_AMOUNT && amount1 <= DONATE_MAX_AMOUNT, AmmError::DonateTooLarge);

    let donor = ctx.accounts.donor.key();
    let pool_key = ctx.accounts.pool.key();
    let before0 = ctx.accounts.vault0.amount;
    let before1 = ctx.accounts.vault1.amount;
    let (seeds_buf, hook, flags, max_rounds, callback_accounts, self_call, admitted) = {
        let p = ctx.accounts.pool.load()?;
        require_not_paused(&p)?;
        require!(p.liquidity.get() >= MIN_DONATE_LIQUIDITY, AmmError::NoLiquidity);
        let self_call = is_self_call(&p.hook, &p.hook_authority, &donor);

        let admitted = [admitted_hook(&p, 0), admitted_hook(&p, 1)];
        (PoolSeeds::of(&p), p.hook, p.hook_flags, p.hook_max_rounds, p.hook_callback_accounts, self_call, admitted)
    };
    let seeds = seeds_buf.seeds();

    let wants_before = should_call(flags, PHASE_BEFORE_DONATE, self_call);
    let wants_after = should_call(flags, PHASE_AFTER_DONATE, self_call);
    let pool_info = ctx.accounts.pool.to_account_info();
    let hook_program_info = ctx.accounts.hook_program.as_ref().map(|h| h.to_account_info());
    let n = callback_accounts as usize;
    let amm_accounts = [
        donor,
        ctx.accounts.token0.key(),
        ctx.accounts.token1.key(),
        ctx.accounts.vault0.key(),
        ctx.accounts.vault1.key(),
    ];


    if wants_before || wants_after {
        require!(ctx.remaining_accounts.len() >= n, ErrorCode::AccountNotEnoughKeys);
        require!(hook_program_info.is_some(), AmmError::HookBadReturn);
    }
    let args = |phase: u8| DonateArgs {
        version: HOOK_INTERFACE_VERSION,
        phase,
        round: 0,
        pool: pool_key.to_bytes(),
        sender: donor.to_bytes(),
        amount0,
        amount1,
        hook_data: hook_data.clone(),
    };
    if wants_before {
        let hp = hook_program_info.as_ref().ok_or(AmmError::HookBadReturn)?;
        let hctx = HookCallContext {
            settle_accounts: &[],
            hook_program: hp,
            pool: &pool_info,
            pool_signer_seeds: &seeds,
            slice: &ctx.remaining_accounts[..n],
            amm_accounts: &amm_accounts,
            pool_hook: hook,
            flags,
            max_rounds,
            callback_accounts,
        };
        call_simple(&hctx, PHASE_BEFORE_DONATE, &args(PHASE_BEFORE_DONATE))?;
    }


    {
        let mut p = ctx.accounts.pool.load_mut()?;
        let liquidity = p.liquidity.get();
        if amount0 > 0 {
            let g = p.fee_growth_global0_x64.get().wrapping_add(donate_growth(amount0, liquidity)?);
            p.fee_growth_global0_x64.set(g);
        }
        if amount1 > 0 {
            let g = p.fee_growth_global1_x64.get().wrapping_add(donate_growth(amount1, liquidity)?);
            p.fee_growth_global1_x64.set(g);
        }
        if self_call {
            p.hook_claims0 = p.hook_claims0.checked_sub(amount0).ok_or(AmmError::HookClaimsInsufficient)?;
            p.hook_claims1 = p.hook_claims1.checked_sub(amount1).ok_or(AmmError::HookClaimsInsufficient)?;
        }
    }


    if wants_after {
        let hp = hook_program_info.as_ref().ok_or(AmmError::HookBadReturn)?;
        let hctx = HookCallContext {
            settle_accounts: &[],
            hook_program: hp,
            pool: &pool_info,
            pool_signer_seeds: &seeds,
            slice: &ctx.remaining_accounts[..n],
            amm_accounts: &amm_accounts,
            pool_hook: hook,
            flags,
            max_rounds,
            callback_accounts,
        };
        call_simple(&hctx, PHASE_AFTER_DONATE, &args(PHASE_AFTER_DONATE))?;
    }


    if !self_call {
        let donor_info = ctx.accounts.donor.to_account_info();
        for c in 0..2u8 {
            let (amount, tp, src, mint, vault, decimals) = if c == 0 {
                (
                    amount0,
                    ctx.accounts.token_program0.to_account_info(),
                    ctx.accounts.token0.to_account_info(),
                    ctx.accounts.mint0.to_account_info(),
                    ctx.accounts.vault0.to_account_info(),
                    ctx.accounts.mint0.decimals,
                )
            } else {
                (
                    amount1,
                    ctx.accounts.token_program1.to_account_info(),
                    ctx.accounts.token1.to_account_info(),
                    ctx.accounts.mint1.to_account_info(),
                    ctx.accounts.vault1.to_account_info(),
                    ctx.accounts.mint1.decimals,
                )
            };
            transfer_checked_in(
                &TransferIn {
                    token_program: &tp,
                    source: &src,
                    mint: &mint,
                    vault: &vault,
                    owner: &donor_info,
                    pool: &pool_info,
                    pool_signer_seeds: &seeds,
                    decimals,
                    admitted: admitted[c as usize],
                    extra: ctx.remaining_accounts,
                },
                amount,
            )?;
        }
    }


    ctx.accounts.vault0.reload()?;
    ctx.accounts.vault1.reload()?;
    let (moved0, moved1) = if self_call { (0, 0) } else { (amount0, amount1) };
    require!(before0.checked_add(moved0) == Some(ctx.accounts.vault0.amount), AmmError::VaultConservationViolated);
    require!(before1.checked_add(moved1) == Some(ctx.accounts.vault1.amount), AmmError::VaultConservationViolated);
    {
        let p = ctx.accounts.pool.load()?;
        require!(
            p.hook_claims0 as u128 + p.protocol_fees_owed0 as u128 <= ctx.accounts.vault0.amount as u128
                && p.hook_claims1 as u128 + p.protocol_fees_owed1 as u128 <= ctx.accounts.vault1.amount as u128,



            AmmError::VaultShortfall
        );
    }

    emit_cpi!(Donated { pool: pool_key, donor, amount0, amount1 });
    Ok(())
}
