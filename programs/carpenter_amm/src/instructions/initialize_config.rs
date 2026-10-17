//!
//!

use anchor_lang::prelude::*;
use anchor_lang::solana_program::bpf_loader_upgradeable;

use crate::constants::AMM_CONFIG_SEED;
use crate::errors::AmmError;
use crate::events::ConfigInitialized;
use crate::state::AmmConfig;

#[event_cpi]
#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, payer = authority, space = AmmConfig::LEN, seeds = [AMM_CONFIG_SEED], bump)]
    pub config: AccountLoader<'info, AmmConfig>,
///
    #[account(
        seeds = [crate::ID.as_ref()],
        bump,
        seeds::program = bpf_loader_upgradeable::ID,
        constraint = program_data.upgrade_authority_address == Some(authority.key()) @ AmmError::Unauthorized,
    )]
    pub program_data: Box<Account<'info, ProgramData>>,
    pub system_program: Program<'info, System>,
}

///
///
pub fn handler(ctx: Context<InitializeConfig>, config_authority: Pubkey) -> Result<()> {
    require_keys_neq!(config_authority, Pubkey::default(), AmmError::Unauthorized);
    {
        let mut c = ctx.accounts.config.load_init()?;
        c.version = 1;
        c.bump = ctx.bumps.config;
        c.authority = config_authority;
        c.pending_authority = Pubkey::default();
        c.protocol_fee_controller = Pubkey::default();


        c.protocol_fee = carpenter_types::constants::PROTOCOL_FEE_DEFAULT_PACKED;
    }
    emit_cpi!(ConfigInitialized { config: ctx.accounts.config.key(), authority: config_authority });
    Ok(())
}
