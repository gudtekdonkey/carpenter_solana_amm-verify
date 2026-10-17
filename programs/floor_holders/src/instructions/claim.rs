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
use anchor_spl::token_interface::TokenAccount;
use floor_math::holders;

use crate::constants::{CARPENTER_AMM_PROGRAM, EVENT_AUTHORITY_SEED, FLOOR_HOOK_PROGRAM, HOLDERS_AUTHORITY_SEED, HOOK_AUTHORITY_SEED};
use crate::errors::HoldersError;
use crate::events::{HolderClaimed, RecordClosed};
use crate::hook_cpi::{self, PayHolderCpi};
use crate::{create, state, token};

#[event_cpi]
#[derive(Accounts)]
pub struct ClaimRewards<'info> {
///
    #[account(mut)]
    pub owner: Signer<'info>,
///
    pub mint: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub holder_book: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub floor_pool: UncheckedAccount<'info>,
///
    pub token_account: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub record: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub destination: Box<InterfaceAccount<'info, TokenAccount>>,
///
    #[account(seeds = [HOLDERS_AUTHORITY_SEED], bump)]
    pub holders_authority: UncheckedAccount<'info>,
///
    #[account(address = FLOOR_HOOK_PROGRAM)]
    pub floor_hook: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub pool: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub vault0: UncheckedAccount<'info>,
///
    pub quote_mint: UncheckedAccount<'info>,
///
    pub quote_token_program: UncheckedAccount<'info>,
///
    #[account(address = CARPENTER_AMM_PROGRAM)]
    pub carpenter_amm: UncheckedAccount<'info>,
///
    #[account(seeds = [HOOK_AUTHORITY_SEED], bump, seeds::program = FLOOR_HOOK_PROGRAM)]
    pub hook_authority: UncheckedAccount<'info>,
///
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = CARPENTER_AMM_PROGRAM)]
    pub amm_event_authority: UncheckedAccount<'info>,
///
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = FLOOR_HOOK_PROGRAM)]
    pub hook_event_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ClaimRewards>) -> Result<()> {
    settle(ctx, None)
}

pub fn authorize(ctx: Context<ClaimRewards>, amount: u64) -> Result<()> {
    use anchor_lang::solana_program::instruction::get_stack_height;
    require!(get_stack_height() == 1, HoldersError::InvalidPayoutAuthorization);
    let sysvar = ctx.remaining_accounts.iter().find(|a| *a.key == pubkey!("Sysvar1nstructions1111111111111111111111111"))
        .ok_or(HoldersError::InvalidPayoutAuthorization)?;
    let data = sysvar.try_borrow_data()?;
    require!(floor_types::payout::adjacent(&data, amount, false), HoldersError::InvalidPayoutAuthorization);
    drop(data);
    settle(ctx, Some(amount))
}

fn settle(ctx: Context<ClaimRewards>, authorized: Option<u64>) -> Result<()> {
    let a = &ctx.accounts;
    let mint = a.mint.key();
    let token_account = a.token_account.key();
    let owner = a.owner.key();
    let book_info = a.holder_book.to_account_info();
    let record_info = a.record.to_account_info();
    let pool_info = a.floor_pool.to_account_info();

    let mut book = state::load_book(&book_info, &mint)?;
    let pool = state::load_book_pool(&pool_info, &book)?;
    let mut rec = state::load_record(&record_info, &mint, &token_account)?;
    require!(rec.owner == owner.to_bytes(), HoldersError::NotRecordOwner);
    require_keys_eq!(a.destination.owner, owner, HoldersError::WrongDestination);
    require!(a.destination.mint.to_bytes() == book.quote_mint, HoldersError::WrongDestination);
    require_keys_eq!(a.pool.key(), pool.amm_pool, HoldersError::WrongPool);

    let live = token::live_balance(&a.token_account.to_account_info(), &mint);
    let mut t = state::tally_book(&book);

    holders::catch_up_at(&mut t, pool.lifetime, Clock::get()?.slot);
    let mut r = state::tally_record(&rec);
    holders::settle(&mut t, &mut r, live, live);

    let paid = r.owed;
    if let Some(amount) = authorized {
        require!(paid == amount, HoldersError::InvalidPayoutAuthorization);
    }
    if paid > 0 {
        r.owed = 0;
        rec.paid = rec.paid.saturating_add(paid);
        book.paid = book.paid.saturating_add(paid);
    }
    state::store_tally_record(&mut rec, &r);
    state::store_tally_book(&mut book, &t);
    let close = live == 0 && r.owed == 0;
    if close {
        book.records = book.records.saturating_sub(1);
    }

    state::write_owned(&record_info, &rec)?;
    state::write_owned(&book_info, &book)?;

    if paid > 0 {
        let cpi = PayHolderCpi {
            floor_hook: &a.floor_hook.to_account_info(),
            holders_authority: &a.holders_authority.to_account_info(),
            holders_authority_bump: ctx.bumps.holders_authority,
            floor_pool: &pool_info,
            pool: &a.pool.to_account_info(),
            vault0: &a.vault0.to_account_info(),
            destination: &a.destination.to_account_info(),
            quote_mint: &a.quote_mint.to_account_info(),
            quote_token_program: &a.quote_token_program.to_account_info(),
            carpenter_amm: &a.carpenter_amm.to_account_info(),
            hook_authority: &a.hook_authority.to_account_info(),
            amm_event_authority: &a.amm_event_authority.to_account_info(),
            hook_event_authority: &a.hook_event_authority.to_account_info(),
        };
        if authorized.is_none() { hook_cpi::pay_holder(&cpi, paid)?; }
        emit_cpi!(HolderClaimed { mint, record: a.record.key(), owner, destination: a.destination.key(), amount: paid });
    }

    if close {
        let refund = create::close_to(&record_info, &a.owner.to_account_info())?;
        emit_cpi!(RecordClosed { mint, record: a.record.key(), owner, refund_lamports: refund });
    }
    Ok(())
}
