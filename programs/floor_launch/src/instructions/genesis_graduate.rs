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
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};
use floor_types::constants::{
    GENESIS_LOCKS, GENESIS_POL_TOKENS, GENESIS_SALE_TOKENS, GENESIS_VESTED_TOKENS, LAUNCH_STATE_GENESIS_CLOSED,
    LAUNCH_STATE_GRADUATED,
};
use floor_types::{disc, seeds as sd};

use crate::constants::{
    FLOOR_HOOK_PROGRAM, FLOOR_VESTING_PROGRAM, GENESIS_SALE_SEED, GRAD_RENT_SEED, LAUNCH_SEED, SALE_VAULT_SEED,
};
use crate::errors::LaunchError;
use crate::events::{FloorFeesApplied, GenesisGraduated, VestingCreated};
use crate::graduate::{self, idx, r, r_s, w, w_s, Graduation};
use crate::graduation_rent;
use crate::lock::LOCK_AUTHORITY_SEED;
use crate::state::{GenesisSale, Launch};





#[event_cpi]
#[derive(Accounts)]
pub struct GraduateGenesis<'info> {
///
///
///
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, seeds = [GENESIS_SALE_SEED], bump)]
    pub genesis_sale: AccountLoader<'info, GenesisSale>,
///
    #[account(mut, address = genesis_sale.load()?.launch @ LaunchError::BadConfig)]
    pub launch: UncheckedAccount<'info>,
///
    #[account(mut, seeds = [SALE_VAULT_SEED, launch.key().as_ref()], bump)]
    pub sale_vault: UncheckedAccount<'info>,
    #[account(mut)]
    pub token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, address = genesis_sale.load()?.mint @ LaunchError::BadConfig)]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
///
    #[account(seeds = [LOCK_AUTHORITY_SEED], bump)]
    pub lock_authority: UncheckedAccount<'info>,
    pub token_2022: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

///
///
pub fn graduate_genesis<'info>(ctx: Context<'info, GraduateGenesis<'info>>) -> Result<()> {
    let ra = ctx.remaining_accounts;
    let a = &ctx.accounts;
    let snap = graduate::launch_snapshot(&a.launch.to_account_info())?;
    crate::instructions::genesis::require_state(&snap, LAUNCH_STATE_GENESIS_CLOSED)?;
    require_keys_eq!(a.token_vault.key(), snap.token_vault, LaunchError::BadConfig);
    let (raised, floor_share_bps, sale_end, open, locked, sold) = {
        let g = a.genesis_sale.load()?;

        require_keys_eq!(a.payer.key(), g.authority, LaunchError::GenesisGraduationDeployerOnly);
        require!(g.sale_closed == 1 && g.graduated == 0, LaunchError::GenesisStepOutOfOrder);


        require!(Clock::get()?.unix_timestamp < g.refund_opens_at(), LaunchError::GenesisRefundWindowOpen);
        require!(g.open_tick == snap.open_tick && g.raised == snap.raised, LaunchError::BadConfig);
        (g.raised, g.floor_share_bps, g.sale_end(), g.open_tick, g.locked == 1, g.sold_tokens)
    };

    let vested_held = if locked { 0 } else { GENESIS_VESTED_TOKENS };
    let checked = graduate::check_accounts_of(ra, &a.launch.key(), Some(&a.sale_vault.key()), &snap)?;



    require!(
        a.token_vault.amount == floor_types::constants::GENESIS_SUPPLY - GENESIS_VESTED_TOKENS + vested_held,
        LaunchError::BadConfig
    );
    let rent = Rent::get()?;
    let sale_reserve = rent.minimum_balance(a.sale_vault.to_account_info().data_len());
    require!(
        a.sale_vault.to_account_info().lamports() >= sale_reserve.checked_add(raised).ok_or(LaunchError::MathOverflow)?,
        LaunchError::EscrowShort
    );


    let reserve = &ra[idx::GRADUATION_RENT];
    let vault0_len = graduation_rent::quote_vault_len(&ra[idx::WSOL_MINT])?;
    let vault1_len = graduation_rent::base_vault_len(&a.mint.to_account_info())?;
    let quote_pools = graduation_rent::quote_pools(&snap.quote_mint());



    let stair = graduate::pool_stair_steps(&ra[idx::FLOOR_POOL])?;
    require!(stair != 0, LaunchError::StairDropped);


    let reserve_available = graduation_rent::require_funded(
        reserve,
        &rent,
        graduation_rent::outstanding(ra, &rent, vault0_len, vault1_len, quote_pools, true)
            .checked_add(graduation_rent::genesis_cushion_rent_lamports(&rent))

            .and_then(|x| x.checked_add(graduation_rent::owner_tail_outstanding(ra, &rent, true)))
            .ok_or(LaunchError::MathOverflow)?,
    )?;
    let payer_before = a.payer.lamports();

    let payer = a.payer.to_account_info();
    let launch = a.launch.to_account_info();
    let sale_vault = a.sale_vault.to_account_info();
    let token_vault = a.token_vault.to_account_info();
    let mint = a.mint.to_account_info();
    let lock_authority = a.lock_authority.to_account_info();
    let token_2022 = a.token_2022.to_account_info();
    let system_program = a.system_program.to_account_info();
    let event_authority = a.event_authority.to_account_info();
    let program = a.program.to_account_info();
    let g = Graduation {
        payer: &payer,
        launch: &launch,
        sale_vault: &sale_vault,
        token_vault: &token_vault,
        mint: &mint,
        lock_authority: &lock_authority,
        lock_bump: ctx.bumps.lock_authority,
        token_2022: &token_2022,
        system_program: &system_program,
        event_authority: &event_authority,
        program: &program,
        graduation_accounts: ra,
    };
    let launch_bump = [snap.bump];
    let launch_seeds: &[&[u8]] = &[LAUNCH_SEED, snap.mint.as_ref(), &launch_bump];


    {
        let slot = Clock::get()?.slot;
        let mut data = launch.try_borrow_mut_data()?;
        let l: &mut Launch = bytemuck::from_bytes_mut(&mut data[graduate::LAUNCH_BYTES]);
        l.state = LAUNCH_STATE_GRADUATED;
        l.graduated_slot = slot;
    }
    a.genesis_sale.load_mut()?.graduated = 1;


    graduate::lift_lock(&g)?;


    let sqrt_open =
        carpenter_clmm_math::tick_math::get_sqrt_price_at_tick(open).map_err(|_| LaunchError::MathOverflow)?;
    if graduation_rent::uncreated(&ra[idx::POOL]) {
        graduate::initialize_pool_cpi(&payer, &launch, &mint, &token_2022, &system_program, ra, sqrt_open, launch_seeds)?;
    } else {

        require!(graduate::pool_sqrt_price(&ra[idx::POOL])? == sqrt_open, LaunchError::PoolMoved);
    }


    let wsol_vault = &ra[idx::WSOL_VAULT];
    let lamports = sale_vault.lamports().checked_sub(sale_reserve).ok_or(LaunchError::EscrowShort)?;
    graduate::wrap_lamports(&payer, &sale_vault, wsol_vault, lamports)?;


    let seedable = graduate::token_amount(wsol_vault)?;
    let fees = seedable.checked_sub(raised).ok_or(LaunchError::EscrowShort)?;
    let (initial_floor, offered_pol) =
        floor_math::genesis_split(raised, floor_share_bps).map_err(|_| error!(LaunchError::MathOverflow))?;
    let returned_pol = offered_pol;


    if seedable > 0 {
        crate::instructions::genesis::seed_floor_cpi(
            &launch,
            &ra[idx::FLOOR_POOL],
            &ra[idx::POOL],
            wsol_vault,
            &ra[idx::VAULT0],
            &ra[idx::WSOL_MINT],
            &ra[idx::SPL_TOKEN],
            &ra[idx::CARPENTER_AMM],
            &ra[idx::AMM_EVENT_AUTHORITY],
            &ra[idx::HOOK_EVENT_AUTHORITY],
            &ra[idx::FLOOR_HOOK],
            seedable,
            launch_seeds,
        )?;
    }


    let (pol_lower, pol_upper) = graduate::pol_range(open).map_err(|_| error!(LaunchError::MathOverflow))?;
    let mut data = Vec::with_capacity(8 + 4 + 4 + 8 + 32);
    data.extend_from_slice(&disc::hook::SEED_LIQUIDITY);
    data.extend_from_slice(&pol_lower.to_le_bytes());
    data.extend_from_slice(&pol_upper.to_le_bytes());
    data.extend_from_slice(&GENESIS_POL_TOKENS.to_le_bytes());
    data.extend_from_slice(&snap.creator.to_bytes());
    let (ta_lo, ta_hi) = (
        graduate::find_tick_array(ra, &checked.pool, pol_lower)?,
        graduate::find_tick_array(ra, &checked.pool, pol_upper)?,
    );
    graduate::cpi(FLOOR_HOOK_PROGRAM, data, &graduate::seed_accounts(&g, Some(idx::POSITION_SEED), ta_lo, ta_hi), &[launch_seeds])?;










    let held_for_claims = sold;
    debug_assert!(held_for_claims <= GENESIS_SALE_TOKENS);
    let ladder_tokens = graduate::token_amount(&token_vault)?
        .checked_sub(vested_held)
        .and_then(|x| x.checked_sub(held_for_claims))
        .ok_or(LaunchError::MathOverflow)?;
    let (lad_lower, lad_upper) = graduate::ladder_range(open)?;

    let stair_start = open;
    let mut data = Vec::with_capacity(8 + 8 + 4 + 32);
    data.extend_from_slice(&disc::hook::SEED_INVENTORY);
    data.extend_from_slice(&ladder_tokens.to_le_bytes());
    data.extend_from_slice(&stair_start.to_le_bytes());
    data.extend_from_slice(&snap.creator.to_bytes());





    require!(
        graduate::pool_stair_owner(&ra[idx::FLOOR_POOL])? == floor_types::constants::STAIR_OWNER_TEAM,
        LaunchError::StairOwnerDropped
    );
    let team = team_owner_arg(&*a.genesis_sale.load()?, sale_end);
    data.extend_from_slice(&team.to_bytes());
    let (ta_lo, ta_hi) = (
        graduate::find_tick_array(ra, &checked.pool, lad_lower)?,
        graduate::find_tick_array(ra, &checked.pool, lad_upper)?,
    );
    let mut accs = graduate::seed_accounts(&g, None, ta_lo, ta_hi);
    graduate::stair_seed_accounts(ra, &checked.pool, graduate::spacing_floor(stair_start)?, stair, &mut accs)?;
    graduate::cpi(FLOOR_HOOK_PROGRAM, data, &accs, &[launch_seeds])?;






    let mut data = Vec::with_capacity(8 + 32);
    data.extend_from_slice(&disc::hook::RATCHET);
    data.extend_from_slice(&snap.creator.to_bytes());
    graduate::cpi(FLOOR_HOOK_PROGRAM, data, &graduate::ratchet_accounts(&g), &[])?;


    let (paused, has_authority) = graduate::pause_state(&mint)?;
    require!(!paused && !has_authority, LaunchError::CannotGraduate);
    require!(
        graduate::token_amount(&token_vault)? == vested_held + held_for_claims,
        LaunchError::CannotGraduate
    );
    require!(crate::lock::hook_slot(&mint)?.is_none(), LaunchError::CannotGraduate);


    let spent = payer_before.saturating_sub(payer.lamports());
    graduation_rent::pay(reserve, &payer, spent.min(reserve_available))?;

    {
        let mut gs = a.genesis_sale.load_mut()?;
        gs.initial_floor = initial_floor;
        gs.offered_pol = offered_pol;
        gs.returned_pol = returned_pol;
        gs.ladder_tokens = ladder_tokens;
        gs.fees_applied = gs.fees_applied.checked_add(fees).ok_or(LaunchError::MathOverflow)?;
    }
    if fees > 0 {
        let cumulative_applied = a.genesis_sale.load()?.fees_applied;
        graduate::emit_event(&g, &FloorFeesApplied { amount: fees, cumulative_applied })?;
    }
    graduate::emit_event(
        &g,
        &GenesisGraduated {
            sale_end,
            raised,
            external_fees: fees,
            initial_floor,
            offered_pol,
            returned_pol,
            ladder_tokens,
            open_tick: open,
        },
    )?;
    Ok(())
}

///
///
///
pub fn team_owner_arg(g: &crate::state::GenesisSale, sale_end: i64) -> floor_types::owners::StairOwnerArg {
    use floor_types::constants::*;

    let (payee, pending, platform) = match g.payee_state {
        TEAM_PAYEE_PENDING => ([0u8; 32], g.payee.to_bytes(), g.payee_platform),
        _ => (g.payee.to_bytes(), [0u8; 32], 0),
    };
    floor_types::owners::StairOwnerArg {
        kind: STAIR_OWNER_TEAM,

        counted: 1,
        payee_bps: GENESIS_TEAM_PAYEE_BPS,
        first_unlock_secs: GENESIS_TEAM_FIRST_CLIFF_SECS,
        every_secs: GENESIS_TEAM_CLIFF_EVERY_SECS,
        anchor: sale_end,
        owner_tokens: GENESIS_TEAM_STAIR_TOKENS,
        payee,
        payee_pending_id: pending,
        payee_platform: platform,


    }
}





#[event_cpi]
#[derive(Accounts)]
pub struct LockGenesis<'info> {
///
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, seeds = [GENESIS_SALE_SEED], bump)]
    pub genesis_sale: AccountLoader<'info, GenesisSale>,
    #[account(address = genesis_sale.load()?.launch @ LaunchError::BadConfig)]
    pub launch: AccountLoader<'info, Launch>,
///
    #[account(mut, address = genesis_sale.load()?.mint @ LaunchError::BadConfig)]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, address = launch.load()?.token_vault @ LaunchError::BadConfig)]
    pub token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
///
    #[account(address = genesis_sale.load()?.beneficiary @ LaunchError::BadConfig)]
    pub beneficiary: UncheckedAccount<'info>,
///
    #[account(mut, seeds = [GRAD_RENT_SEED, launch.key().as_ref()], bump)]
    pub graduation_rent: UncheckedAccount<'info>,
///
    #[account(address = FLOOR_VESTING_PROGRAM)]
    pub floor_vesting: UncheckedAccount<'info>,
///
    #[account(seeds = [sd::VESTING_AUTHORITY], bump, seeds::program = FLOOR_VESTING_PROGRAM)]
    pub vesting_authority: UncheckedAccount<'info>,
///
    #[account(seeds = [sd::EVENT_AUTHORITY], bump, seeds::program = FLOOR_VESTING_PROGRAM)]
    pub vesting_event_authority: UncheckedAccount<'info>,
    pub token_2022: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
///
///
    #[account(seeds = [LOCK_AUTHORITY_SEED], bump)]
    pub lock_authority: UncheckedAccount<'info>,
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
pub fn lock_genesis<'info>(ctx: Context<'info, LockGenesis<'info>>) -> Result<()> {
    let a = &ctx.accounts;
    let (start, graduated) = {
        let g = a.genesis_sale.load()?;

        require!(g.locked == 0 && g.sale_closed == 1, LaunchError::GenesisStepOutOfOrder);
        (g.sale_end(), g.graduated == 1)
    };
    let ra = ctx.remaining_accounts;
    require!(ra.len() == 2 * GENESIS_LOCKS.len(), LaunchError::BadConfig);
    let launch_key = a.launch.key();
    let (mint_key, bump) = {
        let l = a.launch.load()?;
        (l.mint, l.bump)
    };
    let bump = [bump];
    let seeds: &[&[u8]] = &[LAUNCH_SEED, mint_key.as_ref(), &bump];

    let rent = Rent::get()?;
    let reserve = a.graduation_rent.to_account_info();
    let escrow_len = graduation_rent::base_vault_len(&a.mint.to_account_info())?;
    let reserve_available = graduation_rent::require_funded(
        &reserve,
        &rent,
        crate::instructions::genesis::lock_rent(&rent, escrow_len),
    )?;
    let payer = a.payer.to_account_info();
    let payer_before = payer.lamports();

    let launch = a.launch.to_account_info();
    let beneficiary_info = a.beneficiary.to_account_info();
    let mint = a.mint.to_account_info();
    let vesting_authority = a.vesting_authority.to_account_info();
    let token_vault = a.token_vault.to_account_info();
    let token_2022 = a.token_2022.to_account_info();
    let system_program = a.system_program.to_account_info();
    let vesting_ev = a.vesting_event_authority.to_account_info();
    let vesting = a.floor_vesting.to_account_info();

    let mut schedules = [Pubkey::default(); 5];
    let mut run = || -> Result<()> {
        for (i, (amount, cliff, linear)) in GENESIS_LOCKS.iter().enumerate() {
            let nonce = i as u64;
            let (schedule_key, _) = Pubkey::find_program_address(
                &[sd::VESTING_SCHEDULE, launch_key.as_ref(), &nonce.to_le_bytes()],
                &FLOOR_VESTING_PROGRAM,
            );
            let (escrow_key, _) =
                Pubkey::find_program_address(&[sd::VESTING_ESCROW, schedule_key.as_ref()], &FLOOR_VESTING_PROGRAM);
            let (schedule, escrow) = (&ra[2 * i], &ra[2 * i + 1]);
            require_keys_eq!(schedule.key(), schedule_key, LaunchError::BadConfig);
            require_keys_eq!(escrow.key(), escrow_key, LaunchError::BadConfig);

            let mut data = disc::vesting::CREATE_SCHEDULE_AT.to_vec();
            data.extend_from_slice(&nonce.to_le_bytes());
            data.extend_from_slice(&amount.to_le_bytes());
            data.extend_from_slice(&start.to_le_bytes());
            data.extend_from_slice(&cliff.to_le_bytes());
            data.extend_from_slice(&linear.to_le_bytes());



            graduate::cpi(
                FLOOR_VESTING_PROGRAM,
                data,
                &[
                    w_s(&payer),
                    r_s(&launch),
                    r(&beneficiary_info),
                    r(&mint),
                    w(schedule),
                    r(&vesting_authority),
                    w(escrow),
                    w(&token_vault),
                    r(&token_2022),
                    r(&system_program),
                    r(&vesting_ev),
                    r(&vesting),
                ],
                &[seeds],
            )?;
            schedules[i] = schedule_key;
        }
        Ok(())
    };
    if graduated {
        run()?;
    } else {

        let lock = crate::lock::Lock {
            token_2022: &token_2022,
            mint: &mint,
            lock_authority: &a.lock_authority.to_account_info(),
            bump: ctx.bumps.lock_authority,
        };
        crate::lock::around_transfer(&lock, true, run)?;
    }

    let spent = payer_before.saturating_sub(payer.lamports());
    graduation_rent::pay(&reserve, &payer, spent.min(reserve_available))?;
    a.genesis_sale.load_mut()?.locked = 1;
    emit_cpi!(VestingCreated { schedules });
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use floor_types::constants::{TEAM_PAYEE_DEPLOYER, TEAM_PAYEE_PENDING, TEAM_PAYEE_RESOLVED};

///
///
///
///
///
    #[test]
    fn mr_s_genesis_team_slot_zero_is_the_payee_or_its_pending_id_never_both() {
        let mut g: crate::state::GenesisSale = bytemuck::Zeroable::zeroed();
        g.payee = Pubkey::new_from_array([4; 32]);
        for state in [TEAM_PAYEE_DEPLOYER, TEAM_PAYEE_RESOLVED] {
            g.payee_state = state;
            let a = team_owner_arg(&g, 77);
            assert_eq!((a.payee, a.payee_pending_id), ([4; 32], [0; 32]), "state {state}: a wallet");
            assert_eq!(a.anchor, 77, "the cliffs count from the sale end");
        }
        g.payee_platform = 3;
        for state in [TEAM_PAYEE_DEPLOYER, TEAM_PAYEE_RESOLVED] {
            g.payee_state = state;
            assert_eq!(team_owner_arg(&g, 77).payee_platform, 0, "state {state}: a wallet carries platform 0");
        }
        g.payee_state = TEAM_PAYEE_PENDING;
        let a = team_owner_arg(&g, 77);
        assert_eq!((a.payee, a.payee_pending_id), ([0; 32], [4; 32]), "pending: the id, no wallet");
        assert_eq!(a.payee_platform, 3, "pending: the handle's platform (owner ruling 2026-10-11)");
    }
}
