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
#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod graduate;
pub mod graduation_rent;
pub mod instructions;
pub mod lock;
pub mod mint_profile;
pub mod sale;
pub mod state;
pub mod trader_ref;
pub mod whitelist;

use instructions::*;
use state::CreateLaunchParams;

declare_id!("8vfdPWfaA3b9GtJoFUwc36JKDaMfVCJ4oT9HrCrQ6Tdi");

#[program]
pub mod floor_launch {
    use super::*;

    #[allow(clippy::too_many_arguments)]
    pub fn initialize_launch_config(
        ctx: Context<InitializeLaunchConfig>,
        governance: Pubkey,
        attestor: Pubkey,
        cluster: u8,
        hook_program: Pubkey,
        amm_program: Pubkey,
        min_bid_lamports: u64,
        min_sellable: u64,
    ) -> Result<()> {
        instructions::initialize_launch_config::handler(
            ctx,
            governance,
            attestor,
            cluster,
            hook_program,
            amm_program,
            min_bid_lamports,
            min_sellable,
        )
    }

///
///
///
    pub fn set_launch_economics(
        ctx: Context<SetLaunchEconomics>,
        min_bid_lamports: u64,
        bounty_bps: u16,
        max_bounty_lamports: u64,
    ) -> Result<()> {
        instructions::set_launch_economics::handler(ctx, min_bid_lamports, bounty_bps, max_bounty_lamports)
    }

///
///
///
///
    pub fn set_team_fees(ctx: Context<SetTeamFees>, sale_bps: u16, pool_bps: u16) -> Result<()> {
        instructions::set_team_fees::handler(ctx, sale_bps, pool_bps)
    }






///
///
    pub fn init_genesis(
        ctx: Context<InitGenesis>,
        beneficiary: Pubkey,
        floor_share_bps: u16,
        payee: GenesisPayeeArg,
    ) -> Result<()> {
        instructions::genesis::init_genesis(ctx, beneficiary, floor_share_bps, payee)
    }

///
    pub fn set_genesis_schedule(ctx: Context<SetGenesis>, open_time: i64, duration: u64) -> Result<()> {
        instructions::genesis::set_genesis_schedule(ctx, open_time, duration)
    }

///
    pub fn set_genesis_floor_share(ctx: Context<SetGenesis>, floor_share_bps: u16) -> Result<()> {
        instructions::genesis::set_genesis_floor_share(ctx, floor_share_bps)
    }

///
///
    pub fn set_genesis_stair_top(ctx: Context<SetGenesis>, top_mcap: u64) -> Result<()> {
        instructions::genesis::set_genesis_stair_top(ctx, top_mcap)
    }

///
///
///
///
///
    pub fn contribute(ctx: Context<Contribute>, amount: u64, proof: Vec<[u8; 32]>, phase: u8) -> Result<()> {
        instructions::genesis::contribute(ctx, amount, proof, phase)
    }

///
///
///
    pub fn end_genesis_sale(ctx: Context<EndGenesisSale>) -> Result<()> {
        instructions::genesis::end_genesis_sale(ctx)
    }

///
    pub fn graduate_genesis<'info>(ctx: Context<'info, GraduateGenesis<'info>>) -> Result<()> {
        instructions::genesis_graduate::graduate_genesis(ctx)
    }

///
    pub fn lock_genesis<'info>(ctx: Context<'info, LockGenesis<'info>>) -> Result<()> {
        instructions::genesis_graduate::lock_genesis(ctx)
    }

///
///
    pub fn claim_genesis(ctx: Context<ClaimGenesis>) -> Result<()> {
        instructions::genesis::claim_genesis(ctx)
    }

///
///
///
    pub fn refund_genesis(ctx: Context<RefundGenesis>) -> Result<()> {
        instructions::genesis::refund_genesis(ctx)
    }

///
    pub fn flush_floor_fees(ctx: Context<FlushFloorFees>) -> Result<()> {
        instructions::genesis::flush_floor_fees(ctx)
    }

///
    pub fn flush_allocation_dust(ctx: Context<FlushAllocationDust>) -> Result<()> {
        instructions::genesis::flush_allocation_dust(ctx)
    }

///
///
    pub fn transfer_genesis_authority(ctx: Context<SetGenesis>, new_authority: Pubkey) -> Result<()> {
        instructions::genesis::transfer_genesis_authority(ctx, new_authority)
    }

///
    pub fn accept_genesis_authority(ctx: Context<SetGenesis>) -> Result<()> {
        instructions::genesis::accept_genesis_authority(ctx)
    }

///
///
///
    pub fn close_graduation_rent(ctx: Context<CloseGraduationRent>) -> Result<()> {
        instructions::close_graduation_rent::handler(ctx)
    }

///
///
///
///
    pub fn fund_stair_rent(ctx: Context<FundStairRent>) -> Result<()> {
        instructions::fund_stair_rent::handler(ctx)
    }

///
///
///
///
///
    pub fn close_prepared_tick_array(ctx: Context<ClosePreparedTickArray>) -> Result<()> {
        instructions::close_prepared_tick_array::handler(ctx)
    }

///
///
///
///
///
    pub fn set_attestor(ctx: Context<SetAttestor>, new_attestor: Pubkey) -> Result<()> {
        instructions::set_attestor::handler(ctx, new_attestor)
    }

///
    pub fn set_governance(ctx: Context<SetGovernance>, new_governance: Pubkey) -> Result<()> {
        instructions::set_governance::handler(ctx, new_governance)
    }

///
    pub fn accept_governance(ctx: Context<AcceptGovernance>) -> Result<()> {
        instructions::accept_governance::handler(ctx)
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
    pub fn create_launch<'info>(
        ctx: Context<'info, CreateLaunch<'info>>,
        params: CreateLaunchParams,
        nonce: u64,
        name: String,
        symbol: String,
    ) -> Result<()> {
        instructions::create_launch::handler(ctx, params, nonce, name, symbol)
    }

///
    pub fn set_metadata_base(ctx: Context<SetMetadataBase>, base: String) -> Result<()> {
        instructions::set_metadata_base::handler(ctx, base)
    }

///
    pub fn fill_metadata_uri(ctx: Context<FillMetadataUri>) -> Result<()> {
        instructions::fill_metadata_uri::handler(ctx)
    }

///
///
///
///
    pub fn buy<'info>(
        ctx: Context<'info, Buy<'info>>,
        lamports_in: u64,
        min_tokens: u64,
        max_graduation_lamports: u64,
    ) -> Result<()> {
        instructions::buy::handler(ctx, lamports_in, min_tokens, max_graduation_lamports)
    }

///
///
    pub fn sell_back<'info>(ctx: Context<'info, SellBack<'info>>, tokens: u64) -> Result<()> {
        instructions::sell_back::handler(ctx, tokens)
    }

///
///
///
    pub fn sell_back_wsol<'info>(ctx: Context<'info, SellBack<'info>>, tokens: u64) -> Result<()> {
        instructions::sell_back::handler_wsol(ctx, tokens)
    }

    pub fn close_buyer_position(ctx: Context<CloseBuyerPosition>) -> Result<()> {
        instructions::close_buyer_position::handler(ctx)
    }

///
///
    pub fn claim_recipient(ctx: Context<ClaimRecipient>, slot: u8, recipient: Pubkey, deadline: i64) -> Result<()> {
        instructions::claim_recipient::handler(ctx, slot, recipient, deadline)
    }




    pub fn graduate<'info>(ctx: Context<'info, Graduate<'info>>) -> Result<()> {
        instructions::graduate::handler(ctx)
    }

///
///
///
///
    pub fn prepare_graduation<'info>(ctx: Context<'info, PrepareGraduation<'info>>) -> Result<()> {
        instructions::prepare_graduation::handler(ctx)
    }

///
///
    pub fn claim_team_vested<'info>(ctx: Context<'info, ClaimTeamVested<'info>>, slot: u8) -> Result<()> {
        instructions::stair_payee::claim_team_vested(ctx, slot)
    }

///
///
///
    pub fn team_buy_sell_back<'info>(ctx: Context<'info, TeamBuySellBack<'info>>) -> Result<()> {
        instructions::team_buy_sell_back::handler(ctx)
    }

///
///
///
    pub fn claim_team_payee(ctx: Context<ClaimStairPayee>, slot: u8, payee: Pubkey, deadline: i64) -> Result<()> {
        instructions::stair_payee::claim_team_payee(ctx, slot, payee, deadline)
    }

///
///
    pub fn claim_genesis_payee<'info>(
        ctx: Context<'info, ClaimStairPayee<'info>>,
        payee: Pubkey,
        deadline: i64,
    ) -> Result<()> {
        instructions::stair_payee::claim_genesis_payee(ctx, payee, deadline)
    }

///
///
///
    pub fn set_genesis_whitelist(ctx: Context<SetGenesis>, root: [u8; 32], phase: u8) -> Result<()> {
        instructions::genesis::set_genesis_whitelist(ctx, root, phase)
    }

///
///
///
///
    pub fn walk_quote_chain(ctx: Context<WalkQuoteChain>, restart: bool, finish: bool) -> Result<()> {
        instructions::quote_walk::walk_quote_chain_handler(ctx, restart, finish)
    }

///
    pub fn close_quote_walk(ctx: Context<CloseQuoteWalk>) -> Result<()> {
        instructions::quote_walk::close_quote_walk_handler(ctx)
    }

///
///
    pub fn init_preset_registry(ctx: Context<InitPresetRegistry>) -> Result<()> {
        instructions::preset_registry::init_preset_registry(ctx)
    }

///
    pub fn add_preset(ctx: Context<PresetAuthority>, args: state::PresetRowArgs) -> Result<()> {
        instructions::preset_registry::add_preset(ctx, args)
    }

///
    pub fn set_preset_enabled(ctx: Context<PresetAuthority>, id: u8, enabled: bool) -> Result<()> {
        instructions::preset_registry::set_preset_enabled(ctx, id, enabled)
    }

///
    pub fn set_preset_authority(ctx: Context<PresetAuthority>, new: Pubkey) -> Result<()> {
        instructions::preset_registry::set_preset_authority(ctx, new)
    }

///
    pub fn accept_preset_authority(ctx: Context<AcceptPresetAuthority>) -> Result<()> {
        instructions::preset_registry::accept_preset_authority(ctx)
    }

///
///
///
    pub fn link_trader(
        ctx: Context<LinkTrader>,
        tier1: Pubkey,
        tier2: Pubkey,
        tier3: Pubkey,
        deadline: i64,
    ) -> Result<()> {
        instructions::link_trader::handler(ctx, tier1, tier2, tier3, deadline)
    }

///
///
    pub fn sweep_sale_trader_referral(ctx: Context<SweepSaleTraderReferral>) -> Result<()> {
        instructions::sweep_sale_trader_referral::handler(ctx)
    }

///
///
    pub fn open_sale_trader_referral(ctx: Context<OpenSaleTraderReferral>) -> Result<()> {
        instructions::open_sale_trader_referral::handler(ctx)
    }
}





///
///
///
///
///
#[derive(AnchorSerialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GenesisPayeeArg {
    pub pending_id: [u8; 32],
    pub platform: u8,
}

///
pub const GENESIS_PAYEE_ARG_LEN: usize = 33;

impl AnchorDeserialize for GenesisPayeeArg {
    fn deserialize_reader<R: borsh::io::Read>(reader: &mut R) -> borsh::io::Result<Self> {
        let mut buf = [0u8; GENESIS_PAYEE_ARG_LEN];
        let mut got = 0usize;
        while got < buf.len() {
            match reader.read(&mut buf[got..])? {
                0 => break,
                n => got += n,
            }
        }
        match got {
            0 => Ok(Self::default()),
            GENESIS_PAYEE_ARG_LEN => {
                let mut pending_id = [0u8; 32];
                pending_id.copy_from_slice(&buf[..32]);
                Ok(Self { pending_id, platform: buf[32] })
            }
            _ => Err(borsh::io::Error::new(borsh::io::ErrorKind::UnexpectedEof, "payee: 1..=32 bytes")),
        }
    }
}

///
///
pub fn genesis_payee_ok(p: &GenesisPayeeArg) -> bool {
    if p.pending_id == [0u8; 32] {
        p.platform == 0
    } else {
        (1..=floor_types::slots::MAX_SLOT_PLATFORM).contains(&p.platform)
    }
}

#[cfg(test)]
mod genesis_payee_arg_tests {
    use super::*;

    #[test]
    fn the_payee_arg_is_optional_and_refuses_a_partial_one() {
        let mut none: &[u8] = &[];
        assert_eq!(GenesisPayeeArg::deserialize(&mut none).unwrap(), GenesisPayeeArg::default());
        let mut full: &[u8] = &[7u8; GENESIS_PAYEE_ARG_LEN];
        assert_eq!(GenesisPayeeArg::deserialize(&mut full).unwrap().pending_id, [7u8; 32]);
        let mut part: &[u8] = &[7u8; 5];
        assert!(GenesisPayeeArg::deserialize(&mut part).is_err());
    }

///
///
    #[test]
    fn mr_s_the_genesis_payee_carries_its_platform() {
        let mut b = [7u8; 33];
        b[32] = 2;
        let mut full: &[u8] = &b;
        assert_eq!(GenesisPayeeArg::deserialize(&mut full).unwrap(), GenesisPayeeArg { pending_id: [7u8; 32], platform: 2 });
        for n in [1usize, 31, 32] {
            let mut part: &[u8] = &b[..n];
            assert!(GenesisPayeeArg::deserialize(&mut part).is_err(), "{n} bytes");
        }

        assert!(genesis_payee_ok(&GenesisPayeeArg { pending_id: [7u8; 32], platform: 1 }));
        assert!(genesis_payee_ok(&GenesisPayeeArg { pending_id: [7u8; 32], platform: 4 }));
        assert!(!genesis_payee_ok(&GenesisPayeeArg { pending_id: [7u8; 32], platform: 0 }));
        assert!(!genesis_payee_ok(&GenesisPayeeArg { pending_id: [7u8; 32], platform: 5 }));
        assert!(genesis_payee_ok(&GenesisPayeeArg::default()));
        assert!(!genesis_payee_ok(&GenesisPayeeArg { pending_id: [0u8; 32], platform: 1 }));
    }
}
