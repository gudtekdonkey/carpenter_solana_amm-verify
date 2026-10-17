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
use crate::events::HookPaused;
use crate::state::{AmmConfig, ApprovedHook};

#[event_cpi]
#[derive(Accounts)]
pub struct SetHookPaused<'info> {
    #[account(constraint = authority.key() == config.load()?.authority @ AmmError::Unauthorized)]
    pub authority: Signer<'info>,
    #[account(seeds = [AMM_CONFIG_SEED], bump)]
    pub config: AccountLoader<'info, AmmConfig>,
    #[account(mut)]
    pub approved_hook: AccountLoader<'info, ApprovedHook>,
}

pub fn handler(ctx: Context<SetHookPaused>, paused: bool) -> Result<()> {
    let hook_program = {
        let mut a = ctx.accounts.approved_hook.load_mut()?;
        a.paused = paused as u8;
        if !paused {
            let now = Clock::get()?.unix_timestamp;
            a.guardian_paused_until = a.guardian_paused_until.min(now);
        }
        a.hook_program
    };
    emit_cpi!(HookPaused { hook_program, paused });
    Ok(())
}
