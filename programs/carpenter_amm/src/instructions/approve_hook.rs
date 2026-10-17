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
use carpenter_types::hook::{HOOK_INTERFACE_VERSION, MAX_ROUNDS_CAP};
use carpenter_types::layout::HookInterfaceHeader;

use crate::amm_core::fees::hook_flags_valid;
use crate::constants::{
    AFTER_SWAP_FLAG, AMM_CONFIG_SEED, APPROVED_HOOK_SEED, BEFORE_INITIALIZE_FLAG, HOOK_AUTHORITY_SEED,
    HOOK_CONFIG_SEED,
};
use crate::errors::AmmError;
use crate::events::HookApproved;
use crate::instructions::initialize_pool::programdata_slot;
use crate::state::{AmmConfig, ApprovedHook};

#[event_cpi]
#[derive(Accounts)]
pub struct ApproveHook<'info> {
    #[account(constraint = authority.key() == config.load()?.authority @ AmmError::Unauthorized)]
    pub authority: Signer<'info>,
    #[account(seeds = [AMM_CONFIG_SEED], bump)]
    pub config: AccountLoader<'info, AmmConfig>,
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
    #[account(
        init,
        payer = payer,
        space = ApprovedHook::LEN,
        seeds = [APPROVED_HOOK_SEED, hook_program.key().as_ref()],
        bump,
    )]
    pub approved_hook: AccountLoader<'info, ApprovedHook>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

///
///
///
pub(crate) fn check_executable(hook_program: &AccountInfo, hook_programdata: &Pubkey) -> Result<()> {
    require!(hook_program.executable, AmmError::HookNotExecutable);
    require_keys_eq!(*hook_program.owner, bpf_loader_upgradeable::ID, AmmError::HookNotExecutable);
    let data = hook_program.try_borrow_data()?;

    require!(data.len() >= 36 && data[0..4] == 2u32.to_le_bytes(), AmmError::HookNotExecutable);
    require!(data[4..36] == hook_programdata.to_bytes(), AmmError::HookNotExecutable);
    Ok(())
}

pub fn handler(
    ctx: Context<ApproveHook>,
    max_rounds: u8,
    callback_accounts: u8,
    allow_public_pools: bool,
    allow_exclusivity: bool,
    name: [u8; 32],
) -> Result<()> {
    let hook_program = ctx.accounts.hook_program.key();
    check_executable(&ctx.accounts.hook_program.to_account_info(), &ctx.accounts.hook_programdata.key())?;
    let programdata_slot = programdata_slot(&ctx.accounts.hook_programdata.to_account_info())?;

    let header = {
        let data = ctx.accounts.hook_config.try_borrow_data()?;
        HookInterfaceHeader::from_account_data(&data).ok_or(AmmError::HookHeaderMismatch)?
    };
    let flags = header.flags;

    require!(hook_flags_valid(flags), AmmError::HookFlagsInvalid);
    require!(allow_public_pools || flags & BEFORE_INITIALIZE_FLAG != 0, AmmError::HookFlagsInvalid);
    require!(
        header.version == HOOK_INTERFACE_VERSION
            && header.max_rounds == max_rounds
            && header.callback_accounts == callback_accounts,
        AmmError::HookHeaderMismatch
    );
    require!(max_rounds <= MAX_ROUNDS_CAP, AmmError::RoundsTooHigh);
    require!(flags & AFTER_SWAP_FLAG == 0 || max_rounds >= 2, AmmError::HookFlagsInvalid);

    let (hook_authority, hook_authority_bump) = Pubkey::find_program_address(&[HOOK_AUTHORITY_SEED], &hook_program);
    let slot = Clock::get()?.slot;
    {
        let mut a = ctx.accounts.approved_hook.load_init()?;
        a.version = 1;
        a.bump = ctx.bumps.approved_hook;
        a.paused = 0;
        a.max_rounds = max_rounds;
        a.callback_accounts = callback_accounts;
        a.allow_public_pools = allow_public_pools as u8;
        a.flags = flags;
        a.hook_program = hook_program;
        a.hook_authority = hook_authority;
        a.hook_authority_bump = hook_authority_bump;
        a.programdata_slot = programdata_slot;
        a.approved_slot = slot;
        a.name = name;
        a.allow_exclusivity = allow_exclusivity as u8;



        a.allow_hook_quotes = 1;
    }

    emit_cpi!(HookApproved {
        hook_program,
        hook_authority,
        flags,
        max_rounds,
        callback_accounts,
        allow_public_pools,
        allow_exclusivity,
        programdata_slot,
        name,
    });
    Ok(())
}
