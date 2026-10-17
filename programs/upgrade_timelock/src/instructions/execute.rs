use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::{AccountMeta, Instruction};
use anchor_lang::solana_program::program::invoke_signed;

use crate::constants::{AUTHORITY_SEED, OPERATION_SEED, TIMELOCK_SEED};
use crate::errors::TimelockError;
use crate::events::{DelayChanged, GuardianChanged, OperationExecuted, ProposerChanged};
use crate::state::{check_buffer, check_delay, program_data_slot, Action, CodePin, Operation, Timelock};

#[derive(Accounts)]
pub struct Execute<'info> {
    #[account(constraint = proposer.key() == timelock.proposer @ TimelockError::Unauthorized)]
    pub proposer: Signer<'info>,
    #[account(mut, seeds = [TIMELOCK_SEED], bump = timelock.bump)]
    pub timelock: Account<'info, Timelock>,
    #[account(mut, close = payer, seeds = [OPERATION_SEED, operation.id.as_ref()], bump = operation.bump, has_one = payer @ TimelockError::AccountMismatch)]
    pub operation: Account<'info, Operation>,
///
    #[account(mut)]
    pub payer: UncheckedAccount<'info>,
}

pub fn handler<'info>(ctx: Context<'info, Execute<'info>>) -> Result<()> {
    require!(Clock::get()?.unix_timestamp >= ctx.accounts.operation.ready_at, TimelockError::NotReady);
    let action = ctx.accounts.operation.action.clone();
    let pin = ctx.accounts.operation.pin;
    let t = &mut ctx.accounts.timelock;
    match action {
        Action::UpdateDelay { delay } => {

            check_delay(t.delay, delay)?;
            emit!(DelayChanged { previous: t.delay, delay });
            t.delay = delay;
        }
        Action::SetProposer { proposer } => {
            t.pending_proposer = proposer;
            emit!(ProposerChanged { previous: t.proposer, proposer, accepted: false });
        }
        Action::SetGuardian { guardian } => {
            emit!(GuardianChanged { previous: t.guardian, guardian });
            t.guardian = guardian;
        }
        Action::Invoke { program_id, accounts, data } => {
            let rem = ctx.remaining_accounts;

            let extra = usize::from(matches!(pin, CodePin::Program { .. }));
            require!(rem.len() == accounts.len() + 1 + extra, TimelockError::AccountMismatch);
            for (m, info) in accounts.iter().zip(rem.iter()) {
                require_keys_eq!(m.pubkey, *info.key, TimelockError::AccountMismatch);
            }
            require_keys_eq!(*rem[accounts.len()].key, program_id, TimelockError::AccountMismatch);

            match pin {
                CodePin::None => {}
                CodePin::Buffer { buffer, code_hash } => {
                    let info = rem[..accounts.len()]
                        .iter()
                        .find(|a| *a.key == buffer)
                        .ok_or(TimelockError::PinAccountMissing)?;
                    let authority = Pubkey::create_program_address(&[AUTHORITY_SEED, &[t.authority_bump]], &crate::ID)
                        .map_err(|_| TimelockError::BadAction)?;
                    check_buffer(info.owner, &info.try_borrow_data()?, &authority, &code_hash)?;
                }
                CodePin::Program { program_data, deploy_slot } => {
                    let info = &rem[accounts.len() + 1];
                    require_keys_eq!(*info.key, program_data, TimelockError::PinAccountMissing);
                    let slot = program_data_slot(&info.try_borrow_data()?);
                    require!(slot == Some(deploy_slot), TimelockError::ProgramRedeployed);
                }
            }
            let ix = Instruction {
                program_id,
                accounts: accounts
                    .iter()
                    .map(|m| AccountMeta { pubkey: m.pubkey, is_signer: m.is_signer, is_writable: m.is_writable })
                    .collect(),
                data,
            };
            let bump = [t.authority_bump];
            invoke_signed(&ix, rem, &[&[AUTHORITY_SEED, &bump]])?;
        }
    }
    emit!(OperationExecuted { operation: ctx.accounts.operation.key(), id: ctx.accounts.operation.id });
    Ok(())
}
