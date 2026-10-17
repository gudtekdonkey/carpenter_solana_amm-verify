use anchor_lang::prelude::*;

use crate::constants::{AUTHORITY_SEED, OPERATION_SEED, TIMELOCK_SEED};
use crate::errors::TimelockError;
use crate::events::{CodePinned, OperationScheduled};
use crate::instructions::unsignable;
use crate::state::{check_action, pin_for, Action, CodePin, Operation, Timelock};

#[derive(Accounts)]
#[instruction(id: [u8; 32], action: Action)]
pub struct Schedule<'info> {
    #[account(mut, constraint = proposer.key() == timelock.proposer @ TimelockError::Unauthorized)]
    pub proposer: Signer<'info>,
    #[account(seeds = [TIMELOCK_SEED], bump = timelock.bump)]
    pub timelock: Account<'info, Timelock>,
    #[account(init, payer = proposer, space = Operation::space(&action), seeds = [OPERATION_SEED, id.as_ref()], bump)]
    pub operation: Account<'info, Operation>,
    pub system_program: Program<'info, System>,
}

///
///
///
pub fn handler(ctx: Context<Schedule>, id: [u8; 32], action: Action) -> Result<()> {
    let delay = ctx.accounts.timelock.delay;
    check_action(&action, delay, &crate::ID, &unsignable())?;
    let clock = Clock::get()?;
    let authority = Pubkey::create_program_address(
        &[AUTHORITY_SEED, &[ctx.accounts.timelock.authority_bump]],
        &crate::ID,
    )
    .map_err(|_| TimelockError::BadAction)?;
    let pin = {
        let mut held = Vec::with_capacity(ctx.remaining_accounts.len());
        for a in ctx.remaining_accounts.iter() {
            held.push((a.key(), *a.owner, a.try_borrow_data()?));
        }
        pin_for(&action, &authority, clock.slot, |k| {
            held.iter().find(|(key, _, _)| key == k).map(|(_, owner, d)| (*owner, &***d))
        })?
    };
    let now = clock.unix_timestamp;
    let ready_at = now.checked_add(delay as i64).ok_or(TimelockError::BadAction)?;
    let encoded = anchor_lang::prelude::borsh::to_vec(&action)?;
    let op = &mut ctx.accounts.operation;
    op.bump = ctx.bumps.operation;
    op.id = id;
    op.payer = ctx.accounts.proposer.key();
    op.ready_at = ready_at;
    op.action = action;
    op.pin = pin;
    emit!(OperationScheduled { operation: op.key(), id, action: encoded, ready_at });
    if pin != CodePin::None {
        emit!(CodePinned { operation: op.key(), id, pin });
    }
    Ok(())
}
