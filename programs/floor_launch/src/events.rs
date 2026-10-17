//!
//!
//!
//!
//!
//!

use anchor_lang::prelude::*;

#[event]
pub struct LaunchConfigInitialized {
    pub launch_config: Pubkey,
    pub governance: Pubkey,
    pub attestor: Pubkey,
    pub cluster: u8,
}

///
///
#[event]
pub struct LaunchEconomicsUpdated {
    pub launch_config: Pubkey,
    pub authority: Pubkey,
    pub old_min_bid_lamports: u64,
    pub min_bid_lamports: u64,
    pub old_bounty_bps: u16,
    pub bounty_bps: u16,
    pub old_max_bounty_lamports: u64,
    pub max_bounty_lamports: u64,
}

///
///
#[event]
pub struct GraduationRentReserved {
    pub launch: Pubkey,
    pub reserve: Pubkey,
    pub lamports: u64,
}

#[event]
pub struct GraduationRentClosed {
    pub launch: Pubkey,
    pub reserve: Pubkey,
    pub creator: Pubkey,
    pub lamports: u64,
}

///
#[event]
pub struct StairRentFunded {
    pub launch: Pubkey,
    pub amm_pool: Pubkey,
    pub stair_rent: Pubkey,
    pub lamports: u64,
///
    pub tip: u64,
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
#[event]
pub struct LaunchCreated {
    pub launch: Pubkey,
    pub mint: Pubkey,
    pub creator: Pubkey,


    pub amm_pool: Pubkey,
    pub floor_pool: Pubkey,
    pub registry_index: u64,
    pub mode: u8,
///
    pub price_x64: u128,
    pub open_tick: i32,
    pub name: String,
    pub symbol: String,
    pub uri: String,

///
    pub quote_mint: Pubkey,
///
    pub quote_decimals: u8,
///
    pub quote_usd_cents: u64,
///
///
    pub graduation_mcap_usd: u64,
}





///
///
///
///
///
///
///
#[event]
pub struct PayeeSlotsSet {
    pub launch: Pubkey,
    pub creator: Vec<crate::state::SlotArg>,
    pub team: Vec<crate::state::SlotArg>,
}

#[event]
pub struct Bought {
    pub launch: Pubkey,
    pub buyer: Pubkey,
    pub tokens: u64,
    pub cost: u64,
    pub sold: u64,
    pub raised: u64,
    pub buyers: u64,
}

#[event]
pub struct SoldBack {
    pub launch: Pubkey,
    pub buyer: Pubkey,
    pub tokens: u64,
    pub payout: u64,
    pub sold: u64,
    pub raised: u64,
}

#[event]
pub struct Graduated {
    pub launch: Pubkey,
    pub raised: u64,
    pub seeded: u64,
    pub open_tick: i32,
    pub ladder_tokens: u64,
    pub pause_authority_revoked: bool,
}

#[event]
pub struct RecipientClaimed {
    pub launch: Pubkey,
///
    pub slot: u8,
    pub pending_recipient_id: [u8; 32],
    pub recipient: Pubkey,
}

///
///
///
///
///
#[event]
pub struct MetadataBaseSet {
    pub launch_config: Pubkey,
    pub base: String,
    pub set_slot: u64,
    pub fillable_from_slot: u64,
}

///
///
#[event]
pub struct MetadataUriFilled {
    pub launch: Pubkey,
    pub mint: Pubkey,
    pub uri: String,
}

///
///
#[event]
pub struct AttestorChanged {
    pub launch_config: Pubkey,
    pub previous: Pubkey,
    pub attestor: Pubkey,
}

///
///
#[event]
pub struct GovernanceChanged {
    pub launch_config: Pubkey,
    pub previous: Pubkey,
    pub governance: Pubkey,
    pub accepted: bool,
}

///
///
///
///
///
///
///
#[event]
pub struct GraduationPrepared {
    pub launch: Pubkey,
    pub pool: Pubkey,
///
    pub pool_created: bool,
    pub tick_arrays_created: u8,
///
///
    pub reimbursed: u64,
}

///
///
///
///
///
#[event]
pub struct TeamFeesSet {
    pub launch_config: Pubkey,
///
    pub governance: Pubkey,
    pub old_sale_bps: u16,
    pub sale_bps: u16,
    pub old_pool_bps: u16,
    pub pool_bps: u16,
}





///
#[event]
pub struct GenesisInitialized {
    pub genesis_sale: Pubkey,
    pub launch: Pubkey,
    pub mint: Pubkey,
    pub authority: Pubkey,
    pub beneficiary: Pubkey,
    pub floor_share_bps: u16,
    pub duration: u64,
}

///
#[event]
pub struct ScheduleUpdated {
    pub open_time: i64,
    pub duration: u64,
    pub sale_end: i64,
}

///
///
#[event]
pub struct GenesisAuthorityTransferStarted {
    pub previous_authority: Pubkey,
    pub new_authority: Pubkey,
}

///
#[event]
pub struct GenesisAuthorityTransferred {
    pub previous_authority: Pubkey,
    pub new_authority: Pubkey,
}

///
#[event]
pub struct FloorShareUpdated {
    pub previous: u16,
    pub next: u16,
}

///
#[event]
pub struct Contributed {
    pub user: Pubkey,
///
    pub amount: u64,
    pub user_total: u64,
    pub raised_total: u64,
    pub contributor_count: u64,
///
///
///
    pub phase: u8,
    pub phase_user_total: u64,
    pub phase_raised_total: u64,
}

///
///
///
#[event]
pub struct GenesisRefunded {
    pub wallet: Pubkey,
    pub amount: u64,
    pub refunded_total: u64,
}

///
///
///
///
///
///
#[event]
pub struct OpeningTick {
    pub tick: i32,
    pub clamped: bool,
    pub stair_steps: u8,
    pub stair_top_mcap: u64,
}

///
///
#[event]
pub struct GenesisStairTopUpdated {
    pub previous: u64,
    pub next: u64,
}

///
///
#[event]
pub struct GenesisWhitelistUpdated {
    pub previous: [u8; 32],
    pub next: [u8; 32],
///
    pub phase: u8,
}

///
#[event]
pub struct FloorFeesApplied {
    pub amount: u64,
    pub cumulative_applied: u64,
}

///
///
#[event]
pub struct GenesisGraduated {
    pub sale_end: i64,
    pub raised: u64,
    pub external_fees: u64,
    pub initial_floor: u64,
    pub offered_pol: u64,
    pub returned_pol: u64,
    pub ladder_tokens: u64,
    pub open_tick: i32,
}

///
#[event]
pub struct VestingCreated {
    pub schedules: [Pubkey; 5],
}

///
#[event]
pub struct GenesisClaimed {
    pub user: Pubkey,
    pub amount: u64,
    pub cumulative_user_claimed: u64,
    pub total_claimed: u64,
}

///
#[event]
pub struct AllocationDustRouted {
    pub amount: u64,
}

///
///
///
///
///
#[event]
pub struct TeamBought {
    pub launch: Pubkey,
    pub creator: Pubkey,
    pub tokens: u64,
    pub step_tokens: u64,
    pub wallet_tokens: u64,
    pub cost: u64,
    pub fee: u64,
    pub lock_days: u16,
    pub vest_days: [u16; 4],
}

///
///
///
#[event]
pub struct TeamBuySoldBack {
    pub launch: Pubkey,
    pub creator: Pubkey,
    pub tokens: u64,
    pub cost: u64,
    pub fee: u64,
    pub sold: u64,
    pub raised: u64,
}

///
///
#[event]
pub struct TeamVestedClaimed {
    pub launch: Pubkey,
///
    pub slot: u8,
    pub to: Pubkey,
    pub amount: u64,
    pub cliffs_paid: u8,
}

///
///
#[event]
pub struct StairPayeeClaimed {
    pub launch: Pubkey,
///
    pub slot: u8,
    pub pending_id: [u8; 32],
    pub payee: Pubkey,
}

///
///
///
///
///
#[event]
pub struct QuoteChainPriced {
    pub launch: Pubkey,
    pub root_quote: Pubkey,
    pub root_decimals: u8,
    pub hops: u16,
    pub chain_ticks: i64,
}

///
///
#[event]
pub struct PresetAdded {
    pub id: u8,
    pub row: crate::state::PresetRowArgs,
}

///
///
#[event]
pub struct PresetEnabled {
    pub id: u8,
    pub enabled: bool,
}

///
#[event]
pub struct PresetAuthorityTransferStarted {
    pub authority: Pubkey,
    pub pending: Pubkey,
}

///
#[event]
pub struct PresetAuthorityTransferred {
    pub previous: Pubkey,
    pub authority: Pubkey,
}

///
///
#[event]
pub struct TraderLinked {
    pub trader: Pubkey,
    pub tier1: Pubkey,
    pub tier2: Pubkey,
    pub tier3: Pubkey,
}

///
///
///
#[event]
pub struct SaleTraderReferralBooked {
    pub launch: Pubkey,
    pub trader: Pubkey,
    pub s1: u64,
    pub s2: u64,
    pub s3: u64,
}

///
#[event]
pub struct SaleTraderReferralSwept {
    pub launch: Pubkey,
    pub quote_mint: Pubkey,
    pub amount: u64,
}

#[cfg(test)]
mod tests {
///
///
///
    #[test]
    fn mr_s_the_dead_recipient_fields_are_deleted_from_the_events() {
        let src = include_str!("events.rs");
        let body = |name: &str| {
            let at = src.find(&format!("pub struct {name} {{")).expect(name);
            &src[at..at + src[at..].find("\n}").unwrap()]
        };
        assert!(!body("LaunchCreated").contains("pub fee_recipient"), "LaunchCreated.fee_recipient is deleted");
        assert!(!body("TeamBought").contains("pub payee_pending_id"), "TeamBought.payee_pending_id is deleted");
    }

///
///
///
    #[test]
    fn mr_s_create_announces_every_slot_in_one_event() {
        let ev = include_str!("events.rs");
        assert!(!ev.contains(&format!("pub struct {}Set {{", "PayeeSlot")), "the per-slot event is deleted");
        let create = include_str!("instructions/create_launch.rs");
        let code: String = create.lines().filter(|l| !l.trim_start().starts_with("//")).collect::<Vec<_>>().join("\n");
        assert_eq!(code.matches("emit_cpi!(PayeeSlotsSet").count(), 1, "exactly one emit of the slots");
        assert!(!code.contains("emit_cpi!(PayeeSlotSet "), "no per-slot emit");
        let slot = |b: u16| crate::state::SlotArg { bps: b, platform: 1, key: [b as u8; 32] };
        let e = super::PayeeSlotsSet {
            launch: anchor_lang::prelude::Pubkey::new_from_array([9; 32]),
            creator: vec![slot(6_000), slot(4_000)],
            team: vec![slot(10_000)],
        };

        assert_eq!(anchor_lang::prelude::borsh::to_vec(&e).unwrap().len(), 32 + 4 + 70 + 4 + 35);
    }
}
