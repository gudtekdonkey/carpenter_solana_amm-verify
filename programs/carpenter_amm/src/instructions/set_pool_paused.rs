//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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

use crate::constants::{pool_is_paused, AMM_CONFIG_SEED};
use crate::errors::AmmError;
use crate::events::PoolPauseSet;
use crate::state::{AmmConfig, Pool};

#[event_cpi]
#[derive(Accounts)]
pub struct SetPoolPaused<'info> {
    #[account(constraint = authority.key() == config.load()?.authority @ AmmError::Unauthorized)]
    pub authority: Signer<'info>,
    #[account(seeds = [AMM_CONFIG_SEED], bump)]
    pub config: AccountLoader<'info, AmmConfig>,
    #[account(mut)]
    pub pool: AccountLoader<'info, Pool>,
}

///
///
#[inline]
pub fn require_not_paused(pool: &Pool) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    require!(!pool_is_paused(pool.paused, pool.guardian_paused_until, now), AmmError::PoolPaused);
    Ok(())
}

pub fn handler(ctx: Context<SetPoolPaused>, paused: bool) -> Result<()> {
    {
        let mut p = ctx.accounts.pool.load_mut()?;
        p.paused = paused as u8;
        if !paused {

            let now = Clock::get()?.unix_timestamp;
            p.guardian_paused_until = p.guardian_paused_until.min(now);
        }
    }
    emit_cpi!(PoolPauseSet { pool: ctx.accounts.pool.key(), paused });
    Ok(())
}
