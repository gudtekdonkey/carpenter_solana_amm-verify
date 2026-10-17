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
use carpenter_types::constants::DYNAMIC_FEE_FLAG;
use carpenter_types::hook::{
    apply_specified_delta, effective_fee_override, BalanceDelta, FeeSplit, HookDeltaAssembly,
    HookRecordV1, PHASE_AFTER_ACTIONS, PHASE_AFTER_SWAP, PHASE_BEFORE_SWAP,
};

use crate::amm_core::hook_dispatch::{dispatch, map_record_error, HookCallContext};
use crate::errors::AmmError;

///
pub const fn is_record_phase(phase: u8) -> bool {
    matches!(phase, PHASE_BEFORE_SWAP | PHASE_AFTER_SWAP | PHASE_AFTER_ACTIONS)
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
pub fn call_record<A: AnchorSerialize>(ctx: &HookCallContext, phase: u8, args: &A) -> Result<HookRecordV1> {
    require!(is_record_phase(phase), AmmError::HookFieldNotAllowed);
    dispatch(ctx, phase, args)
}

///
///
pub fn amount_to_swap(amount_specified: i128, delta_specified: i128, settles: bool) -> Result<i128> {
    let to_swap = apply_specified_delta(amount_specified, delta_specified).map_err(map_record_error)?;








    require!(
        !settles || delta_specified >= 0 || to_swap.unsigned_abs() <= amount_specified.unsigned_abs(),
        AmmError::HookDeltaExceedsSwapAmount
    );
    Ok(to_swap)
}

///
///
///
///
pub fn settle_specified(amount_specified: i128, to_swap: i128, zero_for_one: bool, swap_delta: &BalanceDelta, after_specified: i128) -> Result<i128> {
    if after_specified == 0 {
        return Ok(0);
    }
    require!(after_specified > 0 && amount_specified < 0, AmmError::HookSettleOverfill);

    let consumed = if zero_for_one { swap_delta.amount0 } else { swap_delta.amount1 }.unsigned_abs();
    let unfilled = to_swap.unsigned_abs().checked_sub(consumed).ok_or(AmmError::MathOverflow)?;
    require!((after_specified as u128) <= unfilled, AmmError::HookSettleOverfill);
    Ok(after_specified)
}

///
///
pub fn taken(component: i128) -> u128 {
    if component > 0 {
        component as u128
    } else {
        0
    }
}

///
pub fn resolved_fee_override(rec: &HookRecordV1, flags: u16, pool_key_fee: u32) -> Option<u32> {
    effective_fee_override(rec, flags, pool_key_fee == DYNAMIC_FEE_FLAG)
}

///
pub fn accumulate_unspecified(unspec: i128, rec: &HookRecordV1) -> Result<i128> {
    unspec.checked_add(rec.delta_unspecified).ok_or_else(|| AmmError::MathOverflow.into())
}

///
///
///
///
///
///
pub fn hook_delta(amount_specified: i128, zero_for_one: bool, specified: i128, unspecified: i128, settles: bool) -> Result<BalanceDelta> {
    let d = HookDeltaAssembly::assemble(amount_specified, zero_for_one, specified, unspecified);
    require!(settles || (d.amount0 >= 0 && d.amount1 >= 0), AmmError::HookDeltaNegative);
    Ok(d)
}

///
pub fn trader_delta(swap_delta: &BalanceDelta, hook: &BalanceDelta) -> Result<BalanceDelta> {
    Ok(BalanceDelta {
        amount0: swap_delta.amount0.checked_sub(hook.amount0).ok_or(AmmError::MathOverflow)?,
        amount1: swap_delta.amount1.checked_sub(hook.amount1).ok_or(AmmError::MathOverflow)?,
    })
}

///
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TraderAmounts {
///
    pub amount_in: u64,
///
    pub amount_out: u64,
}

///
///
///
pub fn trader_amounts(trader: &BalanceDelta, zero_for_one: bool) -> Result<TraderAmounts> {
    let (d_in, d_out) = if zero_for_one { (trader.amount0, trader.amount1) } else { (trader.amount1, trader.amount0) };
    require!(d_in <= 0 && d_out >= 0, AmmError::HookDeltaExceedsSwapOutput);
    Ok(TraderAmounts {
        amount_in: u64::try_from(d_in.unsigned_abs()).map_err(|_| AmmError::MathOverflow)?,
        amount_out: u64::try_from(d_out).map_err(|_| AmmError::MathOverflow)?,
    })
}

///
pub fn check_limit(amount_specified: i128, amounts: &TraderAmounts, amount_limit: u64) -> Result<()> {
    if amount_specified < 0 {
        require!((amounts.amount_out as i128) >= amount_limit as i128, AmmError::SlippageExceeded);
    } else {
        require!((amounts.amount_in as i128) <= amount_limit as i128, AmmError::SlippageExceeded);
    }
    Ok(())
}





///
///
///
///
///
pub fn balance_above_keep(balance: u64, keep: u64) -> Result<u64> {
    require_gt!(balance, keep, AmmError::BalanceAtOrBelowKeep);
    Ok(balance - keep)
}

///
///
///
///
pub fn check_full_fill(amounts: &TraderAmounts, resolved: u64) -> Result<()> {
    require_eq!(amounts.amount_in, resolved, AmmError::BalanceSwapPartialFill);
    Ok(())
}

///
///
///
///
///
pub fn check_rate(amounts: &TraderAmounts, amount_limit: u64, limit_in_basis: u64) -> Result<()> {
    if limit_in_basis == 0 {
        return Ok(());
    }
    require!(
        (amounts.amount_out as u128) * (limit_in_basis as u128) >= (amount_limit as u128) * (amounts.amount_in as u128),
        AmmError::SlippageExceeded
    );
    Ok(())
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
///
///
pub fn swap_fees(amount_specified: i128, zero_for_one: bool, before: Option<&HookRecordV1>, after: Option<&HookRecordV1>) -> (FeeSplit, FeeSplit) {
    let b = before.map(HookRecordV1::fees).unwrap_or_default();
    let a = after.map(HookRecordV1::fees).unwrap_or_default();
    HookDeltaAssembly::assemble_fees(amount_specified, zero_for_one, b, a)
}

#[cfg(test)]
mod tests {
    use super::*;

///
///
    const FLOOR_HOOK_FLAGS: u16 = 0x60CC;

    #[test]
    fn a_before_delta_equal_to_the_amount_skips_the_walk() {
        assert_eq!(amount_to_swap(-1_000, 1_000, true).unwrap(), 0);
        assert_eq!(amount_to_swap(-1_000, 250, true).unwrap(), -750);
        assert!(amount_to_swap(-1_000, 1_001, true).is_err());
    }

    #[test]
    fn hook_delta_is_assembled_once_and_refuses_negative_components() {

        assert_eq!(hook_delta(-10, true, 3, 4, false).unwrap(), BalanceDelta { amount0: 3, amount1: 4 });

        assert_eq!(hook_delta(-10, false, 3, 4, false).unwrap(), BalanceDelta { amount0: 4, amount1: 3 });
        assert!(hook_delta(-10, true, 0, -1, false).is_err());

        assert_eq!(hook_delta(-10, true, 3, -4, true).unwrap(), BalanceDelta { amount0: 3, amount1: -4 });
    }

    #[test]
    fn the_trader_side_is_checked_signed() {


        let swap = BalanceDelta { amount0: -100, amount1: 50 };
        let td = trader_delta(&swap, &BalanceDelta { amount0: 0, amount1: 51 }).unwrap();
        assert_eq!(td, BalanceDelta { amount0: -100, amount1: -1 });
        assert!(trader_amounts(&td, true).is_err());
        let td = trader_delta(&swap, &BalanceDelta { amount0: 0, amount1: 50 }).unwrap();
        assert_eq!(trader_amounts(&td, true).unwrap(), TraderAmounts { amount_in: 100, amount_out: 0 });

        let swap = BalanceDelta { amount0: -70, amount1: 40 };
        let td = trader_delta(&swap, &BalanceDelta { amount0: 30, amount1: 0 }).unwrap();
        assert_eq!(trader_amounts(&td, true).unwrap(), TraderAmounts { amount_in: 100, amount_out: 40 });
    }

    #[test]
    fn limits_are_signed_per_mode() {
        let a = TraderAmounts { amount_in: 100, amount_out: 40 };
        assert!(check_limit(-100, &a, 40).is_ok());
        assert!(check_limit(-100, &a, 41).is_err());
        assert!(check_limit(40, &a, 100).is_ok());
        assert!(check_limit(40, &a, 99).is_err());
    }

///
///
    fn code<T: core::fmt::Debug>(r: Result<T>) -> u32 {
        match r.unwrap_err() {
            anchor_lang::error::Error::AnchorError(e) => e.error_code_number,
            other => panic!("expected an AnchorError, got {other:?}"),
        }
    }

///
///
///
///
    #[test]
    fn balance_above_keep_refuses_at_or_below_keep_and_never_wraps() {
        let below = u32::from(AmmError::BalanceAtOrBelowKeep);
        assert_eq!(balance_above_keep(1_000, 999).unwrap(), 1);
        assert_eq!(code(balance_above_keep(1_000, 1_000)), below, "keep == balance");
        assert_eq!(code(balance_above_keep(1_000, 1_001)), below, "keep > balance: no wrap");
        assert_eq!(code(balance_above_keep(0, 0)), below, "an empty account");
        assert_eq!(code(balance_above_keep(u64::MAX - 1, u64::MAX)), below, "keep = u64::MAX");
        assert_eq!(code(balance_above_keep(u64::MAX, u64::MAX)), below, "keep = balance = u64::MAX");
        assert_eq!(balance_above_keep(u64::MAX, 0).unwrap(), u64::MAX, "the whole account");
        assert_eq!(balance_above_keep(1, 0).unwrap(), 1, "one unit");
        assert_ne!(below, u32::from(AmmError::AmountZero), "never AmountZero");
    }

///
///
    #[test]
    fn a_balance_swap_must_spend_its_whole_resolved_input() {
        let a = TraderAmounts { amount_in: 100, amount_out: 40 };
        assert!(check_full_fill(&a, 100).is_ok());
        assert_eq!(code(check_full_fill(&a, 101)), u32::from(AmmError::BalanceSwapPartialFill), "one unit left behind");
        assert_eq!(code(check_full_fill(&a, 99)), u32::from(AmmError::BalanceSwapPartialFill), "more spent than resolved");
    }

///
///
///
///
    #[test]
    fn the_rate_clause_holds_every_unit_to_the_basis_rate() {
        let slip = u32::from(AmmError::SlippageExceeded);

        let at = TraderAmounts { amount_in: 1_010, amount_out: 505 };
        assert!(check_rate(&at, 500, 1_000).is_ok(), "exactly the rate");
        let short = TraderAmounts { amount_in: 1_010, amount_out: 504 };
        assert_eq!(code(check_rate(&short, 500, 1_000)), slip, "one unit under the rate");

        assert!(check_limit(-1_010, &short, 500).is_ok());

        assert!(check_rate(&short, 500, 0).is_ok(), "basis 0 is exact-in's check alone");

        let m = TraderAmounts { amount_in: 1_000, amount_out: 500 };
        assert!(check_rate(&m, 500, 1_000).is_ok());
        assert_eq!(code(check_rate(&TraderAmounts { amount_in: 1_000, amount_out: 499 }, 500, 1_000)), slip);

        let big = TraderAmounts { amount_in: u64::MAX, amount_out: u64::MAX };
        assert!(check_rate(&big, u64::MAX, u64::MAX).is_ok());
        assert_eq!(code(check_rate(&TraderAmounts { amount_in: u64::MAX, amount_out: u64::MAX - 1 }, u64::MAX, u64::MAX)), slip);
    }

    #[test]
    fn the_override_needs_a_dynamic_pool() {
        let rec = HookRecordV1 { fee_override_pips: Some(2_500), ..HookRecordV1::pass_through(1, PHASE_BEFORE_SWAP, 0) };
        assert_eq!(resolved_fee_override(&rec, FLOOR_HOOK_FLAGS, DYNAMIC_FEE_FLAG), Some(2_500));
        assert_eq!(resolved_fee_override(&rec, FLOOR_HOOK_FLAGS, 3_000), None);
    }



    #[test]
    fn a_negative_specified_delta_may_not_enlarge_the_walk() {

        assert!(amount_to_swap(-1_000, -500, true).is_err());
        assert_eq!(amount_to_swap(-1_000, 400, true).unwrap(), -600);

        assert_eq!(amount_to_swap(1_000, -400, true).unwrap(), 600);
        assert_eq!(amount_to_swap(1_000, 400, true).unwrap(), 1_400);
    }

    #[test]
    fn the_after_swap_fill_is_bounded_by_the_unfilled_exact_in() {

        let walked = BalanceDelta { amount0: -600, amount1: 900 };
        assert_eq!(settle_specified(-1_000, -1_000, true, &walked, 400).unwrap(), 400);
        assert_eq!(settle_specified(-1_000, -1_000, true, &walked, 0).unwrap(), 0);
        assert!(settle_specified(-1_000, -1_000, true, &walked, 401).is_err());
        assert!(settle_specified(-1_000, -1_000, true, &walked, -1).is_err());

        assert!(settle_specified(-1_000, -900, true, &walked, 301).is_err());
        assert_eq!(settle_specified(-1_000, -900, true, &walked, 300).unwrap(), 300);

        let walked1 = BalanceDelta { amount0: 900, amount1: -600 };
        assert_eq!(settle_specified(-1_000, -1_000, false, &walked1, 400).unwrap(), 400);

        assert!(settle_specified(1_000, 1_000, true, &walked, 1).is_err());
    }

    #[test]
    fn a_payout_is_never_counted_as_a_fee() {
        assert_eq!(taken(5), 5);
        assert_eq!(taken(0), 0);
        assert_eq!(taken(-5), 0);
    }
}
