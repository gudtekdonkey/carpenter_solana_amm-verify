use anchor_lang::prelude::*;
use carpenter_amm::amm_core::transfer_hook::append_hook_accounts;
use carpenter_amm::errors::AmmError;

#[test]
fn holder_claim_without_forwarded_quote_hook_accounts_is_refused() {
    let key = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let hook = Pubkey::new_unique();
    let mut lamports = 0;
    let mut data = [];
    let account = AccountInfo::new(&key, false, false, &mut lamports, &mut data, &owner, false);
    let result = append_hook_accounts(&mut Vec::new(), &mut Vec::new(), &hook,
        &account, &account, &account, &account, 1, &[]);
    assert_eq!(result.unwrap_err(), error!(AmmError::TransferHookAccountsMissing));
}
