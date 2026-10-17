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

use crate::constants::LP_FEE_PIPS;
use crate::errors::HookError;
use crate::instructions::{emit_callback_events, HookCallback};
use crate::mech::fees::{assert_claims_cover, split_of, take_fees};
use crate::state::{twap, BeforeSwapArgs, HookRecordV1};

pub fn handler(ctx: Context<HookCallback>, args: BeforeSwapArgs) -> Result<HookRecordV1> {
    let fp = ctx.accounts.floor_pool.load()?;
    require_keys_eq!(ctx.accounts.pool_signer.key(), fp.amm_pool, HookError::NotAmmPool);
    require_keys_eq!(args.pool, fp.amm_pool, HookError::WrongPool);
    require!(fp.pool_bound == 1, HookError::NotBound);
















    require!(fp.ladder_seeded == 1, HookError::NotSeeded);











    drop(fp);
    twap::observe_account(&ctx.accounts.floor_pool.to_account_info(), twap::now_secs()?, args.pre.tick)?;

    let cushion_top = crate::mech::fees::cushion_top_of(&ctx.accounts.floor_pool.to_account_info())?;
    let mut fp = ctx.accounts.floor_pool.load_mut()?;

    let p = args.params;
    require!(p.amount_specified <= 0, HookError::ExactOutNotSupported);

    let quote_is_token1 = fp.quote_is_token1 != 0;
    let is_buy = p.zero_for_one != quote_is_token1;
    let quote_specified = is_buy == (p.amount_specified < 0);

    let mut rec = HookRecordV1 {
        version: args.version,
        phase: args.phase,
        round: args.round,
        fee_override_pips: Some(LP_FEE_PIPS),
        ..Default::default()
    };

    let mut events = Vec::new();
    if quote_specified {

        let spec = u64::try_from(p.amount_specified.unsigned_abs()).map_err(|_| error!(HookError::MathOverflow))?;

        fp.cumulative_volume_eth = fp.cumulative_volume_eth.saturating_add(spec);
        let take = take_fees(&mut fp, spec, is_buy, args.pre.tick, cushion_top, &mut events)?;
        rec.delta_specified = take.total as i128;




        rec.fees = split_of(&take);

        crate::mech::trader_ref::book(&mut fp, &ctx.accounts.trader_bucket.to_account_info(), &args.sender, take.team_fee, &mut events);
    }

    let credit = HookDeltaAssembly::assemble(p.amount_specified, p.zero_for_one, rec.delta_specified, rec.delta_unspecified);
    assert_claims_cover(&fp, args.hook_claims0, args.hook_claims1, credit.amount0, credit.amount1)?;
    drop(fp);
    emit_callback_events(&ctx, events)?;
    Ok(rec)
}
