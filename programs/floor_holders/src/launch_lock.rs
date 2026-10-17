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

use floor_types::seeds::*;

use crate::constants::FLOOR_LAUNCH_PROGRAM;

///
pub fn launch_lock_authority() -> Pubkey {
    Pubkey::find_program_address(&[LOCK_AUTHORITY], &FLOOR_LAUNCH_PROGRAM).0
}
