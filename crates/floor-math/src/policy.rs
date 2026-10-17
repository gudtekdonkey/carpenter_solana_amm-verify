//!
//!

///
///
pub const MIN_BID_LAMPORTS: u64 = 50_000_000;
///
pub const PLATFORM_BPS: u16 = 250;






///
pub const TEAM_SALE_FEE_BPS_DEFAULT: u16 = 100;
///
pub const TEAM_SALE_FEE_BPS_MAX: u16 = 100;
///
pub const TEAM_POOL_FEE_BPS_DEFAULT: u16 = 25;
///
pub const TEAM_POOL_FEE_BPS_MAX: u16 = 50;
///
///
///
///
///
pub const MAX_SKIM_FAR_BPS: u16 = 1000;
///
///
pub const MAX_CREATOR_BPS: u16 = 500;
///
///
///
///
///
pub const MAX_HOLDER_BPS_EVM: u16 = 1000;
///
///
///
pub const MAX_HOLDER_BPS: u16 = 1000;
const _: () = assert!(MAX_HOLDER_BPS == MAX_HOLDER_BPS_EVM, "the Solana holder cap IS the EVM's since 2026-09-22");






