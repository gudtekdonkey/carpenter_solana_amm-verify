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

use floor_math::{FeeTake, FloorMathError, FloorState};
use floor_types::constants::SKIM_RAMP_TICKS;
use floor_types::holders::{HolderBook, HolderRecord};
use floor_types::layout::FloorPool;

use crate::error::{QuoteError, QuoteResult};

const _: () = assert!(core::mem::size_of::<FloorPool>() == floor_math::FLOOR_STATE_SIZE);

///
///
fn mirror(state: &FloorPool) -> &FloorState {
    bytemuck::from_bytes(bytemuck::bytes_of(state))
}

fn mirror_mut(state: &mut FloorPool) -> &mut FloorState {
    bytemuck::from_bytes_mut(bytemuck::bytes_of_mut(state))
}

///
///
///
impl From<FloorMathError> for QuoteError {
    fn from(e: FloorMathError) -> Self {
        match e {
            FloorMathError::Math(m) => QuoteError::from(m),
            FloorMathError::ZeroDenominator
            | FloorMathError::Overflow
            | FloorMathError::SupplyUnderflow
            | FloorMathError::InvalidTickSpacing
            | FloorMathError::InvalidRange
            | FloorMathError::TranchePriceOutOfRange
            | FloorMathError::UnknownMode


            | FloorMathError::ReferralShareOutOfRange



            | FloorMathError::QuoteDecimalsOutOfRange


            | FloorMathError::BadSplit => QuoteError::Overflow,
        }
    }
}

///
///
pub fn skim_bps(distance: i32, is_buy: bool, buy_far: u16, sell_far: u16, buy_at_floor: u16, sell_at_floor: u16) -> u64 {
    floor_math::skim_bps(distance, SKIM_RAMP_TICKS, is_buy, buy_far, sell_far, buy_at_floor, sell_at_floor)
}

///
pub fn band_ticks(state: &FloorPool) -> QuoteResult<i32> {
    Ok(floor_math::band_ticks(state.tick_spacing)?)
}

///
pub fn dear_edge(state: &FloorPool) -> QuoteResult<i32> {
    Ok(floor_math::dear_edge(state.floor_tick, state.tick_spacing)?)
}

///
pub fn bid_width(state: &FloorPool) -> i32 {
    floor_math::bid_width_or_spacing(state.bid_width, state.tick_spacing)
}

///
///
pub fn position_liquidity(sqrt_price_x64: u128, lower: i32, upper: i32, quote: u64, base: u64) -> QuoteResult<u128> {
    Ok(floor_math::position_liquidity(sqrt_price_x64, lower, upper, quote, base)?)
}

///
///
///
pub fn take_fees(state: &mut FloorPool, leg: u64, is_buy: bool, tick: i32, cushion_top: Option<i32>) -> QuoteResult<FeeTake> {
    Ok(floor_math::take_fees(mirror_mut(state), leg, is_buy, tick, cushion_top)?)
}

///
///
///
pub fn intake(state: &mut FloorPool, x: u64) -> QuoteResult<u64> {
    Ok(floor_math::intake(mirror_mut(state), x)?)
}

///
///
pub fn bid_amounts(sqrt_price_x64: u128, bid_lower: i32, width: i32, bid_liquidity: u128, deployed: u64) -> QuoteResult<(u64, u64)> {
    Ok(floor_math::bid_amounts(sqrt_price_x64, bid_lower, width, bid_liquidity, deployed)?)
}

///
///
///
///
pub fn solve_floor_tick_with(quote: u64, extra: u64, state: &FloorPool, supply: u64) -> QuoteResult<i32> {
    Ok(floor_math::solve_floor_tick_with(u128::from(quote), extra, mirror(state), supply)?)
}

///
pub fn sale_tokens_for(lamports: u64, price_x64: u128, remaining: u64) -> QuoteResult<(u64, u64)> {
    Ok(floor_math::sale_tokens_for(lamports, price_x64, remaining)?)
}

///
pub fn sale_buy_with_team(gross: u64, price_x64: u128, remaining: u64, team_bps: u16) -> QuoteResult<(u64, u64, u64)> {
    Ok(floor_math::sale_buy_with_team(gross, price_x64, remaining, team_bps)?)
}

///
pub fn sell_back_with_team(payout: u64, team_bps: u16) -> QuoteResult<(u64, u64)> {
    Ok(floor_math::sell_back_with_team(payout, team_bps)?)
}

///
///
pub fn sell_back_payout_at(tokens: u64, price_x64: u128) -> QuoteResult<u64> {
    Ok(floor_math::sell_back_payout_at(tokens, price_x64)?)
}

///
///
pub fn ceil_to_spacing(tick: i32, spacing: i32) -> QuoteResult<i32> {
    Ok(floor_math::ceil_to_spacing(tick, spacing)?)
}

///
pub fn floor_to_spacing(tick: i32, spacing: i32) -> QuoteResult<i32> {
    Ok(floor_math::floor_to_spacing(tick, spacing)?)
}

///
///
///
///
pub fn holder_claimable(book: &HolderBook, rec: &HolderRecord, pool: &FloorPool, live_balance: u64) -> u64 {
    use floor_math::holders as h;
    let b = h::Book {
        acc: book.acc.get(),
        eligible: book.eligible,
        indexed: book.indexed,
        unallocated: book.unallocated,
        pending: book.pending,
        last_slot: book.last_slot,
    };
    let r = h::Record { shares: rec.shares, snapshot: rec.snapshot.get(), owed: rec.owed };
    h::claimable(b, r, h::lifetime(pool.holder_owed, pool.holder_paid_eth), live_balance)
}

///
///
pub fn referral_tier_dues(pool: &FloorPool) -> [u64; 3] {
    floor_math::referral_tier_dues(mirror(pool))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck::Zeroable;

///
///
///
    #[test]
    fn mirror_is_the_same_bytes() {
        let mut p = FloorPool::zeroed();
        p.version = 1;
        p.bump = 2;
        p.pool_bound = 3;
        p.quote_is_token1 = 4;
        p.ladder_seeded = 5;
        p.needs_ratchet = 6;
        p.mode = 7;
        p.quote_suspended = 8;
        p.tick_spacing = 60;
        p.floor_tick = -1_200;
        p.amm_pool = [0xA1; 32];
        p.pool_initializer = [0xA2; 32];
        p.base_mint = [0xA3; 32];
        p.quote_mint = [0xA4; 32];
        p.pad9 = [0xA5; 32];
        p.pad10 = [0xA6; 32];
        p.skim_buy_far_bps = 201;
        p.skim_sell_far_bps = 202;
        p.creator_buy_bps = 203;
        p.creator_sell_bps = 204;
        p.holder_buy_bps = 205;
        p.holder_sell_bps = 206;
        p.wall_ticks = 207;
        p.platform_bps = 208;
        p.min_bid = 301;
        p.min_sellable = 302;
        p.owner_owed = 303;
        p.owner_paid = 304;
        p.vault_eth = 305;
        p.bid_eth_deployed = 306;
        p.unpaid_skim = 307;
        p.cumulative_volume_eth = 308;
        p.inventory_tokens = 309;
        p.inventory_cost_eth = 310;
        p.ask_tokens_deployed = 311;
        p.ladder_tokens_remaining = 312;
        p.bid_lower = -401;
        p.ask_upper = 402;
        p.ask_width = 403;
        p.seed_lower = -404;
        p.seed_upper = 405;
        p.ladder_lower = -406;
        p.ladder_upper = 407;
        p.wall_split_tick = -408;
        p.bid_liquidity.set(501 | (1 << 100));
        p.ask_liquidity.set(502 | (2 << 100));
        p.seed_liq.set(503 | (3 << 100));
        p.ladder_liq.set(504 | (4 << 100));
        p.creator_fees_owed = 601;
        p.platform_owed = 602;
        p.holder_owed = 603;


        p.wall_tail_liq.set(604 | (5 << 100));
        p.wall_held_tokens = 606;
        p.seed_fees_swept_eth = 607;
        p.seed_fees_swept_tokens = 608;
        p.ladder_proceeds_eth = 609;
        p.platform_paid_eth = 610;
        p.platform_swept = 611;
        p.creator_fees_paid = 612;
        p.holder_paid_eth = 613;
        p.last_ratchet_slot = 614;







        p.bid_width = 615;
        p.skim_buy_at_floor_bps = 616;
        p.skim_sell_at_floor_bps = 617;
        p.pad6 = [0xAB; 32];
        p.max_bounty = 618;
        p.bounty_bps = 619;

        p.wall_band_ticks = 0xCD01;
        p.wall_tail_bps = 0xCD02;
        p.wall_list_bps = 0xCD03;











        p.referral_owed = 620;
        p.referral_fees_paid = 621;
        p.trader_ref_owed = 0xAF01;
        p.trader_ref_booked = 0xAF02;
        p.pad7 = [0xAF; 16];
        p.referral_share_bps = 622;
        p.stair_steps = 0xB1;
        p.stair_ready = 0xB2;
        p.stair_growth_bps = 0xB3B4;
        p.stair_kind = 0xBB;
        p.stair_owner = 0xBA;




        p.referrer = [0x5E; 32];
        p.pad8 = 0xC8;
        p.holders_hooked = 2;
        p.pad4 = [0xC4; 6];



        p.referrer_t2 = [0x72; 32];
        p.referrer_t3 = [0x73; 32];
        p.referral_paid_t2 = 623;
        p.referral_paid_t3 = 624;



        p.team_owed = 625;
        p.team_fees_paid = 626;
        p.team_fee_bps = 627;


        p.pad5 = [0xEE; 6];

        let m = mirror(&p);
        assert_eq!(m.version, 1);
        assert_eq!(m.bump, 2);
        assert_eq!(m.pool_bound, 3);
        assert_eq!(m.quote_is_token1, 4);
        assert_eq!(m.ladder_seeded, 5);
        assert_eq!(m.needs_ratchet, 6);
        assert_eq!(m.mode, 7);
        assert_eq!(m.quote_suspended, 8);
        assert_eq!(m.tick_spacing, 60);
        assert_eq!(m.floor_tick, -1_200);
        assert_eq!(m.amm_pool, [0xA1; 32]);
        assert_eq!(m.pool_initializer, [0xA2; 32]);
        assert_eq!(m.base_mint, [0xA3; 32]);
        assert_eq!(m.quote_mint, [0xA4; 32]);
        assert_eq!(m.pad9, [0xA5; 32]);
        assert_eq!(m.pad10, [0xA6; 32]);
        assert_eq!(m.skim_buy_far_bps, 201);
        assert_eq!(m.skim_sell_far_bps, 202);
        assert_eq!(m.creator_buy_bps, 203);
        assert_eq!(m.creator_sell_bps, 204);
        assert_eq!(m.holder_buy_bps, 205);
        assert_eq!(m.holder_sell_bps, 206);
        assert_eq!(m.wall_ticks, 207);
        assert_eq!(m.platform_bps, 208);
        assert_eq!(m.min_bid, 301);
        assert_eq!(m.min_sellable, 302);
        assert_eq!(m.owner_owed, 303);
        assert_eq!(m.owner_paid, 304);
        assert_eq!(m.vault_eth, 305);
        assert_eq!(m.bid_eth_deployed, 306);
        assert_eq!(m.unpaid_skim, 307);
        assert_eq!(m.cumulative_volume_eth, 308);
        assert_eq!(m.inventory_tokens, 309);
        assert_eq!(m.inventory_cost_eth, 310);
        assert_eq!(m.ask_tokens_deployed, 311);
        assert_eq!(m.ladder_tokens_remaining, 312);
        assert_eq!(m.bid_lower, -401);
        assert_eq!(m.ask_upper, 402);
        assert_eq!(m.ask_width, 403);
        assert_eq!(m.seed_lower, -404);
        assert_eq!(m.seed_upper, 405);
        assert_eq!(m.ladder_lower, -406);
        assert_eq!(m.ladder_upper, 407);
        assert_eq!(m.wall_split_tick, -408);
        assert_eq!(m.bid_liquidity.get(), 501 | (1 << 100));
        assert_eq!(m.ask_liquidity.get(), 502 | (2 << 100));
        assert_eq!(m.seed_liq.get(), 503 | (3 << 100));
        assert_eq!(m.ladder_liq.get(), 504 | (4 << 100));
        assert_eq!(m.creator_fees_owed, 601);
        assert_eq!(m.platform_owed, 602);
        assert_eq!(m.holder_owed, 603);
        assert_eq!(m.wall_tail_liq.get(), 604 | (5 << 100));
        assert_eq!(m.wall_held_tokens, 606);
        assert_eq!(m.seed_fees_swept_eth, 607);
        assert_eq!(m.seed_fees_swept_tokens, 608);
        assert_eq!(m.ladder_proceeds_eth, 609);
        assert_eq!(m.platform_paid_eth, 610);
        assert_eq!(m.platform_swept, 611);
        assert_eq!(m.creator_fees_paid, 612);
        assert_eq!(m.holder_paid_eth, 613);
        assert_eq!(m.last_ratchet_slot, 614);
        assert_eq!(m.bid_width, 615);
        assert_eq!(m.skim_buy_at_floor_bps, 616);
        assert_eq!(m.skim_sell_at_floor_bps, 617);
        assert_eq!(m.pad6, [0xAB; 32]);
        assert_eq!(m.max_bounty, 618);
        assert_eq!(m.bounty_bps, 619);
        assert_eq!((m.wall_band_ticks, m.wall_tail_bps, m.wall_list_bps), (0xCD01, 0xCD02, 0xCD03));
        assert_eq!(m.referral_owed, 620, "referral_owed @584 — a swap with referral_fees_paid lands here");
        assert_eq!(m.referral_fees_paid, 621, "referral_fees_paid @592 — the cumulative ACCRUED meter");
        assert_eq!((m.trader_ref_owed, m.trader_ref_booked), (0xAF01, 0xAF02), "trader referrals @600 / @608");
        assert_eq!(m.pad7, [0xAF; 16]);
        assert_eq!(m.referral_share_bps, 622);
        assert_eq!((m.stair_steps, m.stair_ready, m.stair_growth_bps), (0xB1, 0xB2, 0xB3B4));
        assert_eq!((m.stair_kind, m.stair_owner), (0xBB, 0xBA));
        assert_eq!(m.referrer, [0x5E; 32], "referrer @640 — never the retired distributor @600");
        assert_eq!(m.pad8, 0xC8);
        assert_eq!(m.holders_hooked, 2);
        assert_eq!(m.pad4, [0xC4; 6]);
        assert_eq!(m.referrer_t2, [0x72; 32], "referrer_t2 @680");
        assert_eq!(m.referrer_t3, [0x73; 32], "referrer_t3 @712");
        assert_eq!(m.referral_paid_t2, 623, "referral_paid_t2 @744");
        assert_eq!(m.referral_paid_t3, 624, "referral_paid_t3 @752");
        assert_eq!((m.team_owed, m.team_fees_paid, m.team_fee_bps), (625, 626, 627), "team @760 / @768 / @776");
        assert_eq!(m.pad5, [0xEE; 6], "the tail (no `reserved` left) mirrors");




        assert_eq!(core::mem::size_of::<FloorPool>(), core::mem::size_of::<FloorState>());
    }


    #[test]
    fn band_helpers_read_the_pool() {
        let mut p = FloorPool::zeroed();
        p.tick_spacing = 60;
        p.floor_tick = 1_200;
        assert_eq!(band_ticks(&p).unwrap(), 120);
        assert_eq!(dear_edge(&p).unwrap(), 1_080);
        assert_eq!(bid_width(&p), 60, "a pre-band bid reads as one spacing");
        p.bid_width = 120;
        assert_eq!(bid_width(&p), 120);
    }
}
