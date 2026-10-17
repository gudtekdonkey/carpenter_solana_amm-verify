use anchor_lang::prelude::*;

use crate::constants::TIMELOCK_SEED;
use crate::errors::TimelockError;
use crate::events::ProposerChanged;
use crate::state::Timelock;

#[derive(Accounts)]
pub struct AcceptProposer<'info> {
    #[account(constraint = timelock.pending_proposer != Pubkey::default()
        && candidate.key() == timelock.pending_proposer @ TimelockError::Unauthorized)]
    pub candidate: Signer<'info>,
    #[account(mut, seeds = [TIMELOCK_SEED], bump = timelock.bump)]
    pub timelock: Account<'info, Timelock>,
}

pub fn handler(ctx: Context<AcceptProposer>) -> Result<()> {
    let t = &mut ctx.accounts.timelock;
    let previous = t.proposer;
    t.proposer = t.pending_proposer;
    t.pending_proposer = Pubkey::default();
    emit!(ProposerChanged { previous, proposer: t.proposer, accepted: true });
    Ok(())
}
