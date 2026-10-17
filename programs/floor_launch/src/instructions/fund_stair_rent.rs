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

use crate::constants::{FLOOR_HOOK_PROGRAM, GRAD_RENT_SEED, LAUNCH_SEED};
use crate::errors::LaunchError;
use crate::events::StairRentFunded;
use crate::graduation_rent;
use crate::state::Launch;

#[event_cpi]
#[derive(Accounts)]
pub struct FundStairRent<'info> {
///
    #[account(mut)]
    pub caller: Signer<'info>,
    #[account(mut, seeds = [LAUNCH_SEED, launch.load()?.mint.as_ref()], bump = launch.load()?.bump)]
    pub launch: AccountLoader<'info, Launch>,
///
    #[account(mut, seeds = [GRAD_RENT_SEED, launch.key().as_ref()], bump)]
    pub graduation_rent: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub stair_rent: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<FundStairRent>) -> Result<()> {
    let launch_key = ctx.accounts.launch.key();
    let (amm_pool, budget) = {
        let l = ctx.accounts.launch.load()?;
        require!(l.state == floor_types::constants::LAUNCH_STATE_GRADUATED, LaunchError::CannotGraduate);
        (l.amm_pool, l.stair_rent_budget as u64)
    };
    require!(budget > 0, LaunchError::StairRentFunded);
    let (want, _) = Pubkey::find_program_address(&[floor_types::seeds::STAIR_RENT, amm_pool.as_ref()], &FLOOR_HOOK_PROGRAM);
    require_keys_eq!(ctx.accounts.stair_rent.key(), want, LaunchError::WrongStairRent);

    let reserve = ctx.accounts.graduation_rent.to_account_info();
    let rent = Rent::get()?;
    let amount = budget.min(graduation_rent::available(&reserve, &rent));


    let tip = floor_types::constants::STAIR_RENT_TIP_LAMPORTS.min(amount);
    let dest = ctx.accounts.stair_rent.to_account_info();
    let rest = amount - tip;
    let moved = if dest.lamports().saturating_add(rest) >= rent.minimum_balance(0) { rest } else { 0 };
    let tip = if moved > 0 { tip } else { 0 };
    graduation_rent::pay(&reserve, &ctx.accounts.caller.to_account_info(), tip)?;
    graduation_rent::pay(&reserve, &dest, moved)?;
    ctx.accounts.launch.load_mut()?.stair_rent_budget = 0;

    emit_cpi!(StairRentFunded { launch: launch_key, amm_pool, stair_rent: want, lamports: moved, tip });
    Ok(())
}
