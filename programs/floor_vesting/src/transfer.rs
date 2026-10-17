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
//!

use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::AccountMeta;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::token_2022::spl_token_2022;
use anchor_spl::token_2022::spl_token_2022::extension::transfer_hook::TransferHook;
use anchor_spl::token_2022::spl_token_2022::extension::{BaseStateWithExtensions, StateWithExtensions};
use anchor_spl::token_interface::{transfer_checked, TransferChecked};

use crate::errors::VestingError;

///
///
///
///
///
pub const EXTRA_ACCOUNT_METAS: &[u8] = b"extra-account-metas";

///
///
///
///
///
///
///
///
pub fn armed_hook_in(mint_data: &[u8]) -> Option<Pubkey> {
    let state = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(mint_data).ok()?;
    let hook = state.get_extension::<TransferHook>().ok()?;
    let program = Pubkey::new_from_array(hook.program_id.0.to_bytes());
    (program != Pubkey::default()).then_some(program)
}

///
///
pub fn armed_hook_program(mint: &AccountInfo<'_>) -> Result<Option<Pubkey>> {
    let data = mint.try_borrow_data()?;
    Ok(armed_hook_in(&data))
}

///
///
///
///
///
pub fn forwarded(key: &Pubkey, is_writable: bool, authority: &Pubkey, vesting_authority: &Pubkey) -> Option<AccountMeta> {
    if key == authority || key == vesting_authority {
        return None;
    }
    Some(AccountMeta { pubkey: *key, is_signer: false, is_writable })
}

///
pub struct Transfer<'a, 'info> {
///
    pub token_program: Pubkey,
    pub from: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub to: AccountInfo<'info>,
///
    pub authority: AccountInfo<'info>,
///
///
    pub vesting_authority: Pubkey,
///
///
    pub remaining: &'a [AccountInfo<'info>],
}

impl<'a, 'info> Transfer<'a, 'info> {
///
///
    pub fn run(self, amount: u64, decimals: u8, signer_seeds: &[&[&[u8]]]) -> Result<()> {
        let Some(hook) = armed_hook_program(&self.mint)? else {



            return transfer_checked(
                CpiContext::new_with_signer(
                    self.token_program,
                    TransferChecked { from: self.from, mint: self.mint, to: self.to, authority: self.authority },
                    signer_seeds,
                ),
                amount,
                decimals,
            );
        };



        let (validation, _) = Pubkey::find_program_address(&[EXTRA_ACCOUNT_METAS, self.mint.key.as_ref()], &hook);
        let passed = |key: &Pubkey| self.remaining.iter().any(|a| a.key == key);
        require!(passed(&hook) && passed(&validation), VestingError::HookAccountsMissing);

        let mut ix = spl_token_2022::instruction::transfer_checked(
            &self.token_program,
            self.from.key,
            self.mint.key,
            self.to.key,
            self.authority.key,
            &[],
            amount,
            decimals,
        )?;
        let mut infos = Vec::with_capacity(4 + self.remaining.len());
        infos.extend([self.from.clone(), self.mint.clone(), self.to.clone(), self.authority.clone()]);
        for account in self.remaining {
            if let Some(meta) = forwarded(account.key, account.is_writable, self.authority.key, &self.vesting_authority) {
                ix.accounts.push(meta);
                infos.push(account.clone());
            }
        }
        invoke_signed(&ix, &infos, signer_seeds).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

///
    const EXT_TRANSFER_HOOK: u16 = 14;
    const EXT_PAUSABLE: u16 = 26;

///
///
    fn mint_bytes(exts: &[(u16, Vec<u8>)]) -> Vec<u8> {
        let mut data = vec![0u8; 82];
        data[44] = 6;
        data[45] = 1;
        if exts.is_empty() {
            return data;
        }
        data.resize(165, 0);
        data.push(1);
        for (t, v) in exts {
            data.extend_from_slice(&t.to_le_bytes());
            data.extend_from_slice(&(v.len() as u16).to_le_bytes());
            data.extend_from_slice(v);
        }
        data
    }

///
    fn hook_ext(authority: Pubkey, program: Pubkey) -> (u16, Vec<u8>) {
        let mut v = authority.to_bytes().to_vec();
        v.extend_from_slice(&program.to_bytes());
        (EXT_TRANSFER_HOOK, v)
    }

    fn pausable_ext() -> (u16, Vec<u8>) {
        let mut v = Pubkey::new_unique().to_bytes().to_vec();
        v.push(0);
        (EXT_PAUSABLE, v)
    }

    #[test]
    fn a_classic_mint_and_a_hookless_mint_are_not_armed() {
        assert_eq!(armed_hook_in(&mint_bytes(&[])), None, "82 bytes, no extensions");
        assert_eq!(armed_hook_in(&mint_bytes(&[pausable_ext()])), None, "Pausable only");
    }

    #[test]
    fn an_empty_hook_slot_is_not_armed_whoever_holds_its_authority() {


        let slot = hook_ext(Pubkey::new_unique(), Pubkey::default());
        assert_eq!(armed_hook_in(&mint_bytes(&[slot.clone()])), None);
        assert_eq!(armed_hook_in(&mint_bytes(&[pausable_ext(), slot])), None);
    }

    #[test]
    fn an_armed_hook_is_read_with_or_without_its_authority_and_beside_other_extensions() {
        let program = Pubkey::new_unique();

        assert_eq!(armed_hook_in(&mint_bytes(&[hook_ext(Pubkey::default(), program)])), Some(program));

        let live = hook_ext(Pubkey::new_unique(), program);
        assert_eq!(armed_hook_in(&mint_bytes(&[pausable_ext(), live.clone()])), Some(program));
        assert_eq!(armed_hook_in(&mint_bytes(&[live, pausable_ext()])), Some(program));
    }

    #[test]
    fn bytes_that_are_no_mint_are_never_armed() {
        assert_eq!(armed_hook_in(&[]), None);
        assert_eq!(armed_hook_in(&[0u8; 40]), None);

        let mut data = mint_bytes(&[hook_ext(Pubkey::default(), Pubkey::new_unique())]);
        data[45] = 0;
        assert_eq!(armed_hook_in(&data), None);
    }

    #[test]
    fn the_authority_and_the_escrow_authority_are_withheld_and_nothing_is_ever_a_signer() {
        let authority = Pubkey::new_unique();
        let vesting_authority = Pubkey::new_unique();
        assert!(forwarded(&authority, true, &authority, &vesting_authority).is_none(), "the authority");
        assert!(forwarded(&vesting_authority, false, &authority, &vesting_authority).is_none(), "the escrow authority");

        assert!(forwarded(&vesting_authority, true, &vesting_authority, &vesting_authority).is_none());

        let other = Pubkey::new_unique();
        for w in [false, true] {
            let m = forwarded(&other, w, &authority, &vesting_authority).expect("forwarded");
            assert_eq!(m.pubkey, other);
            assert!(!m.is_signer, "never a signer, whatever the transaction made it");
            assert_eq!(m.is_writable, w, "writability is the transaction's own");
        }
    }

    #[test]
    fn the_validation_seed_is_the_interfaces() {


        assert_eq!(EXTRA_ACCOUNT_METAS, b"extra-account-metas");
    }
}
