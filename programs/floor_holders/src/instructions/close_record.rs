//!
//!
//!
//!
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
use floor_math::holders;

use crate::errors::HoldersError;
use crate::events::RecordClosed;
use crate::{create, state, token};

#[event_cpi]
#[derive(Accounts)]
pub struct CloseRecord<'info> {
///
    #[account(mut)]
    pub owner: Signer<'info>,
///
    pub mint: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub holder_book: UncheckedAccount<'info>,
///
    pub floor_pool: UncheckedAccount<'info>,
///
    pub token_account: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub record: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<CloseRecord>) -> Result<()> {
    let a = &ctx.accounts;
    let mint = a.mint.key();
    let token_account = a.token_account.key();
    let owner = a.owner.key();
    let book_info = a.holder_book.to_account_info();
    let record_info = a.record.to_account_info();

    let mut book = state::load_book(&book_info, &mint)?;
    let pool = state::load_book_pool(&a.floor_pool.to_account_info(), &book)?;
    let rec = state::load_record(&record_info, &mint, &token_account)?;
    require!(rec.owner == owner.to_bytes(), HoldersError::NotRecordOwner);

    let live = token::live_balance(&a.token_account.to_account_info(), &mint);
    let mut t = state::tally_book(&book);

    holders::catch_up_at(&mut t, pool.lifetime, Clock::get()?.slot);
    let mut r = state::tally_record(&rec);
    holders::settle(&mut t, &mut r, live, live);
    require!(live == 0 && r.owed == 0, HoldersError::RecordNotEmpty);

    state::store_tally_book(&mut book, &t);
    book.records = book.records.saturating_sub(1);
    state::write_owned(&book_info, &book)?;
    let refund = create::close_to(&record_info, &a.owner.to_account_info())?;
    emit_cpi!(RecordClosed { mint, record: a.record.key(), owner, refund_lamports: refund });
    Ok(())
}
