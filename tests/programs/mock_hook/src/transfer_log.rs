//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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

use crate::MockHookError;

///
pub const EXECUTE_DISCRIMINATOR: [u8; 8] = [105, 37, 101, 197, 75, 251, 102, 26];

///
pub const TRANSFER_LOG_TAG: [u8; 8] = *b"XFERLOG1";

///
pub const EXECUTE_ACCOUNTS: usize = 9;

///
///
///
///
///
pub const TRANSFER_LOG_LEN: usize = 256;

const CALLS: usize = 8;
const LAST_AMOUNT: usize = 16;
const TOTAL_AMOUNT: usize = 24;
const LAST_SOURCE: usize = 32;
const LAST_MINT: usize = 64;
const LAST_DESTINATION: usize = 96;
const LAST_AUTHORITY: usize = 128;
const LAST_SENDER_RECORD: usize = 160;
const LAST_RECEIVER_RECORD: usize = 192;
const LAST_ACCOUNT_COUNT: usize = 224;
const FAIL_NEXT: usize = 225;
const RELAY: usize = 226;
///
///
const NESTED_INNER_COUNT: usize = 227;
///
const NESTED_INNER_DATA: usize = 232;

///
pub fn execute(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> Result<()> {
    require!(data.len() >= 16 && data[..8] == EXECUTE_DISCRIMINATOR, MockHookError::NotExecute);
    let amount = u64::from_le_bytes(data[8..16].try_into().map_err(|_| MockHookError::NotExecute)?);
    require!(accounts.len() >= EXECUTE_ACCOUNTS, MockHookError::MissingExtraAccounts);
    let log = &accounts[5];
    require_keys_eq!(*log.owner, *program_id, MockHookError::NotALog);
    let mut d = log.try_borrow_mut_data()?;
    require!(d.len() >= TRANSFER_LOG_LEN && d[..8] == TRANSFER_LOG_TAG, MockHookError::NotALog);
    if d[FAIL_NEXT] != 0 {
        return err!(MockHookError::Refused);
    }
    if d[RELAY] == 2 {










        let inner_count = d[NESTED_INNER_COUNT] as usize;
        let mut inner_data = [0u8; 10];
        inner_data.copy_from_slice(&d[NESTED_INNER_DATA..NESTED_INNER_DATA + 10]);
        drop(d);
        require!(accounts.len() >= 8 + inner_count, MockHookError::MissingExtraAccounts);

        {
            let target = accounts[6].key;
            let inner = &accounts[8..8 + inner_count];
            let mut metas = Vec::with_capacity(inner.len());
            let mut infos = Vec::with_capacity(inner.len());
            for a in inner {
                metas.push(if a.is_writable {
                    anchor_lang::solana_program::instruction::AccountMeta::new(*a.key, a.is_signer)
                } else {
                    anchor_lang::solana_program::instruction::AccountMeta::new_readonly(*a.key, a.is_signer)
                });
                infos.push(a.clone());
            }
            infos.push(accounts[6].clone());
            let ix = anchor_lang::solana_program::instruction::Instruction { program_id: *target, accounts: metas, data: inner_data.to_vec() };
            anchor_lang::solana_program::program::invoke(&ix, &infos)?;
        }

        let target = accounts[7].key;
        let exec = &accounts[8 + inner_count..];
        let mut metas = Vec::with_capacity(exec.len());
        let mut infos = Vec::with_capacity(exec.len());
        for a in exec {
            metas.push(if a.is_writable {
                anchor_lang::solana_program::instruction::AccountMeta::new(*a.key, false)
            } else {
                anchor_lang::solana_program::instruction::AccountMeta::new_readonly(*a.key, false)
            });
            infos.push(a.clone());
        }
        infos.push(accounts[7].clone());
        let mut data = EXECUTE_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&u64::MAX.to_le_bytes());
        let ix = anchor_lang::solana_program::instruction::Instruction { program_id: *target, accounts: metas, data };
        anchor_lang::solana_program::program::invoke(&ix, &infos)?;
        return Ok(());
    }
    if d[RELAY] != 0 {


        drop(d);
        require!(accounts.len() >= 8, MockHookError::MissingExtraAccounts);
        let target = accounts[6].key;
        let mut metas = Vec::with_capacity(accounts.len() - 6);
        let mut infos = Vec::with_capacity(accounts.len() - 5);
        for a in core::iter::once(&accounts[0]).chain(accounts[7..].iter()) {
            metas.push(if a.is_writable {
                anchor_lang::solana_program::instruction::AccountMeta::new(*a.key, false)
            } else {
                anchor_lang::solana_program::instruction::AccountMeta::new_readonly(*a.key, false)
            });
            infos.push(a.clone());
        }
        infos.push(accounts[6].clone());
        let mut data = EXECUTE_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&u64::MAX.to_le_bytes());
        let ix = anchor_lang::solana_program::instruction::Instruction { program_id: *target, accounts: metas, data };
        anchor_lang::solana_program::program::invoke(&ix, &infos)?;
        return Ok(());
    }
    let read = |d: &[u8], at: usize| u64::from_le_bytes(d[at..at + 8].try_into().unwrap_or([0u8; 8]));
    let calls = read(&d, CALLS).saturating_add(1);
    let total = read(&d, TOTAL_AMOUNT).saturating_add(amount);
    d[CALLS..CALLS + 8].copy_from_slice(&calls.to_le_bytes());
    d[LAST_AMOUNT..LAST_AMOUNT + 8].copy_from_slice(&amount.to_le_bytes());
    d[TOTAL_AMOUNT..TOTAL_AMOUNT + 8].copy_from_slice(&total.to_le_bytes());
    for (at, i) in [
        (LAST_SOURCE, 0usize),
        (LAST_MINT, 1),
        (LAST_DESTINATION, 2),
        (LAST_AUTHORITY, 3),
        (LAST_SENDER_RECORD, 7),
        (LAST_RECEIVER_RECORD, 8),
    ] {
        d[at..at + 32].copy_from_slice(accounts[i].key.as_ref());
    }
    d[LAST_ACCOUNT_COUNT] = accounts.len().min(255) as u8;
    Ok(())
}
