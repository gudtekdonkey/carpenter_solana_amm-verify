//!
//!
//!
//!
#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

pub mod constants;




pub mod amm_core;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;
use state::HookOpResultV1;

declare_id!("DdeiDx4ncHo9DXGZm1GrPe7mFKv7kNU3yx95uQEV2APA");

#[program]
pub mod carpenter_amm {
    use super::*;



    pub fn initialize_config(ctx: Context<InitializeConfig>, config_authority: Pubkey) -> Result<()> {
        instructions::initialize_config::handler(ctx, config_authority)
    }

    pub fn set_config_authority(ctx: Context<SetConfigAuthority>, new_authority: Pubkey) -> Result<()> {
        instructions::set_config_authority::handler(ctx, new_authority)
    }

    pub fn accept_config_authority(ctx: Context<AcceptConfigAuthority>) -> Result<()> {
        instructions::accept_config_authority::handler(ctx)
    }

    pub fn approve_hook(
        ctx: Context<ApproveHook>,
        max_rounds: u8,
        callback_accounts: u8,
        allow_public_pools: bool,
        allow_exclusivity: bool,
        name: [u8; 32],
    ) -> Result<()> {
        instructions::approve_hook::handler(ctx, max_rounds, callback_accounts, allow_public_pools, allow_exclusivity, name)
    }

    pub fn set_hook_paused(ctx: Context<SetHookPaused>, paused: bool) -> Result<()> {
        instructions::set_hook_paused::handler(ctx, paused)
    }

///
///
    pub fn reapprove_hook(ctx: Context<ReapproveHook>) -> Result<()> {
        instructions::reapprove_hook::handler(ctx)
    }

///
///
    pub fn set_hook_quotes(ctx: Context<SetHookQuotes>, allow: bool) -> Result<()> {
        instructions::set_hook_quotes::handler(ctx, allow)
    }

///
///
///
    pub fn hook_list_quote_mint(ctx: Context<HookListQuoteMint>) -> Result<()> {
        instructions::hook_list_quote_mint::handler(ctx)
    }

///
///
///
    pub fn set_base_transfer_hook(ctx: Context<SetBaseTransferHook>, transfer_hook_program: Pubkey) -> Result<()> {
        instructions::set_base_transfer_hook::handler(ctx, transfer_hook_program)
    }






    pub fn list_quote_mint(ctx: Context<ListQuoteMint>) -> Result<()> {
        instructions::list_quote_mint::handler(ctx)
    }

    pub fn retire_quote_mint(ctx: Context<RetireQuoteMint>) -> Result<()> {
        instructions::retire_quote_mint::handler(ctx)
    }

    pub fn delist_quote_mint(ctx: Context<DelistQuoteMint>) -> Result<()> {
        instructions::delist_quote_mint::handler(ctx)
    }

///
///
///
///
///
    pub fn relist_quote_mint(ctx: Context<RelistQuoteMint>) -> Result<()> {
        instructions::relist_quote_mint::handler(ctx)
    }

///
///
///
///
    pub fn set_quote_caps_admitted(ctx: Context<SetQuoteCapsAdmitted>, admitted: u8) -> Result<()> {
        instructions::set_quote_caps_admitted::handler(ctx, admitted)
    }

///
///
    pub fn set_pool_paused(ctx: Context<SetPoolPaused>, paused: bool) -> Result<()> {
        instructions::set_pool_paused::handler(ctx, paused)
    }

///
    pub fn set_pause_guardian(ctx: Context<SetPauseGuardian>, guardian: Pubkey) -> Result<()> {
        instructions::set_pause_guardian::handler(ctx, guardian)
    }

///
///
    pub fn guardian_pause_pool(ctx: Context<GuardianPausePool>) -> Result<()> {
        instructions::guardian_pause_pool::handler(ctx)
    }

///
///
    pub fn guardian_pause_hook(ctx: Context<GuardianPauseHook>) -> Result<()> {
        instructions::guardian_pause_hook::handler(ctx)
    }







    pub fn initialize_pool<'info>(
        ctx: Context<'info, InitializePool<'info>>,
        fee: u32,
        tick_spacing: u16,
        hook: Pubkey,
        sqrt_price_x64: u128,
    ) -> Result<()> {
        instructions::initialize_pool::handler(ctx, fee, tick_spacing, hook, sqrt_price_x64)
    }






    pub fn initialize_tick_array(ctx: Context<InitializeTickArray>, start_index: i32, rent_owner: RentOwnerArg) -> Result<()> {
        instructions::initialize_tick_array::handler(ctx, start_index, rent_owner.rent_owner)
    }




    pub fn initialize_position(ctx: Context<InitializePosition>, salt: u64) -> Result<()> {
        instructions::initialize_position::handler(ctx, salt)
    }









    #[allow(clippy::too_many_arguments)]
    pub fn modify_liquidity<'info>(
        ctx: Context<'info, ModifyLiquidity<'info>>,
        position_id: [u8; 32],
        salt: u64,
        tick_lower: i32,
        tick_upper: i32,
        liquidity_delta: i128,
        amount0_max: u64,
        amount1_max: u64,
        hook_data: Vec<u8>,
        rent_owner: RentOwnerArg,
        min_out: MinOutArg,
    ) -> Result<HookOpResultV1> {
        instructions::modify_liquidity::handler(
            ctx,
            position_id,
            salt,
            tick_lower,
            tick_upper,
            liquidity_delta,
            amount0_max,
            amount1_max,
            hook_data,
            rent_owner.rent_owner,
            min_out,
        )
    }

    pub fn close_position(ctx: Context<ClosePosition>) -> Result<()> {
        instructions::close_position::handler(ctx)
    }




    pub fn close_tick_array(ctx: Context<CloseTickArray>) -> Result<()> {
        instructions::close_tick_array::handler(ctx)
    }

    pub fn transfer_position(ctx: Context<TransferPosition>, new_owner: Pubkey) -> Result<()> {
        instructions::transfer_position::handler(ctx, new_owner)
    }







    pub fn swap<'info>(
        ctx: Context<'info, Swap<'info>>,
        zero_for_one: bool,
        amount_specified: i128,
        sqrt_price_limit_x64: u128,
        amount_limit: u64,
        hook_data: Vec<u8>,
    ) -> Result<HookOpResultV1> {
        instructions::swap::handler(ctx, zero_for_one, amount_specified, sqrt_price_limit_x64, amount_limit, hook_data)
    }






    pub fn swap_from_balance<'info>(
        ctx: Context<'info, Swap<'info>>,
        zero_for_one: bool,
        keep: u64,
        sqrt_price_limit_x64: u128,
        amount_limit: u64,
        limit_in_basis: u64,
        hook_data: Vec<u8>,
    ) -> Result<HookOpResultV1> {
        instructions::swap::from_balance_handler(ctx, zero_for_one, keep, sqrt_price_limit_x64, amount_limit, limit_in_basis, hook_data)
    }

    pub fn donate<'info>(ctx: Context<'info, Donate<'info>>, amount0: u64, amount1: u64, hook_data: Vec<u8>) -> Result<()> {
        instructions::donate::handler(ctx, amount0, amount1, hook_data)
    }







    pub fn hook_deposit<'info>(ctx: Context<'info, HookDeposit<'info>>, currency: u8, amount: u64) -> Result<HookOpResultV1> {
        instructions::hook_deposit::handler(ctx, currency, amount)
    }

    pub fn hook_withdraw<'info>(ctx: Context<'info, HookWithdraw<'info>>, currency: u8, amount: u64) -> Result<HookOpResultV1> {
        instructions::hook_withdraw::handler(ctx, currency, amount)
    }

    pub fn register_mint_exclusivity(ctx: Context<RegisterMintExclusivity>) -> Result<()> {
        instructions::register_mint_exclusivity::handler(ctx)
    }

    pub fn revoke_mint_exclusivity(ctx: Context<RevokeMintExclusivity>) -> Result<()> {
        instructions::revoke_mint_exclusivity::handler(ctx)
    }

    pub fn update_dynamic_lp_fee(ctx: Context<UpdateDynamicLpFee>, lp_fee: u32) -> Result<HookOpResultV1> {
        instructions::update_dynamic_lp_fee::handler(ctx, lp_fee)
    }






///
    pub fn initialize_protocol_treasury(ctx: Context<InitializeProtocolTreasury>, governance: Pubkey) -> Result<()> {
        instructions::initialize_protocol_treasury::handler(ctx, governance)
    }

///
///
///
///
    pub fn sweep_protocol_fees<'info>(ctx: Context<'info, SweepProtocolFees<'info>>, currency: u8) -> Result<()> {
        instructions::sweep_protocol_fees::handler(ctx, currency)
    }

///
///
    pub fn withdraw_protocol_fees<'info>(ctx: Context<'info, WithdrawProtocolFees<'info>>, amount: u64) -> Result<()> {
        instructions::withdraw_protocol_fees::handler(ctx, amount)
    }

///
    pub fn set_protocol_treasury_governance(ctx: Context<SetProtocolTreasuryGovernance>, new_governance: Pubkey) -> Result<()> {
        instructions::set_protocol_treasury_governance::handler(ctx, new_governance)
    }

///
    pub fn accept_protocol_treasury_governance(ctx: Context<AcceptProtocolTreasuryGovernance>) -> Result<()> {
        instructions::accept_protocol_treasury_governance::handler(ctx)
    }

///
///
    pub fn set_protocol_fee(ctx: Context<SetProtocolFee>, zero_for_one: u16, one_for_zero: u16) -> Result<()> {
        instructions::set_protocol_fee::handler(ctx, zero_for_one, one_for_zero)
    }
}
