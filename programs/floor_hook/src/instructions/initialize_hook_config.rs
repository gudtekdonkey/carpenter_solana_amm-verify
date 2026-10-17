//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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
use floor_types::constants::{FLOOR_CALLBACK_ACCOUNTS, FLOOR_HOOK_FLAGS, FLOOR_MAX_ROUNDS};

use crate::constants::{HOOK_AUTHORITY_SEED, HOOK_CONFIG_SEED, PLATFORM_TREASURY_SEED, WSOL_MINT};
use crate::errors::HookError;
use crate::events::{HookConfigInitialized, TreasuryGovernanceChanged};
use crate::instructions::platform_treasury::{attestor_at, open_treasury, point_config_at_treasury, refuse_as_governance};
use crate::state::{HookConfig, PlatformTreasury};

#[event_cpi]
#[derive(Accounts)]
pub struct InitializeHookConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, payer = authority, space = HookConfig::LEN, seeds = [HOOK_CONFIG_SEED], bump)]
    pub hook_config: AccountLoader<'info, HookConfig>,
    #[account(
        seeds = [crate::ID.as_ref()],
        bump,
        seeds::program = bpf_loader_upgradeable::ID,
        constraint = program_data.upgrade_authority_address == Some(authority.key()) @ HookError::Unauthorized,
    )]
    pub program_data: Box<Account<'info, ProgramData>>,
///
///
///
///
    #[account(init, payer = authority, space = PlatformTreasury::LEN, seeds = [PLATFORM_TREASURY_SEED], bump)]
    pub platform_treasury: AccountLoader<'info, PlatformTreasury>,
///
///
///
    pub launch_config: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

///
pub fn wsol_ata(owner: &Pubkey) -> Pubkey {
    let spl_token = Pubkey::new_from_array(floor_types::keys::SPL_TOKEN_PROGRAM_ID);
    let ata_program = Pubkey::new_from_array(floor_types::keys::ASSOCIATED_TOKEN_PROGRAM_ID);
    Pubkey::find_program_address(&[owner.as_ref(), spl_token.as_ref(), WSOL_MINT.as_ref()], &ata_program).0
}

pub fn handler(ctx: Context<InitializeHookConfig>, launch_program: Pubkey) -> Result<()> {
    require_keys_neq!(launch_program, Pubkey::default(), HookError::BadConfig);


    let platform_owner = ctx.accounts.platform_treasury.key();
    let governance = ctx.accounts.authority.key();
    let attestor = attestor_at(&ctx.accounts.launch_config.to_account_info(), &launch_program)?;
    refuse_as_governance(&governance, &platform_owner, attestor)?;
    {
        let mut t = ctx.accounts.platform_treasury.load_init()?;
        open_treasury(&mut *t, ctx.bumps.platform_treasury, governance);



        t.allow_all_quotes = 1;
    }

    let (launch_authority, _) =
        Pubkey::find_program_address(&[floor_types::seeds::LAUNCH_AUTHORITY], &launch_program);
    let hook_authority = Pubkey::find_program_address(&[HOOK_AUTHORITY_SEED], &crate::ID);

    {
        let mut c = ctx.accounts.hook_config.load_init()?;
        c.version = 1;
        c.bump = ctx.bumps.hook_config;
        c.max_rounds = FLOOR_MAX_ROUNDS;
        c.callback_accounts = FLOOR_CALLBACK_ACCOUNTS;
        c.flags = FLOOR_HOOK_FLAGS;
        c.authority_bump = hook_authority.1;
        c.governance = governance;
        c.launch_program = launch_program;
        c.launch_authority = launch_authority;
        c.hook_authority = hook_authority.0;


        point_config_at_treasury(&mut *c, &platform_owner);


    }

    let platform_destination = wsol_ata(&platform_owner);
    emit_cpi!(HookConfigInitialized {
        launch_program,
        launch_authority,
        hook_authority: hook_authority.0,
        platform_owner,
        platform_destination,
    });

    emit_cpi!(TreasuryGovernanceChanged {
        treasury: platform_owner,
        previous: Pubkey::default(),
        governance,
        accepted: true,
    });
    Ok(())
}

