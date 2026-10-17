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

#![allow(unsafe_code)]

///
///
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct PodU128(pub [u64; 2]);

impl PodU128 {
    pub const fn new(v: u128) -> Self {
        PodU128([v as u64, (v >> 64) as u64])
    }
    pub const fn get(self) -> u128 {
        self.0[0] as u128 | ((self.0[1] as u128) << 64)
    }
    pub fn set(&mut self, v: u128) {
        *self = PodU128::new(v);
    }
}

impl core::fmt::Debug for PodU128 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.get())
    }
}


unsafe impl bytemuck::Zeroable for PodU128 {}
unsafe impl bytemuck::Pod for PodU128 {}

///
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct FloorState {
            pub version: u8,
            pub bump: u8,
            pub pool_bound: u8,
            pub quote_is_token1: u8,
            pub ladder_seeded: u8,
            pub needs_ratchet: u8,
            pub mode: u8,
///
///
///
///
            pub quote_suspended: u8,
            pub tick_spacing: i32,
             pub floor_tick: i32,
             pub amm_pool: [u8; 32],
             pub pool_initializer: [u8; 32],
             pub base_mint: [u8; 32],
              pub quote_mint: [u8; 32],
              pub pad9: [u8; 32],
              pub pad10: [u8; 32],
              pub skim_buy_far_bps: u16,
              pub skim_sell_far_bps: u16,
              pub creator_buy_bps: u16,
              pub creator_sell_bps: u16,
              pub holder_buy_bps: u16,
              pub holder_sell_bps: u16,
///
///
///
///
///
              pub wall_ticks: u16,
              pub platform_bps: u16,
              pub min_bid: u64,
              pub min_sellable: u64,
///
///
///
///
///
///
///
              pub owner_owed: u64,
              pub owner_paid: u64,
              pub vault_eth: u64,
              pub bid_eth_deployed: u64,
              pub unpaid_skim: u64,
              pub cumulative_volume_eth: u64,
              pub inventory_tokens: u64,
              pub inventory_cost_eth: u64,
              pub ask_tokens_deployed: u64,
              pub ladder_tokens_remaining: u64,
              pub bid_lower: i32,
              pub ask_upper: i32,
              pub ask_width: i32,
              pub seed_lower: i32,
              pub seed_upper: i32,
              pub ladder_lower: i32,
              pub ladder_upper: i32,
///
///
///
///
///
              pub wall_split_tick: i32,
              pub bid_liquidity: PodU128,
              pub ask_liquidity: PodU128,
              pub seed_liq: PodU128,
              pub ladder_liq: PodU128,
              pub creator_fees_owed: u64,
              pub platform_owed: u64,
              pub holder_owed: u64,
///
///
///
///
///
              pub wall_tail_liq: PodU128,
///
///
///
///
///
              pub wall_held_tokens: u64,
              pub seed_fees_swept_eth: u64,
              pub seed_fees_swept_tokens: u64,
              pub ladder_proceeds_eth: u64,
              pub platform_paid_eth: u64,
              pub platform_swept: u64,
              pub creator_fees_paid: u64,
              pub holder_paid_eth: u64,
              pub last_ratchet_slot: u64,


///
///
///
              pub bid_width: i32,
///
///
///
///
              pub skim_buy_at_floor_bps: u16,
              pub skim_sell_at_floor_bps: u16,
///
              pub pad6: [u8; 32],
              pub max_bounty: u64,
              pub bounty_bps: u16,
///
///
///
///
              pub wall_band_ticks: u16,
              pub wall_tail_bps: u16,
              pub wall_list_bps: u16,















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
              pub referral_owed: u64,
///
///
///
              pub referral_fees_paid: u64,
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
              pub trader_ref_owed: u64,
///
              pub trader_ref_booked: u64,
              pub pad7: [u8; 16],
///
///
///
///
///
///
///
///
              pub referral_share_bps: u16,
///
///
///
///
///
              pub stair_steps: u8,
              pub stair_ready: u8,
              pub stair_growth_bps: u16,
///
///
              pub stair_kind: u8,
///
              pub stair_owner: u8,





///
///
///
              pub referrer: [u8; 32],
              pub pad8: u8,
///
              pub holders_hooked: u8,
              pub pad4: [u8; 6],










///
///
              pub referrer_t2: [u8; 32],
///
              pub referrer_t3: [u8; 32],
///
              pub referral_paid_t2: u64,
///
              pub referral_paid_t3: u64,



///
///
              pub team_owed: u64,
///
              pub team_fees_paid: u64,
///
///
///
              pub team_fee_bps: u16,
              pub pad5: [u8; 6],
}

///
pub const FLOOR_STATE_SIZE: usize = 784;
#[allow(clippy::manual_is_multiple_of)]
const _: () = assert!(
    core::mem::size_of::<FloorState>() == FLOOR_STATE_SIZE
        && core::mem::size_of::<FloorState>() % 8 == 0
        && core::mem::align_of::<FloorState>() <= 8
);
const _: () = assert!(core::mem::size_of::<PodU128>() == 16 && core::mem::align_of::<PodU128>() == 8);




unsafe impl bytemuck::Zeroable for FloorState {}
unsafe impl bytemuck::Pod for FloorState {}

impl Default for FloorState {
    fn default() -> Self {
        bytemuck::Zeroable::zeroed()
    }
}

impl core::fmt::Debug for FloorState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FloorState")
            .field("tick_spacing", &self.tick_spacing)
            .field("floor_tick", &self.floor_tick)
            .field("vault_eth", &self.vault_eth)
            .field("inventory_tokens", &self.inventory_tokens)
            .field("ladder_tokens_remaining", &self.ladder_tokens_remaining)
            .field("wall_held_tokens", &self.wall_held_tokens)
            .field("wall_tail_liq", &self.wall_tail_liq)
            .field("seed_liq", &self.seed_liq)
            .field("platform_owed", &self.platform_owed)
            .field("creator_fees_owed", &self.creator_fees_owed)
            .finish_non_exhaustive()
    }
}

impl FloorState {
///
///
    pub fn from_bytes(data: &[u8]) -> Option<&FloorState> {
        bytemuck::try_from_bytes(data).ok()
    }

    pub fn from_bytes_mut(data: &mut [u8]) -> Option<&mut FloorState> {
        bytemuck::try_from_bytes_mut(data).ok()
    }

///
    pub fn read_unaligned(data: &[u8]) -> Option<FloorState> {
        bytemuck::try_pod_read_unaligned(data).ok()
    }

    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
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
///
///
///
///
///
///
    pub fn quote_liabilities(&self) -> u128 {
        self.vault_eth as u128
            + self.creator_fees_owed as u128
            + self.platform_owed as u128
            + self.holder_owed as u128


            + self.referral_owed as u128


            + self.team_owed as u128




            + self.owner_owed as u128



            + self.owner_pending() as u128



            + self.trader_ref_owed as u128
    }

///
///
///
///
///
///
///
    pub fn owner_pending(&self) -> u64 {
        u64::from_le_bytes([
            self.pad6[0], self.pad6[1], self.pad6[2], self.pad6[3], self.pad6[4], self.pad6[5], self.pad6[6], self.pad6[7],
        ])
    }

///
    pub fn set_owner_pending(&mut self, v: u64) {
        self.pad6[..8].copy_from_slice(&v.to_le_bytes());
    }




///
///
///
///
    pub fn is_stair(&self) -> bool {
        self.stair_steps != 0
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
    pub fn base_liabilities(&self) -> u128 {
        self.inventory_tokens as u128
            + if self.is_stair() { 0 } else { self.ladder_tokens_remaining as u128 }
            + self.wall_held_tokens as u128
            + self.owner_held() as u128
    }

///
///
///
///
///
    pub fn owner_held(&self) -> u64 {
        self.ask_tokens_deployed
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use core::mem::offset_of;


    #[test]
    fn offsets_match_spec_4_2() {
        macro_rules! at {
            ($f:ident, $o:expr) => {
                assert_eq!(offset_of!(FloorState, $f), $o, stringify!($f));
            };
        }
        at!(version, 0); at!(bump, 1); at!(pool_bound, 2); at!(quote_is_token1, 3);
        at!(ladder_seeded, 4); at!(needs_ratchet, 5); at!(mode, 6); at!(quote_suspended, 7);
        at!(tick_spacing, 8); at!(floor_tick, 12);
        at!(amm_pool, 16); at!(pool_initializer, 48); at!(base_mint, 80); at!(quote_mint, 112);
        at!(pad9, 144); at!(pad10, 176);
        at!(skim_buy_far_bps, 208); at!(skim_sell_far_bps, 210); at!(creator_buy_bps, 212); at!(creator_sell_bps, 214);
        at!(holder_buy_bps, 216); at!(holder_sell_bps, 218); at!(wall_ticks, 220); at!(platform_bps, 222);
        at!(min_bid, 224); at!(min_sellable, 232); at!(owner_owed, 240); at!(owner_paid, 248);
        at!(vault_eth, 256); at!(bid_eth_deployed, 264); at!(unpaid_skim, 272); at!(cumulative_volume_eth, 280);
        at!(inventory_tokens, 288); at!(inventory_cost_eth, 296); at!(ask_tokens_deployed, 304); at!(ladder_tokens_remaining, 312);
        at!(bid_lower, 320); at!(ask_upper, 324); at!(ask_width, 328); at!(seed_lower, 332);
        at!(seed_upper, 336); at!(ladder_lower, 340); at!(ladder_upper, 344); at!(wall_split_tick, 348);
        at!(bid_liquidity, 352); at!(ask_liquidity, 368); at!(seed_liq, 384); at!(ladder_liq, 400);
        at!(creator_fees_owed, 416); at!(platform_owed, 424); at!(holder_owed, 432); at!(wall_tail_liq, 440);
        at!(wall_held_tokens, 456); at!(seed_fees_swept_eth, 464); at!(seed_fees_swept_tokens, 472);
        at!(ladder_proceeds_eth, 480); at!(platform_paid_eth, 488); at!(platform_swept, 496); at!(creator_fees_paid, 504);
        at!(holder_paid_eth, 512); at!(last_ratchet_slot, 520); at!(bid_width, 528);
        at!(skim_buy_at_floor_bps, 532); at!(skim_sell_at_floor_bps, 534); at!(pad6, 536);
        at!(max_bounty, 568); at!(bounty_bps, 576);

        at!(wall_band_ticks, 578); at!(wall_tail_bps, 580); at!(wall_list_bps, 582);



        at!(referral_owed, 584); at!(referral_fees_paid, 592); at!(trader_ref_owed, 600); at!(trader_ref_booked, 608); at!(pad7, 616);
        at!(referral_share_bps, 632); at!(stair_steps, 634); at!(stair_ready, 635); at!(stair_growth_bps, 636); at!(stair_kind, 638); at!(stair_owner, 639);




        at!(referrer, 640); at!(pad8, 672); at!(holders_hooked, 673); at!(pad4, 674);


        at!(referrer_t2, 680); at!(referrer_t3, 712); at!(referral_paid_t2, 744); at!(referral_paid_t3, 752);


        at!(team_owed, 760); at!(team_fees_paid, 768); at!(team_fee_bps, 776); at!(pad5, 778);
        assert_eq!(778 + 6, FLOOR_STATE_SIZE);
    }

///
///
///
///
///
///
    #[test]
    fn quote_liabilities_counts_every_idle_lamport_including_the_referral() {
        let mut s = FloorState::default();
        assert_eq!(s.quote_liabilities(), 0);
        s.vault_eth = 1;
        s.creator_fees_owed = 2;
        s.platform_owed = 4;
        s.holder_owed = 8;
        s.referral_owed = 16;
        s.team_owed = 32;

        assert_eq!(s.quote_liabilities(), 63, "a missing term shows up as a missing bit");



        let mut before = FloorState::default();
        before.creator_fees_owed = 100;
        let mut after = FloorState::default();
        after.creator_fees_owed = 90;
        after.referral_owed = 10;
        assert_eq!(before.quote_liabilities(), after.quote_liabilities());


        let mut big = FloorState::default();
        big.vault_eth = u64::MAX;
        big.creator_fees_owed = u64::MAX;
        big.platform_owed = u64::MAX;
        big.holder_owed = u64::MAX;
        big.referral_owed = u64::MAX;
        big.team_owed = u64::MAX;
        assert_eq!(big.quote_liabilities(), 6 * u64::MAX as u128);
    }

///
///
///
///
    #[test]
    fn trader_ref_owed_is_a_liability_and_the_move_out_of_team_owed_keeps_the_sum() {
        let mut s = FloorState::default();
        s.trader_ref_owed = 128;
        s.trader_ref_booked = 1 << 40;
        assert_eq!(s.quote_liabilities(), 128, "owed counts; the booked meter does not");
        let mut before = FloorState::default();
        before.team_owed = 1_000;
        let mut after = FloorState::default();
        after.team_owed = 630;
        after.trader_ref_owed = 370;
        assert_eq!(before.quote_liabilities(), after.quote_liabilities());
        let mut big = FloorState::default();
        big.team_owed = u64::MAX;
        big.trader_ref_owed = u64::MAX;
        assert_eq!(big.quote_liabilities(), 2 * u64::MAX as u128);
    }

///
///
    #[test]
    fn owner_owed_is_a_liability() {
        let mut s = FloorState::default();
        s.owner_owed = 64;
        assert_eq!(s.quote_liabilities(), 64);
        s.stair_owner = 1;
        assert_eq!(s.quote_liabilities(), 64);
    }

///
///
    #[test]
    fn owner_pending_is_a_liability_in_pad6() {
        let mut s = FloorState::default();
        s.set_owner_pending(0x0102_0304_0506_0708);
        assert_eq!(&s.pad6[..8], &0x0102_0304_0506_0708u64.to_le_bytes());
        assert_eq!(&s.pad6[8..], &[0u8; 24]);
        assert_eq!(s.owner_pending(), 0x0102_0304_0506_0708);
        assert_eq!(s.quote_liabilities(), 0x0102_0304_0506_0708);
        s.owner_owed = 1;
        s.set_owner_pending(2);
        assert_eq!(s.quote_liabilities(), 3);
    }

    #[test]
    fn u128_fields_are_little_endian_low_word_first() {
        let mut s = FloorState::default();
        s.seed_liq.set(0x0102_0304_0506_0708_1112_1314_1516_1718);
        let bytes = s.as_bytes();
        assert_eq!(&bytes[384..392], &0x1112_1314_1516_1718u64.to_le_bytes());
        assert_eq!(&bytes[392..400], &0x0102_0304_0506_0708u64.to_le_bytes());
        assert_eq!(PodU128::new(u128::MAX).get(), u128::MAX);
    }

    #[test]
    fn round_trips_through_bytes() {
        let mut s = FloorState::default();
        s.floor_tick = -123_456;
        s.vault_eth = 42;
        s.ladder_liq.set(u128::MAX - 5);
        s.bid_width = 120;
        s.skim_sell_at_floor_bps = 1000;
        s.pad6 = [7; 32];


        s.trader_ref_owed = 8;
        s.trader_ref_booked = 8;
        s.pad7 = [8; 16];
        s.referral_owed = 13;

        s.referrer = [10; 32];
        s.pad8 = 1;
        s.holders_hooked = 1;

        s.referrer_t2 = [11; 32];
        s.referrer_t3 = [12; 32];
        s.referral_paid_t2 = 14;
        s.referral_paid_t3 = 15;

        s.team_owed = 21;
        s.team_fee_bps = 25;
        s.pad5[5] = 9;

        s.wall_split_tick = -67_920;
        s.wall_tail_liq.set(u128::MAX - 7);
        s.wall_held_tokens = 150_000_000_000_000;
        s.wall_band_ticks = 10_987;
        s.wall_tail_bps = 1_000;
        s.wall_list_bps = 7_500;
        let copy = FloorState::read_unaligned(s.as_bytes()).unwrap();
        assert_eq!(copy, s);
        assert!(FloorState::read_unaligned(&s.as_bytes()[..783]).is_none());
    }

///
///
///
///
    #[test]
    fn base_liabilities_count_held_and_never_a_staircases_standing_tokens() {
        let mut s = FloorState::default();
        assert_eq!(s.base_liabilities(), 0);
        s.inventory_tokens = 1;
        s.ladder_tokens_remaining = 2;
        s.wall_held_tokens = 4;
        assert_eq!(s.base_liabilities(), 7, "not a staircase: inventory + ladder + held");
        s.stair_steps = 3;
        assert_eq!(s.base_liabilities(), 5, "a staircase: its ladder count stands in its positions");
        let mut big = FloorState::default();
        big.inventory_tokens = u64::MAX;
        big.ladder_tokens_remaining = u64::MAX;
        big.wall_held_tokens = u64::MAX;
        assert_eq!(big.base_liabilities(), 3 * u64::MAX as u128);
    }
}
