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
use crate::instructions::initialize_protocol_treasury::refuse_as_governance;
use crate::state::ProtocolTreasury;

#[event_cpi]
#[derive(Accounts)]
pub struct SetProtocolTreasuryGovernance<'info> {
    #[account(constraint = governance.key() == protocol_treasury.load()?.governance @ AmmError::Unauthorized)]
    pub governance: Signer<'info>,
    #[account(mut, seeds = [PROTOCOL_TREASURY_SEED], bump)]
    pub protocol_treasury: AccountLoader<'info, ProtocolTreasury>,
}

pub fn handler(ctx: Context<SetProtocolTreasuryGovernance>, new_governance: Pubkey) -> Result<()> {
    let treasury = ctx.accounts.protocol_treasury.key();
    if new_governance != Pubkey::default() {
        refuse_as_governance(&new_governance, &treasury)?;
    }
    let previous = {
        let mut t = ctx.accounts.protocol_treasury.load_mut()?;
        t.pending_governance = new_governance;
        t.governance
    };
    emit_cpi!(ProtocolTreasuryGovernanceChanged { treasury, previous, governance: new_governance, accepted: false });
    Ok(())
}
