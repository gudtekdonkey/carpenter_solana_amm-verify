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
use carpenter_types::layout::{tick_array_len_ok, tick_array_rent_owner};

use crate::amm_core::bitmap;
use crate::constants::TICK_ARRAY_SEED;
use crate::errors::AmmError;
use crate::events::TickArrayClosed;
use crate::state::{Pool, TickArray, TickArrayBitmap};

#[event_cpi]
#[derive(Accounts)]
pub struct CloseTickArray<'info> {
///
    #[account(mut)]
    pub rent_owner: Signer<'info>,
    pub pool: AccountLoader<'info, Pool>,
    #[account(constraint = bitmap.load()?.pool == pool.key() @ AmmError::SeedsMismatch)]
    pub bitmap: AccountLoader<'info, TickArrayBitmap>,
    #[account(mut, close = rent_owner)]
    pub tick_array: AccountLoader<'info, TickArray>,
}

pub fn handler(ctx: Context<CloseTickArray>) -> Result<()> {
    let pool_key = ctx.accounts.pool.key();
    let spacing = ctx.accounts.pool.load()?.tick_spacing;
    let bitmap_info = ctx.accounts.bitmap.to_account_info();

    bitmap::check_account(&bitmap_info, &pool_key, spacing)?;

    let info = ctx.accounts.tick_array.to_account_info();
    let start_index = {
        let data = info.try_borrow_data()?;


        require!(tick_array_len_ok(data.len()), AmmError::SeedsMismatch);
        let a: &TickArray = bytemuck::from_bytes(&data[8..TickArray::LEN]);

        require_keys_eq!(a.pool, pool_key, AmmError::SeedsMismatch);
        let (want, _) = Pubkey::find_program_address(
            &[TICK_ARRAY_SEED, pool_key.as_ref(), &a.start_index.to_le_bytes()],
            &crate::ID,
        );
        require_keys_eq!(want, info.key(), AmmError::SeedsMismatch);

        let recorded = tick_array_rent_owner(&data).ok_or(AmmError::SeedsMismatch)?;

        require_keys_eq!(Pubkey::new_from_array(recorded), ctx.accounts.rent_owner.key(), AmmError::Unauthorized);

        require!(a.initialized_tick_count == 0, AmmError::TickArrayNotEmpty);



        let words: &[u64] = bytemuck::cast_slice(&a.ticks);
        require!(words.iter().all(|w| *w == 0), AmmError::TickArrayNotZeroed);
        a.start_index
    };

    require!(
        !bitmap::is_array_initialized(&bitmap_info, spacing, start_index)?,
        AmmError::TickArrayBitmapBitSet
    );


    emit_cpi!(TickArrayClosed {
        pool: pool_key,
        tick_array: info.key(),
        start_index,
        rent_owner: ctx.accounts.rent_owner.key(),
        lamports: info.lamports(),
    });
    Ok(())
}
