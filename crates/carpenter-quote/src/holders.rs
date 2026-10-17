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

use floor_types::holders::{HolderBook, HolderRecord};
use floor_types::layout::FloorPool;

use crate::error::{QuoteError, QuoteResult};
use crate::mech;
use crate::state::decode_account;

///
///
pub fn holder_claimable(book_data: &[u8], record_data: &[u8], floor_pool_data: &[u8], live_balance: u64) -> QuoteResult<u64> {
    let book: HolderBook = decode_account(book_data)?;
    let rec: HolderRecord = decode_account(record_data)?;
    let fp: FloorPool = crate::state::decode_floor_pool(floor_pool_data)?;
    if book.version != 1 {
        return Err(QuoteError::QuoteBadAccount("HolderBook"));
    }
    if rec.version != 1 {
        return Err(QuoteError::QuoteBadAccount("HolderRecord"));
    }
    if rec.mint != book.mint || fp.base_mint != book.mint || fp.quote_mint != book.quote_mint {
        return Err(QuoteError::WrongMint);
    }
    if !fp.is_hooked() {
        return Err(QuoteError::NotHooked);
    }

    Ok(mech::holder_claimable(&book, &rec, &fp, live_balance))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::encode_account;
    use bytemuck::Zeroable;
    use floor_math::holders::{catch_up, catch_up_at, claimable, open, settle, Book, Record};

    const MINT: [u8; 32] = [0x11; 32];
    const QUOTE: [u8; 32] = [0x22; 32];

    fn accounts(book: Book, rec: Record, owed: u64, paid: u64) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        let mut hb = HolderBook::zeroed();
        hb.version = 1;
        hb.mint = MINT;
        hb.quote_mint = QUOTE;
        hb.acc.set(book.acc);
        hb.eligible = book.eligible;
        hb.indexed = book.indexed;
        hb.unallocated = book.unallocated;
        hb.pending = book.pending;
        hb.last_slot = book.last_slot;
        let mut hr = HolderRecord::zeroed();
        hr.version = 1;
        hr.mint = MINT;
        hr.shares = rec.shares;
        hr.snapshot.set(rec.snapshot);
        hr.owed = rec.owed;
        let mut fp = FloorPool::zeroed();
        fp.base_mint = MINT;
        fp.quote_mint = QUOTE;
        fp.holders_hooked = 1;
        fp.holder_owed = owed;
        fp.holder_paid_eth = paid;
        (encode_account(&hb), encode_account(&hr), encode_account(&fp))
    }

///
///
///
    #[test]
    fn the_account_bytes_quote_is_the_programs_arithmetic() {
        let mut b = Book::default();
        let mut r1 = open(&mut b, 3_000);
        let _r2 = open(&mut b, 1_000);
        catch_up(&mut b, 4_000);
        settle(&mut b, &mut r1, 3_000, 3_000);


        let (book, rec, pool) = accounts(b, r1, 4_500, 500);
        let got = holder_claimable(&book, &rec, &pool, 3_000).unwrap();
        assert_eq!(got, claimable(b, r1, 5_000, 3_000));
        assert_eq!(got, 3_000, "3/4 of the 4_000 folded; the un-indexed 1_000 waits for a slot's end");


        let mut b2 = b;
        let same_slot = b2.last_slot;
        catch_up_at(&mut b2, 5_000, same_slot);
        assert_eq!(b2.pending, 1_000);
        let (book2, rec2, pool2) = accounts(b2, r1, 4_500, 500);
        let got2 = holder_claimable(&book2, &rec2, &pool2, 3_000).unwrap();
        assert_eq!(got2, claimable(b2, r1, 5_000, 3_000));
        assert_eq!(got2, 3_750, "3/4 of 5_000: 3_000 settled at the first fold, 750 from last slot's tax");


        let zero = holder_claimable(&book, &rec, &pool, 0).unwrap();
        assert_eq!(zero, r1.owed);
    }

    #[test]
    fn three_accounts_of_different_launches_are_refused_by_name() {
        let mut b = Book::default();
        let r = open(&mut b, 1);
        let (book, rec, pool) = accounts(b, r, 0, 0);

        let mut hr: HolderRecord = decode_account(&rec).unwrap();
        hr.mint = [0x33; 32];
        assert_eq!(holder_claimable(&book, &encode_account(&hr), &pool, 1), Err(QuoteError::WrongMint));

        let mut fp: FloorPool = decode_account(&pool).unwrap();
        fp.quote_mint = [0x44; 32];
        assert_eq!(holder_claimable(&book, &rec, &encode_account(&fp), 1), Err(QuoteError::WrongMint));

        let mut fp: FloorPool = decode_account(&pool).unwrap();
        fp.holders_hooked = 0;
        assert_eq!(holder_claimable(&book, &rec, &encode_account(&fp), 1), Err(QuoteError::NotHooked));

        assert_eq!(holder_claimable(&rec, &rec, &pool, 1), Err(QuoteError::QuoteBadAccount("HolderBook")));
        assert_eq!(holder_claimable(&book, &rec, &pool[..100], 1), Err(QuoteError::QuoteBadAccount("FloorPool")));
    }
}
