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

use crate::amm_core::mints::{SPL_TOKEN_ID, TOKEN_2022_ID};
use crate::constants::{AMM_CONFIG_SEED, APPROVED_HOOK_SEED};
use crate::errors::AmmError;
use crate::events::BaseTransferHookSet;
use crate::state::{AmmConfig, ApprovedHook};

#[event_cpi]
#[derive(Accounts)]
pub struct SetBaseTransferHook<'info> {
    #[account(constraint = authority.key() == config.load()?.authority @ AmmError::Unauthorized)]
    pub authority: Signer<'info>,
    #[account(seeds = [AMM_CONFIG_SEED], bump)]
    pub config: AccountLoader<'info, AmmConfig>,
///
///
///
    #[account(mut)]
    pub approved_hook: AccountLoader<'info, ApprovedHook>,
}

pub fn handler(ctx: Context<SetBaseTransferHook>, transfer_hook_program: Pubkey) -> Result<()> {
    require!(
        transfer_hook_program != Pubkey::default()
            && transfer_hook_program != crate::ID
            && transfer_hook_program != TOKEN_2022_ID
            && transfer_hook_program != SPL_TOKEN_ID,
        AmmError::BadBaseTransferHook
    );
    let hook_program = {
        let mut a = ctx.accounts.approved_hook.load_mut()?;
        let expected = Pubkey::create_program_address(&[APPROVED_HOOK_SEED, a.hook_program.as_ref(), &[a.bump]], &crate::ID)
            .map_err(|_| AmmError::SeedsMismatch)?;
        require_keys_eq!(ctx.accounts.approved_hook.key(), expected, AmmError::SeedsMismatch);
        require_keys_eq!(a.base_transfer_hook, Pubkey::default(), AmmError::BaseTransferHookAlreadySet);
        a.base_transfer_hook = transfer_hook_program;
        a.hook_program
    };
    emit_cpi!(BaseTransferHookSet { hook_program, transfer_hook_program });
    Ok(())
}
