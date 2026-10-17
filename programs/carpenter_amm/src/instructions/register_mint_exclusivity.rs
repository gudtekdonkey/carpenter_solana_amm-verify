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

use crate::amm_core::mints::{implied_quote, require_unpooled};
use crate::constants::{AMM_CONFIG_SEED, MINT_EXCLUSIVITY_SEED, MINT_POOLS_SEED};
use crate::errors::AmmError;
use crate::events::MintExclusivityRegistered;
use crate::state::{AmmConfig, ApprovedHook, MintExclusivity};

#[event_cpi]
#[derive(Accounts)]
pub struct RegisterMintExclusivity<'info> {
    #[account(constraint = hook_authority.key() == approved_hook.load()?.hook_authority @ AmmError::NotHookAuthority)]
    pub hook_authority: Signer<'info>,
    #[account(seeds = [AMM_CONFIG_SEED], bump)]
    pub config: AccountLoader<'info, AmmConfig>,
    pub approved_hook: AccountLoader<'info, ApprovedHook>,
    pub mint: Box<InterfaceAccount<'info, Mint>>,
///
    #[account(seeds = [MINT_POOLS_SEED, mint.key().as_ref()], bump)]
    pub mint_pools: UncheckedAccount<'info>,
    #[account(
        init,
        payer = payer,
        space = MintExclusivity::LEN,
        seeds = [MINT_EXCLUSIVITY_SEED, mint.key().as_ref()],
        bump,
    )]
    pub mint_exclusivity: AccountLoader<'info, MintExclusivity>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

///
///
pub fn handler(ctx: Context<RegisterMintExclusivity>) -> Result<()> {
    let mint = ctx.accounts.mint.key();
    let hook_program = {
        let a = ctx.accounts.approved_hook.load()?;
        require!(a.allow_exclusivity == 1, AmmError::ExclusivityNotAllowed);
        a.hook_program
    };


    let protocol_fee = ctx.accounts.config.load()?.current_protocol_fee();
    require!(!implied_quote(&mint), AmmError::ExclusivityQuoteMint);
    require_unpooled(&ctx.accounts.mint_pools.to_account_info(), &mint)?;
    {
        let mut e = ctx.accounts.mint_exclusivity.load_init()?;
        e.mint = mint;
        e.hook_program = hook_program;
        e.bump = ctx.bumps.mint_exclusivity;
        e.has_protocol_fee = 1;
        e.protocol_fee = protocol_fee;
    }
    emit_cpi!(MintExclusivityRegistered { mint, hook_program, protocol_fee });
    Ok(())
}
