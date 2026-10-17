//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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
use anchor_lang::system_program::{self, transfer as lamports_move, Allocate, Assign, Transfer as LamportsMove};

use crate::errors::HoldersError;

///
pub fn is_absent(info: &AccountInfo) -> bool {
    *info.owner == system_program::ID && info.data_is_empty()
}

///
///
///
pub fn create_pda<'info>(
    payer: &AccountInfo<'info>,
    target: &AccountInfo<'info>,
    space: usize,
    signer_seeds: &[&[u8]],
    exists: anchor_lang::error::Error,
) -> Result<()> {
    if !is_absent(target) {
        return Err(exists);
    }
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
        &crate::ID,
    )?;
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
pub fn close_to(info: &AccountInfo, to: &AccountInfo) -> Result<u64> {
    {
        let mut data = info.try_borrow_mut_data()?;
        data.fill(0);
    }
    info.resize(0)?;
    info.assign(&system_program::ID);
    let lamports = info.lamports();
    **info.try_borrow_mut_lamports()? = 0;
    let credited = to.lamports().checked_add(lamports).ok_or(HoldersError::MathOverflow)?;
    **to.try_borrow_mut_lamports()? = credited;
    Ok(lamports)
}
