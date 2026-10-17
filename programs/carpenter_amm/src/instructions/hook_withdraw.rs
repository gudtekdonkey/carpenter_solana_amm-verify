//!
//!
//!
//!
//!
//!

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::amm_core::hook_dispatch::PoolSeeds;
use crate::amm_core::vaults::{admitted_hook, transfer_checked_out};
use crate::errors::AmmError;
use crate::events::HookWithdrew;
use crate::instructions::hook_deposit::{bind_hook_currency, single_currency_result};
use crate::state::Pool;

#[event_cpi]
#[derive(Accounts)]
pub struct HookWithdraw<'info> {
    #[account(constraint = hook_authority.key() == pool.load()?.hook_authority @ AmmError::NotHookAuthority)]
    pub hook_authority: Signer<'info>,
    #[account(mut)]
    pub pool: AccountLoader<'info, Pool>,
    #[account(mut)]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub destination: Box<InterfaceAccount<'info, TokenAccount>>,
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler<'info>(
    ctx: Context<'info, HookWithdraw<'info>>,
    currency: u8,
    amount: u64,
) -> Result<crate::state::HookOpResultV1> {
    let (seeds_buf, admitted) = {
        let mut p = ctx.accounts.pool.load_mut()?;
        require_keys_neq!(p.hook, Pubkey::default(), AmmError::NoHook);
        bind_hook_currency(
            &p,
            currency,
            &ctx.accounts.vault.key(),
            &ctx.accounts.mint.key(),
            &ctx.accounts.token_program.key(),
        )?;

        if currency == 0 {
            p.hook_claims0 = p.hook_claims0.checked_sub(amount).ok_or(AmmError::HookClaimsInsufficient)?;
        } else {
            p.hook_claims1 = p.hook_claims1.checked_sub(amount).ok_or(AmmError::HookClaimsInsufficient)?;
        }

        (PoolSeeds::of(&p), admitted_hook(&p, currency))
    };
    let seeds = seeds_buf.seeds();

    let before = ctx.accounts.vault.amount;
    transfer_checked_out(
        &ctx.accounts.token_program.to_account_info(),
        &ctx.accounts.vault.to_account_info(),
        &ctx.accounts.mint.to_account_info(),
        &ctx.accounts.destination.to_account_info(),
        &ctx.accounts.pool.to_account_info(),
        &seeds,
        amount,
        ctx.accounts.mint.decimals,
        admitted,
        ctx.remaining_accounts,
    )?;
    ctx.accounts.vault.reload()?;
    let after = ctx.accounts.vault.amount;

    require!(before.checked_sub(amount) == Some(after), AmmError::VaultConservationViolated);

    let result = {
        let p = ctx.accounts.pool.load()?;
        let (claims, owed) = if currency == 0 {
            (p.hook_claims0, p.protocol_fees_owed0)
        } else {
            (p.hook_claims1, p.protocol_fees_owed1)
        };

        require!((claims as u128) + (owed as u128) <= after as u128, AmmError::VaultShortfall);
        single_currency_result(&p, currency, -(amount as i128))
    };

    emit_cpi!(HookWithdrew {
        pool: ctx.accounts.pool.key(),
        currency,
        amount,
        destination: ctx.accounts.destination.key(),
    });
    Ok(result.into())
}
