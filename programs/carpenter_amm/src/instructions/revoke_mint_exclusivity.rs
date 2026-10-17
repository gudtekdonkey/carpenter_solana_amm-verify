//!
//!
//!

use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::amm_core::mints::require_unpooled;
use crate::constants::{AMM_CONFIG_SEED, MINT_EXCLUSIVITY_SEED, MINT_POOLS_SEED};
use crate::errors::AmmError;
use crate::events::MintExclusivityRevoked;
use crate::state::{AmmConfig, MintExclusivity};

#[event_cpi]
#[derive(Accounts)]
pub struct RevokeMintExclusivity<'info> {
///
    #[account(mut, constraint = authority.key() == config.load()?.authority @ AmmError::Unauthorized)]
    pub authority: Signer<'info>,
    #[account(seeds = [AMM_CONFIG_SEED], bump)]
    pub config: AccountLoader<'info, AmmConfig>,
    pub mint: Box<InterfaceAccount<'info, Mint>>,
///
    #[account(seeds = [MINT_POOLS_SEED, mint.key().as_ref()], bump)]
    pub mint_pools: UncheckedAccount<'info>,
    #[account(
        mut,
        close = authority,
        seeds = [MINT_EXCLUSIVITY_SEED, mint.key().as_ref()],
        bump,
    )]
    pub mint_exclusivity: AccountLoader<'info, MintExclusivity>,
}

pub fn handler(ctx: Context<RevokeMintExclusivity>) -> Result<()> {
    let mint = ctx.accounts.mint.key();

    require_unpooled(&ctx.accounts.mint_pools.to_account_info(), &mint)?;
    let hook_program = ctx.accounts.mint_exclusivity.load()?.hook_program;

    emit_cpi!(MintExclusivityRevoked { mint, hook_program });
    Ok(())
}
