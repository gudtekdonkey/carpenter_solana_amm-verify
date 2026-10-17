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

use crate::constants::{guardian_pause_until, AMM_CONFIG_SEED};
use crate::errors::AmmError;
use crate::events::GuardianPoolPaused;
use crate::state::{AmmConfig, Pool};

#[event_cpi]
#[derive(Accounts)]
pub struct GuardianPausePool<'info> {
    #[account(constraint = guardian.key() == config.load()?.pause_guardian @ AmmError::Unauthorized)]
    pub guardian: Signer<'info>,
    #[account(seeds = [AMM_CONFIG_SEED], bump)]
    pub config: AccountLoader<'info, AmmConfig>,
    #[account(mut)]
    pub pool: AccountLoader<'info, Pool>,
}

///
pub fn guardian_until(current: i64, now: i64) -> Result<i64> {
    match guardian_pause_until(current, now) {
        Ok(until) => Ok(until),
        Err(true) => err!(AmmError::GuardianPauseRunning),
        Err(false) => err!(AmmError::GuardianCoolOff),
    }
}

pub fn handler(ctx: Context<GuardianPausePool>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let paused_until = {
        let mut p = ctx.accounts.pool.load_mut()?;
        let until = guardian_until(p.guardian_paused_until, now)?;
        p.guardian_paused_until = until;
        until
    };
    emit_cpi!(GuardianPoolPaused { pool: ctx.accounts.pool.key(), guardian: ctx.accounts.guardian.key(), paused_until });
    Ok(())
}
