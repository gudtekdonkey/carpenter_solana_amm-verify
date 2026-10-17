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
#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod mint_state;
pub mod schedule;
pub mod transfer;

use instructions::*;

declare_id!("HHiwbXs3egRwtgEPEKx1PHpBU8gq5VSbhfrGrcCd4CSC");






#[program]
pub mod floor_vesting {
    use super::*;

///
///
///
///
///
///
///
///
    pub fn create_schedule<'info>(
        ctx: Context<'info, CreateSchedule<'info>>,
        nonce: u64,
        amount: u64,
        cliff_seconds: u64,
        duration_seconds: u64,
    ) -> Result<()> {
        instructions::create_schedule::handler(ctx, nonce, amount, cliff_seconds, duration_seconds)
    }

///
///
///
///
///
///
    pub fn create_schedule_at<'info>(
        ctx: Context<'info, CreateScheduleAt<'info>>,
        nonce: u64,
        amount: u64,
        start: i64,
        cliff_seconds: u64,
        linear_seconds: u64,
    ) -> Result<()> {
        instructions::create_schedule_at::handler(ctx, nonce, amount, start, cliff_seconds, linear_seconds)
    }

///
///
///
///
///
///
///
    pub fn claim<'info>(ctx: Context<'info, Claim<'info>>) -> Result<()> {
        instructions::claim::handler(ctx)
    }

///
///
///
///
///
    pub fn close_schedule<'info>(ctx: Context<'info, CloseSchedule<'info>>) -> Result<()> {
        instructions::close_schedule::handler(ctx)
    }
}
