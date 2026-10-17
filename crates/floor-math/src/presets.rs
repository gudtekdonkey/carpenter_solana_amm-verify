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

use crate::constants::{MAX_SKIM_SELL_AT_FLOOR_BPS, MAX_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS, MIN_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS, PRESET_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS};
use crate::modes::{preset, split_tokens, Mode};
use crate::owners::floor_types_team_buy::{BLUECHIP_TEAM_BUY_MAX_BPS, TEAM_BUY_MAX_BPS};
use crate::policy::{MAX_CREATOR_BPS, MAX_SKIM_FAR_BPS};
use crate::stair::MAX_STAIR_STEPS;

///
pub const MAX_PRESET_ID: u8 = 31;
///
pub const SEEDED_PRESETS: u8 = 3;
///
pub const MIN_PRESET_STAIR_GROWTH_BPS: u16 = 5_000;
pub const MAX_PRESET_STAIR_GROWTH_BPS: u16 = 20_000;
///
///
pub const TEAM_VEST_DAYS_MIN: u16 = 10;
pub const TEAM_VEST_DAYS_MAX: u16 = 365;
///
///
pub const TEAM_VEST_DAYS_COOK_MOONSHOT: [u16; 4] = [10, 20, 40, 80];
pub const TEAM_VEST_DAYS_BLUECHIP: [u16; 4] = [30, 60, 90, 120];

///
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PresetRow {
///
    pub graduation_mcap_usd: u64,
///
    pub creator_bps: u16,
    pub skim_buy_far_bps: u16,
    pub skim_sell_far_bps: u16,
///
    pub holder_bps: u16,
    pub sale_bps: u16,
    pub pol_bps: u16,
    pub skim_sell_at_floor_multiple_bps: u32,
    pub stair_steps: u8,
    pub stair_growth_bps: u16,
    pub team_buy_max_bps: u16,
    pub team_vest_days: [u16; 4],
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PresetField {
    GraduationMcapUsd,
    CreatorBps,
    SkimBuyFarBps,
    SkimSellFarBps,
    HolderBps,
    SaleBps,
    PolBps,
    SkimSellAtFloorMultipleBps,
    StairSteps,
    StairGrowthBps,
    TeamBuyMaxBps,
    TeamVestDays,
}

impl PresetField {
///
    pub const fn name(self) -> &'static str {
        match self {
            PresetField::GraduationMcapUsd => "graduation_mcap_usd",
            PresetField::CreatorBps => "creator_bps",
            PresetField::SkimBuyFarBps => "skim_buy_far_bps",
            PresetField::SkimSellFarBps => "skim_sell_far_bps",
            PresetField::HolderBps => "holder_bps",
            PresetField::SaleBps => "sale_bps",
            PresetField::PolBps => "pol_bps",
            PresetField::SkimSellAtFloorMultipleBps => "skim_sell_at_floor_multiple_bps",
            PresetField::StairSteps => "stair_steps",
            PresetField::StairGrowthBps => "stair_growth_bps",
            PresetField::TeamBuyMaxBps => "team_buy_max_bps",
            PresetField::TeamVestDays => "team_vest_days",
        }
    }
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
pub fn validate_preset_row(r: &PresetRow) -> Result<(), PresetField> {
    use PresetField::*;
    if r.graduation_mcap_usd == 0 {
        return Err(GraduationMcapUsd);
    }
    if r.creator_bps > MAX_CREATOR_BPS {
        return Err(CreatorBps);
    }
    if r.skim_buy_far_bps > MAX_SKIM_FAR_BPS {
        return Err(SkimBuyFarBps);
    }
    if r.skim_sell_far_bps > MAX_SKIM_FAR_BPS {
        return Err(SkimSellFarBps);
    }
    if r.holder_bps != 0 {
        return Err(HolderBps);
    }
    if split_tokens(r.sale_bps, r.pol_bps).is_err() {


        let min = crate::modes::MIN_ZONE_BPS;
        let sale_alone_bad = r.sale_bps < min || r.sale_bps as u64 > crate::constants::BPS - 2 * min as u64;
        return Err(if sale_alone_bad { SaleBps } else { PolBps });
    }
    let m = r.skim_sell_at_floor_multiple_bps as u64;
    if !(MIN_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS..=MAX_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS).contains(&m)
        || crate::constants::skim_sell_at_floor_bps_at(r.skim_sell_far_bps, m) as u64
            > MAX_SKIM_SELL_AT_FLOOR_BPS
    {
        return Err(SkimSellAtFloorMultipleBps);
    }
    if !(1..=MAX_STAIR_STEPS).contains(&r.stair_steps) {
        return Err(StairSteps);
    }
    if !(MIN_PRESET_STAIR_GROWTH_BPS..=MAX_PRESET_STAIR_GROWTH_BPS).contains(&r.stair_growth_bps) {
        return Err(StairGrowthBps);
    }
    if r.team_buy_max_bps as u64 > BLUECHIP_TEAM_BUY_MAX_BPS {
        return Err(TeamBuyMaxBps);
    }
    let d = r.team_vest_days;
    if d.iter().any(|x| !(TEAM_VEST_DAYS_MIN..=TEAM_VEST_DAYS_MAX).contains(x)) || d.windows(2).any(|w| w[0] >= w[1]) {
        return Err(TeamVestDays);
    }
    Ok(())
}

///
///
pub fn seed_row(id: u8) -> Option<PresetRow> {
    let mode = match id {
        1..=SEEDED_PRESETS => Mode::try_from(id).ok()?,
        _ => return None,
    };
    let p = preset(mode);
    let (team_buy_max_bps, team_vest_days) = match mode {
        Mode::Bluechip => (BLUECHIP_TEAM_BUY_MAX_BPS as u16, TEAM_VEST_DAYS_BLUECHIP),
        _ => (TEAM_BUY_MAX_BPS as u16, TEAM_VEST_DAYS_COOK_MOONSHOT),
    };
    Some(PresetRow {
        graduation_mcap_usd: p.graduation_mcap_usd,
        creator_bps: p.creator_bps,
        skim_buy_far_bps: p.skim_buy_far_bps,
        skim_sell_far_bps: p.skim_sell_far_bps,
        holder_bps: p.holder_bps,
        sale_bps: p.sale_bps,
        pol_bps: p.pol_bps,
        skim_sell_at_floor_multiple_bps: p.skim_sell_at_floor_multiple_bps,
        stair_steps: p.stair_steps,
        stair_growth_bps: p.stair_growth_bps,
        team_buy_max_bps,
        team_vest_days,
    })
}



const _: () = assert!(PRESET_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS >= MIN_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS && PRESET_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS <= MAX_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS);
const _: () = assert!((MAX_PRESET_ID as usize) < 32 && SEEDED_PRESETS < MAX_PRESET_ID);

#[cfg(test)]
mod tests {
    use super::*;
    use PresetField::*;

///
    fn monkey() -> PresetRow {
        PresetRow {
            graduation_mcap_usd: 75_000,
            creator_bps: 60,
            skim_buy_far_bps: 50,
            skim_sell_far_bps: 50,
            holder_bps: 0,
            sale_bps: 2_000,
            pol_bps: 1_000,
            skim_sell_at_floor_multiple_bps: 30_000,
            stair_steps: 12,
            stair_growth_bps: 10_200,
            team_buy_max_bps: 700,
            team_vest_days: [15, 30, 60, 90],
        }
    }

    fn with(f: impl FnOnce(&mut PresetRow)) -> Result<(), PresetField> {
        let mut r = monkey();
        f(&mut r);
        validate_preset_row(&r)
    }

    #[test]
    fn a_row_inside_every_rule_passes() {
        assert_eq!(validate_preset_row(&monkey()), Ok(()));
    }

///
    #[test]
    fn each_rule_is_refused_by_name_at_its_boundary() {

        assert_eq!(with(|r| r.graduation_mcap_usd = 1), Ok(()));
        assert_eq!(with(|r| r.graduation_mcap_usd = 0), Err(GraduationMcapUsd));

        assert_eq!(with(|r| r.creator_bps = 500), Ok(()));
        assert_eq!(with(|r| r.creator_bps = 501), Err(CreatorBps));
        assert_eq!(with(|r| { r.skim_buy_far_bps = 1_000 }), Ok(()));
        assert_eq!(with(|r| r.skim_buy_far_bps = 1_001), Err(SkimBuyFarBps));

        assert_eq!(with(|r| { r.skim_sell_far_bps = 1_000; r.skim_sell_at_floor_multiple_bps = 15_000 }), Ok(()));
        assert_eq!(with(|r| { r.skim_sell_far_bps = 1_001; r.skim_sell_at_floor_multiple_bps = 10_000 }), Err(SkimSellFarBps));

        assert_eq!(with(|r| r.holder_bps = 0), Ok(()));
        assert_eq!(with(|r| r.holder_bps = 1), Err(HolderBps));
        assert_eq!(with(|r| r.holder_bps = crate::policy::MAX_HOLDER_BPS), Err(HolderBps));
        assert_eq!(with(|r| r.holder_bps = crate::policy::MAX_HOLDER_BPS + 1), Err(HolderBps));

        assert_eq!(with(|r| { r.sale_bps = 1_000; r.pol_bps = 1_000 }), Ok(()));
        assert_eq!(with(|r| { r.sale_bps = 8_000; r.pol_bps = 1_000 }), Ok(()));
        assert_eq!(with(|r| r.sale_bps = 999), Err(SaleBps));
        assert_eq!(with(|r| r.pol_bps = 999), Err(PolBps));
        assert_eq!(with(|r| { r.sale_bps = 8_001; r.pol_bps = 1_000 }), Err(SaleBps), "the sale alone past 80 %, as the EVM names it");
        assert_eq!(with(|r| { r.sale_bps = 4_500; r.pol_bps = 4_501 }), Err(PolBps), "the sum past 90 %");

        assert_eq!(with(|r| r.skim_sell_at_floor_multiple_bps = 10_000), Ok(()));
        assert_eq!(with(|r| r.skim_sell_at_floor_multiple_bps = 9_999), Err(SkimSellAtFloorMultipleBps));
        assert_eq!(with(|r| { r.skim_sell_far_bps = 300; r.skim_sell_at_floor_multiple_bps = 50_000 }), Ok(()));
        assert_eq!(with(|r| { r.skim_sell_far_bps = 0; r.skim_sell_at_floor_multiple_bps = 50_001 }), Err(SkimSellAtFloorMultipleBps));
        assert_eq!(with(|r| { r.skim_sell_far_bps = 301; r.skim_sell_at_floor_multiple_bps = 50_000 }), Err(SkimSellAtFloorMultipleBps));

        assert_eq!(with(|r| { r.skim_sell_far_bps = 1_000; r.skim_sell_at_floor_multiple_bps = 15_009 }), Ok(()));
        assert_eq!(with(|r| { r.skim_sell_far_bps = 1_000; r.skim_sell_at_floor_multiple_bps = 15_010 }), Err(SkimSellAtFloorMultipleBps));

        assert_eq!(with(|r| r.stair_steps = 1), Ok(()));
        assert_eq!(with(|r| r.stair_steps = 16), Ok(()));
        assert_eq!(with(|r| r.stair_steps = 0), Err(StairSteps));
        assert_eq!(with(|r| r.stair_steps = 17), Err(StairSteps));
        assert_eq!(with(|r| r.stair_growth_bps = 5_000), Ok(()));
        assert_eq!(with(|r| r.stair_growth_bps = 20_000), Ok(()));
        assert_eq!(with(|r| r.stair_growth_bps = 4_999), Err(StairGrowthBps));
        assert_eq!(with(|r| r.stair_growth_bps = 20_001), Err(StairGrowthBps));

        assert_eq!(with(|r| r.team_buy_max_bps = 0), Ok(()));
        assert_eq!(with(|r| r.team_buy_max_bps = 1_000), Ok(()));
        assert_eq!(with(|r| r.team_buy_max_bps = 1_001), Err(TeamBuyMaxBps));
        assert_eq!(with(|r| r.team_vest_days = [10, 11, 12, 365]), Ok(()));
        for bad in [[9, 20, 40, 80], [10, 20, 40, 366], [10, 10, 40, 80], [10, 40, 20, 80], [0, 0, 0, 0]] {
            assert_eq!(with(|r| r.team_vest_days = bad), Err(TeamVestDays), "{bad:?}");
        }
        assert_eq!(TeamVestDays.name(), "team_vest_days");
        assert_eq!(GraduationMcapUsd.name(), "graduation_mcap_usd");
    }

///
    #[test]
    fn the_first_broken_rule_in_table_order_is_named() {
        assert_eq!(with(|r| { r.creator_bps = 501; r.stair_steps = 0 }), Err(CreatorBps));
        assert_eq!(with(|r| { r.graduation_mcap_usd = 0; r.team_vest_days = [0; 4] }), Err(GraduationMcapUsd));
    }

///
///
    #[test]
    fn the_seeded_rows_are_todays_presets() {
        for (id, mode) in [(1u8, Mode::Moonshot), (2, Mode::Cook), (3, Mode::Bluechip)] {
            let r = seed_row(id).unwrap();
            let p = preset(mode);
            assert_eq!(
                (r.graduation_mcap_usd, r.creator_bps, r.skim_buy_far_bps, r.skim_sell_far_bps, r.holder_bps),
                (p.graduation_mcap_usd, p.creator_bps, p.skim_buy_far_bps, p.skim_sell_far_bps, p.holder_bps),
                "{mode:?}"
            );
            assert_eq!((r.sale_bps, r.pol_bps, r.skim_sell_at_floor_multiple_bps), (p.sale_bps, p.pol_bps, p.skim_sell_at_floor_multiple_bps));
            assert_eq!(r.skim_sell_at_floor_multiple_bps as u64, PRESET_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS);
            assert_eq!((r.stair_steps, r.stair_growth_bps), (p.stair_steps, p.stair_growth_bps));
            assert_eq!(r.team_buy_max_bps as u64, crate::owners::team_buy_max_bps(id), "{mode:?}");
            assert_eq!(validate_preset_row(&r), Ok(()), "{mode:?} passes the one rule table");
        }

        assert_eq!(seed_row(1).unwrap().graduation_mcap_usd, 25_000);
        assert_eq!(seed_row(2).unwrap().graduation_mcap_usd, 50_000);
        assert_eq!(seed_row(3).unwrap().graduation_mcap_usd, 100_000);
        assert_eq!((seed_row(1).unwrap().team_buy_max_bps, seed_row(1).unwrap().team_vest_days), (500, [10, 20, 40, 80]));
        assert_eq!((seed_row(2).unwrap().team_buy_max_bps, seed_row(2).unwrap().team_vest_days), (500, [10, 20, 40, 80]));
        assert_eq!((seed_row(3).unwrap().team_buy_max_bps, seed_row(3).unwrap().team_vest_days), (1_000, [30, 60, 90, 120]));
        for id in [0u8, 4, 31, 255] {
            assert_eq!(seed_row(id), None, "{id} is not a seeded row");
        }
    }
}
