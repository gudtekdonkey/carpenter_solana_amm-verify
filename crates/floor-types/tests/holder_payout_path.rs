#[test]
fn legacy_claim_cannot_forward_hooked_quote_accounts() {
    let claim = include_str!("../../../programs/floor_holders/src/instructions/claim.rs");
    let cpi = include_str!("../../../programs/floor_holders/src/hook_cpi.rs");
    let transfer = include_str!("../../../programs/carpenter_amm/src/amm_core/transfer_hook.rs");
    assert!(claim.contains("hook_cpi::pay_holder(&cpi, paid)?"));
    assert!(!cpi.contains("remaining_accounts"));
    assert!(transfer.contains("find(hook_program).ok_or(AmmError::TransferHookAccountsMissing)?"));
}
