//!
//!
//!
//!
//!
//!
//!
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
use anchor_spl::token_2022::spl_token_2022;
use anchor_spl::token_2022::spl_token_2022::extension::transfer_hook::TransferHookAccount;
use anchor_spl::token_2022::spl_token_2022::extension::{BaseStateWithExtensions, StateWithExtensions};

use crate::constants::TOKEN_2022_PROGRAM;

///
pub const ATA_PROGRAM: Pubkey = Pubkey::new_from_array(floor_types::keys::ASSOCIATED_TOKEN_PROGRAM_ID);

///
///
const TA_MINT: usize = 0;
const TA_OWNER: usize = 32;
const TA_AMOUNT: usize = 64;
const TA_STATE: usize = 108;
const TA_BASE_LEN: usize = 165;

///
pub fn token_2022_ata(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[owner.as_ref(), TOKEN_2022_PROGRAM.as_ref(), mint.as_ref()], &ATA_PROGRAM).0
}

///


#[allow(deprecated)]
pub fn is_on_curve(key: &Pubkey) -> bool {
    solana_curve25519::edwards::validate_edwards(&solana_curve25519::edwards::PodEdwardsPoint(key.to_bytes()))
}

///
pub const INCINERATOR: Pubkey = pubkey!("1nc1nerator11111111111111111111111111111111");

///
///
///
///
pub fn is_dead_owner(key: &Pubkey) -> bool {
    *key == Pubkey::default() || *key == INCINERATOR
}

///
///
pub fn read_token_2022_account(info: &AccountInfo) -> Option<(Pubkey, Pubkey, u64)> {
    if *info.owner != TOKEN_2022_PROGRAM {
        return None;
    }
    let d = info.try_borrow_data().ok()?;
    if d.len() < TA_BASE_LEN || d[TA_STATE] == 0 {
        return None;
    }
    let mut mint = [0u8; 32];
    mint.copy_from_slice(&d[TA_MINT..TA_MINT + 32]);
    let mut owner = [0u8; 32];
    owner.copy_from_slice(&d[TA_OWNER..TA_OWNER + 32]);
    let mut amount = [0u8; 8];
    amount.copy_from_slice(&d[TA_AMOUNT..TA_AMOUNT + 8]);
    Some((Pubkey::new_from_array(mint), Pubkey::new_from_array(owner), u64::from_le_bytes(amount)))
}

///
///
///
///
pub fn live_balance(info: &AccountInfo, mint: &Pubkey) -> u64 {
    match read_token_2022_account(info) {
        Some((m, _, amount)) if m == *mint => amount,
        _ => 0,
    }
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
pub fn is_transferring_of(info: &AccountInfo, mint: &Pubkey) -> bool {
    if *info.owner != TOKEN_2022_PROGRAM {
        return false;
    }
    let Ok(d) = info.try_borrow_data() else { return false };
    let Ok(state) = StateWithExtensions::<spl_token_2022::state::Account>::unpack(&d) else { return false };
    if state.base.mint != *mint {
        return false;
    }
    match state.get_extension::<TransferHookAccount>() {
        Ok(ext) => bool::from(ext.transferring),
        Err(_) => false,
    }
}
