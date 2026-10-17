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
//!
//!

use anchor_lang::prelude::*;

use crate::amm_core::positions::create_position;
use crate::constants::POSITION_SEED;
use crate::errors::AmmError;
use crate::state::{Pool, Position};

#[derive(Accounts)]
#[instruction(salt: u64)]
pub struct InitializePosition<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
///
///
    pub pool: AccountLoader<'info, Pool>,
///
///
    #[account(mut)]
    pub position: UncheckedAccount<'info>,
///
///
    pub position_authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializePosition>, salt: u64) -> Result<()> {
    let pool_key = ctx.accounts.pool.key();
    let id = ctx.accounts.position_authority.key();
    let (want, bump) = Pubkey::find_program_address(
        &[POSITION_SEED, pool_key.as_ref(), id.as_ref(), &salt.to_le_bytes()],
        &crate::ID,
    );
    let info = ctx.accounts.position.to_account_info();
    require_keys_eq!(info.key(), want, AmmError::SeedsMismatch);

    if *info.owner == crate::ID {

        let data = info.try_borrow_data()?;
        require!(data.len() == Position::LEN && &data[..8] == Position::DISCRIMINATOR, AmmError::SeedsMismatch);
        let p: &Position = bytemuck::from_bytes(&data[8..Position::LEN]);
        require_keys_eq!(p.pool, pool_key, AmmError::SeedsMismatch);
        require_keys_eq!(p.owner, id, AmmError::Unauthorized);
        require_keys_eq!(p.position_id, id, AmmError::SeedsMismatch);
        require!(p.salt == salt, AmmError::SeedsMismatch);
        return Ok(());
    }

    create_position(
        &ctx.accounts.payer.to_account_info(),
        &info,
        &pool_key,
        &id,
        &id.to_bytes(),
        salt,
        bump,
        0,
        0,
    )
}
