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
use anchor_spl::token_2022::spl_token_2022;
use anchor_spl::token_2022::spl_token_2022::extension::pausable::PausableConfig;
use anchor_spl::token_2022::spl_token_2022::extension::{BaseStateWithExtensions, StateWithExtensions};

use crate::errors::VestingError;

///
///
///
pub fn is_paused(mint: &AccountInfo<'_>) -> Result<bool> {
    let data = mint.try_borrow_data()?;
    let Ok(state) = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&data) else {




        return Ok(false);
    };
    Ok(match state.get_extension::<PausableConfig>() {
        Ok(cfg) => bool::from(cfg.paused),
        Err(_) => false,
    })
}

///
pub fn require_not_paused(mint: &AccountInfo<'_>) -> Result<()> {
    require!(!is_paused(mint)?, VestingError::MintPaused);
    Ok(())
}
