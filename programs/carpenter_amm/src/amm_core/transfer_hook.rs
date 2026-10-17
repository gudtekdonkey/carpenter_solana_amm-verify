//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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
use anchor_lang::solana_program::instruction::AccountMeta;

use crate::errors::AmmError;

///
///
pub const EXECUTE_DISCRIMINATOR: [u8; 8] = [105, 37, 101, 197, 75, 251, 102, 26];

///
pub const EXTRA_ACCOUNT_METAS_SEED: &[u8] = b"extra-account-metas";

///
pub const EXTRA_ACCOUNT_META_LEN: usize = 35;

///
const META_LITERAL: u8 = 0;
const META_HOOK_PDA: u8 = 1;
const META_EXTERNAL_PDA_BASE: u8 = 128;

///
///
const EXECUTE_FIXED_ACCOUNTS: usize = 5;

///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawMeta {
    pub discriminator: u8,
    pub address_config: [u8; 32],
    pub is_signer: bool,
    pub is_writable: bool,
}

///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Seed<'a> {
    Literal(&'a [u8]),
    InstructionData { index: u8, length: u8 },
    AccountKey { index: u8 },
    AccountData { account_index: u8, data_index: u8, length: u8 },
}

///
///
pub fn execute_metas(data: &[u8]) -> Option<Vec<RawMeta>> {
    let mut at = 0usize;
    while at < data.len() {
        let rest = &data[at..];
        if rest.len() < 12 {

            return None;
        }
        let disc: [u8; 8] = rest[..8].try_into().ok()?;
        if disc == [0u8; 8] {
            return None;
        }
        let len = u32::from_le_bytes(rest[8..12].try_into().ok()?) as usize;
        let value = rest.get(12..12usize.checked_add(len)?)?;
        if disc == EXECUTE_DISCRIMINATOR {
            let count = u32::from_le_bytes(value.get(..4)?.try_into().ok()?) as usize;
            let body = value.get(4..4usize.checked_add(count.checked_mul(EXTRA_ACCOUNT_META_LEN)?)?)?;
            let mut out = Vec::with_capacity(count);
            for m in body.chunks_exact(EXTRA_ACCOUNT_META_LEN) {
                out.push(RawMeta {
                    discriminator: m[0],
                    address_config: m[1..33].try_into().ok()?,
                    is_signer: m[33] != 0,
                    is_writable: m[34] != 0,
                });
            }
            return Some(out);
        }
        at = at.checked_add(12)?.checked_add(len)?;
    }
    None
}

///
///
pub fn unpack_seeds(config: &[u8; 32]) -> Option<Vec<Seed<'_>>> {
    let mut seeds = Vec::new();
    let mut i = 0usize;
    while i < config.len() {
        match config[i] {
            0 => break,
            1 => {
                let len = *config.get(i + 1)? as usize;
                let bytes = config.get(i + 2..i + 2 + len)?;
                seeds.push(Seed::Literal(bytes));
                i += 2 + len;
            }
            2 => {
                seeds.push(Seed::InstructionData { index: *config.get(i + 1)?, length: *config.get(i + 2)? });
                i += 3;
            }
            3 => {
                seeds.push(Seed::AccountKey { index: *config.get(i + 1)? });
                i += 2;
            }
            4 => {
                seeds.push(Seed::AccountData {
                    account_index: *config.get(i + 1)?,
                    data_index: *config.get(i + 2)?,
                    length: *config.get(i + 3)?,
                });
                i += 4;
            }
            _ => return None,
        }
    }
    Some(seeds)
}

///
pub fn execute_data(amount: u64) -> [u8; 16] {
    let mut d = [0u8; 16];
    d[..8].copy_from_slice(&EXECUTE_DISCRIMINATOR);
    d[8..].copy_from_slice(&amount.to_le_bytes());
    d
}

///
///
pub fn resolve_address(
    meta: &RawMeta,
    ix_data: &[u8],
    hook_program: &Pubkey,
    key_of: &dyn Fn(usize) -> Option<Pubkey>,
    data_of: &dyn Fn(usize, usize, usize) -> Option<Vec<u8>>,
) -> Option<Pubkey> {
    let program = match meta.discriminator {
        META_LITERAL => return Some(Pubkey::new_from_array(meta.address_config)),
        META_HOOK_PDA => *hook_program,
        d if d >= META_EXTERNAL_PDA_BASE => key_of((d - META_EXTERNAL_PDA_BASE) as usize)?,

        _ => return None,
    };
    let seeds = unpack_seeds(&meta.address_config)?;
    let mut owned: Vec<Vec<u8>> = Vec::with_capacity(seeds.len());
    for s in seeds {
        owned.push(match s {
            Seed::Literal(b) => b.to_vec(),
            Seed::InstructionData { index, length } => {
                let start = index as usize;
                ix_data.get(start..start + length as usize)?.to_vec()
            }
            Seed::AccountKey { index } => key_of(index as usize)?.to_bytes().to_vec(),
            Seed::AccountData { account_index, data_index, length } => {
                data_of(account_index as usize, data_index as usize, length as usize)?
            }
        });
    }
    let refs: Vec<&[u8]> = owned.iter().map(|v| v.as_slice()).collect();
    Pubkey::try_find_program_address(&refs, &program).map(|(k, _)| k)
}

///
///
///
///
///
#[allow(clippy::too_many_arguments)]
pub fn append_hook_accounts<'info>(
    cpi_metas: &mut Vec<AccountMeta>,
    cpi_infos: &mut Vec<AccountInfo<'info>>,
    hook_program: &Pubkey,
    source: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    destination: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    amount: u64,
    extra: &[AccountInfo<'info>],
) -> Result<()> {


    let own = [source, mint, destination, authority];
    let find = |key: &Pubkey| extra.iter().find(|a| a.key == key).or_else(|| own.iter().copied().find(|a| a.key == key)).cloned();
    let (validation, _) =
        Pubkey::find_program_address(&[EXTRA_ACCOUNT_METAS_SEED, mint.key.as_ref()], hook_program);
    let program_info = find(hook_program).ok_or(AmmError::TransferHookAccountsMissing)?;
    let validation_info = find(&validation).ok_or(AmmError::TransferHookAccountsMissing)?;

    require_keys_eq!(*validation_info.owner, *hook_program, AmmError::TransferHookAccountsMissing);
    let metas = {
        let data = validation_info.try_borrow_data()?;
        execute_metas(&data).ok_or(AmmError::TransferHookAccountsMissing)?
    };



    let ix_data = execute_data(amount);
    let mut exec_metas: Vec<AccountMeta> = vec![
        AccountMeta::new_readonly(*source.key, false),
        AccountMeta::new_readonly(*mint.key, false),
        AccountMeta::new_readonly(*destination.key, false),
        AccountMeta::new_readonly(*authority.key, false),
        AccountMeta::new_readonly(validation, false),
    ];
    let mut exec_infos: Vec<AccountInfo<'info>> =
        vec![source.clone(), mint.clone(), destination.clone(), authority.clone(), validation_info.clone()];

    for m in &metas {
        let key = {
            let key_of = |i: usize| exec_infos.get(i).map(|a| *a.key);
            let data_of = |i: usize, start: usize, len: usize| {
                let a = exec_infos.get(i)?;
                let d = a.try_borrow_data().ok()?;
                d.get(start..start.checked_add(len)?).map(|s| s.to_vec())
            };
            resolve_address(m, &ix_data, hook_program, &key_of, &data_of).ok_or(AmmError::TransferHookAccountsMissing)?
        };
        let mut meta = AccountMeta { pubkey: key, is_signer: m.is_signer, is_writable: m.is_writable };

        let mut seen = false;
        let (mut signer, mut writable) = (false, false);
        for e in exec_metas.iter().filter(|e| e.pubkey == key) {
            seen = true;
            signer |= e.is_signer;
            writable |= e.is_writable;
        }
        if seen {
            meta.is_signer &= signer;
            meta.is_writable &= writable;
        }
        let info = find(&key).ok_or(AmmError::TransferHookAccountsMissing)?;
        exec_metas.push(meta);
        exec_infos.push(info);
    }

    cpi_metas.extend_from_slice(&exec_metas[EXECUTE_FIXED_ACCOUNTS..]);
    cpi_infos.extend_from_slice(&exec_infos[EXECUTE_FIXED_ACCOUNTS..]);
    cpi_metas.push(AccountMeta::new_readonly(validation, false));
    cpi_infos.push(validation_info);
    cpi_metas.push(AccountMeta::new_readonly(*hook_program, false));
    cpi_infos.push(program_info);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

///
///
    fn list(entries: &[RawMeta]) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&EXECUTE_DISCRIMINATOR);
        v.extend_from_slice(&((4 + entries.len() * EXTRA_ACCOUNT_META_LEN) as u32).to_le_bytes());
        v.extend_from_slice(&(entries.len() as u32).to_le_bytes());
        for e in entries {
            v.push(e.discriminator);
            v.extend_from_slice(&e.address_config);
            v.push(e.is_signer as u8);
            v.push(e.is_writable as u8);
        }
        v
    }

    fn seeds(parts: &[&[u8]]) -> [u8; 32] {
        let mut c = [0u8; 32];
        let mut i = 0;
        for p in parts {
            c[i..i + p.len()].copy_from_slice(p);
            i += p.len();
        }
        c
    }

///
///
    fn floor_holders_list(floor_pool: Pubkey) -> Vec<RawMeta> {
        let book = seeds(&[&[1, 11], b"holder_book", &[3, 1]]);
        let sender = seeds(&[&[1, 6], b"holder", &[3, 1], &[3, 0]]);
        let receiver = seeds(&[&[1, 6], b"holder", &[3, 1], &[3, 2]]);
        vec![
            RawMeta { discriminator: 1, address_config: book, is_signer: false, is_writable: true },
            RawMeta { discriminator: 0, address_config: floor_pool.to_bytes(), is_signer: false, is_writable: false },
            RawMeta { discriminator: 1, address_config: sender, is_signer: false, is_writable: true },
            RawMeta { discriminator: 1, address_config: receiver, is_signer: false, is_writable: true },
        ]
    }

    #[test]
    fn the_execute_discriminator_is_the_interface_s_hash() {
        let h = solana_sha256_hasher::hash(b"spl-transfer-hook-interface:execute");
        assert_eq!(&h.to_bytes()[..8], &EXECUTE_DISCRIMINATOR);
        let d = execute_data(0x0102_0304_0506_0708);
        assert_eq!(&d[..8], &EXECUTE_DISCRIMINATOR);
        assert_eq!(&d[8..], &0x0102_0304_0506_0708u64.to_le_bytes());
    }

    #[test]
    fn floor_holders_list_is_156_bytes_and_reads_back() {
        let fp = Pubkey::new_from_array([33; 32]);
        let entries = floor_holders_list(fp);
        let data = list(&entries);
        assert_eq!(data.len(), 156, "build spec §1.2: the four-entry list is 156 B");
        assert_eq!(execute_metas(&data), Some(entries));
    }

    #[test]
    fn the_tlv_walk_skips_other_entries_and_stops_at_zeros() {
        let fp = Pubkey::new_from_array([34; 32]);
        let entries = floor_holders_list(fp);

        let mut data = vec![9u8; 8];
        data.extend_from_slice(&3u32.to_le_bytes());
        data.extend_from_slice(&[1, 2, 3]);
        data.extend_from_slice(&list(&entries));
        data.extend_from_slice(&[0u8; 40]);
        assert_eq!(execute_metas(&data), Some(entries.clone()));

        let mut other = vec![9u8; 8];
        other.extend_from_slice(&0u32.to_le_bytes());
        assert_eq!(execute_metas(&other), None);
        assert_eq!(execute_metas(&[]), None);
        assert_eq!(execute_metas(&[0u8; 64]), None, "an uninitialised list");

        let mut short = list(&entries);
        short.truncate(100);
        assert_eq!(execute_metas(&short), None);
        let mut lying = list(&entries);
        lying[12..16].copy_from_slice(&5u32.to_le_bytes());
        assert_eq!(execute_metas(&lying), None);
    }

    #[test]
    fn every_seed_kind_unpacks_and_a_bad_one_is_refused() {
        let c = seeds(&[&[1, 3], b"abc", &[2, 8, 8], &[3, 2], &[4, 5, 0, 32]]);
        assert_eq!(
            unpack_seeds(&c),
            Some(vec![
                Seed::Literal(b"abc"),
                Seed::InstructionData { index: 8, length: 8 },
                Seed::AccountKey { index: 2 },
                Seed::AccountData { account_index: 5, data_index: 0, length: 32 },
            ])
        );
        assert_eq!(unpack_seeds(&[0u8; 32]), Some(vec![]));
        assert_eq!(unpack_seeds(&seeds(&[&[7]])), None, "an unknown seed kind");

        let mut over = [b'x'; 32];
        over[0] = 1;
        over[1] = 28;
        over[30] = 1;
        over[31] = 5;
        assert_eq!(unpack_seeds(&over), None, "a literal running past the 32 bytes");
    }

    #[test]
    fn floor_holders_entries_resolve_to_its_pdas() {
        let holders = Pubkey::new_from_array([35; 32]);
        let (source, mint, dest, auth) = (Pubkey::new_from_array([36; 32]), Pubkey::new_from_array([37; 32]), Pubkey::new_from_array([38; 32]), Pubkey::new_from_array([39; 32]));
        let fp = Pubkey::new_from_array([40; 32]);
        let keys = [source, mint, dest, auth];
        let key_of = |i: usize| keys.get(i).copied();
        let data_of = |_: usize, _: usize, _: usize| -> Option<Vec<u8>> { None };
        let data = execute_data(7);
        let entries = floor_holders_list(fp);
        let got: Vec<Pubkey> =
            entries.iter().map(|m| resolve_address(m, &data, &holders, &key_of, &data_of).unwrap()).collect();
        let pda = |s: &[&[u8]]| Pubkey::find_program_address(s, &holders).0;
        assert_eq!(got[0], pda(&[b"holder_book", mint.as_ref()]));
        assert_eq!(got[1], fp);
        assert_eq!(got[2], pda(&[b"holder", mint.as_ref(), source.as_ref()]), "the SENDER record keys on the source token account");
        assert_eq!(got[3], pda(&[b"holder", mint.as_ref(), dest.as_ref()]), "the RECEIVER record keys on the destination");
    }

    #[test]
    fn instruction_data_account_data_and_external_pdas_resolve() {
        let hook = Pubkey::new_from_array([41; 32]);
        let other_program = Pubkey::new_from_array([42; 32]);
        let keys = [Pubkey::new_from_array([43; 32]), Pubkey::new_from_array([44; 32]), Pubkey::new_from_array([45; 32]), Pubkey::new_from_array([46; 32]), Pubkey::new_from_array([47; 32]), other_program];
        let key_of = |i: usize| keys.get(i).copied();
        let blob = [42u8; 64];
        let data_of = |i: usize, start: usize, len: usize| if i == 4 { blob.get(start..start + len).map(|s| s.to_vec()) } else { None };
        let ixd = execute_data(0xAB);

        let m = RawMeta { discriminator: 1, address_config: seeds(&[&[2, 8, 8], &[4, 4, 0, 8]]), is_signer: false, is_writable: false };
        let want = Pubkey::find_program_address(&[&ixd[8..16], &blob[..8]], &hook).0;
        assert_eq!(resolve_address(&m, &ixd, &hook, &key_of, &data_of), Some(want));

        let ext = RawMeta { discriminator: 128 + 5, ..m };
        let want = Pubkey::find_program_address(&[&ixd[8..16], &blob[..8]], &other_program).0;
        assert_eq!(resolve_address(&ext, &ixd, &hook, &key_of, &data_of), Some(want));

        let far = RawMeta { discriminator: 1, address_config: seeds(&[&[3, 9]]), is_signer: false, is_writable: false };
        assert_eq!(resolve_address(&far, &ixd, &hook, &key_of, &data_of), None);
        let past = RawMeta { discriminator: 1, address_config: seeds(&[&[2, 12, 8]]), is_signer: false, is_writable: false };
        assert_eq!(resolve_address(&past, &ixd, &hook, &key_of, &data_of), None);
        let unreadable = RawMeta { discriminator: 1, address_config: seeds(&[&[4, 1, 0, 8]]), is_signer: false, is_writable: false };
        assert_eq!(resolve_address(&unreadable, &ixd, &hook, &key_of, &data_of), None);
    }

    #[test]
    fn pubkey_data_and_unknown_kinds_are_refused_never_guessed() {
        let hook = Pubkey::new_from_array([48; 32]);
        let key_of = |_: usize| -> Option<Pubkey> { None };
        let data_of = |_: usize, _: usize, _: usize| -> Option<Vec<u8>> { None };
        for d in [2u8, 3, 127] {
            let m = RawMeta { discriminator: d, address_config: [1u8; 32], is_signer: false, is_writable: false };
            assert_eq!(resolve_address(&m, &execute_data(0), &hook, &key_of, &data_of), None, "discriminator {d}");
        }

        let lit = Pubkey::new_from_array([49; 32]);
        let m = RawMeta { discriminator: 0, address_config: lit.to_bytes(), is_signer: false, is_writable: true };
        assert_eq!(resolve_address(&m, &execute_data(0), &hook, &key_of, &data_of), Some(lit));
    }
}
