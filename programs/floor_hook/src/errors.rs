//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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

#[error_code(offset = 7000)]
pub enum HookError {

    #[msg("signer is not the required authority")]
    Unauthorized,
    #[msg("register_pool configuration is invalid")]
    BadConfig,
    #[msg("signer is not the launch authority")]
    NotLauncher,
    #[msg("mint already has a pool")]
    MintAlreadyPooled,
    #[msg("callback signer is not this FloorPool's AMM pool (D-14)")]
    NotAmmPool,
    #[msg("pool initializer is not the registered Launch")]
    NotInitializer,
    #[msg("FloorPool is not bound to a pool")]
    NotBound,
    #[msg("pool account is not this FloorPool's AMM pool (§6.3)")]
    WrongPool,
    #[msg("FLOOR pools require the dynamic fee flag")]
    NotDynamicFee,
    #[msg("FLOOR pools refuse exact-output swaps in this build (R-13, DV-8)")]
    ExactOutNotSupported,
    #[msg("the seed range is fixed by the first seed_liquidity")]
    SeedRangeFixed,
    #[msg("range is invalid or yields zero liquidity")]
    BadRange,
    #[msg("the ladder is already seeded")]
    LadderAlreadySeeded,
    #[msg("fee recipient is pending its claim")]
    RecipientPendingClaim,
    #[msg("destination token account is not the configured one")]
    WrongDestination,
    #[msg("recipient already resolved")]
    AlreadyResolved,


    #[msg("a §6.8 invariant failed in an epilogue (WP-0)")]
    InvariantViolated,
    #[msg("the AMM returned no or malformed HookOpResultV1 (WP-0)")]
    AmmReturnMissing,
    #[msg("arithmetic overflow (WP-0)")]
    MathOverflow,


///
///
///
    #[msg("min_bid is outside this quote's hard range (ruling A)")]
    MinBidOutOfRange,
    #[msg("the ratchet bounty rate or ceiling is outside its hard range (ruling D)")]
    BountyOutOfRange,


    #[msg("the pool is not open for trade until graduation has seeded it (Q-11)")]
    NotSeeded,





    #[msg("the quote mint is not on the FLOOR roster (list_quote it first)")]
    QuoteNotListed,
///
///
///
///
///
///
///
///
    #[msg("the quote mint is already on the FLOOR roster")]
    QuoteAlreadyListed,
    #[msg("the quote mint is retired: no new pool may be registered against it")]
    QuoteRetired,
///
///
///
///
///
///
    #[msg("the quote mint decimals are outside the admissible window (no launch could ever price on it)")]
    QuoteDecimalsOutOfRange,
    #[msg("carpenter_amm does not list this mint as a quote mint")]
    QuoteNotAmmQuote,


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
    #[msg("nothing is owed: pay_holder was asked to pay zero")]
    NothingOwed,





///
///
///
    #[msg("the quote mint is refused by the D-19 gate in the quote role (carpenter-mint-gate)")]
    QuoteMintNotSupported,
///
///
    #[msg("a capability of this quote mint is not admitted by carpenter_amm's quote_caps_admitted mask")]
    QuoteCapsNotAdmitted,


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
    #[msg("the quote is suspended (its vault is frozen or its mint is paused): observe_quote must read it clear first")]
    QuoteSuspended,








///
///
///
///
    #[msg("the platform cut is paid only to the program-owned treasury")]
    TreasuryNotMigrated,
///
///
///
    #[msg("the treasury cannot be its own governance: it can never sign a withdrawal")]
    TreasuryIsNotGovernance,
///
///
///
    #[msg("the treasury's governance may not be the launch program's attestor: one key, one role")]
    TreasuryGovernanceIsAttestor,

///
///
    #[msg("a withdrawal moves a non-zero amount out of the treasury, never back into it")]
    BadWithdrawal,
///
///
    #[msg("the withdrawal source is not a token account the platform treasury owns")]
    NotTreasuryAccount,
///
///
///
///
    #[msg("the withdrawal is larger than the treasury account's balance")]
    TreasuryBalanceShort,





///
///
///
///
    #[msg("pay_holder must be signed by floor_holders' holders_authority PDA")]
    NotHoldersAuthority,
///
///
///
    #[msg("pay_holder: the amount exceeds what this pool owes its holders")]
    HolderPayExceedsOwed,

///
///
///
///
///
    #[msg("a holder rate needs FLOOR's empty transfer-hook slot on the base mint, and the slot needs a holder rate")]
    HolderRateNeedsHook,
///
///
///
///
    #[msg("a referrer is allowed only when the fee recipient is the deployer's own wallet, and never the referrer itself")]
    ReferrerNotAllowed,




///
///
    #[msg("claim_referral_fees: the tier must be 1, 2 or 3")]
    BadReferralTier,





///
///
///
///
    #[msg("the pool's FLOOR Platform fee is above its cap (TEAM_POOL_FEE_BPS_MAX)")]
    TeamFeeOutOfRange,


///
///
    #[msg("twap_tick: this pool has no price oracle yet (no tail, or never written)")]
    TwapUnavailable,
///
///
    #[msg("twap_tick: the window is longer than the oracle's oldest checkpoint")]
    TwapWindowTooLong,


///
///
///
///
    #[msg("only a wSOL pool's platform cut feeds $FLOOR, and $FLOOR's pool is wSOL-quoted")]
    BadQuoteToken,
///
///
///
    #[msg("$FLOOR's pool is not live yet: the wSOL cut stays owed on its pool until it graduates")]
    FloorPoolNotLive,
///
///
///
    #[msg("$FLOOR's pool: designation needs a fresh unbound pool, and it never credits itself")]
    FloorPoolNotEligible,


///
///
///
    #[msg("register_pool: the at-floor sell multiple is outside 10,000..=50,000 bps (1x..=5x)")]
    SkimSellMultipleOutOfRange,
///
///
    #[msg("register_pool: the at-floor sell skim (far x multiple) is above 1,500 bps (15 %)")]
    SkimSellAtFloorAboveCap,


///
///
///
    #[msg("register_pool: the staircase is outside steps 1..=16, growth 5,000..=20,000 bps")]
    StairShapeOutOfRange,

///
///
///
    #[msg("the staircase's StairState account is missing or wrong")]
    StairStateMissing,
///
///
    #[msg("the staircase: a step would not be token-only at the market")]
    StairNotTokenOnly,
///
///
    #[msg("set_genesis_stair: only $FLOOR's designated pool, once, before it is bound or seeded")]
    GenesisStairRefused,


///
///
///
    #[msg("the owner steps: the config is outside the rulings, or already written")]
    StairOwnerBadConfig,
///
    #[msg("the owner steps: the payee's handle is not claimed yet")]
    StairPayeePending,
///
///
    #[msg("the owner steps: only the payee, to its own account")]
    StairWrongPayee,
///
///
    #[msg("the owner steps: this step is still locked")]
    StairStepLocked,
///
///
    #[msg("the owner steps: only a step wholly above the price, after the ratchet")]
    StairStepNotAbove,
///
    #[msg("the owner steps: the payee is already resolved")]
    StairPayeeResolved,


///
///
    #[msg("the hook's governance can never be the attestor")]
    HookGovernanceIsAttestor,
///
///
    #[msg("the hook's governance must be a key that can sign")]
    HookGovernanceCannotSign,


///
    #[msg("the staircase rent reserve: that tick array is next to the price")]
    StairArrayHot,


///
///
///
    #[msg("the owner steps: the books claim more than the position holds")]
    OwnerBooksExceedHoldings,


///
///
///
    #[msg("the quote is not admitted: no QuotePolicy, no qualifying FLOOR parent, and allow-all is off")]
    QuoteNotAdmitted,
///
///
    #[msg("a derived quote's terms are the hook's own: min_bid = 0.001 whole token rounded up, max_bounty = 0")]
    DerivedQuoteTerms,


///
///
    #[msg("a FLOOR launch's token is never a listed quote: it is admitted by lineage and priced through its pools")]
    QuoteIsFloorLaunch,


///
///
    #[msg("trader referrals: the trader has no TraderLink (link first)")]
    TraderNotLinked,
///
    #[msg("trader referrals: that bucket is already open")]
    TraderBucketOpen,
///
///
    #[msg("trader referrals: an account is not the bucket, vault, payee or TraderBucket it must be")]
    TraderRefBadAccount,

///
///
///
    #[msg("bad recipient slots: up to 5, each an address or a pending handle id, shares summing to 10,000")]
    BadRecipients,
///
    #[msg("no such slot")]
    BadSlot,
}
