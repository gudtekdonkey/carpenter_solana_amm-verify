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

use crate::constants::{CARPENTER_AMM_PROGRAM, EVENT_AUTHORITY_SEED, HOOK_AUTHORITY_SEED};
use crate::errors::HookError;
use crate::events::StairFeesSwept;
use crate::mech::amm_cpi::{self, AmmCpi, PoolAccounts};
use crate::mech::positions::{Market, Mech};
use crate::mech::{ratchet, stair};
use crate::state::FloorPool;

///
pub const SWEEP_RESERVE: u32 = 1 << 16;
///
pub const SWEEP_TAIL: u32 = 1 << 17;
///
pub const SWEEP_CUSHION: u32 = 1 << 18;
///
///
pub const SWEEP_RELIST: u32 = 1 << 19;

#[event_cpi]
#[derive(Accounts)]
pub struct SweepStairFees<'info> {
    pub cranker: Signer<'info>,
    #[account(mut)]
    pub floor_pool: AccountLoader<'info, FloorPool>,
    #[account(address = floor_pool.load()?.base_mint @ HookError::WrongPool)]
    pub base_mint: Box<InterfaceAccount<'info, Mint>>,
///
    #[account(mut, address = floor_pool.load()?.amm_pool @ HookError::WrongPool)]
    pub pool: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub bitmap: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub vault0: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub vault1: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(address = floor_pool.load()?.quote_mint @ HookError::WrongPool)]
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
    pub spl_token: Interface<'info, TokenInterface>,
    pub token_2022: Interface<'info, TokenInterface>,
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
///
///
    #[account(mut)]
    pub tip_destination: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
}

pub fn handler<'info>(ctx: Context<'info, SweepStairFees<'info>>, which: u32) -> Result<()> {
    let mut s: FloorPool = *ctx.accounts.floor_pool.load()?;
    require!(s.pool_bound == 1, HookError::NotBound);
    s.require_quote_live()?;
    if s.stair_steps == 0 || s.stair_ready == 0 {
        return Ok(());
    }

    let a = &ctx.accounts;

    if let Some(d) = &a.tip_destination {
        require_keys_eq!(d.owner, a.cranker.key(), HookError::WrongDestination);
        require_keys_eq!(d.mint, s.quote_mint, HookError::WrongDestination);
    }
    let (market, spacing) = market_and_sides(a)?;
    let mut m = Mech {
        cpi: AmmCpi {
            carpenter_amm: a.carpenter_amm.to_account_info(),
            amm_event_authority: a.amm_event_authority.to_account_info(),
            hook_authority: a.hook_authority.to_account_info(),
            hook_authority_bump: ctx.bumps.hook_authority,
            hook_program: a.program.to_account_info(),
            floor_pool: a.floor_pool.to_account_info(),
            pool: a.pool.to_account_info(),
        },
        accts: PoolAccounts {
            payer: None,

            rent_owner: Pubkey::default(),
            bitmap: a.bitmap.to_account_info(),
            vault0: a.vault0.to_account_info(),
            vault1: a.vault1.to_account_info(),
            mint0: a.quote_mint.to_account_info(),
            mint1: a.base_mint.to_account_info(),
            token_program0: a.spl_token.to_account_info(),
            token_program1: a.token_2022.to_account_info(),
            system_program: a.system_program.to_account_info(),
            tick_arrays: ctx.remaining_accounts.to_vec(),
        },
        position_seed: None,
        position_bid: None,
        bounty_destination: a.tip_destination.as_ref().map(|d| d.to_account_info()),
        spacing,
        market,
        events: Vec::new(),
        modifies: 0,
    };

    let tipping = m.bounty_destination.is_some();
    let (quote, tokens, cushion, tip) = stair::sweep_fees(&mut m, &mut s, which, tipping)?;
    if quote == 0 && tokens == 0 && cushion == 0 && tip == 0 {
        return Ok(());
    }
    s.needs_ratchet = 1;
    if tip != 0 {
        m.pay_bounty(&s, tip)?;
    }
    require!(
        ratchet::claims_cover_liabilities(&s, m.market.hook_claims0, m.market.hook_claims1),
        HookError::InvariantViolated
    );
    m.flush(&s)?;
    emit_cpi!(StairFeesSwept { pool: s.amm_pool, quote, tokens, bounty: tip });
    Ok(())
}

///
///
///
#[inline(never)]
fn market_and_sides(a: &SweepStairFees<'_>) -> Result<(Market, u16)> {
    let p = amm_cpi::read_pool(&a.pool.to_account_info())?;
    amm_cpi::require_pool_side(&p, 0, &a.vault0.key(), &a.quote_mint.key(), &a.spl_token.key())?;
    amm_cpi::require_pool_side(&p, 1, &a.vault1.key(), &a.base_mint.key(), &a.token_2022.key())?;
    Ok((Market::from_pool(&p), p.tick_spacing))
}
