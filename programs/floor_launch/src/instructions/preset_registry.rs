//!
//!
//!
//!
//!
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

use crate::constants::{LAUNCH_CONFIG_SEED, PRESET_REGISTRY_SEED};
use crate::errors::LaunchError;
use crate::events::{PresetAdded, PresetAuthorityTransferStarted, PresetAuthorityTransferred, PresetEnabled};
use crate::state::{LaunchConfig, PresetAppendFault, PresetRegistry, PresetRowArgs};

///
pub const PRESET_REGISTRY_VERSION: u8 = 1;

#[event_cpi]
#[derive(Accounts)]
pub struct InitPresetRegistry<'info> {
///
    pub governance: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(seeds = [LAUNCH_CONFIG_SEED], bump)]
    pub launch_config: AccountLoader<'info, LaunchConfig>,
    #[account(init, payer = payer, space = PresetRegistry::LEN, seeds = [PRESET_REGISTRY_SEED], bump)]
    pub preset_registry: AccountLoader<'info, PresetRegistry>,
    pub system_program: Program<'info, System>,
}

pub fn init_preset_registry(ctx: Context<InitPresetRegistry>) -> Result<()> {
    let governance = ctx.accounts.launch_config.load()?.governance;
    require_keys_eq!(ctx.accounts.governance.key(), governance, LaunchError::Unauthorized);
    let seeded = ctx.accounts.preset_registry.load_init()?.seed(governance, ctx.bumps.preset_registry);
    for (i, row) in seeded.into_iter().enumerate() {
        emit_cpi!(PresetAdded { id: i as u8 + 1, row: row.into() });
    }
    Ok(())
}

///
#[event_cpi]
#[derive(Accounts)]
pub struct PresetAuthority<'info> {
    pub authority: Signer<'info>,
    #[account(mut, seeds = [PRESET_REGISTRY_SEED], bump)]
    pub preset_registry: AccountLoader<'info, PresetRegistry>,
}

pub fn add_preset(ctx: Context<PresetAuthority>, args: PresetRowArgs) -> Result<()> {
    let row: floor_math::presets::PresetRow = args.into();
    let id = {
        let mut r = ctx.accounts.preset_registry.load_mut()?;
        require_keys_eq!(ctx.accounts.authority.key(), r.authority, LaunchError::Unauthorized);
        match r.append(&row) {
            Ok(id) => id,
            Err(PresetAppendFault::TableFull) => return err!(LaunchError::PresetTableFull),
            Err(PresetAppendFault::Invalid(field)) => {
                msg!("PresetInvalid: {}", field.name());
                return err!(LaunchError::PresetInvalid);
            }
        }
    };
    emit_cpi!(PresetAdded { id, row: args });
    Ok(())
}

pub fn set_preset_enabled(ctx: Context<PresetAuthority>, id: u8, enabled: bool) -> Result<()> {
    {
        let mut r = ctx.accounts.preset_registry.load_mut()?;
        require_keys_eq!(ctx.accounts.authority.key(), r.authority, LaunchError::Unauthorized);
        require!(r.set_enabled(id, enabled), LaunchError::PresetUnknown);
    }
    emit_cpi!(PresetEnabled { id, enabled });
    Ok(())
}

pub fn set_preset_authority(ctx: Context<PresetAuthority>, new: Pubkey) -> Result<()> {
    let authority = {
        let mut r = ctx.accounts.preset_registry.load_mut()?;
        require_keys_eq!(ctx.accounts.authority.key(), r.authority, LaunchError::Unauthorized);
        r.pending_authority = new;
        r.authority
    };
    emit_cpi!(PresetAuthorityTransferStarted { authority, pending: new });
    Ok(())
}

#[event_cpi]
#[derive(Accounts)]
pub struct AcceptPresetAuthority<'info> {
///
    pub pending: Signer<'info>,
    #[account(mut, seeds = [PRESET_REGISTRY_SEED], bump)]
    pub preset_registry: AccountLoader<'info, PresetRegistry>,
}

pub fn accept_preset_authority(ctx: Context<AcceptPresetAuthority>) -> Result<()> {
    let (previous, authority) = {
        let mut r = ctx.accounts.preset_registry.load_mut()?;
        require!(r.pending_authority != Pubkey::default(), LaunchError::Unauthorized);
        require_keys_eq!(ctx.accounts.pending.key(), r.pending_authority, LaunchError::Unauthorized);
        let previous = r.authority;
        r.authority = r.pending_authority;
        r.pending_authority = Pubkey::default();
        (previous, r.authority)
    };
    emit_cpi!(PresetAuthorityTransferred { previous, authority });
    Ok(())
}
