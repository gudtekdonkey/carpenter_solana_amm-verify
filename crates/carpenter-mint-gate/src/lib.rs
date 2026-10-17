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

#![deny(missing_docs)]
#![forbid(unsafe_code)]

use solana_pubkey::Pubkey;
use spl_token_2022_interface::extension::default_account_state::DefaultAccountState;
use spl_token_2022_interface::extension::pausable::PausableConfig;
use spl_token_2022_interface::extension::permanent_delegate::PermanentDelegate;
use spl_token_2022_interface::extension::transfer_hook::TransferHook;
use spl_token_2022_interface::extension::{BaseStateWithExtensions, ExtensionType, StateWithExtensions};
use spl_token_2022_interface::state::Mint;

///
///
pub const SPL_TOKEN_ID: Pubkey = spl_token_2022_interface::inline_spl_token::ID;
///
pub const TOKEN_2022_ID: Pubkey = spl_token_2022_interface::ID;
///
///
pub const TOKEN_2022_NATIVE_MINT: Pubkey = spl_token_2022_interface::native_mint::ID;

///
///
pub const CLASSIC_MINT_LEN: usize = 82;





///
///
pub const QUOTE_CAP_FREEZE: u8 = 1 << 0;
///
pub const QUOTE_CAP_PERMANENT_DELEGATE: u8 = 1 << 1;
///
pub const QUOTE_CAP_PAUSABLE: u8 = 1 << 2;
///
pub const QUOTE_CAP_DEFAULT_STATE: u8 = 1 << 3;
///
pub const QUOTE_CAP_HOOK_SLOT: u8 = 1 << 4;
///
pub const QUOTE_CAP_SCALED_UI: u8 = 1 << 5;
///
pub const QUOTE_CAP_INTEREST: u8 = 1 << 6;
///
pub const QUOTE_CAP_CONFIDENTIAL: u8 = 1 << 7;

///
///
pub const QUOTE_CAPS_CUSTODIAL: u8 =
    QUOTE_CAP_FREEZE | QUOTE_CAP_PERMANENT_DELEGATE | QUOTE_CAP_PAUSABLE | QUOTE_CAP_DEFAULT_STATE | QUOTE_CAP_HOOK_SLOT;

///
pub const QUOTE_CAPS_INERT: u8 = QUOTE_CAP_SCALED_UI | QUOTE_CAP_INTEREST | QUOTE_CAP_CONFIDENTIAL;

///
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MintRole {
///
    Base,
///
///
    ListedQuote,
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
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PausableRule {
///
    Renounced,
///
    HookVouched,
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
///
///
///
///
///
///
///
///
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BaseHook {
///
    None,
///
///
    Admitted(Pubkey),
}

///
///
///
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GateError {
///
///
    MintNotSupported,
///
    MintNotSupportedAsBase,
///
///
    CapabilityNotAdmitted,
}

impl core::fmt::Display for GateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            GateError::MintNotSupported => "MintNotSupported",
            GateError::MintNotSupportedAsBase => "MintNotSupportedAsBase",
            GateError::CapabilityNotAdmitted => "CapabilityNotAdmitted",
        })
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
pub fn check_mint_supported(
    owner: &Pubkey,
    key: &Pubkey,
    data: &[u8],
    role: MintRole,
    pausable: PausableRule,
) -> Result<u8, GateError> {
    check_mint_supported_with_hook(owner, key, data, role, pausable, BaseHook::None)
}

///
///
///
///
pub fn check_mint_supported_with_hook(
    owner: &Pubkey,
    key: &Pubkey,
    data: &[u8],
    role: MintRole,
    pausable: PausableRule,
    base_hook: BaseHook,
) -> Result<u8, GateError> {
    if *owner == SPL_TOKEN_ID {




        if data.len() != CLASSIC_MINT_LEN {
            return Err(GateError::MintNotSupported);
        }
        let state = unpack(data)?;
        return Ok(if state.base.freeze_authority.is_some() { QUOTE_CAP_FREEZE } else { 0 });
    }
    if *owner != TOKEN_2022_ID {
        return Err(GateError::MintNotSupported);
    }
    if *key == TOKEN_2022_NATIVE_MINT {
        return Err(GateError::MintNotSupported);
    }

    let state = unpack(data)?;
    let mut caps: u8 = 0;
    if state.base.freeze_authority.is_some() {
        caps |= QUOTE_CAP_FREEZE;
    }


    let vouched = pausable == PausableRule::HookVouched && role == MintRole::Base;


    let admitted: Option<Pubkey> = match base_hook {
        BaseHook::Admitted(p) if vouched && p != Pubkey::default() => Some(p),
        _ => None,
    };

    let types = state.get_extension_types().map_err(|_| GateError::MintNotSupported)?;
    for t in types {
        match t {

            ExtensionType::Uninitialized
            | ExtensionType::MetadataPointer
            | ExtensionType::TokenMetadata
            | ExtensionType::GroupPointer
            | ExtensionType::GroupMemberPointer
            | ExtensionType::TokenGroup
            | ExtensionType::TokenGroupMember => {}


            ExtensionType::ScaledUiAmount => caps |= QUOTE_CAP_SCALED_UI,
            ExtensionType::InterestBearingConfig => caps |= QUOTE_CAP_INTEREST,
            ExtensionType::ConfidentialTransferMint => caps |= QUOTE_CAP_CONFIDENTIAL,


            ExtensionType::Pausable => {
                let cfg = state.get_extension::<PausableConfig>().map_err(|_| GateError::MintNotSupported)?;



                if bool::from(cfg.paused) && !vouched {
                    return Err(GateError::MintNotSupported);
                }
                if Option::<Pubkey>::from(cfg.authority).is_some() {
                    caps |= QUOTE_CAP_PAUSABLE;
                }
            }
            ExtensionType::PermanentDelegate => {
                let d = state.get_extension::<PermanentDelegate>().map_err(|_| GateError::MintNotSupported)?;
                if Option::<Pubkey>::from(d.delegate).is_some() {
                    caps |= QUOTE_CAP_PERMANENT_DELEGATE;
                }
            }
            ExtensionType::DefaultAccountState => {


                let _ = state.get_extension::<DefaultAccountState>().map_err(|_| GateError::MintNotSupported)?;
                caps |= QUOTE_CAP_DEFAULT_STATE;
            }
            ExtensionType::TransferHook => {
                let h = state.get_extension::<TransferHook>().map_err(|_| GateError::MintNotSupported)?;
                match Option::<Pubkey>::from(h.program_id) {




                    Some(program) => {
                        let renounced = Option::<Pubkey>::from(h.authority).is_none();
                        if !(admitted == Some(program) && renounced) {
                            return Err(GateError::MintNotSupported);
                        }
                    }


                    None => caps |= QUOTE_CAP_HOOK_SLOT,
                }
            }



            ExtensionType::TransferFeeConfig
            | ExtensionType::TransferFeeAmount
            | ExtensionType::ConfidentialTransferFeeConfig
            | ExtensionType::ConfidentialTransferFeeAmount
            | ExtensionType::NonTransferable
            | ExtensionType::MintCloseAuthority => return Err(GateError::MintNotSupported),




            _ => return Err(GateError::MintNotSupported),
        }
    }

    let mut refusing = caps & QUOTE_CAPS_CUSTODIAL;
    if vouched {

        refusing &= !QUOTE_CAP_PAUSABLE;
    }
    if admitted.is_some() {


        refusing &= !QUOTE_CAP_HOOK_SLOT;
    }
    if refusing != 0 && role == MintRole::Base {

        return Err(GateError::MintNotSupportedAsBase);
    }
    Ok(caps)
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
pub fn check_admitted(caps: u8, admitted: u8) -> Result<(), GateError> {
    if caps & QUOTE_CAPS_CUSTODIAL & !admitted != 0 {
        return Err(GateError::CapabilityNotAdmitted);
    }
    Ok(())
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
pub fn authority_digest(data: &[u8]) -> Result<[u8; 32], GateError> {
    let state = unpack(data)?;
    let mut buf = [0u8; 128];
    buf[0..32].copy_from_slice(&state.base.freeze_authority.map(|k| k.to_bytes()).unwrap_or([0u8; 32]));
    if let Ok(d) = state.get_extension::<PermanentDelegate>() {
        buf[32..64].copy_from_slice(&d.delegate.0.to_bytes());
    }
    if let Ok(p) = state.get_extension::<PausableConfig>() {
        buf[64..96].copy_from_slice(&p.authority.0.to_bytes());
    }
    if let Ok(h) = state.get_extension::<TransferHook>() {
        buf[96..128].copy_from_slice(&h.authority.0.to_bytes());
    }
    Ok(solana_sha256_hasher::hash(&buf).to_bytes())
}

///
///
///
///
fn unpack(data: &[u8]) -> Result<StateWithExtensions<'_, Mint>, GateError> {
    StateWithExtensions::<Mint>::unpack(data).map_err(|_| GateError::MintNotSupported)
}
