//!
//!
//!
//!
//!

use anchor_lang::prelude::*;

#[error_code]
pub enum VestingError {
///
///
///
///
///
///
    #[msg("the schedule is not a schedule: zero amount, zero duration, or a cliff past the end")]
    BadSchedule,
///
    #[msg("the beneficiary is the zero address")]
    BadBeneficiary,
///
///
///
///
    #[msg("signer is not the beneficiary of this schedule")]
    NotBeneficiary,
///
    #[msg("nothing is claimable yet")]
    NothingToClaim,
///
///
    #[msg("the mint is paused; no token can move until it is resumed")]
    MintPaused,
///
///
///
///
///
    #[msg("the schedule still owes tokens; it cannot be closed")]
    ScheduleNotFinished,
///
    #[msg("the funding transfer delivered nothing to the escrow")]
    NothingEscrowed,
///
    #[msg("arithmetic overflow")]
    MathOverflow,
///
///
///
///
    #[msg("signer is neither the funder nor the beneficiary of this schedule")]
    NotAParty,
///
///
///
///
///
///
///
///
    #[msg("the escrow holds tokens and no beneficiary token account was given to sweep them to")]
    MissingBeneficiaryAccount,
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
    #[msg("the mint's transfer hook is armed and its accounts were not passed: the hook program and its extra-account-metas list")]
    HookAccountsMissing,
}
