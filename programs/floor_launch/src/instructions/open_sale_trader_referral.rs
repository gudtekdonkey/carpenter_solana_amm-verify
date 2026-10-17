//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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
use anchor_spl::token_interface::{self, Mint, TokenInterface};
use floor_types::keys;

use crate::constants::{LAUNCH_AUTHORITY_SEED, LAUNCH_SEED, TRADER_REF_SALE_SEED};
use crate::errors::LaunchError;
use crate::sale::create_pda;
use crate::state::Launch;

#[derive(Accounts)]
pub struct OpenSaleTraderReferral<'info> {
///
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(seeds = [LAUNCH_SEED, launch.load()?.mint.as_ref()], bump = launch.load()?.bump)]
    pub launch: AccountLoader<'info, Launch>,
///
    #[account(mut, seeds = [TRADER_REF_SALE_SEED, launch.key().as_ref()], bump = launch.load()?.trader_ref_sale_bump)]
    pub trader_ref_sale: UncheckedAccount<'info>,
///
    #[account(seeds = [LAUNCH_AUTHORITY_SEED], bump)]
    pub launch_authority: UncheckedAccount<'info>,
    #[account(address = launch.load()?.quote_mint() @ LaunchError::TraderRefMismatch)]
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
    pub quote_token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<OpenSaleTraderReferral>) -> Result<()> {
    let a = &ctx.accounts;
    let sale = a.trader_ref_sale.to_account_info();
    let owner = *sale.owner;




    if owner == Pubkey::new_from_array(keys::SPL_TOKEN_PROGRAM_ID) || owner == Pubkey::new_from_array(keys::TOKEN_2022_PROGRAM_ID) {
        let d = sale.try_borrow_data()?;
        let ok = owner == a.quote_token_program.key()
            && d.len() >= 64
            && d[0..32] == a.quote_mint.key().to_bytes()
            && d[32..64] == a.launch_authority.key().to_bytes();
        require!(ok, LaunchError::SaleReferralAlreadyOpen);
        return Ok(());
    }
    require_keys_eq!(*a.quote_mint.to_account_info().owner, a.quote_token_program.key(), LaunchError::TraderRefMismatch);
    let quote_mint = a.quote_mint.to_account_info();
    let len = crate::graduation_rent::quote_vault_len(&quote_mint)?;
    let launch_key = a.launch.key();
    let bump = [a.launch.load()?.trader_ref_sale_bump];
    create_pda(
        &a.payer.to_account_info(),
        &sale,
        len,
        &a.quote_token_program.key(),
        &[TRADER_REF_SALE_SEED, launch_key.as_ref(), &bump],
    )?;
    token_interface::initialize_account3(CpiContext::new(
        a.quote_token_program.key(),
        token_interface::InitializeAccount3 { account: sale, mint: quote_mint, authority: a.launch_authority.to_account_info() },
    ))
}
