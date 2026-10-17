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
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::token_interface::{self, TransferChecked};
use floor_types::layout::{self as ly, Layout};
use floor_types::{ids, keys, seeds as sd};

use crate::constants::{FLOOR_HOOK_PROGRAM, LAUNCH_AUTHORITY_SEED, LAUNCH_SEED, TRADER_REF_SALE_SEED, TRADER_REF_SEED};
use crate::errors::LaunchError;

///
pub const SALE_REF_ACCOUNTS: usize = 5;

const fn pk(b: [u8; 32]) -> Pubkey {
    Pubkey::new_from_array(b)
}
const LAUNCH_AUTHORITY: (Pubkey, u8) = {
    let (k, b) = anchor_lang::derive_program_address(&[sd::LAUNCH_AUTHORITY], &ids::FLOOR_LAUNCH_ID);
    (pk(k), b)
};
const HOOK_CONFIG_KEY: Pubkey = pk(anchor_lang::derive_program_address(&[sd::HOOK_CONFIG], &ids::FLOOR_HOOK_ID).0);
const SPL_TOKEN_PROGRAM: Pubkey = pk(keys::SPL_TOKEN_PROGRAM_ID);
const TOKEN_2022_PROGRAM: Pubkey = pk(keys::TOKEN_2022_PROGRAM_ID);
///
pub const CREDIT_TRADER_BUCKET_DISC: [u8; 8] = floor_types::disc::hook::CREDIT_TRADER_BUCKET;
const BUCKET_LEN: usize = 8 + ly::TraderBucket::SIZE;

///
pub struct SaleRef<'a, 'info> {
    pub bucket: &'a AccountInfo<'info>,
    pub sale: &'a AccountInfo<'info>,
    pub launch_authority: &'a AccountInfo<'info>,
    pub hook_config: &'a AccountInfo<'info>,
    pub floor_hook: &'a AccountInfo<'info>,
}

fn is_bucket(a: &AccountInfo<'_>) -> bool {
    if *a.owner != FLOOR_HOOK_PROGRAM {
        return false;
    }
    match a.try_borrow_data() {
        Ok(d) => d.len() == BUCKET_LEN && d[..8] == floor_types::disc::account::TRADER_BUCKET,
        Err(_) => false,
    }
}

///
///
///
///
///
///
pub fn split<'a, 'info>(ra: &'a [AccountInfo<'info>]) -> Result<(Option<SaleRef<'a, 'info>>, &'a [AccountInfo<'info>])> {
    let by_shape = ra.len() >= SALE_REF_ACCOUNTS
        && ra[2].key() == LAUNCH_AUTHORITY.0
        && ra[3].key() == HOOK_CONFIG_KEY
        && ra[4].key() == FLOOR_HOOK_PROGRAM;
    let by_bucket = ra.first().is_some_and(is_bucket);
    if !(by_shape || by_bucket) {
        return Ok((None, ra));
    }
    require!(ra.len() >= SALE_REF_ACCOUNTS, LaunchError::TraderRefMismatch);
    Ok((
        Some(SaleRef { bucket: &ra[0], sale: &ra[1], launch_authority: &ra[2], hook_config: &ra[3], floor_hook: &ra[4] }),
        &ra[SALE_REF_ACCOUNTS..],
    ))
}

///
pub fn token_account_frozen(a: &AccountInfo<'_>) -> bool {
    a.try_borrow_data().is_ok_and(|d| d.len() > 108 && d[108] == 2)
}

///
pub fn unopened(a: &AccountInfo<'_>) -> bool {
    *a.owner == anchor_lang::system_program::ID && a.data_is_empty()
}

///
pub fn sale_address(launch: &Pubkey, bump: u8) -> Result<Pubkey> {
    Pubkey::create_program_address(&[TRADER_REF_SALE_SEED, launch.as_ref(), &[bump]], &crate::ID)
        .map_err(|_| error!(LaunchError::TraderRefMismatch))
}

impl<'a, 'info> SaleRef<'a, 'info> {
///
    pub fn check(&self, trader: &Pubkey, quote_mint: &Pubkey, launch: &Pubkey, sale_bump: u8) -> Result<[bool; 3]> {
        require!(self.bucket.is_writable && self.sale.is_writable, LaunchError::TraderRefMismatch);
        require_keys_eq!(self.launch_authority.key(), LAUNCH_AUTHORITY.0, LaunchError::TraderRefMismatch);
        require_keys_eq!(self.hook_config.key(), HOOK_CONFIG_KEY, LaunchError::TraderRefMismatch);
        require_keys_eq!(self.floor_hook.key(), FLOOR_HOOK_PROGRAM, LaunchError::TraderRefMismatch);




        require_keys_eq!(self.sale.key(), sale_address(launch, sale_bump)?, LaunchError::TraderRefMismatch);
        let owner = *self.sale.owner;
        let sale_unopened = unopened(self.sale);
        require!(sale_unopened || owner == SPL_TOKEN_PROGRAM || owner == TOKEN_2022_PROGRAM, LaunchError::TraderRefMismatch);


        let sale_frozen = !sale_unopened && token_account_frozen(self.sale);


        if unopened(self.bucket) {
            let (expected, _) =
                Pubkey::find_program_address(&[TRADER_REF_SEED, trader.as_ref(), quote_mint.as_ref()], &FLOOR_HOOK_PROGRAM);
            require_keys_eq!(self.bucket.key(), expected, LaunchError::TraderRefMismatch);
            return Ok([false; 3]);
        }
        let (tiers, bump) = {
            let d = self.bucket.try_borrow_data()?;
            require!(is_bucket(self.bucket), LaunchError::TraderRefMismatch);
            let b: &ly::TraderBucket = bytemuck::from_bytes(&d[8..BUCKET_LEN]);
            require!(b.trader == trader.to_bytes() && b.quote_mint == quote_mint.to_bytes(), LaunchError::TraderRefMismatch);
            ([b.tier1 != [0u8; 32], b.tier2 != [0u8; 32], b.tier3 != [0u8; 32]], b.bump)
        };
        let expected =
            Pubkey::create_program_address(&[TRADER_REF_SEED, trader.as_ref(), quote_mint.as_ref(), &[bump]], &FLOOR_HOOK_PROGRAM)
                .map_err(|_| error!(LaunchError::TraderRefMismatch))?;
        require_keys_eq!(self.bucket.key(), expected, LaunchError::TraderRefMismatch);
        Ok(if sale_unopened || sale_frozen { [false; 3] } else { tiers })
    }

///
    pub fn credit(&self, s: [u64; 3]) -> Result<()> {
        let mut data = Vec::with_capacity(8 + 24);
        data.extend_from_slice(&CREDIT_TRADER_BUCKET_DISC);
        for x in s {
            data.extend_from_slice(&x.to_le_bytes());
        }
        let ix = Instruction {
            program_id: FLOOR_HOOK_PROGRAM,
            accounts: vec![
                AccountMeta::new_readonly(self.launch_authority.key(), true),
                AccountMeta::new_readonly(self.hook_config.key(), false),
                AccountMeta::new(self.bucket.key(), false),
            ],
            data,
        };
        invoke_signed(
            &ix,
            &[self.launch_authority.clone(), self.hook_config.clone(), self.bucket.clone(), self.floor_hook.clone()],
            &[&[LAUNCH_AUTHORITY_SEED, &[LAUNCH_AUTHORITY.1]]],
        )?;
        Ok(())
    }
}

///
#[allow(clippy::too_many_arguments)]
pub fn move_from_quote_vault<'info>(
    token_program: &AccountInfo<'info>,
    quote_vault: &AccountInfo<'info>,
    quote_mint: &AccountInfo<'info>,
    sale: &AccountInfo<'info>,
    launch: &AccountInfo<'info>,
    base_mint: &Pubkey,
    launch_bump: u8,
    amount: u64,
) -> Result<()> {
    let decimals = {
        let d = quote_mint.try_borrow_data()?;
        *d.get(44).ok_or(LaunchError::BadConfig)?
    };
    let bump = [launch_bump];
    let launch_seeds: &[&[u8]] = &[LAUNCH_SEED, base_mint.as_ref(), &bump];
    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            token_program.key(),
            TransferChecked { from: quote_vault.clone(), mint: quote_mint.clone(), to: sale.clone(), authority: launch.clone() },
            &[launch_seeds],
        ),
        amount,
        decimals,
    )
}

///
pub fn slices(fee: u64, tiers: [bool; 3]) -> ([u64; 3], u64) {
    let s = floor_math::trader_ref_slices(fee, tiers);
    (s, s[0] + s[1] + s[2])
}
