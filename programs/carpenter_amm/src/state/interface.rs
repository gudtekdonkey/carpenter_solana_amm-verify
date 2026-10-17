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

use anchor_lang::prelude::*;

///
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PoolSnapshot {
    pub sqrt_price_x64: u128,
    pub tick: i32,
    pub liquidity: u128,
    pub lp_fee_pips: u32,
}

///
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BalanceDelta {
    pub amount0: i128,
    pub amount1: i128,
}

///
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HookOpResultV1 {
    pub version: u8,
    pub delta: BalanceDelta,
    pub fees0: u64,
    pub fees1: u64,
    pub post: PoolSnapshot,
    pub hook_claims0: u64,
    pub hook_claims1: u64,
}



impl From<carpenter_types::hook::PoolSnapshot> for PoolSnapshot {
    fn from(s: carpenter_types::hook::PoolSnapshot) -> Self {
        Self { sqrt_price_x64: s.sqrt_price_x64, tick: s.tick, liquidity: s.liquidity, lp_fee_pips: s.lp_fee_pips }
    }
}

impl From<carpenter_types::hook::BalanceDelta> for BalanceDelta {
    fn from(d: carpenter_types::hook::BalanceDelta) -> Self {
        Self { amount0: d.amount0, amount1: d.amount1 }
    }
}

impl From<carpenter_types::hook::HookOpResultV1> for HookOpResultV1 {
    fn from(r: carpenter_types::hook::HookOpResultV1) -> Self {
        Self {
            version: r.version,
            delta: r.delta.into(),
            fees0: r.fees0,
            fees1: r.fees1,
            post: r.post.into(),
            hook_claims0: r.hook_claims0,
            hook_claims1: r.hook_claims1,
        }
    }
}
