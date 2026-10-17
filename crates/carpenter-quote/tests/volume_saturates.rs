//!
//!
//!
use carpenter_quote::clmm::tick_math;
use carpenter_quote::synth::{build_pool, floor_pool_defaults, PoolKeys, SynthKind, SynthPosition};
use carpenter_quote::types::ids::FLOOR_HOOK_ID;
use carpenter_quote::{no_limit, quote_swap, SwapQuoteParams};

fn keys() -> PoolKeys {
    PoolKeys { pool: [7; 32], mint0: [1; 32], mint1: [2; 32], vault0: [3; 32], vault1: [4; 32],
        token_program0: [5; 32], token_program1: [6; 32], hook: FLOOR_HOOK_ID, hook_authority: [9; 32] }
}

#[test]
fn a_pinned_volume_counter_never_stops_the_quoter() {
    let tick = 100_020;
    let k = keys();
    let sqrt = tick_math::get_sqrt_price_at_tick(tick).unwrap();
    let sp = build_pool(&k, SynthKind::Floor, 60, sqrt, 9, 6,
        &[SynthPosition { tick_lower: tick - 12_000, tick_upper: tick + 12_000, liquidity: 50_000_000_000_000 }]).unwrap();
    let set = sp.tick_array_set().unwrap();
    let mut fp = floor_pool_defaults(k.pool, k.mint0, k.mint1, [8; 32]);
    fp.ladder_seeded = 1;
    fp.floor_tick = tick + 60;
    fp.skim_buy_far_bps = 250;
    fp.creator_buy_bps = 0;
    fp.holder_buy_bps = 0;




    fp.team_fee_bps = 0;
    fp.cumulative_volume_eth = 1_000;


    let p = SwapQuoteParams { zero_for_one: true, amount_specified: -(u64::MAX as i128), sqrt_price_limit_x64: sqrt - 1,
        supply0: 0, cushion_top: None, supply1: 1_000_000_000_000_000 };
    let q = quote_swap(&sp.account, &set, Some(&fp), &p).expect("the chain accepts the phantom buy; so must the quote");
    assert!(q.amount_in <= 2, "almost nothing trades");
    assert_eq!(q.fees.hook_total(), 0, "no fee on the nominal amount");
    assert_eq!(q.floor_after.map(|f| f.cumulative_volume_eth), Some(u64::MAX), "saturated, as on chain");

    fp.cumulative_volume_eth = u64::MAX;
    let buy = SwapQuoteParams { zero_for_one: true, amount_specified: -1_000_000_000, sqrt_price_limit_x64: no_limit(true),
        supply0: 0, cushion_top: None, supply1: 1_000_000_000_000_000 };
    let sell = SwapQuoteParams { zero_for_one: false, sqrt_price_limit_x64: no_limit(false), ..buy };
    let (b, s) = (quote_swap(&sp.account, &set, Some(&fp), &buy), quote_swap(&sp.account, &set, Some(&fp), &sell));
    assert!(b.unwrap().amount_out > 0, "a buy still quotes");
    assert!(s.unwrap().amount_out > 0, "a sell still quotes");
}
