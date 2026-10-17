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

use crate::errors::AmmError;
use crate::events::PositionTransferred;
use crate::state::{Pool, Position};

#[event_cpi]
#[derive(Accounts)]
pub struct TransferPosition<'info> {
    pub owner: Signer<'info>,
    #[account(mut, has_one = owner @ AmmError::Unauthorized, has_one = pool @ AmmError::SeedsMismatch)]
    pub position: AccountLoader<'info, Position>,
    pub pool: AccountLoader<'info, Pool>,
}

pub fn handler(ctx: Context<TransferPosition>, new_owner: Pubkey) -> Result<()> {
    let from = ctx.accounts.owner.key();
    require_keys_neq!(new_owner, Pubkey::default(), AmmError::PositionTransferInvalid);
    require_keys_neq!(new_owner, from, AmmError::PositionTransferInvalid);
    {
        let pool = ctx.accounts.pool.load()?;


        if pool.hook != Pubkey::default() {
            require_keys_neq!(from, pool.hook_authority, AmmError::PositionNotTransferable);
            require_keys_neq!(new_owner, pool.hook_authority, AmmError::PositionNotTransferable);
        }
    }
    {
        let mut p = ctx.accounts.position.load_mut()?;

        p.owner = new_owner;
    }
    emit_cpi!(PositionTransferred {
        position: ctx.accounts.position.key(),
        pool: ctx.accounts.pool.key(),
        from,
        to: new_owner,
    });
    Ok(())
}
