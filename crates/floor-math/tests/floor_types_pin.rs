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

#![allow(deprecated)]

use std::path::PathBuf;

use floor_math::constants::*;
use floor_math::modes::Mode;
use floor_math::policy::*;

fn floor_types_constants() -> Option<String> {
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "..", "floor-types", "src", "constants.rs"].iter().collect();
    std::fs::read_to_string(&path).ok()
}

///
fn initializer<'a>(src: &'a str, name: &str) -> &'a str {
    let needle = format!("pub const {name}:");
    let start = src
        .match_indices(&needle)
        .find(|(i, _)| *i == 0 || !src.as_bytes()[i - 1].is_ascii_alphanumeric() && src.as_bytes()[i - 1] != b'_')
        .map(|(i, _)| i + needle.len())
        .unwrap_or_else(|| panic!("floor_types::constants has no `{name}` — the pin needs the dev-dependency form"));
    let rest = &src[start..];

    let eq = rest.find('=').unwrap_or_else(|| panic!("`{name}`: no `=`"));
    let semi = rest[eq..].find(';').map(|i| eq + i).unwrap_or_else(|| panic!("`{name}`: no `;`"));
    rest[eq + 1..semi].trim()
}

fn scalar(text: &str, name: &str) -> i128 {
    let cleaned: String = text.chars().filter(|c| !c.is_whitespace() && *c != '_').collect();
    cleaned
        .parse::<i128>()
        .unwrap_or_else(|_| panic!("`{name}` = `{text}` is not a plain integer literal — redo the pin as a dev-dependency"))
}

fn value(src: &str, name: &str) -> i128 {
    scalar(initializer(src, name), name)
}

fn array(src: &str, name: &str) -> Vec<i128> {
    let text = initializer(src, name);
    let inner = text
        .strip_prefix('[')
        .and_then(|t| t.strip_suffix(']'))
        .unwrap_or_else(|| panic!("`{name}` = `{text}` is not an array literal"));
    inner.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).map(|s| scalar(s, name)).collect()
}

#[test]
fn constants_match_floor_types() {
    let Some(src) = floor_types_constants() else {
        eprintln!("VACUOUS: crates/floor-types is not on this branch; nothing to pin against (WP-0 unmerged)");
        return;
    };

    let pairs: [(&str, i128, &str); 65] = [
        ("SUPPLY", SUPPLY as i128, "SUPPLY"),


        ("SALE", SALE as i128, "SALE_TOKENS"),
        ("POL", POL as i128, "POL_TOKENS"),
        ("LADDER", LADDER as i128, "LADDER_TOKENS"),
        ("MIN_SELLABLE", MIN_SELLABLE as i128, "MIN_SELLABLE"),
        ("MIN_BID_LAMPORTS", MIN_BID_LAMPORTS as i128, "MIN_BID_LAMPORTS"),
        ("PLATFORM_BPS", PLATFORM_BPS as i128, "PLATFORM_BPS"),
        ("MAX_SKIM_FAR_BPS", MAX_SKIM_FAR_BPS as i128, "MAX_SKIM_FAR_BPS"),
        ("MAX_CREATOR_BPS", MAX_CREATOR_BPS as i128, "MAX_CREATOR_BPS"),
        ("MAX_HOLDER_BPS", MAX_HOLDER_BPS as i128, "MAX_HOLDER_BPS"),
        ("SKIM_RAMP_TICKS", SKIM_RAMP_TICKS as i128, "SKIM_RAMP_TICKS"),
        ("BID_BAND_TICKS", BID_BAND_TICKS as i128, "BID_BAND_TICKS"),
        ("TICKS_PER_DOUBLING", TICKS_PER_DOUBLING as i128, "TICKS_PER_DOUBLING"),
        ("LP_FEE_PIPS", LP_FEE_PIPS as i128, "LP_FEE_PIPS"),
        ("DEFAULT_SKIM_BUY_AT_FLOOR_BPS", DEFAULT_SKIM_BUY_AT_FLOOR_BPS as i128, "DEFAULT_SKIM_BUY_AT_FLOOR_BPS"),
        ("DEFAULT_SKIM_SELL_AT_FLOOR_BPS", DEFAULT_SKIM_SELL_AT_FLOOR_BPS as i128, "DEFAULT_SKIM_SELL_AT_FLOOR_BPS"),



        ("SKIM_SELL_AT_FLOOR_MULTIPLE_BPS", SKIM_SELL_AT_FLOOR_MULTIPLE_BPS as i128, "SKIM_SELL_AT_FLOOR_MULTIPLE_BPS"),



        ("PRESET_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS", PRESET_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS as i128, "PRESET_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS"),
        ("MIN_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS", MIN_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS as i128, "MIN_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS"),
        ("MAX_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS", MAX_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS as i128, "MAX_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS"),

        ("MAX_SKIM_SELL_AT_FLOOR_BPS", MAX_SKIM_SELL_AT_FLOOR_BPS as i128, "MAX_SKIM_SELL_AT_FLOOR_BPS"),

        ("MAX_STAIR_STEPS", floor_math::stair::MAX_STAIR_STEPS as i128, "MAX_STAIR_STEPS"),




        ("REFERRAL_CREATOR_SHARE_BPS", REFERRAL_CREATOR_SHARE_BPS as i128, "REFERRAL_CREATOR_SHARE_BPS"),
        ("MAX_REFERRAL_SHARE_BPS", MAX_REFERRAL_SHARE_BPS as i128, "MAX_REFERRAL_SHARE_BPS"),
        ("BID_SALT", BID_SALT as i128, "SALT_BID"),
        ("SEED_SALT", SEED_SALT as i128, "SALT_SEED"),
        ("BPS", BPS as i128, "BPS"),





        ("MIN_QUOTE_DECIMALS", MIN_QUOTE_DECIMALS as i128, "MIN_QUOTE_DECIMALS"),
        ("MAX_QUOTE_DECIMALS", MAX_QUOTE_DECIMALS as i128, "MAX_QUOTE_DECIMALS"),



        ("TEAM_SALE_FEE_BPS_DEFAULT", TEAM_SALE_FEE_BPS_DEFAULT as i128, "TEAM_SALE_FEE_BPS_DEFAULT"),
        ("TEAM_SALE_FEE_BPS_MAX", TEAM_SALE_FEE_BPS_MAX as i128, "TEAM_SALE_FEE_BPS_MAX"),
        ("TEAM_POOL_FEE_BPS_DEFAULT", TEAM_POOL_FEE_BPS_DEFAULT as i128, "TEAM_POOL_FEE_BPS_DEFAULT"),
        ("TEAM_POOL_FEE_BPS_MAX", TEAM_POOL_FEE_BPS_MAX as i128, "TEAM_POOL_FEE_BPS_MAX"),



        ("GENESIS_SALE", floor_math::genesis::GENESIS_SALE as i128, "GENESIS_SALE_TOKENS"),



        ("GENESIS_CORE_TOKENS", floor_math::genesis::GENESIS_CORE_TOKENS as i128, "GENESIS_CORE_TOKENS"),
        ("GENESIS_FF_TOKENS", floor_math::genesis::GENESIS_FF_TOKENS as i128, "GENESIS_FF_TOKENS"),
        ("GENESIS_SALE_PRICE_LAMPORTS_PER_MILLION", floor_math::genesis::GENESIS_SALE_PRICE_LAMPORTS_PER_MILLION as i128, "GENESIS_SALE_PRICE_LAMPORTS_PER_MILLION"),
        ("GENESIS_CORE_CAP_LAMPORTS", floor_math::genesis::GENESIS_CORE_CAP_LAMPORTS as i128, "GENESIS_CORE_CAP_LAMPORTS"),
        ("GENESIS_FF_CAP_LAMPORTS", floor_math::genesis::GENESIS_FF_CAP_LAMPORTS as i128, "GENESIS_FF_CAP_LAMPORTS"),
        ("GENESIS_OPEN_TICK_MIN", floor_math::genesis::GENESIS_OPEN_TICK_MIN as i128, "GENESIS_OPEN_TICK_MIN"),
        ("GENESIS_OPEN_TICK_MAX", floor_math::genesis::GENESIS_OPEN_TICK_MAX as i128, "GENESIS_OPEN_TICK_MAX"),

        ("GENESIS_BUYER_VEST_SECS", floor_math::genesis::GENESIS_BUYER_VEST_SECS as i128, "GENESIS_BUYER_VEST_SECS"),
        ("GENESIS_BUYER_CLIFF_SECS", floor_math::genesis::GENESIS_BUYER_CLIFF_SECS as i128, "GENESIS_BUYER_CLIFF_SECS"),



        ("STAIR_KIND_PRESET", floor_math::stair::STAIR_KIND_PRESET as i128, "STAIR_KIND_PRESET"),
        ("STAIR_KIND_GENESIS", floor_math::stair::STAIR_KIND_GENESIS as i128, "STAIR_KIND_GENESIS"),
        ("GENESIS_STAIR_GROWTH_BPS", floor_math::stair::GENESIS_STAIR_GROWTH_BPS as i128, "GENESIS_STAIR_GROWTH_BPS"),
        ("GENESIS_STAIR_TOKENS", (floor_math::constants::SUPPLY / 10_000 * floor_math::stair::GENESIS_STAIR_STEPS_BPS as u64) as i128, "GENESIS_STAIR_TOKENS"),
        ("GENESIS_STAIR_TAIL_TOKENS", (floor_math::constants::SUPPLY / 10_000 * floor_math::stair::GENESIS_STAIR_TAIL_BPS as u64) as i128, "GENESIS_STAIR_TAIL_TOKENS"),



        ("GENESIS_PROTOCOL_STAIR_TOKENS", floor_math::genesis_layout::GENESIS_PROTOCOL_STAIR_TOKENS as i128, "GENESIS_PROTOCOL_STAIR_TOKENS"),
        ("GENESIS_TEAM_STAIR_TOKENS", floor_math::genesis_layout::GENESIS_TEAM_STAIR_TOKENS as i128, "GENESIS_TEAM_STAIR_TOKENS"),
        ("GENESIS_TEAM_STEP3_TOKENS", floor_math::genesis_layout::GENESIS_TEAM_STEP3_TOKENS as i128, "GENESIS_TEAM_STEP3_TOKENS"),
        ("GENESIS_MIN_STAIR_STEPS", floor_math::genesis_layout::GENESIS_MIN_STAIR_STEPS as i128, "GENESIS_MIN_STAIR_STEPS"),


        ("TEAM_BUY_MAX_BPS", floor_math::owners::floor_types_team_buy::TEAM_BUY_MAX_BPS as i128, "TEAM_BUY_MAX_BPS"),
        ("BLUECHIP_TEAM_BUY_MAX_BPS", floor_math::owners::floor_types_team_buy::BLUECHIP_TEAM_BUY_MAX_BPS as i128, "BLUECHIP_TEAM_BUY_MAX_BPS"),
        ("TEAM_STEPS_MIN_BPS", floor_math::owners::floor_types_team_buy::TEAM_STEPS_MIN_BPS as i128, "TEAM_STEPS_MIN_BPS"),
        ("TEAM_STEPS_MAX_SHARE_BPS", floor_math::owners::floor_types_team_buy::TEAM_STEPS_MAX_SHARE_BPS as i128, "TEAM_STEPS_MAX_SHARE_BPS"),

        ("presets::MAX_PRESET_ID", floor_math::presets::MAX_PRESET_ID as i128, "MAX_PRESET_ID"),
        ("presets::TEAM_VEST_DAYS_MIN", floor_math::presets::TEAM_VEST_DAYS_MIN as i128, "TEAM_CUSTOM_VEST_DAYS_MIN"),
        ("presets::TEAM_VEST_DAYS_MAX", floor_math::presets::TEAM_VEST_DAYS_MAX as i128, "TEAM_CUSTOM_VEST_DAYS_MAX"),





        ("Mode::Cook", Mode::Cook as i128, "MODE_COOK"),
        ("Mode::Bluechip", Mode::Bluechip as i128, "MODE_BLUECHIP"),
        ("Mode::Moonshot", Mode::Moonshot as i128, "MODE_MOONSHOT"),

        ("MIN_ZONE_BPS", floor_math::modes::MIN_ZONE_BPS as i128, "MIN_ZONE_BPS"),
        ("STANDARD_SALE_BPS", floor_math::modes::STANDARD_SALE_BPS as i128, "STANDARD_SALE_BPS"),
        ("STANDARD_POL_BPS", floor_math::modes::STANDARD_POL_BPS as i128, "STANDARD_POL_BPS"),

    ];
    let mut mismatches = Vec::new();
    for (ours, v, theirs) in pairs {
        let t = value(&src, theirs);
        if t != v {
            mismatches.push(format!("floor_math::{ours} = {v} but floor_types::{theirs} = {t}"));
        }
    }
    if Mode::Custom as i128 != value(&src, "MODE_CUSTOM") {
        mismatches.push("Mode::Custom vs MODE_CUSTOM".into());
    }

    if initializer(&src, "MODE_MAX") != "MODE_BLUECHIP" {
        mismatches.push("floor_types::MODE_MAX is not MODE_BLUECHIP".into());
    }


    for dead in ["pub const MODE_MEME", "pub const MODE_NITRO"] {
        if src.contains(dead) {
            mismatches.push(format!("floor_types still defines `{dead}`"));
        }
    }


    if src.contains("pub const LIFT_MIN_GAP_TICKS") {
        mismatches.push("floor_types still defines LIFT_MIN_GAP_TICKS".into());
    }
    if src.contains("pub const ASK_MULTIPLE_T0_BPS") || src.contains("pub const ASK_MULTIPLE_BPS: [u32; 3]") {
        mismatches.push("floor_types still spells the three-tier ask multiple".into());
    }
    assert!(mismatches.is_empty(), "constants drifted from floor_types:\n{}", mismatches.join("\n"));
}


#[test]
fn the_reader_reads_the_shapes_floor_types_uses() {
    let src = "pub const A_B: u64 = 1_000;\n/// doc\npub const B: i32 = -6_932;\npub const ARR: [u32; 3] = [20_000, 40_000, 80_000];\npub const XB: u16 = 7;\n";
    assert_eq!(value(src, "A_B"), 1_000);
    assert_eq!(value(src, "B"), -6_932);
    assert_eq!(value(src, "XB"), 7);
    assert_eq!(array(src, "ARR"), vec![20_000, 40_000, 80_000]);
    assert!(std::panic::catch_unwind(|| value(src, "MISSING")).is_err());
    assert!(std::panic::catch_unwind(|| value("pub const F: u16 = 1 << 13;", "F")).is_err());
}
