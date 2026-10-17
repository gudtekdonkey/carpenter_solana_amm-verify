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

pub use carpenter_types::layout::*;

use crate::constants::MAX_METADATA_BASE_LEN;
use crate::disc::account as disc;
use crate::pod::U128;
use bytemuck::{Pod, Zeroable};
use carpenter_types::layout;





///
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct HookConfig {

    pub version: u8,
    pub bump: u8,
///
    pub max_rounds: u8,
///
    pub callback_accounts: u8,
///
    pub flags: u16,
    pub authority_bump: u8,
    pub pad0: u8,

///
///
///
///
///
    pub governance: [u8; 32],
///
///
    pub platform_destination: [u8; 32],
    pub launch_program: [u8; 32],
///
    pub launch_authority: [u8; 32],
    pub hook_authority: [u8; 32],
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
    pub platform_owner: [u8; 32],
///
///
    pub pending_governance: [u8; 32],
}
layout!(HookConfig, name = "HookConfig", size = 232, disc = disc::HOOK_CONFIG, {
    version: u8 = 0, bump: u8 = 1, max_rounds: u8 = 2, callback_accounts: u8 = 3, flags: u16 = 4,
    authority_bump: u8 = 6, pad0: u8 = 7, governance: [u8; 32] = 8, platform_destination: [u8; 32] = 40,
    launch_program: [u8; 32] = 72, launch_authority: [u8; 32] = 104, hook_authority: [u8; 32] = 136,
    platform_owner: [u8; 32] = 168, pending_governance: [u8; 32] = 200,
});

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
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct QuotePolicy {
    pub version: u8,
    pub bump: u8,
///
///
///
///
    pub decimals: u8,
///
///
///
    pub retired: u8,
    pub pad0: [u8; 4],
///
    pub quote_mint: [u8; 32],
///
///
    pub token_program: [u8; 32],
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
    pub min_bid_floor: u64,
    pub min_bid_ceil: u64,
///
///
    pub max_bounty_ceil: u64,
///
///
///
///
///
///
    pub min_bid: u64,
///
///
    pub max_bounty: u64,
    pub reserved: [u8; 56],
}
layout!(QuotePolicy, name = "QuotePolicy", size = 168, disc = disc::QUOTE_POLICY, {
    version: u8 = 0, bump: u8 = 1, decimals: u8 = 2, retired: u8 = 3, pad0: [u8; 4] = 4,
    quote_mint: [u8; 32] = 8, token_program: [u8; 32] = 40, min_bid_floor: u64 = 72,
    min_bid_ceil: u64 = 80, max_bounty_ceil: u64 = 88, min_bid: u64 = 96, max_bounty: u64 = 104,
    reserved: [u8; 56] = 112,
});

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
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct PlatformTreasury {
    pub version: u8,
    pub bump: u8,
///
///
///
///
///
///
///
    pub allow_all_quotes: u8,
    pub pad0: [u8; 5],
///
///
///
///
    pub governance: [u8; 32],
///
///
    pub pending_governance: [u8; 32],
    pub reserved: [u8; 32],
}
layout!(PlatformTreasury, name = "PlatformTreasury", size = 104, disc = disc::PLATFORM_TREASURY, {
    version: u8 = 0, bump: u8 = 1, allow_all_quotes: u8 = 2, pad0: [u8; 5] = 3, governance: [u8; 32] = 8,
    pending_governance: [u8; 32] = 40, reserved: [u8; 32] = 72,
});

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
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct FloorPlatformPool {
    pub version: u8,
    pub bump: u8,
    pub pad0: [u8; 6],
///
    pub floor_pool: [u8; 32],
///
    pub amm_pool: [u8; 32],
///
    pub base_mint: [u8; 32],
///
///
    pub quote_mint: [u8; 32],
///
///
    pub received: u64,
///
    pub credits: u64,
///
    pub designated_slot: u64,
    pub reserved: [u8; 64],
}
layout!(FloorPlatformPool, name = "FloorPlatformPool", size = 224, disc = disc::FLOOR_PLATFORM_POOL, {
    version: u8 = 0, bump: u8 = 1, pad0: [u8; 6] = 2, floor_pool: [u8; 32] = 8, amm_pool: [u8; 32] = 40,
    base_mint: [u8; 32] = 72, quote_mint: [u8; 32] = 104, received: u64 = 136, credits: u64 = 144,
    designated_slot: u64 = 152, reserved: [u8; 64] = 160,
});

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
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct StairState {
///
    pub version: u8,
    pub bump: u8,
///
    pub steps: u8,
///
///
///
    pub placed: u8,
///
///
    pub first_open: u8,
///
    pub tail_placed: u8,
    pub growth_bps: u16,
///
    pub open_edge: i32,
    pub tail_lower: i32,
    pub tail_upper: i32,
    pub cushion_lower: i32,
    pub cushion_upper: i32,
    pub relist_lower: i32,
    pub relist_upper: i32,
///
///
///
///
    pub relist_cost_tick: i32,
///
    pub stair_total: u64,
///
    pub tail_total: u64,
///
///
    pub collected_total: u64,
    pub to_pol_total: u64,
    pub to_floor_total: u64,
///
    pub cushion_quote: u64,
    pub tail_liq: U128,
    pub cushion_liq: U128,
    pub relist_liq: U128,
///
///
///
///
    pub step_lower: [i32; 16],
    pub step_upper: [i32; 16],
    pub step_liq: [U128; 16],
///
///
///
    pub reserve_lower: i32,
    pub reserve_upper: i32,
    pub reserve_liq: U128,
///
///
///
    pub vault_listed_tokens: u64,
    pub vault_listed_cost: u64,
///
///
    pub vault_fed_bought: u64,
///
///
///
///
///
///
    pub cushion_bottom_bps: u16,
    pub pol_to_mcap_bps: u16,
    pub cushion_ticks: i32,
///
///
///
    pub swept_quote: u64,
}
layout!(StairState, name = "StairState", size = 584, disc = disc::STAIR_STATE, {
    version: u8 = 0, bump: u8 = 1, steps: u8 = 2, placed: u8 = 3, first_open: u8 = 4, tail_placed: u8 = 5,
    growth_bps: u16 = 6, open_edge: i32 = 8, tail_lower: i32 = 12, tail_upper: i32 = 16, cushion_lower: i32 = 20,
    cushion_upper: i32 = 24, relist_lower: i32 = 28, relist_upper: i32 = 32, relist_cost_tick: i32 = 36,
    stair_total: u64 = 40, tail_total: u64 = 48, collected_total: u64 = 56, to_pol_total: u64 = 64,
    to_floor_total: u64 = 72, cushion_quote: u64 = 80, tail_liq: U128 = 88, cushion_liq: U128 = 104,
    relist_liq: U128 = 120, step_lower: [i32; 16] = 136, step_upper: [i32; 16] = 200, step_liq: [U128; 16] = 264,
    reserve_lower: i32 = 520, reserve_upper: i32 = 524, reserve_liq: U128 = 528, vault_listed_tokens: u64 = 544,
    vault_listed_cost: u64 = 552, vault_fed_bought: u64 = 560, cushion_bottom_bps: u16 = 568, pol_to_mcap_bps: u16 = 570,
    cushion_ticks: i32 = 572, swept_quote: u64 = 576,
});

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
///
///
///
///
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct FloorPool {
    pub version: u8,
    pub bump: u8,
    pub pool_bound: u8,
    pub quote_is_token1: u8,
    pub ladder_seeded: u8,
    pub needs_ratchet: u8,
    pub mode: u8,
///
///
///
///
///
    pub quote_suspended: u8,
    pub tick_spacing: i32,
    pub floor_tick: i32,
    pub amm_pool: [u8; 32],
///
    pub pool_initializer: [u8; 32],
    pub base_mint: [u8; 32],
    pub quote_mint: [u8; 32],
///
///
    pub pad9: [u8; 32],
///
    pub pad10: [u8; 32],
    pub skim_buy_far_bps: u16,
    pub skim_sell_far_bps: u16,
    pub creator_buy_bps: u16,
    pub creator_sell_bps: u16,
    pub holder_buy_bps: u16,
    pub holder_sell_bps: u16,
///
///
///
///
///
///
///
///
    pub wall_ticks: u16,
    pub platform_bps: u16,
    pub min_bid: u64,
    pub min_sellable: u64,
///
///
///
///
///
///
    pub owner_owed: u64,
    pub owner_paid: u64,
    pub vault_eth: u64,
    pub bid_eth_deployed: u64,
    pub unpaid_skim: u64,
    pub cumulative_volume_eth: u64,
    pub inventory_tokens: u64,
    pub inventory_cost_eth: u64,
///
///
///
///
    pub ask_tokens_deployed: u64,
    pub ladder_tokens_remaining: u64,
    pub bid_lower: i32,
    pub ask_upper: i32,
    pub ask_width: i32,
    pub seed_lower: i32,
    pub seed_upper: i32,
    pub ladder_lower: i32,
    pub ladder_upper: i32,
///
///
///
///
    pub wall_split_tick: i32,
    pub bid_liquidity: U128,
    pub ask_liquidity: U128,
    pub seed_liq: U128,
    pub ladder_liq: U128,
    pub creator_fees_owed: u64,
    pub platform_owed: u64,
    pub holder_owed: u64,
///
///
///
///
    pub wall_tail_liq: U128,
///
///
///
    pub wall_held_tokens: u64,
    pub seed_fees_swept_eth: u64,
    pub seed_fees_swept_tokens: u64,
    pub ladder_proceeds_eth: u64,
    pub platform_paid_eth: u64,
    pub platform_swept: u64,
    pub creator_fees_paid: u64,
    pub holder_paid_eth: u64,
    pub last_ratchet_slot: u64,






///
    pub bid_width: i32,
///
    pub skim_buy_at_floor_bps: u16,
///
///
///
    pub skim_sell_at_floor_bps: u16,
///
    pub pad6: [u8; 32],
///
    pub max_bounty: u64,
///
    pub bounty_bps: u16,
///
///
///
///
    pub wall_band_ticks: u16,
    pub wall_tail_bps: u16,
    pub wall_list_bps: u16,















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
    pub referral_owed: u64,
///
///
///
///
    pub referral_fees_paid: u64,
///
///
///
    pub trader_ref_owed: u64,
///
    pub trader_ref_booked: u64,
///
///
    pub pad7: [u8; 16],
///
///
///
///
///
    pub referral_share_bps: u16,
///
///
///
///
///
    pub stair_steps: u8,
    pub stair_ready: u8,
    pub stair_growth_bps: u16,
///
///
///
///
///
    pub stair_kind: u8,
///
///
///
///
    pub stair_owner: u8,






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
    pub referrer: [u8; 32],
///
///
    pub pad8: u8,
///
///
    pub holders_hooked: u8,
    pub pad4: [u8; 6],













///
///
    pub referrer_t2: [u8; 32],
///
    pub referrer_t3: [u8; 32],
///
    pub referral_paid_t2: u64,
///
    pub referral_paid_t3: u64,








///
///
    pub team_owed: u64,
///
    pub team_fees_paid: u64,
///
///
///
///
    pub team_fee_bps: u16,
    pub pad5: [u8; 6],
}

impl FloorPool {
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
    pub fn referral_destination(&self) -> Option<&[u8; 32]> {
        if self.referrer == [0u8; 32] { None } else { Some(&self.referrer) }
    }

///
///
///
    pub fn funds_referrals(&self) -> bool {
        self.referral_destination().is_some()
    }

///
    pub fn is_hooked(&self) -> bool {
        self.holders_hooked != 0
    }
}
layout!(FloorPool, name = "FloorPool", size = 784, disc = disc::FLOOR_POOL, {
    version: u8 = 0, bump: u8 = 1, pool_bound: u8 = 2, quote_is_token1: u8 = 3, ladder_seeded: u8 = 4,
    needs_ratchet: u8 = 5, mode: u8 = 6, quote_suspended: u8 = 7, tick_spacing: i32 = 8, floor_tick: i32 = 12,
    amm_pool: [u8; 32] = 16, pool_initializer: [u8; 32] = 48, base_mint: [u8; 32] = 80,
    quote_mint: [u8; 32] = 112, pad9: [u8; 32] = 144, pad10: [u8; 32] = 176,
    skim_buy_far_bps: u16 = 208, skim_sell_far_bps: u16 = 210, creator_buy_bps: u16 = 212,
    creator_sell_bps: u16 = 214, holder_buy_bps: u16 = 216, holder_sell_bps: u16 = 218,
    wall_ticks: u16 = 220, platform_bps: u16 = 222, min_bid: u64 = 224, min_sellable: u64 = 232,
    owner_owed: u64 = 240, owner_paid: u64 = 248, vault_eth: u64 = 256, bid_eth_deployed: u64 = 264,
    unpaid_skim: u64 = 272, cumulative_volume_eth: u64 = 280, inventory_tokens: u64 = 288,
    inventory_cost_eth: u64 = 296, ask_tokens_deployed: u64 = 304, ladder_tokens_remaining: u64 = 312,
    bid_lower: i32 = 320, ask_upper: i32 = 324, ask_width: i32 = 328, seed_lower: i32 = 332,
    seed_upper: i32 = 336, ladder_lower: i32 = 340, ladder_upper: i32 = 344, wall_split_tick: i32 = 348,
    bid_liquidity: U128 = 352, ask_liquidity: U128 = 368, seed_liq: U128 = 384, ladder_liq: U128 = 400,
    creator_fees_owed: u64 = 416, platform_owed: u64 = 424, holder_owed: u64 = 432, wall_tail_liq: U128 = 440,
    wall_held_tokens: u64 = 456, seed_fees_swept_eth: u64 = 464,
    seed_fees_swept_tokens: u64 = 472, ladder_proceeds_eth: u64 = 480, platform_paid_eth: u64 = 488,
    platform_swept: u64 = 496, creator_fees_paid: u64 = 504, holder_paid_eth: u64 = 512,
    last_ratchet_slot: u64 = 520, bid_width: i32 = 528, skim_buy_at_floor_bps: u16 = 532,
    skim_sell_at_floor_bps: u16 = 534, pad6: [u8; 32] = 536, max_bounty: u64 = 568,
    bounty_bps: u16 = 576, wall_band_ticks: u16 = 578, wall_tail_bps: u16 = 580, wall_list_bps: u16 = 582,
    referral_owed: u64 = 584,
    referral_fees_paid: u64 = 592, trader_ref_owed: u64 = 600, trader_ref_booked: u64 = 608, pad7: [u8; 16] = 616,
    referral_share_bps: u16 = 632, stair_steps: u8 = 634, stair_ready: u8 = 635, stair_growth_bps: u16 = 636,
    stair_kind: u8 = 638, stair_owner: u8 = 639, referrer: [u8; 32] = 640,
    pad8: u8 = 672, holders_hooked: u8 = 673, pad4: [u8; 6] = 674,
    referrer_t2: [u8; 32] = 680, referrer_t3: [u8; 32] = 712, referral_paid_t2: u64 = 744,
    referral_paid_t3: u64 = 752, team_owed: u64 = 760, team_fees_paid: u64 = 768, team_fee_bps: u16 = 776,
    pad5: [u8; 6] = 778,
});





///
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct LaunchConfig {
    pub version: u8,
    pub bump: u8,
///
    pub cluster: u8,
    pub pad0: [u8; 5],
///
    pub governance: [u8; 32],
///
///
///
///
    pub attestor: [u8; 32],
    pub hook_program: [u8; 32],
    pub amm_program: [u8; 32],
    pub platform_bps: u16,
    pub tick_spacing: u16,
///
    pub pad1: [u8; 4],
    pub min_bid_lamports: u64,
///
    pub pad3: [u8; 16],
    pub min_sellable: u64,
    pub registry_count: u64,







///
///
///
///
    pub max_bounty_lamports: u64,
///
///
///
///
///
///
    pub metadata_base_set_slot: u64,
///
///
    pub bounty_bps: u16,
///
    pub metadata_base_len: u8,
///
///
    pub metadata_base: [u8; MAX_METADATA_BASE_LEN],
///
///
    pub pad2: [u8; 5],
///
///
///
///
///
    pub pending_governance: [u8; 32],


///
///
    pub team_sale_bps_raw: u16,
///
///
    pub team_pool_bps_raw: u16,
///
///
///
///
    pub team_bps_set: u8,
    pub reserved: [u8; 3],
}
layout!(LaunchConfig, name = "LaunchConfig", size = 312, disc = disc::LAUNCH_CONFIG, {
    version: u8 = 0, bump: u8 = 1, cluster: u8 = 2, pad0: [u8; 5] = 3, governance: [u8; 32] = 8,
    attestor: [u8; 32] = 40, hook_program: [u8; 32] = 72, amm_program: [u8; 32] = 104,
    platform_bps: u16 = 136, tick_spacing: u16 = 138, pad1: [u8; 4] = 140,
    min_bid_lamports: u64 = 144, pad3: [u8; 16] = 152, min_sellable: u64 = 168,
    registry_count: u64 = 176, max_bounty_lamports: u64 = 184, metadata_base_set_slot: u64 = 192,
    bounty_bps: u16 = 200, metadata_base_len: u8 = 202,
    metadata_base: [u8; MAX_METADATA_BASE_LEN] = 203, pad2: [u8; 5] = 267,
    pending_governance: [u8; 32] = 272, team_sale_bps_raw: u16 = 304, team_pool_bps_raw: u16 = 306,
    team_bps_set: u8 = 308, reserved: [u8; 3] = 309,
});

///
///
///
///
///
///
pub const fn team_rate_setting(set: u8, raw: u16, default: u16, cap: u16) -> u16 {
    if set == 0 {
        default
    } else if raw > cap {
        cap
    } else {
        raw
    }
}

impl LaunchConfig {
///
///
    pub const fn team_sale_bps(&self) -> u16 {
        team_rate_setting(
            self.team_bps_set,
            self.team_sale_bps_raw,
            crate::constants::TEAM_SALE_FEE_BPS_DEFAULT,
            crate::constants::TEAM_SALE_FEE_BPS_MAX,
        )
    }
///
///
    pub const fn team_pool_bps(&self) -> u16 {
        team_rate_setting(
            self.team_bps_set,
            self.team_pool_bps_raw,
            crate::constants::TEAM_POOL_FEE_BPS_DEFAULT,
            crate::constants::TEAM_POOL_FEE_BPS_MAX,
        )
    }
}

///
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct RegistryEntry {
    pub index: u64,
    pub launch: [u8; 32],
    pub mint: [u8; 32],
    pub amm_pool: [u8; 32],
    pub mode: u8,
    pub pad0: [u8; 7],
    pub created_slot: u64,
}
layout!(RegistryEntry, name = "RegistryEntry", size = 120, disc = disc::REGISTRY_ENTRY, {
    index: u64 = 0, launch: [u8; 32] = 8, mint: [u8; 32] = 40, amm_pool: [u8; 32] = 72, mode: u8 = 104,
    pad0: [u8; 7] = 105, created_slot: u64 = 112,
});

///
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Launch {
    pub version: u8,
    pub bump: u8,
    pub sale_vault_bump: u8,
    pub lock_bump: u8,
///
    pub state: u8,
    pub mode: u8,
    pub mint_authority_bump: u8,
///
    pub pad0: u8,
    pub mint: [u8; 32],
///
    pub creator: [u8; 32],
///
///
///
///
    pub team_cliffs_paid_by_slot: u32,
///
///
    pub pad3: [u8; 28],
///
    pub pad4: [u8; 32],
///
    pub amm_pool: [u8; 32],
///
    pub floor_pool: [u8; 32],
///
    pub token_vault: [u8; 32],
///
    pub wsol_vault: [u8; 32],
///
///
    pub price_x64: U128,
///
///
///
///
///
///
///
///
    pub quote_usd_cents: u64,
    pub mcap_usd: u64,
    pub created_slot: u64,
    pub nonce: u64,
    pub sale_tokens: u64,
    pub pol_tokens: u64,
    pub ladder_tokens: u64,
    pub sold: u64,
    pub raised: u64,
    pub buyers: u64,
    pub open_tick: i32,
///
///
///
    pub trader_ref_sale_bump: u8,
    pub pad1: [u8; 3],
    pub skim_buy_far: u16,
    pub skim_sell_far: u16,
    pub creator_buy: u16,
    pub creator_sell: u16,
    pub holder_buy: u16,
    pub holder_sell: u16,
///
///
///
    pub team_bps: u16,
    pub pad2: u16,
    pub registry_index: u64,
    pub graduated_slot: u64,
///
///
///
///
///
///
    pub quote_mint: [u8; 32],
///
///
///
    pub team_owed: u64,


///
///
    pub team_step_tokens: u64,
///
///
    pub team_wallet_tokens: u64,
///
    pub team_anchor: i64,
///
    pub team_lock_days: u16,
///
    pub team_vest_days: [u16; 4],
///
    pub pad5: u8,
///
    pub pad6: u8,
///
    pub pad7: [u8; 32],
///
///
///
///
///
    pub stair_rent_budget: u32,
}
layout!(Launch, name = "Launch", size = 512, disc = disc::LAUNCH, {
    version: u8 = 0, bump: u8 = 1, sale_vault_bump: u8 = 2, lock_bump: u8 = 3, state: u8 = 4, mode: u8 = 5,
    mint_authority_bump: u8 = 6, pad0: u8 = 7, mint: [u8; 32] = 8, creator: [u8; 32] = 40,
    team_cliffs_paid_by_slot: u32 = 72, pad3: [u8; 28] = 76, pad4: [u8; 32] = 104, amm_pool: [u8; 32] = 136,
    floor_pool: [u8; 32] = 168, token_vault: [u8; 32] = 200, wsol_vault: [u8; 32] = 232,
    price_x64: U128 = 264, quote_usd_cents: u64 = 280, mcap_usd: u64 = 288, created_slot: u64 = 296, nonce: u64 = 304,
    sale_tokens: u64 = 312, pol_tokens: u64 = 320, ladder_tokens: u64 = 328, sold: u64 = 336,
    raised: u64 = 344, buyers: u64 = 352, open_tick: i32 = 360, trader_ref_sale_bump: u8 = 364, pad1: [u8; 3] = 365,
    skim_buy_far: u16 = 368, skim_sell_far: u16 = 370, creator_buy: u16 = 372, creator_sell: u16 = 374,
    holder_buy: u16 = 376, holder_sell: u16 = 378, team_bps: u16 = 380, pad2: u16 = 382, registry_index: u64 = 384,
    graduated_slot: u64 = 392, quote_mint: [u8; 32] = 400, team_owed: u64 = 432, team_step_tokens: u64 = 440,
    team_wallet_tokens: u64 = 448, team_anchor: i64 = 456, team_lock_days: u16 = 464, team_vest_days: [u16; 4] = 466,
    pad5: u8 = 474, pad6: u8 = 475, pad7: [u8; 32] = 476, stair_rent_budget: u32 = 508,
});

impl Launch {
///
///
///
    pub fn quote_mint(&self) -> [u8; 32] {
        self.quote_mint
    }
}

///
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct BuyerPosition {
    pub bump: u8,
    pub pad0: [u8; 7],
    pub allocation: u64,
    pub contributed: u64,
    pub reserved: [u8; 16],
}
layout!(BuyerPosition, name = "BuyerPosition", size = 40, disc = disc::BUYER_POSITION, {
    bump: u8 = 0, pad0: [u8; 7] = 1, allocation: u64 = 8, contributed: u64 = 16, reserved: [u8; 16] = 24,
});

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
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GenesisSale {
    pub version: u8,
    pub bump: u8,
    pub graduated: u8,
///
    pub sale_closed: u8,
///
    pub locked: u8,
    pub opening_tick_clamped: u8,
    pub pad0: [u8; 2],
    pub floor_share_bps: u16,
    pub pad1: [u8; 2],
    pub open_tick: i32,
///
///
    pub authority: [u8; 32],
    pub beneficiary: [u8; 32],
    pub launch: [u8; 32],
    pub mint: [u8; 32],
    pub open_time: i64,
    pub duration: u64,
    pub raised: u64,
    pub contributor_count: u64,
    pub settled_contributor_count: u64,
    pub total_claimed: u64,
    pub allocation_dust_pending: u64,
    pub allocation_dust_routed: u64,
    pub fees_applied: u64,
    pub initial_floor: u64,
    pub offered_pol: u64,
    pub returned_pol: u64,
    pub ladder_tokens: u64,
///
    pub pending_authority: [u8; 32],
///
///
///
///
///
    pub stair_top_mcap: u64,
///
    pub stair_steps: u8,
///
///
///
///
    pub payee_state: u8,
///
///
    pub payee_platform: u8,
    pub pad2: [u8; 5],
///
///
    pub refunded: u64,
    pub reserved: [u8; 8],
///
///
    pub payee: [u8; 32],
///
///
///
///
    pub whitelist_root: [u8; 32],
///
///
///
    pub ff_root: [u8; 32],
///
    pub raised_core: u64,
///
    pub raised_ff: u64,
///
///
///
    pub sold_tokens: u64,
}
layout!(GenesisSale, name = "GenesisSale", size = 432, disc = disc::GENESIS_SALE, {
    version: u8 = 0, bump: u8 = 1, graduated: u8 = 2, sale_closed: u8 = 3, locked: u8 = 4,
    opening_tick_clamped: u8 = 5, pad0: [u8; 2] = 6, floor_share_bps: u16 = 8, pad1: [u8; 2] = 10,
    open_tick: i32 = 12, authority: [u8; 32] = 16, beneficiary: [u8; 32] = 48, launch: [u8; 32] = 80,
    mint: [u8; 32] = 112, open_time: i64 = 144, duration: u64 = 152, raised: u64 = 160,
    contributor_count: u64 = 168, settled_contributor_count: u64 = 176, total_claimed: u64 = 184,
    allocation_dust_pending: u64 = 192, allocation_dust_routed: u64 = 200, fees_applied: u64 = 208,
    initial_floor: u64 = 216, offered_pol: u64 = 224, returned_pol: u64 = 232, ladder_tokens: u64 = 240,
    pending_authority: [u8; 32] = 248, stair_top_mcap: u64 = 280, stair_steps: u8 = 288, payee_state: u8 = 289,
    payee_platform: u8 = 290, pad2: [u8; 5] = 291, refunded: u64 = 296, reserved: [u8; 8] = 304, payee: [u8; 32] = 312,
    whitelist_root: [u8; 32] = 344, ff_root: [u8; 32] = 376, raised_core: u64 = 408, raised_ff: u64 = 416,
    sold_tokens: u64 = 424,
});

///
///
///
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GenesisContribution {
    pub bump: u8,
    pub settled: u8,
    pub pad0: [u8; 6],
    pub wallet: [u8; 32],
///
    pub contributed: u64,
    pub claimed: u64,
///
///
    pub contributed_ff: u64,
///
///
    pub refunded: u64,
}
layout!(GenesisContribution, name = "GenesisContribution", size = 72, disc = disc::GENESIS_CONTRIBUTION, {
    bump: u8 = 0, settled: u8 = 1, pad0: [u8; 6] = 2, wallet: [u8; 32] = 8, contributed: u64 = 40,
    claimed: u64 = 48, contributed_ff: u64 = 56, refunded: u64 = 64,
});

///
///
///
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct QuoteWalk {
///
    pub payer: [u8; 32],
///
    pub quote_mint: [u8; 32],
///
    pub cur: [u8; 32],
///
    pub ticks: i64,
///
    pub started_at: i64,
///
    pub started_slot: u64,
    pub hops: u16,
    pub bump: u8,
///
    pub done: u8,
    pub root_decimals: u8,
    pub pad0: [u8; 3],
}
layout!(QuoteWalk, name = "QuoteWalk", size = 128, disc = disc::QUOTE_WALK, {
    payer: [u8; 32] = 0, quote_mint: [u8; 32] = 32, cur: [u8; 32] = 64, ticks: i64 = 96, started_at: i64 = 104,
    started_slot: u64 = 112, hops: u16 = 120, bump: u8 = 122, done: u8 = 123, root_decimals: u8 = 124, pad0: [u8; 3] = 125,
});

///
///
///
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Pod, Zeroable)]
pub struct PresetRowZc {
///
    pub graduation_mcap_usd: u64,
///
    pub skim_sell_at_floor_multiple_bps: u32,
///
    pub creator_bps: u16,
    pub skim_buy_far_bps: u16,
    pub skim_sell_far_bps: u16,
///
    pub holder_bps: u16,
///
    pub sale_bps: u16,
    pub pol_bps: u16,
    pub stair_growth_bps: u16,
///
    pub team_buy_max_bps: u16,
///
    pub team_vest_days: [u16; 4],
    pub stair_steps: u8,
///
    pub enabled: u8,
    pub pad: [u8; 10],
}
const _: () = {
    assert!(core::mem::size_of::<PresetRowZc>() == crate::constants::PRESET_ROW_LEN);
    assert!(core::mem::offset_of!(PresetRowZc, graduation_mcap_usd) == 0);
    assert!(core::mem::offset_of!(PresetRowZc, skim_sell_at_floor_multiple_bps) == 8);
    assert!(core::mem::offset_of!(PresetRowZc, creator_bps) == 12);
    assert!(core::mem::offset_of!(PresetRowZc, skim_buy_far_bps) == 14);
    assert!(core::mem::offset_of!(PresetRowZc, skim_sell_far_bps) == 16);
    assert!(core::mem::offset_of!(PresetRowZc, holder_bps) == 18);
    assert!(core::mem::offset_of!(PresetRowZc, sale_bps) == 20);
    assert!(core::mem::offset_of!(PresetRowZc, pol_bps) == 22);
    assert!(core::mem::offset_of!(PresetRowZc, stair_growth_bps) == 24);
    assert!(core::mem::offset_of!(PresetRowZc, team_buy_max_bps) == 26);
    assert!(core::mem::offset_of!(PresetRowZc, team_vest_days) == 28);
    assert!(core::mem::offset_of!(PresetRowZc, stair_steps) == 36);
    assert!(core::mem::offset_of!(PresetRowZc, enabled) == 37);
    assert!(core::mem::offset_of!(PresetRowZc, pad) == 38);
};

///
///
///
///
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct PresetRegistry {
    pub version: u8,
    pub bump: u8,
///
    pub count: u8,
    pub pad0: [u8; 5],
    pub authority: [u8; 32],
    pub pending_authority: [u8; 32],
    pub rows: [PresetRowZc; 32],
}
layout!(PresetRegistry, name = "PresetRegistry", size = 1_608, disc = disc::PRESET_REGISTRY, {
    version: u8 = 0, bump: u8 = 1, count: u8 = 2, pad0: [u8; 5] = 3, authority: [u8; 32] = 8,
    pending_authority: [u8; 32] = 40, rows: [PresetRowZc; 32] = 72,
});
const _: () = assert!(PresetRegistry::SIZE == crate::constants::PRESET_REGISTRY_LEN);

impl PresetRegistry {
///
///
    pub fn row(&self, id: u8) -> Option<&PresetRowZc> {
        if id == 0 || id >= self.count || id > crate::constants::MAX_PRESET_ID {
            return None;
        }
        self.rows.get(id as usize)
    }
}





///
///
///
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct TraderLink {
    pub trader: [u8; 32],
    pub tier1: [u8; 32],
    pub tier2: [u8; 32],
    pub tier3: [u8; 32],
    pub bump: u8,
    pub pad0: [u8; 7],
}
layout!(TraderLink, name = "TraderLink", size = 136, disc = disc::TRADER_LINK, {
    trader: [u8; 32] = 0, tier1: [u8; 32] = 32, tier2: [u8; 32] = 64, tier3: [u8; 32] = 96, bump: u8 = 128,
    pad0: [u8; 7] = 129,
});

///
///
///
///
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct TraderBucket {
    pub trader: [u8; 32],
    pub quote_mint: [u8; 32],
    pub tier1: [u8; 32],
    pub tier2: [u8; 32],
    pub tier3: [u8; 32],
    pub owed: [u64; 3],
    pub bump: u8,
    pub pad0: [u8; 7],
}
layout!(TraderBucket, name = "TraderBucket", size = 192, disc = disc::TRADER_BUCKET, {
    trader: [u8; 32] = 0, quote_mint: [u8; 32] = 32, tier1: [u8; 32] = 64, tier2: [u8; 32] = 96, tier3: [u8; 32] = 128,
    owed: [u64; 3] = 160, bump: u8 = 184, pad0: [u8; 7] = 185,
});

#[cfg(test)]
mod tests {
    use super::*;

///
///
///
///
    #[test]
    fn the_preset_registry_is_the_designs_bytes() {
        use crate::constants::{MAX_PRESET_ID, PRESET_REGISTRY_LEN, PRESET_ROW_LEN};
        assert_eq!((PRESET_REGISTRY_LEN, PRESET_ROW_LEN, MAX_PRESET_ID), (1_608, 48, 31));
        assert_eq!(PresetRegistry::ACCOUNT_LEN, 1_616);
        let f = |n: &str| PresetRegistry::FIELDS.iter().find(|f| f.name == n).map(|f| (f.offset, f.size)).unwrap();
        assert_eq!(f("version"), (0, 1));
        assert_eq!(f("bump"), (1, 1));
        assert_eq!(f("count"), (2, 1));
        assert_eq!(f("pad0"), (3, 5));
        assert_eq!(f("authority"), (8, 32));
        assert_eq!(f("pending_authority"), (40, 32));
        assert_eq!(f("rows"), (72, 1_536));
        assert_eq!(core::mem::size_of::<PresetRowZc>(), 48);

        let mut r = PresetRegistry::zeroed();
        r.count = 5;
        r.rows[4] = PresetRowZc {
            graduation_mcap_usd: 0x0102_0304_0506_0708,
            skim_sell_at_floor_multiple_bps: 50_000,
            creator_bps: 11,
            skim_buy_far_bps: 12,
            skim_sell_far_bps: 13,
            holder_bps: 14,
            sale_bps: 2_000,
            pol_bps: 1_000,
            stair_growth_bps: 10_400,
            team_buy_max_bps: 500,
            team_vest_days: [10, 20, 40, 80],
            stair_steps: 15,
            enabled: 1,
            pad: [0; 10],
        };
        let b = bytemuck::bytes_of(&r);
        let at = 72 + 48 * 4;
        let u16_at = |o: usize| u16::from_le_bytes([b[at + o], b[at + o + 1]]);
        assert_eq!(u64::from_le_bytes(b[at..at + 8].try_into().unwrap()), 0x0102_0304_0506_0708);
        assert_eq!(u32::from_le_bytes(b[at + 8..at + 12].try_into().unwrap()), 50_000);
        assert_eq!((u16_at(12), u16_at(14), u16_at(16), u16_at(18)), (11, 12, 13, 14));
        assert_eq!((u16_at(20), u16_at(22), u16_at(24), u16_at(26)), (2_000, 1_000, 10_400, 500));
        assert_eq!((u16_at(28), u16_at(30), u16_at(32), u16_at(34)), (10, 20, 40, 80));
        assert_eq!((b[at + 36], b[at + 37]), (15, 1));
        assert_eq!(b[2], 5, "count @2");

        assert!(r.row(0).is_none(), "Custom is never a row");
        assert!(r.row(5).is_none() && r.row(31).is_none() && r.row(255).is_none(), "past count");
        assert_eq!(r.row(4).unwrap().stair_steps, 15);
        r.rows[4].enabled = 0;
        assert_eq!(r.row(4).unwrap().enabled, 0, "a disabled row is still a row");
    }

    #[test]
    fn account_lengths_match_the_spec() {
        assert_eq!(HookConfig::ACCOUNT_LEN, 240);
        assert_eq!(FloorPool::ACCOUNT_LEN, 792);
        assert_eq!(LaunchConfig::ACCOUNT_LEN, 320);
        assert_eq!(RegistryEntry::ACCOUNT_LEN, 128);
        assert_eq!(Launch::ACCOUNT_LEN, 520);
        assert_eq!(BuyerPosition::ACCOUNT_LEN, 48);

        assert_eq!(PlatformTreasury::ACCOUNT_LEN, 112);

        assert_eq!(TraderLink::ACCOUNT_LEN, 144);
        assert_eq!(TraderBucket::ACCOUNT_LEN, 200);

        assert_eq!(Pool::ACCOUNT_LEN, 640);
        assert_eq!(Position::ACCOUNT_LEN, 208);
    }

///
///
    #[test]
    fn spec_rent_figures_are_consistent_with_the_lengths() {
        let rent = |len: usize| (128 + len as u64) * 5_080;
        assert_eq!(rent(HookConfig::ACCOUNT_LEN), 1_869_440);
        assert_eq!(rent(FloorPool::ACCOUNT_LEN), 4_673_600);
        assert_eq!(rent(LaunchConfig::ACCOUNT_LEN), 2_275_840);
        assert_eq!(rent(RegistryEntry::ACCOUNT_LEN), 1_300_480);
        assert_eq!(rent(Launch::ACCOUNT_LEN), 3_291_840);
        assert_eq!(rent(BuyerPosition::ACCOUNT_LEN), 894_080);
    }

    #[test]
    fn hook_config_header_is_its_first_eight_bytes() {
        let mut hc = HookConfig::zeroed();
        hc.version = 1;
        hc.bump = 254;
        hc.max_rounds = 3;
        hc.callback_accounts = 1;
        hc.flags = crate::constants::FLOOR_HOOK_FLAGS;
        hc.authority_bump = 253;
        let mut data = vec![0u8; 8];
        data.extend_from_slice(bytemuck::bytes_of(&hc));
        let h = HookInterfaceHeader::from_account_data(&data).unwrap();
        assert_eq!(
            h,
            HookInterfaceHeader { version: 1, bump: 254, max_rounds: 3, callback_accounts: 1, flags: 0x60CC, authority_bump: 253, pad0: 0 }
        );
        assert!(HookInterfaceHeader::from_account_data(&data[..15]).is_none());
    }







    #[test]
    fn floor_pool_carve_out_at_528() {
        let f = |n: &str| FloorPool::FIELDS.iter().find(|f| f.name == n).unwrap();
        assert_eq!((f("bid_width").offset, f("bid_width").size), (528, 4));
        assert_eq!((f("skim_buy_at_floor_bps").offset, f("skim_buy_at_floor_bps").size), (532, 2));
        assert_eq!((f("skim_sell_at_floor_bps").offset, f("skim_sell_at_floor_bps").size), (534, 2));
        assert_eq!((f("pad6").offset, f("pad6").size), (536, 32));




        assert_eq!((f("max_bounty").offset, f("max_bounty").size), (568, 8));
        assert_eq!((f("bounty_bps").offset, f("bounty_bps").size), (576, 2));

        assert_eq!((f("wall_band_ticks").offset, f("wall_band_ticks").size), (578, 2));
        assert_eq!((f("wall_tail_bps").offset, f("wall_tail_bps").size), (580, 2));
        assert_eq!((f("wall_list_bps").offset, f("wall_list_bps").size), (582, 2));


















        assert!(FloorPool::FIELDS.iter().all(|f| f.name != "reserved"), "the reserve is spent");
        assert_eq!((f("pad5").offset + f("pad5").size), 784);
        assert_eq!(FloorPool::SIZE, 784);
        let mut p = FloorPool::zeroed();
        p.bid_width = 120;
        p.skim_sell_at_floor_bps = 1_000;
        p.pad6 = [9; 32];
        let bytes = bytemuck::bytes_of(&p);
        assert_eq!(&bytes[528..532], &120i32.to_le_bytes());
        assert_eq!(&bytes[534..536], &1_000u16.to_le_bytes());
        assert_eq!(&bytes[536..568], &[9u8; 32]);
    }

///
///
///
///
///
///
///
    #[test]
    fn floor_pool_referral_carve_out_at_584() {
        let f = |n: &str| FloorPool::FIELDS.iter().find(|f| f.name == n).unwrap();
        assert_eq!((f("referral_owed").offset, f("referral_owed").size), (584, 8));
        assert_eq!((f("referral_fees_paid").offset, f("referral_fees_paid").size), (592, 8));
        assert_eq!((f("trader_ref_owed").offset, f("trader_ref_owed").size), (600, 8));
        assert_eq!((f("trader_ref_booked").offset, f("trader_ref_booked").size), (608, 8));
        assert_eq!((f("pad7").offset, f("pad7").size), (616, 16));
        assert_eq!((f("referral_share_bps").offset, f("referral_share_bps").size), (632, 2));
        assert_eq!((f("stair_steps").offset, f("stair_steps").size), (634, 1));
        assert_eq!((f("stair_ready").offset, f("stair_ready").size), (635, 1));
        assert_eq!((f("stair_growth_bps").offset, f("stair_growth_bps").size), (636, 2));

        assert_eq!((f("stair_kind").offset, f("stair_kind").size), (638, 1));
        assert_eq!((f("stair_owner").offset, f("stair_owner").size), (639, 1));

        assert_eq!((f("referrer").offset, f("referrer").size), (640, 32));

        assert_eq!(FloorPool::SIZE, 784);
        assert_eq!(FloorPool::ACCOUNT_LEN, 792);

        assert_eq!(f("referral_owed").offset % 8, 0);
        assert_eq!(f("referral_fees_paid").offset % 8, 0);

        let mut p = FloorPool::zeroed();
        p.referral_owed = 7;
        p.referral_fees_paid = 11;
        p.trader_ref_owed = 13;
        p.trader_ref_booked = 17;
        p.pad7 = [3; 16];
        p.referral_share_bps = 1_000;
        let bytes = bytemuck::bytes_of(&p);
        assert_eq!(&bytes[584..592], &7u64.to_le_bytes());
        assert_eq!(&bytes[592..600], &11u64.to_le_bytes());
        assert_eq!(&bytes[600..608], &13u64.to_le_bytes());
        assert_eq!(&bytes[608..616], &17u64.to_le_bytes());
        assert_eq!(&bytes[616..632], &[3u8; 16]);
        assert_eq!(&bytes[632..634], &1_000u16.to_le_bytes());
    }

///
///
///
///
///
///
    #[test]
    fn hook_config_referral_distributor_takes_the_whole_reserve() {
        let f = |n: &str| HookConfig::FIELDS.iter().find(|f| f.name == n).unwrap();

        assert_eq!((f("pending_governance").offset, f("pending_governance").size), (200, 32));
        assert_eq!(HookConfig::SIZE, 232);
        assert_eq!(HookConfig::ACCOUNT_LEN, 240);


        assert!(HookConfig::FIELDS.iter().all(|f| f.name != "reserved"));

        let hc = HookConfig::zeroed();
        assert_eq!(hc.pending_governance, [0u8; 32]);
    }

///
///
///
///
    #[test]
    fn the_referral_carve_out_never_widened_the_account() {
        assert_eq!(FloorPool::SIZE, 784, "the struct is the size it was before the referral fields");
        assert_eq!(FloorPool::ACCOUNT_LEN, 792);
        let last = FloorPool::FIELDS.last().unwrap();

        assert_eq!(last.name, "pad5");
        assert_eq!(last.offset + last.size, FloorPool::SIZE, "the tail still ends at SIZE");




        let carved: usize = FloorPool::FIELDS.iter().filter(|f| f.offset >= 584).map(|f| f.size).sum();
        assert_eq!(carved, 56 + 40 + 80 + 24, "every byte from 584 is a carved field or its pad");
        assert_eq!(200 - 56 - 40 - 80 - 24, 0, "the 200-byte reserve of 2026-09-17 is spent");
    }

///
///
///
///
///
///
///
///
    #[test]
    fn floor_pool_holders_carve_out_at_640() {
        let f = |n: &str| FloorPool::FIELDS.iter().find(|f| f.name == n).unwrap();

        assert_eq!((f("trader_ref_owed").offset, f("trader_ref_owed").size), (600, 8));
        assert_eq!((f("pad7").offset, f("pad7").size), (616, 16), "the P4 reserve");
        assert_eq!((f("referral_share_bps").offset, f("referral_share_bps").size), (632, 2));
        assert_eq!((f("referrer").offset, f("referrer").size), (640, 32));
        assert_eq!((f("pad8").offset, f("pad8").size), (672, 1), "the retired `rewards_version`, padding since 2026-10-04");
        assert_eq!((f("holders_hooked").offset, f("holders_hooked").size), (673, 1));
        assert_eq!((f("pad4").offset, f("pad4").size), (674, 6));


        assert_eq!((f("referrer_t2").offset, f("referrer_t2").size), (680, 32));
        assert_eq!(f("referrer_t2").offset % 8, 0);
        assert_eq!(FloorPool::SIZE, 784);

        let mut p = FloorPool::zeroed();
        p.trader_ref_owed = 0xD1D1_D1D1_D1D1_D1D1;
        p.trader_ref_booked = 0xD1D1_D1D1_D1D1_D1D1;
        p.pad7 = [0xD1; 16];
        p.referrer = [0x5E; 32];
        p.pad8 = 0xD2;
        p.holders_hooked = 1;
        let bytes = bytemuck::bytes_of(&p);
        assert_eq!(&bytes[600..632], &[0xD1u8; 32], "the retired distributor bytes are its own");
        assert_eq!(&bytes[640..672], &[0x5Eu8; 32]);
        assert_eq!(bytes[672], 0xD2, "the retired version byte is its own");
        assert_eq!(bytes[673], 1);
        assert!(p.is_hooked());
        assert_eq!(p.referral_destination(), Some(&[0x5Eu8; 32]), "the referrer is the destination, never @600");
    }

///
///
///
///
///
///
    #[test]
    fn floor_pool_team_carve_out_at_760() {
        let f = |n: &str| FloorPool::FIELDS.iter().find(|f| f.name == n).unwrap();
        assert_eq!((f("referral_paid_t3").offset, f("referral_paid_t3").size), (752, 8), "nothing before 760 moved");
        assert_eq!((f("team_owed").offset, f("team_owed").size), (760, 8));
        assert_eq!((f("team_fees_paid").offset, f("team_fees_paid").size), (768, 8));
        assert_eq!((f("team_fee_bps").offset, f("team_fee_bps").size), (776, 2));
        assert_eq!((f("pad5").offset, f("pad5").size), (778, 6));
        assert!(FloorPool::FIELDS.iter().all(|f| f.name != "reserved"), "the reserve is spent");
        assert_eq!(FloorPool::SIZE, 784);
        let mut p = FloorPool::zeroed();
        (p.team_owed, p.team_fees_paid, p.team_fee_bps) = (7, 11, 25);
        p.referrer_t2 = [0x72; 32];
        let b = bytemuck::bytes_of(&p);
        assert_eq!((&b[760..768], &b[768..776], &b[776..778]), (&7u64.to_le_bytes()[..], &11u64.to_le_bytes()[..], &25u16.to_le_bytes()[..]));
        assert_eq!(&b[680..712], &[0x72u8; 32], "the tiers are untouched by the team carve");
    }

///
    #[test]
    fn a_pool_written_before_the_team_fee_reads_a_zero_rate_and_owes_nothing() {
        let mut old = vec![0u8; FloorPool::SIZE];
        old[0] = 1;
        let p: FloorPool = bytemuck::pod_read_unaligned(&old);
        assert_eq!((p.team_fee_bps, p.team_owed, p.team_fees_paid), (0, 0, 0));
    }

///
///
///
    #[test]
    fn launch_team_carve_out() {
        let f = |n: &str| Launch::FIELDS.iter().find(|f| f.name == n).unwrap();
        assert_eq!((f("holder_sell").offset, f("team_bps").offset, f("team_bps").size), (378, 380, 2));
        assert_eq!((f("pad2").offset, f("pad2").size, f("registry_index").offset), (382, 2, 384));
        assert_eq!((f("quote_mint").offset, f("team_owed").offset, f("team_owed").size), (400, 432, 8));

        assert_eq!((f("team_step_tokens").offset, f("pad7").offset, f("pad7").size), (440, 476, 32));


        assert_eq!((f("team_cliffs_paid_by_slot").offset, f("team_cliffs_paid_by_slot").size), (72, 4));
        assert_eq!((f("pad3").offset, f("pad3").size, f("pad4").offset, f("pad0").offset), (76, 28, 104, 7));
        assert_eq!((f("trader_ref_sale_bump").offset, f("pad1").offset, f("pad1").size), (364, 365, 3));

        assert_eq!((f("stair_rent_budget").offset, f("stair_rent_budget").size, Launch::SIZE), (508, 4, 512));
        assert!(Launch::FIELDS.iter().all(|f| f.name != "reserved"));
        let mut old = vec![0u8; Launch::SIZE];
        old[0] = 3;
        let l: Launch = bytemuck::pod_read_unaligned(&old);
        assert_eq!((l.team_bps, l.team_owed), (0, 0));
        assert_eq!((l.team_step_tokens, l.team_wallet_tokens, l.team_cliffs_paid_by_slot), (0, 0, 0), "no team buy");
    }

///
///
///
///
///
    #[test]
    fn launch_config_team_fee_carve_at_304_reads_the_defaults_from_zero_bytes() {
        let f = |n: &str| LaunchConfig::FIELDS.iter().find(|f| f.name == n).unwrap();
        assert_eq!((f("pending_governance").offset, f("pending_governance").size), (272, 32), "nothing before 304 moved");
        assert_eq!((f("team_sale_bps_raw").offset, f("team_sale_bps_raw").size), (304, 2));
        assert_eq!((f("team_pool_bps_raw").offset, f("team_pool_bps_raw").size), (306, 2));
        assert_eq!((f("team_bps_set").offset, f("team_bps_set").size), (308, 1));
        assert_eq!((f("reserved").offset, f("reserved").size, LaunchConfig::SIZE), (309, 3, 312));

        let mut old = vec![0u8; LaunchConfig::SIZE];
        old[0] = 1;
        let c: LaunchConfig = bytemuck::pod_read_unaligned(&old);
        assert_eq!((c.team_sale_bps(), c.team_pool_bps()), (100, 25), "zero bytes are the ruled defaults");
        assert_eq!((c.team_sale_bps(), c.team_pool_bps()), (crate::constants::TEAM_SALE_FEE_BPS_DEFAULT, crate::constants::TEAM_POOL_FEE_BPS_DEFAULT));

        let mut s = c;
        (s.team_bps_set, s.team_sale_bps_raw, s.team_pool_bps_raw) = (1, 0, 0);
        assert_eq!((s.team_sale_bps(), s.team_pool_bps()), (0, 0), "a SET zero charges nothing");
        (s.team_sale_bps_raw, s.team_pool_bps_raw) = (40, 50);
        assert_eq!((s.team_sale_bps(), s.team_pool_bps()), (40, 50));
        let b = bytemuck::bytes_of(&s);
        assert_eq!((&b[304..306], &b[306..308], b[308]), (&40u16.to_le_bytes()[..], &50u16.to_le_bytes()[..], 1));
    }

///
///
    #[test]
    fn a_raw_rate_above_its_cap_reads_as_the_cap() {
        use crate::constants::{TEAM_POOL_FEE_BPS_MAX, TEAM_SALE_FEE_BPS_MAX};
        let mut c = LaunchConfig::zeroed();
        (c.team_bps_set, c.team_sale_bps_raw, c.team_pool_bps_raw) = (1, u16::MAX, TEAM_POOL_FEE_BPS_MAX + 1);
        assert_eq!((c.team_sale_bps(), c.team_pool_bps()), (TEAM_SALE_FEE_BPS_MAX, TEAM_POOL_FEE_BPS_MAX));
        for raw in 0..=TEAM_POOL_FEE_BPS_MAX {
            assert_eq!(team_rate_setting(1, raw, 25, TEAM_POOL_FEE_BPS_MAX), raw, "every rate up to the cap reads back");
            assert_eq!(team_rate_setting(0, raw, 25, TEAM_POOL_FEE_BPS_MAX), 25, "unset is the default whatever the raw bytes");
        }
    }

///
///
///
///
///
///
///
    #[test]
    fn floor_pool_tiers_carve_out_at_680() {
        let f = |n: &str| FloorPool::FIELDS.iter().find(|f| f.name == n).unwrap();
        assert_eq!((f("referrer").offset, f("referrer").size), (640, 32), "tier 1 did not move");
        assert_eq!((f("referrer_t2").offset, f("referrer_t2").size), (680, 32));
        assert_eq!((f("referrer_t3").offset, f("referrer_t3").size), (712, 32));
        assert_eq!((f("referral_paid_t2").offset, f("referral_paid_t2").size), (744, 8));
        assert_eq!((f("referral_paid_t3").offset, f("referral_paid_t3").size), (752, 8));


        assert_eq!(f("team_owed").offset, 760, "the next field starts at the tiers' last byte");
        assert_eq!(FloorPool::SIZE, 784);
        let mut p = FloorPool::zeroed();
        p.referrer_t2 = [0x72; 32];
        p.referrer_t3 = [0x73; 32];
        p.referral_paid_t2 = 0x0102_0304;
        p.referral_paid_t3 = 0x0506_0708;
        let bytes = bytemuck::bytes_of(&p);
        assert_eq!(&bytes[680..712], &[0x72u8; 32]);
        assert_eq!(&bytes[712..744], &[0x73u8; 32]);
        assert_eq!(&bytes[744..752], &0x0102_0304u64.to_le_bytes());
        assert_eq!(&bytes[752..760], &0x0506_0708u64.to_le_bytes());
        assert!(bytes[640..680].iter().all(|&b| b == 0), "tier 1 / the holders carve untouched");
    }




///
///
///
///
///
///
    #[test]
    fn a_pool_written_before_referrals_reads_as_unreferred() {

        let mut old = vec![0u8; FloorPool::SIZE];
        old[0] = 1;
        old[520..528].copy_from_slice(&42u64.to_le_bytes());
        let p: FloorPool = bytemuck::pod_read_unaligned(&old);
        assert_eq!(p.last_ratchet_slot, 42, "the bytes before the carve-out still decode");
        assert_eq!(p.referral_destination(), None, "an all-zero destination is ABSENT, never the System Program");
        assert!(!p.funds_referrals());


        assert_eq!((p.referral_owed, p.referral_fees_paid, p.referral_share_bps), (0, 0, 0));


        let mut new_pool = FloorPool::zeroed();
        new_pool.referrer = [0x11; 32];
        assert_eq!(new_pool.referral_destination(), Some(&[0x11u8; 32]));
        assert!(new_pool.funds_referrals());
        assert_eq!((p.referrer, p.holders_hooked), ([0u8; 32], 0), "an old pool is unreferred, unhooked");
    }
}
