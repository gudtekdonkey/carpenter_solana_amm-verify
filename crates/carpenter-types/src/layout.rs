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

use crate::disc::account as disc;
use crate::pod::{I128, U128};
use bytemuck::{Pod, Zeroable};
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Field {
    pub name: &'static str,
    pub offset: usize,
    pub size: usize,
}

///
pub trait Layout: Pod {
///
    const NAME: &'static str;
///
    const SIZE: usize;
///
    const FIELDS: &'static [Field];
///
    const DISCRIMINATOR: [u8; 8];
///
    const ACCOUNT_LEN: usize = 8 + Self::SIZE;
}

///
///
///
///
///
///
///
#[macro_export]
macro_rules! layout {
    (
        $t:ident, name = $name:literal, size = $size:expr, disc = $disc:expr,
        { $( $f:ident : $fty:ty = $off:expr ),* $(,)? }
    ) => {
        impl $crate::layout::Layout for $t {
            const NAME: &'static str = $name;
            const SIZE: usize = $size;
            const FIELDS: &'static [$crate::layout::Field] = &[
                $( $crate::layout::Field { name: stringify!($f), offset: $off, size: core::mem::size_of::<$fty>() } ),*
            ];
            const DISCRIMINATOR: [u8; 8] = $disc;
        }
        const _: () = {
            assert!(core::mem::size_of::<$t>() == $size, concat!(stringify!($t), ": size"));
            assert!(core::mem::size_of::<$t>() % 8 == 0, concat!(stringify!($t), ": size % 8"));
            assert!(core::mem::align_of::<$t>() <= 8, concat!(stringify!($t), ": align"));
            $( assert!(core::mem::offset_of!($t, $f) == $off, concat!(stringify!($t), ".", stringify!($f), ": offset")); )*
            let fields = <$t as $crate::layout::Layout>::FIELDS;
            let mut i = 0;
            let mut end = 0;
            while i < fields.len() {
                assert!(fields[i].offset == end, concat!(stringify!($t), ": fields not contiguous"));
                end = fields[i].offset + fields[i].size;
                i += 1;
            }
            assert!(end == $size, concat!(stringify!($t), ": fields do not end at SIZE"));
        };
    };
}





///
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct AmmConfig {
    pub version: u8,
    pub bump: u8,
    pub pad0: [u8; 6],
///
///
///
///
    pub authority: [u8; 32],
    pub pending_authority: [u8; 32],
///
///
///
///
    pub protocol_fee_controller: [u8; 32],
///
///
///
///
///
///
///
    pub pause_guardian: [u8; 32],
///
///
    pub pad1: [u8; 40],
///
///
///
///
///
///
    pub quote_caps_admitted: u8,
///
    pub pad2: [u8; 3],
///
///
///
    pub protocol_fee: u32,
    pub reserved: [u8; 24],
}
layout!(AmmConfig, name = "AmmConfig", size = 208, disc = disc::AMM_CONFIG, {
    version: u8 = 0, bump: u8 = 1, pad0: [u8; 6] = 2,
    authority: [u8; 32] = 8, pending_authority: [u8; 32] = 40, protocol_fee_controller: [u8; 32] = 72,
    pause_guardian: [u8; 32] = 104, pad1: [u8; 40] = 136,
    quote_caps_admitted: u8 = 176, pad2: [u8; 3] = 177,
    protocol_fee: u32 = 180, reserved: [u8; 24] = 184,
});

///
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ApprovedHook {
    pub version: u8,
    pub bump: u8,
///
    pub paused: u8,
    pub max_rounds: u8,
    pub callback_accounts: u8,
    pub allow_public_pools: u8,
    pub flags: u16,
    pub hook_program: [u8; 32],
    pub hook_authority: [u8; 32],
    pub hook_authority_bump: u8,
    pub pad0: [u8; 7],
///
    pub programdata_slot: u64,
    pub approved_slot: u64,
///
    pub name: [u8; 32],
///
    pub allow_exclusivity: u8,
///
///
///
///
///
///
///
///
    pub base_transfer_hook: [u8; 32],
///
///
///
///
///
///
///
    pub allow_hook_quotes: u8,
    pub reserved: [u8; 22],
///
///
///
///
///
    pub guardian_paused_until: i64,
}
layout!(ApprovedHook, name = "ApprovedHook", size = 192, disc = disc::APPROVED_HOOK, {
    version: u8 = 0, bump: u8 = 1, paused: u8 = 2, max_rounds: u8 = 3, callback_accounts: u8 = 4,
    allow_public_pools: u8 = 5, flags: u16 = 6, hook_program: [u8; 32] = 8, hook_authority: [u8; 32] = 40,
    hook_authority_bump: u8 = 72, pad0: [u8; 7] = 73, programdata_slot: u64 = 80, approved_slot: u64 = 88,
    name: [u8; 32] = 96, allow_exclusivity: u8 = 128, base_transfer_hook: [u8; 32] = 129, allow_hook_quotes: u8 = 161,
    reserved: [u8; 22] = 162,
    guardian_paused_until: i64 = 184,
});

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
pub struct MintExclusivity {
    pub mint: [u8; 32],
    pub hook_program: [u8; 32],
    pub bump: u8,
    pub has_protocol_fee: u8,
    pub pad0: [u8; 2],
    pub protocol_fee: u32,
}
layout!(MintExclusivity, name = "MintExclusivity", size = 72, disc = disc::MINT_EXCLUSIVITY, {
    mint: [u8; 32] = 0, hook_program: [u8; 32] = 32, bump: u8 = 64, has_protocol_fee: u8 = 65, pad0: [u8; 2] = 66,
    protocol_fee: u32 = 68,
});

///
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct MintPoolCount {
    pub mint: [u8; 32],
    pub count: u64,
    pub bump: u8,
    pub reserved: [u8; 15],
}
layout!(MintPoolCount, name = "MintPoolCount", size = 56, disc = disc::MINT_POOL_COUNT, {
    mint: [u8; 32] = 0, count: u64 = 32, bump: u8 = 40, reserved: [u8; 15] = 41,
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
///
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct QuoteMint {
    pub mint: [u8; 32],
///
///
///
///
///
    pub authority_digest: [u8; 32],
    pub listed_slot: u64,
///
    pub retired_slot: u64,
///
    pub state: u8,
    pub bump: u8,
///
    pub decimals: u8,
///
    pub caps: u8,
    pub pad0: [u8; 4],
///
///
///
///
    pub hook_program: [u8; 32],
}
layout!(QuoteMint, name = "QuoteMint", size = 120, disc = disc::QUOTE_MINT, {
    mint: [u8; 32] = 0, authority_digest: [u8; 32] = 32, listed_slot: u64 = 64, retired_slot: u64 = 72,
    state: u8 = 80, bump: u8 = 81, decimals: u8 = 82, caps: u8 = 83, pad0: [u8; 4] = 84,
    hook_program: [u8; 32] = 88,
});

///
pub const QUOTE_STATE_LISTED: u8 = 1;
///
pub const QUOTE_STATE_RETIRED: u8 = 2;
///
///
///
///
pub const QUOTE_STATE_HOOK_LISTED: u8 = 3;

///
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Pool {
    pub version: u8,
    pub bump: u8,
    pub hook_flags: u16,
    pub tick_spacing: u16,
    pub hook_max_rounds: u8,
    pub hook_callback_accounts: u8,
///
    pub fee: u32,
///
    pub lp_fee: u32,
///
///
///
///
    pub protocol_fee: u32,
    pub tick_current: i32,
    pub mint0: [u8; 32],
    pub mint1: [u8; 32],
    pub hook: [u8; 32],
    pub hook_authority: [u8; 32],
    pub vault0: [u8; 32],
    pub vault1: [u8; 32],
    pub token_program0: [u8; 32],
    pub token_program1: [u8; 32],
    pub sqrt_price_x64: U128,
    pub liquidity: U128,
///
    pub fee_growth_global0_x64: U128,
    pub fee_growth_global1_x64: U128,
    pub protocol_fees_owed0: u64,
    pub protocol_fees_owed1: u64,
///
    pub hook_claims0: u64,
    pub hook_claims1: u64,
///
    pub volume0: U128,
    pub volume1: U128,
    pub lp_fees0: U128,
    pub lp_fees1: U128,
///
    pub hook_fees0: U128,
    pub hook_fees1: U128,
    pub swap_count: u64,
    pub created_slot: u64,
    pub decimals0: u8,
    pub decimals1: u8,
    pub bitmap_bump: u8,
    pub vault0_bump: u8,
    pub vault1_bump: u8,
    pub pad0: [u8; 3],
///
    pub reserved0: u64,
///
///
///
///
    pub paused: u8,
///
///
///
///
///
    pub base_transfer_hook: [u8; 32],
///
///
///
    pub base_hook_side: u8,
///
///
///
///
///
///
///
    pub protocol_fee_currency: u8,
    pub reserved: [u8; 85],
///
///
///
///
///
    pub guardian_paused_until: i64,
}
layout!(Pool, name = "Pool", size = 632, disc = disc::POOL, {
    version: u8 = 0, bump: u8 = 1, hook_flags: u16 = 2, tick_spacing: u16 = 4,
    hook_max_rounds: u8 = 6, hook_callback_accounts: u8 = 7, fee: u32 = 8, lp_fee: u32 = 12,
    protocol_fee: u32 = 16, tick_current: i32 = 20, mint0: [u8; 32] = 24, mint1: [u8; 32] = 56,
    hook: [u8; 32] = 88, hook_authority: [u8; 32] = 120, vault0: [u8; 32] = 152, vault1: [u8; 32] = 184,
    token_program0: [u8; 32] = 216, token_program1: [u8; 32] = 248, sqrt_price_x64: U128 = 280,
    liquidity: U128 = 296, fee_growth_global0_x64: U128 = 312, fee_growth_global1_x64: U128 = 328,
    protocol_fees_owed0: u64 = 344, protocol_fees_owed1: u64 = 352, hook_claims0: u64 = 360,
    hook_claims1: u64 = 368, volume0: U128 = 376, volume1: U128 = 392, lp_fees0: U128 = 408,
    lp_fees1: U128 = 424, hook_fees0: U128 = 440, hook_fees1: U128 = 456, swap_count: u64 = 472,
    created_slot: u64 = 480, decimals0: u8 = 488, decimals1: u8 = 489, bitmap_bump: u8 = 490,
    vault0_bump: u8 = 491, vault1_bump: u8 = 492, pad0: [u8; 3] = 493, reserved0: u64 = 496,
    paused: u8 = 504, base_transfer_hook: [u8; 32] = 505, base_hook_side: u8 = 537,
    protocol_fee_currency: u8 = 538, reserved: [u8; 85] = 539,
    guardian_paused_until: i64 = 624,
});

///
///
///
///
///
///
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct TickArrayBitmapHeader {
    pub pool: [u8; 32],
    pub tick_spacing: u16,
    pub pad0: u16,
    pub arrays_per_side: u32,
}
layout!(TickArrayBitmapHeader, name = "TickArrayBitmap", size = 40, disc = disc::TICK_ARRAY_BITMAP, {
    pool: [u8; 32] = 0, tick_spacing: u16 = 32, pad0: u16 = 34, arrays_per_side: u32 = 36,
});

///
pub const fn bitmap_arrays_per_side(tick_spacing: u16) -> u32 {
    let span = (crate::constants::TICK_ARRAY_SIZE as u32) * (tick_spacing as u32);
    (crate::constants::MAX_TICK as u32).div_ceil(span)
}

///
pub const fn bitmap_word_count(tick_spacing: u16) -> usize {
    ((2 * bitmap_arrays_per_side(tick_spacing) + 1) as usize).div_ceil(64)
}

///
pub const fn bitmap_account_len(tick_spacing: u16) -> usize {
    8 + TickArrayBitmapHeader::SIZE + 8 * bitmap_word_count(tick_spacing)
}

///
pub const fn tick_array_start_index(tick: i32, tick_spacing: u16) -> i32 {
    let span = crate::constants::TICK_ARRAY_SIZE * tick_spacing as i32;
    tick.div_euclid(span) * span
}

///
///
pub const fn bitmap_bit(start_index: i32, tick_spacing: u16) -> i64 {
    let span = crate::constants::TICK_ARRAY_SIZE * tick_spacing as i32;
    (start_index / span) as i64 + bitmap_arrays_per_side(tick_spacing) as i64
}

///
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct Tick {
    pub liquidity_gross: U128,
    pub liquidity_net: I128,
    pub fee_growth_outside0_x64: U128,
    pub fee_growth_outside1_x64: U128,
}
const _: () = assert!(core::mem::size_of::<Tick>() == 64);

///
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct TickArray {
    pub pool: [u8; 32],
    pub start_index: i32,
    pub initialized_tick_count: u16,
    pub pad0: u16,
    pub ticks: [Tick; 60],
}
layout!(TickArray, name = "TickArray", size = 3880, disc = disc::TICK_ARRAY, {
    pool: [u8; 32] = 0, start_index: i32 = 32, initialized_tick_count: u16 = 36, pad0: u16 = 38,
    ticks: [Tick; 60] = 40,
});

///
pub const TICK_ARRAY_RENT_OWNER_OFFSET: usize = <TickArray as Layout>::ACCOUNT_LEN;
///
///
///
pub const TICK_ARRAY_LEN: usize = TICK_ARRAY_RENT_OWNER_OFFSET + 32;
///
pub const fn tick_array_len_ok(len: usize) -> bool { len == TICK_ARRAY_LEN }
///
pub fn tick_array_rent_owner(data: &[u8]) -> Option<[u8; 32]> {
    if data.len() != TICK_ARRAY_LEN { return None; }
    let mut k = [0u8; 32];
    k.copy_from_slice(&data[TICK_ARRAY_RENT_OWNER_OFFSET..TICK_ARRAY_LEN]);
    Some(k)
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
///
///
///
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Position {
    pub version: u8,
    pub bump: u8,
    pub pad0: [u8; 6],
    pub pool: [u8; 32],
///
    pub owner: [u8; 32],
    pub salt: u64,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub liquidity: U128,
    pub fee_growth_inside0_last_x64: U128,
    pub fee_growth_inside1_last_x64: U128,
///
    pub position_id: [u8; 32],
///
    pub reserved: [u8; 32],
}
layout!(Position, name = "Position", size = 200, disc = disc::POSITION, {
    version: u8 = 0, bump: u8 = 1, pad0: [u8; 6] = 2, pool: [u8; 32] = 8, owner: [u8; 32] = 40,
    salt: u64 = 72, tick_lower: i32 = 80, tick_upper: i32 = 84, liquidity: U128 = 88,
    fee_growth_inside0_last_x64: U128 = 104, fee_growth_inside1_last_x64: U128 = 120,
    position_id: [u8; 32] = 136, reserved: [u8; 32] = 168,
});

///
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct PoolIndexEntry {
    pub pool: [u8; 32],
    pub created_slot: u64,
    pub bump: u8,
    pub reserved: [u8; 7],
}
layout!(PoolIndexEntry, name = "PoolIndexEntry", size = 48, disc = disc::POOL_INDEX_ENTRY, {
    pool: [u8; 32] = 0, created_slot: u64 = 32, bump: u8 = 40, reserved: [u8; 7] = 41,
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
///
///
///
///
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ProtocolTreasury {
    pub version: u8,
    pub bump: u8,
    pub pad0: [u8; 6],
///
    pub governance: [u8; 32],
///
    pub pending_governance: [u8; 32],
    pub reserved: [u8; 32],
}
layout!(ProtocolTreasury, name = "ProtocolTreasury", size = 104, disc = disc::PROTOCOL_TREASURY, {
    version: u8 = 0, bump: u8 = 1, pad0: [u8; 6] = 2, governance: [u8; 32] = 8,
    pending_governance: [u8; 32] = 40, reserved: [u8; 32] = 72,
});




///
///
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Pod, Zeroable)]
pub struct HookInterfaceHeader {
    pub version: u8,
    pub bump: u8,
    pub max_rounds: u8,
    pub callback_accounts: u8,
    pub flags: u16,
    pub authority_bump: u8,
    pub pad0: u8,
}
const _: () = assert!(core::mem::size_of::<HookInterfaceHeader>() == 8);

impl HookInterfaceHeader {
///
    pub const ACCOUNT_OFFSET: usize = 8;

///
    pub fn from_account_data(data: &[u8]) -> Option<Self> {
        let raw = data.get(Self::ACCOUNT_OFFSET..Self::ACCOUNT_OFFSET + 8)?;
        Some(bytemuck::pod_read_unaligned(raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

///
///
    #[test]
    fn account_lengths_match_the_spec() {
        assert_eq!(AmmConfig::ACCOUNT_LEN, 216);
        assert_eq!(ApprovedHook::ACCOUNT_LEN, 200);
        assert_eq!(MintExclusivity::ACCOUNT_LEN, 80);
        assert_eq!(MintPoolCount::ACCOUNT_LEN, 64);
        assert_eq!(Pool::ACCOUNT_LEN, 640);
        assert_eq!(TickArray::ACCOUNT_LEN, 3888);
        assert_eq!(TICK_ARRAY_RENT_OWNER_OFFSET, 3888);
        assert_eq!(TICK_ARRAY_LEN, 3920);
        assert_eq!(Position::ACCOUNT_LEN, 208);
        assert_eq!(PoolIndexEntry::ACCOUNT_LEN, 56);


        assert_eq!(ProtocolTreasury::ACCOUNT_LEN, 112);
    }

///
///
    #[test]
    fn spec_rent_figures_are_consistent_with_the_lengths() {
        let rent = |len: usize| (128 + len as u64) * 5_080;
        assert_eq!(rent(AmmConfig::ACCOUNT_LEN), 1_747_520);
        assert_eq!(rent(ApprovedHook::ACCOUNT_LEN), 1_666_240);
        assert_eq!(rent(MintExclusivity::ACCOUNT_LEN), 1_056_640);
        assert_eq!(rent(MintPoolCount::ACCOUNT_LEN), 975_360);
        assert_eq!(rent(Pool::ACCOUNT_LEN), 3_901_440);

        assert_eq!(rent(TICK_ARRAY_LEN), 20_563_840);
        assert_eq!(rent(Position::ACCOUNT_LEN), 1_706_880);
        assert_eq!(rent(PoolIndexEntry::ACCOUNT_LEN), 934_720);
    }
    #[test]
    fn bitmap_sizes_match_the_spec() {
        assert_eq!(bitmap_arrays_per_side(60), 124);
        assert_eq!(bitmap_word_count(60), 4);
        assert_eq!(bitmap_account_len(60), 80);
        assert_eq!(bitmap_account_len(10), 240);
        assert_eq!(bitmap_account_len(1), 1_904);
        assert_eq!((128 + bitmap_account_len(60) as u64) * 5_080, 1_056_640);
        assert_eq!((128 + bitmap_account_len(1) as u64) * 5_080, 10_322_560);

        for s in [1u16, 10, 60, 200, 32_767] {
            let lo = tick_array_start_index(crate::constants::MIN_TICK, s);
            let hi = tick_array_start_index(crate::constants::MAX_TICK, s);
            assert!(bitmap_bit(lo, s) >= 0, "spacing {s}");
            assert!((bitmap_bit(hi, s) as usize) < 64 * bitmap_word_count(s), "spacing {s}");
        }
    }

    #[test]
    fn tick_array_start_floors_toward_negative_infinity() {
        assert_eq!(tick_array_start_index(0, 60), 0);
        assert_eq!(tick_array_start_index(3599, 60), 0);
        assert_eq!(tick_array_start_index(3600, 60), 3600);
        assert_eq!(tick_array_start_index(-1, 60), -3600);
        assert_eq!(tick_array_start_index(-3600, 60), -3600);
        assert_eq!(tick_array_start_index(-3601, 60), -7200);
    }

    #[test]
    fn position_owner_is_at_account_offset_48() {
        let f = Position::FIELDS.iter().find(|f| f.name == "owner").unwrap();
        assert_eq!(8 + f.offset, 48);
    }

///
    #[test]
    fn position_id_is_at_account_offset_144_and_is_not_the_owner() {
        let id = Position::FIELDS.iter().find(|f| f.name == "position_id").unwrap();
        assert_eq!((8 + id.offset, id.size), (144, 32));
        let mut p = Position::zeroed();
        p.owner = [7u8; 32];
        p.position_id = [9u8; 32];
        let b = bytemuck::bytes_of(&p);
        assert_eq!(&b[40..72], &[7u8; 32]);
        assert_eq!(&b[136..168], &[9u8; 32]);
    }

///
///
    #[test]
    fn position_still_has_a_reserved_tail() {
        let r = Position::FIELDS.iter().find(|f| f.name == "reserved").expect("Position keeps a reserve");
        assert_eq!((8 + r.offset, r.size), (176, 32));
        assert_eq!(Position::SIZE, 200);

        for fields in [AmmConfig::FIELDS, ApprovedHook::FIELDS, Pool::FIELDS, Position::FIELDS, PoolIndexEntry::FIELDS, ProtocolTreasury::FIELDS] {
            assert!(fields.iter().any(|f| f.name == "reserved"));
        }
    }

///
///
///
///
///
///
    #[test]
    fn the_admitted_base_transfer_hook_is_carved_from_the_reserve_tails() {
        let off = |fields: &[Field], name: &str| {
            let f = fields.iter().find(|f| f.name == name).unwrap_or_else(|| panic!("{name}"));
            (f.offset, f.size)
        };
        assert_eq!(off(ApprovedHook::FIELDS, "allow_exclusivity"), (128, 1), "an earlier field did not move");
        assert_eq!(off(ApprovedHook::FIELDS, "base_transfer_hook"), (129, 32));

        assert_eq!(off(ApprovedHook::FIELDS, "allow_hook_quotes"), (161, 1));
        assert_eq!(off(ApprovedHook::FIELDS, "reserved"), (162, 22));
        assert_eq!(ApprovedHook::SIZE, 192);
        assert_eq!(off(Pool::FIELDS, "paused"), (504, 1), "an earlier field did not move");
        assert_eq!(off(Pool::FIELDS, "base_transfer_hook"), (505, 32));
        assert_eq!(off(Pool::FIELDS, "base_hook_side"), (537, 1));

        assert_eq!(off(Pool::FIELDS, "reserved"), (539, 85));
        assert_eq!(Pool::SIZE, 632);

        let mut p = Pool::zeroed();
        p.base_transfer_hook = [0x5A; 32];
        p.base_hook_side = 1;
        let b = bytemuck::bytes_of(&p);
        assert_eq!(&b[505..537], &[0x5A; 32]);
        assert_eq!(b[537], 1);
        let mut a = ApprovedHook::zeroed();
        a.base_transfer_hook = [0x3C; 32];
        assert_eq!(&bytemuck::bytes_of(&a)[129..161], &[0x3C; 32]);

        assert_eq!(ApprovedHook::zeroed().base_transfer_hook, [0u8; 32]);
    }

///
///
///
///
///
///
///
    #[test]
    fn the_protocol_fee_setting_and_currency_are_carved_from_the_reserve_tails() {
        let off = |fields: &[Field], name: &str| {
            let f = fields.iter().find(|f| f.name == name).unwrap_or_else(|| panic!("{name}"));
            (f.offset, f.size)
        };
        assert_eq!(off(AmmConfig::FIELDS, "quote_caps_admitted"), (176, 1), "an earlier field did not move");
        assert_eq!(off(AmmConfig::FIELDS, "pad2"), (177, 3));
        assert_eq!(off(AmmConfig::FIELDS, "protocol_fee"), (180, 4));
        assert_eq!(off(AmmConfig::FIELDS, "reserved"), (184, 24));
        assert_eq!(AmmConfig::SIZE, 208);
        assert_eq!(off(Pool::FIELDS, "protocol_fee"), (16, 4), "the v4 slot did not move");
        assert_eq!(off(Pool::FIELDS, "protocol_fee_currency"), (538, 1));
        assert_eq!(off(Pool::FIELDS, "reserved"), (539, 85));

        let mut c = AmmConfig::zeroed();
        c.protocol_fee = crate::constants::PROTOCOL_FEE_DEFAULT_PACKED;
        let b = bytemuck::bytes_of(&c);
        assert_eq!(&b[180..184], &2_048_500u32.to_le_bytes());
        let mut p = Pool::zeroed();
        p.protocol_fee_currency = crate::constants::PROTOCOL_FEE_ON_CURRENCY0;
        assert_eq!(bytemuck::bytes_of(&p)[538], 1);

        assert_eq!(Pool::zeroed().protocol_fee_currency, crate::constants::PROTOCOL_FEE_ON_INPUT);
    }

///
///
///
///
    #[test]
    fn the_pause_guardian_is_carved_from_dead_and_reserved_bytes() {
        let off = |fields: &[Field], name: &str| {
            let f = fields.iter().find(|f| f.name == name).unwrap_or_else(|| panic!("{name}"));
            (f.offset, f.size)
        };
        assert_eq!(off(AmmConfig::FIELDS, "protocol_fee_controller"), (72, 32), "an earlier field did not move");
        assert_eq!(off(AmmConfig::FIELDS, "pause_guardian"), (104, 32));
        assert_eq!(off(AmmConfig::FIELDS, "pad1"), (136, 40));
        assert_eq!(off(AmmConfig::FIELDS, "quote_caps_admitted"), (176, 1), "a later field did not move");
        assert_eq!(AmmConfig::SIZE, 208);
        assert_eq!(off(ApprovedHook::FIELDS, "guardian_paused_until"), (184, 8));
        assert_eq!(ApprovedHook::SIZE, 192);
        assert_eq!(off(Pool::FIELDS, "protocol_fee_currency"), (538, 1), "an earlier field did not move");
        assert_eq!(off(Pool::FIELDS, "guardian_paused_until"), (624, 8));
        assert_eq!(Pool::SIZE, 632);
        let mut p = Pool::zeroed();
        p.guardian_paused_until = 0x0102_0304_0506_0708;
        assert_eq!(&bytemuck::bytes_of(&p)[624..632], &0x0102_0304_0506_0708i64.to_le_bytes());
        let mut c = AmmConfig::zeroed();
        c.pause_guardian = [0x6A; 32];
        assert_eq!(&bytemuck::bytes_of(&c)[104..136], &[0x6A; 32]);
    }

///
///
    #[test]
    fn a_hook_listed_quote_records_its_hook_in_the_old_reserve() {
        let f = QuoteMint::FIELDS.iter().find(|f| f.name == "hook_program").expect("hook_program");
        assert_eq!((f.offset, f.size), (88, 32));
        assert_eq!(QuoteMint::SIZE, 120);
        assert!(QuoteMint::FIELDS.iter().all(|f| f.name != "reserved"));
        assert_eq!(QUOTE_STATE_HOOK_LISTED, 3);
        assert_ne!(QUOTE_STATE_HOOK_LISTED, QUOTE_STATE_LISTED);
        assert_ne!(QUOTE_STATE_HOOK_LISTED, QUOTE_STATE_RETIRED);
    }

///
///
///
    #[test]
    fn the_protocol_treasury_keys_sit_at_their_account_offsets() {
        let off = |name: &str| {
            let f = ProtocolTreasury::FIELDS.iter().find(|f| f.name == name).unwrap_or_else(|| panic!("{name}"));
            (8 + f.offset, f.size)
        };
        assert_eq!(off("governance"), (16, 32));
        assert_eq!(off("pending_governance"), (48, 32));
        assert_eq!(off("reserved"), (80, 32));
        let mut t = ProtocolTreasury::zeroed();
        t.governance = [0x11; 32];
        t.pending_governance = [0x22; 32];
        let b = bytemuck::bytes_of(&t);
        assert_eq!(&b[8..40], &[0x11; 32]);
        assert_eq!(&b[40..72], &[0x22; 32]);
        assert_ne!(ProtocolTreasury::DISCRIMINATOR, AmmConfig::DISCRIMINATOR);
    }

///
///
///
///
    #[test]
    fn hook_interface_header_is_the_first_eight_bytes_after_the_discriminator() {
        let mut data = vec![0u8; 8];
        data.extend_from_slice(&[1, 254, 3, 1, 0xCC, 0x60, 253, 0]);
        data.extend_from_slice(&[0xAA; 224]);
        let h = HookInterfaceHeader::from_account_data(&data).unwrap();
        assert_eq!(
            h,
            HookInterfaceHeader { version: 1, bump: 254, max_rounds: 3, callback_accounts: 1, flags: 0x60CC, authority_bump: 253, pad0: 0 }
        );
        assert!(HookInterfaceHeader::from_account_data(&data[..15]).is_none());
    }
    #[test]
    fn u128_fields_write_little_endian_at_their_offsets() {
        let mut p = Pool::zeroed();
        p.sqrt_price_x64 = U128::new(crate::constants::MAX_SQRT_PRICE_X64);
        p.hook_claims1 = 0xDEAD_BEEF;
        let bytes = bytemuck::bytes_of(&p);
        assert_eq!(&bytes[280..296], &crate::constants::MAX_SQRT_PRICE_X64.to_le_bytes());
        assert_eq!(&bytes[368..376], &0xDEAD_BEEFu64.to_le_bytes());
    }
}
