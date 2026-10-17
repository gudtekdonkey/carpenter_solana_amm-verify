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
use floor_types::layout::{self as ly, Layout};

use crate::constants::HOOK_CONFIG_SEED;
use crate::errors::HookError;
use crate::state::HookConfig;

#[derive(Accounts)]
pub struct CreditTraderBucket<'info> {
    #[account(constraint = launch_authority.key() == hook_config.load()?.launch_authority @ HookError::NotLauncher)]
    pub launch_authority: Signer<'info>,
    #[account(seeds = [HOOK_CONFIG_SEED], bump)]
    pub hook_config: AccountLoader<'info, HookConfig>,
///
    #[account(mut, owner = crate::ID @ HookError::BadConfig)]
    pub trader_bucket: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<CreditTraderBucket>, s: [u64; 3]) -> Result<()> {
    let info = ctx.accounts.trader_bucket.to_account_info();
    let mut data = info.try_borrow_mut_data()?;
    require!(
        data.len() == 8 + ly::TraderBucket::SIZE && data[..8] == floor_types::disc::account::TRADER_BUCKET,
        HookError::BadConfig
    );
    let b: &mut ly::TraderBucket = bytemuck::from_bytes_mut(&mut data[8..8 + ly::TraderBucket::SIZE]);
    for (owed, add) in b.owed.iter_mut().zip(s) {
        *owed = owed.checked_add(add).ok_or(HookError::MathOverflow)?;
    }
    Ok(())
}
