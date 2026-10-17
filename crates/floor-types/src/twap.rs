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

use bytemuck::{Pod, Zeroable};

use crate::layout::{FloorPool, Layout};

///
pub const TWAP_CHECKPOINTS: usize = 8;
///
///
pub const TWAP_CHECKPOINT_SPACING_SECS: u32 = 300;
///
///
///
///
///
///
///
///
pub const CUSHION_TWAP_WINDOW_SECS: u32 = 900;

///
pub const TWAP_VERSION: u8 = 1;
///
pub const TWAP_ORACLE_OFFSET: usize = FloorPool::ACCOUNT_LEN;
///
pub const TWAP_ORACLE_SIZE: usize = 216;
///
pub const FLOOR_POOL_ACCOUNT_LEN_WITH_TWAP: usize = TWAP_ORACLE_OFFSET + TWAP_ORACLE_SIZE;

///
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Pod, Zeroable)]
pub struct TwapObservation {
///
    pub tick_cumulative: i64,
///
    pub timestamp: u32,
///
///
    pub last_tick: i32,
}

///
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Pod, Zeroable)]
pub struct TwapCheckpoint {
    pub tick_cumulative: i64,
    pub timestamp: u32,
    pub pad: u32,
}

///
///
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct TwapOracle {
///
    pub version: u8,
///
    pub newest: u8,
///
    pub count: u8,
    pub pad0: [u8; 5],
    pub current: TwapObservation,
    pub checkpoints: [TwapCheckpoint; TWAP_CHECKPOINTS],
///
///
///
///
    pub older_surplus_held: u8,
///
///
///
///
///
    pub cushion_top_set: u8,
    pub pad1: [u8; 2],
///
///
///
    pub cushion_top_tick: i32,
///
///
///
///
///
    pub flow_a: [u8; 16],
///
    pub flow_b: u64,
    pub reserved: [u8; 32],
}

const _: () = assert!(core::mem::size_of::<TwapObservation>() == 16);
const _: () = assert!(core::mem::size_of::<TwapCheckpoint>() == 16);
const _: () = assert!(core::mem::size_of::<TwapOracle>() == TWAP_ORACLE_SIZE);
const _: () = assert!(TWAP_ORACLE_OFFSET == 792 && TWAP_ORACLE_OFFSET % 8 == 0);
const _: () = assert!(FLOOR_POOL_ACCOUNT_LEN_WITH_TWAP == 1_008);

///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Observed {
///
    Initialized,
///
    Accrued { checkpointed: bool },
///
    SameSecond,
}

///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TwapError {
///
    Uninitialized,
///
    WindowTooLong,
}

///
pub fn floor_div(a: i64, b: i64) -> i64 {
    let q = a / b;
    if (a % b != 0) && ((a < 0) != (b < 0)) {
        q - 1
    } else {
        q
    }
}

impl TwapOracle {
///
    pub fn empty() -> Self {
        Zeroable::zeroed()
    }

    pub fn is_initialized(&self) -> bool {
        self.version != 0 && self.count != 0
    }

///
    pub fn newest_checkpoint(&self) -> TwapCheckpoint {
        self.checkpoints[self.newest as usize % TWAP_CHECKPOINTS]
    }

///
    pub fn observe(&mut self, now: u32, tick: i32) -> Observed {
        if !self.is_initialized() {

            let (held, set, top) = (self.older_surplus_held, self.cushion_top_set, self.cushion_top_tick);
            let (fa, fb) = (self.flow_a, self.flow_b);
            *self = Self::empty();
            (self.older_surplus_held, self.cushion_top_set, self.cushion_top_tick) = (held, set, top);
            (self.flow_a, self.flow_b) = (fa, fb);
            self.version = TWAP_VERSION;
            self.current = TwapObservation { tick_cumulative: 0, timestamp: now, last_tick: tick };
            self.checkpoints[0] = TwapCheckpoint { tick_cumulative: 0, timestamp: now, pad: 0 };
            self.newest = 0;
            self.count = 1;
            return Observed::Initialized;
        }
        let elapsed = now.wrapping_sub(self.current.timestamp);

        if elapsed == 0 || (elapsed as i32) < 0 {
            return Observed::SameSecond;
        }
        self.current.tick_cumulative = self.current.tick_cumulative.wrapping_add((tick as i64).wrapping_mul(elapsed as i64));
        self.current.timestamp = now;
        self.current.last_tick = tick;
        let checkpointed = now.wrapping_sub(self.newest_checkpoint().timestamp) >= TWAP_CHECKPOINT_SPACING_SECS;
        if checkpointed {
            let next = (self.newest as usize + 1) % TWAP_CHECKPOINTS;
            self.checkpoints[next] =
                TwapCheckpoint { tick_cumulative: self.current.tick_cumulative, timestamp: now, pad: 0 };
            self.newest = next as u8;
            self.count = (self.count as usize + 1).min(TWAP_CHECKPOINTS) as u8;
        }
        Observed::Accrued { checkpointed }
    }

///
    pub fn cumulative_at(&self, now: u32, live_tick: i32) -> i64 {
        let c = self.current;
        let since = now.wrapping_sub(c.timestamp);
        c.tick_cumulative.wrapping_add((live_tick as i64).wrapping_mul(since as i64))
    }

///
    pub fn twap_tick(&self, now: u32, live_tick: i32, window: u32) -> Result<i32, TwapError> {
        if window == 0 {
            return Ok(live_tick);
        }
        if !self.is_initialized() {
            return Err(TwapError::Uninitialized);
        }

        if now.wrapping_sub(self.current.timestamp) >= window {
            return Ok(live_tick);
        }
        let cum_now = self.cumulative_at(now, live_tick);
        let count = (self.count as usize).min(TWAP_CHECKPOINTS);
        for back in 0..count {
            let i = (self.newest as usize + TWAP_CHECKPOINTS - back) % TWAP_CHECKPOINTS;
            let cp = self.checkpoints[i];
            let age = now.wrapping_sub(cp.timestamp);
            if age >= window {
                let avg = floor_div(cum_now.wrapping_sub(cp.tick_cumulative), age as i64);

                return Ok(avg.clamp(i32::MIN as i64, i32::MAX as i64) as i32);
            }
        }
        Err(TwapError::WindowTooLong)
    }

///
///
///
///
///
///
    pub fn settled_tick(&self, now: u32, live_tick: i32, window: u32) -> Result<i32, TwapError> {
        match self.twap_tick(now, live_tick, window) {
            Err(TwapError::WindowTooLong) => {
                let count = (self.count as usize).clamp(1, TWAP_CHECKPOINTS);
                let oldest = self.checkpoints[(self.newest as usize + TWAP_CHECKPOINTS + 1 - count) % TWAP_CHECKPOINTS];
                let age = now.wrapping_sub(oldest.timestamp);
                if age > 0 && (age as i32) > 0 {
                    self.twap_tick(now, live_tick, age)
                } else {
                    Ok(self.current.last_tick)
                }
            }
            r => r,
        }
    }

///
///
    pub fn from_account(data: &[u8]) -> Option<Self> {
        data.get(TWAP_ORACLE_OFFSET..FLOOR_POOL_ACCOUNT_LEN_WITH_TWAP).map(bytemuck::pod_read_unaligned)
    }

///
///
    pub fn cushion_top_of_account(data: &[u8]) -> Option<i32> {
        let o = Self::from_account(data)?;
        (o.cushion_top_set != 0).then_some(o.cushion_top_tick)
    }

///
///
    pub fn net_flow(&self) -> (i128, u64) {
        (i128::from_le_bytes(self.flow_a), self.flow_b)
    }

///
    pub fn set_net_flow(&mut self, a: i128, b: u64) {
        self.flow_a = a.to_le_bytes();
        self.flow_b = b;
    }

///
    pub fn store(&self, data: &mut [u8]) -> bool {
        match data.get_mut(TWAP_ORACLE_OFFSET..FLOOR_POOL_ACCOUNT_LEN_WITH_TWAP) {
            Some(tail) => {
                tail.copy_from_slice(bytemuck::bytes_of(self));
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh(now: u32, tick: i32) -> TwapOracle {
        let mut o = TwapOracle::empty();
        assert_eq!(o.observe(now, tick), Observed::Initialized);
        o
    }

    #[test]
    fn layout_is_pinned() {
        assert_eq!(core::mem::offset_of!(TwapOracle, current), 8);
        assert_eq!(core::mem::offset_of!(TwapOracle, checkpoints), 24);
        assert_eq!(core::mem::offset_of!(TwapOracle, older_surplus_held), 152);

        assert_eq!(core::mem::offset_of!(TwapOracle, cushion_top_set), 153);
        assert_eq!(core::mem::offset_of!(TwapOracle, cushion_top_tick), 156);

        assert_eq!(core::mem::offset_of!(TwapOracle, flow_a), 160);
        assert_eq!(core::mem::offset_of!(TwapOracle, flow_b), 176);
        assert_eq!(core::mem::offset_of!(TwapOracle, reserved), 184);
        assert_eq!(core::mem::offset_of!(TwapObservation, timestamp), 8);
        assert_eq!(core::mem::offset_of!(TwapObservation, last_tick), 12);
        assert_eq!((TWAP_ORACLE_OFFSET, FLOOR_POOL_ACCOUNT_LEN_WITH_TWAP), (792, 1_008));
        assert_eq!((TWAP_CHECKPOINTS, TWAP_CHECKPOINT_SPACING_SECS), (8, 300));
    }

    #[test]
    fn floor_div_rounds_toward_negative_infinity() {
        assert_eq!(floor_div(7, 2), 3);
        assert_eq!(floor_div(-7, 2), -4);
        assert_eq!(floor_div(-8, 2), -4);
        assert_eq!(floor_div(0, 5), 0);
        assert_eq!(floor_div(-1, 300), -1);
    }

    #[test]
    fn the_first_write_initialises() {
        let o = fresh(1_000, -42);
        assert!(o.is_initialized());
        assert_eq!(o.current, TwapObservation { tick_cumulative: 0, timestamp: 1_000, last_tick: -42 });
        assert_eq!((o.count, o.newest), (1, 0));
        assert_eq!(o.checkpoints[0], TwapCheckpoint { tick_cumulative: 0, timestamp: 1_000, pad: 0 });
    }

///
    #[test]
    fn a_same_second_round_trip_does_not_move_it() {
        let mut o = fresh(1_000, 100);

        assert_eq!(o.observe(1_600, 100), Observed::Accrued { checkpointed: true });
        let before = o;
        assert_eq!(o.observe(1_600, 50_000), Observed::SameSecond);
        assert_eq!(o, before);

        assert_eq!(o.twap_tick(2_000, 100, 600), Ok(100));
        assert_eq!(o.twap_tick(1_600, 100, 600), Ok(100));
    }

///
///
    #[test]
    fn a_one_second_excursion_weighs_one_second() {
        let mut o = fresh(0, 0);
        o.observe(1_000, 0);
        o.observe(1_001, 3_000);
        o.observe(2_000, 0);

        assert_eq!(o.current.tick_cumulative, 3_000);
        assert_eq!(o.cumulative_at(2_000, 0), 3_000);
    }

///
    #[test]
    fn a_held_price_moves_it_by_the_time_weighted_amount() {
        let mut o = fresh(10_000, 1_000);

        assert_eq!(o.observe(10_300, 1_000), Observed::Accrued { checkpointed: true });
        assert_eq!(o.observe(11_200, 4_000), Observed::Accrued { checkpointed: true });
        assert_eq!(o.current.tick_cumulative, 1_000 * 300 + 4_000 * 900);

        assert_eq!(o.twap_tick(11_200, 4_000, 1_200), Ok(3_250));

        assert_eq!(o.twap_tick(11_200, 4_000, 900), Ok(4_000));

        assert_eq!(o.twap_tick(11_300, 4_000, 1_000), Ok(4_000));

        assert_eq!(o.twap_tick(11_300, 4_000, 1_300), Ok(3_307));
    }

    #[test]
    fn negative_ticks_round_down() {
        let mut o = fresh(0, -1);
        o.observe(300, -1);
        o.observe(301, -2);

        assert_eq!(o.twap_tick(301, -2, 301), Ok(-2));

        assert_eq!(o.twap_tick(301, -2, 1), Ok(-2));

        assert_eq!(o.twap_tick(400, -2, 400), Ok(-2));
    }

    #[test]
    fn window_zero_is_the_current_tick_and_a_long_window_is_refused() {
        let o = fresh(5_000, 7);
        assert_eq!(o.twap_tick(5_000, 123, 0), Ok(123));
        assert_eq!(o.twap_tick(5_100, 7, 200), Err(TwapError::WindowTooLong));
        assert_eq!(o.twap_tick(5_200, 7, 200), Ok(7));
        assert_eq!(TwapOracle::empty().twap_tick(1, 1, 1), Err(TwapError::Uninitialized));
        assert_eq!(TwapOracle::empty().twap_tick(1, 9, 0), Ok(9));
    }

    #[test]
    fn checkpoints_are_spaced_and_the_ring_keeps_the_newest_eight() {
        let mut o = fresh(0, 10);

        let mut t = 0u32;
        for _ in 0..60 {
            t += 100;
            let r = o.observe(t, 10);
            assert_eq!(r, Observed::Accrued { checkpointed: t % 300 == 0 }, "t = {t}");
        }
        assert_eq!(o.count as usize, TWAP_CHECKPOINTS);
        let mut ts: Vec<u32> = o.checkpoints.iter().map(|c| c.timestamp).collect();
        ts.sort_unstable();
        assert_eq!(ts, vec![3_900, 4_200, 4_500, 4_800, 5_100, 5_400, 5_700, 6_000]);
        assert_eq!(o.newest_checkpoint().timestamp, 6_000);

        assert_eq!(o.twap_tick(6_000, 10, 2_100), Ok(10));
        assert_eq!(o.twap_tick(6_000, 10, 2_101), Err(TwapError::WindowTooLong));
        for c in o.checkpoints {
            assert_eq!(c.tick_cumulative, 10 * c.timestamp as i64);
        }
    }

    #[test]
    fn a_clock_behind_the_last_write_is_a_no_op() {
        let mut o = fresh(1_000, 5);
        let before = o;
        assert_eq!(o.observe(999, 6), Observed::SameSecond);
        assert_eq!(o, before);

        assert_eq!(o.twap_tick(999, 6, 60), Ok(6));

        assert_eq!(o.observe(1_010, 5), Observed::Accrued { checkpointed: false });
        assert_eq!(o.current.tick_cumulative, 5 * 10);
    }

///
///
    #[test]
    fn a_window_with_no_write_in_it_is_the_live_tick_exactly() {
        let mut o = fresh(0, -500);
        o.observe(1_000, -500);
        assert_eq!(o.twap_tick(1_600, 42, 600), Ok(42), "600 s since the last write, window 600");
        assert_eq!(o.twap_tick(1_600, 42, 601), Ok(floor_div(-500 * 1_000 + 42 * 600, 1_600) as i32));
        assert_eq!(o.twap_tick(1_599, 42, 600), Ok(floor_div(-500 * 1_000 + 42 * 599, 1_599) as i32));
    }

///
    #[test]
    fn the_cumulative_wraps_and_still_reads() {
        let mut o = fresh(0, 0);
        let near = i64::MAX - 1_000;
        o.current.tick_cumulative = near;
        o.checkpoints[0].tick_cumulative = near;
        let t = 887_272;
        assert_eq!(o.observe(600, t), Observed::Accrued { checkpointed: true });
        assert!(o.current.tick_cumulative < 0, "it wrapped");
        assert_eq!(o.current.tick_cumulative, near.wrapping_add(t as i64 * 600));
        assert_eq!(o.twap_tick(600, t, 600), Ok(t));
        assert_eq!(o.twap_tick(900, -t, 900), Ok(floor_div(t as i64 * 600 - t as i64 * 300, 900) as i32));
    }

///
    #[test]
    fn the_timestamp_wraps() {
        let mut o = fresh(u32::MAX - 99, 3);
        assert_eq!(o.observe(200, 3), Observed::Accrued { checkpointed: true });
        assert_eq!(o.current.tick_cumulative, 3 * 300);
        assert_eq!(o.twap_tick(500, 3, 600), Ok(3));
        assert_eq!(o.twap_tick(250, 7, 350), Ok(floor_div(3 * 300 + 7 * 50, 350) as i32));
    }

    #[test]
    fn the_account_tail_round_trips_and_a_short_account_has_none() {
        let mut data = vec![0u8; FLOOR_POOL_ACCOUNT_LEN_WITH_TWAP];
        assert_eq!(TwapOracle::from_account(&data), Some(TwapOracle::empty()));
        let o = fresh(77, -3);
        assert!(o.store(&mut data));
        assert_eq!(TwapOracle::from_account(&data), Some(o));
        assert!(data[..TWAP_ORACLE_OFFSET].iter().all(|b| *b == 0), "the struct's bytes are untouched");
        let mut short = vec![0u8; FloorPool::ACCOUNT_LEN];
        assert_eq!(TwapOracle::from_account(&short), None);
        assert!(!o.store(&mut short));
    }

///
///
///
    #[test]
    fn the_settled_tick_ignores_a_same_second_pump() {
        let mut o = TwapOracle::empty();
        assert_eq!(o.observe(1_000, 80_000), Observed::Initialized);

        assert_eq!(o.observe(1_000, 60_000), Observed::SameSecond);
        assert_eq!(o.settled_tick(1_000, 60_000, CUSHION_TWAP_WINDOW_SECS), Ok(80_000), "the tick the second started at");

        assert_eq!(o.observe(1_100, 80_000), Observed::Accrued { checkpointed: false });
        assert_eq!(o.settled_tick(1_100, 60_000, CUSHION_TWAP_WINDOW_SECS), Ok(80_000), "young ring: over its oldest");

        let mut t = 1_100;
        while t < 3_500 {
            t += 300;
            o.observe(t, 80_000);
        }
        assert_eq!(o.observe(t, 50_000), Observed::SameSecond, "the pump's own swap, same second as the last write");
        assert_eq!(o.settled_tick(t, 50_000, CUSHION_TWAP_WINDOW_SECS), Ok(80_000));


        let later = o.settled_tick(t + 1, 50_000, CUSHION_TWAP_WINDOW_SECS).unwrap();
        let one_second = 30_000 / CUSHION_TWAP_WINDOW_SECS as i32;
        assert!(later < 80_000 && later >= 80_000 - one_second - 1, "{later}");
    }
}
