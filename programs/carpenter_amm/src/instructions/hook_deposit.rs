//!
//!
//!
//!
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
use carpenter_types::hook::{BalanceDelta, HookOpResultV1, HOOK_INTERFACE_VERSION};

use crate::amm_core::hook_dispatch::{snapshot, PoolSeeds};
use crate::amm_core::vaults::{admitted_hook, transfer_checked_in, TransferIn};
use crate::errors::AmmError;
use crate::events::HookDeposited;
use crate::instructions::set_pool_paused::require_not_paused;
use crate::state::Pool;

#[event_cpi]
#[derive(Accounts)]
pub struct HookDeposit<'info> {
    pub depositor: Signer<'info>,
    #[account(mut)]
    pub pool: AccountLoader<'info, Pool>,
    #[account(mut)]
    pub source: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    pub token_program: Interface<'info, TokenInterface>,
}

///
///
pub(crate) fn bind_hook_currency(
    pool: &Pool,
    currency: u8,
    vault: &Pubkey,
    mint: &Pubkey,
    token_program: &Pubkey,
) -> Result<()> {
    require!(currency <= 1, AmmError::CurrencyInvalid);
    let (v, m, t) = if currency == 0 {
        (pool.vault0, pool.mint0, pool.token_program0)
    } else {
        (pool.vault1, pool.mint1, pool.token_program1)
    };
    require_keys_eq!(*vault, v, AmmError::VaultMismatch);
    require_keys_eq!(*mint, m, AmmError::MintMismatch);
    require_keys_eq!(*token_program, t, AmmError::TokenProgramMismatch);
    Ok(())
}

///
pub(crate) fn single_currency_result(pool: &Pool, currency: u8, delta: i128) -> HookOpResultV1 {
    HookOpResultV1 {
        version: HOOK_INTERFACE_VERSION,
        delta: if currency == 0 {
            BalanceDelta { amount0: delta, amount1: 0 }
        } else {
            BalanceDelta { amount0: 0, amount1: delta }
        },
        fees0: 0,
        fees1: 0,
        post: snapshot(pool),
        hook_claims0: pool.hook_claims0,
        hook_claims1: pool.hook_claims1,
    }
}

pub fn handler<'info>(
    ctx: Context<'info, HookDeposit<'info>>,
    currency: u8,
    amount: u64,
) -> Result<crate::state::HookOpResultV1> {
    let (seeds_buf, admitted) = {
        let p = ctx.accounts.pool.load()?;
        require_keys_neq!(p.hook, Pubkey::default(), AmmError::NoHook);
        require_not_paused(&p)?;
        bind_hook_currency(
            &p,
            currency,
            &ctx.accounts.vault.key(),
            &ctx.accounts.mint.key(),
            &ctx.accounts.token_program.key(),
        )?;

        (PoolSeeds::of(&p), admitted_hook(&p, currency))
    };
    let seeds = seeds_buf.seeds();

    let before = ctx.accounts.vault.amount;
    {
        let token_program = ctx.accounts.token_program.to_account_info();
        let source = ctx.accounts.source.to_account_info();
        let mint = ctx.accounts.mint.to_account_info();
        let vault = ctx.accounts.vault.to_account_info();
        let owner = ctx.accounts.depositor.to_account_info();
        let pool_info = ctx.accounts.pool.to_account_info();
        transfer_checked_in(
            &TransferIn {
                token_program: &token_program,
                source: &source,
                mint: &mint,
                vault: &vault,
                owner: &owner,
                pool: &pool_info,
                pool_signer_seeds: &seeds,
                decimals: ctx.accounts.mint.decimals,
                admitted,
                extra: ctx.remaining_accounts,
            },
            amount,
        )?;
    }
    ctx.accounts.vault.reload()?;
    let after = ctx.accounts.vault.amount;

    let credited = after.checked_sub(before).ok_or(AmmError::VaultConservationViolated)?;

    let result = {
        let mut p = ctx.accounts.pool.load_mut()?;
        let (claims, owed) = if currency == 0 {
            p.hook_claims0 = p.hook_claims0.checked_add(credited).ok_or(AmmError::MathOverflow)?;
            (p.hook_claims0, p.protocol_fees_owed0)
        } else {
            p.hook_claims1 = p.hook_claims1.checked_add(credited).ok_or(AmmError::MathOverflow)?;
            (p.hook_claims1, p.protocol_fees_owed1)
        };



        require!((claims as u128) + (owed as u128) <= after as u128, AmmError::VaultShortfall);
        single_currency_result(&p, currency, credited as i128)
    };

    emit_cpi!(HookDeposited { pool: ctx.accounts.pool.key(), currency, credited });

    Ok(result.into())
}
