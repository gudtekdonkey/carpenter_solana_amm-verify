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
#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

pub mod alt;
pub mod constants;
pub mod create;
pub mod errors;
pub mod events;
pub mod hook_cpi;
pub mod instructions;
pub mod launch_lock;
pub mod metas;
pub mod state;
pub mod token;

use instructions::*;




declare_id!("2EUkJXSwCdRAuhesw2D9FXyGfbi6Kw2yZDyQUnCwc7te");

#[program]
pub mod floor_holders {
    use super::*;



///
    pub fn initialize_claims_registry(ctx: Context<InitializeClaimsRegistry>) -> Result<()> {
        instructions::initialize_claims_registry::handler(ctx)
    }

///
///
    pub fn rotate_claims_table(ctx: Context<RotateClaimsTable>, recent_slot: u64) -> Result<()> {
        instructions::rotate_claims_table::handler(ctx, recent_slot)
    }



///
///
    pub fn init_launch(ctx: Context<InitLaunch>, hooked: bool) -> Result<()> {
        instructions::init_launch::handler(ctx, hooked)
    }

///
///
    #[instruction(discriminator = [105, 37, 101, 197, 75, 251, 102, 26])]
    pub fn execute(ctx: Context<Execute>, amount: u64) -> Result<()> {
        instructions::execute::handler(ctx, amount)
    }



///
    pub fn open_record(ctx: Context<OpenRecord>) -> Result<()> {
        instructions::open_record::open_handler(ctx)
    }

///
    pub fn touch(ctx: Context<OpenRecord>) -> Result<()> {
        instructions::open_record::touch_handler(ctx)
    }

///
    pub fn sync(ctx: Context<SyncRecord>) -> Result<()> {
        instructions::sync::handler(ctx)
    }

///
    pub fn authorize_claim(ctx: Context<ClaimRewards>, amount: u64) -> Result<()> {
        instructions::claim::authorize(ctx, amount)
    }

    pub fn claim(ctx: Context<ClaimRewards>) -> Result<()> {
        instructions::claim::handler(ctx)
    }

///
    pub fn close_record(ctx: Context<CloseRecord>) -> Result<()> {
        instructions::close_record::handler(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_declared_id_is_floor_types_placeholder() {
        assert_eq!(crate::ID.to_bytes(), floor_types::holders::FLOOR_HOLDERS_ID);
    }

    #[test]
    fn the_execute_discriminator_is_the_one_the_program_dispatches() {
        assert_eq!(constants::EXECUTE_DISCRIMINATOR, [105, 37, 101, 197, 75, 251, 102, 26]);
    }
}
