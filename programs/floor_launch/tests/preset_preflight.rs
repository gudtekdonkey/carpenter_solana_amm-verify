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

use anchor_lang::prelude::Pubkey;
use floor_launch::errors::LaunchError;
use floor_launch::sale::{resolve_config, resolve_team_buy};
use floor_launch::state::{CreateLaunchParams, PresetAppendFault, PresetRegistry};
use floor_math::presets::{seed_row, PresetRow};
use floor_types::constants::{BPS, PLATFORM_BPS, SUPPLY, TEAM_LOCK_MIN_DAYS, TEAM_STEPS_MIN_BPS};

const TABLE: &str = include_str!("../../../fixtures/preset-preflight/rows.tsv");

struct Case {
    name: String,
    id: u8,
    row: PresetRow,
    quote_cents: u64,
    quote_decimals: u8,
    min_bid: u64,
    expect: String,
}

fn cases() -> Vec<Case> {
    let mut out = Vec::new();
    for line in TABLE.lines().filter(|l| !l.is_empty() && !l.starts_with('#')) {
        let c: Vec<&str> = line.split('\t').collect();
        assert_eq!(c.len(), 21, "fixture row `{line}`: 21 tab-separated columns");
        let n = |i: usize| -> u64 { c[i].parse().unwrap_or_else(|_| panic!("{}: column {i} `{}` is not an integer", c[0], c[i])) };
        out.push(Case {
            name: c[0].to_string(),
            id: n(1) as u8,
            row: PresetRow {
                graduation_mcap_usd: n(2),
                creator_bps: n(3) as u16,
                skim_buy_far_bps: n(4) as u16,
                skim_sell_far_bps: n(5) as u16,
                holder_bps: n(6) as u16,
                sale_bps: n(7) as u16,
                pol_bps: n(8) as u16,
                skim_sell_at_floor_multiple_bps: n(9) as u32,
                stair_steps: n(10) as u8,
                stair_growth_bps: n(11) as u16,
                team_buy_max_bps: n(12) as u16,
                team_vest_days: [n(13) as u16, n(14) as u16, n(15) as u16, n(16) as u16],
            },
            quote_cents: n(17),
            quote_decimals: n(18) as u8,
            min_bid: n(19),
            expect: c[20].to_string(),
        });
    }
    out
}

fn name(e: LaunchError) -> String {
    format!("{e:?}")
}

///
fn verdict(c: &Case) -> String {

    let mut reg: PresetRegistry = bytemuck::Zeroable::zeroed();
    reg.count = c.id;
    match reg.append(&c.row) {
        Err(PresetAppendFault::TableFull) => return "row:PresetTableFull".into(),
        Err(PresetAppendFault::Invalid(f)) => return format!("row:PresetInvalid:{}", f.name()),
        Ok(id) => assert_eq!(id, c.id, "{}: append wrote another id", c.name),
    }

    let payee = Pubkey::new_from_array([7; 32]);
    let p = CreateLaunchParams { mode: c.id, quote_usd_cents: c.quote_cents, recipients: vec![floor_launch::state::SlotArg { bps: 10_000, platform: 0, key: payee.to_bytes() }], ..Default::default() };
    let r = match resolve_config(&p, "Preflight", "PRE", "", PLATFORM_BPS, c.min_bid, c.quote_decimals, Some(&c.row)) {
        Ok(r) => r,
        Err(f) => return format!("create:{}", name(f.into())),
    };

    if c.row.team_buy_max_bps > 0 {
        let t = CreateLaunchParams {
            team_buy_tokens: SUPPLY / BPS * c.row.team_buy_max_bps as u64,
            team_steps_bps: TEAM_STEPS_MIN_BPS,
            team_lock_days: TEAM_LOCK_MIN_DAYS,
            ..p.clone()
        };
        if let Err(f) = resolve_team_buy(&t, &r) {
            return format!("teamBuyMax:{}", name(f.into()));
        }
    }
    "ok".into()
}

#[test]
fn the_preflight_table_is_the_programs_verdict_row_for_row() {
    let cs = cases();
    assert!(cs.len() >= 19, "the fixture lost rows: {}", cs.len());
    let mut bad = Vec::new();
    for c in &cs {
        let got = verdict(c);
        if got != c.expect {
            bad.push(format!("{}: program says {got}, fixture (and the SDK) say {}", c.name, c.expect));
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));

    for want in ["ok", "row:PresetTableFull", "row:PresetInvalid:", "create:BadConfig", "teamBuyMax:TeamBuyOutOfRange", "teamBuyMax:TeamStepsOverCap"] {
        assert!(cs.iter().any(|c| c.expect.starts_with(want)), "no fixture row expects {want}");
    }
}

#[test]
fn the_seed_rows_of_the_table_are_floor_maths_seeds() {
    for c in cases().iter().filter(|c| c.name.ends_with("-seed")) {
        assert_eq!(Some(c.row), seed_row(c.id), "{}: the fixture's seed row is not floor_math::presets::seed_row({})", c.name, c.id);
    }
    assert_eq!(cases().iter().filter(|c| c.name.ends_with("-seed")).count(), 3);
}
