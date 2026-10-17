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

use crate::amm_core::mints::{
    authority_digest, check_admitted, check_mint_supported, implied_quote, is_present, require_derived, MintRole,
};
use crate::amm_core::vaults::create_pda;
use crate::constants::{AMM_CONFIG_SEED, MINT_EXCLUSIVITY_SEED, QUOTE_MINT_SEED};
use crate::errors::AmmError;
use crate::events::QuoteMintListed;
use crate::state::{AmmConfig, MintExclusivity, QuoteMint};

#[event_cpi]
#[derive(Accounts)]
pub struct ListQuoteMint<'info> {
    #[account(constraint = authority.key() == config.load()?.authority @ AmmError::Unauthorized)]
    pub authority: Signer<'info>,
    #[account(seeds = [AMM_CONFIG_SEED], bump)]
    pub config: AccountLoader<'info, AmmConfig>,
    pub mint: Box<InterfaceAccount<'info, Mint>>,
///
    #[account(seeds = [MINT_EXCLUSIVITY_SEED, mint.key().as_ref()], bump)]
    pub mint_exclusivity: UncheckedAccount<'info>,
///
    #[account(mut, seeds = [QUOTE_MINT_SEED, mint.key().as_ref()], bump)]
    pub quote_mint: UncheckedAccount<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ListQuoteMint>) -> Result<()> {
    let mint_key = ctx.accounts.mint.key();
    let mint_info = ctx.accounts.mint.to_account_info();
    let entry = ctx.accounts.quote_mint.to_account_info();

    let admitted = {
        let c = ctx.accounts.config.load()?;
        require!(!implied_quote(&mint_key), AmmError::QuoteMintImplied);
        c.quote_caps_admitted
    };
    let excl = ctx.accounts.mint_exclusivity.to_account_info();
    require_derived(&excl, MINT_EXCLUSIVITY_SEED, &mint_key)?;
    require!(
        !is_present(&excl, MintExclusivity::LEN, MintExclusivity::DISCRIMINATOR)?,
        AmmError::MintExclusive
    );

    let bump = require_derived(&entry, QUOTE_MINT_SEED, &mint_key)?;
    require!(
        !is_present(&entry, QuoteMint::LEN, QuoteMint::DISCRIMINATOR)?,
        AmmError::QuoteMintAlreadyListed
    );





    let caps = check_mint_supported(&mint_info, MintRole::ListedQuote)?;
    check_admitted(&mint_info, caps, admitted)?;
    let digest = authority_digest(&mint_info)?;
    let decimals = ctx.accounts.mint.decimals;

    create_pda(
        &ctx.accounts.payer.to_account_info(),
        &entry,
        &crate::ID,
        QuoteMint::LEN,
        &[QUOTE_MINT_SEED, mint_key.as_ref(), &[bump]],
    )?;
    {
        let mut data = entry.try_borrow_mut_data()?;
        data[..8].copy_from_slice(QuoteMint::DISCRIMINATOR);
        let q: &mut QuoteMint = bytemuck::from_bytes_mut(&mut data[8..QuoteMint::LEN]);
        q.mint = mint_key;
        q.authority_digest = digest;
        q.listed_slot = Clock::get()?.slot;
        q.retired_slot = 0;
        q.state = carpenter_types::layout::QUOTE_STATE_LISTED;
        q.bump = bump;
        q.decimals = decimals;
        q.caps = caps;
    }

    emit_cpi!(QuoteMintListed { mint: mint_key, caps, decimals, authority_digest: digest });
    Ok(())
}
