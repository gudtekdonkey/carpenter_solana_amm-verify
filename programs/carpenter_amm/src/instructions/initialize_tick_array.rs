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
use carpenter_types::layout::{tick_array_len_ok, TICK_ARRAY_LEN, TICK_ARRAY_RENT_OWNER_OFFSET};

use crate::constants::TICK_ARRAY_SEED;
use crate::amm_core::ticks::math_err;
use crate::amm_core::vaults::create_pda;
use crate::errors::AmmError;
use crate::state::{Pool, TickArray};

#[derive(Accounts)]
#[instruction(start_index: i32)]
pub struct InitializeTickArray<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub pool: AccountLoader<'info, Pool>,
///
///
    #[account(
        mut,
        seeds = [TICK_ARRAY_SEED, pool.key().as_ref(), &start_index.to_le_bytes()],
        bump,
    )]
    pub tick_array: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
#[derive(AnchorSerialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RentOwnerArg {
    pub rent_owner: Pubkey,
}

impl AnchorDeserialize for RentOwnerArg {
    fn deserialize_reader<R: borsh::io::Read>(reader: &mut R) -> borsh::io::Result<Self> {
        let mut key = [0u8; 32];
        let mut got = 0usize;
        while got < key.len() {
            match reader.read(&mut key[got..])? {
                0 => break,
                n => got += n,
            }
        }
        match got {
            0 => Ok(Self { rent_owner: Pubkey::default() }),
            32 => Ok(Self { rent_owner: Pubkey::new_from_array(key) }),
            _ => Err(borsh::io::Error::new(borsh::io::ErrorKind::UnexpectedEof, "rent_owner: 1..=31 bytes")),
        }
    }
}

pub fn handler(ctx: Context<InitializeTickArray>, start_index: i32, rent_owner: Pubkey) -> Result<()> {
    let spacing = ctx.accounts.pool.load()?.tick_spacing;
    require!(valid_array_start(start_index, spacing)?, AmmError::TickArrayStartInvalid);
    let payer = ctx.accounts.payer.to_account_info();
    ensure_tick_array(
        Some(&payer),
        &rent_owner,
        &ctx.accounts.tick_array.to_account_info(),
        &ctx.accounts.pool.key(),
        start_index,
        ctx.bumps.tick_array,
        true,
    )
}

///
///
///
///
#[inline(never)]
pub fn ensure_tick_array<'info>(
    payer: Option<&AccountInfo<'info>>,
    rent_owner: &Pubkey,
    info: &AccountInfo<'info>,
    pool: &Pubkey,
    start_index: i32,
    bump: u8,
    one_cpi: bool,
) -> Result<()> {
    if *info.owner != crate::ID {
        let payer = payer.ok_or(ErrorCode::AccountNotEnoughKeys)?;
        let start = start_index.to_le_bytes();
        let seeds: &[&[u8]] = &[TICK_ARRAY_SEED, pool.as_ref(), &start, &[bump]];




        if one_cpi && info.lamports() == 0 {
            require_keys_eq!(*info.owner, anchor_lang::system_program::ID, AmmError::SeedsMismatch);
            anchor_lang::system_program::create_account(
                CpiContext::new_with_signer(
                    anchor_lang::system_program::ID,
                    anchor_lang::system_program::CreateAccount { from: payer.clone(), to: info.clone() },
                    &[seeds],
                ),
                Rent::get()?.minimum_balance(TICK_ARRAY_LEN),
                TICK_ARRAY_LEN as u64,
                &crate::ID,
            )?;
        } else {
            create_pda(payer, info, &crate::ID, TICK_ARRAY_LEN, seeds)?;
        }
        let mut data = info.try_borrow_mut_data()?;
        data[..8].copy_from_slice(TickArray::DISCRIMINATOR);
        {
            let a: &mut TickArray = bytemuck::from_bytes_mut(&mut data[8..TickArray::LEN]);
            a.pool = *pool;
            a.start_index = start_index;
            a.initialized_tick_count = 0;
        }
        let owner = resolve_rent_owner(*rent_owner, payer.key);
        data[TICK_ARRAY_RENT_OWNER_OFFSET..TICK_ARRAY_LEN].copy_from_slice(owner.as_ref());
        return Ok(());
    }
    let data = info.try_borrow_data()?;
    require!(tick_array_len_ok(data.len()) && &data[..8] == TickArray::DISCRIMINATOR, AmmError::SeedsMismatch);
    let a: &TickArray = bytemuck::from_bytes(&data[8..TickArray::LEN]);
    require_keys_eq!(a.pool, *pool, AmmError::SeedsMismatch);
    require!(a.start_index == start_index, AmmError::SeedsMismatch);
    Ok(())
}

///
pub fn valid_array_start(start_index: i32, tick_spacing: u16) -> Result<bool> {
    let per = carpenter_clmm_math::bitmap::ticks_in_array(tick_spacing).map_err(math_err)?;
    Ok(start_index % per == 0
        && carpenter_clmm_math::bitmap::is_valid_start_index(start_index, tick_spacing).map_err(math_err)?)
}

///
///
///
pub fn resolve_rent_owner(rent_owner: Pubkey, payer: &Pubkey) -> Pubkey {
    if rent_owner == Pubkey::default() {
        *payer
    } else {
        rent_owner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_zero_rent_owner_is_the_payer_and_any_other_key_is_itself() {
        let payer = Pubkey::new_from_array([1; 32]);
        let named = Pubkey::new_from_array([2; 32]);
        assert_eq!(resolve_rent_owner(Pubkey::default(), &payer), payer);
        assert_eq!(resolve_rent_owner(named, &payer), named);
        assert_eq!(resolve_rent_owner(payer, &payer), payer);
    }

    #[test]
    fn the_rent_owner_arg_is_optional_on_the_wire_and_refuses_a_partial_key() {
        let k = Pubkey::new_from_array([7; 32]);
        let mut none: &[u8] = &[];
        assert_eq!(RentOwnerArg::deserialize(&mut none).unwrap().rent_owner, Pubkey::default(), "0 bytes: the zero key");
        let bytes = k.to_bytes();
        let mut full: &[u8] = &bytes;
        assert_eq!(RentOwnerArg::deserialize(&mut full).unwrap().rent_owner, k, "32 bytes: that key");
        for n in 1..32 {
            let mut part: &[u8] = &bytes[..n];
            assert!(RentOwnerArg::deserialize(&mut part).is_err(), "{n} bytes: refused, never truncated");
        }
        let mut longer = bytes.to_vec();
        longer.extend_from_slice(&[9; 5]);
        let mut long: &[u8] = &longer;
        assert_eq!(RentOwnerArg::deserialize(&mut long).unwrap().rent_owner, k, "bytes after the key are left unread");
        assert_eq!(long, &[9u8; 5][..]);
        let mut out = Vec::new();
        RentOwnerArg { rent_owner: k }.serialize(&mut out).unwrap();
        assert_eq!(out, bytes.to_vec(), "serialises as the bare 32-byte key");
    }

    #[test]
    fn the_tail_sits_after_the_body_and_ends_the_v2_account() {
        assert_eq!(TICK_ARRAY_RENT_OWNER_OFFSET, TickArray::LEN, "the body is unchanged: the tail starts where it ends");
        assert_eq!(TICK_ARRAY_LEN, TickArray::LEN + 32);
    }
}
