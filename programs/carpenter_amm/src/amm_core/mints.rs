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
use carpenter_mint_gate::{self as gate, GateError};
use carpenter_types::keys::WSOL_MINT;

use crate::errors::AmmError;
use crate::state::{MintExclusivity, MintPoolCount, QuoteMint};





pub use carpenter_mint_gate::{
    BaseHook, MintRole, PausableRule, QUOTE_CAPS_CUSTODIAL, QUOTE_CAPS_INERT, QUOTE_CAP_CONFIDENTIAL, QUOTE_CAP_DEFAULT_STATE,
    QUOTE_CAP_FREEZE, QUOTE_CAP_HOOK_SLOT, QUOTE_CAP_INTEREST, QUOTE_CAP_PAUSABLE, QUOTE_CAP_PERMANENT_DELEGATE,
    QUOTE_CAP_SCALED_UI, SPL_TOKEN_ID, TOKEN_2022_ID,
};

///
pub const WSOL: Pubkey = Pubkey::new_from_array(WSOL_MINT);

///
fn gate_err(e: GateError) -> anchor_lang::error::Error {
    match e {
        GateError::MintNotSupported => error!(AmmError::MintNotSupported),
        GateError::MintNotSupportedAsBase => error!(AmmError::MintNotSupportedAsBase),
        GateError::CapabilityNotAdmitted => error!(AmmError::QuoteCapsNotAdmitted),
    }
}

///
///
///
///
///
///
///
///
///
pub fn check_mint_supported_with(mint: &AccountInfo, role: MintRole, pausable: PausableRule) -> Result<u8> {
    check_mint_supported_with_hook(mint, role, pausable, BaseHook::None)
}

///
///
///
///
pub fn check_mint_supported_with_hook(
    mint: &AccountInfo,
    role: MintRole,
    pausable: PausableRule,
    base_hook: BaseHook,
) -> Result<u8> {
    let data = mint.try_borrow_data()?;
    gate::check_mint_supported_with_hook(mint.owner, &mint.key(), &data, role, pausable, base_hook).map_err(gate_err)
}

///
///
///
pub fn check_mint_supported(mint: &AccountInfo, role: MintRole) -> Result<u8> {
    check_mint_supported_with(mint, role, PausableRule::Renounced)
}

///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
pub fn check_admitted(mint: &AccountInfo, caps: u8, admitted: u8) -> Result<()> {
    if *mint.owner == SPL_TOKEN_ID {
        return Ok(());
    }
    gate::check_admitted(caps, admitted).map_err(gate_err)
}

///
///
///
///
///
pub fn authority_digest(mint: &AccountInfo) -> Result<[u8; 32]> {
    let data = mint.try_borrow_data()?;
    gate::authority_digest(&data).map_err(gate_err)
}

///
///
///
///
///
///
pub fn implied_quote(mint: &Pubkey) -> bool {
    *mint == WSOL
}

///
///
pub fn require_derived(info: &AccountInfo, seed: &[u8], mint: &Pubkey) -> Result<u8> {
    let (expected, bump) = Pubkey::find_program_address(&[seed, mint.as_ref()], &crate::ID);
    require_keys_eq!(info.key(), expected, AmmError::SeedsMismatch);
    Ok(bump)
}

///
///
pub fn is_present(info: &AccountInfo, len: usize, discriminator: &[u8]) -> Result<bool> {
    if *info.owner != crate::ID || info.data_len() != len {
        return Ok(false);
    }
    let data = info.try_borrow_data()?;
    Ok(&data[..8] == discriminator)
}

///
pub fn read_exclusivity(info: &AccountInfo, mint: &Pubkey) -> Result<Option<MintExclusivity>> {
    require_derived(info, carpenter_types::seeds::MINT_EXCLUSIVITY, mint)?;
    if !is_present(info, MintExclusivity::LEN, MintExclusivity::DISCRIMINATOR)? {
        return Ok(None);
    }
    let data = info.try_borrow_data()?;
    Ok(Some(bytemuck::pod_read_unaligned(&data[8..MintExclusivity::LEN])))
}

///
pub fn read_pool_count(info: &AccountInfo, mint: &Pubkey) -> Result<Option<u64>> {
    require_derived(info, carpenter_types::seeds::MINT_POOLS, mint)?;
    if !is_present(info, MintPoolCount::LEN, MintPoolCount::DISCRIMINATOR)? {
        return Ok(None);
    }
    let data = info.try_borrow_data()?;
    let mpc: MintPoolCount = bytemuck::pod_read_unaligned(&data[8..MintPoolCount::LEN]);
    Ok(Some(mpc.count))
}

///
pub fn read_quote_mint(info: &AccountInfo, mint: &Pubkey) -> Result<Option<QuoteMint>> {
    require_derived(info, carpenter_types::seeds::QUOTE_MINT, mint)?;
    if !is_present(info, QuoteMint::LEN, QuoteMint::DISCRIMINATOR)? {
        return Ok(None);
    }
    let data = info.try_borrow_data()?;
    Ok(Some(bytemuck::pod_read_unaligned(&data[8..QuoteMint::LEN])))
}

///
///
///
///
///
pub fn may_pair_as_quote(entry: Option<&QuoteMint>, hook: &Pubkey) -> bool {
    match entry {
        Some(q) if q.state == carpenter_types::layout::QUOTE_STATE_LISTED => true,


        Some(q) if q.state == carpenter_types::layout::QUOTE_STATE_HOOK_LISTED => {
            *hook != Pubkey::default() && q.hook_program.to_bytes() == hook.to_bytes()
        }
        _ => false,
    }
}

///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
pub fn ever_listed(entry: Option<&QuoteMint>) -> bool {
    entry.map(|q| q.hook_program == Pubkey::default()).unwrap_or(false)
}

///
///
pub fn require_unpooled(mint_pools: &AccountInfo, mint: &Pubkey) -> Result<()> {
    let count = read_pool_count(mint_pools, mint)?.unwrap_or(0);
    require!(count == 0, AmmError::MintAlreadyPooled);
    Ok(())
}
