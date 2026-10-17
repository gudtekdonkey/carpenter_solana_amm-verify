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
use carpenter_clmm_math::bitmap as mbitmap;
use carpenter_types::layout::{bitmap_account_len, bitmap_arrays_per_side};

use crate::amm_core::ticks::math_err;
use crate::errors::AmmError;
use crate::state::TickArrayBitmap;

///
pub const WORDS_OFFSET: usize = 8 + core::mem::size_of::<TickArrayBitmap>();

///
pub fn check_account(info: &AccountInfo, pool: &Pubkey, tick_spacing: u16) -> Result<()> {
    require_keys_eq!(*info.owner, crate::ID, AmmError::SeedsMismatch);
    let data = info.try_borrow_data()?;
    require!(data.len() == bitmap_account_len(tick_spacing), AmmError::SeedsMismatch);
    require!(&data[..8] == TickArrayBitmap::DISCRIMINATOR, AmmError::SeedsMismatch);
    let hdr: TickArrayBitmap = bytemuck::pod_read_unaligned(&data[8..WORDS_OFFSET]);
    require_keys_eq!(hdr.pool, *pool, AmmError::SeedsMismatch);
    require!(hdr.tick_spacing == tick_spacing, AmmError::SeedsMismatch);
    Ok(())
}

///
pub fn init_account(info: &AccountInfo, pool: &Pubkey, tick_spacing: u16) -> Result<()> {
    let mut data = info.try_borrow_mut_data()?;
    require!(data.len() == bitmap_account_len(tick_spacing), AmmError::SeedsMismatch);
    data[..8].copy_from_slice(TickArrayBitmap::DISCRIMINATOR);
    let hdr = TickArrayBitmap {
        pool: *pool,
        tick_spacing,
        pad0: 0,
        arrays_per_side: bitmap_arrays_per_side(tick_spacing),
    };
    data[8..WORDS_OFFSET].copy_from_slice(bytemuck::bytes_of(&hdr));
    Ok(())
}

///
pub fn set_array_initialized(info: &AccountInfo, tick_spacing: u16, start_index: i32, initialized: bool) -> Result<()> {
    let mut data = info.try_borrow_mut_data()?;
    require!(data.len() == bitmap_account_len(tick_spacing), AmmError::SeedsMismatch);
    let words: &mut [u64] =
        bytemuck::try_cast_slice_mut(&mut data[WORDS_OFFSET..]).map_err(|_| AmmError::SeedsMismatch)?;
    mbitmap::check_words(words, tick_spacing).map_err(math_err)?;
    let bit = mbitmap::array_bit(start_index, tick_spacing).map_err(math_err)?;
    mbitmap::set_initialized(words, bit, initialized).map_err(math_err)
}

///
pub fn is_array_initialized(info: &AccountInfo, tick_spacing: u16, start_index: i32) -> Result<bool> {
    let data = info.try_borrow_data()?;
    require!(data.len() == bitmap_account_len(tick_spacing), AmmError::SeedsMismatch);
    let words: &[u64] = bytemuck::try_cast_slice(&data[WORDS_OFFSET..]).map_err(|_| AmmError::SeedsMismatch)?;
    let bit = mbitmap::array_bit(start_index, tick_spacing).map_err(math_err)?;
    mbitmap::is_initialized(words, bit).map_err(math_err)
}

///
///
pub const fn flip_for(before: u16, after: u16) -> Option<bool> {
    match (before == 0, after == 0) {
        (true, false) => Some(true),
        (false, true) => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_start_after_the_40_byte_header() {
        assert_eq!(WORDS_OFFSET, 48);
    }

    #[test]
    fn flips_only_on_zero_crossings() {
        assert_eq!(flip_for(0, 1), Some(true));
        assert_eq!(flip_for(0, 2), Some(true));
        assert_eq!(flip_for(1, 0), Some(false));
        assert_eq!(flip_for(2, 0), Some(false));
        assert_eq!(flip_for(1, 2), None);
        assert_eq!(flip_for(0, 0), None);
    }
}
