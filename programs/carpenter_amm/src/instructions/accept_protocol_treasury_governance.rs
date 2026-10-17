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

use crate::constants::PROTOCOL_TREASURY_SEED;
use crate::errors::AmmError;
use crate::events::ProtocolTreasuryGovernanceChanged;
use crate::state::ProtocolTreasury;

#[event_cpi]
#[derive(Accounts)]
pub struct AcceptProtocolTreasuryGovernance<'info> {
    pub governance: Signer<'info>,
    #[account(mut, seeds = [PROTOCOL_TREASURY_SEED], bump)]
    pub protocol_treasury: AccountLoader<'info, ProtocolTreasury>,
}

pub fn handler(ctx: Context<AcceptProtocolTreasuryGovernance>) -> Result<()> {
    let treasury = ctx.accounts.protocol_treasury.key();
    let (previous, accepted) = {
        let mut t = ctx.accounts.protocol_treasury.load_mut()?;
        require_keys_neq!(t.pending_governance, Pubkey::default(), AmmError::Unauthorized);
        require_keys_eq!(ctx.accounts.governance.key(), t.pending_governance, AmmError::Unauthorized);
        let previous = t.governance;
        t.governance = t.pending_governance;
        t.pending_governance = Pubkey::default();
        (previous, t.governance)
    };
    emit_cpi!(ProtocolTreasuryGovernanceChanged { treasury, previous, governance: accepted, accepted: true });
    Ok(())
}
