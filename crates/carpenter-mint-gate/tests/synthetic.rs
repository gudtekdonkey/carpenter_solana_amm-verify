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

use carpenter_mint_gate::*;
use solana_pubkey::Pubkey;


const EXT_TRANSFER_FEE_CONFIG: u16 = 1;
const EXT_TRANSFER_FEE_AMOUNT: u16 = 2;
const EXT_MINT_CLOSE_AUTHORITY: u16 = 3;
const EXT_CONFIDENTIAL_TRANSFER_MINT: u16 = 4;
const EXT_DEFAULT_ACCOUNT_STATE: u16 = 6;
const EXT_IMMUTABLE_OWNER: u16 = 7;
const EXT_NON_TRANSFERABLE: u16 = 9;
const EXT_INTEREST_BEARING_CONFIG: u16 = 10;
const EXT_CPI_GUARD: u16 = 11;
const EXT_PERMANENT_DELEGATE: u16 = 12;
const EXT_TRANSFER_HOOK: u16 = 14;
const EXT_CONFIDENTIAL_TRANSFER_FEE_CONFIG: u16 = 16;
const EXT_CONFIDENTIAL_TRANSFER_FEE_AMOUNT: u16 = 17;
const EXT_METADATA_POINTER: u16 = 18;
const EXT_TOKEN_METADATA: u16 = 19;
const EXT_GROUP_POINTER: u16 = 20;
const EXT_TOKEN_GROUP: u16 = 21;
const EXT_GROUP_MEMBER_POINTER: u16 = 22;
const EXT_TOKEN_GROUP_MEMBER: u16 = 23;
const EXT_CONFIDENTIAL_MINT_BURN: u16 = 24;
const EXT_SCALED_UI_AMOUNT: u16 = 25;
const EXT_PAUSABLE: u16 = 26;
///
const EXT_UNKNOWN: u16 = 200;

const ACCOUNT_STATE_INITIALIZED: u8 = 1;
const ACCOUNT_STATE_FROZEN: u8 = 2;

fn k(b: u8) -> Pubkey {
    Pubkey::new_from_array([b; 32])
}

///
///
fn mint_bytes(mint_authority: Option<Pubkey>, freeze: Option<Pubkey>, tlv: &[(u16, Vec<u8>)]) -> Vec<u8> {
    let mut data = vec![0u8; 82];
    if let Some(a) = mint_authority {
        data[0..4].copy_from_slice(&1u32.to_le_bytes());
        data[4..36].copy_from_slice(a.as_ref());
    }
    data[44] = 6;
    data[45] = 1;
    if let Some(f) = freeze {
        data[46..50].copy_from_slice(&1u32.to_le_bytes());
        data[50..82].copy_from_slice(f.as_ref());
    }
    if !tlv.is_empty() {
        data.resize(165, 0);
        data.push(1);
        for (ty, value) in tlv {
            data.extend_from_slice(&ty.to_le_bytes());
            data.extend_from_slice(&(value.len() as u16).to_le_bytes());
            data.extend_from_slice(value);
        }
    }
    data
}

fn opt(kk: Option<Pubkey>) -> [u8; 32] {
    kk.map(|p| p.to_bytes()).unwrap_or([0u8; 32])
}
fn pausable(authority: Option<Pubkey>, paused: bool) -> (u16, Vec<u8>) {
    let mut v = opt(authority).to_vec();
    v.push(paused as u8);
    (EXT_PAUSABLE, v)
}
fn permanent_delegate(d: Option<Pubkey>) -> (u16, Vec<u8>) {
    (EXT_PERMANENT_DELEGATE, opt(d).to_vec())
}
fn transfer_hook(authority: Option<Pubkey>, program: Option<Pubkey>) -> (u16, Vec<u8>) {
    let mut v = opt(authority).to_vec();
    v.extend_from_slice(&opt(program));
    (EXT_TRANSFER_HOOK, v)
}
fn default_state(s: u8) -> (u16, Vec<u8>) {
    (EXT_DEFAULT_ACCOUNT_STATE, vec![s])
}
fn zeros(ty: u16, n: usize) -> (u16, Vec<u8>) {
    (ty, vec![0u8; n])
}

const ALL: [(MintRole, PausableRule); 4] = [
    (MintRole::Base, PausableRule::Renounced),
    (MintRole::Base, PausableRule::HookVouched),
    (MintRole::ListedQuote, PausableRule::Renounced),
    (MintRole::ListedQuote, PausableRule::HookVouched),
];

fn t22(data: &[u8], role: MintRole, rule: PausableRule) -> Result<u8, GateError> {
    check_mint_supported(&TOKEN_2022_ID, &k(0xAA), data, role, rule)
}
fn spl(data: &[u8], role: MintRole, rule: PausableRule) -> Result<u8, GateError> {
    check_mint_supported(&SPL_TOKEN_ID, &k(0xAA), data, role, rule)
}

///
fn everywhere(data: &[u8], want: Result<u8, GateError>, why: &str) {
    for (role, rule) in ALL {
        assert_eq!(t22(data, role, rule), want, "{why} — {role:?}/{rule:?}");
    }
}

///
fn tier_b(data: &[u8], caps: u8, why: &str) {
    assert_eq!(t22(data, MintRole::Base, PausableRule::Renounced), Err(GateError::MintNotSupportedAsBase), "{why}");
    assert_eq!(t22(data, MintRole::Base, PausableRule::HookVouched), Err(GateError::MintNotSupportedAsBase), "{why} — the vouch lifts the pause authority alone");
    assert_eq!(t22(data, MintRole::ListedQuote, PausableRule::Renounced), Ok(caps), "{why}");
    assert_eq!(t22(data, MintRole::ListedQuote, PausableRule::HookVouched), Ok(caps), "{why}");
}





#[test]
fn main_s_accepting_shapes_are_accepted_with_zero_caps() {

    everywhere_spl(&mint_bytes(Some(k(1)), None, &[]), Ok(0), "classic SPL, no freeze");

    everywhere(&mint_bytes(Some(k(1)), None, &[pausable(None, false)]), Ok(0), "Pausable renounced, unpaused");

    everywhere(&mint_bytes(Some(k(1)), None, &[]), Ok(0), "Token-2022, no extension");

    everywhere(
        &mint_bytes(None, None, &[zeros(EXT_METADATA_POINTER, 64), (EXT_TOKEN_METADATA, vec![7u8; 40])]),
        Ok(0),
        "MetadataPointer + TokenMetadata",
    );
    everywhere(
        &mint_bytes(
            Some(k(1)),
            None,
            &[zeros(EXT_GROUP_POINTER, 64), zeros(EXT_TOKEN_GROUP, 80), zeros(EXT_GROUP_MEMBER_POINTER, 64), zeros(EXT_TOKEN_GROUP_MEMBER, 72)],
        ),
        Ok(0),
        "the group set",
    );
}

fn everywhere_spl(data: &[u8], want: Result<u8, GateError>, why: &str) {
    for (role, rule) in ALL {
        assert_eq!(spl(data, role, rule), want, "{why} — {role:?}/{rule:?}");
    }
}

///
///
#[test]
fn main_s_refusing_shapes_are_still_refused_as_a_base_mint() {


    let d = mint_bytes(None, None, &[zeros(EXT_TRANSFER_HOOK, 64)]);
    assert_eq!(t22(&d, MintRole::Base, PausableRule::Renounced), Err(GateError::MintNotSupportedAsBase));
    assert_eq!(t22(&d, MintRole::ListedQuote, PausableRule::Renounced), Ok(QUOTE_CAP_HOOK_SLOT));


    everywhere(&mint_bytes(None, None, &[zeros(EXT_MINT_CLOSE_AUTHORITY, 32)]), Err(GateError::MintNotSupported), "MintCloseAuthority");


    let d = mint_bytes(None, Some(k(9)), &[]);
    assert_eq!(t22(&d, MintRole::Base, PausableRule::Renounced), Err(GateError::MintNotSupportedAsBase));
    assert_eq!(t22(&d, MintRole::Base, PausableRule::HookVouched), Err(GateError::MintNotSupportedAsBase), "main: freeze stays refused under the vouch");
    assert_eq!(t22(&d, MintRole::ListedQuote, PausableRule::Renounced), Ok(QUOTE_CAP_FREEZE));



    let d = mint_bytes(None, None, &[pausable(Some(k(9)), false)]);
    assert_eq!(t22(&d, MintRole::Base, PausableRule::Renounced), Err(GateError::MintNotSupportedAsBase));
    assert_eq!(t22(&d, MintRole::Base, PausableRule::HookVouched), Ok(QUOTE_CAP_PAUSABLE), "main 27db686: any authority, at creation, hook-vouched");
    assert_eq!(t22(&d, MintRole::ListedQuote, PausableRule::Renounced), Ok(QUOTE_CAP_PAUSABLE));




    let d = mint_bytes(None, None, &[pausable(None, true)]);
    assert_eq!(t22(&d, MintRole::Base, PausableRule::Renounced), Err(GateError::MintNotSupported));
    assert_eq!(t22(&d, MintRole::Base, PausableRule::HookVouched), Ok(0), "main 27db686: paused or not, hook-vouched");
    assert_eq!(t22(&d, MintRole::ListedQuote, PausableRule::Renounced), Err(GateError::MintNotSupported), "a paused quote can never trade");
    assert_eq!(t22(&d, MintRole::ListedQuote, PausableRule::HookVouched), Err(GateError::MintNotSupported), "the vouch is the base side's alone");


    let d = mint_bytes(None, None, &[]);
    for (role, rule) in ALL {
        assert_eq!(check_mint_supported(&TOKEN_2022_ID, &TOKEN_2022_NATIVE_MINT, &d, role, rule), Err(GateError::MintNotSupported), "{role:?}/{rule:?}");
    }
}





#[test]
fn the_prepare_graduation_shape_passes_only_under_hook_vouched() {

    let d = mint_bytes(Some(k(1)), None, &[pausable(Some(k(0x10)), true)]);
    assert_eq!(t22(&d, MintRole::Base, PausableRule::HookVouched), Ok(QUOTE_CAP_PAUSABLE), "the ONE arm every prepare_graduation reverts on");
    assert_eq!(t22(&d, MintRole::Base, PausableRule::Renounced), Err(GateError::MintNotSupported), "paused, unvouched: the Tier-C word, before any role");
    assert_eq!(t22(&d, MintRole::ListedQuote, PausableRule::Renounced), Err(GateError::MintNotSupported));
    assert_eq!(t22(&d, MintRole::ListedQuote, PausableRule::HookVouched), Err(GateError::MintNotSupported), "a paused QUOTE is refused whatever the caller vouches");



    for (extra, why) in [
        (permanent_delegate(Some(k(2))), "a permanent delegate beside the vouched pause"),
        (default_state(ACCOUNT_STATE_INITIALIZED), "a default state beside the vouched pause"),
        (transfer_hook(Some(k(3)), None), "a hook slot beside the vouched pause"),
    ] {
        let d = mint_bytes(Some(k(1)), None, &[pausable(Some(k(0x10)), true), extra]);
        assert_eq!(t22(&d, MintRole::Base, PausableRule::HookVouched), Err(GateError::MintNotSupportedAsBase), "{why}");
    }
    let d = mint_bytes(Some(k(1)), Some(k(4)), &[pausable(Some(k(0x10)), true)]);
    assert_eq!(t22(&d, MintRole::Base, PausableRule::HookVouched), Err(GateError::MintNotSupportedAsBase), "a freeze authority beside the vouched pause");

    let d = mint_bytes(Some(k(1)), None, &[pausable(Some(k(0x10)), true), zeros(EXT_TRANSFER_FEE_CONFIG, 108)]);
    assert_eq!(t22(&d, MintRole::Base, PausableRule::HookVouched), Err(GateError::MintNotSupported), "a transfer fee beside the vouched pause");
}





#[test]
fn tier_a_is_recorded_and_refuses_nothing_in_either_role() {
    everywhere(&mint_bytes(Some(k(1)), None, &[zeros(EXT_SCALED_UI_AMOUNT, 56)]), Ok(QUOTE_CAP_SCALED_UI), "ScaledUiAmount");
    everywhere(&mint_bytes(Some(k(1)), None, &[zeros(EXT_INTEREST_BEARING_CONFIG, 52)]), Ok(QUOTE_CAP_INTEREST), "InterestBearingConfig");
    everywhere(&mint_bytes(Some(k(1)), None, &[zeros(EXT_CONFIDENTIAL_TRANSFER_MINT, 65)]), Ok(QUOTE_CAP_CONFIDENTIAL), "ConfidentialTransferMint");
    everywhere(
        &mint_bytes(Some(k(1)), None, &[zeros(EXT_SCALED_UI_AMOUNT, 56), zeros(EXT_INTEREST_BEARING_CONFIG, 52), zeros(EXT_CONFIDENTIAL_TRANSFER_MINT, 65)]),
        Ok(QUOTE_CAPS_INERT),
        "all three",
    );

    assert_eq!(check_admitted(QUOTE_CAPS_INERT, 0), Ok(()));
}





#[test]
fn a_live_freeze_authority_is_tier_b() {
    tier_b(&mint_bytes(Some(k(1)), Some(k(2)), &[]), QUOTE_CAP_FREEZE, "freeze authority");
    tier_b(&mint_bytes(Some(k(1)), Some(k(2)), &[zeros(EXT_METADATA_POINTER, 64)]), QUOTE_CAP_FREEZE, "freeze authority beside metadata");
}

#[test]
fn a_permanent_delegate_is_tier_b_only_when_a_delegate_is_set() {
    tier_b(&mint_bytes(Some(k(1)), None, &[permanent_delegate(Some(k(2)))]), QUOTE_CAP_PERMANENT_DELEGATE, "a delegate");
    everywhere(&mint_bytes(Some(k(1)), None, &[permanent_delegate(None)]), Ok(0), "the extension with no delegate holds no power");
}

#[test]
fn a_live_pause_authority_is_tier_b_and_a_renounced_one_is_nothing() {
    let d = mint_bytes(Some(k(1)), None, &[pausable(Some(k(2)), false)]);
    assert_eq!(t22(&d, MintRole::Base, PausableRule::Renounced), Err(GateError::MintNotSupportedAsBase));
    assert_eq!(t22(&d, MintRole::ListedQuote, PausableRule::Renounced), Ok(QUOTE_CAP_PAUSABLE));
    assert_eq!(t22(&d, MintRole::ListedQuote, PausableRule::HookVouched), Ok(QUOTE_CAP_PAUSABLE));
    everywhere(&mint_bytes(Some(k(1)), None, &[pausable(None, false)]), Ok(0), "renounced, unpaused");
}

#[test]
fn a_default_account_state_is_tier_b_whatever_its_value() {
    tier_b(&mint_bytes(Some(k(1)), None, &[default_state(ACCOUNT_STATE_INITIALIZED)]), QUOTE_CAP_DEFAULT_STATE, "default Initialized");


    tier_b(&mint_bytes(Some(k(1)), None, &[default_state(ACCOUNT_STATE_FROZEN)]), QUOTE_CAP_DEFAULT_STATE, "default Frozen");
}

#[test]
fn a_transfer_hook_latch_is_tier_b_and_an_armed_hook_is_tier_c() {

    tier_b(&mint_bytes(Some(k(1)), None, &[transfer_hook(Some(k(2)), None)]), QUOTE_CAP_HOOK_SLOT, "the latch, live authority");

    tier_b(&mint_bytes(Some(k(1)), None, &[transfer_hook(None, None)]), QUOTE_CAP_HOOK_SLOT, "the latch, no authority");

    everywhere(&mint_bytes(Some(k(1)), None, &[transfer_hook(Some(k(2)), Some(k(3)))]), Err(GateError::MintNotSupported), "armed, live authority");
    everywhere(&mint_bytes(Some(k(1)), None, &[transfer_hook(None, Some(k(3)))]), Err(GateError::MintNotSupported), "armed, renounced authority");
}

#[test]
fn the_five_custodial_bits_together_are_the_custodial_mask_and_nothing_else_sets_a_tier_b_bit() {
    let d = mint_bytes(
        Some(k(1)),
        Some(k(2)),
        &[permanent_delegate(Some(k(3))), pausable(Some(k(4)), false), default_state(ACCOUNT_STATE_INITIALIZED), transfer_hook(Some(k(5)), None)],
    );
    tier_b(&d, QUOTE_CAPS_CUSTODIAL, "all five");
    assert_eq!(QUOTE_CAPS_CUSTODIAL, 0b0001_1111, "bits 0..=4, lane A's positions");
    assert_eq!(QUOTE_CAPS_INERT, 0b1110_0000, "bits 5..=7");
    assert_eq!(QUOTE_CAPS_CUSTODIAL & QUOTE_CAPS_INERT, 0);
    assert_eq!(QUOTE_CAPS_CUSTODIAL | QUOTE_CAPS_INERT, u8::MAX, "every bit of the byte is named");
}





#[test]
fn tier_c_is_refused_in_every_role() {
    for (tlv, why) in [
        (zeros(EXT_TRANSFER_FEE_CONFIG, 108), "TransferFeeConfig"),
        (zeros(EXT_TRANSFER_FEE_AMOUNT, 8), "TransferFeeAmount"),
        (zeros(EXT_CONFIDENTIAL_TRANSFER_FEE_CONFIG, 129), "ConfidentialTransferFeeConfig"),
        (zeros(EXT_CONFIDENTIAL_TRANSFER_FEE_AMOUNT, 64), "ConfidentialTransferFeeAmount"),
        (zeros(EXT_NON_TRANSFERABLE, 0), "NonTransferable"),
        (zeros(EXT_MINT_CLOSE_AUTHORITY, 32), "MintCloseAuthority, no authority"),
        ((EXT_MINT_CLOSE_AUTHORITY, k(7).to_bytes().to_vec()), "MintCloseAuthority, live"),
        (zeros(EXT_CONFIDENTIAL_MINT_BURN, 68), "ConfidentialMintBurn (the `_` tail)"),
        (zeros(EXT_CPI_GUARD, 1), "an account-side type on a mint (the `_` tail)"),
        (zeros(EXT_IMMUTABLE_OWNER, 0), "another account-side type (the `_` tail)"),
        (zeros(EXT_UNKNOWN, 4), "an extension id this interface does not know"),
    ] {
        everywhere(&mint_bytes(Some(k(1)), None, &[tlv]), Err(GateError::MintNotSupported), why);
    }
}

#[test]
fn a_tier_c_arm_beside_a_tier_b_one_is_the_tier_c_word_even_as_a_base() {


    let d = mint_bytes(Some(k(1)), Some(k(2)), &[zeros(EXT_TRANSFER_FEE_CONFIG, 108)]);
    everywhere(&d, Err(GateError::MintNotSupported), "freeze + transfer fee");

    let d = mint_bytes(Some(k(1)), None, &[permanent_delegate(Some(k(2))), zeros(EXT_TRANSFER_FEE_CONFIG, 108)]);
    everywhere(&d, Err(GateError::MintNotSupported), "delegate then fee");
}





#[test]
fn an_owner_that_is_neither_token_program_is_refused() {
    let d = mint_bytes(Some(k(1)), None, &[]);
    for (role, rule) in ALL {
        assert_eq!(check_mint_supported(&Pubkey::default(), &k(0xAA), &d, role, rule), Err(GateError::MintNotSupported), "system-owned");
        assert_eq!(check_mint_supported(&k(0x55), &k(0xAA), &d, role, rule), Err(GateError::MintNotSupported), "some program");
    }
}

#[test]
fn bytes_the_interface_cannot_parse_are_refused() {

    let mut d = mint_bytes(Some(k(1)), None, &[]);
    d[45] = 0;
    everywhere(&d, Err(GateError::MintNotSupported), "is_initialized = 0");

    everywhere(&[0u8; 355], Err(GateError::MintNotSupported), "Multisig::LEN");

    everywhere(&[1u8; 40], Err(GateError::MintNotSupported), "40 bytes");

    let mut d = mint_bytes(Some(k(1)), None, &[zeros(EXT_METADATA_POINTER, 64)]);
    let n = d.len();
    d.truncate(n - 10);
    everywhere(&d, Err(GateError::MintNotSupported), "truncated TLV");

    let mut d = mint_bytes(Some(k(1)), None, &[zeros(EXT_METADATA_POINTER, 64)]);
    d[165] = 2;
    everywhere(&d, Err(GateError::MintNotSupported), "AccountType::Account");

    let mut d = mint_bytes(Some(k(1)), None, &[zeros(EXT_METADATA_POINTER, 64)]);
    d[100] = 1;
    everywhere(&d, Err(GateError::MintNotSupported), "dirty padding");
}

#[test]
fn a_classic_spl_mint_is_exactly_82_bytes_and_passes_with_its_freeze_bit_recorded() {
    everywhere_spl(&mint_bytes(Some(k(1)), None, &[]), Ok(0), "no freeze");

    everywhere_spl(&mint_bytes(Some(k(1)), Some(k(2)), &[]), Ok(QUOTE_CAP_FREEZE), "USDC's shape");


    everywhere_spl(&mint_bytes(Some(k(1)), None, &[zeros(EXT_TRANSFER_FEE_CONFIG, 108)]), Err(GateError::MintNotSupported), "TLV under the SPL owner");
    everywhere_spl(&[0u8; 81], Err(GateError::MintNotSupported), "81 bytes");
    let mut d = mint_bytes(Some(k(1)), None, &[]);
    d[45] = 0;
    everywhere_spl(&d, Err(GateError::MintNotSupported), "uninitialised");


    let d = mint_bytes(None, None, &[]);
    for (role, rule) in ALL {
        assert_eq!(check_mint_supported(&SPL_TOKEN_ID, &TOKEN_2022_NATIVE_MINT, &d, role, rule), Ok(0), "{role:?}/{rule:?}: the key is only refused under Token-2022");
    }
    assert_eq!(CLASSIC_MINT_LEN, <spl_token_2022_interface::state::Mint as solana_program_pack::Pack>::LEN);
}





#[test]
fn check_admitted_is_the_custodial_bits_minus_the_mask_and_nothing_else() {
    for bit in [QUOTE_CAP_FREEZE, QUOTE_CAP_PERMANENT_DELEGATE, QUOTE_CAP_PAUSABLE, QUOTE_CAP_DEFAULT_STATE, QUOTE_CAP_HOOK_SLOT] {
        assert_eq!(check_admitted(bit, 0), Err(GateError::CapabilityNotAdmitted), "bit {bit:#010b} under the default mask");
        assert_eq!(check_admitted(bit, bit), Ok(()));
        assert_eq!(check_admitted(bit, QUOTE_CAPS_CUSTODIAL & !bit), Err(GateError::CapabilityNotAdmitted), "every other bit admitted but this one");
        assert_eq!(check_admitted(bit, QUOTE_CAPS_CUSTODIAL), Ok(()));
    }
    for bit in [QUOTE_CAP_SCALED_UI, QUOTE_CAP_INTEREST, QUOTE_CAP_CONFIDENTIAL] {
        assert_eq!(check_admitted(bit, 0), Ok(()), "Tier A is never subject to the mask");
    }
    assert_eq!(check_admitted(0, 0), Ok(()), "no capability, nothing to admit");
    assert_eq!(check_admitted(u8::MAX, QUOTE_CAPS_CUSTODIAL), Ok(()));
    assert_eq!(check_admitted(u8::MAX, u8::MAX), Ok(()));

    assert_eq!(check_admitted(QUOTE_CAP_FREEZE, QUOTE_CAPS_INERT), Err(GateError::CapabilityNotAdmitted));
}





#[test]
fn the_digest_hashes_freeze_delegate_pause_hook_in_that_order() {
    let d = mint_bytes(Some(k(1)), Some(k(0xF1)), &[permanent_delegate(Some(k(0xD2))), pausable(Some(k(0xA3)), false), transfer_hook(Some(k(0xB4)), None)]);
    let mut buf = [0u8; 128];
    buf[0..32].copy_from_slice(&[0xF1; 32]);
    buf[32..64].copy_from_slice(&[0xD2; 32]);
    buf[64..96].copy_from_slice(&[0xA3; 32]);
    buf[96..128].copy_from_slice(&[0xB4; 32]);
    assert_eq!(authority_digest(&d), Ok(solana_sha256_hasher::hash(&buf).to_bytes()));


    let mut buf = [0u8; 128];
    buf[0..32].copy_from_slice(&[0xF1; 32]);
    assert_eq!(authority_digest(&mint_bytes(Some(k(1)), Some(k(0xF1)), &[])), Ok(solana_sha256_hasher::hash(&buf).to_bytes()));

    assert_eq!(authority_digest(&mint_bytes(Some(k(1)), None, &[])), Ok(solana_sha256_hasher::hash(&[0u8; 128]).to_bytes()));
    assert_eq!(authority_digest(&mint_bytes(None, None, &[pausable(None, false), permanent_delegate(None)])), Ok(solana_sha256_hasher::hash(&[0u8; 128]).to_bytes()));

    assert_eq!(authority_digest(&mint_bytes(Some(k(1)), None, &[])), authority_digest(&mint_bytes(Some(k(2)), None, &[])));

    assert_eq!(
        authority_digest(&mint_bytes(None, None, &[transfer_hook(Some(k(0xB4)), None)])),
        authority_digest(&mint_bytes(None, None, &[transfer_hook(Some(k(0xB4)), Some(k(0xC5)))])),
    );

    assert_ne!(authority_digest(&mint_bytes(None, Some(k(1)), &[])), authority_digest(&mint_bytes(None, Some(k(2)), &[])));

    assert_eq!(authority_digest(&[0u8; 40]), Err(GateError::MintNotSupported));
}

#[test]
fn the_errors_name_themselves() {
    assert_eq!(GateError::MintNotSupported.to_string(), "MintNotSupported");
    assert_eq!(GateError::MintNotSupportedAsBase.to_string(), "MintNotSupportedAsBase");
    assert_eq!(GateError::CapabilityNotAdmitted.to_string(), "CapabilityNotAdmitted");
}











///
const HOLDERS: u8 = 0x71;
const LOCK: u8 = 0x10;

fn t22h(data: &[u8], role: MintRole, rule: PausableRule, base_hook: BaseHook) -> Result<u8, GateError> {
    check_mint_supported_with_hook(&TOKEN_2022_ID, &k(0xAA), data, role, rule, base_hook)
}

///
///
fn floor_hooked_mint_at_creation() -> Vec<u8> {
    mint_bytes(
        Some(k(LOCK)),
        None,
        &[zeros(EXT_METADATA_POINTER, 64), pausable(Some(k(LOCK)), true), transfer_hook(Some(k(LOCK)), None)],
    )
}

///
///
fn floor_hooked_mint_graduated() -> Vec<u8> {
    mint_bytes(None, None, &[zeros(EXT_METADATA_POINTER, 64), pausable(None, false), transfer_hook(None, Some(k(HOLDERS)))])
}

#[test]
fn floor_s_empty_slot_and_its_armed_hook_are_admitted_on_a_vouched_base() {
    let admitted = BaseHook::Admitted(k(HOLDERS));

    assert_eq!(
        t22h(&floor_hooked_mint_at_creation(), MintRole::Base, PausableRule::HookVouched, admitted),
        Ok(QUOTE_CAP_PAUSABLE | QUOTE_CAP_HOOK_SLOT),
        "the ONE arm every hooked graduation reverts on"
    );

    assert_eq!(t22h(&floor_hooked_mint_graduated(), MintRole::Base, PausableRule::HookVouched, admitted), Ok(0));

    let d = mint_bytes(Some(k(1)), None, &[transfer_hook(None, Some(k(HOLDERS)))]);
    assert_eq!(t22h(&d, MintRole::Base, PausableRule::HookVouched, admitted), Ok(0));
}

#[test]
fn a_third_party_s_empty_slot_under_another_approved_hook_is_refused() {



    for base_hook in [BaseHook::None, BaseHook::Admitted(Pubkey::default())] {
        for authority in [Some(k(0x33)), None, Some(k(LOCK))] {
            let d = mint_bytes(Some(k(1)), None, &[transfer_hook(authority, None)]);
            assert_eq!(
                t22h(&d, MintRole::Base, PausableRule::HookVouched, base_hook),
                Err(GateError::MintNotSupportedAsBase),
                "{base_hook:?}, slot authority {authority:?}"
            );
        }
        assert_eq!(
            t22h(&floor_hooked_mint_at_creation(), MintRole::Base, PausableRule::HookVouched, base_hook),
            Err(GateError::MintNotSupportedAsBase),
            "FLOOR's own launch mint under a hook with nothing admitted: {base_hook:?}"
        );
    }
}

#[test]
fn an_armed_hook_that_is_not_the_admitted_program_is_refused() {
    let admitted = BaseHook::Admitted(k(HOLDERS));
    for (authority, why) in [(None, "renounced"), (Some(k(2)), "live")] {
        let d = mint_bytes(Some(k(1)), None, &[transfer_hook(authority, Some(k(0x99)))]);
        assert_eq!(t22h(&d, MintRole::Base, PausableRule::HookVouched, admitted), Err(GateError::MintNotSupported), "{why}");
        assert_eq!(t22h(&d, MintRole::Base, PausableRule::HookVouched, BaseHook::None), Err(GateError::MintNotSupported), "{why}");
    }
}

#[test]
fn the_admitted_program_with_a_live_slot_authority_is_refused() {

    let admitted = BaseHook::Admitted(k(HOLDERS));
    for authority in [k(LOCK), k(0x44)] {
        let d = mint_bytes(Some(k(1)), None, &[transfer_hook(Some(authority), Some(k(HOLDERS)))]);
        assert_eq!(t22h(&d, MintRole::Base, PausableRule::HookVouched, admitted), Err(GateError::MintNotSupported));
    }
}

#[test]
fn a_hook_on_a_quote_leg_is_refused_whatever_is_admitted() {
    let admitted = BaseHook::Admitted(k(HOLDERS));

    let armed = mint_bytes(Some(k(1)), None, &[transfer_hook(None, Some(k(HOLDERS)))]);
    for rule in [PausableRule::Renounced, PausableRule::HookVouched] {
        assert_eq!(t22h(&armed, MintRole::ListedQuote, rule, admitted), Err(GateError::MintNotSupported), "{rule:?}");
    }


    let latch = mint_bytes(Some(k(1)), None, &[transfer_hook(Some(k(2)), None)]);
    for rule in [PausableRule::Renounced, PausableRule::HookVouched] {
        assert_eq!(t22h(&latch, MintRole::ListedQuote, rule, admitted), Ok(QUOTE_CAP_HOOK_SLOT), "{rule:?}");
        assert_eq!(
            t22h(&latch, MintRole::ListedQuote, rule, admitted),
            t22h(&latch, MintRole::ListedQuote, rule, BaseHook::None),
            "{rule:?}: admission never reaches the quote role"
        );
    }
}

#[test]
fn the_admission_needs_the_vouch() {


    let admitted = BaseHook::Admitted(k(HOLDERS));
    let empty = mint_bytes(Some(k(1)), None, &[transfer_hook(Some(k(LOCK)), None)]);
    assert_eq!(t22h(&empty, MintRole::Base, PausableRule::Renounced, admitted), Err(GateError::MintNotSupportedAsBase));
    let armed = mint_bytes(Some(k(1)), None, &[transfer_hook(None, Some(k(HOLDERS)))]);
    assert_eq!(t22h(&armed, MintRole::Base, PausableRule::Renounced, admitted), Err(GateError::MintNotSupported));
    assert_eq!(t22h(&floor_hooked_mint_graduated(), MintRole::Base, PausableRule::Renounced, admitted), Err(GateError::MintNotSupported));
}

#[test]
fn the_admission_lifts_the_latch_and_nothing_else() {
    let admitted = BaseHook::Admitted(k(HOLDERS));

    for (extra, why) in [
        (permanent_delegate(Some(k(2))), "a permanent delegate"),
        (default_state(ACCOUNT_STATE_INITIALIZED), "a default account state"),
    ] {
        let d = mint_bytes(Some(k(1)), None, &[transfer_hook(Some(k(LOCK)), None), extra]);
        assert_eq!(t22h(&d, MintRole::Base, PausableRule::HookVouched, admitted), Err(GateError::MintNotSupportedAsBase), "{why}");
    }
    let d = mint_bytes(Some(k(1)), Some(k(4)), &[transfer_hook(Some(k(LOCK)), None)]);
    assert_eq!(t22h(&d, MintRole::Base, PausableRule::HookVouched, admitted), Err(GateError::MintNotSupportedAsBase), "a freeze authority");

    for hook in [transfer_hook(Some(k(LOCK)), None), transfer_hook(None, Some(k(HOLDERS)))] {
        let d = mint_bytes(Some(k(1)), None, &[hook, zeros(EXT_TRANSFER_FEE_CONFIG, 108)]);
        assert_eq!(t22h(&d, MintRole::Base, PausableRule::HookVouched, admitted), Err(GateError::MintNotSupported), "a transfer fee");
    }
}

#[test]
fn check_mint_supported_is_the_hook_form_with_nothing_admitted() {


    let shapes = [
        mint_bytes(Some(k(1)), None, &[]),
        mint_bytes(Some(k(1)), Some(k(2)), &[]),
        mint_bytes(Some(k(1)), None, &[pausable(Some(k(0x10)), true)]),
        mint_bytes(Some(k(1)), None, &[transfer_hook(Some(k(2)), None)]),
        mint_bytes(Some(k(1)), None, &[transfer_hook(None, Some(k(HOLDERS)))]),
        floor_hooked_mint_at_creation(),
        floor_hooked_mint_graduated(),
        mint_bytes(Some(k(1)), None, &[zeros(EXT_TRANSFER_FEE_CONFIG, 108)]),
    ];
    for d in &shapes {
        for (role, rule) in ALL {
            assert_eq!(t22(d, role, rule), t22h(d, role, rule, BaseHook::None), "{role:?}/{rule:?}");
        }
    }
}
