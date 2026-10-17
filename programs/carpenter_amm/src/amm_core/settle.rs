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
use carpenter_types::hook::BalanceDelta;

use crate::amm_core::vaults::{transfer_checked_in, transfer_checked_out, TransferIn};
use crate::errors::AmmError;

///
///
///
///
pub fn projected_vault(before0: u64, before1: u64, trader: &BalanceDelta, self_call: bool) -> Result<(u64, u64)> {
    if self_call {
        return Ok((before0, before1));
    }
    let project = |before: u64, delta: i128| -> Result<u64> {
        let v = (before as i128).checked_sub(delta).ok_or(AmmError::MathOverflow)?;
        require!(v >= 0, AmmError::VaultConservationViolated);
        u64::try_from(v).map_err(|_| AmmError::MathOverflow.into())
    };
    Ok((project(before0, trader.amount0)?, project(before1, trader.amount1)?))
}

///
///
///
///
///
#[allow(clippy::too_many_arguments)]
pub fn pay_in<'a, 'info>(
    token_program: &'a AccountInfo<'info>,
    trader_token_in: &'a AccountInfo<'info>,
    mint_in: &'a AccountInfo<'info>,
    vault_in: &'a AccountInfo<'info>,
    trader: &'a AccountInfo<'info>,
    pool: &'a AccountInfo<'info>,
    pool_seeds: &'a [&'a [u8]],
    amount: u64,
    decimals: u8,
    admitted: Option<Pubkey>,
    extra: &'a [AccountInfo<'info>],
) -> Result<()> {
    transfer_checked_in(
        &TransferIn {
            token_program,
            source: trader_token_in,
            mint: mint_in,
            vault: vault_in,
            owner: trader,
            pool,
            pool_signer_seeds: pool_seeds,
            decimals,
            admitted,
            extra,
        },
        amount,
    )
}

///
///
#[allow(clippy::too_many_arguments)]
pub fn pay_out<'info>(
    token_program: &AccountInfo<'info>,
    vault_out: &AccountInfo<'info>,
    mint_out: &AccountInfo<'info>,
    trader_token_out: &AccountInfo<'info>,
    pool: &AccountInfo<'info>,
    pool_seeds: &[&[u8]],
    amount: u64,
    decimals: u8,
    admitted: Option<Pubkey>,
    extra: &[AccountInfo<'info>],
) -> Result<()> {
    transfer_checked_out(token_program, vault_out, mint_out, trader_token_out, pool, pool_seeds, amount, decimals, admitted, extra)
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
///
///
///
pub fn check_vault_conservation(
    before: u64,
    after: u64,
    transferred_in: u64,
    transferred_out: u64,
    hook_claims: u64,
    protocol_fees_owed: u64,
) -> Result<()> {
    let expected = (before as u128)
        .checked_add(transferred_in as u128)
        .and_then(|v| v.checked_sub(transferred_out as u128))
        .ok_or(AmmError::VaultConservationViolated)?;
    require!(after as u128 == expected, AmmError::VaultConservationViolated);
    require!(
        hook_claims as u128 + protocol_fees_owed as u128 <= after as u128,
        AmmError::VaultShortfall
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_projected_vault_follows_the_trader_delta() {
        let td = BalanceDelta { amount0: -100, amount1: 40 };
        assert_eq!(projected_vault(1_000, 500, &td, false).unwrap(), (1_100, 460));
        assert_eq!(projected_vault(1_000, 500, &td, true).unwrap(), (1_000, 500), "a self-call moves no tokens");
        let too_much = BalanceDelta { amount0: 0, amount1: 501 };
        assert!(projected_vault(1_000, 500, &too_much, false).is_err());
    }

    #[test]
    fn conservation_is_an_exact_equality_plus_a_claims_bound() {
        assert!(check_vault_conservation(100, 150, 50, 0, 150, 0).is_ok());
        assert!(check_vault_conservation(100, 60, 0, 40, 10, 50).is_ok());
        assert!(check_vault_conservation(100, 151, 50, 0, 0, 0).is_err(), "a gift mid-instruction is refused");
        assert!(check_vault_conservation(100, 60, 0, 40, 11, 50).is_err(), "claims over the vault");
        assert!(check_vault_conservation(10, 0, 0, 11, 0, 0).is_err());
    }

///
    #[test]
    fn a_shortfall_and_an_accounting_bug_are_different_errors() {
        let code = |r: Result<()>| match r.unwrap_err() {
            anchor_lang::error::Error::AnchorError(e) => e.error_code_number,
            other => panic!("expected an AnchorError, got {other:?}"),
        };
        let exact = code(check_vault_conservation(100, 151, 50, 0, 0, 0));
        let bound = code(check_vault_conservation(100, 60, 0, 40, 11, 50));
        assert_eq!(exact, u32::from(AmmError::VaultConservationViolated), "the exact clause");
        assert_eq!(bound, u32::from(AmmError::VaultShortfall), "the bound clause");
        assert_ne!(exact, bound, "a seizure must be distinguishable from an arithmetic bug");

        assert_eq!(code(check_vault_conservation(10, 0, 0, 11, 0, 0)), exact);
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
    #[test]
    fn every_claims_bound_in_the_program_is_a_shortfall() {
        use std::path::PathBuf;
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut scanned = Vec::new();
        let mut stack = vec![root.clone()];
        while let Some(dir) = stack.pop() {
            for e in std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
                let p = e.expect("dir entry").path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.extension().is_some_and(|x| x == "rs") {
                    scanned.push(p);
                }
            }
        }
        assert!(scanned.len() >= 20, "the scan found only {} files; it is vacuous", scanned.len());

        let mut found = 0usize;
        for path in &scanned {
            let raw = std::fs::read_to_string(path).expect("read");




            let src: String = raw
                .lines()
                .map(|l| match l.find("//") {
                    Some(i) => &l[..i],
                    None => l,
                })
                .collect::<Vec<_>>()
                .join("\n");


            for stmt in src.split(';') {
                let claims = stmt.contains("hook_claims") || stmt.contains("(claims as u128)");
                let owed = stmt.contains("protocol_fees_owed") || stmt.contains("(owed as u128)");
                if !(claims && owed && stmt.contains("<=")) {
                    continue;
                }
                found += 1;
                let rel = path.strip_prefix(&root).unwrap_or(path).display().to_string();
                assert!(
                    stmt.contains("VaultShortfall"),
                    "{rel}: a claims bound that does not raise VaultShortfall:\n{stmt}"
                );
            }
        }

        assert_eq!(found, 5, "the scan found {found} claims bounds; the program has five");
    }
}
