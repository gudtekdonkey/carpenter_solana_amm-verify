//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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
use anchor_lang::solana_program::instruction::{AccountMeta, Instruction};
use anchor_lang::solana_program::program::{get_return_data, invoke_signed};
use floor_types::disc::amm as disc;
use floor_types::hook::{decode_op_result, HookOpResultV1};
use floor_types::layout::{tick_array_start_index, Layout, Pool as PoolLayout};

use crate::constants::{CARPENTER_AMM_PROGRAM, HOOK_AUTHORITY_SEED};
use crate::errors::HookError;

///
#[derive(Clone)]
pub struct AmmCpi<'info> {
///
    pub carpenter_amm: AccountInfo<'info>,
///
    pub amm_event_authority: AccountInfo<'info>,
///
    pub hook_authority: AccountInfo<'info>,
    pub hook_authority_bump: u8,
///
    pub hook_program: AccountInfo<'info>,
///
    pub floor_pool: AccountInfo<'info>,
///
    pub pool: AccountInfo<'info>,
}

///
#[derive(Clone)]
pub struct PoolAccounts<'info> {
///
///
    pub payer: Option<AccountInfo<'info>>,
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
    pub rent_owner: Pubkey,
    pub bitmap: AccountInfo<'info>,
    pub vault0: AccountInfo<'info>,
    pub vault1: AccountInfo<'info>,
    pub mint0: AccountInfo<'info>,
    pub mint1: AccountInfo<'info>,
    pub token_program0: AccountInfo<'info>,
    pub token_program1: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
///
///
    pub tick_arrays: Vec<AccountInfo<'info>>,
}



///
///
pub fn read_pool(pool: &AccountInfo) -> Result<PoolLayout> {
    require_keys_eq!(*pool.owner, CARPENTER_AMM_PROGRAM, HookError::WrongPool);
    let data = pool.try_borrow_data()?;
    require!(data.len() >= PoolLayout::ACCOUNT_LEN, HookError::WrongPool);
    require!(data[..8] == PoolLayout::DISCRIMINATOR, HookError::WrongPool);
    let p: PoolLayout = bytemuck::pod_read_unaligned(&data[8..PoolLayout::ACCOUNT_LEN]);
    drop(data);
    Ok(p)
}

///
///
pub fn position_key(pool: &Pubkey, position_id: &Pubkey, salt: u64) -> Pubkey {
    Pubkey::find_program_address(
        &[floor_types::seeds::POSITION, pool.as_ref(), position_id.as_ref(), &salt.to_le_bytes()],
        &CARPENTER_AMM_PROGRAM,
    )
    .0
}

///
///
///
///
///
///
pub fn created_position_is(info: &AccountInfo, pool: &Pubkey, position_id: &Pubkey, salt: u64) -> bool {
    use floor_types::layout::{Layout, Position as P};
    if *info.owner != CARPENTER_AMM_PROGRAM {
        return false;
    }
    let Ok(d) = info.try_borrow_data() else { return false };
    if d.len() < P::ACCOUNT_LEN || d[..8] != P::DISCRIMINATOR {
        return false;
    }
    let field = |name: &str| P::FIELDS.iter().find(|f| f.name == name).map(|f| 8 + f.offset).unwrap_or(0);
    let (at_pool, at_id, at_salt, at_bump) = (field("pool"), field("position_id"), field("salt"), field("bump"));
    if d[at_pool..at_pool + 32] != pool.to_bytes()
        || d[at_id..at_id + 32] != position_id.to_bytes()
        || d[at_salt..at_salt + 8] != salt.to_le_bytes()
    {
        return false;
    }
    Pubkey::create_program_address(
        &[floor_types::seeds::POSITION, pool.as_ref(), position_id.as_ref(), &salt.to_le_bytes(), &[d[at_bump]]],
        &CARPENTER_AMM_PROGRAM,
    )
    .is_ok_and(|k| k == *info.key)
}

///
pub fn tick_array_key(pool: &Pubkey, start_index: i32) -> Pubkey {
    Pubkey::find_program_address(
        &[floor_types::seeds::TICK_ARRAY, pool.as_ref(), &start_index.to_le_bytes()],
        &CARPENTER_AMM_PROGRAM,
    )
    .0
}

///
///
///
///
///
///
fn find_tick_array<'info>(set: &[AccountInfo<'info>], pool: &Pubkey, tick: i32, spacing: u16) -> Result<AccountInfo<'info>> {
    let start = tick_array_start_index(tick, spacing);
    if let Some(a) = set.iter().find(|a| created_tick_array_is(a, pool, start)) {
        return Ok(a.clone());
    }
    let key = tick_array_key(pool, start);
    set.iter()
        .find(|a| *a.key == key)
        .cloned()
        .ok_or_else(|| error!(anchor_lang::error::ErrorCode::AccountNotEnoughKeys))
}

///
///
///
///
pub fn has_tick_array(set: &[AccountInfo], pool: &Pubkey, tick: i32, spacing: u16) -> bool {
    find_tick_array(set, pool, tick, spacing).is_ok()
}

///
fn created_tick_array_is(info: &AccountInfo, pool: &Pubkey, start: i32) -> bool {
    use floor_types::layout::{Layout, TickArray as T};
    if *info.owner != CARPENTER_AMM_PROGRAM {
        return false;
    }
    let Ok(d) = info.try_borrow_data() else { return false };
    d.len() >= T::ACCOUNT_LEN && d[..8] == T::DISCRIMINATOR && d[8..40] == pool.to_bytes() && d[40..44] == start.to_le_bytes()
}



fn meta_as_passed(info: &AccountInfo, signer: bool) -> AccountMeta {
    if info.is_writable {
        AccountMeta::new(*info.key, signer)
    } else {
        AccountMeta::new_readonly(*info.key, signer)
    }
}

fn meta_mut(info: &AccountInfo, signer: bool) -> AccountMeta {
    AccountMeta::new(*info.key, signer)
}

fn meta_ro(info: &AccountInfo) -> AccountMeta {
    AccountMeta::new_readonly(*info.key, false)
}

///
fn absent<'info>(cpi: &AmmCpi<'info>) -> (AccountMeta, AccountInfo<'info>) {
    (AccountMeta::new_readonly(CARPENTER_AMM_PROGRAM, false), cpi.carpenter_amm.clone())
}

///
fn read_op_result() -> Result<HookOpResultV1> {
    let (from, bytes) = get_return_data().ok_or(HookError::AmmReturnMissing)?;
    require_keys_eq!(from, CARPENTER_AMM_PROGRAM, HookError::AmmReturnMissing);
    decode_op_result(&bytes).map_err(|_| error!(HookError::AmmReturnMissing))
}

///
fn call_as_hook_authority(cpi: &AmmCpi, ix: Instruction, infos: Vec<AccountInfo>) -> Result<HookOpResultV1> {
    let bump = [cpi.hook_authority_bump];
    let seeds: &[&[u8]] = &[HOOK_AUTHORITY_SEED, &bump];
    call(ix, infos, &[seeds])
}



///
///
///
///
///
#[allow(clippy::too_many_arguments)]
pub(crate) fn modify<'info>(
    cpi: &AmmCpi<'info>,
    accts: &PoolAccounts<'info>,
    position: &AccountInfo<'info>,
    spacing: u16,
    salt: u64,
    tick_lower: i32,
    tick_upper: i32,
    liquidity_delta: i128,
) -> Result<HookOpResultV1> {


    if !created_position_is(position, cpi.pool.key, cpi.hook_authority.key, salt) {
        require_keys_eq!(
            *position.key,
            position_key(cpi.pool.key, cpi.hook_authority.key, salt),
            anchor_lang::error::ErrorCode::ConstraintSeeds
        );
    }
    let ta_lower = find_tick_array(&accts.tick_arrays, cpi.pool.key, tick_lower, spacing)?;
    let ta_upper = find_tick_array(&accts.tick_arrays, cpi.pool.key, tick_upper, spacing)?;

    let mut data = Vec::with_capacity(8 + 32 + 8 + 4 + 4 + 16 + 8 + 8 + 4 + 32);
    data.extend_from_slice(&disc::MODIFY_LIQUIDITY);


    data.extend_from_slice(&cpi.hook_authority.key.to_bytes());
    data.extend_from_slice(&salt.to_le_bytes());
    data.extend_from_slice(&tick_lower.to_le_bytes());
    data.extend_from_slice(&tick_upper.to_le_bytes());
    data.extend_from_slice(&liquidity_delta.to_le_bytes());

    data.extend_from_slice(&u64::MAX.to_le_bytes());
    data.extend_from_slice(&u64::MAX.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());





    data.extend_from_slice(&accts.rent_owner.to_bytes());

    let (payer_meta, payer_info) = match &accts.payer {
        Some(p) => (meta_mut(p, true), p.clone()),
        None => absent(cpi),
    };
    let (none_meta, none_info) = absent(cpi);
    let (none_meta_authority, _) = absent(cpi);





    let metas = vec![
        AccountMeta::new_readonly(*cpi.hook_authority.key, true),
        payer_meta,
        meta_mut(&cpi.pool, false),
        meta_mut(&accts.bitmap, false),
        meta_mut(position, false),
        meta_mut(&ta_lower, false),
        meta_mut(&ta_upper, false),
        none_meta.clone(),
        none_meta,
        meta_mut(&accts.vault0, false),
        meta_mut(&accts.vault1, false),
        meta_ro(&accts.mint0),
        meta_ro(&accts.mint1),
        meta_ro(&accts.token_program0),
        meta_ro(&accts.token_program1),
        meta_ro(&accts.system_program),
        meta_ro(&cpi.hook_program),
        none_meta_authority,
        meta_ro(&cpi.amm_event_authority),
        meta_ro(&cpi.carpenter_amm),
        meta_ro(&cpi.floor_pool),
    ];
    let infos = vec![
        cpi.hook_authority.clone(),
        payer_info,
        cpi.pool.clone(),
        accts.bitmap.clone(),
        position.clone(),
        ta_lower,
        ta_upper,
        none_info,
        accts.vault0.clone(),
        accts.vault1.clone(),
        accts.mint0.clone(),
        accts.mint1.clone(),
        accts.token_program0.clone(),
        accts.token_program1.clone(),
        accts.system_program.clone(),
        cpi.hook_program.clone(),
        cpi.amm_event_authority.clone(),
        cpi.carpenter_amm.clone(),
        cpi.floor_pool.clone(),
    ];
    let ix = Instruction { program_id: CARPENTER_AMM_PROGRAM, accounts: metas, data };
    call_as_hook_authority(cpi, ix, infos)
}

fn liq_i128(liquidity: u128) -> Result<i128> {
    i128::try_from(liquidity).map_err(|_| error!(HookError::MathOverflow))
}

///
///
#[allow(clippy::too_many_arguments)]
pub fn open<'info>(
    cpi: &AmmCpi<'info>,
    accts: &PoolAccounts<'info>,
    position: &AccountInfo<'info>,
    spacing: u16,
    salt: u64,
    tick_lower: i32,
    tick_upper: i32,
    liquidity: u128,
) -> Result<HookOpResultV1> {
    require!(liquidity > 0, HookError::BadRange);
    modify(cpi, accts, position, spacing, salt, tick_lower, tick_upper, liq_i128(liquidity)?)
}

///
///
///
///
///
#[allow(clippy::too_many_arguments)]
pub fn close<'info>(
    cpi: &AmmCpi<'info>,
    accts: &PoolAccounts<'info>,
    position: &AccountInfo<'info>,
    spacing: u16,
    salt: u64,
    tick_lower: i32,
    tick_upper: i32,
    liquidity: u128,
) -> Result<HookOpResultV1> {
    require!(salt != floor_types::constants::SALT_SEED, HookError::InvariantViolated);
    require!(liquidity > 0, HookError::BadRange);
    modify(cpi, accts, position, spacing, salt, tick_lower, tick_upper, -liq_i128(liquidity)?)
}

///
#[allow(clippy::too_many_arguments)]
pub fn poke<'info>(
    cpi: &AmmCpi<'info>,
    accts: &PoolAccounts<'info>,
    position: &AccountInfo<'info>,
    spacing: u16,
    salt: u64,
    tick_lower: i32,
    tick_upper: i32,
) -> Result<HookOpResultV1> {
    modify(cpi, accts, position, spacing, salt, tick_lower, tick_upper, 0)
}

///
///
///
///
///
///
///
pub fn used(r: &HookOpResultV1) -> Result<(u64, u64)> {
    let p0 = r.delta.amount0.checked_sub(r.fees0 as i128).ok_or(HookError::MathOverflow)?;
    let p1 = r.delta.amount1.checked_sub(r.fees1 as i128).ok_or(HookError::MathOverflow)?;
    Ok((neg_to_u64(p0)?, neg_to_u64(p1)?))
}

///
pub fn back(r: &HookOpResultV1) -> Result<(u64, u64)> {
    Ok((pos_to_u64(r.delta.amount0)?, pos_to_u64(r.delta.amount1)?))
}

///
pub fn fees(r: &HookOpResultV1) -> (u64, u64) {
    (r.fees0, r.fees1)
}

///
pub fn neg_to_u64(v: i128) -> Result<u64> {
    require!(v <= 0, HookError::AmmReturnMissing);
    u64::try_from(-v).map_err(|_| error!(HookError::MathOverflow))
}

///
pub fn pos_to_u64(v: i128) -> Result<u64> {
    require!(v >= 0, HookError::AmmReturnMissing);
    u64::try_from(v).map_err(|_| error!(HookError::MathOverflow))
}



///
///
///
///
///
pub fn swap_exact_in<'info>(
    cpi: &AmmCpi<'info>,
    accts: &PoolAccounts<'info>,
    zero_for_one: bool,
    amount_in: u64,
    sqrt_price_limit_x64: u128,
) -> Result<HookOpResultV1> {
    require!(amount_in > 0, HookError::BadRange);
    let mut data = Vec::with_capacity(8 + 1 + 16 + 16 + 8 + 4);
    data.extend_from_slice(&disc::SWAP);
    data.push(zero_for_one as u8);
    data.extend_from_slice(&(-(amount_in as i128)).to_le_bytes());
    data.extend_from_slice(&sqrt_price_limit_x64.to_le_bytes());
    data.extend_from_slice(&0u64.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());

    let (none_meta, none_info) = absent(cpi);
    let n = accts.tick_arrays.len();







    let mut metas = vec![
        AccountMeta::new_readonly(*cpi.hook_authority.key, true),
        meta_mut(&cpi.pool, false),
        meta_mut(&accts.bitmap, false),
        none_meta.clone(),
        none_meta,
        meta_mut(&accts.vault0, false),
        meta_mut(&accts.vault1, false),
        meta_ro(&accts.mint0),
        meta_ro(&accts.mint1),
        meta_ro(&accts.token_program0),
        meta_ro(&accts.token_program1),
        meta_ro(&cpi.hook_program),
        meta_ro(&cpi.amm_event_authority),
        meta_ro(&cpi.carpenter_amm),
        meta_ro(&cpi.floor_pool),
    ];
    metas.reserve_exact(n);
    let mut infos = vec![
        cpi.hook_authority.clone(),
        cpi.pool.clone(),
        accts.bitmap.clone(),
        none_info,
        accts.vault0.clone(),
        accts.vault1.clone(),
        accts.mint0.clone(),
        accts.mint1.clone(),
        accts.token_program0.clone(),
        accts.token_program1.clone(),
        cpi.hook_program.clone(),
        cpi.amm_event_authority.clone(),
        cpi.carpenter_amm.clone(),
        cpi.floor_pool.clone(),
    ];
    infos.reserve_exact(n);
    let fixed = metas.len();
    for ta in &accts.tick_arrays {


        if metas[fixed..].iter().any(|m| m.pubkey == *ta.key) {
            continue;
        }
        metas.push(meta_as_passed(ta, false));
        infos.push(ta.clone());
    }
    let ix = Instruction { program_id: CARPENTER_AMM_PROGRAM, accounts: metas, data };
    call_as_hook_authority(cpi, ix, infos)
}



///
///
///
#[allow(clippy::too_many_arguments)]
pub fn deposit<'info>(
    carpenter_amm: &AccountInfo<'info>,
    amm_event_authority: &AccountInfo<'info>,
    pool: &AccountInfo<'info>,
    depositor: &AccountInfo<'info>,
    source: &AccountInfo<'info>,
    vault: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    currency: u8,
    amount: u64,
) -> Result<HookOpResultV1> {
    deposit_signed(carpenter_amm, amm_event_authority, pool, depositor, source, vault, mint, token_program, currency, amount, &[])
}

///
///
///
#[allow(clippy::too_many_arguments)]
pub fn deposit_signed<'info>(
    carpenter_amm: &AccountInfo<'info>,
    amm_event_authority: &AccountInfo<'info>,
    pool: &AccountInfo<'info>,
    depositor: &AccountInfo<'info>,
    source: &AccountInfo<'info>,
    vault: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    currency: u8,
    amount: u64,
    depositor_seeds: &[&[&[u8]]],
) -> Result<HookOpResultV1> {
    let mut data = Vec::with_capacity(8 + 1 + 8);
    data.extend_from_slice(&disc::HOOK_DEPOSIT);
    data.push(currency);
    data.extend_from_slice(&amount.to_le_bytes());

    let metas = vec![
        AccountMeta::new_readonly(*depositor.key, true),
        meta_mut(pool, false),
        meta_mut(source, false),
        meta_mut(vault, false),
        meta_ro(mint),
        meta_ro(token_program),
        meta_ro(amm_event_authority),
        meta_ro(carpenter_amm),
    ];
    let infos = vec![
        depositor.clone(),
        pool.clone(),
        source.clone(),
        vault.clone(),
        mint.clone(),
        token_program.clone(),
        amm_event_authority.clone(),
        carpenter_amm.clone(),
    ];
    let ix = Instruction { program_id: CARPENTER_AMM_PROGRAM, accounts: metas, data };
    call(ix, infos, depositor_seeds)
}

///
///
#[allow(clippy::too_many_arguments)]
pub fn withdraw<'info>(
    cpi: &AmmCpi<'info>,
    vault: &AccountInfo<'info>,
    destination: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    currency: u8,
    amount: u64,
) -> Result<HookOpResultV1> {
    let mut data = Vec::with_capacity(8 + 1 + 8);
    data.extend_from_slice(&disc::HOOK_WITHDRAW);
    data.push(currency);
    data.extend_from_slice(&amount.to_le_bytes());


    let metas = vec![
        AccountMeta::new_readonly(*cpi.hook_authority.key, true),
        meta_mut(&cpi.pool, false),
        meta_mut(vault, false),
        meta_mut(destination, false),
        meta_ro(mint),
        meta_ro(token_program),
        meta_ro(&cpi.amm_event_authority),
        meta_ro(&cpi.carpenter_amm),
    ];
    let infos = vec![
        cpi.hook_authority.clone(),
        cpi.pool.clone(),
        vault.clone(),
        destination.clone(),
        mint.clone(),
        token_program.clone(),
        cpi.amm_event_authority.clone(),
        cpi.carpenter_amm.clone(),
    ];
    let ix = Instruction { program_id: CARPENTER_AMM_PROGRAM, accounts: metas, data };
    call_as_hook_authority(cpi, ix, infos)
}

///
///
///
///
///
#[allow(clippy::too_many_arguments)]
pub fn withdraw_hooked<'info>(
    cpi: &AmmCpi<'info>,
    vault: &AccountInfo<'info>,
    destination: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    currency: u8,
    amount: u64,
    extra: &[AccountInfo<'info>],
) -> Result<HookOpResultV1> {
    let mut data = Vec::with_capacity(8 + 1 + 8);
    data.extend_from_slice(&disc::HOOK_WITHDRAW);
    data.push(currency);
    data.extend_from_slice(&amount.to_le_bytes());
    let mut metas = vec![
        AccountMeta::new_readonly(*cpi.hook_authority.key, true),
        meta_mut(&cpi.pool, false),
        meta_mut(vault, false),
        meta_mut(destination, false),
        meta_ro(mint),
        meta_ro(token_program),
        meta_ro(&cpi.amm_event_authority),
        meta_ro(&cpi.carpenter_amm),
    ];
    let mut infos = vec![
        cpi.hook_authority.clone(),
        cpi.pool.clone(),
        vault.clone(),
        destination.clone(),
        mint.clone(),
        token_program.clone(),
        cpi.amm_event_authority.clone(),
        cpi.carpenter_amm.clone(),
    ];
    for a in extra {
        metas.push(if a.is_writable { AccountMeta::new(*a.key, false) } else { AccountMeta::new_readonly(*a.key, false) });
        infos.push(a.clone());
    }
    let ix = Instruction { program_id: CARPENTER_AMM_PROGRAM, accounts: metas, data };
    call_as_hook_authority(cpi, ix, infos)
}

///
///
///
///
pub fn holders_hook_keys(mint: &Pubkey, source: &Pubkey, destination: &Pubkey) -> [Pubkey; 5] {
    use crate::constants::FLOOR_HOLDERS_PROGRAM as H;
    use floor_types::holders::{EXTRA_ACCOUNT_METAS, HOLDER_BOOK, HOLDER_RECORD};
    let pda = |seeds: &[&[u8]]| Pubkey::find_program_address(seeds, &H).0;
    [
        pda(&[HOLDER_BOOK, mint.as_ref()]),
        pda(&[HOLDER_RECORD, mint.as_ref(), source.as_ref()]),
        pda(&[HOLDER_RECORD, mint.as_ref(), destination.as_ref()]),
        pda(&[EXTRA_ACCOUNT_METAS, mint.as_ref()]),
        H,
    ]
}

///
///
///
pub fn split_hook_accounts<'info>(
    remaining: &[AccountInfo<'info>],
    floor_pool: &AccountInfo<'info>,
    hook_keys: Option<[Pubkey; 5]>,
) -> (Vec<AccountInfo<'info>>, Vec<AccountInfo<'info>>) {
    let Some(keys) = hook_keys else { return (remaining.to_vec(), Vec::new()) };
    let (mut hook, rest): (Vec<_>, Vec<_>) = remaining.iter().cloned().partition(|a| keys.contains(a.key));
    hook.push(floor_pool.clone());
    (rest, hook)
}



///
#[allow(clippy::too_many_arguments)]
pub fn register_mint_exclusivity<'info>(
    carpenter_amm: &AccountInfo<'info>,
    amm_event_authority: &AccountInfo<'info>,
    hook_authority: &AccountInfo<'info>,
    hook_authority_bump: u8,
    amm_config: &AccountInfo<'info>,
    approved_hook: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    mint_pools: &AccountInfo<'info>,
    mint_exclusivity: &AccountInfo<'info>,
    payer: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
) -> Result<()> {
    let data = disc::REGISTER_MINT_EXCLUSIVITY.to_vec();


    let metas = vec![
        AccountMeta::new_readonly(*hook_authority.key, true),
        meta_ro(amm_config),
        meta_ro(approved_hook),
        meta_ro(mint),
        meta_ro(mint_pools),
        meta_mut(mint_exclusivity, false),
        meta_mut(payer, true),
        meta_ro(system_program),
        meta_ro(amm_event_authority),
        meta_ro(carpenter_amm),
    ];
    let infos = vec![
        hook_authority.clone(),
        amm_config.clone(),
        approved_hook.clone(),
        mint.clone(),
        mint_pools.clone(),
        mint_exclusivity.clone(),
        payer.clone(),
        system_program.clone(),
        amm_event_authority.clone(),
        carpenter_amm.clone(),
    ];
    let ix = Instruction { program_id: CARPENTER_AMM_PROGRAM, accounts: metas, data };
    let bump = [hook_authority_bump];
    let seeds: &[&[u8]] = &[HOOK_AUTHORITY_SEED, &bump];
    invoke_signed(&ix, &infos, &[seeds])?;
    free_in_reverse(ix, infos);
    Ok(())
}

///
///
///
///
#[allow(clippy::too_many_arguments)]
pub fn hook_list_quote_mint<'info>(
    carpenter_amm: &AccountInfo<'info>,
    amm_event_authority: &AccountInfo<'info>,
    hook_authority: &AccountInfo<'info>,
    hook_authority_bump: u8,
    amm_config: &AccountInfo<'info>,
    approved_hook: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    quote_mint_entry: &AccountInfo<'info>,
    payer: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
) -> Result<()> {
    let data = disc::HOOK_LIST_QUOTE_MINT.to_vec();


    let metas = vec![
        AccountMeta::new_readonly(*hook_authority.key, true),
        meta_ro(amm_config),
        meta_ro(approved_hook),
        meta_ro(mint),
        meta_mut(quote_mint_entry, false),
        meta_mut(payer, true),
        meta_ro(system_program),
        meta_ro(amm_event_authority),
        meta_ro(carpenter_amm),
    ];
    let infos = vec![
        hook_authority.clone(),
        amm_config.clone(),
        approved_hook.clone(),
        mint.clone(),
        quote_mint_entry.clone(),
        payer.clone(),
        system_program.clone(),
        amm_event_authority.clone(),
        carpenter_amm.clone(),
    ];
    let ix = Instruction { program_id: CARPENTER_AMM_PROGRAM, accounts: metas, data };
    let bump = [hook_authority_bump];
    let seeds: &[&[u8]] = &[HOOK_AUTHORITY_SEED, &bump];
    invoke_signed(&ix, &infos, &[seeds])?;
    free_in_reverse(ix, infos);
    Ok(())
}



///
///
///
pub fn require_pool_side(p: &PoolLayout, currency: u8, vault: &Pubkey, mint: &Pubkey, token_program: &Pubkey) -> Result<()> {
    require_keys_eq!(Pubkey::new_from_array(p.hook), crate::ID, HookError::WrongPool);
    let (v, m, t) = match currency {
        0 => (p.vault0, p.mint0, p.token_program0),
        1 => (p.vault1, p.mint1, p.token_program1),
        _ => return err!(HookError::WrongPool),
    };
    require_keys_eq!(*vault, Pubkey::new_from_array(v), HookError::WrongPool);
    require_keys_eq!(*mint, Pubkey::new_from_array(m), HookError::WrongPool);
    require_keys_eq!(*token_program, Pubkey::new_from_array(t), HookError::WrongPool);
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
fn call(ix: Instruction, infos: Vec<AccountInfo>, signer_seeds: &[&[&[u8]]]) -> Result<HookOpResultV1> {
    invoke_signed(&ix, &infos, signer_seeds)?;
    let r = read_op_result();
    free_in_reverse(ix, infos);
    r
}

///
///
///
///
fn free_in_reverse(ix: Instruction, infos: Vec<AccountInfo>) {
    drop(infos);
    let Instruction { accounts, data, .. } = ix;
    drop(accounts);
    drop(data);
}
