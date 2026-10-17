//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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
use floor_types::constants::{MAX_METADATA_BASE_LEN, METADATA_FILL_DELAY_SLOTS};
use floor_types::metadata::normalize_metadata_base;

use crate::constants::LAUNCH_CONFIG_SEED;
use crate::errors::LaunchError;
use crate::events::MetadataBaseSet;
use crate::state::LaunchConfig;

#[event_cpi]
#[derive(Accounts)]
pub struct SetMetadataBase<'info> {
    pub governance: Signer<'info>,
    #[account(mut, seeds = [LAUNCH_CONFIG_SEED], bump)]
    pub launch_config: AccountLoader<'info, LaunchConfig>,
}

pub fn handler(ctx: Context<SetMetadataBase>, base: String) -> Result<()> {







    {
        let c = ctx.accounts.launch_config.load()?;
        require_keys_eq!(c.governance, ctx.accounts.governance.key(), LaunchError::Unauthorized);
    }
    let normalised = normalize_metadata_base(&base).ok_or(LaunchError::MetadataBaseInvalid)?;
    let slot = Clock::get()?.slot;

    {
        let mut c = ctx.accounts.launch_config.load_mut()?;


        c.metadata_base = [0u8; MAX_METADATA_BASE_LEN];
        c.metadata_base[..normalised.len()].copy_from_slice(normalised.as_bytes());
        c.metadata_base_len = normalised.len() as u8;
        c.metadata_base_set_slot = slot;
    }

    emit_cpi!(MetadataBaseSet {
        launch_config: ctx.accounts.launch_config.key(),
        base: normalised.to_string(),
        set_slot: slot,


        fillable_from_slot: slot.saturating_add(METADATA_FILL_DELAY_SLOTS),
    });
    Ok(())
}
