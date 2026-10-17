//!
//!
//!
//!
//!

use anchor_lang::prelude::*;

#[event]
pub struct ConfigInitialized {
    pub config: Pubkey,
    pub authority: Pubkey,
}

#[event]
pub struct ConfigAuthorityChanged {
    pub config: Pubkey,
    pub previous: Pubkey,
///
    pub authority: Pubkey,
///
    pub accepted: bool,
}

#[event]
pub struct HookApproved {
    pub hook_program: Pubkey,
    pub hook_authority: Pubkey,
    pub flags: u16,
    pub max_rounds: u8,
    pub callback_accounts: u8,
    pub allow_public_pools: bool,
    pub allow_exclusivity: bool,
    pub programdata_slot: u64,
    pub name: [u8; 32],
}

#[event]
pub struct HookPaused {
    pub hook_program: Pubkey,
    pub paused: bool,
}

///
///
///
#[event]
pub struct HookReapproved {
    pub hook_program: Pubkey,
    pub previous_programdata_slot: u64,
    pub programdata_slot: u64,
    pub approved_slot: u64,
}

///
///
///
#[event]
pub struct QuoteMintListed {
    pub mint: Pubkey,
    pub caps: u8,
    pub decimals: u8,
    pub authority_digest: [u8; 32],
}

///
///
#[event]
pub struct QuoteMintRetired {
    pub mint: Pubkey,
    pub retired_slot: u64,
}

///
///
///
///
///
///
#[event]
pub struct QuoteMintRelisted {
    pub mint: Pubkey,
    pub caps: u8,
    pub decimals: u8,
    pub authority_digest: [u8; 32],
    pub retired_slot: u64,
}

///
///
#[event]
pub struct QuoteMintDelisted {
    pub mint: Pubkey,
    pub was_retired: bool,
    pub listed_slot: u64,
    pub rent_refunded: u64,
}

///
///
///
///
///
#[event]
pub struct QuoteCapsAdmittedSet {
    pub previous: u8,
    pub admitted: u8,
}

///
#[event]
pub struct PoolPauseSet {
    pub pool: Pubkey,
    pub paused: bool,
}

#[event]
pub struct PoolInitialized {
    pub pool: Pubkey,
    pub mint0: Pubkey,
    pub mint1: Pubkey,
    pub fee: u32,
    pub tick_spacing: u16,
    pub hook: Pubkey,
    pub sqrt_price_x64: u128,
    pub tick: i32,
}

#[event]
pub struct LiquidityModified {
    pub pool: Pubkey,
    pub owner: Pubkey,
    pub salt: u64,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub liquidity_delta: i128,
    pub amount0: i128,
    pub amount1: i128,
    pub fees0: u64,
    pub fees1: u64,
///
///
    pub position: Pubkey,
}

///
#[event]
pub struct FeesTruncated {
    pub position: Pubkey,
    pub currency: u8,
}

#[event]
pub struct PositionClosed {
    pub position: Pubkey,
    pub owner: Pubkey,
}

///
#[event]
pub struct PositionTransferred {
    pub position: Pubkey,
    pub pool: Pubkey,
    pub from: Pubkey,
    pub to: Pubkey,
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
#[event]
pub struct Swapped {
    pub pool: Pubkey,
    pub trader: Pubkey,
    pub zero_for_one: bool,
    pub amount_in: u64,
    pub amount_out: u64,
    pub hook_delta0: i128,
    pub hook_delta1: i128,
    pub lp_fee_total: u64,
    pub fee_override: Option<u32>,
    pub pre_sqrt_price_x64: u128,
    pub pre_tick: i32,
    pub post_sqrt_price_x64: u128,
    pub post_tick: i32,
    pub signals: u8,
    pub bid_replaced: bool,
    pub absorbed: u64,
    pub nudge_spent: u64,
///
///
///
///
///
///
///
    pub protocol_fee_total: u64,
///
    pub protocol_fee_currency: u8,
///
    pub fee_split0: [u64; 5],
///
    pub fee_split1: [u64; 5],
///
///
///
///
///
///
///
///
///
    pub walk_sqrt_price_x64: u128,
///
    pub walk_tick: i32,
}

#[event]
pub struct Donated {
    pub pool: Pubkey,
    pub donor: Pubkey,
    pub amount0: u64,
    pub amount1: u64,
}

#[event]
pub struct HookDeposited {
    pub pool: Pubkey,
    pub currency: u8,
///
    pub credited: u64,
}

#[event]
pub struct HookWithdrew {
    pub pool: Pubkey,
    pub currency: u8,
    pub amount: u64,
    pub destination: Pubkey,
}

#[event]
pub struct MintExclusivityRegistered {
    pub mint: Pubkey,
    pub hook_program: Pubkey,
///
///
    pub protocol_fee: u32,
}

#[event]
pub struct MintExclusivityRevoked {
    pub mint: Pubkey,
    pub hook_program: Pubkey,
}

#[event]
pub struct LpFeeUpdated {
    pub pool: Pubkey,
    pub lp_fee: u32,
}

///
///
///
#[event]
pub struct BaseTransferHookSet {
    pub hook_program: Pubkey,
    pub transfer_hook_program: Pubkey,
}

///
///
///
///
#[event]
pub struct TickArrayClosed {
    pub pool: Pubkey,
    pub tick_array: Pubkey,
    pub start_index: i32,
    pub rent_owner: Pubkey,
    pub lamports: u64,
}



///
///
///
///
#[event]
pub struct ProtocolTreasuryGovernanceChanged {
    pub treasury: Pubkey,
    pub previous: Pubkey,
    pub governance: Pubkey,
    pub accepted: bool,
}

///
///
///
#[event]
pub struct ProtocolFeesSwept {
    pub pool: Pubkey,
    pub currency: u8,
    pub mint: Pubkey,
    pub destination: Pubkey,
    pub amount: u64,
///
    pub sweeper: Pubkey,
}

///
///
#[event]
pub struct ProtocolFeesWithdrawn {
    pub mint: Pubkey,
    pub source: Pubkey,
    pub destination: Pubkey,
    pub governance: Pubkey,
    pub amount: u64,
    pub remaining: u64,
}

///
///
///
///
///
#[event]
pub struct ProtocolFeeSet {
    pub previous: u32,
    pub protocol_fee: u32,
}

///
///
#[event]
pub struct PauseGuardianSet {
    pub previous: Pubkey,
    pub guardian: Pubkey,
}

///
///
#[event]
pub struct GuardianPoolPaused {
    pub pool: Pubkey,
    pub guardian: Pubkey,
    pub paused_until: i64,
}

///
///
#[event]
pub struct GuardianHookPaused {
    pub hook_program: Pubkey,
    pub guardian: Pubkey,
    pub paused_until: i64,
}

///
///
#[event]
pub struct HookQuotesSet {
    pub hook_program: Pubkey,
    pub previous: bool,
    pub allowed: bool,
}

///
///
#[event]
pub struct QuoteMintHookListed {
    pub mint: Pubkey,
    pub hook_program: Pubkey,
    pub caps: u8,
    pub decimals: u8,
}
