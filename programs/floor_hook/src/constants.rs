//!
//!

use anchor_lang::prelude::*;

pub use floor_types::constants::*;



















///
pub const CARPENTER_AMM_PROGRAM: Pubkey = Pubkey::new_from_array(floor_types::ids::CARPENTER_AMM_ID);
///
pub const FLOOR_LAUNCH_PROGRAM: Pubkey = Pubkey::new_from_array(floor_types::ids::FLOOR_LAUNCH_ID);




///
///
///
///
///
pub const FLOOR_HOLDERS_PROGRAM: Pubkey = Pubkey::new_from_array(floor_types::ids::FLOOR_HOLDERS_ID);
///
///
pub const HOLDERS_AUTHORITY_SEED: &[u8] = floor_types::holders::HOLDERS_AUTHORITY;
///
///
///
pub const LOCK_AUTHORITY_SEED: &[u8] = floor_types::seeds::LOCK_AUTHORITY;

///
///
pub fn holders_authority() -> Pubkey {
    Pubkey::find_program_address(&[HOLDERS_AUTHORITY_SEED], &FLOOR_HOLDERS_PROGRAM).0
}




///
pub const WSOL_MINT: Pubkey = Pubkey::new_from_array(floor_types::keys::WSOL_MINT);

#[constant]
pub const HOOK_CONFIG_SEED: &[u8] = floor_types::seeds::HOOK_CONFIG;
#[constant]
pub const FLOOR_POOL_SEED: &[u8] = floor_types::seeds::FLOOR_POOL;
///
///
#[constant]
pub const STAIR_RENT_SEED: &[u8] = floor_types::seeds::STAIR_RENT;
///
#[constant]
pub const QUOTE_POLICY_SEED: &[u8] = floor_types::seeds::QUOTE_POLICY;
#[constant]
pub const HOOK_AUTHORITY_SEED: &[u8] = floor_types::seeds::HOOK_AUTHORITY;


///
///
///
#[constant]
pub const PLATFORM_TREASURY_SEED: &[u8] = floor_types::seeds::PLATFORM_TREASURY;
///
///
///
///
pub const TEAM_TREASURY_SEED: &[u8] = floor_types::seeds::TEAM_TREASURY;
///
///
///
#[constant]
pub const FLOOR_PLATFORM_POOL_SEED: &[u8] = floor_types::seeds::FLOOR_PLATFORM_POOL;
///
///
///
pub const LAUNCH_CONFIG_SEED: &[u8] = floor_types::seeds::LAUNCH_CONFIG;

///
pub const AMM_CONFIG_SEED: &[u8] = floor_types::seeds::AMM_CONFIG;
pub const APPROVED_HOOK_SEED: &[u8] = floor_types::seeds::APPROVED_HOOK;
pub const MINT_POOLS_SEED: &[u8] = floor_types::seeds::MINT_POOLS;
///
///
pub const QUOTE_MINT_SEED: &[u8] = floor_types::seeds::QUOTE_MINT;
pub const MINT_EXCLUSIVITY_SEED: &[u8] = floor_types::seeds::MINT_EXCLUSIVITY;
pub const EVENT_AUTHORITY_SEED: &[u8] = floor_types::seeds::EVENT_AUTHORITY;
