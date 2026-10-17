//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::{invoke, invoke_signed};
use anchor_spl::token_2022::spl_token_2022;
use anchor_spl::token_2022::spl_token_2022::extension::pausable::PausableConfig;
use anchor_spl::token_2022::spl_token_2022::extension::transfer_hook::TransferHook;
use anchor_spl::token_2022::spl_token_2022::extension::{BaseStateWithExtensions, ExtensionType, StateWithExtensions};
use anchor_spl::token_2022::spl_token_2022::instruction::AuthorityType;
use anchor_spl::token_interface::{pausable_initialize, pausable_pause, pausable_resume, PausableInitialize, PausableToggle};

use crate::errors::LaunchError;

///
pub const LOCK_AUTHORITY_SEED: &[u8] = floor_types::seeds::LOCK_AUTHORITY;

///
pub struct Lock<'a, 'info> {
///
    pub token_2022: &'a AccountInfo<'info>,
///
    pub mint: &'a AccountInfo<'info>,
///
    pub lock_authority: &'a AccountInfo<'info>,
///
    pub bump: u8,
}

impl<'a, 'info> Lock<'a, 'info> {
    fn toggle(&self) -> PausableToggle<'info> {
        PausableToggle {
            token_program_id: self.token_2022.clone(),
            mint: self.mint.clone(),
            authority: self.lock_authority.clone(),
        }
    }

    fn resume(&self) -> Result<()> {
        let bump = [self.bump];
        let seeds: &[&[u8]] = &[LOCK_AUTHORITY_SEED, &bump];
        pausable_resume(CpiContext::new_with_signer(self.token_2022.key(), self.toggle(), &[seeds]))
    }

    fn pause(&self) -> Result<()> {
        let bump = [self.bump];
        let seeds: &[&[u8]] = &[LOCK_AUTHORITY_SEED, &bump];
        pausable_pause(CpiContext::new_with_signer(self.token_2022.key(), self.toggle(), &[seeds]))
    }
}

///
///
pub fn install(lock: &Lock<'_, '_>) -> Result<()> {
    pausable_initialize(
        CpiContext::new(
            lock.token_2022.key(),
            PausableInitialize { token_program_id: lock.token_2022.clone(), mint: lock.mint.clone() },
        ),
        lock.lock_authority.key(),
    )
}

///
///
pub fn engage(lock: &Lock<'_, '_>) -> Result<()> {
    lock.pause()
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
pub fn around_transfer(lock: &Lock<'_, '_>, repause: bool, move_tokens: impl FnOnce() -> Result<()>) -> Result<()> {
    lock.resume()?;
    move_tokens()?;
    if repause {
        lock.pause()?;
    }
    Ok(())
}

///
///
pub fn lift(lock: &Lock<'_, '_>) -> Result<()> {
    lock.resume()?;
    let bump = [lock.bump];
    let seeds: &[&[u8]] = &[LOCK_AUTHORITY_SEED, &bump];
    anchor_spl::token_2022::set_authority(
        CpiContext::new_with_signer(
            lock.token_2022.key(),
            anchor_spl::token_2022::SetAuthority {
                current_authority: lock.lock_authority.clone(),
                account_or_mint: lock.mint.clone(),
            },
            &[seeds],
        ),
        AuthorityType::Pause,
        None,
    )?;
    let (paused, authority) = read(lock.mint)?;
    require!(!paused && authority.is_none(), LaunchError::LockStateInvalid);
    Ok(())
}

///
pub fn read(mint: &AccountInfo<'_>) -> Result<(bool, Option<Pubkey>)> {
    let data = mint.try_borrow_data()?;
    let state = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&data)?;
    let cfg = state.get_extension::<PausableConfig>()?;
    Ok((bool::from(cfg.paused), Option::<Pubkey>::from(cfg.authority)))
}

///
///
///
pub fn assert_state(mint: &AccountInfo<'_>, lock_authority: &Pubkey, bonding: bool) -> Result<()> {
    let (paused, authority) = read(mint)?;
    let want_authority = if bonding { Some(*lock_authority) } else { None };
    require!(paused == bonding && authority == want_authority, LaunchError::LockStateInvalid);
    Ok(())
}






















///
const T22_TRANSFER_HOOK_EXT: u8 = 36;
///
const TRANSFER_HOOK_INITIALIZE: u8 = 0;
const TRANSFER_HOOK_UPDATE: u8 = 1;

///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HookSlot {
///
    pub authority: Option<Pubkey>,
///
    pub program_id: Option<Pubkey>,
}

///
///
pub fn hook_slot(mint: &AccountInfo<'_>) -> Result<Option<HookSlot>> {
    let data = mint.try_borrow_data()?;
    let state = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&data)?;
    if !state.get_extension_types()?.contains(&ExtensionType::TransferHook) {
        return Ok(None);
    }
    let h = state.get_extension::<TransferHook>()?;
    Ok(Some(HookSlot { authority: Option::<Pubkey>::from(h.authority), program_id: Option::<Pubkey>::from(h.program_id) }))
}

///
///
///
///
pub fn install_hook_slot(lock: &Lock<'_, '_>) -> Result<()> {
    let mut data = vec![T22_TRANSFER_HOOK_EXT, TRANSFER_HOOK_INITIALIZE];
    data.extend_from_slice(lock.lock_authority.key.as_ref());
    data.extend_from_slice(&[0u8; 32]);
    let ix = Instruction {
        program_id: *lock.token_2022.key,
        accounts: vec![AccountMeta::new(*lock.mint.key, false)],
        data,
    };
    invoke(&ix, &[lock.mint.clone(), lock.token_2022.clone()])?;
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
pub fn arm_hook(lock: &Lock<'_, '_>, program: &Pubkey) -> Result<()> {
    let bump = [lock.bump];
    let seeds: &[&[u8]] = &[LOCK_AUTHORITY_SEED, &bump];
    let mut data = vec![T22_TRANSFER_HOOK_EXT, TRANSFER_HOOK_UPDATE];
    data.extend_from_slice(program.as_ref());
    let ix = Instruction {
        program_id: *lock.token_2022.key,
        accounts: vec![AccountMeta::new(*lock.mint.key, false), AccountMeta::new_readonly(*lock.lock_authority.key, true)],
        data,
    };
    invoke_signed(&ix, &[lock.mint.clone(), lock.lock_authority.clone(), lock.token_2022.clone()], &[seeds])?;
    anchor_spl::token_2022::set_authority(
        CpiContext::new_with_signer(
            lock.token_2022.key(),
            anchor_spl::token_2022::SetAuthority {
                current_authority: lock.lock_authority.clone(),
                account_or_mint: lock.mint.clone(),
            },
            &[seeds],
        ),
        AuthorityType::TransferHookProgramId,
        None,
    )?;
    let slot = hook_slot(lock.mint)?;
    require!(slot == Some(HookSlot { authority: None, program_id: Some(*program) }), LaunchError::HookNotArmed);
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
pub fn invoke_holders_init(lock: &Lock<'_, '_>, ix: &Instruction, infos: &[AccountInfo<'_>]) -> Result<()> {
    require_keys_eq!(ix.program_id, crate::constants::FLOOR_HOLDERS_PROGRAM, LaunchError::BadConfig);
    require!(ix.data.len() >= 8 && ix.data[..8] == crate::constants::holders_disc::INIT_LAUNCH, LaunchError::BadConfig);
    let bump = [lock.bump];
    let seeds: &[&[u8]] = &[LOCK_AUTHORITY_SEED, &bump];
    invoke_signed(ix, infos, &[seeds])?;
    Ok(())
}
