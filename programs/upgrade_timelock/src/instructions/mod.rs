pub mod accept_proposer;
pub mod cancel;
pub mod execute;
pub mod initialize;
pub mod schedule;

pub use accept_proposer::*;
pub use cancel::*;
pub use execute::*;
pub use initialize::*;
pub use schedule::*;

use anchor_lang::prelude::*;

use crate::constants::{AUTHORITY_SEED, TIMELOCK_SEED};

///
pub fn unsignable() -> [Pubkey; 2] {
    [
        Pubkey::find_program_address(&[TIMELOCK_SEED], &crate::ID).0,
        Pubkey::find_program_address(&[AUTHORITY_SEED], &crate::ID).0,
    ]
}
