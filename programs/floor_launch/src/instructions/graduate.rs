//!
//!
//!
//!
//!
//!

use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};

use floor_types::constants::LAUNCH_STATE_BONDING;

use crate::constants::{LAUNCH_SEED, SALE_VAULT_SEED};
use crate::errors::LaunchError;
use crate::graduate::{self, Graduation};
use crate::lock::LOCK_AUTHORITY_SEED;
use crate::state::Launch;

#[event_cpi]
#[derive(Accounts)]
pub struct Graduate<'info> {
///
    #[account(mut)]
    pub anyone: Signer<'info>,
    #[account(mut, seeds = [LAUNCH_SEED, mint.key().as_ref()], bump)]
    pub launch: AccountLoader<'info, Launch>,
///
    #[account(mut, seeds = [SALE_VAULT_SEED, launch.key().as_ref()], bump)]
    pub sale_vault: UncheckedAccount<'info>,
    #[account(mut, address = launch.load()?.token_vault @ LaunchError::BadConfig)]
    pub token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, address = launch.load()?.mint @ LaunchError::BadConfig)]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
///
    #[account(seeds = [LOCK_AUTHORITY_SEED], bump)]
    pub lock_authority: UncheckedAccount<'info>,
    pub token_2022: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

///
///
///
///
pub fn handler<'info>(ctx: Context<'info, Graduate<'info>>) -> Result<()> {
    {
        let l = ctx.accounts.launch.load()?;
        require!(l.state == LAUNCH_STATE_BONDING && l.sold == l.sale_tokens, LaunchError::CannotGraduate);
    }



    let a = &ctx.accounts;
    let payer = a.anyone.to_account_info();
    let launch = a.launch.to_account_info();
    let sale_vault = a.sale_vault.to_account_info();
    let token_vault = a.token_vault.to_account_info();
    let mint = a.mint.to_account_info();
    let lock_authority = a.lock_authority.to_account_info();
    let token_2022 = a.token_2022.to_account_info();
    let system_program = a.system_program.to_account_info();
    let event_authority = a.event_authority.to_account_info();
    let program = a.program.to_account_info();
    let g = Graduation {
        payer: &payer,
        launch: &launch,
        sale_vault: &sale_vault,
        token_vault: &token_vault,
        mint: &mint,
        lock_authority: &lock_authority,
        lock_bump: ctx.bumps.lock_authority,
        token_2022: &token_2022,
        system_program: &system_program,
        event_authority: &event_authority,
        program: &program,
        graduation_accounts: ctx.remaining_accounts,
    };
    graduate::run(&g)?;
    Ok(())
}
