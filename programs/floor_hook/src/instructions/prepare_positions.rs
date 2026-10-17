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
use anchor_lang::solana_program::instruction::{AccountMeta, Instruction};
use anchor_lang::solana_program::program::invoke_signed;
use floor_types::constants::{
    SALT_BID, SALT_SEED, SALT_STAIR_RESERVE, SALT_STAIR_STEP0, SALT_STAIR_TAIL,
};
use floor_types::disc::amm as amm_disc;

use crate::constants::{CARPENTER_AMM_PROGRAM, HOOK_AUTHORITY_SEED};
use crate::errors::HookError;
use crate::mech::amm_cpi::position_key;
use crate::state::FloorPool;

#[derive(Accounts)]
pub struct PreparePositions<'info> {
///
///
    pub signer: Signer<'info>,
    pub floor_pool: AccountLoader<'info, FloorPool>,
///
    #[account(address = floor_pool.load()?.amm_pool @ HookError::WrongPool)]
    pub pool: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub position_seed: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub position_bid: UncheckedAccount<'info>,
///
///
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
///
    #[account(address = CARPENTER_AMM_PROGRAM @ HookError::BadConfig)]
    pub carpenter_amm: UncheckedAccount<'info>,
///
    #[account(seeds = [HOOK_AUTHORITY_SEED], bump)]
    pub hook_authority: UncheckedAccount<'info>,
}

///
///
const SALTS: [(u64, usize); 2] = [(SALT_SEED, 0), (SALT_BID, 1)];

pub fn handler<'info>(ctx: Context<'info, PreparePositions<'info>>) -> Result<()> {
    let stair = {
        let s = ctx.accounts.floor_pool.load()?;
        require!(s.pool_bound == 1, HookError::NotBound);
        require_keys_eq!(ctx.accounts.signer.key(), s.pool_initializer, HookError::NotInitializer);
        s.stair_steps != 0
    };

    let a = &ctx.accounts;
    let pool_key = a.pool.key();
    let authority = a.hook_authority.key();
    let slots = [
        a.position_seed.to_account_info(),
        a.position_bid.to_account_info(),
    ];
    let bump = [ctx.bumps.hook_authority];
    let signer_seeds: &[&[u8]] = &[HOOK_AUTHORITY_SEED, &bump];

    for (salt, slot) in SALTS {
        plant(a, &slots[slot], salt, &pool_key, &authority, signer_seeds)?;
    }




    if stair {
        let find = |key: Pubkey| {
            ctx.remaining_accounts
                .iter()
                .find(|x| *x.key == key)
                .ok_or_else(|| error!(anchor_lang::error::ErrorCode::AccountNotEnoughKeys))
        };
        for salt in [SALT_STAIR_STEP0, SALT_STAIR_RESERVE, SALT_STAIR_TAIL] {
            let pos = find(position_key(&pool_key, &authority, salt))?;
            plant(a, pos, salt, &pool_key, &authority, signer_seeds)?;
        }
        let fp = a.floor_pool.key();
        let st = find(crate::mech::stair::stair_key(&fp).0)?;
        let owner = a.floor_pool.load()?.stair_owner;
        crate::mech::stair::create_stair_state(&a.payer.to_account_info(), st, &a.system_program.to_account_info(), &fp, owner)?;
    }
    Ok(())
}

///
///
fn plant<'info>(
    a: &PreparePositions<'info>,
    position: &AccountInfo<'info>,
    salt: u64,
    pool_key: &Pubkey,
    authority: &Pubkey,
    signer_seeds: &[&[u8]],
) -> Result<()> {
    {


        require_keys_eq!(
            *position.key,
            position_key(pool_key, authority, salt),
            anchor_lang::error::ErrorCode::ConstraintSeeds
        );

        let mut data = Vec::with_capacity(16);
        data.extend_from_slice(&amm_disc::INITIALIZE_POSITION);
        data.extend_from_slice(&salt.to_le_bytes());


        let ix = Instruction {
            program_id: CARPENTER_AMM_PROGRAM,
            accounts: vec![
                AccountMeta::new(a.payer.key(), true),
                AccountMeta::new_readonly(*pool_key, false),
                AccountMeta::new(*position.key, false),
                AccountMeta::new_readonly(*authority, true),
                AccountMeta::new_readonly(a.system_program.key(), false),
            ],
            data,
        };
        invoke_signed(
            &ix,
            &[
                a.payer.to_account_info(),
                a.pool.to_account_info(),
                position.clone(),
                a.hook_authority.to_account_info(),
                a.system_program.to_account_info(),
            ],
            &[signer_seeds],
        )?;
    }
    Ok(())
}
