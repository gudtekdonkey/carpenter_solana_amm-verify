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
pub const MIN_TICK: i32 = -443_636;
///
pub const MAX_TICK: i32 = 443_636;
///
pub const MIN_SQRT_PRICE_X64: u128 = 4_295_048_016;
///
pub const MAX_SQRT_PRICE_X64: u128 = 79_226_673_521_066_979_257_578_248_091;

///
pub const TICK_ARRAY_SIZE: i32 = 60;
///
pub const MIN_TICK_SPACING: u16 = 1;
///
pub const MAX_TICK_SPACING: u16 = 32_767;

///
pub const DYNAMIC_FEE_FLAG: u32 = 0x80_0000;
///
pub const MAX_LP_FEE: u32 = 1_000_000;
///
///
pub const MAX_SWAP_FEE: u32 = 1_000_000;
///
pub const MIN_DONATE_LIQUIDITY: u128 = 1_000_000;
///
pub const DONATE_MAX_AMOUNT: u64 = u64::MAX / 2;

///
pub const MAX_QUOTE_MINTS: usize = 2;












///
///
///
pub const PROTOCOL_FEE_DEFAULT: u32 = 500;
///
///
///
pub const MAX_PROTOCOL_FEE: u32 = 1_000;
///
///
///
pub const PROTOCOL_FEE_DEFAULT_PACKED: u32 = 2_048_500;
///
pub const PIPS_DENOMINATOR: u32 = 1_000_000;
///
///
///
pub const PROTOCOL_FEE_ON_INPUT: u8 = 0;
///
///
///
///
pub const PROTOCOL_FEE_ON_CURRENCY0: u8 = 1;
///
pub const PROTOCOL_FEE_ON_CURRENCY1: u8 = 2;


const _: () = {

    assert!(PROTOCOL_FEE_DEFAULT_PACKED == PROTOCOL_FEE_DEFAULT | (PROTOCOL_FEE_DEFAULT << 12));

    assert!(PROTOCOL_FEE_DEFAULT > 0 && PROTOCOL_FEE_DEFAULT <= MAX_PROTOCOL_FEE);

    assert!(MAX_PROTOCOL_FEE <= 0xfff);

    assert!(MAX_PROTOCOL_FEE < PIPS_DENOMINATOR / 2);

    assert!(PROTOCOL_FEE_ON_INPUT == 0);
    assert!(PROTOCOL_FEE_ON_CURRENCY0 != PROTOCOL_FEE_ON_CURRENCY1);
    assert!(PROTOCOL_FEE_ON_CURRENCY0 != PROTOCOL_FEE_ON_INPUT && PROTOCOL_FEE_ON_CURRENCY1 != PROTOCOL_FEE_ON_INPUT);
};


pub const BEFORE_INITIALIZE_FLAG: u16 = 1 << 13;
pub const AFTER_INITIALIZE_FLAG: u16 = 1 << 12;
pub const BEFORE_ADD_LIQUIDITY_FLAG: u16 = 1 << 11;
pub const AFTER_ADD_LIQUIDITY_FLAG: u16 = 1 << 10;
pub const BEFORE_REMOVE_LIQUIDITY_FLAG: u16 = 1 << 9;
pub const AFTER_REMOVE_LIQUIDITY_FLAG: u16 = 1 << 8;
pub const BEFORE_SWAP_FLAG: u16 = 1 << 7;
pub const AFTER_SWAP_FLAG: u16 = 1 << 6;
pub const BEFORE_DONATE_FLAG: u16 = 1 << 5;
pub const AFTER_DONATE_FLAG: u16 = 1 << 4;
pub const BEFORE_SWAP_RETURNS_DELTA_FLAG: u16 = 1 << 3;
pub const AFTER_SWAP_RETURNS_DELTA_FLAG: u16 = 1 << 2;
pub const AFTER_ADD_LIQUIDITY_RETURNS_DELTA_FLAG: u16 = 1 << 1;
pub const AFTER_REMOVE_LIQUIDITY_RETURNS_DELTA_FLAG: u16 = 1 << 0;
///
pub const REQUESTS_ACTIONS_FLAG: u16 = 1 << 14;
///
///
///
///
///
///
///
///
///
pub const SETTLES_FLAG: u16 = 1 << 15;
///
pub const ALL_HOOK_FLAGS: u16 = 0xFFFF;
///
pub const V4_HOOK_FLAGS: u16 = 0x3FFF;

///
///
pub const GUARDIAN_PAUSE_MAX_SECS: i64 = 72 * 3_600;
///
///
///
pub const GUARDIAN_COOLOFF_SECS: i64 = 72 * 3_600;

///
///
pub const fn pool_is_paused(paused: u8, guardian_paused_until: i64, now: i64) -> bool {
    paused != 0 || now < guardian_paused_until
}

///
///
pub const fn guardian_pause_until(guardian_paused_until: i64, now: i64) -> Result<i64, bool> {
    if now < guardian_paused_until {
        return Err(true);
    }
    if guardian_paused_until != 0 && now < guardian_paused_until + GUARDIAN_COOLOFF_SECS {
        return Err(false);
    }
    Ok(now + GUARDIAN_PAUSE_MAX_SECS)
}

#[cfg(test)]
mod tests {
    use super::*;

///
///
    #[test]
    fn flag_masks_are_the_named_bits() {
        let v4 = BEFORE_INITIALIZE_FLAG
            | AFTER_INITIALIZE_FLAG
            | BEFORE_ADD_LIQUIDITY_FLAG
            | AFTER_ADD_LIQUIDITY_FLAG
            | BEFORE_REMOVE_LIQUIDITY_FLAG
            | AFTER_REMOVE_LIQUIDITY_FLAG
            | BEFORE_SWAP_FLAG
            | AFTER_SWAP_FLAG
            | BEFORE_DONATE_FLAG
            | AFTER_DONATE_FLAG
            | BEFORE_SWAP_RETURNS_DELTA_FLAG
            | AFTER_SWAP_RETURNS_DELTA_FLAG
            | AFTER_ADD_LIQUIDITY_RETURNS_DELTA_FLAG
            | AFTER_REMOVE_LIQUIDITY_RETURNS_DELTA_FLAG;
        assert_eq!(v4, V4_HOOK_FLAGS);
        assert_eq!(v4 | REQUESTS_ACTIONS_FLAG | SETTLES_FLAG, ALL_HOOK_FLAGS);
        assert_eq!(SETTLES_FLAG, 1 << 15, "the settle flag is the last bit");
    }

    #[test]
    fn swap_fee_caps_agree_with_v4() {
        assert_eq!(MAX_SWAP_FEE, MAX_LP_FEE);
        assert!(DYNAMIC_FEE_FLAG > MAX_LP_FEE, "the dynamic flag is not a fee");
    }

///
///
///
    #[test]
    fn the_guardian_pause_lapses_cannot_extend_and_cools_off() {
        let t = 1_790_000_000;
        assert_eq!(guardian_pause_until(0, t), Ok(t + GUARDIAN_PAUSE_MAX_SECS), "never paused: no cool-off");
        let until = t + GUARDIAN_PAUSE_MAX_SECS;
        assert!(pool_is_paused(0, until, t) && pool_is_paused(0, until, until - 1));
        assert!(!pool_is_paused(0, until, until), "it lapses on its own at the 72nd hour");
        assert_eq!(guardian_pause_until(until, until - 1), Err(true), "no extending");
        assert_eq!(guardian_pause_until(until, until), Err(false), "the cool-off starts at the end");
        assert_eq!(guardian_pause_until(until, until + GUARDIAN_COOLOFF_SECS - 1), Err(false));
        assert_eq!(guardian_pause_until(until, until + GUARDIAN_COOLOFF_SECS), Ok(until + GUARDIAN_COOLOFF_SECS + GUARDIAN_PAUSE_MAX_SECS));
        assert!(pool_is_paused(1, 0, t) && pool_is_paused(1, 0, i64::MAX), "the authority's pause never lapses");
        assert_eq!((GUARDIAN_PAUSE_MAX_SECS, GUARDIAN_COOLOFF_SECS), (259_200, 259_200));
    }
}
