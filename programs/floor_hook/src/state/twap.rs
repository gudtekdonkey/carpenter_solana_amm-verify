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

pub use floor_types::twap::*;

///
///
pub fn now_secs() -> Result<u32> {
    Ok(Clock::get()?.unix_timestamp as u32)
}

///
///
///
///
pub fn observe_account(floor_pool: &AccountInfo, now: u32, tick: i32) -> Result<Observed> {
    let mut data = floor_pool.try_borrow_mut_data()?;
    let mut o = TwapOracle::from_account(&data).ok_or(error!(crate::errors::HookError::InvariantViolated))?;
    let r = o.observe(now, tick);
    if r != Observed::SameSecond {
        o.store(&mut data);
    }
    Ok(r)
}

///
pub fn twap_tick_of_account(floor_pool: &AccountInfo, now: u32, current_tick: i32, window: u32) -> Result<i32> {
    if window == 0 {
        return Ok(current_tick);
    }
    let data = floor_pool.try_borrow_data()?;
    let o = TwapOracle::from_account(&data).ok_or(error!(crate::errors::HookError::TwapUnavailable))?;
    o.twap_tick(now, current_tick, window).map_err(|e| match e {
        TwapError::Uninitialized => error!(crate::errors::HookError::TwapUnavailable),
        TwapError::WindowTooLong => error!(crate::errors::HookError::TwapWindowTooLong),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_account<R>(len: usize, f: impl FnOnce(&AccountInfo) -> R) -> (R, Vec<u8>) {
        let key = Pubkey::new_unique();
        let owner = crate::ID;
        let mut lamports = 0u64;
        let mut data = vec![0u8; len];
        let r = {
            let ai = AccountInfo::new(&key, false, true, &mut lamports, &mut data, &owner, false);
            f(&ai)
        };
        (r, data)
    }

///
    #[test]
    fn the_account_write_touches_the_tail_only_and_refuses_a_short_account() {
        let ((), data) = with_account(crate::state::FloorPool::LEN_WITH_TWAP, |ai| {
            assert_eq!(observe_account(ai, 1_000, 50).unwrap(), Observed::Initialized);
            assert_eq!(observe_account(ai, 1_000, 9_999).unwrap(), Observed::SameSecond);
            assert_eq!(observe_account(ai, 1_300, 60).unwrap(), Observed::Accrued { checkpointed: true });

            assert_eq!(twap_tick_of_account(ai, 1_600, 60, 600).unwrap(), (60 * 300 + 60 * 300) / 600);
            assert_eq!(twap_tick_of_account(ai, 1_600, 70, 0).unwrap(), 70);
            assert!(twap_tick_of_account(ai, 1_600, 60, 601).is_err());
        });
        assert!(data[..crate::state::FloorPool::LEN].iter().all(|b| *b == 0), "the struct's bytes untouched");
        let o = TwapOracle::from_account(&data).unwrap();
        assert_eq!(o.current, TwapObservation { tick_cumulative: 60 * 300, timestamp: 1_300, last_tick: 60 });

        let ((), old) = with_account(crate::state::FloorPool::LEN, |ai| {
            assert!(observe_account(ai, 1_000, 50).is_err(), "a short account is refused, never skipped (2026-10-04)");
            assert!(twap_tick_of_account(ai, 1_000, 50, 1).is_err());
            assert_eq!(twap_tick_of_account(ai, 1_000, 50, 0).unwrap(), 50);
        });
        assert!(old.iter().all(|b| *b == 0));
    }
}
