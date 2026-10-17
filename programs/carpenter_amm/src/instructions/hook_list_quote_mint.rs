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

use carpenter_types::layout::{QUOTE_STATE_HOOK_LISTED, QUOTE_STATE_LISTED, QUOTE_STATE_RETIRED};

use crate::amm_core::mints::{authority_digest, check_admitted, check_mint_supported, implied_quote, read_quote_mint, require_derived, MintRole};
use crate::amm_core::vaults::create_pda;
use crate::constants::{AMM_CONFIG_SEED, QUOTE_MINT_SEED};
use crate::errors::AmmError;
use crate::events::QuoteMintHookListed;
use crate::state::{AmmConfig, ApprovedHook, QuoteMint};

#[event_cpi]
#[derive(Accounts)]
pub struct HookListQuoteMint<'info> {
    #[account(constraint = hook_authority.key() == approved_hook.load()?.hook_authority @ AmmError::NotHookAuthority)]
    pub hook_authority: Signer<'info>,
    #[account(seeds = [AMM_CONFIG_SEED], bump)]
    pub config: AccountLoader<'info, AmmConfig>,
    pub approved_hook: AccountLoader<'info, ApprovedHook>,
    pub mint: Box<InterfaceAccount<'info, Mint>>,
///
    #[account(mut, seeds = [QUOTE_MINT_SEED, mint.key().as_ref()], bump)]
    pub quote_mint: UncheckedAccount<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<HookListQuoteMint>) -> Result<()> {
    let mint_key = ctx.accounts.mint.key();
    let mint_info = ctx.accounts.mint.to_account_info();
    let entry = ctx.accounts.quote_mint.to_account_info();
    let hook_program = {
        let a = ctx.accounts.approved_hook.load()?;
        require!(a.allow_hook_quotes == 1, AmmError::HookQuotesNotAllowed);
        a.hook_program
    };
    require!(!implied_quote(&mint_key), AmmError::QuoteMintImplied);

    if let Some(q) = read_quote_mint(&entry, &mint_key)? {
        return match q.state {
            QUOTE_STATE_LISTED => Ok(()),
            QUOTE_STATE_HOOK_LISTED if q.hook_program == hook_program => Ok(()),
            QUOTE_STATE_RETIRED => err!(AmmError::QuoteMintRetired),
            _ => err!(AmmError::QuoteMintAlreadyListed),
        };
    }

    let admitted = ctx.accounts.config.load()?.quote_caps_admitted;
    let caps = check_mint_supported(&mint_info, MintRole::ListedQuote)?;
    check_admitted(&mint_info, caps, admitted)?;
    let digest = authority_digest(&mint_info)?;
    let decimals = ctx.accounts.mint.decimals;

    let bump = require_derived(&entry, QUOTE_MINT_SEED, &mint_key)?;
    create_pda(&ctx.accounts.payer.to_account_info(), &entry, &crate::ID, QuoteMint::LEN, &[QUOTE_MINT_SEED, mint_key.as_ref(), &[bump]])?;
    {
        let mut data = entry.try_borrow_mut_data()?;
        data[..8].copy_from_slice(QuoteMint::DISCRIMINATOR);
        let q: &mut QuoteMint = bytemuck::from_bytes_mut(&mut data[8..QuoteMint::LEN]);
        q.mint = mint_key;
        q.authority_digest = digest;
        q.listed_slot = Clock::get()?.slot;
        q.retired_slot = 0;
        q.state = QUOTE_STATE_HOOK_LISTED;
        q.bump = bump;
        q.decimals = decimals;
        q.caps = caps;
        q.hook_program = hook_program;
    }
    emit_cpi!(QuoteMintHookListed { mint: mint_key, hook_program, caps, decimals });
    Ok(())
}
