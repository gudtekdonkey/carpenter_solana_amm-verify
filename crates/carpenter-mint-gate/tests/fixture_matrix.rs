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

use std::str::FromStr;

use carpenter_mint_gate::*;
use solana_pubkey::Pubkey;

///
struct Fixture {
    label: String,
    owner: Pubkey,
    mint: Pubkey,
    data: Vec<u8>,
    freeze: Option<Pubkey>,
    permanent_delegate: Option<Pubkey>,
    pause_authority: Option<Pubkey>,
    hook_authority: Option<Pubkey>,
    d19_mirror: String,
}

fn key(v: &serde_json::Value) -> Option<Pubkey> {
    match v {
        serde_json::Value::String(s) if s != "None" => Some(Pubkey::from_str(s).expect("a base58 key")),
        _ => None,
    }
}

fn load() -> Vec<Fixture> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/stock-mints/mints.json");
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let doc: serde_json::Value = serde_json::from_str(&text).expect("mints.json parses");
    assert_eq!(doc["schema"], "carpenter/stock-mint-fixture/v1", "the fixture schema this suite reads");
    assert_eq!(doc["tokenProgram"], TOKEN_2022_ID.to_string());
    doc["mints"]
        .as_array()
        .expect("mints[]")
        .iter()
        .map(|m| {
            let data = hex::decode(m["dataHex"].as_str().expect("dataHex")).expect("hex");
            assert_eq!(data.len(), m["len"].as_u64().unwrap() as usize, "{}: dataHex is the whole account", m["label"]);
            Fixture {
                label: m["label"].as_str().unwrap().to_string(),
                owner: key(&m["owner"]).expect("owner"),
                mint: key(&m["mint"]).expect("mint"),
                data,
                freeze: key(&m["freezeAuthority"]),
                permanent_delegate: key(&m["permanentDelegate"]),
                pause_authority: key(&m["pausable"]["authority"]),
                hook_authority: key(&m["transferHook"]["authority"]),
                d19_mirror: m["d19Mirror"].as_str().unwrap().to_string(),
            }
        })
        .collect()
}

fn find<'a>(all: &'a [Fixture], label: &str) -> &'a Fixture {
    all.iter().find(|f| f.label == label).unwrap_or_else(|| panic!("fixture {label} is missing"))
}

fn gate(f: &Fixture, role: MintRole, pausable: PausableRule) -> Result<u8, GateError> {
    check_mint_supported(&f.owner, &f.mint, &f.data, role, pausable)
}

const XSTOCK_CUSTODIAL: u8 =
    QUOTE_CAP_FREEZE | QUOTE_CAP_PERMANENT_DELEGATE | QUOTE_CAP_PAUSABLE | QUOTE_CAP_DEFAULT_STATE | QUOTE_CAP_HOOK_SLOT;
const XSTOCK_CAPS: u8 = XSTOCK_CUSTODIAL | QUOTE_CAP_SCALED_UI | QUOTE_CAP_CONFIDENTIAL;
const BACKPACK_CAPS: u8 = QUOTE_CAP_FREEZE | QUOTE_CAP_PERMANENT_DELEGATE | QUOTE_CAP_PAUSABLE | QUOTE_CAP_DEFAULT_STATE;

#[test]
fn the_fixture_is_the_four_shapes_this_suite_expects() {
    let all = load();
    let labels: Vec<&str> = all.iter().map(|f| f.label.as_str()).collect();
    assert_eq!(labels, ["backed-xstock-8dp", "backed-xstock-8dp-hook-armed", "backpack-securities-6dp", "impostor-pump"]);
    for f in &all {
        assert_eq!(f.owner, TOKEN_2022_ID, "{}: every fixture mint is Token-2022", f.label);
    }
}





#[test]
fn backed_xstock_is_refused_as_a_base_mint_and_admitted_as_a_listed_quote_with_the_five_custodial_bits() {
    let all = load();
    let f = find(&all, "backed-xstock-8dp");

    assert_eq!(gate(f, MintRole::Base, PausableRule::Renounced), Err(GateError::MintNotSupportedAsBase));
    assert_eq!(gate(f, MintRole::Base, PausableRule::HookVouched), Err(GateError::MintNotSupportedAsBase),
        "the vouch lifts the pause authority alone; freeze, delegate, default state and the hook slot still refuse a base");

    let caps = gate(f, MintRole::ListedQuote, PausableRule::Renounced).expect("a listed quote");
    assert_eq!(caps & QUOTE_CAPS_CUSTODIAL, XSTOCK_CUSTODIAL, "the plan's Tier-B word: FREEZE|PERMANENT_DELEGATE|PAUSABLE|DEFAULT_STATE|HOOK_SLOT");
    assert_eq!(caps, XSTOCK_CAPS, "plus the two Tier-A record bits the fixture actually ships (ScaledUiAmount, ConfidentialTransferMint)");
    assert_eq!(caps & QUOTE_CAP_INTEREST, 0, "no InterestBearingConfig on an xStock");

    assert_eq!(gate(f, MintRole::ListedQuote, PausableRule::HookVouched), Ok(XSTOCK_CAPS));
}





#[test]
fn an_armed_transfer_hook_is_refused_in_every_role_with_the_tier_c_word() {
    let all = load();
    let f = find(&all, "backed-xstock-8dp-hook-armed");
    for (role, rule) in [
        (MintRole::Base, PausableRule::Renounced),
        (MintRole::Base, PausableRule::HookVouched),
        (MintRole::ListedQuote, PausableRule::Renounced),
        (MintRole::ListedQuote, PausableRule::HookVouched),
    ] {
        assert_eq!(gate(f, role, rule), Err(GateError::MintNotSupported), "{role:?}/{rule:?}: an armed hook is their code inside our transfer");
    }
}





#[test]
fn backpack_is_refused_as_a_base_mint_and_admitted_as_a_listed_quote_with_freeze_delegate_pause_and_default_state() {
    let all = load();
    let f = find(&all, "backpack-securities-6dp");
    assert_eq!(f.freeze, f.permanent_delegate, "the fixture's ONE shared key");
    assert_eq!(f.freeze, f.pause_authority);
    assert_eq!(f.hook_authority, None, "no TransferHook on the Backpack shape");

    assert_eq!(gate(f, MintRole::Base, PausableRule::Renounced), Err(GateError::MintNotSupportedAsBase));
    assert_eq!(gate(f, MintRole::Base, PausableRule::HookVouched), Err(GateError::MintNotSupportedAsBase));
    let caps = gate(f, MintRole::ListedQuote, PausableRule::Renounced).expect("a listed quote");
    assert_eq!(caps, BACKPACK_CAPS, "FREEZE|PERMANENT_DELEGATE|PAUSABLE, plus the DEFAULT_STATE the fixture set; no hook slot, no Tier-A bit");
    assert_eq!(caps & QUOTE_CAP_HOOK_SLOT, 0);
    assert_eq!(caps & QUOTE_CAPS_INERT, 0);
}





#[test]
fn the_impostor_is_tier_a_and_passes_both_roles_with_no_capability_at_all() {
    let all = load();
    let f = find(&all, "impostor-pump");
    assert_eq!(f.freeze, None);
    assert_eq!(f.permanent_delegate, None);
    assert_eq!(f.pause_authority, None);
    assert_eq!(gate(f, MintRole::Base, PausableRule::Renounced), Ok(0));
    assert_eq!(gate(f, MintRole::Base, PausableRule::HookVouched), Ok(0));
    assert_eq!(gate(f, MintRole::ListedQuote, PausableRule::Renounced), Ok(0));



}






#[test]
fn every_mint_main_accepts_today_is_accepted_in_the_base_role_and_every_refusal_stays_a_refusal() {



    for f in load() {
        let ours = gate(&f, MintRole::Base, PausableRule::Renounced);
        match f.d19_mirror.as_str() {
            "accepted" => assert_eq!(ours, Ok(0), "{}: main accepts it, so must the base role", f.label),
            refused => {
                assert!(refused.starts_with("refused"), "{}: d19Mirror is `{refused}`", f.label);
                assert!(ours.is_err(), "{}: main refuses it ({refused}); the base role must too, got {ours:?}", f.label);
            }
        }
    }
}





#[test]
fn the_governance_mask_admits_the_xstock_only_once_every_one_of_its_five_powers_is_admitted() {
    let all = load();
    let caps = gate(find(&all, "backed-xstock-8dp"), MintRole::ListedQuote, PausableRule::Renounced).unwrap();

    assert_eq!(check_admitted(caps, 0), Err(GateError::CapabilityNotAdmitted));

    let mainnet = QUOTE_CAPS_CUSTODIAL & !QUOTE_CAP_PERMANENT_DELEGATE;
    assert_eq!(check_admitted(caps, mainnet), Err(GateError::CapabilityNotAdmitted));

    assert_eq!(check_admitted(caps, QUOTE_CAPS_CUSTODIAL), Ok(()));

    assert_eq!(check_admitted(caps & QUOTE_CAPS_INERT, 0), Ok(()));

    let bp = gate(find(&all, "backpack-securities-6dp"), MintRole::ListedQuote, PausableRule::Renounced).unwrap();
    assert_eq!(check_admitted(bp, mainnet), Err(GateError::CapabilityNotAdmitted));
    assert_eq!(check_admitted(bp, mainnet | QUOTE_CAP_PERMANENT_DELEGATE), Ok(()));

    let imp = gate(find(&all, "impostor-pump"), MintRole::ListedQuote, PausableRule::Renounced).unwrap();
    assert_eq!(check_admitted(imp, 0), Ok(()));
}





///
///
fn expected_digest(f: &Fixture) -> [u8; 32] {
    let mut buf = [0u8; 128];
    let put = |buf: &mut [u8; 128], at: usize, k: Option<Pubkey>| {
        if let Some(k) = k {
            buf[at..at + 32].copy_from_slice(&k.to_bytes());
        }
    };
    put(&mut buf, 0, f.freeze);
    put(&mut buf, 32, f.permanent_delegate);
    put(&mut buf, 64, f.pause_authority);
    put(&mut buf, 96, f.hook_authority);
    solana_sha256_hasher::hash(&buf).to_bytes()
}

#[test]
fn the_authority_digest_is_freeze_delegate_pause_hook_in_that_order_absent_as_zeros() {
    let all = load();
    for f in &all {
        assert_eq!(authority_digest(&f.data), Ok(expected_digest(f)), "{}", f.label);
    }


    let d: Vec<[u8; 32]> = all.iter().map(|f| authority_digest(&f.data).unwrap()).collect();
    for i in 0..d.len() {
        for j in i + 1..d.len() {
            assert_ne!(d[i], d[j], "{} vs {}", all[i].label, all[j].label);
        }
    }
    assert_eq!(authority_digest(&find(&all, "impostor-pump").data), Ok(solana_sha256_hasher::hash(&[0u8; 128]).to_bytes()));

    assert_eq!(authority_digest(&find(&all, "backed-xstock-8dp").data), authority_digest(&find(&all, "backed-xstock-8dp").data));
}





#[test]
fn the_interface_crate_s_ids_are_the_repo_s_keys_constants() {
    assert_eq!(SPL_TOKEN_ID.to_bytes(), carpenter_types::keys::SPL_TOKEN_PROGRAM_ID);
    assert_eq!(TOKEN_2022_ID.to_bytes(), carpenter_types::keys::TOKEN_2022_PROGRAM_ID);
    assert_eq!(TOKEN_2022_NATIVE_MINT.to_bytes(), carpenter_types::keys::TOKEN_2022_NATIVE_MINT);
}
