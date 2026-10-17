use anchor_lang::prelude::*;

use crate::constants::{OPERATION_SEED, TIMELOCK_SEED};
use crate::errors::TimelockError;
use crate::events::OperationCancelled;
use crate::state::{Operation, Timelock};

#[derive(Accounts)]
pub struct Cancel<'info> {
///
    #[account(constraint = (canceller.key() == timelock.proposer
        || (timelock.guardian != Pubkey::default() && canceller.key() == timelock.guardian)) @ TimelockError::Unauthorized)]
    pub canceller: Signer<'info>,
    #[account(seeds = [TIMELOCK_SEED], bump = timelock.bump)]
    pub timelock: Account<'info, Timelock>,
    #[account(mut, close = payer, seeds = [OPERATION_SEED, operation.id.as_ref()], bump = operation.bump, has_one = payer @ TimelockError::AccountMismatch)]
    pub operation: Account<'info, Operation>,
///
    #[account(mut)]
    pub payer: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<Cancel>) -> Result<()> {
    emit!(OperationCancelled {
        operation: ctx.accounts.operation.key(),
        id: ctx.accounts.operation.id,
        by: ctx.accounts.canceller.key(),
    });
    Ok(())
}
