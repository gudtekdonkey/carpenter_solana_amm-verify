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
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::{CARPENTER_AMM_PROGRAM, EVENT_AUTHORITY_SEED};
use crate::errors::HookError;
use crate::events::InventoryDonated;
use crate::mech::amm_cpi;
use crate::mech::ratchet;
use crate::state::FloorPool;

#[event_cpi]
#[derive(Accounts)]
pub struct DonateInventory<'info> {
    pub signer: Signer<'info>,
///
///
    #[account(
        mut,
        constraint = signer.key() == floor_pool.load()?.pool_initializer @ HookError::NotInitializer,
    )]
    pub floor_pool: AccountLoader<'info, FloorPool>,
///
    #[account(mut, address = floor_pool.load()?.amm_pool @ HookError::WrongPool)]
    pub pool: UncheckedAccount<'info>,
    #[account(mut)]
    pub source: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub vault1: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(address = floor_pool.load()?.base_mint @ HookError::WrongPool)]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    pub token_2022: Interface<'info, TokenInterface>,
///
    #[account(address = CARPENTER_AMM_PROGRAM)]
    pub carpenter_amm: UncheckedAccount<'info>,
///
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = CARPENTER_AMM_PROGRAM)]
    pub amm_event_authority: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<DonateInventory>, tokens: u64) -> Result<()> {
    let mut s: FloorPool = *ctx.accounts.floor_pool.load()?;
    require!(s.pool_bound == 1 && s.ladder_seeded == 1, HookError::NotBound);
    require!(tokens > 0, HookError::BadRange);

    let a = &ctx.accounts;
    let p = amm_cpi::read_pool(&a.pool.to_account_info())?;
    amm_cpi::require_pool_side(&p, 1, &a.vault1.key(), &a.mint.key(), &a.token_2022.key())?;

    let r = amm_cpi::deposit(
        &a.carpenter_amm.to_account_info(),
        &a.amm_event_authority.to_account_info(),
        &a.pool.to_account_info(),
        &a.signer.to_account_info(),
        &a.source.to_account_info(),
        &a.vault1.to_account_info(),
        &a.mint.to_account_info(),
        &a.token_2022.to_account_info(),
        1,
        tokens,
    )?;
    let credited = amm_cpi::pos_to_u64(r.delta.amount1)?;
    require!(credited > 0, HookError::BadRange);




    require!(s.stair_steps != 0, HookError::InvariantViolated);
    crate::mech::positions::to_wall(&mut s, credited)?;
    s.needs_ratchet = 1;
    require!(
        ratchet::claims_cover_liabilities(&s, r.hook_claims0, r.hook_claims1),
        HookError::InvariantViolated
    );
    *ctx.accounts.floor_pool.load_mut()? = s;

    emit_cpi!(InventoryDonated { pool: a.pool.key(), from: a.signer.key(), amount: credited });
    Ok(())
}
