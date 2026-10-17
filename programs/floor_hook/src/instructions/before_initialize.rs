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
//!
//!

use anchor_lang::prelude::*;
use floor_types::constants::{DYNAMIC_FEE_FLAG, MAX_TICK};
use floor_types::hook::{HOOK_INTERFACE_VERSION, PHASE_BEFORE_INITIALIZE};

use crate::errors::HookError;
use crate::instructions::HookCallback;
use crate::mech::positions::math;
use crate::state::{BeforeInitializeArgs, HookRecordV1};

pub fn handler(ctx: Context<HookCallback>, args: BeforeInitializeArgs) -> Result<HookRecordV1> {
    let pool_key = ctx.accounts.pool_signer.key();
    let mut s = ctx.accounts.floor_pool.load_mut()?;

    require_keys_eq!(pool_key, s.amm_pool, HookError::NotAmmPool);
    require_keys_eq!(args.pool, pool_key, HookError::NotAmmPool);
    require!(
        args.version == HOOK_INTERFACE_VERSION && args.phase == PHASE_BEFORE_INITIALIZE && args.round == 0,
        HookError::BadConfig
    );
    require_keys_eq!(args.sender, s.pool_initializer, HookError::NotInitializer);
    require!(s.pool_bound == 0, HookError::BadConfig);
    require!(args.fee == DYNAMIC_FEE_FLAG, HookError::NotDynamicFee);
    require!(args.tick_spacing as i32 == s.tick_spacing, HookError::BadConfig);





    require_keys_eq!(args.mint0, s.quote_mint, HookError::BadConfig);
    require_keys_eq!(args.mint1, s.base_mint, HookError::BadConfig);

    s.pool_bound = 1;
    s.floor_tick = math::ceil_to_spacing(MAX_TICK, s.tick_spacing)?;

    Ok(HookRecordV1 { version: args.version, phase: args.phase, round: args.round, ..Default::default() })
}
