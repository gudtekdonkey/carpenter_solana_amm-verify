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

use crate::amm_core::mints::require_derived;
use crate::constants::{AMM_CONFIG_SEED, QUOTE_MINT_SEED};
use crate::errors::AmmError;
use crate::events::QuoteMintRetired;
use crate::state::{AmmConfig, QuoteMint};

#[event_cpi]
#[derive(Accounts)]
pub struct RetireQuoteMint<'info> {
    #[account(constraint = authority.key() == config.load()?.authority @ AmmError::Unauthorized)]
    pub authority: Signer<'info>,
    #[account(seeds = [AMM_CONFIG_SEED], bump)]
    pub config: AccountLoader<'info, AmmConfig>,
///
    pub mint: UncheckedAccount<'info>,
    #[account(mut, seeds = [QUOTE_MINT_SEED, mint.key().as_ref()], bump)]
    pub quote_mint: AccountLoader<'info, QuoteMint>,
}

pub fn handler(ctx: Context<RetireQuoteMint>) -> Result<()> {
    let mint_key = ctx.accounts.mint.key();
    require_derived(&ctx.accounts.quote_mint.to_account_info(), QUOTE_MINT_SEED, &mint_key)?;
    let slot = Clock::get()?.slot;
    {
        let mut q = ctx.accounts.quote_mint.load_mut()?;

        require!(
            q.state == carpenter_types::layout::QUOTE_STATE_LISTED || q.state == carpenter_types::layout::QUOTE_STATE_HOOK_LISTED,
            AmmError::QuoteMintNotListed
        );
        require_keys_eq!(q.mint, mint_key, AmmError::SeedsMismatch);
        q.state = carpenter_types::layout::QUOTE_STATE_RETIRED;
        q.retired_slot = slot;
    }
    emit_cpi!(QuoteMintRetired { mint: mint_key, retired_slot: slot });
    Ok(())
}
