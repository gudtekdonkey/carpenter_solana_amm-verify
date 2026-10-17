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

use crate::constants::{HOOK_CONFIG_SEED, TEAM_TREASURY_SEED};
use crate::errors::HookError;
use crate::events::{TeamWithdrawn, TreasuryGovernanceChanged};
use crate::instructions::platform_treasury::{attestor_at, open_treasury, refuse_as_governance, treasury_address};
use crate::state::{HookConfig, PlatformTreasury};

///
fn team_attestor<'info>(hook_config: &AccountLoader<'info, HookConfig>, launch_config: &UncheckedAccount<'info>) -> Result<Option<Pubkey>> {
    let launch_program = hook_config.load()?.launch_program;
    attestor_at(&launch_config.to_account_info(), &launch_program)
}





#[event_cpi]
#[derive(Accounts)]
pub struct InitializeTeamTreasury<'info> {
    #[account(mut, constraint = governance.key() == hook_config.load()?.governance @ HookError::Unauthorized)]
    pub governance: Signer<'info>,
    #[account(seeds = [HOOK_CONFIG_SEED], bump)]
    pub hook_config: AccountLoader<'info, HookConfig>,
    #[account(init, payer = governance, space = PlatformTreasury::LEN, seeds = [TEAM_TREASURY_SEED], bump)]
    pub team_treasury: AccountLoader<'info, PlatformTreasury>,
///
    pub launch_config: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_team_handler(ctx: Context<InitializeTeamTreasury>, team_governance: Pubkey) -> Result<()> {
    let treasury = ctx.accounts.team_treasury.key();
    let attestor = team_attestor(&ctx.accounts.hook_config, &ctx.accounts.launch_config)?;
    refuse_as_governance(&team_governance, &treasury, attestor)?;

    let platform_treasury = treasury_address().0;
    require_keys_neq!(team_governance, platform_treasury, HookError::TreasuryIsNotGovernance);
    open_treasury(&mut *ctx.accounts.team_treasury.load_init()?, ctx.bumps.team_treasury, team_governance);
    emit_cpi!(TreasuryGovernanceChanged { treasury, previous: Pubkey::default(), governance: team_governance, accepted: true });
    Ok(())
}





#[event_cpi]
#[derive(Accounts)]
pub struct WithdrawTeam<'info> {
    #[account(constraint = governance.key() == team_treasury.load()?.governance @ HookError::Unauthorized)]
    pub governance: Signer<'info>,
    #[account(seeds = [TEAM_TREASURY_SEED], bump)]
    pub team_treasury: AccountLoader<'info, PlatformTreasury>,
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub source: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub destination: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Interface<'info, TokenInterface>,
}

///
pub fn withdraw_team_handler(ctx: Context<WithdrawTeam>, amount: u64) -> Result<()> {
    {
        let a = &ctx.accounts;
        let treasury = a.team_treasury.key();
        require!(amount > 0, HookError::BadWithdrawal);
        require_keys_eq!(a.source.owner, treasury, HookError::NotTreasuryAccount);
        require_keys_neq!(a.destination.owner, treasury, HookError::BadWithdrawal);
        require!(amount <= a.source.amount, HookError::TreasuryBalanceShort);
        let bump = [ctx.bumps.team_treasury];
        let signer: &[&[u8]] = &[TEAM_TREASURY_SEED, &bump];
        token_interface::transfer_checked(
            CpiContext::new_with_signer(
                a.token_program.key(),
                TransferChecked { from: a.source.to_account_info(), mint: a.mint.to_account_info(), to: a.destination.to_account_info(), authority: a.team_treasury.to_account_info() },
                &[signer],
            ),
            amount,
            a.mint.decimals,
        )?;
    }
    ctx.accounts.source.reload()?;
    let a = &ctx.accounts;
    let event = TeamWithdrawn { mint: a.mint.key(), source: a.source.key(), destination: a.destination.key(), governance: a.governance.key(), amount, remaining: a.source.amount };
    emit_cpi!(event);
    Ok(())
}





#[event_cpi]
#[derive(Accounts)]
pub struct SetTeamTreasuryGovernance<'info> {
    #[account(constraint = governance.key() == team_treasury.load()?.governance @ HookError::Unauthorized)]
    pub governance: Signer<'info>,
    #[account(mut, seeds = [TEAM_TREASURY_SEED], bump)]
    pub team_treasury: AccountLoader<'info, PlatformTreasury>,
    #[account(seeds = [HOOK_CONFIG_SEED], bump)]
    pub hook_config: AccountLoader<'info, HookConfig>,
///
    pub launch_config: UncheckedAccount<'info>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct AcceptTeamTreasuryGovernance<'info> {
    pub governance: Signer<'info>,
    #[account(mut, seeds = [TEAM_TREASURY_SEED], bump)]
    pub team_treasury: AccountLoader<'info, PlatformTreasury>,
    #[account(seeds = [HOOK_CONFIG_SEED], bump)]
    pub hook_config: AccountLoader<'info, HookConfig>,
///
    pub launch_config: UncheckedAccount<'info>,
}

///
pub fn set_team_governance_handler(ctx: Context<SetTeamTreasuryGovernance>, new_governance: Pubkey) -> Result<()> {
    let a = &ctx.accounts;
    let treasury = a.team_treasury.key();
    if new_governance != Pubkey::default() {
        let attestor = team_attestor(&a.hook_config, &a.launch_config)?;
        refuse_as_governance(&new_governance, &treasury, attestor)?;
    }
    let previous = {
        let mut t = a.team_treasury.load_mut()?;
        t.pending_governance = new_governance;
        t.governance
    };
    emit_cpi!(TreasuryGovernanceChanged { treasury, previous, governance: new_governance, accepted: false });
    Ok(())
}

///
///
pub fn accept_team_governance_handler(ctx: Context<AcceptTeamTreasuryGovernance>) -> Result<()> {
    let a = &ctx.accounts;
    let treasury = a.team_treasury.key();
    let pending = a.team_treasury.load()?.pending_governance;
    require_keys_neq!(pending, Pubkey::default(), HookError::Unauthorized);
    require_keys_eq!(a.governance.key(), pending, HookError::Unauthorized);
    let attestor = team_attestor(&a.hook_config, &a.launch_config)?;
    refuse_as_governance(&pending, &treasury, attestor)?;
    let previous = {
        let mut t = a.team_treasury.load_mut()?;
        let previous = t.governance;
        t.governance = pending;
        t.pending_governance = Pubkey::default();
        previous
    };
    emit_cpi!(TreasuryGovernanceChanged { treasury, previous, governance: pending, accepted: true });
    Ok(())
}
