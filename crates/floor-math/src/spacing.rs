//!
//!
//!

use carpenter_clmm_math::tick_math::{MAX_TICK, MIN_TICK};

use crate::error::{FloorMathError, FloorMathResult};

///
///
pub fn ceil_to_spacing(tick: i32, spacing: i32) -> FloorMathResult<i32> {
    if spacing <= 0 {
        return Err(FloorMathError::InvalidTickSpacing);
    }
    let (t, s) = (tick as i64, spacing as i64);
    let mut compressed = t / s;
    if t > 0 && t % s != 0 {
        compressed += 1;
    }
    let mut result = compressed * s;
    if result > MAX_TICK as i64 - s {
        result = (MAX_TICK as i64 / s) * s - s;
    }
    Ok(result as i32)
}

///
///
pub fn floor_to_spacing(tick: i32, spacing: i32) -> FloorMathResult<i32> {
    if spacing <= 0 {
        return Err(FloorMathError::InvalidTickSpacing);
    }
    let (t, s) = (tick as i64, spacing as i64);
    let mut compressed = t / s;
    if t < 0 && t % s != 0 {
        compressed -= 1;
    }
    let mut result = compressed * s;
    if result < MIN_TICK as i64 + s {
        result = (MIN_TICK as i64 / s) * s + s;
    }
    Ok(result as i32)
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn ceil_rounds_toward_the_cheaper_token() {
        assert_eq!(ceil_to_spacing(0, 60).unwrap(), 0);
        assert_eq!(ceil_to_spacing(1, 60).unwrap(), 60);
        assert_eq!(ceil_to_spacing(60, 60).unwrap(), 60);
        assert_eq!(ceil_to_spacing(-1, 60).unwrap(), 0);
        assert_eq!(ceil_to_spacing(-61, 60).unwrap(), -60);
    }


    #[test]
    fn floor_rounds_toward_the_dearer_token() {
        assert_eq!(floor_to_spacing(0, 60).unwrap(), 0);
        assert_eq!(floor_to_spacing(1, 60).unwrap(), 0);
        assert_eq!(floor_to_spacing(-1, 60).unwrap(), -60);
        assert_eq!(floor_to_spacing(-60, 60).unwrap(), -60);
        assert_eq!(floor_to_spacing(119, 60).unwrap(), 60);
    }


    #[test]
    fn edge_clamps_match_the_evm_rule_at_solana_bounds() {

        assert_eq!(ceil_to_spacing(MAX_TICK, 60).unwrap(), 443_520);

        assert_eq!(ceil_to_spacing(MIN_TICK, 60).unwrap(), -443_580);

        assert_eq!(floor_to_spacing(MIN_TICK, 60).unwrap(), -443_520);
        assert_eq!(floor_to_spacing(MAX_TICK, 60).unwrap(), 443_580);

        assert_eq!(ceil_to_spacing(i32::MAX, 60).unwrap(), 443_520);
        assert_eq!(floor_to_spacing(i32::MIN, 60).unwrap(), -443_520);
        assert_eq!(ceil_to_spacing(i32::MIN, 1).unwrap(), i32::MIN);
        assert_eq!(floor_to_spacing(i32::MAX, 1).unwrap(), i32::MAX);
    }

    #[test]
    fn zero_or_negative_spacing_is_refused() {
        assert_eq!(ceil_to_spacing(5, 0), Err(FloorMathError::InvalidTickSpacing));
        assert_eq!(floor_to_spacing(5, -60), Err(FloorMathError::InvalidTickSpacing));
    }


    #[test]
    fn results_are_on_grid_and_adjacent() {
        for s in [1, 10, 60, 200] {
            for t in (-440_000..440_000).step_by(997) {
                let c = ceil_to_spacing(t, s).unwrap();
                let f = floor_to_spacing(t, s).unwrap();
                assert_eq!(c % s, 0);
                assert_eq!(f % s, 0);
                assert!(c >= t && c - t < s, "ceil {t} {s} -> {c}");
                assert!(f <= t && t - f < s, "floor {t} {s} -> {f}");
            }
        }
    }
}
