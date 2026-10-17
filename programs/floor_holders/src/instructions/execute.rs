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
use floor_math::holders;
use floor_types::holders::{HolderBook, HolderRecord};

use crate::constants::{EXECUTE_ACCOUNTS, FLOOR_HOOK_PROGRAM};
use crate::errors::HoldersError;
use crate::{metas, state, token};

///
///
///
#[derive(Accounts)]
pub struct Execute<'info> {
///
    pub source: UncheckedAccount<'info>,
///
    pub mint: UncheckedAccount<'info>,
///
    pub destination: UncheckedAccount<'info>,
///
    pub authority: UncheckedAccount<'info>,
}

///
const R_METAS: usize = 0;
const R_BOOK: usize = 1;
const R_POOL: usize = 2;
const R_SENDER: usize = 3;
const R_RECEIVER: usize = 4;
const REMAINING: usize = EXECUTE_ACCOUNTS - 4;

pub fn handler(ctx: Context<Execute>, amount: u64) -> Result<()> {
    let rest = ctx.remaining_accounts;
    require!(rest.len() >= REMAINING, HoldersError::MissingExtraAccounts);
    let a = &ctx.accounts;
    let source = a.source.to_account_info();
    let destination = a.destination.to_account_info();
    let mint = a.mint.key();


    require!(
        token::is_transferring_of(&source, &mint) && token::is_transferring_of(&destination, &mint),
        HoldersError::NotTransferring
    );

    let metas_info = &rest[R_METAS];
    let book_info = &rest[R_BOOK];
    let pool_info = &rest[R_POOL];



    let slot = Clock::get()?.slot;


    let mut book: HolderBook = state::read_owned(book_info).ok_or(HoldersError::WrongMint)?;
    require!(book.mint == mint.to_bytes(), HoldersError::WrongMint);


    require_keys_eq!(*metas_info.owner, crate::ID, HoldersError::WrongMint);
    {
        let d = metas_info.try_borrow_data()?;
        require!(metas::named_floor_pool(&d) == Some(book.floor_pool), HoldersError::WrongMint);
    }
    require!(pool_info.key.to_bytes() == book.floor_pool, HoldersError::WrongPool);
    require_keys_eq!(*pool_info.owner, FLOOR_HOOK_PROGRAM, HoldersError::WrongPool);
    let lifetime = state::read_floor_pool(pool_info).map(|p| p.lifetime).ok_or(HoldersError::WrongPool)?;

    let mut t = state::tally_book(&book);
    holders::catch_up_at(&mut t, lifetime, slot);

    let sender_info = &rest[R_SENDER];
    let receiver_info = &rest[R_RECEIVER];
    let src_post = token::live_balance(&source, &mint);
    let dst_post = token::live_balance(&destination, &mint);

    if sender_info.key == receiver_info.key {

        settle_slot(&mut t, sender_info, &mint, source.key, src_post, src_post)?;
    } else {
        settle_slot(&mut t, sender_info, &mint, source.key, src_post.saturating_add(amount), src_post)?;
        settle_slot(&mut t, receiver_info, &mint, destination.key, dst_post.saturating_sub(amount), dst_post)?;
    }

    state::store_tally_book(&mut book, &t);
    state::write_owned(book_info, &book)?;
    Ok(())
}

///
///
fn settle_slot(
    t: &mut holders::Book,
    slot: &AccountInfo,
    mint: &Pubkey,
    token_account: &Pubkey,
    pre: u64,
    post: u64,
) -> Result<()> {
    let Some(mut rec) = state::read_owned::<HolderRecord>(slot) else { return Ok(()) };
    if rec.mint != mint.to_bytes() || rec.token_account != token_account.to_bytes() || !slot.is_writable {
        return Ok(());
    }
    let mut r = state::tally_record(&rec);
    holders::settle(t, &mut r, pre, post);
    state::store_tally_record(&mut rec, &r);
    state::write_owned(slot, &rec)
}
