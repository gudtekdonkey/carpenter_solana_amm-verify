//!
//!
//!
//!
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

use crate::constants::PLATFORM_TREASURY_SEED;
use crate::errors::HookError;
use crate::events::AllowAllQuotesSet;
use crate::state::PlatformTreasury;

#[event_cpi]
#[derive(Accounts)]
pub struct SetAllowAllQuotes<'info> {
    #[account(constraint = governance.key() == platform_treasury.load()?.governance @ HookError::Unauthorized)]
    pub governance: Signer<'info>,
    #[account(mut, seeds = [PLATFORM_TREASURY_SEED], bump)]
    pub platform_treasury: AccountLoader<'info, PlatformTreasury>,
}

pub fn handler(ctx: Context<SetAllowAllQuotes>, allow: bool) -> Result<()> {
    let previous = {
        let mut t = ctx.accounts.platform_treasury.load_mut()?;
        let previous = t.allow_all_quotes == 1;
        t.allow_all_quotes = u8::from(allow);
        previous
    };
    emit_cpi!(AllowAllQuotesSet {
        treasury: ctx.accounts.platform_treasury.key(),
        governance: ctx.accounts.governance.key(),
        previous,
        allowed: allow,
    });
    Ok(())
}
