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
use floor_math::netflow::{cushion_netflow_on, flow_cap, note_flow, NetFlow};

use crate::mech::positions::{math, Mech};
use crate::state::twap::TwapOracle;
use crate::state::FloorPool;

///
pub fn meter_of(floor_pool: &AccountInfo) -> Result<NetFlow> {
    let data = floor_pool.try_borrow_data()?;
    Ok(TwapOracle::from_account(&data).map_or(NetFlow::default(), |o| {
        let (a, b) = o.net_flow();
        NetFlow { a, b }
    }))
}

///
///
///
///
pub fn note_account(floor_pool: &AccountInfo, is_buy: bool, v: u64, pre: i32, post: i32, supply: u64) -> Result<()> {
    if !cushion_netflow_on() || v == 0 {
        return Ok(());
    }
    let mut data = floor_pool.try_borrow_mut_data()?;
    let Some(mut o) = TwapOracle::from_account(&data) else { return Ok(()) };
    let (a, b) = o.net_flow();
    let mcap = floor_math::quote_cost(supply, post).unwrap_or(u128::MAX);
    let f = note_flow(NetFlow { a, b }, is_buy, v, pre, post, mcap);
    if (f.a, f.b) != (a, b) {
        o.set_net_flow(f.a, f.b);
        o.store(&mut data);
    }
    Ok(())
}

///
///
///
///
pub fn cap_target(m: &Mech, settled: i32, standing_lower: Option<i32>) -> Result<i32> {
    if !cushion_netflow_on() || standing_lower.is_none() {
        return Ok(settled);
    }
    Ok(flow_cap(settled, standing_lower, meter_of(&m.cpi.floor_pool)?))
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
///
#[inline(never)]
pub fn climb_line(
    m: &Mech,
    s: &FloorPool,
    supply: u64,
    reserve: u64,
    cushion: Option<(i32, i32, u128)>,
    width: i32,
    stand: i32,
) -> Result<Option<i32>> {
    if !cushion_netflow_on() {
        return Ok(None);
    }
    let vq = s.vault_eth.saturating_sub(reserve);


    let hi = s.floor_tick;
    let lo = math::floor_to_spacing(stand.min(hi) - math::band_ticks(s)? - 2 * s.tick_spacing, s.tick_spacing)?;
    floor_math::netflow::climb_line_within(m.market.sqrt_price_x64, m.market.tick, math::state(s), supply, vq, cushion, width, lo, hi)
        .map_err(|_| error!(crate::errors::HookError::MathOverflow))
}

#[cfg(test)]
mod tests {
///
///
///
///
    #[test]
    fn the_hold_is_bracketed_as_the_property_test_proves() {
        let src = include_str!("netflow.rs");
        let body = &src[src.find("pub fn climb_line(").unwrap()..src.find("#[cfg(test)]").unwrap()];
        assert!(body.contains("let hi = s.floor_tick;"), "hi is the standing floor");
        assert!(
            body.contains("let lo = math::floor_to_spacing(stand.min(hi) - math::band_ticks(s)? - 2 * s.tick_spacing, s.tick_spacing)?;"),
            "lo is the passes' lower edge off the stand line"
        );
        assert!(body.contains("climb_line_within("), "the bracketed solve");
        let run = include_str!("ratchet.rs");
        let stand = run.find("let stand0 = math::ceil_to_spacing(m.market.tick + 1 + math::band_ticks(s)?, s.tick_spacing)?;").expect("stand0");
        let hold = run.find("crate::mech::netflow::climb_line(m, s, supply, reserve, cushion, out.cushion_ticks, stand0)").expect("the hold");
        assert!(stand < hold, "the stand line is read before the hold");

        let c1 = run.find("let lift_pub = math::ceil_to_spacing(m.market.tick + 1 + math::band_ticks(s)?, s.tick_spacing)?;").expect("C1");
        assert!(hold < c1, "the hold before C1's cap");
    }
}
