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
use anchor_spl::token_2022::Token2022;
use floor_types::constants::METADATA_FILL_DELAY_SLOTS;
use floor_types::metadata::metadata_uri;

use crate::constants::{LAUNCH_CONFIG_SEED, LAUNCH_SEED, MINT_AUTHORITY_SEED};
use crate::errors::LaunchError;
use crate::events::MetadataUriFilled;
use crate::mint_profile::{self, FillUri};
use crate::state::{Launch, LaunchConfig};

#[event_cpi]
#[derive(Accounts)]
pub struct FillMetadataUri<'info> {
///
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(seeds = [LAUNCH_CONFIG_SEED], bump)]
    pub launch_config: AccountLoader<'info, LaunchConfig>,
///
///
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,
///
    #[account(seeds = [LAUNCH_SEED, mint.key().as_ref()], bump)]
    pub launch: AccountLoader<'info, Launch>,
///
    #[account(seeds = [MINT_AUTHORITY_SEED], bump)]
    pub mint_authority: UncheckedAccount<'info>,
    pub token_2022: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<FillMetadataUri>) -> Result<()> {
    let mint_key = ctx.accounts.mint.key();

    let base = {
        let c = ctx.accounts.launch_config.load()?;
        let len = c.metadata_base_len as usize;
        require!(len > 0, LaunchError::MetadataBaseUnset);



        require!(
            Clock::get()?.slot >= c.metadata_base_set_slot.saturating_add(METADATA_FILL_DELAY_SLOTS),
            LaunchError::MetadataBaseTooFresh
        );


        String::from_utf8(c.metadata_base[..len].to_vec()).map_err(|_| LaunchError::MetadataBaseInvalid)?
    };
    {
        let l = ctx.accounts.launch.load()?;
        require_keys_eq!(l.mint, mint_key, LaunchError::BadConfig);
    }

    let uri = metadata_uri(&base, &mint_key.to_string());
    let ma_bump = [ctx.bumps.mint_authority];
    let ma_seeds: &[&[u8]] = &[MINT_AUTHORITY_SEED, &ma_bump];
    let payer = ctx.accounts.payer.to_account_info();
    let mint = ctx.accounts.mint.to_account_info();
    let mint_authority = ctx.accounts.mint_authority.to_account_info();
    let token_2022 = ctx.accounts.token_2022.to_account_info();
    let system_program = ctx.accounts.system_program.to_account_info();

    mint_profile::fill_uri(
        &FillUri {
            payer: &payer,
            mint: &mint,
            mint_authority: &mint_authority,
            mint_authority_seeds: ma_seeds,
            token_2022: &token_2022,
            system_program: &system_program,
        },
        uri.clone(),
    )?;


    mint_profile::verify_metadata(&mint, &uri, None)?;

    emit_cpi!(MetadataUriFilled { launch: ctx.accounts.launch.key(), mint: mint_key, uri });
    Ok(())
}
