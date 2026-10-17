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
pub struct SeedInventory<'info> {
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
///
    #[account(mut)]
    pub bitmap: UncheckedAccount<'info>,


///
    #[account(mut)]
    pub tick_array_lower: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub tick_array_upper: UncheckedAccount<'info>,
    #[account(mut)]
    pub source: Box<InterfaceAccount<'info, TokenAccount>>,
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
    ctx: Context<'info, SeedInventory<'info>>,
    tokens: u64,
    open_tick: i32,
    rent_owner: Pubkey,
    owner: Option<floor_types::owners::StairOwnerArg>,
) -> Result<()> {
    let mut s: FloorPool = *ctx.accounts.floor_pool.load()?;
    require!(s.pool_bound == 1, HookError::NotBound);

    s.require_quote_live()?;
    require!(s.ladder_seeded == 0, HookError::LadderAlreadySeeded);
    require!(tokens > 0, HookError::BadRange);
    s.ladder_seeded = 1;

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
        tokens,
    )?;
    let credited = amm_cpi::pos_to_u64(dep.delta.amount1)?;
    require!(credited > 0, HookError::BadRange);

    let mut tick_arrays = vec![a.tick_array_lower.to_account_info(), a.tick_array_upper.to_account_info()];
    tick_arrays.extend(ctx.remaining_accounts.iter().cloned());
    if s.stair_steps != 0 {


        ensure_stair(a, ctx.remaining_accounts, s.stair_owner)?;
    }
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
        position_seed: None,
        position_bid: None,
        bounty_destination: None,
        spacing,
        market,
        events: Vec::new(),
        modifies: 0,
    };



    require!((s.stair_owner != 0) == owner.is_some(), HookError::StairOwnerBadConfig);
    seed::seed_inventory(&mut m, &mut s, credited, open_tick, owner.as_ref())?;



    require!(
        ratchet::claims_cover_liabilities(&s, m.market.hook_claims0, m.market.hook_claims1),
        HookError::InvariantViolated
    );
    m.flush(&s)?;

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
#[inline(never)]
fn ensure_stair<'info>(a: &SeedInventory<'info>, remaining: &[AccountInfo<'info>], stair_owner: u8) -> Result<()> {
    let fp = a.floor_pool.key();


    if let Ok((acct, _)) = crate::mech::stair::load(remaining, &fp) {
        if acct.data_len() >= crate::mech::owners::stair_account_len(stair_owner) {
            return Ok(());
        }
    }
    let key = crate::mech::stair::stair_key(&fp).0;
    let target = remaining.iter().find(|x| *x.key == key).ok_or_else(|| error!(HookError::StairStateMissing))?;
    crate::mech::stair::create_stair_state(&a.payer.to_account_info(), target, &a.system_program.to_account_info(), &fp, stair_owner)
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
fn bound_market(a: &SeedInventory<'_>) -> Result<(Market, u16)> {
    let p = amm_cpi::read_pool(&a.pool.to_account_info())?;
    amm_cpi::require_pool_side(&p, 0, &a.vault0.key(), &a.quote_mint.key(), &a.spl_token.key())?;
    amm_cpi::require_pool_side(&p, 1, &a.vault1.key(), &a.mint.key(), &a.token_2022.key())?;
    Ok((Market::from_pool(&p), p.tick_spacing))
}
