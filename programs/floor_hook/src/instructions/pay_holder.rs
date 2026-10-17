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
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::{CARPENTER_AMM_PROGRAM, EVENT_AUTHORITY_SEED, HOOK_AUTHORITY_SEED};
use crate::errors::HookError;
use crate::events::HolderPaid;
use crate::mech::amm_cpi::{self, AmmCpi};
use crate::mech::positions::{add, sub};
use crate::mech::ratchet;
use crate::state::FloorPool;

///
///
///
///
#[event_cpi]
#[derive(Accounts)]
pub struct PayHolder<'info> {
///
///
///
///
    pub holders_authority: Signer<'info>,
    #[account(mut)]
    pub floor_pool: AccountLoader<'info, FloorPool>,
///
    #[account(mut, address = floor_pool.load()?.amm_pool @ HookError::WrongPool)]
    pub pool: UncheckedAccount<'info>,
    #[account(mut)]
    pub vault0: Box<InterfaceAccount<'info, TokenAccount>>,
///
///
    #[account(mut)]
    pub destination: Box<InterfaceAccount<'info, TokenAccount>>,
///
    #[account(address = floor_pool.load()?.quote_mint @ HookError::WrongPool)]
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
    pub spl_token: Interface<'info, TokenInterface>,
///
    #[account(address = CARPENTER_AMM_PROGRAM)]
    pub carpenter_amm: UncheckedAccount<'info>,
///
    #[account(seeds = [HOOK_AUTHORITY_SEED], bump)]
    pub hook_authority: UncheckedAccount<'info>,
///
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = CARPENTER_AMM_PROGRAM)]
    pub amm_event_authority: UncheckedAccount<'info>,
}

///
///
pub fn book_pay(s: &mut FloorPool, amount: u64) -> Result<()> {
    require!(amount > 0, HookError::NothingOwed);
    require!(amount <= s.holder_owed, HookError::HolderPayExceedsOwed);
    s.holder_owed = sub(s.holder_owed, amount)?;
    s.holder_paid_eth = add(s.holder_paid_eth, amount)?;
    Ok(())
}

pub fn handler<'info>(ctx: Context<'info, PayHolder<'info>>, amount: u64) -> Result<()> {
    require_keys_eq!(ctx.accounts.holders_authority.key(), crate::constants::holders_authority(), HookError::NotHoldersAuthority);
    payout(ctx, amount)
}

pub fn authorized<'info>(ctx: Context<'info, PayHolder<'info>>, amount: u64) -> Result<()> {
    use anchor_lang::solana_program::instruction::get_stack_height;
    require!(get_stack_height() == 1, HookError::NotHoldersAuthority);
    let sysvar = ctx.remaining_accounts.iter().find(|a| *a.key == pubkey!("Sysvar1nstructions1111111111111111111111111"))
        .ok_or(HookError::NotHoldersAuthority)?;
    let data = sysvar.try_borrow_data()?;
    let index = floor_types::payout::current_index(&data).ok_or(HookError::NotHoldersAuthority)?;
    let prior = index.checked_sub(1).ok_or(HookError::NotHoldersAuthority)?;
    let auth = floor_types::payout::instruction(&data, prior).ok_or(HookError::NotHoldersAuthority)?;
    require!(floor_types::payout::adjacent(&data, amount, true), HookError::NotHoldersAuthority);
    require_keys_eq!(ctx.accounts.destination.owner, ctx.accounts.holders_authority.key(), HookError::WrongDestination);
    require!(auth.key(1) == Some(ctx.accounts.floor_pool.load()?.base_mint.as_ref()), HookError::WrongPool);
    drop(data);
    payout(ctx, amount)
}

fn payout<'info>(ctx: Context<'info, PayHolder<'info>>, amount: u64) -> Result<()> {
    let mut s: FloorPool = *ctx.accounts.floor_pool.load()?;
    require!(s.pool_bound == 1, HookError::NotBound);


    book_pay(&mut s, amount)?;

    let a = &ctx.accounts;
    let p = amm_cpi::read_pool(&a.pool.to_account_info())?;
    amm_cpi::require_pool_side(&p, 0, &a.vault0.key(), &a.quote_mint.key(), &a.spl_token.key())?;


    require_keys_eq!(a.destination.mint, s.quote_mint, HookError::WrongDestination);

    let pool = s.amm_pool;
    let destination = a.destination.key();
    *ctx.accounts.floor_pool.load_mut()? = s;

    let a = &ctx.accounts;
    let cpi = AmmCpi {
        carpenter_amm: a.carpenter_amm.to_account_info(),
        amm_event_authority: a.amm_event_authority.to_account_info(),
        hook_authority: a.hook_authority.to_account_info(),
        hook_authority_bump: ctx.bumps.hook_authority,
        hook_program: a.program.to_account_info(),
        floor_pool: a.floor_pool.to_account_info(),
        pool: a.pool.to_account_info(),
    };
    let w = amm_cpi::withdraw_hooked(
        &cpi,
        &a.vault0.to_account_info(),
        &a.destination.to_account_info(),
        &a.quote_mint.to_account_info(),
        &a.spl_token.to_account_info(),
        0,
        amount,
        ctx.remaining_accounts,
    )?;



    require!(
        ratchet::claims_cover_liabilities(&s, w.hook_claims0, w.hook_claims1),
        HookError::InvariantViolated
    );

    emit_cpi!(HolderPaid { pool, destination, amount });
    Ok(())
}

#[cfg(test)]
mod tests {
//!
//!
    use super::*;
    use crate::constants::holders_authority;

    fn pool(owed: u64, paid: u64) -> FloorPool {
        let mut s: FloorPool = bytemuck::Zeroable::zeroed();
        s.holder_owed = owed;
        s.holder_paid_eth = paid;
        s
    }

///
///
///
///
///
///
    #[test]
    fn a_pay_moves_owed_to_paid_one_for_one_and_never_past_what_is_owed() {
        let mut s = pool(1_000, 7);
        book_pay(&mut s, 400).unwrap();
        assert_eq!((s.holder_owed, s.holder_paid_eth), (600, 407));

        assert_eq!(s.holder_owed + s.holder_paid_eth, 1_007);


        book_pay(&mut s, 600).unwrap();
        assert_eq!((s.holder_owed, s.holder_paid_eth), (0, 1_007));


        let mut s = pool(10, 0);
        assert_eq!(book_pay(&mut s, 11).unwrap_err(), error!(HookError::HolderPayExceedsOwed));
        assert_eq!((s.holder_owed, s.holder_paid_eth), (10, 0), "a refusal writes nothing");


        assert_eq!(book_pay(&mut s, 0).unwrap_err(), error!(HookError::NothingOwed));

        let mut s = pool(0, 5);
        assert_eq!(book_pay(&mut s, 1).unwrap_err(), error!(HookError::HolderPayExceedsOwed));
    }

///
///
///
    #[test]
    fn the_only_signer_is_floor_holders_holders_authority_pda() {
        let (want, _) = Pubkey::find_program_address(
            &[b"holders_authority"],
            &Pubkey::new_from_array(floor_types::ids::FLOOR_HOLDERS_ID),
        );
        assert_eq!(holders_authority(), want);
        assert_ne!(holders_authority(), Pubkey::find_program_address(&[b"holders_authority"], &crate::ID).0);
        assert_ne!(
            holders_authority(),
            Pubkey::find_program_address(&[b"holders_authority"], &crate::constants::FLOOR_LAUNCH_PROGRAM).0
        );
        assert_ne!(holders_authority(), crate::constants::FLOOR_HOLDERS_PROGRAM);
    }
}
