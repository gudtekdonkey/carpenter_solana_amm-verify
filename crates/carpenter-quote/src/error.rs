//!
//!
//!
//!
//!
//!
//!

use core::fmt;

use carpenter_clmm_math::MathError;
use floor_types::hook::RecordError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuoteError {

///
    QuoteBadAccount(&'static str),
///
    QuoteBadBitmap,
///
    QuoteBadTickArray,
///
///
    QuoteHookNotQuotable,
///
    QuoteHookStateMissing,
///
///
    QuoteActionNotSupported,
///
///
///
///
///
    QuoteReverseOrientation,


///
    AmountZero,
///
///
///
///
///
///
///
///
///
    AmountTooLarge,
///
    PriceLimitInvalid,
///
    InvalidFeeForExactOut,
///
///
    TickArrayMissing { start_index: i32 },
///
    HookActionAccountMissing { start_index: i32 },
///
    HookDeltaExceedsSwapAmount,
///
    HookDeltaExceedsSwapOutput,
///
    HookDeltaNegative,
///
    HookClaimsInsufficient,
///
    HookRoundsExceeded,
///
    HookFieldNotAllowed,
///
    FeeInvalid,
///
    HookFeesInconsistent,
///
    AmountOverflow,


///
    ExactOutNotSupported,
///
    NotBound,
///
    WrongPool,
///
///
///
///
    NotSeeded,
///
    NotDynamicFee,
///
    InvariantViolated,


///
    SaleOver,
///
    Nothing,
///
    InsufficientAllocation,


///
///
    WrongMint,
///
    NotHooked,


///
    Math(MathError),
///
    Overflow,
}

impl QuoteError {
///
    pub fn name(&self) -> &'static str {
        use QuoteError::*;
        match self {
            QuoteBadAccount(_) => "QuoteBadAccount",
            QuoteBadBitmap => "QuoteBadBitmap",
            QuoteBadTickArray => "QuoteBadTickArray",
            QuoteHookNotQuotable => "QuoteHookNotQuotable",
            QuoteHookStateMissing => "QuoteHookStateMissing",
            QuoteActionNotSupported => "QuoteActionNotSupported",
            QuoteReverseOrientation => "QuoteReverseOrientation",
            AmountZero => "AmountZero",

            AmountTooLarge => "MathOverflow",
            PriceLimitInvalid => "PriceLimitInvalid",
            InvalidFeeForExactOut => "InvalidFeeForExactOut",
            TickArrayMissing { .. } => "TickArrayMissing",
            HookActionAccountMissing { .. } => "HookActionAccountMissing",
            HookDeltaExceedsSwapAmount => "HookDeltaExceedsSwapAmount",
            HookDeltaExceedsSwapOutput => "HookDeltaExceedsSwapOutput",
            HookDeltaNegative => "HookDeltaNegative",
            HookClaimsInsufficient => "HookClaimsInsufficient",
            HookRoundsExceeded => "HookRoundsExceeded",
            HookFieldNotAllowed => "HookFieldNotAllowed",
            FeeInvalid => "FeeInvalid",
            HookFeesInconsistent => "HookFeesInconsistent",

            AmountOverflow => "MathOverflow",
            ExactOutNotSupported => "ExactOutNotSupported",
            NotBound => "NotBound",
            WrongPool => "WrongPool",
            NotSeeded => "NotSeeded",
            NotDynamicFee => "NotDynamicFee",
            InvariantViolated => "InvariantViolated",
            SaleOver => "SaleOver",
            Nothing => "Nothing",
            InsufficientAllocation => "InsufficientAllocation",
            WrongMint => "WrongMint",
            NotHooked => "NotHooked",
            Math(e) => math_name(*e),
            Overflow => "MathOverflow",
        }
    }
}

fn math_name(e: MathError) -> &'static str {
    match e {
        MathError::TickOutOfRange => "TickOutOfRange",
        MathError::SqrtPriceOutOfRange => "SqrtPriceOutOfRange",
        MathError::CalculateOverflow => "CalculateOverflow",
        MathError::ZeroLiquidity => "ZeroLiquidity",
        MathError::ZeroSqrtPrice => "ZeroSqrtPrice",
        MathError::MaxTokenOverflow => "MaxTokenOverflow",
        MathError::LiquidityUnderflow => "LiquidityUnderflow",
        MathError::LiquidityOverflow => "LiquidityOverflow",
        MathError::PriceTargetOvershoot => "PriceTargetOvershoot",
        MathError::InvalidTickArrayBoundary => "InvalidTickArrayBoundary",
        MathError::FeeRateTooHigh => "FeeRateTooHigh",
        MathError::InvalidFeeForExactOut => "InvalidFeeForExactOut",
        MathError::InvalidTickSpacing => "InvalidTickSpacing",
        MathError::BitmapLengthMismatch => "BitmapLengthMismatch",
        MathError::DirectionMismatch => "DirectionMismatch",
    }
}

impl From<MathError> for QuoteError {
    fn from(e: MathError) -> Self {
        match e {

            MathError::InvalidFeeForExactOut => QuoteError::InvalidFeeForExactOut,
            other => QuoteError::Math(other),
        }
    }
}

impl From<RecordError> for QuoteError {
    fn from(e: RecordError) -> Self {
        match e {
            RecordError::BadReturn | RecordError::FieldNotAllowed => QuoteError::HookFieldNotAllowed,
            RecordError::FeeTooLarge => QuoteError::FeeInvalid,
            RecordError::RoundsExceeded => QuoteError::HookRoundsExceeded,
            RecordError::DeltaExceedsSwapAmount => QuoteError::HookDeltaExceedsSwapAmount,
            RecordError::Overflow => QuoteError::Overflow,
            RecordError::FeesInconsistent => QuoteError::HookFeesInconsistent,
        }
    }
}

impl fmt::Display for QuoteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuoteError::QuoteBadAccount(which) => write!(f, "QuoteBadAccount({which})"),
            QuoteError::TickArrayMissing { start_index } => write!(f, "TickArrayMissing({start_index})"),
            QuoteError::HookActionAccountMissing { start_index } => write!(f, "HookActionAccountMissing({start_index})"),
            other => f.write_str(other.name()),
        }
    }
}

impl std::error::Error for QuoteError {}

pub type QuoteResult<T> = Result<T, QuoteError>;
