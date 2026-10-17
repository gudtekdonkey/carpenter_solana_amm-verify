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

use floor_types::constants::{TEAM_POOL_FEE_BPS_MAX, TEAM_SALE_FEE_BPS_MAX};

use crate::constants::{FLOOR_HOOK_PROGRAM, LAUNCH_CONFIG_SEED, TEAM_TREASURY_SEED};
use crate::errors::LaunchError;
use crate::events::TeamFeesSet;



use crate::instructions::treasury_gov;
use crate::state::LaunchConfig;

#[event_cpi]
#[derive(Accounts)]
pub struct SetTeamFees<'info> {
///
    pub governance: Signer<'info>,
    #[account(mut, seeds = [LAUNCH_CONFIG_SEED], bump)]
    pub launch_config: AccountLoader<'info, LaunchConfig>,
///
///
///
    #[account(seeds = [TEAM_TREASURY_SEED], bump, seeds::program = FLOOR_HOOK_PROGRAM)]
    pub team_treasury: UncheckedAccount<'info>,
}

///
pub fn check_team_fee_caps(sale_bps: u16, pool_bps: u16) -> Result<()> {
    require!(sale_bps <= TEAM_SALE_FEE_BPS_MAX, LaunchError::TeamFeeOutOfRange);
    require!(pool_bps <= TEAM_POOL_FEE_BPS_MAX, LaunchError::TeamFeeOutOfRange);
    Ok(())
}

pub fn handler(ctx: Context<SetTeamFees>, sale_bps: u16, pool_bps: u16) -> Result<()> {
    check_team_fee_caps(sale_bps, pool_bps)?;
    let team = treasury_gov(&ctx.accounts.team_treasury.to_account_info())?.ok_or(error!(LaunchError::TeamTreasuryNotOpen))?;
    require_keys_eq!(ctx.accounts.governance.key(), team, LaunchError::Unauthorized);

    let (old_sale_bps, old_pool_bps) = {
        let mut c = ctx.accounts.launch_config.load_mut()?;
        let old = (c.team_sale_bps(), c.team_pool_bps());
        c.team_sale_bps_raw = sale_bps;
        c.team_pool_bps_raw = pool_bps;
        c.team_bps_set = 1;
        old
    };
    emit_cpi!(TeamFeesSet {
        launch_config: ctx.accounts.launch_config.key(),
        governance: ctx.accounts.governance.key(),
        old_sale_bps,
        sale_bps,
        old_pool_bps,
        pool_bps,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

///
///
///
    #[test]
    fn the_caps_are_inclusive_and_zero_is_allowed() {
        assert!(check_team_fee_caps(0, 0).is_ok());
        assert!(check_team_fee_caps(100, 50).is_ok());
        assert!(check_team_fee_caps(40, 10).is_ok());
        assert_eq!(check_team_fee_caps(101, 0).unwrap_err(), error!(LaunchError::TeamFeeOutOfRange));
        assert_eq!(check_team_fee_caps(0, 51).unwrap_err(), error!(LaunchError::TeamFeeOutOfRange));
        assert_eq!(check_team_fee_caps(100, 100).unwrap_err(), error!(LaunchError::TeamFeeOutOfRange), "the pool cap is 50, not the sale's 100");
        assert_eq!(check_team_fee_caps(u16::MAX, u16::MAX).unwrap_err(), error!(LaunchError::TeamFeeOutOfRange));
    }
}
