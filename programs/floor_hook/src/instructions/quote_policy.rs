//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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
use anchor_spl::token_interface::Mint;
use floor_types::constants::{MAX_QUOTE_DECIMALS, MIN_QUOTE_DECIMALS};

use crate::constants::{
    AMM_CONFIG_SEED, CARPENTER_AMM_PROGRAM, FLOOR_LAUNCH_PROGRAM, HOOK_CONFIG_SEED, LOCK_AUTHORITY_SEED, PLATFORM_TREASURY_SEED, QUOTE_MINT_SEED,
    QUOTE_POLICY_SEED,
    WSOL_MINT,
};
use crate::errors::HookError;
use crate::events::{QuoteEconomicsSet, QuoteListed, QuoteRetired};
use crate::instructions::register_pool::{amm_lists_quote, base_hook_slot, read_amm_config, BaseHookSlot};
use crate::state::{HookConfig, PlatformTreasury, QuotePolicy};

///
///
///
///
///
///
///
///
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct QuoteTerms {
///
    pub min_bid_floor: u64,
    pub min_bid_ceil: u64,
///
    pub max_bounty_ceil: u64,
///
///
    pub min_bid: u64,
///
    pub max_bounty: u64,
}

///
///
///
///
///
pub fn validate_terms(decimals: u8, t: &QuoteTerms) -> Result<()> {
    require!(
        (MIN_QUOTE_DECIMALS..=MAX_QUOTE_DECIMALS).contains(&decimals),
        HookError::QuoteDecimalsOutOfRange
    );





    require!(t.min_bid_floor > 0 && t.min_bid_floor <= t.min_bid_ceil, HookError::MinBidOutOfRange);




    require!((t.min_bid_floor..=t.min_bid_ceil).contains(&t.min_bid), HookError::MinBidOutOfRange);
    require!(t.max_bounty <= t.max_bounty_ceil, HookError::BountyOutOfRange);
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
fn check_quote_mint_supported(mint: &AccountInfo, admitted: u8) -> Result<u8> {
    let data = mint.try_borrow_data()?;
    let caps = carpenter_mint_gate::check_mint_supported(
        mint.owner,
        &mint.key(),
        &data,
        carpenter_mint_gate::MintRole::ListedQuote,
        carpenter_mint_gate::PausableRule::Renounced,
    )
    .map_err(|_| error!(HookError::QuoteMintNotSupported))?;



    if *mint.owner != carpenter_mint_gate::SPL_TOKEN_ID {
        carpenter_mint_gate::check_admitted(caps, admitted).map_err(|_| error!(HookError::QuoteCapsNotAdmitted))?;
    }
    Ok(caps)
}





#[event_cpi]
#[derive(Accounts)]
pub struct ListQuote<'info> {
///
///
///
///
///
///
    #[account(constraint = governance.key() == platform_treasury.load()?.governance @ HookError::Unauthorized)]
    pub governance: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(seeds = [HOOK_CONFIG_SEED], bump)]
    pub hook_config: AccountLoader<'info, HookConfig>,
///
    #[account(seeds = [PLATFORM_TREASURY_SEED], bump)]
    pub platform_treasury: AccountLoader<'info, PlatformTreasury>,
///
///
///
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init,
        payer = payer,
        space = QuotePolicy::LEN,
        seeds = [QUOTE_POLICY_SEED, quote_mint.key().as_ref()],
        bump,
    )]
    pub quote_policy: AccountLoader<'info, QuotePolicy>,
    pub system_program: Program<'info, System>,
///
///
///
    #[account(seeds = [AMM_CONFIG_SEED], bump, seeds::program = CARPENTER_AMM_PROGRAM)]
    pub amm_config: UncheckedAccount<'info>,
///
///
///
///
///
    #[account(seeds = [QUOTE_MINT_SEED, quote_mint.key().as_ref()], bump, seeds::program = CARPENTER_AMM_PROGRAM)]
    pub amm_quote_mint: UncheckedAccount<'info>,
///
///
///
///
///
    #[account(seeds = [floor_types::seeds::LAUNCH, quote_mint.key().as_ref()], bump, seeds::program = FLOOR_LAUNCH_PROGRAM)]
    pub floor_launch_entry: UncheckedAccount<'info>,
}

pub fn list_quote_handler(ctx: Context<ListQuote>, terms: QuoteTerms) -> Result<()> {
    let mint = &ctx.accounts.quote_mint;














    require_keys_neq!(mint.key(), WSOL_MINT, HookError::QuoteAlreadyListed);

    require!(*ctx.accounts.floor_launch_entry.owner != FLOOR_LAUNCH_PROGRAM, HookError::QuoteIsFloorLaunch);






    let amm_config = ctx.accounts.amm_config.to_account_info();
    require!(
        amm_lists_quote(&amm_config, Some(&ctx.accounts.amm_quote_mint.to_account_info()), &mint.key())?,
        HookError::QuoteNotAmmQuote
    );




    let admitted = read_amm_config(&amm_config)?.map(|c| c.quote_caps_admitted).ok_or(HookError::BadConfig)?;
    check_quote_mint_supported(&mint.to_account_info(), admitted)?;







    {
        let lock_authority =
            Pubkey::find_program_address(&[LOCK_AUTHORITY_SEED], &ctx.accounts.hook_config.load()?.launch_program).0;
        let info = mint.to_account_info();
        let data = info.try_borrow_data()?;
        require!(
            base_hook_slot(info.owner, &data, &lock_authority)? != BaseHookSlot::Floor,
            HookError::QuoteCapsNotAdmitted
        );
    }

    let decimals = mint.decimals;
    validate_terms(decimals, &terms)?;

    {
        let mut q = ctx.accounts.quote_policy.load_init()?;
        q.version = 1;
        q.bump = ctx.bumps.quote_policy;
        q.decimals = decimals;
        q.retired = 0;
        q.quote_mint = mint.key();



        q.token_program = *mint.to_account_info().owner;
        q.min_bid_floor = terms.min_bid_floor;
        q.min_bid_ceil = terms.min_bid_ceil;
        q.max_bounty_ceil = terms.max_bounty_ceil;
        q.min_bid = terms.min_bid;
        q.max_bounty = terms.max_bounty;
    }

    emit_cpi!(QuoteListed {
        quote_mint: mint.key(),
        token_program: *mint.to_account_info().owner,
        decimals,
        min_bid_floor: terms.min_bid_floor,
        min_bid_ceil: terms.min_bid_ceil,
        max_bounty_ceil: terms.max_bounty_ceil,
        min_bid: terms.min_bid,
        max_bounty: terms.max_bounty,
    });
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
///
#[event_cpi]
#[derive(Accounts)]
pub struct SetQuoteEconomics<'info> {
///
    #[account(constraint = governance.key() == platform_treasury.load()?.governance @ HookError::Unauthorized)]
    pub governance: Signer<'info>,
///
    #[account(seeds = [PLATFORM_TREASURY_SEED], bump)]
    pub platform_treasury: AccountLoader<'info, PlatformTreasury>,
    #[account(
        mut,
        seeds = [QUOTE_POLICY_SEED, quote_policy.load()?.quote_mint.as_ref()],
        bump = quote_policy.load()?.bump,
    )]
    pub quote_policy: AccountLoader<'info, QuotePolicy>,
}

pub fn set_quote_economics_handler(ctx: Context<SetQuoteEconomics>, min_bid: u64, max_bounty: u64) -> Result<()> {
    let (mint, old) = {
        let mut q = ctx.accounts.quote_policy.load_mut()?;


        let terms = QuoteTerms {
            min_bid_floor: q.min_bid_floor,
            min_bid_ceil: q.min_bid_ceil,
            max_bounty_ceil: q.max_bounty_ceil,
            min_bid,
            max_bounty,
        };
        validate_terms(q.decimals, &terms)?;
        let old = (q.min_bid, q.max_bounty);
        q.min_bid = min_bid;
        q.max_bounty = max_bounty;
        (q.quote_mint, old)
    };
    emit_cpi!(QuoteEconomicsSet {
        quote_mint: mint,
        old_min_bid: old.0,
        min_bid,
        old_max_bounty: old.1,
        max_bounty,
    });
    Ok(())
}





#[event_cpi]
#[derive(Accounts)]
pub struct RetireQuote<'info> {
///
    #[account(constraint = governance.key() == platform_treasury.load()?.governance @ HookError::Unauthorized)]
    pub governance: Signer<'info>,
///
    #[account(seeds = [PLATFORM_TREASURY_SEED], bump)]
    pub platform_treasury: AccountLoader<'info, PlatformTreasury>,
    #[account(
        mut,
        seeds = [QUOTE_POLICY_SEED, quote_policy.load()?.quote_mint.as_ref()],
        bump = quote_policy.load()?.bump,
    )]
    pub quote_policy: AccountLoader<'info, QuotePolicy>,
}

pub fn retire_quote_handler(ctx: Context<RetireQuote>) -> Result<()> {
    let mint = {
        let mut q = ctx.accounts.quote_policy.load_mut()?;










        require_keys_neq!(q.quote_mint, WSOL_MINT, HookError::BadConfig);


        q.retired = 1;
        q.quote_mint
    };
    emit_cpi!(QuoteRetired { quote_mint: mint });
    Ok(())
}

#[cfg(test)]
mod tests {
//!
    use super::*;
    use crate::instructions::register_pool::{fallback_quote, wsol_terms};
    use anchor_spl::token_2022::spl_token_2022::extension::ExtensionType;
    use anchor_spl::token_2022::spl_token_2022::state::Mint as MintState;
    use carpenter_mint_gate::{
        MintRole, PausableRule, QUOTE_CAPS_CUSTODIAL, QUOTE_CAP_FREEZE, QUOTE_CAP_HOOK_SLOT, QUOTE_CAP_PAUSABLE,
    };
    use floor_types::keys::{SPL_TOKEN_PROGRAM_ID, TOKEN_2022_NATIVE_MINT, TOKEN_2022_PROGRAM_ID};

    fn ok() -> QuoteTerms {
        QuoteTerms {
            min_bid_floor: 1_000_000,
            min_bid_ceil: 5_000_000_000,
            max_bounty_ceil: 500_000_000,
            min_bid: 50_000_000,
            max_bounty: 50_000_000,
        }
    }

///
///
///
///
///
///
///
    #[test]
    fn the_listed_values_sit_inside_the_listed_bounds() {
        let mut t = ok();
        t.min_bid = t.min_bid_floor - 1;
        assert_eq!(validate_terms(9, &t).unwrap_err(), error!(HookError::MinBidOutOfRange));
        t.min_bid = t.min_bid_ceil + 1;
        assert_eq!(validate_terms(9, &t).unwrap_err(), error!(HookError::MinBidOutOfRange));
        t.min_bid = t.min_bid_floor;
        assert!(validate_terms(9, &t).is_ok(), "the floor itself is admissible");
        t.min_bid = t.min_bid_ceil;
        assert!(validate_terms(9, &t).is_ok(), "and so is the ceiling");

        let mut t = ok();
        t.max_bounty = t.max_bounty_ceil + 1;
        assert_eq!(validate_terms(9, &t).unwrap_err(), error!(HookError::BountyOutOfRange));
        t.max_bounty = t.max_bounty_ceil;
        assert!(validate_terms(9, &t).is_ok());
        t.max_bounty = 0;
        assert!(validate_terms(9, &t).is_ok(), "no bounty is a legal value under any ceiling");
    }

///
///
///
///
///
    #[test]
    fn only_a_mint_inside_the_window_may_be_listed() {
        for d in 0..=32u8 {
            let r = validate_terms(d, &ok());
            if (MIN_QUOTE_DECIMALS..=MAX_QUOTE_DECIMALS).contains(&d) {
                assert!(r.is_ok(), "d={d}");
            } else {
                assert_eq!(r.unwrap_err(), error!(HookError::QuoteDecimalsOutOfRange), "d={d}");
            }
        }
    }

///
///
///
///
///
///
    #[test]
    fn a_range_that_admits_nothing_is_refused_at_listing() {
        let mut t = ok();
        t.min_bid_floor = 0;
        assert_eq!(validate_terms(9, &t).unwrap_err(), error!(HookError::MinBidOutOfRange));

        let mut t = ok();
        t.min_bid_floor = 10;
        t.min_bid_ceil = 9;
        assert_eq!(validate_terms(9, &t).unwrap_err(), error!(HookError::MinBidOutOfRange));



        let mut t = ok();
        t.min_bid_floor = 7;
        t.min_bid_ceil = 7;
        t.min_bid = 7;
        assert!(validate_terms(9, &t).is_ok());
    }

///
///
///
///
///
    #[test]
    fn a_zero_bounty_ceiling_is_legal() {
        let mut t = ok();
        t.max_bounty_ceil = 0;
        t.max_bounty = 0;
        assert!(validate_terms(9, &t).is_ok());
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
    #[test]
    fn the_mint_the_retire_refuses_is_the_mint_the_fallback_accepts() {
        assert_eq!(WSOL_MINT, fallback_quote());
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
    #[test]
    fn the_wsol_fallback_is_the_constants_it_replaces() {
        let t = wsol_terms();
        assert_eq!(t.min_bid_floor, floor_types::constants::MIN_BID_LAMPORTS_FLOOR);
        assert_eq!(t.min_bid_ceil, floor_types::constants::MIN_BID_LAMPORTS_CEIL);
        assert_eq!(t.max_bounty_ceil, floor_types::constants::MAX_BOUNTY_LAMPORTS_CEIL);
        assert!(validate_terms(9, &t).is_ok(), "wSOL's own terms are a legal listing at its 9 decimals");
    }





///
///
///
///
///
///
///
///
    fn ext_value_len(e: ExtensionType) -> usize {
        ExtensionType::try_calculate_account_len::<MintState>(&[e]).expect("account len") - 166 - 4
    }

    fn t22_mint(exts: &[ExtensionType], decimals: u8, freeze: bool, pause_authority: Option<[u8; 32]>) -> Vec<u8> {
        let total = 166 + exts.iter().map(|e| 4 + ext_value_len(*e)).sum::<usize>();
        let mut d = vec![0u8; total];
        d[44] = decimals;
        d[45] = 1;
        if freeze {
            d[46..50].copy_from_slice(&1u32.to_le_bytes());
            d[50..82].copy_from_slice(&[9u8; 32]);
        }
        d[165] = 1;
        let mut off = 166;
        for e in exts {
            let vl = ext_value_len(*e);
            d[off..off + 2].copy_from_slice(&u16::from(*e).to_le_bytes());
            d[off + 2..off + 4].copy_from_slice(&(vl as u16).to_le_bytes());
            if *e == ExtensionType::Pausable {
                if let Some(a) = pause_authority {
                    d[off + 4..off + 36].copy_from_slice(&a);
                }
            }
            off += 4 + vl;
        }
        d
    }

    fn check(owner: [u8; 32], key: Pubkey, mut data: Vec<u8>, admitted: u8) -> Result<u8> {
        let mut lamports = 1u64;
        let owner = Pubkey::new_from_array(owner);
        let ai = AccountInfo::new(&key, false, false, &mut lamports, &mut data, &owner, false);
        check_quote_mint_supported(&ai, admitted)
    }

///
///
///
    fn as_base(owner: [u8; 32], key: Pubkey, data: &[u8]) -> core::result::Result<u8, carpenter_mint_gate::GateError> {
        carpenter_mint_gate::check_mint_supported(&Pubkey::new_from_array(owner), &key, data, MintRole::Base, PausableRule::Renounced)
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
    #[test]
    fn the_quote_mints_extensions_are_judged_at_listing() {
        let spl = SPL_TOKEN_PROGRAM_ID;
        let t22 = TOKEN_2022_PROGRAM_ID;
        let k = Pubkey::new_unique();
        let all = QUOTE_CAPS_CUSTODIAL;





        let mut spl_mint = vec![0u8; 82];
        spl_mint[45] = 1;
        assert_eq!(check(spl, k, spl_mint.clone(), 0).unwrap(), 0);
        assert_eq!(check(spl, k, vec![0u8; 82], all).unwrap_err(), error!(HookError::QuoteMintNotSupported), "an uninitialized mint");


        let mut usdc = spl_mint.clone();
        usdc[46..50].copy_from_slice(&1u32.to_le_bytes());
        usdc[50..82].copy_from_slice(&[9u8; 32]);
        assert_eq!(check(spl, k, usdc, 0).unwrap(), QUOTE_CAP_FREEZE, "USDC lists under the default mask, freeze recorded");

        assert_eq!(check([3u8; 32], k, spl_mint, all).unwrap_err(), error!(HookError::QuoteMintNotSupported));

        assert_eq!(
            check(t22, Pubkey::new_from_array(TOKEN_2022_NATIVE_MINT), t22_mint(&[], 9, false, None), all).unwrap_err(),
            error!(HookError::QuoteMintNotSupported)
        );


        assert_eq!(check(t22, k, t22_mint(&[], 6, false, None), 0).unwrap(), 0);
        assert_eq!(check(t22, k, t22_mint(&[ExtensionType::MetadataPointer], 6, false, None), 0).unwrap(), 0);

        assert_eq!(check(t22, k, t22_mint(&[ExtensionType::Pausable], 6, false, None), 0).unwrap(), 0);


        assert_eq!(
            check(t22, k, t22_mint(&[ExtensionType::TransferFeeConfig], 6, false, None), all).unwrap_err(),
            error!(HookError::QuoteMintNotSupported),
            "sent != received"
        );
        assert_eq!(
            check(t22, k, t22_mint(&[ExtensionType::NonTransferable], 6, false, None), all).unwrap_err(),
            error!(HookError::QuoteMintNotSupported),
            "could never leave a vault"
        );

        let mut paused = t22_mint(&[ExtensionType::Pausable], 6, false, Some([7u8; 32]));
        let n = paused.len();
        paused[n - 1] = 1;
        assert_eq!(check(t22, k, paused, all).unwrap_err(), error!(HookError::QuoteMintNotSupported), "paused right now");



        let freeze = t22_mint(&[], 6, true, None);
        assert_eq!(check(t22, k, freeze.clone(), all).unwrap(), QUOTE_CAP_FREEZE, "a freeze authority: admitted, recorded");
        assert_eq!(check(t22, k, freeze.clone(), 0).unwrap_err(), error!(HookError::QuoteCapsNotAdmitted), "…unless the mask withholds it");
        assert_eq!(as_base(t22, k, &freeze), Err(carpenter_mint_gate::GateError::MintNotSupportedAsBase), "…and never as a base mint");

        let pausable = t22_mint(&[ExtensionType::Pausable], 6, false, Some([7u8; 32]));
        assert_eq!(check(t22, k, pausable.clone(), all).unwrap(), QUOTE_CAP_PAUSABLE, "a live pause authority: admitted, recorded");
        assert_eq!(check(t22, k, pausable.clone(), all & !QUOTE_CAP_PAUSABLE).unwrap_err(), error!(HookError::QuoteCapsNotAdmitted));
        assert_eq!(as_base(t22, k, &pausable), Err(carpenter_mint_gate::GateError::MintNotSupportedAsBase));


        let latch = t22_mint(&[ExtensionType::TransferHook], 6, false, None);
        assert_eq!(check(t22, k, latch.clone(), all).unwrap(), QUOTE_CAP_HOOK_SLOT, "the latch: admitted, recorded");
        assert_eq!(check(t22, k, latch.clone(), all & !QUOTE_CAP_HOOK_SLOT).unwrap_err(), error!(HookError::QuoteCapsNotAdmitted));
        assert_eq!(as_base(t22, k, &latch), Err(carpenter_mint_gate::GateError::MintNotSupportedAsBase));

        let mut armed = t22_mint(&[ExtensionType::TransferHook], 6, false, None);
        let n = armed.len();
        armed[n - 32..].copy_from_slice(&[5u8; 32]);
        assert_eq!(check(t22, k, armed, all).unwrap_err(), error!(HookError::QuoteMintNotSupported), "re-entry");
    }


///
///
///
///
///
///
    #[test]
    fn the_wsol_range_means_something_else_at_six_decimals() {
        let w = wsol_terms();

        assert_eq!(floor_types::constants::MIN_BID_LAMPORTS / 1_000_000, 50, "0.05 SOL reads as $50 of USDC");

        let usdc = QuoteTerms {
            min_bid_floor: 1_000,
            min_bid_ceil: 5_000_000,
            max_bounty_ceil: 500_000,
            min_bid: 50_000,
            max_bounty: 50_000,
        };
        assert!(validate_terms(6, &usdc).is_ok());

        assert!(!(w.min_bid_floor..=w.min_bid_ceil).contains(&usdc.min_bid_floor));
    }
}
