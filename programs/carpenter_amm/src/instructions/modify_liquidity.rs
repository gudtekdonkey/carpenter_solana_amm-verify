//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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
use carpenter_types::hook::{
    AfterModifyLiquidityArgs, BalanceDelta, BeforeModifyLiquidityArgs, HookOpResultV1, ModifyLiquidityParams,
    PoolSnapshot, HOOK_INTERFACE_VERSION, MAX_HOOK_DATA, PHASE_AFTER_ADD_LIQUIDITY, PHASE_AFTER_REMOVE_LIQUIDITY,
    PHASE_BEFORE_ADD_LIQUIDITY, PHASE_BEFORE_REMOVE_LIQUIDITY,
};
use carpenter_types::layout::tick_array_start_index;

use crate::amm_core::bitmap;
use crate::amm_core::fees::clamp_paid_fee;
use crate::amm_core::hook_dispatch::{call_simple, is_self_call, should_call, snapshot, HookCallContext, PoolSeeds};
use crate::amm_core::positions::{create_position, modify_position, set_range, TickArrays};
use crate::amm_core::ticks::check_range;
use crate::amm_core::vaults::{admitted_hook, transfer_checked_in, transfer_checked_out, TransferIn};
use crate::constants::{POSITION_SEED, TICK_ARRAY_SEED};
use crate::errors::AmmError;
use crate::events::{FeesTruncated, LiquidityModified};
use crate::instructions::initialize_tick_array::ensure_tick_array;
use crate::instructions::set_pool_paused::require_not_paused;
use crate::state::{Pool, Position, TickArray, TickArrayBitmap};

#[event_cpi]
#[derive(Accounts)]
pub struct ModifyLiquidity<'info> {
    pub owner: Signer<'info>,
///
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    #[account(mut)]
    pub pool: AccountLoader<'info, Pool>,
    #[account(mut, constraint = bitmap.load()?.pool == pool.key() @ AmmError::SeedsMismatch)]
    pub bitmap: AccountLoader<'info, TickArrayBitmap>,
///
///
    #[account(mut)]
    pub position: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub tick_array_lower: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub tick_array_upper: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub owner_token0: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
///
    #[account(mut)]
    pub owner_token1: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
    #[account(mut, address = pool.load()?.vault0 @ AmmError::VaultMismatch)]
    pub vault0: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, address = pool.load()?.vault1 @ AmmError::VaultMismatch)]
    pub vault1: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(address = pool.load()?.mint0 @ AmmError::MintMismatch)]
    pub mint0: Box<InterfaceAccount<'info, Mint>>,
    #[account(address = pool.load()?.mint1 @ AmmError::MintMismatch)]
    pub mint1: Box<InterfaceAccount<'info, Mint>>,
    #[account(address = pool.load()?.token_program0 @ AmmError::TokenProgramMismatch)]
    pub token_program0: Interface<'info, TokenInterface>,
    #[account(address = pool.load()?.token_program1 @ AmmError::TokenProgramMismatch)]
    pub token_program1: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
///
    pub hook_program: Option<UncheckedAccount<'info>>,
///
///
///
    pub position_authority: Option<Signer<'info>>,
}

///
struct PoolView {
    seeds: PoolSeeds,
    hook: Pubkey,
    hook_authority: Pubkey,
    flags: u16,
    max_rounds: u8,
    callback_accounts: u8,
    tick_spacing: u16,
///
    admitted: [Option<Pubkey>; 2],
}

///
///
///
///
///
///
///
#[derive(AnchorSerialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MinOutArg {
    pub amount0_min: u64,
    pub amount1_min: u64,
}

impl AnchorDeserialize for MinOutArg {
    fn deserialize_reader<R: borsh::io::Read>(reader: &mut R) -> borsh::io::Result<Self> {
        let mut b = [0u8; 16];
        let mut got = 0usize;
        while got < b.len() {
            match reader.read(&mut b[got..])? {
                0 => break,
                n => got += n,
            }
        }
        match got {
            0 => Ok(Self::default()),
            16 => Ok(Self {
                amount0_min: u64::from_le_bytes(b[..8].try_into().unwrap()),
                amount1_min: u64::from_le_bytes(b[8..].try_into().unwrap()),
            }),
            _ => Err(borsh::io::Error::new(borsh::io::ErrorKind::UnexpectedEof, "min_out: 1..=15 bytes")),
        }
    }
}

///
///
///
///
///
///
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn ensure_position<'info>(
    payer: Option<&AccountInfo<'info>>,
    info: &AccountInfo<'info>,
    pool: &Pubkey,
    owner: &Pubkey,
    position_id: &[u8; 32],
    id_signed: bool,
    salt: u64,
    bump: u8,
    tick_lower: i32,
    tick_upper: i32,
) -> Result<()> {
    if *info.owner != crate::ID {
        require!(id_signed, AmmError::PositionIdNotSigned);
        let payer = payer.ok_or(ErrorCode::AccountNotEnoughKeys)?;


        return create_position(payer, info, pool, owner, position_id, salt, bump, tick_lower, tick_upper);
    }
    let mut data = info.try_borrow_mut_data()?;
    require!(data.len() == Position::LEN && &data[..8] == Position::DISCRIMINATOR, AmmError::SeedsMismatch);
    let p: &mut Position = bytemuck::from_bytes_mut(&mut data[8..Position::LEN]);
    require_keys_eq!(p.pool, *pool, AmmError::SeedsMismatch);
    require_keys_eq!(p.owner, *owner, AmmError::Unauthorized);
    require_keys_eq!(p.position_id, Pubkey::new_from_array(*position_id), AmmError::SeedsMismatch);
    require!(p.salt == salt, AmmError::SeedsMismatch);
    set_range(p, tick_lower, tick_upper)
}

#[allow(clippy::too_many_arguments)]
pub fn handler<'info>(
    ctx: Context<'info, ModifyLiquidity<'info>>,
    position_id: [u8; 32],
    salt: u64,
    tick_lower: i32,
    tick_upper: i32,
    liquidity_delta: i128,
    amount0_max: u64,
    amount1_max: u64,
    hook_data: Vec<u8>,
    rent_owner: Pubkey,
    min_out: MinOutArg,
) -> Result<crate::state::HookOpResultV1> {
    require!(hook_data.len() <= MAX_HOOK_DATA, AmmError::HookDataTooLong);
    let owner = ctx.accounts.owner.key();
    let pool_key = ctx.accounts.pool.key();


    let id_signed = match &ctx.accounts.position_authority {
        Some(a) => {
            require!(a.key().to_bytes() == position_id, AmmError::SeedsMismatch);
            true
        }
        None => owner.to_bytes() == position_id,
    };

    let view = {
        let p = ctx.accounts.pool.load()?;


        if liquidity_delta > 0 {
            require_not_paused(&p)?;
        }
        PoolView {
            seeds: PoolSeeds::of(&p),
            hook: p.hook,
            hook_authority: p.hook_authority,
            flags: p.hook_flags,
            max_rounds: p.hook_max_rounds,
            callback_accounts: p.hook_callback_accounts,
            tick_spacing: p.tick_spacing,
            admitted: [admitted_hook(&p, 0), admitted_hook(&p, 1)],
        }
    };
    let pool_seeds = view.seeds.seeds();
    let spacing = view.tick_spacing;
    check_range(tick_lower, tick_upper, spacing)?;
    let self_call = is_self_call(&view.hook, &view.hook_authority, &owner);

    let bitmap_info = ctx.accounts.bitmap.to_account_info();
    bitmap::check_account(&bitmap_info, &pool_key, spacing)?;


    let start_lower = tick_array_start_index(tick_lower, spacing);
    let start_upper = tick_array_start_index(tick_upper, spacing);
    let same_array = start_lower == start_upper;
    let ta_lower = ctx.accounts.tick_array_lower.to_account_info();
    let ta_upper = ctx.accounts.tick_array_upper.to_account_info();
    let (want_lower, bump_lower) = Pubkey::find_program_address(
        &[TICK_ARRAY_SEED, pool_key.as_ref(), &start_lower.to_le_bytes()],
        &crate::ID,
    );
    let (want_upper, bump_upper) = Pubkey::find_program_address(
        &[TICK_ARRAY_SEED, pool_key.as_ref(), &start_upper.to_le_bytes()],
        &crate::ID,
    );
    require_keys_eq!(ta_lower.key(), want_lower, AmmError::SeedsMismatch);
    require_keys_eq!(ta_upper.key(), want_upper, AmmError::SeedsMismatch);

    let payer_info = ctx.accounts.payer.as_ref().map(|p| p.to_account_info());
    ensure_tick_array(payer_info.as_ref(), &rent_owner, &ta_lower, &pool_key, start_lower, bump_lower, false)?;
    if !same_array {
        ensure_tick_array(payer_info.as_ref(), &rent_owner, &ta_upper, &pool_key, start_upper, bump_upper, false)?;
    }
    let position_info = ctx.accounts.position.to_account_info();
    let (want_position, position_bump) = Pubkey::find_program_address(
        &[POSITION_SEED, pool_key.as_ref(), &position_id, &salt.to_le_bytes()],
        &crate::ID,
    );
    require_keys_eq!(position_info.key(), want_position, AmmError::SeedsMismatch);
    ensure_position(
        payer_info.as_ref(),
        &position_info,
        &pool_key,
        &owner,
        &position_id,
        id_signed,
        salt,
        position_bump,
        tick_lower,
        tick_upper,
    )?;


    let pool_info = ctx.accounts.pool.to_account_info();
    let hook_program_info = ctx.accounts.hook_program.as_ref().map(|h| h.to_account_info());
    let n = view.callback_accounts as usize;
    let mut amm_accounts = vec![
        owner,
        bitmap_info.key(),
        position_info.key(),
        ta_lower.key(),
        ta_upper.key(),
        ctx.accounts.vault0.key(),
        ctx.accounts.vault1.key(),
    ];
    if let Some(p) = &payer_info {
        amm_accounts.push(p.key());
    }
    if let Some(a) = &ctx.accounts.position_authority {
        amm_accounts.push(a.key());
    }
    if let Some(t) = &ctx.accounts.owner_token0 {
        amm_accounts.push(t.key());
    }
    if let Some(t) = &ctx.accounts.owner_token1 {
        amm_accounts.push(t.key());
    }
    let params = ModifyLiquidityParams { salt, tick_lower, tick_upper, liquidity_delta };
    let (phase_before, phase_after) = if liquidity_delta > 0 {
        (PHASE_BEFORE_ADD_LIQUIDITY, PHASE_AFTER_ADD_LIQUIDITY)
    } else {
        (PHASE_BEFORE_REMOVE_LIQUIDITY, PHASE_AFTER_REMOVE_LIQUIDITY)
    };
    let call_before = should_call(view.flags, phase_before, self_call);
    let call_after = should_call(view.flags, phase_after, self_call);
    if call_before || call_after {
        require!(ctx.remaining_accounts.len() >= n, ErrorCode::AccountNotEnoughKeys);
    }


    if call_before {
        let pre: PoolSnapshot = snapshot(&*ctx.accounts.pool.load()?);
        let hp = hook_program_info.as_ref().ok_or(AmmError::HookBadReturn)?;
        let hctx = HookCallContext {
            settle_accounts: &[],
            hook_program: hp,
            pool: &pool_info,
            pool_signer_seeds: &pool_seeds,
            slice: &ctx.remaining_accounts[..n],
            amm_accounts: &amm_accounts,
            pool_hook: view.hook,
            flags: view.flags,
            max_rounds: view.max_rounds,
            callback_accounts: view.callback_accounts,
        };
        let args = BeforeModifyLiquidityArgs {
            version: HOOK_INTERFACE_VERSION,
            phase: phase_before,
            round: 0,
            pool: pool_key.to_bytes(),
            sender: owner.to_bytes(),
            params,
            pre,
            hook_data: hook_data.clone(),
        };
        call_simple(&hctx, phase_before, &args)?;
    }


    let (principal, owed0, owed1, flips) = {
        let mut pool = ctx.accounts.pool.load_mut()?;
        let mut pos_data = position_info.try_borrow_mut_data()?;
        let position: &mut Position = bytemuck::from_bytes_mut(&mut pos_data[8..Position::LEN]);
        if same_array {
            let mut d = ta_lower.try_borrow_mut_data()?;
            let arr: &mut TickArray = bytemuck::from_bytes_mut(&mut d[8..TickArray::LEN]);
            let before = arr.initialized_tick_count;
            let (principal, f0, f1) =
                modify_position(&mut pool, position, TickArrays::Same(&mut *arr), liquidity_delta)?;
            let flip = bitmap::flip_for(before, arr.initialized_tick_count);
            (principal, f0, f1, [(start_lower, flip), (start_upper, None)])
        } else {
            let mut dl = ta_lower.try_borrow_mut_data()?;
            let mut du = ta_upper.try_borrow_mut_data()?;
            let lower: &mut TickArray = bytemuck::from_bytes_mut(&mut dl[8..TickArray::LEN]);
            let upper: &mut TickArray = bytemuck::from_bytes_mut(&mut du[8..TickArray::LEN]);
            let (bl, bu) = (lower.initialized_tick_count, upper.initialized_tick_count);
            let (principal, f0, f1) = modify_position(
                &mut pool,
                position,
                TickArrays::Split { lower: &mut *lower, upper: &mut *upper },
                liquidity_delta,
            )?;
            let fl = bitmap::flip_for(bl, lower.initialized_tick_count);
            let fu = bitmap::flip_for(bu, upper.initialized_tick_count);
            (principal, f0, f1, [(start_lower, fl), (start_upper, fu)])
        }
    };
    for (start, flip) in flips {
        if let Some(set) = flip {
            bitmap::set_array_initialized(&bitmap_info, spacing, start, set)?;
        }
    }


    let before0 = ctx.accounts.vault0.amount;
    let before1 = ctx.accounts.vault1.amount;
    let (claims0, claims1, protocol0, protocol1) = {
        let p = ctx.accounts.pool.load()?;
        (p.hook_claims0, p.hook_claims1, p.protocol_fees_owed0, p.protocol_fees_owed1)
    };
    let out0 = u64::try_from(principal.amount0.max(0)).map_err(|_| AmmError::MathOverflow)?;
    let out1 = u64::try_from(principal.amount1.max(0)).map_err(|_| AmmError::MathOverflow)?;
    let (paid0, truncated0) = clamp_paid_fee(owed0, before0, claims0, protocol0, out0);
    let (paid1, truncated1) = clamp_paid_fee(owed1, before1, claims1, protocol1, out1);
    let delta = BalanceDelta {
        amount0: principal.amount0.checked_add(paid0 as i128).ok_or(AmmError::MathOverflow)?,
        amount1: principal.amount1.checked_add(paid1 as i128).ok_or(AmmError::MathOverflow)?,
    };

    require!(delta.amount0 >= 0 || delta.amount0.unsigned_abs() <= amount0_max as u128, AmmError::SlippageExceeded);
    require!(delta.amount1 >= 0 || delta.amount1.unsigned_abs() <= amount1_max as u128, AmmError::SlippageExceeded);

    require!(min_out.amount0_min == 0 || delta.amount0 >= min_out.amount0_min as i128, AmmError::SlippageExceeded);
    require!(min_out.amount1_min == 0 || delta.amount1 >= min_out.amount1_min as i128, AmmError::SlippageExceeded);


    if call_after {
        let post = snapshot(&*ctx.accounts.pool.load()?);
        let hp = hook_program_info.as_ref().ok_or(AmmError::HookBadReturn)?;
        let hctx = HookCallContext {
            settle_accounts: &[],
            hook_program: hp,
            pool: &pool_info,
            pool_signer_seeds: &pool_seeds,
            slice: &ctx.remaining_accounts[..n],
            amm_accounts: &amm_accounts,
            pool_hook: view.hook,
            flags: view.flags,
            max_rounds: view.max_rounds,
            callback_accounts: view.callback_accounts,
        };
        let args = AfterModifyLiquidityArgs {
            version: HOOK_INTERFACE_VERSION,
            phase: phase_after,
            round: 0,
            pool: pool_key.to_bytes(),
            sender: owner.to_bytes(),
            params,
            post,
            delta,
            fees_accrued: BalanceDelta { amount0: paid0 as i128, amount1: paid1 as i128 },
            hook_data,
        };
        call_simple(&hctx, phase_after, &args)?;
    }


    if self_call {
        let mut p = ctx.accounts.pool.load_mut()?;
        p.hook_claims0 = apply_claims(p.hook_claims0, delta.amount0)?;
        p.hook_claims1 = apply_claims(p.hook_claims1, delta.amount1)?;
    } else {
        let owner_info = ctx.accounts.owner.to_account_info();
        settle_currency(
            &ctx.accounts.token_program0.to_account_info(),
            ctx.accounts.owner_token0.as_ref().map(|t| t.to_account_info()),
            &ctx.accounts.mint0.to_account_info(),
            &ctx.accounts.vault0.to_account_info(),
            &owner_info,
            &pool_info,
            &pool_seeds,
            ctx.accounts.mint0.decimals,
            delta.amount0,
            view.admitted[0],
            ctx.remaining_accounts,
        )?;
        settle_currency(
            &ctx.accounts.token_program1.to_account_info(),
            ctx.accounts.owner_token1.as_ref().map(|t| t.to_account_info()),
            &ctx.accounts.mint1.to_account_info(),
            &ctx.accounts.vault1.to_account_info(),
            &owner_info,
            &pool_info,
            &pool_seeds,
            ctx.accounts.mint1.decimals,
            delta.amount1,
            view.admitted[1],
            ctx.remaining_accounts,
        )?;
    }


    ctx.accounts.vault0.reload()?;
    ctx.accounts.vault1.reload()?;
    let moved0 = if self_call { 0 } else { delta.amount0 };
    let moved1 = if self_call { 0 } else { delta.amount1 };
    require!(ctx.accounts.vault0.amount as i128 == before0 as i128 - moved0, AmmError::VaultConservationViolated);
    require!(ctx.accounts.vault1.amount as i128 == before1 as i128 - moved1, AmmError::VaultConservationViolated);
    {
        let p = ctx.accounts.pool.load()?;
        require!(
            p.hook_claims0 as u128 + p.protocol_fees_owed0 as u128 <= ctx.accounts.vault0.amount as u128
                && p.hook_claims1 as u128 + p.protocol_fees_owed1 as u128 <= ctx.accounts.vault1.amount as u128,



            AmmError::VaultShortfall
        );
    }


    emit_cpi!(LiquidityModified {
        pool: pool_key,
        owner,
        salt,
        tick_lower,
        tick_upper,
        liquidity_delta,
        amount0: delta.amount0,
        amount1: delta.amount1,
        fees0: paid0,
        fees1: paid1,
        position: position_info.key(),
    });
    if truncated0 {
        emit_cpi!(FeesTruncated { position: position_info.key(), currency: 0 });
    }
    if truncated1 {
        emit_cpi!(FeesTruncated { position: position_info.key(), currency: 1 });
    }
    let result = {
        let p = ctx.accounts.pool.load()?;
        HookOpResultV1 {
            version: HOOK_INTERFACE_VERSION,
            delta,
            fees0: paid0,
            fees1: paid1,
            post: snapshot(&p),
            hook_claims0: p.hook_claims0,
            hook_claims1: p.hook_claims1,
        }
    };
    Ok(result.into())
}

///
///
fn apply_claims(claims: u64, delta: i128) -> Result<u64> {
    if delta < 0 {
        let debit = u64::try_from(delta.unsigned_abs()).map_err(|_| AmmError::HookClaimsInsufficient)?;
        Ok(claims.checked_sub(debit).ok_or(AmmError::HookClaimsInsufficient)?)
    } else {
        let credit = u64::try_from(delta).map_err(|_| AmmError::MathOverflow)?;
        Ok(claims.checked_add(credit).ok_or(AmmError::MathOverflow)?)
    }
}

///
///
#[allow(clippy::too_many_arguments)]
fn settle_currency<'info>(
    token_program: &AccountInfo<'info>,
    owner_token: Option<AccountInfo<'info>>,
    mint: &AccountInfo<'info>,
    vault: &AccountInfo<'info>,
    owner: &AccountInfo<'info>,
    pool: &AccountInfo<'info>,
    pool_seeds: &[&[u8]],
    decimals: u8,
    delta: i128,
    admitted: Option<Pubkey>,
    extra: &[AccountInfo<'info>],
) -> Result<()> {
    if delta == 0 {
        return Ok(());
    }
    let owner_token = owner_token.ok_or(ErrorCode::AccountNotEnoughKeys)?;
    let amount = u64::try_from(delta.unsigned_abs()).map_err(|_| AmmError::MathOverflow)?;
    if delta < 0 {
        transfer_checked_in(
            &TransferIn {
                token_program,
                source: &owner_token,
                mint,
                vault,
                owner,
                pool,
                pool_signer_seeds: pool_seeds,
                decimals,
                admitted,
                extra,
            },
            amount,
        )
    } else {
        transfer_checked_out(token_program, vault, mint, &owner_token, pool, pool_seeds, amount, decimals, admitted, extra)
    }
}

#[cfg(test)]
mod min_out_tests {
    use super::*;

    #[test]
    fn the_min_out_arg_is_optional_on_the_wire_and_refuses_a_partial_one() {
        let mut none: &[u8] = &[];
        assert_eq!(MinOutArg::deserialize(&mut none).unwrap(), MinOutArg::default(), "0 bytes: no bound");
        let m = MinOutArg { amount0_min: 7, amount1_min: u64::MAX };
        let mut bytes = Vec::new();
        m.serialize(&mut bytes).unwrap();
        assert_eq!(bytes.len(), 16, "it serialises as the two u64s");
        let mut full: &[u8] = &bytes;
        assert_eq!(MinOutArg::deserialize(&mut full).unwrap(), m, "16 bytes: those minimums");
        for n in 1..16 {
            let mut part: &[u8] = &bytes[..n];
            assert!(MinOutArg::deserialize(&mut part).is_err(), "{n} bytes: refused, never truncated");
        }
    }
}
