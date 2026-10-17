//!
//!
//!
//!
//!

use carpenter_quote::clmm::{liquidity_math, tick_math};
use carpenter_quote::floor::{self, AfterSwapIn, FeesTaken, SwapCtx};
use carpenter_quote::synth::{build_pool, floor_pool_defaults, PoolKeys, SynthKind, SynthPool, SynthPosition};
use carpenter_quote::types::constants::{MAX_SQRT_PRICE_X64, MIN_SQRT_PRICE_X64};
use carpenter_quote::types::hook::{BalanceDelta, FEE_CREATOR, FEE_HOLDER, FEE_PLATFORM, FEE_SKIM, FEE_TEAM};
use carpenter_quote::types::ids::FLOOR_HOOK_ID;
use carpenter_quote::types::layout::FloorPool;
use carpenter_quote::mech::floor_to_spacing;
use carpenter_quote::{no_limit, quote_swap, QuoteError, SwapQuoteParams, TickArraySet};

fn keys(hook: [u8; 32]) -> PoolKeys {
    PoolKeys {
        pool: [7; 32],
        mint0: [1; 32],
        mint1: [2; 32],
        vault0: [3; 32],
        vault1: [4; 32],
        token_program0: [5; 32],
        token_program1: [6; 32],
        hook,
        hook_authority: if hook == [0; 32] { [0; 32] } else { [9; 32] },
    }
}

fn plain(fee: u32, spacing: u16, tick: i32, positions: &[SynthPosition]) -> SynthPool {
    let sqrt = tick_math::get_sqrt_price_at_tick(tick).unwrap();
    build_pool(&keys([0; 32]), SynthKind::Plain { fee }, spacing, sqrt, 9, 6, positions).unwrap()
}

fn params(zfo: bool, amount: i128) -> SwapQuoteParams {
    SwapQuoteParams { zero_for_one: zfo, amount_specified: amount, sqrt_price_limit_x64: no_limit(zfo), supply0: 0, cushion_top: None, supply1: 1_000_000_000_000_000 }
}

fn pos(lo: i32, up: i32, l: u128) -> SynthPosition {
    SynthPosition { tick_lower: lo, tick_upper: up, liquidity: l }
}

#[test]
fn exact_in_inside_one_range_matches_the_closed_form() {
    let sp = plain(3_000, 60, 0, &[pos(-6_000, 6_000, 10_000_000_000_000)]);
    let set = sp.tick_array_set().unwrap();
    let q = quote_swap(&sp.account, &set, None, &params(true, -1_000_000)).unwrap();
    assert_eq!(q.amount_in, 1_000_000);

    let less_fee = 1_000_000u64 * (1_000_000 - 3_000) / 1_000_000;
    assert_eq!(q.fees.lp, 1_000_000 - less_fee);
    let start = tick_math::get_sqrt_price_at_tick(0).unwrap();
    let out = liquidity_math::get_delta_amount_1_unsigned(q.sqrt_after, start, 10_000_000_000_000, false).unwrap();
    assert_eq!(q.amount_out, out);
    assert!(q.sqrt_after < start);

    assert_eq!(q.arrays_touched, vec![-7_200]);
    assert_eq!(q.pool_after.swap_count, 1);
    assert_eq!(q.pool_after.volume0.get(), 1_000_000);
    assert_eq!(q.pool_after.lp_fees0.get(), q.fees.lp as u128);
    let growth = ((q.fees.lp as u128) << 64) / 10_000_000_000_000u128;
    assert_eq!(q.pool_after.fee_growth_global0_x64.get(), growth);
}

#[test]
fn crossing_a_tick_flips_outside_growth_and_changes_liquidity() {


    let sp = plain(500, 60, 0, &[pos(-600, 600, 5_000_000_000_000), pos(-60_000, 60_000, 1_000_000_000_000)]);
    let set = sp.tick_array_set().unwrap();
    let q = quote_swap(&sp.account, &set, None, &params(true, -200_000_000_000)).unwrap();
    assert!(q.tick_after < -600 && q.tick_after > -60_000, "crossed -600 only: {}", q.tick_after);
    assert_eq!(q.pool_after.liquidity.get(), 1_000_000_000_000);
    let arr = q.arrays_after.iter().find(|a| a.start_index == -3_600).expect("array holding -600");
    let t = &arr.ticks[((-600 - -3_600) / 60) as usize];
    assert_ne!(t.fee_growth_outside0_x64.get(), 0, "outside growth flipped to the in-flight global");
}

#[test]
fn exact_out_never_charges_more_than_the_exact_in_that_produced_it() {
    let sp = plain(3_000, 10, 1_234, &[pos(-10_000, 10_000, 77_000_000_000), pos(1_000, 2_000, 9_000_000_000_000)]);
    let set = sp.tick_array_set().unwrap();
    for amount in [1_000u64, 123_456, 9_999_999, 5_000_000_000] {
        for zfo in [true, false] {
            let a = quote_swap(&sp.account, &set, None, &params(zfo, -(amount as i128))).unwrap();
            if a.amount_out == 0 {
                continue;
            }
            let b = quote_swap(&sp.account, &set, None, &params(zfo, a.amount_out as i128)).unwrap();
            assert_eq!(b.amount_out, a.amount_out);


            assert!(b.amount_in <= a.amount_in + 4, "zfo {zfo} amount {amount}: {} > {}", b.amount_in, a.amount_in);
        }
    }
}

#[test]
fn zero_liquidity_walks_to_the_limit_and_moves_nothing() {
    let sp = plain(3_000, 60, 0, &[]);
    let set = sp.tick_array_set().unwrap();
    let limit = tick_math::get_sqrt_price_at_tick(-120).unwrap();
    let p = SwapQuoteParams { sqrt_price_limit_x64: limit, ..params(true, -1_000) };
    let q = quote_swap(&sp.account, &set, None, &p).unwrap();
    assert_eq!((q.amount_in, q.amount_out), (0, 0));
    assert_eq!(q.sqrt_after, limit);
}

#[test]
fn r12_a_full_fee_refuses_exact_out_and_eats_exact_in() {
    let sp = plain(1_000_000, 60, 0, &[pos(-600, 600, 1_000_000_000_000)]);
    let set = sp.tick_array_set().unwrap();
    assert_eq!(quote_swap(&sp.account, &set, None, &params(true, 1_000)).unwrap_err(), QuoteError::InvalidFeeForExactOut);
    let q = quote_swap(&sp.account, &set, None, &params(true, -1_000)).unwrap();
    assert_eq!((q.amount_in, q.amount_out, q.fees.lp), (1_000, 0, 1_000));
}

#[test]
fn refusals_carry_the_chain_names() {
    let sp = plain(3_000, 60, 0, &[pos(-600, 600, 1_000_000_000_000)]);
    let set = sp.tick_array_set().unwrap();
    assert_eq!(quote_swap(&sp.account, &set, None, &params(true, 0)).unwrap_err().name(), "AmountZero");
    let wrong_side = SwapQuoteParams { sqrt_price_limit_x64: MAX_SQRT_PRICE_X64 - 1, ..params(true, -5) };
    assert_eq!(quote_swap(&sp.account, &set, None, &wrong_side).unwrap_err().name(), "PriceLimitInvalid");
    let at_bound = SwapQuoteParams { sqrt_price_limit_x64: MIN_SQRT_PRICE_X64, ..params(true, -5) };
    assert_eq!(quote_swap(&sp.account, &set, None, &at_bound).unwrap_err().name(), "PriceLimitInvalid");
    let too_big = params(false, -(u64::MAX as i128) - 1);
    let e = quote_swap(&sp.account, &set, None, &too_big).unwrap_err();
    assert_eq!(e, QuoteError::AmountTooLarge);


    assert_eq!(e.name(), "MathOverflow");
}

///
///
///
///
#[test]
fn the_spacing_snaps_are_floor_maths_at_the_bounds() {
    use carpenter_quote::price;
    use carpenter_quote::types::constants::{FLOOR_TICK_SPACING, MAX_TICK, MIN_TICK};
    let s = i32::from(FLOOR_TICK_SPACING);
    assert_eq!(price::ceil_to_spacing(MAX_TICK, FLOOR_TICK_SPACING).unwrap(), carpenter_quote::mech::ceil_to_spacing(MAX_TICK, s).unwrap());
    assert_eq!(price::floor_to_spacing(MIN_TICK, FLOOR_TICK_SPACING).unwrap(), carpenter_quote::mech::floor_to_spacing(MIN_TICK, s).unwrap());

    assert_eq!(price::ceil_to_spacing(MAX_TICK, 60).unwrap(), 443_520);
    assert_eq!(price::floor_to_spacing(MIN_TICK, 60).unwrap(), -443_520);


    let fp = floor_pool_defaults([1; 32], [2; 32], [3; 32], [4; 32]);
    assert_eq!(fp.floor_tick, carpenter_quote::mech::ceil_to_spacing(MAX_TICK, s).unwrap());

    assert_eq!(price::ceil_to_spacing(i32::MIN, 60).unwrap_err().name(), "TickOutOfRange");
    assert_eq!(price::floor_to_spacing(i32::MIN, 60).unwrap_err().name(), "TickOutOfRange");

    assert_eq!(price::max_usable_tick(60).unwrap(), 443_580);
    assert_eq!(price::min_usable_tick(60).unwrap(), -443_580);
}

#[test]
fn a_marked_array_that_was_not_supplied_is_named() {
    let sp = plain(3_000, 60, 0, &[pos(-7_200, 7_200, 1_000_000_000_000), pos(-600, 600, 1_000_000_000_000)]);

    let arrays: Vec<_> = sp.arrays.iter().copied().filter(|a| a.start_index != -7_200).collect();
    let set = TickArraySet::new(&sp.account.key, 60, arrays, Some(sp.words.clone())).unwrap();
    let err = quote_swap(&sp.account, &set, None, &params(true, -(u64::MAX as i128))).unwrap_err();
    assert_eq!(err, QuoteError::TickArrayMissing { start_index: -7_200 });
}

#[test]
fn quoting_twice_is_identical_and_sequential_quotes_compose() {
    let sp = plain(3_000, 60, 60, &[pos(-6_000, 6_000, 10_000_000_000_000), pos(-120, 240, 3_000_000_000_000)]);
    let set = sp.tick_array_set().unwrap();
    let a = quote_swap(&sp.account, &set, None, &params(false, -777_777)).unwrap();
    let b = quote_swap(&sp.account, &set, None, &params(false, -777_777)).unwrap();
    assert_eq!(bytemuck::bytes_of(&a.pool_after), bytemuck::bytes_of(&b.pool_after));
    assert_eq!((a.amount_in, a.amount_out), (b.amount_in, b.amount_out));


    let mut next = sp.account;
    next.pool = a.pool_after;
    let mut arrays = set.arrays.clone();
    for arr in &a.arrays_after {
        arrays.insert(arr.start_index, *arr);
    }
    let set2 = TickArraySet::new(&next.key, 60, arrays.into_values().collect(), Some(set.words.clone())).unwrap();
    let c = quote_swap(&next, &set2, None, &params(true, -1_000)).unwrap();
    assert_eq!(c.pool_after.swap_count, 2);
}



fn floor_pool(tick: i32) -> (SynthPool, FloorPool) {
    let k = keys(FLOOR_HOOK_ID);
    let sqrt = tick_math::get_sqrt_price_at_tick(tick).unwrap();
    let sp = build_pool(&k, SynthKind::Floor, 60, sqrt, 9, 6, &[pos(tick - 60 * 200, tick + 60 * 200, 50_000_000_000_000)]).unwrap();
    let mut fp = floor_pool_defaults(k.pool, k.mint0, k.mint1, [8; 32]);
    fp.ladder_seeded = 1;
    fp.floor_tick = tick + 60 * 500;
    (sp, fp)
}

///
///
///
///
///
#[test]
fn an_unseeded_floor_pool_refuses_every_swap_with_not_seeded() {
    let (sp, mut fp) = floor_pool(100_020);
    let set = sp.tick_array_set().unwrap();
    for seeded in [0u8, 2, 0xFF] {
        fp.ladder_seeded = seeded;
        for zfo in [true, false] {
            for amount in [-1i128, -1_000, -1_000_000_000_000, 1_000] {
                assert_eq!(quote_swap(&sp.account, &set, Some(&fp), &params(zfo, amount)).unwrap_err(), QuoteError::NotSeeded, "seeded {seeded} zfo {zfo} amount {amount}");
            }
        }
    }
    assert_eq!(QuoteError::NotSeeded.name(), "NotSeeded", "the program's own name (floor_hook HookError::NotSeeded)");
    fp.ladder_seeded = 0;
    fp.pool_bound = 0;
    assert_eq!(quote_swap(&sp.account, &set, Some(&fp), &params(true, -1_000)).unwrap_err(), QuoteError::NotBound, "NotBound first");
    fp.pool_bound = 1;
    fp.amm_pool = [0xAB; 32];
    assert_eq!(quote_swap(&sp.account, &set, Some(&fp), &params(true, -1_000)).unwrap_err(), QuoteError::WrongPool, "WrongPool first");
    fp.amm_pool = sp.account.key;
    fp.ladder_seeded = 1;
    assert!(quote_swap(&sp.account, &set, Some(&fp), &params(true, -1_000)).is_ok(), "seeded: quotes");
}

#[test]
fn floor_refuses_exact_out_by_name() {
    let (sp, fp) = floor_pool(100_020);
    let set = sp.tick_array_set().unwrap();
    for zfo in [true, false] {
        assert_eq!(quote_swap(&sp.account, &set, Some(&fp), &params(zfo, 1_000)).unwrap_err(), QuoteError::ExactOutNotSupported);
    }
}

#[test]
fn floor_needs_its_state_and_binds_its_pool() {
    let (sp, mut fp) = floor_pool(100_020);
    let set = sp.tick_array_set().unwrap();
    assert_eq!(quote_swap(&sp.account, &set, None, &params(true, -1_000)).unwrap_err(), QuoteError::QuoteHookStateMissing);
    fp.amm_pool = [0xAB; 32];
    assert_eq!(quote_swap(&sp.account, &set, Some(&fp), &params(true, -1_000)).unwrap_err(), QuoteError::WrongPool);
    fp.amm_pool = sp.account.key;
    fp.pool_bound = 0;
    assert_eq!(quote_swap(&sp.account, &set, Some(&fp), &params(true, -1_000)).unwrap_err(), QuoteError::NotBound);




    fp.pool_bound = 1;




    let mut swapped = fp;
    swapped.quote_is_token1 = 1;
    assert_eq!(quote_swap(&sp.account, &set, Some(&swapped), &params(true, -1_000)).unwrap_err(), QuoteError::QuoteReverseOrientation);
    let mut agreed = swapped;
    agreed.quote_mint = sp.account.pool.mint1;
    agreed.base_mint = sp.account.pool.mint0;
    assert_eq!(quote_swap(&sp.account, &set, Some(&agreed), &params(true, -1_000)).unwrap_err(), QuoteError::QuoteReverseOrientation);
    assert_eq!(QuoteError::QuoteReverseOrientation.name(), "QuoteReverseOrientation", "quoter-only: no on-chain twin");
    let mut foreign = fp;
    foreign.base_mint = [0xEE; 32];
    assert_eq!(quote_swap(&sp.account, &set, Some(&foreign), &params(true, -1_000)).unwrap_err(), QuoteError::WrongPool);
    let mut mis_spaced = fp;
    mis_spaced.tick_spacing = 10;
    assert_eq!(quote_swap(&sp.account, &set, Some(&mis_spaced), &params(true, -1_000)).unwrap_err(), QuoteError::WrongPool);




    let mut static_fee = sp.clone();
    static_fee.account.pool.fee = 3_000;
    static_fee.account.pool.lp_fee = 3_000;
    let set2 = static_fee.tick_array_set().unwrap();
    assert_eq!(quote_swap(&static_fee.account, &set2, Some(&fp), &params(true, -1_000)).unwrap_err(), QuoteError::NotDynamicFee);

    assert!(quote_swap(&sp.account, &set, Some(&fp), &params(true, -1_000)).is_ok());
}

#[test]
fn floor_buy_far_from_the_floor_pays_the_far_skim_and_the_creator_fee() {
    let (sp, fp) = floor_pool(100_020);
    let set = sp.tick_array_set().unwrap();
    let leg: u64 = 1_000_000_000;
    let q = quote_swap(&sp.account, &set, Some(&fp), &params(true, -(leg as i128))).unwrap();

    let skim = leg * u64::from(fp.skim_buy_far_bps) / 10_000;
    let creator = leg * u64::from(fp.creator_buy_bps) / 10_000;

    let team = leg * 25 / 10_000;
    assert_eq!(q.fees.skim, skim);
    assert_eq!(q.fees.creator, creator);
    assert_eq!(q.fees.team, team);
    assert_eq!(q.fees.platform_cut, skim * 250 / 10_000);




    assert_eq!(q.fees.holder, 0);
    assert_eq!(q.fees.hook_total() as i128, q.hook_delta.amount0);
    assert_eq!(q.amount_in, leg, "exact-in: the trader pays exactly what they specified");
    assert_eq!(q.hook_delta.amount0, (skim + creator + team) as i128);
    let f = q.floor_after.unwrap();
    assert_eq!(f.vault_eth, skim - skim * 250 / 10_000);
    assert_eq!(f.creator_fees_owed, creator);
    assert_eq!(f.cumulative_volume_eth, leg);
    assert_eq!(q.pool_after.hook_claims0, skim + creator + team);

    let swapped = leg - skim - creator - team;
    assert_eq!(q.fees.lp, swapped - swapped * (1_000_000 - 2_500) / 1_000_000);
    assert!(q.replace_actions.is_empty() && q.replace.is_none(), "a buy never re-places the bid");
}



///
///
///
///
///
///
fn graduated(open: i32, pol_lower: i32, pol_upper: i32) -> (SynthPool, FloorPool, i32) {
    let s = 60i32;
    let lad_lower = floor_to_spacing(open - 20_794, s).unwrap();
    let lad_upper = floor_to_spacing(open - 6_932, s).unwrap();
    let k = keys(FLOOR_HOOK_ID);
    let sqrt = tick_math::get_sqrt_price_at_tick(open).unwrap();
    let positions = [pos(pol_lower, pol_upper, 40_000_000_000_000), pos(lad_lower, lad_upper, 20_000_000_000_000)];
    let sp = build_pool(&k, SynthKind::Floor, 60, sqrt, 9, 6, &positions).unwrap();
    let mut fp = floor_pool_defaults(k.pool, k.mint0, k.mint1, [8; 32]);
    fp.ladder_seeded = 1;
    fp.seed_lower = pol_lower;
    fp.seed_upper = pol_upper;
    fp.ladder_lower = lad_lower;
    fp.ladder_upper = lad_upper;
    (sp, fp, lad_lower)
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
///
///
#[test]
fn a_graduated_pool_fills_a_buy_past_the_ladder_top_only_with_the_full_range_pol() {
    let open = 80_040;
    let leg: i128 = -50_000_000_000_000;
    let limit = no_limit(true);



    let (sp, fp, lad_lower) = graduated(open, floor_to_spacing(open - 6_932, 60).unwrap(), floor_to_spacing(open, 60).unwrap());
    let bounded = quote_swap(&sp.account, &sp.tick_array_set().unwrap(), Some(&fp), &params(true, leg)).unwrap();
    assert_eq!(
        bounded.sqrt_after, limit,
        "the bounded POL runs out at the ladder's far end and the price walks free to the bound"
    );


    let (sp, fp, lad_lower_full) = graduated(open, -443_580, floor_to_spacing(open, 60).unwrap());
    assert_eq!(lad_lower, lad_lower_full);
    let full = quote_swap(&sp.account, &sp.tick_array_set().unwrap(), Some(&fp), &params(true, leg)).unwrap();
    assert!(full.sqrt_after > limit, "the tail is still a counterparty, so the price settles rather than running out");
    let after = tick_math::get_tick_at_sqrt_price(full.sqrt_after).unwrap();
    assert!(after < lad_lower, "and the buy really did walk past the ladder top ({after} < {lad_lower})");
    assert!(
        full.amount_out > bounded.amount_out,
        "the same SOL buys strictly more with a counterparty above 8× ({} > {})",
        full.amount_out,
        bounded.amount_out
    );





    let sell = params(false, -1_000_000_000_000);
    let (sp_b, fp_b, _) = graduated(open, floor_to_spacing(open - 6_932, 60).unwrap(), floor_to_spacing(open, 60).unwrap());
    let (sp_f, fp_f, _) = graduated(open, -443_580, floor_to_spacing(open, 60).unwrap());
    let sb = quote_swap(&sp_b.account, &sp_b.tick_array_set().unwrap(), Some(&fp_b), &sell).unwrap();
    let sf = quote_swap(&sp_f.account, &sp_f.tick_array_set().unwrap(), Some(&fp_f), &sell).unwrap();
    assert_eq!((sb.amount_in, sb.amount_out, sb.sqrt_after), (sf.amount_in, sf.amount_out, sf.sqrt_after));
    assert_eq!(sb.amount_out, 0, "nothing is seeded above the open in either arrangement");
}

///
///
///
///
///
///
///
#[test]
fn a_holder_tax_is_reported_and_closes_the_fee_identity() {
    let (sp, mut fp) = floor_pool(100_020);
    fp.holder_buy_bps = 50;
    let set = sp.tick_array_set().unwrap();
    let leg: u64 = 1_000_000_000;
    let q = quote_swap(&sp.account, &set, Some(&fp), &params(true, -(leg as i128))).unwrap();
    let holder = leg * u64::from(fp.holder_buy_bps) / 10_000;
    assert_eq!(q.fees.holder, holder);
    assert_ne!(holder, 0);
    assert_eq!(q.fees.hook_total() as i128, q.hook_delta.amount0, "skim + creator + holder + team IS the hook delta");
    assert_eq!(q.floor_after.unwrap().holder_owed, holder);
    assert_eq!(q.pool_after.hook_claims0, q.fees.hook_total());
}

///
#[test]
fn a_floor_quote_carries_the_team_fee_on_top() {
    let (sp, fp) = floor_pool(100_020);
    assert_eq!(fp.team_fee_bps, 25);
    let set = sp.tick_array_set().unwrap();
    let q = quote_swap(&sp.account, &set, Some(&fp), &params(true, -1_000_000_000)).unwrap();
    assert_eq!((q.fees.team, q.fee_split0[FEE_TEAM], q.floor_after.unwrap().team_owed), (2_500_000, 2_500_000, 2_500_000));
    assert_eq!(q.fees.hook_total(), q.fees.skim + q.fees.creator + q.fees.holder + q.fees.team);
    let mut old = fp;
    old.team_fee_bps = 0;
    let q0 = quote_swap(&sp.account, &set, Some(&old), &params(true, -1_000_000_000)).unwrap();
    assert_eq!(q0.fees.team, 0);
    assert!(q.amount_out < q0.amount_out);
}

#[test]
fn a_tiny_leg_quotes_no_team_fee() {
    let (sp, fp) = floor_pool(100_020);
    let set = sp.tick_array_set().unwrap();
    let q = quote_swap(&sp.account, &set, Some(&fp), &params(true, -399)).unwrap();
    assert_eq!((q.fees.team, q.floor_after.unwrap().team_owed), (0, 0));
}

///
///
///
///
///
#[test]
fn a_floor_quote_follows_the_pools_own_rate_the_cap_a_lower_one_or_zero() {
    let (sp, fp) = floor_pool(100_020);
    let set = sp.tick_array_set().unwrap();
    let mut outs = Vec::new();
    for (bps, fee) in [(carpenter_quote::types::constants::TEAM_POOL_FEE_BPS_MAX, 5_000_000u64), (10, 1_000_000), (0, 0)] {
        let mut p = fp;
        p.team_fee_bps = bps;
        let q = quote_swap(&sp.account, &set, Some(&p), &params(true, -1_000_000_000)).unwrap();
        assert_eq!((q.fees.team, q.fee_split0[FEE_TEAM], q.floor_after.unwrap().team_owed), (fee, fee, fee), "{bps} bps");
        outs.push(q.amount_out);
    }
    assert!(outs[0] < outs[1] && outs[1] < outs[2], "the dearer the pool's own rate, the fewer tokens out: {outs:?}");
}

///
fn ctx_for(sp: &SynthPool, zfo: bool, amount_specified: i128) -> SwapCtx {
    let p = &sp.account.pool;
    SwapCtx {
        pool_key: sp.account.key,
        zero_for_one: zfo,
        amount_specified,
        pre_tick: p.tick_current,
        supply1: params(zfo, amount_specified).supply1,
        claims0: p.hook_claims0,
        claims1: p.hook_claims1,
        mint0: p.mint0,
        mint1: p.mint1,
        tick_spacing: p.tick_spacing,



        fee: p.fee,
        cushion_top: None,
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
#[test]
fn the_record_s_fee_split_is_the_breakdown_in_the_ruling_s_order() {

    let (sp, fp) = floor_pool(100_020);
    let set = sp.tick_array_set().unwrap();
    let leg: u64 = 1_000_000_000;
    let q = quote_swap(&sp.account, &set, Some(&fp), &params(true, -(leg as i128))).unwrap();
    let mut state = fp;
    let mut taken = FeesTaken::default();
    let rec = floor::before_swap(&mut state, &ctx_for(&sp, true, -(leg as i128)), &mut taken).unwrap();
    assert_eq!(rec.fees, [q.fees.skim, q.fees.creator, q.fees.holder, q.fees.platform_cut, q.fees.team], "[skim, creator, holder, platform, team]");
    assert_eq!(rec.fees, [taken.skim, taken.creator, taken.holder, taken.platform_cut, taken.team]);
    assert_ne!(rec.fees[FEE_SKIM], 0);
    assert_ne!(rec.fees[FEE_CREATOR], 0);
    assert_ne!(rec.fees[FEE_SKIM], rec.fees[FEE_CREATOR], "the two non-zero entries are distinguishable");
    assert_eq!(rec.fees[FEE_HOLDER], 0, "the default state carries a zero holder rate");
    assert_eq!(rec.fees_charged(), Some(rec.delta_specified as u64), "the split explains delta_specified exactly");
    assert_eq!(rec.fee_delta(), Some(rec.delta_specified));
    assert_eq!(rec.delta_unspecified, 0);

    assert_eq!(rec.fees[FEE_PLATFORM], rec.fees[FEE_SKIM] * 250 / 10_000);
    assert!(rec.fees[FEE_PLATFORM] < rec.fees[FEE_SKIM]);
    assert_eq!(rec.fees[FEE_SKIM] - rec.fees[FEE_PLATFORM], state.vault_eth, "the vault keeps skim − platform");

    assert_eq!((q.fee_split0, q.fee_split1), (rec.fees, [0; 5]));
    assert_eq!(q.fee_split0[FEE_SKIM] + q.fee_split0[FEE_CREATOR] + q.fee_split0[FEE_HOLDER] + q.fee_split0[FEE_TEAM], q.hook_delta.amount0 as u64);
    assert_eq!(q.hook_delta.amount1, 0);

    assert_eq!(bytemuck::bytes_of(&state), bytemuck::bytes_of(&q.floor_after.unwrap()));


    let (sp, fp) = floor_pool(100_020);
    let sell: i128 = -50_000_000;
    let mut state = fp;
    let mut taken = FeesTaken::default();
    let ctx = ctx_for(&sp, false, sell);
    let rec0 = floor::before_swap(&mut state, &ctx, &mut taken).unwrap();
    assert_eq!(rec0.fees, [0; 5], "before_swap takes nothing when the quote leg is unspecified");
    assert_eq!(rec0.delta_specified, 0);

    let out: i128 = 700_000_000;
    let post_tick = sp.account.pool.tick_current + 3;
    let a = AfterSwapIn {
        post_sqrt_price_x64: tick_math::get_sqrt_price_at_tick(post_tick).unwrap(),
        post_tick,
        swap_delta: BalanceDelta { amount0: out, amount1: sell },
        before_delta_specified: 0,
        before_delta_unspecified: 0,
    };
    let rec1 = floor::after_swap(&mut state, &ctx, &a, &mut taken).unwrap();
    assert_eq!(rec1.fees, [taken.skim, taken.creator, taken.holder, taken.platform_cut, taken.team], "[skim, creator, holder, platform, team]");
    assert_ne!(rec1.fees[FEE_SKIM], 0);
    assert_ne!(rec1.fees[FEE_CREATOR], 0);
    assert_ne!(rec1.fees[FEE_SKIM], rec1.fees[FEE_CREATOR]);
    assert_eq!(rec1.delta_specified, 0);
    assert_eq!(rec1.fees_charged(), Some(rec1.delta_unspecified as u64), "the split explains delta_unspecified exactly");
    assert_eq!(rec1.fee_delta(), Some(rec1.delta_unspecified));
    assert_eq!(rec1.fees[FEE_PLATFORM], rec1.fees[FEE_SKIM] * 250 / 10_000);

    assert_eq!(rec1.fees[FEE_SKIM], (out as u64) * u64::from(fp.skim_sell_far_bps) / 10_000);
    assert_eq!(rec1.fees[FEE_CREATOR], (out as u64) * u64::from(fp.creator_sell_bps) / 10_000);
    assert!(rec1.actions.is_empty() && !rec1.want_callback, "no lift: the vault is empty");

    let (f0, f1) = carpenter_quote::types::hook::HookDeltaAssembly::assemble_fees(sell, false, rec0.fees, rec1.fees);
    assert_eq!((f0, f1), (rec1.fees, [0; 5]));
}



///
///
///
///
fn replace_fixture(tick: i32, bid_quote: u64, vault: u64, pol_edge_at_dear: bool) -> (SynthPool, FloorPool) {
    let k = keys(FLOOR_HOOK_ID);
    let sqrt = tick_math::get_sqrt_price_at_tick(tick).unwrap();
    let floor_tick = tick + 300;
    let (lower, upper) = (floor_tick - 120, floor_tick);
    let bid_liq = carpenter_quote::mech::position_liquidity(sqrt, lower, upper, bid_quote, 0).unwrap();
    let mut positions = vec![pos(tick - 60 * 200, tick + 60 * 200, 50_000_000_000_000), pos(lower, upper, bid_liq)];
    if pol_edge_at_dear {
        positions.push(pos(lower - 6_000, lower, 20_000_000_000_000));
    }
    let mut sp = build_pool(&k, SynthKind::Floor, 60, sqrt, 9, 6, &positions).unwrap();
    sp.account.pool.hook_claims0 = vault;
    let mut fp = floor_pool_defaults(k.pool, k.mint0, k.mint1, [8; 32]);
    fp.ladder_seeded = 1;
    fp.floor_tick = floor_tick;
    fp.vault_eth = vault;
    fp.bid_lower = lower;
    fp.bid_width = upper - lower;
    fp.bid_liquidity.set(bid_liq);

    fp.bid_eth_deployed = carpenter_quote::amounts_for_liquidity(sqrt, tick, lower, upper, bid_liq as i128).unwrap().0;
    (sp, fp)
}

fn sell_to(limit_tick: i32) -> SwapQuoteParams {

    SwapQuoteParams { sqrt_price_limit_x64: tick_math::get_sqrt_price_at_tick(limit_tick).unwrap(), ..params(false, -900_000_000_000_000) }
}

#[test]
fn a_sell_that_stops_short_of_the_band_re_places_nothing() {
    let (sp, fp) = replace_fixture(100_020, 10_000_000_000, 1_000_000_000, true);
    let set = sp.tick_array_set().unwrap();
    let q = quote_swap(&sp.account, &set, Some(&fp), &sell_to(100_140)).unwrap();
    assert_eq!(q.tick_after_trade, 100_140);
    assert!(q.replace_actions.is_empty(), "the bid still holds no tokens: nothing to re-place");
    assert!(q.replace.is_none());
    let f = q.floor_after.unwrap();
    assert_eq!((f.bid_lower, f.bid_width, f.bid_liquidity.get()), (fp.bid_lower, 120, fp.bid_liquidity.get()));
    assert_eq!(f.needs_ratchet, 0);

    assert!(q.fees.skim > 0);
    let leg = (q.hook_delta.amount0 as u64) - q.fees.creator - q.fees.team;
    assert_eq!(leg, q.fees.skim);
}

#[test]
fn a_sell_into_the_band_closes_nudges_on_dust_and_re_opens_the_bid_narrowed_from_the_dear_side() {
    let vault = 1_000_000_000u64;
    let (sp, fp) = replace_fixture(100_020, 10_000_000_000, vault, true);
    let set = sp.tick_array_set().unwrap();
    let q = quote_swap(&sp.account, &set, Some(&fp), &sell_to(100_260)).unwrap();
    assert_eq!(q.tick_after_trade, 100_260, "the sell lands inside the band");


    use carpenter_quote::types::hook::HookAction as A;
    assert_eq!(q.replace_actions.len(), 3, "{:?}", q.replace_actions);
    let dear = tick_math::get_sqrt_price_at_tick(100_200).unwrap();
    let bid_eth_left = carpenter_quote::mech::bid_amounts(q.sqrt_after_trade, 100_200, 120, fp.bid_liquidity.get(), fp.bid_eth_deployed).unwrap().0;
    assert!(bid_eth_left > 0 && bid_eth_left < fp.bid_eth_deployed, "half filled: {bid_eth_left}");


    let kept_skim = q.fees.skim - q.fees.platform_cut;
    assert!(kept_skim > 0);
    let idle = vault + kept_skim;
    match q.replace_actions[0] {
        A::ModifyPosition { salt, tick_lower, tick_upper, liquidity_delta } => {
            assert_eq!((salt, tick_lower, tick_upper), (1, 100_200, 100_320));
            assert_eq!(liquidity_delta, -(fp.bid_liquidity.get() as i128));
        }
        a => panic!("round 1[0] is the close, got {a:?}"),
    }
    match q.replace_actions[1] {
        A::Swap { zero_for_one, amount_specified, sqrt_price_limit_x64 } => {
            assert!(zero_for_one, "a buy");


            let budget = (idle + bid_eth_left).min(fp.min_bid);
            assert!(budget < idle + bid_eth_left, "the cap BINDS here: {budget} < {}", idle + bid_eth_left);
            assert_eq!(amount_specified, -(budget as i128), "min(idle + bid_eth_left, min_bid)");
            assert_eq!(sqrt_price_limit_x64, dear, "limited AT the dear edge");
        }
        a => panic!("round 1[1] is the nudge, got {a:?}"),
    }
    let r = q.replace.expect("a re-place happened");


    assert_eq!((r.from_tick, r.to_tick), (100_260, 100_256), "the nudge stops where the dust ends");
    let b = r.booking;
    assert!(b.absorbed > 0, "the tokens the bid held joined the wall (fix 1)");
    assert_eq!(b.eth_back, bid_eth_left, "the close returns the bid's live quote (principal; fees unknown to a quote)");
    assert!(b.fees_unknown);
    assert!(b.nudge_spent > 0 && b.nudge_got > 0, "third-party liquidity sat in the gap: {b:?}");
    assert!(b.nudge_spent < idle + bid_eth_left);
    assert!(b.redeployed);




    assert_eq!((b.from_lower, b.to_lower, b.to_upper), (100_200, 100_260, 100_320), "narrowed from the dear side");
    match q.replace_actions[2] {
        A::ModifyPosition { salt, tick_lower, tick_upper, liquidity_delta } => {
            assert_eq!((salt, tick_lower, tick_upper), (1, 100_260, 100_320), "narrowed from the dear side (DV-13)");
            assert_eq!(liquidity_delta, b.bid_liquidity as i128);
            assert!(liquidity_delta > 0);
        }
        a => panic!("round 2[0] is the open, got {a:?}"),
    }


    let f = q.floor_after.unwrap();
    assert_eq!((f.bid_lower, f.bid_width), (100_260, 60), "one spacing wide: the dust nudge did not reach 100_200");
    assert_eq!(f.bid_liquidity.get(), b.bid_liquidity);
    assert_eq!(f.bid_eth_deployed, b.bid_eth_deployed);
    assert_eq!(f.needs_ratchet, 1);



    assert_eq!(f.wall_held_tokens, fp.wall_held_tokens, "the held reserve is untouched");
    assert_eq!(f.inventory_tokens, fp.inventory_tokens + b.absorbed + b.nudge_got);
    assert!(f.inventory_cost_eth > fp.inventory_cost_eth, "and their cost is kept");
    assert_eq!(f.vault_eth, idle + b.eth_back - b.nudge_spent - b.bid_eth_deployed);
    assert_eq!(f.floor_tick, fp.floor_tick, "the cheap edge never moves in a re-place");



    assert_eq!(q.tick_after, 100_256);


    assert!(q.sqrt_after > dear, "short of the dear edge: it never got down to it");



    let wide_only = 50_000_000_000_000u128;
    assert_eq!(q.pool_after.liquidity.get(), wide_only, "the new band AND the POL sit below the market: neither active");

    let sell_fee = q.hook_delta.amount0 as u64;
    assert_eq!(q.pool_after.hook_claims0, vault + sell_fee + b.eth_back - b.nudge_spent - b.bid_eth_deployed);





    let owed = floor_math::FloorState::quote_liabilities(bytemuck::cast_ref(&f));
    assert!(q.pool_after.hook_claims0 as u128 >= owed, "I-3 holds after the open");

    assert!(q.action_arrays.contains(&97_200), "{:?}", q.action_arrays);
    assert!(q.all_arrays().contains(&97_200));

    let again = quote_swap(&sp.account, &set, Some(&fp), &sell_to(100_260)).unwrap();
    assert_eq!(again.replace_actions, q.replace_actions);
    assert_eq!(bytemuck::bytes_of(&again.pool_after), bytemuck::bytes_of(&q.pool_after));
}

///
///
///
///
#[test]
fn an_uninitialised_dear_edge_narrows_the_re_placed_band_by_one_spacing() {
    let (sp, fp) = replace_fixture(100_020, 10_000_000_000, 1_000_000_000, false);
    let set = sp.tick_array_set().unwrap();
    let q = quote_swap(&sp.account, &set, Some(&fp), &sell_to(100_260)).unwrap();
    let r = q.replace.expect("a re-place happened");


    assert!(r.to_tick > 100_200, "stopped short of the dear edge: {}", r.to_tick);
    assert_eq!(r.to_tick, 100_256);
    assert_eq!((r.booking.to_lower, r.booking.to_upper), (100_260, 100_320));
    let f = q.floor_after.unwrap();
    assert_eq!((f.bid_lower, f.bid_width), (100_260, 60));
}

///
///
///
///
///
///
#[test]
fn a_sell_through_the_band_still_re_places_and_the_nudge_stops_where_the_budget_ends() {
    let vault = 1_000_000_000u64;
    let (sp, fp) = replace_fixture(100_020, 10_000_000_000, vault, true);
    let set = sp.tick_array_set().unwrap();
    let q = quote_swap(&sp.account, &set, Some(&fp), &sell_to(100_500)).unwrap();
    assert_eq!(q.tick_after_trade, 100_500);
    let r = q.replace.expect("a re-place happened");
    assert_eq!(r.booking.eth_back, 0, "fully crossed: the close returns no quote");
    assert!(r.booking.absorbed > 0);
    let idle = vault + q.fees.skim - q.fees.platform_cut;

    assert_eq!(r.booking.nudge_spent, idle.min(fp.min_bid), "the budget is capped at min_bid");
    assert!(r.booking.nudge_spent < idle, "the cap BINDS: one sell no longer empties the vault");
    assert!(q.tick_after > 100_200 && q.tick_after < 100_500, "stopped short of the edge: {}", q.tick_after);
    assert_eq!(q.tick_after, r.to_tick);
    let f = q.floor_after.unwrap();
    assert_eq!(f.floor_tick, fp.floor_tick, "the cheap edge is a promise; it does not move");

    assert!(f.inventory_cost_eth > fp.inventory_cost_eth, "the basis is kept");
    assert!(f.inventory_tokens > fp.inventory_tokens, "the absorbed tokens are the vault's inventory");




    assert!(r.booking.redeployed, "the bid is re-placed: the vault was not emptied");


    assert_eq!(f.vault_eth + f.bid_eth_deployed, idle - r.booking.nudge_spent, "nothing leaks");
    assert_eq!(q.replace_actions.len(), 3, "close, nudge, open: {:?}", q.replace_actions);
    assert!(f.bid_liquidity.get() > 0 && f.bid_eth_deployed > 0, "a real bid stands behind the floor");
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
///
#[test]
fn an_empty_gap_costs_the_dust_nudge_nothing_and_the_bid_comes_back_flush_with_the_pol() {

    let k = keys(FLOOR_HOOK_ID);
    let tick = 100_020;
    let sqrt = tick_math::get_sqrt_price_at_tick(tick).unwrap();
    let floor_tick = tick + 300;
    let (lower, upper) = (floor_tick - 120, floor_tick);
    let (bid_quote, vault) = (10_000_000_000u64, 1_000_000_000u64);
    let bid_liq = carpenter_quote::mech::position_liquidity(sqrt, lower, upper, bid_quote, 0).unwrap();
    let pol_liq = 20_000_000_000_000u128;
    let positions = vec![pos(lower, upper, bid_liq), pos(lower - 6_000, lower, pol_liq)];
    let mut sp = build_pool(&k, SynthKind::Floor, 60, sqrt, 9, 6, &positions).unwrap();
    sp.account.pool.hook_claims0 = vault;
    let mut fp = floor_pool_defaults(k.pool, k.mint0, k.mint1, [8; 32]);
    fp.ladder_seeded = 1;
    fp.floor_tick = floor_tick;
    fp.vault_eth = vault;
    fp.bid_lower = lower;
    fp.bid_width = upper - lower;
    fp.bid_liquidity.set(bid_liq);
    fp.bid_eth_deployed = carpenter_quote::amounts_for_liquidity(sqrt, tick, lower, upper, bid_liq as i128).unwrap().0;

    let set = sp.tick_array_set().unwrap();
    let q = quote_swap(&sp.account, &set, Some(&fp), &sell_to(100_260)).unwrap();
    let r = q.replace.expect("a re-place happened");
    let f = q.floor_after.unwrap();
    let dear = fp.floor_tick - fp.bid_width;

    assert_eq!(r.booking.nudge_spent, 0, "an EMPTY gap is crossed on zero input, whatever the budget");
    assert_eq!(r.to_tick, dear - 1, "the nudge still reaches the dear edge (v4: next − 1)");
    assert_eq!(f.bid_lower, dear, "FLUSH: the bid's lower edge IS the POL's upper edge — no hole");
    assert_eq!(f.bid_width, 120, "and the whole band came back");
    assert_eq!(q.pool_after.liquidity.get(), pol_liq, "the POL is active at the resting price");
    assert_eq!(f.floor_tick, fp.floor_tick, "the published floor never moves in a re-place");
}








mod carpenter_platform_fee {
    use super::*;
    use carpenter_quote::types::constants::{
        PROTOCOL_FEE_DEFAULT_PACKED, PROTOCOL_FEE_ON_CURRENCY0, PROTOCOL_FEE_ON_CURRENCY1, PROTOCOL_FEE_ON_INPUT,
    };
    use carpenter_quote::types::protocol_fee::{compose_swap_fee, gross_for_net, output_share};

    const L: u128 = 10_000_000_000_000;

    fn pool_with(fee: u32, packed: u32, code: u8) -> SynthPool {
        plain(fee, 60, 0, &[pos(-6_000, 6_000, L)]).with_protocol_fee(packed, code)
    }

///
///
///
    #[test]
    fn on_the_input_it_is_v4s_composed_split() {
        let amount = 1_000_000_000i128;
        for (zfo, code) in [
            (true, PROTOCOL_FEE_ON_INPUT),
            (false, PROTOCOL_FEE_ON_INPUT),
            (true, PROTOCOL_FEE_ON_CURRENCY0),
            (false, PROTOCOL_FEE_ON_CURRENCY1),
        ] {
            let sp = pool_with(3_000, PROTOCOL_FEE_DEFAULT_PACKED, code);
            let q = quote_swap(&sp.account, &sp.tick_array_set().unwrap(), None, &params(zfo, -amount)).unwrap();
            assert_eq!(q.fees.protocol, 500_000, "zfo {zfo} code {code}: floor(1e9 · 500 / 1e6)");
            assert_eq!(q.fees.protocol_currency, if zfo { 0 } else { 1 }, "the input currency");
            let composed = compose_swap_fee(500, 3_000);
            assert_eq!(composed, 3_499);
            let flat = plain(composed, 60, 0, &[pos(-6_000, 6_000, L)]);
            let f = quote_swap(&flat.account, &flat.tick_array_set().unwrap(), None, &params(zfo, -amount)).unwrap();
            assert_eq!(q.amount_out, f.amount_out, "the composed rate moves the price");
            assert_eq!(q.fees.lp + q.fees.protocol, f.fees.lp, "lp + protocol == the step fee");
            let (owed_in, owed_out) = if zfo {
                (q.pool_after.protocol_fees_owed0, q.pool_after.protocol_fees_owed1)
            } else {
                (q.pool_after.protocol_fees_owed1, q.pool_after.protocol_fees_owed0)
            };
            assert_eq!((owed_in, owed_out), (q.fees.protocol, 0));
            let growth = if zfo { q.pool_after.fee_growth_global0_x64.get() } else { q.pool_after.fee_growth_global1_x64.get() };
            assert_eq!(growth, ((q.fees.lp as u128) << 64) / L, "R-11 growth is the LP half only");
            let lp_counter = if zfo { q.pool_after.lp_fees0.get() } else { q.pool_after.lp_fees1.get() };
            assert_eq!(lp_counter, q.fees.lp as u128);
        }
    }

///
///
    #[test]
    fn on_the_output_it_comes_off_what_the_seller_receives() {
        for (zfo, code) in [(false, PROTOCOL_FEE_ON_CURRENCY0), (true, PROTOCOL_FEE_ON_CURRENCY1)] {
            let sp = pool_with(3_000, PROTOCOL_FEE_DEFAULT_PACKED, code);
            let q = quote_swap(&sp.account, &sp.tick_array_set().unwrap(), None, &params(zfo, -1_000_000_000)).unwrap();
            let bare = pool_with(3_000, 0, code);
            let b = quote_swap(&bare.account, &bare.tick_array_set().unwrap(), None, &params(zfo, -1_000_000_000)).unwrap();
            let want = output_share(b.amount_out, 500);
            assert!(want > 0);
            assert_eq!(q.fees.protocol, want, "zfo {zfo}: floor(gross_out · 500 / 1e6)");
            assert_eq!(q.amount_out, b.amount_out - want, "zfo {zfo}: net of it");
            assert_eq!(q.amount_in, b.amount_in);
            assert_eq!(q.fees.lp, b.fees.lp, "the LPs are paid exactly what a fee-free walk pays them");
            assert_eq!(q.sqrt_after, b.sqrt_after, "the fee never enters the curve");
            let quote_currency = if code == PROTOCOL_FEE_ON_CURRENCY0 { 0u8 } else { 1u8 };
            assert_eq!(q.fees.protocol_currency, quote_currency);
            let owed = [q.pool_after.protocol_fees_owed0, q.pool_after.protocol_fees_owed1];
            assert_eq!(owed[quote_currency as usize], want);
            assert_eq!(owed[1 - quote_currency as usize], 0);
        }
    }

///
///
    #[test]
    fn an_exact_out_through_the_output_fee_nets_exactly() {
        for named in [1i128, 1_999, 2_000, 1_000_000, 123_456_789] {
            let sp = pool_with(3_000, PROTOCOL_FEE_DEFAULT_PACKED, PROTOCOL_FEE_ON_CURRENCY0);
            let q = quote_swap(&sp.account, &sp.tick_array_set().unwrap(), None, &params(false, named)).unwrap();
            assert_eq!(q.amount_out as i128, named);
            let gross = gross_for_net(named as u64, 500).unwrap();
            assert_eq!(q.fees.protocol, gross - named as u64);
            let bare = pool_with(3_000, 0, PROTOCOL_FEE_ON_CURRENCY0);
            let b = quote_swap(&bare.account, &bare.tick_array_set().unwrap(), None, &params(false, gross as i128)).unwrap();
            assert_eq!(q.amount_in, b.amount_in, "named {named}");
        }
    }

///
///
    #[test]
    fn a_zero_rate_quotes_exactly_as_before() {
        for code in [PROTOCOL_FEE_ON_INPUT, PROTOCOL_FEE_ON_CURRENCY0, PROTOCOL_FEE_ON_CURRENCY1] {
            for zfo in [true, false] {
                let sp = pool_with(3_000, 0, code);
                let q = quote_swap(&sp.account, &sp.tick_array_set().unwrap(), None, &params(zfo, -1_000_000_000)).unwrap();
                let old = plain(3_000, 60, 0, &[pos(-6_000, 6_000, L)]);
                let o = quote_swap(&old.account, &old.tick_array_set().unwrap(), None, &params(zfo, -1_000_000_000)).unwrap();
                assert_eq!(q.fees.protocol, 0);
                assert_eq!((q.amount_in, q.amount_out, q.fees.lp), (o.amount_in, o.amount_out, o.fees.lp));
                let mut p = q.pool_after;
                p.protocol_fee_currency = 0;
                assert_eq!(bytemuck::bytes_of(&p), bytemuck::bytes_of(&o.pool_after), "code {code} zfo {zfo}");
            }
        }
    }

///
///
///
///
///
    #[test]
    fn a_floor_sell_pays_on_the_quote_and_the_nudge_pays_out_of_the_backing() {
        let vault = 1_000_000_000u64;
        let (sp, fp) = replace_fixture(100_020, 10_000_000_000, vault, true);
        let sp = sp.with_protocol_fee(PROTOCOL_FEE_DEFAULT_PACKED, PROTOCOL_FEE_ON_CURRENCY0);
        let set = sp.tick_array_set().unwrap();
        let q = quote_swap(&sp.account, &set, Some(&fp), &sell_to(100_260)).unwrap();
        let r = q.replace.expect("a re-place happened");
        let b = r.booking;
        assert!(b.nudge_spent > 0, "the nudge actually bought");


        assert_eq!(q.fees.protocol_currency, 0);
        let net_out = q.swap_delta.amount0 as u64;
        assert_eq!(q.fees.protocol, output_share(net_out + q.fees.protocol, 500), "floor(gross · 500 / 1e6)");
        assert!(q.fees.protocol > 0);


        let owed0 = q.pool_after.protocol_fees_owed0;
        assert_eq!(q.pool_after.protocol_fees_owed1, 0);
        let nudge_fee = owed0 - q.fees.protocol;
        let floor_share = (b.nudge_spent as u128 * 500 / 1_000_000) as u64;
        assert!(nudge_fee >= floor_share && nudge_fee <= floor_share + 3, "nudge fee {nudge_fee} vs 500 pips of {}", b.nudge_spent);
        assert!(nudge_fee > 0, "the vault's own buy paid");


        let idle = vault + q.fees.skim - q.fees.platform_cut;
        let f = q.floor_after.unwrap();
        assert_eq!(f.vault_eth, idle + b.eth_back - b.nudge_spent - b.bid_eth_deployed);
        assert!(f.inventory_cost_eth >= fp.inventory_cost_eth + b.nudge_spent, "the basis is kept, the nudge's spend inside it (2026-10-04)");
        let sell_fee = q.hook_delta.amount0 as u64;
        assert_eq!(q.pool_after.hook_claims0, vault + sell_fee + b.eth_back - b.nudge_spent - b.bid_eth_deployed);
        let owed = floor_math::FloorState::quote_liabilities(bytemuck::cast_ref(&f));
        assert!(q.pool_after.hook_claims0 as u128 >= owed, "I-3 holds");
    }
}
