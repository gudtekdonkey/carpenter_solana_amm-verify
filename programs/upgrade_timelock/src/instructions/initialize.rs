use anchor_lang::prelude::*;
use anchor_lang::solana_program::bpf_loader_upgradeable;

use crate::constants::{AUTHORITY_SEED, INITIAL_DELAY, TIMELOCK_SEED};
use crate::errors::TimelockError;
use crate::events::TimelockInitialized;
use crate::instructions::unsignable;
use crate::state::Timelock;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, payer = authority, space = Timelock::LEN, seeds = [TIMELOCK_SEED], bump)]
    pub timelock: Account<'info, Timelock>,
///
    #[account(
        seeds = [crate::ID.as_ref()],
        bump,
        seeds::program = bpf_loader_upgradeable::ID,
        constraint = program_data.upgrade_authority_address == Some(authority.key()) @ TimelockError::Unauthorized,
    )]
    pub program_data: Account<'info, ProgramData>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>, proposer: Pubkey) -> Result<()> {
    require_keys_neq!(proposer, Pubkey::default(), TimelockError::BadAction);
    require!(!unsignable().contains(&proposer), TimelockError::BadAction);
    let t = &mut ctx.accounts.timelock;
    t.bump = ctx.bumps.timelock;
    t.authority_bump = Pubkey::find_program_address(&[AUTHORITY_SEED], &crate::ID).1;
    t.proposer = proposer;
    t.pending_proposer = Pubkey::default();
    t.guardian = Pubkey::default();
    t.delay = INITIAL_DELAY;
    emit!(TimelockInitialized { proposer, delay: INITIAL_DELAY });
    Ok(())
}
