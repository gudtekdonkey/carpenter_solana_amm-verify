//!
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

#[error_code(offset = 8000)]
pub enum HoldersError {
///
    #[msg("init_launch must be signed by floor_launch's lock authority")]
    NotLaunchAuthority,
///
    #[msg("the account belongs to a different mint")]
    WrongMint,
///
///
    #[msg("the floor pool is not this launch's")]
    WrongPool,
///
///
    #[msg("execute may only run inside a Token-2022 transfer")]
    NotTransferring,
///
    #[msg("execute needs the nine accounts the extra-account list names")]
    MissingExtraAccounts,
///
///
    #[msg("not the owner's Token-2022 associated token account for this mint")]
    NotAssociatedTokenAccount,
///
///
///
///
    #[msg("an off-curve (program-owned) owner cannot open a holder record")]
    OwnerOffCurve,
///
    #[msg("this token account already has a holder record")]
    RecordExists,
///
    #[msg("no holder record for this token account")]
    NoRecord,
///
    #[msg("signer is not the owner of this holder record")]
    NotRecordOwner,
///
    #[msg("the record still holds a balance or is still owed")]
    RecordNotEmpty,
///
    #[msg("this launch is not hooked: it has no holder rewards")]
    NotHooked,
///
///
    #[msg("destination must be the record owner's account of the pool's quote")]
    WrongDestination,
///
    #[msg("the current claims lookup table still has room")]
    ClaimsTableNotFull,
///
    #[msg("arithmetic overflow")]
    MathOverflow,
    #[msg("invalid atomic holder payout authorization")]
    InvalidPayoutAuthorization,
}
