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
use floor_types::constants::LAUNCH_STATE_BONDING;
use floor_types::disc;

use crate::constants::{FLOOR_HOOK_PROGRAM, HOOK_CONFIG_SEED, LAUNCH_AUTHORITY_SEED, SALE_VAULT_SEED};
use crate::errors::LaunchError;
use crate::events::TeamBuySoldBack;
use crate::sale;
use crate::state::Launch;

use super::quote_custody::{self, QuoteLeg};

#[event_cpi]
#[derive(Accounts)]
pub struct TeamBuySellBack<'info> {
///
    #[account(mut, address = launch.load()?.creator @ LaunchError::BadConfig)]
    pub creator: Signer<'info>,
///
    #[account(mut)]
    pub launch: AccountLoader<'info, Launch>,
///
    #[account(mut, seeds = [SALE_VAULT_SEED, launch.key().as_ref()], bump)]
    pub sale_vault: UncheckedAccount<'info>,
///
    #[account(seeds = [LAUNCH_AUTHORITY_SEED], bump)]
    pub launch_authority: UncheckedAccount<'info>,
///
    #[account(seeds = [HOOK_CONFIG_SEED], bump, seeds::program = FLOOR_HOOK_PROGRAM)]
    pub hook_config: UncheckedAccount<'info>,
///
    #[account(mut, address = launch.load()?.floor_pool @ LaunchError::BadConfig, owner = FLOOR_HOOK_PROGRAM @ LaunchError::BadConfig)]
    pub floor_pool: UncheckedAccount<'info>,
///
    #[account(address = FLOOR_HOOK_PROGRAM)]
    pub floor_hook: UncheckedAccount<'info>,
}

pub fn handler<'info>(ctx: Context<'info, TeamBuySellBack<'info>>) -> Result<()> {
    let launch_key = ctx.accounts.launch.key();
    let (native, quote_mint, quote_vault, mint, tokens, price, bps) = {
        let l = ctx.accounts.launch.load()?;
        require!(l.state == LAUNCH_STATE_BONDING, LaunchError::SaleOver);
        let tokens = l.team_step_tokens.checked_add(l.team_wallet_tokens).ok_or(LaunchError::MathOverflow)?;
        (l.native_quote(), l.quote_mint(), l.wsol_vault, l.mint, tokens, l.price_x64.get(), l.team_bps)
    };
    require!(tokens > 0, LaunchError::Nothing);

    require!(ctx.remaining_accounts.len() == if native { 0 } else { 4 }, LaunchError::NotAssociatedTokenAccount);


    let cost = sale::team_buy_cost(tokens, price).ok_or(LaunchError::MathOverflow)?;
    let fee = sale::team_fee_of(cost, bps).map_err(|_| LaunchError::MathOverflow)?;
    let received = cost.checked_sub(fee).ok_or(LaunchError::MathOverflow)?;
    let (sold, raised, bump) = {
        let mut l = ctx.accounts.launch.load_mut()?;
        l.sold = l.sold.checked_sub(tokens).ok_or(LaunchError::MathOverflow)?;
        l.raised = l.raised.checked_sub(cost).ok_or(LaunchError::MathOverflow)?;
        l.team_owed = l.team_owed.checked_add(fee).ok_or(LaunchError::MathOverflow)?;
        l.team_step_tokens = 0;
        l.team_wallet_tokens = 0;
        l.team_lock_days = 0;
        l.team_vest_days = [0; 4];


        (l.sold, l.raised, l.bump)
    };


    clear_team_buy_books(&ctx)?;


    if native {
        let vault = ctx.accounts.sale_vault.to_account_info();
        let creator = ctx.accounts.creator.to_account_info();
        let reserve = Rent::get()?.minimum_balance(vault.data_len());
        let left = vault.lamports().checked_sub(received).ok_or(LaunchError::EscrowShort)?;
        require!(left >= reserve, LaunchError::EscrowShort);
        **vault.try_borrow_mut_lamports()? = left;
        let credited = creator.lamports().checked_add(received).ok_or(LaunchError::MathOverflow)?;
        **creator.try_borrow_mut_lamports()? = credited;
    } else {
        let ra = &ctx.remaining_accounts[..4];
        let leg = QuoteLeg {
            quote_mint: &ra[0],
            token_program: &ra[1],
            quote_vault: &ra[2],
            buyer_ata: &ra[3],
            buyer: &ctx.accounts.creator.to_account_info(),
            launch: &ctx.accounts.launch.to_account_info(),
            launch_bump: bump,
            base_mint: mint,
        };
        leg.check(&quote_mint, &quote_vault)?;
        quote_custody::pay(&leg, received)?;
    }


    emit_cpi!(TeamBuySoldBack { launch: launch_key, creator: ctx.accounts.creator.key(), tokens, cost, fee, sold, raised });
    Ok(())
}

///
///
fn clear_team_buy_books(ctx: &Context<TeamBuySellBack>) -> Result<()> {
    let a = &ctx.accounts;
    let ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: FLOOR_HOOK_PROGRAM,
        accounts: vec![
            AccountMeta::new_readonly(a.launch_authority.key(), true),
            AccountMeta::new_readonly(a.hook_config.key(), false),
            AccountMeta::new(a.floor_pool.key(), false),
        ],
        data: disc::hook::CLEAR_TEAM_BUY_BOOKS.to_vec(),
    };
    let infos = [
        a.launch_authority.to_account_info(),
        a.hook_config.to_account_info(),
        a.floor_pool.to_account_info(),
        a.floor_hook.to_account_info(),
    ];
    let bump = [ctx.bumps.launch_authority];
    anchor_lang::solana_program::program::invoke_signed(&ix, &infos, &[&[LAUNCH_AUTHORITY_SEED, &bump]])?;
    Ok(())
}

#[cfg(test)]
mod tests {
///
///
///
///
///
    #[test]
    fn mr_s_the_sell_back_zeroes_the_team_buy_and_leaves_the_slots() {
        let src = include_str!("team_buy_sell_back.rs");
        let code: String = src
            .split("#[cfg(test)]")
            .next()
            .unwrap()
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        for z in ["l.team_step_tokens = 0;", "l.team_wallet_tokens = 0;", "l.team_lock_days = 0;", "l.team_vest_days = [0; 4];"] {
            assert!(code.contains(z), "the sell-back must write `{z}`");
        }
        assert!(code.contains("clear_team_buy_books(&ctx)?;"), "the hook's owner books are cleared");
        for w in ["PayeeSlots", "resolve_stair_payee", "team_cliffs_paid_by_slot"] {
            assert!(!code.contains(w), "the sell-back touches no payee slot (`{w}`)");
        }
    }
}
