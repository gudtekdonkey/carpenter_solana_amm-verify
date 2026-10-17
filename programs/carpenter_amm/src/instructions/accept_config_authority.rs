//!
//!

use anchor_lang::prelude::*;

use crate::constants::AMM_CONFIG_SEED;
use crate::errors::AmmError;
use crate::events::ConfigAuthorityChanged;
use crate::state::AmmConfig;

#[event_cpi]
#[derive(Accounts)]
pub struct AcceptConfigAuthority<'info> {
    pub authority: Signer<'info>,
    #[account(mut, seeds = [AMM_CONFIG_SEED], bump)]
    pub config: AccountLoader<'info, AmmConfig>,
}

pub fn handler(ctx: Context<AcceptConfigAuthority>) -> Result<()> {
    let (previous, accepted) = {
        let mut c = ctx.accounts.config.load_mut()?;
        require_keys_neq!(c.pending_authority, Pubkey::default(), AmmError::Unauthorized);
        require_keys_eq!(ctx.accounts.authority.key(), c.pending_authority, AmmError::Unauthorized);
        let previous = c.authority;
        c.authority = c.pending_authority;
        c.pending_authority = Pubkey::default();
        (previous, c.authority)
    };
    emit_cpi!(ConfigAuthorityChanged {
        config: ctx.accounts.config.key(),
        previous,
        authority: accepted,
        accepted: true,
    });
    Ok(())
}
