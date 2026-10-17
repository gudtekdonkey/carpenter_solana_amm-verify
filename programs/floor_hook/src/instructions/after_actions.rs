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
//!
//!
//!
//!

use anchor_lang::prelude::*;

use crate::errors::HookError;
use crate::instructions::{emit_callback_events, CallbackEvent, HookCallback};
use crate::mech::fees::assert_claims_cover;
use crate::mech::lift::{
    amm_max_rounds, book_nudge, book_open_owed, book_open_plan, book_teardown, books_open_from_round_3, open_action,
    owed_from_result, plan_deploy, replaced_event, replica_owed, unbacked_event,
};
use crate::state::{AfterActionsArgs, HookRecordV1};

pub fn handler(ctx: Context<HookCallback>, args: AfterActionsArgs) -> Result<HookRecordV1> {
    let mut fp = ctx.accounts.floor_pool.load_mut()?;
    require_keys_eq!(ctx.accounts.pool_signer.key(), fp.amm_pool, HookError::NotAmmPool);
    require_keys_eq!(args.pool, fp.amm_pool, HookError::WrongPool);
    require!(fp.pool_bound == 1, HookError::NotBound);



    let budget = amm_max_rounds(&ctx.accounts.pool_signer.to_account_info());

    let mut rec = HookRecordV1 {
        version: args.version,
        phase: args.phase,
        round: args.round,
        ..Default::default()
    };

    let mut events = Vec::new();
    match args.round {
        2 => {


            require!(args.results.len() == 1 || args.results.len() == 2, HookError::InvariantViolated);
            let from_lower = fp.bid_lower;
            let (tokens_absorbed, eth_back) = book_teardown(&mut fp, &args.results[0], &mut events)?;
            if args.results.len() == 2 {
                book_nudge(&mut fp, &args.results[1])?;
            }
            let last = &args.results[args.results.len() - 1];
            if let Some(plan) = plan_deploy(&fp, last.post_sqrt_price_x64, last.post_tick)? {
                book_open_plan(&mut fp, &plan)?;
                if books_open_from_round_3(args.round, budget) {
                    rec.want_callback = true;
                } else {
                    book_open_owed(&mut fp, replica_owed(&plan)?)?;
                }
                rec.actions.push(open_action(&plan));
            } else {



                events.push(CallbackEvent::FloorUnbacked(unbacked_event(&fp)));
            }
            fp.needs_ratchet = 1;
            events.push(CallbackEvent::BidReplaced(replaced_event(&fp, tokens_absorbed, eth_back, from_lower, fp.bid_lower)));
        }
        3 => {


            require!(books_open_from_round_3(2, budget), HookError::InvariantViolated);
            require!(args.results.len() == 1, HookError::InvariantViolated);
            let owed = owed_from_result(&fp, &args.results[0])?;
            book_open_owed(&mut fp, owed)?;
        }
        _ => return err!(HookError::InvariantViolated),
    }

    assert_claims_cover(&fp, args.hook_claims0, args.hook_claims1, 0, 0)?;
    drop(fp);

    emit_callback_events(&ctx, events)?;
    Ok(rec)
}
