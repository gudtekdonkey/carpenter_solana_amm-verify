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
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::amm_core::vaults::{admitted_hook, transfer_checked_out};
use crate::constants::PROTOCOL_TREASURY_SEED;
use crate::errors::AmmError;
use crate::events::ProtocolFeesWithdrawn;
use crate::state::{Pool, ProtocolTreasury};

#[event_cpi]
#[derive(Accounts)]
pub struct WithdrawProtocolFees<'info> {
    #[account(constraint = governance.key() == protocol_treasury.load()?.governance @ AmmError::Unauthorized)]
    pub governance: Signer<'info>,
    #[account(seeds = [PROTOCOL_TREASURY_SEED], bump)]
    pub protocol_treasury: AccountLoader<'info, ProtocolTreasury>,
    pub mint: Box<InterfaceAccount<'info, Mint>>,
///
    #[account(mut)]
    pub source: Box<InterfaceAccount<'info, TokenAccount>>,
///
    #[account(mut)]
    pub destination: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Interface<'info, TokenInterface>,
///
///
    pub hook_pool: Option<AccountLoader<'info, Pool>>,
}

///
///
pub fn admitted_for(pool: Option<&Pool>, mint: &Pubkey) -> Result<Option<Pubkey>> {
    let Some(p) = pool else { return Ok(None) };
    let side = if p.mint0 == *mint {
        0
    } else if p.mint1 == *mint {
        1
    } else {
        return err!(AmmError::MintMismatch);
    };
    Ok(admitted_hook(p, side))
}

pub fn handler<'info>(ctx: Context<'info, WithdrawProtocolFees<'info>>, amount: u64) -> Result<()> {
    let a = &ctx.accounts;
    let treasury = a.protocol_treasury.key();
    let mint = a.mint.key();
    require!(amount > 0, AmmError::AmountZero);
    require_keys_eq!(*a.mint.to_account_info().owner, a.token_program.key(), AmmError::TokenProgramMismatch);
    require_keys_eq!(a.source.owner, treasury, AmmError::NotTreasuryAccount);
    require_keys_eq!(a.source.mint, mint, AmmError::MintMismatch);
    require_keys_eq!(a.destination.mint, mint, AmmError::MintMismatch);
    require_keys_neq!(a.destination.owner, treasury, AmmError::WrongDestination);


    require!(amount <= a.source.amount, AmmError::TreasuryBalanceShort);
    let admitted = match &a.hook_pool {
        Some(loader) => admitted_for(Some(&*loader.load()?), &mint)?,
        None => admitted_for(None, &mint)?,
    };

    let bump = [ctx.bumps.protocol_treasury];
    let signer: [&[u8]; 2] = [PROTOCOL_TREASURY_SEED, &bump];


    transfer_checked_out(
        &a.token_program.to_account_info(),
        &a.source.to_account_info(),
        &a.mint.to_account_info(),
        &a.destination.to_account_info(),
        &a.protocol_treasury.to_account_info(),
        &signer,
        amount,
        a.mint.decimals,
        admitted,
        ctx.remaining_accounts,
    )?;

    ctx.accounts.source.reload()?;
    let a = &ctx.accounts;
    emit_cpi!(ProtocolFeesWithdrawn {
        mint,
        source: a.source.key(),
        destination: a.destination.key(),
        governance: a.governance.key(),
        amount,
        remaining: a.source.amount,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

///
///
///
    fn pool_of(mint0: Pubkey, mint1: Pubkey, hook: Pubkey, side: u8) -> Pool {
        Pool { mint0, mint1, base_transfer_hook: hook, base_hook_side: side, ..bytemuck::Zeroable::zeroed() }
    }

///
///
///
    #[test]
    fn a_withdraw_admits_only_what_the_named_pool_admits_on_the_mint_s_side() {
        let (quote, base, hook) = (Pubkey::new_unique(), Pubkey::new_unique(), Pubkey::new_unique());
        let hooked = pool_of(quote, base, hook, 1);
        assert_eq!(admitted_for(Some(&hooked), &base).unwrap(), Some(hook), "the pool's admitted base hook");
        assert_eq!(admitted_for(Some(&hooked), &quote).unwrap(), None, "the quote side admits nothing");
        let plain = pool_of(quote, base, Pubkey::default(), 0);
        assert_eq!(admitted_for(Some(&plain), &base).unwrap(), None, "a pool that admits nothing");
        assert_eq!(admitted_for(None, &base).unwrap(), None, "no pool, nothing admitted");
        let foreign = Pubkey::new_unique();
        match admitted_for(Some(&hooked), &foreign) {
            Err(anchor_lang::error::Error::AnchorError(e)) => assert_eq!(e.error_name, "MintMismatch"),
            other => panic!("a pool that does not hold the mint: {other:?}"),
        }
    }
}
