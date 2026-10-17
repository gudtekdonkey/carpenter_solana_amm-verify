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
//!
//!
//!
//!
//!
//!

use anchor_lang::prelude::*;
use floor_types::hook::HookDeltaAssembly;

use crate::errors::HookError;
use crate::instructions::{emit_callback_events, HookCallback};
use crate::mech::fees::{assert_claims_cover, split_of, take_fees};
use crate::mech::lift::plan_replace;
use crate::state::{AfterSwapArgs, HookRecordV1};

pub fn handler(ctx: Context<HookCallback>, args: AfterSwapArgs) -> Result<HookRecordV1> {
    let mut fp = ctx.accounts.floor_pool.load_mut()?;
    require_keys_eq!(ctx.accounts.pool_signer.key(), fp.amm_pool, HookError::NotAmmPool);
    require_keys_eq!(args.pool, fp.amm_pool, HookError::WrongPool);
    require!(fp.pool_bound == 1, HookError::NotBound);

    let p = args.params;

    require!(p.amount_specified <= 0, HookError::ExactOutNotSupported);

    let quote_is_token1 = fp.quote_is_token1 != 0;
    let is_buy = p.zero_for_one != quote_is_token1;
    let quote_specified = is_buy == (p.amount_specified < 0);

    let mut rec = HookRecordV1 {
        version: args.version,
        phase: args.phase,
        round: args.round,
        ..Default::default()
    };


    let mut events = Vec::new();
    if !quote_specified {
        let quote_leg = if quote_is_token1 { args.swap_delta.amount1 } else { args.swap_delta.amount0 };
        let leg = u64::try_from(quote_leg.unsigned_abs()).map_err(|_| error!(HookError::MathOverflow))?;
        if leg > 0 {
            fp.cumulative_volume_eth = fp.cumulative_volume_eth.saturating_add(leg);


            let take = take_fees(&mut fp, leg, is_buy, args.post.tick, None, &mut events)?;
            rec.delta_unspecified = take.total as i128;


            rec.fees = split_of(&take);

            crate::mech::trader_ref::book(&mut fp, &ctx.accounts.trader_bucket.to_account_info(), &args.sender, take.team_fee, &mut events);
        }
    }















    let stair_placed = fp.stair_steps != 0 && fp.stair_ready != 0;
    if is_buy && stair_placed && fp.unpaid_skim == 0 && fp.needs_ratchet == 0 {
        let quote_leg = if quote_is_token1 { args.swap_delta.amount1 } else { args.swap_delta.amount0 };
        if quote_leg.unsigned_abs() >= u128::from(fp.min_bid) && args.post.tick <= fp.ladder_upper {
            fp.needs_ratchet = 1;
        }
    }


    if !is_buy {
        if let Some(actions) = plan_replace(&fp, &args.post)? {
            rec.actions = actions;
            rec.want_callback = true;
        }
    }


    let unspecified = args
        .before_delta_unspecified
        .checked_add(rec.delta_unspecified)
        .ok_or(HookError::MathOverflow)?;
    let credit = HookDeltaAssembly::assemble(p.amount_specified, p.zero_for_one, args.before_delta_specified, unspecified);
    assert_claims_cover(&fp, args.hook_claims0, args.hook_claims1, credit.amount0, credit.amount1)?;




    if floor_math::netflow::cushion_netflow_on() {
        let quote_leg = if quote_is_token1 { args.swap_delta.amount1 } else { args.swap_delta.amount0 };
        let v = u64::try_from(quote_leg.unsigned_abs()).unwrap_or(u64::MAX);
        let supply = if quote_is_token1 { args.supply0 } else { args.supply1 };
        drop(fp);
        crate::mech::netflow::note_account(&ctx.accounts.floor_pool.to_account_info(), is_buy, v, args.pre.tick, args.post.tick, supply)?;
    }

    emit_callback_events(&ctx, events)?;
    Ok(rec)
}
