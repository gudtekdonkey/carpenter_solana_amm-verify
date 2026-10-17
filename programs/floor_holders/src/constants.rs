//!
//!
//!
//!
//!
//!
//!
//!

use anchor_lang::prelude::*;

pub use floor_types::holders::{
    CLAIMS_ADDRS_HOOKED, CLAIMS_ADDRS_REFERRED, CLAIMS_TABLE_MAX, FLOOR_POOL_ACCT_HOLDER_OWED, FLOOR_POOL_ACCT_HOLDER_PAID,
};



///
pub const FLOOR_HOOK_PROGRAM: Pubkey = Pubkey::new_from_array(floor_types::ids::FLOOR_HOOK_ID);
///
pub const FLOOR_LAUNCH_PROGRAM: Pubkey = Pubkey::new_from_array(floor_types::ids::FLOOR_LAUNCH_ID);
///
pub const CARPENTER_AMM_PROGRAM: Pubkey = Pubkey::new_from_array(floor_types::ids::CARPENTER_AMM_ID);
///
pub const TOKEN_2022_PROGRAM: Pubkey = Pubkey::new_from_array(floor_types::keys::TOKEN_2022_PROGRAM_ID);
///
pub const ALT_PROGRAM: Pubkey = Pubkey::new_from_array(floor_types::holders::ADDRESS_LOOKUP_TABLE_PROGRAM_ID);



#[constant]
pub const HOLDERS_AUTHORITY_SEED: &[u8] = floor_types::holders::HOLDERS_AUTHORITY;
#[constant]
pub const CLAIMS_ALT_AUTHORITY_SEED: &[u8] = floor_types::holders::CLAIMS_ALT_AUTHORITY;
#[constant]
pub const CLAIMS_REGISTRY_SEED: &[u8] = floor_types::holders::CLAIMS_REGISTRY;
#[constant]
pub const HOLDER_BOOK_SEED: &[u8] = floor_types::holders::HOLDER_BOOK;
///
///
#[constant]
pub const EXTRA_ACCOUNT_METAS_SEED: &[u8] = floor_types::holders::EXTRA_ACCOUNT_METAS;
#[constant]
pub const HOLDER_RECORD_SEED: &[u8] = floor_types::holders::HOLDER_RECORD;
///
pub const EVENT_AUTHORITY_SEED: &[u8] = floor_types::seeds::EVENT_AUTHORITY;
///
pub const HOOK_AUTHORITY_SEED: &[u8] = floor_types::seeds::HOOK_AUTHORITY;





///
///
pub const FLOOR_POOL_SEED: &[u8] = floor_types::seeds::FLOOR_POOL;
pub const AMM_POOL_SEED: &[u8] = floor_types::seeds::POOL;
pub const AMM_VAULT_SEED: &[u8] = floor_types::seeds::VAULT;



///
///
///
///
///
///
pub const EXECUTE_DISCRIMINATOR: [u8; 8] = floor_types::disc::holders::EXECUTE;

///
///
pub const PAY_HOLDER_DISCRIMINATOR: [u8; 8] = floor_types::disc::hook::PAY_HOLDER;

///
pub const ALT_IX_CREATE: u32 = 0;
pub const ALT_IX_EXTEND: u32 = 2;

///
///
pub const EXECUTE_ACCOUNTS: usize = floor_types::holders::EXECUTE_ACCOUNTS;

///
pub const BOOK_VERSION: u8 = 1;
pub const RECORD_VERSION: u8 = 1;
pub const REGISTRY_VERSION: u8 = 1;

const _: () = assert!(EXECUTE_ACCOUNTS == 9);
const _: () = assert!(CLAIMS_TABLE_MAX == 256);
const _: () = assert!(CLAIMS_ADDRS_HOOKED == 5 && CLAIMS_ADDRS_REFERRED == 3);
