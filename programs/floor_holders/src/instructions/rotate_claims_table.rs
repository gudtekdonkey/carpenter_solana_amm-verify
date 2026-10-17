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

use crate::alt::{derive_table, AltCpi};
use crate::constants::{ALT_PROGRAM, CLAIMS_ADDRS_HOOKED, CLAIMS_ALT_AUTHORITY_SEED, CLAIMS_REGISTRY_SEED, CLAIMS_TABLE_MAX};
use crate::errors::HoldersError;
use crate::events::ClaimsTableRotated;
use crate::state;

#[event_cpi]
#[derive(Accounts)]
pub struct RotateClaimsTable<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
///
    #[account(mut, seeds = [CLAIMS_REGISTRY_SEED], bump)]
    pub claims_registry: UncheckedAccount<'info>,
///
    #[account(seeds = [CLAIMS_ALT_AUTHORITY_SEED], bump)]
    pub claims_alt_authority: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub new_table: UncheckedAccount<'info>,
///
    #[account(address = ALT_PROGRAM)]
    pub alt_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RotateClaimsTable>, recent_slot: u64) -> Result<()> {
    let a = &ctx.accounts;
    let registry_info = a.claims_registry.to_account_info();
    let mut reg = state::read_registry(&registry_info).ok_or(anchor_lang::error::ErrorCode::AccountNotInitialized)?;

    let rotate_at = CLAIMS_TABLE_MAX - CLAIMS_ADDRS_HOOKED as u16;
    let full = reg.current_table == [0u8; 32] || reg.current_len > rotate_at;
    require!(full, HoldersError::ClaimsTableNotFull);

    let authority = a.claims_alt_authority.key();
    let (expected, table_bump) = derive_table(&authority, recent_slot);
    require_keys_eq!(a.new_table.key(), expected, anchor_lang::error::ErrorCode::ConstraintSeeds);

    let alt = AltCpi {
        alt_program: &a.alt_program.to_account_info(),
        table: &a.new_table.to_account_info(),
        authority: &a.claims_alt_authority.to_account_info(),
        authority_bump: ctx.bumps.claims_alt_authority,
        payer: &a.payer.to_account_info(),
        system_program: &a.system_program.to_account_info(),
    };
    alt.create(recent_slot, table_bump)?;

    reg.current_table = expected.to_bytes();
    reg.current_len = 0;
    reg.tables = reg.tables.saturating_add(1);
    let tables = reg.tables;
    state::write_owned(&registry_info, &reg)?;

    emit_cpi!(ClaimsTableRotated { table: expected, tables });
    Ok(())
}
