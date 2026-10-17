//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
#![forbid(unsafe_code)]

pub mod error;
pub mod floor;
pub mod holders;
pub mod liquidity;
pub mod mech;
pub mod modify;
pub mod price;
pub mod referral;
pub mod sale;
pub mod state;
pub mod swap;
pub mod synth;
pub mod walk;

pub use error::{QuoteError, QuoteResult};
pub use liquidity::{amounts_for_liquidity, fees_owed, quote_liquidity};
pub use sale::{sale_buy, sale_sell_back, sale_sell_back_stateless, SaleBuyQuote, SaleSellBackQuote};
pub use state::TickArraySet;
pub use floor::{bid_holds_tokens, replace_actions, ReplaceBooking};
pub use holders::holder_claimable;
pub use referral::referral_tier_dues;
pub use swap::{quote_swap, PoolAccount, QuoteFees, ReplaceQuote, SwapQuote, SwapQuoteParams};

///
///
pub use carpenter_clmm_math as clmm;
pub use floor_types as types;

///
///
pub fn no_limit(zero_for_one: bool) -> u128 {
    if zero_for_one {
        floor_types::constants::MIN_SQRT_PRICE_X64 + 1
    } else {
        floor_types::constants::MAX_SQRT_PRICE_X64 - 1
    }
}
