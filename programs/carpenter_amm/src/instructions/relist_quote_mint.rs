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
use anchor_spl::token_interface::Mint;

use crate::amm_core::mints::{authority_digest, check_admitted, check_mint_supported, require_derived, MintRole};
use crate::constants::{AMM_CONFIG_SEED, QUOTE_MINT_SEED};
use crate::errors::AmmError;
use crate::events::QuoteMintRelisted;
use crate::state::{AmmConfig, QuoteMint};

#[event_cpi]
#[derive(Accounts)]
pub struct RelistQuoteMint<'info> {
    #[account(constraint = authority.key() == config.load()?.authority @ AmmError::Unauthorized)]
    pub authority: Signer<'info>,
    #[account(seeds = [AMM_CONFIG_SEED], bump)]
    pub config: AccountLoader<'info, AmmConfig>,
///
///
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, seeds = [QUOTE_MINT_SEED, mint.key().as_ref()], bump)]
    pub quote_mint: AccountLoader<'info, QuoteMint>,
}

pub fn handler(ctx: Context<RelistQuoteMint>) -> Result<()> {
    let mint_key = ctx.accounts.mint.key();
    let mint_info = ctx.accounts.mint.to_account_info();
    require_derived(&ctx.accounts.quote_mint.to_account_info(), QUOTE_MINT_SEED, &mint_key)?;




    let admitted = ctx.accounts.config.load()?.quote_caps_admitted;
    let caps = check_mint_supported(&mint_info, MintRole::ListedQuote)?;
    check_admitted(&mint_info, caps, admitted)?;
    let digest = authority_digest(&mint_info)?;
    let decimals = ctx.accounts.mint.decimals;

    let retired_slot = {
        let mut q = ctx.accounts.quote_mint.load_mut()?;
        require!(q.state == carpenter_types::layout::QUOTE_STATE_RETIRED, AmmError::QuoteMintNotRetired);
        require_keys_eq!(q.mint, mint_key, AmmError::SeedsMismatch);
        let was = q.retired_slot;


        q.state = if q.hook_program == Pubkey::default() {
            carpenter_types::layout::QUOTE_STATE_LISTED
        } else {
            carpenter_types::layout::QUOTE_STATE_HOOK_LISTED
        };
        q.retired_slot = 0;
        q.caps = caps;
        q.authority_digest = digest;
        q.decimals = decimals;
        was
    };

    emit_cpi!(QuoteMintRelisted { mint: mint_key, caps, decimals, authority_digest: digest, retired_slot });
    Ok(())
}
