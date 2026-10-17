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

use crate::constants::{HOOK_AUTHORITY_SEED, HOOK_CONFIG_SEED, PLATFORM_TREASURY_SEED, TEAM_TREASURY_SEED};
use crate::errors::HookError;
use crate::events::HookGovernanceChanged;
use crate::instructions::platform_treasury::attestor_at;
use crate::state::HookConfig;

///
///
pub fn unsignable_addresses(program_id: &Pubkey, launch_authority: &Pubkey) -> [Pubkey; 5] {
    let pda = |seed: &[u8]| Pubkey::find_program_address(&[seed], program_id).0;
    [
        pda(HOOK_CONFIG_SEED),
        pda(HOOK_AUTHORITY_SEED),
        pda(PLATFORM_TREASURY_SEED),
        pda(TEAM_TREASURY_SEED),
        *launch_authority,
    ]
}

///
///
pub fn refuse_as_hook_governance(
    candidate: &Pubkey,
    unsignable: &[Pubkey],
    attestor: Option<Pubkey>,
) -> Result<()> {
    require_keys_neq!(*candidate, Pubkey::default(), HookError::BadConfig);
    if let Some(attestor) = attestor {
        require_keys_neq!(*candidate, attestor, HookError::HookGovernanceIsAttestor);
    }
    require!(!unsignable.contains(candidate), HookError::HookGovernanceCannotSign);
    Ok(())
}

///
#[event_cpi]
#[derive(Accounts)]
pub struct SetHookGovernance<'info> {
    #[account(constraint = governance.key() == hook_config.load()?.governance @ HookError::Unauthorized)]
    pub governance: Signer<'info>,
    #[account(mut, seeds = [HOOK_CONFIG_SEED], bump)]
    pub hook_config: AccountLoader<'info, HookConfig>,
///
///
    pub launch_config: UncheckedAccount<'info>,
}

///
#[event_cpi]
#[derive(Accounts)]
pub struct AcceptHookGovernance<'info> {
    pub governance: Signer<'info>,
    #[account(mut, seeds = [HOOK_CONFIG_SEED], bump)]
    pub hook_config: AccountLoader<'info, HookConfig>,
///
    pub launch_config: UncheckedAccount<'info>,
}

fn refuse<'info>(
    candidate: &Pubkey,
    hook_config: &AccountLoader<'info, HookConfig>,
    launch_config: &UncheckedAccount<'info>,
) -> Result<()> {
    let (launch_program, launch_authority) = {
        let c = hook_config.load()?;
        (c.launch_program, c.launch_authority)
    };
    let attestor = attestor_at(&launch_config.to_account_info(), &launch_program)?;
    refuse_as_hook_governance(candidate, &unsignable_addresses(&crate::ID, &launch_authority), attestor)
}

pub fn set_handler(ctx: Context<SetHookGovernance>, new_governance: Pubkey) -> Result<()> {
    let a = &ctx.accounts;
    if new_governance != Pubkey::default() {
        refuse(&new_governance, &a.hook_config, &a.launch_config)?;
    }
    let previous = {
        let mut c = a.hook_config.load_mut()?;
        c.pending_governance = new_governance;
        c.governance
    };
    emit_cpi!(HookGovernanceChanged { previous, governance: new_governance, accepted: false });
    Ok(())
}

///
///
pub fn accept_handler(ctx: Context<AcceptHookGovernance>) -> Result<()> {
    let a = &ctx.accounts;
    let pending = a.hook_config.load()?.pending_governance;
    require_keys_neq!(pending, Pubkey::default(), HookError::Unauthorized);
    require_keys_eq!(a.governance.key(), pending, HookError::Unauthorized);
    refuse(&pending, &a.hook_config, &a.launch_config)?;
    let previous = {
        let mut c = a.hook_config.load_mut()?;
        let previous = c.governance;
        c.governance = pending;
        c.pending_governance = Pubkey::default();
        previous
    };
    emit_cpi!(HookGovernanceChanged { previous, governance: pending, accepted: true });
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
    fn a_hook_governance_candidate_is_refused_in_every_role_it_may_not_hold() {
        let launch_authority = Pubkey::new_unique();
        let unsignable = unsignable_addresses(&crate::ID, &launch_authority);
        let attestor = Pubkey::new_unique();
        let ok = Pubkey::new_unique();
        assert!(refuse_as_hook_governance(&ok, &unsignable, Some(attestor)).is_ok());
        assert!(refuse_as_hook_governance(&ok, &unsignable, None).is_ok(), "no launch config yet: no attestor");
        assert_eq!(name(refuse_as_hook_governance(&Pubkey::default(), &unsignable, None)), "BadConfig");
        assert_eq!(
            name(refuse_as_hook_governance(&attestor, &unsignable, Some(attestor))),
            "HookGovernanceIsAttestor"
        );
        for key in unsignable {
            assert_eq!(name(refuse_as_hook_governance(&key, &unsignable, None)), "HookGovernanceCannotSign");
        }
    }

///
///
    #[test]
    fn the_new_errors_are_appended_at_7060_and_7061() {
        assert_eq!(u32::from(HookError::HookGovernanceIsAttestor), 7060);
        assert_eq!(u32::from(HookError::HookGovernanceCannotSign), 7061);
        assert_eq!(u32::from(HookError::StairPayeeResolved), 7059, "the last code before them, unmoved");
    }

///
///
    #[test]
    fn the_unsignable_addresses_are_the_programs_own_pdas() {
        let la = Pubkey::new_unique();
        let u = unsignable_addresses(&crate::ID, &la);
        let (hook_config, _) = Pubkey::find_program_address(&[HOOK_CONFIG_SEED], &crate::ID);
        let (treasury, _) = Pubkey::find_program_address(&[PLATFORM_TREASURY_SEED], &crate::ID);
        assert!(u.contains(&hook_config) && u.contains(&treasury) && u.contains(&la));
        for i in 0..u.len() {
            for j in i + 1..u.len() {
                assert_ne!(u[i], u[j], "every entry distinct");
            }
        }
    }
}
