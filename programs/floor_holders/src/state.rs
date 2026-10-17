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
use floor_math::holders::{Book, Record};
use floor_types::holders::{ClaimsRegistry, HolderBook, HolderRecord};
use floor_types::layout::{FloorPool, Layout};
use floor_types::U128;

use crate::constants::{FLOOR_HOOK_PROGRAM, FLOOR_POOL_ACCT_HOLDER_OWED, FLOOR_POOL_ACCT_HOLDER_PAID};



///
pub fn read_owned<T: Layout>(info: &AccountInfo) -> Option<T> {
    if *info.owner != crate::ID {
        return None;
    }
    let data = info.try_borrow_data().ok()?;
    if data.len() != T::ACCOUNT_LEN || data[..8] != T::DISCRIMINATOR {
        return None;
    }
    Some(bytemuck::pod_read_unaligned::<T>(&data[8..T::ACCOUNT_LEN]))
}

///
pub fn write_owned<T: Layout>(info: &AccountInfo, v: &T) -> Result<()> {
    let mut data = info.try_borrow_mut_data()?;
    require!(data.len() == T::ACCOUNT_LEN, anchor_lang::error::ErrorCode::AccountDidNotSerialize);
    data[8..T::ACCOUNT_LEN].copy_from_slice(bytemuck::bytes_of(v));
    Ok(())
}

///
pub fn init_owned<T: Layout>(info: &AccountInfo, v: &T) -> Result<()> {
    let mut data = info.try_borrow_mut_data()?;
    require!(data.len() == T::ACCOUNT_LEN, anchor_lang::error::ErrorCode::AccountDidNotSerialize);
    data[..8].copy_from_slice(&T::DISCRIMINATOR);
    data[8..T::ACCOUNT_LEN].copy_from_slice(bytemuck::bytes_of(v));
    Ok(())
}

///
pub fn tally_book(b: &HolderBook) -> Book {
    Book {
        acc: b.acc.get(),
        eligible: b.eligible,
        indexed: b.indexed,
        unallocated: b.unallocated,
        pending: b.pending,
        last_slot: b.last_slot,
    }
}

///
pub fn store_tally_book(b: &mut HolderBook, t: &Book) {
    b.acc = U128::new(t.acc);
    b.eligible = t.eligible;
    b.indexed = t.indexed;
    b.unallocated = t.unallocated;
    b.pending = t.pending;
    b.last_slot = t.last_slot;
}

///
pub fn tally_record(r: &HolderRecord) -> Record {
    Record { shares: r.shares, snapshot: r.snapshot.get(), owed: r.owed }
}

///
pub fn store_tally_record(r: &mut HolderRecord, t: &Record) {
    r.shares = t.shares;
    r.snapshot = U128::new(t.snapshot);
    r.owed = t.owed;
}

///
///
pub fn load_book(info: &AccountInfo, mint: &Pubkey) -> Result<HolderBook> {
    let book = read_owned::<HolderBook>(info).ok_or(crate::errors::HoldersError::NotHooked)?;
    require!(book.mint == mint.to_bytes(), crate::errors::HoldersError::WrongMint);
    Ok(book)
}

///
///
///
pub fn load_record(info: &AccountInfo, mint: &Pubkey, token_account: &Pubkey) -> Result<HolderRecord> {
    let rec = read_owned::<HolderRecord>(info).ok_or(crate::errors::HoldersError::NoRecord)?;
    require!(rec.mint == mint.to_bytes(), crate::errors::HoldersError::WrongMint);
    require!(rec.token_account == token_account.to_bytes(), crate::errors::HoldersError::NoRecord);
    Ok(rec)
}

///
pub fn read_registry(info: &AccountInfo) -> Option<ClaimsRegistry> {
    read_owned::<ClaimsRegistry>(info)
}



const FP_AMM_POOL: usize = 8 + core::mem::offset_of!(FloorPool, amm_pool);
const FP_BASE_MINT: usize = 8 + core::mem::offset_of!(FloorPool, base_mint);
const FP_QUOTE_MINT: usize = 8 + core::mem::offset_of!(FloorPool, quote_mint);


const _: () = assert!(8 + core::mem::offset_of!(FloorPool, holder_owed) == FLOOR_POOL_ACCT_HOLDER_OWED);
const _: () = assert!(8 + core::mem::offset_of!(FloorPool, holder_paid_eth) == FLOOR_POOL_ACCT_HOLDER_PAID);

///
#[derive(Clone, Copy, Debug)]
pub struct PoolView {
    pub amm_pool: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
///
    pub lifetime: u64,
}

fn key_at(d: &[u8], off: usize) -> Pubkey {
    let mut k = [0u8; 32];
    k.copy_from_slice(&d[off..off + 32]);
    Pubkey::new_from_array(k)
}

fn u64_at(d: &[u8], off: usize) -> u64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&d[off..off + 8]);
    u64::from_le_bytes(b)
}

///
///
///
///
///
///
///
///
pub fn read_floor_pool(info: &AccountInfo) -> Option<PoolView> {
    if *info.owner != FLOOR_HOOK_PROGRAM {
        return None;
    }
    let d = info.try_borrow_data().ok()?;
    pool_view(&d)
}

///
///
pub fn pool_view(d: &[u8]) -> Option<PoolView> {
    if d.len() < floor_types::twap::FLOOR_POOL_ACCOUNT_LEN_WITH_TWAP || d[..8] != FloorPool::DISCRIMINATOR {
        return None;
    }
    let owed = u64_at(&d, FLOOR_POOL_ACCT_HOLDER_OWED);
    let paid = u64_at(&d, FLOOR_POOL_ACCT_HOLDER_PAID);
    Some(PoolView {
        amm_pool: key_at(&d, FP_AMM_POOL),
        base_mint: key_at(&d, FP_BASE_MINT),
        quote_mint: key_at(&d, FP_QUOTE_MINT),
        lifetime: floor_math::holders::lifetime(owed, paid),
    })
}

///
///
pub fn load_book_pool(info: &AccountInfo, book: &HolderBook) -> Result<PoolView> {
    require!(info.key.to_bytes() == book.floor_pool, crate::errors::HoldersError::WrongPool);
    let p = read_floor_pool(info).ok_or(crate::errors::HoldersError::WrongPool)?;
    require!(p.base_mint.to_bytes() == book.mint, crate::errors::HoldersError::WrongPool);
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;

///
///
    #[test]
    fn the_tally_round_trips_through_the_layouts() {
        let mut b: HolderBook = bytemuck::Zeroable::zeroed();
        b.records = 7;
        b.paid = 9;
        let t = Book { acc: (3u128 << 64) | 5, eligible: 11, indexed: 13, unallocated: 17, pending: 19, last_slot: 23 };
        store_tally_book(&mut b, &t);
        assert_eq!(tally_book(&b), t);
        assert_eq!((b.records, b.paid), (7, 9), "a store writes the tally's six fields only");

        let mut r: HolderRecord = bytemuck::Zeroable::zeroed();
        r.paid = 21;
        let x = Record { shares: 19, snapshot: u128::MAX - 1, owed: 23 };
        store_tally_record(&mut r, &x);
        assert_eq!(tally_record(&r), x);
        assert_eq!(r.paid, 21);
    }

///
///
    #[test]
    fn the_raw_pool_reads_are_the_structs_fields() {
        let mut p: FloorPool = bytemuck::Zeroable::zeroed();
        p.amm_pool = [1; 32];
        p.base_mint = [2; 32];
        p.quote_mint = [3; 32];
        p.holder_owed = 40;
        p.holder_paid_eth = 2;
        let mut d = FloorPool::DISCRIMINATOR.to_vec();
        d.extend_from_slice(bytemuck::bytes_of(&p));
        assert_eq!(key_at(&d, FP_AMM_POOL).to_bytes(), [1; 32]);
        assert_eq!(key_at(&d, FP_BASE_MINT).to_bytes(), [2; 32]);
        assert_eq!(key_at(&d, FP_QUOTE_MINT).to_bytes(), [3; 32]);
        assert_eq!(u64_at(&d, FLOOR_POOL_ACCT_HOLDER_OWED) + u64_at(&d, FLOOR_POOL_ACCT_HOLDER_PAID), 42);
    }

///
///
    #[test]
    fn mr_s_a_floor_pool_with_the_slots_tail_is_read_and_a_short_one_is_not() {
        let mut p: FloorPool = bytemuck::Zeroable::zeroed();
        p.amm_pool = [1; 32];
        p.base_mint = [2; 32];
        p.quote_mint = [3; 32];
        p.holder_owed = 40;
        p.holder_paid_eth = 2;
        let mut d = FloorPool::DISCRIMINATOR.to_vec();
        d.extend_from_slice(bytemuck::bytes_of(&p));
        let twap = floor_types::twap::FLOOR_POOL_ACCOUNT_LEN_WITH_TWAP;
        let slots = floor_types::slots::FLOOR_POOL_ACCOUNT_LEN_WITH_SLOTS;
        assert_eq!((twap, slots), (1_008, 1_576));
        for len in [twap, slots, slots + 1] {
            let mut a = d.clone();
            a.resize(len, 0xAB);
            let v = pool_view(&a).unwrap_or_else(|| panic!("a {len}-byte FloorPool is read"));
            assert_eq!(v.amm_pool.to_bytes(), [1; 32]);
            assert_eq!(v.base_mint.to_bytes(), [2; 32]);
            assert_eq!(v.quote_mint.to_bytes(), [3; 32]);
            assert_eq!(v.lifetime, 42);
        }
        for len in [twap - 1, FloorPool::ACCOUNT_LEN, 8] {
            let mut a = d.clone();
            a.resize(len, 0);
            assert!(pool_view(&a).is_none(), "a {len}-byte account is refused");
        }
        let mut wrong = d.clone();
        wrong.resize(slots, 0);
        wrong[0] ^= 1;
        assert!(pool_view(&wrong).is_none(), "another discriminator is refused at any length");
    }
}
