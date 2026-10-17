//!
//!
//!
//!
//!
//!
//!

use anchor_lang::prelude::*;

#[error_code]
pub enum LaunchError {

    #[msg("signer is not the required authority")]
    Unauthorized,


    #[msg("launch configuration is invalid")]
    BadConfig,
///
///
    #[msg("mint must sort above its quote mint (D-05)")]
    MintBelowQuote,
    #[msg("fee recipient / pending recipient configuration is invalid")]
    BadRecipientConfig,
///
///
///
///
    #[msg("registry_count changed; re-read and re-sign")]
    RegistryIndexRace,


    #[msg("the purchase buys nothing")]
    Nothing,
    #[msg("fewer tokens than min_tokens")]
    Slippage,
    #[msg("graduation would cost the final buyer more than max_graduation_lamports")]
    GraduationCostExceeded,
    #[msg("the sale is over")]
    SaleOver,
    #[msg("sell-back exceeds the allocation")]
    InsufficientAllocation,
    #[msg("CPI Guard account has no sufficient delegation to the Launch")]
    DelegateMissing,
    #[msg("sale vault holds less than it must")]
    EscrowShort,


    #[msg("the sale cannot graduate in this state")]
    CannotGraduate,


    #[msg("the launch has no pending recipient")]
    NotPending,
    #[msg("the attestation has expired")]
    Expired,
    #[msg("the Ed25519 attestation is malformed or not from the attestor")]
    BadAttestation,


    #[msg("buyer position still holds an allocation during the sale (WP-0)")]
    PositionNotClosable,
    #[msg("arithmetic overflow (WP-0)")]
    MathOverflow,




    #[msg("the mint's Pausable lock is not in the state the launch requires (WP-L1)")]
    LockStateInvalid,
    #[msg("the Token-2022 mint profile is not the §7.1 profile (WP-L1)")]
    MintProfileInvalid,
    #[msg("an account the program creates already holds data or belongs to another program (WP-L1)")]
    AccountInUse,
    #[msg("a token account is not the associated token account it must be (WP-L1)")]
    NotAssociatedTokenAccount,



    #[msg("the metadata base is not an absolute ASCII URL within the length bound")]
    MetadataBaseInvalid,
    #[msg("the launch config has no metadata base set")]
    MetadataBaseUnset,
    #[msg("this mint's metadata uri is already final")]
    MetadataUriFinal,
    #[msg("the metadata base is too fresh to fill from: governance's correction window is open")]
    MetadataBaseTooFresh,








    #[msg("min_bid_lamports is outside the hard range 0.001 .. 5 SOL (ruling A)")]
    MinBidOutOfRange,
    #[msg("the ratchet bounty rate or ceiling is outside its hard range (ruling D)")]
    BountyOutOfRange,
    #[msg("the graduation rent reserve holds less than graduation needs (ruling B)")]
    RentReserveShort,
    #[msg("the graduation rent reserve may only be closed by the launch creator (ruling B)")]
    NotLaunchCreator,


    #[msg("the prepared pool is no longer at the launch's open tick (Q-11)")]
    PoolMoved,
    #[msg("the pool this launch graduates into does not exist yet (Q-11)")]
    PoolNotPrepared,





///
///
///
///
///
///
    #[msg("the quote mint is not on the FLOOR roster: pass its QuotePolicy (list_quote it first)")]
    QuoteNotListed,
///
    #[msg("the quote mint is retired: no new launch may be created on it")]
    QuoteRetired,


///
///
///
    #[msg("the attestor may not be the current governance: one key, one role")]
    AttestorIsGovernance,
///
///
///
    #[msg("governance may not be the current attestor: one key, one role")]
    GovernanceIsAttestor,


///
///
///
///
///
    #[msg("the attestor may not be the platform treasury's governance: one key, one role")]
    AttestorIsTreasuryGovernance,





///
///
///
///
///
///
///
///
///
    #[msg("a referrer is allowed only when the fee recipient is the signing creator, with no pending handle, and not the creator itself")]
    ReferrerNotAllowed,
///
///
///
///
///
    #[msg("the referrer must be a wallet (an on-curve key), not a program address")]
    ReferrerOffCurve,
///
///
///
    #[msg("graduation did not arm the holders hook and renounce its authority (step 8b)")]
    HookNotArmed,
///
///
///
    #[msg("sell_back on a hooked launch needs [floor_holders, holder_book, floor_pool, holder_record] for this seller")]
    HolderRecordMismatch,








///
///
///
    #[msg("a FLOOR Platform fee setting is above its cap (sale 100 bps, pool 50 bps)")]
    TeamFeeOutOfRange,
///
///
///
    #[msg("the team treasury is not open: there is no team governance to set the FLOOR Platform fee")]
    TeamTreasuryNotOpen,
///
///
///
///
///
///
///
    #[msg("the attestor may not be the team treasury's governance: one key, one role")]
    AttestorIsTeamTreasuryGovernance,




///
    #[msg("the genesis raise has no schedule")]
    GenesisNotScheduled,
///
    #[msg("the genesis raise has opened: its schedule and floor share are frozen")]
    GenesisScheduleFrozen,
///
///
    #[msg("the genesis schedule is invalid")]
    GenesisInvalidSchedule,
///
///
    #[msg("the genesis raise is not open")]
    GenesisSaleNotActive,
///
    #[msg("the genesis raise has not ended")]
    GenesisSaleNotEnded,
///
///
    #[msg("the genesis launch has not graduated")]
    GenesisNotGraduated,
///
    #[msg("a contribution must be more than zero")]
    GenesisZeroContribution,
///
///
    #[msg("the genesis configuration is invalid")]
    GenesisBadConfig,
///
    #[msg("a contributor has not claimed yet: the allocation dust is not final")]
    GenesisAllocationsOutstanding,
///
///
///
    #[msg("that genesis step is out of order")]
    GenesisStepOutOfOrder,
///
///
    #[msg("only the genesis authority may do that")]
    NotGenesisAuthority,
///
///
///
    #[msg("the genesis reserve is locked until the sale has graduated and made its locks")]
    GenesisRentLocked,

///
///
///
///
    #[msg("the at-floor sell multiple must be 0 (1.5x) or 10,000..=50,000 bps (1x..=5x)")]
    SkimSellMultipleOutOfRange,
///
///
///
    #[msg("the at-floor sell skim (far sell x multiple) must be no higher than 1,500 bps (15 %)")]
    SkimSellAtFloorAboveCap,
///
///
///
///
    #[msg("the hook did not record the staircase (upgrade floor_hook first)")]
    StairDropped,
///
///
///
///
    #[msg("the staircase does not fit between the open price and the top of the tick range")]
    StairDoesNotFit,
///
///
///
    #[msg("the $FLOOR staircase's top market cap (quote units) must be set, and non-zero, before the raise opens")]
    GenesisStairTopUnset,
///
///
///
    #[msg("the team buy must be at most 5% of supply with at least 80% on team steps, a lock of 90..=1825 days")]
    TeamBuyOutOfRange,
///
///
    #[msg("a team buy needs a staircase preset (team steps live on the staircase)")]
    TeamBuyNeedsStaircase,
///
///
    #[msg("the hook did not record the team steps (upgrade floor_hook first)")]
    StairOwnerDropped,
///
    #[msg("nothing of the deployer's wallet part has vested since the last claim")]
    TeamNothingVested,
///
    #[msg("the steps' payee is not a pending handle")]
    PayeeNotPending,
///
///
    #[msg("the team steps would be more than 25 % of the staircase")]
    TeamStepsOverCap,
///
    #[msg("this launch's staircase rent budget is already moved, or it has none")]
    StairRentFunded,
///
    #[msg("the stair_rent account is not this launch's pool reserve")]
    WrongStairRent,
///
///
///
    #[msg("the graduation rent reserve is locked until the launch graduates (9A)")]
    GraduationRentLocked,
///
///
    #[msg("the attestor may not be the AMM's pause guardian")]
    AttestorIsPauseGuardian,
///
///
    #[msg("the genesis whitelist root is not set (set_genesis_whitelist, non-zero, before the open)")]
    GenesisWhitelistUnset,
///
///
    #[msg("this wallet is not on the genesis whitelist (or the proof is wrong)")]
    GenesisNotWhitelisted,
///
///
///
    #[msg("a wallet may contribute at most 20 SOL in the Core phase and 5 SOL in phase 2")]
    GenesisWalletCapExceeded,
///
///
    #[msg("the genesis schedule carries Core's 300 seconds; phase 2 runs until the authority closes it")]
    GenesisDurationNotFixed,
///
///
///
    #[msg("only the genesis authority may close the raise and graduate it")]
    GenesisGraduationDeployerOnly,
///
///
    #[msg("nothing of the raise allocation has vested yet (14-day cliff from the sale end)")]
    GenesisNothingVested,
///
///
///
    #[msg("the genesis raise is not in that phase (Core: the first 300 s; F&F: from then until the authority closes it)")]
    GenesisWrongPhase,
///
///
///
    #[msg("this genesis phase is sold out (Core 20M tokens / 80 SOL; phase 2 22.5M / 90 SOL)")]
    GenesisPhaseSoldOut,
///
///
///
    #[msg("the genesis refund opens 14 days after the authority closes the raise, and only if it has not graduated")]
    GenesisRefundNotOpen,
///
///
    #[msg("this wallet has nothing to refund (never contributed, or already refunded)")]
    GenesisNothingToRefund,
///
///
///
    #[msg("the genesis refund window is open: the raise can no longer graduate (or this wallet was refunded)")]
    GenesisRefundWindowOpen,
///
///
///
///
    #[msg("the quote chain's hop is not a graduated preset (or $FLOOR) pool of the current quote")]
    QuoteChainBadHop,
///
///
///
    #[msg("a parent pool's TWAP does not yet reach 30 minutes: the child cannot be priced off it")]
    ParentTwapTooYoung,
///
///
///
    #[msg("the quote chain was cut short: its root is a graduated FLOOR launch token")]
    QuoteChainCut,
///
///
///
    #[msg("the quote walk is stale (over 60 s since its first step): restart it")]
    QuoteWalkStale,
///
///
    #[msg("the quote walk has not reached its root (or is already finished)")]
    QuoteWalkIncomplete,
///
///
    #[msg("the quote walk does not belong to this payer and quote")]
    QuoteWalkMismatch,
///
///
///
    #[msg("no preset is registered under this id")]
    PresetUnknown,
///
///
    #[msg("this preset is disabled for new launches")]
    PresetDisabled,
///
    #[msg("the preset registry is full (ids up to 31)")]
    PresetTableFull,
///
///
///
    #[msg("the preset row breaks the rule table (the field is logged)")]
    PresetInvalid,
///
///
///
    #[msg("the trader link's tiers are not well formed (tier 1 set, no gaps, no self, no repeats)")]
    BadTraderLinkShape,
///
    #[msg("this trader is already linked")]
    AlreadyLinked,
///
///
    #[msg("a trader referral tier is not a wallet (off the ed25519 curve)")]
    TraderTierOffCurve,

///
///
///
    #[msg("the trader-referral accounts do not belong to this trade (bucket, sale account or hook accounts)")]
    TraderRefMismatch,
///
///
///
    #[msg("the launch's sale referral account is already open")]
    SaleReferralAlreadyOpen,
///
///
///
///
///
    #[msg("bad creator recipients (the check code is in the log)")]
    BadRecipients,
///
///
///
    #[msg("$FLOOR's genesis payee is claimed through claim_genesis_payee")]
    GenesisPayeeUseGenesisClaim,

///
///
    #[msg("the wSOL payout account is not the seller's native SPL Token wSOL account")]
    BadWsolDestination,
///
    #[msg("a wSOL payout exists only on a launch quoted in wSOL")]
    WsolPayoutNotNative,
}
