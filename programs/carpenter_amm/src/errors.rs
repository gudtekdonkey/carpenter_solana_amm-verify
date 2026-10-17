//!
//!
//!
//!
//!
//!
//!
//!

use anchor_lang::prelude::*;

#[error_code]
pub enum AmmError {



    #[msg("signer is not the required authority")]
    Unauthorized,


    #[msg("hook program is not an executable loader-v3 program")]
    HookNotExecutable,
    #[msg("hook flags fail v4's isValidHookAddress rules or the Solana additions")]
    HookFlagsInvalid,
    #[msg("max_rounds exceeds MAX_ROUNDS_CAP")]
    RoundsTooHigh,
    #[msg("approve_hook args disagree with the hook's interface header")]
    HookHeaderMismatch,
    #[msg("hook is not approved")]
    HookNotApproved,
    #[msg("hook is paused for new pools")]
    HookPausedForNewPools,
    #[msg("hook ProgramData slot differs from the approval: re-approve")]
    HookUpgradedReapprove,


    #[msg("mint0 must sort below mint1")]
    CurrenciesOutOfOrder,
    #[msg("tick spacing out of range")]
    TickSpacingInvalid,
    #[msg("fee out of range, or the fee/hook combination is invalid")]
    FeeInvalid,
    #[msg("mint is not supported (D-19)")]
    MintNotSupported,
    #[msg("mint is exclusive to another hook (D-20)")]
    MintExclusive,
    #[msg("account does not match its PDA seeds")]
    SeedsMismatch,


    #[msg("tick array start index is not aligned or out of bounds")]
    TickArrayStartInvalid,
    #[msg("tick is not aligned to the spacing or is out of bounds")]
    TickInvalid,
    #[msg("a position's range may change only while its liquidity is zero")]
    PositionRangeLocked,
    #[msg("slippage limit exceeded")]
    SlippageExceeded,
    #[msg("insufficient liquidity")]
    InsufficientLiquidity,
    #[msg("position still holds liquidity")]
    PositionNotEmpty,


    #[msg("hook return data is missing, from the wrong program, undecodable or not an echo (O-1)")]
    HookBadReturn,
    #[msg("hook record sets a field its phase or flags do not allow")]
    HookFieldNotAllowed,
    #[msg("hook delta flips the specified amount (O-4)")]
    HookDeltaExceedsSwapAmount,
    #[msg("hook delta flips the trader's side (O-4)")]
    HookDeltaExceedsSwapOutput,
    #[msg("hook action needs an account the transaction did not supply (O-6)")]
    HookActionAccountMissing,
    #[msg("hook claims do not cover the debit (O-5)")]
    HookClaimsInsufficient,
    #[msg("hook asked for a callback beyond max_rounds (O-8)")]
    HookRoundsExceeded,
    #[msg("negative hook deltas are not supported in v1")]
    HookDeltaNegative,
    #[msg("hook slice contains one of the AMM's own accounts (§5.2)")]
    HookSliceAliasesAmmAccount,


    #[msg("a needed tick array was not supplied")]
    TickArrayMissing,
    #[msg("price limit is on the wrong side of the current price")]
    PriceLimitInvalid,
    #[msg("amount is zero")]
    AmountZero,
    #[msg("a fee of 100% cannot quote exact output (R-12)")]
    InvalidFeeForExactOut,


    #[msg("pool liquidity is below MIN_DONATE_LIQUIDITY")]
    NoLiquidity,
    #[msg("donation exceeds u64::MAX / 2")]
    DonateTooLarge,


    #[msg("pool has no hook")]
    NoHook,
    #[msg("currency must be 0 or 1")]
    CurrencyInvalid,
    #[msg("vault does not belong to the pool for that currency")]
    VaultMismatch,
    #[msg("mint does not belong to the pool for that currency")]
    MintMismatch,
    #[msg("token program does not belong to the pool for that currency")]
    TokenProgramMismatch,
    #[msg("signer is not the hook authority")]
    NotHookAuthority,
    #[msg("hook is not allowed to register mint exclusivity")]
    ExclusivityNotAllowed,
    #[msg("a quote mint cannot be made exclusive")]
    ExclusivityQuoteMint,
    #[msg("mint already has a pool")]
    MintAlreadyPooled,
    #[msg("pool fee is not dynamic")]
    NotDynamicFee,
    #[msg("token account has CPI Guard and no sufficient delegation to the pool (R-15)")]
    DelegateMissing,


    #[msg("vault balance does not equal its booked movement, or claims exceed it (O-10) (WP-0)")]
    VaultConservationViolated,
    #[msg("duplicate key in the tick-array set (§5.6) (WP-0)")]
    DuplicateTickArray,
    #[msg("arithmetic overflow (WP-0)")]
    MathOverflow,
    #[msg("hook slice length is not pool.hook_callback_accounts (§5.2) (WP-0)")]
    HookSliceLengthMismatch,
    #[msg("hook_data exceeds MAX_HOOK_DATA (§5.3) (WP-0)")]
    HookDataTooLong,


    #[msg("creating a position needs its position_id to sign")]
    PositionIdNotSigned,
    #[msg("a hook's own position cannot be transferred")]
    PositionNotTransferable,
    #[msg("new owner is the zero address or the current owner")]
    PositionTransferInvalid,

    #[msg("hook fee split claims more than its delta, or a platform cut above the skim (O-4, 2026-09-17)")]
    HookFeesInconsistent,


    #[msg("pool is paused: swaps, adds, donations and deposits are refused; removals and withdrawals are not")]
    PoolPaused,








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
    #[msg("vault holds less than the claims booked against it (O-10 bound; P2 2026-09-18)")]
    VaultShortfall,


    #[msg("quote roster entry is absent, or present and not Listed")]
    QuoteMintNotListed,
    #[msg("quote mint is retired: no new pool may open with it in the quote slot")]
    QuoteMintRetired,
    #[msg("quote mint is already on the roster")]
    QuoteMintAlreadyListed,
    #[msg("quote mint still has pools: retire it instead of delisting it")]
    QuoteMintPooled,
    #[msg("wSOL is the implied quote and cannot be rostered")]
    QuoteMintImplied,
///
///
///
    #[msg("quote roster entry is not retired: only a retired entry can be relisted")]
    QuoteMintNotRetired,


    #[msg("vault was created frozen (DefaultAccountState); no pool is opened")]
    VaultFrozenAtCreation,


    #[msg("mint's transfer hook has been ARMED since the pool opened; the pool is halted")]
    QuoteHookArmed,
    #[msg("mint is paused by its issuer; no transfer can settle")]
    QuotePaused,
    #[msg("mint carries a live transfer fee; sent would not equal received")]
    QuoteFeeArmed,


///
///
///
///
    #[msg("mint carries a custodial capability and may only be a LISTED QUOTE, never a base mint (D-19 role split)")]
    MintNotSupportedAsBase,
///
///
///
    #[msg("a capability of this quote mint is not admitted by AmmConfig.quote_caps_admitted (set_quote_caps_admitted)")]
    QuoteCapsNotAdmitted,
///
///
    #[msg("the admission mask may name only the Tier-B (custodial) capability bits")]
    QuoteCapsInvalid,





///
///
    #[msg("trader_token_in holds no more than keep: nothing to spend (swap_from_balance)")]
    BalanceAtOrBelowKeep,
///
///
    #[msg("trader_token_in is not owned by the trader: swap_from_balance spends only the signer's own account")]
    BalanceSwapNotOwner,
///
    #[msg("a hook-authority swap settles against claims and has no balance to spend")]
    BalanceSwapSelfCall,
///
///
    #[msg("swap_from_balance did not spend its whole resolved input (price limit or liquidity reached)")]
    BalanceSwapPartialFill,






///
///
///
    #[msg("mint's armed transfer hook is not the program this pool admits on its base side, or its authority is live")]
    TransferHookNotAdmitted,
///
///
///
    #[msg("ApprovedHook.base_transfer_hook is already set; it is set once")]
    BaseTransferHookAlreadySet,
///
    #[msg("not a transfer-hook program the AMM can admit (zero, the AMM, or a token program)")]
    BadBaseTransferHook,
///
///
///
    #[msg("a transfer of a hooked mint is missing the transfer hook's accounts (its extra-account-metas list, the program, or an entry)")]
    TransferHookAccountsMissing,



///
    #[msg("tick array still counts an initialized tick")]
    TickArrayNotEmpty,
///
    #[msg("tick array's ticks are not all zero bytes")]
    TickArrayNotZeroed,
///
    #[msg("the pool's bitmap still marks this tick array")]
    TickArrayBitmapBitSet,









///
///
///
    #[msg("protocol fee above MAX_PROTOCOL_FEE (1,000 pips = 0.1% per direction)")]
    ProtocolFeeTooLarge,
///
///
///
    #[msg("destination is not the protocol treasury's token account for this mint (sweep), or is the treasury's own (withdraw)")]
    WrongDestination,
///
    #[msg("source is not a token account the protocol treasury owns")]
    NotTreasuryAccount,
///
    #[msg("withdraw exceeds the treasury account's balance")]
    TreasuryBalanceShort,
///
///
    #[msg("not a key the protocol treasury can be governed by (zero, or the treasury itself)")]
    BadTreasuryGovernance,


///
///
    #[msg("a guardian pause is already running here; the guardian cannot extend it")]
    GuardianPauseRunning,
///
///
    #[msg("inside the guardian's cool-off after its last pause of this pool / hook")]
    GuardianCoolOff,

///
///
    #[msg("this approved hook may not list quote mints (ApprovedHook.allow_hook_quotes is 0)")]
    HookQuotesNotAllowed,


///
///
    #[msg("a settling hook's slice must start with the [\"hook_settle\", pool] PDA")]
    HookSettleSignerMissing,
///
///
    #[msg("a settling hook lowered a vault balance")]
    HookSettleVaultDecreased,
///
///
    #[msg("a settling hook took more than the unfilled remainder of an exact-in")]
    HookSettleOverfill,
}
