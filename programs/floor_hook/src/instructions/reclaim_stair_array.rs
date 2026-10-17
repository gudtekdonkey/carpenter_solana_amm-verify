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
use floor_types::layout::tick_array_start_index;

use crate::constants::{CARPENTER_AMM_PROGRAM, EVENT_AUTHORITY_SEED, STAIR_RENT_SEED};
use crate::errors::HookError;
use crate::events::StairArrayReclaimed;
use crate::mech::amm_cpi;
use crate::state::FloorPool;

#[derive(Accounts)]
pub struct ReclaimStairArray<'info> {
///
    #[account(mut)]
    pub cranker: Signer<'info>,
    pub floor_pool: AccountLoader<'info, FloorPool>,
///
    #[account(mut, seeds = [STAIR_RENT_SEED, floor_pool.load()?.amm_pool.as_ref()], bump)]
    pub stair_rent: UncheckedAccount<'info>,
///
    #[account(address = floor_pool.load()?.amm_pool @ HookError::WrongPool)]
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
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ReclaimStairArray>, start_index: i32) -> Result<()> {
    let a = &ctx.accounts;
    let amm_pool = a.pool.key();
    let p = amm_cpi::read_pool(&a.pool.to_account_info())?;
    let span = floor_types::constants::TICK_ARRAY_SIZE * p.tick_spacing as i32;
    let market = tick_array_start_index(p.tick_current, p.tick_spacing);
    require!((start_index - market).abs() > span, HookError::StairArrayHot);
    require_keys_eq!(a.tick_array.key(), amm_cpi::tick_array_key(&amm_pool, start_index), HookError::WrongPool);

    let before = a.stair_rent.lamports();
    let ix = Instruction {
        program_id: CARPENTER_AMM_PROGRAM,
        accounts: vec![
            AccountMeta::new(a.stair_rent.key(), true),
            AccountMeta::new_readonly(amm_pool, false),
            AccountMeta::new_readonly(a.bitmap.key(), false),
            AccountMeta::new(a.tick_array.key(), false),
            AccountMeta::new_readonly(a.amm_event_authority.key(), false),
            AccountMeta::new_readonly(CARPENTER_AMM_PROGRAM, false),
        ],
        data: floor_types::disc::amm::CLOSE_TICK_ARRAY.to_vec(),
    };
    let bump = [ctx.bumps.stair_rent];
    let seeds: &[&[u8]] = &[STAIR_RENT_SEED, amm_pool.as_ref(), &bump];
    invoke_signed(
        &ix,
        &[
            a.stair_rent.to_account_info(),
            a.pool.to_account_info(),
            a.bitmap.to_account_info(),
            a.tick_array.to_account_info(),
            a.amm_event_authority.to_account_info(),
            a.carpenter_amm.to_account_info(),
        ],
        &[seeds],
    )?;
    let reclaimed = a.stair_rent.lamports().saturating_sub(before);
    let spare = a.stair_rent.lamports().saturating_sub(Rent::get()?.minimum_balance(0));
    let tip = floor_types::constants::STAIR_RENT_TIP_LAMPORTS.min(reclaimed).min(spare);
    if tip > 0 {
        anchor_lang::system_program::transfer(
            CpiContext::new_with_signer(
                a.system_program.key(),
                anchor_lang::system_program::Transfer { from: a.stair_rent.to_account_info(), to: a.cranker.to_account_info() },
                &[seeds],
            ),
            tip,
        )?;
    }
    emit!(StairArrayReclaimed { pool: amm_pool, reserve: a.stair_rent.key(), start_index, lamports: reclaimed, tip });
    Ok(())
}
