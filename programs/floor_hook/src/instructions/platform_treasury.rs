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
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};
use floor_types::layout::Layout;

use crate::constants::{HOOK_CONFIG_SEED, LAUNCH_CONFIG_SEED, PLATFORM_TREASURY_SEED};
use crate::errors::HookError;
use crate::events::{PlatformWithdrawn, TreasuryGovernanceChanged};
use crate::instructions::initialize_hook_config::wsol_ata;
use crate::state::{HookConfig, PlatformTreasury};

///
pub fn treasury_address() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[PLATFORM_TREASURY_SEED], &crate::ID)
}

///
///
pub fn open_treasury(t: &mut PlatformTreasury, bump: u8, governance: Pubkey) {
    t.version = 1;
    t.bump = bump;
    t.governance = governance;
    t.pending_governance = Pubkey::default();
}

///
///
pub fn point_config_at_treasury(c: &mut HookConfig, treasury: &Pubkey) {
    c.platform_owner = *treasury;
    c.platform_destination = wsol_ata(treasury);
}

///
///
///
///
pub fn attestor_of(owner: &Pubkey, data: &[u8], launch_program: &Pubkey) -> Result<Option<Pubkey>> {
    if owner != launch_program {
        return Ok(None);
    }
    type L = floor_types::layout::LaunchConfig;
    const OFF: usize = 8 + core::mem::offset_of!(floor_types::layout::LaunchConfig, attestor);
    require!(data.len() >= 8 + <L as Layout>::SIZE, HookError::BadConfig);
    require!(data[..8] == floor_types::disc::account::LAUNCH_CONFIG, HookError::BadConfig);
    Ok(Some(Pubkey::new_from_array(data[OFF..OFF + 32].try_into().unwrap())))
}

///
///
///
pub fn refuse_as_governance(candidate: &Pubkey, treasury: &Pubkey, attestor: Option<Pubkey>) -> Result<()> {
    require_keys_neq!(*candidate, Pubkey::default(), HookError::BadConfig);
    require_keys_neq!(*candidate, *treasury, HookError::TreasuryIsNotGovernance);
    if let Some(attestor) = attestor {
        require_keys_neq!(*candidate, attestor, HookError::TreasuryGovernanceIsAttestor);
    }
    Ok(())
}

///
///
pub fn attestor_at(launch_config: &AccountInfo, launch_program: &Pubkey) -> Result<Option<Pubkey>> {
    let (expected, _) = Pubkey::find_program_address(&[LAUNCH_CONFIG_SEED], launch_program);
    require_keys_eq!(launch_config.key(), expected, HookError::BadConfig);
    attestor_of(launch_config.owner, &launch_config.try_borrow_data()?, launch_program)
}





///
///
///
///
///
///
///
///
///
#[event_cpi]
#[derive(Accounts)]
pub struct WithdrawPlatform<'info> {
    #[account(constraint = governance.key() == platform_treasury.load()?.governance @ HookError::Unauthorized)]
    pub governance: Signer<'info>,
    #[account(seeds = [PLATFORM_TREASURY_SEED], bump)]
    pub platform_treasury: AccountLoader<'info, PlatformTreasury>,
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub source: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub destination: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn withdraw_handler(ctx: Context<WithdrawPlatform>, amount: u64) -> Result<()> {
    {
        let a = &ctx.accounts;
        let treasury = a.platform_treasury.key();
        require!(amount > 0, HookError::BadWithdrawal);
        require_keys_eq!(a.source.owner, treasury, HookError::NotTreasuryAccount);
        require_keys_neq!(a.destination.owner, treasury, HookError::BadWithdrawal);

        require!(amount <= a.source.amount, HookError::TreasuryBalanceShort);

        let bump = [ctx.bumps.platform_treasury];
        let signer: &[&[u8]] = &[PLATFORM_TREASURY_SEED, &bump];
        token_interface::transfer_checked(
            CpiContext::new_with_signer(
                a.token_program.key(),
                TransferChecked {
                    from: a.source.to_account_info(),
                    mint: a.mint.to_account_info(),
                    to: a.destination.to_account_info(),
                    authority: a.platform_treasury.to_account_info(),
                },
                &[signer],
            ),
            amount,
            a.mint.decimals,
        )?;
    }

    ctx.accounts.source.reload()?;
    let a = &ctx.accounts;
    let event = PlatformWithdrawn {
        mint: a.mint.key(),
        source: a.source.key(),
        destination: a.destination.key(),
        governance: a.governance.key(),
        amount,
        remaining: a.source.amount,
    };
    emit_cpi!(event);
    Ok(())
}





///
///
///
///
#[event_cpi]
#[derive(Accounts)]
pub struct SetTreasuryGovernance<'info> {
    #[account(constraint = governance.key() == platform_treasury.load()?.governance @ HookError::Unauthorized)]
    pub governance: Signer<'info>,
    #[account(mut, seeds = [PLATFORM_TREASURY_SEED], bump)]
    pub platform_treasury: AccountLoader<'info, PlatformTreasury>,
    #[account(seeds = [HOOK_CONFIG_SEED], bump)]
    pub hook_config: AccountLoader<'info, HookConfig>,
///
    pub launch_config: UncheckedAccount<'info>,
}

///
///
#[event_cpi]
#[derive(Accounts)]
pub struct AcceptTreasuryGovernance<'info> {
    pub governance: Signer<'info>,
    #[account(mut, seeds = [PLATFORM_TREASURY_SEED], bump)]
    pub platform_treasury: AccountLoader<'info, PlatformTreasury>,
    #[account(seeds = [HOOK_CONFIG_SEED], bump)]
    pub hook_config: AccountLoader<'info, HookConfig>,
///
    pub launch_config: UncheckedAccount<'info>,
}

///
fn attestor_for<'info>(
    hook_config: &AccountLoader<'info, HookConfig>,
    launch_config: &UncheckedAccount<'info>,
) -> Result<Option<Pubkey>> {
    let launch_program = hook_config.load()?.launch_program;
    attestor_at(&launch_config.to_account_info(), &launch_program)
}

pub fn set_governance_handler(ctx: Context<SetTreasuryGovernance>, new_governance: Pubkey) -> Result<()> {
    let a = &ctx.accounts;
    let treasury = a.platform_treasury.key();
    if new_governance != Pubkey::default() {
        let attestor = attestor_for(&a.hook_config, &a.launch_config)?;
        refuse_as_governance(&new_governance, &treasury, attestor)?;
    }
    let previous = {
        let mut t = a.platform_treasury.load_mut()?;
        t.pending_governance = new_governance;
        t.governance
    };
    emit_cpi!(TreasuryGovernanceChanged { treasury, previous, governance: new_governance, accepted: false });
    Ok(())
}

///
///
///
pub fn accept_governance_handler(ctx: Context<AcceptTreasuryGovernance>) -> Result<()> {
    let a = &ctx.accounts;
    let treasury = a.platform_treasury.key();
    let pending = a.platform_treasury.load()?.pending_governance;
    require_keys_neq!(pending, Pubkey::default(), HookError::Unauthorized);
    require_keys_eq!(a.governance.key(), pending, HookError::Unauthorized);
    let attestor = attestor_for(&a.hook_config, &a.launch_config)?;
    refuse_as_governance(&pending, &treasury, attestor)?;
    let previous = {
        let mut t = a.platform_treasury.load_mut()?;
        let previous = t.governance;
        t.governance = pending;
        t.pending_governance = Pubkey::default();
        previous
    };
    emit_cpi!(TreasuryGovernanceChanged { treasury, previous, governance: pending, accepted: true });
    Ok(())
}

#[cfg(test)]
mod tests {
//!
//!
    use super::*;

    fn launch_config_bytes(attestor: &Pubkey) -> Vec<u8> {
        let mut d = floor_types::disc::account::LAUNCH_CONFIG.to_vec();
        let mut body = vec![0u8; <floor_types::layout::LaunchConfig as Layout>::SIZE];
        let off = core::mem::offset_of!(floor_types::layout::LaunchConfig, attestor);
        body[off..off + 32].copy_from_slice(attestor.as_ref());
        d.extend_from_slice(&body);
        d
    }

    #[test]
    fn the_attestor_is_read_only_off_the_launch_programs_own_config() {
        let launch_program = Pubkey::new_unique();
        let attestor = Pubkey::new_unique();
        let data = launch_config_bytes(&attestor);
        assert_eq!(attestor_of(&launch_program, &data, &launch_program).unwrap(), Some(attestor));

        assert_eq!(attestor_of(&Pubkey::default(), &[], &launch_program).unwrap(), None);

        let mut wrong = data.clone();
        wrong[0] ^= 1;
        assert!(attestor_of(&launch_program, &wrong, &launch_program).is_err(), "a foreign discriminator");
        assert!(attestor_of(&launch_program, &data[..40], &launch_program).is_err(), "a short body");
    }

    #[test]
    fn a_governance_candidate_is_refused_in_every_role_it_may_not_hold() {
        let (treasury, _) = treasury_address();
        let attestor = Pubkey::new_unique();
        let ok = Pubkey::new_unique();
        let code = |r: Result<()>| match r {
            Err(anchor_lang::error::Error::AnchorError(e)) => e.error_name,
            other => format!("{other:?}"),
        };
        assert!(refuse_as_governance(&ok, &treasury, Some(attestor)).is_ok());
        assert!(refuse_as_governance(&ok, &treasury, None).is_ok(), "no launch config yet: no attestor");
        assert_eq!(code(refuse_as_governance(&Pubkey::default(), &treasury, None)), "BadConfig");
        assert_eq!(code(refuse_as_governance(&treasury, &treasury, None)), "TreasuryIsNotGovernance");
        assert_eq!(code(refuse_as_governance(&attestor, &treasury, Some(attestor))), "TreasuryGovernanceIsAttestor");
    }

    #[test]
    fn the_config_points_at_the_treasury_and_its_wsol_ata() {
        let (treasury, bump) = treasury_address();
        let mut c: HookConfig = bytemuck::Zeroable::zeroed();
        point_config_at_treasury(&mut c, &treasury);
        assert_eq!(c.platform_owner, treasury);
        assert_eq!(c.platform_destination, wsol_ata(&treasury));
        let mut t: PlatformTreasury = bytemuck::Zeroable::zeroed();
        t.pending_governance = Pubkey::new_unique();
        let g = Pubkey::new_unique();
        open_treasury(&mut t, bump, g);
        assert_eq!((t.version, t.bump, t.governance, t.pending_governance), (1, bump, g, Pubkey::default()));
    }
}
