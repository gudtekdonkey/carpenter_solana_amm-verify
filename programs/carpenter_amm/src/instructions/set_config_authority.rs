//!
//!

use anchor_lang::prelude::*;

use crate::constants::AMM_CONFIG_SEED;
use crate::errors::AmmError;
use crate::events::ConfigAuthorityChanged;
use crate::state::AmmConfig;

#[event_cpi]
#[derive(Accounts)]
pub struct SetConfigAuthority<'info> {
    pub authority: Signer<'info>,
    #[account(mut, seeds = [AMM_CONFIG_SEED], bump)]
    pub config: AccountLoader<'info, AmmConfig>,
}

///
///
pub fn handler(ctx: Context<SetConfigAuthority>, new_authority: Pubkey) -> Result<()> {
    let previous = {
        let mut c = ctx.accounts.config.load_mut()?;
        require_keys_eq!(ctx.accounts.authority.key(), c.authority, AmmError::Unauthorized);
        c.pending_authority = new_authority;
        c.authority
    };
    emit_cpi!(ConfigAuthorityChanged {
        config: ctx.accounts.config.key(),
        previous,
        authority: new_authority,
        accepted: false,
    });
    Ok(())
}
