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

use crate::constants::{EVENT_AUTHORITY_SEED, CARPENTER_AMM_PROGRAM, HOOK_AUTHORITY_SEED, STAIR_RENT_SEED};
use crate::errors::HookError;
use crate::events::{Ratcheted, StairRentReimbursed};
use crate::mech::amm_cpi::{self, AmmCpi, PoolAccounts};
use crate::mech::positions::{Market, Mech, MechEvent};
use crate::mech::ratchet;
use crate::state::FloorPool;

#[event_cpi]
#[derive(Accounts)]
pub struct Ratchet<'info> {
///
///
    #[account(mut)]
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
///
///
    #[account(address = floor_pool.load()?.quote_mint @ HookError::WrongPool)]
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
    pub spl_token: Interface<'info, TokenInterface>,
    pub token_2022: Interface<'info, TokenInterface>,
///
    #[account(mut)]
    pub position_seed: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub position_bid: UncheckedAccount<'info>,



///
///
///
    #[account(mut)]
    pub bounty_destination: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
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
///
///
///
///
///
///
///
///
pub fn entry<'info>(ctx: Context<'info, Ratchet<'info>>, rent_owner: Pubkey) -> Result<()> {
    let reserve = stair_rent_reserve(&ctx, rent_owner)?;
    let cranker = ctx.accounts.cranker.to_account_info();
    let system = ctx.accounts.system_program.to_account_info();
    let before = cranker.lamports();
    let owner = reserve.as_ref().map(|r| *r.info.key).unwrap_or(rent_owner);
    handler(ctx, owner)?;
    if let Some(r) = reserve {
        reimburse_cranker(&r, &cranker, &system, before)?;
    }
    Ok(())
}

///
///
#[inline(never)]
fn stair_rent_reserve<'info>(ctx: &Context<'info, Ratchet<'info>>, rent_owner: Pubkey) -> Result<Option<StairRentReserve<'info>>> {
    if rent_owner != Pubkey::default() {
        return Ok(None);
    }
    let amm_pool = ctx.accounts.floor_pool.load()?.amm_pool;
    let (key, bump) = Pubkey::find_program_address(&[STAIR_RENT_SEED, amm_pool.as_ref()], &crate::ID);
    let Some(r) = ctx.remaining_accounts.iter().find(|a| *a.key == key) else { return Ok(None) };
    if !r.is_writable || *r.owner != anchor_lang::solana_program::system_program::ID {
        return Ok(None);
    }
    let rent = Rent::get()?;
    let available = r.lamports().saturating_sub(rent.minimum_balance(0));
    if available < rent.minimum_balance(floor_types::layout::TICK_ARRAY_LEN) {
        return Ok(None);
    }
    Ok(Some(StairRentReserve { info: r.clone(), bump, amm_pool }))
}

///
struct StairRentReserve<'info> {
    info: AccountInfo<'info>,
    bump: u8,
    amm_pool: Pubkey,
}

///
///
#[inline(never)]
fn reimburse_cranker<'info>(
    r: &StairRentReserve<'info>,
    cranker: &AccountInfo<'info>,
    system: &AccountInfo<'info>,
    before: u64,
) -> Result<()> {
    let spent = before.saturating_sub(cranker.lamports());
    let reserve = &r.info;
    let available = reserve.lamports().saturating_sub(Rent::get()?.minimum_balance(0));
    let amount = spent.min(available);
    if amount == 0 {
        return Ok(());
    }
    let seeds: &[&[u8]] = &[STAIR_RENT_SEED, r.amm_pool.as_ref(), &[r.bump]];
    anchor_lang::system_program::transfer(
        CpiContext::new_with_signer(
            *system.key,
            anchor_lang::system_program::Transfer { from: reserve.clone(), to: cranker.clone() },
            &[seeds],
        ),
        amount,
    )?;
    emit!(StairRentReimbursed { reserve: *reserve.key, cranker: *cranker.key, spent, lamports: amount });
    Ok(())
}

///
#[inline(never)]
pub fn handler<'info>(ctx: Context<'info, Ratchet<'info>>, rent_owner: Pubkey) -> Result<()> {
    crate::cu::mark("cu h.entry");
    let mut s: FloorPool = *ctx.accounts.floor_pool.load()?;
    require!(s.pool_bound == 1, HookError::NotBound);


    s.require_quote_live()?;

    let a = &ctx.accounts;

    let (market, spacing) = bound_market(a)?;
    crate::cu::mark("cu h.market");


    if ratchet::is_noop(&s, market.tick)? && !older_surplus_due(&ctx.accounts.floor_pool.to_account_info(), &s, market.tick)? {
        return Ok(());
    }




    if let Some(d) = &a.bounty_destination {
        require_keys_eq!(d.owner, a.cranker.key(), HookError::WrongDestination);


        require_keys_eq!(d.mint, s.quote_mint, HookError::WrongDestination);
    }







    twap_checkpoint(&a.floor_pool.to_account_info(), market.tick)?;
    crate::cu::mark("cu h.twap");

    let supply = a.base_mint.supply;
    let entry = ratchet::Entry::of(&s);

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
            payer: Some(a.cranker.to_account_info()),
            rent_owner,
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
        position_seed: Some(a.position_seed.to_account_info()),
        position_bid: Some(a.position_bid.to_account_info()),
        bounty_destination: a.bounty_destination.as_ref().map(|d| d.to_account_info()),
        spacing,
        market,






        events: Vec::with_capacity(RATCHET_EVENTS_MAX),
        modifies: 0,
    };

    crate::cu::mark("cu h.mech");
    let (at, bounty) = ratchet::run(&mut m, &mut s, supply)?;
    s.last_ratchet_slot = Clock::get()?.slot;
    ratchet::epilogue(&s, &entry, m.market.hook_claims0, m.market.hook_claims1)?;
    crate::cu::mark("cu h.epilogue");
    m.flush(&s)?;
    crate::cu::mark("cu h.flush");

    let pool = s.amm_pool;
    emit_mech_events(&ctx, m.events)?;
    emit_cpi!(Ratcheted {
        pool,
        floor_tick: s.floor_tick,
        ask_tick: at,
        vault_eth: s.vault_eth,
        inventory_tokens: s.inventory_tokens,
        bounty,
    });
    crate::cu::mark("cu h.events");


    #[cfg(all(target_os = "solana", feature = "custom-heap", feature = "heap-trace", not(feature = "no-entrypoint")))]
    {
        let (in_use, peak, requested) = crate::heap::trace();
        msg!("heap-trace: ratchet in_use={} peak={} requested={}", in_use, peak, requested);
    }
    Ok(())
}

///
///
#[inline(never)]
fn emit_mech_events<'info>(ctx: &Context<'info, Ratchet<'info>>, events: Vec<MechEvent>) -> Result<()> {
    for e in events {
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
#[inline(never)]
fn older_surplus_due(floor_pool: &AccountInfo<'_>, s: &FloorPool, tick: i32) -> Result<bool> {
    ratchet::older_surplus_due(floor_pool, s, tick)
}

///
const RATCHET_EVENTS_MAX: usize = 12;

///
///
#[inline(never)]
fn twap_checkpoint(floor_pool: &AccountInfo<'_>, entry_tick: i32) -> Result<()> {
    crate::state::twap::observe_account(floor_pool, crate::state::twap::now_secs()?, entry_tick)?;
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
///
///
///
///
#[inline(never)]
fn bound_market(a: &Ratchet<'_>) -> Result<(Market, u16)> {
    let p = amm_cpi::read_pool(&a.pool.to_account_info())?;
    amm_cpi::require_pool_side(&p, 0, &a.vault0.key(), &a.quote_mint.key(), &a.spl_token.key())?;
    amm_cpi::require_pool_side(&p, 1, &a.vault1.key(), &a.base_mint.key(), &a.token_2022.key())?;
    Ok((Market::from_pool(&p), p.tick_spacing))
}
