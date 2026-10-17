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
//!
//!

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::{ESCROW, VESTING_AUTHORITY};
use crate::errors::VestingError;
use crate::events::Claimed;
use crate::mint_state;
use crate::schedule::VestingSchedule;
use crate::transfer::Transfer;

#[event_cpi]
#[derive(Accounts)]
pub struct Claim<'info> {
///
    pub beneficiary: Signer<'info>,
///
///
///
///
///
///
///
///
    #[account(mut, has_one = beneficiary @ VestingError::NotBeneficiary)]
    pub schedule: Box<Account<'info, VestingSchedule>>,
///
///
    #[account(address = schedule.mint, mint::token_program = token_program)]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
///
    #[account(seeds = [VESTING_AUTHORITY], bump)]
    pub vesting_authority: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [ESCROW, schedule.key().as_ref()],
        bump = schedule.escrow_bump,
        token::token_program = token_program,
    )]
    pub escrow: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, token::mint = mint, token::authority = beneficiary, token::token_program = token_program)]
    pub beneficiary_ata: Box<InterfaceAccount<'info, TokenAccount>>,
///
    #[account(address = schedule.token_program)]
    pub token_program: Interface<'info, TokenInterface>,
}

///
///
///
pub fn handler<'info>(ctx: Context<'info, Claim<'info>>) -> Result<()> {

    mint_state::require_not_paused(&ctx.accounts.mint.to_account_info())?;


    let now = Clock::get()?.unix_timestamp;
    let amount = ctx.accounts.schedule.claimable_at(now)?;
    require!(amount > 0, VestingError::NothingToClaim);


    let s = &mut ctx.accounts.schedule;
    s.claimed = s.claimed.checked_add(amount).ok_or(VestingError::MathOverflow)?;
    let (claimed, total) = (s.claimed, s.total);

    let bump = [ctx.bumps.vesting_authority];
    let seeds: &[&[u8]] = &[VESTING_AUTHORITY, &bump];
    Transfer {
        token_program: ctx.accounts.token_program.key(),
        from: ctx.accounts.escrow.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.beneficiary_ata.to_account_info(),
        authority: ctx.accounts.vesting_authority.to_account_info(),
        vesting_authority: ctx.accounts.vesting_authority.key(),
        remaining: ctx.remaining_accounts,
    }
    .run(amount, ctx.accounts.mint.decimals, &[seeds])?;

    emit_cpi!(Claimed {
        schedule: ctx.accounts.schedule.key(),
        beneficiary: ctx.accounts.beneficiary.key(),
        amount,
        claimed,
        total,
    });
    Ok(())
}
