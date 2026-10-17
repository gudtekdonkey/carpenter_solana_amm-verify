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
use anchor_lang::system_program;
use anchor_spl::associated_token::{self, get_associated_token_address_with_program_id, AssociatedToken};
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};
use floor_types::constants::{LAUNCH_STATE_BONDING, TOKEN_DECIMALS};

use crate::constants::{BUYER_SEED, LAUNCH_SEED, SALE_VAULT_SEED};
use crate::errors::LaunchError;
use crate::events::{Bought, SaleTraderReferralBooked};
use crate::graduate::idx;
use crate::lock::{self, Lock, LOCK_AUTHORITY_SEED};
use crate::sale::{self, create_pda, GraduationCtx};
use crate::state::{BuyerPosition, Launch};
use crate::trader_ref;

use super::quote_custody::{self, QuoteLeg};

#[event_cpi]
#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut, seeds = [LAUNCH_SEED, mint.key().as_ref()], bump)]
    pub launch: AccountLoader<'info, Launch>,
///
    #[account(mut, seeds = [SALE_VAULT_SEED, launch.key().as_ref()], bump)]
    pub sale_vault: UncheckedAccount<'info>,
    #[account(mut, address = launch.load()?.token_vault @ LaunchError::BadConfig)]
    pub token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
///
    #[account(mut, seeds = [BUYER_SEED, launch.key().as_ref(), buyer.key().as_ref()], bump)]
    pub buyer_position: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub buyer_ata: UncheckedAccount<'info>,
    #[account(mut, address = launch.load()?.mint @ LaunchError::BadConfig)]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
///
    #[account(seeds = [LOCK_AUTHORITY_SEED], bump)]
    pub lock_authority: UncheckedAccount<'info>,
    pub ata_program: Program<'info, AssociatedToken>,
    pub token_2022: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

///
#[derive(Clone, Copy)]
struct Settled {
    tokens: u64,
    cost: u64,
    sold: u64,
    raised: u64,
    buyers: u64,
    sold_out: bool,
    launch_bump: u8,
}

///
///
///
///
pub fn handler<'info>(
    ctx: Context<'info, Buy<'info>>,
    lamports_in: u64,
    min_tokens: u64,
    max_graduation_lamports: u64,
) -> Result<()> {

    let (tokens, cost, fee, launch_bump, native, quote_mint, quote_vault, sale_bump) = {
        let l = ctx.accounts.launch.load()?;
        require!(l.state == LAUNCH_STATE_BONDING, LaunchError::SaleOver);
        let remaining = l.sale_tokens.checked_sub(l.sold).ok_or(LaunchError::MathOverflow)?;
        let (tokens, cost, fee) = sale::sale_buy_with_team(lamports_in, l.price_x64.get(), remaining, l.team_bps)
            .map_err(|_| LaunchError::MathOverflow)?;
        (tokens, cost, fee, l.bump, l.native_quote(), l.quote_mint(), l.wsol_vault, l.trader_ref_sale_bump)
    };
    require!(tokens > 0, LaunchError::Nothing);
    require!(tokens >= min_tokens, LaunchError::Slippage);

    let spent = cost.checked_add(fee).ok_or(LaunchError::MathOverflow)?;



    let (referral, ra) = trader_ref::split(ctx.remaining_accounts)?;
    let launch_key = ctx.accounts.launch.key();
    let (slices, sliced) = match &referral {
        Some(r) => trader_ref::slices(fee, r.check(&ctx.accounts.buyer.key(), &quote_mint, &launch_key, sale_bump)?),
        None => ([0u64; 3], 0),
    };




    let graduation_accounts: &[AccountInfo<'info>] = if native {
        system_program::transfer(
            CpiContext::new(
                system_program::ID,
                system_program::Transfer {
                    from: ctx.accounts.buyer.to_account_info(),
                    to: ctx.accounts.sale_vault.to_account_info(),
                },
            ),
            spent - sliced,
        )?;

        if let (Some(r), true) = (&referral, sliced > 0) {
            system_program::transfer(
                CpiContext::new(
                    system_program::ID,
                    system_program::Transfer { from: ctx.accounts.buyer.to_account_info(), to: r.sale.clone() },
                ),
                sliced,
            )?;
        }
        ra
    } else {
        let (buyer_quote_ata, rest) = ra.split_first().ok_or(LaunchError::NotAssociatedTokenAccount)?;
        require!(rest.len() >= crate::graduate::GRADUATION_FIXED_ACCOUNTS, LaunchError::NotAssociatedTokenAccount);
        let leg = QuoteLeg {
            quote_mint: &rest[idx::WSOL_MINT],
            token_program: &rest[idx::SPL_TOKEN],
            quote_vault: &rest[idx::WSOL_VAULT],
            buyer_ata: buyer_quote_ata,
            buyer: &ctx.accounts.buyer.to_account_info(),
            launch: &ctx.accounts.launch.to_account_info(),
            launch_bump,
            base_mint: ctx.accounts.mint.key(),
        };
        leg.check(&quote_mint, &quote_vault)?;
        quote_custody::take(&leg, spent)?;
        if let (Some(r), true) = (&referral, sliced > 0) {
            trader_ref::move_from_quote_vault(
                &rest[idx::SPL_TOKEN],
                &rest[idx::WSOL_VAULT],
                &rest[idx::WSOL_MINT],
                r.sale,
                &ctx.accounts.launch.to_account_info(),
                &ctx.accounts.mint.key(),
                launch_bump,
                sliced,
            )?;
        }
        rest
    };


    ensure_position(&ctx)?;

    let s = settle(&ctx, tokens, cost, fee - sliced, launch_bump)?;


    if let (Some(r), true) = (&referral, sliced > 0) {
        r.credit(slices)?;
        emit_cpi!(SaleTraderReferralBooked {
            launch: launch_key,
            trader: ctx.accounts.buyer.key(),
            s1: slices[0],
            s2: slices[1],
            s3: slices[2],
        });
    }


    deliver(&ctx, &s)?;


    emit_cpi!(Bought {
        launch: ctx.accounts.launch.key(),
        buyer: ctx.accounts.buyer.key(),
        tokens: s.tokens,
        cost: s.cost,
        sold: s.sold,
        raised: s.raised,
        buyers: s.buyers,
    });


    if s.sold_out {
        let before = ctx.accounts.buyer.lamports();
        graduate(&ctx, graduation_accounts)?;
        let after = ctx.accounts.buyer.lamports();
        require!(before.saturating_sub(after) <= max_graduation_lamports, LaunchError::GraduationCostExceeded);
    }


    let bonding = ctx.accounts.launch.load()?.state == LAUNCH_STATE_BONDING;
    lock::assert_state(&ctx.accounts.mint.to_account_info(), &ctx.accounts.lock_authority.key(), bonding)
}

///
///
#[inline(never)]
fn ensure_position(ctx: &Context<Buy>) -> Result<()> {
    let info = ctx.accounts.buyer_position.to_account_info();
    if info.owner == &crate::ID {
        let data = info.try_borrow_data()?;
        require!(
            data.len() == BuyerPosition::LEN && &data[..8] == BuyerPosition::DISCRIMINATOR,
            LaunchError::AccountInUse
        );
        return Ok(());
    }
    let launch_key = ctx.accounts.launch.key();
    let buyer_key = ctx.accounts.buyer.key();
    let bump = [ctx.bumps.buyer_position];
    create_pda(
        &ctx.accounts.buyer.to_account_info(),
        &info,
        BuyerPosition::LEN,
        &crate::ID,
        &[BUYER_SEED, launch_key.as_ref(), buyer_key.as_ref(), &bump],
    )?;
    let mut data = info.try_borrow_mut_data()?;
    data[..8].copy_from_slice(BuyerPosition::DISCRIMINATOR);
    let p: &mut BuyerPosition = bytemuck::from_bytes_mut(&mut data[8..BuyerPosition::LEN]);
    p.bump = ctx.bumps.buyer_position;
    Ok(())
}

#[inline(never)]
fn settle(ctx: &Context<Buy>, tokens: u64, cost: u64, fee: u64, launch_bump: u8) -> Result<Settled> {
    let info = ctx.accounts.buyer_position.to_account_info();
    let was_empty = {
        let mut data = info.try_borrow_mut_data()?;
        let p: &mut BuyerPosition = bytemuck::from_bytes_mut(&mut data[8..BuyerPosition::LEN]);
        let was_empty = p.allocation == 0;
        p.allocation = p.allocation.checked_add(tokens).ok_or(LaunchError::MathOverflow)?;
        p.contributed = p.contributed.checked_add(cost).ok_or(LaunchError::MathOverflow)?;
        was_empty
    };
    let mut l = ctx.accounts.launch.load_mut()?;
    l.sold = l.sold.checked_add(tokens).ok_or(LaunchError::MathOverflow)?;
    l.raised = l.raised.checked_add(cost).ok_or(LaunchError::MathOverflow)?;
    l.team_owed = l.team_owed.checked_add(fee).ok_or(LaunchError::MathOverflow)?;
    if was_empty {
        l.buyers = l.buyers.checked_add(1).ok_or(LaunchError::MathOverflow)?;
    }
    require!(l.sold <= l.sale_tokens, LaunchError::MathOverflow);
    Ok(Settled {
        tokens,
        cost,
        sold: l.sold,
        raised: l.raised,
        buyers: l.buyers,
        sold_out: l.sold == l.sale_tokens,
        launch_bump,
    })
}

///
///
#[inline(never)]
fn deliver(ctx: &Context<Buy>, s: &Settled) -> Result<()> {
    let a = &ctx.accounts;
    let mint_key = a.mint.key();
    let token_2022 = a.token_2022.to_account_info();
    require_keys_eq!(
        a.buyer_ata.key(),
        get_associated_token_address_with_program_id(&a.buyer.key(), &mint_key, &token_2022.key()),
        LaunchError::NotAssociatedTokenAccount
    );

    associated_token::create_idempotent(CpiContext::new(
        a.ata_program.key(),
        associated_token::Create {
            payer: a.buyer.to_account_info(),
            associated_token: a.buyer_ata.to_account_info(),
            authority: a.buyer.to_account_info(),
            mint: a.mint.to_account_info(),
            system_program: a.system_program.to_account_info(),
            token_program: token_2022.clone(),
        },
    ))?;

    let mint = a.mint.to_account_info();
    let lock_authority = a.lock_authority.to_account_info();
    let lock = Lock { token_2022: &token_2022, mint: &mint, lock_authority: &lock_authority, bump: ctx.bumps.lock_authority };
    let launch_bump = [s.launch_bump];
    let launch_seeds: &[&[u8]] = &[LAUNCH_SEED, mint_key.as_ref(), &launch_bump];
    lock::around_transfer(&lock, !s.sold_out, || {
        anchor_spl::token_2022::transfer_checked(
            CpiContext::new_with_signer(
                token_2022.key(),
                anchor_spl::token_2022::TransferChecked {
                    from: a.token_vault.to_account_info(),
                    mint: mint.clone(),
                    to: a.buyer_ata.to_account_info(),
                    authority: a.launch.to_account_info(),
                },
                &[launch_seeds],
            ),
            s.tokens,
            TOKEN_DECIMALS,
        )
    })
}

///
///
#[inline(never)]
fn graduate<'info>(ctx: &Context<'info, Buy<'info>>, graduation_accounts: &[AccountInfo<'info>]) -> Result<()> {
    let a = &ctx.accounts;

    let (launch_bump, lock_bump) = {
        let l = a.launch.load()?;
        (l.bump, l.lock_bump)
    };
    let payer = a.buyer.to_account_info();
    let launch = a.launch.to_account_info();
    let sale_vault = a.sale_vault.to_account_info();
    let token_vault = a.token_vault.to_account_info();
    let mint = a.mint.to_account_info();
    let lock_authority = a.lock_authority.to_account_info();
    let token_2022 = a.token_2022.to_account_info();
    let system_program = a.system_program.to_account_info();
    let event_authority = a.event_authority.to_account_info();
    let program = a.program.to_account_info();
    sale::graduate_hook(GraduationCtx {
        payer: &payer,
        launch: &launch,
        sale_vault: &sale_vault,
        token_vault: &token_vault,
        mint: &mint,
        lock_authority: &lock_authority,
        launch_bump,
        lock_bump,
        token_2022: &token_2022,
        system_program: &system_program,
        event_authority: &event_authority,
        program: &program,
        graduation_accounts,
    })
}
