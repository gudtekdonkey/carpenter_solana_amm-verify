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
//!
//!
//!
//!

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{close_account, CloseAccount, Mint, TokenAccount, TokenInterface};

use crate::constants::{ESCROW, VESTING_AUTHORITY};
use crate::errors::VestingError;
use crate::events::ScheduleClosed;
use crate::mint_state;
use crate::schedule::VestingSchedule;
use crate::transfer::Transfer;

#[event_cpi]
#[derive(Accounts)]
pub struct CloseSchedule<'info> {
///
    pub signer: Signer<'info>,
///
///
    #[account(mut, close = funder)]
    pub schedule: Box<Account<'info, VestingSchedule>>,
///
    #[account(mut, address = schedule.funder)]
    pub funder: UncheckedAccount<'info>,
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
///
///
///
///
///
///
///
    #[account(
        mut,
        token::mint = mint,
        token::authority = schedule.beneficiary,
        token::token_program = token_program,
    )]
    pub beneficiary_ata: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
    #[account(address = schedule.token_program)]
    pub token_program: Interface<'info, TokenInterface>,
}

///
///
pub fn handler<'info>(ctx: Context<'info, CloseSchedule<'info>>) -> Result<()> {
    let s = &ctx.accounts.schedule;
    let signer = ctx.accounts.signer.key();
    require!(signer == s.funder || signer == s.beneficiary, VestingError::NotAParty);
    require!(s.claimed == s.total, VestingError::ScheduleNotFinished);

    let schedule_key = s.key();
    let funder = s.funder;
    let escrow_rent = ctx.accounts.escrow.to_account_info().lamports();
    let schedule_rent = ctx.accounts.schedule.to_account_info().lamports();

    let bump = [ctx.bumps.vesting_authority];
    let seeds: &[&[u8]] = &[VESTING_AUTHORITY, &bump];



    let swept = ctx.accounts.escrow.amount;
    if swept > 0 {



        let destination = ctx
            .accounts
            .beneficiary_ata
            .as_ref()
            .ok_or(VestingError::MissingBeneficiaryAccount)?
            .to_account_info();
        mint_state::require_not_paused(&ctx.accounts.mint.to_account_info())?;



        Transfer {
            token_program: ctx.accounts.token_program.key(),
            from: ctx.accounts.escrow.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            to: destination,
            authority: ctx.accounts.vesting_authority.to_account_info(),
            vesting_authority: ctx.accounts.vesting_authority.key(),
            remaining: ctx.remaining_accounts,
        }
        .run(swept, ctx.accounts.mint.decimals, &[seeds])?;
        ctx.accounts.escrow.reload()?;
    }

    close_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        CloseAccount {
            account: ctx.accounts.escrow.to_account_info(),
            destination: ctx.accounts.funder.to_account_info(),
            authority: ctx.accounts.vesting_authority.to_account_info(),
        },
        &[seeds],
    ))?;

    emit_cpi!(ScheduleClosed {
        schedule: schedule_key,
        funder,


        rent_returned: escrow_rent.checked_add(schedule_rent).ok_or(VestingError::MathOverflow)?,



        swept,
    });
    Ok(())
}
