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

use crate::amm_core::mints::QUOTE_CAPS_CUSTODIAL;
use crate::constants::AMM_CONFIG_SEED;
use crate::errors::AmmError;
use crate::events::QuoteCapsAdmittedSet;
use crate::state::AmmConfig;

#[event_cpi]
#[derive(Accounts)]
pub struct SetQuoteCapsAdmitted<'info> {
    #[account(constraint = authority.key() == config.load()?.authority @ AmmError::Unauthorized)]
    pub authority: Signer<'info>,
    #[account(mut, seeds = [AMM_CONFIG_SEED], bump)]
    pub config: AccountLoader<'info, AmmConfig>,
}

pub fn handler(ctx: Context<SetQuoteCapsAdmitted>, admitted: u8) -> Result<()> {
    require!(admitted & !QUOTE_CAPS_CUSTODIAL == 0, AmmError::QuoteCapsInvalid);
    let previous = {
        let mut c = ctx.accounts.config.load_mut()?;
        let p = c.quote_caps_admitted;
        c.quote_caps_admitted = admitted;
        p
    };
    emit_cpi!(QuoteCapsAdmittedSet { previous, admitted });
    Ok(())
}
