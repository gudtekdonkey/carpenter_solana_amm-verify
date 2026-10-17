//!
//!
//!
//!
//!

use anchor_lang::prelude::*;

pub use floor_types::constants::*;

pub const CARPENTER_AMM_PROGRAM: Pubkey = Pubkey::new_from_array(floor_types::ids::CARPENTER_AMM_ID);
pub const FLOOR_HOOK_PROGRAM: Pubkey = Pubkey::new_from_array(floor_types::ids::FLOOR_HOOK_ID);
///
///
///
///
///
pub const FLOOR_HOLDERS_PROGRAM: Pubkey = Pubkey::new_from_array(floor_types::holders::FLOOR_HOLDERS_ID);
pub const WSOL_MINT: Pubkey = Pubkey::new_from_array(floor_types::keys::WSOL_MINT);

///
///
///
pub use floor_types::disc::holders as holders_disc;

#[constant]
pub const LAUNCH_CONFIG_SEED: &[u8] = floor_types::seeds::LAUNCH_CONFIG;
#[constant]
pub const REGISTRY_ENTRY_SEED: &[u8] = floor_types::seeds::REGISTRY_ENTRY;
#[constant]
pub const LAUNCH_SEED: &[u8] = floor_types::seeds::LAUNCH;
#[constant]
pub const SALE_VAULT_SEED: &[u8] = floor_types::seeds::SALE_VAULT;
#[constant]
pub const GRAD_RENT_SEED: &[u8] = floor_types::seeds::GRAD_RENT;
#[constant]
pub const BUYER_SEED: &[u8] = floor_types::seeds::BUYER;
#[constant]
pub const MINT_SEED: &[u8] = floor_types::seeds::MINT;
#[constant]
pub const MINT_AUTHORITY_SEED: &[u8] = floor_types::seeds::MINT_AUTHORITY;
#[constant]
pub const LAUNCH_AUTHORITY_SEED: &[u8] = floor_types::seeds::LAUNCH_AUTHORITY;

///
pub const HOOK_CONFIG_SEED: &[u8] = floor_types::seeds::HOOK_CONFIG;
pub const HOOK_AUTHORITY_SEED: &[u8] = floor_types::seeds::HOOK_AUTHORITY;
///
///
///
pub const PLATFORM_TREASURY_SEED: &[u8] = floor_types::seeds::PLATFORM_TREASURY;
///
///
///
pub const TEAM_TREASURY_SEED: &[u8] = floor_types::seeds::TEAM_TREASURY;
///
pub const FLOOR_PLATFORM_POOL_SEED: &[u8] = floor_types::seeds::FLOOR_PLATFORM_POOL;
pub const AMM_CONFIG_SEED: &[u8] = floor_types::seeds::AMM_CONFIG;
pub const APPROVED_HOOK_SEED: &[u8] = floor_types::seeds::APPROVED_HOOK;
pub const MINT_POOLS_SEED: &[u8] = floor_types::seeds::MINT_POOLS;
pub const MINT_EXCLUSIVITY_SEED: &[u8] = floor_types::seeds::MINT_EXCLUSIVITY;
pub const EVENT_AUTHORITY_SEED: &[u8] = floor_types::seeds::EVENT_AUTHORITY;
///
///
pub const HOLDER_BOOK_SEED: &[u8] = floor_types::holders::HOLDER_BOOK;
pub const HOLDER_RECORD_SEED: &[u8] = floor_types::holders::HOLDER_RECORD;



#[constant]
pub const GENESIS_SALE_SEED: &[u8] = floor_types::seeds::GENESIS_SALE;
#[constant]
pub const GENESIS_CONTRIBUTION_SEED: &[u8] = floor_types::seeds::GENESIS_CONTRIBUTION;
///
pub const QUOTE_WALK_SEED: &[u8] = floor_types::seeds::QUOTE_WALK;
///
pub const PRESET_REGISTRY_SEED: &[u8] = floor_types::constants::PRESET_REGISTRY_SEED;
///
pub const PRESET_REGISTRY_KEY: Pubkey = Pubkey::new_from_array(
    anchor_lang::derive_program_address(&[floor_types::constants::PRESET_REGISTRY_SEED], &floor_types::ids::FLOOR_LAUNCH_ID).0,
);
///
pub const TRADER_LINK_SEED: &[u8] = floor_types::seeds::TRADER_LINK;
///
pub const TRADER_REF_SEED: &[u8] = floor_types::seeds::TRADER_REF;
///
pub const TRADER_REF_SALE_SEED: &[u8] = floor_types::seeds::TRADER_REF_SALE;
///
pub const TRADER_REF_VAULT_SEED: &[u8] = floor_types::seeds::TRADER_REF_VAULT;
///
pub const FLOOR_VESTING_PROGRAM: Pubkey = Pubkey::new_from_array(floor_types::ids::FLOOR_VESTING_ID);
