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

use carpenter_clmm_math::big_num::U256;

///
pub const ACC_SHIFT: u32 = 64;

///
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Book {
///
    pub acc: u128,
///
    pub eligible: u64,
///
    pub indexed: u64,
///
    pub unallocated: u64,
///
    pub pending: u64,
///
    pub last_slot: u64,
}

///
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Record {
///
    pub shares: u64,
///
    pub snapshot: u128,
///
    pub owed: u64,
}

///
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Settled {
///
    pub earned: u64,
///
///
    pub clipped: u64,
}

///
///
///
pub fn lifetime(holder_owed: u64, holder_paid_eth: u64) -> u64 {
    holder_owed.saturating_add(holder_paid_eth)
}

///
fn mul_shr64_floor(a: u64, b: u128) -> U256 {
    (U256::from(a) * U256::from(b)) >> (ACC_SHIFT as usize)
}

///
fn mul_shr64_ceil(a: u128, b: u64) -> U256 {
    let p = U256::from(a) * U256::from(b);
    let q = p >> (ACC_SHIFT as usize);
    if (p & U256::from(u64::MAX)).is_zero() {
        q
    } else {
        q + U256::from(1u8)
    }
}

///
fn sat_u64(x: U256) -> u64 {
    if x > U256::from(u64::MAX) {
        u64::MAX
    } else {
        x.low_u64()
    }
}

///
///
///
///
pub fn catch_up_at(book: &mut Book, lifetime: u64, slot: u64) {
    if slot != book.last_slot {
        let p = book.pending;
        book.pending = 0;
        fold(book, p);
        book.last_slot = slot;
    }
    let delta = lifetime.saturating_sub(book.indexed);
    book.indexed = book.indexed.max(lifetime);
    book.pending = book.pending.saturating_add(delta);
}

///
///
///
///
///
///
///
///
pub fn catch_up(book: &mut Book, lifetime: u64) {
    let delta = lifetime.saturating_sub(book.indexed);
    book.indexed = book.indexed.max(lifetime);
    fold(book, delta);
}

///
fn fold(book: &mut Book, amount: u64) {
    let pot = book.unallocated.saturating_add(amount);
    if pot == 0 {
        return;
    }
    if book.eligible == 0 {
        book.unallocated = pot;
        return;
    }

    let inc = ((pot as u128) << ACC_SHIFT) / book.eligible as u128;
    if inc == 0 {


        book.unallocated = pot;
        return;
    }
    book.acc = book.acc.wrapping_add(inc);

    let released = sat_u64(mul_shr64_ceil(inc, book.eligible));
    book.unallocated = pot.saturating_sub(released);
}

///
///
pub fn open(book: &mut Book, balance: u64) -> Record {
    book.eligible = book.eligible.saturating_add(balance);
    Record { shares: balance, snapshot: book.acc, owed: 0 }
}

///
///
///
///
///
///
///
pub fn settle(book: &mut Book, rec: &mut Record, pre: u64, post: u64) -> Settled {
    let d = book.acc.wrapping_sub(rec.snapshot);
    let base = rec.shares.min(pre);
    let earned = sat_u64(mul_shr64_floor(base, d));
    let clipped = if rec.shares > base { sat_u64(mul_shr64_floor(rec.shares - base, d)) } else { 0 };
    rec.owed = rec.owed.saturating_add(earned);
    book.unallocated = book.unallocated.saturating_add(clipped);
    book.eligible = book.eligible.saturating_sub(rec.shares).saturating_add(post);
    rec.shares = post;
    rec.snapshot = book.acc;
    Settled { earned, clipped }
}

///
///
///
///
///
pub fn claimable(book: Book, rec: Record, lifetime: u64, live_balance: u64) -> u64 {
    let mut b = book;
    let mut r = rec;
    let next = b.last_slot.wrapping_add(1);
    catch_up_at(&mut b, lifetime, next);
    settle(&mut b, &mut r, live_balance, live_balance);
    r.owed
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE: u128 = 1 << 64;

    #[test]
    fn lifetime_saturates() {
        assert_eq!(lifetime(3, 4), 7);
        assert_eq!(lifetime(u64::MAX, 1), u64::MAX);
    }

///
    #[test]
    fn a_lone_holder_takes_the_whole_tax() {
        let mut b = Book::default();
        catch_up(&mut b, 0);
        let mut r = open(&mut b, 1_000);
        catch_up(&mut b, 777);
        assert_eq!(b.acc, 777 * ONE / 1_000);
        let s = settle(&mut b, &mut r, 1_000, 1_000);
        assert_eq!(s.earned, 776, "777/1000 per share floors: 1000 × floor(777·2^64/1000) / 2^64 = 776.99…");
        assert_eq!(b.unallocated, 0, "the ceiling of what acc released is the whole pot");
    }

///
    #[test]
    fn rounding_is_down_for_every_holder() {
        let mut b = Book::default();
        let mut a = open(&mut b, 1);
        let mut c = open(&mut b, 1);
        catch_up(&mut b, 1);
        let ea = settle(&mut b, &mut a, 1, 1).earned;
        let ec = settle(&mut b, &mut c, 1, 1).earned;
        assert_eq!(ea + ec, 0, "half a unit each floors to nothing — the unit stays on the pool");
    }

///
    #[test]
    fn a_pot_with_no_holder_parks_then_releases() {
        let mut b = Book::default();
        catch_up(&mut b, 500);
        assert_eq!((b.unallocated, b.acc, b.indexed), (500, 0, 500));
        let mut r = open(&mut b, 10);
        catch_up(&mut b, 500);
        assert_eq!(b.unallocated, 0);
        assert_eq!(settle(&mut b, &mut r, 10, 10).earned, 500);
    }

///
///
///
    #[allow(clippy::too_many_arguments)]
    fn transfer(b: &mut Book, lifetime: u64, slot: u64, from: Option<&mut Record>, to: Option<&mut Record>, from_post: u64, to_post: u64, amount: u64) {
        catch_up_at(b, lifetime, slot);
        if let Some(r) = from {
            settle(b, r, from_post + amount, from_post);
        }
        if let Some(r) = to {
            settle(b, r, to_post - amount, to_post);
        }
    }

///
    fn sync(b: &mut Book, lifetime: u64, slot: u64, r: &mut Record, live: u64) {
        catch_up_at(b, lifetime, slot);
        settle(b, r, live, live);
    }

///
///
///
///
///
///
///
///
    #[test]
    fn a_loan_inside_one_transaction_earns_none_of_its_tax() {
        const TAX: u64 = 1_000_000;
        const E: u64 = 300;
        const X: u64 = 693;
        const BOUGHT: u64 = 50;
        let mut b = Book::default();
        catch_up_at(&mut b, 0, 1);
        let mut h = open(&mut b, E);
        let mut a = open(&mut b, 0);

        transfer(&mut b, 0, 10, None, Some(&mut a), 0, X, X);

        transfer(&mut b, TAX, 10, None, Some(&mut a), 0, X + BOUGHT, BOUGHT);
        transfer(&mut b, TAX, 10, Some(&mut a), None, BOUGHT, 0, X);
        assert_eq!(b.pending, TAX, "the slot's tax waits for the slot to end");

        sync(&mut b, TAX, 11, &mut a, BOUGHT);
        sync(&mut b, TAX, 11, &mut h, E);



        assert_eq!(a.owed, TAX * BOUGHT / (E + BOUGHT), "only what A still held at the slot's end earned: not the loan");
        assert!(h.owed + a.owed >= TAX - 2 && h.owed + a.owed <= TAX, "the rest is H's, rounding down");
    }

///
///
    #[test]
    fn the_slot_end_holders_share_the_slots_tax() {
        let mut b = Book::default();
        catch_up_at(&mut b, 0, 1);
        let mut h = open(&mut b, 100);
        let mut buyer = open(&mut b, 0);
        let mut seller = open(&mut b, 100);

        transfer(&mut b, 1_000, 5, None, Some(&mut buyer), 0, 100, 100);
        transfer(&mut b, 2_000, 5, Some(&mut seller), None, 0, 0, 100);
        sync(&mut b, 2_000, 6, &mut h, 100);
        sync(&mut b, 2_000, 6, &mut buyer, 100);
        sync(&mut b, 2_000, 6, &mut seller, 0);
        assert_eq!(seller.owed, 0, "sold before the slot ended: none of the slot's tax");
        assert_eq!((h.owed, buyer.owed), (1_000, 1_000), "the two slot-end holders split both taxes");
    }

///
    #[test]
    fn nothing_folds_inside_the_slot_it_arrived_in() {
        let mut b = Book::default();
        catch_up_at(&mut b, 500, 1);
        catch_up_at(&mut b, 500, 2);
        assert_eq!((b.unallocated, b.pending), (500, 0), "parked: nobody held at slot 1's end");
        let mut r = open(&mut b, 10);
        catch_up_at(&mut b, 500, 2);
        assert_eq!(b.unallocated, 500, "a record opened in slot 2 does not take slot 2's fold");
        sync(&mut b, 500, 3, &mut r, 10);
        assert_eq!(r.owed, 500, "it takes the parked pot at the next slot");
    }

///
///
    #[test]
    fn claimable_is_the_real_settle_on_copies() {
        let mut b = Book::default();
        catch_up_at(&mut b, 0, 1);
        let mut r = open(&mut b, 3);
        catch_up_at(&mut b, 100, 1);
        let before = (b, r);
        let c = claimable(b, r, 150, 3);
        assert_eq!((b, r), before, "nothing written");
        catch_up_at(&mut b, 150, 2);
        settle(&mut b, &mut r, 3, 3);
        assert_eq!(c, r.owed);
        assert_eq!(c, 99, "slot 1's 100 over 3 shares floors to 99; the 50 waits for slot 2's end");
    }
}
