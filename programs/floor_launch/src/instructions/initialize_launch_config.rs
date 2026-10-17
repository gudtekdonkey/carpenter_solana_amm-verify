//!
//!
//!
//!

use anchor_lang::prelude::*;
use anchor_lang::solana_program::bpf_loader_upgradeable;

use floor_types::constants::{
    BOUNTY_BPS, CLUSTER_LOCALNET, FLOOR_TICK_SPACING, MAX_BOUNTY_LAMPORTS,
    MAX_METADATA_BASE_LEN, MIN_BID_LAMPORTS_CEIL, MIN_BID_LAMPORTS_FLOOR, PLATFORM_BPS,
};

use crate::constants::{
    AMM_CONFIG_SEED, CARPENTER_AMM_PROGRAM, FLOOR_HOOK_PROGRAM, LAUNCH_CONFIG_SEED, PLATFORM_TREASURY_SEED, TEAM_TREASURY_SEED,
};
use crate::errors::LaunchError;
use crate::events::LaunchConfigInitialized;
use crate::instructions::set_attestor::{amm_guardian, treasury_gov};
use crate::state::LaunchConfig;

#[event_cpi]
#[derive(Accounts)]
pub struct InitializeLaunchConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, payer = authority, space = LaunchConfig::LEN, seeds = [LAUNCH_CONFIG_SEED], bump)]
    pub launch_config: AccountLoader<'info, LaunchConfig>,
    #[account(
        seeds = [crate::ID.as_ref()],
        bump,
        seeds::program = bpf_loader_upgradeable::ID,
        constraint = program_data.upgrade_authority_address == Some(authority.key()) @ LaunchError::Unauthorized,
    )]
    pub program_data: Box<Account<'info, ProgramData>>,
///
///
///
    #[account(seeds = [PLATFORM_TREASURY_SEED], bump, seeds::program = FLOOR_HOOK_PROGRAM)]
    pub platform_treasury: UncheckedAccount<'info>,
///
///
///
    #[account(seeds = [TEAM_TREASURY_SEED], bump, seeds::program = FLOOR_HOOK_PROGRAM)]
    pub team_treasury: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
///
///
    #[account(seeds = [AMM_CONFIG_SEED], bump, seeds::program = CARPENTER_AMM_PROGRAM)]
    pub amm_config: UncheckedAccount<'info>,
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
///
///
///
#[allow(clippy::too_many_arguments)]
pub fn handler(
    ctx: Context<InitializeLaunchConfig>,
    governance: Pubkey,
    attestor: Pubkey,
    cluster: u8,
    hook_program: Pubkey,
    amm_program: Pubkey,
    min_bid_lamports: u64,
    min_sellable: u64,
) -> Result<()> {
    require!(cluster <= CLUSTER_LOCALNET, LaunchError::BadConfig);
    require_keys_eq!(hook_program, FLOOR_HOOK_PROGRAM, LaunchError::BadConfig);
    require_keys_eq!(amm_program, CARPENTER_AMM_PROGRAM, LaunchError::BadConfig);
    require!(min_sellable > 0, LaunchError::BadConfig);
    require!(
        (MIN_BID_LAMPORTS_FLOOR..=MIN_BID_LAMPORTS_CEIL).contains(&min_bid_lamports),
        LaunchError::MinBidOutOfRange
    );
    require_keys_neq!(governance, Pubkey::default(), LaunchError::BadConfig);
    require_keys_neq!(attestor, Pubkey::default(), LaunchError::BadConfig);
    require_keys_neq!(governance, attestor, LaunchError::GovernanceIsAttestor);

    if let Some(g) = treasury_gov(&ctx.accounts.platform_treasury.to_account_info())? {
        require_keys_neq!(attestor, g, LaunchError::AttestorIsTreasuryGovernance);
    }

    if let Some(t) = treasury_gov(&ctx.accounts.team_treasury.to_account_info())? {
        require_keys_neq!(attestor, t, LaunchError::AttestorIsTeamTreasuryGovernance);
    }

    if let Some(g) = amm_guardian(&ctx.accounts.amm_config.to_account_info())? {
        require_keys_neq!(attestor, g, LaunchError::AttestorIsPauseGuardian);
    }

    let mut c = ctx.accounts.launch_config.load_init()?;
    c.version = 1;
    c.bump = ctx.bumps.launch_config;
    c.cluster = cluster;
    c.governance = governance;
    c.attestor = attestor;
    c.hook_program = hook_program;
    c.amm_program = amm_program;
    c.platform_bps = PLATFORM_BPS;
    c.tick_spacing = FLOOR_TICK_SPACING;
    c.min_bid_lamports = min_bid_lamports;
    c.min_sellable = min_sellable;
    c.registry_count = 0;
    c.bounty_bps = BOUNTY_BPS;
    c.max_bounty_lamports = MAX_BOUNTY_LAMPORTS;



    c.metadata_base_len = 0;
    c.metadata_base = [0u8; MAX_METADATA_BASE_LEN];





    c.metadata_base_set_slot = 0;
    c.pending_governance = Pubkey::default();
    drop(c);

    emit_cpi!(LaunchConfigInitialized {
        launch_config: ctx.accounts.launch_config.key(),
        governance,
        attestor,
        cluster,
    });
    Ok(())
}
