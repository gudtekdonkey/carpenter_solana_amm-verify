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

use borsh::{BorshDeserialize, BorshSerialize};

///
#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RegisterPoolArgs {
///
    pub amm_pool: [u8; 32],
    pub base_mint: [u8; 32],
///
    pub quote_mint: [u8; 32],
///
    pub pool_initializer: [u8; 32],
    pub mode: u8,
    pub skim_buy_far_bps: u16,
    pub skim_sell_far_bps: u16,
    pub creator_buy_bps: u16,
    pub creator_sell_bps: u16,
///
    pub holder_buy_bps: u16,
///
    pub holder_sell_bps: u16,
///
///
///
    pub deployer: [u8; 32],
///
    pub pad: [u8; 32],
    pub min_bid: u64,

    pub min_sellable: u64,
    pub platform_bps: u16,
///
///
    pub max_bounty: u64,
///
    pub bounty_bps: u16,

///
///
///
///
///
    pub referrer: [u8; 32],
}

///
///
pub const REGISTER_POOL_ARGS_LEN: usize = 128 + 1 + 12 + 64 + 16 + 2 + 8 + 2 + 32;

///
///
///
///
#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReferralTiersArg {
    pub referrer_t2: [u8; 32],
    pub referrer_t3: [u8; 32],
}

///
pub const REFERRAL_TIERS_ARG_LEN: usize = 64;

///
///
///
///
#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TeamFeeArg {
    pub team_fee_bps: u16,
}

///
pub const TEAM_FEE_ARG_LEN: usize = 2;

///
///
///
///
///
///
#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SkimSellMultipleArg {
    pub skim_sell_at_floor_multiple_bps: u32,
}

///
pub const SKIM_SELL_MULTIPLE_ARG_LEN: usize = 4;

///
///
///
#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StairArg {
    pub steps: u8,
    pub growth_bps: u16,
}

///
pub const STAIR_ARG_LEN: usize = 3;




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
pub fn referral_tiers_well_formed(creator: &[u8; 32], t1: &[u8; 32], t2: &[u8; 32], t3: &[u8; 32]) -> bool {
    const Z: [u8; 32] = [0u8; 32];
    let (s1, s2, s3) = (*t1 != Z, *t2 != Z, *t3 != Z);
    if (s2 && !s1) || (s3 && !s2) {
        return false;
    }
    if (s1 && t1 == creator) || (s2 && t2 == creator) || (s3 && t3 == creator) {
        return false;
    }
    if (s2 && t2 == t1) || (s3 && (t3 == t1 || t3 == t2)) {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_and_round_trip() {
        let a = RegisterPoolArgs {
            amm_pool: [1; 32],
            base_mint: [2; 32],
            quote_mint: crate::keys::WSOL_MINT,
            pool_initializer: [3; 32],
            mode: crate::constants::MODE_CUSTOM,
            skim_buy_far_bps: 250,
            skim_sell_far_bps: 250,
            creator_buy_bps: 25,
            creator_sell_bps: 25,
            holder_buy_bps: 0,
            holder_sell_bps: 0,
            deployer: [4; 32],
            pad: [0; 32],
            min_bid: crate::constants::MIN_BID_LAMPORTS,
            min_sellable: crate::constants::MIN_SELLABLE,
            platform_bps: crate::constants::PLATFORM_BPS,
            max_bounty: crate::constants::MAX_BOUNTY_LAMPORTS,
            bounty_bps: crate::constants::BOUNTY_BPS,
            referrer: [7; 32],
        };
        let bytes = borsh::to_vec(&a).unwrap();
        assert_eq!(bytes.len(), REGISTER_POOL_ARGS_LEN);
        assert_eq!(REGISTER_POOL_ARGS_LEN, 265);

        assert_eq!(&bytes[213..221], &crate::constants::MIN_SELLABLE.to_le_bytes());
        assert_eq!(&bytes[221..223], &crate::constants::PLATFORM_BPS.to_le_bytes());
        assert_eq!(&bytes[223..231], &crate::constants::MAX_BOUNTY_LAMPORTS.to_le_bytes());
        assert_eq!(&bytes[231..233], &crate::constants::BOUNTY_BPS.to_le_bytes());
        assert_eq!(&bytes[233..265], &[7u8; 32]);
        assert_eq!(borsh::from_slice::<RegisterPoolArgs>(&bytes).unwrap(), a);
    }

///
///
///
    #[test]
    fn the_trailing_arguments_are_required_and_in_order() {
        let tiers = ReferralTiersArg { referrer_t2: [2; 32], referrer_t3: [3; 32] };
        let team = TeamFeeArg { team_fee_bps: crate::constants::TEAM_POOL_FEE_BPS_MAX };
        let skim = SkimSellMultipleArg { skim_sell_at_floor_multiple_bps: crate::constants::PRESET_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS };
        let stair = StairArg { steps: 9, growth_bps: 9_500 };
        assert_eq!(borsh::to_vec(&tiers).unwrap().len(), REFERRAL_TIERS_ARG_LEN);
        assert_eq!(&borsh::to_vec(&tiers).unwrap()[..32], &[2u8; 32], "t2 first");
        assert_eq!(borsh::to_vec(&team).unwrap(), 50u16.to_le_bytes().to_vec());
        assert_eq!(borsh::to_vec(&skim).unwrap(), vec![0x98, 0x3A, 0, 0], "15,000 LE (the presets at 1.5x since 2026-10-12; ~~50,000~~)");
        assert_eq!(borsh::to_vec(&stair).unwrap(), vec![9, 0x1c, 0x25]);
        let mut data = borsh::to_vec(&RegisterPoolArgs::default()).unwrap();
        for part in [borsh::to_vec(&tiers).unwrap(), borsh::to_vec(&team).unwrap(), borsh::to_vec(&skim).unwrap(), borsh::to_vec(&stair).unwrap()] {
            data.extend_from_slice(&part);
        }
        assert_eq!(data.len(), REGISTER_POOL_ARGS_LEN + REFERRAL_TIERS_ARG_LEN + TEAM_FEE_ARG_LEN + SKIM_SELL_MULTIPLE_ARG_LEN + STAIR_ARG_LEN);
        assert_eq!(data.len(), 338, "+ the 8-byte discriminator = 346 (347 with the optional owner kind)");
        let mut r = &data[..];
        assert_eq!(<RegisterPoolArgs as BorshDeserialize>::deserialize(&mut r).unwrap(), RegisterPoolArgs::default());
        assert_eq!(<ReferralTiersArg as BorshDeserialize>::deserialize(&mut r).unwrap(), tiers);
        assert_eq!(<TeamFeeArg as BorshDeserialize>::deserialize(&mut r).unwrap(), team);
        assert_eq!(<SkimSellMultipleArg as BorshDeserialize>::deserialize(&mut r).unwrap(), skim);
        assert_eq!(<StairArg as BorshDeserialize>::deserialize(&mut r).unwrap(), stair);
        assert!(r.is_empty());

        let mut short = &data[REGISTER_POOL_ARGS_LEN..REGISTER_POOL_ARGS_LEN + 63];
        assert!(<ReferralTiersArg as BorshDeserialize>::deserialize(&mut short).is_err());
        let mut none: &[u8] = &[];
        assert!(<StairArg as BorshDeserialize>::deserialize(&mut none).is_err(), "0 bytes is no longer 'not given'");
    }

///
    #[test]
    fn the_default_referrer_is_zero_so_an_unsaid_launch_is_unreferred() {
        let d = RegisterPoolArgs::default();
        assert_eq!(d.referrer, [0u8; 32], "ZERO is the safe default");
        let bytes = borsh::to_vec(&d).unwrap();
        assert_eq!(bytes.len(), 265);
        assert!(bytes[233..].iter().all(|&b| b == 0));
        assert!(borsh::from_slice::<RegisterPoolArgs>(&bytes[..264]).is_err(), "short: refused, never padded");
    }

///
    #[test]
    fn the_tier_checks_are_the_specs() {
        let (c, a, b, d, z) = ([9u8; 32], [1u8; 32], [2u8; 32], [3u8; 32], [0u8; 32]);
        assert!(referral_tiers_well_formed(&c, &z, &z, &z), "unreferred");
        assert!(referral_tiers_well_formed(&c, &a, &z, &z), "one tier");
        assert!(referral_tiers_well_formed(&c, &a, &b, &z), "two tiers");
        assert!(referral_tiers_well_formed(&c, &a, &b, &d), "three tiers");
        assert!(!referral_tiers_well_formed(&c, &z, &b, &z), "gap: t2 without t1");
        assert!(!referral_tiers_well_formed(&c, &a, &z, &d), "gap: t3 without t2");
        assert!(!referral_tiers_well_formed(&c, &z, &z, &d), "gap: t3 alone");
        assert!(!referral_tiers_well_formed(&c, &c, &z, &z), "t1 is the creator");
        assert!(!referral_tiers_well_formed(&c, &a, &c, &z), "t2 is the creator");
        assert!(!referral_tiers_well_formed(&c, &a, &b, &c), "t3 is the creator");
        assert!(!referral_tiers_well_formed(&c, &a, &a, &z), "t2 == t1");
        assert!(!referral_tiers_well_formed(&c, &a, &b, &a), "t3 == t1");
        assert!(!referral_tiers_well_formed(&c, &a, &b, &b), "t3 == t2");
    }
}
