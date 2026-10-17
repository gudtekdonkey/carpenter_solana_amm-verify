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
//!
//!
//!
//!
//!
//!

use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;
use state::Action;

declare_id!("BZfAEAbonruxE1FV7zNoWnSxkUrE71dLAh4qjrLsKGWA");

#[program]
pub mod upgrade_timelock {
    use super::*;

///
///
    pub fn initialize(ctx: Context<Initialize>, proposer: Pubkey) -> Result<()> {
        instructions::initialize::handler(ctx, proposer)
    }

///
///
///
    pub fn schedule(ctx: Context<Schedule>, id: [u8; 32], action: Action) -> Result<()> {
        instructions::schedule::handler(ctx, id, action)
    }

///
///
    pub fn execute<'info>(ctx: Context<'info, Execute<'info>>) -> Result<()> {
        instructions::execute::handler(ctx)
    }

///
    pub fn cancel(ctx: Context<Cancel>) -> Result<()> {
        instructions::cancel::handler(ctx)
    }

///
    pub fn accept_proposer(ctx: Context<AcceptProposer>) -> Result<()> {
        instructions::accept_proposer::handler(ctx)
    }
}
