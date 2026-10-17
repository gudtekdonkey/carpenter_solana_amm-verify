//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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
use anchor_spl::associated_token::{self, get_associated_token_address_with_program_id};
use anchor_spl::token_2022::spl_token_2022;
use anchor_spl::token_2022::spl_token_2022::extension::metadata_pointer::MetadataPointer;
use anchor_spl::token_2022::spl_token_2022::extension::{BaseStateWithExtensions, ExtensionType, StateWithExtensions};
use anchor_spl::token_2022::spl_token_2022::instruction::AuthorityType;
use anchor_spl::token_2022_extensions::spl_pod::optional_keys::OptionalNonZeroPubkey;
use anchor_spl::token_2022_extensions::spl_token_metadata_interface::state::{Field, TokenMetadata};
use anchor_spl::token_interface::{
    metadata_pointer_initialize, token_metadata_initialize, token_metadata_update_authority,
    token_metadata_update_field, MetadataPointerInitialize, TokenMetadataInitialize,
    TokenMetadataUpdateAuthority, TokenMetadataUpdateField,
};
use floor_types::constants::{SUPPLY, TOKEN_DECIMALS};

use crate::constants::MINT_SEED;
use crate::errors::LaunchError;
use crate::lock::{self, Lock};
use crate::sale::create_pda;

///
pub struct MintProfile<'a, 'info> {
///
    pub creator: &'a AccountInfo<'info>,
    pub mint: &'a AccountInfo<'info>,
    pub mint_bump: u8,
///
    pub nonce: u64,
///
    pub mint_authority: &'a AccountInfo<'info>,
    pub mint_authority_seeds: &'a [&'a [u8]],
    pub lock: Lock<'a, 'info>,
///
    pub launch: &'a AccountInfo<'info>,
    pub token_vault: &'a AccountInfo<'info>,
    pub token_2022: &'a AccountInfo<'info>,
    pub ata_program: &'a AccountInfo<'info>,
    pub system_program: &'a AccountInfo<'info>,
///
///
    pub hooked: bool,
}

///
pub const FIXED_EXTENSIONS: [ExtensionType; 2] = [ExtensionType::MetadataPointer, ExtensionType::Pausable];
///
///
///
pub const FIXED_EXTENSIONS_HOOKED: [ExtensionType; 3] =
    [ExtensionType::MetadataPointer, ExtensionType::Pausable, ExtensionType::TransferHook];

///
pub fn fixed_extensions(hooked: bool) -> &'static [ExtensionType] {
    if hooked {
        &FIXED_EXTENSIONS_HOOKED
    } else {
        &FIXED_EXTENSIONS
    }
}

///
pub struct FillUri<'a, 'info> {
///
    pub payer: &'a AccountInfo<'info>,
    pub mint: &'a AccountInfo<'info>,
///
    pub mint_authority: &'a AccountInfo<'info>,
    pub mint_authority_seeds: &'a [&'a [u8]],
    pub token_2022: &'a AccountInfo<'info>,
    pub system_program: &'a AccountInfo<'info>,
}

///
///
///
///
pub fn build(p: &MintProfile<'_, '_>, name: String, symbol: String, uri: String) -> Result<()> {

    let renounce_now = !uri.is_empty();
    let tp = p.token_2022.key();
    let mint_key = p.mint.key();
    let creator_key = p.creator.key();


    let fixed_len = ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(fixed_extensions(p.hooked))?;
    let nonce = p.nonce.to_le_bytes();
    let bump = [p.mint_bump];
    let mint_seeds: &[&[u8]] = &[MINT_SEED, creator_key.as_ref(), &nonce, &bump];
    create_pda(p.creator, p.mint, fixed_len, &tp, mint_seeds)?;


    metadata_pointer_initialize(
        CpiContext::new(tp, MetadataPointerInitialize { token_program_id: p.token_2022.clone(), mint: p.mint.clone() }),
        None,
        Some(mint_key),
    )?;




    if p.hooked {
        lock::install_hook_slot(&p.lock)?;
    }


    lock::install(&p.lock)?;


    anchor_spl::token_2022::initialize_mint2(
        CpiContext::new(tp, anchor_spl::token_2022::InitializeMint2 { mint: p.mint.clone() }),
        TOKEN_DECIMALS,
        &p.mint_authority.key(),
        None,
    )?;


    let metadata = TokenMetadata {
        update_authority: OptionalNonZeroPubkey::try_from(Some(p.mint_authority.key()))?,
        mint: mint_key,
        name: name.clone(),
        symbol: symbol.clone(),
        uri: uri.clone(),
        additional_metadata: vec![],
    };
    let final_len = p.mint.data_len().checked_add(metadata.tlv_size_of()?).ok_or(LaunchError::MintProfileInvalid)?;
    let required = Rent::get()?.minimum_balance(final_len);
    let have = p.mint.lamports();
    if have < required {
        anchor_lang::system_program::transfer(
            CpiContext::new(
                anchor_lang::system_program::ID,
                anchor_lang::system_program::Transfer { from: p.creator.clone(), to: p.mint.clone() },
            ),
            required - have,
        )?;
    }


    token_metadata_initialize(
        CpiContext::new_with_signer(
            tp,
            TokenMetadataInitialize {
                program_id: p.token_2022.clone(),
                metadata: p.mint.clone(),
                update_authority: p.mint_authority.clone(),
                mint_authority: p.mint_authority.clone(),
                mint: p.mint.clone(),
            },
            &[p.mint_authority_seeds],
        ),
        name,
        symbol,
        uri,
    )?;



    if renounce_now {
        renounce_metadata_authority(tp, p.token_2022, p.mint, p.mint_authority, p.mint_authority_seeds)?;
    }



    require_keys_eq!(
        p.token_vault.key(),
        get_associated_token_address_with_program_id(&p.launch.key(), &mint_key, &tp),
        LaunchError::NotAssociatedTokenAccount
    );
    associated_token::create_idempotent(CpiContext::new(
        p.ata_program.key(),
        associated_token::Create {
            payer: p.creator.clone(),
            associated_token: p.token_vault.clone(),
            authority: p.launch.clone(),
            mint: p.mint.clone(),
            system_program: p.system_program.clone(),
            token_program: p.token_2022.clone(),
        },
    ))?;


    anchor_spl::token_2022::mint_to(
        CpiContext::new_with_signer(
            tp,
            anchor_spl::token_2022::MintTo {
                mint: p.mint.clone(),
                to: p.token_vault.clone(),
                authority: p.mint_authority.clone(),
            },
            &[p.mint_authority_seeds],
        ),
        SUPPLY,
    )?;


    anchor_spl::token_2022::set_authority(
        CpiContext::new_with_signer(
            tp,
            anchor_spl::token_2022::SetAuthority {
                current_authority: p.mint_authority.clone(),
                account_or_mint: p.mint.clone(),
            },
            &[p.mint_authority_seeds],
        ),
        AuthorityType::MintTokens,
        None,
    )?;


    lock::engage(&p.lock)
}

///
///
///
///
///
///
fn renounce_metadata_authority<'info>(
    token_program_id: Pubkey,
    token_2022: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    mint_authority: &AccountInfo<'info>,
    seeds: &[&[u8]],
) -> Result<()> {
    token_metadata_update_authority(
        CpiContext::new_with_signer(
            token_program_id,
            TokenMetadataUpdateAuthority {
                program_id: token_2022.clone(),
                metadata: mint.clone(),
                current_authority: mint_authority.clone(),
                new_authority: mint_authority.clone(),
            },
            &[seeds],
        ),
        OptionalNonZeroPubkey::default(),
    )
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
pub fn fill_uri(p: &FillUri<'_, '_>, uri: String) -> Result<()> {
    require!(!uri.is_empty(), LaunchError::MetadataBaseUnset);
    {
        let data = p.mint.try_borrow_data()?;
        let state = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&data)?;
        let md = state.get_variable_len_extension::<TokenMetadata>()?;
        require!(
            md.uri.is_empty() && Option::<Pubkey>::from(md.update_authority) == Some(p.mint_authority.key()),
            LaunchError::MetadataUriFinal
        );
    }

    let final_len = p.mint.data_len().checked_add(uri.len()).ok_or(LaunchError::MintProfileInvalid)?;
    let required = Rent::get()?.minimum_balance(final_len);
    let have = p.mint.lamports();
    if have < required {
        anchor_lang::system_program::transfer(
            CpiContext::new(
                p.system_program.key(),
                anchor_lang::system_program::Transfer { from: p.payer.clone(), to: p.mint.clone() },
            ),
            required - have,
        )?;
    }

    let tp = p.token_2022.key();
    token_metadata_update_field(
        CpiContext::new_with_signer(
            tp,
            TokenMetadataUpdateField {
                program_id: p.token_2022.clone(),
                metadata: p.mint.clone(),
                update_authority: p.mint_authority.clone(),
            },
            &[p.mint_authority_seeds],
        ),
        Field::Uri,
        uri,
    )?;

    renounce_metadata_authority(tp, p.token_2022, p.mint, p.mint_authority, p.mint_authority_seeds)
}

///
///
pub fn verify_metadata(mint: &AccountInfo<'_>, uri: &str, update_authority: Option<Pubkey>) -> Result<()> {
    let data = mint.try_borrow_data()?;
    let state = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&data)?;
    let md = state.get_variable_len_extension::<TokenMetadata>()?;
    require!(
        Option::<Pubkey>::from(md.update_authority) == update_authority && md.mint == mint.key() && md.uri == uri,
        LaunchError::MintProfileInvalid
    );
    Ok(())
}

///
///
///
///
///
pub fn verify(
    mint: &AccountInfo<'_>,
    token_vault: &AccountInfo<'_>,
    lock_authority: &Pubkey,
    uri: &str,
    metadata_authority: Option<Pubkey>,
    hooked: bool,
) -> Result<()> {
    lock::assert_state(mint, lock_authority, true)?;
    {
        let data = mint.try_borrow_data()?;
        let state = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&data)?;
        require!(
            state.base.mint_authority.is_none()
                && state.base.freeze_authority.is_none()
                && state.base.decimals == TOKEN_DECIMALS
                && state.base.supply == SUPPLY,
            LaunchError::MintProfileInvalid
        );
        let mut types = state.get_extension_types()?;
        types.sort_by_key(|t| *t as u16);
        let mut want = vec![ExtensionType::MetadataPointer, ExtensionType::TokenMetadata, ExtensionType::Pausable];
        if hooked {
            want.push(ExtensionType::TransferHook);
        }
        want.sort_by_key(|t| *t as u16);
        require!(types == want, LaunchError::MintProfileInvalid);

        let pointer = state.get_extension::<MetadataPointer>()?;
        require!(
            Option::<Pubkey>::from(pointer.authority).is_none()
                && Option::<Pubkey>::from(pointer.metadata_address) == Some(mint.key()),
            LaunchError::MintProfileInvalid
        );
    }

    let want_slot = hooked.then_some(lock::HookSlot { authority: Some(*lock_authority), program_id: None });
    require!(lock::hook_slot(mint)? == want_slot, LaunchError::MintProfileInvalid);

    verify_metadata(mint, uri, metadata_authority)?;
    let vault = token_vault.try_borrow_data()?;
    let vault = StateWithExtensions::<spl_token_2022::state::Account>::unpack(&vault)?;
    require!(vault.base.amount == SUPPLY && vault.base.mint == mint.key(), LaunchError::MintProfileInvalid);
    Ok(())
}
