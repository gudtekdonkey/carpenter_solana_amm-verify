//!
//!

use anchor_lang::prelude::*;

pub use carpenter_types::constants::*;
pub use carpenter_types::hook::{HOOK_INTERFACE_VERSION, MAX_ACTIONS_PER_RECORD, MAX_HOOK_DATA, MAX_ROUNDS_CAP};

#[constant]
pub const AMM_CONFIG_SEED: &[u8] = carpenter_types::seeds::AMM_CONFIG;
#[constant]
pub const APPROVED_HOOK_SEED: &[u8] = carpenter_types::seeds::APPROVED_HOOK;
#[constant]
pub const MINT_EXCLUSIVITY_SEED: &[u8] = carpenter_types::seeds::MINT_EXCLUSIVITY;
#[constant]
pub const MINT_POOLS_SEED: &[u8] = carpenter_types::seeds::MINT_POOLS;
///
#[constant]
pub const QUOTE_MINT_SEED: &[u8] = carpenter_types::seeds::QUOTE_MINT;
#[constant]
pub const POOL_SEED: &[u8] = carpenter_types::seeds::POOL;
#[constant]
pub const BITMAP_SEED: &[u8] = carpenter_types::seeds::BITMAP;
#[constant]
pub const TICK_ARRAY_SEED: &[u8] = carpenter_types::seeds::TICK_ARRAY;
#[constant]
pub const POSITION_SEED: &[u8] = carpenter_types::seeds::POSITION;
#[constant]
pub const POOL_INDEX_SEED: &[u8] = carpenter_types::seeds::POOL_INDEX;
#[constant]
pub const VAULT_SEED: &[u8] = carpenter_types::seeds::VAULT;
///
///
#[constant]
pub const PROTOCOL_TREASURY_SEED: &[u8] = carpenter_types::seeds::PROTOCOL_TREASURY;

///
pub const HOOK_AUTHORITY_SEED: &[u8] = carpenter_types::seeds::HOOK_AUTHORITY;
///
pub const HOOK_CONFIG_SEED: &[u8] = carpenter_types::seeds::HOOK_CONFIG;
