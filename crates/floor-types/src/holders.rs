//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!

use crate::disc::account as disc;
use crate::layout::FloorPool;
use crate::pod::U128;
use bytemuck::{Pod, Zeroable};
use carpenter_types::layout;





///
///
///
pub use crate::ids::FLOOR_HOLDERS_ID;

///
///
pub const ADDRESS_LOOKUP_TABLE_PROGRAM_ID: [u8; 32] = [
    2, 119, 166, 175, 151, 51, 155, 122, 200, 141, 24, 146, 201, 4, 70, 245, 0, 2, 48, 146, 102, 246, 46, 83, 193, 24,
    36, 73, 130, 0, 0, 0,
];





///
///
pub const HOLDERS_AUTHORITY: &[u8] = b"holders_authority";
///
pub const CLAIMS_ALT_AUTHORITY: &[u8] = b"claims_alt_authority";
///
pub const CLAIMS_REGISTRY: &[u8] = b"claims_registry";
///
pub const HOLDER_BOOK: &[u8] = b"holder_book";
///
///
///
pub const EXTRA_ACCOUNT_METAS: &[u8] = b"extra-account-metas";
///
///
///
pub const HOLDER_RECORD: &[u8] = b"holder";





///
///
///
pub const FLOOR_POOL_ACCT_HOLDER_OWED: usize = 440;
///
///
pub const FLOOR_POOL_ACCT_HOLDER_PAID: usize = 520;
///
pub const FLOOR_POOL_ACCT_REFERRER: usize = 648;


///
///
pub const FLOOR_POOL_ACCT_HOLDERS_HOOKED: usize = 681;
///
///
///
pub const FLOOR_POOL_ACCT_REFERRER_T2: usize = 688;
///
///
///
///
///
pub const FLOOR_POOL_ACCT_TEAM_FEE_BPS: usize = 784;



///
///
///
///
pub const FLOOR_POOL_ACCT_SKIM_SELL_AT_FLOOR_BPS: usize = 542;
///
///
///
///
pub const FLOOR_POOL_ACCT_STAIR_STEPS: usize = 642;
pub const FLOOR_POOL_ACCT_STAIR_READY: usize = 643;
pub const FLOOR_POOL_ACCT_STAIR_GROWTH_BPS: usize = 644;
///
pub const FLOOR_POOL_ACCT_STAIR_KIND: usize = 646;
///
///
pub const FLOOR_POOL_ACCT_STAIR_OWNER: usize = 8 + core::mem::offset_of!(crate::layout::FloorPool, stair_owner);
const _: () = assert!(FLOOR_POOL_ACCT_STAIR_OWNER == 647);






const _: () = {
    assert!(core::mem::offset_of!(FloorPool, holder_owed) + 8 == FLOOR_POOL_ACCT_HOLDER_OWED);
    assert!(core::mem::offset_of!(FloorPool, holder_paid_eth) + 8 == FLOOR_POOL_ACCT_HOLDER_PAID);
    assert!(core::mem::offset_of!(FloorPool, referrer) + 8 == FLOOR_POOL_ACCT_REFERRER);
    assert!(core::mem::offset_of!(FloorPool, referrer_t2) + 8 == FLOOR_POOL_ACCT_REFERRER_T2);
    assert!(core::mem::offset_of!(FloorPool, referrer_t3) + 8 == FLOOR_POOL_ACCT_REFERRER_T2 + 32);
    assert!(core::mem::offset_of!(FloorPool, team_fee_bps) + 8 == FLOOR_POOL_ACCT_TEAM_FEE_BPS);
    assert!(core::mem::offset_of!(FloorPool, holders_hooked) + 8 == FLOOR_POOL_ACCT_HOLDERS_HOOKED);
    assert!(core::mem::offset_of!(FloorPool, skim_sell_at_floor_bps) + 8 == FLOOR_POOL_ACCT_SKIM_SELL_AT_FLOOR_BPS);
    assert!(core::mem::offset_of!(FloorPool, stair_steps) + 8 == FLOOR_POOL_ACCT_STAIR_STEPS);
    assert!(core::mem::offset_of!(FloorPool, stair_ready) + 8 == FLOOR_POOL_ACCT_STAIR_READY);
    assert!(core::mem::offset_of!(FloorPool, stair_growth_bps) + 8 == FLOOR_POOL_ACCT_STAIR_GROWTH_BPS);
    assert!(core::mem::offset_of!(FloorPool, stair_kind) + 8 == FLOOR_POOL_ACCT_STAIR_KIND);
};

///
///
///
///
pub fn lifetime_from_floor_pool_account(data: &[u8]) -> Option<u64> {
    let owed = u64::from_le_bytes(data.get(FLOOR_POOL_ACCT_HOLDER_OWED..FLOOR_POOL_ACCT_HOLDER_OWED + 8)?.try_into().ok()?);
    let paid = u64::from_le_bytes(data.get(FLOOR_POOL_ACCT_HOLDER_PAID..FLOOR_POOL_ACCT_HOLDER_PAID + 8)?.try_into().ok()?);
    Some(owed.saturating_add(paid))
}





///
pub const CLAIMS_TABLE_MAX: u16 = 256;
///
pub const CLAIMS_ADDRS_HOOKED: u8 = 5;
///
pub const CLAIMS_ADDRS_REFERRED: u8 = 3;
///
///
///
pub const CLAIMS_TABLE_ROTATE_AT: u16 = CLAIMS_TABLE_MAX - CLAIMS_ADDRS_HOOKED as u16;





///
///
pub const EXTRA_ACCOUNT_METAS_ENTRIES: usize = 4;
///
pub const EXTRA_ACCOUNT_METAS_LEN: usize = 8 + 4 + 4 + 35 * EXTRA_ACCOUNT_METAS_ENTRIES;
///
pub const EXECUTE_ACCOUNTS: usize = 5 + EXTRA_ACCOUNT_METAS_ENTRIES;

///
///
pub const TOKEN_ACCOUNT_LEN_UNHOOKED: usize = 174;
///
pub const TOKEN_ACCOUNT_LEN_HOOKED: usize = 179;
///
pub const AMM_BASE_VAULT_LEN_UNHOOKED: usize = 170;
///
pub const AMM_BASE_VAULT_LEN_HOOKED: usize = 175;





///
///
///
///
///
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct HolderBook {
///
    pub version: u8,
    pub bump: u8,
///
    pub metas_bump: u8,
    pub pad0: [u8; 5],
    pub mint: [u8; 32],
///
    pub floor_pool: [u8; 32],
    pub quote_mint: [u8; 32],
///
    pub acc: U128,
///
    pub eligible: u64,
///
    pub indexed: u64,
///
///
    pub unallocated: u64,
///
    pub paid: u64,
///
    pub records: u64,
    pub created_slot: u64,
///
///
    pub pending: u64,
///
    pub last_slot: u64,
    pub reserved: [u8; 16],
}
layout!(HolderBook, name = "HolderBook", size = 200, disc = disc::HOLDER_BOOK, {
    version: u8 = 0, bump: u8 = 1, metas_bump: u8 = 2, pad0: [u8; 5] = 3, mint: [u8; 32] = 8,
    floor_pool: [u8; 32] = 40, quote_mint: [u8; 32] = 72, acc: U128 = 104, eligible: u64 = 120,
    indexed: u64 = 128, unallocated: u64 = 136, paid: u64 = 144, records: u64 = 152,
    created_slot: u64 = 160, pending: u64 = 168, last_slot: u64 = 176, reserved: [u8; 16] = 184,
});


///
///
///
///
///
///
///
///
///
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct HolderRecord {
///
    pub version: u8,
    pub bump: u8,
    pub pad0: [u8; 6],
    pub mint: [u8; 32],
    pub token_account: [u8; 32],
///
///
    pub owner: [u8; 32],
    pub shares: u64,
///
    pub snapshot: U128,
    pub owed: u64,
    pub paid: u64,
    pub opened_slot: u64,
    pub reserved: [u8; 8],
}
layout!(HolderRecord, name = "HolderRecord", size = 160, disc = disc::HOLDER_RECORD, {
    version: u8 = 0, bump: u8 = 1, pad0: [u8; 6] = 2, mint: [u8; 32] = 8, token_account: [u8; 32] = 40,
    owner: [u8; 32] = 72, shares: u64 = 104, snapshot: U128 = 112, owed: u64 = 128, paid: u64 = 136,
    opened_slot: u64 = 144, reserved: [u8; 8] = 152,
});

///
pub const HOLDER_RECORD_ACCT_OWNER: usize = 80;
const _: () = assert!(core::mem::offset_of!(HolderRecord, owner) + 8 == HOLDER_RECORD_ACCT_OWNER);


///
///
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ClaimsRegistry {
    pub version: u8,
    pub bump: u8,
///
    pub authority_bump: u8,
    pub pad0: [u8; 5],
///
    pub current_table: [u8; 32],
    pub current_len: u16,
    pub pad1: [u8; 6],
///
    pub tables: u64,
    pub reserved: [u8; 32],
}
layout!(ClaimsRegistry, name = "ClaimsRegistry", size = 88, disc = disc::CLAIMS_REGISTRY, {
    version: u8 = 0, bump: u8 = 1, authority_bump: u8 = 2, pad0: [u8; 5] = 3,
    current_table: [u8; 32] = 8, current_len: u16 = 40, pad1: [u8; 6] = 42, tables: u64 = 48,
    reserved: [u8; 32] = 56,
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::Layout;

///
    #[test]
    fn account_lengths_and_rent_match_the_spec() {
        assert_eq!(HolderBook::ACCOUNT_LEN, 208);
        assert_eq!(HolderRecord::ACCOUNT_LEN, 168);
        assert_eq!(ClaimsRegistry::ACCOUNT_LEN, 96);
        let rent = |len: usize| (128 + len as u64) * 5_080;
        assert_eq!(rent(HolderBook::ACCOUNT_LEN), 1_706_880);
        assert_eq!(rent(HolderRecord::ACCOUNT_LEN), 1_503_680);
        assert_eq!(EXTRA_ACCOUNT_METAS_LEN, 156);
        assert_eq!(rent(EXTRA_ACCOUNT_METAS_LEN), 1_442_720);
        assert_eq!(EXECUTE_ACCOUNTS, 9);
    }

///
///
    #[test]
    fn the_generated_id_is_not_the_placeholder_and_collides_with_nothing() {
        assert_eq!(FLOOR_HOLDERS_ID[0], 18);
        assert_eq!(FLOOR_HOLDERS_ID[31], 3);
        assert!(!(FLOOR_HOLDERS_ID[0] == 24 && FLOOR_HOLDERS_ID[31] == 216), "still the spec's placeholder");
        for other in [crate::ids::FLOOR_HOOK_ID, crate::ids::FLOOR_LAUNCH_ID, crate::ids::FLOOR_VESTING_ID] {
            assert_ne!(FLOOR_HOLDERS_ID, other);
        }
        assert_eq!(crate::ids::FLOOR_HOLDERS_ID, FLOOR_HOLDERS_ID, "re-exported from ids");
    }

///
    #[test]
    fn seeds_are_the_spec_strings() {
        assert_eq!(HOLDERS_AUTHORITY, b"holders_authority");
        assert_eq!(CLAIMS_ALT_AUTHORITY, b"claims_alt_authority");
        assert_eq!(CLAIMS_REGISTRY, b"claims_registry");
        assert_eq!(HOLDER_BOOK, b"holder_book");
        assert_eq!(EXTRA_ACCOUNT_METAS, b"extra-account-metas");
        assert_eq!(HOLDER_RECORD, b"holder");
    }

///
///
///
    #[test]
    fn the_raw_pool_offsets_read_what_the_struct_wrote() {
        let mut p = FloorPool::zeroed();
        p.holder_owed = 0x1111;
        p.holder_paid_eth = 0x2222;
        p.referrer = [0x33; 32];
        p.holders_hooked = 1;
        let mut data = FloorPool::DISCRIMINATOR.to_vec();
        data.extend_from_slice(bytemuck::bytes_of(&p));
        assert_eq!(data.len(), FloorPool::ACCOUNT_LEN);
        let r = |o: usize| u64::from_le_bytes(data[o..o + 8].try_into().unwrap());
        assert_eq!(r(FLOOR_POOL_ACCT_HOLDER_OWED), 0x1111);
        assert_eq!(r(FLOOR_POOL_ACCT_HOLDER_PAID), 0x2222);
        assert_eq!(&data[FLOOR_POOL_ACCT_REFERRER..FLOOR_POOL_ACCT_REFERRER + 32], &[0x33; 32]);
        assert_eq!(data[FLOOR_POOL_ACCT_HOLDERS_HOOKED], 1);
        assert_eq!(lifetime_from_floor_pool_account(&data), Some(0x3333));

        let mut big = p;
        big.holder_owed = u64::MAX;
        let mut d2 = FloorPool::DISCRIMINATOR.to_vec();
        d2.extend_from_slice(bytemuck::bytes_of(&big));
        assert_eq!(lifetime_from_floor_pool_account(&d2), Some(u64::MAX));
        assert_eq!(lifetime_from_floor_pool_account(&data[..FLOOR_POOL_ACCT_HOLDER_PAID]), None);
    }

///
    #[test]
    fn the_record_owner_is_at_account_offset_80() {
        let mut r = HolderRecord::zeroed();
        r.owner = [0x44; 32];
        let mut data = HolderRecord::DISCRIMINATOR.to_vec();
        data.extend_from_slice(bytemuck::bytes_of(&r));
        assert_eq!(&data[HOLDER_RECORD_ACCT_OWNER..HOLDER_RECORD_ACCT_OWNER + 32], &[0x44; 32]);
    }


///
    #[test]
    fn the_claims_table_constants_agree() {
        assert_eq!(CLAIMS_TABLE_MAX, 256);
        assert_eq!(CLAIMS_TABLE_ROTATE_AT, 251);
        assert!(CLAIMS_ADDRS_REFERRED < CLAIMS_ADDRS_HOOKED);
    }
}
