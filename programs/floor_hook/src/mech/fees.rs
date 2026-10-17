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
//!
//!
//!

use anchor_lang::prelude::*;
use floor_math::FeeTake;
use floor_types::hook::FeeSplit;

use crate::constants::BPS;
use crate::errors::HookError;
use crate::events::{CreatorFee, PlatformCut, ReferralFunded, Skimmed};
use crate::instructions::CallbackEvent;
use crate::state::FloorPool;

///
///
pub fn bps_of(amount: u64, bps: u64) -> Result<u64> {
    let v = (amount as u128)
        .checked_mul(bps as u128)
        .ok_or(HookError::MathOverflow)?
        / (BPS as u128);
    u64::try_from(v).map_err(|_| error!(HookError::MathOverflow))
}

///
///
fn map<T>(r: floor_math::FloorMathResult<T>) -> Result<T> {
    r.map_err(|e| {
        msg!("floor-math: {}", e);
        error!(HookError::MathOverflow)
    })
}

///
///
///
///
fn state_mut(fp: &mut FloorPool) -> &mut floor_math::FloorState {
    bytemuck::cast_mut(fp)
}

///
///
fn state(fp: &FloorPool) -> &floor_math::FloorState {
    bytemuck::cast_ref(fp)
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
pub fn split_of(t: &FeeTake) -> FeeSplit {
    [t.skim_fee, t.creator_fee, t.holder_fee, t.cut, t.team_fee]
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
pub fn take_fees(
    fp: &mut FloorPool,
    leg: u64,
    is_buy: bool,
    tick: i32,
    cushion_top: Option<i32>,
    events: &mut Vec<CallbackEvent>,
) -> Result<FeeTake> {
    let take = map(floor_math::take_fees(state_mut(fp), leg, is_buy, tick, cushion_top))?;
    if take.cut > 0 {
        events.push(CallbackEvent::PlatformCut(PlatformCut { pool: fp.amm_pool, amount: take.cut }));
    }
    if take.skim_fee > 0 {
        events.push(CallbackEvent::Skimmed(Skimmed {
            pool: fp.amm_pool,
            is_buy,
            bps: take.skim_bps as u16,
            fee: take.skim_fee,
            kept: take.kept,
        }));
    }








    if take.referral_fee > 0 {



        events.push(CallbackEvent::ReferralFunded(ReferralFunded {
            pool: fp.amm_pool,
            distributor: fp.referrer,
            asset: fp.quote_mint,
            amount: take.referral_fee,
            gross_creator_fee: take.creator_fee,
        }));
    }
    if take.creator_fee > 0 {
        let net = take.creator_fee - take.referral_fee;
        events.push(CallbackEvent::CreatorFee(CreatorFee { pool: fp.amm_pool, is_buy, amount: net }));
    }
    Ok(take)
}

///
///
pub fn cushion_top_of(floor_pool: &AccountInfo) -> Result<Option<i32>> {
    if !floor_math::fees::cushion_gap_skim_on() {
        return Ok(None);
    }
    let data = floor_pool.try_borrow_data()?;
    Ok(floor_types::twap::TwapOracle::cushion_top_of_account(&data))
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
pub fn assert_claims_cover(fp: &FloorPool, claims0: u64, claims1: u64, credit0: i128, credit1: i128) -> Result<()> {
    let have0 = (claims0 as i128).checked_add(credit0).ok_or(HookError::MathOverflow)?;
    let have1 = (claims1 as i128).checked_add(credit1).ok_or(HookError::MathOverflow)?;
    let (have_quote, have_base) = if fp.quote_is_token1 != 0 { (have1, have0) } else { (have0, have1) };







    let quote_liab = state(fp).quote_liabilities() as i128;


    let base_liab = state(fp).base_liabilities() as i128;

    if have_quote < quote_liab {
        msg!("I-3 quote: claims {} < liabilities {}", have_quote, quote_liab);
        return err!(HookError::InvariantViolated);
    }
    if have_base < base_liab {
        msg!("I-3 base: claims {} < liabilities {}", have_base, base_liab);
        return err!(HookError::InvariantViolated);
    }
    Ok(())
}
