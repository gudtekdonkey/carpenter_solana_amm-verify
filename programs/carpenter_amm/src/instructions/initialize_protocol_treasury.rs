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

use crate::constants::{AMM_CONFIG_SEED, PROTOCOL_TREASURY_SEED};
use crate::errors::AmmError;
use crate::events::ProtocolTreasuryGovernanceChanged;
use crate::state::{AmmConfig, ProtocolTreasury};

///
pub fn treasury_address() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[PROTOCOL_TREASURY_SEED], &crate::ID)
}

///
pub fn open_treasury(t: &mut ProtocolTreasury, bump: u8, governance: Pubkey) {
    t.version = 1;
    t.bump = bump;
    t.governance = governance;
    t.pending_governance = Pubkey::default();
}

///
///
///
///
pub fn refuse_as_governance(candidate: &Pubkey, treasury: &Pubkey) -> Result<()> {
    require_keys_neq!(*candidate, Pubkey::default(), AmmError::BadTreasuryGovernance);
    require_keys_neq!(*candidate, *treasury, AmmError::BadTreasuryGovernance);
    Ok(())
}

#[event_cpi]
#[derive(Accounts)]
pub struct InitializeProtocolTreasury<'info> {
///
    #[account(mut, constraint = authority.key() == config.load()?.authority @ AmmError::Unauthorized)]
    pub authority: Signer<'info>,
    #[account(seeds = [AMM_CONFIG_SEED], bump)]
    pub config: AccountLoader<'info, AmmConfig>,
    #[account(init, payer = authority, space = ProtocolTreasury::LEN, seeds = [PROTOCOL_TREASURY_SEED], bump)]
    pub protocol_treasury: AccountLoader<'info, ProtocolTreasury>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeProtocolTreasury>, governance: Pubkey) -> Result<()> {
    let treasury = ctx.accounts.protocol_treasury.key();
    refuse_as_governance(&governance, &treasury)?;
    open_treasury(&mut *ctx.accounts.protocol_treasury.load_init()?, ctx.bumps.protocol_treasury, governance);
    emit_cpi!(ProtocolTreasuryGovernanceChanged { treasury, previous: Pubkey::default(), governance, accepted: true });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(r: Result<()>) -> String {
        match r {
            Err(anchor_lang::error::Error::AnchorError(e)) => e.error_name,
            other => format!("{other:?}"),
        }
    }

///
///
    #[test]
    fn a_governance_candidate_is_never_zero_and_never_the_treasury() {
        let (treasury, _) = treasury_address();
        assert!(refuse_as_governance(&Pubkey::new_unique(), &treasury).is_ok());
        assert_eq!(name(refuse_as_governance(&Pubkey::default(), &treasury)), "BadTreasuryGovernance");
        assert_eq!(name(refuse_as_governance(&treasury, &treasury)), "BadTreasuryGovernance");
    }

    #[test]
    fn a_new_treasury_names_its_governance_and_nothing_pending() {
        let (_, bump) = treasury_address();
        let mut t: ProtocolTreasury = bytemuck::Zeroable::zeroed();
        t.pending_governance = Pubkey::new_unique();
        let g = Pubkey::new_unique();
        open_treasury(&mut t, bump, g);
        assert_eq!((t.version, t.bump, t.governance, t.pending_governance), (1, bump, g, Pubkey::default()));
    }

    #[test]
    fn the_treasury_is_the_seed_s_pda_under_this_program() {
        let (a, _) = treasury_address();
        let (b, _) = Pubkey::find_program_address(&[b"protocol_treasury"], &crate::ID);
        assert_eq!(a, b);
    }
}
