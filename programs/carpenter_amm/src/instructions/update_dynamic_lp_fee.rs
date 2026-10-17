//!
//!
//!
//!

use anchor_lang::prelude::*;

use carpenter_types::hook::{BalanceDelta, HOOK_INTERFACE_VERSION};

use crate::amm_core::fees::is_dynamic_fee;
use crate::amm_core::hook_dispatch::snapshot;
use crate::constants::MAX_LP_FEE;
use crate::errors::AmmError;
use crate::events::LpFeeUpdated;
use crate::state::{HookOpResultV1, Pool};

#[event_cpi]
#[derive(Accounts)]
pub struct UpdateDynamicLpFee<'info> {
    #[account(constraint = hook_authority.key() == pool.load()?.hook_authority @ AmmError::NotHookAuthority)]
    pub hook_authority: Signer<'info>,
    #[account(mut)]
    pub pool: AccountLoader<'info, Pool>,
}

pub fn handler(ctx: Context<UpdateDynamicLpFee>, lp_fee: u32) -> Result<HookOpResultV1> {
    let result = {
        let mut p = ctx.accounts.pool.load_mut()?;

        require_keys_neq!(p.hook, Pubkey::default(), AmmError::NotHookAuthority);
        require!(is_dynamic_fee(p.fee), AmmError::NotDynamicFee);
        require!(lp_fee <= MAX_LP_FEE, AmmError::FeeInvalid);
        p.lp_fee = lp_fee;
        carpenter_types::hook::HookOpResultV1 {
            version: HOOK_INTERFACE_VERSION,
            delta: BalanceDelta { amount0: 0, amount1: 0 },
            fees0: 0,
            fees1: 0,
            post: snapshot(&p),
            hook_claims0: p.hook_claims0,
            hook_claims1: p.hook_claims1,
        }
    };
    emit_cpi!(LpFeeUpdated { pool: ctx.accounts.pool.key(), lp_fee });
    Ok(result.into())
}
