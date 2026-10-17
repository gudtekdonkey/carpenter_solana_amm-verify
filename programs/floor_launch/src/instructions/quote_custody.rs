//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use anchor_spl::token_2022::spl_token_2022;
use anchor_spl::token_2022::spl_token_2022::extension::cpi_guard::CpiGuard;
use anchor_spl::token_2022::spl_token_2022::extension::{BaseStateWithExtensions, StateWithExtensions};
use anchor_spl::token_interface::{self, TransferChecked};
use floor_types::keys::{SPL_TOKEN_PROGRAM_ID, TOKEN_2022_PROGRAM_ID};

use crate::constants::LAUNCH_SEED;
use crate::errors::LaunchError;
use crate::graduate::token_amount;

///
const MINT_DECIMALS_OFFSET: usize = 44;

///
pub struct QuoteLeg<'a, 'info> {
    pub quote_mint: &'a AccountInfo<'info>,
    pub token_program: &'a AccountInfo<'info>,
///
    pub quote_vault: &'a AccountInfo<'info>,
///
    pub buyer_ata: &'a AccountInfo<'info>,
    pub buyer: &'a AccountInfo<'info>,
    pub launch: &'a AccountInfo<'info>,
    pub launch_bump: u8,
///
    pub base_mint: Pubkey,
}

impl<'a, 'info> QuoteLeg<'a, 'info> {
///
///
    pub fn check(&self, quote_mint: &Pubkey, quote_vault: &Pubkey) -> Result<()> {
        require_keys_eq!(self.quote_mint.key(), *quote_mint, LaunchError::BadConfig);
        require_keys_eq!(self.quote_vault.key(), *quote_vault, LaunchError::BadConfig);
        let owner = *self.quote_mint.owner;
        require!(
            owner == Pubkey::new_from_array(SPL_TOKEN_PROGRAM_ID) || owner == Pubkey::new_from_array(TOKEN_2022_PROGRAM_ID),
            LaunchError::BadConfig
        );
        require_keys_eq!(self.token_program.key(), owner, LaunchError::BadConfig);
        require_keys_eq!(
            self.buyer_ata.key(),
            get_associated_token_address_with_program_id(&self.buyer.key(), quote_mint, &owner),
            LaunchError::NotAssociatedTokenAccount
        );
        Ok(())
    }

    fn decimals(&self) -> Result<u8> {
        let data = self.quote_mint.try_borrow_data()?;
        data.get(MINT_DECIMALS_OFFSET).copied().ok_or_else(|| error!(LaunchError::BadConfig))
    }
}

///
///
pub fn take(leg: &QuoteLeg<'_, '_>, cost: u64) -> Result<()> {
    let delegate_path = cpi_guard_locked(leg.buyer_ata)?;
    if delegate_path {
        let data = leg.buyer_ata.try_borrow_data()?;
        let state = StateWithExtensions::<spl_token_2022::state::Account>::unpack(&data)?;
        require!(Option::<Pubkey>::from(state.base.delegate) == Some(leg.launch.key()), LaunchError::DelegateMissing);
        require!(state.base.delegated_amount >= cost, LaunchError::DelegateMissing);
    }
    let decimals = leg.decimals()?;
    let bump = [leg.launch_bump];
    let launch_seeds: &[&[u8]] = &[LAUNCH_SEED, leg.base_mint.as_ref(), &bump];
    let launch_signer = [launch_seeds];
    let (authority, signer): (AccountInfo, &[&[&[u8]]]) =
        if delegate_path { (leg.launch.clone(), &launch_signer) } else { (leg.buyer.clone(), &[]) };
    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            leg.token_program.key(),
            TransferChecked {
                from: leg.buyer_ata.clone(),
                mint: leg.quote_mint.clone(),
                to: leg.quote_vault.clone(),
                authority,
            },
            signer,
        ),
        cost,
        decimals,
    )
}

///
///
///
pub fn pay(leg: &QuoteLeg<'_, '_>, payout: u64) -> Result<()> {
    require!(token_amount(leg.quote_vault)? >= payout, LaunchError::EscrowShort);
    let decimals = leg.decimals()?;
    let bump = [leg.launch_bump];
    let launch_seeds: &[&[u8]] = &[LAUNCH_SEED, leg.base_mint.as_ref(), &bump];
    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            leg.token_program.key(),
            TransferChecked {
                from: leg.quote_vault.clone(),
                mint: leg.quote_mint.clone(),
                to: leg.buyer_ata.clone(),
                authority: leg.launch.clone(),
            },
            &[launch_seeds],
        ),
        payout,
        decimals,
    )
}

///
///
pub fn cpi_guard_locked(ata: &AccountInfo<'_>) -> Result<bool> {
    if *ata.owner != Pubkey::new_from_array(TOKEN_2022_PROGRAM_ID) {
        return Ok(false);
    }
    let data = ata.try_borrow_data()?;
    let state = StateWithExtensions::<spl_token_2022::state::Account>::unpack(&data)?;
    Ok(match state.get_extension::<CpiGuard>() {
        Ok(guard) => bool::from(guard.lock_cpi),
        Err(_) => false,
    })
}
