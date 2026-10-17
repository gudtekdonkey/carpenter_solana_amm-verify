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

use crate::constants::BID_BAND_TICKS;
use crate::error::{FloorMathError, FloorMathResult};
use crate::spacing::ceil_to_spacing;

///
///
pub fn band_ticks(spacing: i32) -> FloorMathResult<i32> {
    ceil_to_spacing(BID_BAND_TICKS, spacing)
}

///
///
///
///
pub fn dear_edge(floor_tick: i32, spacing: i32) -> FloorMathResult<i32> {
    floor_tick.checked_sub(band_ticks(spacing)?).ok_or(FloorMathError::Overflow)
}

///
///
///
pub fn publish(solved: i32, spacing: i32) -> FloorMathResult<i32> {
    let raised = solved.checked_add(band_ticks(spacing)?).ok_or(FloorMathError::Overflow)?;
    ceil_to_spacing(raised, spacing)
}

///
///
pub fn bid_width_or_spacing(bid_width: i32, spacing: i32) -> i32 {
    if bid_width == 0 {
        spacing
    } else {
        bid_width
    }
}

///
///
pub fn live_floor_tick(solved: i32, spacing: i32) -> FloorMathResult<i32> {
    publish(solved, spacing)
}







#[cfg(test)]
mod tests {

    use super::*;
    use carpenter_clmm_math::tick_math::{MAX_TICK, MIN_TICK};


    #[test]
    fn band_ticks_by_spacing() {
        assert_eq!(band_ticks(1).unwrap(), 120);
        assert_eq!(band_ticks(10).unwrap(), 120);
        assert_eq!(band_ticks(60).unwrap(), 120);
        assert_eq!(band_ticks(200).unwrap(), 200);
        assert_eq!(band_ticks(7).unwrap(), 126);
        assert!(band_ticks(0).is_err());
    }


    #[test]
    fn dear_edge_and_publish_round_trip_on_the_grid() {
        for s in [1, 10, 60, 200] {
            for f in (-400_000..400_000).step_by(997) {
                let f = ceil_to_spacing(f, s).unwrap();
                let d = dear_edge(f, s).unwrap();
                assert_eq!(d, f - band_ticks(s).unwrap());
                assert_eq!(publish(d, s).unwrap(), f, "spacing {s} floor {f}");
            }
        }
        assert_eq!(dear_edge(0, 60).unwrap(), -120);
        assert_eq!(publish(-120, 60).unwrap(), 0);

        assert_eq!(publish(-119, 60).unwrap(), 60);
    }


    #[test]
    fn publish_clamps_at_max() {
        let hi = ceil_to_spacing(MAX_TICK, 60).unwrap();
        assert_eq!(hi, 443_520);
        assert_eq!(publish(hi, 60).unwrap(), hi);
        assert_eq!(publish(hi - 60, 60).unwrap(), hi);
        assert_eq!(publish(hi - 120, 60).unwrap(), hi);
        assert_eq!(publish(hi - 180, 60).unwrap(), hi - 60);
        assert_eq!(publish(MAX_TICK, 60).unwrap(), hi);

        let lo = ceil_to_spacing(MIN_TICK, 60).unwrap();
        assert_eq!(dear_edge(lo, 60).unwrap(), lo - 120);
    }




    #[test]
    fn dear_edge_and_publish_are_checked() {
        assert_eq!(dear_edge(i32::MIN, 60), Err(crate::error::FloorMathError::Overflow));
        assert_eq!(dear_edge(i32::MIN + 120, 60), Ok(i32::MIN));
        assert_eq!(dear_edge(i32::MIN + 119, 60), Err(crate::error::FloorMathError::Overflow));
        assert_eq!(publish(i32::MAX, 60), Err(crate::error::FloorMathError::Overflow));
        assert_eq!(publish(i32::MAX - 119, 60), Err(crate::error::FloorMathError::Overflow));


        assert_eq!(publish(i32::MAX - 120, 60), Ok(443_520));
    }


    #[test]
    fn zero_width_reads_as_one_spacing() {
        assert_eq!(bid_width_or_spacing(0, 60), 60);
        assert_eq!(bid_width_or_spacing(120, 60), 120);
        assert_eq!(bid_width_or_spacing(60, 60), 60);
    }

    #[test]
    fn live_floor_tick_is_publish() {
        assert_eq!(live_floor_tick(-120, 60).unwrap(), 0);
        assert_eq!(live_floor_tick(443_400, 60).unwrap(), 443_520);
    }



}
