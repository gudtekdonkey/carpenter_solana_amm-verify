//!
//!
//!
//!
//!
//!
//!
//!
//!
//!


///
pub const AMM_CONFIG: &[u8] = b"amm_config";
///
pub const APPROVED_HOOK: &[u8] = b"approved_hook";
///
pub const MINT_EXCLUSIVITY: &[u8] = b"mint_excl";
///
pub const MINT_POOLS: &[u8] = b"mint_pools";
///
///
///
pub const QUOTE_MINT: &[u8] = b"quote_mint";
///
pub const POOL: &[u8] = b"pool";
///
pub const BITMAP: &[u8] = b"bitmap";
///
pub const TICK_ARRAY: &[u8] = b"tick_array";
///
///
///
///
///
pub const POSITION: &[u8] = b"position";
///
pub const POOL_INDEX: &[u8] = b"pool_index";
///
pub const VAULT: &[u8] = b"vault";
///
///
///
///
///
pub const PROTOCOL_TREASURY: &[u8] = b"protocol_treasury";
///
pub const EVENT_AUTHORITY: &[u8] = b"__event_authority";


///
///
pub const HOOK_CONFIG: &[u8] = b"hook_config";
///
///
pub const HOOK_AUTHORITY: &[u8] = b"hook_authority";
///
///
///
///
pub const HOOK_SETTLE: &[u8] = b"hook_settle";



///
///
pub fn pool_seed_parts(fee: u32, tick_spacing: u16) -> ([u8; 4], [u8; 2]) {
    (fee.to_le_bytes(), tick_spacing.to_le_bytes())
}

///
pub fn tick_array_seed(start_index: i32) -> [u8; 4] {
    start_index.to_le_bytes()
}

///
pub fn u64_seed(v: u64) -> [u8; 8] {
    v.to_le_bytes()
}

///
pub fn mints_sorted(mint0: &[u8; 32], mint1: &[u8; 32]) -> bool {
    mint0 < mint1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeds_are_the_spec_strings() {
        assert_eq!(AMM_CONFIG, b"amm_config");
        assert_eq!(MINT_EXCLUSIVITY, b"mint_excl");
        assert_eq!(HOOK_CONFIG, b"hook_config");
        assert_eq!(EVENT_AUTHORITY, b"__event_authority");
        assert_eq!(PROTOCOL_TREASURY, b"protocol_treasury");
    }

    #[test]
    fn integers_are_little_endian() {
        assert_eq!(pool_seed_parts(0x80_0000, 60), ([0x00, 0x00, 0x80, 0x00], [60, 0]));
        assert_eq!(tick_array_seed(-3600), (-3600i32).to_le_bytes());
        assert_eq!(u64_seed(1), [1, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn mints_sort_bytewise_above_wsol() {
        let mut above = crate::keys::WSOL_MINT;
        above[31] = 2;
        assert!(mints_sorted(&crate::keys::WSOL_MINT, &above));
        assert!(!mints_sorted(&above, &crate::keys::WSOL_MINT));
    }
}
