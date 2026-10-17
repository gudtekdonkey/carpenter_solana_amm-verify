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

pub mod pod;

use anchor_lang::prelude::*;
use floor_types::constants::MAX_METADATA_BASE_LEN;
use floor_types::layout::Layout;

pub use pod::U128;

///
#[account(zero_copy)]
#[derive(Debug)]
pub struct LaunchConfig {
    pub version: u8,
    pub bump: u8,
    pub cluster: u8,
    pub pad0: [u8; 5],
    pub governance: Pubkey,
    pub attestor: Pubkey,
    pub hook_program: Pubkey,
    pub amm_program: Pubkey,
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
    pub max_bounty_lamports: u64,
///
///
///
///
    pub metadata_base_set_slot: u64,
///
    pub bounty_bps: u16,
///
    pub metadata_base_len: u8,
///
///
///
    pub metadata_base: [u8; MAX_METADATA_BASE_LEN],
///
///
    pub pad2: [u8; 5],
///
///
    pub pending_governance: Pubkey,




    pub team_sale_bps_raw: u16,
    pub team_pool_bps_raw: u16,
    pub team_bps_set: u8,
    pub reserved: [u8; 3],
}

impl LaunchConfig {
///
///
    pub fn team_sale_bps(&self) -> u16 {
        floor_types::layout::team_rate_setting(
            self.team_bps_set,
            self.team_sale_bps_raw,
            floor_types::constants::TEAM_SALE_FEE_BPS_DEFAULT,
            floor_types::constants::TEAM_SALE_FEE_BPS_MAX,
        )
    }
///
///
    pub fn team_pool_bps(&self) -> u16 {
        floor_types::layout::team_rate_setting(
            self.team_bps_set,
            self.team_pool_bps_raw,
            floor_types::constants::TEAM_POOL_FEE_BPS_DEFAULT,
            floor_types::constants::TEAM_POOL_FEE_BPS_MAX,
        )
    }
}

///
#[account(zero_copy)]
#[derive(Debug)]
pub struct RegistryEntry {
    pub index: u64,
    pub launch: Pubkey,
    pub mint: Pubkey,
    pub amm_pool: Pubkey,
    pub mode: u8,
    pub pad0: [u8; 7],
    pub created_slot: u64,
}

///
#[account(zero_copy)]
#[derive(Debug)]
pub struct Launch {
    pub version: u8,
    pub bump: u8,
    pub sale_vault_bump: u8,
    pub lock_bump: u8,
    pub state: u8,
    pub mode: u8,
    pub mint_authority_bump: u8,
///
///
    pub pad0: u8,
    pub mint: Pubkey,
    pub creator: Pubkey,
///
///
///
///
    pub team_cliffs_paid_by_slot: u32,
///
    pub pad3: [u8; 28],
///
    pub pad4: [u8; 32],
    pub amm_pool: Pubkey,
    pub floor_pool: Pubkey,
    pub token_vault: Pubkey,
    pub wsol_vault: Pubkey,
    pub price_x64: U128,
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
    pub quote_mint: Pubkey,
///
///
///
    pub team_owed: u64,


    pub team_step_tokens: u64,
    pub team_wallet_tokens: u64,
    pub team_anchor: i64,
    pub team_lock_days: u16,
    pub team_vest_days: [u16; 4],
///
    pub pad5: u8,
///
    pub pad6: u8,
///
    pub pad7: [u8; 32],
///
///
    pub stair_rent_budget: u32,
}

impl Launch {
///
///
///
///
///
    pub fn quote_mint(&self) -> Pubkey {

        self.quote_mint
    }

///
///
    pub fn native_quote(&self) -> bool {
        self.quote_mint() == crate::constants::WSOL_MINT
    }
}

///
#[account(zero_copy)]
#[derive(Debug)]
pub struct BuyerPosition {
    pub bump: u8,
    pub pad0: [u8; 7],
    pub allocation: u64,
    pub contributed: u64,
    pub reserved: [u8; 16],
}

///
///
#[account(zero_copy)]
#[derive(Debug)]
pub struct GenesisSale {
    pub version: u8,
    pub bump: u8,
    pub graduated: u8,
    pub sale_closed: u8,
    pub locked: u8,
    pub opening_tick_clamped: u8,
    pub pad0: [u8; 2],
    pub floor_share_bps: u16,
    pub pad1: [u8; 2],
    pub open_tick: i32,
    pub authority: Pubkey,
    pub beneficiary: Pubkey,
    pub launch: Pubkey,
    pub mint: Pubkey,
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
///
///
    pub pending_authority: Pubkey,
///
///
///
    pub stair_top_mcap: u64,
///
    pub stair_steps: u8,
///
///
    pub payee_state: u8,
///
///
    pub payee_platform: u8,
    pub pad2: [u8; 5],
///
    pub refunded: u64,
    pub reserved: [u8; 8],
///
    pub payee: Pubkey,
///
///
///
    pub whitelist_root: [u8; 32],
///
///
///
    pub ff_root: [u8; 32],
    pub raised_core: u64,
    pub raised_ff: u64,
    pub sold_tokens: u64,
}

impl GenesisSale {
///
///
///
    pub fn sale_end(&self) -> i64 {
        if self.open_time == 0 {
            0
        } else if self.sale_closed == 0 {
            i64::MAX
        } else {

            self.open_time.saturating_add(self.duration as i64)
        }
    }
///
    pub fn opened(&self, now: i64) -> bool {
        self.open_time != 0 && now >= self.open_time
    }
///
    pub fn core_end(&self) -> i64 {
        self.open_time.saturating_add(floor_types::constants::GENESIS_CORE_PHASE_SECS as i64)
    }
///
///
    pub fn refund_opens_at(&self) -> i64 {
        if self.open_time == 0 || self.sale_closed == 0 {
            i64::MAX
        } else {
            self.sale_end().saturating_add(floor_types::constants::GENESIS_REFUND_DELAY_SECS as i64)
        }
    }
///
///
    pub fn phase_at(&self, now: i64) -> u8 {
        if !self.opened(now) || now >= self.sale_end() {
            0
        } else if now < self.core_end() {
            floor_types::constants::GENESIS_PHASE_CORE
        } else {
            floor_types::constants::GENESIS_PHASE_FF
        }
    }
}

///
#[account(zero_copy)]
#[derive(Debug)]
pub struct GenesisContribution {
    pub bump: u8,
    pub settled: u8,
    pub pad0: [u8; 6],
    pub wallet: Pubkey,
///
    pub contributed: u64,
    pub claimed: u64,
///
    pub contributed_ff: u64,
///
    pub refunded: u64,
}

///
///
#[account(zero_copy)]
#[derive(Debug)]
pub struct QuoteWalk {
    pub payer: Pubkey,
    pub quote_mint: Pubkey,
///
    pub cur: Pubkey,
    pub ticks: i64,
    pub started_at: i64,
    pub started_slot: u64,
    pub hops: u16,
    pub bump: u8,
    pub done: u8,
    pub root_decimals: u8,
    pub pad0: [u8; 3],
}

///
///
///
///
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SlotArg {
    pub bps: u16,
    pub platform: u8,
    pub key: [u8; 32],
}

impl From<&floor_types::slots::SlotArg> for SlotArg {
    fn from(a: &floor_types::slots::SlotArg) -> Self {
        Self { bps: a.bps, platform: a.platform, key: a.key }
    }
}

impl From<&SlotArg> for floor_types::slots::SlotArg {
    fn from(a: &SlotArg) -> Self {
        Self { bps: a.bps, platform: a.platform, key: a.key }
    }
}

impl From<&SlotArg> for floor_math::slots::Slot {
    fn from(a: &SlotArg) -> Self {
        Self { bps: a.bps, platform: a.platform, key: a.key }
    }
}

///
///
#[zero_copy]
#[derive(Debug, Default, PartialEq, Eq)]
pub struct PresetRowZc {
    pub graduation_mcap_usd: u64,
    pub skim_sell_at_floor_multiple_bps: u32,
    pub creator_bps: u16,
    pub skim_buy_far_bps: u16,
    pub skim_sell_far_bps: u16,
    pub holder_bps: u16,
    pub sale_bps: u16,
    pub pol_bps: u16,
    pub stair_growth_bps: u16,
    pub team_buy_max_bps: u16,
    pub team_vest_days: [u16; 4],
    pub stair_steps: u8,
///
    pub enabled: u8,
    pub pad: [u8; 10],
}
const _: () = assert!(core::mem::size_of::<PresetRowZc>() == core::mem::size_of::<floor_types::layout::PresetRowZc>());

impl PresetRowZc {
///
    pub fn numbers(&self) -> floor_math::presets::PresetRow {
        floor_math::presets::PresetRow {
            graduation_mcap_usd: self.graduation_mcap_usd,
            creator_bps: self.creator_bps,
            skim_buy_far_bps: self.skim_buy_far_bps,
            skim_sell_far_bps: self.skim_sell_far_bps,
            holder_bps: self.holder_bps,
            sale_bps: self.sale_bps,
            pol_bps: self.pol_bps,
            skim_sell_at_floor_multiple_bps: self.skim_sell_at_floor_multiple_bps,
            stair_steps: self.stair_steps,
            stair_growth_bps: self.stair_growth_bps,
            team_buy_max_bps: self.team_buy_max_bps,
            team_vest_days: self.team_vest_days,
        }
    }

///
    pub fn from_numbers(r: &floor_math::presets::PresetRow, enabled: bool) -> Self {
        Self {
            graduation_mcap_usd: r.graduation_mcap_usd,
            skim_sell_at_floor_multiple_bps: r.skim_sell_at_floor_multiple_bps,
            creator_bps: r.creator_bps,
            skim_buy_far_bps: r.skim_buy_far_bps,
            skim_sell_far_bps: r.skim_sell_far_bps,
            holder_bps: r.holder_bps,
            sale_bps: r.sale_bps,
            pol_bps: r.pol_bps,
            stair_growth_bps: r.stair_growth_bps,
            team_buy_max_bps: r.team_buy_max_bps,
            team_vest_days: r.team_vest_days,
            stair_steps: r.stair_steps,
            enabled: enabled as u8,
            pad: [0; 10],
        }
    }
}

///
///
///
///
#[account(zero_copy)]
#[derive(Debug)]
pub struct PresetRegistry {
    pub version: u8,
    pub bump: u8,
///
    pub count: u8,
    pub pad0: [u8; 5],
///
    pub authority: Pubkey,
///
    pub pending_authority: Pubkey,
    pub rows: [PresetRowZc; 32],
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetAppendFault {
///
    TableFull,
///
    Invalid(floor_math::presets::PresetField),
}

impl PresetRegistry {
///
    pub fn row(&self, id: u8) -> Option<&PresetRowZc> {
        if id == 0 || id >= self.count || id > floor_types::constants::MAX_PRESET_ID {
            return None;
        }
        self.rows.get(id as usize)
    }

///
///
    pub fn seed(&mut self, authority: Pubkey, bump: u8) -> [floor_math::presets::PresetRow; floor_math::presets::SEEDED_PRESETS as usize] {
        use floor_math::presets::{seed_row, SEEDED_PRESETS};
        self.version = crate::instructions::preset_registry::PRESET_REGISTRY_VERSION;
        self.bump = bump;
        self.authority = authority;
        self.pending_authority = Pubkey::default();
        let mut seeded = [floor_math::presets::PresetRow::default(); SEEDED_PRESETS as usize];
        for id in 1..=SEEDED_PRESETS {

            let row = seed_row(id).unwrap_or_default();
            self.rows[id as usize] = PresetRowZc::from_numbers(&row, true);
            seeded[id as usize - 1] = row;
        }
        self.count = SEEDED_PRESETS + 1;
        seeded
    }

///
///
    pub fn append(&mut self, row: &floor_math::presets::PresetRow) -> core::result::Result<u8, PresetAppendFault> {
        let id = self.count;
        if id > floor_types::constants::MAX_PRESET_ID {
            return Err(PresetAppendFault::TableFull);
        }
        floor_math::presets::validate_preset_row(row).map_err(PresetAppendFault::Invalid)?;
        self.rows[id as usize] = PresetRowZc::from_numbers(row, true);
        self.count = id + 1;
        Ok(id)
    }

///
    pub fn set_enabled(&mut self, id: u8, enabled: bool) -> bool {
        if self.row(id).is_none() {
            return false;
        }
        self.rows[id as usize].enabled = enabled as u8;
        true
    }
}

///
///
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PresetRowArgs {
    pub graduation_mcap_usd: u64,
    pub creator_bps: u16,
    pub skim_buy_far_bps: u16,
    pub skim_sell_far_bps: u16,
    pub holder_bps: u16,
    pub sale_bps: u16,
    pub pol_bps: u16,
    pub skim_sell_at_floor_multiple_bps: u32,
    pub stair_steps: u8,
    pub stair_growth_bps: u16,
    pub team_buy_max_bps: u16,
    pub team_vest_days: [u16; 4],
}

impl From<PresetRowArgs> for floor_math::presets::PresetRow {
    fn from(a: PresetRowArgs) -> Self {
        Self {
            graduation_mcap_usd: a.graduation_mcap_usd,
            creator_bps: a.creator_bps,
            skim_buy_far_bps: a.skim_buy_far_bps,
            skim_sell_far_bps: a.skim_sell_far_bps,
            holder_bps: a.holder_bps,
            sale_bps: a.sale_bps,
            pol_bps: a.pol_bps,
            skim_sell_at_floor_multiple_bps: a.skim_sell_at_floor_multiple_bps,
            stair_steps: a.stair_steps,
            stair_growth_bps: a.stair_growth_bps,
            team_buy_max_bps: a.team_buy_max_bps,
            team_vest_days: a.team_vest_days,
        }
    }
}

impl From<floor_math::presets::PresetRow> for PresetRowArgs {
    fn from(r: floor_math::presets::PresetRow) -> Self {
        Self {
            graduation_mcap_usd: r.graduation_mcap_usd,
            creator_bps: r.creator_bps,
            skim_buy_far_bps: r.skim_buy_far_bps,
            skim_sell_far_bps: r.skim_sell_far_bps,
            holder_bps: r.holder_bps,
            sale_bps: r.sale_bps,
            pol_bps: r.pol_bps,
            skim_sell_at_floor_multiple_bps: r.skim_sell_at_floor_multiple_bps,
            stair_steps: r.stair_steps,
            stair_growth_bps: r.stair_growth_bps,
            team_buy_max_bps: r.team_buy_max_bps,
            team_vest_days: r.team_vest_days,
        }
    }
}

///
///
#[account(zero_copy)]
#[derive(Debug)]
pub struct TraderLink {
    pub trader: Pubkey,
    pub tier1: Pubkey,
    pub tier2: Pubkey,
    pub tier3: Pubkey,
    pub bump: u8,
    pub pad0: [u8; 7],
}

///
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct CreateLaunchParams {
///
///
///
///
    pub mode: u8,



///
    pub graduation_mcap_usd: u64,
///
    pub skim_buy_bps: u16,
///
    pub skim_sell_bps: u16,
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
    pub creator_buy_bps: u16,
///
///
    pub creator_sell_bps: u16,
///
///
///
    pub holder_buy_bps: u16,
///
///
    pub holder_sell_bps: u16,
///
///
///
///
///
///
///
    pub recipients: Vec<SlotArg>,






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









    pub quote_usd_cents: u64,








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



    pub sale_bps: u16,
    pub pol_bps: u16,
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





    pub skim_sell_at_floor_multiple_bps: u16,
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
    pub team_buy_tokens: u64,
    pub team_steps_bps: u16,
    pub team_lock_days: u16,
    pub team_vest_days: [u16; 4],
///
///
///
///
///
    pub team_payees: Vec<SlotArg>,
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
    LaunchConfig => floor_types::layout::LaunchConfig,
    RegistryEntry => floor_types::layout::RegistryEntry,
    Launch => floor_types::layout::Launch,
    BuyerPosition => floor_types::layout::BuyerPosition,
    GenesisSale => floor_types::layout::GenesisSale,
    GenesisContribution => floor_types::layout::GenesisContribution,
    PresetRegistry => floor_types::layout::PresetRegistry,
    TraderLink => floor_types::layout::TraderLink,
}
