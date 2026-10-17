//!

use anchor_lang::prelude::*;

pub const TIMELOCK_SEED: &[u8] = b"timelock";
///
pub const AUTHORITY_SEED: &[u8] = b"authority";
pub const OPERATION_SEED: &[u8] = b"operation";

///
pub const INITIAL_DELAY: u64 = 10 * 60;
///
pub const MAX_DELAY: u64 = 30 * 24 * 60 * 60;

///
pub const MAX_INVOKE_ACCOUNTS: usize = 32;
pub const MAX_INVOKE_DATA: usize = 1_024;

///
pub const LOADER_V3: Pubkey = pubkey!("BPFLoaderUpgradeab1e11111111111111111111111");
pub const LOADER_V4: Pubkey = pubkey!("LoaderV411111111111111111111111111111111111");
///
pub const FIXED_CODE_OWNERS: [Pubkey; 3] = [
    pubkey!("NativeLoader1111111111111111111111111111111"),
    pubkey!("BPFLoader2111111111111111111111111111111111"),
    pubkey!("BPFLoader1111111111111111111111111111111111"),
];
///
pub const BUFFER_HEADER: usize = 37;
