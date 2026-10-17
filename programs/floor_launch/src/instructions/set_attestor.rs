//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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
use floor_types::layout::Layout;

use crate::constants::{
    AMM_CONFIG_SEED, CARPENTER_AMM_PROGRAM, FLOOR_HOOK_PROGRAM, LAUNCH_CONFIG_SEED, PLATFORM_TREASURY_SEED, TEAM_TREASURY_SEED,
};
use crate::errors::LaunchError;
use crate::events::AttestorChanged;
use crate::state::LaunchConfig;

#[event_cpi]
#[derive(Accounts)]
pub struct SetAttestor<'info> {
    pub governance: Signer<'info>,
    #[account(mut, seeds = [LAUNCH_CONFIG_SEED], bump)]
    pub launch_config: AccountLoader<'info, LaunchConfig>,
///
///
    #[account(seeds = [PLATFORM_TREASURY_SEED], bump, seeds::program = FLOOR_HOOK_PROGRAM)]
    pub platform_treasury: UncheckedAccount<'info>,
///
///
///
    #[account(seeds = [TEAM_TREASURY_SEED], bump, seeds::program = FLOOR_HOOK_PROGRAM)]
    pub team_treasury: UncheckedAccount<'info>,
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
pub fn treasury_gov(info: &AccountInfo) -> Result<Option<Pubkey>> {
    if *info.owner == anchor_lang::system_program::ID && info.data_is_empty() {
        return Ok(None);
    }
    require_keys_eq!(*info.owner, FLOOR_HOOK_PROGRAM, LaunchError::BadConfig);
    type T = floor_types::layout::PlatformTreasury;
    const OFF: usize = 8 + core::mem::offset_of!(floor_types::layout::PlatformTreasury, governance);
    let data = info.try_borrow_data()?;
    require!(data.len() >= 8 + <T as Layout>::SIZE, LaunchError::BadConfig);
    require!(data[..8] == floor_types::disc::account::PLATFORM_TREASURY, LaunchError::BadConfig);
    Ok(Some(Pubkey::new_from_array(data[OFF..OFF + 32].try_into().unwrap())))
}

///
///
///
///
pub fn amm_guardian(info: &AccountInfo) -> Result<Option<Pubkey>> {
    if *info.owner == anchor_lang::system_program::ID && info.data_is_empty() {
        return Ok(None);
    }
    require_keys_eq!(*info.owner, CARPENTER_AMM_PROGRAM, LaunchError::BadConfig);
    type T = floor_types::layout::AmmConfig;
    const OFF: usize = 8 + core::mem::offset_of!(floor_types::layout::AmmConfig, pause_guardian);
    let data = info.try_borrow_data()?;
    require!(data.len() >= 8 + <T as Layout>::SIZE, LaunchError::BadConfig);
    require!(data[..8] == floor_types::disc::account::AMM_CONFIG, LaunchError::BadConfig);
    let g = Pubkey::new_from_array(data[OFF..OFF + 32].try_into().unwrap());
    Ok((g != Pubkey::default()).then_some(g))
}

pub fn handler(ctx: Context<SetAttestor>, new_attestor: Pubkey) -> Result<()> {
    let previous = {
        let mut c = ctx.accounts.launch_config.load_mut()?;
        require_keys_eq!(ctx.accounts.governance.key(), c.governance, LaunchError::Unauthorized);
        require_keys_neq!(new_attestor, Pubkey::default(), LaunchError::BadConfig);
        require_keys_neq!(new_attestor, c.governance, LaunchError::AttestorIsGovernance);
        if let Some(g) = treasury_gov(&ctx.accounts.platform_treasury.to_account_info())? {
            require_keys_neq!(new_attestor, g, LaunchError::AttestorIsTreasuryGovernance);
        }

        if let Some(t) = treasury_gov(&ctx.accounts.team_treasury.to_account_info())? {
            require_keys_neq!(new_attestor, t, LaunchError::AttestorIsTeamTreasuryGovernance);
        }

        if let Some(g) = amm_guardian(&ctx.accounts.amm_config.to_account_info())? {
            require_keys_neq!(new_attestor, g, LaunchError::AttestorIsPauseGuardian);
        }
        let previous = c.attestor;
        c.attestor = new_attestor;
        previous
    };
    emit_cpi!(AttestorChanged {
        launch_config: ctx.accounts.launch_config.key(),
        previous,
        attestor: new_attestor,
    });
    Ok(())
}
