


use core::fmt;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathError {
    TickOutOfRange,
    SqrtPriceOutOfRange,
    CalculateOverflow,
    ZeroLiquidity,
    ZeroSqrtPrice,
    MaxTokenOverflow,
    LiquidityUnderflow,
    LiquidityOverflow,
    PriceTargetOvershoot,
    InvalidTickArrayBoundary,
    FeeRateTooHigh,
    InvalidFeeForExactOut,
    InvalidTickSpacing,
    BitmapLengthMismatch,
    DirectionMismatch,
}

pub type MathResult<T> = core::result::Result<T, MathError>;

impl fmt::Display for MathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            MathError::TickOutOfRange => "tick outside [MIN_TICK, MAX_TICK]",
            MathError::SqrtPriceOutOfRange => {
                "sqrt price outside [MIN_SQRT_PRICE_X64, MAX_SQRT_PRICE_X64)"
            }
            MathError::CalculateOverflow => "arithmetic overflow",
            MathError::ZeroLiquidity => "liquidity is zero",
            MathError::ZeroSqrtPrice => "sqrt price is zero",
            MathError::MaxTokenOverflow => "token amount exceeds u64",
            MathError::LiquidityUnderflow => "liquidity delta underflows",
            MathError::LiquidityOverflow => "liquidity delta overflows",
            MathError::PriceTargetOvershoot => "swap step moved past its target price",
            MathError::InvalidTickArrayBoundary => "not a valid tick-array start index",
            MathError::FeeRateTooHigh => "fee rate above 1,000,000 pips",
            MathError::InvalidFeeForExactOut => "a 100% fee cannot be used for exact output",
            MathError::InvalidTickSpacing => "tick spacing must be at least 1",
            MathError::BitmapLengthMismatch => "tick-array bitmap has the wrong number of words",
            MathError::DirectionMismatch => {
                "zero_for_one disagrees with the direction from current to target price"
            }
        };
        f.write_str(s)
    }
}

impl std::error::Error for MathError {}
