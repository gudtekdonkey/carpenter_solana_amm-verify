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
//!
//!

use anchor_lang::prelude::*;

use floor_types::constants::{
    MAX_BOUNTY_BPS, MAX_BOUNTY_LAMPORTS_CEIL, MIN_BID_LAMPORTS_CEIL, MIN_BID_LAMPORTS_FLOOR,
};

use crate::constants::{FLOOR_HOOK_PROGRAM, LAUNCH_CONFIG_SEED, PLATFORM_TREASURY_SEED};
use crate::errors::LaunchError;
use crate::events::LaunchEconomicsUpdated;
use crate::instructions::treasury_gov;
use crate::state::LaunchConfig;

#[event_cpi]
#[derive(Accounts)]
pub struct SetLaunchEconomics<'info> {
    pub authority: Signer<'info>,
    #[account(mut, seeds = [LAUNCH_CONFIG_SEED], bump)]
    pub launch_config: AccountLoader<'info, LaunchConfig>,
///
///
    #[account(seeds = [PLATFORM_TREASURY_SEED], bump, seeds::program = FLOOR_HOOK_PROGRAM)]
    pub platform_treasury: UncheckedAccount<'info>,
}

///
///
///
///
///
///
///
pub fn handler(
    ctx: Context<SetLaunchEconomics>,
    min_bid_lamports: u64,
    bounty_bps: u16,
    max_bounty_lamports: u64,
) -> Result<()> {
    let gov = treasury_gov(&ctx.accounts.platform_treasury.to_account_info())?;
    require!(gov == Some(ctx.accounts.authority.key()), LaunchError::Unauthorized);
    require!(
        (MIN_BID_LAMPORTS_FLOOR..=MIN_BID_LAMPORTS_CEIL).contains(&min_bid_lamports),
        LaunchError::MinBidOutOfRange
    );
    require!(bounty_bps <= MAX_BOUNTY_BPS, LaunchError::BountyOutOfRange);
    require!(max_bounty_lamports <= MAX_BOUNTY_LAMPORTS_CEIL, LaunchError::BountyOutOfRange);

    let mut c = ctx.accounts.launch_config.load_mut()?;
    let old = (c.min_bid_lamports, c.bounty_bps, c.max_bounty_lamports);
    c.min_bid_lamports = min_bid_lamports;
    c.bounty_bps = bounty_bps;
    c.max_bounty_lamports = max_bounty_lamports;
    drop(c);

    emit_cpi!(LaunchEconomicsUpdated {
        launch_config: ctx.accounts.launch_config.key(),
        authority: ctx.accounts.authority.key(),
        old_min_bid_lamports: old.0,
        min_bid_lamports,
        old_bounty_bps: old.1,
        bounty_bps,
        old_max_bounty_lamports: old.2,
        max_bounty_lamports,
    });
    Ok(())
}
