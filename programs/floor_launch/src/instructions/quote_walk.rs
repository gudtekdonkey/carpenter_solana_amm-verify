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
use anchor_lang::system_program;

use floor_types::constants::QUOTE_WALK_MAX_AGE_SECS;

use super::quote_chain::{end_at_root, floor_platform_pool, walk_hops, Chain, WALK_FIXED};
use crate::constants::QUOTE_WALK_SEED;
use crate::errors::LaunchError;
use crate::sale::create_pda;
use crate::state::QuoteWalk;

const WALK_LEN: usize = 8 + core::mem::size_of::<QuoteWalk>();

#[derive(Accounts)]
pub struct WalkQuoteChain<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
///
///
    pub quote_mint: UncheckedAccount<'info>,
///
///
    #[account(mut, seeds = [QUOTE_WALK_SEED, payer.key().as_ref(), quote_mint.key().as_ref()], bump)]
    pub quote_walk: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CloseQuoteWalk<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, close = payer, constraint = quote_walk.load()?.payer == payer.key() @ LaunchError::QuoteWalkMismatch)]
    pub quote_walk: AccountLoader<'info, QuoteWalk>,
}

///
pub fn walk_quote_chain_handler(ctx: Context<WalkQuoteChain>, restart: bool, finish: bool) -> Result<()> {
    let clock = Clock::get()?;
    let now = clock.unix_timestamp;
    let now32 = u32::try_from(now).map_err(|_| LaunchError::MathOverflow)?;
    let payer = ctx.accounts.payer.key();
    let quote = ctx.accounts.quote_mint.key();
    let bump = ctx.bumps.quote_walk;
    let info = ctx.accounts.quote_walk.to_account_info();
    let created = *info.owner == system_program::ID;
    if created {
        create_pda(
            &ctx.accounts.payer.to_account_info(),
            &info,
            WALK_LEN,
            &crate::ID,
            &[QUOTE_WALK_SEED, payer.as_ref(), quote.as_ref(), &[bump]],
        )?;
    }
    require_keys_eq!(*info.owner, crate::ID, LaunchError::QuoteWalkMismatch);
    let mut data = info.try_borrow_mut_data()?;
    require!(data.len() == WALK_LEN, LaunchError::QuoteWalkMismatch);
    if created {
        data[..8].copy_from_slice(QuoteWalk::DISCRIMINATOR);
    }
    require!(data[..8] == *QuoteWalk::DISCRIMINATOR, LaunchError::QuoteWalkMismatch);
    let w: &mut QuoteWalk = bytemuck::from_bytes_mut(&mut data[8..WALK_LEN]);
    if created || restart {
        *w = QuoteWalk {
            payer,
            quote_mint: quote,
            cur: quote,
            ticks: 0,
            started_at: now,
            started_slot: clock.slot,
            hops: 0,
            bump,
            done: 0,
            root_decimals: 0,
            pad0: [0; 3],
        };
    } else {
        require!(w.done == 0, LaunchError::QuoteWalkIncomplete);
        require!(now.saturating_sub(w.started_at) <= QUOTE_WALK_MAX_AGE_SECS, LaunchError::QuoteWalkStale);
    }
    let rem = ctx.remaining_accounts;
    let tail = if finish { WALK_FIXED - 1 } else { 0 };
    require!(rem.len() > tail, LaunchError::QuoteChainBadHop);
    let floor_pool = floor_platform_pool(&rem[0])?;
    let (cur, ticks, hops) = walk_hops(&rem[1..rem.len() - tail], &w.cur, floor_pool, now32)?;
    w.ticks = w.ticks.checked_add(ticks).ok_or(LaunchError::MathOverflow)?;
    let hops = u16::try_from(hops).map_err(|_| LaunchError::MathOverflow)?;
    w.hops = w.hops.checked_add(hops).ok_or(LaunchError::MathOverflow)?;
    w.cur = cur;
    if finish {
        w.root_decimals = end_at_root(&rem[rem.len() - tail..], &cur, floor_pool)?;
        w.done = 1;
    }
    Ok(())
}

///
pub fn close_quote_walk_handler(_ctx: Context<CloseQuoteWalk>) -> Result<()> {
    Ok(())
}

///
///
///
pub fn read_finished(acct: &AccountInfo, payer: &Pubkey, quote: &Pubkey, now: i64) -> Result<Chain> {
    let (expected, _) = Pubkey::find_program_address(&[QUOTE_WALK_SEED, payer.as_ref(), quote.as_ref()], &crate::ID);
    require_keys_eq!(acct.key(), expected, LaunchError::QuoteWalkMismatch);
    require_keys_eq!(*acct.owner, crate::ID, LaunchError::QuoteWalkMismatch);
    require!(acct.is_writable, LaunchError::QuoteWalkMismatch);
    let data = acct.try_borrow_data()?;
    require!(data.len() == WALK_LEN && data[..8] == *QuoteWalk::DISCRIMINATOR, LaunchError::QuoteWalkMismatch);
    let w: QuoteWalk = bytemuck::pod_read_unaligned(&data[8..WALK_LEN]);
    require!(w.payer == *payer && w.quote_mint == *quote, LaunchError::QuoteWalkMismatch);
    require!(w.done == 1, LaunchError::QuoteWalkIncomplete);
    require!(now.saturating_sub(w.started_at) <= QUOTE_WALK_MAX_AGE_SECS, LaunchError::QuoteWalkStale);
    Ok(Chain { root: w.cur, root_decimals: w.root_decimals, ticks: w.ticks, hops: w.hops })
}

///
///
pub fn close_into<'info>(acct: &AccountInfo<'info>, payer: &AccountInfo<'info>) -> Result<()> {
    payer.add_lamports(acct.lamports())?;
    **acct.try_borrow_mut_lamports()? = 0;
    acct.assign(&system_program::ID);
    acct.resize(0)?;
    Ok(())
}
