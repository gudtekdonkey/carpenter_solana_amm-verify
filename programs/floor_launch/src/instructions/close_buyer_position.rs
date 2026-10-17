//!
//!
//!

use anchor_lang::prelude::*;

use crate::constants::BUYER_SEED;
use crate::errors::LaunchError;
use crate::state::{BuyerPosition, Launch};

#[derive(Accounts)]
pub struct CloseBuyerPosition<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    pub launch: AccountLoader<'info, Launch>,
    #[account(
        mut,
        close = buyer,
        seeds = [BUYER_SEED, launch.key().as_ref(), buyer.key().as_ref()],
        bump,
    )]
    pub buyer_position: AccountLoader<'info, BuyerPosition>,
}

///
///
///
///
///
pub fn handler(ctx: Context<CloseBuyerPosition>) -> Result<()> {
    let graduated = ctx.accounts.launch.load()?.state == floor_types::constants::LAUNCH_STATE_GRADUATED;
    let allocation = ctx.accounts.buyer_position.load()?.allocation;
    require!(allocation == 0 || graduated, LaunchError::PositionNotClosable);
    Ok(())
}
