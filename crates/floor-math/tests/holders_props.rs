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
use floor_math::holders::{catch_up, claimable, lifetime, open, settle, Book, Record, ACC_SHIFT};



struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {

        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 {
        if n == 0 {
            0
        } else {
            self.next() % n
        }
    }
///
    fn spread(&mut self, bits: u32) -> u64 {
        let b = self.below(bits as u64 + 1) as u32;
        if b == 0 {
            0
        } else {
            self.next() >> (64 - b)
        }
    }
}



#[derive(Clone)]
struct Holder {
    balance: u64,
    rec: Option<Record>,
    paid: u64,
}

#[derive(Clone)]
struct World {
    book: Book,
///
    holder_owed: u64,
///
    holder_paid: u64,
    holders: Vec<Holder>,
///
    settles: u64,
    folds: u64,
}

impl World {
    fn new(n: usize) -> Self {
        World {
            book: Book::default(),
            holder_owed: 0,
            holder_paid: 0,
            holders: vec![Holder { balance: 0, rec: None, paid: 0 }; n],
            settles: 0,
            folds: 0,
        }
    }
    fn lifetime(&self) -> u64 {
        lifetime(self.holder_owed, self.holder_paid)
    }
    fn fold(&mut self) {
        let lt = self.lifetime();
        catch_up(&mut self.book, lt);
        self.folds += 1;
    }
    fn settle_holder(&mut self, h: usize, pre: u64, post: u64) {
        if let Some(mut r) = self.holders[h].rec {
            settle(&mut self.book, &mut r, pre, post);
            self.holders[h].rec = Some(r);
            self.settles += 1;
        }
    }
///
    fn tax(&mut self, t: u64) {
        self.holder_owed = self.holder_owed.saturating_add(t);
    }
///
    fn open(&mut self, h: usize) {
        if self.holders[h].rec.is_some() {
            self.fold();
            let b = self.holders[h].balance;
            self.settle_holder(h, b, b);
            return;
        }
        self.fold();
        let r = open(&mut self.book, self.holders[h].balance);
        self.holders[h].rec = Some(r);
    }
///
    fn transfer(&mut self, from: usize, to: usize, amount: u64) {
        let amount = amount.min(self.holders[from].balance);
        if from == to {
            self.fold();
            let b = self.holders[from].balance;
            self.settle_holder(from, b, b);
            return;
        }
        self.holders[from].balance -= amount;
        self.holders[to].balance += amount;
        self.fold();
        let (fb, tb) = (self.holders[from].balance, self.holders[to].balance);
        self.settle_holder(from, fb + amount, fb);
        self.settle_holder(to, tb - amount, tb);
    }
///
    fn burn_external(&mut self, h: usize, amount: u64) {
        let amount = amount.min(self.holders[h].balance);
        self.holders[h].balance -= amount;
    }
///
    fn burn_floor(&mut self, h: usize, amount: u64) {
        let amount = amount.min(self.holders[h].balance);
        self.sync(h);
        self.holders[h].balance -= amount;
        self.sync(h);
    }
    fn sync(&mut self, h: usize) {
        self.fold();
        let b = self.holders[h].balance;
        self.settle_holder(h, b, b);
    }
///
    fn claim(&mut self, h: usize) -> u64 {
        if self.holders[h].rec.is_none() {
            return 0;
        }
        self.sync(h);
        let mut r = self.holders[h].rec.unwrap();
        let paid = r.owed;

        assert!(paid <= self.holder_owed, "a claim of {paid} exceeds holder_owed {}", self.holder_owed);
        r.owed = 0;
        self.holder_owed -= paid;
        self.holder_paid += paid;
        self.holders[h].paid += paid;
        self.holders[h].rec = Some(r);

        if self.holders[h].balance == 0 {
            self.close(h);
        }
        paid
    }
///
    fn close(&mut self, h: usize) {
        if let Some(r) = self.holders[h].rec {
            if self.holders[h].balance == 0 && r.owed == 0 {
                self.sync(h);
                let r = self.holders[h].rec.unwrap();
                assert_eq!(r.shares, 0, "a closable record holds no shares");
                self.holders[h].rec = None;
            }
        }
    }

///
    fn assert_conserved(&self, seed: u64, step: usize) {
        let one = U256::from(1u128 << ACC_SHIFT);
        let mut lhs = U256::from(self.book.unallocated) * one;
        let mut shares_sum: u128 = 0;
        for h in &self.holders {
            lhs = lhs + U256::from(h.paid) * one;
            if let Some(r) = h.rec {
                lhs = lhs + U256::from(r.owed) * one;
                lhs = lhs + U256::from(r.shares) * U256::from(self.book.acc.wrapping_sub(r.snapshot));
                shares_sum += r.shares as u128;
            }
        }
        let rhs = U256::from(self.book.indexed) * one;
        assert!(lhs <= rhs, "seed {seed:#x} step {step}: accounted {lhs} > indexed·2^64 {rhs}");
        assert!(self.book.indexed <= self.lifetime(), "seed {seed:#x} step {step}: indexed past lifetime");
        assert_eq!(self.book.eligible as u128, shares_sum, "seed {seed:#x} step {step}: eligible is Σ shares");
        let paid: u64 = self.holders.iter().map(|h| h.paid).sum();
        assert_eq!(paid, self.holder_paid, "seed {seed:#x} step {step}: Σ paid is holder_paid_eth");
        assert!(paid <= self.lifetime(), "seed {seed:#x} step {step}: T-SMT-01 Σ paid > lifetime");
    }
}

fn random_history(seed: u64, steps: usize) -> World {
    let mut rng = Rng(seed);
    let n = 2 + rng.below(6) as usize;
    let mut w = World::new(n);

    for h in 0..n {
        w.holders[h].balance = rng.spread(50);
        if rng.below(3) != 0 {
            w.open(h);
        }
    }
    w.assert_conserved(seed, 0);
    for step in 1..=steps {
        match rng.below(10) {
            0..=2 => {

                w.tax(rng.spread(40));
                let (a, b) = (rng.below(n as u64) as usize, rng.below(n as u64) as usize);
                let amt = rng.spread(50);
                w.transfer(a, b, amt);
            }
            3 => {
                let (a, b) = (rng.below(n as u64) as usize, rng.below(n as u64) as usize);
                let amt = rng.spread(50);
                w.transfer(a, b, amt);
            }
            4 => w.tax(rng.spread(40)),
            5 => {
                let h = rng.below(n as u64) as usize;
                let amt = rng.spread(50);
                if rng.below(2) == 0 {
                    w.burn_external(h, amt);
                } else {
                    w.burn_floor(h, amt);
                }
            }
            6 => {
                let h = rng.below(n as u64) as usize;
                w.claim(h);
            }
            7 => {
                let h = rng.below(n as u64) as usize;
                w.open(h);
            }
            8 => {
                let h = rng.below(n as u64) as usize;
                w.sync(h);
            }
            _ => {
                let h = rng.below(n as u64) as usize;
                w.close(h);
            }
        }
        w.assert_conserved(seed, step);
    }
    w
}

///
#[test]
fn t_smt_01_03_random_histories_conserve_and_never_overdraw() {
    for i in 0..2_000u64 {
        let seed = 0xF100_0000_0000_0000 ^ i.wrapping_mul(0x9E37_79B9);
        random_history(seed, 60);
    }
}

///
///
#[test]
fn t_smt_03_final_dust_is_bounded_and_stays_on_the_pool() {
    for i in 0..500u64 {
        let seed = 0xD057_0000_0000_0000 ^ i;
        let mut w = random_history(seed, 40);


        for _ in 0..3 {
            for h in 0..w.holders.len() {
                w.claim(h);
            }
        }
        w.assert_conserved(seed, usize::MAX);
        let owed_left: u64 = w.holders.iter().filter_map(|h| h.rec.map(|r| r.owed)).sum();
        assert_eq!(owed_left, 0, "seed {seed:#x}: every recorded holder claimed");
        let pending_units: u64 = w
            .holders
            .iter()
            .filter_map(|h| h.rec)
            .map(|r| {
                let p = (U256::from(r.shares) * U256::from(w.book.acc.wrapping_sub(r.snapshot))) >> (ACC_SHIFT as usize);
                p.low_u64()
            })
            .sum();




        let accounted = w.book.unallocated + pending_units;
        assert!(accounted <= w.holder_owed, "seed {seed:#x}: more promised than the pool holds");
        let dust = w.holder_owed - accounted;
        let bound = 2 * w.settles + w.folds + w.holders.len() as u64;
        assert!(dust <= bound, "seed {seed:#x}: dust {dust} > bound {bound}");
    }
}

///
///
#[test]
fn t_smt_02_a_burner_never_gains() {
    for i in 0..2_000u64 {
        let mut rng = Rng(0xB0B0_0000_0000_0000 ^ i);
        let n = 2 + rng.below(4) as usize;
        let mut base = World::new(n);
        for h in 0..n {
            base.holders[h].balance = 1 + rng.spread(45);
            base.open(h);
        }
        let s0 = base.holders[0].balance;
        let others: u64 = base.holders[1..].iter().map(|h| h.balance).sum();
        let t1 = rng.spread(40);
        let t2 = rng.spread(40);
        let burn = 1 + rng.below(s0);
        let mut worlds = [base.clone(), base.clone(), base];
        for (k, w) in worlds.iter_mut().enumerate() {
            w.tax(t1);
            w.fold();
            match k {
                0 => w.burn_external(0, burn),
                1 => w.burn_floor(0, burn),
                _ => {}
            }
            w.tax(t2);
            w.fold();
        }
        let got: Vec<u64> = worlds.iter_mut().map(|w| w.claim(0)).collect();


        let unburned = ((t1 as u128 + t2 as u128) * s0 as u128) / (s0 as u128 + others as u128);
        assert!(got[0] as u128 <= unburned, "case {i}: an external burner gained: {} > {unburned}", got[0]);


        assert!(got[0] <= got[1] + 1, "case {i}: external burn {} > FLOOR burn {}", got[0], got[1]);
        assert!(got[1] <= got[2] + 1, "case {i}: FLOOR burn {} > no burn {}", got[1], got[2]);



        for w in worlds.iter_mut() {
            for _ in 0..3 {
                for h in 0..n {
                    w.claim(h);
                }
            }
        }
        let total = |w: &World| w.holders.iter().map(|h| h.paid).sum::<u64>();
        let slack = 4 * n as u64 + 8;
        let (ext, flr) = (total(&worlds[0]), total(&worlds[1]));
        assert!(ext + slack >= flr, "case {i}: the burn clip was lost, not returned ({ext} vs {flr})");
        assert!(ext <= t1 + t2 && flr <= t1 + t2, "case {i}: paid past the tax");
    }
}

///
#[test]
fn t_smt_04_a_buyer_shares_its_own_buys_tax_only_on_its_pre_buy_balance() {
    let mut rng = Rng(0xB0_0000_0000);
    for _ in 0..2_000 {

        let mut w = World::new(3);
        w.holders[0].balance = 1_000_000_000 + rng.spread(50);
        w.holders[1].balance = rng.spread(40);
        w.holders[2].balance = 1 + rng.spread(40);
        w.open(1);
        w.open(2);
        let (pre, other) = (w.holders[1].balance, w.holders[2].balance);
        let tax = rng.spread(40);
        let bought = 1 + rng.spread(40);
        w.tax(tax);
        w.transfer(0, 1, bought);
        let owed = w.holders[1].rec.unwrap().owed;

        let exact = (tax as u128 * pre as u128) / (pre as u128 + other as u128);
        assert!(owed as u128 <= exact, "the buyer earned past its pre-buy share: {owed} > {exact}");
        assert!(owed as u128 + 2 >= exact, "the buyer lost more than rounding: {owed} vs {exact}");
    }
}

///
#[test]
fn t_smt_05_a_seller_shares_its_own_sells_tax_on_the_tokens_it_sells() {
    let mut rng = Rng(0x5E11_0000);
    for _ in 0..2_000 {
        let mut w = World::new(3);
        w.holders[0].balance = rng.spread(50);
        w.holders[1].balance = 1 + rng.spread(40);
        w.holders[2].balance = 1 + rng.spread(40);
        w.open(1);
        w.open(2);
        let (pre, other) = (w.holders[1].balance, w.holders[2].balance);
        let sold = 1 + rng.below(pre);
        let tax = rng.spread(40);
        w.tax(tax);
        w.transfer(1, 0, sold);
        let owed = w.holders[1].rec.unwrap().owed;
        let exact = (tax as u128 * pre as u128) / (pre as u128 + other as u128);
        assert!(owed as u128 <= exact);
        assert!(owed as u128 + 2 >= exact, "the seller lost its share on the sold tokens: {owed} vs {exact}");
        assert_eq!(w.holders[1].rec.unwrap().shares, pre - sold, "it holds what it kept");
    }
}

///
#[test]
fn t_smt_06_a_pot_with_no_eligible_holder_parks_then_releases() {
    let mut rng = Rng(0x0A5C);
    for _ in 0..1_000 {
        let mut w = World::new(2);
        w.holders[0].balance = 1 + rng.spread(50);
        let parked = rng.spread(50);
        w.tax(parked);
        w.fold();
        assert_eq!(w.book.unallocated, parked);
        assert_eq!(w.book.acc, 0, "nothing released with nobody to release to");
        w.open(0);
        assert_eq!(w.book.unallocated, parked);
        let paid = w.claim(0);
        assert_eq!(w.book.unallocated, 0, "the whole pot is released");
        assert!(paid <= parked && parked - paid <= 1, "the lone holder takes the pot, less < 1 unit: {paid} of {parked}");
    }
}

///
///
///
#[test]
fn t_smt_07_repeated_folds_with_no_new_tax_never_raise_a_claim() {

    let mut b = Book::default();
    let r: Vec<Record> = (0..3).map(|_| open(&mut b, 1)).collect();
    catch_up(&mut b, 1);
    let first: Vec<u64> = r.iter().map(|x| claimable(b, *x, 1, 1)).collect();
    for _ in 0..1_000 {
        catch_up(&mut b, 1);
    }
    let after: Vec<u64> = r.iter().map(|x| claimable(b, *x, 1, 1)).collect();
    assert_eq!(first, after, "a fold with no new tax changed a claim");
    assert!(after.iter().sum::<u64>() <= 1, "three shares claim more than the one unit of tax");

    let mut rng = Rng(0x7777);
    for _ in 0..500 {
        let mut b = Book::default();
        let recs: Vec<(Record, u64)> = (0..1 + rng.below(5))
            .map(|_| {
                let s = 1 + rng.spread(30);
                (open(&mut b, s), s)
            })
            .collect();
        let lt = rng.spread(40);
        catch_up(&mut b, lt);
        let sum = |b: &Book| recs.iter().map(|(r, s)| claimable(*b, *r, lt, *s)).sum::<u64>();
        let before = sum(&b);
        for _ in 0..50 {
            catch_up(&mut b, lt);
        }
        assert_eq!(sum(&b), before);
        assert!(before <= lt);
    }
}

///
///
#[test]
fn t_smt_08_one_raw_share_and_a_wrapping_acc_stay_exact() {


    let mut b = Book::default();
    let mut r = open(&mut b, 1);
    let mut lt: u64 = 0;
    let mut paid: u64 = 0;
    for k in 0..8u64 {
        let before = lt;
        lt = lt.saturating_add(u64::MAX / 4 - k);
        catch_up(&mut b, lt);
        let s = settle(&mut b, &mut r, 1, 1);
        assert_eq!(s.earned, lt - before, "fold {k}: one share takes the whole fold");
        paid = paid.saturating_add(r.owed);
        r.owed = 0;
    }
    assert_eq!(paid, u64::MAX, "the whole saturated lifetime, to the unit");




    let big = 1u64 << 63;
    let mut b = Book::default();
    let mut a = open(&mut b, 1);
    catch_up(&mut b, big);
    assert_eq!(b.acc, 1u128 << 127);
    let s = settle(&mut b, &mut a, 0, 0);
    assert_eq!((s.earned, s.clipped), (0, big));
    assert_eq!((b.unallocated, b.eligible), (big, 0));
    let mut r = open(&mut b, 1);
    catch_up(&mut b, big);
    assert_eq!(b.acc, 0, "acc wrapped: 2^127 + 2^127 = 2^128 ≡ 0 (mod 2^128)");
    let s = settle(&mut b, &mut r, 1, 1);
    assert_eq!(s.earned, big, "across the wrap, exact: the whole lifetime, and A got none of it");


    let mut b = Book::default();
    let mut r = open(&mut b, u64::MAX);
    catch_up(&mut b, u64::MAX);
    catch_up(&mut b, u64::MAX);
    let s = settle(&mut b, &mut r, u64::MAX, 0);
    assert_eq!(s.earned, u64::MAX);
    assert_eq!(b.eligible, 0);

    let mut b = Book { acc: u128::MAX, eligible: u64::MAX, indexed: 0, unallocated: 0, ..Book::default() };
    let mut r = Record { shares: u64::MAX, snapshot: 0, owed: u64::MAX - 1 };
    let s = settle(&mut b, &mut r, u64::MAX, u64::MAX);
    assert_eq!((s.earned, r.owed), (u64::MAX, u64::MAX), "saturated, never wrapped");
    assert_eq!(b.eligible, u64::MAX);
}
