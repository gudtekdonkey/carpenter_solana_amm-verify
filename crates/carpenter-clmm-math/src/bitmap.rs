// Derived from raydium-io/raydium-clmm @ ed7c84a5, programs/amm/src/libraries/tick_array_bit_map.rs and the
// start-index helpers of programs/amm/src/states/tick_array.rs (`get_array_start_index`,
// `check_is_valid_start_index`, `tick_count`), Apache-2.0.
// Modified by FLOOR, 2026: re-shaped for spec D-07 / §4.1 `TickArrayBitmap`. Upstream searches a fixed
// `U1024` (512 arrays either side of 0, plus an extension account beyond that); here the bitmap is a
// spacing-sized `&[u64]` that covers the whole tick range, bit `k/(60·s) + arrays_per_side` for the array
// starting at `k`, so there is no extension and no extension-binding bug class. The most/least-significant
// set-bit search keeps upstream's direction semantics (zero_for_one searches toward lower start indices),
// done word by word instead of by shifting a 1024-bit integer. A miss returns `None`: no extension exists
// to ask, which is the case Raydium fcad161 fixed ("default-bitmap-boundary misclassification").
// `array_start_index` refuses a tick outside [MIN_TICK, MAX_TICK] (upstream's product overflowed i32).
// Anchor / `ErrorCode` replaced by `MathResult` / `MathError`. Upstream's bitmap tests are ported
// against the new layout (same start indices, same expected sequences) where their assertion survives it.

use crate::error::{MathError, MathResult};
use crate::tick_math::{MAX_TICK, MIN_TICK};

///
pub const TICK_ARRAY_SIZE: i32 = 60;

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
///
    Down,
///
    Up,
}

impl Direction {
    #[inline]
    pub const fn from_zero_for_one(zero_for_one: bool) -> Self {
        if zero_for_one {
            Direction::Down
        } else {
            Direction::Up
        }
    }
}

///
#[inline]
pub fn ticks_in_array(tick_spacing: u16) -> MathResult<i32> {
    if tick_spacing == 0 {
        return Err(MathError::InvalidTickSpacing);
    }
    Ok(TICK_ARRAY_SIZE * i32::from(tick_spacing))
}

///
pub fn arrays_per_side(tick_spacing: u16) -> MathResult<u32> {
    let per = ticks_in_array(tick_spacing)?;
    Ok(((MAX_TICK + per - 1) / per) as u32)
}

///
pub fn bit_count(tick_spacing: u16) -> MathResult<u32> {
    Ok(2 * arrays_per_side(tick_spacing)? + 1)
}

///
pub fn word_count(tick_spacing: u16) -> MathResult<usize> {
    Ok(((bit_count(tick_spacing)? + 63) / 64) as usize)
}

///
///
///
///
///
///
///
pub fn array_start_index(tick: i32, tick_spacing: u16) -> MathResult<i32> {
    let per = ticks_in_array(tick_spacing)?;
    if !(MIN_TICK..=MAX_TICK).contains(&tick) {
        return Err(MathError::TickOutOfRange);
    }
    let mut start = tick / per;
    if tick < 0 && tick % per != 0 {
        start -= 1;
    }
    Ok(start * per)
}

///
///
pub fn is_valid_start_index(start_index: i32, tick_spacing: u16) -> MathResult<bool> {
    let per = ticks_in_array(tick_spacing)?;
    if start_index > MAX_TICK {
        return Ok(false);
    }
    if start_index < MIN_TICK {
        return Ok(start_index == array_start_index(MIN_TICK, tick_spacing)?);
    }
    Ok(start_index % per == 0)
}

///
pub fn array_bit(start_index: i32, tick_spacing: u16) -> MathResult<u32> {
    if !is_valid_start_index(start_index, tick_spacing)? {
        return Err(MathError::InvalidTickArrayBoundary);
    }
    let per = ticks_in_array(tick_spacing)?;
    let aps = arrays_per_side(tick_spacing)? as i32;
    let bit = start_index / per + aps;
    debug_assert!(bit >= 0 && (bit as u32) < bit_count(tick_spacing)?);
    Ok(bit as u32)
}

///
pub fn start_index_of_bit(bit: u32, tick_spacing: u16) -> MathResult<i32> {
    if bit >= bit_count(tick_spacing)? {
        return Err(MathError::InvalidTickArrayBoundary);
    }
    let per = ticks_in_array(tick_spacing)?;
    let aps = arrays_per_side(tick_spacing)? as i32;
    let start = (bit as i32 - aps) * per;
    if !is_valid_start_index(start, tick_spacing)? {

        return Err(MathError::InvalidTickArrayBoundary);
    }
    Ok(start)
}

///
pub fn check_words(words: &[u64], tick_spacing: u16) -> MathResult<()> {
    if words.len() != word_count(tick_spacing)? {
        return Err(MathError::BitmapLengthMismatch);
    }
    Ok(())
}

///
pub fn is_initialized(words: &[u64], bit: u32) -> MathResult<bool> {
    let w = words
        .get((bit / 64) as usize)
        .ok_or(MathError::InvalidTickArrayBoundary)?;
    Ok(w & (1u64 << (bit % 64)) != 0)
}

///
pub fn set_initialized(words: &mut [u64], bit: u32, initialized: bool) -> MathResult<()> {
    let w = words
        .get_mut((bit / 64) as usize)
        .ok_or(MathError::InvalidTickArrayBoundary)?;
    let mask = 1u64 << (bit % 64);
    if initialized {
        *w |= mask;
    } else {
        *w &= !mask;
    }
    Ok(())
}

///
///
///
///
///
///
pub fn next_initialized_array(words: &[u64], start: u32, dir: Direction) -> Option<u32> {
    let capacity = words.len() as u64 * 64;
    match dir {
        Direction::Down => {
            if start == 0 || words.is_empty() {
                return None;
            }

            let top = core::cmp::min(start as u64 - 1, capacity - 1);
            let mut wi = (top / 64) as usize;
            let b = (top % 64) as u32;

            let mut word = if b == 63 {
                words[wi]
            } else {
                words[wi] & ((1u64 << (b + 1)) - 1)
            };
            loop {
                if word != 0 {
                    return Some(wi as u32 * 64 + (63 - word.leading_zeros()));
                }
                if wi == 0 {
                    return None;
                }
                wi -= 1;
                word = words[wi];
            }
        }
        Direction::Up => {
            let first = start as u64 + 1;
            if first >= capacity {
                return None;
            }
            let mut wi = (first / 64) as usize;
            let b = (first % 64) as u32;

            let mut word = words[wi] & (u64::MAX << b);
            loop {
                if word != 0 {
                    return Some(wi as u32 * 64 + word.trailing_zeros());
                }
                wi += 1;
                if wi == words.len() {
                    return None;
                }
                word = words[wi];
            }
        }
    }
}

///
///
///
///
pub fn next_initialized_array_start_index(
    words: &[u64],
    tick_spacing: u16,
    last_start_index: i32,
    zero_for_one: bool,
) -> MathResult<Option<i32>> {
    check_words(words, tick_spacing)?;
    let bit = array_bit(last_start_index, tick_spacing)?;
    match next_initialized_array(words, bit, Direction::from_zero_for_one(zero_for_one)) {
        None => Ok(None),
        Some(found) => start_index_of_bit(found, tick_spacing).map(Some),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn bitmap_with(tick_spacing: u16, starts: &[i32]) -> Vec<u64> {
        let mut words = vec![0u64; word_count(tick_spacing).unwrap()];
        for &s in starts {
            set_initialized(&mut words, array_bit(s, tick_spacing).unwrap(), true).unwrap();
        }
        words
    }

///
    #[test]
    fn sizes_match_spec_4_1() {
        assert_eq!(word_count(60).unwrap(), 4);
        assert_eq!(8 + 40 + 8 * word_count(60).unwrap(), 80);
        assert_eq!(8 + 40 + 8 * word_count(10).unwrap(), 240);
        assert_eq!(8 + 40 + 8 * word_count(1).unwrap(), 1_904);
        assert_eq!(arrays_per_side(60).unwrap(), 124);
        assert_eq!(ticks_in_array(0), Err(MathError::InvalidTickSpacing));
    }

///
///
    #[test]
    fn bit_round_trip_covers_the_full_tick_range() {
        for spacing in [1u16, 2, 10, 15, 60, 100, 200, 1000, 32767] {
            let per = ticks_in_array(spacing).unwrap();
            let aps = arrays_per_side(spacing).unwrap();
            let lo = array_start_index(MIN_TICK, spacing).unwrap();
            let hi = array_start_index(MAX_TICK, spacing).unwrap();
            assert_eq!(array_bit(lo, spacing).unwrap(), 0, "spacing {spacing}");
            assert_eq!(
                array_bit(hi, spacing).unwrap(),
                2 * aps - 1,
                "spacing {spacing}"
            );
            let mut expected = 0;
            let mut s = lo;
            while s <= hi {
                let bit = array_bit(s, spacing).unwrap();
                assert_eq!(bit, expected);
                assert_eq!(start_index_of_bit(bit, spacing).unwrap(), s);
                expected += 1;
                s += per;
            }
            assert!(array_bit(lo - per, spacing).is_err());
            assert!(array_bit(hi + per, spacing).is_err());
            assert!(array_bit(per / 2, spacing).is_err() || per == 1);

            assert!(start_index_of_bit(2 * aps, spacing).is_err());
            assert!(start_index_of_bit(bit_count(spacing).unwrap(), spacing).is_err());
        }
    }

///
///
    #[test]
    fn get_array_start_index_test() {
        assert_eq!(array_start_index(120, 3).unwrap(), 0);
        assert_eq!(array_start_index(1002, 30).unwrap(), 0);
        assert_eq!(array_start_index(-120, 3).unwrap(), -180);
        assert_eq!(array_start_index(-1002, 30).unwrap(), -1800);
        assert_eq!(array_start_index(-20, 10).unwrap(), -600);
        assert_eq!(array_start_index(20, 10).unwrap(), 0);
        assert_eq!(array_start_index(-1002, 10).unwrap(), -1200);
        assert_eq!(array_start_index(-600, 10).unwrap(), -600);
        assert_eq!(array_start_index(-30720, 1).unwrap(), -30720);
        assert_eq!(array_start_index(30720, 1).unwrap(), 30720);
        assert_eq!(array_start_index(MIN_TICK, 1).unwrap(), -443640);
        assert_eq!(array_start_index(MAX_TICK, 1).unwrap(), 443580);
        assert_eq!(array_start_index(MAX_TICK, 60).unwrap(), 442800);
        assert_eq!(array_start_index(MIN_TICK, 60).unwrap(), -446400);
    }

///
///
    #[test]
    fn array_start_index_rounds_toward_negative_infinity() {
        assert_eq!(array_start_index(0, 10).unwrap(), 0);
        assert_eq!(array_start_index(599, 10).unwrap(), 0);
        assert_eq!(array_start_index(600, 10).unwrap(), 600);
        assert_eq!(array_start_index(-1, 10).unwrap(), -600);
        assert_eq!(array_start_index(-601, 10).unwrap(), -1200);
    }

///
///
///
    #[test]
    fn is_valid_start_index_covers_every_branch() {
        for spacing in [1u16, 10, 60, 32_767, u16::MAX] {
            let per = ticks_in_array(spacing).unwrap();
            let lo = array_start_index(MIN_TICK, spacing).unwrap();
            let hi = array_start_index(MAX_TICK, spacing).unwrap();
            assert!(is_valid_start_index(lo, spacing).unwrap(), "lo, spacing {spacing}");
            assert!(is_valid_start_index(hi, spacing).unwrap(), "hi, spacing {spacing}");
            assert!(is_valid_start_index(0, spacing).unwrap(), "0, spacing {spacing}");

            assert!(!is_valid_start_index(lo - per, spacing).unwrap(), "lo - per, spacing {spacing}");
            if lo < MIN_TICK {
                assert!(!is_valid_start_index(lo + 1, spacing).unwrap(), "lo + 1, spacing {spacing}");
            }

            assert!(!is_valid_start_index(MAX_TICK + 1, spacing).unwrap(), "MAX+1, spacing {spacing}");
            assert!(!is_valid_start_index(hi + per, spacing).unwrap(), "hi + per, spacing {spacing}");
            assert!(!is_valid_start_index(i32::MAX, spacing).unwrap(), "i32::MAX, spacing {spacing}");

            assert!(!is_valid_start_index(1, spacing).unwrap(), "1, spacing {spacing}");
            assert!(!is_valid_start_index(per - 1, spacing).unwrap(), "per - 1, spacing {spacing}");
            assert!(!is_valid_start_index(-1, spacing).unwrap(), "-1, spacing {spacing}");
        }

        assert!(is_valid_start_index(-446_400, 60).unwrap());
        assert!(!is_valid_start_index(-450_000, 60).unwrap());
        assert!(is_valid_start_index(442_800, 60).unwrap());
        assert!(!is_valid_start_index(446_400, 60).unwrap());
        assert!(!is_valid_start_index(442_801, 60).unwrap());
    }

///
///
    #[test]
    fn array_start_index_refuses_out_of_range_ticks() {
        for tick in [i32::MIN, i32::MAX, MIN_TICK - 1, MAX_TICK + 1] {
            for spacing in [1u16, 60, 32_767, u16::MAX] {
                assert_eq!(
                    array_start_index(tick, spacing),
                    Err(MathError::TickOutOfRange),
                    "tick {tick} spacing {spacing}"
                );
            }
        }
        for spacing in [32_767u16, u16::MAX] {
            assert!(array_start_index(MIN_TICK, spacing).unwrap() <= MIN_TICK);
            assert!(array_start_index(MAX_TICK, spacing).unwrap() <= MAX_TICK);
        }
    }

///
///
    #[test]
    fn find_next_init_pos_in_bit_map_with_eigenvalues() {
        let s = 10;
        let words = bitmap_with(
            s,
            &[
                -307200, -39000, -38400, -1800, -1200, -600, 0, 600, 1200, 38400, 306600,
            ],
        );
        let next = |from: i32, zfo: bool| {
            next_initialized_array_start_index(&words, s, from, zfo)
                .unwrap()
                .unwrap()
        };
        assert_eq!(next(0, true), -600);
        assert_eq!(next(-600, true), -1200);
        assert_eq!(next(-1200, true), -1800);
        assert_eq!(next(-1800, true), -38400);
        assert_eq!(next(-38400, true), -39000);
        assert_eq!(next(-39000, true), -307200);
        assert_eq!(
            next_initialized_array_start_index(&words, s, -307200, true).unwrap(),
            None
        );

        assert_eq!(next(0, false), 600);
        assert_eq!(next(600, false), 1200);
        assert_eq!(next(1200, false), 38400);
        assert_eq!(next(38400, false), 306600);
        assert_eq!(
            next_initialized_array_start_index(&words, s, 306600, false).unwrap(),
            None
        );
    }

///
///
///
    #[test]
    fn next_initialized_tick_array_with_all_initialized_bit_test() {
        for spacing in [1u16, 10, 60] {
            let per = ticks_in_array(spacing).unwrap();
            let mut words = vec![0u64; word_count(spacing).unwrap()];
            let lo = array_start_index(MIN_TICK, spacing).unwrap();
            let hi = array_start_index(MAX_TICK, spacing).unwrap();
            let mut s = lo;
            while s <= hi {
                set_initialized(&mut words, array_bit(s, spacing).unwrap(), true).unwrap();
                s += per;
            }
            let mut s = lo;
            while s < hi {
                let n = next_initialized_array_start_index(&words, spacing, s, false)
                    .unwrap()
                    .unwrap();
                assert_eq!(n, s + per);
                s = n;
            }
            assert_eq!(
                next_initialized_array_start_index(&words, spacing, hi, false).unwrap(),
                None
            );
            let mut s = hi;
            while s > lo {
                let n = next_initialized_array_start_index(&words, spacing, s, true)
                    .unwrap()
                    .unwrap();
                assert_eq!(n, s - per);
                s = n;
            }
            assert_eq!(
                next_initialized_array_start_index(&words, spacing, lo, true).unwrap(),
                None
            );
        }
    }

///
///
///
    #[test]
    fn exhausted_direction_is_none_not_an_error() {
        for spacing in [1u16, 10, 15, 60] {
            let words = vec![0u64; word_count(spacing).unwrap()];
            for zfo in [true, false] {
                assert_eq!(
                    next_initialized_array_start_index(&words, spacing, 0, zfo).unwrap(),
                    None
                );
            }
            let full = vec![u64::MAX; word_count(spacing).unwrap()];
            let lo = array_start_index(MIN_TICK, spacing).unwrap();
            let hi = array_start_index(MAX_TICK, spacing).unwrap();
            assert_eq!(
                next_initialized_array_start_index(&full, spacing, lo, true).unwrap(),
                None
            );


            assert_eq!(
                next_initialized_array_start_index(&full, spacing, hi, false),
                Err(MathError::InvalidTickArrayBoundary)
            );
        }
    }

    #[test]
    fn wrong_length_and_invalid_start_are_refused() {
        let words = vec![0u64; 3];
        assert_eq!(
            next_initialized_array_start_index(&words, 60, 0, true),
            Err(MathError::BitmapLengthMismatch)
        );
        let words = vec![0u64; 4];
        assert_eq!(
            next_initialized_array_start_index(&words, 60, 1, true),
            Err(MathError::InvalidTickArrayBoundary)
        );
        assert!(is_initialized(&words, 4 * 64).is_err());
    }

///
    #[test]
    fn raw_search_crosses_word_edges() {
        let mut w = vec![0u64; 3];
        for b in [0u32, 63, 64, 127, 191] {
            set_initialized(&mut w, b, true).unwrap();
        }
        use Direction::*;
        assert_eq!(next_initialized_array(&w, 191, Down), Some(127));
        assert_eq!(next_initialized_array(&w, 127, Down), Some(64));
        assert_eq!(next_initialized_array(&w, 64, Down), Some(63));
        assert_eq!(next_initialized_array(&w, 63, Down), Some(0));
        assert_eq!(next_initialized_array(&w, 0, Down), None);
        assert_eq!(next_initialized_array(&w, 0, Up), Some(63));
        assert_eq!(next_initialized_array(&w, 63, Up), Some(64));
        assert_eq!(next_initialized_array(&w, 64, Up), Some(127));
        assert_eq!(next_initialized_array(&w, 127, Up), Some(191));
        assert_eq!(next_initialized_array(&w, 191, Up), None);
        assert_eq!(next_initialized_array(&w, 10_000, Down), Some(191));
        assert_eq!(next_initialized_array(&w, 10_000, Up), None);
        assert_eq!(next_initialized_array(&[], 5, Down), None);
        assert!(is_initialized(&w, 127).unwrap());
        set_initialized(&mut w, 127, false).unwrap();
        assert!(!is_initialized(&w, 127).unwrap());
        assert_eq!(next_initialized_array(&w, 191, Down), Some(64));
    }

///
    #[test]
    fn raw_search_matches_a_linear_scan() {
        let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
        for _ in 0..200 {
            let mut w = vec![0u64; 5];
            for word in w.iter_mut() {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;

                *word = x & (x >> 3) & (x >> 5) & (x >> 11);
            }
            let set = |b: u32| is_initialized(&w, b).unwrap();
            for start in 0..(5 * 64) {
                let down = (0..start).rev().find(|&b| set(b));
                let up = (start + 1..5 * 64).find(|&b| set(b));
                assert_eq!(next_initialized_array(&w, start, Direction::Down), down);
                assert_eq!(next_initialized_array(&w, start, Direction::Up), up);
            }
        }
    }
}
