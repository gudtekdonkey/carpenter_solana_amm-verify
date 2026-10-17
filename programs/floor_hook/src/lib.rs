//!
//!
//!
//!
//!
//!
//!
#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

pub mod constants;
pub mod cu;
pub mod errors;
pub mod events;
pub mod heap;
pub mod instructions;
pub mod mech;
pub mod state;





#[cfg(all(target_os = "solana", not(feature = "no-entrypoint"), not(feature = "custom-heap")))]
compile_error!("floor_hook must be built with its `custom-heap` feature (a default feature): without it the entrypoint installs the fixed 32 KiB bump allocator the ratchet ran out of (heap.rs)");

use instructions::*;
use instructions::quote_policy::QuoteTerms;
use state::{
    AfterActionsArgs, AfterSwapArgs, BeforeInitializeArgs, BeforeSwapArgs, HookRecordV1, RegisterPoolArgs,
};

declare_id!("9uXLyyZginMWdUeu22kp7XV7cpV8183xfsVqUxKi5Viy");

#[program]
pub mod floor_hook {
    use super::*;



///
///
///
    pub fn initialize_hook_config(ctx: Context<InitializeHookConfig>, launch_program: Pubkey) -> Result<()> {
        instructions::initialize_hook_config::handler(ctx, launch_program)
    }

///
///
///
///
///
///
///
    pub fn register_pool<'info>(
        ctx: Context<'info, RegisterPool<'info>>,
        args: RegisterPoolArgs,
        tiers: ReferralTiersArg,
        team: TeamFeeArg,
        skim: SkimSellMultipleArg,
        stair: StairArg,
        owner: StairOwnerKindArg,
        slots: SlotsArg,
    ) -> Result<()> {
        instructions::register_pool::handler(ctx, args, tiers, team, skim, stair, owner.kind, slots)
    }



    pub fn before_initialize(ctx: Context<HookCallback>, args: BeforeInitializeArgs) -> Result<HookRecordV1> {
        instructions::before_initialize::handler(ctx, args)
    }

    pub fn before_swap(ctx: Context<HookCallback>, args: BeforeSwapArgs) -> Result<HookRecordV1> {
        instructions::before_swap::handler(ctx, args)
    }

    pub fn after_swap(ctx: Context<HookCallback>, args: AfterSwapArgs) -> Result<HookRecordV1> {
        instructions::after_swap::handler(ctx, args)
    }

    pub fn after_actions(ctx: Context<HookCallback>, args: AfterActionsArgs) -> Result<HookRecordV1> {
        instructions::after_actions::handler(ctx, args)
    }






    pub fn seed_floor(ctx: Context<SeedFloor>, amount: u64, team: u64) -> Result<()> {
        instructions::seed_floor::handler(ctx, amount, team)
    }









    pub fn seed_liquidity<'info>(
        ctx: Context<'info, SeedLiquidity<'info>>,
        lower: i32,
        upper: i32,
        token_amount: u64,
        rent_owner: RentOwnerArg,
    ) -> Result<()> {
        instructions::seed_liquidity::handler(ctx, lower, upper, token_amount, rent_owner.rent_owner)
    }

///
///
///
    pub fn seed_inventory<'info>(
        ctx: Context<'info, SeedInventory<'info>>,
        tokens: u64,
        open_tick: i32,
        rent_owner: RentOwnerArg,
        owner: StairOwnerWire,
    ) -> Result<()> {
        instructions::seed_inventory::handler(ctx, tokens, open_tick, rent_owner.rent_owner, owner.arg())
    }

    pub fn ratchet<'info>(ctx: Context<'info, Ratchet<'info>>, rent_owner: RentOwnerArg) -> Result<()> {
        instructions::ratchet::entry(ctx, rent_owner.rent_owner)
    }

///
///
    pub fn reclaim_stair_array(ctx: Context<ReclaimStairArray>, start_index: i32) -> Result<()> {
        instructions::reclaim_stair_array::handler(ctx, start_index)
    }






    pub fn prepare_positions<'info>(ctx: Context<'info, PreparePositions<'info>>) -> Result<()> {
        instructions::prepare_positions::handler(ctx)
    }







    pub fn donate_inventory(ctx: Context<DonateInventory>, tokens: u64) -> Result<()> {
        instructions::donate_inventory::handler(ctx, tokens)
    }

///
///
    pub fn sweep_stair_fees<'info>(ctx: Context<'info, SweepStairFees<'info>>, which: u32) -> Result<()> {
        instructions::sweep_stair_fees::handler(ctx, which)
    }





///
///
///
    pub fn twap_tick(ctx: Context<TwapTick>, window: u32) -> Result<i32> {
        instructions::twap_tick::handler(ctx, window)
    }

    pub fn claim_creator_fees(ctx: Context<ClaimCreatorFees>, slot: u8) -> Result<()> {
        instructions::claim_creator_fees::handler(ctx, slot)
    }

///
///
///
    pub fn sweep_platform_cut<'info>(ctx: Context<'info, SweepPlatformCut<'info>>) -> Result<()> {
        instructions::sweep_platform_cut::handler(ctx)
    }

///
///
///
    pub fn designate_floor_pool(ctx: Context<DesignateFloorPool>) -> Result<()> {
        instructions::floor_platform_pool::designate_handler(ctx)
    }

///
///
///
///
///
///
///
///
    pub fn set_genesis_stair(ctx: Context<SetGenesisStair>, steps: u8, growth_bps: u16, owner: StairOwnerKindArg) -> Result<()> {
        instructions::floor_platform_pool::set_genesis_stair_handler(ctx, steps, growth_bps, owner.kind)
    }

///
///
///
    pub fn sweep_stair_payee(ctx: Context<SweepStairPayee>, slot: u8) -> Result<()> {
        instructions::stair_owner::sweep_payee_handler(ctx, slot)
    }

///
///
///
///
    pub fn withdraw_stair_tokens<'info>(ctx: Context<'info, WithdrawStairTokens<'info>>, step: u8, slot: u8) -> Result<()> {
        instructions::stair_owner::withdraw_handler(ctx, step, slot)
    }

///
///
    pub fn resolve_stair_payee(ctx: Context<ResolveStairPayee>, slot: u8, payee: Pubkey) -> Result<()> {
        instructions::stair_owner::resolve_payee_handler(ctx, slot, payee)
    }

///
///
    pub fn clear_team_buy_books(ctx: Context<ClearTeamBuyBooks>) -> Result<()> {
        instructions::stair_owner::clear_team_buy_books_handler(ctx)
    }

///
///
    pub fn credit_trader_bucket(ctx: Context<CreditTraderBucket>, s: [u64; 3]) -> Result<()> {
        instructions::credit_trader_bucket::handler(ctx, s)
    }

///
///
    pub fn open_trader_bucket(ctx: Context<OpenTraderBucket>) -> Result<()> {
        instructions::trader_referral::open_trader_bucket_handler(ctx)
    }

///
    pub fn open_trader_ref_vault(ctx: Context<OpenTraderRefVault>) -> Result<()> {
        instructions::trader_referral::open_trader_ref_vault_handler(ctx)
    }

///
    pub fn sweep_trader_referral(ctx: Context<SweepTraderReferral>) -> Result<()> {
        instructions::trader_referral::sweep_trader_referral_handler(ctx)
    }

///
///
    pub fn claim_trader_referral<'info>(ctx: Context<'info, ClaimTraderReferral<'info>>) -> Result<()> {
        instructions::trader_referral::claim_trader_referral_handler(ctx)
    }

///
///
///
    pub fn credit_floor_pool(ctx: Context<CreditFloorPool>) -> Result<()> {
        instructions::floor_platform_pool::credit_handler(ctx)
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
    pub fn claim_referral_fees(ctx: Context<ClaimReferralFees>, tier: ReferralTierArg) -> Result<()> {
        instructions::claim_referral_fees::handler(ctx, tier.tier)
    }

///
///
///
///
///
///
    pub fn pay_holder_authorized<'info>(ctx: Context<'info, PayHolder<'info>>, amount: u64) -> Result<()> {
        instructions::pay_holder::authorized(ctx, amount)
    }

    pub fn pay_holder<'info>(ctx: Context<'info, PayHolder<'info>>, amount: u64) -> Result<()> {
        instructions::pay_holder::handler(ctx, amount)
    }

    pub fn resolve_recipient(ctx: Context<ResolveRecipient>, slot: u8, recipient: Pubkey) -> Result<()> {
        instructions::resolve_recipient::handler(ctx, slot, recipient)
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
    pub fn list_quote(ctx: Context<ListQuote>, terms: QuoteTerms) -> Result<()> {
        instructions::quote_policy::list_quote_handler(ctx, terms)
    }

///
///
    pub fn set_quote_economics(ctx: Context<SetQuoteEconomics>, min_bid: u64, max_bounty: u64) -> Result<()> {
        instructions::quote_policy::set_quote_economics_handler(ctx, min_bid, max_bounty)
    }

///
    pub fn retire_quote(ctx: Context<RetireQuote>) -> Result<()> {
        instructions::quote_policy::retire_quote_handler(ctx)
    }

///
///
///
    pub fn set_allow_all_quotes(ctx: Context<SetAllowAllQuotes>, allow: bool) -> Result<()> {
        instructions::allow_all_quotes::handler(ctx, allow)
    }



///
///
///
///
///
    pub fn observe_quote(ctx: Context<ObserveQuote>) -> Result<()> {
        instructions::observe_quote::handler(ctx)
    }






///
///
    pub fn withdraw_platform(ctx: Context<WithdrawPlatform>, amount: u64) -> Result<()> {
        instructions::platform_treasury::withdraw_handler(ctx, amount)
    }

///
    pub fn set_treasury_governance(ctx: Context<SetTreasuryGovernance>, new_governance: Pubkey) -> Result<()> {
        instructions::platform_treasury::set_governance_handler(ctx, new_governance)
    }

///
    pub fn accept_treasury_governance(ctx: Context<AcceptTreasuryGovernance>) -> Result<()> {
        instructions::platform_treasury::accept_governance_handler(ctx)
    }






///
///
    pub fn initialize_team_treasury(ctx: Context<InitializeTeamTreasury>, team_governance: Pubkey) -> Result<()> {
        instructions::team_treasury::initialize_team_handler(ctx, team_governance)
    }

///
///
    pub fn withdraw_team(ctx: Context<WithdrawTeam>, amount: u64) -> Result<()> {
        instructions::team_treasury::withdraw_team_handler(ctx, amount)
    }

///
    pub fn set_team_treasury_governance(ctx: Context<SetTeamTreasuryGovernance>, new_governance: Pubkey) -> Result<()> {
        instructions::team_treasury::set_team_governance_handler(ctx, new_governance)
    }

///
    pub fn accept_team_treasury_governance(ctx: Context<AcceptTeamTreasuryGovernance>) -> Result<()> {
        instructions::team_treasury::accept_team_governance_handler(ctx)
    }




///
///
    pub fn set_hook_governance(ctx: Context<SetHookGovernance>, new_governance: Pubkey) -> Result<()> {
        instructions::hook_governance::set_handler(ctx, new_governance)
    }

///
    pub fn accept_hook_governance(ctx: Context<AcceptHookGovernance>) -> Result<()> {
        instructions::hook_governance::accept_handler(ctx)
    }
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
#[derive(AnchorSerialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RentOwnerArg {
    pub rent_owner: Pubkey,
}

impl AnchorDeserialize for RentOwnerArg {
    fn deserialize_reader<R: borsh::io::Read>(reader: &mut R) -> borsh::io::Result<Self> {
        let mut key = [0u8; 32];
        let mut got = 0usize;
        while got < key.len() {
            match reader.read(&mut key[got..])? {
                0 => break,
                n => got += n,
            }
        }
        match got {
            0 => Ok(Self { rent_owner: Pubkey::default() }),
            32 => Ok(Self { rent_owner: Pubkey::new_from_array(key) }),
            _ => Err(borsh::io::Error::new(borsh::io::ErrorKind::UnexpectedEof, "rent_owner: 1..=31 bytes")),
        }
    }
}

///
fn read_up_to<R: borsh::io::Read>(reader: &mut R, buf: &mut [u8]) -> borsh::io::Result<usize> {
    let mut got = 0usize;
    while got < buf.len() {
        match reader.read(&mut buf[got..])? {
            0 => break,
            n => got += n,
        }
    }
    Ok(got)
}

///
///
///
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReferralTiersArg {
    pub referrer_t2: Pubkey,
    pub referrer_t3: Pubkey,
}

///
///
///
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TeamFeeArg {
    pub team_fee_bps: u16,
}

///
///
///
///
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SkimSellMultipleArg {
    pub skim_sell_at_floor_multiple_bps: u32,
}

///
///
///
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StairArg {
    pub steps: u8,
    pub growth_bps: u16,
}

///
///
///
///
///
#[derive(AnchorSerialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StairOwnerKindArg {
    pub kind: u8,
}

///
///
///
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SlotArg {
    pub bps: u16,
    pub platform: u8,
    pub key: [u8; 32],
}

impl From<&SlotArg> for floor_types::slots::SlotArg {
    fn from(a: &SlotArg) -> Self {
        Self { bps: a.bps, platform: a.platform, key: a.key }
    }
}

///
///
///
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct SlotsArg {
    pub creator: Vec<SlotArg>,
    pub team: Vec<SlotArg>,
}

impl AnchorDeserialize for StairOwnerKindArg {
    fn deserialize_reader<R: borsh::io::Read>(reader: &mut R) -> borsh::io::Result<Self> {
        let mut b = [0u8; 1];
        match read_up_to(reader, &mut b)? {
            0 => Ok(Self::default()),
            _ => Ok(Self { kind: b[0] }),
        }
    }
}

///
///
///
///
#[derive(AnchorSerialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StairOwnerWire {
    pub kind: u8,
    pub counted: u8,
    pub payee_bps: u16,
    pub first_unlock_secs: u32,
    pub every_secs: u32,
    pub anchor: i64,
    pub owner_tokens: u64,
    pub payee: Pubkey,
    pub payee_pending_id: [u8; 32],
///
    pub payee_platform: u8,
}

impl StairOwnerWire {
///
    pub fn arg(&self) -> Option<floor_types::owners::StairOwnerArg> {
        (self.kind != 0).then_some(floor_types::owners::StairOwnerArg {
            kind: self.kind,
            counted: self.counted,
            payee_bps: self.payee_bps,
            first_unlock_secs: self.first_unlock_secs,
            every_secs: self.every_secs,
            anchor: self.anchor,
            owner_tokens: self.owner_tokens,
            payee: self.payee.to_bytes(),
            payee_pending_id: self.payee_pending_id,
            payee_platform: self.payee_platform,
        })
    }
}

impl AnchorDeserialize for StairOwnerWire {
    fn deserialize_reader<R: borsh::io::Read>(reader: &mut R) -> borsh::io::Result<Self> {
        let mut buf = [0u8; floor_types::owners::STAIR_OWNER_ARG_LEN];
        let got = read_up_to(reader, &mut buf)?;
        match floor_types::owners::stair_owner_from_trailing(&buf[..got]) {
            Ok(None) => Ok(Self::default()),
            Ok(Some(a)) => Ok(Self {
                kind: a.kind,
                counted: a.counted,
                payee_bps: a.payee_bps,
                first_unlock_secs: a.first_unlock_secs,
                every_secs: a.every_secs,
                anchor: a.anchor,
                owner_tokens: a.owner_tokens,
                payee: Pubkey::new_from_array(a.payee),
                payee_pending_id: a.payee_pending_id,
                payee_platform: a.payee_platform,
            }),
            Err(()) => Err(borsh::io::Error::new(borsh::io::ErrorKind::UnexpectedEof, "owner: 1..=92 bytes")),
        }
    }
}

///
///
///
///
#[derive(AnchorSerialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferralTierArg {
    pub tier: u8,
}

impl Default for ReferralTierArg {
    fn default() -> Self {
        Self { tier: 1 }
    }
}

impl AnchorDeserialize for ReferralTierArg {
    fn deserialize_reader<R: borsh::io::Read>(reader: &mut R) -> borsh::io::Result<Self> {
        let mut b = [0u8; 1];
        match read_up_to(reader, &mut b)? {
            0 => Ok(Self::default()),
            _ => Ok(Self { tier: b[0] }),
        }
    }
}

#[cfg(test)]
mod referral_tier_arg_tests {
    use super::*;

    #[test]
    fn the_tier_arg_defaults_to_tier_one_and_carries_any_byte() {
        let mut none: &[u8] = &[];
        assert_eq!(ReferralTierArg::deserialize(&mut none).unwrap().tier, 1, "0 bytes: tier 1");
        for t in [0u8, 1, 2, 3, 4, 255] {
            let bytes = [t];
            let mut r: &[u8] = &bytes;
            assert_eq!(ReferralTierArg::deserialize(&mut r).unwrap().tier, t);
        }
        let mut out = Vec::new();
        ReferralTierArg { tier: 2 }.serialize(&mut out).unwrap();
        assert_eq!(out, vec![2u8]);
    }

///
    #[test]
    fn the_register_pool_tail_is_required_and_in_order() {
        let tiers = ReferralTiersArg { referrer_t2: Pubkey::new_from_array([2; 32]), referrer_t3: Pubkey::new_from_array([3; 32]) };
        let five = SkimSellMultipleArg { skim_sell_at_floor_multiple_bps: 50_000 };
        let stair = StairArg { steps: 12, growth_bps: 10_000 };
        let mut tail = Vec::new();
        tiers.serialize(&mut tail).unwrap();
        TeamFeeArg { team_fee_bps: 25 }.serialize(&mut tail).unwrap();
        five.serialize(&mut tail).unwrap();
        stair.serialize(&mut tail).unwrap();
        assert_eq!(tail.len(), 73);
        let mut r: &[u8] = &tail;
        assert_eq!(ReferralTiersArg::deserialize(&mut r).unwrap(), tiers);
        assert_eq!(TeamFeeArg::deserialize(&mut r).unwrap().team_fee_bps, 25);
        assert_eq!(SkimSellMultipleArg::deserialize(&mut r).unwrap(), five);
        assert_eq!(StairArg::deserialize(&mut r).unwrap(), stair);
        let mut none: &[u8] = &[];
        assert!(ReferralTiersArg::deserialize(&mut none).is_err(), "0 bytes: refused (no older caller to tolerate)");
        let mut none: &[u8] = &[];
        assert!(SkimSellMultipleArg::deserialize(&mut none).is_err());
    }
}

#[cfg(test)]
mod rent_owner_arg_tests {
    use super::*;

    #[test]
    fn the_rent_owner_arg_is_optional_on_the_wire_and_refuses_a_partial_key() {
        let k = Pubkey::new_from_array([7; 32]);
        let mut none: &[u8] = &[];
        assert_eq!(RentOwnerArg::deserialize(&mut none).unwrap().rent_owner, Pubkey::default(), "0 bytes: the zero key");
        let bytes = k.to_bytes();
        let mut full: &[u8] = &bytes;
        assert_eq!(RentOwnerArg::deserialize(&mut full).unwrap().rent_owner, k, "32 bytes: that key");
        for n in 1..32 {
            let mut part: &[u8] = &bytes[..n];
            assert!(RentOwnerArg::deserialize(&mut part).is_err(), "{n} bytes: refused, never truncated");
        }
        let mut out = Vec::new();
        RentOwnerArg { rent_owner: k }.serialize(&mut out).unwrap();
        assert_eq!(out, bytes.to_vec(), "serialises as the bare 32-byte key");
    }
}
