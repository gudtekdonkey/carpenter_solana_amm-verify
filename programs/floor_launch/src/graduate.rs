//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!

use core::mem::size_of;
use core::ops::DerefMut;

use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::token_2022::spl_token_2022;
use spl_token_2022::extension::pausable::PausableConfig;
use spl_token_2022::extension::{BaseStateWithExtensions, StateWithExtensions};

use floor_types::constants::{
    DYNAMIC_FEE_FLAG, FLOOR_TICK_SPACING, LAUNCH_STATE_BONDING,




    LAUNCH_STATE_GRADUATED, MIN_TICK, SALT_BID, SALT_SEED, SALT_STAIR_RESERVE, SALT_STAIR_STEP0, SALT_STAIR_TAIL,
};
use floor_types::layout::tick_array_start_index;
use floor_types::{disc, ids, keys, seeds as sd};

use crate::constants::{
    CARPENTER_AMM_PROGRAM, FLOOR_HOLDERS_PROGRAM, FLOOR_HOOK_PROGRAM, GRAD_RENT_SEED, LAUNCH_SEED, SALE_VAULT_SEED,
};
use crate::graduation_rent;
use crate::errors::LaunchError;
use crate::events::Graduated;
use crate::lock::{self, Lock};
use crate::state::Launch;

///
///
pub mod idx {
    pub const LAUNCH_CONFIG: usize = 0;
    pub const CARPENTER_AMM: usize = 1;
    pub const AMM_CONFIG: usize = 2;
    pub const APPROVED_HOOK: usize = 3;
    pub const HOOK_PROGRAMDATA: usize = 4;
    pub const POOL: usize = 5;
    pub const BITMAP: usize = 6;
    pub const VAULT0: usize = 7;
    pub const VAULT1: usize = 8;
    pub const POOL_INDEX_ENTRY: usize = 9;
///
///
    pub const WSOL_MINT: usize = 10;
///
    pub const SPL_TOKEN: usize = 11;
    pub const MINT_EXCLUSIVITY0: usize = 12;
    pub const MINT_EXCLUSIVITY1: usize = 13;
    pub const MINT_POOLS0: usize = 14;
    pub const MINT_POOLS1: usize = 15;
    pub const FLOOR_HOOK: usize = 16;
    pub const HOOK_CONFIG: usize = 17;
    pub const FLOOR_POOL: usize = 18;
    pub const HOOK_AUTHORITY: usize = 19;
    pub const POSITION_SEED: usize = 20;
///
///
///
    pub const POSITION_BID: usize = 21;
    pub const SYSTEM_PROGRAM: usize = 22;
    pub const AMM_EVENT_AUTHORITY: usize = 23;
    pub const HOOK_EVENT_AUTHORITY: usize = 24;
///
///
    pub const WSOL_VAULT: usize = 25;
///
    pub const GRADUATION_RENT: usize = 26;
///
///
///
///
///
///
    pub const QUOTE_MINT0: usize = 27;
    pub const QUOTE_MINT1: usize = 28;
///
///
///
///
///
///
///
    pub const STAIR_STATE: usize = 29;
    pub const POSITION_STAIR_STEP0: usize = 30;
    pub const POSITION_STAIR_RESERVE: usize = 31;
    pub const POSITION_STAIR_TAIL: usize = 32;
///
    pub const TICK_ARRAYS: usize = 33;
}

///
pub const GRADUATION_FIXED_ACCOUNTS: usize = idx::TICK_ARRAYS;

///
pub const WRITABLE_FIXED: [usize; 15] = [
    idx::POOL,
    idx::BITMAP,
    idx::VAULT0,
    idx::VAULT1,
    idx::POOL_INDEX_ENTRY,
    idx::MINT_POOLS1,
    idx::FLOOR_POOL,
    idx::POSITION_SEED,
    idx::POSITION_BID,
    idx::WSOL_VAULT,
    idx::GRADUATION_RENT,

    idx::STAIR_STATE,
    idx::POSITION_STAIR_STEP0,
    idx::POSITION_STAIR_RESERVE,
    idx::POSITION_STAIR_TAIL,
];



const fn pk(b: [u8; 32]) -> Pubkey {
    Pubkey::new_from_array(b)
}

const SPL_TOKEN_PROGRAM: Pubkey = pk(keys::SPL_TOKEN_PROGRAM_ID);
const TOKEN_2022_PROGRAM: Pubkey = pk(keys::TOKEN_2022_PROGRAM_ID);
const ATA_PROGRAM: Pubkey = pk(keys::ASSOCIATED_TOKEN_PROGRAM_ID);
const SYSTEM_PROGRAM: Pubkey = pk(keys::SYSTEM_PROGRAM_ID);

const LAUNCH_CONFIG_KEY: Pubkey =
    pk(anchor_lang::derive_program_address(&[sd::LAUNCH_CONFIG], &ids::FLOOR_LAUNCH_ID).0);
const AMM_CONFIG_KEY: Pubkey = pk(anchor_lang::derive_program_address(&[sd::AMM_CONFIG], &ids::CARPENTER_AMM_ID).0);
const APPROVED_HOOK_KEY: Pubkey =
    pk(anchor_lang::derive_program_address(&[sd::APPROVED_HOOK, &ids::FLOOR_HOOK_ID], &ids::CARPENTER_AMM_ID).0);
///
const HOOK_PROGRAMDATA_KEY: Pubkey =
    pk(anchor_lang::derive_program_address(&[&ids::FLOOR_HOOK_ID], &keys::BPF_LOADER_UPGRADEABLE_ID).0);
///
///
const MINT_EXCLUSIVITY0_KEY: Pubkey =
    pk(anchor_lang::derive_program_address(&[sd::MINT_EXCLUSIVITY, &keys::WSOL_MINT], &ids::CARPENTER_AMM_ID).0);
const MINT_POOLS0_KEY: Pubkey =
    pk(anchor_lang::derive_program_address(&[sd::MINT_POOLS, &keys::WSOL_MINT], &ids::CARPENTER_AMM_ID).0);
const HOOK_CONFIG_KEY: Pubkey = pk(anchor_lang::derive_program_address(&[sd::HOOK_CONFIG], &ids::FLOOR_HOOK_ID).0);
const HOOK_AUTHORITY_KEY: Pubkey =
    pk(anchor_lang::derive_program_address(&[sd::HOOK_AUTHORITY], &ids::FLOOR_HOOK_ID).0);
const AMM_EVENT_AUTHORITY_KEY: Pubkey =
    pk(anchor_lang::derive_program_address(&[sd::EVENT_AUTHORITY], &ids::CARPENTER_AMM_ID).0);
const HOOK_EVENT_AUTHORITY_KEY: Pubkey =
    pk(anchor_lang::derive_program_address(&[sd::EVENT_AUTHORITY], &ids::FLOOR_HOOK_ID).0);

///
const SPL_SYNC_NATIVE_TAG: u8 = 17;
///
const SYSTEM_TRANSFER_TAG: [u8; 4] = 2u32.to_le_bytes();
///
const TOKEN_ACCOUNT_AMOUNT: core::ops::Range<usize> = 64..72;
///
pub(crate) const LAUNCH_BYTES: core::ops::Range<usize> = 8..8 + size_of::<Launch>();

///
///
///
///
///
///
pub struct Graduation<'a, 'info> {
///
    pub payer: &'a AccountInfo<'info>,
///
///
    pub launch: &'a AccountInfo<'info>,
    pub sale_vault: &'a AccountInfo<'info>,
    pub token_vault: &'a AccountInfo<'info>,
    pub mint: &'a AccountInfo<'info>,
    pub lock_authority: &'a AccountInfo<'info>,
///
    pub lock_bump: u8,
    pub token_2022: &'a AccountInfo<'info>,
    pub system_program: &'a AccountInfo<'info>,
///
    pub event_authority: &'a AccountInfo<'info>,
    pub program: &'a AccountInfo<'info>,
///
    pub graduation_accounts: &'a [AccountInfo<'info>],
}

///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraduationOutcome {
    pub raised: u64,
///
    pub seeded: u64,
///
    pub team: u64,
    pub open_tick: i32,
    pub ladder_tokens: u64,
}

///
///
pub fn run(g: &Graduation<'_, '_>) -> Result<GraduationOutcome> {
    graduate_inner(g)
}

fn graduate_inner(g: &Graduation<'_, '_>) -> Result<GraduationOutcome> {
    let ra = g.graduation_accounts;
    if ra.len() < GRADUATION_FIXED_ACCOUNTS {
        return Err(error!(ErrorCode::AccountNotEnoughKeys).with_account_name("graduation accounts (§7.4)"));
    }


    let snap = launch_snapshot(g.launch)?;




    require!(snap.state == LAUNCH_STATE_BONDING, LaunchError::CannotGraduate);
    require!(snap.sold == snap.sale_tokens, LaunchError::CannotGraduate);
    require_keys_eq!(g.mint.key(), snap.mint, LaunchError::BadConfig);
    require_keys_eq!(g.token_vault.key(), snap.token_vault, LaunchError::BadConfig);
    require_keys_eq!(g.token_2022.key(), TOKEN_2022_PROGRAM, LaunchError::BadConfig);
    require_keys_eq!(g.system_program.key(), SYSTEM_PROGRAM, LaunchError::BadConfig);
    require_keys_eq!(g.program.key(), crate::ID, LaunchError::BadConfig);

    let accounts = check_accounts(g, &snap)?;




    let rent = Rent::get()?;
    let rent_reserve = &g.graduation_accounts[idx::GRADUATION_RENT];




    let vault0_len = graduation_rent::quote_vault_len(&ra[idx::WSOL_MINT])?;

    let vault1_len = graduation_rent::base_vault_len(g.mint)?;
    let quote_pools = graduation_rent::quote_pools(&snap.quote_mint());


    let stair = pool_stair_steps(&ra[idx::FLOOR_POOL])? != 0;

    let owner = pool_stair_owner(&ra[idx::FLOOR_POOL])? != 0;
    let reserve_available = graduation_rent::require_funded(
        rent_reserve,
        &rent,
        graduation_rent::outstanding(ra, &rent, vault0_len, vault1_len, quote_pools, stair)
            .checked_add(graduation_rent::owner_tail_outstanding(ra, &rent, owner))


            .and_then(|x| x.checked_add(snap.stair_rent_budget as u64))
            .ok_or(LaunchError::MathOverflow)?,
    )?;


    let payer_before = g.payer.lamports();

    let open = snap.open_tick;
    let launch_key = g.launch.key();
    let mint_key = snap.mint;
    let launch_bump = [snap.bump];
    let launch_seeds: &[&[u8]] = &[LAUNCH_SEED, mint_key.as_ref(), &launch_bump];


    {
        let slot = Clock::get()?.slot;
        let mut data = g.launch.try_borrow_mut_data()?;

        let l: &mut Launch = bytemuck::from_bytes_mut(&mut data.deref_mut()[LAUNCH_BYTES]);
        l.state = LAUNCH_STATE_GRADUATED;
        l.graduated_slot = slot;
        l.team_owed = 0;

        l.team_anchor = Clock::get()?.unix_timestamp;
    }





    lift_lock(g)?;


    let sqrt_open = carpenter_clmm_math::tick_math::get_sqrt_price_at_tick(open).map_err(|_| LaunchError::MathOverflow)?;
    if graduation_rent::uncreated(&ra[idx::POOL]) {
        initialize_pool_cpi(g.payer, g.launch, g.mint, g.token_2022, g.system_program, ra, sqrt_open, launch_seeds)?;
    } else {



















        require!(pool_sqrt_price(&ra[idx::POOL])? == sqrt_open, LaunchError::PoolMoved);
    }


    let wsol_vault = &ra[idx::WSOL_VAULT];
    let seedable = if snap.native_quote() {
        wrap_native_escrow(g, &snap, wsol_vault)?
    } else {







        let held = token_amount(wsol_vault)?;
        require!(held >= snap.raised.checked_add(snap.team_owed).ok_or(LaunchError::MathOverflow)?, LaunchError::EscrowShort);
        held
    };




    let mut data = disc::hook::SEED_FLOOR.to_vec();
    data.extend_from_slice(&seedable.to_le_bytes());
    data.extend_from_slice(&snap.team_owed.to_le_bytes());
    cpi(
        FLOOR_HOOK_PROGRAM,
        data,
        &[
            r_s(g.launch),
            w(&ra[idx::FLOOR_POOL]),
            w(&ra[idx::POOL]),
            w(wsol_vault),
            w(&ra[idx::VAULT0]),
            r(&ra[idx::WSOL_MINT]),
            r(&ra[idx::SPL_TOKEN]),
            r(&ra[idx::CARPENTER_AMM]),
            r(&ra[idx::AMM_EVENT_AUTHORITY]),
            r(&ra[idx::HOOK_EVENT_AUTHORITY]),
            r(&ra[idx::FLOOR_HOOK]),
        ],
        &[launch_seeds],
    )?;



    let (pol_lower, pol_upper) = pol_range(open).map_err(|_| error!(LaunchError::MathOverflow))?;






    let mut data = Vec::with_capacity(8 + 4 + 4 + 8 + 32);
    data.extend_from_slice(&disc::hook::SEED_LIQUIDITY);
    data.extend_from_slice(&pol_lower.to_le_bytes());
    data.extend_from_slice(&pol_upper.to_le_bytes());
    data.extend_from_slice(&snap.pol_tokens.to_le_bytes());
    data.extend_from_slice(&snap.creator.to_bytes());
    let (ta_lo, ta_hi) = (
        find_tick_array(ra, &accounts.pool, pol_lower)?,
        find_tick_array(ra, &accounts.pool, pol_upper)?,
    );
    cpi(FLOOR_HOOK_PROGRAM, data, &seed_accounts(g, Some(idx::POSITION_SEED), ta_lo, ta_hi), &[launch_seeds])?;







    let ladder_tokens = token_amount(g.token_vault)?.checked_sub(snap.team_wallet_tokens).ok_or(LaunchError::MathOverflow)?;
    let (lad_lower, lad_upper) = ladder_range(open)?;
    let stair = pool_stair_steps(&ra[idx::FLOOR_POOL])?;
    let wall_start = open;
    let mut data = Vec::with_capacity(8 + 8 + 4 + 32);
    data.extend_from_slice(&disc::hook::SEED_INVENTORY);
    data.extend_from_slice(&ladder_tokens.to_le_bytes());
    data.extend_from_slice(&wall_start.to_le_bytes());
    data.extend_from_slice(&snap.creator.to_bytes());

    if snap.team_step_tokens != 0 {
        data.extend_from_slice(&team_buy_owner_arg(&snap).to_bytes());
    }
    let (ta_lo, ta_hi) = (
        find_tick_array(ra, &accounts.pool, lad_lower)?,
        find_tick_array(ra, &accounts.pool, lad_upper)?,
    );
    let mut accs = seed_accounts(g, None, ta_lo, ta_hi);



    if stair != 0 {
        stair_seed_accounts(ra, &accounts.pool, spacing_floor(wall_start)?, stair, &mut accs)?;
    }
    cpi(FLOOR_HOOK_PROGRAM, data, &accs, &[launch_seeds])?;




    let mut data = Vec::with_capacity(8 + 32);
    data.extend_from_slice(&disc::hook::RATCHET);
    data.extend_from_slice(&snap.creator.to_bytes());
    cpi(FLOOR_HOOK_PROGRAM, data, &ratchet_accounts(g), &[])?;






    let hooked = lock::hook_slot(g.mint)?.is_some();
    if hooked {
        arm_holders_hook(g)?;
    }


    let (paused, has_authority) = pause_state(g.mint)?;
    require!(!paused && !has_authority, LaunchError::CannotGraduate);

    require!(token_amount(g.token_vault)? == snap.team_wallet_tokens, LaunchError::CannotGraduate);


    let slot = lock::hook_slot(g.mint)?;
    let want = hooked.then_some(lock::HookSlot { authority: None, program_id: Some(FLOOR_HOLDERS_PROGRAM) });
    require!(slot == want, LaunchError::HookNotArmed);





    let spent = payer_before.saturating_sub(g.payer.lamports());
    graduation_rent::pay(rent_reserve, g.payer, spent.min(reserve_available))?;


    let out = GraduationOutcome {
        raised: snap.raised,
        seeded: seedable.checked_sub(snap.team_owed).ok_or(LaunchError::MathOverflow)?,
        team: snap.team_owed,
        open_tick: open,
        ladder_tokens,
    };
    emit_event(
        g,
        &Graduated {
            launch: launch_key,
            raised: out.raised,
            seeded: out.seeded,
            open_tick: out.open_tick,
            ladder_tokens: out.ladder_tokens,
            pause_authority_revoked: true,
        },
    )?;
    Ok(out)
}

///
///
///
pub(crate) fn wrap_native_escrow<'info>(g: &Graduation<'_, 'info>, snap: &Launch, wsol_vault: &AccountInfo<'info>) -> Result<u64> {
    let reserve = Rent::get()?.minimum_balance(g.sale_vault.data_len());
    let seedable = g.sale_vault.lamports().checked_sub(reserve).ok_or(LaunchError::EscrowShort)?;
    require!(seedable >= snap.raised.checked_add(snap.team_owed).ok_or(LaunchError::MathOverflow)?, LaunchError::EscrowShort);
    let sale_after = g.sale_vault.lamports().checked_sub(seedable).ok_or(LaunchError::EscrowShort)?;
    let payer_after = g.payer.lamports().checked_add(seedable).ok_or(LaunchError::MathOverflow)?;















    **g.sale_vault.try_borrow_mut_lamports()? = sale_after;
    **g.payer.try_borrow_mut_lamports()? = payer_after;
    let mut data = SYSTEM_TRANSFER_TAG.to_vec();
    data.extend_from_slice(&seedable.to_le_bytes());
    cpi(SYSTEM_PROGRAM, data, &[w_s(g.payer), w(wsol_vault), w(g.sale_vault)], &[])?;
    cpi(SPL_TOKEN_PROGRAM, vec![SPL_SYNC_NATIVE_TAG], &[w(wsol_vault)], &[])?;
    Ok(seedable)
}

///
///
///
///
pub(crate) fn wrap_lamports<'info>(
    payer: &AccountInfo<'info>,
    sale_vault: &AccountInfo<'info>,
    wsol_vault: &AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }
    let sale_after = sale_vault.lamports().checked_sub(amount).ok_or(LaunchError::EscrowShort)?;
    let payer_after = payer.lamports().checked_add(amount).ok_or(LaunchError::MathOverflow)?;
    **sale_vault.try_borrow_mut_lamports()? = sale_after;
    **payer.try_borrow_mut_lamports()? = payer_after;
    let mut data = SYSTEM_TRANSFER_TAG.to_vec();
    data.extend_from_slice(&amount.to_le_bytes());
    cpi(SYSTEM_PROGRAM, data, &[w_s(payer), w(wsol_vault), w(sale_vault)], &[])?;
    cpi(SPL_TOKEN_PROGRAM, vec![SPL_SYNC_NATIVE_TAG], &[w(wsol_vault)], &[])?;
    Ok(())
}

///
///
///
///
///
///
///
#[allow(clippy::too_many_arguments)]
pub(crate) fn initialize_pool_cpi<'info>(
    payer: &AccountInfo<'info>,
    launch: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    token_2022: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    ra: &[AccountInfo<'info>],
    sqrt_open: u128,
    launch_seeds: &[&[u8]],
) -> Result<()> {
    let mut data = disc::amm::INITIALIZE_POOL.to_vec();
    data.extend_from_slice(&DYNAMIC_FEE_FLAG.to_le_bytes());
    data.extend_from_slice(&FLOOR_TICK_SPACING.to_le_bytes());
    data.extend_from_slice(FLOOR_HOOK_PROGRAM.as_ref());
    data.extend_from_slice(&sqrt_open.to_le_bytes());











    cpi(
        CARPENTER_AMM_PROGRAM,
        data,
        &[
            w_s(payer),
            r_s(launch),
            r(&ra[idx::AMM_CONFIG]),
            w(&ra[idx::POOL]),
            w(&ra[idx::BITMAP]),
            w(&ra[idx::VAULT0]),
            w(&ra[idx::VAULT1]),
            r(&ra[idx::WSOL_MINT]),
            r(mint),
            r(&ra[idx::SPL_TOKEN]),
            r(token_2022),
            r(&ra[idx::MINT_EXCLUSIVITY0]),
            r(&ra[idx::MINT_EXCLUSIVITY1]),





            (&ra[idx::MINT_POOLS0], ra[idx::MINT_POOLS0].is_writable, false),
            w(&ra[idx::MINT_POOLS1]),
            w(&ra[idx::POOL_INDEX_ENTRY]),
            r(&ra[idx::APPROVED_HOOK]),
            r(&ra[idx::FLOOR_HOOK]),
            r(&ra[idx::HOOK_PROGRAMDATA]),
            r(system_program),

            r(&ra[idx::QUOTE_MINT0]),
            r(&ra[idx::QUOTE_MINT1]),
            r(&ra[idx::AMM_EVENT_AUTHORITY]),
            r(&ra[idx::CARPENTER_AMM]),
            w(&ra[idx::FLOOR_POOL]),
            r(&ra[idx::HOOK_CONFIG]),
            r(&ra[idx::HOOK_EVENT_AUTHORITY]),
            r(&ra[idx::FLOOR_HOOK]),
        ],
        &[launch_seeds],
    )?;    Ok(())
}



///
///
pub(crate) fn launch_snapshot(info: &AccountInfo<'_>) -> Result<Launch> {
    if info.owner != &crate::ID {
        return Err(error!(ErrorCode::AccountOwnedByWrongProgram).with_account_name("launch"));
    }
    if !info.is_writable {
        return Err(error!(ErrorCode::AccountNotMutable).with_account_name("launch"));
    }
    let data = info.try_borrow_data()?;
    if data.len() < LAUNCH_BYTES.end {
        return Err(error!(ErrorCode::AccountDiscriminatorNotFound).with_account_name("launch"));
    }
    if &data[..8] != Launch::DISCRIMINATOR {
        return Err(error!(ErrorCode::AccountDiscriminatorMismatch).with_account_name("launch"));
    }
    Ok(bytemuck::pod_read_unaligned::<Launch>(&data[LAUNCH_BYTES]))
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
pub(crate) fn lift_lock(g: &Graduation<'_, '_>) -> Result<()> {
    lock::lift(&Lock {
        token_2022: g.token_2022,
        mint: g.mint,
        lock_authority: g.lock_authority,
        bump: g.lock_bump,
    })
}

///
///
///
///
fn arm_holders_hook(g: &Graduation<'_, '_>) -> Result<()> {
    lock::arm_hook(
        &Lock { token_2022: g.token_2022, mint: g.mint, lock_authority: g.lock_authority, bump: g.lock_bump },
        &FLOOR_HOLDERS_PROGRAM,
    )
}

///
pub(crate) fn pause_state(mint: &AccountInfo<'_>) -> Result<(bool, bool)> {
    let data = mint.try_borrow_data()?;
    let state = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&data)?;
    let cfg = state.get_extension::<PausableConfig>()?;
    Ok((bool::from(cfg.paused), cfg.authority.0.to_bytes() != [0u8; 32]))
}



///
pub(crate) struct Checked {
    pub(crate) pool: Pubkey,
}

fn check_accounts(g: &Graduation<'_, '_>, snap: &Launch) -> Result<Checked> {
    check_accounts_of(g.graduation_accounts, &g.launch.key(), Some(&g.sale_vault.key()), snap)
}

///
///
///
///
///
pub(crate) fn check_accounts_of(
    ra: &[AccountInfo<'_>],
    launch_key: &Pubkey,
    sale_vault_key: Option<&Pubkey>,
    snap: &Launch,
) -> Result<Checked> {
    if ra.len() < GRADUATION_FIXED_ACCOUNTS {
        return Err(error!(ErrorCode::AccountNotEnoughKeys).with_account_name("graduation accounts (§7.4)"));
    }
    let mint = snap.mint;




    let quote = snap.quote_mint();
    let quote_program = if snap.native_quote() {
        SPL_TOKEN_PROGRAM
    } else {
        require_keys_eq!(ra[idx::WSOL_MINT].key(), quote, LaunchError::BadConfig);
        let owner = *ra[idx::WSOL_MINT].owner;
        require!(owner == SPL_TOKEN_PROGRAM || owner == TOKEN_2022_PROGRAM, LaunchError::BadConfig);
        owner
    };
    let (mint_exclusivity0, mint_pools0) = if snap.native_quote() {
        (MINT_EXCLUSIVITY0_KEY, MINT_POOLS0_KEY)
    } else {
        (
            Pubkey::find_program_address(&[sd::MINT_EXCLUSIVITY, quote.as_ref()], &CARPENTER_AMM_PROGRAM).0,
            Pubkey::find_program_address(&[sd::MINT_POOLS, quote.as_ref()], &CARPENTER_AMM_PROGRAM).0,
        )
    };


    let pool = Pubkey::find_program_address(
        &[
            sd::POOL,
            quote.as_ref(),
            mint.as_ref(),
            &DYNAMIC_FEE_FLAG.to_le_bytes(),
            &FLOOR_TICK_SPACING.to_le_bytes(),
            FLOOR_HOOK_PROGRAM.as_ref(),
        ],
        &CARPENTER_AMM_PROGRAM,
    )
    .0;
    require_keys_eq!(pool, snap.amm_pool, LaunchError::BadConfig);
    let amm = |seeds: &[&[u8]]| Pubkey::find_program_address(seeds, &CARPENTER_AMM_PROGRAM).0;
    let position = |salt: u64| amm(&[sd::POSITION, pool.as_ref(), HOOK_AUTHORITY_KEY.as_ref(), &salt.to_le_bytes()]);
    let floor_pool = Pubkey::find_program_address(&[sd::FLOOR_POOL, pool.as_ref()], &FLOOR_HOOK_PROGRAM).0;
    require_keys_eq!(floor_pool, snap.floor_pool, LaunchError::BadConfig);
    let launch_key = *launch_key;
    let wsol_vault = Pubkey::find_program_address(
        &[launch_key.as_ref(), quote_program.as_ref(), quote.as_ref()],
        &ATA_PROGRAM,
    )
    .0;
    require_keys_eq!(wsol_vault, snap.wsol_vault, LaunchError::BadConfig);




    if let Some(got) = sale_vault_key {
        let sale_vault =
            Pubkey::create_program_address(&[SALE_VAULT_SEED, launch_key.as_ref(), &[snap.sale_vault_bump]], &crate::ID)
                .map_err(|_| error!(LaunchError::BadConfig))?;
        require_keys_eq!(*got, sale_vault, LaunchError::BadConfig);
    }

    let grad_rent = Pubkey::find_program_address(&[GRAD_RENT_SEED, launch_key.as_ref()], &crate::ID).0;

    let want: [(usize, Pubkey, &'static str); GRADUATION_FIXED_ACCOUNTS] = [
        (idx::LAUNCH_CONFIG, LAUNCH_CONFIG_KEY, "launch_config"),
        (idx::CARPENTER_AMM, CARPENTER_AMM_PROGRAM, "carpenter_amm"),
        (idx::AMM_CONFIG, AMM_CONFIG_KEY, "amm_config"),
        (idx::APPROVED_HOOK, APPROVED_HOOK_KEY, "approved_hook"),
        (idx::HOOK_PROGRAMDATA, HOOK_PROGRAMDATA_KEY, "hook_programdata"),
        (idx::POOL, pool, "pool"),
        (idx::BITMAP, amm(&[sd::BITMAP, pool.as_ref()]), "bitmap"),
        (idx::VAULT0, amm(&[sd::VAULT, pool.as_ref(), quote.as_ref()]), "vault0"),
        (idx::VAULT1, amm(&[sd::VAULT, pool.as_ref(), mint.as_ref()]), "vault1"),
        (idx::POOL_INDEX_ENTRY, amm(&[sd::POOL_INDEX, pool.as_ref()]), "pool_index_entry"),
        (idx::WSOL_MINT, quote, "wsol_mint"),
        (idx::SPL_TOKEN, quote_program, "spl_token"),
        (idx::MINT_EXCLUSIVITY0, mint_exclusivity0, "mint_exclusivity0"),
        (idx::MINT_EXCLUSIVITY1, amm(&[sd::MINT_EXCLUSIVITY, mint.as_ref()]), "mint_exclusivity1"),
        (idx::MINT_POOLS0, mint_pools0, "mint_pools0"),
        (idx::MINT_POOLS1, amm(&[sd::MINT_POOLS, mint.as_ref()]), "mint_pools1"),
        (idx::FLOOR_HOOK, FLOOR_HOOK_PROGRAM, "floor_hook"),
        (idx::HOOK_CONFIG, HOOK_CONFIG_KEY, "hook_config"),
        (idx::FLOOR_POOL, floor_pool, "floor_pool"),
        (idx::HOOK_AUTHORITY, HOOK_AUTHORITY_KEY, "hook_authority"),
        (idx::POSITION_SEED, position(SALT_SEED), "position_seed"),
        (idx::POSITION_BID, position(SALT_BID), "position_bid"),
        (idx::SYSTEM_PROGRAM, SYSTEM_PROGRAM, "system_program"),
        (idx::AMM_EVENT_AUTHORITY, AMM_EVENT_AUTHORITY_KEY, "amm_event_authority"),
        (idx::HOOK_EVENT_AUTHORITY, HOOK_EVENT_AUTHORITY_KEY, "hook_event_authority"),
        (idx::WSOL_VAULT, wsol_vault, "wsol_vault"),
        (idx::GRADUATION_RENT, grad_rent, "graduation_rent"),

        (idx::QUOTE_MINT0, amm(&[sd::QUOTE_MINT, quote.as_ref()]), "quote_mint0"),
        (idx::QUOTE_MINT1, amm(&[sd::QUOTE_MINT, mint.as_ref()]), "quote_mint1"),

        (idx::STAIR_STATE, Pubkey::find_program_address(&[sd::STAIR, floor_pool.as_ref()], &FLOOR_HOOK_PROGRAM).0, "stair_state"),
        (idx::POSITION_STAIR_STEP0, position(SALT_STAIR_STEP0), "position_stair_step0"),
        (idx::POSITION_STAIR_RESERVE, position(SALT_STAIR_RESERVE), "position_stair_reserve"),
        (idx::POSITION_STAIR_TAIL, position(SALT_STAIR_TAIL), "position_stair_tail"),
    ];
    for (i, key, name) in want.iter() {
        let got = ra[*i].key();
        if got != *key {
            return Err(error!(ErrorCode::ConstraintAddress).with_account_name(*name).with_pubkeys((got, *key)));
        }
    }
    for i in WRITABLE_FIXED {
        if !ra[i].is_writable {
            return Err(error!(ErrorCode::ConstraintMut).with_account_name(want[i].2));
        }
    }








    if !snap.native_quote() && !ra[idx::MINT_POOLS0].is_writable {
        return Err(error!(ErrorCode::ConstraintMut).with_account_name(want[idx::MINT_POOLS0].2));
    }


    let arrays = &ra[idx::TICK_ARRAYS..];
    for (k, a) in arrays.iter().enumerate() {
        if !a.is_writable {
            return Err(error!(ErrorCode::ConstraintMut).with_account_name("tick_array"));
        }
        let dup_array = arrays[..k].iter().any(|b| b.key == a.key);
        let dup_fixed = want.iter().any(|(_, key, _)| key == a.key);
        if dup_array || dup_fixed {
            return Err(error!(ErrorCode::ConstraintAddress).with_account_name("tick_array (duplicate)"));
        }
    }
    Ok(Checked { pool })
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
pub fn pol_range(open_tick: i32) -> floor_math::FloorMathResult<(i32, i32)> {
    let s = FLOOR_TICK_SPACING as i32;
    let lower = floor_math::spacing::ceil_to_spacing(MIN_TICK, s)?;
    let mut upper = floor_math::spacing::floor_to_spacing(open_tick, s)?;
    if upper <= lower {
        upper = lower + s;
    }
    Ok((lower, upper))
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
pub(crate) fn ladder_range(open: i32) -> Result<(i32, i32)> {
    let lower = floor_math::spacing::ceil_to_spacing(MIN_TICK, FLOOR_TICK_SPACING as i32)
        .map_err(|_| error!(LaunchError::MathOverflow))?;
    Ok((lower, spacing_floor(open)?))
}



///
pub(crate) fn spacing_floor(tick: i32) -> Result<i32> {
    floor_math::spacing::floor_to_spacing(tick, FLOOR_TICK_SPACING as i32).map_err(|_| error!(LaunchError::MathOverflow))
}

///
///
///
///
///
///
///
pub(crate) fn find_tick_array<'a, 'info>(ra: &'a [AccountInfo<'info>], pool: &Pubkey, tick: i32) -> Result<&'a AccountInfo<'info>> {
    let start = floor_types::layout::tick_array_start_index(tick, FLOOR_TICK_SPACING);
    let key = Pubkey::find_program_address(&[sd::TICK_ARRAY, pool.as_ref(), &start.to_le_bytes()], &CARPENTER_AMM_PROGRAM).0;
    ra[idx::TICK_ARRAYS..]
        .iter()
        .find(|a| *a.key == key)
        .ok_or_else(|| error!(ErrorCode::AccountNotEnoughKeys).with_account_name("tick_array"))
}

///
///
///
///
///
pub(crate) fn pool_sqrt_price(info: &AccountInfo<'_>) -> Result<u128> {
    use floor_types::layout::{Layout, Pool as PoolLayout};
    require_keys_eq!(*info.owner, CARPENTER_AMM_PROGRAM, LaunchError::BadConfig);
    let data = info.try_borrow_data()?;
    let end = 8 + PoolLayout::SIZE;
    require!(data.len() >= end, LaunchError::BadConfig);
    require!(data[..8] == PoolLayout::DISCRIMINATOR, LaunchError::BadConfig);
    let p: PoolLayout = bytemuck::pod_read_unaligned(&data[8..end]);
    Ok(p.sqrt_price_x64.get())
}

///
pub(crate) fn token_amount(account: &AccountInfo<'_>) -> Result<u64> {
    let data = account.try_borrow_data()?;
    let bytes = data.get(TOKEN_ACCOUNT_AMOUNT).ok_or(LaunchError::BadConfig)?;
    Ok(u64::from_le_bytes(bytes.try_into().map_err(|_| LaunchError::BadConfig)?))
}



///
pub(crate) type Acc<'a, 'info> = (&'a AccountInfo<'info>, bool, bool);

pub(crate) fn r<'a, 'info>(a: &'a AccountInfo<'info>) -> Acc<'a, 'info> {
    (a, false, false)
}
pub(crate) fn w<'a, 'info>(a: &'a AccountInfo<'info>) -> Acc<'a, 'info> {
    (a, true, false)
}
pub(crate) fn r_s<'a, 'info>(a: &'a AccountInfo<'info>) -> Acc<'a, 'info> {
    (a, false, true)
}
pub(crate) fn w_s<'a, 'info>(a: &'a AccountInfo<'info>) -> Acc<'a, 'info> {
    (a, true, true)
}

///
pub(crate) fn cpi<'info>(program_id: Pubkey, data: Vec<u8>, accs: &[Acc<'_, 'info>], signer_seeds: &[&[&[u8]]]) -> Result<()> {
    let metas = accs
        .iter()
        .map(|(a, writable, signer)| {
            if *writable {
                AccountMeta::new(*a.key, *signer)
            } else {
                AccountMeta::new_readonly(*a.key, *signer)
            }
        })
        .collect();
    let infos: Vec<AccountInfo<'info>> = accs.iter().map(|(a, _, _)| (*a).clone()).collect();
    invoke_signed(&Instruction { program_id, accounts: metas, data }, &infos, signer_seeds)?;
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
pub(crate) fn seed_accounts<'a, 'info>(
    g: &'a Graduation<'a, 'info>,
    position: Option<usize>,
    ta_lower: &'a AccountInfo<'info>,
    ta_upper: &'a AccountInfo<'info>,
) -> Vec<Acc<'a, 'info>> {
    let ra = g.graduation_accounts;
    let mut v = vec![r_s(g.launch), w(&ra[idx::FLOOR_POOL]), w(&ra[idx::POOL]), w(&ra[idx::BITMAP])];
    if let Some(p) = position {
        v.push(w(&ra[p]));
    }
    v.extend([
        w(ta_lower),
        w(ta_upper),
        w(g.token_vault),
        w(&ra[idx::VAULT0]),
        w(&ra[idx::VAULT1]),
        r(&ra[idx::WSOL_MINT]),
        r(g.mint),
        r(&ra[idx::SPL_TOKEN]),
        r(g.token_2022),
        w_s(g.payer),
        r(g.system_program),
        r(&ra[idx::CARPENTER_AMM]),
        r(&ra[idx::HOOK_AUTHORITY]),
        r(&ra[idx::AMM_EVENT_AUTHORITY]),
        r(&ra[idx::HOOK_EVENT_AUTHORITY]),
        r(&ra[idx::FLOOR_HOOK]),
    ]);
    v
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
pub(crate) fn ratchet_accounts<'a, 'info>(g: &'a Graduation<'a, 'info>) -> Vec<Acc<'a, 'info>> {
    let ra = g.graduation_accounts;
    let mut v = vec![
        w_s(g.payer),
        w(&ra[idx::FLOOR_POOL]),
        r(g.mint),
        w(&ra[idx::POOL]),
        w(&ra[idx::BITMAP]),
        w(&ra[idx::VAULT0]),
        w(&ra[idx::VAULT1]),
        r(&ra[idx::WSOL_MINT]),
        r(&ra[idx::SPL_TOKEN]),
        r(g.token_2022),
        w(&ra[idx::POSITION_SEED]),
        w(&ra[idx::POSITION_BID]),

        r(&ra[idx::FLOOR_HOOK]),
        r(g.system_program),
        r(&ra[idx::CARPENTER_AMM]),
        r(&ra[idx::HOOK_AUTHORITY]),
        r(&ra[idx::AMM_EVENT_AUTHORITY]),
        r(&ra[idx::HOOK_EVENT_AUTHORITY]),
        r(&ra[idx::FLOOR_HOOK]),


        w(&ra[idx::STAIR_STATE]),
        w(&ra[idx::POSITION_STAIR_STEP0]),
        w(&ra[idx::POSITION_STAIR_RESERVE]),
        w(&ra[idx::POSITION_STAIR_TAIL]),
    ];
    v.extend(ra[idx::TICK_ARRAYS..].iter().map(w));
    v
}

///
///
///
///
pub fn pool_stair_steps(floor_pool: &AccountInfo) -> Result<u8> {
    let data = floor_pool.try_borrow_data()?;
    Ok(data.get(floor_types::holders::FLOOR_POOL_ACCT_STAIR_STEPS).copied().unwrap_or(0))
}

///
pub fn pool_stair_owner(floor_pool: &AccountInfo) -> Result<u8> {
    let data = floor_pool.try_borrow_data()?;
    Ok(data.get(floor_types::holders::FLOOR_POOL_ACCT_STAIR_OWNER).copied().unwrap_or(0))
}

///
///
///
///
///
pub fn team_buy_owner_arg(l: &Launch) -> floor_types::owners::StairOwnerArg {
    use floor_types::constants::*;
    let (payee, pending) = ([0u8; 32], [0u8; 32]);
    floor_types::owners::StairOwnerArg {
        kind: STAIR_OWNER_TEAM_BUY,
        counted: 1,
        payee_bps: TEAM_BUY_PAYEE_BPS,
        first_unlock_secs: l.team_lock_days as u32 * 86_400,
        every_secs: 0,
        anchor: 0,
        owner_tokens: l.team_step_tokens,
        payee,
        payee_pending_id: pending,
        payee_platform: 0,
    }
}

///
///
///
///
///
pub(crate) fn stair_array_starts(open_edge: i32, steps: u8) -> Result<Vec<i32>> {
    let s = FLOOR_TICK_SPACING as i32;
    let m = |_| error!(LaunchError::MathOverflow);
    let mut out = Vec::with_capacity(2);
    for i in [1u8, steps] {
        let edge = floor_math::stair::stair_edge(open_edge, i, s).map_err(m)?;
        let st = tick_array_start_index(edge, FLOOR_TICK_SPACING);
        if !out.contains(&st) {
            out.push(st);
        }
    }
    Ok(out)
}

///
///
///
pub(crate) fn stair_seed_accounts<'a, 'info>(
    ra: &'a [AccountInfo<'info>],
    pool: &Pubkey,
    open_edge: i32,
    steps: u8,
    accs: &mut Vec<Acc<'a, 'info>>,
) -> Result<()> {
    accs.push(w(&ra[idx::STAIR_STATE]));
    accs.push(w(&ra[idx::POSITION_STAIR_STEP0]));
    accs.push(w(&ra[idx::POSITION_STAIR_RESERVE]));
    accs.push(w(&ra[idx::POSITION_STAIR_TAIL]));
    for st in stair_array_starts(open_edge, steps)? {
        let key = Pubkey::find_program_address(&[sd::TICK_ARRAY, pool.as_ref(), &st.to_le_bytes()], &CARPENTER_AMM_PROGRAM).0;
        let a = ra[idx::TICK_ARRAYS..]
            .iter()
            .find(|a| *a.key == key)
            .ok_or_else(|| error!(ErrorCode::AccountNotEnoughKeys).with_account_name("tick_array (staircase)"))?;
        accs.push(w(a));
    }
    Ok(())
}



///
///
///
pub(crate) fn emit_event<E: anchor_lang::Event>(g: &Graduation<'_, '_>, event: &E) -> Result<()> {
    require_keys_eq!(g.event_authority.key(), crate::EVENT_AUTHORITY_AND_BUMP.0, LaunchError::BadConfig);
    let mut data = anchor_lang::event::EVENT_IX_TAG_LE.to_vec();
    data.extend_from_slice(&event.data());
    let ix = Instruction {
        program_id: crate::ID,
        accounts: vec![AccountMeta::new_readonly(*g.event_authority.key, true)],
        data,
    };
    invoke_signed(
        &ix,
        &[g.event_authority.clone(), g.program.clone()],
        &[&[sd::EVENT_AUTHORITY, &[crate::EVENT_AUTHORITY_AND_BUMP.1]]],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;






    #[test]
    fn the_fixed_list_is_33_and_the_writable_set_is_the_spec_one() {


        assert_eq!(GRADUATION_FIXED_ACCOUNTS, 33);
        assert_eq!(
            (idx::STAIR_STATE, idx::POSITION_STAIR_STEP0, idx::POSITION_STAIR_RESERVE, idx::POSITION_STAIR_TAIL),
            (29, 30, 31, 32)
        );
        assert_eq!((idx::QUOTE_MINT0, idx::QUOTE_MINT1), (27, 28));
        assert_eq!((idx::POSITION_SEED, idx::POSITION_BID), (20, 21));
        assert_eq!(idx::HOOK_PROGRAMDATA, 4);
        assert_eq!(idx::GRADUATION_RENT, 26);
        assert_eq!(idx::TICK_ARRAYS, 33);
        assert_eq!(WRITABLE_FIXED, [5, 6, 7, 8, 9, 15, 18, 20, 21, 25, 26, 29, 30, 31, 32]);
    }

///
///
///
///
    #[test]
    fn a_staircase_graduation_needs_the_arrays_of_its_inner_edges() {
        let s = FLOOR_TICK_SPACING as i32;
        let price = floor_math::modes::tranche_price_x64(40_000, 20_000, 9).unwrap();
        let open = floor_math::price::open_tick(price).unwrap();
        let open_edge = spacing_floor(open).unwrap();
        let (pol_lo, pol_hi) = pol_range(open).unwrap();
        let pol: Vec<i32> = [pol_lo, pol_hi].iter().map(|t| tick_array_start_index(*t, FLOOR_TICK_SPACING)).collect();
        for n in [1u8, 2, 8, 9, 11] {
            let extra = stair_array_starts(open_edge, n).unwrap();
            assert!(!extra.is_empty() && extra.len() <= 2, "{n}: {extra:?}");
            let e1 = floor_math::stair::stair_edge(open_edge, 1, s).unwrap();
            let en = floor_math::stair::stair_edge(open_edge, n, s).unwrap();
            let mut edges = vec![open_edge, e1, pol_lo];
            if n > 1 {
                edges.push(floor_math::stair::reserve_range(open_edge, 1, n, s).unwrap().0);
            }
            edges.push(floor_math::stair::tail_range(open_edge, n, s).unwrap().1);
            assert_eq!(floor_math::stair::tail_range(open_edge, n, s).unwrap().1, en);
            for e in edges {
                let st = tick_array_start_index(e, FLOOR_TICK_SPACING);
                assert!(extra.contains(&st) || pol.contains(&st), "{n}: edge {e}'s array {st} is carried");
            }
        }
    }

///
///
    #[test]
    fn tick_array_start_floors_toward_negative_infinity() {
        use floor_types::constants::TICK_ARRAY_SIZE;
        use floor_types::layout::tick_array_start_index as start;
        let width = TICK_ARRAY_SIZE * FLOOR_TICK_SPACING as i32;
        assert_eq!(width, 3600);
        assert_eq!(start(-1, FLOOR_TICK_SPACING), -3600);
        assert_eq!(start(3599, FLOOR_TICK_SPACING), 0);
        assert_eq!(start(-3600, FLOOR_TICK_SPACING), -3600);
        assert_eq!(start(3600, FLOOR_TICK_SPACING), 3600);
    }

    #[test]
    fn launch_bytes_are_the_spec_struct_after_the_discriminator() {

        assert_eq!(size_of::<Launch>(), 512);
        assert_eq!(LAUNCH_BYTES, 8..520);
    }

    #[test]
    fn the_pol_is_the_full_range_tail_not_one_doubling_under_the_open() {
        let (lower, upper) = pol_range(80_040).unwrap();
        assert_eq!(lower, -443_580, "ceil_to_spacing(MIN_TICK, 60) = minUsableTick(60) on Solana's grid");
        assert_eq!(upper, 80_040, "floor_to_spacing(open, 60)");
        assert_ne!(lower, 80_040 - 6_932 - 48, "not [open − 6932, open] (the pre-window range)");





        let (lower, upper) = pol_range(-443_600).unwrap();
        assert_eq!((lower, upper), (-443_580, -443_520), "floor_to_spacing's MIN_TICK + s clamp, not the :470 branch");
        assert!(upper > lower, "the clamp alone keeps lower < upper, at every reachable open");
        assert!(!crate::sale::open_tick_in_bounds(-443_600), "and create_launch refuses an open this low");
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
    fn the_ladder_range_is_the_whole_band_from_the_open_to_the_top_of_the_range() {
        let open = 80_040;
        let s = FLOOR_TICK_SPACING as i32;
        let (pol_lower, pol_upper) = pol_range(open).unwrap();
        let (lad_lower, lad_upper) = ladder_range(open).unwrap();
        assert_eq!((lad_lower, lad_upper), (pol_lower, pol_upper), "the same two edges, so the same two arrays");
        assert_eq!((lad_lower, lad_upper), (-443_580, 80_040));

        let old_lower = floor_math::spacing::floor_to_spacing(open - 20_794, s).unwrap();
        let old_upper = floor_math::spacing::floor_to_spacing(open - 6_932, s).unwrap();
        assert!(lad_lower < old_lower && lad_upper > old_upper, "strictly wider than the retired 2x..8x band");

        assert_eq!(ladder_range(80_040 + 59).unwrap().1, 80_040);
    }
}
