//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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
use floor_math::holders;
use floor_types::holders::HolderRecord;
use floor_types::layout::Layout;

use crate::constants::{HOLDER_RECORD_SEED, RECORD_VERSION};
use crate::errors::HoldersError;
use crate::events::RecordOpened;
use crate::{create, state, token};

#[event_cpi]
#[derive(Accounts)]
pub struct OpenRecord<'info> {
///
    #[account(mut)]
    pub payer: Signer<'info>,
///
    pub owner: UncheckedAccount<'info>,
///
    pub mint: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub holder_book: UncheckedAccount<'info>,
///
    pub floor_pool: UncheckedAccount<'info>,
///
    pub token_account: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub record: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

///
enum Outcome {
    Opened { shares: u64 },
    Synced,
}

fn open_or_touch(ctx: &Context<OpenRecord>, touch: bool) -> Result<Outcome> {
    let a = &ctx.accounts;
    let mint = a.mint.key();
    let owner = a.owner.key();
    let token_account = a.token_account.key();

    require!(token::is_on_curve(&owner), HoldersError::OwnerOffCurve);


    require!(!token::is_dead_owner(&owner), HoldersError::OwnerOffCurve);
    require_keys_eq!(token_account, token::token_2022_ata(&owner, &mint), HoldersError::NotAssociatedTokenAccount);
    let ta_info = a.token_account.to_account_info();
    let (ta_mint, ta_owner, balance) =
        token::read_token_2022_account(&ta_info).ok_or(HoldersError::NotAssociatedTokenAccount)?;
    require_keys_eq!(ta_mint, mint, HoldersError::NotAssociatedTokenAccount);
    require_keys_eq!(ta_owner, owner, HoldersError::NotAssociatedTokenAccount);

    let book_info = a.holder_book.to_account_info();
    let mut book = state::load_book(&book_info, &mint)?;
    let pool = state::load_book_pool(&a.floor_pool.to_account_info(), &book)?;

    let (want_record, record_bump) =
        Pubkey::find_program_address(&[HOLDER_RECORD_SEED, mint.as_ref(), token_account.as_ref()], &crate::ID);
    require_keys_eq!(a.record.key(), want_record, anchor_lang::error::ErrorCode::ConstraintSeeds);
    let record_info = a.record.to_account_info();

    let mut t = state::tally_book(&book);

    holders::catch_up_at(&mut t, pool.lifetime, Clock::get()?.slot);

    if !create::is_absent(&record_info) {

        require!(touch, HoldersError::RecordExists);
        let mut rec = state::load_record(&record_info, &mint, &token_account)?;
        let mut r = state::tally_record(&rec);
        holders::settle(&mut t, &mut r, balance, balance);
        state::store_tally_record(&mut rec, &r);
        state::write_owned(&record_info, &rec)?;
        state::store_tally_book(&mut book, &t);
        state::write_owned(&book_info, &book)?;
        return Ok(Outcome::Synced);
    }

    let bump_bytes = [record_bump];
    let seeds: &[&[u8]] = &[HOLDER_RECORD_SEED, mint.as_ref(), token_account.as_ref(), &bump_bytes];
    create::create_pda(&a.payer.to_account_info(), &record_info, HolderRecord::ACCOUNT_LEN, seeds, HoldersError::RecordExists.into())?;

    let r = holders::open(&mut t, balance);
    let mut rec: HolderRecord = bytemuck::Zeroable::zeroed();
    rec.version = RECORD_VERSION;
    rec.bump = record_bump;
    rec.mint = mint.to_bytes();
    rec.token_account = token_account.to_bytes();
    rec.owner = owner.to_bytes();
    rec.opened_slot = Clock::get()?.slot;
    state::store_tally_record(&mut rec, &r);
    state::init_owned(&record_info, &rec)?;

    state::store_tally_book(&mut book, &t);
    book.records = book.records.saturating_add(1);
    state::write_owned(&book_info, &book)?;
    Ok(Outcome::Opened { shares: r.shares })
}

fn emit_opened(ctx: &Context<OpenRecord>, shares: u64) -> Result<()> {
    let a = &ctx.accounts;
    emit_cpi!(RecordOpened {
        mint: a.mint.key(),
        record: a.record.key(),
        token_account: a.token_account.key(),
        owner: a.owner.key(),
        payer: a.payer.key(),
        shares,
    });
    Ok(())
}

pub fn open_handler(ctx: Context<OpenRecord>) -> Result<()> {
    match open_or_touch(&ctx, false)? {
        Outcome::Opened { shares } => emit_opened(&ctx, shares),
        Outcome::Synced => Ok(()),
    }
}

pub fn touch_handler(ctx: Context<OpenRecord>) -> Result<()> {
    match open_or_touch(&ctx, true)? {
        Outcome::Opened { shares } => emit_opened(&ctx, shares),
        Outcome::Synced => Ok(()),
    }
}
