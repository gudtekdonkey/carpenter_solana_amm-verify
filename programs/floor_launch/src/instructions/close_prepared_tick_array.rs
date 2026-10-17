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
use anchor_lang::solana_program::instruction::{AccountMeta, Instruction};
use anchor_lang::solana_program::program::invoke_signed;

use crate::constants::{CARPENTER_AMM_PROGRAM, EVENT_AUTHORITY_SEED, GRAD_RENT_SEED, LAUNCH_SEED};
use crate::errors::LaunchError;
use crate::graduation_rent;
use crate::state::Launch;

#[derive(Accounts)]
pub struct ClosePreparedTickArray<'info> {
///
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(seeds = [LAUNCH_SEED, launch.load()?.mint.as_ref()], bump = launch.load()?.bump)]
    pub launch: AccountLoader<'info, Launch>,
///
///
    #[account(mut, seeds = [GRAD_RENT_SEED, launch.key().as_ref()], bump)]
    pub graduation_rent: UncheckedAccount<'info>,
///
    pub pool: UncheckedAccount<'info>,
///
    pub bitmap: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub tick_array: UncheckedAccount<'info>,
///
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = CARPENTER_AMM_PROGRAM)]
    pub amm_event_authority: UncheckedAccount<'info>,
///
    #[account(address = CARPENTER_AMM_PROGRAM)]
    pub carpenter_amm: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<ClosePreparedTickArray>) -> Result<()> {
    let a = &ctx.accounts;
    let (creator, graduated) = {
        let l = a.launch.load()?;
        (l.creator, l.state == floor_types::constants::LAUNCH_STATE_GRADUATED)
    };
    require_keys_eq!(a.creator.key(), creator, LaunchError::NotLaunchCreator);

    require!(graduated, LaunchError::GraduationRentLocked);

    let reserve = a.graduation_rent.to_account_info();
    let before = reserve.lamports();
    let ix = Instruction {
        program_id: CARPENTER_AMM_PROGRAM,
        accounts: vec![
            AccountMeta::new(a.graduation_rent.key(), true),
            AccountMeta::new_readonly(a.pool.key(), false),
            AccountMeta::new_readonly(a.bitmap.key(), false),
            AccountMeta::new(a.tick_array.key(), false),
            AccountMeta::new_readonly(a.amm_event_authority.key(), false),
            AccountMeta::new_readonly(CARPENTER_AMM_PROGRAM, false),
        ],
        data: floor_types::disc::amm::CLOSE_TICK_ARRAY.to_vec(),
    };
    let launch_key = a.launch.key();
    let bump = [ctx.bumps.graduation_rent];
    let seeds: &[&[u8]] = &[GRAD_RENT_SEED, launch_key.as_ref(), &bump];
    invoke_signed(
        &ix,
        &[
            reserve.clone(),
            a.pool.to_account_info(),
            a.bitmap.to_account_info(),
            a.tick_array.to_account_info(),
            a.amm_event_authority.to_account_info(),
            a.carpenter_amm.to_account_info(),
        ],
        &[seeds],
    )?;

    let gained = reserve.lamports().checked_sub(before).ok_or_else(|| error!(LaunchError::MathOverflow))?;
    graduation_rent::pay(&reserve, &a.creator.to_account_info(), gained)
}
