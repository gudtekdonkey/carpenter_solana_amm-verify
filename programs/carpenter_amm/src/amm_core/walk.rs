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

use core::cell::{Ref, RefMut};

use anchor_lang::prelude::*;
use carpenter_clmm_math::bitmap as cbitmap;
use carpenter_clmm_math::liquidity_math::add_delta;
use carpenter_clmm_math::swap_math::compute_swap;
use carpenter_clmm_math::tick_math::{get_sqrt_price_at_tick, get_tick_at_sqrt_price};
use carpenter_clmm_math::MathError;
use carpenter_types::constants::{MAX_LP_FEE, MAX_SQRT_PRICE_X64, MAX_SWAP_FEE, MAX_TICK, MIN_SQRT_PRICE_X64, MIN_TICK};
use carpenter_types::hook::{BalanceDelta, PoolSnapshot};
use carpenter_types::protocol_fee::{gross_for_net, input_share, output_share, walk_fee};
use carpenter_types::layout::tick_array_len_ok;

use crate::amm_core::ticks::cross_tick;
use crate::constants::TICK_ARRAY_SEED;
use crate::errors::AmmError;
use crate::state::{Pool, TickArray};

///
const BITMAP_WORDS_OFFSET: usize = 8 + 40;
const TICKS_PER_ARRAY: usize = 60;





///
pub fn math_err(e: MathError) -> Error {
    match e {
        MathError::InvalidFeeForExactOut => AmmError::InvalidFeeForExactOut.into(),
        MathError::FeeRateTooHigh => AmmError::FeeInvalid.into(),
        MathError::TickOutOfRange => AmmError::TickInvalid.into(),
        MathError::InvalidTickArrayBoundary | MathError::BitmapLengthMismatch | MathError::InvalidTickSpacing => {
            AmmError::SeedsMismatch.into()
        }
        MathError::LiquidityUnderflow | MathError::LiquidityOverflow | MathError::ZeroLiquidity => {
            AmmError::InsufficientLiquidity.into()
        }
        _ => AmmError::MathOverflow.into(),
    }
}





///
///
pub fn pool_mut<'a>(info: &'a AccountInfo<'_>) -> Result<RefMut<'a, Pool>> {
    let data = info.try_borrow_mut_data()?;
    require!(data.len() == Pool::LEN, AmmError::SeedsMismatch);
    Ok(RefMut::map(data, |d| bytemuck::from_bytes_mut::<Pool>(&mut (**d)[8..Pool::LEN])))
}

///
pub fn pool_ref<'a>(info: &'a AccountInfo<'_>) -> Result<Ref<'a, Pool>> {
    let data = info.try_borrow_data()?;
    require!(data.len() == Pool::LEN, AmmError::SeedsMismatch);
    Ok(Ref::map(data, |d| bytemuck::from_bytes::<Pool>(&(**d)[8..Pool::LEN])))
}

///
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PoolWork {
    pub tick_spacing: u16,
    pub lp_fee: u32,
    pub protocol_fee: u32,
///
    pub protocol_fee_currency: u8,
    pub tick: i32,
    pub sqrt_price_x64: u128,
    pub liquidity: u128,
    pub fee_growth_global0_x64: u128,
    pub fee_growth_global1_x64: u128,
    pub protocol_fees_owed0: u64,
    pub protocol_fees_owed1: u64,
    pub hook_claims0: u64,
    pub hook_claims1: u64,
    pub volume0: u128,
    pub volume1: u128,
    pub lp_fees0: u128,
    pub lp_fees1: u128,
    pub hook_fees0: u128,
    pub hook_fees1: u128,
    pub swap_count: u64,
}

impl PoolWork {
    pub fn read(p: &Pool) -> Self {
        Self {
            tick_spacing: p.tick_spacing,
            lp_fee: p.lp_fee,
            protocol_fee: p.protocol_fee,
            protocol_fee_currency: p.protocol_fee_currency,
            tick: p.tick_current,
            sqrt_price_x64: p.sqrt_price_x64.get(),
            liquidity: p.liquidity.get(),
            fee_growth_global0_x64: p.fee_growth_global0_x64.get(),
            fee_growth_global1_x64: p.fee_growth_global1_x64.get(),
            protocol_fees_owed0: p.protocol_fees_owed0,
            protocol_fees_owed1: p.protocol_fees_owed1,
            hook_claims0: p.hook_claims0,
            hook_claims1: p.hook_claims1,
            volume0: p.volume0.get(),
            volume1: p.volume1.get(),
            lp_fees0: p.lp_fees0.get(),
            lp_fees1: p.lp_fees1.get(),
            hook_fees0: p.hook_fees0.get(),
            hook_fees1: p.hook_fees1.get(),
            swap_count: p.swap_count,
        }
    }

///
///
    pub fn write(&self, p: &mut Pool) {
        p.tick_current = self.tick;
        p.sqrt_price_x64.set(self.sqrt_price_x64);
        p.liquidity.set(self.liquidity);
        p.fee_growth_global0_x64.set(self.fee_growth_global0_x64);
        p.fee_growth_global1_x64.set(self.fee_growth_global1_x64);
        p.protocol_fees_owed0 = self.protocol_fees_owed0;
        p.protocol_fees_owed1 = self.protocol_fees_owed1;
        p.hook_claims0 = self.hook_claims0;
        p.hook_claims1 = self.hook_claims1;
        p.volume0.set(self.volume0);
        p.volume1.set(self.volume1);
        p.lp_fees0.set(self.lp_fees0);
        p.lp_fees1.set(self.lp_fees1);
        p.hook_fees0.set(self.hook_fees0);
        p.hook_fees1.set(self.hook_fees1);
        p.swap_count = self.swap_count;
    }

///
    pub fn flush(&self, pool: &AccountInfo<'_>) -> Result<()> {
        let mut p = pool_mut(pool)?;
        self.write(&mut p);
        Ok(())
    }

///
    pub fn reload(pool: &AccountInfo<'_>) -> Result<Self> {
        let p = pool_ref(pool)?;
        Ok(Self::read(&p))
    }

    pub fn snapshot(&self) -> PoolSnapshot {
        PoolSnapshot {
            sqrt_price_x64: self.sqrt_price_x64,
            tick: self.tick,
            liquidity: self.liquidity,
            lp_fee_pips: self.lp_fee,
        }
    }

///
///
///
    pub fn apply_claims_delta(&mut self, delta: &BalanceDelta) -> Result<()> {
        self.hook_claims0 = apply_signed(self.hook_claims0, delta.amount0)?;
        self.hook_claims1 = apply_signed(self.hook_claims1, delta.amount1)?;
        Ok(())
    }

///
    pub fn count_input(&mut self, zero_for_one: bool, input: u64, lp_fees: u64) -> Result<()> {
        let (v, f) = if zero_for_one {
            (&mut self.volume0, &mut self.lp_fees0)
        } else {
            (&mut self.volume1, &mut self.lp_fees1)
        };
        *v = v.checked_add(input as u128).ok_or(AmmError::MathOverflow)?;
        *f = f.checked_add(lp_fees as u128).ok_or(AmmError::MathOverflow)?;
        Ok(())
    }
}

///
pub fn apply_signed(claim: u64, delta: i128) -> Result<u64> {
    let next = (claim as i128).checked_add(delta).ok_or(AmmError::MathOverflow)?;
    require!(next >= 0, AmmError::HookClaimsInsufficient);
    u64::try_from(next).map_err(|_| AmmError::MathOverflow.into())
}





///
///
pub struct BitmapWords {
    pub tick_spacing: u16,
    pub words: Vec<u64>,
}

impl BitmapWords {
///
///
    pub fn read(info: &AccountInfo<'_>, tick_spacing: u16) -> Result<Self> {
        let data = info.try_borrow_data()?;
        require!(
            data.len() == carpenter_types::layout::bitmap_account_len(tick_spacing)
                && data[..8] == carpenter_types::disc::account::TICK_ARRAY_BITMAP,
            AmmError::SeedsMismatch
        );
        let words: Vec<u64> = data[BITMAP_WORDS_OFFSET..]
            .chunks_exact(8)
            .map(|c| u64::from_le_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]))
            .collect();
        cbitmap::check_words(&words, tick_spacing).map_err(math_err)?;
        Ok(Self { tick_spacing, words })
    }

    pub fn is_initialized(&self, start_index: i32) -> Result<bool> {
        let bit = cbitmap::array_bit(start_index, self.tick_spacing).map_err(math_err)?;
        cbitmap::is_initialized(&self.words, bit).map_err(math_err)
    }

///
///
///
///
    pub fn set(&mut self, info: &AccountInfo<'_>, start_index: i32, initialized: bool) -> Result<()> {
        let bit = cbitmap::array_bit(start_index, self.tick_spacing).map_err(math_err)?;
        if cbitmap::is_initialized(&self.words, bit).map_err(math_err)? == initialized {
            return Ok(());
        }
        require!(info.is_writable, AmmError::HookActionAccountMissing);
        cbitmap::set_initialized(&mut self.words, bit, initialized).map_err(math_err)?;
        let wi = (bit / 64) as usize;
        let mut data = info.try_borrow_mut_data()?;
        let at = BITMAP_WORDS_OFFSET + 8 * wi;
        data[at..at + 8].copy_from_slice(&self.words[wi].to_le_bytes());
        Ok(())
    }
}





///
///
///
///
///
///
///
pub struct TickArraySet<'r, 'info> {
    accounts: &'r [AccountInfo<'info>],
///
    verified: Vec<(i32, usize)>,
}

impl<'r, 'info> TickArraySet<'r, 'info> {
///
///
    pub fn new(accounts: &'r [AccountInfo<'info>]) -> Result<Self> {
        for (i, a) in accounts.iter().enumerate() {
            for b in &accounts[i + 1..] {
                require_keys_neq!(*a.key, *b.key, AmmError::DuplicateTickArray);
            }
        }
        Ok(Self { accounts, verified: Vec::new() })
    }

///
///
    pub fn keys(&self) -> impl Iterator<Item = &'info Pubkey> + use<'r, 'info> {
        self.accounts.iter().map(|a| a.key)
    }

    pub fn info(&self, idx: usize) -> &'r AccountInfo<'info> {
        &self.accounts[idx]
    }

///
///
///
///
///
    pub fn find(&mut self, pool_key: &Pubkey, start_index: i32, missing: AmmError) -> Result<usize> {
        if let Some(&(_, i)) = self.verified.iter().find(|(s, _)| *s == start_index) {
            return Ok(i);
        }
        let (key, _) = Pubkey::find_program_address(
            &[TICK_ARRAY_SEED, pool_key.as_ref(), &start_index.to_le_bytes()],
            &crate::ID,
        );
        let i = self.accounts.iter().position(|a| *a.key == key).ok_or(missing)?;
        let a = &self.accounts[i];
        require_keys_eq!(*a.owner, crate::ID, AmmError::SeedsMismatch);
        {
            let data = a.try_borrow_data()?;
            require!(
                tick_array_len_ok(data.len()) && data[..8] == carpenter_types::disc::account::TICK_ARRAY,
                AmmError::SeedsMismatch
            );
            let ta = bytemuck::from_bytes::<TickArray>(&data[8..TickArray::LEN]);
            require_keys_eq!(ta.pool, *pool_key, AmmError::SeedsMismatch);
            require!(ta.start_index == start_index, AmmError::SeedsMismatch);
        }
        self.verified.push((start_index, i));
        Ok(i)
    }

///
    pub fn find_key(&self, key: &Pubkey) -> Option<&'r AccountInfo<'info>> {
        self.accounts.iter().find(|a| a.key == key)
    }

///
    pub fn with_array<R>(&self, idx: usize, f: impl FnOnce(&TickArray) -> R) -> Result<R> {
        let data = self.accounts[idx].try_borrow_data()?;
        Ok(f(bytemuck::from_bytes::<TickArray>(&data[8..TickArray::LEN])))
    }

///
///
///
///
///
///
    pub fn with_array_mut<R>(
        &self,
        idx: usize,
        missing: AmmError,
        f: impl FnOnce(&mut TickArray) -> R,
    ) -> Result<R> {
        let a = &self.accounts[idx];



        if !a.is_writable {
            return Err(missing.into());
        }
        let mut data = a.try_borrow_mut_data()?;
        Ok(f(bytemuck::from_bytes_mut::<TickArray>(&mut data[8..TickArray::LEN])))
    }
}

///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TickLoc {
    pub array: usize,
    pub offset: usize,
}

///
///
pub fn next_initialized_tick(
    set: &mut TickArraySet<'_, '_>,
    bitmap: &BitmapWords,
    pool_key: &Pubkey,
    tick: i32,
    zero_for_one: bool,
    missing: AmmError,
) -> Result<Option<(i32, TickLoc)>> {
    let spacing = bitmap.tick_spacing;
    let s = spacing as i32;
    let start = cbitmap::array_start_index(tick, spacing).map_err(math_err)?;
    let compressed = tick.div_euclid(s) * s;
    let current_offset = ((compressed - start) / s) as usize;



    if bitmap.is_initialized(start)? {
        let idx = set.find(pool_key, start, missing)?;
        let found = set.with_array(idx, |ta| search_array(ta, current_offset, zero_for_one, zero_for_one))?;
        if let Some(o) = found {
            return Ok(Some((start + o as i32 * s, TickLoc { array: idx, offset: o })));
        }
    }

    let mut last = start;
    loop {
        let next = cbitmap::next_initialized_array_start_index(&bitmap.words, spacing, last, zero_for_one)
            .map_err(math_err)?;
        let Some(ns) = next else { return Ok(None) };
        let idx = set.find(pool_key, ns, missing)?;
        let from = if zero_for_one { TICKS_PER_ARRAY - 1 } else { 0 };
        let found = set.with_array(idx, |ta| search_array(ta, from, zero_for_one, true))?;
        if let Some(o) = found {
            return Ok(Some((ns + o as i32 * s, TickLoc { array: idx, offset: o })));
        }

        last = ns;
    }
}

///
fn search_array(ta: &TickArray, from: usize, zero_for_one: bool, inclusive: bool) -> Option<usize> {
    if zero_for_one {
        let top = (if inclusive { Some(from) } else { from.checked_sub(1) })?;
        (0..=top.min(TICKS_PER_ARRAY - 1)).rev().find(|&o| ta.ticks[o].liquidity_gross.get() != 0)
    } else {
        let first = if inclusive { from } else { from + 1 };
        (first..TICKS_PER_ARRAY).find(|&o| ta.ticks[o].liquidity_gross.get() != 0)
    }
}





#[derive(Clone, Copy, Debug)]
pub struct WalkParams {
    pub zero_for_one: bool,
///
    pub amount_specified: i128,
    pub sqrt_price_limit_x64: u128,
///
    pub lp_fee: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WalkResult {
///
    pub delta: BalanceDelta,
///
    pub amount_in: u64,
///
    pub amount_out: u64,
///
    pub lp_fees: u64,
///
///
    pub protocol_fees: u64,
///
    pub protocol_currency: u8,
///
    pub swap_fee: u32,
}

///
///
pub fn compose_swap_fee(protocol_fee: u32, lp_fee: u32) -> u32 {
    carpenter_types::protocol_fee::compose_swap_fee(protocol_fee, lp_fee)
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
pub fn walk(
    st: &mut PoolWork,
    pool_key: &Pubkey,
    bitmap: &BitmapWords,
    set: &mut TickArraySet<'_, '_>,
    p: &WalkParams,
    missing: AmmError,
) -> Result<WalkResult> {
    let zfo = p.zero_for_one;
    require!(p.lp_fee <= MAX_LP_FEE, AmmError::FeeInvalid);
    let wf = walk_fee(st.protocol_fee, st.protocol_fee_currency, zfo, p.lp_fee);
    let swap_fee = wf.swap_fee;
    require!(!(swap_fee >= MAX_SWAP_FEE && p.amount_specified > 0), AmmError::InvalidFeeForExactOut);
    if p.amount_specified == 0 {
        return Ok(WalkResult { swap_fee, protocol_currency: wf.currency, ..WalkResult::default() });
    }
    let limit = p.sqrt_price_limit_x64;
    if zfo {
        require!(limit < st.sqrt_price_x64 && limit > MIN_SQRT_PRICE_X64, AmmError::PriceLimitInvalid);
    } else {
        require!(limit > st.sqrt_price_x64 && limit < MAX_SQRT_PRICE_X64, AmmError::PriceLimitInvalid);
    }

    let exact_in = p.amount_specified < 0;
    let named = u64::try_from(p.amount_specified.unsigned_abs()).map_err(|_| AmmError::MathOverflow)?;


    let magnitude = if !exact_in && wf.output_pips > 0 {
        gross_for_net(named, wf.output_pips).ok_or(AmmError::MathOverflow)?
    } else {
        named
    };
    let mut remaining = magnitude;
    let mut calculated: u64 = 0;
    let mut lp_fees: u64 = 0;
    let mut protocol_fees: u64 = 0;
    let mut fg_global = if zfo { st.fee_growth_global0_x64 } else { st.fee_growth_global1_x64 };

    while remaining != 0 && st.sqrt_price_x64 != limit {
        let price_start = st.sqrt_price_x64;
        let (tick_next, loc) = match next_initialized_tick(set, bitmap, pool_key, st.tick, zfo, missing)? {
            Some((t, l)) => (t.clamp(MIN_TICK, MAX_TICK), Some(l)),
            None => (if zfo { MIN_TICK } else { MAX_TICK }, None),
        };
        let sqrt_next = get_sqrt_price_at_tick(tick_next).map_err(math_err)?;
        let target = if zfo { sqrt_next.max(limit) } else { sqrt_next.min(limit) };

        let step = compute_swap(st.sqrt_price_x64, target, st.liquidity, remaining, swap_fee, exact_in, zfo, true)
            .map_err(math_err)?;
        st.sqrt_price_x64 = step.sqrt_price_next_x64;

        let gross_in = step.amount_in.checked_add(step.fee_amount).ok_or(AmmError::MathOverflow)?;
        if exact_in {
            remaining = remaining.checked_sub(gross_in).ok_or(AmmError::MathOverflow)?;
            calculated = calculated.checked_add(step.amount_out).ok_or(AmmError::MathOverflow)?;
        } else {
            remaining = remaining.checked_sub(step.amount_out).ok_or(AmmError::MathOverflow)?;
            calculated = calculated.checked_add(gross_in).ok_or(AmmError::MathOverflow)?;
        }

        let mut fee = step.fee_amount;
        if wf.input_pips > 0 {












            let share = input_share(fee, gross_in, swap_fee, wf.input_pips).ok_or(AmmError::MathOverflow)?;
            fee -= share;
            protocol_fees = protocol_fees.checked_add(share).ok_or(AmmError::MathOverflow)?;
        }
        lp_fees = lp_fees.checked_add(fee).ok_or(AmmError::MathOverflow)?;

        if st.liquidity > 0 {

            fg_global = fg_global.wrapping_add(((fee as u128) << 64) / st.liquidity);
        }

        if st.sqrt_price_x64 == sqrt_next {
            if let Some(l) = loc {
                let (fg0, fg1) = if zfo {
                    (fg_global, st.fee_growth_global1_x64)
                } else {
                    (st.fee_growth_global0_x64, fg_global)
                };
                let net =
                    set.with_array_mut(l.array, missing, |ta| cross_tick(&mut ta.ticks[l.offset], fg0, fg1))?;
                let net = if zfo { net.checked_neg().ok_or(AmmError::MathOverflow)? } else { net };
                st.liquidity = add_delta(st.liquidity, net).map_err(math_err)?;
            }
            st.tick = if zfo { tick_next - 1 } else { tick_next };
        } else if st.sqrt_price_x64 != price_start {
            st.tick = get_tick_at_sqrt_price(st.sqrt_price_x64).map_err(math_err)?;
        }
    }

    if zfo {
        st.fee_growth_global0_x64 = fg_global;
    } else {
        st.fee_growth_global1_x64 = fg_global;
    }


    let settled = magnitude - remaining;
    let (input, gross_output) = if exact_in { (settled, calculated) } else { (calculated, settled) };


    let out_share = output_share(gross_output, wf.output_pips);
    let output = gross_output - out_share;
    protocol_fees = protocol_fees.checked_add(out_share).ok_or(AmmError::MathOverflow)?;


    if wf.currency == 0 {
        st.protocol_fees_owed0 = st.protocol_fees_owed0.checked_add(protocol_fees).ok_or(AmmError::MathOverflow)?;
    } else {
        st.protocol_fees_owed1 = st.protocol_fees_owed1.checked_add(protocol_fees).ok_or(AmmError::MathOverflow)?;
    }

    let delta = if zfo {
        BalanceDelta { amount0: -(input as i128), amount1: output as i128 }
    } else {
        BalanceDelta { amount0: output as i128, amount1: -(input as i128) }
    };
    Ok(WalkResult {
        delta,
        amount_in: input,
        amount_out: output,
        lp_fees,
        protocol_fees,
        protocol_currency: wf.currency,
        swap_fee,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swap_fee_composition_matches_protocol_fee_library() {
        assert_eq!(compose_swap_fee(0, 3_000), 3_000);
        assert_eq!(compose_swap_fee(1_000, 0), 1_000);

        assert_eq!(compose_swap_fee(1_000, 3_000), 3_997);
        assert_eq!(compose_swap_fee(1_000, 1_000_000), 1_000_000);
    }

    #[test]
    fn claims_delta_debits_are_checked() {
        assert_eq!(apply_signed(10, -10).unwrap(), 0);
        assert_eq!(apply_signed(10, 5).unwrap(), 15);
        assert!(apply_signed(10, -11).is_err());
        assert!(apply_signed(u64::MAX, 1).is_err());
    }

    fn array_with(initialized: &[usize]) -> TickArray {
        let mut ta: TickArray = bytemuck::Zeroable::zeroed();
        for &o in initialized {
            ta.ticks[o].liquidity_gross.set(1);
        }
        ta
    }










    use carpenter_types::constants::{
        PIPS_DENOMINATOR, PROTOCOL_FEE_DEFAULT_PACKED, PROTOCOL_FEE_ON_CURRENCY0, PROTOCOL_FEE_ON_CURRENCY1,
    };

    const TEST_SPACING: u16 = 60;
    const TEST_LIQUIDITY: u128 = 1_000_000_000_000;
///
    const SQRT_ONE: u128 = 1u128 << 64;

    fn work(protocol_fee: u32, lp_fee: u32) -> PoolWork {
        PoolWork {
            tick_spacing: TEST_SPACING,
            lp_fee,
            protocol_fee,
            tick: 0,
            sqrt_price_x64: SQRT_ONE,
            liquidity: TEST_LIQUIDITY,
            ..PoolWork::default()
        }
    }

///
///
    fn walk_exact_in(st: &mut PoolWork, zero_for_one: bool, amount: u64) -> WalkResult {
        walk_signed(st, zero_for_one, -(amount as i128))
    }

///
    fn walk_signed(st: &mut PoolWork, zero_for_one: bool, amount_specified: i128) -> WalkResult {
        let accounts: Vec<AccountInfo<'_>> = Vec::new();
        let mut set = TickArraySet::new(&accounts).unwrap();
        let bitmap =
            BitmapWords { tick_spacing: TEST_SPACING, words: vec![0u64; cbitmap::word_count(TEST_SPACING).unwrap()] };
        let limit = if zero_for_one { MIN_SQRT_PRICE_X64 + 1 } else { MAX_SQRT_PRICE_X64 - 1 };
        let params = WalkParams { zero_for_one, amount_specified, sqrt_price_limit_x64: limit, lp_fee: st.lp_fee };

        walk(st, &Pubkey::new_from_array([7u8; 32]), &bitmap, &mut set, &params, AmmError::TickArrayMissing).unwrap()
    }

///
///
///
///
///
    #[test]
    fn a_quote_side_pool_takes_the_fee_off_a_sells_output() {
        const LP: u32 = 2_500;
        const AMOUNT: u64 = 1_000_000_000;
        let mut st = work(PROTOCOL_FEE_DEFAULT_PACKED, LP);
        st.protocol_fee_currency = PROTOCOL_FEE_ON_CURRENCY0;
        let r = walk_exact_in(&mut st, false, AMOUNT);

        let mut plain = work(0, LP);
        let p = walk_exact_in(&mut plain, false, AMOUNT);
        assert_eq!(r.swap_fee, LP, "the steps run at the LP fee alone");
        assert_eq!(st.sqrt_price_x64, plain.sqrt_price_x64, "the fee never enters the curve");
        assert_eq!(r.amount_in, p.amount_in);
        assert_eq!(r.lp_fees, p.lp_fees, "the LPs' fee is what a protocol-free walk pays");
        let want = ((p.amount_out as u128) * 500 / PIPS_DENOMINATOR as u128) as u64;
        assert!(want > 0, "the branch actually ran");
        assert_eq!(r.protocol_fees, want, "floor(gross_out · 500 / 1e6)");
        assert_eq!(r.amount_out, p.amount_out - want, "the caller receives the output net of it");
        assert_eq!(r.delta.amount0, (p.amount_out - want) as i128, "the delta is net");
        assert_eq!(r.protocol_currency, 0);
        assert_eq!((st.protocol_fees_owed0, st.protocol_fees_owed1), (want, 0), "owed on the OUTPUT, the quote");
        assert_eq!(st.fee_growth_global1_x64, plain.fee_growth_global1_x64, "LP growth unchanged");
        assert_eq!(st.fee_growth_global0_x64, 0);


        let mut st = work(PROTOCOL_FEE_DEFAULT_PACKED, LP);
        st.protocol_fee_currency = PROTOCOL_FEE_ON_CURRENCY0;
        let b = walk_exact_in(&mut st, true, AMOUNT);
        assert_eq!(b.swap_fee, 2_999, "compose(500, 2500)");
        assert_eq!(b.protocol_fees, 500_000, "floor(1e9 · 500 / 1e6)");
        assert_eq!((st.protocol_fees_owed0, st.protocol_fees_owed1), (500_000, 0));
        assert_eq!(b.protocol_currency, 0);
    }

///
///
    #[test]
    fn a_currency1_quote_takes_the_fee_off_a_zero_for_one_output() {
        let mut st = work(PROTOCOL_FEE_DEFAULT_PACKED, 3_000);
        st.protocol_fee_currency = PROTOCOL_FEE_ON_CURRENCY1;
        let r = walk_exact_in(&mut st, true, 1_000_000_000);
        let mut plain = work(0, 3_000);
        let p = walk_exact_in(&mut plain, true, 1_000_000_000);
        let want = ((p.amount_out as u128) * 500 / PIPS_DENOMINATOR as u128) as u64;
        assert_eq!(r.protocol_fees, want);
        assert_eq!(r.amount_out, p.amount_out - want);
        assert_eq!((st.protocol_fees_owed0, st.protocol_fees_owed1), (0, want));
        assert_eq!(r.protocol_currency, 1);
    }

///
///
///
    #[test]
    fn an_exact_out_sell_nets_exactly_what_it_named() {
        for named in [1u64, 1_999, 2_000, 2_001, 1_000_000, 987_654_321] {
            let mut st = work(PROTOCOL_FEE_DEFAULT_PACKED, 2_500);
            st.protocol_fee_currency = PROTOCOL_FEE_ON_CURRENCY0;
            let r = walk_signed(&mut st, false, named as i128);
            assert_eq!(r.amount_out, named, "named {named}: exactly");
            let gross = carpenter_types::protocol_fee::gross_for_net(named, 500).unwrap();
            assert_eq!(r.protocol_fees, gross - named, "named {named}: the gross-up is the fee");
            assert_eq!(st.protocol_fees_owed0, gross - named);

            let mut plain = work(0, 2_500);
            let p = walk_signed(&mut plain, false, gross as i128);
            assert_eq!(r.amount_in, p.amount_in, "named {named}");
        }
    }

///
///
    #[test]
    fn a_zero_rate_books_nothing_on_either_side() {
        for code in [0u8, PROTOCOL_FEE_ON_CURRENCY0, PROTOCOL_FEE_ON_CURRENCY1] {
            for zfo in [true, false] {
                let mut st = work(0, 2_500);
                st.protocol_fee_currency = code;
                let r = walk_exact_in(&mut st, zfo, 1_000_000_000);
                let mut plain = work(0, 2_500);
                let p = walk_exact_in(&mut plain, zfo, 1_000_000_000);
                assert_eq!(r.protocol_fees, 0);
                assert_eq!((st.protocol_fees_owed0, st.protocol_fees_owed1), (0, 0));
                assert_eq!(r.delta, p.delta);
                assert_eq!(st, plain_with_code(plain, code), "code {code} zfo {zfo}: the same walk");
            }
        }
    }

    fn plain_with_code(mut w: PoolWork, code: u8) -> PoolWork {
        w.protocol_fee_currency = code;
        w
    }

///
///
///
///
    #[test]
    fn the_protocol_fee_splits_by_direction_and_conserves_the_step_fee() {
        const LP: u32 = 3_000;
        const PF_DOWN: u32 = 1_000;
        const PF_UP: u32 = 2_000;
        const AMOUNT: u64 = 1_000_000_000;
        let packed = PF_DOWN | (PF_UP << 12);

        for (zfo, pf) in [(true, PF_DOWN), (false, PF_UP)] {
            let mut st = work(packed, LP);
            let r = walk_exact_in(&mut st, zfo, AMOUNT);


            assert_eq!(r.swap_fee, compose_swap_fee(pf, LP), "zero_for_one = {zfo}: the direction's own nibble");
            assert_ne!(compose_swap_fee(PF_DOWN, LP), compose_swap_fee(PF_UP, LP), "the legs must differ");


            assert_eq!(r.amount_in, AMOUNT, "the whole amount is consumed in one step");
            let want = ((r.amount_in as u128) * (pf as u128) / PIPS_DENOMINATOR as u128) as u64;
            assert_eq!(r.protocol_fees, want, "zero_for_one = {zfo}: floor(gross_in · protocol_fee / 1e6)");
            assert!(r.protocol_fees > 0, "the branch actually ran");



            let mut plain = work(0, r.swap_fee);
            let p = walk_exact_in(&mut plain, zfo, AMOUNT);
            assert_eq!(p.swap_fee, r.swap_fee);
            assert_eq!(p.amount_in, r.amount_in);
            assert_eq!(p.amount_out, r.amount_out, "the split moves no output");
            assert_eq!(p.protocol_fees, 0, "a protocol-free walk books none");
            assert_eq!(r.lp_fees + r.protocol_fees, p.lp_fees, "protocol + lp == the step fee");


            let (owed_in, owed_out, fg_in, fg_out) = if zfo {
                (st.protocol_fees_owed0, st.protocol_fees_owed1, st.fee_growth_global0_x64, st.fee_growth_global1_x64)
            } else {
                (st.protocol_fees_owed1, st.protocol_fees_owed0, st.fee_growth_global1_x64, st.fee_growth_global0_x64)
            };
            assert_eq!(owed_in, r.protocol_fees, "zero_for_one = {zfo}: owed on the input currency");
            assert_eq!(owed_out, 0, "zero_for_one = {zfo}: nothing owed on the output currency");
            assert_eq!(fg_in, ((r.lp_fees as u128) << 64) / TEST_LIQUIDITY, "R-11 growth is the LP half only");
            assert_eq!(fg_out, 0);
        }
    }

///
///
    #[test]
    fn a_zero_lp_fee_makes_the_whole_step_fee_the_protocol_share() {
        const PF: u32 = 1_000;
        let mut st = work(PF, 0);
        let r = walk_exact_in(&mut st, true, 1_000_000_000);
        assert_eq!(r.swap_fee, PF, "compose_swap_fee(p, 0) == p");
        assert_eq!(r.lp_fees, 0, "nothing is left for the LPs");
        assert!(r.protocol_fees > 0);
        assert_eq!(st.protocol_fees_owed0, r.protocol_fees);
        assert_eq!(st.fee_growth_global0_x64, 0, "no LP fee, no growth");


        let mut plain = work(0, PF);
        let p = walk_exact_in(&mut plain, true, 1_000_000_000);
        assert_eq!(r.protocol_fees, p.lp_fees);
    }

///
///
    #[test]
    fn the_nibble_split_matches_the_protocol_fee_library() {
        let packed: u32 = 1_000 | (2_000 << 12);
        assert_eq!(packed & 0xfff, 1_000, "getZeroForOneFee = self & 0xfff");
        assert_eq!((packed >> 12) & 0xfff, 2_000, "getOneForZeroFee = self >> 12");
    }

    #[test]
    fn array_search_is_inclusive_down_and_exclusive_up() {
        let ta = array_with(&[10, 20]);
        assert_eq!(search_array(&ta, 20, true, true), Some(20));
        assert_eq!(search_array(&ta, 19, true, true), Some(10));
        assert_eq!(search_array(&ta, 9, true, true), None);
        assert_eq!(search_array(&ta, 10, false, false), Some(20));
        assert_eq!(search_array(&ta, 20, false, false), None);
        assert_eq!(search_array(&ta, 0, false, true), Some(10));
        assert_eq!(search_array(&ta, 59, true, true), Some(20));
        assert_eq!(search_array(&ta, 0, true, false), None);
    }
}
