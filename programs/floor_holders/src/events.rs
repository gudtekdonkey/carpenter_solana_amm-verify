//!
//!
//!
//!
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

///
#[event]
pub struct LaunchHooked {
    pub mint: Pubkey,
    pub floor_pool: Pubkey,
    pub holder_book: Pubkey,
    pub extra_account_metas: Pubkey,
}

///
#[event]
pub struct ClaimsTableRotated {
    pub table: Pubkey,
///
    pub tables: u64,
}

///
#[event]
pub struct ClaimsTableExtended {
    pub table: Pubkey,
///
    pub mint: Pubkey,
    pub floor_pool: Pubkey,
    pub added: u8,
///
    pub len: u16,
}

///
#[event]
pub struct RecordOpened {
    pub mint: Pubkey,
    pub record: Pubkey,
    pub token_account: Pubkey,
    pub owner: Pubkey,
///
    pub payer: Pubkey,
    pub shares: u64,
}

///
#[event]
pub struct HolderClaimed {
    pub mint: Pubkey,
    pub record: Pubkey,
    pub owner: Pubkey,
    pub destination: Pubkey,
    pub amount: u64,
}

///
///
#[event]
pub struct RecordClosed {
    pub mint: Pubkey,
    pub record: Pubkey,
    pub owner: Pubkey,
    pub refund_lamports: u64,
}
