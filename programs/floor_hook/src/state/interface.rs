//!
//!
//!
//!
//!
//!
//!

use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PoolSnapshot {
    pub sqrt_price_x64: u128,
    pub tick: i32,
    pub liquidity: u128,
    pub lp_fee_pips: u32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BalanceDelta {
    pub amount0: i128,
    pub amount1: i128,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SwapParams {
    pub zero_for_one: bool,
    pub amount_specified: i128,
    pub sqrt_price_limit_x64: u128,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct BeforeInitializeArgs {
    pub version: u8,
    pub phase: u8,
    pub round: u8,
    pub pool: Pubkey,
    pub sender: Pubkey,
    pub mint0: Pubkey,
    pub mint1: Pubkey,
    pub fee: u32,
    pub tick_spacing: u16,
    pub sqrt_price_x64: u128,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct BeforeSwapArgs {
    pub version: u8,
    pub phase: u8,
    pub round: u8,
    pub pool: Pubkey,
    pub sender: Pubkey,
    pub params: SwapParams,
    pub pre: PoolSnapshot,
    pub supply0: u64,
    pub supply1: u64,
    pub hook_claims0: u64,
    pub hook_claims1: u64,
    pub hook_data: Vec<u8>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct AfterSwapArgs {
    pub version: u8,
    pub phase: u8,
    pub round: u8,
    pub pool: Pubkey,
    pub sender: Pubkey,
    pub params: SwapParams,
    pub pre: PoolSnapshot,
    pub post: PoolSnapshot,
    pub swap_delta: BalanceDelta,
    pub before_delta_specified: i128,
    pub before_delta_unspecified: i128,
    pub supply0: u64,
    pub supply1: u64,
    pub hook_claims0: u64,
    pub hook_claims1: u64,
    pub hook_data: Vec<u8>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct AfterActionsArgs {
    pub version: u8,
    pub phase: u8,
    pub round: u8,
    pub pool: Pubkey,
    pub post: PoolSnapshot,
    pub supply0: u64,
    pub supply1: u64,
    pub hook_claims0: u64,
    pub hook_claims1: u64,
    pub results: Vec<ActionResult>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ActionResult {
    pub index: u8,
    pub kind: u8,
    pub delta: BalanceDelta,
    pub fees0: u64,
    pub fees1: u64,
    pub post_sqrt_price_x64: u128,
    pub post_tick: i32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct HookRecordV1 {
    pub version: u8,
    pub phase: u8,
    pub round: u8,
    pub want_callback: bool,
    pub fee_override_pips: Option<u32>,
    pub delta_specified: i128,
    pub delta_unspecified: i128,
    pub signals: u8,



    pub fees: [u64; 5],
    pub actions: Vec<HookAction>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum HookAction {
    ModifyPosition { salt: u64, tick_lower: i32, tick_upper: i32, liquidity_delta: i128 },
    Swap { zero_for_one: bool, amount_specified: i128, sqrt_price_limit_x64: u128 },
}

///
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RegisterPoolArgs {
    pub amm_pool: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub pool_initializer: Pubkey,
    pub mode: u8,
    pub skim_buy_far_bps: u16,
    pub skim_sell_far_bps: u16,
    pub creator_buy_bps: u16,
    pub creator_sell_bps: u16,
    pub holder_buy_bps: u16,
    pub holder_sell_bps: u16,
///
///
    pub deployer: Pubkey,
///
    pub pad: [u8; 32],
    pub min_bid: u64,

    pub min_sellable: u64,
    pub platform_bps: u16,
///
    pub max_bounty: u64,
///
    pub bounty_bps: u16,


///
///
///
///
///
///
///
///
///
    pub referrer: Pubkey,




}



impl From<floor_types::hook::HookAction> for HookAction {
    fn from(a: floor_types::hook::HookAction) -> Self {
        match a {
            floor_types::hook::HookAction::ModifyPosition { salt, tick_lower, tick_upper, liquidity_delta } => {
                HookAction::ModifyPosition { salt, tick_lower, tick_upper, liquidity_delta }
            }
            floor_types::hook::HookAction::Swap { zero_for_one, amount_specified, sqrt_price_limit_x64 } => {
                HookAction::Swap { zero_for_one, amount_specified, sqrt_price_limit_x64 }
            }
        }
    }
}

impl From<floor_types::hook::HookRecordV1> for HookRecordV1 {
    fn from(r: floor_types::hook::HookRecordV1) -> Self {
        Self {
            version: r.version,
            phase: r.phase,
            round: r.round,
            want_callback: r.want_callback,
            fee_override_pips: r.fee_override_pips,
            delta_specified: r.delta_specified,
            delta_unspecified: r.delta_unspecified,
            signals: r.signals,
            fees: r.fees,
            actions: r.actions.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<RegisterPoolArgs> for floor_types::register::RegisterPoolArgs {
    fn from(a: RegisterPoolArgs) -> Self {
        Self {
            amm_pool: a.amm_pool.to_bytes(),
            base_mint: a.base_mint.to_bytes(),
            quote_mint: a.quote_mint.to_bytes(),
            pool_initializer: a.pool_initializer.to_bytes(),
            mode: a.mode,
            skim_buy_far_bps: a.skim_buy_far_bps,
            skim_sell_far_bps: a.skim_sell_far_bps,
            creator_buy_bps: a.creator_buy_bps,
            creator_sell_bps: a.creator_sell_bps,
            holder_buy_bps: a.holder_buy_bps,
            holder_sell_bps: a.holder_sell_bps,
            deployer: a.deployer.to_bytes(),
            pad: a.pad,
            min_bid: a.min_bid,
            min_sellable: a.min_sellable,
            platform_bps: a.platform_bps,
            max_bounty: a.max_bounty,
            bounty_bps: a.bounty_bps,


            referrer: a.referrer.to_bytes(),
        }
    }
}
