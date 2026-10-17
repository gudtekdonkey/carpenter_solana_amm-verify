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

use crate::constants::{ESCROW, SCHEDULE, VESTING_AUTHORITY};
use crate::errors::VestingError;
use crate::events::ScheduleCreated;
use crate::mint_state;
use crate::schedule::VestingSchedule;
use crate::transfer::Transfer;

#[event_cpi]
#[derive(Accounts)]
#[instruction(nonce: u64)]
pub struct CreateSchedule<'info> {
///
    #[account(mut)]
    pub funder: Signer<'info>,
///
///
    pub beneficiary: UncheckedAccount<'info>,
    #[account(mint::token_program = token_program)]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init,
        payer = funder,
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
        payer = funder,
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

///
///
pub fn handler<'info>(
    ctx: Context<'info, CreateSchedule<'info>>,
    nonce: u64,
    amount: u64,
    cliff_seconds: u64,
    duration_seconds: u64,
) -> Result<()> {

    require!(
        amount > 0 && duration_seconds > 0 && cliff_seconds <= duration_seconds,
        VestingError::BadSchedule
    );
    require_keys_neq!(ctx.accounts.beneficiary.key(), Pubkey::default(), VestingError::BadBeneficiary);


    let cliff_offset = i64::try_from(cliff_seconds).map_err(|_| VestingError::BadSchedule)?;
    let duration_offset = i64::try_from(duration_seconds).map_err(|_| VestingError::BadSchedule)?;
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


    let now = Clock::get()?.unix_timestamp;
    let cliff = now.checked_add(cliff_offset).ok_or(VestingError::MathOverflow)?;
    let end = now.checked_add(duration_offset).ok_or(VestingError::MathOverflow)?;
    let schedule_key = ctx.accounts.schedule.key();
    let s = &mut ctx.accounts.schedule;
    s.funder = ctx.accounts.funder.key();
    s.beneficiary = ctx.accounts.beneficiary.key();
    s.mint = ctx.accounts.mint.key();
    s.token_program = ctx.accounts.token_program.key();
    s.nonce = nonce;
    s.total = total;
    s.claimed = 0;
    s.start = now;
    s.cliff = cliff;
    s.end = end;
    s.bump = ctx.bumps.schedule;
    s.escrow_bump = ctx.bumps.escrow;
    s.linear_after_cliff = 0;
    s.reserved = [0u8; 61];

    emit_cpi!(ScheduleCreated {
        schedule: schedule_key,
        funder: s.funder,
        beneficiary: s.beneficiary,
        mint: s.mint,
        escrow: ctx.accounts.escrow.key(),
        nonce,
        total,
        start: now,
        cliff,
        end,
    });
    Ok(())
}
