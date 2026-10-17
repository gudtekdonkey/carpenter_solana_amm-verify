//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!

use crate::constants::{EXECUTE_DISCRIMINATOR, HOLDER_BOOK_SEED, HOLDER_RECORD_SEED};

///
pub const META_LEN: usize = 35;
///
pub const HEADER_LEN: usize = 8 + 4 + 4;
///
pub const ENTRIES: usize = 4;
///
pub const LIST_LEN: usize = HEADER_LEN + ENTRIES * META_LEN;
const _: () = assert!(LIST_LEN == floor_types::holders::EXTRA_ACCOUNT_METAS_LEN);
const _: () = assert!(ENTRIES == floor_types::holders::EXTRA_ACCOUNT_METAS_ENTRIES);

///
const META_FIXED: u8 = 0;
///
const META_PDA_SELF: u8 = 1;
///
const SEED_LITERAL: u8 = 1;
const SEED_ACCOUNT_KEY: u8 = 3;

///
const IX_SOURCE: u8 = 0;
const IX_MINT: u8 = 1;
const IX_DESTINATION: u8 = 2;

///
pub const fn entry_offset(i: usize) -> usize {
    HEADER_LEN + i * META_LEN
}

fn fixed(key: &[u8; 32], writable: bool) -> [u8; META_LEN] {
    let mut m = [0u8; META_LEN];
    m[0] = META_FIXED;
    m[1..33].copy_from_slice(key);
    m[33] = 0;
    m[34] = writable as u8;
    m
}

///
fn pda_self(literal: &[u8], keys: &[u8], writable: bool) -> [u8; META_LEN] {
    let mut cfg = [0u8; 32];
    let mut at = 0;
    cfg[at] = SEED_LITERAL;
    cfg[at + 1] = literal.len() as u8;
    cfg[at + 2..at + 2 + literal.len()].copy_from_slice(literal);
    at += 2 + literal.len();
    for k in keys {
        cfg[at] = SEED_ACCOUNT_KEY;
        cfg[at + 1] = *k;
        at += 2;
    }
    let mut m = [0u8; META_LEN];
    m[0] = META_PDA_SELF;
    m[1..33].copy_from_slice(&cfg);
    m[33] = 0;
    m[34] = writable as u8;
    m
}


const _: () = assert!(2 + HOLDER_BOOK_SEED.len() + 2 <= 32);
const _: () = assert!(2 + HOLDER_RECORD_SEED.len() + 2 + 2 <= 32);

///
pub fn list_bytes(floor_pool: &[u8; 32]) -> [u8; LIST_LEN] {
    let mut out = [0u8; LIST_LEN];
    out[0..8].copy_from_slice(&EXECUTE_DISCRIMINATOR);
    out[8..12].copy_from_slice(&((4 + ENTRIES * META_LEN) as u32).to_le_bytes());
    out[12..16].copy_from_slice(&(ENTRIES as u32).to_le_bytes());
    let entries = [
        pda_self(HOLDER_BOOK_SEED, &[IX_MINT], true),
        fixed(floor_pool, false),
        pda_self(HOLDER_RECORD_SEED, &[IX_MINT, IX_SOURCE], true),
        pda_self(HOLDER_RECORD_SEED, &[IX_MINT, IX_DESTINATION], true),
    ];
    for (i, e) in entries.iter().enumerate() {
        out[entry_offset(i)..entry_offset(i) + META_LEN].copy_from_slice(e);
    }
    out
}

///
///
///
pub fn named_floor_pool(data: &[u8]) -> Option<[u8; 32]> {
    if data.len() != LIST_LEN || data[0..8] != EXECUTE_DISCRIMINATOR {
        return None;
    }
    let e = entry_offset(1);
    if data[e] != META_FIXED {
        return None;
    }
    let mut k = [0u8; 32];
    k.copy_from_slice(&data[e + 1..e + 33]);
    Some(k)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_list_is_156_bytes_under_the_execute_tlv() {
        let l = list_bytes(&[7u8; 32]);
        assert_eq!(l.len(), 156);
        assert_eq!(&l[0..8], &EXECUTE_DISCRIMINATOR);
        assert_eq!(u32::from_le_bytes(l[8..12].try_into().unwrap()), 144, "value = count + 4 × 35");
        assert_eq!(u32::from_le_bytes(l[12..16].try_into().unwrap()), 4);
    }

    #[test]
    fn the_execute_discriminator_is_the_interface_hash() {


        assert_eq!(EXECUTE_DISCRIMINATOR, [105, 37, 101, 197, 75, 251, 102, 26]);
    }

    #[test]
    fn each_entry_is_the_documented_shape() {
        let pool = [9u8; 32];
        let l = list_bytes(&pool);

        let e = &l[entry_offset(0)..entry_offset(0) + META_LEN];
        assert_eq!(e[0], 1);
        assert_eq!(e[1], 1);
        assert_eq!(e[2] as usize, b"holder_book".len());
        assert_eq!(&e[3..3 + 11], b"holder_book");
        assert_eq!(&e[14..16], &[3, 1]);
        assert!(e[16..33].iter().all(|b| *b == 0), "zero-padded");
        assert_eq!((e[33], e[34]), (0, 1));

        let e = &l[entry_offset(1)..entry_offset(1) + META_LEN];
        assert_eq!(e[0], 0);
        assert_eq!(&e[1..33], &pool);
        assert_eq!((e[33], e[34]), (0, 0));

        let e = &l[entry_offset(2)..entry_offset(2) + META_LEN];
        assert_eq!((e[0], e[1], e[2]), (1, 1, 6));
        assert_eq!(&e[3..9], b"holder");
        assert_eq!(&e[9..13], &[3, 1, 3, 0]);
        assert!(e[13..33].iter().all(|b| *b == 0));
        assert_eq!((e[33], e[34]), (0, 1));

        let e = &l[entry_offset(3)..entry_offset(3) + META_LEN];
        assert_eq!(&e[9..13], &[3, 1, 3, 2]);
        assert_eq!((e[33], e[34]), (0, 1));
    }

    #[test]
    fn named_floor_pool_reads_the_fixed_entry_and_nothing_else() {
        let pool = [5u8; 32];
        let l = list_bytes(&pool);
        assert_eq!(named_floor_pool(&l), Some(pool));
        assert_eq!(named_floor_pool(&l[..155]), None, "short");
        let mut bad = l;
        bad[0] ^= 1;
        assert_eq!(named_floor_pool(&bad), None, "another TLV type");
    }
}
