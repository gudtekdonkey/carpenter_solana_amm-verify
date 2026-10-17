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
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke;
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};
use floor_types::constants::{LAUNCH_STATE_BONDING, TOKEN_DECIMALS};

use crate::constants::{
    holders_disc, BUYER_SEED, FLOOR_HOLDERS_PROGRAM, HOLDER_BOOK_SEED, HOLDER_RECORD_SEED, LAUNCH_SEED, SALE_VAULT_SEED,
};
use crate::errors::LaunchError;
use crate::events::{SaleTraderReferralBooked, SoldBack};
use crate::lock::{self, Lock, LOCK_AUTHORITY_SEED};
use crate::sale;
use crate::state::{BuyerPosition, Launch};
use crate::trader_ref;

use super::quote_custody::{self, QuoteLeg};

#[event_cpi]
#[derive(Accounts)]
pub struct SellBack<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut, seeds = [LAUNCH_SEED, mint.key().as_ref()], bump)]
    pub launch: AccountLoader<'info, Launch>,
///
    #[account(mut, seeds = [SALE_VAULT_SEED, launch.key().as_ref()], bump)]
    pub sale_vault: UncheckedAccount<'info>,
    #[account(mut, seeds = [BUYER_SEED, launch.key().as_ref(), buyer.key().as_ref()], bump)]
    pub buyer_position: AccountLoader<'info, BuyerPosition>,
///
    #[account(mut, token::mint = mint, token::authority = buyer, token::token_program = token_2022)]
    pub buyer_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, address = launch.load()?.token_vault @ LaunchError::BadConfig)]
    pub token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, address = launch.load()?.mint @ LaunchError::BadConfig)]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
///
    #[account(seeds = [LOCK_AUTHORITY_SEED], bump)]
    pub lock_authority: UncheckedAccount<'info>,
    pub token_2022: Program<'info, Token2022>,
}

///
///
pub fn handler<'info>(ctx: Context<'info, SellBack<'info>>, tokens: u64) -> Result<()> {
    run(ctx, tokens, false)
}

///
///
pub fn handler_wsol<'info>(ctx: Context<'info, SellBack<'info>>, tokens: u64) -> Result<()> {
    run(ctx, tokens, true)
}

///
pub const WSOL_OUT_ACCOUNTS: usize = 2;

fn run<'info>(ctx: Context<'info, SellBack<'info>>, tokens: u64, wsol_out: bool) -> Result<()> {
    let launch_key = ctx.accounts.launch.key();
    let mint_key = ctx.accounts.mint.key();


    let (native, quote_mint, quote_vault, floor_pool, sale_bump, price_x64) = {
        let l = ctx.accounts.launch.load()?;
        require!(l.state == LAUNCH_STATE_BONDING, LaunchError::SaleOver);
        (l.native_quote(), l.quote_mint(), l.wsol_vault, l.floor_pool, l.trader_ref_sale_bump, l.price_x64.get())
    };

    require!(!wsol_out || native, LaunchError::WsolPayoutNotNative);


    let (referral, ra) = trader_ref::split(ctx.remaining_accounts)?;
    let tiers = match &referral {
        Some(r) => Some(r.check(&ctx.accounts.buyer.key(), &quote_mint, &launch_key, sale_bump)?),
        None => None,
    };





    let quote_accounts = if !native { 4 } else if wsol_out { WSOL_OUT_ACCOUNTS } else { 0 };
    let hooked = lock::hook_slot(&ctx.accounts.mint.to_account_info())?.is_some();
    if hooked {
        require!(ra.len() == quote_accounts + HOLDER_TAIL, LaunchError::HolderRecordMismatch);
    } else if wsol_out {
        require!(ra.len() == WSOL_OUT_ACCOUNTS, LaunchError::BadWsolDestination);
    } else {
        require!(ra.len() == quote_accounts, LaunchError::NotAssociatedTokenAccount);
    }
    if wsol_out {
        check_wsol_destination(&ra[0], &ra[1], &ctx.accounts.buyer.key())?;
    }
    require!(tokens > 0, LaunchError::Nothing);
    require_keys_eq!(
        ctx.accounts.buyer_ata.key(),
        get_associated_token_address_with_program_id(&ctx.accounts.buyer.key(), &mint_key, &ctx.accounts.token_2022.key()),
        LaunchError::NotAssociatedTokenAccount
    );

    let holder_tail = if hooked {
        let tail = &ra[quote_accounts..];
        check_holder_tail(tail, &mint_key, &floor_pool, &ctx.accounts.buyer_ata.key())?;
        Some(tail)
    } else {
        None
    };
    let delegate_path = cpi_guard_locked(&ctx.accounts.buyer_ata.to_account_info())?;
    if delegate_path {
        require!(
            Option::<Pubkey>::from(ctx.accounts.buyer_ata.delegate) == Some(launch_key),
            LaunchError::DelegateMissing
        );
        require!(ctx.accounts.buyer_ata.delegated_amount >= tokens, LaunchError::DelegateMissing);
    }


    let (payout, fee, slices, sliced, sold, raised, launch_bump) = {
        let mut p = ctx.accounts.buyer_position.load_mut()?;
        require!(tokens <= p.allocation, LaunchError::InsufficientAllocation);

        let (payout, contributed, allocation) =
            sale::sell_back_settle(p.contributed, p.allocation, tokens, price_x64).map_err(|_| LaunchError::MathOverflow)?;
        (p.allocation, p.contributed) = (allocation, contributed);
        drop(p);


        let mut l = ctx.accounts.launch.load_mut()?;
        l.sold = l.sold.checked_sub(tokens).ok_or(LaunchError::MathOverflow)?;
        l.raised = l.raised.checked_sub(payout).ok_or(LaunchError::MathOverflow)?;

        let fee = sale::team_fee_of(payout, l.team_bps).map_err(|_| LaunchError::MathOverflow)?;

        let (slices, sliced) = match tiers {
            Some(t) => trader_ref::slices(fee, t),
            None => ([0u64; 3], 0),
        };

        let kept = fee - sliced;
        l.team_owed = l.team_owed.checked_add(kept).ok_or(LaunchError::MathOverflow)?;
        (payout, fee, slices, sliced, l.sold, l.raised, l.bump)
    };
    let received = payout.checked_sub(fee).ok_or(LaunchError::MathOverflow)?;


    {
        let a = &ctx.accounts;
        let token_2022 = a.token_2022.to_account_info();
        let mint = a.mint.to_account_info();
        let lock_authority = a.lock_authority.to_account_info();
        let lock = Lock { token_2022: &token_2022, mint: &mint, lock_authority: &lock_authority, bump: ctx.bumps.lock_authority };
        let launch_bump = [launch_bump];
        let launch_seeds: &[&[u8]] = &[LAUNCH_SEED, mint_key.as_ref(), &launch_bump];
        let authority = if delegate_path { a.launch.to_account_info() } else { a.buyer.to_account_info() };
        let launch_signer = [launch_seeds];
        let signer: &[&[&[u8]]] = if delegate_path { &launch_signer } else { &[] };
        lock::around_transfer(&lock, true, || {
            anchor_spl::token_2022::transfer_checked(
                CpiContext::new_with_signer(
                    token_2022.key(),
                    anchor_spl::token_2022::TransferChecked {
                        from: a.buyer_ata.to_account_info(),
                        mint: mint.clone(),
                        to: a.token_vault.to_account_info(),
                        authority,
                    },
                    signer,
                ),
                tokens,
                TOKEN_DECIMALS,
            )
        })?;
    }




    if let Some(tail) = holder_tail {
        sync_holder_record(tail, &ctx.accounts.mint.to_account_info(), &ctx.accounts.buyer_ata.to_account_info())?;
    }



    if let (Some(r), true) = (&referral, sliced > 0) {
        r.credit(slices)?;
    }




    if native {
        let vault = ctx.accounts.sale_vault.to_account_info();
        let buyer = ctx.accounts.buyer.to_account_info();
        let reserve = Rent::get()?.minimum_balance(vault.data_len());
        let out = received.checked_add(sliced).ok_or(LaunchError::MathOverflow)?;
        let left = vault.lamports().checked_sub(out).ok_or(LaunchError::EscrowShort)?;
        require!(left >= reserve, LaunchError::EscrowShort);
        **vault.try_borrow_mut_lamports()? = left;


        let to = if wsol_out { ra[0].clone() } else { buyer };
        let credited = to.lamports().checked_add(received).ok_or(LaunchError::MathOverflow)?;
        **to.try_borrow_mut_lamports()? = credited;


        if let (Some(r), true) = (&referral, sliced > 0) {
            let sale_lamports = r.sale.lamports().checked_add(sliced).ok_or(LaunchError::MathOverflow)?;
            **r.sale.try_borrow_mut_lamports()? = sale_lamports;
        }
        if wsol_out {







            let mut carry = vec![vault.clone(), ra[0].clone()];
            if let (Some(r), true) = (&referral, sliced > 0) {
                carry.push(r.sale.clone());
            }
            let ev = {
                let l = ctx.accounts.launch.load()?;
                SoldBack { launch: launch_key, buyer: ctx.accounts.buyer.key(), tokens, payout, sold: l.sold, raised: l.raised }
            };
            emit_carrying(&ctx.accounts.event_authority.to_account_info(), &ctx.accounts.program.to_account_info(), &ev, &carry)?;
            let ix = anchor_spl::token::spl_token::instruction::sync_native(&anchor_spl::token::ID, &ra[0].key())?;
            invoke(&ix, &[ra[0].clone(), ra[1].clone()])?;
        }
    } else {
        let ra = &ra[..4];
        let leg = QuoteLeg {
            quote_mint: &ra[0],
            token_program: &ra[1],
            quote_vault: &ra[2],
            buyer_ata: &ra[3],
            buyer: &ctx.accounts.buyer.to_account_info(),
            launch: &ctx.accounts.launch.to_account_info(),
            launch_bump,
            base_mint: mint_key,
        };
        leg.check(&quote_mint, &quote_vault)?;
        quote_custody::pay(&leg, received)?;
        if let (Some(r), true) = (&referral, sliced > 0) {
            trader_ref::move_from_quote_vault(
                &ra[1],
                &ra[2],
                &ra[0],
                r.sale,
                &ctx.accounts.launch.to_account_info(),
                &mint_key,
                launch_bump,
                sliced,
            )?;
        }
    }


    if !wsol_out {
        emit_cpi!(SoldBack { launch: launch_key, buyer: ctx.accounts.buyer.key(), tokens, payout, sold, raised });
    }
    if sliced > 0 {
        emit_cpi!(SaleTraderReferralBooked {
            launch: launch_key,
            trader: ctx.accounts.buyer.key(),
            s1: slices[0],
            s2: slices[1],
            s3: slices[2],
        });
    }


    lock::assert_state(&ctx.accounts.mint.to_account_info(), &ctx.accounts.lock_authority.key(), true)
}

///
///
///
///
fn check_wsol_destination(wsol: &AccountInfo<'_>, token_program: &AccountInfo<'_>, seller: &Pubkey) -> Result<()> {
    use anchor_spl::token::spl_token::native_mint;
    require_keys_eq!(token_program.key(), anchor_spl::token::ID, LaunchError::BadWsolDestination);
    require_keys_eq!(*wsol.owner, anchor_spl::token::ID, LaunchError::BadWsolDestination);
    require!(wsol.is_writable, LaunchError::BadWsolDestination);
    let d = wsol.try_borrow_data()?;
    require!(d.len() == 165, LaunchError::BadWsolDestination);
    require!(d[0..32] == native_mint::ID.to_bytes() && d[32..64] == seller.to_bytes(), LaunchError::BadWsolDestination);
    require!(d[108] == 1, LaunchError::BadWsolDestination);
    require!(d[109..113] == [1, 0, 0, 0], LaunchError::BadWsolDestination);
    Ok(())
}

///
///
///
fn emit_carrying<'info>(
    event_authority: &AccountInfo<'info>,
    program: &AccountInfo<'info>,
    event: &SoldBack,
    carry: &[AccountInfo<'info>],
) -> Result<()> {
    let mut data = anchor_lang::event::EVENT_IX_TAG_LE.to_vec();
    data.extend(anchor_lang::Event::data(event));
    let mut accounts = vec![AccountMeta::new_readonly(event_authority.key(), true)];
    let mut infos = vec![event_authority.clone()];
    for a in carry {
        accounts.push(AccountMeta::new(a.key(), false));
        infos.push(a.clone());
    }
    infos.push(program.clone());
    let ix = Instruction { program_id: crate::ID, accounts, data };
    anchor_lang::solana_program::program::invoke_signed(&ix, &infos, &[&[b"__event_authority", &[crate::EVENT_AUTHORITY_AND_BUMP.1]]])?;
    Ok(())
}

///
pub const HOLDER_TAIL: usize = 4;

///
///
///
///
///
///
fn check_holder_tail(tail: &[AccountInfo<'_>], mint: &Pubkey, floor_pool: &Pubkey, buyer_ata: &Pubkey) -> Result<()> {
    require!(tail.len() == HOLDER_TAIL, LaunchError::HolderRecordMismatch);
    let book = Pubkey::find_program_address(&[HOLDER_BOOK_SEED, mint.as_ref()], &FLOOR_HOLDERS_PROGRAM).0;
    let record =
        Pubkey::find_program_address(&[HOLDER_RECORD_SEED, mint.as_ref(), buyer_ata.as_ref()], &FLOOR_HOLDERS_PROGRAM).0;
    require_keys_eq!(tail[0].key(), FLOOR_HOLDERS_PROGRAM, LaunchError::HolderRecordMismatch);
    require_keys_eq!(tail[1].key(), book, LaunchError::HolderRecordMismatch);
    require_keys_eq!(tail[2].key(), *floor_pool, LaunchError::HolderRecordMismatch);
    require_keys_eq!(tail[3].key(), record, LaunchError::HolderRecordMismatch);
    require!(tail[1].is_writable && tail[3].is_writable, LaunchError::HolderRecordMismatch);
    Ok(())
}

///
///
///
///
fn sync_holder_record<'info>(
    tail: &[AccountInfo<'info>],
    mint: &AccountInfo<'info>,
    buyer_ata: &AccountInfo<'info>,
) -> Result<()> {
    let record = &tail[3];
    if *record.owner != FLOOR_HOLDERS_PROGRAM {
        return Ok(());
    }
    let ix = Instruction {
        program_id: FLOOR_HOLDERS_PROGRAM,
        accounts: vec![
            AccountMeta::new_readonly(mint.key(), false),
            AccountMeta::new(tail[1].key(), false),
            AccountMeta::new_readonly(tail[2].key(), false),
            AccountMeta::new_readonly(buyer_ata.key(), false),
            AccountMeta::new(record.key(), false),
        ],
        data: holders_disc::SYNC.to_vec(),
    };
    invoke(&ix, &[mint.clone(), tail[1].clone(), tail[2].clone(), buyer_ata.clone(), record.clone(), tail[0].clone()])?;
    Ok(())
}

///
///
///
fn cpi_guard_locked(ata: &AccountInfo<'_>) -> Result<bool> {
    quote_custody::cpi_guard_locked(ata)
}
