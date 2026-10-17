//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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
use anchor_lang::solana_program::bpf_loader_upgradeable;
use anchor_spl::token_interface::{Mint, TokenInterface};
use carpenter_clmm_math::tick_math;
use carpenter_types::constants::{MAX_TICK_SPACING, MIN_TICK_SPACING};
use carpenter_types::hook::{
    AfterInitializeArgs, BeforeInitializeArgs, HOOK_INTERFACE_VERSION, PHASE_AFTER_INITIALIZE,
    PHASE_BEFORE_INITIALIZE,
};
use carpenter_types::layout::bitmap_account_len;

use crate::amm_core::bitmap;
use crate::amm_core::fees::{check_key_fee, hook_fee_valid, initial_lp_fee};
use crate::amm_core::hook_dispatch::{call_simple, is_self_call, should_call, HookCallContext, PoolSeeds};
use crate::amm_core::mints::{
    check_admitted, check_mint_supported_with, check_mint_supported_with_hook, ever_listed, implied_quote, is_present,
    may_pair_as_quote, read_exclusivity, read_quote_mint, BaseHook, MintRole, PausableRule, WSOL,
};
use crate::amm_core::ticks::math_err;
use crate::amm_core::vaults::{create_pda, create_vault};
use crate::constants::{
    AMM_CONFIG_SEED, APPROVED_HOOK_SEED, BITMAP_SEED, MINT_EXCLUSIVITY_SEED, MINT_POOLS_SEED,
    POOL_INDEX_SEED, POOL_SEED, QUOTE_MINT_SEED, VAULT_SEED,
};
use crate::errors::AmmError;
use crate::events::PoolInitialized;
use crate::state::{AmmConfig, ApprovedHook, MintPoolCount, Pool, PoolIndexEntry};

#[event_cpi]
#[derive(Accounts)]
#[instruction(fee: u32, tick_spacing: u16, hook: Pubkey)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
///
    pub initializer: Signer<'info>,
///
    #[account(seeds = [AMM_CONFIG_SEED], bump)]
    pub config: AccountLoader<'info, AmmConfig>,
///
    #[account(
        mut,
        seeds = [
            POOL_SEED,
            mint0.key().as_ref(),
            mint1.key().as_ref(),
            &fee.to_le_bytes(),
            &tick_spacing.to_le_bytes(),
            hook.as_ref(),
        ],
        bump,
    )]
    pub pool: UncheckedAccount<'info>,
///
    #[account(mut, seeds = [BITMAP_SEED, pool.key().as_ref()], bump)]
    pub bitmap: UncheckedAccount<'info>,
///
    #[account(mut, seeds = [VAULT_SEED, pool.key().as_ref(), mint0.key().as_ref()], bump)]
    pub vault0: UncheckedAccount<'info>,
///
    #[account(mut, seeds = [VAULT_SEED, pool.key().as_ref(), mint1.key().as_ref()], bump)]
    pub vault1: UncheckedAccount<'info>,
    #[account(mint::token_program = token_program0)]
    pub mint0: Box<InterfaceAccount<'info, Mint>>,
    #[account(mint::token_program = token_program1)]
    pub mint1: Box<InterfaceAccount<'info, Mint>>,
    pub token_program0: Interface<'info, TokenInterface>,
    pub token_program1: Interface<'info, TokenInterface>,
///
    #[account(seeds = [MINT_EXCLUSIVITY_SEED, mint0.key().as_ref()], bump)]
    pub mint_exclusivity0: UncheckedAccount<'info>,
///
    #[account(seeds = [MINT_EXCLUSIVITY_SEED, mint1.key().as_ref()], bump)]
    pub mint_exclusivity1: UncheckedAccount<'info>,
///
    #[account(seeds = [MINT_POOLS_SEED, mint0.key().as_ref()], bump)]
    pub mint_pools0: UncheckedAccount<'info>,
///
    #[account(seeds = [MINT_POOLS_SEED, mint1.key().as_ref()], bump)]
    pub mint_pools1: UncheckedAccount<'info>,
///
///
    #[account(init, payer = payer, space = PoolIndexEntry::LEN, seeds = [POOL_INDEX_SEED, pool.key().as_ref()], bump)]
    pub pool_index_entry: AccountLoader<'info, PoolIndexEntry>,
///
    #[account(seeds = [APPROVED_HOOK_SEED, hook.as_ref()], bump)]
    pub approved_hook: Option<AccountLoader<'info, ApprovedHook>>,
///
    #[account(address = hook @ AmmError::HookNotApproved)]
    pub hook_program: Option<UncheckedAccount<'info>>,
///
///
///
///
    #[account(
        seeds = [hook.as_ref()],
        bump,
        seeds::program = bpf_loader_upgradeable::ID,
        owner = bpf_loader_upgradeable::ID @ AmmError::HookNotApproved,
    )]
    pub hook_programdata: Option<UncheckedAccount<'info>>,
    pub system_program: Program<'info, System>,
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
    #[account(seeds = [QUOTE_MINT_SEED, mint0.key().as_ref()], bump)]
    pub quote_mint0: UncheckedAccount<'info>,
///
    #[account(seeds = [QUOTE_MINT_SEED, mint1.key().as_ref()], bump)]
    pub quote_mint1: UncheckedAccount<'info>,
}

///
#[derive(Clone, Copy, Default)]
struct HookTerms {
    flags: u16,
    max_rounds: u8,
    callback_accounts: u8,
    hook_authority: Pubkey,
}

///
pub(crate) fn programdata_slot(info: &AccountInfo) -> Result<u64> {
    require_keys_eq!(*info.owner, bpf_loader_upgradeable::ID, AmmError::HookNotExecutable);
    let data = info.try_borrow_data()?;
    require!(data.len() >= 45, AmmError::HookNotExecutable);
    require!(data[0..4] == 3u32.to_le_bytes(), AmmError::HookNotExecutable);
    Ok(u64::from_le_bytes(data[4..12].try_into().map_err(|_| AmmError::HookNotExecutable)?))
}

///
fn approve_terms(accounts: &InitializePool, hook: &Pubkey) -> Result<HookTerms> {
    let approved = accounts.approved_hook.as_ref().ok_or(AmmError::HookNotApproved)?;
    require!(accounts.hook_program.is_some(), AmmError::HookNotApproved);
    let programdata = accounts.hook_programdata.as_ref().ok_or(AmmError::HookUpgradedReapprove)?;

    let a = approved.load()?;
    require_keys_eq!(a.hook_program, *hook, AmmError::HookNotApproved);
    require!(a.paused == 0, AmmError::HookPausedForNewPools);

    require!(Clock::get()?.unix_timestamp >= a.guardian_paused_until, AmmError::HookPausedForNewPools);

    let (expected, _) = Pubkey::find_program_address(&[hook.as_ref()], &bpf_loader_upgradeable::ID);
    require_keys_eq!(programdata.key(), expected, AmmError::HookUpgradedReapprove);
    let slot = programdata_slot(&programdata.to_account_info())?;
    require!(slot == a.programdata_slot, AmmError::HookUpgradedReapprove);

    Ok(HookTerms {
        flags: a.flags,
        max_rounds: a.max_rounds,
        callback_accounts: a.callback_accounts,
        hook_authority: a.hook_authority,
    })
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
fn check_exclusivity(info: &AccountInfo, mint: &Pubkey, hook: &Pubkey, vouched: bool) -> Result<Option<u32>> {
    if let Some(excl) = read_exclusivity(info, mint)? {
        require!(!vouched, AmmError::ExclusivityQuoteMint);
        require_keys_eq!(excl.hook_program, *hook, AmmError::MintExclusive);
        return Ok((excl.has_protocol_fee == 1).then_some(excl.protocol_fee));
    }
    Ok(None)
}

pub fn handler<'info>(
    ctx: Context<'info, InitializePool<'info>>,
    fee: u32,
    tick_spacing: u16,
    hook: Pubkey,
    sqrt_price_x64: u128,
) -> Result<()> {
    let mint0_key = ctx.accounts.mint0.key();
    let mint1_key = ctx.accounts.mint1.key();
    let pool_key = ctx.accounts.pool.key();
    let has_hook = hook != Pubkey::default();


    require!(mint0_key.to_bytes() < mint1_key.to_bytes(), AmmError::CurrenciesOutOfOrder);
    require!((MIN_TICK_SPACING..=MAX_TICK_SPACING).contains(&tick_spacing), AmmError::TickSpacingInvalid);
    check_key_fee(fee)?;
    let tick = tick_math::get_tick_at_sqrt_price(sqrt_price_x64).map_err(math_err)?;
























    let quote0_info = ctx.accounts.quote_mint0.to_account_info();
    let quote1_info = ctx.accounts.quote_mint1.to_account_info();



    let (legacy0, legacy1, admitted, current_protocol_fee) = {
        let c = ctx.accounts.config.load()?;
        (implied_quote(&mint0_key), implied_quote(&mint1_key), c.quote_caps_admitted, c.current_protocol_fee())
    };
    let (counts0, counts1) = (!legacy0, !legacy1);
    let mut roles = [MintRole::Base, MintRole::Base];
    let mut retired = [false, false];
    let mut vouched = [legacy0, legacy1];
    for (i, (legacy, info, mint)) in
        [(legacy0, &quote0_info, &mint0_key), (legacy1, &quote1_info, &mint1_key)].into_iter().enumerate()
    {


        let entry = read_quote_mint(info, mint)?;
        retired[i] = entry.map(|q| q.state == carpenter_types::layout::QUOTE_STATE_RETIRED).unwrap_or(false);
        vouched[i] = legacy || ever_listed(entry.as_ref());



        if let Some(q) = entry.as_ref() {
            require!(
                !(retired[i] && has_hook && q.hook_program.to_bytes() == hook.to_bytes()),
                AmmError::QuoteMintRetired
            );
        }
        if legacy || may_pair_as_quote(entry.as_ref(), &hook) {
            roles[i] = MintRole::ListedQuote;
        }
    }



























    let admitted_program: Option<Pubkey> = match (&ctx.accounts.approved_hook, has_hook) {
        (Some(a), true) => {
            let p = a.load()?.base_transfer_hook;
            (p != Pubkey::default()).then_some(p)
        }
        _ => None,
    };
    let base_side: Option<usize> = match roles {
        [MintRole::Base, MintRole::ListedQuote] => Some(0),
        [MintRole::ListedQuote, MintRole::Base] => Some(1),
        _ => None,
    };
    let admitted_base: Option<(Pubkey, usize)> = match (admitted_program, base_side) {
        (Some(p), Some(side)) => Some((p, side)),
        _ => None,
    };
    let base_hook = |i: usize| match admitted_base {
        Some((p, side)) if side == i => BaseHook::Admitted(p),
        _ => BaseHook::None,
    };
    let pausable = |role: MintRole| {
        if has_hook && role == MintRole::Base {
            PausableRule::HookVouched
        } else {
            PausableRule::Renounced
        }
    };
    for (i, info) in [ctx.accounts.mint0.to_account_info(), ctx.accounts.mint1.to_account_info()].iter().enumerate() {
        let verdict = check_mint_supported_with_hook(info, roles[i], pausable(roles[i]), base_hook(i)).and_then(|caps| {
            if roles[i] == MintRole::ListedQuote {
                check_admitted(info, caps, admitted)?;
            }
            Ok(caps)
        });
        if let Err(e) = verdict {



            if retired[i] && check_mint_supported_with(info, MintRole::ListedQuote, pausable(MintRole::ListedQuote)).is_ok() {
                return err!(AmmError::QuoteMintRetired);
            }
            return Err(e);
        }
    }


    let terms = if has_hook { approve_terms(ctx.accounts, &hook)? } else { HookTerms::default() };
    require!(hook_fee_valid(has_hook, terms.flags, fee), AmmError::FeeInvalid);


    let recorded0 = check_exclusivity(&ctx.accounts.mint_exclusivity0.to_account_info(), &mint0_key, &hook, vouched[0])?;
    let recorded1 = check_exclusivity(&ctx.accounts.mint_exclusivity1.to_account_info(), &mint1_key, &hook, vouched[1])?;



    let recorded0 = if roles[0] == MintRole::Base { recorded0 } else { None };
    let recorded1 = if roles[1] == MintRole::Base { recorded1 } else { None };
    let protocol_fee = carpenter_types::protocol_fee::rate_for_new_pool(current_protocol_fee, recorded0, recorded1);

    let n = terms.callback_accounts as usize;
    if has_hook {
        require!(ctx.remaining_accounts.len() >= n, ErrorCode::AccountNotEnoughKeys);
    }


    let payer = ctx.accounts.payer.to_account_info();
    let pool_info = ctx.accounts.pool.to_account_info();
    let seeds_buf = PoolSeeds::new(mint0_key, mint1_key, fee, tick_spacing, hook, ctx.bumps.pool);
    let pool_seeds = seeds_buf.seeds();
    create_pda(&payer, &pool_info, &crate::ID, Pool::LEN, &pool_seeds)?;

    let bitmap_info = ctx.accounts.bitmap.to_account_info();
    create_pda(
        &payer,
        &bitmap_info,
        &crate::ID,
        bitmap_account_len(tick_spacing),
        &[BITMAP_SEED, pool_key.as_ref(), &[ctx.bumps.bitmap]],
    )?;
    bitmap::init_account(&bitmap_info, &pool_key, tick_spacing)?;

    let vault0_info = ctx.accounts.vault0.to_account_info();
    let vault1_info = ctx.accounts.vault1.to_account_info();
    create_vault(
        &payer,
        &vault0_info,
        &ctx.accounts.mint0.to_account_info(),
        &ctx.accounts.token_program0.to_account_info(),
        &pool_info,
        &[VAULT_SEED, pool_key.as_ref(), mint0_key.as_ref(), &[ctx.bumps.vault0]],
    )?;
    create_vault(
        &payer,
        &vault1_info,
        &ctx.accounts.mint1.to_account_info(),
        &ctx.accounts.token_program1.to_account_info(),
        &pool_info,
        &[VAULT_SEED, pool_key.as_ref(), mint1_key.as_ref(), &[ctx.bumps.vault1]],
    )?;

    let index_info = ctx.accounts.pool_index_entry.to_account_info();

    let mint_pools0 = ctx.accounts.mint_pools0.to_account_info();
    let mint_pools1 = ctx.accounts.mint_pools1.to_account_info();
    for (counts, info, mint, bump) in [
        (counts0, &mint_pools0, &mint0_key, ctx.bumps.mint_pools0),
        (counts1, &mint_pools1, &mint1_key, ctx.bumps.mint_pools1),
    ] {
        if !counts {
            continue;
        }
        require!(info.is_writable, ErrorCode::ConstraintMut);
        if !is_present(info, MintPoolCount::LEN, MintPoolCount::DISCRIMINATOR)? {

            create_pda(&payer, info, &crate::ID, MintPoolCount::LEN, &[MINT_POOLS_SEED, mint.as_ref(), &[bump]])?;
            let mut data = info.try_borrow_mut_data()?;
            data[..8].copy_from_slice(MintPoolCount::DISCRIMINATOR);
            let mpc: &mut MintPoolCount = bytemuck::from_bytes_mut(&mut data[8..MintPoolCount::LEN]);
            mpc.mint = *mint;
            mpc.count = 0;
            mpc.bump = bump;
        }
    }


    let self_call = is_self_call(&hook, &terms.hook_authority, &ctx.accounts.initializer.key());
    let amm_accounts = [
        ctx.accounts.payer.key(),
        ctx.accounts.initializer.key(),
        ctx.accounts.config.key(),
        bitmap_info.key(),
        vault0_info.key(),
        vault1_info.key(),
        index_info.key(),
        mint_pools0.key(),
        mint_pools1.key(),
        ctx.accounts.mint_exclusivity0.key(),
        ctx.accounts.mint_exclusivity1.key(),

        quote0_info.key(),
        quote1_info.key(),
    ];
    let hook_program_info = ctx.accounts.hook_program.as_ref().map(|h| h.to_account_info());
    if should_call(terms.flags, PHASE_BEFORE_INITIALIZE, self_call) {
        let hp = hook_program_info.as_ref().ok_or(AmmError::HookNotApproved)?;
        let hctx = HookCallContext {
            settle_accounts: &[],
            hook_program: hp,
            pool: &pool_info,
            pool_signer_seeds: &pool_seeds,
            slice: &ctx.remaining_accounts[..n],
            amm_accounts: &amm_accounts,
            pool_hook: hook,
            flags: terms.flags,
            max_rounds: terms.max_rounds,
            callback_accounts: terms.callback_accounts,
        };
        let args = BeforeInitializeArgs {
            version: HOOK_INTERFACE_VERSION,
            phase: PHASE_BEFORE_INITIALIZE,
            round: 0,
            pool: pool_key.to_bytes(),
            sender: ctx.accounts.initializer.key().to_bytes(),
            mint0: mint0_key.to_bytes(),
            mint1: mint1_key.to_bytes(),
            fee,
            tick_spacing,
            sqrt_price_x64,
        };
        call_simple(&hctx, PHASE_BEFORE_INITIALIZE, &args)?;
    }


    let slot = Clock::get()?.slot;
    {
        let mut data = pool_info.try_borrow_mut_data()?;
        data[..8].copy_from_slice(Pool::DISCRIMINATOR);
        let p: &mut Pool = bytemuck::from_bytes_mut(&mut data[8..Pool::LEN]);
        p.version = 1;
        p.bump = ctx.bumps.pool;
        p.hook_flags = terms.flags;
        p.tick_spacing = tick_spacing;
        p.hook_max_rounds = terms.max_rounds;
        p.hook_callback_accounts = terms.callback_accounts;
        p.fee = fee;
        p.lp_fee = initial_lp_fee(fee);

















        p.protocol_fee = protocol_fee;
        p.protocol_fee_currency = carpenter_types::protocol_fee::currency_for_roles(
            roles[0] == MintRole::ListedQuote,
            roles[1] == MintRole::ListedQuote,
            mint0_key == WSOL,
            mint1_key == WSOL,
        );
        p.tick_current = tick;
        p.mint0 = mint0_key;
        p.mint1 = mint1_key;
        p.hook = hook;
        p.hook_authority = terms.hook_authority;
        p.vault0 = vault0_info.key();
        p.vault1 = vault1_info.key();
        p.token_program0 = ctx.accounts.token_program0.key();
        p.token_program1 = ctx.accounts.token_program1.key();
        p.sqrt_price_x64.set(sqrt_price_x64);
        p.created_slot = slot;
        p.decimals0 = ctx.accounts.mint0.decimals;
        p.decimals1 = ctx.accounts.mint1.decimals;
        p.bitmap_bump = ctx.bumps.bitmap;
        p.vault0_bump = ctx.bumps.vault0;
        p.vault1_bump = ctx.bumps.vault1;
        p.paused = 0;



        let (admitted_hook, admitted_side) = admitted_base.map(|(p, side)| (p, side as u8)).unwrap_or_default();
        p.base_transfer_hook = admitted_hook;
        p.base_hook_side = admitted_side;
    }
    {


        let mut e = ctx.accounts.pool_index_entry.load_init()?;
        e.pool = pool_key;
        e.created_slot = slot;
        e.bump = ctx.bumps.pool_index_entry;
    }


    for (counts, info) in [(counts0, &mint_pools0), (counts1, &mint_pools1)] {
        if counts {
            let mut data = info.try_borrow_mut_data()?;
            let mpc: &mut MintPoolCount = bytemuck::from_bytes_mut(&mut data[8..MintPoolCount::LEN]);
            mpc.count = mpc.count.checked_add(1).ok_or(AmmError::MathOverflow)?;
        }
    }


    if should_call(terms.flags, PHASE_AFTER_INITIALIZE, self_call) {
        let hp = hook_program_info.as_ref().ok_or(AmmError::HookNotApproved)?;
        let hctx = HookCallContext {
            settle_accounts: &[],
            hook_program: hp,
            pool: &pool_info,
            pool_signer_seeds: &pool_seeds,
            slice: &ctx.remaining_accounts[..n],
            amm_accounts: &amm_accounts,
            pool_hook: hook,
            flags: terms.flags,
            max_rounds: terms.max_rounds,
            callback_accounts: terms.callback_accounts,
        };
        let args = AfterInitializeArgs {
            version: HOOK_INTERFACE_VERSION,
            phase: PHASE_AFTER_INITIALIZE,
            round: 0,
            pool: pool_key.to_bytes(),
            sender: ctx.accounts.initializer.key().to_bytes(),
            mint0: mint0_key.to_bytes(),
            mint1: mint1_key.to_bytes(),
            fee,
            tick_spacing,
            sqrt_price_x64,
            tick,
        };
        call_simple(&hctx, PHASE_AFTER_INITIALIZE, &args)?;
    }

    emit_cpi!(PoolInitialized {
        pool: pool_key,
        mint0: mint0_key,
        mint1: mint1_key,
        fee,
        tick_spacing,
        hook,
        sqrt_price_x64,
        tick,
    });
    Ok(())
}
