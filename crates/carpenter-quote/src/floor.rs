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

use carpenter_clmm_math::tick_math;
use floor_math::FeeTake;



use floor_types::constants::{DYNAMIC_FEE_FLAG, LP_FEE_PIPS, MAX_TICK, SALT_BID};
use floor_types::hook::{
    ActionResult, BalanceDelta, FeeSplit, HookAction, HookDeltaAssembly, HookRecordV1, HOOK_INTERFACE_VERSION,
    PHASE_AFTER_ACTIONS, PHASE_AFTER_SWAP, PHASE_BEFORE_SWAP,
};
use floor_types::layout::FloorPool;

use crate::error::{QuoteError, QuoteResult};
use crate::liquidity::amounts_for_liquidity;
use crate::mech;

///
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FeesTaken {
///
    pub skim: u64,
    pub creator: u64,
///
///
    pub holder: u64,
///
    pub platform_cut: u64,
///
///
///
///
    pub referral: u64,
///
///
///
    pub team: u64,
}

impl FeesTaken {
///
///
    pub fn total(&self) -> u64 {
        self.skim + self.creator + self.holder + self.team
    }

    fn add(&mut self, t: FeeTake) {
        self.skim += t.skim_fee;
        self.creator += t.creator_fee;
        self.holder += t.holder_fee;
        self.platform_cut += t.cut;
        self.referral += t.referral_fee;
        self.team += t.team_fee;
    }
}

///
#[derive(Clone, Copy, Debug)]
pub struct SwapCtx {
    pub pool_key: [u8; 32],
    pub zero_for_one: bool,
    pub amount_specified: i128,
    pub pre_tick: i32,
    pub supply1: u64,
///
    pub claims0: u64,
    pub claims1: u64,
///
    pub mint0: [u8; 32],
    pub mint1: [u8; 32],
    pub tick_spacing: u16,
///
    pub fee: u32,
///
///
    pub cushion_top: Option<i32>,
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
fn bind(fp: &FloorPool, ctx: &SwapCtx) -> QuoteResult<()> {
    if fp.amm_pool != ctx.pool_key {
        return Err(QuoteError::WrongPool);
    }
    if fp.pool_bound != 1 {
        return Err(QuoteError::NotBound);
    }
    if quote_is_token1(fp) {
        return Err(QuoteError::QuoteReverseOrientation);
    }

    let (quote_mint, base_mint) = if quote_is_token1(fp) { (ctx.mint1, ctx.mint0) } else { (ctx.mint0, ctx.mint1) };
    if fp.quote_mint != quote_mint || fp.base_mint != base_mint {
        return Err(QuoteError::WrongPool);
    }
    if fp.tick_spacing != i32::from(ctx.tick_spacing) {
        return Err(QuoteError::WrongPool);
    }






    if ctx.fee != DYNAMIC_FEE_FLAG {
        return Err(QuoteError::NotDynamicFee);
    }
    Ok(())
}

fn quote_is_token1(fp: &FloorPool) -> bool {
    fp.quote_is_token1 != 0
}

///
fn quote_specified(fp: &FloorPool, zfo: bool, amount_specified: i128) -> bool {
    (zfo != quote_is_token1(fp)) == (amount_specified < 0)
}

fn is_buy(fp: &FloorPool, zfo: bool) -> bool {
    zfo != quote_is_token1(fp)
}

fn add(a: u64, b: u64) -> QuoteResult<u64> {
    a.checked_add(b).ok_or(QuoteError::Overflow)
}

fn sub(a: u64, b: u64) -> QuoteResult<u64> {
    a.checked_sub(b).ok_or(QuoteError::Overflow)
}

///
///
///
///
///
///
fn vault_bought(fp: &mut FloorPool, tokens: u64, cost: u64) -> QuoteResult<()> {
    if fp.stair_steps == 0 {
        return Err(QuoteError::InvariantViolated);
    }
    if tokens == 0 {
        return Ok(());
    }
    fp.inventory_tokens = add(fp.inventory_tokens, tokens)?;
    fp.inventory_cost_eth = add(fp.inventory_cost_eth, cost)?;
    Ok(())
}

fn legs(fp: &FloorPool, d: BalanceDelta) -> (i128, i128) {
    if quote_is_token1(fp) {
        (d.amount1, d.amount0)
    } else {
        (d.amount0, d.amount1)
    }
}

fn credited(x: i128) -> QuoteResult<u64> {
    u64::try_from(x).map_err(|_| QuoteError::Overflow)
}

fn debited(x: i128) -> QuoteResult<u64> {
    u64::try_from(x.checked_neg().ok_or(QuoteError::Overflow)?).map_err(|_| QuoteError::Overflow)
}

///
///
///
///
fn take_fees(fp: &mut FloorPool, leg: u64, buy: bool, tick: i32, cushion_top: Option<i32>, taken: &mut FeesTaken) -> QuoteResult<FeeTake> {
    let t = mech::take_fees(fp, leg, buy, tick, cushion_top)?;
    taken.add(t);
    Ok(t)
}

///
///
///
///
fn split_of(t: FeeTake) -> FeeSplit {
    [t.skim_fee, t.creator_fee, t.holder_fee, t.cut, t.team_fee]
}

///
///
fn epilogue_i3(fp: &FloorPool, claims0: u64, claims1: u64, credit: BalanceDelta) -> QuoteResult<()> {
    let have0 = (claims0 as i128) + credit.amount0;
    let have1 = (claims1 as i128) + credit.amount1;






    let owe0 = floor_math::FloorState::quote_liabilities(bytemuck::cast_ref(fp)) as i128;



    let owe1 = floor_math::FloorState::base_liabilities(bytemuck::cast_ref(fp)) as i128;
    if have0 >= owe0 && have1 >= owe1 {
        Ok(())
    } else {
        Err(QuoteError::InvariantViolated)
    }
}

///
pub fn before_swap(fp: &mut FloorPool, ctx: &SwapCtx, taken: &mut FeesTaken) -> QuoteResult<HookRecordV1> {




    if fp.amm_pool == ctx.pool_key && fp.pool_bound == 1 && fp.ladder_seeded != 1 {
        return Err(QuoteError::NotSeeded);
    }
    bind(fp, ctx)?;
    if ctx.amount_specified > 0 {
        return Err(QuoteError::ExactOutNotSupported);
    }
    let mut rec = HookRecordV1::pass_through(HOOK_INTERFACE_VERSION, PHASE_BEFORE_SWAP, 0);
    rec.fee_override_pips = Some(LP_FEE_PIPS);
    if quote_specified(fp, ctx.zero_for_one, ctx.amount_specified) {
        let spec = u64::try_from(ctx.amount_specified.unsigned_abs()).map_err(|_| QuoteError::AmountTooLarge)?;



        fp.cumulative_volume_eth = fp.cumulative_volume_eth.saturating_add(spec);
        let buy = is_buy(fp, ctx.zero_for_one);
        let t = take_fees(fp, spec, buy, ctx.pre_tick, ctx.cushion_top, taken)?;
        rec.delta_specified = t.total as i128;
        rec.fees = split_of(t);
    }
    let credit = HookDeltaAssembly::assemble(ctx.amount_specified, ctx.zero_for_one, rec.delta_specified, rec.delta_unspecified);
    epilogue_i3(fp, ctx.claims0, ctx.claims1, credit)?;
    Ok(rec)
}

///
#[derive(Clone, Copy, Debug)]
pub struct AfterSwapIn {
    pub post_sqrt_price_x64: u128,
    pub post_tick: i32,
    pub swap_delta: BalanceDelta,
    pub before_delta_specified: i128,
    pub before_delta_unspecified: i128,
}

///
///
pub fn after_swap(fp: &mut FloorPool, ctx: &SwapCtx, a: &AfterSwapIn, taken: &mut FeesTaken) -> QuoteResult<HookRecordV1> {
    bind(fp, ctx)?;
    let zfo = ctx.zero_for_one;
    let mut rec = HookRecordV1::pass_through(HOOK_INTERFACE_VERSION, PHASE_AFTER_SWAP, 1);

    if !quote_specified(fp, zfo, ctx.amount_specified) {
        let quote_leg = if quote_is_token1(fp) { a.swap_delta.amount1 } else { a.swap_delta.amount0 };
        let leg = u64::try_from(quote_leg.unsigned_abs()).map_err(|_| QuoteError::Overflow)?;
        if leg > 0 {
            fp.cumulative_volume_eth = fp.cumulative_volume_eth.saturating_add(leg);
            let buy = is_buy(fp, zfo);

            let t = take_fees(fp, leg, buy, a.post_tick, None, taken)?;
            rec.delta_unspecified = t.total as i128;
            rec.fees = split_of(t);
        }
    }

    if !is_buy(fp, zfo) {
        if let Some(plan) = replace_actions(fp, a.post_sqrt_price_x64)? {
            rec.actions.extend_from_slice(&plan);
            rec.want_callback = true;
        }
    }

    let unspecified = a.before_delta_unspecified.checked_add(rec.delta_unspecified).ok_or(QuoteError::Overflow)?;
    let credit = HookDeltaAssembly::assemble(ctx.amount_specified, zfo, a.before_delta_specified, unspecified);
    epilogue_i3(fp, ctx.claims0, ctx.claims1, credit)?;
    Ok(rec)
}

///
///
///
pub fn bid_holds_tokens(fp: &FloorPool, post_sqrt: u128) -> QuoteResult<bool> {
    if fp.bid_liquidity.get() == 0 {
        return Ok(false);
    }
    let (_, tokens_held) = mech::bid_amounts(post_sqrt, fp.bid_lower, mech::bid_width(fp), fp.bid_liquidity.get(), fp.bid_eth_deployed)?;
    Ok(tokens_held > 0)
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
pub fn replace_actions(fp: &FloorPool, post_sqrt: u128) -> QuoteResult<Option<Vec<HookAction>>> {
    if !bid_holds_tokens(fp, post_sqrt)? {
        return Ok(None);
    }
    let width = mech::bid_width(fp);
    let lower = fp.bid_lower;
    let upper = lower.checked_add(width).ok_or(QuoteError::Overflow)?;
    let liq = i128::try_from(fp.bid_liquidity.get()).map_err(|_| QuoteError::Overflow)?;
    let mut plan = vec![HookAction::ModifyPosition {
        salt: SALT_BID,
        tick_lower: lower,
        tick_upper: upper,
        liquidity_delta: -liq,
    }];

    let (bid_eth, _) = mech::bid_amounts(post_sqrt, lower, width, fp.bid_liquidity.get(), fp.bid_eth_deployed)?;




    let budget = add(fp.vault_eth, bid_eth)?.min(fp.min_bid);
    let edge = mech::dear_edge(fp)?;
    let limit = tick_math::get_sqrt_price_at_tick(edge)?;

    let at_or_under = if quote_is_token1(fp) { post_sqrt >= limit } else { post_sqrt <= limit };
    if budget > 0 && !at_or_under {
        plan.push(HookAction::Swap {
            zero_for_one: !quote_is_token1(fp),
            amount_specified: -(budget as i128),
            sqrt_price_limit_x64: limit,
        });
    }
    Ok(Some(plan))
}

///
///
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReplaceBooking {
///
    pub absorbed: u64,
///
    pub eth_back: u64,
///
    pub nudge_spent: u64,
    pub nudge_got: u64,
///
    pub from_lower: i32,
///
    pub redeployed: bool,
    pub to_lower: i32,
    pub to_upper: i32,
    pub bid_eth_deployed: u64,
    pub bid_liquidity: u128,
///
///
///
///
    pub fees_unknown: bool,
}

///
///
///
///
///
///
pub fn after_actions(
    fp: &mut FloorPool,
    ctx: &SwapCtx,
    round: u8,
    results: &[ActionResult],
    results_carry_fees: bool,
    claims0: u64,
    claims1: u64,
) -> QuoteResult<(HookRecordV1, ReplaceBooking)> {
    bind(fp, ctx)?;
    let mut rec = HookRecordV1::pass_through(HOOK_INTERFACE_VERSION, PHASE_AFTER_ACTIONS, round);
    let mut booking = ReplaceBooking { from_lower: fp.bid_lower, ..ReplaceBooking::default() };
    if results.is_empty() {
        epilogue_i3(fp, claims0, claims1, BalanceDelta::default())?;
        return Ok((rec, booking));
    }


    let close = &results[0];
    if close.kind != HookAction::KIND_MODIFY_POSITION {
        return Err(QuoteError::HookFieldNotAllowed);
    }
    let (q, b) = legs(fp, close.delta);
    let quote_back = credited(q)?;
    let tok_back = credited(b)?;
    let (fees0, fees1) = if quote_is_token1(fp) { (close.fees1, close.fees0) } else { (close.fees0, close.fees1) };
    let principal0 = sub(quote_back, fees0)?;
    let kept_fees = mech::intake(fp, fees0)?;
    fp.vault_eth = add(fp.vault_eth, add(principal0, kept_fees)?)?;



    let cost = fp.bid_eth_deployed.saturating_sub(principal0);
    vault_bought(fp, tok_back, cost)?;
    let _ = fees1;
    fp.bid_liquidity.set(0);
    fp.bid_eth_deployed = 0;
    booking.absorbed = tok_back;
    booking.eth_back = quote_back;
    booking.fees_unknown = !results_carry_fees;


    let (mut sqrt, mut tick) = (close.post_sqrt_price_x64, close.post_tick);
    if let Some(nudge) = results.get(1) {
        if nudge.kind != HookAction::KIND_SWAP {
            return Err(QuoteError::HookFieldNotAllowed);
        }
        let (q, b) = legs(fp, nudge.delta);
        let spent = if q < 0 { debited(q)? } else { 0 };
        let got = if b > 0 { credited(b)? } else { 0 };



        if spent > 0 && got > 0 {
            fp.vault_eth = sub(fp.vault_eth, spent)?;
            vault_bought(fp, got, spent)?;
            booking.nudge_spent = spent;
            booking.nudge_got = got;
        }
        sqrt = nudge.post_sqrt_price_x64;
        tick = nudge.post_tick;
    }


    if let Some((lower, upper, liq, owed)) = deploy_bid(fp, sqrt, tick)? {
        fp.vault_eth = sub(fp.vault_eth, owed)?;
        fp.bid_eth_deployed = owed;
        fp.bid_liquidity.set(liq);
        fp.bid_lower = lower;
        fp.bid_width = upper - lower;
        rec.actions.push(HookAction::ModifyPosition {
            salt: SALT_BID,
            tick_lower: lower,
            tick_upper: upper,
            liquidity_delta: i128::try_from(liq).map_err(|_| QuoteError::Overflow)?,
        });
        booking.redeployed = true;
        booking.to_lower = lower;
        booking.to_upper = upper;
        booking.bid_eth_deployed = owed;
        booking.bid_liquidity = liq;
    }
    fp.needs_ratchet = 1;



    epilogue_i3(fp, claims0, claims1, BalanceDelta::default())?;
    Ok((rec, booking))
}

///
///
///
///
///
///
///
pub fn deploy_bid(fp: &FloorPool, sqrt: u128, current: i32) -> QuoteResult<Option<(i32, i32, u128, u64)>> {
    if fp.vault_eth < fp.min_bid {
        return Ok(None);
    }
    let spacing = fp.tick_spacing;
    let mut upper = fp.floor_tick;
    let mut lower = upper.checked_sub(mech::band_ticks(fp)?).ok_or(QuoteError::Overflow)?;
    let min_lower = mech::ceil_to_spacing(current.checked_add(1).ok_or(QuoteError::Overflow)?, spacing)?;
    if lower < min_lower {
        lower = min_lower;
    }
    if lower >= upper {
        upper = lower.checked_add(spacing).ok_or(QuoteError::Overflow)?;
    }
    if upper > MAX_TICK {
        return Ok(None);
    }
    let liq = mech::position_liquidity(sqrt, lower, upper, fp.vault_eth, 0)?;
    if liq == 0 {
        return Ok(None);
    }

    let (a0, a1) = amounts_for_liquidity(sqrt, current, lower, upper, i128::try_from(liq).map_err(|_| QuoteError::Overflow)?)?;
    let owed = if quote_is_token1(fp) { a1 } else { a0 };
    Ok(Some((lower, upper, liq, owed)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck::Zeroable;

    fn pool(vault: u64, floor_tick: i32) -> FloorPool {
        let mut fp = FloorPool::zeroed();
        fp.tick_spacing = 60;
        fp.floor_tick = floor_tick;
        fp.vault_eth = vault;
        fp.min_bid = 50_000_000;


        fp.stair_steps = 1;
        fp
    }



    #[test]
    fn deploy_bid_geometry() {
        let fp = pool(1_000_000_000, 100_320);
        let at = |t: i32| tick_math::get_sqrt_price_at_tick(t).unwrap();

        let (lo, up, liq, owed) = deploy_bid(&fp, at(100_020), 100_020).unwrap().unwrap();
        assert_eq!((lo, up), (100_200, 100_320));
        assert!(liq > 0 && owed <= 1_000_000_000 && owed > 999_000_000, "{owed}");

        assert_eq!(deploy_bid(&fp, at(100_200), 100_199).unwrap().unwrap().0, 100_200);

        let (lo, up, _, _) = deploy_bid(&fp, at(100_200), 100_200).unwrap().unwrap();
        assert_eq!((lo, up), (100_260, 100_320));

        let (lo, up, _, _) = deploy_bid(&fp, at(100_210), 100_210).unwrap().unwrap();
        assert_eq!((lo, up), (100_260, 100_320));


        let (lo, up, _, _) = deploy_bid(&fp, at(100_270), 100_270).unwrap().unwrap();
        assert_eq!((lo, up), (100_320, 100_380));
    }






    #[test]
    fn fees_unknown_is_derived_from_the_results_provenance() {
        let at = |t: i32| tick_math::get_sqrt_price_at_tick(t).unwrap();
        let mut fp = pool(1_000_000_000, 100_320);
        fp.amm_pool = [7; 32];
        fp.pool_bound = 1;
        fp.quote_mint = [1; 32];
        fp.base_mint = [2; 32];
        fp.platform_bps = 250;
        fp.bid_lower = 100_200;
        fp.bid_width = 120;
        fp.bid_liquidity.set(1_000_000_000_000);
        fp.bid_eth_deployed = 500_000_000;
        let ctx = SwapCtx {
            pool_key: [7; 32],
            zero_for_one: false,
            amount_specified: -1,
            pre_tick: 100_020,
            supply1: 0,
            claims0: 0,
            claims1: 0,
            mint0: [1; 32],
            mint1: [2; 32],
            tick_spacing: 60,


            fee: DYNAMIC_FEE_FLAG,
            cushion_top: None,
        };
        let close = ActionResult {
            index: 0,
            kind: HookAction::KIND_MODIFY_POSITION,
            delta: BalanceDelta { amount0: 400_000_000 + 1_234, amount1: 5_000 },
            fees0: 1_234,
            fees1: 0,
            post_sqrt_price_x64: at(100_020),
            post_tick: 100_020,
        };
        let claims = u64::MAX / 4;
        let mut a = fp;
        let (_, quoter) = after_actions(&mut a, &ctx, 2, &[close], false, claims, claims).unwrap();
        let mut b = fp;
        let (_, chain) = after_actions(&mut b, &ctx, 2, &[close], true, claims, claims).unwrap();
        assert!(quoter.fees_unknown, "modify.rs produced the result: the fees are not the position's");
        assert!(!chain.fees_unknown, "the chain's result carries the position's fees");

        assert_eq!(ReplaceBooking { fees_unknown: true, ..chain }, quoter);
        assert_eq!(bytemuck::bytes_of(&a), bytemuck::bytes_of(&b));
        let kept = 1_234 - 1_234 * 250 / 10_000;
        assert_eq!(quoter.eth_back, 400_000_000 + 1_234);
        assert_eq!(a.platform_owed, 1_234 - kept);

        assert_eq!(a.vault_eth + a.bid_eth_deployed, 1_000_000_000 + 400_000_000 + kept);
        assert!(quoter.redeployed);
    }

    #[test]
    fn deploy_bid_last_resort_and_refusals() {
        let fp = pool(1_000_000_000, 100_320);
        let at = |t: i32| tick_math::get_sqrt_price_at_tick(t).unwrap();

        let (lo, up, liq, _) = deploy_bid(&fp, at(100_500), 100_500).unwrap().unwrap();
        assert_eq!((lo, up), (100_560, 100_620));
        assert!(liq > 0);

        let (lo, up, _, _) = deploy_bid(&fp, at(100_320), 100_320).unwrap().unwrap();
        assert_eq!((lo, up), (100_380, 100_440));

        assert_eq!(deploy_bid(&pool(49_999_999, 100_320), at(100_020), 100_020).unwrap(), None);



        let top = pool(1_000_000_000, 443_520);
        let (lo, up, _, _) = deploy_bid(&top, at(443_520), 443_520).unwrap().unwrap();
        assert_eq!((lo, up), (443_520, 443_580));
    }

///
///
///
///
    #[test]
    fn i3_counts_the_held_tokens_and_never_the_staircases_standing_ones() {
        let mut fp = pool(0, 100_320);
        fp.inventory_tokens = 10;
        fp.ladder_tokens_remaining = 450;
        fp.wall_held_tokens = 150;
        assert_eq!(epilogue_i3(&fp, 0, 160, BalanceDelta::default()), Ok(()));
        assert_eq!(epilogue_i3(&fp, 0, 159, BalanceDelta::default()), Err(QuoteError::InvariantViolated), "held is owed");
        fp.stair_steps = 0;
        assert_eq!(epilogue_i3(&fp, 0, 609, BalanceDelta::default()), Err(QuoteError::InvariantViolated), "a non-staircase state counts its ladder idle");
        assert_eq!(epilogue_i3(&fp, 0, 610, BalanceDelta::default()), Ok(()));
    }
}
