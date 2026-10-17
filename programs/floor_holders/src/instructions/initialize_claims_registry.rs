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
use floor_types::holders::ClaimsRegistry;
use floor_types::layout::Layout;

use crate::constants::{CLAIMS_ALT_AUTHORITY_SEED, CLAIMS_REGISTRY_SEED, REGISTRY_VERSION};
use crate::create;
use crate::state;

#[derive(Accounts)]
pub struct InitializeClaimsRegistry<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
///
    #[account(mut, seeds = [CLAIMS_REGISTRY_SEED], bump)]
    pub claims_registry: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeClaimsRegistry>) -> Result<()> {
    let a = &ctx.accounts;
    let info = a.claims_registry.to_account_info();
    if !create::is_absent(&info) {
        return err!(anchor_lang::error::ErrorCode::AccountDiscriminatorAlreadySet);
    }
    let bump = ctx.bumps.claims_registry;
    let bump_bytes = [bump];
    let seeds: &[&[u8]] = &[CLAIMS_REGISTRY_SEED, &bump_bytes];
    create::create_pda(
        &a.payer.to_account_info(),
        &info,
        ClaimsRegistry::ACCOUNT_LEN,
        seeds,
        anchor_lang::error::ErrorCode::AccountDiscriminatorAlreadySet.into(),
    )?;

    let (_, authority_bump) = Pubkey::find_program_address(&[CLAIMS_ALT_AUTHORITY_SEED], &crate::ID);
    let mut r: ClaimsRegistry = bytemuck::Zeroable::zeroed();
    r.version = REGISTRY_VERSION;
    r.bump = bump;
    r.authority_bump = authority_bump;
    state::init_owned(&info, &r)
}
