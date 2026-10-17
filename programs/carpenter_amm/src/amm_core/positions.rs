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
use carpenter_clmm_math::{liquidity_math, tick_math};
use carpenter_types::hook::BalanceDelta;

use crate::amm_core::fees::{fee_growth_inside, fees_owed};
use crate::amm_core::ticks::{clear_tick, math_err, max_liquidity_per_tick, tick_offset, update_tick};
use crate::amm_core::vaults::create_pda;
use crate::constants::POSITION_SEED;
use crate::errors::AmmError;
use crate::state::{Pool, Position, Tick, TickArray};

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
#[allow(clippy::too_many_arguments)]
pub fn create_position<'info>(
    payer: &AccountInfo<'info>,
    info: &AccountInfo<'info>,
    pool: &Pubkey,
    owner: &Pubkey,
    position_id: &[u8; 32],
    salt: u64,
    bump: u8,
    tick_lower: i32,
    tick_upper: i32,
) -> Result<()> {
    let salt_bytes = salt.to_le_bytes();
    create_pda(
        payer,
        info,
        &crate::ID,
        Position::LEN,
        &[POSITION_SEED, pool.as_ref(), position_id, &salt_bytes, &[bump]],
    )?;
    let mut data = info.try_borrow_mut_data()?;
    data[..8].copy_from_slice(Position::DISCRIMINATOR);
    let p: &mut Position = bytemuck::from_bytes_mut(&mut data[8..Position::LEN]);
    p.version = 1;
    p.bump = bump;
    p.pool = *pool;
    p.owner = *owner;
    p.position_id = Pubkey::new_from_array(*position_id);
    p.salt = salt;
    p.tick_lower = tick_lower;
    p.tick_upper = tick_upper;
    Ok(())
}

///
///
pub enum TickArrays<'a> {
    Same(&'a mut TickArray),
    Split { lower: &'a mut TickArray, upper: &'a mut TickArray },
}

///
///
///
pub fn set_range(position: &mut Position, tick_lower: i32, tick_upper: i32) -> Result<()> {
    if position.tick_lower == tick_lower && position.tick_upper == tick_upper {
        return Ok(());
    }
    require!(position.liquidity.get() == 0, AmmError::PositionRangeLocked);
    position.tick_lower = tick_lower;
    position.tick_upper = tick_upper;
    position.fee_growth_inside0_last_x64.set(0);
    position.fee_growth_inside1_last_x64.set(0);
    Ok(())
}

///
fn ticks_mut<'b>(arrays: &'b mut TickArrays<'_>, li: usize, ui: usize) -> Result<(&'b mut Tick, &'b mut Tick)> {
    match arrays {
        TickArrays::Same(a) => {
            require!(li < ui, AmmError::TickInvalid);
            let (lo, hi) = a.ticks.split_at_mut(ui);
            Ok((&mut lo[li], &mut hi[0]))
        }
        TickArrays::Split { lower, upper } => Ok((&mut lower.ticks[li], &mut upper.ticks[ui])),
    }
}

fn step_count(count: &mut u16, flipped: bool, up: bool) -> Result<()> {
    if flipped {
        *count = if up { count.checked_add(1) } else { count.checked_sub(1) }.ok_or(AmmError::MathOverflow)?;
    }
    Ok(())
}

///
pub fn modify_position(
    pool: &mut Pool,
    position: &mut Position,
    mut arrays: TickArrays<'_>,
    liquidity_delta: i128,
) -> Result<(BalanceDelta, u64, u64)> {
    let spacing = pool.tick_spacing;
    let tick_lower = position.tick_lower;
    let tick_upper = position.tick_upper;
    let tick_current = pool.tick_current;
    let g0 = pool.fee_growth_global0_x64.get();
    let g1 = pool.fee_growth_global1_x64.get();

    let (lower_start, upper_start) = match &arrays {
        TickArrays::Same(a) => {
            require_keys_eq!(a.pool, position.pool, AmmError::SeedsMismatch);
            (a.start_index, a.start_index)
        }
        TickArrays::Split { lower, upper } => {
            require_keys_eq!(lower.pool, position.pool, AmmError::SeedsMismatch);
            require_keys_eq!(upper.pool, position.pool, AmmError::SeedsMismatch);
            require!(lower.start_index != upper.start_index, AmmError::DuplicateTickArray);
            (lower.start_index, upper.start_index)
        }
    };
    let li = tick_offset(lower_start, tick_lower, spacing)?;
    let ui = tick_offset(upper_start, tick_upper, spacing)?;


    let mut flipped_lower = false;
    let mut flipped_upper = false;
    let (fees0, fees1) = {
        let (lt, ut) = ticks_mut(&mut arrays, li, ui)?;
        if liquidity_delta != 0 {
            let max_liq = max_liquidity_per_tick(spacing);
            flipped_lower = update_tick(lt, tick_lower, tick_current, liquidity_delta, g0, g1, false, max_liq)?;
            flipped_upper = update_tick(ut, tick_upper, tick_current, liquidity_delta, g0, g1, true, max_liq)?;
        }
        let (inside0, inside1) = fee_growth_inside(lt, ut, tick_lower, tick_upper, tick_current, g0, g1);


        let liquidity = position.liquidity.get();
        if liquidity_delta == 0 {
            require!(liquidity > 0, AmmError::InsufficientLiquidity);
        }
        let fees0 = fees_owed(inside0, position.fee_growth_inside0_last_x64.get(), liquidity);
        let fees1 = fees_owed(inside1, position.fee_growth_inside1_last_x64.get(), liquidity);
        let liquidity_after = liquidity_math::add_delta(liquidity, liquidity_delta).map_err(math_err)?;
        position.liquidity.set(liquidity_after);
        position.fee_growth_inside0_last_x64.set(inside0);
        position.fee_growth_inside1_last_x64.set(inside1);


        if liquidity_delta < 0 {
            if flipped_lower {
                clear_tick(lt);
            }
            if flipped_upper {
                clear_tick(ut);
            }
        }
        (fees0, fees1)
    };


    let up = liquidity_delta > 0;
    match &mut arrays {
        TickArrays::Same(a) => {
            step_count(&mut a.initialized_tick_count, flipped_lower, up)?;
            step_count(&mut a.initialized_tick_count, flipped_upper, up)?;
        }
        TickArrays::Split { lower, upper } => {
            step_count(&mut lower.initialized_tick_count, flipped_lower, up)?;
            step_count(&mut upper.initialized_tick_count, flipped_upper, up)?;
        }
    }


    let mut amount0: u64 = 0;
    let mut amount1: u64 = 0;
    if liquidity_delta != 0 {
        let abs = liquidity_delta.unsigned_abs();
        let sa = tick_math::get_sqrt_price_at_tick(tick_lower).map_err(math_err)?;
        let sb = tick_math::get_sqrt_price_at_tick(tick_upper).map_err(math_err)?;
        if tick_current < tick_lower {
            amount0 = liquidity_math::get_delta_amount_0_unsigned(sa, sb, abs, up).map_err(math_err)?;
        } else if tick_current < tick_upper {
            let price = pool.sqrt_price_x64.get();
            amount0 = liquidity_math::get_delta_amount_0_unsigned(price, sb, abs, up).map_err(math_err)?;
            amount1 = liquidity_math::get_delta_amount_1_unsigned(sa, price, abs, up).map_err(math_err)?;
            let active = liquidity_math::add_delta(pool.liquidity.get(), liquidity_delta).map_err(math_err)?;
            pool.liquidity.set(active);
        } else {
            amount1 = liquidity_math::get_delta_amount_1_unsigned(sa, sb, abs, up).map_err(math_err)?;
        }
    }
    let principal = if up {
        BalanceDelta { amount0: -(amount0 as i128), amount1: -(amount1 as i128) }
    } else {
        BalanceDelta { amount0: amount0 as i128, amount1: amount1 as i128 }
    };
    Ok((principal, fees0, fees1))
}
