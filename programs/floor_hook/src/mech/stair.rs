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

use crate::cu::mark;
use anchor_lang::prelude::*;
use anchor_lang::system_program;
use floor_math::stair::VaultListTarget;
use floor_types::constants::{
    MAX_STAIR_STEPS, SALT_CUSHION, SALT_POL_RELIST, SALT_STAIR_RESERVE, SALT_STAIR_STEP0, SALT_STAIR_TAIL,
    STAIR_STEPS_PER_RATCHET,
};
use floor_types::layout::Layout;

use crate::errors::HookError;
use crate::events::{StairRatcheted, VaultTokensFed};
use crate::mech::amm_cpi;
use crate::mech::owners;
use crate::mech::positions::{add, intake, math, sub, take_held, to_wall, Mech, MechEvent};
use crate::state::{FloorPool, StairState};
use floor_types::owners::StairOwners;

fn map<T>(r: floor_math::FloorMathResult<T>) -> Result<T> {
    r.map_err(|e| {
        msg!("floor-math: {}", e);
        error!(HookError::MathOverflow)
    })
}

///
pub fn stair_key(floor_pool: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[floor_types::seeds::STAIR, floor_pool.as_ref()], &crate::ID)
}

///
pub fn step_salt(i: u8) -> u64 {
    SALT_STAIR_STEP0 + i as u64
}

///
///
///
///
#[inline(never)]
pub fn load<'info>(set: &[AccountInfo<'info>], floor_pool: &Pubkey) -> Result<(AccountInfo<'info>, Box<StairState>)> {
    let disc = <floor_types::layout::StairState as Layout>::DISCRIMINATOR;
    let acct = set
        .iter()
        .find(|a| {
            if *a.owner != crate::ID {
                return false;
            }
            let Ok(data) = a.try_borrow_data() else { return false };
            if data.len() < StairState::LEN || data[..8] != disc {
                return false;
            }
            let bump = data[8 + core::mem::offset_of!(StairState, bump)];
            Pubkey::create_program_address(&[floor_types::seeds::STAIR, floor_pool.as_ref(), &[bump]], &crate::ID)
                .is_ok_and(|k| k == *a.key)
        })
        .cloned()
        .ok_or_else(|| error!(HookError::StairStateMissing))?;
    let st = {
        let data = acct.try_borrow_data()?;
        require!(data.len() >= StairState::LEN, HookError::StairStateMissing);
        require!(data[..8] == <floor_types::layout::StairState as Layout>::DISCRIMINATOR, HookError::StairStateMissing);
        Box::new(*bytemuck::from_bytes::<StairState>(&data[8..StairState::LEN]))
    };
    Ok((acct, st))
}

///
#[inline(never)]
pub fn store(acct: &AccountInfo, st: &StairState) -> Result<()> {
    let mut data = acct.try_borrow_mut_data()?;
    require!(data.len() >= StairState::LEN, HookError::StairStateMissing);
    data[8..StairState::LEN].copy_from_slice(bytemuck::bytes_of(st));
    Ok(())
}

///
///
///
///
///
#[inline(never)]
pub fn create_stair_state<'info>(
    payer: &AccountInfo<'info>,
    target: &AccountInfo<'info>,
    system: &AccountInfo<'info>,
    floor_pool: &Pubkey,
    stair_owner: u8,
) -> Result<()> {


    let space = crate::mech::owners::stair_account_len(stair_owner);
    if *target.owner == crate::ID {


        if target.data_len() < space {
            let required = Rent::get()?.minimum_balance(space);
            let have = target.lamports();
            if have < required {
                system_program::transfer(
                    CpiContext::new(system.key(), system_program::Transfer { from: payer.clone(), to: target.clone() }),
                    required - have,
                )?;
            }
            target.resize(space)?;
        }
        return Ok(());
    }
    let (key, bump) = stair_key(floor_pool);
    require_keys_eq!(*target.key, key, HookError::StairStateMissing);
    let required = Rent::get()?.minimum_balance(space);
    let have = target.lamports();
    if have < required {
        system_program::transfer(
            CpiContext::new(system.key(), system_program::Transfer { from: payer.clone(), to: target.clone() }),
            required - have,
        )?;
    }
    let seeds: &[&[u8]] = &[floor_types::seeds::STAIR, floor_pool.as_ref(), &[bump]];
    system_program::allocate(
        CpiContext::new_with_signer(system.key(), system_program::Allocate { account_to_allocate: target.clone() }, &[seeds]),
        space as u64,
    )?;
    system_program::assign(
        CpiContext::new_with_signer(system.key(), system_program::Assign { account_to_assign: target.clone() }, &[seeds]),
        &crate::ID,
    )?;
    let mut data = target.try_borrow_mut_data()?;
    data[..8].copy_from_slice(&<floor_types::layout::StairState as Layout>::DISCRIMINATOR);
    let mut st = <StairState as bytemuck::Zeroable>::zeroed();
    st.bump = bump;
    data[8..StairState::LEN].copy_from_slice(bytemuck::bytes_of(&st));
    Ok(())
}

///
///
///
///
///
#[inline(never)]
pub fn planned_of(s: &FloorPool, total: u64, n: u8, growth_bps: u16) -> Result<[u64; 16]> {
    map(floor_math::genesis_layout::planned_steps(layout_f(s), total, n, growth_bps))
}

///
pub fn layout_f(s: &FloorPool) -> bool {
    s.stair_kind == floor_types::constants::STAIR_KIND_GENESIS && s.stair_owner == floor_types::constants::STAIR_OWNER_TEAM
}



///
///
///
///
///
///
///
#[inline(never)]
pub fn place_at_graduation(
    m: &mut Mech,
    s: &mut FloorPool,
    credited: u64,
    owner: Option<&floor_types::owners::StairOwnerArg>,
) -> Result<()> {
    let (acct, mut st) = load(&m.accts.tick_arrays, m.cpi.floor_pool.key)?;
    require!(st.version == 0, HookError::StairStateMissing);
    let spacing = s.tick_spacing;
    let n = s.stair_steps;
    st.version = 1;
    st.steps = n;
    st.growth_bps = s.stair_growth_bps;

    st.open_edge = s.ladder_upper;


    let params = floor_math::stair::stair_params(s.stair_kind).ok_or(error!(HookError::StairShapeOutOfRange))?;
    st.cushion_bottom_bps = params.cushion_bottom_bps as u16;
    st.pol_to_mcap_bps = params.pol_to_mcap_bps as u16;
    st.cushion_ticks = params.cushion_ticks;



    let extra = match owner {
        Some(a) if a.kind == floor_types::constants::STAIR_OWNER_TEAM_BUY => a.owner_tokens,
        _ => 0,
    };







    let custom = (s.mode == floor_types::constants::MODE_CUSTOM && s.stair_kind == floor_types::constants::STAIR_KIND_PRESET)
        || layout_f(s);
    let ladder = sub(credited, extra)?;
    let (steps_total, tail_total) = if custom {
        floor_math::stair::stair_tail_split_supply(ladder, &params)
    } else {
        floor_math::stair::stair_tail_split_for(ladder, &params)
    };
    let steps_total = add(steps_total, extra)?;
    st.stair_total = steps_total;
    st.tail_total = tail_total;


    let planned = planned_of(s, steps_total, n, st.growth_bps)?;
    for i in 0..n {
        let (lo, hi) = map(floor_math::stair::step_range(st.open_edge, i, n, spacing))?;
        st.step_lower[i as usize] = lo;
        st.step_upper[i as usize] = hi;
    }

    let mut books = match owner {
        Some(a) => {
            crate::mech::owners::validate(a, s)?;
            let mut o = crate::mech::owners::read(&acct, s)?.ok_or(error!(HookError::StairStateMissing))?;
            require!(a.kind != floor_types::constants::STAIR_OWNER_TEAM || a.owner_tokens <= steps_total, HookError::StairOwnerBadConfig);
            crate::mech::owners::fill(&mut o, a, &planned, n, Clock::get()?.unix_timestamp)?;
            crate::mech::owners::fill_team_slots(&m.cpi.floor_pool, a)?;
            Some(o)
        }
        None => None,
    };
    let mut standing = 0u64;


    let own = 1u8;
    let left = place_own(m, s, &mut st, 0, planned[0])?;
    standing = add(standing, sub(planned[0], left)?)?;
    to_wall(s, left)?;
    if let Some(o) = books.as_deref_mut() {
        let placed0 = sub(planned[0], left)?;
        let dropped = o.tokens[0].saturating_sub(placed0);
        o.tokens[0] -= dropped;
        crate::mech::owners::book_clamp(o, dropped);
    }
    st.placed = own;
    if own < n {
        let tokens: u64 = planned[own as usize..n as usize].iter().sum();
        let (lo, hi) = map(floor_math::stair::reserve_range(st.open_edge, own, n, spacing))?;
        let used = open_whole(m, s, SALT_STAIR_RESERVE, lo, hi, tokens)?;
        st.reserve_lower = lo;
        st.reserve_upper = hi;
        st.reserve_liq.set(used.1);
        standing = add(standing, used.0)?;
        to_wall(s, sub(tokens, used.0)?)?;
        if let Some(o) = books.as_deref_mut() {


            let fed = floor_math::owners::record_reserve_shortfall(&planned, &mut o.cut, 0, own, n, sub(tokens, used.0)?);
            o.set_reserve_fed(fed);
            crate::mech::owners::cap_reserve(o, own, n, used.0);
        }
    }
    let (lo, hi) = map(floor_math::stair::tail_range(st.open_edge, n, spacing))?;
    let used = open_whole(m, s, SALT_STAIR_TAIL, lo, hi, tail_total)?;
    st.tail_lower = lo;
    st.tail_upper = hi;
    st.tail_liq.set(used.1);
    st.tail_placed = 1;
    standing = add(standing, used.0)?;
    to_wall(s, sub(tail_total, used.0)?)?;
    st.first_open = 0;

    let counted = counted_tokens_with(m.market.sqrt_price_x64, s, &st, books.as_deref(), &mut SqrtCache::default())?;
    s.ladder_tokens_remaining = crate::mech::owners::ladder_less_counted(standing, counted)?;
    s.ladder_lower = lo;
    s.ladder_upper = frontier(&st);
    s.stair_ready = 1;
    s.needs_ratchet = 1;
    if let Some(o) = books.as_deref() {
        require!(o.owner_total != 0, HookError::StairOwnerBadConfig);
        crate::mech::owners::write(&acct, o)?;
        m.events.push(MechEvent::StairOwnersPlaced(crate::events::StairOwnersPlaced {
            pool: s.amm_pool,
            kind: o.kind,
            counted: o.counted,
            payee_bps: o.payee_bps,
            owner_total: o.owner_total,
            first_step: o.first_step,
            payee_count: crate::state::slots::read(&m.cpi.floor_pool)?.team_count,
            anchor: o.anchor,
            first_unlock_secs: o.first_unlock_secs,
            every_secs: o.every_secs,
        }));
    }
    store(&acct, &st)
}

///
///
#[inline(never)]
fn open_whole(m: &mut Mech, s: &FloorPool, salt: u64, lower: i32, upper: i32, tokens: u64) -> Result<(u64, u128)> {
    require!(m.market.tick >= upper, HookError::StairNotTokenOnly);
    let liq = math::position_liquidity(m.market.sqrt_price_x64, lower, upper, 0, tokens)?;
    if liq == 0 {
        return Ok((0, 0));
    }
    let used = open_token_only(m, s, salt, lower, upper, liq)?;
    Ok((used, liq))
}

///
///
///
#[inline(never)]
fn place_own(m: &mut Mech, s: &FloorPool, st: &mut StairState, i: u8, tokens: u64) -> Result<u64> {
    let line = m.market.tick;
    place_own_at(m, s, st, i, tokens, line)
}

///
#[inline(never)]
fn place_own_at(m: &mut Mech, s: &FloorPool, st: &mut StairState, i: u8, tokens: u64, line: i32) -> Result<u64> {
    let k = i as usize;
    st.step_liq[k].set(0);
    if tokens == 0 {
        return Ok(0);
    }
    let range = map(floor_math::stair::unsold_range(st.step_lower[k], st.step_upper[k], line, s.tick_spacing))?;
    let Some((lo, hi)) = range else { return Ok(tokens) };
    let liq = math::position_liquidity(m.market.sqrt_price_x64, lo, hi, 0, tokens)?;
    if liq == 0 {
        return Ok(tokens);
    }
    let used = open_token_only(m, s, step_salt(i), lo, hi, liq)?;
    st.step_upper[k] = hi;
    st.step_liq[k].set(liq);
    sub(tokens, used)
}

#[inline(never)]
fn open_token_only(m: &mut Mech, s: &FloorPool, salt: u64, lower: i32, upper: i32, liq: u128) -> Result<u64> {
    let r = m.open(s, salt, lower, upper, liq)?;
    let (q, b) = amm_cpi::used(&r)?;
    require!(q == 0, HookError::StairNotTokenOnly);
    Ok(b)
}

#[inline(never)]
fn open_quote_only(m: &mut Mech, s: &FloorPool, salt: u64, lower: i32, upper: i32, liq: u128) -> Result<u64> {
    let r = m.open(s, salt, lower, upper, liq)?;
    let (q, b) = amm_cpi::used(&r)?;
    require!(b == 0, HookError::InvariantViolated);
    Ok(q)
}

#[inline(never)]
fn close_position(m: &mut Mech, s: &FloorPool, salt: u64, lower: i32, upper: i32, liq: u128) -> Result<(u64, u64)> {
    let r = m.close(s, salt, lower, upper, liq)?;
    amm_cpi::back(&r)
}

///
///
pub fn frontier(st: &StairState) -> i32 {
    let mut at = if st.first_open < st.placed {
        st.step_upper[st.first_open as usize]
    } else if st.placed < st.steps && st.reserve_liq.get() != 0 {
        st.reserve_upper
    } else {
        st.tail_upper
    };
    if st.relist_liq.get() != 0 && st.relist_upper > at {
        at = st.relist_upper;
    }
    at
}

///
///
///
///
///
///
fn ask_line(m: &Mech, s: &FloorPool) -> Result<(i32, bool)> {
    let clear = math::floor_to_spacing(math::dear_edge(s)? - floor_math::stair::STAIR_CLEAR_TICKS, s.tick_spacing)?;
    let at = math::floor_to_spacing(m.market.tick, s.tick_spacing)?;
    if clear >= at {
        return Ok((m.market.tick, false));
    }




    let lower = clear.saturating_sub(math::ceil_to_spacing(floor_math::stair::doubling_ticks(1).unwrap_or(6_932), s.tick_spacing)?);
    let pool = m.cpi.pool.key;
    if !amm_cpi::has_tick_array(&m.accts.tick_arrays, pool, clear, m.spacing)
        || !amm_cpi::has_tick_array(&m.accts.tick_arrays, pool, lower, m.spacing)
    {
        return Ok((m.market.tick, false));
    }
    Ok((clear, true))
}

///
fn live_frontier(st: &StairState) -> Option<i32> {
    let mut at = if st.first_open < st.placed {
        let i = st.first_open as usize;
        if st.step_liq[i].get() != 0 { Some(st.step_upper[i]) } else { None }
    } else if st.placed < st.steps && st.reserve_liq.get() != 0 {
        Some(st.reserve_upper)
    } else if st.tail_liq.get() != 0 {
        Some(st.tail_upper)
    } else {
        None
    };
    if st.relist_liq.get() != 0 && at.map_or(true, |a| st.relist_upper > a) {
        at = Some(st.relist_upper);
    }
    at
}

///
///
#[inline(never)]
fn clear_of_asks(s: &FloorPool, st: &StairState) -> Result<Option<i32>> {
    match live_frontier(st) {
        Some(ask) => Ok(Some(map(floor_math::stair::clear_of_ask(ask, math::band_ticks(s)?, s.tick_spacing))?)),
        None => Ok(None),
    }
}



///
#[derive(Default)]
struct Take {
    collected: u64,
    pol_tokens: u64,
    more_left: bool,
///
    ops: u8,
///
    listed: u64,
///
    modifies: u8,
///
///
    budget: u8,
///
///
///
    reserved: u8,
///
///
///
    basis_tokens: u64,
    basis_cost: u128,
///
///
///
    books: Option<Box<StairOwners>>,
    owner_quote: u64,
    owner_fee: u64,
///
///
///
    relist_q: u64,
    home: u64,
///
///
///
///
///
    collect_deferred: bool,
///
///
    pending_in: u64,
///
///
    team: Option<Box<TeamCtx>>,
}

///
///
#[derive(Default)]
struct TeamCtx {
    bps: [u16; floor_types::slots::MAX_SLOTS],
    n: u8,
    parts: floor_types::slots::TeamParts,
///
///
    slot_ask: [u64; floor_types::slots::MAX_SLOTS],
///
    out_quote: u64,
    dirty: bool,
}

impl TeamCtx {
///
    fn load(fp: &AccountInfo) -> Result<Option<Box<TeamCtx>>> {
        let data = fp.try_borrow_data()?;
        let Some(t) = floor_types::slots::PayeeSlots::from_account(&data) else { return Ok(None) };
        let (bps, n) = t.team_bps();
        if n <= 1 {
            return Ok(None);
        }
        let parts = floor_types::slots::TeamParts::from_account(&data);
        Ok(Some(Box::new(TeamCtx { bps, n, parts, ..TeamCtx::default() })))
    }

///
///
    fn ask_of(&mut self, i: usize, quote: u64, payee_bps: u16) -> bool {
        let out = self.parts.out_of(i);
        if out == 0 || quote == 0 || floor_math::slots::in_weight(&self.bps, self.n, out) == 0 {
            return false;
        }
        let a = floor_math::owners::payee_due(quote, payee_bps);
        let c = floor_math::slots::cascade(a, &self.bps, self.n, out);
        for (x, v) in self.slot_ask.iter_mut().zip(c) {
            *x = x.saturating_add(v);
        }
        self.out_quote = self.out_quote.saturating_add(quote);
        true
    }

    fn slot_total(&self) -> u64 {
        self.slot_ask.iter().fold(0u64, |a, &v| a.saturating_add(v))
    }

///
///
    fn fold_pending(&mut self, pending: u64) -> u64 {
        let mut left = pending;
        for (a, p) in self.slot_ask.iter_mut().zip(self.parts.pending_ask.iter_mut()) {
            let v = core::mem::take(p).min(left);
            left -= v;
            *a = a.saturating_add(v);
        }
        self.dirty = true;
        pending - left
    }

///
///
///
    fn book_pass(&mut self, pass: &PayeePass, due: u64, pending_in: u64, deferred: bool) {
        if self.slot_total() == 0 {
            return;
        }
        let a0 = due.saturating_add(pending_in);
        for s in 0..floor_types::slots::MAX_SLOTS {
            let a = core::mem::take(&mut self.slot_ask[s]);
            if deferred {
                self.parts.pending_ask[s] = self.parts.pending_ask[s].saturating_add(floor_math::slots::ask_paid(a, pass.aside, due));
            } else {
                let q = floor_math::slots::ask_paid(a, pass.paid, a0);
                self.parts.quote_credit[s] = self.parts.quote_credit[s].saturating_add(q);
                self.parts.part_quote = self.parts.part_quote.saturating_add(q);
            }
        }
        self.dirty = true;
    }

///
///
    fn held_of(&mut self, i: usize, kept: u64) {
        let out = self.parts.out_of(i);
        if out == 0 || kept == 0 || floor_math::slots::in_weight(&self.bps, self.n, out) == 0 {
            return;
        }
        let c = floor_math::slots::cascade(kept, &self.bps, self.n, out);
        for (x, v) in self.parts.token_credit.iter_mut().zip(c) {
            *x = x.saturating_add(v);
        }
        self.parts.part_tokens = self.parts.part_tokens.saturating_add(kept);
        self.dirty = true;
    }
}

impl Take {
///
    fn room(&self, n: u8) -> bool {
        self.modifies + self.reserved + n <= self.budget
    }

///
    fn owner_on(&self, i: usize) -> u64 {
        self.books.as_ref().map_or(0, |o| o.tokens[i])
    }

///
    fn owner_in_reserve(&self, placed: u8, steps: u8) -> u64 {
        self.books.as_ref().map_or(0, |o| owners::in_reserve(o, placed, steps))
    }

///
///
    fn book_close(&mut self, i: usize, before: u64, after: u64, principal: u64, fee: u64) -> Result<()> {
        let Some(o) = self.books.as_deref_mut() else { return Ok(()) };
        let (oq, of) = owners::book_close(o, i, before, after, principal, fee);
        if let Some(t) = self.team.as_deref_mut() {
            t.ask_of(i, oq, o.payee_bps);
        }
        self.collected = sub(self.collected, of)?;
        self.owner_quote = add(self.owner_quote, oq)?;
        self.owner_fee = add(self.owner_fee, of)?;
        Ok(())
    }

///
    fn clamp_step(&mut self, i: usize, tokens: u64) {
        if let Some(o) = self.books.as_deref_mut() {
            let dropped = o.tokens[i].saturating_sub(tokens);
            o.tokens[i] -= dropped;
            owners::book_clamp(o, dropped);
        }
    }

///
///
    fn cap_reserve(&mut self, placed: u8, steps: u8, held: u64) {
        if let Some(o) = self.books.as_deref_mut() {
            owners::cap_reserve(o, placed, steps, held);
        }
    }

///
    fn set_reserve_fed(&mut self, v: u64) {
        if let Some(o) = self.books.as_deref_mut() {
            o.set_reserve_fed(v);
        }
    }

///
///
///
    fn reopened_reserve(&mut self, planned: &[u64; 16], placed: u8, steps: u8, prot: u64, shortfall: u64, used: u64) {
        if let Some(o) = self.books.as_deref_mut() {
            let fed = floor_math::owners::record_reserve_shortfall(planned, &mut o.cut, prot, placed, steps, shortfall);
            o.set_reserve_fed(fed);
            owners::cap_reserve(o, placed, steps, used - fed.min(used));
        }
    }

///
///
///
///
    fn hold_step(&mut self, i: usize, avail: u64) -> u64 {
        let kept = self.books.as_deref_mut().map_or(0, |o| owners::hold_step(o, i, avail));

        if let Some(t) = self.team.as_deref_mut() {
            t.held_of(i, kept);
        }
        kept
    }

///
    fn hold_reserve(&mut self, placed: u8, steps: u8, avail: u64) -> u64 {
        let mut kept = 0u64;
        for k in placed..steps.min(MAX_STAIR_STEPS) {
            kept += self.hold_step(k as usize, avail - kept);
        }
        kept
    }
}

///
///
#[inline(never)]
fn reserve_slot_asks(t: &mut TeamCtx, was: &[u64; 16], now: &[u64; 16], oq: u64, payee_bps: u16, j: u8, n: u8) {
    let sold: u128 = (j as usize..n.min(MAX_STAIR_STEPS) as usize).map(|k| was[k].saturating_sub(now[k]) as u128).sum();
    if sold == 0 || oq == 0 {
        return;
    }
    for k in j as usize..n.min(MAX_STAIR_STEPS) as usize {
        let q = (oq as u128 * was[k].saturating_sub(now[k]) as u128 / sold) as u64;
        t.ask_of(k, q, payee_bps);
    }
}

///
///
fn to_owner_held(s: &mut FloorPool, tokens: u64) -> Result<()> {
    s.ask_tokens_deployed = add(s.ask_tokens_deployed, tokens)?;
    Ok(())
}

impl Take {
}

///
#[inline(never)]
fn close_split(m: &mut Mech, s: &FloorPool, salt: u64, lower: i32, upper: i32, liq: u128) -> Result<(u64, u64, u64, u64)> {
    let r = m.close(s, salt, lower, upper, liq)?;
    let (q, b) = amm_cpi::back(&r)?;
    let (fq, fb) = amm_cpi::fees(&r);
    Ok((q, b, fq.min(q), fb.min(b)))
}

///
const SPLIT_MODIFIES: u8 = 3;
///
///
const STAIR_MODIFY_BUDGET: u8 = 4;
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
pub const RATCHET_MODIFY_BUDGET: u8 = 8;
///
pub const BID_MODIFIES: u8 = 1;

///
#[derive(Default)]
pub struct StairOutcome {
///
    pub cushion: Option<(i32, i32, u128)>,
///
///
///
    pub hold: Option<i32>,
///
///
    pub inflow: u64,
///
///
    pub parked: bool,
///
    pub cushion_ticks: i32,
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
#[inline(never)]
pub fn sweep_fees(m: &mut Mech, s: &mut FloorPool, which: u32, tip: bool) -> Result<(u64, u64, u64, u64)> {
    use crate::instructions::sweep_stair_fees::{SWEEP_CUSHION, SWEEP_RELIST, SWEEP_RESERVE, SWEEP_TAIL};
    let (acct, mut st) = load(&m.accts.tick_arrays, m.cpi.floor_pool.key)?;


    let mut books = owners::read(&acct, s)?;
    let (mut quote, mut tokens, mut to_vault) = (0u64, 0u64, 0u64);
    for i in st.first_open..st.placed.min(MAX_STAIR_STEPS) {
        let k = i as usize;
        if which & (1 << i) == 0 || st.step_liq[k].get() == 0 {
            continue;
        }
        let owner_on = books.as_ref().map_or(0, |o| o.tokens[k]);
        let before = if owner_on != 0 { owners::token_only(st.step_lower[k], st.step_upper[k], st.step_liq[k].get())? } else { 0 };
        let r = m.poke(s, step_salt(i), st.step_lower[k], st.step_upper[k])?;
        let (q, b) = amm_cpi::back(&r)?;
        let owned = books.as_deref_mut().map_or(0, |o| if owner_on != 0 { owners::fee_share(o, q, before, owner_on) } else { 0 });
        (quote, tokens, to_vault) = (add(quote, q - owned)?, add(tokens, b)?, add(to_vault, owned)?);
    }
    if which & SWEEP_RESERVE != 0 && st.placed < st.steps && st.reserve_liq.get() != 0 {
        let owner_on = books.as_ref().map_or(0, |o| owners::in_reserve(o, st.placed, st.steps));
        let before = if owner_on != 0 { owners::token_only(st.reserve_lower, st.reserve_upper, st.reserve_liq.get())? } else { 0 };
        let r = m.poke(s, SALT_STAIR_RESERVE, st.reserve_lower, st.reserve_upper)?;
        let (q, b) = amm_cpi::back(&r)?;
        let owned = books.as_deref_mut().map_or(0, |o| if owner_on != 0 { owners::fee_share(o, q, before, owner_on) } else { 0 });
        (quote, tokens, to_vault) = (add(quote, q - owned)?, add(tokens, b)?, add(to_vault, owned)?);
    }
    if which & SWEEP_TAIL != 0 && st.tail_liq.get() != 0 {
        let r = m.poke(s, SALT_STAIR_TAIL, st.tail_lower, st.tail_upper)?;
        let (q, b) = amm_cpi::back(&r)?;
        (quote, tokens) = (add(quote, q)?, add(tokens, b)?);
    }



    if which & SWEEP_RELIST != 0 && st.relist_liq.get() != 0 {
        let r = m.poke(s, SALT_POL_RELIST, st.relist_lower, st.relist_upper)?;
        let (q, b) = amm_cpi::back(&r)?;
        (quote, tokens) = (add(quote, q)?, add(tokens, b)?);
    }
    let mut cushion = 0u64;
    if which & SWEEP_CUSHION != 0 && st.cushion_liq.get() != 0 {
        let r = m.poke(s, SALT_CUSHION, st.cushion_lower, st.cushion_upper)?;
        let (q, b) = amm_cpi::back(&r)?;
        (cushion, tokens) = (q, add(tokens, b)?);
    }
    if quote == 0 && tokens == 0 && cushion == 0 && to_vault == 0 {
        return Ok((0, 0, 0, 0));
    }

    let tip = if tip { super::ratchet::keeper_tip(s, quote) } else { 0 };
    quote -= tip;
    to_wall(s, tokens)?;
    if let Some(o) = books.as_deref() {
        if to_vault != 0 {
            owners::write(&acct, o)?;
        }
    }
    if quote != 0 {
        st.swept_quote = add(st.swept_quote, quote)?;
        store(&acct, &st)?;
    }
    Ok((quote, tokens, add(cushion, to_vault)?, tip))
}

///
///
///
///
#[inline(never)]
fn close_cushion(m: &mut Mech, s: &mut FloorPool, lower: i32, upper: i32, liq: u128) -> Result<(u64, u64)> {
    let r = m.close(s, SALT_CUSHION, lower, upper, liq)?;
    let (q, b) = amm_cpi::back(&r)?;
    let (fq, fb) = amm_cpi::fees(&r);
    let (fq, fb) = (fq.min(q), fb.min(b));
    to_wall(s, fb)?;
    Ok((q - fq, b - fb))
}

///
///
#[inline(never)]
pub fn ratchet_positions(m: &mut Mech, s: &mut FloorPool, supply: u64) -> Result<StairOutcome> {
    if s.stair_ready == 0 {
        return Ok(StairOutcome::default());
    }
    let (acct, mut st) = load(&m.accts.tick_arrays, m.cpi.floor_pool.key)?;


    let budget = STAIR_MODIFY_BUDGET.min(RATCHET_MODIFY_BUDGET.saturating_sub(m.modifies.saturating_add(BID_MODIFIES)));



    let swept = core::mem::take(&mut st.swept_quote);

    let books = owners::read(&acct, s)?;
    let owner_sold0 = books.as_ref().map_or(0, |o| o.owner_sold);

    let team = if books.is_some() { TeamCtx::load(&m.cpi.floor_pool)? } else { None };
    let mut take = Take { budget, collected: swept, books, team, ..Take::default() };








    let params = st.params();
    let width = params.cushion_ticks;
    mark("cu st.load");
    let parked = st.cushion_liq.get() != 0 && cushion_target(m, s, &st, width)?.is_none();
    mark("cu st.target1");
    let busy = pol_side_busy(m, &st, parked)?;
    mark("cu st.busy");
    collect_steps(m, s, &mut st, &mut take, busy)?;
    mark("cu st.collect_steps");
    let cushion_held = collect_relist(m, s, &mut st, &mut take, parked)?;
    mark("cu st.collect_relist");
    feed_vault_tokens(m, s, &mut st, &mut take, cushion_held.1 != 0 && !parked)?;
    mark("cu st.feed");



    if !take.collect_deferred {
        let pending = s.owner_pending();
        if pending != 0 {
            take.collected = add(take.collected, pending)?;

            let slots_part = take.team.as_deref_mut().map_or(0, |t| t.fold_pending(pending));
            take.pending_in = pending - slots_part;
            s.set_owner_pending(0);
        }
    }





    let p = m.market.sqrt_price_x64;
    let mut sqrts = SqrtCache(Vec::with_capacity(48));
    if take.collected != 0 || st.cushion_liq.get() != 0 {
        let standing = add(standing_tokens_with(p, &st, &mut sqrts)?, take.pol_tokens)?;
        let counted = counted_tokens_with(p, s, &st, take.books.as_deref(), &mut sqrts)?;
        s.ladder_tokens_remaining = owners::ladder_less_counted(standing, counted)?;
    }
    mark("cu st.standing1");










    let arm = if take.collected == 0 { None } else { cushion_arm(m, s, &st, &take, cushion_held, width)? };
    mark("cu st.target2");
    take.home = if arm.is_some() { take.relist_q.min(take.collected) } else { 0 };
    let held = (add(cushion_held.0, take.home)?, cushion_held.1);
    let owed_now = book_payee(m, s, &st, supply, &mut take, held)?;
    mark("cu st.payee");



    let (pol_quote, split) = if take.collected == 0 {
        (0, floor_math::stair::StairSplit::default())
    } else {




        let classic = math::position_amounts(p, s.seed_lower, s.seed_upper, s.seed_liq.get())?.0;
        let pol_quote = add(classic, held.0)?;
        let floor_sqrt = math::sqrt_price_at_tick(s.floor_tick)?;
        let rest = sub(take.collected, take.home)?;
        let shares = map(floor_math::stair::stair_split_for(rest, p, floor_sqrt, pol_quote, supply, &params))?;
        mark("cu st.split_for");




        let to_pol = match arm {
            None => 0,
            Some(target) => {
                let first = floor_first_of(m, s, &st, supply, rest, held, &shares, take.room(2), target)?;
                mark("cu st.floor_first");
                add(take.home, cushion_share(rest, first, &shares))?
            }
        };
        let split = floor_math::stair::StairSplit { to_pol, to_floor: take.collected - to_pol, ..shares };
        (pol_quote, split)
    };


    mark("cu st.split");
    let (open_seed, open_floor) = open_cushion(m, s, &st, supply, &params)?;
    mark("cu st.open_cushion");
    let mut inflow = 0;
    if split.to_floor > 0 {
        let kept = intake(s, split.to_floor, &mut m.events)?;
        inflow = kept;
        s.vault_eth = add(s.vault_eth, kept)?;
        s.unpaid_skim = add(s.unpaid_skim, kept)?;
        s.ladder_proceeds_eth = add(s.ladder_proceeds_eth, kept)?;
    }
    let cushion_before = (st.cushion_lower, st.cushion_upper, st.cushion_liq.get());



    let sentinel = s.floor_tick;
    if let Some(f) = open_floor {
        s.floor_tick = f;
    }
    let placed = refollow_cushion(m, s, &mut st, cushion_held, split.to_pol + open_seed, supply, &mut take);
    s.floor_tick = sentinel;
    placed?;
    mark("cu st.refollow");
    let relist_before = st.relist_liq.get();

    let mut relisted = 0;
    take.reserved = 0;
    if take.room(1) {
        relisted = place_relist(m, s, &mut st, take.pol_tokens, (take.basis_tokens, take.basis_cost))?;
        take.modifies += (relist_before == 0 && st.relist_liq.get() != 0) as u8;
    } else if take.pol_tokens != 0 {



        hold_with_basis(s, take.pol_tokens, (take.basis_tokens, take.basis_cost))?;
    }

    mark("cu st.relist");


    let standing = standing_tokens_with(m.market.sqrt_price_x64, &st, &mut sqrts)?;
    let counted = counted_tokens_with(m.market.sqrt_price_x64, s, &st, take.books.as_deref(), &mut sqrts)?;
    s.ladder_tokens_remaining = owners::ladder_less_counted(standing, counted)?;
    mark("cu st.standing2");
    s.ladder_upper = frontier(&st);
    if take.more_left {
        s.needs_ratchet = 1;
    }
    st.collected_total = st.collected_total.saturating_add(take.collected);
    st.to_pol_total = st.to_pol_total.saturating_add(split.to_pol);
    st.to_floor_total = st.to_floor_total.saturating_add(split.to_floor);


    let moved = cushion_before != (st.cushion_lower, st.cushion_upper, st.cushion_liq.get());
    if take.collected != 0 || take.ops != 0 || take.listed != 0 || relisted != 0 || moved {
        m.events.push(MechEvent::StairRatcheted(StairRatcheted {
            pool: s.amm_pool,
            collected: take.collected,
            to_pol: split.to_pol,
            to_floor: split.to_floor,
            cushion: st.cushion_quote,
            pol_quote,
            short_floor: split.short_floor,
            short_pol: split.short_pol,
            first_open: st.first_open,
            relisted,
        }));
    }
    store(&acct, &st)?;
    if let Some(o) = take.books.as_deref() {
        if take.owner_quote != 0 || take.owner_fee != 0 || o.owner_sold != owner_sold0 || owed_now.is_some() {
            let (due, paid) = owed_now.unwrap_or((0, 0));
            m.events.push(MechEvent::StairOwnerBooked(crate::events::StairOwnerBooked {
                pool: s.amm_pool,
                owner_sold: o.owner_sold - owner_sold0,
                owner_quote: take.owner_quote,
                due,
                paid,
                forfeited: due - paid,
                fees_to_floor: take.owner_fee,
                owner_owed: s.owner_owed,
            }));
        }
        owners::write(&acct, o)?;
    }
    if let Some(t) = take.team.as_deref() {
        if t.dirty {
            crate::state::slots::write_parts(&m.cpi.floor_pool, &t.parts)?;
        }
    }
    let cushion = if st.cushion_liq.get() != 0 { Some((st.cushion_lower, st.cushion_upper, st.cushion_liq.get())) } else { None };


    let hold = match cushion {
        Some((_, upper, _)) if parked => {
            Some(math::ceil_to_spacing(upper + math::band_ticks(s)?, s.tick_spacing)?)
        }
        _ => None,
    };



    let hold = match (hold, clear_of_asks(s, &st)?) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (a, b) => a.or(b),
    };
    Ok(StairOutcome { cushion, hold, inflow, parked, cushion_ticks: width })
}

///
///
///
///
///
///
///
///
#[inline(never)]
fn book_payee(m: &Mech, s: &mut FloorPool, st: &StairState, supply: u64, take: &mut Take, held: (u64, u64)) -> Result<Option<(u64, u64)>> {
    let Some(bps) = take.books.as_ref().map(|o| o.payee_bps) else { return Ok(None) };


    let (out_quote, slot_total) = take.team.as_deref().map_or((0, 0), |t| (t.out_quote, t.slot_total()));
    let due = add(floor_math::owners::payee_due(take.owner_quote.saturating_sub(out_quote), bps), slot_total)?;


    let collected = sub(take.collected, take.home)?;
    let pass = payee_pass(take.collect_deferred, due, take.pending_in, collected, || {
        let p = m.market.sqrt_price_x64;
        let liq = st.cushion_liq.get();
        let range = if liq != 0 { Some((st.cushion_lower, st.cushion_upper)) } else { None };
        let base = s.ladder_tokens_remaining;
        s.ladder_tokens_remaining = add(base, held.1)?;
        let pbps = s.platform_bps as u64;
        let keep = {
            let state = math::state(s);
            let need = map(floor_math::stair::BackingNeed::new(state, supply, s.floor_tick, range).and_then(|n| n.with_quote(p, held.0)))?;
            let need = need.min(i128::MAX as u128) as i128;
            map(floor_math::stair::floor_first_least(collected, |k| {
                let have = floor_math::stair::backing_have(state, k) + (k - floor_math::fees::part(k, pbps)?) as u128;
                Ok(have.min(i128::MAX as u128) as i128 - need)
            }))?
        };
        s.ladder_tokens_remaining = base;
        Ok(keep)
    })?;

    if let Some(t) = take.team.as_deref_mut() {
        t.book_pass(&pass, due, take.pending_in, take.collect_deferred);
    }

    take.collected = sub(sub(take.collected, pass.aside)?, pass.paid)?;
    s.set_owner_pending(add(s.owner_pending(), pass.aside)?);
    s.owner_owed = add(s.owner_owed, pass.paid)?;
    if let Some(o) = take.books.as_deref_mut() {
        o.owed_total = o.owed_total.saturating_add(pass.paid);
        o.forfeited = o.forfeited.saturating_add(pass.forfeited);
    }
    Ok((pass.decided != 0).then_some((pass.decided, pass.paid)))
}

///
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct PayeePass {
///
    aside: u64,
///
    paid: u64,
///
    forfeited: u64,
///
    decided: u64,
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
fn payee_pass(deferred: bool, due: u64, pending_in: u64, collected: u64, keep: impl FnOnce() -> Result<u64>) -> Result<PayeePass> {
    if deferred {
        let aside = due.min(collected);
        let forfeited = due - aside;
        return Ok(PayeePass { aside, forfeited, decided: forfeited, ..PayeePass::default() });
    }
    let due = add(due, pending_in)?;
    if due == 0 {
        return Ok(PayeePass::default());
    }
    let (paid, forfeited) = floor_math::owners::payee_after_floor_first(due, collected, keep()?);
    Ok(PayeePass { paid, forfeited, decided: due, aside: 0 })
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
#[inline(never)]
pub fn trim_cushion(m: &mut Mech, s: &mut FloorPool, floor0: i32) -> Result<()> {
    if s.stair_ready == 0 {
        return Ok(());
    }
    let (acct, mut st) = load(&m.accts.tick_arrays, m.cpi.floor_pool.key)?;
    let liq = st.cushion_liq.get();
    let dear = math::dear_edge(s)?;
    if liq == 0 || st.cushion_upper <= dear {
        return Ok(());
    }
    let (lo, hi) = (st.cushion_lower, st.cushion_upper);







    let set = &m.accts.tick_arrays;
    let pool = m.cpi.pool.key;
    let has = |t: i32| has_tick_array_at(set, pool, t, m.spacing);
    let range = if m.market.tick >= lo {
        None
    } else if dear > lo {
        Some((lo, dear))
    } else {
        let width = st.params().cushion_ticks;
        let home = map(floor_math::stair::cushion_home_top(m.market.tick, s.floor_tick, s.tick_spacing, width))?;
        map(floor_math::stair::cushion_range_above_w(m.market.tick, home, s.tick_spacing, s.floor_tick, width))?
    };

    let missing = range.is_some_and(|(a, b)| !has(a) || !has(b));
    let range = range.filter(|_| !missing);
    let Some((nlo, nhi)) = range else {
        let hold = math::ceil_to_spacing(hi + math::band_ticks(s)?, s.tick_spacing)?;
        if s.floor_tick < hold {
            s.floor_tick = if hold < floor0 { hold } else { floor0 };



            if missing {
                s.needs_ratchet = 1;
            }
        }
        return Ok(());
    };

    let (q, b) = close_cushion(m, s, lo, hi, liq)?;
    to_wall(s, b)?;
    st.cushion_liq.set(0);
    st.cushion_quote = 0;
    let l = math::position_liquidity(m.market.sqrt_price_x64, nlo, nhi, q, 0)?;
    if l > 0 {
        st.cushion_quote = open_quote_only(m, s, SALT_CUSHION, nlo, nhi, l)?;
        st.cushion_lower = nlo;
        st.cushion_upper = nhi;
        st.cushion_liq.set(l);
    }
    if nlo != lo {



        s.needs_ratchet = 1;
    }
    store(&acct, &st)
}

fn has_tick_array_at(set: &[AccountInfo], pool: &Pubkey, tick: i32, spacing: u16) -> bool {
    amm_cpi::has_tick_array(set, pool, tick, spacing)
}

///
///
///
#[inline(never)]
fn collect_steps(m: &mut Mech, s: &mut FloorPool, st: &mut StairState, take: &mut Take, busy: bool) -> Result<()> {
    let spacing = s.tick_spacing;
    let (line, clamped) = ask_line(m, s)?;
    loop {
        if st.first_open < st.placed {
            let i = st.first_open as usize;
            let (lower, upper, liq) = (st.step_lower[i], st.step_upper[i], st.step_liq[i].get());
            if liq == 0 {
                st.first_open += 1;
                continue;
            }
            if m.market.tick >= upper {
                return Ok(());
            }
            if take.ops == STAIR_STEPS_PER_RATCHET || busy || !take.room(2) {
                take.more_left = true;
                take.collect_deferred = true;
                return Ok(());
            }
            take.ops += 1;


            let before = if take.owner_on(i) != 0 { owners::token_only(lower, upper, liq)? } else { 0 };
            let (q, b, fq, fb) = close_split(m, s, step_salt(i as u8), lower, upper, liq)?;
            take.modifies += 1;
            take.collected = add(take.collected, q)?;
            take.book_close(i, before, b - fb, q - fq, fq)?;
            match map(floor_math::stair::unsold_range(lower, upper, line, spacing))? {
                Some((nl, nu)) => {
                    let nliq = math::position_liquidity(m.market.sqrt_price_x64, nl, nu, 0, b)?;
                    let used = if nliq > 0 { open_token_only(m, s, step_salt(i as u8), nl, nu, nliq)? } else { 0 };
                    take.modifies += (nliq > 0) as u8;

                    let kept = if nliq == 0 { take.hold_step(i, sub(b, used)?) } else { 0 };
                    to_owner_held(s, kept)?;
                    to_wall(s, sub(sub(b, used)?, kept)?)?;
                    st.step_upper[i] = nu;
                    st.step_liq[i].set(nliq);
                    if nliq == 0 {
                        st.first_open += 1;
                        continue;
                    }
                    take.clamp_step(i, used);
                    return Ok(());
                }
                None => {


                    let kept = take.hold_step(i, b);
                    to_owner_held(s, kept)?;
                    let b = b - kept;
                    if clamped {
                        take.pol_tokens = add(take.pol_tokens, b)?;
                        take.reserved = take.reserved.max((b != 0) as u8);
                    } else {
                        to_wall(s, b)?;
                    }
                    st.step_liq[i].set(0);
                    st.first_open += 1;
                }
            }
        } else if st.placed < st.steps {

            if st.reserve_liq.get() == 0 {

                st.placed = st.steps;
                st.first_open = st.steps;
                continue;
            }
            if m.market.tick >= st.reserve_upper {
                return Ok(());
            }
            if take.ops == STAIR_STEPS_PER_RATCHET || busy || !take.room(SPLIT_MODIFIES) {
                take.more_left = true;
                take.collect_deferred = true;
                return Ok(());
            }
            take.ops += 1;
            take.modifies += SPLIT_MODIFIES;
            split_reserve(m, s, st, take)?;
        } else {
            break;
        }
    }

    let liq = st.tail_liq.get();
    if liq == 0 || m.market.tick >= st.tail_upper {
        return Ok(());
    }
    if take.ops == STAIR_STEPS_PER_RATCHET || busy || !take.room(2) {
        take.more_left = true;
        take.collect_deferred = true;
        return Ok(());
    }
    take.ops += 1;
    take.modifies += 2;
    let (lower, upper) = (st.tail_lower, st.tail_upper);
    let (q, b) = close_position(m, s, SALT_STAIR_TAIL, lower, upper, liq)?;
    take.collected = add(take.collected, q)?;
    match map(floor_math::stair::unsold_range(lower, upper, line, spacing))? {
        Some((nl, nu)) => {
            let nliq = math::position_liquidity(m.market.sqrt_price_x64, nl, nu, 0, b)?;
            let used = if nliq > 0 { open_token_only(m, s, SALT_STAIR_TAIL, nl, nu, nliq)? } else { 0 };
            to_wall(s, sub(b, used)?)?;
            st.tail_upper = nu;
            st.tail_liq.set(nliq);
        }
        None => {
            if clamped {
                take.pol_tokens = add(take.pol_tokens, b)?;
                take.reserved = take.reserved.max((b != 0) as u8);
            } else {
                to_wall(s, b)?;
            }
            st.tail_liq.set(0);
        }
    }
    Ok(())
}

///
///
///
///
///
///
#[inline(never)]
fn split_reserve(m: &mut Mech, s: &mut FloorPool, st: &mut StairState, take: &mut Take) -> Result<()> {
    let (n, j) = (st.steps, st.placed);
    let (rl, ru, rliq) = (st.reserve_lower, st.reserve_upper, st.reserve_liq.get());

    let before = if take.owner_in_reserve(j, n) != 0 { owners::token_only(rl, ru, rliq)? } else { 0 };
    let (q, b, fq, fb) = close_split(m, s, SALT_STAIR_RESERVE, rl, ru, rliq)?;
    take.collected = add(take.collected, q)?;
    st.reserve_liq.set(0);
    let planned = planned_of(s, st.stair_total, n, st.growth_bps)?;
    if let Some(o) = take.books.as_deref_mut() {

        let was = Box::new(o.tokens);
        let (oq, of) = owners::book_reserve(o, before, b - fb, q - fq, fq, &planned, j, n);

        if let Some(t) = take.team.as_deref_mut() {
            reserve_slot_asks(t, &was, &o.tokens, oq, o.payee_bps, j, n);
        }
        take.collected = sub(take.collected, of)?;
        take.owner_quote = add(take.owner_quote, oq)?;
        take.owner_fee = add(take.owner_fee, of)?;
    }



    let raw = planned;
    let prot = take.books.as_deref().map_or(0, |o| o.reserve_fed()).saturating_add(fb).min(b);

    let planned = owners::planned_less_cut(&planned, take.books.as_deref());
    let (to_step, rest) = map(floor_math::stair::reserve_split(b - prot, &planned, j, n))?;
    let (line, clamped) = ask_line(m, s)?;
    let not_taken = place_own_at(m, s, st, j, to_step, line)?;


    let kept = if to_step != 0 && not_taken == to_step {
        take.hold_step(j as usize, not_taken)
    } else {
        take.clamp_step(j as usize, sub(to_step, not_taken)?);
        0
    };
    to_owner_held(s, kept)?;
    let not_taken = not_taken - kept;
    let prot = add(prot, not_taken)?;
    let mut rest = add(add(rest, not_taken)?, prot - not_taken)?;
    take.set_reserve_fed(0);
    st.placed = j + 1;
    if st.placed >= n {

        return hold_or_relist(s, take, rest, clamped);
    }
    if !floor_math::stair::keeps_reserve(st.placed, n) {

        let last = n - 1;
        let had = rest;
        rest = place_own_at(m, s, st, last, rest, line)?;
        if had != 0 && rest == had {
            let kept = take.hold_step(last as usize, rest);
            to_owner_held(s, kept)?;
            rest -= kept;
        } else {
            take.clamp_step(last as usize, sub(had, rest)?);
        }
        st.placed = n;
        return hold_or_relist(s, take, rest, clamped);
    }
    let (lo, hi) = (st.step_lower[n as usize - 1], st.step_upper[st.placed as usize]);
    match map(floor_math::stair::unsold_range(lo, hi, line, s.tick_spacing))? {
        Some((nl, nu)) if rest > 0 => {
            let liq = math::position_liquidity(m.market.sqrt_price_x64, nl, nu, 0, rest)?;
            let used = if liq > 0 { open_token_only(m, s, SALT_STAIR_RESERVE, nl, nu, liq)? } else { 0 };
            st.reserve_lower = nl;
            st.reserve_upper = nu;
            st.reserve_liq.set(liq);
            let mut kept = 0;
            if liq == 0 {

                kept = take.hold_reserve(st.placed, n, sub(rest, used)?);
                to_owner_held(s, kept)?;
            } else {


                take.reopened_reserve(&raw, st.placed, n, prot, sub(rest, used)?, used);
            }
            to_wall(s, sub(sub(rest, used)?, kept)?)
        }
        _ => {

            let kept = take.hold_reserve(st.placed, n, rest);
            to_owner_held(s, kept)?;
            hold_or_relist(s, take, rest - kept, clamped)
        }
    }
}

///
///
fn hold_or_relist(s: &mut FloorPool, take: &mut Take, tokens: u64, clamped: bool) -> Result<()> {
    if clamped {
        take.pol_tokens = add(take.pol_tokens, tokens)?;
        take.reserved = take.reserved.max((tokens != 0) as u8);
        Ok(())
    } else {
        to_wall(s, tokens)
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
fn feed_vault_tokens(m: &mut Mech, s: &mut FloorPool, st: &mut StairState, take: &mut Take, cushion_bought: bool) -> Result<()> {
    let bought = s.inventory_tokens;
    if bought == 0 {
        return Ok(());
    }




    let pol = if cushion_bought { 2 + (take.reserved == 0) as u8 } else { 1 };
    if !take.room(1 + pol) {
        take.more_left = true;
        return Ok(());
    }
    let cost = s.inventory_cost_eth;
    let cap = map(floor_math::stair::vault_relist_cap_tick(bought, cost))?;
    let mut own = [None; MAX_STAIR_STEPS as usize];
    for i in st.first_open..st.placed {
        if st.step_liq[i as usize].get() != 0 {
            own[i as usize] = Some(st.step_upper[i as usize]);
        }
    }
    let reserve = (st.placed < st.steps && st.reserve_liq.get() != 0).then_some(st.reserve_upper);
    let tail = (st.tail_liq.get() != 0).then_some(st.tail_upper);
    let (salt, lo, hi, step) = match floor_math::stair::vault_list_target(cap, st.first_open, st.placed, &own, reserve, tail) {
        VaultListTarget::Step(i) => (step_salt(i), st.step_lower[i as usize], st.step_upper[i as usize], i),
        VaultListTarget::Reserve => (SALT_STAIR_RESERVE, st.reserve_lower, st.reserve_upper, st.placed),
        VaultListTarget::Tail => (SALT_STAIR_TAIL, st.tail_lower, st.tail_upper, st.steps),
        VaultListTarget::Hold => return Ok(()),
    };
    if m.market.tick < hi {
        take.more_left = true;
        return Ok(());
    }
    let tokens = add(bought, s.wall_held_tokens)?;
    let dl = math::position_liquidity(m.market.sqrt_price_x64, lo, hi, 0, tokens)?;
    if dl == 0 {
        return Ok(());
    }
    let free = take_held(s)?;


    let liq_before = match salt {
        SALT_STAIR_RESERVE => st.reserve_liq.get(),
        SALT_STAIR_TAIL => st.tail_liq.get(),
        x => st.step_liq[(x - SALT_STAIR_STEP0) as usize].get(),
    };
    let owner_on = match salt {
        SALT_STAIR_RESERVE => take.owner_in_reserve(st.placed, st.steps),
        SALT_STAIR_TAIL => 0,
        x => take.owner_on((x - SALT_STAIR_STEP0) as usize),
    };
    let before = if owner_on != 0 { owners::token_only(lo, hi, liq_before)? } else { 0 };
    let r = m.open(s, salt, lo, hi, dl)?;
    take.modifies += 1;
    let (q, used) = amm_cpi::used(&r)?;
    require!(q == 0 && used <= tokens, HookError::StairNotTokenOnly);
    let (fee_q, fee_b) = amm_cpi::fees(&r);
    let owner_fee = match take.books.as_deref_mut() {
        Some(o) if owner_on != 0 => owners::fee_share(o, fee_q, before, owner_on),
        _ => 0,
    };
    take.owner_fee = add(take.owner_fee, owner_fee)?;
    take.collected = add(take.collected, sub(fee_q, owner_fee)?)?;

    to_wall(s, add(sub(tokens, used)?, fee_b)?)?;
    let grow = |l: &mut crate::state::pod::U128| -> Result<()> {
        let v = l.get().checked_add(dl).ok_or(HookError::MathOverflow)?;
        l.set(v);
        Ok(())
    };
    match salt {
        SALT_STAIR_RESERVE => grow(&mut st.reserve_liq)?,
        SALT_STAIR_TAIL => grow(&mut st.tail_liq)?,
        x => grow(&mut st.step_liq[(x - SALT_STAIR_STEP0) as usize])?,
    }
    if salt == SALT_STAIR_RESERVE {


        if let Some(o) = take.books.as_deref_mut() {
            o.set_reserve_fed(o.reserve_fed().saturating_add(used));
        }
    }
    s.inventory_tokens = 0;
    s.inventory_cost_eth = 0;
    st.vault_listed_tokens = st.vault_listed_tokens.saturating_add(used);
    st.vault_listed_cost = st.vault_listed_cost.saturating_add(cost);
    st.vault_fed_bought = st.vault_fed_bought.saturating_add(bought);
    take.listed = used;
    m.events.push(MechEvent::VaultTokensFed(VaultTokensFed {
        pool: s.amm_pool,
        step,
        into_reserve: salt == SALT_STAIR_RESERVE,
        tokens: used,
        bought,
        free,
        cost,
        cap_tick: cap.unwrap_or(i32::MAX),
    }));
    Ok(())
}

///
///
#[inline(never)]
fn pol_side_busy(m: &Mech, st: &StairState, parked: bool) -> Result<bool> {
    let c_liq = st.cushion_liq.get();
    let cushion_tokens = !parked
        && c_liq != 0
        && math::position_amounts(m.market.sqrt_price_x64, st.cushion_lower, st.cushion_upper, c_liq)?.1 != 0;
    let relist_bought = st.relist_liq.get() != 0 && m.market.tick < st.relist_upper;
    Ok(cushion_tokens || relist_bought)
}

///
///
///
///
///
///
///
///
fn cushion_takes_quote(take: &Take, st: &StairState, held: (u64, u64), tick: i32) -> bool {
    let liq = st.cushion_liq.get();
    take.room(2) || (take.room(1) && (liq == 0 || (held.1 == 0 && tick < st.cushion_lower)))
}

///
///
///
#[inline(never)]
fn collect_relist(m: &mut Mech, s: &mut FloorPool, st: &mut StairState, take: &mut Take, parked: bool) -> Result<(u64, u64)> {
    let p = m.market.sqrt_price_x64;
    let c_liq = st.cushion_liq.get();
    let cushion = if c_liq != 0 { math::position_amounts(p, st.cushion_lower, st.cushion_upper, c_liq)? } else { (0, 0) };
    let liq = st.relist_liq.get();
    if liq != 0 {
        let bought = m.market.tick < st.relist_upper;
        let due = bought || (cushion.1 != 0 && !parked);
        if due && !take.room(2) {

            take.more_left = true;
            take.collect_deferred |= bought;
        } else if due {
            let (q, b) = close_position(m, s, SALT_POL_RELIST, st.relist_lower, st.relist_upper, liq)?;
            take.modifies += 1;
            take.reserved = 1;
            take.collected = add(take.collected, q)?;
            take.relist_q = add(take.relist_q, q)?;
            take.pol_tokens = add(take.pol_tokens, b)?;
            st.relist_liq.set(0);


            if st.relist_cost_tick != 0 && b != 0 {
                take.basis_tokens = add(take.basis_tokens, b)?;
                take.basis_cost = take.basis_cost.saturating_add(math_cost(b, st.relist_cost_tick)?);
            }
            st.relist_cost_tick = 0;
        }
    }
    Ok(cushion)
}

///
///
///
///
#[inline(never)]
fn refollow_cushion(
    m: &mut Mech,
    s: &mut FloorPool,
    st: &mut StairState,
    held: (u64, u64),
    to_pol: u64,
    supply: u64,
    take: &mut Take,
) -> Result<()> {
    if st.cushion_liq.get() == 0 && to_pol == 0 {
        return Ok(());
    }



    let width = st.params().cushion_ticks;
    let target = cushion_target(m, s, st, width)?;
    let liq = st.cushion_liq.get();
    if target.is_none() && liq != 0 {
        return Ok(());
    }








    let stood = st.cushion_lower != st.cushion_upper;
    let range = match target {
        Some((tl, _)) if stood && tl < st.cushion_lower => climb(m, s, st, held, to_pol, supply, tl)?,
        _ => target,
    };
    let same = liq != 0 && range == Some((st.cushion_lower, st.cushion_upper));
    if same && to_pol == 0 && held.1 == 0 {
        return Ok(());
    }



    let no_room_to_move = !take.room(2);
    if liq != 0 && held.1 == 0 && to_pol > 0 && (same || no_room_to_move) && m.market.tick < st.cushion_lower {
        let (lo, hi) = (st.cushion_lower, st.cushion_upper);
        let dl = math::position_liquidity(m.market.sqrt_price_x64, lo, hi, to_pol, 0)?;
        if dl > 0 {
            let r = m.open(s, SALT_CUSHION, lo, hi, dl)?;
            take.modifies += 1;
            let (q, b) = amm_cpi::used(&r)?;
            require!(b == 0, HookError::InvariantViolated);



            to_wall(s, amm_cpi::fees(&r).1)?;
            st.cushion_liq.set(liq.checked_add(dl).ok_or(HookError::MathOverflow)?);
            st.cushion_quote = add(st.cushion_quote, q)?;
            if !same {
                take.more_left = true;
            }
            return Ok(());
        }
    }
    if liq != 0 && held.1 == 0 && to_pol == 0 && no_room_to_move {
        take.more_left = true;
        return Ok(());
    }




    let relist = (liq != 0 && held.1 != 0 && take.reserved == 0) as u8;
    if !take.room((liq != 0) as u8 + 1 + relist) {
        take.more_left = true;
        return Ok(());
    }
    take.reserved += relist;
    let mut quote = to_pol;
    if liq != 0 {
        take.modifies += 1;

        let (q, b) = close_cushion(m, s, st.cushion_lower, st.cushion_upper, liq)?;
        quote = add(quote, q)?;
        take.pol_tokens = add(take.pol_tokens, b)?;
        if b != 0 {


            take.basis_tokens = add(take.basis_tokens, b)?;
            take.basis_cost = take.basis_cost.saturating_add(st.cushion_quote.saturating_sub(q) as u128);
        }
        st.cushion_liq.set(0);
    }
    st.cushion_quote = 0;
    if quote == 0 {
        return Ok(());
    }
    let p = m.market.sqrt_price_x64;
    let l = match range {
        Some((lo, hi)) => math::position_liquidity(p, lo, hi, quote, 0)?,
        None => 0,
    };
    match range {
        Some((lo, hi)) if l > 0 => {
            let used = open_quote_only(m, s, SALT_CUSHION, lo, hi, l)?;
            take.modifies += 1;

            st.cushion_lower = lo;
            st.cushion_upper = hi;
            st.cushion_liq.set(l);
            st.cushion_quote = used;
        }
        _ => {


            let kept = intake(s, quote, &mut m.events)?;
            s.vault_eth = add(s.vault_eth, kept)?;
        }
    }
    Ok(())
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
#[inline(never)]
fn open_cushion(
    m: &Mech,
    s: &mut FloorPool,
    st: &StairState,
    supply: u64,
    params: &floor_math::stair::StairParams,
) -> Result<(u64, Option<i32>)> {
    if s.stair_kind != floor_types::constants::STAIR_KIND_GENESIS
        || s.vault_eth == 0
        || st.cushion_liq.get() != 0
        || st.collected_total != 0
        || s.floor_tick != math::ceil_to_spacing(floor_types::constants::MAX_TICK, s.tick_spacing)?
    {
        return Ok((0, None));
    }
    let p = m.market.sqrt_price_x64;
    let classic = math::position_amounts(p, s.seed_lower, s.seed_upper, s.seed_liq.get())?.0;

    let shares = map(floor_math::stair::stair_split_for(s.vault_eth, p, 0, classic, supply, params))?;
    s.vault_eth = sub(s.vault_eth, shares.to_pol)?;





    let floor = open_floor(m, s, st, supply, shares.to_pol)?;
    Ok((shares.to_pol, floor))
}

///
#[inline(never)]
fn open_floor(m: &Mech, s: &FloorPool, st: &StairState, supply: u64, quote: u64) -> Result<Option<i32>> {
    if quote == 0 {
        return Ok(None);
    }
    let sp = s.tick_spacing as i32;
    let p = m.market.sqrt_price_x64;
    let tick = m.market.tick;
    let width = st.params().cushion_ticks;
    let band = math::band_ticks(s)?;
    let top = math::ceil_to_spacing(tick, s.tick_spacing)?;


    let dearest = (top + sp + band).max(clear_of_asks(s, st)?.unwrap_or(i32::MIN));
    let cheapest = top + math::ceil_to_spacing(width, s.tick_spacing)? + band;
    let state = math::state(s);
    let have = floor_math::stair::backing_have(state, 0);

    Ok(floor_math::stair::open_fixpoint_floor(
        dearest,
        cheapest,
        sp,
        |f| floor_math::stair::cushion_range_above_w(tick, tick, sp, f, width),
        |f, r| {
            let need = floor_math::stair::BackingNeed::new(state, supply, f, Some(r))?.with_quote(p, quote)?;
            Ok(have >= need)
        },
    )
    .ok()
    .flatten())
}

///
///
///
#[inline(never)]
fn cushion_arm(m: &Mech, s: &FloorPool, st: &StairState, take: &Take, held: (u64, u64), width: i32) -> Result<Option<(i32, i32)>> {
    Ok(match cushion_target(m, s, st, width)? {
        Some(t) if cushion_takes_quote(take, st, held, m.market.tick) => Some(t),
        _ => None,
    })
}

///
///
///
fn cushion_share(rest: u64, k: u64, shares: &floor_math::stair::StairSplit) -> u64 {
    pol_share(rest - k, shares)
}

///
fn pol_share(rest: u64, shares: &floor_math::stair::StairSplit) -> u64 {
    let both = shares.short_floor + shares.short_pol;
    if both == 0 {
        0
    } else {
        (rest as u128 * shares.short_pol / both) as u64
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
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn floor_first_of(
    m: &Mech,
    s: &mut FloorPool,
    st: &StairState,
    supply: u64,
    collected: u64,
    held: (u64, u64),
    shares: &floor_math::stair::StairSplit,
    room_to_move: bool,
    target: (i32, i32),
) -> Result<u64> {
    let p = m.market.sqrt_price_x64;
    let liq = st.cushion_liq.get();



    let stays = liq != 0 && held.1 == 0 && !room_to_move && m.market.tick < st.cushion_lower;






    let range = if stays {
        Some((st.cushion_lower, st.cushion_upper))
    } else {
        let spot = map(floor_math::stair::cushion_range_above_w(m.market.tick, m.market.tick, s.tick_spacing, s.floor_tick, st.params().cushion_ticks))?;
        Some(spot.unwrap_or(target))
    };
    let base = s.ladder_tokens_remaining;
    s.ladder_tokens_remaining = add(base, held.1)?;
    let bps = s.platform_bps as u64;
    let k = {
        let state = math::state(s);


        let need_at = map(floor_math::stair::BackingNeed::new(state, supply, s.floor_tick, range))?;
        map(floor_math::stair::floor_first_least(collected, |k| {
            let to_pol = cushion_share(collected, k, shares);
            let g = collected - to_pol;
            let have = floor_math::stair::backing_have(state, g) + (g - floor_math::fees::part(g, bps)?) as u128;
            let quote = held.0.checked_add(to_pol).ok_or(floor_math::FloorMathError::Overflow)?;
            let need = need_at.with_quote(p, quote)?;
            Ok(have as i128 - need.min(i128::MAX as u128) as i128)
        }))?
    };
    s.ladder_tokens_remaining = base;
    Ok(k)
}

///
///
///
///
///
#[inline(never)]
fn climb(m: &Mech, s: &mut FloorPool, st: &StairState, held: (u64, u64), to_pol: u64, supply: u64, target_lower: i32) -> Result<Option<(i32, i32)>> {
    let spacing = s.tick_spacing;
    let p = m.market.sqrt_price_x64;
    let quote = add(held.0, to_pol)?;
    let width = st.params().cushion_ticks;
    let dear = math::dear_edge(s)?;
    let base = s.ladder_tokens_remaining;
    s.ladder_tokens_remaining = add(base, held.1)?;
    let lower = {
        let state = math::state(s);
        let have = floor_math::stair::backing_have(state, 0);

        let need_at = map(floor_math::stair::BackingNeed::new(state, supply, s.floor_tick, None))?;

        let aim = need_at.climb_aim(have, quote, dear, width, spacing);
        map(floor_math::stair::cushion_climb_aimed(target_lower, st.cushion_lower, spacing, aim, |l| {
            let Some((lo, hi)) = floor_math::stair::cushion_clip(floor_math::stair::cushion_range_from_w(l, spacing, width)?, dear) else {
                return Ok(false);
            };
            Ok(need_at.with_range(p, lo, hi, quote)? <= have)
        }))?
    };
    s.ladder_tokens_remaining = base;
    let stay = Some((st.cushion_lower, st.cushion_upper));
    if lower == st.cushion_lower {
        return Ok(stay);
    }
    let range = floor_math::stair::cushion_clip(map(floor_math::stair::cushion_range_from_w(lower, spacing, width))?, dear);


    if let Some((lo, hi)) = range {
        if lower != target_lower {
            let set = &m.accts.tick_arrays;
            let pool = m.cpi.pool.key;
            if !amm_cpi::has_tick_array(set, pool, lo, m.spacing) || !amm_cpi::has_tick_array(set, pool, hi, m.spacing) {
                s.needs_ratchet = 1;
                return Ok(stay);
            }
        }
    }
    Ok(range)
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
#[inline(never)]
fn cushion_target(m: &Mech, s: &FloorPool, st: &StairState, width: i32) -> Result<Option<(i32, i32)>> {
    let settled = settled_tick(m)?;
    let standing = (st.cushion_liq.get() != 0).then_some(st.cushion_lower);
    let settled = crate::mech::netflow::cap_target(m, settled, standing)?;
    let target = map(floor_math::stair::cushion_range_above_w(m.market.tick, settled, s.tick_spacing, s.floor_tick, width))?;
    if target.is_some() {
        return Ok(target);
    }
    let home = map(floor_math::stair::cushion_home_top(m.market.tick, s.floor_tick, s.tick_spacing, width))?;
    map(floor_math::stair::cushion_range_above_w(m.market.tick, home, s.tick_spacing, s.floor_tick, width))
}

///
///
///
///
#[inline(never)]
pub(crate) fn settled_tick(m: &Mech) -> Result<i32> {
    let data = m.cpi.floor_pool.try_borrow_data()?;
    let o = crate::state::twap::TwapOracle::from_account(&data).ok_or(error!(HookError::TwapUnavailable))?;
    let now = crate::state::twap::now_secs()?;
    o.settled_tick(now, m.market.tick, floor_types::twap::CUSHION_TWAP_WINDOW_SECS)
        .map_err(|_| error!(HookError::TwapUnavailable))
}

///
///
///
///
///
///
///
#[inline(never)]
fn place_relist(m: &mut Mech, s: &mut FloorPool, st: &mut StairState, pol_tokens: u64, basis: (u64, u128)) -> Result<u64> {
    if pol_tokens == 0 {
        return Ok(0);
    }
    if st.relist_liq.get() != 0 {


        hold_with_basis(s, pol_tokens, basis)?;
        return Ok(0);
    }
    let spacing = s.tick_spacing;
    let p = m.market.sqrt_price_x64;
    let mut line = ask_line(m, s)?.0;
    let cost_tick = map(floor_math::stair::cushion_cost_tick(basis.0.min(pol_tokens), basis.1))?;
    if let Some(c) = cost_tick {
        let cline = map(floor_math::stair::relist_cost_line(c, spacing))?;
        if cline < line {
            let lower = cline.saturating_sub(math::ceil_to_spacing(floor_math::stair::doubling_ticks(1).unwrap_or(6_932), spacing)?);
            let pool = m.cpi.pool.key;
            if !amm_cpi::has_tick_array(&m.accts.tick_arrays, pool, cline, m.spacing)
                || !amm_cpi::has_tick_array(&m.accts.tick_arrays, pool, lower, m.spacing)
            {
                s.needs_ratchet = 1;
                hold_with_basis(s, pol_tokens, basis)?;
                return Ok(0);
            }
            line = cline;
        }
    }
    let range = map(floor_math::stair::relist_range(line, spacing))?;
    let l = match range {
        Some((lo, hi)) => math::position_liquidity(p, lo, hi, 0, pol_tokens)?,
        None => 0,
    };
    match range {
        Some((lo, hi)) if l > 0 => {
            let used = open_token_only(m, s, SALT_POL_RELIST, lo, hi, l)?;
            to_wall(s, sub(pol_tokens, used)?)?;
            st.relist_lower = lo;
            st.relist_upper = hi;
            st.relist_liq.set(l);

            st.relist_cost_tick = cost_tick.map_or(0, |c| if c == 0 { 1 } else { c });
            Ok(used)
        }
        _ => {
            hold_with_basis(s, pol_tokens, basis)?;
            Ok(0)
        }
    }
}

///
///
///
fn hold_with_basis(s: &mut FloorPool, tokens: u64, basis: (u64, u128)) -> Result<()> {
    let bought = basis.0.min(tokens);
    if bought != 0 && basis.1 != 0 {
        crate::mech::positions::vault_bought(s, bought, basis.1.min(u64::MAX as u128) as u64)?;
        to_wall(s, tokens - bought)
    } else {
        to_wall(s, tokens)
    }
}

///
fn math_cost(tokens: u64, tick: i32) -> Result<u128> {
    map(floor_math::position::quote_cost(tokens, tick))
}

///
///
pub fn standing_tokens(sqrt_price_x64: u128, st: &StairState) -> Result<u64> {
    standing_tokens_with(sqrt_price_x64, st, &mut SqrtCache::default())
}

///
///
///
#[derive(Default)]
pub struct SqrtCache(Vec<(i32, u128)>);

impl SqrtCache {
    fn at(&mut self, tick: i32) -> Result<u128> {
        if let Some(&(_, p)) = self.0.iter().find(|(t, _)| *t == tick) {
            return Ok(p);
        }
        let p = math::sqrt_price_at_tick(tick)?;
        self.0.push((tick, p));
        Ok(p)
    }
}

///
///
pub fn standing_tokens_with(sqrt_price_x64: u128, st: &StairState, cache: &mut SqrtCache) -> Result<u64> {
    let mut total = 0u64;
    let mut one = |lo: i32, hi: i32, liq: u128| -> Result<u64> {
        if liq == 0 {
            return Ok(0);
        }
        let (a, b) = (cache.at(lo)?, cache.at(hi)?);
        map(floor_math::base_at_sqrt(sqrt_price_x64, a, b, liq))
    };
    for i in st.first_open..st.steps.min(MAX_STAIR_STEPS) {
        let k = i as usize;
        total = add(total, one(st.step_lower[k], st.step_upper[k], st.step_liq[k].get())?)?;
    }
    for (lo, hi, liq) in [
        (st.reserve_lower, st.reserve_upper, st.reserve_liq.get()),
        (st.tail_lower, st.tail_upper, st.tail_liq.get()),
        (st.relist_lower, st.relist_upper, st.relist_liq.get()),
    ] {
        total = add(total, one(lo, hi, liq)?)?;
    }
    Ok(total)
}

///
///
///
///
///
pub fn counted_tokens_with(sqrt_price_x64: u128, s: &FloorPool, st: &StairState, o: Option<&floor_types::owners::StairOwners>, cache: &mut SqrtCache) -> Result<u64> {
    let Some(o) = o.filter(|o| o.counted != 0) else { return Ok(0) };
    let mut one = |lo: i32, hi: i32, liq: u128| -> Result<u64> {
        if liq == 0 {
            return Ok(0);
        }
        let (a, b) = (cache.at(lo)?, cache.at(hi)?);
        map(floor_math::base_at_sqrt(sqrt_price_x64, a, b, liq))
    };
    let n = st.steps.min(MAX_STAIR_STEPS);
    let mut total = 0u64;
    for i in st.first_open..st.placed.min(n) {
        let k = i as usize;
        total = add(total, o.tokens[k].min(one(st.step_lower[k], st.step_upper[k], st.step_liq[k].get())?))?;
    }
    if st.placed < n {






        let team = owners::in_reserve(o, st.placed, n);
        if team != 0 && st.reserve_liq.get() != 0 {
            let (rl, ru, rliq) = (st.reserve_lower, st.reserve_upper, st.reserve_liq.get());
            let held = one(rl, ru, rliq)?;
            let before = owners::token_only(rl, ru, rliq)?;
            let planned = planned_of(s, st.stair_total, n, st.growth_bps)?;
            let c = floor_math::owners::reserve_close(before, held, &planned, &o.cut, &o.tokens, o.reserve_fed(), st.placed, n);
            let left = team - c.owner_total.min(team);
            total = add(total, left.min(held - c.protocol_left.min(held)))?;
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck::Zeroable;

    #[test]
    fn the_salts_do_not_collide_with_the_hooks_others() {
        use floor_types::constants::*;
        let fixed = [SALT_BID, SALT_SEED, SALT_STAIR_TAIL, SALT_CUSHION, SALT_POL_RELIST, SALT_STAIR_RESERVE];
        let mut all: Vec<u64> = fixed.to_vec();
        all.extend((0..MAX_STAIR_STEPS).map(step_salt));
        let n = all.len();
        all.sort_unstable();
        all.dedup();
        assert_eq!(all.len(), n, "every hook position has its own salt");
        assert_eq!(step_salt(MAX_STAIR_STEPS - 1), 125);
    }

    #[test]
    fn the_frontier_is_the_first_unsold_steps_cheap_edge_or_the_reserves_or_the_relists() {
        let mut st = StairState::zeroed();
        st.steps = 3;
        st.step_upper = [0; 16];
        st.step_upper[0] = 90_000;
        st.step_upper[1] = 83_040;
        st.step_upper[2] = 76_080;
        st.tail_upper = 69_120;

        st.placed = 1;
        st.reserve_upper = 83_040;
        st.reserve_liq.set(5);
        assert_eq!(frontier(&st), 90_000, "step 0 unsold: its cheap edge");
        st.first_open = 1;
        assert_eq!(frontier(&st), 83_040, "step 0 sold, nothing split yet: the reserve's cheap edge (a buy into it is due a ratchet)");
        st.reserve_upper = 80_000;
        assert_eq!(frontier(&st), 80_000, "a reserve re-opened under a market inside it: its own edge");
        st.placed = 2;
        st.reserve_upper = 76_080;
        assert_eq!(frontier(&st), 83_040, "step 1 split off: its own edge");
        st.first_open = 2;
        assert_eq!(frontier(&st), 76_080);
        st.reserve_liq.set(0);
        st.placed = 3;
        st.first_open = 3;
        assert_eq!(frontier(&st), 69_120, "every step sold: the tail's edge");
        st.relist_upper = 70_000;
        assert_eq!(frontier(&st), 69_120, "no re-list standing: ignored");
        st.relist_liq.set(1);
        assert_eq!(frontier(&st), 70_000, "the re-list sits cheaper: a buy into it is due a ratchet too");

        st.placed = 1;
        st.first_open = 1;
        st.relist_liq.set(0);
        assert_eq!(frontier(&st), 69_120);
    }

    #[test]
    fn standing_tokens_count_every_unsold_position_at_the_market() {
        let mut st = StairState::zeroed();
        st.steps = 2;
        let p = math::sqrt_price_at_tick(95_000).unwrap();
        let (l0, u0, l1, u1) = (83_040, 90_000, 76_080, 83_040);
        st.step_lower[0] = l0;
        st.step_upper[0] = u0;
        st.step_lower[1] = l1;
        st.step_upper[1] = u1;
        let liq0 = math::position_liquidity(p, l0, u0, 0, 1_000_000).unwrap();
        let liq1 = math::position_liquidity(p, l1, u1, 0, 2_000_000).unwrap();
        st.step_liq[0].set(liq0);
        st.step_liq[1].set(liq1);
        let all = standing_tokens(p, &st).unwrap();
        assert!(all <= 3_000_000 && 3_000_000 - all < 100, "{all} (liquidity rounds down: dust)");

        let mid = math::sqrt_price_at_tick(86_520).unwrap();
        let some = standing_tokens(mid, &st).unwrap();
        assert!(some < all && some > 2_000_000 - 100, "{some}");
        st.first_open = 1;
        assert!(standing_tokens(p, &st).unwrap() <= 2_000_000);

        st.step_liq[1].set(0);
        st.reserve_lower = l1;
        st.reserve_upper = u1;
        st.reserve_liq.set(liq1);
        let with_reserve = standing_tokens(p, &st).unwrap();
        assert!(with_reserve <= 2_000_000 && 2_000_000 - with_reserve < 100, "{with_reserve}");
    }

///
///
///
    fn team_stair(protocol: [u64; 4], team: [u64; 4], fed: u64, placed: u8) -> (StairState, StairOwners) {
        let mut st = StairState::zeroed();
        st.steps = 4;
        st.placed = placed;

        st.growth_bps = 10_000;
        st.stair_total = (0..4).map(|k| protocol[k] + team[k]).sum::<u64>();
        let edges = [(83_040, 90_000), (76_080, 83_040), (69_120, 76_080), (62_160, 69_120)];
        let p = math::sqrt_price_at_tick(95_000).unwrap();
        for k in 0..placed as usize {
            st.step_lower[k] = edges[k].0;
            st.step_upper[k] = edges[k].1;
            st.step_liq[k].set(math::position_liquidity(p, edges[k].0, edges[k].1, 0, protocol[k] + team[k]).unwrap());
        }
        if placed < 4 {
            let r: u64 = (placed as usize..4).map(|k| protocol[k] + team[k]).sum::<u64>() + fed;
            st.reserve_lower = edges[3].0;
            st.reserve_upper = edges[placed as usize].1;
            st.reserve_liq.set(math::position_liquidity(p, st.reserve_lower, st.reserve_upper, 0, r).unwrap());
        }
        let mut o = StairOwners::zeroed();
        o.counted = 1;
        o.tokens[..4].copy_from_slice(&team);
        o.set_reserve_fed(fed);
        (st, o)
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
    #[test]
    fn ruling_a_every_team_token_on_the_staircase_is_circulating() {

        let (protocol, team) = ([100_000_000_000_000u64; 4], [25_000_000_000_000u64; 4]);
        let p = math::sqrt_price_at_tick(95_000).unwrap();
        for placed in [1u8, 2, 4] {
            let (st, o) = team_stair(protocol, team, 0, placed);
            let standing = standing_tokens(p, &st).unwrap();
            let counted = counted_tokens_with(p, &FloorPool::zeroed(), &st, Some(&o), &mut SqrtCache::default()).unwrap();
            let t: u64 = team.iter().sum();
            assert!(counted <= t && t - counted <= 1_000, "placed {placed}: every team token counts ({counted} of {t}; liquidity rounds down: dust)");
            let ladder = owners::ladder_less_counted(standing, counted).unwrap();
            let pr: u64 = protocol.iter().sum();
            assert!(ladder <= pr && pr - ladder <= 1_000, "placed {placed}: what is left off circulating is the protocol's ({ladder} of {pr})");

            let mut f = floor_math::state::FloorState::default();
            f.tick_spacing = 60;
            f.floor_tick = floor_math::ceil_to_spacing(floor_types::constants::MAX_TICK, 60).unwrap();
            f.min_sellable = floor_math::constants::MIN_SELLABLE;
            f.ladder_tokens_remaining = ladder;
            let with = floor_math::circulating_supply(&f, floor_types::constants::SUPPLY).unwrap();
            let mut without = f;
            without.ladder_tokens_remaining = standing;
            let wo = floor_math::circulating_supply(&without, floor_types::constants::SUPPLY).unwrap();
            assert_eq!(with - wo, counted, "placed {placed}: the team's tokens are in circulating supply");

            let mut strictly = false;
            for q in [1_000_000_000u128, 20_000_000_000, 400_000_000_000, 10_000_000_000_000] {
                let a = floor_math::solve_floor_tick_with(q, 0, &f, floor_types::constants::SUPPLY).unwrap();
                let b = floor_math::solve_floor_tick_with(q, 0, &without, floor_types::constants::SUPPLY).unwrap();
                assert!(a >= b, "placed {placed}, q {q}: counted {a} is dearer than uncounted {b}");
                strictly |= a > b;
            }
            assert!(strictly, "placed {placed}: counting the team moves the floor at some vault");
        }

        let (st, mut o) = team_stair(protocol, team, 0, 4);
        o.tokens[0] = protocol[0] + team[0] + 1_000;
        let capped = counted_tokens_with(p, &FloorPool::zeroed(), &st, Some(&o), &mut SqrtCache::default()).unwrap();
        assert!(capped <= standing_tokens(p, &st).unwrap(), "the cap: never more than the positions hold");

        o.counted = 0;
        assert_eq!(counted_tokens_with(p, &FloorPool::zeroed(), &st, Some(&o), &mut SqrtCache::default()).unwrap(), 0);
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
    #[test]
    fn ruling_a_the_reserves_team_share_is_counted_with_the_market_inside_it() {

        let (protocol, team, fed) = ([0u64, 0, 50_000_000_000, 50_000_000_000], [0u64, 0, 150_000_000_000, 150_000_000_000], 600_000_000_000u64);
        let (mut st, o) = team_stair(protocol, team, fed, 2);

        st.stair_total = 4 * (protocol[2] + team[2]);
        let before = standing_tokens(math::sqrt_price_at_tick(95_000).unwrap(), &st).unwrap();

        let inside = math::sqrt_price_at_tick((st.reserve_lower + st.reserve_upper) / 2).unwrap();
        let held = standing_tokens(inside, &st).unwrap();
        let sold = before - held;
        assert!(sold > 100_000_000_000 && sold < fed, "the scenario: the market sold more than the steps' protocol part, less than the fed record ({sold})");
        let planned = planned_of(&FloorPool::zeroed(), st.stair_total, 4, st.growth_bps).unwrap();
        assert_eq!(planned[2], protocol[2] + team[2], "the plan the reserve's close reads");
        let mut owner = [0u64; 16];
        for k in 2..4 {
            owner[k] = team[k];
        }
        let close = floor_math::owners::reserve_close(before, held, &planned, &[0; 16], &owner, fed, 2, 4);
        let truth = team[2] + team[3] - close.owner_total;
        let counted = counted_tokens_with(inside, &FloorPool::zeroed(), &st, Some(&o), &mut SqrtCache::default()).unwrap();
        assert!(counted + 1_000 >= truth, "the reserve's team share under-counted: {counted} counted where the close leaves the team {truth}");
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
    fn a_reserve_hole_is_charged_to_the_fed_excess_first_then_the_step() {
        let mut planned = [0u64; 16];
        planned[2] = 100;
        planned[3] = 100;
        let mut o = StairOwners::zeroed();
        o.tokens[2] = 40;
        o.tokens[3] = 40;
        o.owner_total = 80;
        o.set_reserve_fed(60);
        assert!(o.conserved());
        let (q, _) = owners::book_reserve(&mut o, 210, 180, 3_000, 0, &planned, 2, 4);
        assert_eq!(o.reserve_fed(), 0, "the hole and the sale took the fed record first (the EVM's excess 0)");
        assert_eq!((o.cut[2], o.cut[3]), (20, 0), "the step booked only what sold of it, never the hole");
        assert_eq!((o.tokens[2], o.tokens[3], o.owner_sold, q), (32, 40, 8, 800));
        assert!(o.conserved());

        let mut o = StairOwners::zeroed();
        o.tokens[2] = 40;
        o.tokens[3] = 40;
        o.owner_total = 80;
        o.set_reserve_fed(60);
        let (q, _) = owners::book_reserve(&mut o, 130, 130, 0, 0, &planned, 2, 4);
        assert_eq!((o.reserve_fed(), o.cut[2], o.cut[3], o.owner_sold, q), (0, 70, 0, 0, 0));
        assert!(o.conserved());
    }

///
///
///
///
///
    #[test]
    fn graduation_places_step_zero_the_reserve_and_the_tail_inside_the_buy() {
        let src = include_str!("stair.rs");
        let body = |name: &str| {
            let at = src.find(&format!("fn {name}")).expect(name);
            let end = at + src[at..].find("\n}\n").expect("end");
            &src[at..end]
        };
        let g = body("place_at_graduation");
        assert!(g.contains("place_own(m, s, &mut st, 0, planned[0])"), "step 0 at its planned amount");
        assert!(g.contains("open_whole(m, s, SALT_STAIR_RESERVE, lo, hi, tokens)"), "the reserve");
        assert!(g.contains("open_whole(m, s, SALT_STAIR_TAIL, lo, hi, tail_total)"), "the tail");
        assert!(g.contains("s.stair_ready = 1;"), "the pool trades when the buy lands");
        assert!(!g.contains("for i in 0..n {\n        let left"), "no per-step placement loop");
        assert_eq!(src.matches(["fn ", "place("].concat().as_str()).count(), 0, "no placement instruction's body");
        let sp = body("split_reserve");

        assert!(sp.contains("reserve_split(b - prot, &planned, j, n)"), "the split conserves tokens (floor-math's rule)");
        assert!(sp.contains("add(rest, not_taken)"), "what step j cannot take goes on to the reserve");
        let c = body("collect_steps");
        assert!(c.contains("m.market.tick >= st.reserve_upper"), "a split only once the market entered the reserve");
        assert!(c.matches("take.ops == STAIR_STEPS_PER_RATCHET").count() == 3, "every operation under the one cap");


        let l = body("feed_vault_tokens");
        assert!(l.contains("vault_relist_cap_tick(bought, cost)") && l.contains("vault_list_target("));
        assert!(l.contains("if m.market.tick < hi {"), "never added to a position the market stands inside");
        assert!(l.contains("add(bought, s.wall_held_tokens)"), "the whole inventory moves, the free tokens with the bought");
        assert!(l.contains("s.inventory_tokens = 0;") && l.contains("s.inventory_cost_eth = 0;"), "the pair is zeroed after a move");
        assert!(!l.contains("pol_tokens"), "rounding never joins the POL re-list");
    }

///
///
///
///
    #[test]
    fn the_cushions_quote_is_never_swept_into_the_vault() {
        let src = include_str!("stair.rs");
        let body = |name: &str| {
            let at = src.find(&format!("fn {name}(")).expect(name);
            let end = at + src[at..].find("\n}\n").expect("end");
            &src[at..end]
        };
        let refollow = body("refollow_cushion");
        assert_eq!(refollow.matches("intake(").count(), 1, "one intake: the unreachable no-range arm");
        let arm = refollow.find("_ => {").expect("the no-range arm");
        assert!(refollow[arm..].contains("intake("), "and it is in that arm");
        assert!(!body("collect_relist").contains("SALT_CUSHION"), "the re-list's pass never closes the cushion");
        let rp = body("ratchet_positions");
        assert!(rp.find("stair_split").unwrap() < rp.find("refollow_cushion(").unwrap(), "split first, then the cushion");


        assert!(rp.contains("stair_split_for(rest, p, floor_sqrt, pol_quote, supply, &params)"), "the pool's own POL target");
        let (above, from) = (["cushion_range_above", "("].concat(), ["cushion_range_from", "("].concat());
        assert!(!src.contains(above.as_str()) && !src.contains(from.as_str()), "never the presets' width on every pool");
        assert!(rp.contains("add(classic, held.0)"), "the POL's quote counts the cushion's, with the re-list's home");

        assert!(refollow.contains("cushion_target(m, s, st, width)"), "the cushion's range is the one target");
        assert!(!refollow.contains("cushion_range(m.market.tick"), "never the bare spot");
        let target = body("cushion_target");
        assert!(target.contains("cushion_range_above_w(m.market.tick, settled, s.tick_spacing, s.floor_tick, width)"), "the settled tick, above the band");
        assert!(body("settled_tick").contains("CUSHION_TWAP_WINDOW_SECS"));
    }

///
///
///
///
///
    #[test]
    fn a_close_that_hands_the_pol_tokens_promises_the_relists_open() {
        let src = include_str!("stair.rs");
        let body = |name: &str| {
            let at = src.find(&format!("fn {name}(")).expect(name);
            let end = at + src[at..].find("\n}\n").expect("end");
            &src[at..end]
        };
        assert!(body("room").contains("self.modifies + self.reserved + n <= self.budget"), "every room check counts the promise");
        let cr = body("collect_relist");
        assert!(cr.contains("due && !take.room(2)") && cr.contains("take.reserved = 1;"), "the re-list closes only with its re-open promised");
        let rf = body("refollow_cushion");
        assert!(rf.contains("let relist = (liq != 0 && held.1 != 0 && take.reserved == 0) as u8;") && rf.contains("take.reserved += relist;"));
        let rp = body("ratchet_positions");
        let (spend, place) = (rp.find("take.reserved = 0;").expect("spent"), rp.find("place_relist(m, s, &mut st, take.pol_tokens,").expect("placed"));
        assert!(spend < place, "the promise is spent right before the re-list's open");
        assert!(rp.contains("relisted = place_relist("), "the event reports what was placed");

        let mut t = Take { budget: STAIR_MODIFY_BUDGET, ..Take::default() };
        t.modifies = 1;
        t.reserved = 1;
        assert!(!t.room(1 + 2), "the feed (1) beside the cushion's move (2) and the promised open: no room, it waits");
        assert!(t.room(2), "the cushion's close and re-open fit beside the promise");
        t.modifies += 2;
        t.reserved = 0;
        assert!(t.room(1), "and the re-list's open is left its room");
    }

///
///
///
///
///
///
    #[test]
    fn the_audit_fixes_are_where_they_belong() {
        let src = include_str!("stair.rs");
        let body = |name: &str| {
            let at = src.find(&format!("fn {name}(")).expect(name);
            let end = at + src[at..].find("\n}\n").expect("end");
            &src[at..end]
        };
        let rf = body("refollow_cushion");
        assert!(rf.contains("st.cushion_quote.saturating_sub(q) as u128"), "H1: what the cushion paid, its ORIGINAL cost");
        let cr = body("collect_relist");
        assert!(cr.contains("math_cost(b, st.relist_cost_tick)"), "H1: the re-list's unsold tokens keep their cost line");
        let pr = body("place_relist");
        assert!(pr.contains("relist_cost_line(c, spacing)") && pr.contains("if cline < line"), "H1: never under 1.1 × cost");
        assert!(pr.contains("ask_line(m, s)?.0"), "L7: from the clear line");
        assert!(pr.contains("hold_with_basis(s, pol_tokens, basis)"), "H1: held with their cost, never as free tokens");
        let t = body("cushion_target");
        assert!(t.contains("cushion_home_top(m.market.tick, s.floor_tick, s.tick_spacing, width)"), "N1: the home, off the published floor");
        assert!(t.contains("cushion_range_above_w(m.market.tick, home, s.tick_spacing, s.floor_tick, width)"), "N1: its range");
        assert!(!t.contains("cushion_range_above_w(m.market.tick, m.market.tick,"), "N1: never the bare spot's range");
        assert_eq!(floor_math::stair::RELIST_COST_MARKUP_TICKS, 296);
        assert!(1.0001f64.powi(296) >= 1.03 && 1.0001f64.powi(295) < 1.03, "296 ticks is the least that makes 1.03 ×");
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
    #[test]
    fn d5b_a_deferred_ratchet_sets_aside_and_the_finishing_ratchet_decides_the_sum() {
        let bps = 4_000u16;
        let due1 = floor_math::owners::payee_due(1_000_000, bps);
        assert_eq!(due1, 400_000);
        let never = || -> Result<u64> { panic!("a deferred ratchet must not ask floor first") };


        let r1 = payee_pass(true, due1, 0, 1_200_000, never).unwrap();
        assert_eq!(r1, PayeePass { aside: 400_000, paid: 0, forfeited: 0, decided: 0 });

        let bug = payee_pass(false, due1, 0, 1_200_000, || Ok(1_200_000)).unwrap();
        assert_eq!((bug.paid, bug.forfeited), (0, 400_000), "deciding on the half pass forfeits the share");

        let r1b = payee_pass(true, 20_000, r1.aside, 50_000, never).unwrap();
        assert_eq!(r1b, PayeePass { aside: 20_000, paid: 0, forfeited: 0, decided: 0 });
        let pending = r1.aside + r1b.aside;


        let due2 = floor_math::owners::payee_due(250_000, bps);
        let r2 = payee_pass(false, due2, pending, 300_000 + pending, || Ok(100_000)).unwrap();
        assert_eq!(r2, PayeePass { aside: 0, paid: due2 + pending, forfeited: 0, decided: due2 + pending });

        let r3 = payee_pass(false, due2, pending, 300_000 + pending, || Ok(500_000)).unwrap();
        assert_eq!((r3.paid, r3.forfeited, r3.decided), (220_000, 300_000, 520_000));
        assert_eq!(r3.paid + r3.forfeited, due2 + pending, "booked + forfeited is the whole due, never a debt");

        assert_eq!(payee_pass(false, 0, 0, 1, never).unwrap(), PayeePass::default());
    }

///
///
///
///
///
    #[test]
    fn d5b_every_deferred_collection_is_marked_and_the_pending_share_folds_back_once() {
        let src = include_str!("stair.rs");
        let body = |name: &str| {
            let at = src.find(&format!("fn {name}(")).expect(name);
            let end = at + src[at..].find("\n}\n").expect("end");
            &src[at..end]
        };
        let cs = body("collect_steps");
        assert_eq!(cs.matches("take.more_left = true;").count(), 3, "collect_steps' three deferral points");
        assert_eq!(cs.matches("take.collect_deferred = true;").count(), 3, "each marks the collection deferred");
        for (i, _) in cs.match_indices("take.more_left = true;") {
            let next = &cs[i..i + cs[i..].find("return Ok(());").expect("return")];
            assert!(next.contains("take.collect_deferred = true;"), "a deferral point left unmarked");
        }
        assert!(body("collect_relist").contains("take.collect_deferred |= bought;"), "the re-list bought into");
        let rp = body("ratchet_positions");
        let fold = rp.find("if !take.collect_deferred {").expect("the fold");
        assert!(rp[fold..].starts_with("if !take.collect_deferred {\n        let pending = s.owner_pending();"));
        assert!(fold > rp.find("feed_vault_tokens(").expect("feed") && fold < rp.find("cushion_arm(").expect("arm"));
        let code = &src[..src.find("#[cfg(test)]").expect("tests")];
        assert_eq!(code.matches("s.set_owner_pending(").count(), 2, "the fold clears it; book_payee sets aside");
    }

    #[test]
    fn the_relists_proceeds_go_home_before_the_payee_and_floor_first() {
        let src = include_str!("stair.rs");
        let body = |name: &str| {
            let at = src.find(&format!("fn {name}(")).expect(name);
            let end = at + src[at..].find("\n}\n").expect("end");
            &src[at..end]
        };
        let rp = body("ratchet_positions");
        let at = |pat: &str| rp.find(pat).unwrap_or_else(|| panic!("ratchet_positions lacks `{pat}`"));
        let arm = at("cushion_arm(m, s, &st, &take, cushion_held, width)?");
        let home = at("take.home = if arm.is_some() { take.relist_q.min(take.collected) } else { 0 };");
        let held = at("let held = (add(cushion_held.0, take.home)?, cushion_held.1);");
        let payee = at("book_payee(m, s, &st, supply, &mut take, held)?");
        let rest = at("let rest = sub(take.collected, take.home)?;");
        let first = at("floor_first_of(m, s, &st, supply, rest, held, &shares, take.room(2), target)?");
        let to_pol = at("add(take.home, cushion_share(rest, first, &shares))?");
        assert!(arm < home && home < held && held < payee && payee < rest && rest < first && first < to_pol, "the order");
        assert!(rp.contains("let to_pol = match arm {\n            None => 0,"), "the park: nothing home, all to the vault");
        assert_eq!(rp.matches("take.relist_q").count(), 1, "the re-list's proceeds read once, for the home");
        assert!(body("book_payee").contains("let collected = sub(take.collected, take.home)?;"), "the payee never takes the home");
        let ca = body("cushion_arm");
        assert!(ca.contains("Some(t) if cushion_takes_quote(take, st, held, m.market.tick) => Some(t)") && ca.contains("_ => None"));
        assert!(body("collect_relist").contains("take.relist_q = add(take.relist_q, q)?;"));

        assert!(body("floor_first_of").contains("let to_pol = cushion_share(collected, k, shares);"), "the probe");
        let code = &src[..src.find("#[cfg(test)]").expect("tests")];
        assert_eq!(code.matches("cushion_share(").count(), 3, "its definition, the probe and the split");
        assert_eq!(body("ratchet_positions").matches("pol_share(").count(), 0, "the split never bypasses the helper");
    }

///
///
///
    #[test]
    fn a_parked_cushion_climbs_from_its_recorded_range() {
        let src = include_str!("stair.rs");
        let at = src.find("fn refollow_cushion(").expect("refollow_cushion");
        let rf = &src[at..at + src[at..].find("\n}\n").expect("end")];
        assert!(rf.contains("let stood = st.cushion_lower != st.cushion_upper;"), "where it stands is its recorded range");
        assert!(rf.contains("Some((tl, _)) if stood && tl < st.cushion_lower => climb(m, s, st, held, to_pol, supply, tl)?,"), "the climb reads it");
        assert!(!rf.contains("if liq != 0 && tl < st.cushion_lower"), "never the liquidity alone");
    }

///
///
///
///
    #[test]
    fn the_trim_follows_the_band_and_the_hold_covers_the_rest() {
        let src = include_str!("stair.rs");
        let body = |name: &str| {
            let at = src.find(&format!("fn {name}(")).expect(name);
            let end = at + src[at..].find("\n}\n").expect("end");
            &src[at..end]
        };
        let trim = body("trim_cushion");
        assert!(!trim.contains("intake("), "the trim never banks the cushion's quote");
        assert!(trim.contains("position_liquidity(m.market.sqrt_price_x64, nlo, nhi, q, 0)"), "all its quote, re-placed whole");
        assert!(trim.contains("Some((lo, dear))"), "its own dear edge to the band's");
        assert!(trim.contains("if m.market.tick >= lo {\n        None"), "only a cushion the market stands above: all quote");


        assert!(trim.contains("cushion_home_top(m.market.tick, s.floor_tick, s.tick_spacing, width)"), "N1: the home");
        assert!(trim.contains("cushion_range_above_w(m.market.tick, home, s.tick_spacing, s.floor_tick, width)"), "N1: from the home");
        assert!(!trim.contains("cushion_range_above_w(m.market.tick, m.market.tick,"), "N1: never the bare spot's range");
        assert!(trim.contains("if missing {\n                s.needs_ratchet = 1;"), "A1: a rise withheld for an array makes the next ratchet due");
        assert!(trim.contains("if hold < floor0 { hold } else { floor0 }"), "a hold never lowers the standing floor");
        let rp = body("ratchet_positions");
        assert!(rp.contains("Some((_, upper, _)) if parked =>"), "the hold before the solve: a parked cushion");
        assert!(!trim.contains("RATCHET_MODIFY_BUDGET"), "the trim is never held by the budget (owner pick 2026-10-02, \"5\")");
        let ratchet = include_str!("ratchet.rs");
        let run = &ratchet[ratchet.find("pub fn run(").expect("run")..];
        let at = |needle: &str| run.find(needle).expect(needle);
        let (hold, solve, trim_at, bid) =
            (at("out.hold"), at("passes(m, s, supply, cushion, reserve, cut, lift_pub"), at("trim_cushion(m, s, floor0)"), at("deploy_bid(m, s)"));
        assert!(hold < solve && solve < trim_at && trim_at < bid, "the hold caps the solve; the trim follows it, before the bid");


        assert!(run.contains("passes(m, s, supply, cushion, reserve, 0, lift_settled, &mut pend)"), "the older surplus at the settled pace");
        assert!(ratchet.contains("s.vault_eth.saturating_sub(reserve).saturating_sub(cut)"), "the passes solve on the vault less the cut");
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
    fn a15_mr_s_pv2_3_4_proceeds_and_held_of_a_partly_withdrawn_step() {
        let mut parts = floor_types::slots::TeamParts::default();
        parts.mark_out(4, 0);
        let mut t = TeamCtx { bps: [5_000, 3_000, 1_999, 1, 0], n: 4, parts, ..TeamCtx::default() };

        assert!(!t.ask_of(3, 1_000_000, 10_000));
        t.held_of(3, 1_000);
        assert_eq!((t.slot_total(), t.parts.part_tokens, t.dirty), (0, 0, false));

        assert!(t.ask_of(4, 777_777, 10_000));
        assert_eq!(t.slot_ask, [0, 466_666, 310_955, 156, 0]);
        assert_eq!(t.out_quote, 777_777);

        let due = 500_000 + t.slot_total();
        assert_eq!(due, 1_277_777);
        let pass = payee_pass(false, due, 0, 2_000_000, || Ok(1_100_000)).unwrap();
        assert_eq!(pass.paid, 900_000);
        t.book_pass(&pass, due, 0, false);
        assert_eq!(t.parts.quote_credit, [0, 328_695, 219_020, 109, 0]);
        assert_eq!(t.parts.part_quote, 547_824);
        assert_eq!(pass.paid - t.parts.part_quote, 352_176, "pooled");
        assert_eq!(t.slot_total(), 0, "the asks are spent");

        t.held_of(4, 1_000_003);
        assert_eq!(&t.parts.token_credit[..4], &[0, 600_001, 399_801, 201]);
        assert_eq!(t.parts.part_tokens, 1_000_003);

        let mut d = TeamCtx { bps: [5_000, 3_000, 1_999, 1, 0], n: 4, parts, ..TeamCtx::default() };
        d.ask_of(4, 777_777, 10_000);
        let due = 500_000 + d.slot_total();
        let aside = payee_pass(true, due, 0, 5_000_000, || unreachable!()).unwrap();
        assert_eq!(aside.aside, due);
        d.book_pass(&aside, due, 0, true);
        assert_eq!(d.parts.pending_ask, [0, 466_666, 310_955, 156, 0]);
        assert_eq!(d.parts.part_quote, 0, "nothing decided");
        let pending = aside.aside;
        assert_eq!(d.fold_pending(pending), 777_777);
        assert_eq!(d.slot_ask, [0, 466_666, 310_955, 156, 0]);
        assert_eq!(d.parts.pending_ask, [0; 5]);
    }
}
