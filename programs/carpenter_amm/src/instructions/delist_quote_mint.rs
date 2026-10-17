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

use crate::amm_core::mints::{read_pool_count, read_quote_mint, require_derived};
use crate::constants::{AMM_CONFIG_SEED, MINT_POOLS_SEED, QUOTE_MINT_SEED};
use crate::errors::AmmError;
use crate::events::QuoteMintDelisted;
use crate::state::AmmConfig;

#[event_cpi]
#[derive(Accounts)]
pub struct DelistQuoteMint<'info> {
    #[account(constraint = authority.key() == config.load()?.authority @ AmmError::Unauthorized)]
    pub authority: Signer<'info>,
    #[account(seeds = [AMM_CONFIG_SEED], bump)]
    pub config: AccountLoader<'info, AmmConfig>,
///
    pub mint: UncheckedAccount<'info>,
///
    #[account(seeds = [MINT_POOLS_SEED, mint.key().as_ref()], bump)]
    pub mint_pools: UncheckedAccount<'info>,
///
///
///
///
    #[account(mut, seeds = [QUOTE_MINT_SEED, mint.key().as_ref()], bump)]
    pub quote_mint: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub rent_destination: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<DelistQuoteMint>) -> Result<()> {
    let mint_key = ctx.accounts.mint.key();
    let entry = ctx.accounts.quote_mint.to_account_info();
    require_derived(&entry, QUOTE_MINT_SEED, &mint_key)?;

    let q = read_quote_mint(&entry, &mint_key)?.ok_or(AmmError::QuoteMintNotListed)?;
    require_keys_eq!(q.mint, mint_key, AmmError::SeedsMismatch);
    let (state, listed_slot) = (q.state, q.listed_slot);




    let pools = read_pool_count(&ctx.accounts.mint_pools.to_account_info(), &mint_key)?.unwrap_or(0);
    require!(pools == 0, AmmError::QuoteMintPooled);



    {
        let mut data = entry.try_borrow_mut_data()?;
        data.fill(0);
    }
    let lamports = entry.lamports();
    **entry.try_borrow_mut_lamports()? = 0;
    **ctx.accounts.rent_destination.try_borrow_mut_lamports()? = ctx
        .accounts
        .rent_destination
        .lamports()
        .checked_add(lamports)
        .ok_or(AmmError::MathOverflow)?;

    emit_cpi!(QuoteMintDelisted { mint: mint_key, was_retired: state == carpenter_types::layout::QUOTE_STATE_RETIRED, listed_slot, rent_refunded: lamports });
    Ok(())
}
