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
use anchor_spl::token_2022::spl_token_2022::extension::pausable::PausableConfig;
use anchor_spl::token_2022::spl_token_2022::extension::{BaseStateWithExtensions, StateWithExtensions};
use anchor_spl::token_2022::spl_token_2022::state::{AccountState, Mint as MintState};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::errors::HookError;
use crate::events::QuoteObserved;
use crate::mech::amm_cpi;
use crate::state::FloorPool;

#[event_cpi]
#[derive(Accounts)]
pub struct ObserveQuote<'info> {
    #[account(mut)]
    pub floor_pool: AccountLoader<'info, FloorPool>,
///
    #[account(address = floor_pool.load()?.amm_pool @ HookError::WrongPool)]
    pub pool: UncheckedAccount<'info>,
///
///
    pub quote_vault: Box<InterfaceAccount<'info, TokenAccount>>,
///
///
    #[account(address = floor_pool.load()?.quote_mint @ HookError::WrongPool)]
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
///
    pub quote_token_program: Interface<'info, TokenInterface>,
}

pub fn handler(ctx: Context<ObserveQuote>) -> Result<()> {
    let a = &ctx.accounts;
    {
        let s = a.floor_pool.load()?;
        require!(s.pool_bound == 1, HookError::NotBound);
    }
    let p = amm_cpi::read_pool(&a.pool.to_account_info())?;
    amm_cpi::require_pool_side(&p, 0, &a.quote_vault.key(), &a.quote_mint.key(), &a.quote_token_program.key())?;

    let frozen = a.quote_vault.state == AccountState::Frozen;
    let paused = mint_paused(&a.quote_mint.to_account_info())?;

    let pool = a.pool.key();
    let mut s = a.floor_pool.load_mut()?;
    s.quote_suspended = u8::from(frozen || paused);
    drop(s);

    emit_cpi!(QuoteObserved { pool, frozen, paused, slot: Clock::get()?.slot });
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
pub(crate) fn mint_paused(mint: &AccountInfo) -> Result<bool> {
    if *mint.owner == anchor_spl::token::ID {
        return Ok(false);
    }
    let data = mint.try_borrow_data()?;
    let state = StateWithExtensions::<MintState>::unpack(&data)?;
    Ok(state.get_extension::<PausableConfig>().map(|p| bool::from(p.paused)).unwrap_or(false))
}
