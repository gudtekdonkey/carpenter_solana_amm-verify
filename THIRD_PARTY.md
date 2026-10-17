# Third-party code

| Component | Source | Licence | Where |
|---|---|---|---|
| Raydium Concentrated Liquidity math libraries | `raydium-io/raydium-clmm` @ `ed7c84a54ced59c55981780546adb0b4583dcf85` (`programs/amm/src/libraries/`) | Apache-2.0 | `crates/carpenter-clmm-math` — each derived file names its upstream path and FLOOR's changes in its header; see that crate's `NOTICE` and `LICENSE-APACHE-2.0` |
| Uniswap v4-core `SwapMath.computeSwapStep` fee-rounding (behaviour followed) | `Uniswap/v4-core` @ `46c6834` | MIT | `crates/carpenter-clmm-math/src/swap_math.rs` |
| Rust crate dependencies | crates.io, exact versions in `Cargo.lock` | per crate | — |

Licence texts: `LICENSES/`.
