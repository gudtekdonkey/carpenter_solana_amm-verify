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
use floor_math::holders;

use crate::{state, token};

#[derive(Accounts)]
pub struct SyncRecord<'info> {
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

pub fn handler(ctx: Context<SyncRecord>) -> Result<()> {
    let a = &ctx.accounts;
    let mint = a.mint.key();
    let token_account = a.token_account.key();
    let book_info = a.holder_book.to_account_info();
    let record_info = a.record.to_account_info();

    let mut book = state::load_book(&book_info, &mint)?;
    let pool = state::load_book_pool(&a.floor_pool.to_account_info(), &book)?;
    let mut rec = state::load_record(&record_info, &mint, &token_account)?;
    let live = token::live_balance(&a.token_account.to_account_info(), &mint);

    let mut t = state::tally_book(&book);

    holders::catch_up_at(&mut t, pool.lifetime, Clock::get()?.slot);
    let mut r = state::tally_record(&rec);
    holders::settle(&mut t, &mut r, live, live);

    state::store_tally_record(&mut rec, &r);
    state::write_owned(&record_info, &rec)?;
    state::store_tally_book(&mut book, &t);
    state::write_owned(&book_info, &book)
}
