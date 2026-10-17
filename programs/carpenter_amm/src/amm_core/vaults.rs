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
use anchor_lang::solana_program::program::{invoke, invoke_signed};
use anchor_lang::solana_program::program_option::COption;
use anchor_lang::system_program::{self, transfer as lamports_move, Allocate, Assign, Transfer as LamportsMove};
use anchor_spl::token_2022::spl_token_2022::extension::cpi_guard::CpiGuard;
use anchor_spl::token_2022::spl_token_2022::extension::pausable::PausableConfig;
use anchor_spl::token_2022::spl_token_2022::extension::transfer_fee::TransferFeeConfig;
use anchor_spl::token_2022::spl_token_2022::extension::transfer_hook::TransferHook;
use anchor_spl::token_2022::spl_token_2022::extension::{BaseStateWithExtensions, ExtensionType, StateWithExtensions};
use anchor_spl::token_2022::spl_token_2022::state::{Account as TokenAccountState, Mint as MintState};
use anchor_spl::token_interface::{self, InitializeAccount3, TransferChecked};

use crate::amm_core::mints::{SPL_TOKEN_ID, TOKEN_2022_ID};
use crate::amm_core::transfer_hook::append_hook_accounts;
use crate::errors::AmmError;
use crate::state::Pool;

///
pub const SPL_TOKEN_ACCOUNT_LEN: usize = 165;
///
///
const TOKEN_ACCOUNT_STATE_OFFSET: usize = 108;
const TOKEN_ACCOUNT_STATE_INITIALIZED: u8 = 1;

///
///
///
///
pub fn create_pda<'info>(
    payer: &AccountInfo<'info>,
    target: &AccountInfo<'info>,
    owner: &Pubkey,
    space: usize,
    signer_seeds: &[&[u8]],
) -> Result<()> {
    require_keys_eq!(*target.owner, system_program::ID, AmmError::SeedsMismatch);
    require!(target.data_is_empty(), AmmError::SeedsMismatch);

    let required = Rent::get()?.minimum_balance(space);
    let have = target.lamports();
    if have < required {
        lamports_move(
            CpiContext::new(system_program::ID, LamportsMove { from: payer.clone(), to: target.clone() }),
            required - have,
        )?;
    }
    let signers: &[&[&[u8]]] = &[signer_seeds];
    system_program::allocate(
        CpiContext::new_with_signer(system_program::ID, Allocate { account_to_allocate: target.clone() }, signers),
        space as u64,
    )?;
    system_program::assign(
        CpiContext::new_with_signer(system_program::ID, Assign { account_to_assign: target.clone() }, signers),
        owner,
    )?;
    Ok(())
}

///
///
pub fn token_account_len(mint: &AccountInfo) -> Result<usize> {
    if *mint.owner == SPL_TOKEN_ID {
        return Ok(SPL_TOKEN_ACCOUNT_LEN);
    }
    require_keys_eq!(*mint.owner, TOKEN_2022_ID, AmmError::MintNotSupported);
    let data = mint.try_borrow_data()?;
    let state = StateWithExtensions::<MintState>::unpack(&data).map_err(|_| AmmError::MintNotSupported)?;
    let mint_types = state.get_extension_types().map_err(|_| AmmError::MintNotSupported)?;
    let account_types = ExtensionType::get_required_init_account_extensions(&mint_types);
    let len = ExtensionType::try_calculate_account_len::<TokenAccountState>(&account_types)
        .map_err(|_| AmmError::MintNotSupported)?;
    Ok(len)
}

///
///
pub fn create_vault<'info>(
    payer: &AccountInfo<'info>,
    vault: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    pool: &AccountInfo<'info>,
    vault_seeds: &[&[u8]],
) -> Result<()> {
    require_keys_eq!(*mint.owner, token_program.key(), AmmError::TokenProgramMismatch);
    let len = token_account_len(mint)?;
    create_pda(payer, vault, &token_program.key(), len, vault_seeds)?;


    token_interface::initialize_account3(CpiContext::new(
        token_program.key(),
        InitializeAccount3 { account: vault.clone(), mint: mint.clone(), authority: pool.clone() },
    ))?;











    let data = vault.try_borrow_data()?;
    require!(data.len() > TOKEN_ACCOUNT_STATE_OFFSET, AmmError::VaultFrozenAtCreation);
    require!(data[TOKEN_ACCOUNT_STATE_OFFSET] == TOKEN_ACCOUNT_STATE_INITIALIZED, AmmError::VaultFrozenAtCreation);
    Ok(())
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
pub fn require_transfer_safe(mint: &AccountInfo, admitted: Option<Pubkey>) -> Result<Option<Pubkey>> {
    if *mint.owner == SPL_TOKEN_ID {
        return Ok(None);
    }
    require_keys_eq!(*mint.owner, TOKEN_2022_ID, AmmError::MintNotSupported);
    let data = mint.try_borrow_data()?;
    let state = StateWithExtensions::<MintState>::unpack(&data).map_err(|_| AmmError::MintNotSupported)?;



    let mut armed: Option<Pubkey> = None;
    if let Ok(h) = state.get_extension::<TransferHook>() {
        let program = Pubkey::new_from_array(h.program_id.0.to_bytes());
        if program != Pubkey::default() {
            let admitted = admitted.ok_or(AmmError::QuoteHookArmed)?;
            let renounced = h.authority.0.to_bytes() == [0u8; 32];
            require!(program == admitted && renounced, AmmError::TransferHookNotAdmitted);
            armed = Some(program);
        }
    }

    if let Ok(p) = state.get_extension::<PausableConfig>() {
        require!(!bool::from(p.paused), AmmError::QuotePaused);
    }










    if let Ok(f) = state.get_extension::<TransferFeeConfig>() {
        let inert = f.transfer_fee_config_authority.0.to_bytes() == [0u8; 32]
            && f.withdraw_withheld_authority.0.to_bytes() == [0u8; 32]
            && u16::from(f.older_transfer_fee.transfer_fee_basis_points) == 0
            && u16::from(f.newer_transfer_fee.transfer_fee_basis_points) == 0;
        require!(inert, AmmError::QuoteFeeArmed);
    }
    Ok(armed)
}

///
///
///
///
pub fn admitted_hook(pool: &Pool, side: u8) -> Option<Pubkey> {
    if pool.base_transfer_hook != Pubkey::default() && pool.base_hook_side == side {
        Some(pool.base_transfer_hook)
    } else {
        None
    }
}

///
pub fn cpi_guard_locked(token_account: &AccountInfo) -> Result<bool> {
    if *token_account.owner != TOKEN_2022_ID {
        return Ok(false);
    }
    let data = token_account.try_borrow_data()?;
    let state =
        StateWithExtensions::<TokenAccountState>::unpack(&data).map_err(|_| AmmError::MintMismatch)?;
    Ok(match state.get_extension::<CpiGuard>() {
        Ok(g) => bool::from(g.lock_cpi),
        Err(_) => false,
    })
}

///
pub struct TransferIn<'a, 'info> {
    pub token_program: &'a AccountInfo<'info>,
    pub source: &'a AccountInfo<'info>,
    pub mint: &'a AccountInfo<'info>,
    pub vault: &'a AccountInfo<'info>,
///
    pub owner: &'a AccountInfo<'info>,
///
    pub pool: &'a AccountInfo<'info>,
    pub pool_signer_seeds: &'a [&'a [u8]],
    pub decimals: u8,
///
    pub admitted: Option<Pubkey>,
///
///
    pub extra: &'a [AccountInfo<'info>],
}

///
///
///
pub fn transfer_checked_in(t: &TransferIn, amount: u64) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }

    let hooked = require_transfer_safe(t.mint, t.admitted)?;
    require_keys_eq!(*t.source.owner, t.token_program.key(), AmmError::TokenProgramMismatch);
    let guarded = cpi_guard_locked(t.source)?;
    if guarded {
        let (acct_owner, delegate, delegated) = {
            let data = t.source.try_borrow_data()?;
            let state = StateWithExtensions::<TokenAccountState>::unpack(&data)
                .map_err(|_| AmmError::DelegateMissing)?;
            (state.base.owner, state.base.delegate, state.base.delegated_amount)
        };
        require_keys_eq!(acct_owner, t.owner.key(), AmmError::DelegateMissing);
        require!(delegate == COption::Some(t.pool.key()), AmmError::DelegateMissing);
        require!(delegated >= amount, AmmError::DelegateMissing);
        if let Some(program) = hooked {
            let leg = HookedLeg { token_program: t.token_program, from: t.source, mint: t.mint, to: t.vault, authority: t.pool };
            return transfer_checked_hooked(&leg, Some(t.pool_signer_seeds), &program, t.extra, amount, t.decimals);
        }
        let signers: &[&[&[u8]]] = &[t.pool_signer_seeds];
        token_interface::transfer_checked(
            CpiContext::new_with_signer(
                t.token_program.key(),
                TransferChecked {
                    from: t.source.clone(),
                    mint: t.mint.clone(),
                    to: t.vault.clone(),
                    authority: t.pool.clone(),
                },
                signers,
            ),
            amount,
            t.decimals,
        )
    } else {
        if let Some(program) = hooked {
            let leg = HookedLeg { token_program: t.token_program, from: t.source, mint: t.mint, to: t.vault, authority: t.owner };
            return transfer_checked_hooked(&leg, None, &program, t.extra, amount, t.decimals);
        }
        token_interface::transfer_checked(
            CpiContext::new(
                t.token_program.key(),
                TransferChecked {
                    from: t.source.clone(),
                    mint: t.mint.clone(),
                    to: t.vault.clone(),
                    authority: t.owner.clone(),
                },
            ),
            amount,
            t.decimals,
        )
    }
}

///
///
#[allow(clippy::too_many_arguments)]
pub fn transfer_checked_out<'info>(
    token_program: &AccountInfo<'info>,
    vault: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    destination: &AccountInfo<'info>,
    pool: &AccountInfo<'info>,
    pool_signer_seeds: &[&[u8]],
    amount: u64,
    decimals: u8,
    admitted: Option<Pubkey>,
    extra: &[AccountInfo<'info>],
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }

    let hooked = require_transfer_safe(mint, admitted)?;
    if let Some(program) = hooked {
        let leg = HookedLeg { token_program, from: vault, mint, to: destination, authority: pool };
        return transfer_checked_hooked(&leg, Some(pool_signer_seeds), &program, extra, amount, decimals);
    }
    let signers: &[&[&[u8]]] = &[pool_signer_seeds];
    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            token_program.key(),
            TransferChecked { from: vault.clone(), mint: mint.clone(), to: destination.clone(), authority: pool.clone() },
            signers,
        ),
        amount,
        decimals,
    )
}

///
struct HookedLeg<'a, 'info> {
    token_program: &'a AccountInfo<'info>,
    from: &'a AccountInfo<'info>,
    mint: &'a AccountInfo<'info>,
    to: &'a AccountInfo<'info>,
///
///
    authority: &'a AccountInfo<'info>,
}

///
///
///
///
///
///
///
///
fn transfer_checked_hooked<'info>(
    leg: &HookedLeg<'_, 'info>,
    signer_seeds: Option<&[&[u8]]>,
    hook_program: &Pubkey,
    extra: &[AccountInfo<'info>],
    amount: u64,
    decimals: u8,
) -> Result<()> {
    let mut ix = anchor_spl::token_2022::spl_token_2022::instruction::transfer_checked(
        leg.token_program.key,
        leg.from.key,
        leg.mint.key,
        leg.to.key,
        leg.authority.key,
        &[],
        amount,
        decimals,
    )?;
    let mut infos = vec![leg.from.clone(), leg.mint.clone(), leg.to.clone(), leg.authority.clone()];
    append_hook_accounts(&mut ix.accounts, &mut infos, hook_program, leg.from, leg.mint, leg.to, leg.authority, amount, extra)?;
    match signer_seeds {
        Some(seeds) => invoke_signed(&ix, &infos, &[seeds])?,
        None => invoke(&ix, &infos)?,
    }
    Ok(())
}

///
///
pub fn token_amount(token_account: &AccountInfo) -> Result<u64> {
    let data = token_account.try_borrow_data()?;
    require!(data.len() >= 72, AmmError::VaultMismatch);
    Ok(u64::from_le_bytes(data[64..72].try_into().map_err(|_| AmmError::VaultMismatch)?))
}
