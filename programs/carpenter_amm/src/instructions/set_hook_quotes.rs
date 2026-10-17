//!
//!
//!
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

use crate::constants::{AMM_CONFIG_SEED, APPROVED_HOOK_SEED};
use crate::errors::AmmError;
use crate::events::HookQuotesSet;
use crate::state::{AmmConfig, ApprovedHook};

#[event_cpi]
#[derive(Accounts)]
pub struct SetHookQuotes<'info> {
    #[account(constraint = authority.key() == config.load()?.authority @ AmmError::Unauthorized)]
    pub authority: Signer<'info>,
    #[account(seeds = [AMM_CONFIG_SEED], bump)]
    pub config: AccountLoader<'info, AmmConfig>,
///
///
    #[account(mut)]
    pub approved_hook: AccountLoader<'info, ApprovedHook>,
}

pub fn handler(ctx: Context<SetHookQuotes>, allow: bool) -> Result<()> {
    let (hook_program, previous) = {
        let mut a = ctx.accounts.approved_hook.load_mut()?;
        let expected = Pubkey::create_program_address(&[APPROVED_HOOK_SEED, a.hook_program.as_ref(), &[a.bump]], &crate::ID)
            .map_err(|_| AmmError::SeedsMismatch)?;
        require_keys_eq!(ctx.accounts.approved_hook.key(), expected, AmmError::SeedsMismatch);
        let previous = a.allow_hook_quotes == 1;
        a.allow_hook_quotes = u8::from(allow);
        (a.hook_program, previous)
    };
    emit_cpi!(HookQuotesSet { hook_program, previous, allowed: allow });
    Ok(())
}
