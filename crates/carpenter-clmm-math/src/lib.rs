// Derived from raydium-io/raydium-clmm @ ed7c84a5, programs/amm/src/libraries/mod.rs (Apache-2.0;
// see NOTICE). FLOOR changed: the glob re-exports are dropped (each caller names the module),
// and `bitmap`, `error` and the crate doc below are FLOOR's own.
//
// ⛔ This line used to read "New in FLOOR, 2026 (not derived)" above the "Adapted from Raydium
// CLMM" line below, and the guard's `not derived` test cancelled the declaration, so the file was
// recorded nowhere (SO-LICENCE review, 2026-09-17). A file declares one or the other, never both.

//! Q64.64 concentrated-liquidity math for FLOOR's v4-shaped Solana AMM (spec D-01, D-06, §5.6, §9 WP-M1).
//!
//! Adapted from Raydium CLMM `ed7c84a5` (Apache-2.0; see `NOTICE`). No Solana dependency (D-30): keys,
//! accounts and errors belong to the program crates, which convert [`MathError`] into their own codes.
//!
//! - [`tick_math`]: tick ↔ sqrt price, ticks ±443,636, `MIN/MAX_SQRT_PRICE_X64` (D-06).
//! - [`sqrt_price_math`]: the next sqrt price from an input or output amount.
//! - [`liquidity_math`]: amount ↔ liquidity, `add_delta`.
//! - [`swap_math`]: one swap step, `compute_swap`, with v4's LP-fee rounding on the fee-on-input path.
//! - [`full_math`]: overflow-resistant multiply-divide; [`big_num`]: `U128`, `U256`, `U512`.
//! - [`bitmap`]: the spacing-sized tick-array bitmap of D-07 / §4.1.

pub mod big_num;
pub mod bitmap;
pub mod error;
pub mod fixed_point_64;
pub mod full_math;
pub mod liquidity_math;
pub mod sqrt_price_math;
pub mod swap_math;
pub mod tick_math;
pub mod unsafe_math;

pub use error::{MathError, MathResult};
