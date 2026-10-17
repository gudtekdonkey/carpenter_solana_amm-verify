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

use crate::constants::{GENESIS_SALE_SEED, GRAD_RENT_SEED, LAUNCH_SEED};
use crate::errors::LaunchError;
use crate::events::GraduationRentClosed;
use crate::graduation_rent;
use crate::state::{GenesisSale, Launch};
use floor_types::constants::{GENESIS_LADDER_TOKENS, GENESIS_POL_TOKENS, GENESIS_SALE_TOKENS};

///
///
///
pub fn is_genesis_launch(l: &Launch) -> bool {
    l.sale_tokens == GENESIS_SALE_TOKENS && l.pol_tokens == GENESIS_POL_TOKENS && l.ladder_tokens == GENESIS_LADDER_TOKENS
}

///
///
fn require_genesis_done(launch: &Pubkey, remaining: &[AccountInfo]) -> Result<()> {
    require!(remaining.len() == 1, LaunchError::GenesisRentLocked);
    let info = &remaining[0];
    let (pda, _) = Pubkey::find_program_address(&[GENESIS_SALE_SEED], &crate::ID);
    require_keys_eq!(info.key(), pda, LaunchError::GenesisRentLocked);
    require_keys_eq!(*info.owner, crate::ID, LaunchError::GenesisRentLocked);
    let data = info.try_borrow_data()?;
    require!(
        data.len() >= 8 + core::mem::size_of::<GenesisSale>() && data[..8] == *GenesisSale::DISCRIMINATOR,
        LaunchError::GenesisRentLocked
    );
    let g: &GenesisSale = bytemuck::from_bytes(&data[8..8 + core::mem::size_of::<GenesisSale>()]);
    require_keys_eq!(g.launch, *launch, LaunchError::GenesisRentLocked);
    require!(g.graduated == 1 && g.locked == 1, LaunchError::GenesisRentLocked);
    Ok(())
}

#[event_cpi]
#[derive(Accounts)]
pub struct CloseGraduationRent<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(seeds = [LAUNCH_SEED, launch.load()?.mint.as_ref()], bump = launch.load()?.bump)]
    pub launch: AccountLoader<'info, Launch>,
///
    #[account(mut, seeds = [GRAD_RENT_SEED, launch.key().as_ref()], bump)]
    pub graduation_rent: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<CloseGraduationRent>) -> Result<()> {
    let launch_key = ctx.accounts.launch.key();
    let (creator, genesis, keep, graduated) = {
        let l = ctx.accounts.launch.load()?;




        let graduated = l.state == floor_types::constants::LAUNCH_STATE_GRADUATED;
        (l.creator, is_genesis_launch(&l), if graduated { l.stair_rent_budget as u64 } else { 0 }, graduated)
    };
    require_keys_eq!(ctx.accounts.creator.key(), creator, LaunchError::NotLaunchCreator);
    if genesis {

        require_genesis_done(&launch_key, ctx.remaining_accounts)?;
    } else {

        require!(graduated, LaunchError::GraduationRentLocked);
    }

    let reserve = ctx.accounts.graduation_rent.to_account_info();
    let to = ctx.accounts.creator.to_account_info();
    let lamports = graduation_rent::close(&reserve, &to, keep)?;

    emit_cpi!(GraduationRentClosed {
        launch: launch_key,
        reserve: ctx.accounts.graduation_rent.key(),
        creator,
        lamports,
    });
    Ok(())
}
