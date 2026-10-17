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

pub use carpenter_types::seeds::*;




///
pub const FLOOR_POOL: &[u8] = b"floor_pool";
///
///
///
pub const STAIR: &[u8] = b"stair";
///
///
///
///
///
pub const STAIR_RENT: &[u8] = b"stair_rent";
///
///
///
pub const QUOTE_POLICY: &[u8] = b"quote";
///
///
///
///
///
///
///
///
///
pub const PLATFORM_TREASURY: &[u8] = b"platform_treasury";
///
///
///
///
///
///
///
///
pub const TEAM_TREASURY: &[u8] = b"team_treasury";
///
///
///
///
///
///
pub const FLOOR_PLATFORM_POOL: &[u8] = b"floor_platform_pool";


///
///
pub const TRADER_LINK: &[u8] = b"trader_link";
///
///
///
pub const TRADER_REF: &[u8] = b"trader_ref";
///
///
pub const TRADER_REF_VAULT: &[u8] = b"trader_ref_vault";
///
///
pub const TRADER_REF_SALE: &[u8] = b"trader_ref_sale";


pub use crate::holders::{
    CLAIMS_ALT_AUTHORITY, CLAIMS_REGISTRY, EXTRA_ACCOUNT_METAS, HOLDERS_AUTHORITY, HOLDER_BOOK, HOLDER_RECORD,
};


///
pub const LAUNCH_CONFIG: &[u8] = b"launch_config";
///
pub const REGISTRY_ENTRY: &[u8] = b"entry";
///
pub const LAUNCH: &[u8] = b"launch";
///
pub const SALE_VAULT: &[u8] = b"sale_vault";
///
///
pub const GRAD_RENT: &[u8] = b"grad_rent";
///
pub const BUYER: &[u8] = b"buyer";
///
pub const MINT: &[u8] = b"mint";
///
pub const MINT_AUTHORITY: &[u8] = b"mint_authority";
///
///
pub const LOCK_AUTHORITY: &[u8] = b"lock_authority";
///
///
pub const LAUNCH_AUTHORITY: &[u8] = b"launch_authority";
///
///
///
///
///
pub const GENESIS_SALE: &[u8] = b"genesis_sale";
///
///
///
pub const GENESIS_CONTRIBUTION: &[u8] = b"genesis_contrib";
///
///
///
pub const QUOTE_WALK: &[u8] = b"quote_walk";
///
///
pub const PRESET_REGISTRY: &[u8] = crate::constants::PRESET_REGISTRY_SEED;





///
pub const VESTING_SCHEDULE: &[u8] = b"schedule";
///
pub const VESTING_ESCROW: &[u8] = b"escrow";
///
pub const VESTING_AUTHORITY: &[u8] = b"vesting_authority";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeds_are_the_spec_strings() {
        assert_eq!(FLOOR_POOL, b"floor_pool");
        assert_eq!(QUOTE_POLICY, b"quote");
        assert_eq!(QUOTE_WALK, b"quote_walk");
        assert_eq!(PLATFORM_TREASURY, b"platform_treasury");

        assert_eq!(TEAM_TREASURY, b"team_treasury");
        assert_ne!(TEAM_TREASURY, PLATFORM_TREASURY);

        assert_eq!(FLOOR_PLATFORM_POOL, b"floor_platform_pool");
        assert_ne!(FLOOR_PLATFORM_POOL, PLATFORM_TREASURY);
        assert_ne!(FLOOR_PLATFORM_POOL, TEAM_TREASURY);
        assert_eq!(REGISTRY_ENTRY, b"entry");
        assert_eq!(GRAD_RENT, b"grad_rent");
        assert_eq!(PRESET_REGISTRY, b"preset_registry");

        assert_eq!(HOLDER_BOOK, b"holder_book");
        assert_eq!(HOLDER_RECORD, b"holder");
        assert_eq!(EXTRA_ACCOUNT_METAS, b"extra-account-metas");
        assert_eq!(HOLDERS_AUTHORITY, b"holders_authority");

        assert_eq!(AMM_CONFIG, b"amm_config");
        assert_eq!(HOOK_CONFIG, b"hook_config");
        assert_eq!(EVENT_AUTHORITY, b"__event_authority");

        assert_eq!(TRADER_LINK, b"trader_link");
        assert_eq!(TRADER_REF, b"trader_ref");
        assert_eq!(TRADER_REF_VAULT, b"trader_ref_vault");
        assert_eq!(TRADER_REF_SALE, b"trader_ref_sale");

        assert_ne!(TRADER_REF_VAULT, PLATFORM_TREASURY);
        assert_ne!(TRADER_REF_VAULT, TEAM_TREASURY);
    }

    #[test]
    fn floor_mints_sort_above_wsol() {
        let mut above = crate::keys::WSOL_MINT;
        above[31] = 2;
        assert!(mints_sorted(&crate::keys::WSOL_MINT, &above));
        assert!(!mints_sorted(&above, &crate::keys::WSOL_MINT));
    }
}
