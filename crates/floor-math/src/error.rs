//!

use core::fmt;

use carpenter_clmm_math::MathError;

///
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FloorMathError {
///
    Math(MathError),
///
    ZeroDenominator,
///
    Overflow,
///
    SupplyUnderflow,
///
    InvalidTickSpacing,
///
    InvalidRange,
///
    TranchePriceOutOfRange,
///
    UnknownMode,
///
    BadSplit,
///
///
///
///
///
///
///
    ReferralShareOutOfRange,
///
///
///
///
///
///
///
///
    QuoteDecimalsOutOfRange,

}

pub type FloorMathResult<T> = core::result::Result<T, FloorMathError>;

impl From<MathError> for FloorMathError {
    fn from(e: MathError) -> Self {
        FloorMathError::Math(e)
    }
}

impl fmt::Display for FloorMathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FloorMathError::Math(e) => write!(f, "clmm math: {e}"),
            FloorMathError::ZeroDenominator => f.write_str("zero denominator"),
            FloorMathError::Overflow => f.write_str("arithmetic overflow"),
            FloorMathError::SupplyUnderflow => {
                f.write_str("supply is below inventory plus ladder tokens")
            }
            FloorMathError::InvalidTickSpacing => f.write_str("tick spacing must be at least 1"),
            FloorMathError::InvalidRange => f.write_str("position range is empty or inverted"),
            FloorMathError::TranchePriceOutOfRange => {
                f.write_str("tranche price outside (0, 2^77)")
            }
            FloorMathError::UnknownMode => f.write_str("unknown launch mode"),
            FloorMathError::BadSplit => f.write_str("supply split outside the rule"),
            FloorMathError::ReferralShareOutOfRange => {
                f.write_str("referral share outside 0..=10000")
            }
            FloorMathError::QuoteDecimalsOutOfRange => {


                f.write_str("quote mint decimals outside the admissible window")
            }
        }
    }
}

impl std::error::Error for FloorMathError {}
