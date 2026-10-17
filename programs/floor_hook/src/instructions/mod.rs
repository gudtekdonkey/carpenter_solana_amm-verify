//!
//!
//!
//!
//!
//!
//!
//!
//!

#![allow(ambiguous_glob_reexports)]

use anchor_lang::prelude::*;

use crate::state::FloorPool;

pub mod after_actions;
///
pub mod allow_all_quotes;
pub mod after_swap;
pub mod before_initialize;
pub mod before_swap;
pub mod claim_creator_fees;
pub mod claim_referral_fees;
pub mod credit_trader_bucket;
pub mod trader_referral;
pub mod donate_inventory;
///
///
pub mod floor_platform_pool;
///
pub mod hook_governance;
pub mod initialize_hook_config;
pub mod observe_quote;
pub mod pay_holder;
pub mod platform_treasury;
pub mod prepare_positions;
pub mod quote_lineage;
pub mod quote_policy;
pub mod ratchet;
pub mod reclaim_stair_array;
pub mod register_pool;
pub mod resolve_recipient;
pub mod seed_floor;
pub mod seed_inventory;
pub mod seed_liquidity;
pub mod sweep_platform_cut;
///
pub mod stair_owner;
pub mod sweep_stair_fees;
///
pub mod team_treasury;
///
pub mod twap_tick;

pub use allow_all_quotes::*;
pub use claim_creator_fees::*;
pub use claim_referral_fees::*;
pub use credit_trader_bucket::*;
pub use trader_referral::*;
pub use donate_inventory::*;
pub use floor_platform_pool::*;
pub use hook_governance::*;
pub use initialize_hook_config::*;
pub use observe_quote::*;
pub use pay_holder::*;
pub use platform_treasury::*;
pub use prepare_positions::*;
pub use quote_policy::*;
pub use ratchet::*;
pub use reclaim_stair_array::*;
pub use register_pool::*;
pub use resolve_recipient::*;
pub use seed_floor::*;
pub use seed_inventory::*;
pub use seed_liquidity::*;
pub use sweep_platform_cut::*;
pub use stair_owner::*;
pub use sweep_stair_fees::*;
pub use team_treasury::*;
pub use twap_tick::*;

///
///
///
///
///
///
///
///
#[event_cpi]
#[derive(Accounts)]
pub struct HookCallback<'info> {
///
///
    pub pool_signer: Signer<'info>,
    #[account(mut)]
    pub floor_pool: AccountLoader<'info, FloorPool>,
///
///
///
///
///
///
    pub trader_bucket: UncheckedAccount<'info>,
}

///
///
///
pub enum CallbackEvent {
    PlatformCut(crate::events::PlatformCut),
    Skimmed(crate::events::Skimmed),
    ReferralFunded(crate::events::ReferralFunded),
    CreatorFee(crate::events::CreatorFee),
    TraderReferralBooked(crate::events::TraderReferralBooked),
    FloorUnbacked(crate::events::FloorUnbacked),
    BidReplaced(crate::events::BidReplaced),
}

///
///
#[inline(never)]
pub fn emit_callback_events(ctx: &Context<HookCallback>, events: Vec<CallbackEvent>) -> Result<()> {
    for e in events {
        match e {
            CallbackEvent::PlatformCut(ev) => emit_cpi!(ev),
            CallbackEvent::Skimmed(ev) => emit_cpi!(ev),
            CallbackEvent::ReferralFunded(ev) => emit_cpi!(ev),
            CallbackEvent::CreatorFee(ev) => emit_cpi!(ev),
            CallbackEvent::TraderReferralBooked(ev) => emit_cpi!(ev),
            CallbackEvent::FloorUnbacked(ev) => emit_cpi!(ev),
            CallbackEvent::BidReplaced(ev) => emit_cpi!(ev),
        }
    }
    Ok(())
}
