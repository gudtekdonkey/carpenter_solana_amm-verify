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

///
///
///
///
///
///
///
///
///
///
///
#[account]
#[derive(Debug)]
pub struct VestingSchedule {
///
    pub funder: Pubkey,
///
    pub beneficiary: Pubkey,
///
    pub mint: Pubkey,
///
///
    pub token_program: Pubkey,
///
    pub nonce: u64,
///
///
///
///
///
    pub total: u64,
///
    pub claimed: u64,
///
    pub start: i64,
///
    pub cliff: i64,
///
    pub end: i64,
///
    pub bump: u8,
///
    pub escrow_bump: u8,
///
///
///
///
    pub linear_after_cliff: u8,
///
    pub reserved: [u8; 61],
}

impl VestingSchedule {
///
///
///
///
///
///
///
    pub const LEN: usize = 8 + 32 * 4 + 8 * 3 + 8 * 3 + 1 + 1 + 62;

///
///
///
    pub fn vested_at(&self, now: i64) -> Result<u64> {
        if self.linear_after_cliff != 0 {
            vested_after_cliff(self.total, self.cliff, self.end, now)
        } else {
            vested(self.total, self.start, self.cliff, self.end, now)
        }
    }

///
    pub fn claimable_at(&self, now: i64) -> Result<u64> {
        Ok(self.vested_at(now)?.saturating_sub(self.claimed))
    }
}

///
///
///
///
///
///
///
///
///
pub fn vested(total: u64, start: i64, cliff: i64, end: i64, now: i64) -> Result<u64> {
    if now < cliff {
        return Ok(0);
    }
    if now >= end {
        return Ok(total);
    }

    let elapsed = now.checked_sub(start).ok_or(crate::errors::VestingError::MathOverflow)? as u128;
    let span = end.checked_sub(start).ok_or(crate::errors::VestingError::MathOverflow)? as u128;
    let v = (total as u128)
        .checked_mul(elapsed)
        .ok_or(crate::errors::VestingError::MathOverflow)?
        / span;
    u64::try_from(v).map_err(|_| crate::errors::VestingError::MathOverflow.into())
}

///
///
///
///
///
pub fn vested_after_cliff(total: u64, cliff: i64, end: i64, now: i64) -> Result<u64> {
    if now < cliff {
        return Ok(0);
    }
    if now >= end {
        return Ok(total);
    }
    let elapsed = now.checked_sub(cliff).ok_or(crate::errors::VestingError::MathOverflow)? as u128;
    let span = end.checked_sub(cliff).ok_or(crate::errors::VestingError::MathOverflow)? as u128;
    let v = (total as u128)
        .checked_mul(elapsed)
        .ok_or(crate::errors::VestingError::MathOverflow)?
        / span;
    u64::try_from(v).map_err(|_| crate::errors::VestingError::MathOverflow.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    const START: i64 = 1_000_000;

    #[test]
    fn nothing_vests_before_the_cliff_and_the_cliff_instant_pays_its_share() {

        let cliff = START + 30 * 86_400;
        let end = START + 180 * 86_400;
        assert_eq!(vested(1_800_000, START, cliff, end, START).unwrap(), 0);
        assert_eq!(vested(1_800_000, START, cliff, end, cliff - 1).unwrap(), 0);


        assert_eq!(vested(1_800_000, START, cliff, end, cliff).unwrap(), 300_000);
    }

    #[test]
    fn accrual_is_linear_between_the_cliff_and_the_end() {
        let end = START + 100;
        for t in 0..=100i64 {
            let want = 1000u64 * (t as u64) / 100;
            assert_eq!(vested(1000, START, START, end, START + t).unwrap(), want, "t={t}");
        }
    }

    #[test]
    fn everything_is_vested_at_the_end_and_after_it() {
        let end = START + 100;
        assert_eq!(vested(7, START, START, end, end).unwrap(), 7);
        assert_eq!(vested(7, START, START, end, end + 10_000_000).unwrap(), 7);
        assert_eq!(vested(7, START, START, end, i64::MAX).unwrap(), 7);
    }

    #[test]
    fn a_cliff_equal_to_the_vest_pays_everything_at_the_end_and_nothing_before() {
        let end = START + 100;
        assert_eq!(vested(1000, START, end, end, end - 1).unwrap(), 0);
        assert_eq!(vested(1000, START, end, end, end).unwrap(), 1000);
    }

    #[test]
    fn a_u64_max_total_does_not_overflow_the_product() {

        let end = START + 1_000_000;
        let half = vested(u64::MAX, START, START, end, START + 500_000).unwrap();
        assert_eq!(half, u64::MAX / 2);
    }

    #[test]
    fn vesting_never_runs_backwards() {
        let cliff = START + 40;
        let end = START + 100;
        let mut last = 0;
        for t in 0..200i64 {
            let v = vested(123_456_789, START, cliff, end, START + t).unwrap();
            assert!(v >= last, "t={t}: {v} < {last}");
            last = v;
        }
        assert_eq!(last, 123_456_789);
    }

///
///
///
///
///
    #[test]
    fn sgr_v1_linear_after_cliff_is_the_evm_lock_at() {
        const D: i64 = 86_400;
        assert_eq!(vested_after_cliff(50, START, START + 30 * D, START + 30 * D).unwrap(), 50);
        let (cliff, end) = (START + 180 * D, START + 545 * D);
        let total = 700_000_000_000_000u64;
        assert_eq!(vested_after_cliff(total, cliff, end, START + 200 * D).unwrap(), (total as u128 * 20 / 365) as u64);
        assert_eq!(vested_after_cliff(total, cliff, end, cliff - 1).unwrap(), 0);


        assert_eq!(vested_after_cliff(total, cliff, end, cliff).unwrap(), 0);
        assert!(vested(total, START, cliff, end, cliff).unwrap() > 0);
        assert_eq!(vested_after_cliff(total, cliff, end, end).unwrap(), total);

        let lock = 10_000_000_000_000u64;
        for d in 3..=6i64 {
            let (cliff, end) = (START + d * D - 1, START + d * D);
            assert_eq!(vested_after_cliff(lock, cliff, end, cliff - 1).unwrap(), 0, "day {d}: nothing before the cliff");
            assert_eq!(vested_after_cliff(lock, cliff, end, cliff).unwrap(), 0, "day {d}: nothing at the cliff instant");
            assert_eq!(vested_after_cliff(lock, cliff, end, end).unwrap(), lock, "day {d}: the whole lock");
            assert_eq!(vested_after_cliff(lock, cliff, end, end + 30 * D).unwrap(), lock, "day {d}: never more");
        }
    }

    #[test]
    fn the_account_len_matches_what_borsh_writes() {
        let s = VestingSchedule {
            funder: Pubkey::new_unique(),
            beneficiary: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
            token_program: Pubkey::new_unique(),
            nonce: 1,
            total: 2,
            claimed: 3,
            start: 4,
            cliff: 5,
            end: 6,
            bump: 7,
            escrow_bump: 8,
            linear_after_cliff: 1,
            reserved: [0u8; 61],
        };
        let mut buf = Vec::new();
        s.serialize(&mut buf).unwrap();
        assert_eq!(buf.len() + 8, VestingSchedule::LEN);
    }
}
