//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::system_program;
use anchor_spl::associated_token::{self, get_associated_token_address_with_program_id, AssociatedToken};
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenInterface};
use floor_types::constants::{
    DYNAMIC_FEE_FLAG, FLOOR_TICK_SPACING, LAUNCH_STATE_BONDING,
};
use floor_types::layout::{Layout, QuotePolicy as QuotePolicyLayout};
use floor_types::metadata::metadata_uri;
use floor_types::register::RegisterPoolArgs;

use crate::constants::{
    holders_disc, AMM_CONFIG_SEED, APPROVED_HOOK_SEED, EVENT_AUTHORITY_SEED, CARPENTER_AMM_PROGRAM,
    FLOOR_HOLDERS_PROGRAM, FLOOR_HOOK_PROGRAM, GRAD_RENT_SEED, HOOK_AUTHORITY_SEED, HOOK_CONFIG_SEED,
    LAUNCH_AUTHORITY_SEED, LAUNCH_CONFIG_SEED, LAUNCH_SEED, MINT_AUTHORITY_SEED, MINT_EXCLUSIVITY_SEED,
    MINT_POOLS_SEED, MINT_SEED, REGISTRY_ENTRY_SEED, SALE_VAULT_SEED, TRADER_LINK_SEED, TRADER_REF_SALE_SEED, WSOL_MINT,
};
use crate::graduation_rent;
use crate::errors::LaunchError;
use crate::events::{GraduationRentReserved, LaunchCreated, PayeeSlotsSet, QuoteChainPriced, TeamBought};
use crate::lock::{Lock, LOCK_AUTHORITY_SEED};
use crate::mint_profile::{self, MintProfile};
use crate::sale::{self, create_pda, ResolvedConfig};
use crate::state::{CreateLaunchParams, Launch, LaunchConfig, RegistryEntry, U128};

#[event_cpi]
#[derive(Accounts)]
#[instruction(params: CreateLaunchParams, nonce: u64)]
pub struct CreateLaunch<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(mut, seeds = [LAUNCH_CONFIG_SEED], bump)]
    pub launch_config: AccountLoader<'info, LaunchConfig>,
///
///
///
///
///
///
///
    #[account(
        init,
        payer = creator,
        space = RegistryEntry::LEN,
        seeds = [REGISTRY_ENTRY_SEED, &launch_config.load()?.registry_count.to_le_bytes()],
        bump,
    )]
    pub registry_entry: AccountLoader<'info, RegistryEntry>,
///
    #[account(mut, seeds = [MINT_SEED, creator.key().as_ref(), &nonce.to_le_bytes()], bump)]
    pub mint: UncheckedAccount<'info>,
///
    #[account(seeds = [MINT_AUTHORITY_SEED], bump)]
    pub mint_authority: UncheckedAccount<'info>,
///
    #[account(seeds = [LOCK_AUTHORITY_SEED], bump)]
    pub lock_authority: UncheckedAccount<'info>,
///
    #[account(mut, seeds = [LAUNCH_SEED, mint.key().as_ref()], bump)]
    pub launch: UncheckedAccount<'info>,
///
    #[account(mut, seeds = [SALE_VAULT_SEED, launch.key().as_ref()], bump)]
    pub sale_vault: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub token_vault: UncheckedAccount<'info>,
///
///
///
///
///
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
///
///
///
    #[account(mut)]
    pub wsol_vault: UncheckedAccount<'info>,
    pub token_2022: Program<'info, Token2022>,
///
///
    pub quote_token_program: Interface<'info, TokenInterface>,
    pub ata_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
///
    #[account(seeds = [LAUNCH_AUTHORITY_SEED], bump)]
    pub launch_authority: UncheckedAccount<'info>,
///
    #[account(address = FLOOR_HOOK_PROGRAM)]
    pub floor_hook: UncheckedAccount<'info>,
///
    #[account(seeds = [HOOK_CONFIG_SEED], bump, seeds::program = FLOOR_HOOK_PROGRAM)]
    pub hook_config: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub floor_pool: UncheckedAccount<'info>,
///
    #[account(address = CARPENTER_AMM_PROGRAM)]
    pub carpenter_amm: UncheckedAccount<'info>,
///
    #[account(seeds = [AMM_CONFIG_SEED], bump, seeds::program = CARPENTER_AMM_PROGRAM)]
    pub amm_config: UncheckedAccount<'info>,
///
    #[account(seeds = [APPROVED_HOOK_SEED, FLOOR_HOOK_PROGRAM.as_ref()], bump, seeds::program = CARPENTER_AMM_PROGRAM)]
    pub approved_hook: UncheckedAccount<'info>,
///
    #[account(seeds = [MINT_POOLS_SEED, mint.key().as_ref()], bump, seeds::program = CARPENTER_AMM_PROGRAM)]
    pub mint_pools: UncheckedAccount<'info>,
///
    #[account(mut, seeds = [MINT_EXCLUSIVITY_SEED, mint.key().as_ref()], bump, seeds::program = CARPENTER_AMM_PROGRAM)]
    pub mint_exclusivity: UncheckedAccount<'info>,
///
    #[account(seeds = [HOOK_AUTHORITY_SEED], bump, seeds::program = FLOOR_HOOK_PROGRAM)]
    pub hook_authority: UncheckedAccount<'info>,
///
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = CARPENTER_AMM_PROGRAM)]
    pub amm_event_authority: UncheckedAccount<'info>,
///
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = FLOOR_HOOK_PROGRAM)]
    pub hook_event_authority: UncheckedAccount<'info>,
///
///
///
///
    #[account(mut, seeds = [GRAD_RENT_SEED, launch.key().as_ref()], bump)]
    pub graduation_rent: UncheckedAccount<'info>,







///
    #[account(address = FLOOR_HOLDERS_PROGRAM)]
    pub floor_holders: Option<UncheckedAccount<'info>>,
///
    #[account(mut)]
    pub holder_book: Option<UncheckedAccount<'info>>,
///
///
    #[account(mut)]
    pub extra_account_metas: Option<UncheckedAccount<'info>>,
///
    #[account(mut)]
    pub claims_registry: Option<UncheckedAccount<'info>>,
///
    #[account(mut)]
    pub claims_table: Option<UncheckedAccount<'info>>,
///
    pub claims_alt_authority: Option<UncheckedAccount<'info>>,
///
    pub alt_program: Option<UncheckedAccount<'info>>,
///
    pub holders_event_authority: Option<UncheckedAccount<'info>>,
///
///
///
///
///
    #[account(address = crate::constants::PRESET_REGISTRY_KEY)]
    pub preset_registry: UncheckedAccount<'info>,
///
///
///
///
///
///
    #[account(seeds = [TRADER_LINK_SEED, creator.key().as_ref()], bump)]
    pub creator_trader_link: UncheckedAccount<'info>,




}

///
///
///
fn load_preset(registry: &UncheckedAccount, mode: u8) -> Result<Option<floor_math::presets::PresetRow>> {
    if mode == floor_types::constants::MODE_CUSTOM {
        return Ok(None);
    }
    if registry.owner != &crate::ID {
        return err!(LaunchError::PresetUnknown);
    }
    let data = registry.try_borrow_data()?;
    Ok(sale::preset_from_registry_data(&data, mode).map_err(LaunchError::from)?)
}

///
#[derive(Clone, Copy)]
struct ConfigView {
    platform_bps: u16,
    min_bid_lamports: u64,
    min_sellable: u64,
    registry_index: u64,
///
    bounty_bps: u16,
    max_bounty_lamports: u64,
///
///
///
    team_sale_bps: u16,
    team_pool_bps: u16,
}

///
///
///
///
pub fn predict_amm_pool(quote_mint: &Pubkey, mint: &Pubkey) -> Pubkey {
    let (fee, spacing) = floor_types::seeds::pool_seed_parts(DYNAMIC_FEE_FLAG, FLOOR_TICK_SPACING);
    Pubkey::find_program_address(
        &[
            floor_types::seeds::POOL,
            quote_mint.as_ref(),
            mint.as_ref(),
            &fee,
            &spacing,
            FLOOR_HOOK_PROGRAM.as_ref(),
        ],
        &CARPENTER_AMM_PROGRAM,
    )
    .0
}

///
///
struct QuoteEconomics<'info> {
    min_bid: u64,
    max_bounty: u64,
///
///
///
    hook_accounts: Vec<AccountInfo<'info>>,
///
///
///
    walk: core::ops::Range<usize>,
///
    team_source: usize,
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
fn quote_economics<'info>(
    remaining: &[AccountInfo<'info>],
    cfg: &ConfigView,
    quote_mint: &Pubkey,
    quote_token_program: &Pubkey,
    quote_decimals: u8,
    team_buy: bool,
) -> Result<QuoteEconomics<'info>> {


    if *quote_mint == WSOL_MINT {
        require!(remaining.len() <= team_buy as usize, LaunchError::BadConfig);
        require!(remaining.is_empty(), LaunchError::BadConfig);
        return Ok(QuoteEconomics { min_bid: cfg.min_bid_lamports, max_bounty: cfg.max_bounty_lamports, hook_accounts: Vec::new(), walk: 0..0, team_source: 0 });
    }
    let acct = remaining.first().ok_or(LaunchError::QuoteNotListed)?;
    let (expected, _) =
        Pubkey::find_program_address(&[floor_types::seeds::QUOTE_POLICY, quote_mint.as_ref()], &FLOOR_HOOK_PROGRAM);
    require_keys_eq!(acct.key(), expected, LaunchError::QuoteNotListed);






    if *acct.owner != FLOOR_HOOK_PROGRAM {
        const DERIVED: usize = 7;
        require!(remaining.len() >= DERIVED + team_buy as usize, LaunchError::QuoteNotListed);
        let walk = walk_range(remaining.len(), DERIVED, team_buy)?;
        return Ok(QuoteEconomics {
            min_bid: floor_math::min_bid_for(quote_decimals),
            max_bounty: 0,
            hook_accounts: remaining[..DERIVED].to_vec(),
            team_source: walk.end,
            walk,
        });
    }



    require!(remaining.len() <= 2 + team_buy as usize, LaunchError::BadConfig);
    let q = {
        let data = acct.try_borrow_data()?;
        require!(
            data.len() == 8 + QuotePolicyLayout::SIZE && data[..8] == QuotePolicyLayout::DISCRIMINATOR,
            LaunchError::QuoteNotListed
        );
        bytemuck::pod_read_unaligned::<QuotePolicyLayout>(&data[8..8 + QuotePolicyLayout::SIZE])
    };
    require!(q.quote_mint == quote_mint.to_bytes(), LaunchError::QuoteNotListed);


    require!(q.token_program == quote_token_program.to_bytes(), LaunchError::BadConfig);
    require!(q.retired == 0, LaunchError::QuoteRetired);
    let mut hook_accounts = vec![acct.clone()];
    if let Some(r) = remaining.get(1) {

        hook_accounts.push(r.clone());
    }
    Ok(QuoteEconomics { min_bid: q.min_bid, max_bounty: q.max_bounty, hook_accounts, team_source: 2, walk: 2..2 })
}

///
///
///
///
fn walk_range(len: usize, hook_len: usize, team_buy: bool) -> Result<core::ops::Range<usize>> {
    let end = len.checked_sub(team_buy as usize).ok_or(LaunchError::QuoteChainBadHop)?;
    if end == hook_len + 1 {
        return Ok(hook_len..end);
    }
    require!(end >= hook_len + super::quote_chain::WALK_FIXED, LaunchError::QuoteChainBadHop);
    require!((end - hook_len - super::quote_chain::WALK_FIXED) % 2 == 0, LaunchError::QuoteChainBadHop);
    Ok(hook_len..end)
}

///
///
///
///
pub fn handler<'info>(
    ctx: Context<'info, CreateLaunch<'info>>,
    params: CreateLaunchParams,
    nonce: u64,
    name: String,
    symbol: String,
) -> Result<()> {

    let cfg = read_config(&ctx.accounts.launch_config)?;
    let mint_key = ctx.accounts.mint.key();

    let uri = resolve_uri(&ctx.accounts.launch_config, &mint_key)?;


    let quote_mint_key = ctx.accounts.quote_mint.key();
    let quote_program = ctx.accounts.quote_token_program.key();
    require_keys_eq!(*ctx.accounts.quote_mint.to_account_info().owner, quote_program, LaunchError::BadConfig);
    let economics =
        quote_economics(ctx.remaining_accounts, &cfg, &quote_mint_key, &quote_program, ctx.accounts.quote_mint.decimals, params.team_buy_tokens != 0)?;











    let chain = if quote_mint_key == WSOL_MINT || economics.walk.is_empty() {
        super::quote_chain::Chain { root: quote_mint_key, root_decimals: ctx.accounts.quote_mint.decimals, ticks: 0, hops: 0 }
    } else if economics.walk.len() == 1 {
        let now = Clock::get()?.unix_timestamp;
        super::quote_walk::read_finished(&ctx.remaining_accounts[economics.walk.start], &ctx.accounts.creator.key(), &quote_mint_key, now)?
    } else {
        let now = u32::try_from(Clock::get()?.unix_timestamp).map_err(|_| LaunchError::MathOverflow)?;
        super::quote_chain::walk(&ctx.remaining_accounts[economics.walk.clone()], &quote_mint_key, now)?
    };



    let preset = load_preset(&ctx.accounts.preset_registry, params.mode)?;
    let resolved = sale::resolve_config_chained(
        &params,
        &name,
        &symbol,
        &uri,
        cfg.platform_bps,
        economics.min_bid,
        chain.root_decimals,
        chain.ticks,
        preset.as_ref(),
    )
    .map_err(LaunchError::from)?;

    let deployer = sale::resolve_team_buy(&params, &resolved).map_err(LaunchError::from)?;


    let team_payees = match &deployer {
        Some(_) => sale::resolve_team_payees(&params, &ctx.accounts.creator.key()).map_err(LaunchError::from)?,
        None => Vec::new(),
    };








    let creator_key = ctx.accounts.creator.key();
    let by_governance = {
        let c = ctx.accounts.launch_config.load()?;
        creator_key == c.governance
    };
    let upline = if by_governance {
        sale::Upline::default()
    } else {
        let link = ctx.accounts.creator_trader_link.to_account_info();
        let data = link.try_borrow_data()?;
        sale::creator_upline(&params, &ctx.accounts.creator.key(), link.owner, &data)
    };


    let hooked = resolved.hooked();





    let slots = holders_slots(ctx.accounts)?;
    let upline = if !hooked && !slots { sale::Upline::default() } else { upline };
    let referred = upline.is_set();
    holders_accounts_match(ctx.accounts, hooked, slots)?;




    require!(mint_key.to_bytes() > quote_mint_key.to_bytes(), LaunchError::MintBelowQuote);




    let amm_pool = predict_amm_pool(&quote_mint_key, &mint_key);
    let (floor_pool, _) =
        Pubkey::find_program_address(&[floor_types::seeds::FLOOR_POOL, amm_pool.as_ref()], &FLOOR_HOOK_PROGRAM);
    require_keys_eq!(ctx.accounts.floor_pool.key(), floor_pool, LaunchError::BadConfig);

    let launch_key = ctx.accounts.launch.key();
    require_keys_eq!(
        ctx.accounts.wsol_vault.key(),
        get_associated_token_address_with_program_id(&launch_key, &quote_mint_key, &quote_program),
        LaunchError::NotAssociatedTokenAccount
    );




    let metadata_authority = if uri.is_empty() { Some(ctx.accounts.mint_authority.key()) } else { None };
    build_mint(&ctx, nonce, name.clone(), symbol.clone(), uri.clone(), hooked)?;




    associated_token::create_idempotent(CpiContext::new(
        ctx.accounts.ata_program.key(),
        associated_token::Create {
            payer: ctx.accounts.creator.to_account_info(),
            associated_token: ctx.accounts.wsol_vault.to_account_info(),
            authority: ctx.accounts.launch.to_account_info(),
            mint: ctx.accounts.quote_mint.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.quote_token_program.to_account_info(),
        },
    ))?;





    write_launch(&ctx, &params, &resolved, &cfg, nonce, amm_pool, floor_pool)?;

    if let Some(d) = &deployer {
        team_buy(&ctx, d, resolved.price_x64, cfg.team_sale_bps, economics.team_source)?;
    }


    let owner_kind = if deployer.is_some() { floor_types::constants::STAIR_OWNER_TEAM_BUY } else { 0 };


    let payee_slots = sale::register_slots(&params, team_payees);
    register_pool(&ctx, &params, &upline, &resolved, &cfg, &economics, amm_pool, owner_kind, &payee_slots)?;


    if hooked || referred {
        init_holders(&ctx, hooked)?;
    }


    write_registry(&ctx, &cfg, amm_pool, resolved.mode)?;


    mint_profile::verify(
        &ctx.accounts.mint.to_account_info(),
        &ctx.accounts.token_vault.to_account_info(),
        &ctx.accounts.lock_authority.key(),
        &uri,
        metadata_authority,
        hooked,
    )?;





    emit_cpi!(GraduationRentReserved {
        launch: launch_key,
        reserve: ctx.accounts.graduation_rent.key(),
        lamports: graduation_rent::graduation_rent_lamports(
            &Rent::get()?,
            graduation_rent::quote_vault_len(&ctx.accounts.quote_mint.to_account_info())?,
            graduation_rent::base_vault_len(&ctx.accounts.mint.to_account_info())?,
            graduation_rent::quote_pools(&ctx.accounts.quote_mint.key()),
            resolved.stair_steps != 0,
        ),
    });
    emit_cpi!(LaunchCreated {
        launch: launch_key,
        mint: mint_key,
        creator: ctx.accounts.creator.key(),
        amm_pool,
        floor_pool,
        registry_index: cfg.registry_index,
        mode: resolved.mode,
        price_x64: resolved.price_x64,
        open_tick: resolved.open_tick,
        name,
        symbol,
        uri,






        quote_mint: quote_mint_key,
        quote_decimals: ctx.accounts.quote_mint.decimals,
        quote_usd_cents: params.quote_usd_cents,
        graduation_mcap_usd: resolved.mcap_usd,
    });
    if chain.hops > 0 {
        emit_cpi!(QuoteChainPriced {
            launch: launch_key,
            root_quote: chain.root,
            root_decimals: chain.root_decimals,
            hops: chain.hops,
            chain_ticks: chain.ticks,
        });
    }




    emit_cpi!(PayeeSlotsSet {
        launch: launch_key,
        creator: payee_slots.creator.iter().map(Into::into).collect(),
        team: payee_slots.team.iter().map(Into::into).collect(),
    });


    if economics.walk.len() == 1 && quote_mint_key != WSOL_MINT {
        super::quote_walk::close_into(&ctx.remaining_accounts[economics.walk.start], &ctx.accounts.creator.to_account_info())?;
    }
    Ok(())
}

#[inline(never)]
fn read_config(loader: &AccountLoader<'_, LaunchConfig>) -> Result<ConfigView> {
    let c = loader.load()?;
    require_keys_eq!(c.hook_program, FLOOR_HOOK_PROGRAM, LaunchError::BadConfig);
    require_keys_eq!(c.amm_program, CARPENTER_AMM_PROGRAM, LaunchError::BadConfig);
    Ok(ConfigView {
        platform_bps: c.platform_bps,
        min_bid_lamports: c.min_bid_lamports,
        min_sellable: c.min_sellable,
        registry_index: c.registry_count,
        bounty_bps: c.bounty_bps,
        max_bounty_lamports: c.max_bounty_lamports,

        team_sale_bps: c.team_sale_bps(),
        team_pool_bps: c.team_pool_bps(),
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
#[inline(never)]
fn resolve_uri(loader: &AccountLoader<'_, LaunchConfig>, mint: &Pubkey) -> Result<String> {
    let c = loader.load()?;
    let len = c.metadata_base_len as usize;
    if len == 0 {
        return Ok(String::new());
    }
    let base = core::str::from_utf8(&c.metadata_base[..len]).map_err(|_| LaunchError::MetadataBaseInvalid)?;
    Ok(metadata_uri(base, &mint.to_string()))
}

///
///
///
///
#[inline(never)]
///
fn holders_slots(a: &CreateLaunch) -> Result<bool> {
    let common = [
        a.floor_holders.is_some(),
        a.claims_registry.is_some(),
        a.claims_table.is_some(),
        a.claims_alt_authority.is_some(),
        a.alt_program.is_some(),
        a.holders_event_authority.is_some(),
    ];
    require!(common.iter().all(|s| *s == common[0]), LaunchError::BadConfig);
    Ok(common[0])
}

///
///
fn holders_accounts_match(a: &CreateLaunch, hooked: bool, slots: bool) -> Result<()> {
    if hooked {
        require!(slots, LaunchError::BadConfig);
    }
    require!(a.holder_book.is_some() == hooked && a.extra_account_metas.is_some() == hooked, LaunchError::BadConfig);
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
#[inline(never)]
fn init_holders<'info>(ctx: &Context<'info, CreateLaunch<'info>>, hooked: bool) -> Result<()> {
    let a = &ctx.accounts;
    let need = |o: &Option<UncheckedAccount<'info>>| -> Result<AccountInfo<'info>> {
        o.as_ref().map(|u| u.to_account_info()).ok_or_else(|| error!(LaunchError::BadConfig))
    };
    let floor_holders = need(&a.floor_holders)?;
    let claims_registry = need(&a.claims_registry)?;
    let claims_table = need(&a.claims_table)?;
    let claims_alt_authority = need(&a.claims_alt_authority)?;
    let alt_program = need(&a.alt_program)?;
    let holders_event_authority = need(&a.holders_event_authority)?;

    let (book, metas) = if hooked {
        (need(&a.holder_book)?, need(&a.extra_account_metas)?)
    } else {
        (floor_holders.clone(), floor_holders.clone())
    };
    let opt_meta = |info: &AccountInfo<'info>| {
        if hooked {
            AccountMeta::new(info.key(), false)
        } else {
            AccountMeta::new_readonly(FLOOR_HOLDERS_PROGRAM, false)
        }
    };

    let mut data = holders_disc::INIT_LAUNCH.to_vec();
    data.push(hooked as u8);
    let metas_list = vec![
        AccountMeta::new_readonly(a.lock_authority.key(), true),
        AccountMeta::new(a.creator.key(), true),
        AccountMeta::new_readonly(a.mint.key(), false),
        AccountMeta::new_readonly(a.quote_mint.key(), false),
        AccountMeta::new_readonly(a.floor_pool.key(), false),
        opt_meta(&book),
        opt_meta(&metas),
        AccountMeta::new(claims_registry.key(), false),
        AccountMeta::new(claims_table.key(), false),
        AccountMeta::new_readonly(claims_alt_authority.key(), false),
        AccountMeta::new_readonly(alt_program.key(), false),
        AccountMeta::new_readonly(a.system_program.key(), false),
        AccountMeta::new_readonly(holders_event_authority.key(), false),
        AccountMeta::new_readonly(FLOOR_HOLDERS_PROGRAM, false),
    ];
    let infos = vec![
        a.lock_authority.to_account_info(),
        a.creator.to_account_info(),
        a.mint.to_account_info(),
        a.quote_mint.to_account_info(),
        a.floor_pool.to_account_info(),
        book,
        metas,
        claims_registry,
        claims_table,
        claims_alt_authority,
        alt_program,
        a.system_program.to_account_info(),
        holders_event_authority,
        floor_holders,
    ];
    let ix = Instruction { program_id: FLOOR_HOLDERS_PROGRAM, accounts: metas_list, data };
    let token_2022 = a.token_2022.to_account_info();
    let mint = a.mint.to_account_info();
    let lock_authority = a.lock_authority.to_account_info();
    crate::lock::invoke_holders_init(
        &Lock { token_2022: &token_2022, mint: &mint, lock_authority: &lock_authority, bump: ctx.bumps.lock_authority },
        &ix,
        &infos,
    )
}

#[inline(never)]
fn build_mint(ctx: &Context<CreateLaunch>, nonce: u64, name: String, symbol: String, uri: String, hooked: bool) -> Result<()> {
    let a = &ctx.accounts;
    let creator = a.creator.to_account_info();
    let mint = a.mint.to_account_info();
    let mint_authority = a.mint_authority.to_account_info();
    let lock_authority = a.lock_authority.to_account_info();
    let launch = a.launch.to_account_info();
    let token_vault = a.token_vault.to_account_info();
    let token_2022 = a.token_2022.to_account_info();
    let ata_program = a.ata_program.to_account_info();
    let system_program = a.system_program.to_account_info();
    let ma_bump = [ctx.bumps.mint_authority];
    let ma_seeds: &[&[u8]] = &[MINT_AUTHORITY_SEED, &ma_bump];
    let profile = MintProfile {
        creator: &creator,
        mint: &mint,
        mint_bump: ctx.bumps.mint,
        nonce,
        mint_authority: &mint_authority,
        mint_authority_seeds: ma_seeds,
        lock: Lock { token_2022: &token_2022, mint: &mint, lock_authority: &lock_authority, bump: ctx.bumps.lock_authority },
        launch: &launch,
        token_vault: &token_vault,
        token_2022: &token_2022,
        ata_program: &ata_program,
        system_program: &system_program,
        hooked,
    };
    mint_profile::build(&profile, name, symbol, uri)
}

#[inline(never)]
fn write_launch(
    ctx: &Context<CreateLaunch>,
    params: &CreateLaunchParams,
    r: &ResolvedConfig,
    cfg: &ConfigView,
    nonce: u64,
    amm_pool: Pubkey,
    floor_pool: Pubkey,
) -> Result<()> {
    let a = &ctx.accounts;
    let creator = a.creator.to_account_info();
    let launch = a.launch.to_account_info();
    let sale_vault = a.sale_vault.to_account_info();
    let mint_key = a.mint.key();
    let launch_key = a.launch.key();

    let launch_bump = [ctx.bumps.launch];
    create_pda(&creator, &launch, Launch::LEN, &crate::ID, &[LAUNCH_SEED, mint_key.as_ref(), &launch_bump])?;

    let sv_bump = [ctx.bumps.sale_vault];
    create_pda(&creator, &sale_vault, 0, &crate::ID, &[SALE_VAULT_SEED, launch_key.as_ref(), &sv_bump])?;








    let grad_rent_info = ctx.accounts.graduation_rent.to_account_info();
    let gr_bump = [ctx.bumps.graduation_rent];


    let vault0_len = graduation_rent::quote_vault_len(&ctx.accounts.quote_mint.to_account_info())?;


    let vault1_len = graduation_rent::base_vault_len(&ctx.accounts.mint.to_account_info())?;
    let quote_pools = graduation_rent::quote_pools(&ctx.accounts.quote_mint.key());



    let stair_budget = graduation_rent::stair_rent_budget_lamports(&Rent::get()?, r.stair_steps);
    let stair_budget_u32 = u32::try_from(stair_budget).map_err(|_| error!(LaunchError::MathOverflow))?;
    graduation_rent::fund(
        &creator,
        &grad_rent_info,
        &[GRAD_RENT_SEED, launch_key.as_ref(), &gr_bump],
        vault0_len,
        vault1_len,
        quote_pools,


        r.stair_steps != 0,
        stair_budget,
    )?;

    let mut data = launch.try_borrow_mut_data()?;
    data[..8].copy_from_slice(Launch::DISCRIMINATOR);
    let l: &mut Launch = bytemuck::from_bytes_mut(&mut data[8..Launch::LEN]);


    l.version = 3;
    l.stair_rent_budget = stair_budget_u32;
    l.bump = ctx.bumps.launch;
    l.sale_vault_bump = ctx.bumps.sale_vault;
    l.lock_bump = ctx.bumps.lock_authority;
    l.state = LAUNCH_STATE_BONDING;
    l.mode = r.mode;
    l.mint_authority_bump = ctx.bumps.mint_authority;


    l.pad0 = 0;
    l.mint = mint_key;
    l.creator = a.creator.key();
    l.pad3 = [0u8; 28];
    l.pad4 = [0u8; 32];
    l.team_cliffs_paid_by_slot = 0;
    l.pad5 = 0;
    l.pad6 = 0;
    l.pad7 = [0u8; 32];
    l.amm_pool = amm_pool;
    l.floor_pool = floor_pool;
    l.token_vault = a.token_vault.key();
    l.wsol_vault = a.wsol_vault.key();
    l.price_x64 = U128::new(r.price_x64);
    l.quote_usd_cents = params.quote_usd_cents;
    l.mcap_usd = r.mcap_usd;
    l.created_slot = Clock::get()?.slot;
    l.nonce = nonce;



    l.sale_tokens = r.sale_tokens;
    l.pol_tokens = r.pol_tokens;
    l.ladder_tokens = r.stair_tokens;
    l.sold = 0;
    l.raised = 0;
    l.buyers = 0;
    l.open_tick = r.open_tick;


    l.trader_ref_sale_bump = Pubkey::find_program_address(&[TRADER_REF_SALE_SEED, launch_key.as_ref()], &crate::ID).1;
    l.skim_buy_far = r.skim_buy_far_bps;
    l.skim_sell_far = r.skim_sell_far_bps;




    l.creator_buy = r.creator_buy_bps;
    l.creator_sell = r.creator_sell_bps;



    l.holder_buy = r.holder_buy_bps;
    l.holder_sell = r.holder_sell_bps;
    l.registry_index = cfg.registry_index;
    l.graduated_slot = 0;
    l.quote_mint = a.quote_mint.key();



    l.team_bps = cfg.team_sale_bps;
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
#[inline(never)]
fn team_buy<'info>(ctx: &Context<'info, CreateLaunch<'info>>, d: &sale::TeamBuy, price_x64: u128, team_bps: u16, source_index: usize) -> Result<()> {
    let a = &ctx.accounts;
    let cost = sale::team_buy_cost(d.tokens, price_x64).ok_or(LaunchError::MathOverflow)?;
    let fee = floor_math::sale::team_fee_of(cost, team_bps).map_err(|_| error!(LaunchError::MathOverflow))?;
    let spent = cost.checked_add(fee).ok_or(LaunchError::MathOverflow)?;
    let creator = a.creator.to_account_info();
    if a.quote_mint.key() == WSOL_MINT {
        system_program::transfer(
            CpiContext::new(
                system_program::ID,
                system_program::Transfer { from: creator.clone(), to: a.sale_vault.to_account_info() },
            ),
            spent,
        )?;
    } else {

        let source = ctx.remaining_accounts.get(source_index).ok_or(LaunchError::NotAssociatedTokenAccount)?;
        anchor_spl::token_interface::transfer_checked(
            CpiContext::new(
                a.quote_token_program.key(),
                anchor_spl::token_interface::TransferChecked {
                    from: source.clone(),
                    mint: a.quote_mint.to_account_info(),
                    to: a.wsol_vault.to_account_info(),
                    authority: creator.clone(),
                },
            ),
            spent,
            a.quote_mint.decimals,
        )?;
    }

    let tail = graduation_rent::owner_tail_rent_lamports(&Rent::get()?);
    system_program::transfer(
        CpiContext::new(
            system_program::ID,
            system_program::Transfer { from: creator, to: a.graduation_rent.to_account_info() },
        ),
        tail,
    )?;
    {
        let mut data = a.launch.try_borrow_mut_data()?;
        let l: &mut Launch = bytemuck::from_bytes_mut(&mut data[8..Launch::LEN]);
        l.sold = l.sold.checked_add(d.tokens).ok_or(LaunchError::MathOverflow)?;
        l.raised = l.raised.checked_add(cost).ok_or(LaunchError::MathOverflow)?;
        l.team_owed = l.team_owed.checked_add(fee).ok_or(LaunchError::MathOverflow)?;
        require!(l.sold < l.sale_tokens, LaunchError::TeamBuyOutOfRange);
        l.team_step_tokens = d.step_tokens;
        l.team_wallet_tokens = d.wallet_tokens;
        l.team_lock_days = d.lock_days;
        l.team_vest_days = d.vest_days;


    }
    emit_cpi!(TeamBought {
        launch: a.launch.key(),
        creator: a.creator.key(),
        tokens: d.tokens,
        step_tokens: d.step_tokens,
        wallet_tokens: d.wallet_tokens,
        cost,
        fee,
        lock_days: d.lock_days,
        vest_days: d.vest_days,
    });
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
#[inline(never)]
fn register_pool<'info>(
    ctx: &Context<'info, CreateLaunch<'info>>,
    _params: &CreateLaunchParams,
    upline: &sale::Upline,
    r: &ResolvedConfig,
    cfg: &ConfigView,
    economics: &QuoteEconomics<'info>,
    amm_pool: Pubkey,
    owner_kind: u8,
    slots: &floor_types::slots::SlotsArg,
) -> Result<()> {
    let a = &ctx.accounts;
    let args = RegisterPoolArgs {
        amm_pool: amm_pool.to_bytes(),
        base_mint: a.mint.key().to_bytes(),

        quote_mint: a.quote_mint.key().to_bytes(),
        pool_initializer: a.launch.key().to_bytes(),
        mode: r.mode,
        skim_buy_far_bps: r.skim_buy_far_bps,
        skim_sell_far_bps: r.skim_sell_far_bps,



        creator_buy_bps: r.creator_buy_bps,
        creator_sell_bps: r.creator_sell_bps,






        holder_buy_bps: r.holder_buy_bps,
        holder_sell_bps: r.holder_sell_bps,



        deployer: a.creator.key().to_bytes(),
        pad: [0u8; 32],


        min_bid: economics.min_bid,
        min_sellable: cfg.min_sellable,


        platform_bps: cfg.platform_bps,



        max_bounty: economics.max_bounty,
        bounty_bps: cfg.bounty_bps,









        referrer: upline.tier1.to_bytes(),



    };
    let mut data = floor_types::disc::hook::REGISTER_POOL.to_vec();
    data.extend_from_slice(&borsh::to_vec(&args).map_err(|_| LaunchError::BadConfig)?);





    data.extend_from_slice(&upline.tier2.to_bytes());
    data.extend_from_slice(&upline.tier3.to_bytes());





    data.extend_from_slice(&cfg.team_pool_bps.to_le_bytes());





    data.extend_from_slice(&r.skim_sell_at_floor_multiple_bps.to_le_bytes());





    data.push(r.stair_steps);
    data.extend_from_slice(&r.stair_growth_bps.to_le_bytes());



    data.push(owner_kind);




    let slots_at = data.len();
    data.extend_from_slice(&borsh::to_vec(slots).map_err(|_| LaunchError::BadConfig)?);
    debug_assert_eq!(
        slots_at,
        8 + floor_types::register::REGISTER_POOL_ARGS_LEN
            + floor_types::register::REFERRAL_TIERS_ARG_LEN
            + floor_types::register::TEAM_FEE_ARG_LEN
            + floor_types::register::SKIM_SELL_MULTIPLE_ARG_LEN
            + floor_types::register::STAIR_ARG_LEN
            + floor_types::owners::STAIR_OWNER_KIND_ARG_LEN
    );

    let mut metas = vec![
        AccountMeta::new_readonly(a.launch_authority.key(), true),
        AccountMeta::new(a.creator.key(), true),
        AccountMeta::new_readonly(a.hook_config.key(), false),
        AccountMeta::new(a.floor_pool.key(), false),
        AccountMeta::new_readonly(a.carpenter_amm.key(), false),
        AccountMeta::new_readonly(a.amm_config.key(), false),
        AccountMeta::new_readonly(a.approved_hook.key(), false),
        AccountMeta::new_readonly(a.mint.key(), false),
        AccountMeta::new_readonly(a.mint_pools.key(), false),
        AccountMeta::new(a.mint_exclusivity.key(), false),
        AccountMeta::new_readonly(a.hook_authority.key(), false),
        AccountMeta::new_readonly(a.system_program.key(), false),
        AccountMeta::new_readonly(a.amm_event_authority.key(), false),
        AccountMeta::new_readonly(a.hook_event_authority.key(), false),
        AccountMeta::new_readonly(FLOOR_HOOK_PROGRAM, false),
    ];
    let mut infos = vec![
        a.launch_authority.to_account_info(),
        a.creator.to_account_info(),
        a.hook_config.to_account_info(),
        a.floor_pool.to_account_info(),
        a.carpenter_amm.to_account_info(),
        a.amm_config.to_account_info(),
        a.approved_hook.to_account_info(),
        a.mint.to_account_info(),
        a.mint_pools.to_account_info(),
        a.mint_exclusivity.to_account_info(),
        a.hook_authority.to_account_info(),
        a.system_program.to_account_info(),
        a.amm_event_authority.to_account_info(),
        a.hook_event_authority.to_account_info(),
        a.floor_hook.to_account_info(),
    ];





    for acct in &economics.hook_accounts {
        metas.push(if acct.is_writable { AccountMeta::new(acct.key(), false) } else { AccountMeta::new_readonly(acct.key(), false) });
        infos.push(acct.clone());
    }
    let ix = Instruction { program_id: FLOOR_HOOK_PROGRAM, accounts: metas, data };
    let bump = [ctx.bumps.launch_authority];
    invoke_signed(&ix, &infos, &[&[LAUNCH_AUTHORITY_SEED, &bump]])?;




    Ok(())
}



#[inline(never)]
fn write_registry(ctx: &Context<CreateLaunch>, cfg: &ConfigView, amm_pool: Pubkey, mode: u8) -> Result<()> {
    let a = &ctx.accounts;
    {


        let mut e = a.registry_entry.load_init()?;
        e.index = cfg.registry_index;
        e.launch = a.launch.key();
        e.mint = a.mint.key();
        e.amm_pool = amm_pool;
        e.mode = mode;
        e.created_slot = Clock::get()?.slot;
    }
    let mut c = a.launch_config.load_mut()?;


    require_eq!(c.registry_count, cfg.registry_index, LaunchError::RegistryIndexRace);
    c.registry_count = cfg.registry_index.checked_add(1).ok_or(LaunchError::MathOverflow)?;
    Ok(())
}
