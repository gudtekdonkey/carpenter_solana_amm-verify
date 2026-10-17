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

use floor_math::presets::{validate_preset_row, PresetField, PresetField::*, PresetRow};

///
///
fn monkey() -> PresetRow {
    PresetRow {
        graduation_mcap_usd: 40_000,
        creator_bps: 120,
        skim_buy_far_bps: 50,
        skim_sell_far_bps: 80,
        holder_bps: 0,
        sale_bps: 2_500,
        pol_bps: 1_500,
        skim_sell_at_floor_multiple_bps: 30_000,
        stair_steps: 12,
        stair_growth_bps: 9_000,
        team_buy_max_bps: 700,
        team_vest_days: [15, 30, 60, 120],
    }
}

fn with(f: impl FnOnce(&mut PresetRow)) -> Result<(), PresetField> {
    let mut r = monkey();
    f(&mut r);
    validate_preset_row(&r)
}

#[test]
fn the_base_row_passes() {
    assert_eq!(validate_preset_row(&monkey()), Ok(()));
}

///
#[test]
fn rule_graduation_mcap_usd() {
    assert_eq!(with(|r| r.graduation_mcap_usd = 0), Err(GraduationMcapUsd));
    assert_eq!(with(|r| r.graduation_mcap_usd = 1), Ok(()));
    assert_eq!(with(|r| r.graduation_mcap_usd = u64::MAX), Ok(()));
}

///
#[test]
fn rule_creator_bps() {
    assert_eq!(with(|r| r.creator_bps = 0), Ok(()));
    assert_eq!(with(|r| r.creator_bps = 500), Ok(()));
    assert_eq!(with(|r| r.creator_bps = 501), Err(CreatorBps));
    assert_eq!(with(|r| r.creator_bps = u16::MAX), Err(CreatorBps));
}

///
#[test]
fn rule_skim_buy_far_bps() {
    assert_eq!(with(|r| r.skim_buy_far_bps = 1_000), Ok(()));
    assert_eq!(with(|r| r.skim_buy_far_bps = 1_001), Err(SkimBuyFarBps));
}

///
#[test]
fn rule_skim_sell_far_bps() {
    let at_1x = |far: u16| with(|r| { r.skim_sell_at_floor_multiple_bps = 10_000; r.skim_sell_far_bps = far });
    assert_eq!(at_1x(1_000), Ok(()));
    assert_eq!(at_1x(1_001), Err(SkimSellFarBps));
    assert_eq!(at_1x(0), Ok(()));
}

///
///
///
#[test]
fn rule_holder_bps() {
    assert_eq!(with(|r| r.holder_bps = 0), Ok(()));
    assert_eq!(with(|r| r.holder_bps = 1), Err(HolderBps));
    assert_eq!(with(|r| r.holder_bps = floor_math::policy::MAX_HOLDER_BPS), Err(HolderBps));
    assert_eq!(with(|r| r.holder_bps = 1_001), Err(HolderBps));
    assert_eq!(with(|r| r.holder_bps = u16::MAX), Err(HolderBps));
}

///
#[test]
fn rule_split() {
    let split = |s: u16, p: u16| with(|r| { r.sale_bps = s; r.pol_bps = p });
    assert_eq!(split(1_000, 1_000), Ok(()));
    assert_eq!(split(8_000, 1_000), Ok(()));
    assert_eq!(split(1_000, 8_000), Ok(()));
    assert_eq!(split(999, 1_000), Err(SaleBps));
    assert_eq!(split(1_000, 999), Err(PolBps));
    assert_eq!(split(8_001, 1_000), Err(SaleBps));
    assert_eq!(split(4_500, 4_501), Err(PolBps));
    assert_eq!(split(0, 0), Err(SaleBps));
    assert_eq!(split(u16::MAX, 1_000), Err(SaleBps));
    assert_eq!(split(1_000, u16::MAX), Err(PolBps));
}

///
#[test]
fn rule_skim_sell_at_floor_multiple_range() {
    let m = |x: u32| with(|r| { r.skim_sell_far_bps = 300; r.skim_sell_at_floor_multiple_bps = x });
    assert_eq!(m(10_000), Ok(()));
    assert_eq!(m(50_000), Ok(()));
    assert_eq!(m(9_999), Err(SkimSellAtFloorMultipleBps));
    assert_eq!(m(50_001), Err(SkimSellAtFloorMultipleBps));
    assert_eq!(m(0), Err(SkimSellAtFloorMultipleBps));
}

///
#[test]
fn rule_at_floor_product() {
    let p = |far: u16, x: u32| with(|r| { r.skim_sell_far_bps = far; r.skim_sell_at_floor_multiple_bps = x });
    assert_eq!(p(300, 50_000), Ok(()), "3 % at 5× = 15 %");
    assert_eq!(p(1_000, 15_000), Ok(()), "10 % at 1.5× = 15 %");
    assert_eq!(p(0, 50_000), Ok(()), "0 % at 5×");
    assert_eq!(p(301, 50_000), Err(SkimSellAtFloorMultipleBps), "1,505");
    assert_eq!(p(1_000, 15_010), Err(SkimSellAtFloorMultipleBps), "1,501");
    assert_eq!(p(1_000, 50_000), Err(SkimSellAtFloorMultipleBps), "5,000");
    assert_eq!(p(1_000, u32::MAX), Err(SkimSellAtFloorMultipleBps));
}

///
#[test]
fn rule_stair_steps() {
    assert_eq!(with(|r| r.stair_steps = 1), Ok(()));
    assert_eq!(with(|r| r.stair_steps = 16), Ok(()));
    assert_eq!(with(|r| r.stair_steps = 0), Err(StairSteps));
    assert_eq!(with(|r| r.stair_steps = 17), Err(StairSteps));
}

///
#[test]
fn rule_stair_growth_bps() {
    assert_eq!(with(|r| r.stair_growth_bps = 5_000), Ok(()));
    assert_eq!(with(|r| r.stair_growth_bps = 20_000), Ok(()));
    assert_eq!(with(|r| r.stair_growth_bps = 4_999), Err(StairGrowthBps));
    assert_eq!(with(|r| r.stair_growth_bps = 20_001), Err(StairGrowthBps));
    assert_eq!(with(|r| r.stair_growth_bps = 0), Err(StairGrowthBps));
}

///
#[test]
fn rule_team_buy_max_bps() {
    assert_eq!(with(|r| r.team_buy_max_bps = 0), Ok(()));
    assert_eq!(with(|r| r.team_buy_max_bps = 1_000), Ok(()));
    assert_eq!(with(|r| r.team_buy_max_bps = 1_001), Err(TeamBuyMaxBps));
}

///
#[test]
fn rule_team_vest_days() {
    for ok in [[10, 11, 12, 13], [362, 363, 364, 365], [10, 20, 40, 365]] {
        assert_eq!(with(|r| r.team_vest_days = ok), Ok(()), "{ok:?}");
    }
    for bad in [
        [9, 20, 40, 80],
        [10, 20, 40, 366],
        [10, 10, 40, 80],
        [10, 20, 20, 80],
        [10, 20, 40, 40],
        [80, 40, 20, 10],
        [0, 0, 0, 0],
        [10, 20, 40, 0],
        [366, 367, 368, 369],
    ] {
        assert_eq!(with(|r| r.team_vest_days = bad), Err(TeamVestDays), "{bad:?}");
    }
}

///
#[test]
fn every_field_has_its_wire_name() {
    let all = [
        (GraduationMcapUsd, "graduation_mcap_usd"),
        (CreatorBps, "creator_bps"),
        (SkimBuyFarBps, "skim_buy_far_bps"),
        (SkimSellFarBps, "skim_sell_far_bps"),
        (HolderBps, "holder_bps"),
        (SaleBps, "sale_bps"),
        (PolBps, "pol_bps"),
        (SkimSellAtFloorMultipleBps, "skim_sell_at_floor_multiple_bps"),
        (StairSteps, "stair_steps"),
        (StairGrowthBps, "stair_growth_bps"),
        (TeamBuyMaxBps, "team_buy_max_bps"),
        (TeamVestDays, "team_vest_days"),
    ];
    for (f, name) in all {
        assert_eq!(f.name(), name);
    }
}
