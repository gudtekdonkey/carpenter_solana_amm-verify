//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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
use floor_types::holders::HolderBook;
use floor_types::layout::Layout;

use crate::alt::AltCpi;
use crate::constants::{
    ALT_PROGRAM, AMM_POOL_SEED, AMM_VAULT_SEED, BOOK_VERSION, CARPENTER_AMM_PROGRAM, CLAIMS_ADDRS_HOOKED,
    CLAIMS_ADDRS_REFERRED, CLAIMS_ALT_AUTHORITY_SEED, CLAIMS_REGISTRY_SEED, CLAIMS_TABLE_MAX, EXTRA_ACCOUNT_METAS_SEED,
    FLOOR_HOOK_PROGRAM, FLOOR_POOL_SEED, HOLDER_BOOK_SEED,
};
use crate::errors::HoldersError;
use crate::events::{ClaimsTableExtended, LaunchHooked};
use crate::{create, launch_lock, metas, state};

#[event_cpi]
#[derive(Accounts)]
pub struct InitLaunch<'info> {
///
    pub lock_authority: Signer<'info>,
///
    #[account(mut)]
    pub payer: Signer<'info>,
///
///
    pub mint: UncheckedAccount<'info>,
///
    pub quote_mint: UncheckedAccount<'info>,
///
    pub floor_pool: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub holder_book: Option<UncheckedAccount<'info>>,
///
    #[account(mut)]
    pub extra_account_metas: Option<UncheckedAccount<'info>>,
///
    #[account(mut, seeds = [CLAIMS_REGISTRY_SEED], bump)]
    pub claims_registry: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub claims_table: UncheckedAccount<'info>,
///
    #[account(seeds = [CLAIMS_ALT_AUTHORITY_SEED], bump)]
    pub claims_alt_authority: UncheckedAccount<'info>,
///
    #[account(address = ALT_PROGRAM)]
    pub alt_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

///
pub fn derive_launch_pools(quote_mint: &Pubkey, mint: &Pubkey) -> (Pubkey, Pubkey, Pubkey) {
    let (fee, spacing) = floor_types::seeds::pool_seed_parts(
        floor_types::constants::DYNAMIC_FEE_FLAG,
        floor_types::constants::FLOOR_TICK_SPACING,
    );
    let amm_pool = Pubkey::find_program_address(
        &[AMM_POOL_SEED, quote_mint.as_ref(), mint.as_ref(), &fee, &spacing, FLOOR_HOOK_PROGRAM.as_ref()],
        &CARPENTER_AMM_PROGRAM,
    )
    .0;
    let floor_pool = Pubkey::find_program_address(&[FLOOR_POOL_SEED, amm_pool.as_ref()], &FLOOR_HOOK_PROGRAM).0;
    let vault0 = Pubkey::find_program_address(&[AMM_VAULT_SEED, amm_pool.as_ref(), quote_mint.as_ref()], &CARPENTER_AMM_PROGRAM).0;
    (amm_pool, floor_pool, vault0)
}

pub fn handler(ctx: Context<InitLaunch>, hooked: bool) -> Result<()> {
    let a = &ctx.accounts;
    require_keys_eq!(a.lock_authority.key(), launch_lock::launch_lock_authority(), HoldersError::NotLaunchAuthority);

    let mint = a.mint.key();
    let quote_mint = a.quote_mint.key();
    let (amm_pool, floor_pool, vault0) = derive_launch_pools(&quote_mint, &mint);
    require_keys_eq!(a.floor_pool.key(), floor_pool, HoldersError::WrongPool);
    let pool = state::read_floor_pool(&a.floor_pool.to_account_info()).ok_or(HoldersError::WrongPool)?;
    require_keys_eq!(pool.base_mint, mint, HoldersError::WrongPool);
    require_keys_eq!(pool.quote_mint, quote_mint, HoldersError::WrongPool);
    require_keys_eq!(pool.amm_pool, amm_pool, HoldersError::WrongPool);

    let payer = a.payer.to_account_info();
    let mut book_key = Pubkey::default();
    if hooked {
        let book_info = a.holder_book.as_ref().ok_or(anchor_lang::error::ErrorCode::AccountNotEnoughKeys)?.to_account_info();
        let metas_info =
            a.extra_account_metas.as_ref().ok_or(anchor_lang::error::ErrorCode::AccountNotEnoughKeys)?.to_account_info();

        let (want_book, book_bump) = Pubkey::find_program_address(&[HOLDER_BOOK_SEED, mint.as_ref()], &crate::ID);
        require_keys_eq!(book_info.key(), want_book, anchor_lang::error::ErrorCode::ConstraintSeeds);
        let (want_metas, metas_bump) = Pubkey::find_program_address(&[EXTRA_ACCOUNT_METAS_SEED, mint.as_ref()], &crate::ID);
        require_keys_eq!(metas_info.key(), want_metas, anchor_lang::error::ErrorCode::ConstraintSeeds);

        let book_bump_bytes = [book_bump];
        let book_seeds: &[&[u8]] = &[HOLDER_BOOK_SEED, mint.as_ref(), &book_bump_bytes];
        create::create_pda(
            &payer,
            &book_info,
            HolderBook::ACCOUNT_LEN,
            book_seeds,
            anchor_lang::error::ErrorCode::AccountDiscriminatorAlreadySet.into(),
        )?;
        let metas_bump_bytes = [metas_bump];
        let metas_seeds: &[&[u8]] = &[EXTRA_ACCOUNT_METAS_SEED, mint.as_ref(), &metas_bump_bytes];
        create::create_pda(
            &payer,
            &metas_info,
            metas::LIST_LEN,
            metas_seeds,
            anchor_lang::error::ErrorCode::AccountDiscriminatorAlreadySet.into(),
        )?;

        let mut book: HolderBook = bytemuck::Zeroable::zeroed();
        book.version = BOOK_VERSION;
        book.bump = book_bump;
        book.metas_bump = metas_bump;
        book.mint = mint.to_bytes();
        book.floor_pool = floor_pool.to_bytes();
        book.quote_mint = quote_mint.to_bytes();



        book.indexed = pool.lifetime;
        book.created_slot = Clock::get()?.slot;
        state::init_owned(&book_info, &book)?;
        {
            let mut data = metas_info.try_borrow_mut_data()?;
            data.copy_from_slice(&metas::list_bytes(&floor_pool.to_bytes()));
        }
        book_key = want_book;
        emit_cpi!(LaunchHooked { mint, floor_pool, holder_book: want_book, extra_account_metas: want_metas });
    }


    let registry_info = a.claims_registry.to_account_info();
    let Some(mut reg) = state::read_registry(&registry_info) else { return Ok(()) };
    let n: u8 = if hooked { CLAIMS_ADDRS_HOOKED } else { CLAIMS_ADDRS_REFERRED };
    let fits = (reg.current_len as u32) + (n as u32) <= CLAIMS_TABLE_MAX as u32;
    if reg.current_table == [0u8; 32] || a.claims_table.key().to_bytes() != reg.current_table || !fits {
        return Ok(());
    }
    let addresses: Vec<Pubkey> =
        if hooked { vec![mint, book_key, floor_pool, amm_pool, vault0] } else { vec![floor_pool, amm_pool, vault0] };
    debug_assert_eq!(addresses.len(), n as usize);
    let alt = AltCpi {
        alt_program: &a.alt_program.to_account_info(),
        table: &a.claims_table.to_account_info(),
        authority: &a.claims_alt_authority.to_account_info(),
        authority_bump: ctx.bumps.claims_alt_authority,
        payer: &payer,
        system_program: &a.system_program.to_account_info(),
    };
    alt.extend(&addresses)?;
    reg.current_len += n as u16;
    let (table, len) = (Pubkey::new_from_array(reg.current_table), reg.current_len);
    state::write_owned(&registry_info, &reg)?;
    emit_cpi!(ClaimsTableExtended {
        table,
        mint: if hooked { mint } else { Pubkey::default() },
        floor_pool,
        added: n,
        len,
    });
    Ok(())
}
