//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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
use anchor_spl::associated_token::{self, get_associated_token_address_with_program_id, AssociatedToken};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::amm_core::hook_dispatch::PoolSeeds;
use crate::amm_core::settle::check_vault_conservation;
use crate::amm_core::vaults::{admitted_hook, transfer_checked_out};
use crate::constants::PROTOCOL_TREASURY_SEED;
use crate::errors::AmmError;
use crate::events::ProtocolFeesSwept;
use crate::instructions::hook_deposit::bind_hook_currency;
use crate::state::{Pool, ProtocolTreasury};

#[event_cpi]
#[derive(Accounts)]
pub struct SweepProtocolFees<'info> {
///
///
    #[account(mut)]
    pub sweeper: Signer<'info>,
    #[account(mut)]
    pub pool: AccountLoader<'info, Pool>,
///
    #[account(mut)]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>,
///
    pub mint: Box<InterfaceAccount<'info, Mint>>,
///
    pub token_program: Interface<'info, TokenInterface>,
///
///
    #[account(seeds = [PROTOCOL_TREASURY_SEED], bump)]
    pub protocol_treasury: AccountLoader<'info, ProtocolTreasury>,
///
///
///
///
    #[account(mut)]
    pub destination: UncheckedAccount<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

///
///
pub fn treasury_destination(treasury: &Pubkey, mint: &Pubkey, token_program: &Pubkey) -> Pubkey {
    get_associated_token_address_with_program_id(treasury, mint, token_program)
}

pub fn handler<'info>(ctx: Context<'info, SweepProtocolFees<'info>>, currency: u8) -> Result<()> {
    let treasury = ctx.accounts.protocol_treasury.key();
    let mint_key = ctx.accounts.mint.key();
    let token_program_key = ctx.accounts.token_program.key();
    let derived = treasury_destination(&treasury, &mint_key, &token_program_key);
    require_keys_eq!(ctx.accounts.destination.key(), derived, AmmError::WrongDestination);

    let (amount, claims, seeds_buf, admitted) = {
        let mut p = ctx.accounts.pool.load_mut()?;
        bind_hook_currency(&p, currency, &ctx.accounts.vault.key(), &mint_key, &token_program_key)?;
        let (owed, claims) = if currency == 0 {
            (p.protocol_fees_owed0, p.hook_claims0)
        } else {
            (p.protocol_fees_owed1, p.hook_claims1)
        };

        if owed == 0 {
            return Ok(());
        }


        if currency == 0 {
            p.protocol_fees_owed0 = 0;
        } else {
            p.protocol_fees_owed1 = 0;
        }

        (owed, claims, PoolSeeds::of(&p), admitted_hook(&p, currency))
    };
    let seeds = seeds_buf.seeds();

    let a = &ctx.accounts;



    if a.destination.data_is_empty() {
        associated_token::create_idempotent(CpiContext::new(
            a.associated_token_program.key(),
            associated_token::Create {
                payer: a.sweeper.to_account_info(),
                associated_token: a.destination.to_account_info(),
                authority: a.protocol_treasury.to_account_info(),
                mint: a.mint.to_account_info(),
                system_program: a.system_program.to_account_info(),
                token_program: a.token_program.to_account_info(),
            },
        ))?;
    }


    {
        let info = a.destination.to_account_info();
        require_keys_eq!(*info.owner, token_program_key, AmmError::WrongDestination);
        let dest = TokenAccount::try_deserialize(&mut &info.try_borrow_data()?[..])?;
        require_keys_eq!(dest.owner, treasury, AmmError::WrongDestination);
        require_keys_eq!(dest.mint, mint_key, AmmError::WrongDestination);
    }

    let before = a.vault.amount;
    transfer_checked_out(
        &a.token_program.to_account_info(),
        &a.vault.to_account_info(),
        &a.mint.to_account_info(),
        &a.destination.to_account_info(),
        &a.pool.to_account_info(),
        &seeds,
        amount,
        a.mint.decimals,
        admitted,
        ctx.remaining_accounts,
    )?;
    ctx.accounts.vault.reload()?;
    let after = ctx.accounts.vault.amount;

    check_vault_conservation(before, after, 0, amount, claims, 0)?;

    emit_cpi!(ProtocolFeesSwept {
        pool: ctx.accounts.pool.key(),
        currency,
        mint: mint_key,
        destination: derived,
        amount,
        sweeper: ctx.accounts.sweeper.key(),
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

///
///
    #[test]
    fn the_destination_is_the_treasury_ata_per_mint_and_program() {
        let (treasury, _) = crate::instructions::initialize_protocol_treasury::treasury_address();
        let (m0, m1) = (Pubkey::new_unique(), Pubkey::new_unique());
        let spl = crate::amm_core::mints::SPL_TOKEN_ID;
        let t22 = crate::amm_core::mints::TOKEN_2022_ID;
        let d = treasury_destination(&treasury, &m0, &spl);
        assert_eq!(d, get_associated_token_address_with_program_id(&treasury, &m0, &spl));
        assert_ne!(d, treasury_destination(&treasury, &m1, &spl));
        assert_ne!(d, treasury_destination(&treasury, &m0, &t22));
        assert_ne!(d, treasury_destination(&Pubkey::new_unique(), &m0, &spl), "never another wallet's");
    }
}
