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
//!
//!
//!
//!

use anchor_lang::prelude::*;
use anchor_lang::solana_program::bpf_loader_upgradeable;
use carpenter_types::hook::HOOK_INTERFACE_VERSION;
use carpenter_types::layout::HookInterfaceHeader;

use crate::amm_core::fees::hook_flags_valid;
use crate::constants::{AMM_CONFIG_SEED, APPROVED_HOOK_SEED, HOOK_CONFIG_SEED};
use crate::errors::AmmError;
use crate::events::HookReapproved;
use crate::instructions::approve_hook::check_executable;
use crate::instructions::initialize_pool::programdata_slot;
use crate::state::{AmmConfig, ApprovedHook};

#[event_cpi]
#[derive(Accounts)]
pub struct ReapproveHook<'info> {
    #[account(constraint = authority.key() == config.load()?.authority @ AmmError::Unauthorized)]
    pub authority: Signer<'info>,
    #[account(seeds = [AMM_CONFIG_SEED], bump)]
    pub config: AccountLoader<'info, AmmConfig>,
///
///
    pub hook_program: UncheckedAccount<'info>,
///
    #[account(
        seeds = [hook_program.key().as_ref()],
        bump,
        seeds::program = bpf_loader_upgradeable::ID,
    )]
    pub hook_programdata: UncheckedAccount<'info>,
///
    #[account(
        seeds = [HOOK_CONFIG_SEED],
        bump,
        seeds::program = hook_program.key(),
        owner = hook_program.key() @ AmmError::HookHeaderMismatch,
    )]
    pub hook_config: UncheckedAccount<'info>,
///
    #[account(
        mut,
        seeds = [APPROVED_HOOK_SEED, hook_program.key().as_ref()],
        bump = approved_hook.load()?.bump,
    )]
    pub approved_hook: AccountLoader<'info, ApprovedHook>,
}

pub fn handler(ctx: Context<ReapproveHook>) -> Result<()> {
    let hook_program = ctx.accounts.hook_program.key();
    check_executable(&ctx.accounts.hook_program.to_account_info(), &ctx.accounts.hook_programdata.key())?;
    let programdata_slot = programdata_slot(&ctx.accounts.hook_programdata.to_account_info())?;

    let header = {
        let data = ctx.accounts.hook_config.try_borrow_data()?;
        HookInterfaceHeader::from_account_data(&data).ok_or(AmmError::HookHeaderMismatch)?
    };

    let (previous_programdata_slot, approved_slot) = {
        let a = ctx.accounts.approved_hook.load()?;
        require_keys_eq!(a.hook_program, hook_program, AmmError::HookNotApproved);
        require!(hook_flags_valid(a.flags), AmmError::HookFlagsInvalid);
        require!(
            header.version == HOOK_INTERFACE_VERSION
                && header.flags == a.flags
                && header.max_rounds == a.max_rounds
                && header.callback_accounts == a.callback_accounts,
            AmmError::HookHeaderMismatch
        );
        (a.programdata_slot, a.approved_slot)
    };

    let approved_slot = if programdata_slot == previous_programdata_slot {
        approved_slot
    } else {
        let now = Clock::get()?.slot;
        let mut a = ctx.accounts.approved_hook.load_mut()?;
        a.programdata_slot = programdata_slot;
        a.approved_slot = now;
        now
    };

    emit_cpi!(HookReapproved { hook_program, previous_programdata_slot, programdata_slot, approved_slot });
    Ok(())
}
