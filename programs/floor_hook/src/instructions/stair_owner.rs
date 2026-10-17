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
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use floor_types::constants::{SALT_STAIR_RESERVE, SALT_STAIR_STEP0};

use crate::constants::{CARPENTER_AMM_PROGRAM, EVENT_AUTHORITY_SEED, HOOK_AUTHORITY_SEED, HOOK_CONFIG_SEED};
use crate::errors::HookError;
use crate::events::{StairPayeeResolved, StairPayeeSwept, StairTokensWithdrawn};
use crate::mech::amm_cpi::{self, AmmCpi, PoolAccounts};
use crate::mech::positions::{add, math, sub, to_wall, Market, Mech, MechEvent};
use crate::mech::{owners, ratchet, stair};
use crate::state::{slots, FloorPool, HookConfig, StairState};
use floor_types::owners::StairOwners;



#[event_cpi]
#[derive(Accounts)]
pub struct SweepStairPayee<'info> {
    pub anyone: Signer<'info>,
    #[account(mut)]
    pub floor_pool: AccountLoader<'info, FloorPool>,
///
    #[account(mut, address = floor_pool.load()?.amm_pool @ HookError::WrongPool)]
    pub pool: UncheckedAccount<'info>,
///
///
    #[account(mut)]
    pub stair_state: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub vault0: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub destination: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(address = floor_pool.load()?.quote_mint @ HookError::WrongPool)]
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
    pub spl_token: Interface<'info, TokenInterface>,
///
    #[account(address = CARPENTER_AMM_PROGRAM)]
    pub carpenter_amm: UncheckedAccount<'info>,
///
    #[account(seeds = [HOOK_AUTHORITY_SEED], bump)]
    pub hook_authority: UncheckedAccount<'info>,
///
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = CARPENTER_AMM_PROGRAM)]
    pub amm_event_authority: UncheckedAccount<'info>,
///
///
    #[account(mut)]
    pub tip_destination: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
}

pub fn sweep_payee_handler(ctx: Context<SweepStairPayee>, slot: u8) -> Result<()> {
    let mut s: FloorPool = *ctx.accounts.floor_pool.load()?;






    let fp = ctx.accounts.floor_pool.to_account_info();
    let mut t = slots::read(&fp)?;
    slots::slot(&t, slots::SLOT_KIND_TEAM, slot)?;

    if s.stair_owner == 0 || s.owner_owed == 0 {
        return Ok(());
    }

    let (due, payee) = team_proceeds_due(&s, &t, &slots::read_parts(&fp)?, slot)?;
    if due == 0 {
        return Ok(());
    }
    require!(s.pool_bound == 1, HookError::NotBound);
    let a = &ctx.accounts;
    let (acct, _) = stair::load(core::slice::from_ref(&a.stair_state.to_account_info()), &a.floor_pool.key())?;
    let mut o = owners::read(&acct, &s)?.ok_or(error!(HookError::StairStateMissing))?;
    let p = amm_cpi::read_pool(&a.pool.to_account_info())?;
    amm_cpi::require_pool_side(&p, 0, &a.vault0.key(), &a.quote_mint.key(), &a.spl_token.key())?;
    require_keys_eq!(a.destination.owner, payee, HookError::WrongDestination);
    require_keys_eq!(a.destination.mint, s.quote_mint, HookError::WrongDestination);







    if let Some(d) = &a.tip_destination {
        require_keys_eq!(d.owner, a.anyone.key(), HookError::WrongDestination);
        require_keys_eq!(d.mint, s.quote_mint, HookError::WrongDestination);
    }
    let owed = due;
    let tip = if a.tip_destination.is_some() {
        let now = Clock::get()?.unix_timestamp;
        let t = ratchet::team_tip(&s, owed, o.last_tip_at, now);
        if t != 0 {
            o.last_tip_at = now;
        }
        t
    } else {
        0
    };
    let paid = book_team_sweep(&mut s, &mut t, slot, owed, tip)?;
    if tip != 0 {
        o.tips_paid = add(o.tips_paid, tip)?;
        owners::write(&acct, &o)?;
    }
    slots::write(&fp, &t)?;
    let pool = s.amm_pool;
    *ctx.accounts.floor_pool.load_mut()? = s;

    let a = &ctx.accounts;
    let cpi = AmmCpi {
        carpenter_amm: a.carpenter_amm.to_account_info(),
        amm_event_authority: a.amm_event_authority.to_account_info(),
        hook_authority: a.hook_authority.to_account_info(),
        hook_authority_bump: ctx.bumps.hook_authority,
        hook_program: a.program.to_account_info(),
        floor_pool: a.floor_pool.to_account_info(),
        pool: a.pool.to_account_info(),
    };
    let mut w = amm_cpi::withdraw(
        &cpi,
        &a.vault0.to_account_info(),
        &a.destination.to_account_info(),
        &a.quote_mint.to_account_info(),
        &a.spl_token.to_account_info(),
        0,
        paid,
    )?;
    if let (Some(d), true) = (&a.tip_destination, tip != 0) {
        w = amm_cpi::withdraw(
            &cpi,
            &a.vault0.to_account_info(),
            &d.to_account_info(),
            &a.quote_mint.to_account_info(),
            &a.spl_token.to_account_info(),
            0,
            tip,
        )?;
    }
    require!(ratchet::claims_cover_liabilities(&s, w.hook_claims0, w.hook_claims1), HookError::InvariantViolated);
    emit_cpi!(StairPayeeSwept { pool, slot, payee, amount: paid, tip });
    Ok(())
}



#[event_cpi]
#[derive(Accounts)]
pub struct WithdrawStairTokens<'info> {
///
    #[account(mut)]
    pub payee: Signer<'info>,
    #[account(mut)]
    pub floor_pool: AccountLoader<'info, FloorPool>,
    #[account(address = floor_pool.load()?.base_mint @ HookError::WrongPool)]
    pub base_mint: Box<InterfaceAccount<'info, Mint>>,
///
    #[account(mut, address = floor_pool.load()?.amm_pool @ HookError::WrongPool)]
    pub pool: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub bitmap: UncheckedAccount<'info>,
    #[account(mut)]
    pub vault0: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub vault1: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(address = floor_pool.load()?.quote_mint @ HookError::WrongPool)]
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
    pub spl_token: Interface<'info, TokenInterface>,
    pub token_2022: Interface<'info, TokenInterface>,
///
    #[account(mut)]
    pub payee_base: Box<InterfaceAccount<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
///
    #[account(address = CARPENTER_AMM_PROGRAM)]
    pub carpenter_amm: UncheckedAccount<'info>,
///
    #[account(seeds = [HOOK_AUTHORITY_SEED], bump)]
    pub hook_authority: UncheckedAccount<'info>,
///
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = CARPENTER_AMM_PROGRAM)]
    pub amm_event_authority: UncheckedAccount<'info>,
}

pub fn withdraw_handler<'info>(ctx: Context<'info, WithdrawStairTokens<'info>>, step: u8, slot: u8) -> Result<()> {
    let mut s: FloorPool = *ctx.accounts.floor_pool.load()?;
    require!(s.pool_bound == 1 && s.stair_ready == 1, HookError::NotBound);
    require!(s.stair_owner != 0, HookError::StairOwnerBadConfig);
    s.require_quote_live()?;
    let a = &ctx.accounts;
    let (market, spacing, paused) = withdraw_market(a)?;




    let hook_keys = (s.holders_hooked != 0)
        .then(|| amm_cpi::holders_hook_keys(&a.base_mint.key(), &a.vault1.key(), &a.payee_base.key()));
    let (tick_arrays, hook_accounts) =
        amm_cpi::split_hook_accounts(ctx.remaining_accounts, &a.floor_pool.to_account_info(), hook_keys);
    let mut m = Mech {
        cpi: AmmCpi {
            carpenter_amm: a.carpenter_amm.to_account_info(),
            amm_event_authority: a.amm_event_authority.to_account_info(),
            hook_authority: a.hook_authority.to_account_info(),
            hook_authority_bump: ctx.bumps.hook_authority,
            hook_program: a.program.to_account_info(),
            floor_pool: a.floor_pool.to_account_info(),
            pool: a.pool.to_account_info(),
        },
        accts: PoolAccounts {
            payer: None,
            rent_owner: Pubkey::default(),
            bitmap: a.bitmap.to_account_info(),
            vault0: a.vault0.to_account_info(),
            vault1: a.vault1.to_account_info(),
            mint0: a.quote_mint.to_account_info(),
            mint1: a.base_mint.to_account_info(),
            token_program0: a.spl_token.to_account_info(),
            token_program1: a.token_2022.to_account_info(),
            system_program: a.system_program.to_account_info(),
            tick_arrays,
        },
        position_seed: None,
        position_bid: None,
        bounty_destination: None,
        spacing,
        market,
        events: Vec::new(),
        modifies: 0,
    };
    let (acct, mut st) = stair::load(&m.accts.tick_arrays, m.cpi.floor_pool.key)?;
    let mut o = owners::read(&acct, &s)?.ok_or(error!(HookError::StairStateMissing))?;
    require!(o.version != 0, HookError::StairStateMissing);




    let sl = slots::read_slot(&a.floor_pool.to_account_info(), slots::SLOT_KIND_TEAM, slot)?;
    require!(!sl.is_pending(), HookError::StairPayeePending);
    require_keys_eq!(a.payee.key(), Pubkey::new_from_array(sl.key), HookError::StairWrongPayee);
    require_keys_eq!(a.payee_base.owner, a.payee.key(), HookError::WrongDestination);
    require_keys_eq!(a.payee_base.mint, s.base_mint, HookError::WrongDestination);
    let now = Clock::get()?.unix_timestamp;
    if step == floor_types::constants::STAIR_HELD_STEP {


        require!(
            floor_math::owners::step_unlocked(o.first_step, o.first_step, now, o.anchor, o.first_unlock_secs, o.every_secs),
            HookError::StairStepLocked
        );
        let paid = pay_slot(&m, &mut s, &acct, &mut o, a, &hook_accounts, slot)?;
        require!(paid > 0, HookError::NothingOwed);
        emit_cpi!(StairTokensWithdrawn { pool: s.amm_pool, slot, payee: a.payee.key(), step, from_reserve: false, tokens: paid });
        return Ok(());
    }
    require!(
        floor_math::owners::step_unlocked(step, o.first_step, now, o.anchor, o.first_unlock_secs, o.every_secs),
        HookError::StairStepLocked
    );
    let i = step as usize;
    require!(step < st.steps, HookError::StairStepNotAbove);



    let tokens = slot_step_part(&a.floor_pool.to_account_info(), &o, i, slot)?;
    require!(tokens > 0, HookError::NothingOwed);

    let (salt, lo, hi, liq, in_reserve) = if step >= st.first_open && step < st.placed {
        (SALT_STAIR_STEP0 + step as u64, st.step_lower[i], st.step_upper[i], st.step_liq[i].get(), false)
    } else if step >= st.placed && st.reserve_liq.get() != 0 {
        (SALT_STAIR_RESERVE, st.reserve_lower, st.reserve_upper, st.reserve_liq.get(), true)
    } else {
        return err!(HookError::StairStepNotAbove);
    };

    require!(liq != 0 && m.market.tick >= hi, HookError::StairStepNotAbove);



    let before = owners::token_only(lo, hi, liq)?;





    let tokens = bounded_take(&s, &st, &o, i, in_reserve, before, tokens)?;
    let out = if paused { owner_liquidity(lo, hi, tokens, liq)? } else { liq };
    let r = m.close(&s, salt, lo, hi, out)?;
    let (q, b) = amm_cpi::back(&r)?;
    let (fq, fb) = amm_cpi::fees(&r);
    let (fq, fb) = (fq.min(q), fb.min(b));


    let owner_on = if in_reserve { owners::in_reserve(&o, st.placed, st.steps) } else { o.tokens[i] };
    let owned = owners::fee_share(&mut o, fq, before, owner_on);
    st.swept_quote = add(st.swept_quote, sub(fq, owned)?)?;
    to_wall(&mut s, fb)?;
    let principal = sub(b, fb)?;
    let take = tokens.min(principal);
    let rest = principal - take;


    let l = if paused {
        0
    } else if rest > 0 {
        math::position_liquidity(m.market.sqrt_price_x64, lo, hi, 0, rest)?
    } else {
        0
    };
    let used = if l > 0 {
        let r = m.open(&s, salt, lo, hi, l)?;
        let (q2, used) = amm_cpi::used(&r)?;
        require!(q2 == 0, HookError::StairNotTokenOnly);
        used
    } else {
        0
    };
    to_wall(&mut s, sub(rest, used)?)?;
    let l = if paused { liq - out } else { l };




    book_slot_exit(&a.floor_pool.to_account_info(), &mut o, i, slot, take)?;


    s.ask_tokens_deployed = add(s.ask_tokens_deployed, take)?;
    if in_reserve {
        st.reserve_liq.set(l);
        o.cut[i] = o.cut[i].saturating_add(take);


        reopened_reserve(&s, &st, &mut o, sub(rest, used)?, owners::token_only(lo, hi, l)?)?;
    } else {
        st.step_liq[i].set(l);

        clamp_own_step(&mut o, i, lo, hi, l)?;
    }

    s.ladder_tokens_remaining = ladder_after(m.market.sqrt_price_x64, &s, &st, &o)?;
    s.ladder_upper = stair::frontier(&st);
    s.needs_ratchet = 1;
    stair::store(&acct, &st)?;
    let paid = pay_slot(&m, &mut s, &acct, &mut o, a, &hook_accounts, slot)?;
    for e in core::mem::take(&mut m.events) {
        if let MechEvent::PlatformCut(ev) = e {
            emit_cpi!(ev);
        }
    }
    emit_cpi!(StairTokensWithdrawn {
        pool: s.amm_pool,
        slot,
        payee: a.payee.key(),
        step,
        from_reserve: in_reserve,
        tokens: paid,
    });
    Ok(())
}

///
///
///
///
///
///
#[inline(never)]
fn pay_slot<'info>(
    m: &Mech<'info>,
    s: &mut FloorPool,
    acct: &AccountInfo<'info>,
    o: &mut StairOwners,
    a: &WithdrawStairTokens<'info>,
    hook_accounts: &[AccountInfo<'info>],
    slot: u8,
) -> Result<u64> {
    let fp = a.floor_pool.to_account_info();
    let mut t = slots::read(&fp)?;
    let due = book_held_withdraw(o, s, &mut t, &slots::read_parts(&fp)?, slot)?;
    slots::write(&fp, &t)?;
    owners::write(acct, o)?;
    m.flush(s)?;
    if due == 0 {
        return Ok(0);
    }
    let w = amm_cpi::withdraw_hooked(
        &m.cpi,
        &a.vault1.to_account_info(),
        &a.payee_base.to_account_info(),
        &a.base_mint.to_account_info(),
        &a.token_2022.to_account_info(),
        1,
        due,
        hook_accounts,
    )?;
    require!(ratchet::claims_cover_liabilities(s, w.hook_claims0, w.hook_claims1), HookError::InvariantViolated);
    Ok(due)
}

///
///
#[inline(never)]
fn slot_step_part(fp: &AccountInfo, o: &StairOwners, i: usize, slot: u8) -> Result<u64> {
    Ok(step_part(&slots::read(fp)?, &slots::read_parts(fp)?, o.tokens[i], i, slot))
}

///
///
pub(crate) fn step_part(t: &slots::PayeeSlots, p: &slots::TeamParts, dv: u64, i: usize, slot: u8) -> u64 {
    let (bps, n) = t.team_bps();
    let out = p.out_of(i);
    if slot >= n || out & (1 << slot) != 0 {
        return 0;
    }
    floor_math::slots::part(dv, bps[slot as usize], floor_math::slots::in_weight(&bps, n, out))
}

///
///
///
///
#[inline(never)]
fn book_slot_exit(fp: &AccountInfo, o: &mut StairOwners, i: usize, slot: u8, take: u64) -> Result<()> {
    let t = slots::read(fp)?;
    let mut p = slots::read_parts(fp)?;
    let x = step_part(&t, &p, o.tokens[i], i, slot);
    require!(take <= x, HookError::InvariantViolated);
    book_exit(o, &mut p, t.team_count, i, slot, x, take)?;
    if t.team_count > 1 {
        slots::write_parts(fp, &p)?;
    }
    Ok(())
}

///
pub(crate) fn book_exit(o: &mut StairOwners, p: &mut slots::TeamParts, n: u8, i: usize, slot: u8, x: u64, take: u64) -> Result<()> {
    o.tokens[i] = sub(o.tokens[i], x)?;
    owners::book_clamp(o, x - take);
    o.set_held(add(o.held(), take)?);
    if n > 1 {
        p.mark_out(i, slot as usize);
        p.token_credit[slot as usize] = add(p.token_credit[slot as usize], take)?;
        p.part_tokens = add(p.part_tokens, take)?;
    }
    Ok(())
}

///
///
#[inline(never)]
fn clamp_own_step(o: &mut StairOwners, i: usize, lo: i32, hi: i32, liq: u128) -> Result<()> {
    if o.tokens[i] == 0 {
        return Ok(());
    }
    let holds = if liq == 0 { 0 } else { owners::token_only(lo, hi, liq)? };
    let dropped = o.tokens[i].saturating_sub(holds);
    o.tokens[i] -= dropped;
    owners::book_clamp(o, dropped);
    Ok(())
}

///
///
///
///
///
///
#[inline(never)]
fn bounded_take(s: &FloorPool, st: &StairState, o: &StairOwners, i: usize, in_reserve: bool, before: u64, tokens: u64) -> Result<u64> {
    let real = if in_reserve {
        let planned = stair::planned_of(s, st.stair_total, st.steps, st.growth_bps)?;
        floor_math::owners::reserve_allot(before, &planned, &o.cut, o.reserve_fed(), st.placed, st.steps).0[i]
    } else {
        before
    };
    require!(tokens <= real.saturating_add(floor_math::owners::OWNER_BOOKS_DUST), HookError::OwnerBooksExceedHoldings);
    Ok(tokens.min(real))
}

///
///
#[inline(never)]
fn reopened_reserve(s: &FloorPool, st: &StairState, o: &mut StairOwners, shortfall: u64, held: u64) -> Result<()> {
    let planned = stair::planned_of(s, st.stair_total, st.steps, st.growth_bps)?;
    let fed0 = o.reserve_fed();
    let fed = floor_math::owners::record_reserve_shortfall(&planned, &mut o.cut, fed0, st.placed, st.steps, shortfall);
    o.set_reserve_fed(fed);
    owners::cap_reserve(o, st.placed, st.steps, held - fed.min(held));
    Ok(())
}

///
#[inline(never)]
fn ladder_after(sqrt_price_x64: u128, s: &FloorPool, st: &StairState, o: &StairOwners) -> Result<u64> {
    let standing = stair::standing_tokens(sqrt_price_x64, st)?;
    let counted = stair::counted_tokens_with(sqrt_price_x64, s, st, Some(o), &mut stair::SqrtCache::default())?;
    owners::ladder_less_counted(standing, counted)
}

///
///
#[inline(never)]
fn withdraw_market(a: &WithdrawStairTokens<'_>) -> Result<(Market, u16, bool)> {
    let p = amm_cpi::read_pool(&a.pool.to_account_info())?;
    amm_cpi::require_pool_side(&p, 0, &a.vault0.key(), &a.quote_mint.key(), &a.spl_token.key())?;
    amm_cpi::require_pool_side(&p, 1, &a.vault1.key(), &a.base_mint.key(), &a.token_2022.key())?;
    let now = Clock::get()?.unix_timestamp;
    let paused = floor_types::constants::pool_is_paused(p.paused, p.guardian_paused_until, now);
    Ok((Market::from_pool(&p), p.tick_spacing, paused))
}

///
///
///
///
pub fn owner_liquidity(lo: i32, hi: i32, tokens: u64, liq: u128) -> Result<u128> {
    use carpenter_clmm_math::liquidity_math::get_delta_amount_1_unsigned as d1;
    let sa = math::sqrt_price_at_tick(lo)?;
    let sb = math::sqrt_price_at_tick(hi)?;
    require!(sb > sa, HookError::BadRange);
    let mut l = ((tokens as u128) << 64).div_ceil(sb - sa).min(liq);
    while l < liq && d1(sa, sb, l, false).map_err(|_| error!(HookError::MathOverflow))? < tokens {
        l += 1;
    }
    Ok(l)
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
#[derive(Accounts)]
pub struct ClearTeamBuyBooks<'info> {
    #[account(constraint = launch_authority.key() == hook_config.load()?.launch_authority @ HookError::NotLauncher)]
    pub launch_authority: Signer<'info>,
    #[account(seeds = [HOOK_CONFIG_SEED], bump)]
    pub hook_config: AccountLoader<'info, HookConfig>,
    #[account(mut)]
    pub floor_pool: AccountLoader<'info, FloorPool>,
}

///
///
pub fn check_clear_team_buy_books(s: &FloorPool) -> Result<()> {
    require!(
        s.stair_owner == floor_types::constants::STAIR_OWNER_TEAM_BUY
            && s.ladder_seeded == 0
            && s.stair_ready == 0,
        HookError::StairOwnerBadConfig
    );
    Ok(())
}

pub fn clear_team_buy_books_handler(ctx: Context<ClearTeamBuyBooks>) -> Result<()> {
    let mut s = ctx.accounts.floor_pool.load_mut()?;
    check_clear_team_buy_books(&s)?;
    s.stair_owner = 0;
    s.owner_owed = 0;
    s.owner_paid = 0;
    Ok(())
}

#[event_cpi]
#[derive(Accounts)]
pub struct ResolveStairPayee<'info> {
    #[account(constraint = launch_authority.key() == hook_config.load()?.launch_authority @ HookError::NotLauncher)]
    pub launch_authority: Signer<'info>,
    #[account(seeds = [HOOK_CONFIG_SEED], bump)]
    pub hook_config: AccountLoader<'info, HookConfig>,
///
///
    #[account(mut)]
    pub floor_pool: AccountLoader<'info, FloorPool>,
}

pub fn resolve_payee_handler(ctx: Context<ResolveStairPayee>, slot: u8, payee: Pubkey) -> Result<()> {
    let s: FloorPool = *ctx.accounts.floor_pool.load()?;
    require_keys_neq!(payee, Pubkey::default(), HookError::BadConfig);

    let fp = ctx.accounts.floor_pool.to_account_info();
    let mut t = slots::read(&fp)?;
    let pending_id = slots::resolve_slot(&mut t, slots::SLOT_KIND_TEAM, slot, payee, HookError::StairPayeeResolved)?;
    slots::write(&fp, &t)?;
    emit_cpi!(StairPayeeResolved { pool: s.amm_pool, slot, pending_id, payee });
    Ok(())
}



///
///
///
pub(crate) fn team_proceeds_due(s: &FloorPool, t: &slots::PayeeSlots, p: &slots::TeamParts, slot: u8) -> Result<(u64, Pubkey)> {
    let sl = slots::slot(t, slots::SLOT_KIND_TEAM, slot)?;
    if sl.is_pending() {
        return Ok((0, Pubkey::default()));
    }
    let wallet = Pubkey::new_from_array(sl.key);



    let Some(pooled) = s.owner_paid.checked_add(s.owner_owed).and_then(|l| l.checked_sub(p.part_quote)) else {
        return Ok((0, wallet));
    };
    let credit = p.quote_credit.get(slot as usize).copied().unwrap_or(0);
    Ok((floor_math::slots::slot_due_with_credit(pooled, sl.bps, credit, sl.paid, s.owner_owed), wallet))
}

///
///
///
pub(crate) fn book_team_sweep(s: &mut FloorPool, t: &mut slots::PayeeSlots, slot: u8, owed: u64, tip: u64) -> Result<u64> {
    let paid = owed - tip;
    s.owner_owed -= owed;
    s.owner_paid = add(s.owner_paid, owed)?;
    if let Some(m) = t.slot_mut(slots::SLOT_KIND_TEAM, slot) {
        m.paid = add(m.paid, owed)?;
    }
    Ok(paid)
}

///
///
///
///
pub(crate) fn book_held_withdraw(
    o: &mut StairOwners,
    s: &mut FloorPool,
    t: &mut slots::PayeeSlots,
    p: &slots::TeamParts,
    slot: u8,
) -> Result<u64> {
    let sl = slots::slot(t, slots::SLOT_KIND_TEAM, slot)?;
    let held = o.held();



    let Some(pooled) = held.checked_add(o.owner_withdrawn).and_then(|l| l.checked_sub(p.part_tokens)) else {
        return Ok(0);
    };
    let credit = p.token_credit.get(slot as usize).copied().unwrap_or(0);
    let due = floor_math::slots::slot_due_with_credit(pooled, sl.bps, credit, sl.tokens_paid, held);
    o.set_held(held - due);
    o.owner_withdrawn = add(o.owner_withdrawn, due)?;
    s.ask_tokens_deployed = sub(s.ask_tokens_deployed, due)?;
    if let Some(x) = t.slot_mut(slots::SLOT_KIND_TEAM, slot) {
        x.tokens_paid = add(x.tokens_paid, due)?;
    }
    Ok(due)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code(r: Result<()>) -> u32 {
        match r.unwrap_err() {
            anchor_lang::error::Error::AnchorError(e) => e.error_code_number,
            e => panic!("not an anchor error: {e:?}"),
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
    fn a_paused_withdrawal_decreases_by_the_owners_tokens_alone() {
        use carpenter_clmm_math::liquidity_math::get_delta_amount_1_unsigned as d1;
        for (lo, hi) in [(-200_000, -190_000), (-50_000, -40_000), (0, 600), (100_000, 120_000)] {
            let (sa, sb) = (math::sqrt_price_at_tick(lo).unwrap(), math::sqrt_price_at_tick(hi).unwrap());
            let liq = math::position_liquidity(sb, lo, hi, 0, 50_000_000_000_000).unwrap();
            let held = d1(sa, sb, liq, false).unwrap();
            for tokens in [1u64, 7, 1_000_003, 20_000_000_000_000, held] {
                let l = owner_liquidity(lo, hi, tokens, liq).unwrap();
                assert!(l <= liq, "never more than the position");
                let out = d1(sa, sb, l, false).unwrap();
                assert!(out >= tokens, "[{lo}, {hi}] {tokens}: pays {out}");
                if l > 0 {
                    assert!(d1(sa, sb, l - 1, false).unwrap() < tokens, "the LEAST such liquidity");
                }
                let rest = d1(sa, sb, liq - l, false).unwrap();
                assert!(rest + out <= held && held - rest - out <= 1, "the remainder stays in the position");
            }
            assert_eq!(owner_liquidity(lo, hi, held + 1, liq).unwrap(), liq, "capped at the position");
        }
    }



    fn ecode(e: anchor_lang::error::Error) -> u32 {
        match e {
            anchor_lang::error::Error::AnchorError(a) => a.error_code_number,
            e => panic!("not an anchor error: {e:?}"),
        }
    }
    fn team(args: &[(u16, u8, u8)]) -> slots::PayeeSlots {
        let a: Vec<slots::SlotArg> =
            args.iter().map(|&(bps, platform, b)| slots::SlotArg { bps, platform, key: [b; 32] }).collect();
        let mut t = slots::PayeeSlots::default();
        t.fill(slots::SLOT_KIND_TEAM, &a);
        t
    }

///
///
///
    #[test]
    fn mr_s_team_proceeds_sweep_per_slot_with_the_tip_charged_to_the_slot() {
        let mut t = team(&[(7_000, 0, 1), (3_000, 0, 2)]);
        let mut s: FloorPool = bytemuck::Zeroable::zeroed();
        s.owner_owed = 10_000;
        s.bounty_bps = 100;
        s.max_bounty = 50_000_000;
        let (due, who) = team_proceeds_due(&s, &t, &slots::TeamParts::default(), 0).unwrap();
        assert_eq!((due, who), (7_000, Pubkey::new_from_array([1; 32])));
        let tip = ratchet::team_tip(&s, due, 0, 1_000_000);
        assert_eq!(tip, 70, "1 % of the slot's due, the day's first tipped sweep");
        assert_eq!(book_team_sweep(&mut s, &mut t, 0, due, tip).unwrap(), 6_930, "the payee gets due − tip");
        assert_eq!((s.owner_owed, s.owner_paid, t.team[0].paid, t.team[1].paid), (3_000, 7_000, 7_000, 0));

        let (due1, _) = team_proceeds_due(&s, &t, &slots::TeamParts::default(), 1).unwrap();
        assert_eq!(due1, 3_000);
        assert_eq!(book_team_sweep(&mut s, &mut t, 1, due1, 0).unwrap(), 3_000, "an untipped sweep pays the whole due");
        assert_eq!((s.owner_owed, s.owner_paid), (0, 10_000));

        s.owner_owed = 10_000;
        assert_eq!(team_proceeds_due(&s, &t, &slots::TeamParts::default(), 0).unwrap().0, 7_000);
        assert_eq!(team_proceeds_due(&s, &t, &slots::TeamParts::default(), 1).unwrap().0, 3_000);

        let mut one = team(&[(10_000, 0, 5)]);
        let mut p: FloorPool = bytemuck::Zeroable::zeroed();
        p.owner_owed = 999_999;
        let (d, _) = team_proceeds_due(&p, &one, &slots::TeamParts::default(), 0).unwrap();
        assert_eq!(d, 999_999);
        book_team_sweep(&mut p, &mut one, 0, d, 0).unwrap();
        assert_eq!((p.owner_owed, p.owner_paid, one.team[0].paid), (0, 999_999, 999_999));


        let pend = team(&[(5_000, 0, 1), (5_000, 3, 2)]);
        assert_eq!(team_proceeds_due(&s, &pend, &slots::TeamParts::default(), 1).unwrap(), (0, Pubkey::default()));
        assert!(team_proceeds_due(&s, &pend, &slots::TeamParts::default(), 0).is_ok(), "the address slot sweeps as usual");
        assert_eq!(ecode(team_proceeds_due(&s, &pend, &slots::TeamParts::default(), 2).unwrap_err()), 7000 + HookError::BadSlot as u32);
    }

///
///
///
///
    #[test]
    fn mr_s_a_pending_team_slots_sweep_is_a_silent_no_op() {
        let pend = team(&[(6_000, 0, 1), (4_000, 3, 2)]);
        let mut s: FloorPool = bytemuck::Zeroable::zeroed();
        s.stair_owner = 1;
        s.owner_owed = 10_000;
        let (sb, tb) = (s, pend);
        assert_eq!(team_proceeds_due(&s, &pend, &slots::TeamParts::default(), 1).unwrap(), (0, Pubkey::default()), "pending: due 0, no payee");
        assert_eq!(bytemuck::bytes_of(&s), bytemuck::bytes_of(&sb), "nothing written");
        assert_eq!(pend, tb);
        assert_eq!(team_proceeds_due(&s, &pend, &slots::TeamParts::default(), 0).unwrap().0, 6_000, "the address slot's share is unmoved");
        assert_eq!(ecode(team_proceeds_due(&s, &pend, &slots::TeamParts::default(), 5).unwrap_err()), 7000 + HookError::BadSlot as u32);

        let src = include_str!("stair_owner.rs");
        let body = &src[src.find("pub fn sweep_payee_handler").unwrap()..src.find("pub fn withdraw_handler").unwrap()];
        let due = body.find("team_proceeds_due(&s, &t, &slots::read_parts(&fp)?, slot)?").expect("the due");
        let ret = body[due..].find("if due == 0 {").expect("the zero-due return") + due;
        assert!(ret < body.find("require!(s.pool_bound == 1").unwrap() && ret < body.find("team_tip").unwrap());
        assert!(!body.contains("StairPayeePending"), "the sweep never refuses a pending slot");
    }

///
///
///
///
///
    #[test]
    fn mr_s_corrupt_team_books_whose_lifetime_overflows_pay_nothing() {
        let mut t = team(&[(10_000, 0, 7)]);
        let mut s: FloorPool = bytemuck::Zeroable::zeroed();
        s.owner_owed = u64::MAX;
        s.owner_paid = 1;
        assert_eq!(team_proceeds_due(&s, &t, &slots::TeamParts::default(), 0).unwrap(), (0, Pubkey::new_from_array([7; 32])));
        let mut o: StairOwners = bytemuck::Zeroable::zeroed();
        o.set_held(u64::MAX);
        o.owner_withdrawn = 1;
        s.ask_tokens_deployed = u64::MAX;
        let (ob, sb, tb) = (o, s, t);
        assert_eq!(book_held_withdraw(&mut o, &mut s, &mut t, &slots::TeamParts::default(), 0).unwrap(), 0);
        assert_eq!(bytemuck::bytes_of(&o), bytemuck::bytes_of(&ob), "the books untouched");
        assert_eq!(bytemuck::bytes_of(&s), bytemuck::bytes_of(&sb));
        assert_eq!(t, tb);
    }

///
///
///
    #[test]
    fn mr_s_the_sweep_refuses_a_bad_slot_before_the_idle_no_op() {
        let src = include_str!("stair_owner.rs");
        let body = &src[src.find("pub fn sweep_payee_handler").unwrap()..];
        let check = body.find("slots::slot(&t, slots::SLOT_KIND_TEAM, slot)?").expect("the slot check");
        let idle = body.find("s.stair_owner == 0 || s.owner_owed == 0").expect("the idle no-op");
        assert!(check < idle, "BadSlot must come before the idle return");
    }

    #[test]
    fn mr_s_team_tokens_withdraw_per_slot_through_held() {
        let mut t = team(&[(2_500, 0, 1), (7_500, 0, 2)]);
        let mut o: StairOwners = bytemuck::Zeroable::zeroed();
        let mut s: FloorPool = bytemuck::Zeroable::zeroed();
        o.set_held(1_000_001);
        s.ask_tokens_deployed = 1_000_001;
        assert_eq!(book_held_withdraw(&mut o, &mut s, &mut t, &slots::TeamParts::default(), 1).unwrap(), 750_000);
        assert_eq!((o.held(), o.owner_withdrawn, s.ask_tokens_deployed, t.team[1].tokens_paid), (250_001, 750_000, 250_001, 750_000));
        assert_eq!(book_held_withdraw(&mut o, &mut s, &mut t, &slots::TeamParts::default(), 1).unwrap(), 0, "nothing more due: moves nothing");
        assert_eq!(o.held() + o.owner_withdrawn, 1_000_001, "conserved");

        o.set_held(o.held() + 1_000_000);
        s.ask_tokens_deployed += 1_000_000;
        assert_eq!(book_held_withdraw(&mut o, &mut s, &mut t, &slots::TeamParts::default(), 0).unwrap(), 500_000, "25 % of T, never drawn");
        assert_eq!(book_held_withdraw(&mut o, &mut s, &mut t, &slots::TeamParts::default(), 1).unwrap(), 750_000, "75 % of T − 750,000");
        assert_eq!((o.held(), s.ask_tokens_deployed), (1, 1), "one unit of dust stays held");
        assert_eq!(t.team[0].tokens_paid + t.team[1].tokens_paid + o.held(), 2_000_001);

        let mut one = team(&[(10_000, 0, 1)]);
        let mut o2: StairOwners = bytemuck::Zeroable::zeroed();
        let mut s2: FloorPool = bytemuck::Zeroable::zeroed();
        o2.set_held(40);
        o2.owner_withdrawn = 0;
        one.team[0].tokens_paid = 0;
        s2.ask_tokens_deployed = 40;
        assert_eq!(book_held_withdraw(&mut o2, &mut s2, &mut one, &slots::TeamParts::default(), 0).unwrap(), 40, "one slot at 10,000 takes everything held");
        assert_eq!(ecode(book_held_withdraw(&mut o2, &mut s2, &mut one, &slots::TeamParts::default(), 1).unwrap_err()), 7000 + HookError::BadSlot as u32);
    }

    #[test]
    fn only_an_unplaced_team_buy_pool_is_cleared() {
        let refused = 7000 + HookError::StairOwnerBadConfig as u32;
        let fresh = || {
            let mut p: FloorPool = bytemuck::Zeroable::zeroed();
            p.stair_owner = floor_types::constants::STAIR_OWNER_TEAM_BUY;
            p
        };
        assert!(check_clear_team_buy_books(&fresh()).is_ok());
        let mut p = fresh();
        p.stair_owner = floor_types::constants::STAIR_OWNER_TEAM;
        assert_eq!(code(check_clear_team_buy_books(&p)), refused, "$FLOOR's team books are never cleared");
        let mut p = fresh();
        p.stair_owner = 0;
        assert_eq!(code(check_clear_team_buy_books(&p)), refused, "a pool with no books");

        let mut p = fresh();
        p.pool_bound = 1;
        assert!(check_clear_team_buy_books(&p).is_ok(), "bound (prepared), nothing placed");
        for bound in [0u8, 1] {
            let mut p = fresh();
            p.pool_bound = bound;
            p.ladder_seeded = 1;
            assert_eq!(code(check_clear_team_buy_books(&p)), refused, "seeded (bound {bound})");
            let mut p = fresh();
            p.pool_bound = bound;
            p.stair_ready = 1;
            assert_eq!(code(check_clear_team_buy_books(&p)), refused, "placed (bound {bound})");
        }
    }

///
///
///
///
///
///
///
    #[test]
    fn a15_each_slot_withdraws_its_own_share_of_a_step() {
        let t0 = team(&[(5_000, 0, 1), (3_000, 0, 2), (1_999, 0, 3), (1, 0, 4)]);
        let mut t = t0;
        let mut o: StairOwners = bytemuck::Zeroable::zeroed();
        let mut s: FloorPool = bytemuck::Zeroable::zeroed();
        let mut p = slots::TeamParts::default();
        o.tokens[2] = 1_000_003;
        o.owner_total = 1_000_003;
        let want = [(3u8, 100u64), (0, 500_001), (2, 199_900), (1, 300_002)];
        let mut paid_total = 0u64;
        for (k, &(slot, x)) in want.iter().enumerate() {
            assert_eq!(step_part(&t, &p, o.tokens[2], 2, slot), x, "slot {slot}'s part");

            let take = if slot == 0 { x - 1 } else { x };
            book_exit(&mut o, &mut p, t.team_count, 2, slot, x, take).unwrap();
            s.ask_tokens_deployed += take;
            assert_eq!(step_part(&t, &p, o.tokens[2], 2, slot), 0, "out of the step now");

            for (other, _) in &want[k + 1..] {
                assert_eq!(book_held_withdraw(&mut o, &mut s, &mut t, &p, *other).unwrap(), 0, "slot {other} before its own exit");
            }
            let due = book_held_withdraw(&mut o, &mut s, &mut t, &p, slot).unwrap();
            assert_eq!(due, take, "slot {slot} is paid exactly what it took");
            paid_total += due;
            assert!(o.conserved(), "conservation after slot {slot}");
        }
        assert_eq!(o.tokens[2], 0);
        assert_eq!((paid_total, o.clamped(), o.held()), (1_000_002, 1, 0));
        assert_eq!(p.part_tokens, 1_000_002);
        assert_eq!(p.out_of(2), 0b1111);

        o.set_held(10_000);
        s.ask_tokens_deployed = 10_000;
        let dues: Vec<u64> = (0..4).map(|k| book_held_withdraw(&mut o, &mut s, &mut t, &p, k).unwrap()).collect();
        assert_eq!(dues, vec![5_000, 3_000, 1_999, 1]);

        let one = team(&[(10_000, 0, 1)]);
        let mut o1: StairOwners = bytemuck::Zeroable::zeroed();
        let mut p1 = slots::TeamParts::default();
        o1.tokens[5] = 777;
        assert_eq!(step_part(&one, &p1, 777, 5, 0), 777);
        book_exit(&mut o1, &mut p1, 1, 5, 0, 777, 777).unwrap();
        assert_eq!((o1.tokens[5], o1.held(), p1), (0, 777, slots::TeamParts::default()));
        assert_eq!(step_part(&t0, &slots::TeamParts::default(), 9, 2, 4), 0, "a slot past the count");
    }

///
///
    #[test]
    fn a15_mr_s_pv2_5_proceeds_dues_through_the_sweep_books() {
        let t = team(&[(5_000, 0, 1), (3_000, 0, 2), (1_999, 0, 3), (1, 0, 4)]);
        let mut s: FloorPool = bytemuck::Zeroable::zeroed();
        s.owner_owed = 900_000;
        let p = slots::TeamParts { part_quote: 547_824, quote_credit: [0, 328_695, 219_020, 109, 0], ..Default::default() };
        let dues: Vec<u64> = (0..4).map(|k| team_proceeds_due(&s, &t, &p, k).unwrap().0).collect();
        assert_eq!(dues, vec![176_088, 434_347, 289_419, 144]);
        assert_eq!(dues.iter().sum::<u64>(), 899_998);

        let bad = slots::TeamParts { part_quote: 900_001, ..p };
        assert_eq!(team_proceeds_due(&s, &t, &bad, 0).unwrap().0, 0);
    }
}
