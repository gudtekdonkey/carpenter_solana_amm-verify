//!
//!
//!
//!
//!
//!
//!
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
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke;
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use anchor_spl::token_interface::{self, Mint, TokenInterface, TransferChecked};
use floor_types::{ids, keys, seeds as sd};

use crate::constants::{LAUNCH_AUTHORITY_SEED, LAUNCH_SEED};
use crate::errors::LaunchError;
use crate::events::SaleTraderReferralSwept;
use crate::graduate::token_amount;
use crate::state::Launch;
use crate::trader_ref;

const fn pk(b: [u8; 32]) -> Pubkey {
    Pubkey::new_from_array(b)
}
///
pub const TRADER_REF_VAULT_AUTHORITY: Pubkey =
    pk(anchor_lang::derive_program_address(&[sd::TRADER_REF_VAULT], &ids::FLOOR_HOOK_ID).0);
const SPL_TOKEN_PROGRAM: Pubkey = pk(keys::SPL_TOKEN_PROGRAM_ID);
const WSOL: Pubkey = pk(keys::WSOL_MINT);
///
const SPL_SYNC_NATIVE_TAG: u8 = 17;

#[event_cpi]
#[derive(Accounts)]
pub struct SweepSaleTraderReferral<'info> {
    #[account(seeds = [LAUNCH_SEED, launch.load()?.mint.as_ref()], bump = launch.load()?.bump)]
    pub launch: AccountLoader<'info, Launch>,
///
    #[account(mut)]
    pub trader_ref_sale: UncheckedAccount<'info>,
///
    #[account(seeds = [LAUNCH_AUTHORITY_SEED], bump)]
    pub launch_authority: UncheckedAccount<'info>,
    #[account(address = launch.load()?.quote_mint() @ LaunchError::TraderRefMismatch)]
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
///
    #[account(mut)]
    pub vault: UncheckedAccount<'info>,
    pub quote_token_program: Interface<'info, TokenInterface>,
}

pub fn handler(ctx: Context<SweepSaleTraderReferral>) -> Result<()> {
    let a = &ctx.accounts;
    let launch_key = a.launch.key();
    let (sale_bump, quote_mint) = {
        let l = a.launch.load()?;
        (l.trader_ref_sale_bump, l.quote_mint())
    };
    require_keys_eq!(a.trader_ref_sale.key(), trader_ref::sale_address(&launch_key, sale_bump)?, LaunchError::TraderRefMismatch);
    let token_program = a.quote_token_program.key();
    require_keys_eq!(*a.quote_mint.to_account_info().owner, token_program, LaunchError::TraderRefMismatch);
    require_keys_eq!(
        a.vault.key(),
        get_associated_token_address_with_program_id(&TRADER_REF_VAULT_AUTHORITY, &quote_mint, &token_program),
        LaunchError::TraderRefMismatch
    );



    let (sale, vault) = (a.trader_ref_sale.to_account_info(), a.vault.to_account_info());
    if trader_ref::unopened(&sale) || trader_ref::unopened(&vault) {
        return Ok(());
    }
    require_keys_eq!(*a.trader_ref_sale.owner, token_program, LaunchError::TraderRefMismatch);
    require_keys_eq!(*a.vault.owner, token_program, LaunchError::TraderRefMismatch);
    if trader_ref::token_account_frozen(&sale) || trader_ref::token_account_frozen(&vault) {
        return Ok(());
    }


    if quote_mint == WSOL && token_program == SPL_TOKEN_PROGRAM {
        invoke(
            &Instruction {
                program_id: SPL_TOKEN_PROGRAM,
                accounts: vec![AccountMeta::new(a.trader_ref_sale.key(), false)],
                data: vec![SPL_SYNC_NATIVE_TAG],
            },
            &[a.trader_ref_sale.to_account_info(), a.quote_token_program.to_account_info()],
        )?;
    }
    let amount = token_amount(&a.trader_ref_sale.to_account_info())?;
    if amount == 0 {
        return Ok(());
    }
    let bump = [ctx.bumps.launch_authority];
    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            token_program,
            TransferChecked {
                from: a.trader_ref_sale.to_account_info(),
                mint: a.quote_mint.to_account_info(),
                to: a.vault.to_account_info(),
                authority: a.launch_authority.to_account_info(),
            },
            &[&[LAUNCH_AUTHORITY_SEED, &bump]],
        ),
        amount,
        a.quote_mint.decimals,
    )?;
    emit_cpi!(SaleTraderReferralSwept { launch: launch_key, quote_mint, amount });
    Ok(())
}
