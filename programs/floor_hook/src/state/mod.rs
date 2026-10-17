//!
//!
//!
//!
//!
//!

pub mod interface;
pub mod pod;
pub mod slots;
///
pub mod twap;

use anchor_lang::prelude::*;
use floor_types::layout::Layout;

pub use interface::*;
pub use pod::U128;

///
///
#[account(zero_copy)]
#[derive(Debug)]
pub struct HookConfig {
    pub version: u8,
    pub bump: u8,
    pub max_rounds: u8,
    pub callback_accounts: u8,
    pub flags: u16,
    pub authority_bump: u8,
    pub pad0: u8,
    pub governance: Pubkey,
    pub platform_destination: Pubkey,
    pub launch_program: Pubkey,
    pub launch_authority: Pubkey,
    pub hook_authority: Pubkey,
///
///
///
    pub platform_owner: Pubkey,
///
///
///
///
///
    pub pending_governance: Pubkey,
}

///
#[account(zero_copy)]
#[derive(Debug)]
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
///
///
///
///
    pub quote_suspended: u8,
    pub tick_spacing: i32,
    pub floor_tick: i32,
    pub amm_pool: Pubkey,
    pub pool_initializer: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
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
    pub wall_tail_liq: U128,
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
///
    pub bid_width: i32,
///
    pub skim_buy_at_floor_bps: u16,
///
///
///
///
///
    pub skim_sell_at_floor_bps: u16,
///
///
    pub pad6: [u8; 32],









///
    pub max_bounty: u64,
///
    pub bounty_bps: u16,
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
    pub referral_owed: u64,
///
///
///
///
    pub referral_fees_paid: u64,
///
///
    pub trader_ref_owed: u64,
///
    pub trader_ref_booked: u64,
///
    pub pad7: [u8; 16],
///
///
///
    pub referral_share_bps: u16,
///
///
///
    pub stair_steps: u8,
///
///
    pub stair_ready: u8,
///
    pub stair_growth_bps: u16,
///
///
///
    pub stair_kind: u8,
///
///
    pub stair_owner: u8,





///
///
///
///
///
    pub referrer: Pubkey,
///
///
    pub pad8: u8,
///
///
    pub holders_hooked: u8,
    pub pad4: [u8; 6],










///
///
    pub referrer_t2: Pubkey,
///
    pub referrer_t3: Pubkey,
///
    pub referral_paid_t2: u64,
///
    pub referral_paid_t3: u64,




    pub team_owed: u64,
    pub team_fees_paid: u64,
    pub team_fee_bps: u16,
    pub pad5: [u8; 6],
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
#[account(zero_copy)]
#[derive(Debug)]
pub struct QuotePolicy {
    pub version: u8,
    pub bump: u8,
    pub decimals: u8,
    pub retired: u8,
    pub pad0: [u8; 4],
    pub quote_mint: Pubkey,
    pub token_program: Pubkey,
    pub min_bid_floor: u64,
    pub min_bid_ceil: u64,
    pub max_bounty_ceil: u64,
    pub min_bid: u64,
    pub max_bounty: u64,
    pub reserved: [u8; 56],
}









///
///
///
///
///
///
#[account(zero_copy)]
#[derive(Debug)]
pub struct PlatformTreasury {
    pub version: u8,
    pub bump: u8,
///
///
///
    pub allow_all_quotes: u8,
    pub pad0: [u8; 5],
    pub governance: Pubkey,
    pub pending_governance: Pubkey,
    pub reserved: [u8; 32],
}

///
///
///
///
#[account(zero_copy)]
#[derive(Debug)]
pub struct FloorPlatformPool {
    pub version: u8,
    pub bump: u8,
    pub pad0: [u8; 6],
    pub floor_pool: Pubkey,
    pub amm_pool: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub received: u64,
    pub credits: u64,
    pub designated_slot: u64,
    pub reserved: [u8; 64],
}

///
///
///
///
///
#[account(zero_copy)]
#[derive(Debug)]
pub struct StairState {
    pub version: u8,
    pub bump: u8,
    pub steps: u8,
    pub placed: u8,
    pub first_open: u8,
    pub tail_placed: u8,
    pub growth_bps: u16,
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
    pub stair_total: u64,
    pub tail_total: u64,
    pub collected_total: u64,
    pub to_pol_total: u64,
    pub to_floor_total: u64,
    pub cushion_quote: u64,
    pub tail_liq: U128,
    pub cushion_liq: U128,
    pub relist_liq: U128,
    pub step_lower: [i32; 16],
    pub step_upper: [i32; 16],
    pub step_liq: [U128; 16],
///
    pub reserve_lower: i32,
    pub reserve_upper: i32,
    pub reserve_liq: U128,
///
    pub vault_listed_tokens: u64,
    pub vault_listed_cost: u64,
///
    pub vault_fed_bought: u64,
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

impl StairState {
    pub const LEN: usize = 8 + <floor_types::layout::StairState as Layout>::SIZE;

///
    pub fn params(&self) -> floor_math::stair::StairParams {
        floor_math::stair::stored_stair_params(self.cushion_bottom_bps, self.pol_to_mcap_bps, self.cushion_ticks)
    }
}
const _: () = assert!(core::mem::size_of::<StairState>() == <floor_types::layout::StairState as Layout>::SIZE);
const _: () = {
    let want = <floor_types::layout::StairState as floor_types::layout::Layout>::FIELDS;
    assert!(core::mem::offset_of!(StairState, step_liq) == want[want.len() - 11].offset, "StairState: step_liq offset");
    assert!(core::mem::offset_of!(StairState, step_liq) == 264, "StairState: step_liq offset");
    assert!(core::mem::offset_of!(StairState, cushion_quote) == 80, "StairState: cushion_quote offset");
    assert!(core::mem::offset_of!(StairState, tail_liq) == 88, "StairState: tail_liq offset");
    assert!(core::mem::offset_of!(StairState, step_lower) == 136, "StairState: step_lower offset");
    assert!(core::mem::offset_of!(StairState, reserve_lower) == 520, "StairState: reserve_lower offset");
    assert!(core::mem::offset_of!(StairState, reserve_liq) == 528, "StairState: reserve_liq offset");
    assert!(core::mem::offset_of!(StairState, vault_listed_cost) == 552, "StairState: vault_listed_cost offset");
    assert!(core::mem::offset_of!(StairState, vault_fed_bought) == 560, "StairState: vault_fed_bought offset");
    assert!(core::mem::offset_of!(StairState, cushion_bottom_bps) == 568, "StairState: cushion_bottom_bps offset");
    assert!(core::mem::offset_of!(StairState, pol_to_mcap_bps) == 570, "StairState: pol_to_mcap_bps offset");
    assert!(core::mem::offset_of!(StairState, cushion_ticks) == 572, "StairState: cushion_ticks offset");
    assert!(core::mem::offset_of!(StairState, swept_quote) == want[want.len() - 1].offset, "StairState: swept_quote offset");
};

impl FloorPlatformPool {
    pub const LEN: usize = 8 + <floor_types::layout::FloorPlatformPool as Layout>::SIZE;
}
const _: () = assert!(
    core::mem::size_of::<FloorPlatformPool>() == <floor_types::layout::FloorPlatformPool as Layout>::SIZE
);

impl PlatformTreasury {
    pub const LEN: usize = 8 + <floor_types::layout::PlatformTreasury as Layout>::SIZE;
}
const _: () =
    assert!(core::mem::size_of::<PlatformTreasury>() == <floor_types::layout::PlatformTreasury as Layout>::SIZE);

impl HookConfig {
    pub const LEN: usize = 8 + <floor_types::layout::HookConfig as Layout>::SIZE;
}
impl QuotePolicy {
    pub const LEN: usize = 8 + <floor_types::layout::QuotePolicy as Layout>::SIZE;
}
const _: () = assert!(core::mem::size_of::<QuotePolicy>() == <floor_types::layout::QuotePolicy as Layout>::SIZE);
impl FloorPool {
    pub const LEN: usize = 8 + <floor_types::layout::FloorPool as Layout>::SIZE;
///
///
///
///
    pub const LEN_WITH_TWAP: usize = floor_types::twap::FLOOR_POOL_ACCOUNT_LEN_WITH_TWAP;
///
///
    pub const LEN_WITH_SLOTS: usize = floor_types::slots::FLOOR_POOL_ACCOUNT_LEN_WITH_SLOTS;
///
///
    pub const LEN_WITH_PARTS: usize = floor_types::slots::FLOOR_POOL_ACCOUNT_LEN_WITH_PARTS;

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
    pub fn require_quote_live(&self) -> Result<()> {
        require!(self.quote_suspended == 0, crate::errors::HookError::QuoteSuspended);
        Ok(())
    }

///
///
///
    pub fn owner_pending(&self) -> u64 {
        bytemuck::cast_ref::<FloorPool, floor_math::FloorState>(self).owner_pending()
    }

///
    pub fn set_owner_pending(&mut self, v: u64) {
        bytemuck::cast_mut::<FloorPool, floor_math::FloorState>(self).set_owner_pending(v);
    }
}

const _: () = assert!(FloorPool::LEN_WITH_TWAP == FloorPool::LEN + floor_types::twap::TWAP_ORACLE_SIZE);
const _: () = assert!(floor_types::twap::TWAP_ORACLE_OFFSET == FloorPool::LEN);
const _: () = assert!(core::mem::size_of::<HookConfig>() == <floor_types::layout::HookConfig as Layout>::SIZE);
const _: () = assert!(core::mem::size_of::<FloorPool>() == <floor_types::layout::FloorPool as Layout>::SIZE);








///
const fn str_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}















const _: () = {
    macro_rules! pin {
        ($($f:ident),* $(,)?) => {{
            const MINE: &[(&str, usize)] = &[$((stringify!($f), core::mem::offset_of!(FloorPool, $f))),*];
            let want = <floor_types::layout::FloorPool as floor_types::layout::Layout>::FIELDS;
            assert!(want.len() == MINE.len(), "FloorPool: the field COUNT differs from spec §4.2");
            let mut i = 0;
            while i < want.len() {
                assert!(str_eq(want[i].name, MINE[i].0), "FloorPool: a field NAME differs from spec §4.2");
                assert!(want[i].offset == MINE[i].1, "FloorPool: a field OFFSET differs from spec §4.2");
                i += 1;
            }
        }};
    }
    pin!(
        version,
        bump,
        pool_bound,
        quote_is_token1,
        ladder_seeded,
        needs_ratchet,
        mode,
        quote_suspended,
        tick_spacing,
        floor_tick,
        amm_pool,
        pool_initializer,
        base_mint,
        quote_mint,
        pad9,
        pad10,
        skim_buy_far_bps,
        skim_sell_far_bps,
        creator_buy_bps,
        creator_sell_bps,
        holder_buy_bps,
        holder_sell_bps,
        wall_ticks,
        platform_bps,
        min_bid,
        min_sellable,
        owner_owed,
        owner_paid,
        vault_eth,
        bid_eth_deployed,
        unpaid_skim,
        cumulative_volume_eth,
        inventory_tokens,
        inventory_cost_eth,
        ask_tokens_deployed,
        ladder_tokens_remaining,
        bid_lower,
        ask_upper,
        ask_width,
        seed_lower,
        seed_upper,
        ladder_lower,
        ladder_upper,
        wall_split_tick,
        bid_liquidity,
        ask_liquidity,
        seed_liq,
        ladder_liq,
        creator_fees_owed,
        platform_owed,
        holder_owed,
        wall_tail_liq,
        wall_held_tokens,
        seed_fees_swept_eth,
        seed_fees_swept_tokens,
        ladder_proceeds_eth,
        platform_paid_eth,
        platform_swept,
        creator_fees_paid,
        holder_paid_eth,
        last_ratchet_slot,






        bid_width,
        skim_buy_at_floor_bps,
        skim_sell_at_floor_bps,
        pad6,
        max_bounty,
        bounty_bps,

        wall_band_ticks,
        wall_tail_bps,
        wall_list_bps,
        referral_owed,
        referral_fees_paid,
        trader_ref_owed,
        trader_ref_booked,
        pad7,
        referral_share_bps,
        stair_steps,
        stair_ready,
        stair_growth_bps,
        stair_kind,
        stair_owner,

        referrer,
        pad8,
        holders_hooked,
        pad4,

        referrer_t2,
        referrer_t3,
        referral_paid_t2,
        referral_paid_t3,


        team_owed,
        team_fees_paid,
        team_fee_bps,
        pad5,
    );
};
