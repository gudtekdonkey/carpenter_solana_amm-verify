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

use crate::constants::LAUNCH_CONFIG_SEED;
use crate::errors::LaunchError;
use crate::events::GovernanceChanged;
use crate::state::LaunchConfig;

#[event_cpi]
#[derive(Accounts)]
pub struct SetGovernance<'info> {
    pub governance: Signer<'info>,
    #[account(mut, seeds = [LAUNCH_CONFIG_SEED], bump)]
    pub launch_config: AccountLoader<'info, LaunchConfig>,
}

pub fn handler(ctx: Context<SetGovernance>, new_governance: Pubkey) -> Result<()> {
    let previous = {
        let mut c = ctx.accounts.launch_config.load_mut()?;
        require_keys_eq!(ctx.accounts.governance.key(), c.governance, LaunchError::Unauthorized);
        if new_governance != Pubkey::default() {
            require_keys_neq!(new_governance, c.attestor, LaunchError::GovernanceIsAttestor);
        }
        c.pending_governance = new_governance;
        c.governance
    };
    emit_cpi!(GovernanceChanged {
        launch_config: ctx.accounts.launch_config.key(),
        previous,
        governance: new_governance,
        accepted: false,
    });
    Ok(())
}
