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

use crate::constants::{ESCROW, SCHEDULE, VESTING_AUTHORITY};
use crate::errors::VestingError;
use crate::events::{AbsoluteSchedule, ScheduleCreated};
use crate::mint_state;
use crate::schedule::VestingSchedule;
use crate::transfer::Transfer;

#[event_cpi]
#[derive(Accounts)]
#[instruction(nonce: u64)]
pub struct CreateScheduleAt<'info> {
///
    #[account(mut)]
    pub payer: Signer<'info>,
///
    pub funder: Signer<'info>,
///
    pub beneficiary: UncheckedAccount<'info>,
    #[account(mint::token_program = token_program)]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init,
        payer = payer,
        space = VestingSchedule::LEN,
        seeds = [SCHEDULE, funder.key().as_ref(), &nonce.to_le_bytes()],
        bump,
    )]
    pub schedule: Box<Account<'info, VestingSchedule>>,
///
    #[account(seeds = [VESTING_AUTHORITY], bump)]
    pub vesting_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = payer,
        token::mint = mint,
        token::authority = vesting_authority,
        token::token_program = token_program,
        seeds = [ESCROW, schedule.key().as_ref()],
        bump,
    )]
    pub escrow: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, token::mint = mint, token::authority = funder, token::token_program = token_program)]
    pub funder_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(
    ctx: Context<'info, CreateScheduleAt<'info>>,
    nonce: u64,
    amount: u64,
    start: i64,
    cliff_seconds: u64,
    linear_seconds: u64,
) -> Result<()> {


    require!(amount > 0 && linear_seconds > 0 && start >= 0, VestingError::BadSchedule);
    require_keys_neq!(ctx.accounts.beneficiary.key(), Pubkey::default(), VestingError::BadBeneficiary);

    let cliff_offset = i64::try_from(cliff_seconds).map_err(|_| VestingError::BadSchedule)?;
    let linear_offset = i64::try_from(linear_seconds).map_err(|_| VestingError::BadSchedule)?;
    let cliff = start.checked_add(cliff_offset).ok_or(VestingError::BadSchedule)?;
    let end = cliff.checked_add(linear_offset).ok_or(VestingError::BadSchedule)?;
    mint_state::require_not_paused(&ctx.accounts.mint.to_account_info())?;


    let before = ctx.accounts.escrow.amount;
    Transfer {
        token_program: ctx.accounts.token_program.key(),
        from: ctx.accounts.funder_ata.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.escrow.to_account_info(),
        authority: ctx.accounts.funder.to_account_info(),
        vesting_authority: ctx.accounts.vesting_authority.key(),
        remaining: ctx.remaining_accounts,
    }
    .run(amount, ctx.accounts.mint.decimals, &[])?;


    ctx.accounts.escrow.reload()?;
    let total = ctx.accounts.escrow.amount.checked_sub(before).ok_or(VestingError::MathOverflow)?;
    require!(total > 0, VestingError::NothingEscrowed);


    let schedule_key = ctx.accounts.schedule.key();
    let s = &mut ctx.accounts.schedule;
    s.funder = ctx.accounts.funder.key();
    s.beneficiary = ctx.accounts.beneficiary.key();
    s.mint = ctx.accounts.mint.key();
    s.token_program = ctx.accounts.token_program.key();
    s.nonce = nonce;
    s.total = total;
    s.claimed = 0;
    s.start = start;
    s.cliff = cliff;
    s.end = end;
    s.bump = ctx.bumps.schedule;
    s.escrow_bump = ctx.bumps.escrow;
    s.linear_after_cliff = 1;
    s.reserved = [0u8; 61];

    emit_cpi!(ScheduleCreated {
        schedule: schedule_key,
        funder: s.funder,
        beneficiary: s.beneficiary,
        mint: s.mint,
        escrow: ctx.accounts.escrow.key(),
        nonce,
        total,
        start,
        cliff,
        end,
    });
    emit_cpi!(AbsoluteSchedule { schedule: schedule_key, start, cliff, end, linear_after_cliff: true });
    Ok(())
}
