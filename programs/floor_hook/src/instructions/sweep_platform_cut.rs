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
use anchor_spl::associated_token::{self, get_associated_token_address_with_program_id, AssociatedToken};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::{
    EVENT_AUTHORITY_SEED, CARPENTER_AMM_PROGRAM, HOOK_AUTHORITY_SEED, HOOK_CONFIG_SEED, PLATFORM_TREASURY_SEED,
    TEAM_TREASURY_SEED, WSOL_MINT,
};
use crate::errors::HookError;
use crate::events::{PlatformSwept, TeamSwept};
use crate::mech::amm_cpi::{self, AmmCpi};
use crate::mech::positions::add;
use crate::mech::ratchet;
use crate::state::{FloorPool, HookConfig, PlatformTreasury};

#[event_cpi]
#[derive(Accounts)]
pub struct SweepPlatformCut<'info> {
///
///
    #[account(mut)]
    pub anyone: Signer<'info>,
    #[account(mut)]
    pub floor_pool: AccountLoader<'info, FloorPool>,
    #[account(seeds = [HOOK_CONFIG_SEED], bump)]
    pub hook_config: AccountLoader<'info, HookConfig>,
///
    #[account(mut, address = floor_pool.load()?.amm_pool @ HookError::WrongPool)]
    pub pool: UncheckedAccount<'info>,
    #[account(mut)]
    pub vault0: Box<InterfaceAccount<'info, TokenAccount>>,
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
    #[account(mut)]
    pub destination: UncheckedAccount<'info>,
///
///
///
///
///
///
    #[account(address = floor_pool.load()?.quote_mint @ HookError::WrongPool)]
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
    pub spl_token: Interface<'info, TokenInterface>,
///
    #[account(address = CARPENTER_AMM_PROGRAM)]
    pub carpenter_amm: UncheckedAccount<'info>,
///
    #[account(seeds = [HOOK_AUTHORITY_SEED], bump)]
    pub hook_authority: UncheckedAccount<'info>,
///
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = CARPENTER_AMM_PROGRAM)]
    pub amm_event_authority: UncheckedAccount<'info>,
///
///
///
    #[account(seeds = [PLATFORM_TREASURY_SEED], bump)]
    pub platform_treasury: UncheckedAccount<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

///
///
///
pub fn platform_destination_for(owner: &Pubkey, quote_mint: &Pubkey, token_program: &Pubkey) -> Pubkey {
    get_associated_token_address_with_program_id(owner, quote_mint, token_program)
}

///
///
pub fn platform_leg_amount(s: &FloorPool, quote_mint: &Pubkey) -> u64 {
    if *quote_mint == WSOL_MINT { 0 } else { s.platform_owed }
}

///
pub const TEAM_LEG_ACCOUNTS: usize = 2;

///
fn team_treasury_open(info: &AccountInfo) -> Result<bool> {
    if info.owner != &crate::ID || info.data_len() != PlatformTreasury::LEN {
        return Ok(false);
    }
    Ok(&info.try_borrow_data()?[..8] == PlatformTreasury::DISCRIMINATOR)
}

///
fn team_leg<'a, 'info>(s: &FloorPool, rest: &'a [AccountInfo<'info>], quote_mint: &Pubkey, token_program: &Pubkey) -> Result<Option<(&'a AccountInfo<'info>, &'a AccountInfo<'info>)>> {
    require!(rest.is_empty() || rest.len() == TEAM_LEG_ACCOUNTS, HookError::WrongDestination);
    if s.team_owed == 0 || rest.is_empty() {
        return Ok(None);
    }
    let (team_treasury, team_destination) = (&rest[0], &rest[1]);
    if !team_treasury_open(team_treasury)? {
        return Ok(None);
    }
    let (team_pda, _) = Pubkey::find_program_address(&[TEAM_TREASURY_SEED], &crate::ID);
    require_keys_eq!(team_treasury.key(), team_pda, HookError::WrongDestination);
    let team_derived = platform_destination_for(&team_pda, quote_mint, token_program);
    require_keys_eq!(team_destination.key(), team_derived, HookError::WrongDestination);
    require!(team_destination.is_writable, HookError::WrongDestination);
    Ok(Some((team_treasury, team_destination)))
}

///
///
pub fn handler<'info>(ctx: Context<'info, SweepPlatformCut<'info>>) -> Result<()> {
    let mut s: FloorPool = *ctx.accounts.floor_pool.load()?;
    require!(s.pool_bound == 1, HookError::NotBound);
    let (platform_destination, platform_owner) = {
        let c = ctx.accounts.hook_config.load()?;
        (c.platform_destination, c.platform_owner)
    };


    let treasury = ctx.accounts.platform_treasury.key();
    require_keys_eq!(platform_owner, treasury, HookError::TreasuryNotMigrated);

    let a = &ctx.accounts;
    let p = amm_cpi::read_pool(&a.pool.to_account_info())?;
    amm_cpi::require_pool_side(&p, 0, &a.vault0.key(), &a.quote_mint.key(), &a.spl_token.key())?;









    let derived = platform_destination_for(&treasury, &a.quote_mint.key(), &a.spl_token.key());
    require_keys_eq!(a.destination.key(), derived, HookError::WrongDestination);
    if a.quote_mint.key() == WSOL_MINT {
        require_keys_eq!(derived, platform_destination, HookError::WrongDestination);
    }



    let team = team_leg(&s, ctx.remaining_accounts, &a.quote_mint.key(), &a.spl_token.key())?;



    let amount = platform_leg_amount(&s, &a.quote_mint.key());
    if amount == 0 && team.is_none() {
        return Ok(());
    }
    if amount > 0 {




        if a.destination.data_is_empty() {
            associated_token::create_idempotent(CpiContext::new(
                a.associated_token_program.key(),
                associated_token::Create {
                    payer: a.anyone.to_account_info(),
                    associated_token: a.destination.to_account_info(),
                    authority: a.platform_treasury.to_account_info(),
                    mint: a.quote_mint.to_account_info(),
                    system_program: a.system_program.to_account_info(),
                    token_program: a.spl_token.to_account_info(),
                },
            ))?;
        }


        let info = a.destination.to_account_info();
        require_keys_eq!(*info.owner, a.spl_token.key(), HookError::WrongDestination);
        let dest = TokenAccount::try_deserialize(&mut &info.try_borrow_data()?[..])?;
        require_keys_eq!(dest.owner, treasury, HookError::WrongDestination);
        require_keys_eq!(dest.mint, a.quote_mint.key(), HookError::WrongDestination);
    }



    let team_amount = match team {
        Some((team_treasury, team_dest)) => {
            if team_dest.data_is_empty() {
                associated_token::create_idempotent(CpiContext::new(
                    a.associated_token_program.key(),
                    associated_token::Create {
                        payer: a.anyone.to_account_info(),
                        associated_token: team_dest.clone(),
                        authority: team_treasury.clone(),
                        mint: a.quote_mint.to_account_info(),
                        system_program: a.system_program.to_account_info(),
                        token_program: a.spl_token.to_account_info(),
                    },
                ))?;
            }
            require_keys_eq!(*team_dest.owner, a.spl_token.key(), HookError::WrongDestination);
            let acct = TokenAccount::try_deserialize(&mut &team_dest.try_borrow_data()?[..])?;
            require_keys_eq!(acct.owner, team_treasury.key(), HookError::WrongDestination);
            require_keys_eq!(acct.mint, a.quote_mint.key(), HookError::WrongDestination);
            s.team_owed
        }
        None => 0,
    };



    s.platform_owed = s.platform_owed.checked_sub(amount).ok_or(HookError::MathOverflow)?;
    s.platform_swept = add(s.platform_swept, amount)?;
    if team_amount > 0 {
        s.team_owed = 0;
        s.team_fees_paid = add(s.team_fees_paid, team_amount)?;
    }
    let pool = s.amm_pool;
    *ctx.accounts.floor_pool.load_mut()? = s;

    let a = &ctx.accounts;
    let cpi = AmmCpi {
        carpenter_amm: a.carpenter_amm.to_account_info(),
        amm_event_authority: a.amm_event_authority.to_account_info(),
        hook_authority: a.hook_authority.to_account_info(),
        hook_authority_bump: ctx.bumps.hook_authority,
        hook_program: a.program.to_account_info(),
        floor_pool: a.floor_pool.to_account_info(),
        pool: a.pool.to_account_info(),
    };
    if amount > 0 {
        let w = amm_cpi::withdraw(
            &cpi,
            &a.vault0.to_account_info(),
            &a.destination.to_account_info(),
            &a.quote_mint.to_account_info(),
            &a.spl_token.to_account_info(),
            0,
            amount,
        )?;


        require!(
            ratchet::claims_cover_liabilities(&s, w.hook_claims0, w.hook_claims1),
            HookError::InvariantViolated
        );
        emit_cpi!(PlatformSwept { pool, destination: derived, amount });
    }


    if let (Some((_, team_dest)), true) = (team, team_amount > 0) {
        let w = amm_cpi::withdraw(&cpi, &a.vault0.to_account_info(), team_dest, &a.quote_mint.to_account_info(), &a.spl_token.to_account_info(), 0, team_amount)?;
        require!(ratchet::claims_cover_liabilities(&s, w.hook_claims0, w.hook_claims1), HookError::InvariantViolated);
        emit_cpi!(TeamSwept { pool, mint: a.quote_mint.key(), amount: team_amount });
    }
    Ok(())
}
