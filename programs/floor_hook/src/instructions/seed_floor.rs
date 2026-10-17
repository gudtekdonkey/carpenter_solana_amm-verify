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

use crate::constants::{EVENT_AUTHORITY_SEED, CARPENTER_AMM_PROGRAM};
use crate::errors::HookError;
use crate::mech::amm_cpi;
use crate::mech::positions::MechEvent;
use crate::mech::ratchet;
use crate::mech::seed;
use crate::state::FloorPool;

#[event_cpi]
#[derive(Accounts)]
pub struct SeedFloor<'info> {
    pub donor: Signer<'info>,
    #[account(mut)]
    pub floor_pool: AccountLoader<'info, FloorPool>,
///
    #[account(mut, address = floor_pool.load()?.amm_pool @ HookError::WrongPool)]
    pub pool: UncheckedAccount<'info>,
    #[account(mut)]
    pub source: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub vault0: Box<InterfaceAccount<'info, TokenAccount>>,
///
///
///
///
///
///
    #[account(address = floor_pool.load()?.quote_mint @ HookError::WrongPool)]
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
    pub spl_token: Interface<'info, TokenInterface>,
///
    #[account(address = CARPENTER_AMM_PROGRAM)]
    pub carpenter_amm: UncheckedAccount<'info>,
///
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = CARPENTER_AMM_PROGRAM)]
    pub amm_event_authority: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<SeedFloor>, amount: u64, team: u64) -> Result<()> {
    let mut s: FloorPool = *ctx.accounts.floor_pool.load()?;
    require!(s.pool_bound == 1, HookError::NotBound);

    s.require_quote_live()?;
    require!(amount > 0, HookError::BadRange);
    if team > 0 {
        require_keys_eq!(ctx.accounts.donor.key(), s.pool_initializer, HookError::NotInitializer);
    }

    let a = &ctx.accounts;
    let p = amm_cpi::read_pool(&a.pool.to_account_info())?;
    amm_cpi::require_pool_side(&p, 0, &a.vault0.key(), &a.quote_mint.key(), &a.spl_token.key())?;

    let r = amm_cpi::deposit(
        &a.carpenter_amm.to_account_info(),
        &a.amm_event_authority.to_account_info(),
        &a.pool.to_account_info(),
        &a.donor.to_account_info(),
        &a.source.to_account_info(),
        &a.vault0.to_account_info(),
        &a.quote_mint.to_account_info(),
        &a.spl_token.to_account_info(),
        0,
        amount,
    )?;
    let credited = amm_cpi::pos_to_u64(r.delta.amount0)?;
    require!(team <= credited, HookError::BadRange);

    let mut events: Vec<MechEvent> = Vec::new();
    seed::book_seed_floor(&mut s, credited, team, &mut events)?;



    require!(
        ratchet::claims_cover_liabilities(&s, r.hook_claims0, r.hook_claims1),
        HookError::InvariantViolated
    );
    *ctx.accounts.floor_pool.load_mut()? = s;

    for e in events {
        if let MechEvent::PlatformCut(ev) = e {
            emit_cpi!(ev);
        }
    }
    Ok(())
}
