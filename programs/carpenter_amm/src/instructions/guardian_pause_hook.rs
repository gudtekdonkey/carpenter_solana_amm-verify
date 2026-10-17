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

use crate::constants::AMM_CONFIG_SEED;
use crate::errors::AmmError;
use crate::events::GuardianHookPaused;
use crate::instructions::guardian_pause_pool::guardian_until;
use crate::state::{AmmConfig, ApprovedHook};

#[event_cpi]
#[derive(Accounts)]
pub struct GuardianPauseHook<'info> {
    #[account(constraint = guardian.key() == config.load()?.pause_guardian @ AmmError::Unauthorized)]
    pub guardian: Signer<'info>,
    #[account(seeds = [AMM_CONFIG_SEED], bump)]
    pub config: AccountLoader<'info, AmmConfig>,
    #[account(mut)]
    pub approved_hook: AccountLoader<'info, ApprovedHook>,
}

pub fn handler(ctx: Context<GuardianPauseHook>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let (hook_program, paused_until) = {
        let mut a = ctx.accounts.approved_hook.load_mut()?;
        let until = guardian_until(a.guardian_paused_until, now)?;
        a.guardian_paused_until = until;
        (a.hook_program, until)
    };
    emit_cpi!(GuardianHookPaused { hook_program, guardian: ctx.accounts.guardian.key(), paused_until });
    Ok(())
}
