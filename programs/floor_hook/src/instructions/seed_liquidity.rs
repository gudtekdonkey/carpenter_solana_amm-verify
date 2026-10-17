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
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::{EVENT_AUTHORITY_SEED, CARPENTER_AMM_PROGRAM, HOOK_AUTHORITY_SEED};
use crate::errors::HookError;
use crate::mech::amm_cpi::{self, AmmCpi, PoolAccounts};
use crate::mech::positions::{Market, Mech, MechEvent};
use crate::mech::ratchet;
use crate::mech::seed;
use crate::state::FloorPool;

#[event_cpi]
#[derive(Accounts)]
pub struct SeedLiquidity<'info> {
    pub signer: Signer<'info>,
    #[account(mut)]
    pub floor_pool: AccountLoader<'info, FloorPool>,
///
    #[account(mut, address = floor_pool.load()?.amm_pool @ HookError::WrongPool)]
    pub pool: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub bitmap: UncheckedAccount<'info>,
///
///
    #[account(mut)]
    pub position_seed: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub tick_array_lower: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub tick_array_upper: UncheckedAccount<'info>,
    #[account(mut, constraint = source.owner == signer.key() @ HookError::WrongDestination)]
    pub source: Box<InterfaceAccount<'info, TokenAccount>>,
///
///
///
///
    #[account(mut)]
    pub vault0: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub vault1: Box<InterfaceAccount<'info, TokenAccount>>,
///
///
///
///
///
///
    #[account(address = floor_pool.load()?.quote_mint @ HookError::WrongPool)]
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(address = floor_pool.load()?.base_mint @ HookError::WrongPool)]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    pub spl_token: Interface<'info, TokenInterface>,
    pub token_2022: Interface<'info, TokenInterface>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
///
    #[account(address = CARPENTER_AMM_PROGRAM)]
    pub carpenter_amm: UncheckedAccount<'info>,
///
    #[account(seeds = [HOOK_AUTHORITY_SEED], bump)]
    pub hook_authority: UncheckedAccount<'info>,
///
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = CARPENTER_AMM_PROGRAM)]
    pub amm_event_authority: UncheckedAccount<'info>,
}

pub fn handler<'info>(
    ctx: Context<'info, SeedLiquidity<'info>>,
    lower: i32,
    upper: i32,
    token_amount: u64,
    rent_owner: Pubkey,
) -> Result<()> {
    let mut s: FloorPool = *ctx.accounts.floor_pool.load()?;
    require!(s.pool_bound == 1, HookError::NotBound);

    s.require_quote_live()?;
    require!(token_amount > 0, HookError::BadRange);
    seed::fix_seed_range(&mut s, lower, upper, &ctx.accounts.signer.key())?;

    let a = &ctx.accounts;


    let (mut market, spacing) = bound_market(a)?;

    let cpi = AmmCpi {
        carpenter_amm: a.carpenter_amm.to_account_info(),
        amm_event_authority: a.amm_event_authority.to_account_info(),
        hook_authority: a.hook_authority.to_account_info(),
        hook_authority_bump: ctx.bumps.hook_authority,
        hook_program: a.program.to_account_info(),
        floor_pool: a.floor_pool.to_account_info(),
        pool: a.pool.to_account_info(),
    };


    let dep = amm_cpi::deposit(
        &cpi.carpenter_amm,
        &cpi.amm_event_authority,
        &cpi.pool,
        &a.signer.to_account_info(),
        &a.source.to_account_info(),
        &a.vault1.to_account_info(),
        &a.mint.to_account_info(),
        &a.token_2022.to_account_info(),
        1,
        token_amount,
    )?;
    let credited = amm_cpi::pos_to_u64(dep.delta.amount1)?;
    require!(credited > 0, HookError::BadRange);

    let mut tick_arrays = vec![a.tick_array_lower.to_account_info(), a.tick_array_upper.to_account_info()];
    tick_arrays.extend(ctx.remaining_accounts.iter().cloned());
    market.update(&dep);
    let mut m = Mech {
        cpi,
        accts: PoolAccounts {
            payer: Some(a.payer.to_account_info()),
            rent_owner,
            bitmap: a.bitmap.to_account_info(),
            vault0: a.vault0.to_account_info(),
            vault1: a.vault1.to_account_info(),
            mint0: a.quote_mint.to_account_info(),
            mint1: a.mint.to_account_info(),
            token_program0: a.spl_token.to_account_info(),
            token_program1: a.token_2022.to_account_info(),
            system_program: a.system_program.to_account_info(),
            tick_arrays,
        },
        position_seed: Some(a.position_seed.to_account_info()),
        position_bid: None,
        bounty_destination: None,
        spacing,
        market,
        events: Vec::new(),
        modifies: 0,
    };

    let used = seed::open_seed(&mut m, &mut s, credited)?;
    m.flush(&s)?;

    let refund = credited - used;
    let (claims0, claims1) = if refund > 0 {
        let w = amm_cpi::withdraw(
            &m.cpi,
            &a.vault1.to_account_info(),
            &a.source.to_account_info(),
            &a.mint.to_account_info(),
            &a.token_2022.to_account_info(),
            1,
            refund,
        )?;
        (w.hook_claims0, w.hook_claims1)
    } else {
        (m.market.hook_claims0, m.market.hook_claims1)
    };



    require!(
        ratchet::claims_cover_liabilities(&s, claims0, claims1),
        HookError::InvariantViolated
    );


    for e in m.events {
        match e {
            MechEvent::PlatformCut(ev) => emit_cpi!(ev),
            MechEvent::PolRaised(ev) => emit_cpi!(ev),
            MechEvent::StairRatcheted(ev) => emit_cpi!(ev),
            MechEvent::VaultTokensFed(ev) => emit_cpi!(ev),
            MechEvent::StairOwnerBooked(ev) => emit_cpi!(ev),
            MechEvent::StairOwnersPlaced(ev) => emit_cpi!(ev),
        }
    }
    Ok(())
}

///
///
///
///
///
///
///
///
///
///
#[inline(never)]
fn bound_market(a: &SeedLiquidity<'_>) -> Result<(Market, u16)> {
    let p = amm_cpi::read_pool(&a.pool.to_account_info())?;
    amm_cpi::require_pool_side(&p, 0, &a.vault0.key(), &a.quote_mint.key(), &a.spl_token.key())?;
    amm_cpi::require_pool_side(&p, 1, &a.vault1.key(), &a.mint.key(), &a.token_2022.key())?;
    Ok((Market::from_pool(&p), p.tick_spacing))
}
