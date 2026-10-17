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

use crate::constants::AMM_CONFIG_SEED;
use crate::errors::AmmError;
use crate::events::PauseGuardianSet;
use crate::state::AmmConfig;

#[event_cpi]
#[derive(Accounts)]
pub struct SetPauseGuardian<'info> {
    #[account(constraint = authority.key() == config.load()?.authority @ AmmError::Unauthorized)]
    pub authority: Signer<'info>,
    #[account(mut, seeds = [AMM_CONFIG_SEED], bump)]
    pub config: AccountLoader<'info, AmmConfig>,
}

pub fn handler(ctx: Context<SetPauseGuardian>, guardian: Pubkey) -> Result<()> {
    let previous = {
        let mut c = ctx.accounts.config.load_mut()?;
        let previous = c.pause_guardian;
        c.pause_guardian = guardian;
        previous
    };
    emit_cpi!(PauseGuardianSet { previous, guardian });
    Ok(())
}
