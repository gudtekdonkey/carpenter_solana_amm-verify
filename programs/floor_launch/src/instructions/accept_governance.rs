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
pub struct AcceptGovernance<'info> {
    pub governance: Signer<'info>,
    #[account(mut, seeds = [LAUNCH_CONFIG_SEED], bump)]
    pub launch_config: AccountLoader<'info, LaunchConfig>,
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
pub fn handler(ctx: Context<AcceptGovernance>) -> Result<()> {
    let (previous, accepted) = {
        let mut c = ctx.accounts.launch_config.load_mut()?;
        require_keys_neq!(c.pending_governance, Pubkey::default(), LaunchError::Unauthorized);
        require_keys_eq!(ctx.accounts.governance.key(), c.pending_governance, LaunchError::Unauthorized);
        require_keys_neq!(c.pending_governance, c.attestor, LaunchError::GovernanceIsAttestor);
        let previous = c.governance;
        c.governance = c.pending_governance;
        c.pending_governance = Pubkey::default();
        (previous, c.governance)
    };
    emit_cpi!(GovernanceChanged {
        launch_config: ctx.accounts.launch_config.key(),
        previous,
        governance: accepted,
        accepted: true,
    });
    Ok(())
}
