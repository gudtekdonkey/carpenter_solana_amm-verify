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

pub use carpenter_types::constants::*;


///
pub const FLOOR_EVM_HOOK_FLAGS: u16 = BEFORE_INITIALIZE_FLAG
    | BEFORE_SWAP_FLAG
    | AFTER_SWAP_FLAG
    | BEFORE_SWAP_RETURNS_DELTA_FLAG
    | AFTER_SWAP_RETURNS_DELTA_FLAG;
///
pub const FLOOR_HOOK_FLAGS: u16 = FLOOR_EVM_HOOK_FLAGS | REQUESTS_ACTIONS_FLAG;





///
///
///
///
///
pub const FLOOR_MAX_ROUNDS: u8 = 3;
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
pub const FLOOR_CALLBACK_ACCOUNTS: u8 = 4;
///
pub const FLOOR_TICK_SPACING: u16 = 60;

///
///
///
///
///
pub const TOKEN_DECIMALS: u8 = 6;

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
pub const MIN_QUOTE_DECIMALS: u8 = 0;
pub const MAX_QUOTE_DECIMALS: u8 = 36;
///
pub const SUPPLY: u64 = 1_000_000_000_000_000;
///
///
///
pub const SALE_TOKENS: u64 = 300_000_000_000_000;
///
pub const POL_TOKENS: u64 = 100_000_000_000_000;
///
pub const LADDER_TOKENS: u64 = 600_000_000_000_000;





///
pub const SKIM_RAMP_TICKS: i32 = 16_094;
///
///
///
///
pub const BID_BAND_TICKS: i32 = 120;
///
pub const TICKS_PER_DOUBLING: i32 = 6_932;



///
pub const LP_FEE_PIPS: u32 = 2_500;
///
///
///
///
///
///
///
pub const DEFAULT_SKIM_SELL_AT_FLOOR_BPS: u16 = 1_000;
///
///
pub const DEFAULT_SKIM_BUY_AT_FLOOR_BPS: u16 = 0;
///
///
///
///
///
///
pub const SKIM_SELL_AT_FLOOR_MULTIPLE_BPS: u32 = 15_000;
///
///
///
pub const fn skim_sell_at_floor_bps(sell_far: u16) -> u16 {
    (sell_far as u32 * SKIM_SELL_AT_FLOOR_MULTIPLE_BPS / BPS as u32) as u16
}
///
///
///
///
///
///
///
pub const PRESET_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS: u32 = 15_000;
///
///
pub const MIN_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS: u32 = 10_000;
pub const MAX_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS: u32 = 50_000;
///
///
///
///
///
pub const MAX_SKIM_SELL_AT_FLOOR_BPS: u16 = 1_500;
const _: () = assert!(skim_sell_at_floor_bps(MAX_SKIM_FAR_BPS) == MAX_SKIM_SELL_AT_FLOOR_BPS);
///
///
///
pub const fn skim_sell_at_floor_bps_at(sell_far: u16, multiple_bps: u32) -> u16 {
    let v = sell_far as u64 * multiple_bps as u64 / BPS as u64;
    if v > u16::MAX as u64 { u16::MAX } else { v as u16 }
}
const _: () = assert!(skim_sell_at_floor_bps_at(MAX_SKIM_FAR_BPS, MAX_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS) == 5_000);
const _: () =
    assert!(skim_sell_at_floor_bps_at(MAX_SKIM_FAR_BPS, SKIM_SELL_AT_FLOOR_MULTIPLE_BPS) == skim_sell_at_floor_bps(MAX_SKIM_FAR_BPS));
///
pub const MIN_SELLABLE: u64 = 1_000_000_000;
///
///
///
///
///
///
pub const MIN_BID_LAMPORTS: u64 = 50_000_000;
///
///
///
///
pub const MIN_BID_LAMPORTS_FLOOR: u64 = 1_000_000;
///
///
///
///
///
///
///
///
pub const MIN_BID_LAMPORTS_CEIL: u64 = 5_000_000_000;



///
///
///
pub const BOUNTY_BPS: u16 = 100;
///
///
///
pub const MAX_BOUNTY_BPS: u16 = 200;
///
///
///
pub const MAX_BOUNTY_LAMPORTS: u64 = 50_000_000;
///
///
///
///
pub const MAX_BOUNTY_LAMPORTS_CEIL: u64 = 500_000_000;

///
///
///
///
///
///
pub const TEAM_TIP_CAP_DIVISOR: u64 = 20;
///
///
pub const TEAM_TIP_INTERVAL_SECS: i64 = 86_400;
const _: () = assert!(MAX_BOUNTY_LAMPORTS / TEAM_TIP_CAP_DIVISOR == 2_500_000);

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
pub const REFERRAL_CREATOR_SHARE_BPS: u16 = 1_000;
///
///
///
///
pub const MAX_REFERRAL_SHARE_BPS: u16 = 10_000;


///
pub const BPS: u64 = 10_000;
///
pub const PLATFORM_BPS: u16 = 250;














///
pub const TEAM_SALE_FEE_BPS_DEFAULT: u16 = 100;
///
pub const TEAM_SALE_FEE_BPS_MAX: u16 = 100;
///
pub const TEAM_POOL_FEE_BPS_DEFAULT: u16 = 25;
///
///
pub const TEAM_POOL_FEE_BPS_MAX: u16 = 50;

///
///
///
///
pub const MAX_SKIM_FAR_BPS: u16 = 1_000;
///
///
pub const MAX_CREATOR_BPS: u16 = 500;
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
pub const MAX_HOLDER_BPS: u16 = 1_000;

///
pub const SALT_BID: u64 = 1;
pub const SALT_SEED: u64 = 3;


///
///
///
///
///
///
///
///
///
pub const SALT_STAIR_TAIL: u64 = 102;
pub const SALT_CUSHION: u64 = 103;
pub const SALT_POL_RELIST: u64 = 104;
pub const SALT_STAIR_RESERVE: u64 = 105;
pub const SALT_STAIR_STEP0: u64 = 110;
///
pub const MAX_STAIR_STEPS: u8 = 16;
///
///
///
///
///
///
///
///
pub const STAIR_STEPS_PER_RATCHET: u8 = 1;

///
///
///
///
pub const STAIR_RENT_ROUNDS: u64 = 5;
///
///
///
///
pub const STAIR_RENT_TIP_LAMPORTS: u64 = 10_000;







///
pub const STAIR_OWNER_NONE: u8 = 0;
///
///
pub const STAIR_OWNER_TEAM: u8 = 1;
///
///
pub const STAIR_OWNER_TEAM_BUY: u8 = 2;

///
///
///
///
pub const STAIR_HELD_STEP: u8 = u8::MAX;

///
///
pub const TEAM_BUY_MAX_BPS: u16 = 500;
///
///
///
pub const BLUECHIP_TEAM_BUY_MAX_BPS: u16 = 1_000;
///
///
pub const fn team_buy_max_bps(mode: u8) -> u16 {
    if mode == MODE_BLUECHIP { BLUECHIP_TEAM_BUY_MAX_BPS } else { TEAM_BUY_MAX_BPS }
}
///
///
///
pub const TEAM_STEPS_MIN_BPS: u16 = 8_000;
///
///
///
///
pub const TEAM_STEPS_MAX_SHARE_BPS: u16 = 2_500;
///
pub const TEAM_LOCK_MIN_DAYS: u16 = 90;
///
///
pub const TEAM_DAYS_MAX: u16 = 1_825;
///
///
pub const TEAM_BUY_PAYEE_BPS: u16 = 8_000;
///
///
///
pub const TEAM_VEST_DAYS_COOK_MOONSHOT: [u16; 4] = [10, 20, 40, 80];
pub const TEAM_VEST_DAYS_BLUECHIP: [u16; 4] = [30, 60, 90, 120];
///
///
pub const TEAM_CUSTOM_VEST_DAYS_MIN: u16 = 10;
pub const TEAM_CUSTOM_VEST_DAYS_MAX: u16 = 365;
///
///
pub const TEAM_PAYEE_DEPLOYER: u8 = 0;
pub const TEAM_PAYEE_PENDING: u8 = 1;
pub const TEAM_PAYEE_RESOLVED: u8 = 2;






///
pub const LAUNCH_STATE_BONDING: u8 = 0;
pub const LAUNCH_STATE_GRADUATED: u8 = 1;
///
///
///
///
///
///
pub const LAUNCH_STATE_GENESIS: u8 = 2;
///
///
///
pub const LAUNCH_STATE_GENESIS_CLOSED: u8 = 3;





///
pub const GENESIS_SUPPLY: u64 = SUPPLY;
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
pub const GENESIS_SALE_TOKENS: u64 = 42_500_000_000_000;
const _: () = assert!(GENESIS_SALE_TOKENS == GENESIS_CORE_TOKENS + GENESIS_FF_TOKENS);
///
pub const GENESIS_CORE_TOKENS: u64 = 20_000_000_000_000;
///
pub const GENESIS_FF_TOKENS: u64 = 22_500_000_000_000;
///
///
///
///
pub const GENESIS_SALE_PRICE_LAMPORTS_PER_MILLION: u64 = 4_000_000_000;
///
///
pub const GENESIS_CORE_CAP_LAMPORTS: u64 = 80_000_000_000;
///
pub const GENESIS_FF_CAP_LAMPORTS: u64 = 90_000_000_000;
///
pub const GENESIS_POL_TOKENS: u64 = 10_000_000_000_000;
///
///
///
///
///
pub const GENESIS_PROTOCOL_STAIR_TOKENS: u64 = 197_000_000_000_000;
///
///
///
///
///
pub const GENESIS_TEAM_STAIR_TOKENS: u64 = 700_500_000_000_000;
///
pub const GENESIS_TEAM_STEP3_TOKENS: u64 = 45_000_000_000_000;
///
///
pub const GENESIS_MIN_STAIR_STEPS: u8 = 4;
///
///
pub const GENESIS_STAIR_TOKENS: u64 = 897_500_000_000_000;
const _: () = assert!(GENESIS_STAIR_TOKENS == GENESIS_PROTOCOL_STAIR_TOKENS + GENESIS_TEAM_STAIR_TOKENS);
///
pub const GENESIS_STAIR_TAIL_TOKENS: u64 = 10_000_000_000_000;
///
///
///
pub const GENESIS_LADDER_TOKENS: u64 = 907_500_000_000_000;
const _: () = assert!(GENESIS_LADDER_TOKENS == GENESIS_STAIR_TOKENS + GENESIS_STAIR_TAIL_TOKENS);
///
///
pub const GENESIS_VESTED_TOKENS: u64 = 40_000_000_000_000;
const _: () = assert!(
    GENESIS_SALE_TOKENS + GENESIS_POL_TOKENS + GENESIS_LADDER_TOKENS + GENESIS_VESTED_TOKENS == GENESIS_SUPPLY
);
///
///
pub const GENESIS_STAIR_GROWTH_BPS: u16 = 9_500;
///
///
///
pub const GENESIS_TEAM_PAYEE_BPS: u16 = 4_000;
///
///
pub const GENESIS_TEAM_FIRST_CLIFF_SECS: u32 = 30 * 86_400;
pub const GENESIS_TEAM_CLIFF_EVERY_SECS: u32 = 30 * 86_400;
///
///
pub const STAIR_KIND_PRESET: u8 = 0;
pub const STAIR_KIND_GENESIS: u8 = 1;
///
///
pub const GENESIS_SKIM_FAR_BPS: u16 = 250;
pub const GENESIS_CREATOR_BPS: u16 = 50;
///
///
///
///
///
///
///
pub const GENESIS_DEFAULT_DURATION: u64 = GENESIS_CORE_PHASE_SECS;
///
pub const GENESIS_DURATION_SECS: u64 = GENESIS_DEFAULT_DURATION;
///
pub const GENESIS_CORE_PHASE_SECS: u64 = 300;
///
///
///
///
pub const GENESIS_PHASE_CORE: u8 = 1;
pub const GENESIS_PHASE_FF: u8 = 2;
///
///
///
///
pub const GENESIS_CORE_WALLET_CAP_LAMPORTS: u64 = 20_000_000_000;
///
///
///
///
pub const GENESIS_FF_WALLET_CAP_LAMPORTS: u64 = 5_000_000_000;
///
///
pub const GENESIS_MAX_PROOF_LEN: usize = 24;
///
///
///
///
pub const GENESIS_BUYER_VEST_SECS: u64 = 2_592_000;
pub const GENESIS_BUYER_CLIFF_SECS: u64 = 1_209_600;
const _: () = assert!(GENESIS_BUYER_CLIFF_SECS < GENESIS_BUYER_VEST_SECS);
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
pub const GENESIS_REFUND_DELAY_SECS: u64 = 14 * 86_400;

///
///
///
///
///
///
pub const QUOTE_WALK_MAX_AGE_SECS: i64 = 60;
///
pub const GENESIS_SCHEDULE_HEADROOM: u64 = 545 * 86_400;
///
///
///
///
///
///
///
///
///
pub const GENESIS_LOCKS: [(u64, u64, u64); 4] = [
    (GENESIS_LOCK_CHUNK, 3 * 86_400 - 1, 1),
    (GENESIS_LOCK_CHUNK, 4 * 86_400 - 1, 1),
    (GENESIS_LOCK_CHUNK, 5 * 86_400 - 1, 1),
    (GENESIS_LOCK_CHUNK, 6 * 86_400 - 1, 1),
];
///
pub const GENESIS_LOCK_CHUNK: u64 = 10_000_000_000_000;
const _: () = assert!(4 * GENESIS_LOCK_CHUNK == GENESIS_VESTED_TOKENS);
///
///
pub const GENESIS_EVM_OPEN_TICK_MIN: i32 = -120_000;
pub const GENESIS_EVM_OPEN_TICK_MAX: i32 = 600_000;
///
///
///
///
///
///
///
pub const GENESIS_DECIMALS_SHIFT_TICKS: i32 = -69_081;
pub const GENESIS_OPEN_TICK_MIN: i32 = -189_081;
const _: () = assert!(GENESIS_OPEN_TICK_MIN == GENESIS_EVM_OPEN_TICK_MIN + GENESIS_DECIMALS_SHIFT_TICKS);
pub const GENESIS_OPEN_TICK_MAX: i32 = 443_515;

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
pub const MODE_CUSTOM: u8 = 0;
pub const MODE_COOK: u8 = 2;
pub const MODE_BLUECHIP: u8 = 3;
///
pub const MODE_MOONSHOT: u8 = 1;
///
pub const MODE_MAX: u8 = MODE_BLUECHIP;

///
///
///
///
///
pub const PRESET_REGISTRY_SEED: &[u8] = b"preset_registry";
///
pub const MAX_PRESET_ID: u8 = 31;
///
pub const PRESET_REGISTRY_LEN: usize = 1_608;
///
pub const PRESET_ROW_LEN: usize = 48;
const _: () = assert!(PRESET_REGISTRY_LEN == 72 + PRESET_ROW_LEN * (MAX_PRESET_ID as usize + 1));

///
///
///
///
///
pub const MIN_ZONE_BPS: u16 = 1_000;
pub const STANDARD_SALE_BPS: u16 = 3_000;
pub const STANDARD_POL_BPS: u16 = 1_000;

///
pub const CLUSTER_MAINNET: u8 = 0;
pub const CLUSTER_DEVNET: u8 = 1;
pub const CLUSTER_LOCALNET: u8 = 2;

///
pub const PENDING_PLATFORM_NONE: u8 = 0;
pub const PENDING_PLATFORM_TWITTER: u8 = 1;
pub const PENDING_PLATFORM_GITHUB: u8 = 2;
pub const PENDING_PLATFORM_DISCORD: u8 = 3;
pub const PENDING_PLATFORM_TELEGRAM: u8 = 4;

///
///
///
///
pub const MAX_NAME_LEN: usize = 64;
pub const MAX_SYMBOL_LEN: usize = 16;
pub const MAX_URI_LEN: usize = 200;
pub const MAX_PENDING_HANDLE_LEN: usize = 64;

///
///
///
pub const MAX_METADATA_BASE_LEN: usize = 64;
pub const METADATA_URI_SUFFIX: &str = ".json";

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
pub const METADATA_FILL_DELAY_SLOTS: u64 = 1_800_000;

///
///
pub const GOVERNANCE_TIMELOCK_SECONDS: u64 = 604_800;
///
///
pub const NOMINAL_SLOT_MS: u64 = 400;
const _: () = assert!(METADATA_FILL_DELAY_SLOTS * NOMINAL_SLOT_MS >= GOVERNANCE_TIMELOCK_SECONDS * 1_000);

///
pub const RECIPIENT_CLAIM_PREFIX: &[u8; 24] = b"FLOOR_RECIPIENT_CLAIM_V1";
///
pub const RECIPIENT_CLAIM_MSG_LEN: usize = 24 + 32 + 1 + 32 + 32 + 32 + 8;
///
///
///
///
pub const STAIR_PAYEE_CLAIM_PREFIX: &[u8; 24] = b"FLOOR_STAIR_PAYEE_CLM_V1";

///
///
pub const TRADER_LINK_PREFIX: &[u8; 24] = b"FLOOR_TRADER_REF_LINK_V1";
///
///
pub const TRADER_LINK_MSG_LEN: usize = 24 + 32 + 1 + 32 * 4 + 8;
const _: () = assert!(TRADER_LINK_MSG_LEN == 193);

///
///
pub fn trader_link_message(
    floor_launch_id: &[u8; 32],
    cluster: u8,
    trader: &[u8; 32],
    tiers: [&[u8; 32]; 3],
    deadline: i64,
) -> [u8; TRADER_LINK_MSG_LEN] {
    let mut m = [0u8; TRADER_LINK_MSG_LEN];
    let cluster = [cluster];
    let deadline = deadline.to_le_bytes();
    let parts: [&[u8]; 8] =
        [TRADER_LINK_PREFIX, floor_launch_id, &cluster, trader, tiers[0], tiers[1], tiers[2], &deadline];
    let mut at = 0;
    for p in parts {
        m[at..at + p.len()].copy_from_slice(p);
        at += p.len();
    }
    debug_assert_eq!(at, TRADER_LINK_MSG_LEN);
    m
}

#[cfg(test)]
mod tests {
    use super::*;

///
    #[test]
    fn the_trader_link_message_is_193_bytes_in_the_spec_order() {
        assert_ne!(&TRADER_LINK_PREFIX[..], &RECIPIENT_CLAIM_PREFIX[..]);
        assert_ne!(&TRADER_LINK_PREFIX[..], &STAIR_PAYEE_CLAIM_PREFIX[..]);
        let m = trader_link_message(&[7; 32], 2, &[1; 32], [&[3; 32], &[4; 32], &[0; 32]], 1_900_000_000);
        assert_eq!(m.len(), 193);
        assert_eq!(&m[..24], b"FLOOR_TRADER_REF_LINK_V1");
        assert_eq!(&m[24..56], &[7u8; 32]);
        assert_eq!(m[56], 2);
        assert_eq!(&m[57..89], &[1u8; 32]);
        assert_eq!(&m[89..121], &[3u8; 32]);
        assert_eq!(&m[121..153], &[4u8; 32]);
        assert_eq!(&m[153..185], &[0u8; 32], "an absent tier is the default key");
        assert_eq!(&m[185..193], &1_900_000_000i64.to_le_bytes());
    }

    #[test]
    fn floor_flags_are_0x60cc() {
        assert_eq!(FLOOR_EVM_HOOK_FLAGS, 0x20CC);
        assert_eq!(FLOOR_HOOK_FLAGS, 0x60CC);
    }

    #[test]
    fn supply_splits_exactly() {
        assert_eq!(SALE_TOKENS + POL_TOKENS + LADDER_TOKENS, SUPPLY);
    }



    #[test]
    fn the_split_is_30_10_60() {
        assert_eq!((SALE_TOKENS * 100 / SUPPLY, POL_TOKENS * 100 / SUPPLY, LADDER_TOKENS * 100 / SUPPLY), (30, 10, 60));
    }


    #[test]
    fn band_multiple_and_skim_defaults() {
        assert_eq!(BID_BAND_TICKS, 120);
        assert_eq!((DEFAULT_SKIM_BUY_AT_FLOOR_BPS, DEFAULT_SKIM_SELL_AT_FLOOR_BPS), (0, 1_000));
        assert!(DEFAULT_SKIM_SELL_AT_FLOOR_BPS <= MAX_SKIM_FAR_BPS);
    }




    #[test]
    fn caps_and_the_at_floor_sell_multiple_2026_09_25() {
        assert_eq!((MAX_CREATOR_BPS, MAX_HOLDER_BPS, MAX_SKIM_FAR_BPS), (500, 1_000, 1_000));
        assert_eq!(SKIM_SELL_AT_FLOOR_MULTIPLE_BPS, 15_000);
        assert_eq!(skim_sell_at_floor_bps(0), 0);
        assert_eq!(skim_sell_at_floor_bps(150), 225, "Meme: 1.5 % far -> 2.25 % at the floor");
        assert_eq!(skim_sell_at_floor_bps(200), 300, "2 % far -> 3 % at the floor (Bluechip 2026-09-25; Cook's final 2026-09-29)");

        assert_eq!(skim_sell_at_floor_bps(100), 150, "Moonshot: 1 % far -> 1.5 % at the floor");
        assert_eq!(skim_sell_at_floor_bps(300), 450, "Bluechip: 3 % far -> 4.5 % at the floor");
        assert_eq!(skim_sell_at_floor_bps(MAX_SKIM_FAR_BPS), 1_500, "no cap: 10 % far -> 15 %");
        assert_eq!(skim_sell_at_floor_bps(1), 1, "rounded down: 1.5 -> 1");
        assert_eq!(skim_sell_at_floor_bps(333), 499, "rounded down: 499.5 -> 499");
    }





    #[test]
    fn the_at_floor_sell_cap_is_fifteen_percent_2026_09_30() {
        assert_eq!(MAX_SKIM_SELL_AT_FLOOR_BPS, 1_500);
        assert!(skim_sell_at_floor_bps_at(1_000, 15_000) <= MAX_SKIM_SELL_AT_FLOOR_BPS, "10 % far at 1.5x: exactly the cap");
        assert!(skim_sell_at_floor_bps_at(1_000, 16_000) > MAX_SKIM_SELL_AT_FLOOR_BPS);
        assert!(skim_sell_at_floor_bps_at(300, 50_000) <= MAX_SKIM_SELL_AT_FLOOR_BPS, "3 % far at 5x: exactly the cap");
        assert!(skim_sell_at_floor_bps_at(310, 50_000) > MAX_SKIM_SELL_AT_FLOOR_BPS);
    }

    #[test]
    fn the_at_floor_sell_multiple_per_launch_2026_09_29() {
        assert_eq!(PRESET_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS, 15_000);
        assert_eq!((MIN_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS, MAX_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS), (10_000, 50_000));
        assert!((MIN_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS..=MAX_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS).contains(&SKIM_SELL_AT_FLOOR_MULTIPLE_BPS));

        assert_eq!(skim_sell_at_floor_bps_at(200, PRESET_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS), 300, "every preset: 2 % far -> 3 % at the floor");
        assert_eq!(skim_sell_at_floor_bps_at(250, 15_000), 375, "$FLOOR: 2.5 % far -> 3.75 % (unchanged)");
        for far in [0u16, 1, 150, 333, MAX_SKIM_FAR_BPS] {
            assert_eq!(skim_sell_at_floor_bps_at(far, SKIM_SELL_AT_FLOOR_MULTIPLE_BPS), skim_sell_at_floor_bps(far), "{far}");
        }
        assert_eq!(skim_sell_at_floor_bps_at(1, 15_000), 1, "rounded down");
        assert_eq!(skim_sell_at_floor_bps_at(u16::MAX, u32::MAX), u16::MAX, "saturates, never wraps");
    }



    #[test]
    fn referral_share_is_the_evm_default_under_the_evm_ceiling() {
        assert_eq!(REFERRAL_CREATOR_SHARE_BPS, 1_000);
        assert_eq!(MAX_REFERRAL_SHARE_BPS as u64, BPS);
        assert!(REFERRAL_CREATOR_SHARE_BPS <= MAX_REFERRAL_SHARE_BPS);
    }





    #[test]
    fn the_platform_fee_defaults_sit_inside_their_ruled_caps() {
        assert_eq!((TEAM_SALE_FEE_BPS_DEFAULT, TEAM_SALE_FEE_BPS_MAX), (100, 100));
        assert_eq!((TEAM_POOL_FEE_BPS_DEFAULT, TEAM_POOL_FEE_BPS_MAX), (25, 50));
        assert!(TEAM_SALE_FEE_BPS_DEFAULT <= TEAM_SALE_FEE_BPS_MAX);
        assert!(TEAM_POOL_FEE_BPS_DEFAULT <= TEAM_POOL_FEE_BPS_MAX);
        assert!(TEAM_SALE_FEE_BPS_MAX < PLATFORM_BPS && TEAM_POOL_FEE_BPS_MAX < PLATFORM_BPS);
    }

///
///
///
///
///
///
    #[test]
    fn the_refund_opens_fourteen_days_after_the_close() {
        assert_eq!(GENESIS_REFUND_DELAY_SECS, 1_209_600);
        assert_eq!(QUOTE_WALK_MAX_AGE_SECS, 60);
    }

    #[test]
    fn the_two_phase_raise_is_core_then_ff_at_one_fixed_price() {
        assert_eq!(GENESIS_CORE_PHASE_SECS, 300);
        assert_eq!((GENESIS_DEFAULT_DURATION, GENESIS_DURATION_SECS), (300, 300));
        assert_eq!((GENESIS_CORE_WALLET_CAP_LAMPORTS, GENESIS_FF_WALLET_CAP_LAMPORTS), (20_000_000_000, 5_000_000_000));
        assert_eq!(18 * GENESIS_FF_WALLET_CAP_LAMPORTS, GENESIS_FF_CAP_LAMPORTS);
        assert_eq!(GENESIS_SALE_PRICE_LAMPORTS_PER_MILLION, 4_000_000_000);
        assert_eq!(GENESIS_CORE_CAP_LAMPORTS as u128 * 1_000_000_000_000, GENESIS_CORE_TOKENS as u128 * GENESIS_SALE_PRICE_LAMPORTS_PER_MILLION as u128);
        assert_eq!(GENESIS_FF_CAP_LAMPORTS as u128 * 1_000_000_000_000, GENESIS_FF_TOKENS as u128 * GENESIS_SALE_PRICE_LAMPORTS_PER_MILLION as u128);
        assert_eq!((GENESIS_PHASE_CORE, GENESIS_PHASE_FF), (1, 2));
        assert_eq!(GENESIS_MAX_PROOF_LEN, 24);
    }

    #[test]
    fn claim_message_is_161_bytes() {
        assert_eq!(RECIPIENT_CLAIM_MSG_LEN, 161);
    }

///
///
///
    #[test]
    fn floor_genesis_splits_one_billion_with_four_short_locks() {



        assert_eq!(
            (GENESIS_SALE_TOKENS, GENESIS_POL_TOKENS, GENESIS_PROTOCOL_STAIR_TOKENS, GENESIS_TEAM_STAIR_TOKENS, GENESIS_STAIR_TAIL_TOKENS, GENESIS_VESTED_TOKENS),
            (42_500_000_000_000, 10_000_000_000_000, 197_000_000_000_000, 700_500_000_000_000, 10_000_000_000_000, 40_000_000_000_000)
        );
        assert_eq!((GENESIS_CORE_TOKENS, GENESIS_FF_TOKENS), (20_000_000_000_000, 22_500_000_000_000));

        assert_eq!((GENESIS_TEAM_STEP3_TOKENS, GENESIS_MIN_STAIR_STEPS), (45_000_000_000_000, 4));
        assert_eq!(GENESIS_SALE_TOKENS + GENESIS_POL_TOKENS + GENESIS_STAIR_TOKENS + GENESIS_STAIR_TAIL_TOKENS + GENESIS_VESTED_TOKENS, SUPPLY);
        assert_eq!(GENESIS_LADDER_TOKENS, 907_500_000_000_000);
        assert_eq!(GENESIS_LOCKS.iter().map(|l| l.0).sum::<u64>(), GENESIS_VESTED_TOKENS);
        for (i, (amount, cliff, linear)) in GENESIS_LOCKS.iter().enumerate() {
            assert_eq!((*amount, cliff + linear), (10_000_000_000_000, (3 + i as u64) * 86_400), "lock {i}: 10M whole at day {}", 3 + i);
            assert_eq!(*linear, 1, "the smallest line create_schedule_at accepts");
        }
        assert!(GENESIS_SCHEDULE_HEADROOM >= 6 * 86_400, "the longest lock ends inside the headroom");
        assert_eq!((GENESIS_STAIR_GROWTH_BPS, STAIR_KIND_PRESET, STAIR_KIND_GENESIS), (9_500, 0, 1));
        assert_eq!(GENESIS_TEAM_PAYEE_BPS, 4_000);
        assert_eq!((GENESIS_TEAM_FIRST_CLIFF_SECS, GENESIS_TEAM_CLIFF_EVERY_SECS), (30 * 86_400, 30 * 86_400));
    }
}
