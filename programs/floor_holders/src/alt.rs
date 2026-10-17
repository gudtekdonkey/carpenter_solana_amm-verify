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

use crate::constants::{ALT_IX_CREATE, ALT_IX_EXTEND, ALT_PROGRAM, CLAIMS_ALT_AUTHORITY_SEED};

///
pub fn derive_table(authority: &Pubkey, recent_slot: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[authority.as_ref(), &recent_slot.to_le_bytes()], &ALT_PROGRAM)
}

///
pub struct AltCpi<'a, 'info> {
    pub alt_program: &'a AccountInfo<'info>,
    pub table: &'a AccountInfo<'info>,
    pub authority: &'a AccountInfo<'info>,
    pub authority_bump: u8,
    pub payer: &'a AccountInfo<'info>,
    pub system_program: &'a AccountInfo<'info>,
}

impl<'a, 'info> AltCpi<'a, 'info> {
    fn metas(&self) -> Vec<AccountMeta> {
        vec![
            AccountMeta::new(*self.table.key, false),
            AccountMeta::new_readonly(*self.authority.key, true),
            AccountMeta::new(*self.payer.key, true),
            AccountMeta::new_readonly(*self.system_program.key, false),
        ]
    }

    fn invoke(&self, data: Vec<u8>) -> Result<()> {
        let ix = Instruction { program_id: ALT_PROGRAM, accounts: self.metas(), data };
        let infos = [
            self.table.clone(),
            self.authority.clone(),
            self.payer.clone(),
            self.system_program.clone(),
            self.alt_program.clone(),
        ];
        let bump = [self.authority_bump];
        let seeds: &[&[u8]] = &[CLAIMS_ALT_AUTHORITY_SEED, &bump];
        invoke_signed(&ix, &infos, &[seeds])?;
        Ok(())
    }

///
    pub fn create(&self, recent_slot: u64, bump_seed: u8) -> Result<()> {
        let mut data = Vec::with_capacity(4 + 8 + 1);
        data.extend_from_slice(&ALT_IX_CREATE.to_le_bytes());
        data.extend_from_slice(&recent_slot.to_le_bytes());
        data.push(bump_seed);
        self.invoke(data)
    }

///
    pub fn extend(&self, addresses: &[Pubkey]) -> Result<()> {
        let mut data = Vec::with_capacity(4 + 8 + 32 * addresses.len());
        data.extend_from_slice(&ALT_IX_EXTEND.to_le_bytes());
        data.extend_from_slice(&(addresses.len() as u64).to_le_bytes());
        for a in addresses {
            data.extend_from_slice(a.as_ref());
        }
        self.invoke(data)
    }
}
