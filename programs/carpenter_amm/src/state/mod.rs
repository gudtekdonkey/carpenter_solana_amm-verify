//!
//!
//!
//!
//!
//!
//!

pub mod interface;
pub mod pod;

use anchor_lang::prelude::*;
use carpenter_types::layout::Layout;

pub use interface::{BalanceDelta, HookOpResultV1, PoolSnapshot};
pub use pod::{I128, U128};

///
#[account(zero_copy)]
#[derive(Debug)]
pub struct AmmConfig {
    pub version: u8,
    pub bump: u8,
    pub pad0: [u8; 6],
    pub authority: Pubkey,
    pub pending_authority: Pubkey,
    pub protocol_fee_controller: Pubkey,
///
///
///
    pub pause_guardian: Pubkey,
///
    pub pad1: [u8; 40],
///
///
///
    pub quote_caps_admitted: u8,
///
    pub pad2: [u8; 3],
///
///
    pub protocol_fee: u32,
    pub reserved: [u8; 24],
}

impl AmmConfig {
///
    pub fn current_protocol_fee(&self) -> u32 {
        self.protocol_fee
    }
}

///
#[account(zero_copy)]
#[derive(Debug)]
pub struct ApprovedHook {
    pub version: u8,
    pub bump: u8,
    pub paused: u8,
    pub max_rounds: u8,
    pub callback_accounts: u8,
    pub allow_public_pools: u8,
    pub flags: u16,
    pub hook_program: Pubkey,
    pub hook_authority: Pubkey,
    pub hook_authority_bump: u8,
    pub pad0: [u8; 7],
    pub programdata_slot: u64,
    pub approved_slot: u64,
    pub name: [u8; 32],
    pub allow_exclusivity: u8,
///
///
///
    pub base_transfer_hook: Pubkey,
///
///
///
    pub allow_hook_quotes: u8,
    pub reserved: [u8; 22],
///
///
    pub guardian_paused_until: i64,
}

///
#[account(zero_copy)]
#[derive(Debug)]
pub struct MintExclusivity {
    pub mint: Pubkey,
    pub hook_program: Pubkey,
    pub bump: u8,
///
///
    pub has_protocol_fee: u8,
    pub pad0: [u8; 2],
///
///
    pub protocol_fee: u32,
}

///
#[account(zero_copy)]
#[derive(Debug)]
pub struct MintPoolCount {
    pub mint: Pubkey,
    pub count: u64,
    pub bump: u8,
    pub reserved: [u8; 15],
}

///
///
#[account(zero_copy)]
#[derive(Debug)]
pub struct QuoteMint {
    pub mint: Pubkey,
    pub authority_digest: [u8; 32],
    pub listed_slot: u64,
    pub retired_slot: u64,
    pub state: u8,
    pub bump: u8,
    pub decimals: u8,
    pub caps: u8,
    pub pad0: [u8; 4],
///
///
    pub hook_program: Pubkey,
}

///
#[account(zero_copy)]
#[derive(Debug)]
pub struct Pool {
    pub version: u8,
    pub bump: u8,
    pub hook_flags: u16,
    pub tick_spacing: u16,
    pub hook_max_rounds: u8,
    pub hook_callback_accounts: u8,
    pub fee: u32,
    pub lp_fee: u32,
    pub protocol_fee: u32,
    pub tick_current: i32,
    pub mint0: Pubkey,
    pub mint1: Pubkey,
    pub hook: Pubkey,
    pub hook_authority: Pubkey,
    pub vault0: Pubkey,
    pub vault1: Pubkey,
    pub token_program0: Pubkey,
    pub token_program1: Pubkey,
    pub sqrt_price_x64: U128,
    pub liquidity: U128,
    pub fee_growth_global0_x64: U128,
    pub fee_growth_global1_x64: U128,
    pub protocol_fees_owed0: u64,
    pub protocol_fees_owed1: u64,
    pub hook_claims0: u64,
    pub hook_claims1: u64,
    pub volume0: U128,
    pub volume1: U128,
    pub lp_fees0: U128,
    pub lp_fees1: U128,
    pub hook_fees0: U128,
    pub hook_fees1: U128,
    pub swap_count: u64,
    pub created_slot: u64,
    pub decimals0: u8,
    pub decimals1: u8,
    pub bitmap_bump: u8,
    pub vault0_bump: u8,
    pub vault1_bump: u8,
    pub pad0: [u8; 3],
    pub reserved0: u64,
///
///
    pub paused: u8,
///
///
    pub base_transfer_hook: Pubkey,
///
    pub base_hook_side: u8,
///
///
///
///
    pub protocol_fee_currency: u8,
    pub reserved: [u8; 85],
///
///
///
    pub guardian_paused_until: i64,
}

///
///
///
///
#[account(zero_copy)]
#[derive(Debug)]
pub struct TickArrayBitmap {
    pub pool: Pubkey,
    pub tick_spacing: u16,
    pub pad0: u16,
    pub arrays_per_side: u32,
}

///
#[zero_copy]
#[derive(Debug, Default)]
pub struct Tick {
    pub liquidity_gross: U128,
    pub liquidity_net: I128,
    pub fee_growth_outside0_x64: U128,
    pub fee_growth_outside1_x64: U128,
}

///
#[account(zero_copy)]
#[derive(Debug)]
pub struct TickArray {
    pub pool: Pubkey,
    pub start_index: i32,
    pub initialized_tick_count: u16,
    pub pad0: u16,
    pub ticks: [Tick; 60],
}

///
///
///
///
///
///
#[account(zero_copy)]
#[derive(Debug)]
pub struct Position {
    pub version: u8,
    pub bump: u8,
    pub pad0: [u8; 6],
    pub pool: Pubkey,
    pub owner: Pubkey,
    pub salt: u64,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub liquidity: U128,
    pub fee_growth_inside0_last_x64: U128,
    pub fee_growth_inside1_last_x64: U128,
    pub position_id: Pubkey,
    pub reserved: [u8; 32],
}

///
#[account(zero_copy)]
#[derive(Debug)]
pub struct PoolIndexEntry {
    pub pool: Pubkey,
    pub created_slot: u64,
    pub bump: u8,
    pub reserved: [u8; 7],
}

///
///
///
///
///
#[account(zero_copy)]
#[derive(Debug)]
pub struct ProtocolTreasury {
    pub version: u8,
    pub bump: u8,
    pub pad0: [u8; 6],
    pub governance: Pubkey,
    pub pending_governance: Pubkey,
    pub reserved: [u8; 32],
}

macro_rules! twin {
    ($($t:ident => $ft:ty),* $(,)?) => {$(
        impl $t {
///
            pub const LEN: usize = 8 + <$ft as Layout>::SIZE;
        }
        const _: () = assert!(core::mem::size_of::<$t>() == <$ft as Layout>::SIZE);
    )*};
}

twin! {
    AmmConfig => carpenter_types::layout::AmmConfig,
    ApprovedHook => carpenter_types::layout::ApprovedHook,
    MintExclusivity => carpenter_types::layout::MintExclusivity,
    MintPoolCount => carpenter_types::layout::MintPoolCount,
    QuoteMint => carpenter_types::layout::QuoteMint,
    Pool => carpenter_types::layout::Pool,
    TickArrayBitmap => carpenter_types::layout::TickArrayBitmapHeader,
    TickArray => carpenter_types::layout::TickArray,
    Position => carpenter_types::layout::Position,
    PoolIndexEntry => carpenter_types::layout::PoolIndexEntry,
    ProtocolTreasury => carpenter_types::layout::ProtocolTreasury,
}

const _: () = assert!(core::mem::size_of::<Tick>() == core::mem::size_of::<carpenter_types::layout::Tick>());
