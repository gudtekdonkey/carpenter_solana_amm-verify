//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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
use anchor_lang::system_program;
use anchor_spl::associated_token::{self, AssociatedToken};
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};
use floor_types::constants::{
    BPS, FLOOR_TICK_SPACING, GENESIS_CREATOR_BPS, GENESIS_DEFAULT_DURATION, GENESIS_DURATION_SECS, GENESIS_LADDER_TOKENS,
    GENESIS_CORE_CAP_LAMPORTS, GENESIS_CORE_WALLET_CAP_LAMPORTS, GENESIS_FF_CAP_LAMPORTS,
    GENESIS_FF_WALLET_CAP_LAMPORTS, GENESIS_LOCKS, GENESIS_PHASE_CORE, GENESIS_PHASE_FF,
    GENESIS_POL_TOKENS, GENESIS_SALE_TOKENS, GENESIS_SCHEDULE_HEADROOM, GENESIS_SKIM_FAR_BPS, GENESIS_STAIR_GROWTH_BPS,
    LAUNCH_STATE_BONDING,
    LAUNCH_STATE_GENESIS, LAUNCH_STATE_GENESIS_CLOSED, LAUNCH_STATE_GRADUATED, MODE_CUSTOM, TOKEN_DECIMALS,
    LADDER_TOKENS, POL_TOKENS, SALE_TOKENS,
};
use floor_types::layout::{FloorPool as FloorPoolLayout, Layout};
use floor_types::disc;

use crate::constants::{
    CARPENTER_AMM_PROGRAM, EVENT_AUTHORITY_SEED, FLOOR_HOOK_PROGRAM, GENESIS_CONTRIBUTION_SEED, GENESIS_SALE_SEED,
    GRAD_RENT_SEED, HOOK_CONFIG_SEED, LAUNCH_AUTHORITY_SEED, LAUNCH_CONFIG_SEED, LAUNCH_SEED, SALE_VAULT_SEED, WSOL_MINT,
    FLOOR_PLATFORM_POOL_SEED,
};
use crate::errors::LaunchError;
use crate::events::{
    AllocationDustRouted, Contributed, FloorFeesApplied, FloorShareUpdated, GenesisAuthorityTransferStarted,
    GenesisAuthorityTransferred, GenesisClaimed, GenesisInitialized, GenesisStairTopUpdated, GenesisWhitelistUpdated,
    GenesisRefunded, OpeningTick, ScheduleUpdated,
};
use crate::graduate::{self, r, r_s, w};
use crate::graduation_rent;
use crate::state::{GenesisContribution, GenesisSale, Launch, LaunchConfig};

///
///
///
pub const VESTING_SCHEDULE_LEN: usize = 248;

///
///
///
pub fn lock_rent(rent: &Rent, escrow_len: usize) -> u64 {
    GENESIS_LOCKS.len() as u64 * (rent.minimum_balance(VESTING_SCHEDULE_LEN) + rent.minimum_balance(escrow_len))
}





#[event_cpi]
#[derive(Accounts)]
pub struct InitGenesis<'info> {
///
///
///
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(seeds = [LAUNCH_CONFIG_SEED], bump)]
    pub launch_config: AccountLoader<'info, LaunchConfig>,
    #[account(mut, seeds = [LAUNCH_SEED, mint.key().as_ref()], bump)]
    pub launch: AccountLoader<'info, Launch>,
    #[account(address = launch.load()?.mint @ LaunchError::BadConfig)]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
///
///
    #[account(mut, address = launch.load()?.floor_pool @ LaunchError::BadConfig, owner = FLOOR_HOOK_PROGRAM @ LaunchError::BadConfig)]
    pub floor_pool: UncheckedAccount<'info>,
///
///
    #[account(init, payer = authority, space = GenesisSale::LEN, seeds = [GENESIS_SALE_SEED], bump)]
    pub genesis_sale: AccountLoader<'info, GenesisSale>,
///
    #[account(mut, seeds = [GRAD_RENT_SEED, launch.key().as_ref()], bump)]
    pub graduation_rent: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,

///
    #[account(seeds = [LAUNCH_AUTHORITY_SEED], bump)]
    pub launch_authority: UncheckedAccount<'info>,
///
    #[account(seeds = [HOOK_CONFIG_SEED], bump, seeds::program = FLOOR_HOOK_PROGRAM)]
    pub hook_config: UncheckedAccount<'info>,
///
    #[account(mut, seeds = [FLOOR_PLATFORM_POOL_SEED], bump, seeds::program = FLOOR_HOOK_PROGRAM)]
    pub floor_platform_pool: UncheckedAccount<'info>,
///
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = FLOOR_HOOK_PROGRAM)]
    pub hook_event_authority: UncheckedAccount<'info>,
///
    #[account(address = FLOOR_HOOK_PROGRAM)]
    pub floor_hook: UncheckedAccount<'info>,
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
fn designate_floor_pool(ctx: &Context<InitGenesis>) -> Result<()> {
    let a = &ctx.accounts;
    let metas = vec![
        AccountMeta::new_readonly(a.launch_authority.key(), true),
        AccountMeta::new(a.authority.key(), true),
        AccountMeta::new_readonly(a.hook_config.key(), false),
        AccountMeta::new(a.floor_pool.key(), false),
        AccountMeta::new(a.floor_platform_pool.key(), false),
        AccountMeta::new_readonly(a.system_program.key(), false),
        AccountMeta::new_readonly(a.hook_event_authority.key(), false),
        AccountMeta::new_readonly(FLOOR_HOOK_PROGRAM, false),
    ];
    let infos = [
        a.launch_authority.to_account_info(),
        a.authority.to_account_info(),
        a.hook_config.to_account_info(),
        a.floor_pool.to_account_info(),
        a.floor_platform_pool.to_account_info(),
        a.system_program.to_account_info(),
        a.hook_event_authority.to_account_info(),
        a.floor_hook.to_account_info(),
    ];
    let ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: FLOOR_HOOK_PROGRAM,
        accounts: metas,
        data: disc::hook::DESIGNATE_FLOOR_POOL.to_vec(),
    };
    let bump = [ctx.bumps.launch_authority];
    anchor_lang::solana_program::program::invoke_signed(&ix, &infos, &[&[LAUNCH_AUTHORITY_SEED, &bump]])?;
    Ok(())
}

///
///
///
///
///
///
pub fn init_genesis(
    ctx: Context<InitGenesis>,
    beneficiary: Pubkey,
    floor_share_bps: u16,
    payee: crate::GenesisPayeeArg,
) -> Result<()> {
    let authority = ctx.accounts.authority.key();
    require_keys_eq!(authority, ctx.accounts.launch_config.load()?.governance, LaunchError::NotGenesisAuthority);
    require!(floor_share_bps as u64 <= BPS, LaunchError::GenesisBadConfig);

    require!(crate::genesis_payee_ok(&payee), LaunchError::GenesisBadConfig);

    let launch_key = ctx.accounts.launch.key();
    let (mint, token_vault, sale_vault, wsol_vault, amm_pool, floor_pool) = {
        let l = ctx.accounts.launch.load()?;
        require_keys_eq!(l.creator, authority, LaunchError::NotGenesisAuthority);
        require!(
            l.state == LAUNCH_STATE_BONDING
                && l.sold == 0
                && l.raised == 0
                && l.buyers == 0
                && l.team_owed == 0
                && l.mode == MODE_CUSTOM
                && l.native_quote()


                && l.skim_buy_far == GENESIS_SKIM_FAR_BPS
                && l.skim_sell_far == GENESIS_SKIM_FAR_BPS
                && l.creator_buy == GENESIS_CREATOR_BPS
                && l.creator_sell == GENESIS_CREATOR_BPS
                && l.holder_buy == 0
                && l.holder_sell == 0




                && l.sale_tokens == SALE_TOKENS
                && l.pol_tokens == POL_TOKENS
                && l.ladder_tokens == LADDER_TOKENS


                && l.team_step_tokens == 0
                && l.team_wallet_tokens == 0,
            LaunchError::GenesisBadConfig
        );
        let sale_vault =
            Pubkey::create_program_address(&[SALE_VAULT_SEED, launch_key.as_ref(), &[l.sale_vault_bump]], &crate::ID)
                .map_err(|_| error!(LaunchError::BadConfig))?;
        (l.mint, l.token_vault, sale_vault, l.wsol_vault, l.amm_pool, l.floor_pool)
    };
    {
        let data = ctx.accounts.floor_pool.try_borrow_data()?;
        require!(
            data.len() >= 8 + FloorPoolLayout::SIZE && data[..8] == FloorPoolLayout::DISCRIMINATOR,
            LaunchError::BadConfig
        );
        let p: FloorPoolLayout = bytemuck::pod_read_unaligned(&data[8..8 + FloorPoolLayout::SIZE]);







        let t = floor_types::slots::PayeeSlots::from_account(&data).ok_or(error!(LaunchError::GenesisBadConfig))?;
        require!(genesis_slots_ok(&t, &authority), LaunchError::GenesisBadConfig);
        require!(
            p.referrer == [0u8; 32]
                && p.referrer_t2 == [0u8; 32]
                && p.referrer_t3 == [0u8; 32]
                && p.holders_hooked == 0
                && p.holder_buy_bps == 0
                && p.holder_sell_bps == 0
                && p.stair_kind == floor_types::constants::STAIR_KIND_PRESET
                && p.stair_ready == 0



                && p.skim_sell_at_floor_bps
                    == floor_types::constants::skim_sell_at_floor_bps(GENESIS_SKIM_FAR_BPS),
            LaunchError::GenesisBadConfig
        );
    }




    let genesis_key = ctx.accounts.genesis_sale.key();
    for bad in [
        Pubkey::default(),
        genesis_key,
        launch_key,
        mint,
        token_vault,
        sale_vault,
        wsol_vault,
        amm_pool,
        floor_pool,
        crate::ID,
        FLOOR_HOOK_PROGRAM,
        CARPENTER_AMM_PROGRAM,
        crate::constants::FLOOR_VESTING_PROGRAM,
    ] {
        require_keys_neq!(beneficiary, bad, LaunchError::GenesisBadConfig);
    }





    let escrow_len = graduation_rent::base_vault_len(&ctx.accounts.mint.to_account_info())?;
    let rent = Rent::get()?;

    let top_up = lock_rent(&rent, escrow_len)
        .checked_add(graduation_rent::genesis_cushion_rent_lamports(&rent))

        .and_then(|x| x.checked_add(graduation_rent::owner_tail_rent_lamports(&rent)))
        .ok_or(LaunchError::MathOverflow)?;
    system_program::transfer(
        CpiContext::new(
            system_program::ID,
            system_program::Transfer {
                from: ctx.accounts.authority.to_account_info(),
                to: ctx.accounts.graduation_rent.to_account_info(),
            },
        ),
        top_up,
    )?;

    {
        let mut l = ctx.accounts.launch.load_mut()?;
        l.state = LAUNCH_STATE_GENESIS;


        l.sale_tokens = GENESIS_SALE_TOKENS;
        l.pol_tokens = GENESIS_POL_TOKENS;
        l.ladder_tokens = GENESIS_LADDER_TOKENS;



        l.team_bps = 0;


        l.price_x64 = crate::state::U128::new(0);
    }
    let mut g = ctx.accounts.genesis_sale.load_init()?;
    g.version = 1;
    g.bump = ctx.bumps.genesis_sale;
    g.floor_share_bps = floor_share_bps;
    g.authority = authority;
    g.beneficiary = beneficiary;
    g.launch = launch_key;
    g.mint = mint;
    g.open_time = 0;
    g.duration = GENESIS_DEFAULT_DURATION;


    if payee.pending_id == [0u8; 32] {
        g.payee_state = floor_types::constants::TEAM_PAYEE_DEPLOYER;
        g.payee = beneficiary;
    } else {
        g.payee_state = floor_types::constants::TEAM_PAYEE_PENDING;
        g.payee = Pubkey::new_from_array(payee.pending_id);
        g.payee_platform = payee.platform;
    }
    drop(g);

    designate_floor_pool(&ctx)?;

    emit_cpi!(GenesisInitialized {
        genesis_sale: genesis_key,
        launch: launch_key,
        mint,
        authority,
        beneficiary,
        floor_share_bps,
        duration: GENESIS_DEFAULT_DURATION,
    });
    Ok(())
}





#[event_cpi]
#[derive(Accounts)]
pub struct SetGenesis<'info> {
    pub authority: Signer<'info>,
    #[account(mut, seeds = [GENESIS_SALE_SEED], bump)]
    pub genesis_sale: AccountLoader<'info, GenesisSale>,
}

///
///
///
///
///
///
///
pub fn set_genesis_schedule(ctx: Context<SetGenesis>, open_time: i64, duration: u64) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let mut g = ctx.accounts.genesis_sale.load_mut()?;
    require_keys_eq!(ctx.accounts.authority.key(), g.authority, LaunchError::NotGenesisAuthority);
    require!(!g.opened(now), LaunchError::GenesisScheduleFrozen);


    require!(open_time == 0 || g.stair_top_mcap != 0, LaunchError::GenesisStairTopUnset);


    require!(duration == GENESIS_DURATION_SECS, LaunchError::GenesisDurationNotFixed);

    require!(
        open_time == 0 || (g.whitelist_root != [0u8; 32] && g.ff_root != [0u8; 32]),
        LaunchError::GenesisWhitelistUnset
    );
    let fits = (open_time as i128) + (duration as i128) + (GENESIS_SCHEDULE_HEADROOM as i128) <= i64::MAX as i128;
    require!(
        duration != 0 && open_time >= 0 && (open_time == 0 || open_time > now) && fits,
        LaunchError::GenesisInvalidSchedule
    );
    g.open_time = open_time;
    g.duration = duration;


    let sale_end = if g.sale_closed == 0 { 0 } else { g.sale_end() };
    drop(g);
    emit_cpi!(ScheduleUpdated { open_time, duration, sale_end });
    Ok(())
}

///
pub fn set_genesis_floor_share(ctx: Context<SetGenesis>, floor_share_bps: u16) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let mut g = ctx.accounts.genesis_sale.load_mut()?;
    require_keys_eq!(ctx.accounts.authority.key(), g.authority, LaunchError::NotGenesisAuthority);
    require!(!g.opened(now), LaunchError::GenesisScheduleFrozen);
    require!(floor_share_bps as u64 <= BPS, LaunchError::GenesisBadConfig);
    let previous = g.floor_share_bps;
    g.floor_share_bps = floor_share_bps;
    drop(g);
    emit_cpi!(FloorShareUpdated { previous, next: floor_share_bps });
    Ok(())
}

///
///
///
///
///
pub fn set_genesis_stair_top(ctx: Context<SetGenesis>, top_mcap: u64) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let mut g = ctx.accounts.genesis_sale.load_mut()?;
    require_keys_eq!(ctx.accounts.authority.key(), g.authority, LaunchError::NotGenesisAuthority);
    require!(!g.opened(now), LaunchError::GenesisScheduleFrozen);
    require!(top_mcap != 0, LaunchError::GenesisStairTopUnset);
    let previous = g.stair_top_mcap;
    g.stair_top_mcap = top_mcap;
    drop(g);
    emit_cpi!(GenesisStairTopUpdated { previous, next: top_mcap });
    Ok(())
}

///
///
///
///
///
///
pub fn set_genesis_whitelist(ctx: Context<SetGenesis>, root: [u8; 32], phase: u8) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let mut g = ctx.accounts.genesis_sale.load_mut()?;
    require_keys_eq!(ctx.accounts.authority.key(), g.authority, LaunchError::NotGenesisAuthority);
    require!(!g.opened(now), LaunchError::GenesisScheduleFrozen);
    require!(phase == GENESIS_PHASE_CORE || phase == GENESIS_PHASE_FF, LaunchError::GenesisWrongPhase);
    require!(root != [0u8; 32], LaunchError::GenesisWhitelistUnset);
    let slot = if phase == GENESIS_PHASE_CORE { &mut g.whitelist_root } else { &mut g.ff_root };
    let previous = *slot;
    *slot = root;
    drop(g);
    emit_cpi!(GenesisWhitelistUpdated { previous, next: root, phase });
    Ok(())
}





///
///
///
///
///
pub fn transfer_genesis_authority(ctx: Context<SetGenesis>, new_authority: Pubkey) -> Result<()> {
    let mut g = ctx.accounts.genesis_sale.load_mut()?;
    require_keys_eq!(ctx.accounts.authority.key(), g.authority, LaunchError::NotGenesisAuthority);
    g.pending_authority = new_authority;
    let previous_authority = g.authority;
    drop(g);
    emit_cpi!(GenesisAuthorityTransferStarted { previous_authority, new_authority });
    Ok(())
}

///
///
///
pub fn accept_genesis_authority(ctx: Context<SetGenesis>) -> Result<()> {
    let signer = ctx.accounts.authority.key();
    let mut g = ctx.accounts.genesis_sale.load_mut()?;
    require!(g.pending_authority != Pubkey::default(), LaunchError::NotGenesisAuthority);
    require_keys_eq!(signer, g.pending_authority, LaunchError::NotGenesisAuthority);
    let previous_authority = g.authority;
    g.authority = signer;
    g.pending_authority = Pubkey::default();
    drop(g);
    emit_cpi!(GenesisAuthorityTransferred { previous_authority, new_authority: signer });
    Ok(())
}





#[event_cpi]
#[derive(Accounts)]
pub struct Contribute<'info> {
    #[account(mut)]
    pub contributor: Signer<'info>,
    #[account(mut, seeds = [GENESIS_SALE_SEED], bump)]
    pub genesis_sale: AccountLoader<'info, GenesisSale>,
    #[account(mut, address = genesis_sale.load()?.launch @ LaunchError::BadConfig)]
    pub launch: AccountLoader<'info, Launch>,
///
    #[account(mut, seeds = [SALE_VAULT_SEED, launch.key().as_ref()], bump)]
    pub sale_vault: UncheckedAccount<'info>,
///
///
    #[account(mut, seeds = [GENESIS_CONTRIBUTION_SEED, contributor.key().as_ref()], bump)]
    pub contribution: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

///
///
///
///
///
///
///
pub fn contribute(ctx: Context<Contribute>, amount: u64, proof: Vec<[u8; 32]>, phase: u8) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let contributor = ctx.accounts.contributor.key();
    let raised_phase = {
        let g = ctx.accounts.genesis_sale.load()?;
        let state = ctx.accounts.launch.load()?.state;
        require!(
            g.opened(now) && now < g.sale_end() && g.graduated == 0 && state == LAUNCH_STATE_GENESIS,
            LaunchError::GenesisSaleNotActive
        );
        require!(phase != 0 && g.phase_at(now) == phase, LaunchError::GenesisWrongPhase);
        let root = if phase == GENESIS_PHASE_CORE { &g.whitelist_root } else { &g.ff_root };
        require!(
            crate::whitelist::verify(root, &contributor.to_bytes(), &proof),
            LaunchError::GenesisNotWhitelisted
        );
        if phase == GENESIS_PHASE_CORE { g.raised_core } else { g.raised_ff }
    };
    require!(amount > 0, LaunchError::GenesisZeroContribution);
    let core = phase == GENESIS_PHASE_CORE;


    let phase_cap = if core { GENESIS_CORE_CAP_LAMPORTS } else { GENESIS_FF_CAP_LAMPORTS };
    let left = phase_cap.saturating_sub(raised_phase);
    require!(left > 0, LaunchError::GenesisPhaseSoldOut);
    let amount = amount.min(left);

    let info = ctx.accounts.contribution.to_account_info();
    let first = info.data_is_empty();

    let (before_total, before_ff) = if first {
        (0, 0)
    } else {
        require_keys_eq!(*info.owner, crate::ID, LaunchError::BadConfig);
        let data = info.try_borrow_data()?;
        require!(data.len() >= GenesisContribution::LEN && data[..8] == *GenesisContribution::DISCRIMINATOR, LaunchError::BadConfig);
        let c: &GenesisContribution = bytemuck::from_bytes(&data[8..GenesisContribution::LEN]);
        (c.contributed, c.contributed_ff)
    };
    let (before_phase, cap) = if core {
        (before_total.checked_sub(before_ff).ok_or(LaunchError::MathOverflow)?, GENESIS_CORE_WALLET_CAP_LAMPORTS)
    } else {
        (before_ff, GENESIS_FF_WALLET_CAP_LAMPORTS)
    };
    let phase_user_total = before_phase.checked_add(amount).ok_or(LaunchError::MathOverflow)?;
    require!(phase_user_total <= cap, LaunchError::GenesisWalletCapExceeded);
    if first {
        let bump = [ctx.bumps.contribution];
        crate::sale::create_pda(
            &ctx.accounts.contributor.to_account_info(),
            &info,
            GenesisContribution::LEN,
            &crate::ID,
            &[GENESIS_CONTRIBUTION_SEED, contributor.as_ref(), &bump],
        )?;
        let mut data = info.try_borrow_mut_data()?;
        data[..8].copy_from_slice(GenesisContribution::DISCRIMINATOR);
        let c: &mut GenesisContribution = bytemuck::from_bytes_mut(&mut data[8..GenesisContribution::LEN]);
        c.bump = ctx.bumps.contribution;
        c.wallet = contributor;
    }
    system_program::transfer(
        CpiContext::new(
            system_program::ID,
            system_program::Transfer {
                from: ctx.accounts.contributor.to_account_info(),
                to: ctx.accounts.sale_vault.to_account_info(),
            },
        ),
        amount,
    )?;

    let user_total = {
        require_keys_eq!(*info.owner, crate::ID, LaunchError::BadConfig);
        let mut data = info.try_borrow_mut_data()?;
        require!(data[..8] == *GenesisContribution::DISCRIMINATOR, LaunchError::BadConfig);
        let c: &mut GenesisContribution = bytemuck::from_bytes_mut(&mut data[8..GenesisContribution::LEN]);
        c.contributed = c.contributed.checked_add(amount).ok_or(LaunchError::MathOverflow)?;
        if !core {
            c.contributed_ff = c.contributed_ff.checked_add(amount).ok_or(LaunchError::MathOverflow)?;
        }
        c.contributed
    };
    let mut g = ctx.accounts.genesis_sale.load_mut()?;
    if first {
        g.contributor_count = g.contributor_count.checked_add(1).ok_or(LaunchError::MathOverflow)?;
    }
    g.raised = g.raised.checked_add(amount).ok_or(LaunchError::MathOverflow)?;
    let phase_raised_total = if core {
        g.raised_core = g.raised_core.checked_add(amount).ok_or(LaunchError::MathOverflow)?;
        g.raised_core
    } else {
        g.raised_ff = g.raised_ff.checked_add(amount).ok_or(LaunchError::MathOverflow)?;
        g.raised_ff
    };
    let (raised_total, contributor_count) = (g.raised, g.contributor_count);
    drop(g);


    {
        let mut l = ctx.accounts.launch.load_mut()?;
        l.raised = raised_total;
        l.buyers = contributor_count;
    }
    emit_cpi!(Contributed {
        user: contributor,
        amount,
        user_total,
        raised_total,
        contributor_count,
        phase,
        phase_user_total,
        phase_raised_total,
    });
    Ok(())
}





#[event_cpi]
#[derive(Accounts)]
pub struct EndGenesisSale<'info> {
///
///
    pub caller: Signer<'info>,
    #[account(mut, seeds = [GENESIS_SALE_SEED], bump)]
    pub genesis_sale: AccountLoader<'info, GenesisSale>,
    #[account(mut, address = genesis_sale.load()?.launch @ LaunchError::BadConfig)]
    pub launch: AccountLoader<'info, Launch>,

///
    #[account(seeds = [LAUNCH_AUTHORITY_SEED], bump)]
    pub launch_authority: UncheckedAccount<'info>,
///
    #[account(seeds = [HOOK_CONFIG_SEED], bump, seeds::program = FLOOR_HOOK_PROGRAM)]
    pub hook_config: UncheckedAccount<'info>,
///
///
    #[account(seeds = [FLOOR_PLATFORM_POOL_SEED], bump, seeds::program = FLOOR_HOOK_PROGRAM)]
    pub floor_platform_pool: UncheckedAccount<'info>,
///
    #[account(mut, address = launch.load()?.floor_pool @ LaunchError::BadConfig, owner = FLOOR_HOOK_PROGRAM @ LaunchError::BadConfig)]
    pub floor_pool: UncheckedAccount<'info>,
///
    #[account(address = FLOOR_HOOK_PROGRAM)]
    pub floor_hook: UncheckedAccount<'info>,
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
pub fn end_genesis_sale(ctx: Context<EndGenesisSale>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let (tick, clamped, top_mcap, steps) = {
        let mut g = ctx.accounts.genesis_sale.load_mut()?;
        require!(g.open_time != 0, LaunchError::GenesisNotScheduled);

        require_keys_eq!(ctx.accounts.caller.key(), g.authority, LaunchError::GenesisGraduationDeployerOnly);

        require!(now >= g.core_end(), LaunchError::GenesisSaleNotEnded);
        let mut l = ctx.accounts.launch.load_mut()?;
        require!(l.state == LAUNCH_STATE_GENESIS && g.sale_closed == 0, LaunchError::GenesisStepOutOfOrder);

        g.duration = (now - g.open_time) as u64;



        let (tick, clamped) = floor_math::genesis_open_tick().map_err(|_| error!(LaunchError::MathOverflow))?;
        let (core_sold, ff_sold) =
            floor_math::genesis::genesis_sold(g.raised_core, g.raised_ff).map_err(|_| error!(LaunchError::MathOverflow))?;
        g.sold_tokens = core_sold + ff_sold;




        let steps = floor_math::genesis_layout::genesis_layout_steps(g.stair_top_mcap, tick, FLOOR_TICK_SPACING as i32)
            .map_err(|_| error!(LaunchError::MathOverflow))?;
        g.open_tick = tick;
        g.opening_tick_clamped = clamped as u8;
        g.sale_closed = 1;
        g.stair_steps = steps;
        l.open_tick = tick;
        l.state = LAUNCH_STATE_GENESIS_CLOSED;
        (tick, clamped, g.stair_top_mcap, steps)
    };
    set_genesis_stair(&ctx, steps)?;
    require!(
        crate::graduate::pool_stair_steps(&ctx.accounts.floor_pool.to_account_info())? == steps,
        LaunchError::StairDropped
    );


    require!(
        crate::graduate::pool_stair_owner(&ctx.accounts.floor_pool.to_account_info())?
            == floor_types::constants::STAIR_OWNER_TEAM,
        LaunchError::StairOwnerDropped
    );


    emit_cpi!(OpeningTick { tick, clamped, stair_steps: steps, stair_top_mcap: top_mcap });
    Ok(())
}

///
///
fn set_genesis_stair(ctx: &Context<EndGenesisSale>, steps: u8) -> Result<()> {
    let a = &ctx.accounts;
    let mut data = disc::hook::SET_GENESIS_STAIR.to_vec();
    data.push(steps);
    data.extend_from_slice(&GENESIS_STAIR_GROWTH_BPS.to_le_bytes());

    data.push(floor_types::constants::STAIR_OWNER_TEAM);
    let ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: FLOOR_HOOK_PROGRAM,
        accounts: vec![
            AccountMeta::new_readonly(a.launch_authority.key(), true),
            AccountMeta::new_readonly(a.hook_config.key(), false),
            AccountMeta::new_readonly(a.floor_platform_pool.key(), false),
            AccountMeta::new(a.floor_pool.key(), false),
        ],
        data,
    };
    let infos = [
        a.launch_authority.to_account_info(),
        a.hook_config.to_account_info(),
        a.floor_platform_pool.to_account_info(),
        a.floor_pool.to_account_info(),
        a.floor_hook.to_account_info(),
    ];
    let bump = [ctx.bumps.launch_authority];
    anchor_lang::solana_program::program::invoke_signed(&ix, &infos, &[&[LAUNCH_AUTHORITY_SEED, &bump]])?;
    Ok(())
}





#[event_cpi]
#[derive(Accounts)]
pub struct ClaimGenesis<'info> {
    #[account(mut)]
    pub claimant: Signer<'info>,
    #[account(mut, seeds = [GENESIS_SALE_SEED], bump)]
    pub genesis_sale: AccountLoader<'info, GenesisSale>,
    #[account(address = genesis_sale.load()?.launch @ LaunchError::BadConfig)]
    pub launch: AccountLoader<'info, Launch>,
///
///
    #[account(mut, seeds = [GENESIS_CONTRIBUTION_SEED, claimant.key().as_ref()], bump)]
    pub contribution: UncheckedAccount<'info>,
    #[account(address = genesis_sale.load()?.mint @ LaunchError::BadConfig)]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, address = launch.load()?.token_vault @ LaunchError::BadConfig)]
    pub token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
///
    #[account(mut)]
    pub claimant_ata: UncheckedAccount<'info>,
    pub token_2022: Program<'info, Token2022>,
    pub ata_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
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
pub fn claim_genesis(ctx: Context<ClaimGenesis>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let sale_end = {
        let g = ctx.accounts.genesis_sale.load()?;
        require!(g.graduated == 1, LaunchError::GenesisNotGraduated);
        g.sale_end()
    };
    let info = ctx.accounts.contribution.to_account_info();
    if info.data_is_empty() {
        return Ok(());
    }
    require_keys_eq!(*info.owner, crate::ID, LaunchError::BadConfig);
    let (contributed, contributed_ff, claimed_before, settled) = {
        let data = info.try_borrow_data()?;
        require!(data[..8] == *GenesisContribution::DISCRIMINATOR, LaunchError::BadConfig);
        let c: &GenesisContribution = bytemuck::from_bytes(&data[8..GenesisContribution::LEN]);


        require!(c.refunded == 0, LaunchError::GenesisRefundWindowOpen);
        (c.contributed, c.contributed_ff, c.claimed, c.settled)
    };
    if contributed == 0 || settled != 0 {
        return Ok(());
    }

    let elapsed = now.saturating_sub(sale_end);
    require!(elapsed >= floor_types::constants::GENESIS_BUYER_CLIFF_SECS as i64, LaunchError::GenesisNothingVested);
    let mut g = ctx.accounts.genesis_sale.load_mut()?;


    let core = contributed.checked_sub(contributed_ff).ok_or(LaunchError::MathOverflow)?;
    let allocation =
        floor_math::genesis_allocation(core, contributed_ff).map_err(|_| error!(LaunchError::MathOverflow))?;
    let total = floor_math::genesis::genesis_buyer_vested(allocation, elapsed);
    let amount = total.checked_sub(claimed_before).ok_or(LaunchError::MathOverflow)?;
    let settles = total == allocation;
    if amount == 0 && !settles {
        return Ok(());
    }
    g.total_claimed = g.total_claimed.checked_add(amount).ok_or(LaunchError::MathOverflow)?;
    if settles {
        g.settled_contributor_count = g.settled_contributor_count.checked_add(1).ok_or(LaunchError::MathOverflow)?;
    }
    if settles && g.settled_contributor_count == g.contributor_count {


        g.allocation_dust_pending = g
            .sold_tokens
            .checked_sub(g.total_claimed)
            .and_then(|x| x.checked_sub(g.allocation_dust_routed))
            .ok_or(LaunchError::MathOverflow)?;
    }
    let total_claimed = g.total_claimed;
    drop(g);
    {
        let mut data = info.try_borrow_mut_data()?;
        let c: &mut GenesisContribution = bytemuck::from_bytes_mut(&mut data[8..GenesisContribution::LEN]);
        c.claimed = total;
        c.settled = settles as u8;
    }

    if amount != 0 {
        let a = &ctx.accounts;
        associated_token::create_idempotent(CpiContext::new(
            a.ata_program.key(),
            associated_token::Create {
                payer: a.claimant.to_account_info(),
                associated_token: a.claimant_ata.to_account_info(),
                authority: a.claimant.to_account_info(),
                mint: a.mint.to_account_info(),
                system_program: a.system_program.to_account_info(),
                token_program: a.token_2022.to_account_info(),
            },
        ))?;
        let (mint_key, bump) = {
            let l = a.launch.load()?;
            (l.mint, l.bump)
        };
        let bump = [bump];
        let seeds: &[&[u8]] = &[LAUNCH_SEED, mint_key.as_ref(), &bump];
        token_interface::transfer_checked(
            CpiContext::new_with_signer(
                a.token_2022.key(),
                TransferChecked {
                    from: a.token_vault.to_account_info(),
                    mint: a.mint.to_account_info(),
                    to: a.claimant_ata.to_account_info(),
                    authority: a.launch.to_account_info(),
                },
                &[seeds],
            ),
            amount,
            TOKEN_DECIMALS,
        )?;
    }
    emit_cpi!(GenesisClaimed {
        user: ctx.accounts.claimant.key(),
        amount,
        cumulative_user_claimed: total,
        total_claimed,
    });
    Ok(())
}





#[event_cpi]
#[derive(Accounts)]
pub struct RefundGenesis<'info> {
///
    #[account(mut)]
    pub contributor: Signer<'info>,
    #[account(mut, seeds = [GENESIS_SALE_SEED], bump)]
    pub genesis_sale: AccountLoader<'info, GenesisSale>,
    #[account(address = genesis_sale.load()?.launch @ LaunchError::BadConfig)]
    pub launch: AccountLoader<'info, Launch>,
///
///
    #[account(mut, seeds = [SALE_VAULT_SEED, launch.key().as_ref()], bump)]
    pub sale_vault: UncheckedAccount<'info>,
///
///
    #[account(mut, seeds = [GENESIS_CONTRIBUTION_SEED, contributor.key().as_ref()], bump)]
    pub contribution: UncheckedAccount<'info>,
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
pub fn refund_genesis(ctx: Context<RefundGenesis>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    {
        let g = ctx.accounts.genesis_sale.load()?;
        require!(
            g.sale_closed == 1 && g.graduated == 0 && now >= g.refund_opens_at(),
            LaunchError::GenesisRefundNotOpen
        );
    }
    let info = ctx.accounts.contribution.to_account_info();
    require!(!info.data_is_empty(), LaunchError::GenesisNothingToRefund);
    require_keys_eq!(*info.owner, crate::ID, LaunchError::BadConfig);
    let amount = {
        let mut data = info.try_borrow_mut_data()?;
        require!(
            data.len() >= GenesisContribution::LEN && data[..8] == *GenesisContribution::DISCRIMINATOR,
            LaunchError::BadConfig
        );
        let c: &mut GenesisContribution = bytemuck::from_bytes_mut(&mut data[8..GenesisContribution::LEN]);
        require!(c.contributed > 0 && c.refunded == 0, LaunchError::GenesisNothingToRefund);

        c.refunded = c.contributed;
        c.contributed
    };

    let vault = ctx.accounts.sale_vault.to_account_info();
    let reserve = Rent::get()?.minimum_balance(vault.data_len());
    let left = vault.lamports().checked_sub(amount).ok_or(LaunchError::EscrowShort)?;
    require!(left >= reserve, LaunchError::EscrowShort);
    **vault.try_borrow_mut_lamports()? = left;
    let wallet = ctx.accounts.contributor.to_account_info();
    let credited = wallet.lamports().checked_add(amount).ok_or(LaunchError::MathOverflow)?;
    **wallet.try_borrow_mut_lamports()? = credited;
    let refunded_total = {
        let mut g = ctx.accounts.genesis_sale.load_mut()?;
        g.refunded = g.refunded.checked_add(amount).ok_or(LaunchError::MathOverflow)?;
        g.refunded
    };
    emit_cpi!(GenesisRefunded { wallet: ctx.accounts.contributor.key(), amount, refunded_total });
    Ok(())
}





#[event_cpi]
#[derive(Accounts)]
pub struct FlushFloorFees<'info> {
///
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, seeds = [GENESIS_SALE_SEED], bump)]
    pub genesis_sale: AccountLoader<'info, GenesisSale>,
    #[account(address = genesis_sale.load()?.launch @ LaunchError::BadConfig)]
    pub launch: AccountLoader<'info, Launch>,
///
    #[account(mut, seeds = [SALE_VAULT_SEED, launch.key().as_ref()], bump)]
    pub sale_vault: UncheckedAccount<'info>,
    #[account(mut, address = launch.load()?.wsol_vault @ LaunchError::BadConfig)]
    pub wsol_vault: Box<InterfaceAccount<'info, TokenAccount>>,
///
    #[account(mut, address = launch.load()?.floor_pool @ LaunchError::BadConfig)]
    pub floor_pool: UncheckedAccount<'info>,
///
    #[account(mut, address = launch.load()?.amm_pool @ LaunchError::BadConfig)]
    pub pool: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub vault0: UncheckedAccount<'info>,
///
    #[account(address = WSOL_MINT @ LaunchError::BadConfig)]
    pub wsol_mint: UncheckedAccount<'info>,
    pub spl_token: Interface<'info, TokenInterface>,
///
    #[account(address = CARPENTER_AMM_PROGRAM)]
    pub carpenter_amm: UncheckedAccount<'info>,
///
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = CARPENTER_AMM_PROGRAM)]
    pub amm_event_authority: UncheckedAccount<'info>,
///
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = FLOOR_HOOK_PROGRAM)]
    pub hook_event_authority: UncheckedAccount<'info>,
///
    #[account(address = FLOOR_HOOK_PROGRAM)]
    pub floor_hook: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

///
///
///
pub fn flush_floor_fees(ctx: Context<FlushFloorFees>) -> Result<()> {
    require!(ctx.accounts.genesis_sale.load()?.graduated == 1, LaunchError::GenesisNotGraduated);
    let a = &ctx.accounts;
    let sale_vault = a.sale_vault.to_account_info();
    let wsol_vault = a.wsol_vault.to_account_info();
    let lamports = sale_vault.lamports().saturating_sub(Rent::get()?.minimum_balance(sale_vault.data_len()));
    graduate::wrap_lamports(&a.payer.to_account_info(), &sale_vault, &wsol_vault, lamports)?;
    let amount = graduate::token_amount(&wsol_vault)?;
    if amount == 0 {
        return Ok(());
    }
    let (mint_key, bump) = {
        let l = a.launch.load()?;
        (l.mint, l.bump)
    };
    let bump = [bump];
    let seeds: &[&[u8]] = &[LAUNCH_SEED, mint_key.as_ref(), &bump];
    seed_floor_cpi(
        &a.launch.to_account_info(),
        &a.floor_pool.to_account_info(),
        &a.pool.to_account_info(),
        &wsol_vault,
        &a.vault0.to_account_info(),
        &a.wsol_mint.to_account_info(),
        &a.spl_token.to_account_info(),
        &a.carpenter_amm.to_account_info(),
        &a.amm_event_authority.to_account_info(),
        &a.hook_event_authority.to_account_info(),
        &a.floor_hook.to_account_info(),
        amount,
        seeds,
    )?;
    let mut g = ctx.accounts.genesis_sale.load_mut()?;
    g.fees_applied = g.fees_applied.checked_add(amount).ok_or(LaunchError::MathOverflow)?;
    let cumulative_applied = g.fees_applied;
    drop(g);
    emit_cpi!(FloorFeesApplied { amount, cumulative_applied });
    Ok(())
}

///
///
///
#[allow(clippy::too_many_arguments)]
pub(crate) fn seed_floor_cpi<'info>(
    launch: &AccountInfo<'info>,
    floor_pool: &AccountInfo<'info>,
    pool: &AccountInfo<'info>,
    source: &AccountInfo<'info>,
    vault0: &AccountInfo<'info>,
    quote_mint: &AccountInfo<'info>,
    spl_token: &AccountInfo<'info>,
    carpenter_amm: &AccountInfo<'info>,
    amm_event_authority: &AccountInfo<'info>,
    hook_event_authority: &AccountInfo<'info>,
    floor_hook: &AccountInfo<'info>,
    amount: u64,
    launch_seeds: &[&[u8]],
) -> Result<()> {
    let mut data = disc::hook::SEED_FLOOR.to_vec();
    data.extend_from_slice(&amount.to_le_bytes());
    data.extend_from_slice(&0u64.to_le_bytes());
    graduate::cpi(
        FLOOR_HOOK_PROGRAM,
        data,
        &[
            r_s(launch),
            w(floor_pool),
            w(pool),
            w(source),
            w(vault0),
            r(quote_mint),
            r(spl_token),
            r(carpenter_amm),
            r(amm_event_authority),
            r(hook_event_authority),
            r(floor_hook),
        ],
        &[launch_seeds],
    )
}





#[event_cpi]
#[derive(Accounts)]
pub struct FlushAllocationDust<'info> {
    #[account(mut, seeds = [GENESIS_SALE_SEED], bump)]
    pub genesis_sale: AccountLoader<'info, GenesisSale>,
    #[account(address = genesis_sale.load()?.launch @ LaunchError::BadConfig)]
    pub launch: AccountLoader<'info, Launch>,
///
    #[account(mut, address = launch.load()?.floor_pool @ LaunchError::BadConfig)]
    pub floor_pool: UncheckedAccount<'info>,
///
    #[account(mut, address = launch.load()?.amm_pool @ LaunchError::BadConfig)]
    pub pool: UncheckedAccount<'info>,
    #[account(mut, address = launch.load()?.token_vault @ LaunchError::BadConfig)]
    pub token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
///
    #[account(mut)]
    pub vault1: UncheckedAccount<'info>,
    #[account(address = genesis_sale.load()?.mint @ LaunchError::BadConfig)]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    pub token_2022: Program<'info, Token2022>,
///
    #[account(address = CARPENTER_AMM_PROGRAM)]
    pub carpenter_amm: UncheckedAccount<'info>,
///
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = CARPENTER_AMM_PROGRAM)]
    pub amm_event_authority: UncheckedAccount<'info>,
///
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = FLOOR_HOOK_PROGRAM)]
    pub hook_event_authority: UncheckedAccount<'info>,
///
    #[account(address = FLOOR_HOOK_PROGRAM)]
    pub floor_hook: UncheckedAccount<'info>,
}

///
///
///
pub fn flush_allocation_dust(ctx: Context<FlushAllocationDust>) -> Result<()> {
    let amount = {
        let mut g = ctx.accounts.genesis_sale.load_mut()?;
        require!(g.graduated == 1, LaunchError::GenesisNotGraduated);
        require!(g.settled_contributor_count == g.contributor_count, LaunchError::GenesisAllocationsOutstanding);
        let amount = g.allocation_dust_pending;
        if amount == 0 {
            return Ok(());
        }
        g.allocation_dust_pending = 0;
        g.allocation_dust_routed = g.allocation_dust_routed.checked_add(amount).ok_or(LaunchError::MathOverflow)?;
        amount
    };
    let a = &ctx.accounts;
    let (mint_key, bump) = {
        let l = a.launch.load()?;
        (l.mint, l.bump)
    };
    let bump = [bump];
    let seeds: &[&[u8]] = &[LAUNCH_SEED, mint_key.as_ref(), &bump];
    let mut data = disc::hook::DONATE_INVENTORY.to_vec();
    data.extend_from_slice(&amount.to_le_bytes());


    let launch = a.launch.to_account_info();
    let floor_pool = a.floor_pool.to_account_info();
    let pool = a.pool.to_account_info();
    let token_vault = a.token_vault.to_account_info();
    let vault1 = a.vault1.to_account_info();
    let mint = a.mint.to_account_info();
    let token_2022 = a.token_2022.to_account_info();
    let amm = a.carpenter_amm.to_account_info();
    let amm_ev = a.amm_event_authority.to_account_info();
    let hook_ev = a.hook_event_authority.to_account_info();
    let hook = a.floor_hook.to_account_info();
    graduate::cpi(
        FLOOR_HOOK_PROGRAM,
        data,
        &[
            r_s(&launch),
            w(&floor_pool),
            w(&pool),
            w(&token_vault),
            w(&vault1),
            r(&mint),
            r(&token_2022),
            r(&amm),
            r(&amm_ev),
            r(&hook_ev),
            r(&hook),
        ],
        &[seeds],
    )?;
    emit_cpi!(AllocationDustRouted { amount });
    Ok(())
}

///
pub(crate) fn require_state(l: &Launch, want: u8) -> Result<()> {
    require!(l.state == want, LaunchError::GenesisStepOutOfOrder);
    Ok(())
}

#[allow(dead_code)]
const _STATES_DISTINCT: () = assert!(
    LAUNCH_STATE_GENESIS != LAUNCH_STATE_GRADUATED
        && LAUNCH_STATE_GENESIS_CLOSED != LAUNCH_STATE_GRADUATED
        && LAUNCH_STATE_GENESIS != LAUNCH_STATE_BONDING
);

///
///
///
pub fn genesis_slots_ok(t: &floor_types::slots::PayeeSlots, authority: &Pubkey) -> bool {
    use floor_types::slots::*;
    let c0 = t.creator[0];
    t.version == PAYEE_SLOTS_VERSION
        && t.creator_count == 1
        && t.team_count == 0
        && t.creator_pending_mask == 0
        && t.team_pending_mask == 0
        && c0.key == authority.to_bytes()
        && c0.platform == 0
        && c0.state == SLOT_STATE_WALLET
        && c0.bps == SLOTS_TOTAL_BPS
}

#[cfg(test)]
mod tests {
    use super::*;

///
///
///
///
    #[test]
    fn mr_s_genesis_takes_one_address_creator_slot_the_authority_at_ten_thousand() {
        use floor_types::slots::*;
        let auth = Pubkey::new_from_array([9; 32]);
        let mut ok = PayeeSlots::default();
        ok.fill(SLOT_KIND_CREATOR, &[SlotArg { bps: 10_000, platform: 0, key: auth.to_bytes() }]);
        assert!(genesis_slots_ok(&ok, &auth));
        assert!(!genesis_slots_ok(&ok, &Pubkey::new_from_array([8; 32])), "another wallet");
        assert!(!genesis_slots_ok(&PayeeSlots::default(), &auth), "never written");
        let mut two = PayeeSlots::default();
        two.fill(
            SLOT_KIND_CREATOR,

            &[SlotArg { bps: 10_000, platform: 0, key: auth.to_bytes() }, SlotArg { bps: 1, platform: 0, key: [7; 32] }],
        );
        assert!(!genesis_slots_ok(&two, &auth), "two creator slots");
        let mut pending = PayeeSlots::default();
        pending.fill(SLOT_KIND_CREATOR, &[SlotArg { bps: 10_000, platform: 1, key: auth.to_bytes() }]);
        assert!(!genesis_slots_ok(&pending, &auth), "a pending id is never the authority's wallet");
        let mut team = ok;
        team.fill(SLOT_KIND_TEAM, &[SlotArg { bps: 10_000, platform: 0, key: auth.to_bytes() }]);
        assert!(!genesis_slots_ok(&team, &auth), "a team slot at create");
        let mut short = ok;
        short.creator[0].bps = 9_999;
        assert!(!genesis_slots_ok(&short, &auth), "9,999 bps");
        let mut v = ok;
        v.version = 2;
        assert!(!genesis_slots_ok(&v, &auth), "an unknown version");
    }

///
    #[test]
    fn sgr_l1_the_genesis_clamp_is_inside_the_open_bounds() {
        use floor_types::constants::{GENESIS_OPEN_TICK_MAX, GENESIS_OPEN_TICK_MIN};
        assert!(crate::sale::open_tick_in_bounds(GENESIS_OPEN_TICK_MAX));
        assert!(!crate::sale::open_tick_in_bounds(GENESIS_OPEN_TICK_MAX + 1));
        assert!(crate::sale::open_tick_in_bounds(GENESIS_OPEN_TICK_MIN));
    }

///
///
    #[test]
    fn sgr_l2_lock_rent_is_a_pair_per_lock_and_there_are_four() {
        let rent = Rent::default();
        assert_eq!(GENESIS_LOCKS.len(), 4);
        assert_eq!(lock_rent(&rent, 170), 4 * (rent.minimum_balance(248) + rent.minimum_balance(170)));
    }

///
///
///
///
    #[test]
    fn sgr_l3_the_genesis_path_carries_no_wall_and_places_the_staircase() {
        let genesis = include_str!("genesis.rs");
        let graduate = include_str!("genesis_graduate.rs");
        let code = |src: &str| -> String { src.lines().filter(|l| !l.trim_start().starts_with("//")).collect::<Vec<_>>().join("\n") };
        let (genesis, graduate) = (code(genesis), code(graduate));
        for bad in [["GENESIS_WALL", "_MULTIPLE"].concat(), ["wall_ticks", "_centi("].concat()] {
            assert!(!genesis.contains(bad.as_str()) && !graduate.contains(bad.as_str()), "{bad}");
        }
        assert!(!graduate.contains(["snap.wall_min", "_start_ticks"].concat().as_str()), "a staircase reads no minimum start");
        assert!(graduate.contains("graduate::stair_seed_accounts(ra, &checked.pool"), "seed_inventory places the staircase");
        assert!(graduate.contains("outstanding(ra, &rent, vault0_len, vault1_len, quote_pools, true)"), "its rows are charged");

        assert!(!genesis.contains(["stair_rent_lamports", "(&rent))"].concat().as_str()), "the staircase's rows are not prepaid twice");
        assert!(genesis.contains("genesis_layout_steps(g.stair_top_mcap, tick"), "the count off the top and the FIXED open, at least 4 (layout F)");
        assert!(genesis.contains("floor_math::genesis_open_tick()"), "the open is the fixed price (2026-10-07)");
        assert!(genesis.contains("disc::hook::SET_GENESIS_STAIR"), "written into the pool");
        assert!(genesis.contains("open_time == 0 || g.stair_top_mcap != 0"), "no open without a top");
    }
}
