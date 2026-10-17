//!
//!
//!
//!
//!
//!

use anchor_lang::prelude::*;

#[event]
pub struct HookConfigInitialized {
    pub launch_program: Pubkey,
    pub launch_authority: Pubkey,
    pub hook_authority: Pubkey,
    pub platform_owner: Pubkey,
    pub platform_destination: Pubkey,
}

///
#[event]
pub struct Skimmed {
    pub pool: Pubkey,
    pub is_buy: bool,
    pub bps: u16,
    pub fee: u64,
    pub kept: u64,
}

///
#[event]
pub struct CreatorFee {
    pub pool: Pubkey,
    pub is_buy: bool,
    pub amount: u64,
}

///
#[event]
pub struct PlatformCut {
    pub pool: Pubkey,
    pub amount: u64,
}









///
///
///
///
#[event]
pub struct PolRaised {
    pub pool: Pubkey,
    pub from_edge: i32,
    pub to_edge: i32,
    pub quote_moved: u64,
    pub tokens_readded: u64,
}

///
///
///
///
///
#[event]
pub struct BidReplaced {
    pub pool: Pubkey,
    pub tokens_absorbed: u64,
    pub eth_back: u64,
    pub from_lower: i32,
    pub to_lower: i32,
}

///
///
///
///
///
///
///
#[event]
pub struct FloorUnbacked {
    pub pool: Pubkey,
    pub floor_tick: i32,
    pub vault_eth: u64,
    pub min_bid: u64,
}




///
///
///
///
#[event]
pub struct StairRatcheted {
    pub pool: Pubkey,
    pub collected: u64,
    pub to_pol: u64,
    pub to_floor: u64,
    pub cushion: u64,
    pub pol_quote: u64,
    pub short_floor: u128,
    pub short_pol: u128,
    pub first_open: u8,
    pub relisted: u64,
}

///
///
///
#[event]
pub struct StairOwnersPlaced {
    pub pool: Pubkey,
    pub kind: u8,
    pub counted: u8,
    pub payee_bps: u16,
    pub owner_total: u64,
    pub first_step: u8,
///
///
    pub payee_count: u8,
    pub anchor: i64,
    pub first_unlock_secs: u32,
    pub every_secs: u32,
}

///
///
///
///
#[event]
pub struct StairOwnerBooked {
    pub pool: Pubkey,
    pub owner_sold: u64,
    pub owner_quote: u64,
    pub due: u64,
    pub paid: u64,
    pub forfeited: u64,
    pub fees_to_floor: u64,
    pub owner_owed: u64,
}

///
#[event]
pub struct StairPayeeSwept {
    pub pool: Pubkey,
///
    pub slot: u8,
    pub payee: Pubkey,
///
    pub amount: u64,
///
    pub tip: u64,
}

///
///
#[event]
pub struct StairTokensWithdrawn {
    pub pool: Pubkey,
///
    pub slot: u8,
    pub payee: Pubkey,
    pub step: u8,
    pub from_reserve: bool,
    pub tokens: u64,
}

///
#[event]
pub struct StairPayeeResolved {
    pub pool: Pubkey,
///
    pub slot: u8,
    pub pending_id: [u8; 32],
    pub payee: Pubkey,
}

///
///
///
#[event]
pub struct StairFeesSwept {
    pub pool: Pubkey,
    pub quote: u64,
    pub tokens: u64,
///
    pub bounty: u64,
}

///
///
///
///
///
#[event]
pub struct VaultTokensFed {
    pub pool: Pubkey,
    pub step: u8,
    pub into_reserve: bool,
    pub tokens: u64,
    pub bought: u64,
    pub free: u64,
    pub cost: u64,
    pub cap_tick: i32,
}




///
///
///
#[event]
pub struct Ratcheted {
    pub pool: Pubkey,
    pub floor_tick: i32,
    pub ask_tick: i32,
    pub vault_eth: u64,
    pub inventory_tokens: u64,
    pub bounty: u64,
}

#[event]
pub struct CreatorClaimed {
    pub pool: Pubkey,
///
    pub slot: u8,
    pub creator: Pubkey,
    pub amount: u64,
}

#[event]
pub struct PlatformSwept {
    pub pool: Pubkey,
    pub destination: Pubkey,
    pub amount: u64,
}

#[event]
pub struct RecipientResolved {
    pub pool: Pubkey,
///
    pub slot: u8,
    pub pending_recipient_id: [u8; 32],
    pub recipient: Pubkey,
}

#[event]
pub struct PoolRegistered {
    pub floor_pool: Pubkey,
    pub amm_pool: Pubkey,
    pub base_mint: Pubkey,
    pub pool_initializer: Pubkey,
    pub mode: u8,
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
#[event]
pub struct ReferralFunded {
    pub pool: Pubkey,
    pub distributor: Pubkey,
///
    pub asset: Pubkey,
///
    pub amount: u64,
///
    pub gross_creator_fee: u64,
}

///
///
///
///
///
///
///
#[event]
pub struct QuoteListed {
    pub quote_mint: Pubkey,
    pub token_program: Pubkey,
    pub decimals: u8,
    pub min_bid_floor: u64,
    pub min_bid_ceil: u64,
    pub max_bounty_ceil: u64,
    pub min_bid: u64,
    pub max_bounty: u64,
}

///
///
///
#[event]
pub struct QuoteEconomicsSet {
    pub quote_mint: Pubkey,
    pub old_min_bid: u64,
    pub min_bid: u64,
    pub old_max_bounty: u64,
    pub max_bounty: u64,
}

///
///
#[event]
pub struct QuoteRetired {
    pub quote_mint: Pubkey,
}

///
///
///
///
///
///
///
#[event]
pub struct PoolRewardsTerms {
    pub floor_pool: Pubkey,
    pub amm_pool: Pubkey,
    pub base_mint: Pubkey,
    pub referrer: Pubkey,
    pub referral_share_bps: u16,
    pub holders_hooked: bool,
}

///
///
///
///
///
///
///
#[event]
pub struct PoolReferralTiers {
    pub floor_pool: Pubkey,
    pub amm_pool: Pubkey,
    pub referrer: Pubkey,
    pub referrer_t2: Pubkey,
    pub referrer_t3: Pubkey,
}

///
///
///
///
///
///
///
#[event]
pub struct ReferralTierPaid {
    pub pool: Pubkey,
    pub tier: u8,
    pub referrer: Pubkey,
    pub amount: u64,
}

///
///
///
///
#[event]
pub struct HolderPaid {
    pub pool: Pubkey,
    pub destination: Pubkey,
    pub amount: u64,
}

///
///
///
#[event]
pub struct ReferralPaid {
    pub pool: Pubkey,
    pub referrer: Pubkey,
    pub destination: Pubkey,
    pub amount: u64,
}

///
///
///
///
///
///
///
///
#[event]
pub struct QuoteObserved {
    pub pool: Pubkey,
    pub frozen: bool,
    pub paused: bool,
    pub slot: u64,
}



///
///
///
///
///
#[event]
pub struct PlatformWithdrawn {
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
pub struct TreasuryGovernanceChanged {
    pub treasury: Pubkey,
    pub previous: Pubkey,
    pub governance: Pubkey,
    pub accepted: bool,
}



///
///
///
#[event]
pub struct TeamSwept {
    pub pool: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
}

///
///
///
///
#[event]
pub struct TeamWithdrawn {
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
#[event]
pub struct InventoryDonated {
    pub pool: Pubkey,
    pub from: Pubkey,
    pub amount: u64,
}



///
///
#[event]
pub struct FloorPoolDesignated {
    pub floor_pool: Pubkey,
    pub amm_pool: Pubkey,
    pub base_mint: Pubkey,
}

///
///
///
///
#[event]
pub struct PlatformPoolCredit {
    pub source_pool: Pubkey,
    pub target_pool: Pubkey,
    pub amount: u64,
}



///
///
#[event]
pub struct HookGovernanceChanged {
    pub previous: Pubkey,
    pub governance: Pubkey,
    pub accepted: bool,
}

///
///
#[event]
pub struct StairRentReimbursed {
    pub reserve: Pubkey,
    pub cranker: Pubkey,
    pub spent: u64,
    pub lamports: u64,
}

///
///
#[event]
pub struct StairArrayReclaimed {
    pub pool: Pubkey,
    pub reserve: Pubkey,
    pub start_index: i32,
    pub lamports: u64,
///
    pub tip: u64,
}

///
///
#[event]
pub struct AllowAllQuotesSet {
    pub treasury: Pubkey,
    pub governance: Pubkey,
    pub previous: bool,
    pub allowed: bool,
}

///
///
///
///
#[event]
pub struct QuoteAdmitted {
    pub floor_pool: Pubkey,
    pub quote_mint: Pubkey,
    pub how: u8,
    pub parent: Pubkey,
    pub decimals: u8,
    pub min_bid: u64,
    pub max_bounty: u64,
    pub cap_tick: i32,
}

///
///
///
///
#[event]
pub struct TraderReferralBooked {
    pub pool: Pubkey,
    pub trader: Pubkey,
    pub s: [u64; 3],
}

///
///
#[event]
pub struct TraderReferralSwept {
    pub pool: Pubkey,
    pub quote_mint: Pubkey,
    pub amount: u64,
}

///
///
#[event]
pub struct TraderReferralClaimed {
    pub referrer: Pubkey,
    pub quote_mint: Pubkey,
    pub paid: u64,
}
