//!
//!
//!
//!
//!
//!
//!
//!

use anchor_lang::prelude::*;

use crate::errors::AmmError;
use crate::events::PositionClosed;
use crate::state::Position;

#[event_cpi]
#[derive(Accounts)]
pub struct ClosePosition<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut, close = owner, has_one = owner @ AmmError::Unauthorized)]
    pub position: AccountLoader<'info, Position>,
}

pub fn handler(ctx: Context<ClosePosition>) -> Result<()> {
    {
        let p = ctx.accounts.position.load()?;
        require!(p.liquidity.get() == 0, AmmError::PositionNotEmpty);
    }

    emit_cpi!(PositionClosed { position: ctx.accounts.position.key(), owner: ctx.accounts.owner.key() });
    Ok(())
}
