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

use floor_math::slots::{check_slots, slot_cliff_part, slot_due, slot_earned, Slot};

fn oracle(l: u64, bps: u16) -> u64 {
    ((l as u128 * bps.min(10_000) as u128) / 10_000) as u64
}

///
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

///
fn random_split(r: &mut Rng, n: usize) -> Vec<u16> {
    let mut cuts: Vec<u16> = (0..n - 1).map(|_| 1 + r.below(9_999) as u16).collect();
    cuts.sort_unstable();
    cuts.dedup();
    while cuts.len() < n - 1 {
        let c = 1 + r.below(9_999) as u16;
        if !cuts.contains(&c) {
            cuts.push(c);
            cuts.sort_unstable();
        }
    }
    let mut out = Vec::with_capacity(n);
    let mut prev = 0u16;
    for c in cuts.iter().copied().chain(std::iter::once(10_000)) {
        out.push(c - prev);
        prev = c;
    }
    assert_eq!(out.iter().map(|&b| b as u32).sum::<u32>(), 10_000);
    assert!(out.iter().all(|&b| b > 0));
    out
}

#[test]
fn mr_review_earned_is_the_spec_oracle_over_edges() {
    let ls = [0u64, 1, 2, 9_999, 10_000, 10_001, 1 << 63, u64::MAX - 1, u64::MAX];
    for &l in &ls {
        for bps in 0..=10_000u16 {
            assert_eq!(slot_earned(l, bps), oracle(l, bps), "L={l} bps={bps}");
        }

        assert_eq!(slot_earned(l, 10_000), l);

        for bps in [10_001u16, 20_000, u16::MAX] {
            assert_eq!(slot_earned(l, bps), l, "clamped at L={l} bps={bps}");
        }
    }
}

///
fn pv1_rows() -> Vec<(u64, u16, u64)> {
    let src = include_str!("../fixtures/mr-s-pv1.json");
    let rows = &src[src.find("\"rows\"").expect("rows")..];
    let mut out = Vec::new();
    let mut i = 0;
    let b = rows.as_bytes();
    while let Some(o) = rows[i..].find('[') {
        let start = i + o + 1;
        let Some(c) = rows[start..].find(']') else { break };
        let inner = &rows[start..start + c];
        i = start + c + 1;
        let nums: Vec<&str> = inner.split(',').map(|s| s.trim().trim_matches('"')).collect();
        if nums.len() != 3 || nums.iter().any(|s| s.is_empty() || !s.bytes().all(|x| x.is_ascii_digit())) {
            continue;
        }
        out.push((nums[0].parse().unwrap(), nums[1].parse().unwrap(), nums[2].parse().unwrap()));
        let _ = b;
    }
    out
}

#[test]
fn mr_review_pv1_fixture_reread_independently() {
    let rows = pv1_rows();
    assert!(rows.len() >= 90, "the fixture's rows were read ({} found)", rows.len());
    assert!(rows.iter().any(|r| r.0 == u64::MAX), "the u64::MAX row is in the vector");
    assert!(rows.iter().any(|r| r.1 == 10_000), "a one-slot (10,000) row is in the vector");
    for (l, bps, e) in rows {
        assert_eq!(oracle(l, bps), e, "the FIXTURE disagrees with the spec's own formula at L={l} bps={bps}");
        assert_eq!(slot_earned(l, bps), e, "slot_earned disagrees with MR-S-PV1 at L={l} bps={bps}");
    }
}

///
///
struct Book {
    owed: u64,
    paid: u64,
    slot_paid: Vec<u64>,
    bps: Vec<u16>,
}
impl Book {
    fn lifetime(&self) -> u64 {
        self.paid + self.owed
    }
    fn claim(&mut self, i: usize) -> u64 {
        let due = slot_due(self.lifetime(), self.bps[i], self.slot_paid[i], self.owed);
        assert!(due <= self.owed, "a claim above owed");
        self.owed -= due;
        self.paid += due;
        self.slot_paid[i] += due;
        due
    }
}

#[test]
fn mr_review_random_books_conserve_and_leave_dust_below_n() {
    let mut r = Rng(0xD1B5_4A32_D192_ED03);
    for case in 0..400 {
        let n = 1 + (case % 5) as usize;
        let bps = random_split(&mut r, n);

        let pending: Vec<bool> = (0..n).map(|_| r.below(4) == 0).collect();
        let mut b = Book { owed: 0, paid: 0, slot_paid: vec![0; n], bps: bps.clone() };
        let scale = [1u64, 7, 1_000, 1_000_000_007, 1 << 40][r.below(5) as usize];
        for _ in 0..300 {
            b.owed += r.below(scale) + r.below(3);
            for _ in 0..r.below(n as u64 + 1) {
                let i = r.below(n as u64) as usize;
                if !pending[i] {
                    b.claim(i);
                }
            }
            let l = b.lifetime();
            assert_eq!(b.slot_paid.iter().sum::<u64>(), b.paid, "Σ slot.paid is the pool's paid");
            for i in 0..n {
                assert!(b.slot_paid[i] <= oracle(l, bps[i]), "slot {i} above its earned share");
            }
        }

        for _ in 0..2 {
            for i in 0..n {
                b.claim(i);
            }
        }
        let l = b.lifetime();
        for i in 0..n {
            assert_eq!(b.slot_paid[i], oracle(l, bps[i]), "case {case}: slot {i} holds exactly earned(L)");
            assert_eq!(b.claim(i), 0, "nothing left for slot {i}");
        }
        assert!(b.owed <= n as u64 - 1, "case {case}: dust {} with {n} slots at L={l} (bps {bps:?})", b.owed);
        if n == 1 {
            assert_eq!(b.owed, 0, "one slot at 10,000 leaves no dust (D2)");
        }
    }
}

#[test]
fn mr_review_due_saturates_caps_and_never_panics() {
    let m = u64::MAX;

    assert_eq!(slot_due(100, 5_000, 51, 1_000), 0);
    assert_eq!(slot_due(0, 10_000, m, m), 0);

    assert_eq!(slot_due(m, 10_000, 0, 0), 0);

    assert_eq!(slot_due(m, 10_000, 0, 5), 5);
    assert_eq!(slot_due(m, 5_000, 0, m), m / 2);

    assert_eq!(slot_due(m, 1, 0, m), m / 10_000);
    for bps in [0u16, 1, 3_333, 9_999, 10_000, 10_001, u16::MAX] {
        for (l, p, o) in [(0, 0, 0), (m, m, m), (m, 0, m), (1, m, 1), (m, m - 1, 1)] {
            let d = slot_due(l, bps, p, o);
            assert!(d <= o);
            assert_eq!(d, oracle(l, bps).saturating_sub(p).min(o));
        }
    }
}

#[test]
fn mr_review_cliff_parts_partition_the_vested_whole() {
    let mut r = Rng(0x9E37_79B9_7F4A_7C15);
    for _ in 0..500 {
        let total = r.below(1 << 50) + 1;
        let k = 1 + r.below(6);

        let mut v: Vec<u64> = (0..k - 1).map(|_| r.below(total + 1)).collect();
        v.sort_unstable();
        v.insert(0, 0);
        v.push(total);
        let n = 1 + r.below(5) as usize;
        let bps = random_split(&mut r, n);
        let mut sum_slots = 0u64;
        for &b in &bps {

            let s: u64 = (1..v.len()).map(|j| slot_cliff_part(v[j - 1], v[j], b)).sum();
            assert_eq!(s, oracle(total, b));

            let mid = v[r.below(v.len() as u64) as usize];
            assert_eq!(slot_cliff_part(0, mid, b) + slot_cliff_part(mid, total, b), oracle(total, b));
            sum_slots += s;
        }
        assert!(total - sum_slots <= n as u64 - 1, "vesting dust ≤ n − 1");
    }
}

#[test]
fn mr_review_check_codes_first_failing_rule_wins() {
    let w = |b: u8, bps: u16| Slot { bps, platform: 0, key: [b; 32] };
    let h = |b: u8, bps: u16, p: u8| Slot { bps, platform: p, key: [b; 32] };

    assert_eq!(check_slots(&[w(1, 0), w(2, 2_000), w(3, 2_000), w(4, 2_000), w(5, 2_000), w(6, 2_000)]), 1);

    assert_eq!(check_slots(&[w(0, 0), w(2, 5_000)]), 2);

    assert_eq!(check_slots(&[h(1, 5_000, 9), w(2, 4_000)]), 3);

    assert_eq!(check_slots(&[h(1, 2_000, 5), w(2, 4_000), w(2, 4_000)]), 4);

    assert_eq!(check_slots(&[w(2, 2_500), w(2, 2_500), h(3, 2_500, 1), h(3, 2_500, 2)]), 5);

    assert_eq!(check_slots(&[h(3, 5_000, 1), h(3, 5_000, 4)]), 6);

    assert_eq!(check_slots(&[w(7, 5_000), h(7, 5_000, 1)]), 0);

    assert_eq!(check_slots(&[w(1, 2_000), h(2, 2_000, 1), w(3, 2_000), h(4, 2_000, 2), w(5, 2_000)]), 0);

    assert_eq!(check_slots(&[w(9, 10_000)]), 0);

    assert_eq!(check_slots(&[w(1, 60_000), w(2, 15_536)]), 3);
}
