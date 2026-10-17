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

use crate::constants::{FLOOR_HOOK_PROGRAM, HOLDERS_AUTHORITY_SEED, PAY_HOLDER_DISCRIMINATOR};

///
pub struct PayHolderCpi<'a, 'info> {
    pub floor_hook: &'a AccountInfo<'info>,
    pub holders_authority: &'a AccountInfo<'info>,
    pub holders_authority_bump: u8,
    pub floor_pool: &'a AccountInfo<'info>,
    pub pool: &'a AccountInfo<'info>,
    pub vault0: &'a AccountInfo<'info>,
    pub destination: &'a AccountInfo<'info>,
    pub quote_mint: &'a AccountInfo<'info>,
    pub quote_token_program: &'a AccountInfo<'info>,
    pub carpenter_amm: &'a AccountInfo<'info>,
    pub hook_authority: &'a AccountInfo<'info>,
    pub amm_event_authority: &'a AccountInfo<'info>,
///
    pub hook_event_authority: &'a AccountInfo<'info>,
}

///
pub fn pay_holder(cpi: &PayHolderCpi, amount: u64) -> Result<()> {
    let mut data = Vec::with_capacity(8 + 8);
    data.extend_from_slice(&PAY_HOLDER_DISCRIMINATOR);
    data.extend_from_slice(&amount.to_le_bytes());

    let metas = vec![
        AccountMeta::new_readonly(*cpi.holders_authority.key, true),
        AccountMeta::new(*cpi.floor_pool.key, false),
        AccountMeta::new(*cpi.pool.key, false),
        AccountMeta::new(*cpi.vault0.key, false),
        AccountMeta::new(*cpi.destination.key, false),
        AccountMeta::new_readonly(*cpi.quote_mint.key, false),
        AccountMeta::new_readonly(*cpi.quote_token_program.key, false),
        AccountMeta::new_readonly(*cpi.carpenter_amm.key, false),
        AccountMeta::new_readonly(*cpi.hook_authority.key, false),
        AccountMeta::new_readonly(*cpi.amm_event_authority.key, false),
        AccountMeta::new_readonly(*cpi.hook_event_authority.key, false),
        AccountMeta::new_readonly(*cpi.floor_hook.key, false),
    ];
    let infos = [
        cpi.holders_authority.clone(),
        cpi.floor_pool.clone(),
        cpi.pool.clone(),
        cpi.vault0.clone(),
        cpi.destination.clone(),
        cpi.quote_mint.clone(),
        cpi.quote_token_program.clone(),
        cpi.carpenter_amm.clone(),
        cpi.hook_authority.clone(),
        cpi.amm_event_authority.clone(),
        cpi.hook_event_authority.clone(),
        cpi.floor_hook.clone(),
    ];
    let ix = Instruction { program_id: FLOOR_HOOK_PROGRAM, accounts: metas, data };
    let bump = [cpi.holders_authority_bump];
    let seeds: &[&[u8]] = &[HOLDERS_AUTHORITY_SEED, &bump];
    invoke_signed(&ix, &infos, &[seeds])?;
    Ok(())
}
