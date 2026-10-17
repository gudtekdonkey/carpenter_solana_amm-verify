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
use floor_types::layout::{Layout, TraderBucket};

use crate::events::TraderReferralBooked;
use crate::instructions::CallbackEvent;
use crate::state::FloorPool;

///
///
///
pub fn book(fp: &mut FloorPool, bucket: &AccountInfo, sender: &Pubkey, tfee: u64, events: &mut Vec<CallbackEvent>) {
    if tfee == 0 || !bucket.is_writable || bucket.owner != &crate::ID {
        return;
    }

    let Ok(mut data) = bucket.try_borrow_mut_data() else { return };
    if data.len() < TraderBucket::ACCOUNT_LEN || data[..8] != TraderBucket::DISCRIMINATOR {
        return;
    }

    let mut b: TraderBucket = bytemuck::pod_read_unaligned(&data[8..TraderBucket::ACCOUNT_LEN]);
    if b.trader != sender.to_bytes() || b.quote_mint != fp.quote_mint.to_bytes() {
        return;
    }
    const ZERO: [u8; 32] = [0; 32];
    let s = floor_math::trader_ref::trader_ref_slices(tfee, [b.tier1 != ZERO, b.tier2 != ZERO, b.tier3 != ZERO]);
    let Some(sum) = s[0].checked_add(s[1]).and_then(|x| x.checked_add(s[2])) else { return };
    if sum == 0 {
        return;
    }


    let (Some(o0), Some(o1), Some(o2)) = (b.owed[0].checked_add(s[0]), b.owed[1].checked_add(s[1]), b.owed[2].checked_add(s[2])) else {
        return;
    };
    if fp.team_owed.checked_sub(sum).is_none()
        || fp.trader_ref_owed.checked_add(sum).is_none()
        || fp.trader_ref_booked.checked_add(sum).is_none()
    {
        return;
    }
    fp.team_owed -= sum;
    fp.trader_ref_owed += sum;
    fp.trader_ref_booked += sum;
    b.owed = [o0, o1, o2];
    data[8..TraderBucket::ACCOUNT_LEN].copy_from_slice(bytemuck::bytes_of(&b));
    drop(data);
    events.push(CallbackEvent::TraderReferralBooked(TraderReferralBooked { pool: fp.amm_pool, trader: *sender, s }));
}

#[cfg(test)]
mod tests {
//!
//!
//!
//!
    use super::*;

    const QUOTE: Pubkey = Pubkey::new_from_array([7; 32]);
    const TRADER: Pubkey = Pubkey::new_from_array([1; 32]);

    fn pool() -> FloorPool {
        let mut fp: FloorPool = bytemuck::Zeroable::zeroed();
        fp.quote_mint = QUOTE;
        fp.amm_pool = Pubkey::new_from_array([9; 32]);
        fp.team_owed = 1_000_000;
        fp
    }

    fn bucket(trader: Pubkey, quote: Pubkey, tiers: [[u8; 32]; 3]) -> Vec<u8> {
        let mut b: TraderBucket = bytemuck::Zeroable::zeroed();
        b.trader = trader.to_bytes();
        b.quote_mint = quote.to_bytes();
        (b.tier1, b.tier2, b.tier3) = (tiers[0], tiers[1], tiers[2]);
        let mut d = TraderBucket::DISCRIMINATOR.to_vec();
        d.extend_from_slice(bytemuck::bytes_of(&b));
        d
    }

    fn owed_of(data: &[u8]) -> [u64; 3] {
        bytemuck::pod_read_unaligned::<TraderBucket>(&data[8..TraderBucket::ACCOUNT_LEN]).owed
    }

///
    fn run(mut data: Vec<u8>, owner: Pubkey, writable: bool, sender: Pubkey, tfee: u64) -> (FloorPool, Vec<u8>) {
        let key = Pubkey::new_from_array([3; 32]);
        let mut lamports = 1u64;
        let mut fp = pool();
        {
            let ai = AccountInfo::new(&key, false, writable, &mut lamports, &mut data, &owner, false);
            book(&mut fp, &ai, &sender, tfee, &mut Vec::new());
        }
        (fp, data)
    }

    const ALL: [[u8; 32]; 3] = [[2; 32], [3; 32], [4; 32]];

    #[test]
    fn a_referred_leg_books_30_5_2_and_moves_team_owed_into_trader_ref_owed() {
        let (fp, data) = run(bucket(TRADER, QUOTE, ALL), crate::ID, true, TRADER, 10_001);

        assert_eq!(owed_of(&data), [3_000, 500, 200]);
        assert_eq!(fp.team_owed, 1_000_000 - 3_700);
        assert_eq!((fp.trader_ref_owed, fp.trader_ref_booked), (3_700, 3_700));

        assert_eq!(fp.team_owed + fp.trader_ref_owed, 1_000_000);
    }

    #[test]
    fn missing_tiers_stay_with_floor() {
        let one = [[2; 32], [0; 32], [0; 32]];
        let (fp, data) = run(bucket(TRADER, QUOTE, one), crate::ID, true, TRADER, 10_000);
        assert_eq!(owed_of(&data), [3_000, 0, 0]);
        assert_eq!(fp.team_owed, 1_000_000 - 3_000);
        let two = [[2; 32], [3; 32], [0; 32]];
        let (_, data) = run(bucket(TRADER, QUOTE, two), crate::ID, true, TRADER, 10_000);
        assert_eq!(owed_of(&data), [3_000, 500, 0]);

        let gap = [[2; 32], [0; 32], [4; 32]];
        let (_, data) = run(bucket(TRADER, QUOTE, gap), crate::ID, true, TRADER, 10_000);
        assert_eq!(owed_of(&data), [3_000, 0, 0]);
    }

    #[test]
    fn every_invalid_bucket_books_nothing() {
        let good = || bucket(TRADER, QUOTE, ALL);
        let cases: [(&str, Vec<u8>, Pubkey, bool, Pubkey); 7] = [
            ("foreign trader", bucket(Pubkey::new_from_array([5; 32]), QUOTE, ALL), crate::ID, true, TRADER),
            ("another sender", good(), crate::ID, true, Pubkey::new_from_array([5; 32])),
            ("wrong quote", bucket(TRADER, Pubkey::new_from_array([8; 32]), ALL), crate::ID, true, TRADER),
            ("foreign owner", good(), Pubkey::new_from_array([6; 32]), true, TRADER),
            ("system-owned (an uninitialised PDA)", vec![], Pubkey::default(), true, TRADER),
            ("read-only meta", good(), crate::ID, false, TRADER),
            ("short account", good()[..TraderBucket::ACCOUNT_LEN - 1].to_vec(), crate::ID, true, TRADER),
        ];
        for (name, data, owner, writable, sender) in cases {
            let before = data.clone();
            let (fp, after) = run(data, owner, writable, sender, 10_000);
            assert_eq!(fp.team_owed, 1_000_000, "{name}: team_owed moved");
            assert_eq!((fp.trader_ref_owed, fp.trader_ref_booked), (0, 0), "{name}");
            assert_eq!(after, before, "{name}: the bucket was written");
        }
        let mut bad_disc = good();
        bad_disc[0] ^= 1;
        let (fp, _) = run(bad_disc, crate::ID, true, TRADER, 10_000);
        assert_eq!(fp.trader_ref_owed, 0, "wrong discriminator");

        let (fp, _) = run(good(), crate::ID, true, TRADER, 3);
        assert_eq!(fp.trader_ref_owed, 0);
    }

    #[test]
    fn an_overflow_anywhere_writes_nothing() {
        let mut d = bucket(TRADER, QUOTE, ALL);
        let mut b: TraderBucket = bytemuck::pod_read_unaligned(&d[8..TraderBucket::ACCOUNT_LEN]);
        b.owed[2] = u64::MAX;
        d[8..TraderBucket::ACCOUNT_LEN].copy_from_slice(bytemuck::bytes_of(&b));
        let (fp, after) = run(d.clone(), crate::ID, true, TRADER, 10_000);
        assert_eq!((fp.team_owed, fp.trader_ref_owed), (1_000_000, 0));
        assert_eq!(after, d);
    }
}
