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

#[event]
pub struct ScheduleCreated {
    pub schedule: Pubkey,
    pub funder: Pubkey,
    pub beneficiary: Pubkey,
    pub mint: Pubkey,
    pub escrow: Pubkey,
    pub nonce: u64,
///
///
    pub total: u64,
    pub start: i64,
    pub cliff: i64,
    pub end: i64,
}

#[event]
pub struct Claimed {
    pub schedule: Pubkey,
    pub beneficiary: Pubkey,
///
    pub amount: u64,
///
    pub claimed: u64,
    pub total: u64,
}

#[event]
pub struct ScheduleClosed {
    pub schedule: Pubkey,
    pub funder: Pubkey,
///
    pub rent_returned: u64,
///
///
///
///
///
    pub swept: u64,
}

///
///
///
///
#[event]
pub struct AbsoluteSchedule {
    pub schedule: Pubkey,
    pub start: i64,
    pub cliff: i64,
    pub end: i64,
    pub linear_after_cliff: bool,
}
