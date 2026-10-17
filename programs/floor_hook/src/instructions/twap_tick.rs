//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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

use crate::errors::HookError;
use crate::mech::amm_cpi;
use crate::state::{twap, FloorPool};

#[derive(Accounts)]
pub struct TwapTick<'info> {
    pub floor_pool: AccountLoader<'info, FloorPool>,
///
///
    #[account(address = floor_pool.load()?.amm_pool @ HookError::WrongPool)]
    pub pool: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<TwapTick>, window: u32) -> Result<i32> {



    require!(ctx.accounts.floor_pool.load()?.quote_is_token1 == 0, HookError::InvariantViolated);
    let current = amm_cpi::read_pool(&ctx.accounts.pool.to_account_info())?.tick_current;
    twap::twap_tick_of_account(&ctx.accounts.floor_pool.to_account_info(), twap::now_secs()?, current, window)
}
