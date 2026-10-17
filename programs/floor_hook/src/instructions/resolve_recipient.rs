//!
//!
//!
//!
//!
//!
//!
//!

use anchor_lang::prelude::*;

use crate::constants::HOOK_CONFIG_SEED;
use crate::errors::HookError;
use crate::events::RecipientResolved;
use crate::state::{slots, FloorPool, HookConfig};

#[event_cpi]
#[derive(Accounts)]
pub struct ResolveRecipient<'info> {
    #[account(constraint = launch_authority.key() == hook_config.load()?.launch_authority @ HookError::NotLauncher)]
    pub launch_authority: Signer<'info>,
    #[account(seeds = [HOOK_CONFIG_SEED], bump)]
    pub hook_config: AccountLoader<'info, HookConfig>,
    #[account(mut)]
    pub floor_pool: AccountLoader<'info, FloorPool>,
}

pub fn handler(ctx: Context<ResolveRecipient>, slot: u8, recipient: Pubkey) -> Result<()> {



    require_keys_neq!(recipient, Pubkey::default(), HookError::BadConfig);
    let fp = ctx.accounts.floor_pool.to_account_info();
    let mut t = slots::read(&fp)?;
    let pending_recipient_id =
        slots::resolve_slot(&mut t, slots::SLOT_KIND_CREATOR, slot, recipient, HookError::AlreadyResolved)?;
    slots::write(&fp, &t)?;
    let pool = ctx.accounts.floor_pool.load()?.amm_pool;
    emit_cpi!(RecipientResolved { pool, slot, pending_recipient_id, recipient });
    Ok(())
}
