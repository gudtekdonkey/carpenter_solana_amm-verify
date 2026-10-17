//!

use carpenter_clmm_math::big_num::U256;
use carpenter_clmm_math::fixed_point_64::Q64;
use carpenter_clmm_math::full_math::MulDiv;

use crate::constants::{quote_unit, BPS, SUPPLY};
use crate::error::{FloorMathError, FloorMathResult};

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
#[repr(u8)]
pub enum Mode {
///
///
///
///
    Custom = 0,
///
    Moonshot = 1,
///
    Cook = 2,
    Bluechip = 3,
}

impl TryFrom<u8> for Mode {
    type Error = FloorMathError;
    fn try_from(v: u8) -> FloorMathResult<Mode> {
        match v {
            0 => Ok(Mode::Custom),
            1 => Ok(Mode::Moonshot),
            2 => Ok(Mode::Cook),
            3 => Ok(Mode::Bluechip),

            _ => Err(FloorMathError::UnknownMode),
        }
    }
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Preset {
///
///
    pub graduation_mcap_usd: u64,
///
    pub creator_bps: u16,
///
    pub skim_buy_far_bps: u16,
    pub skim_sell_far_bps: u16,


///
///
///
///
    pub holder_bps: u16,
///
///
    pub sale_bps: u16,
    pub pol_bps: u16,



///
///
///
///
///
///
    pub skim_sell_at_floor_multiple_bps: u32,
///
///
///
///
///
///
    pub stair_steps: u8,
///
///
    pub stair_growth_bps: u16,
}

impl Preset {
///
    pub const fn is_stair(&self) -> bool {
        self.stair_steps != 0
    }
}

///
///
///
pub const fn skim_sell_at_floor_multiple_for(m: Mode) -> u32 {
    match m {
        Mode::Custom => crate::constants::SKIM_SELL_AT_FLOOR_MULTIPLE_BPS as u32,
        _ => preset(m).skim_sell_at_floor_multiple_bps,
    }
}

///
///
///
pub fn custom_stair(cap_usd: u64) -> (u8, u16) {
    crate::stair::custom_stair(cap_usd)
}

///
///
///
pub const MIN_ZONE_BPS: u16 = 1_000;
pub const STANDARD_SALE_BPS: u16 = 3_000;
pub const STANDARD_POL_BPS: u16 = 1_000;

///
///
///
pub fn split_tokens(sale_bps: u16, pol_bps: u16) -> FloorMathResult<(u64, u64, u64)> {
    let (s, p) = (sale_bps as u64, pol_bps as u64);
    let min = MIN_ZONE_BPS as u64;
    if s < min || p < min || s + p > BPS - min {
        return Err(FloorMathError::BadSplit);
    }
    let sale = SUPPLY / BPS * s;
    let pol = SUPPLY / BPS * p;
    Ok((sale, pol, SUPPLY - sale - pol))
}





































































///
pub const COOK_GRADUATION_MCAP_USD: u64 = 50_000;
///
pub const COOK_CREATOR_BPS: u16 = 75;
///
///
///
pub const COOK_SKIM_BPS: u16 = 200;
///
pub const COOK_HOLDER_BPS: u16 = 0;
///
pub const COOK_SALE_BPS: u16 = 2_000;
pub const COOK_POL_BPS: u16 = 1_000;
///
///
///
pub const COOK_STAIR_STEPS: u8 = 15;
pub const COOK_STAIR_GROWTH_BPS: u16 = 10_400;
///
///
///
pub const BLUECHIP_GRADUATION_MCAP_USD: u64 = 100_000;
pub const BLUECHIP_CREATOR_BPS: u16 = 100;
///
///
pub const BLUECHIP_SKIM_BPS: u16 = 200;
///
///
pub const BLUECHIP_HOLDER_BPS: u16 = 0;
pub const BLUECHIP_SALE_BPS: u16 = 2_000;
pub const BLUECHIP_POL_BPS: u16 = 1_000;
///
///
///
pub const BLUECHIP_STAIR_STEPS: u8 = 14;
pub const BLUECHIP_STAIR_GROWTH_BPS: u16 = 10_400;
///
///
pub const MOONSHOT_GRADUATION_MCAP_USD: u64 = 25_000;
pub const MOONSHOT_CREATOR_BPS: u16 = 50;
///
///
///
pub const MOONSHOT_SKIM_BPS: u16 = 200;
///
pub const MOONSHOT_HOLDER_BPS: u16 = 0;
pub const MOONSHOT_SALE_BPS: u16 = 2_000;
pub const MOONSHOT_POL_BPS: u16 = 1_000;
///
///
pub const MOONSHOT_STAIR_STEPS: u8 = 15;
pub const MOONSHOT_STAIR_GROWTH_BPS: u16 = 10_000;
const _: () = assert!(COOK_SALE_BPS == crate::stair::STAIR_SALE_BPS && COOK_POL_BPS == crate::stair::STAIR_POL_BPS);
const _: () = assert!(BLUECHIP_SALE_BPS == crate::stair::STAIR_SALE_BPS && BLUECHIP_POL_BPS == crate::stair::STAIR_POL_BPS);
const _: () = assert!(MOONSHOT_SALE_BPS == crate::stair::STAIR_SALE_BPS && MOONSHOT_POL_BPS == crate::stair::STAIR_POL_BPS);

///
///
///
///
pub const fn preset(m: Mode) -> Preset {
    match m {
        Mode::Cook => Preset {
            graduation_mcap_usd: COOK_GRADUATION_MCAP_USD,
            creator_bps: COOK_CREATOR_BPS,
            skim_buy_far_bps: COOK_SKIM_BPS,
            skim_sell_far_bps: COOK_SKIM_BPS,
            holder_bps: COOK_HOLDER_BPS,
            sale_bps: COOK_SALE_BPS,
            pol_bps: COOK_POL_BPS,
            skim_sell_at_floor_multiple_bps: crate::constants::PRESET_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS as u32,
            stair_steps: COOK_STAIR_STEPS,
            stair_growth_bps: COOK_STAIR_GROWTH_BPS,
        },
        Mode::Bluechip => Preset {
            graduation_mcap_usd: BLUECHIP_GRADUATION_MCAP_USD,
            creator_bps: BLUECHIP_CREATOR_BPS,
            skim_buy_far_bps: BLUECHIP_SKIM_BPS,
            skim_sell_far_bps: BLUECHIP_SKIM_BPS,
            holder_bps: BLUECHIP_HOLDER_BPS,
            sale_bps: BLUECHIP_SALE_BPS,
            pol_bps: BLUECHIP_POL_BPS,
            skim_sell_at_floor_multiple_bps: crate::constants::PRESET_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS as u32,
            stair_steps: BLUECHIP_STAIR_STEPS,
            stair_growth_bps: BLUECHIP_STAIR_GROWTH_BPS,
        },
        Mode::Moonshot => Preset {
            graduation_mcap_usd: MOONSHOT_GRADUATION_MCAP_USD,
            creator_bps: MOONSHOT_CREATOR_BPS,
            skim_buy_far_bps: MOONSHOT_SKIM_BPS,
            skim_sell_far_bps: MOONSHOT_SKIM_BPS,
            holder_bps: MOONSHOT_HOLDER_BPS,
            sale_bps: MOONSHOT_SALE_BPS,
            pol_bps: MOONSHOT_POL_BPS,
            skim_sell_at_floor_multiple_bps: crate::constants::PRESET_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS as u32,
            stair_steps: MOONSHOT_STAIR_STEPS,
            stair_growth_bps: MOONSHOT_STAIR_GROWTH_BPS,
        },


        Mode::Custom => Preset {
            graduation_mcap_usd: 0,
            creator_bps: 0,
            skim_buy_far_bps: 0,
            skim_sell_far_bps: 0,
            holder_bps: 0,
            sale_bps: 0,
            pol_bps: 0,

            skim_sell_at_floor_multiple_bps: 0,
            stair_steps: 0,
            stair_growth_bps: 0,
        },
    }
}

///
///
///
///
///
///
///
///
pub const MAX_TRANCHE_PRICE_X64: u128 = 1 << 77;

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
///
///
///
///
///
///
pub fn tranche_price_x64(mcap_usd: u64, quote_usd_cents: u64, quote_decimals: u8) -> FloorMathResult<u128> {
    if quote_usd_cents == 0 {
        return Err(FloorMathError::ZeroDenominator);
    }
    let unit = quote_unit(quote_decimals).ok_or(FloorMathError::QuoteDecimalsOutOfRange)?;





    let num = U256::from(mcap_usd) * U256::from(100u64) * U256::from(unit);
    let den = U256::from(quote_usd_cents) * U256::from(SUPPLY);
    let p = num.mul_div_floor(U256::from(Q64), den).ok_or(FloorMathError::Overflow)?;
    if p.is_zero() || p >= U256::from(MAX_TRANCHE_PRICE_X64) {
        return Err(FloorMathError::TranchePriceOutOfRange);
    }
    Ok(p.as_u128())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{MAX_QUOTE_DECIMALS, MIN_QUOTE_DECIMALS};





    #[test]
    fn presets_are_the_contracts() {
        let stair = |cap: u64, creator: u16, skim: u16, holder: u16, steps: u8, growth: u16| Preset {
            graduation_mcap_usd: cap, creator_bps: creator, skim_buy_far_bps: skim, skim_sell_far_bps: skim,
            holder_bps: holder, sale_bps: 2_000, pol_bps: 1_000, skim_sell_at_floor_multiple_bps: 15_000, stair_steps: steps, stair_growth_bps: growth,
        };

        assert_eq!(preset(Mode::Moonshot), stair(25_000, 50, 200, 0, 15, 10_000));

        assert_eq!(preset(Mode::Cook), stair(50_000, 75, 200, 0, 15, 10_400));
        assert_eq!(preset(Mode::Bluechip), stair(100_000, 100, 200, 0, 14, 10_400));
        for m in [Mode::Cook, Mode::Bluechip, Mode::Moonshot] {
            let p = preset(m);
            assert!(p.is_stair(), "{m:?} is a staircase preset");
            assert!(crate::stair::stair_shape_ok(p.stair_steps, p.stair_growth_bps), "{m:?}'s staircase is admitted");
            assert!(split_tokens(p.sale_bps, p.pol_bps).is_ok(), "{m:?}'s split breaks the rule");
            assert!(p.holder_bps <= crate::policy::MAX_HOLDER_BPS, "{m:?}'s holder rate under the cap");
            assert_eq!(p.holder_bps, 0, "{m:?}: no preset carries a holder rate (owner 2026-10-03)");
        }

        let at = |m: Mode| crate::constants::skim_sell_at_floor_bps_at(preset(m).skim_sell_far_bps, skim_sell_at_floor_multiple_for(m) as u64);

        assert_eq!(at(Mode::Cook), 300);
        assert_eq!(at(Mode::Moonshot), 300);
        assert_eq!(at(Mode::Bluechip), 300);
        assert_eq!(skim_sell_at_floor_multiple_for(Mode::Custom), 15_000, "Custom (and $FLOOR) keep 1.5x");
        assert_eq!(preset(Mode::Custom), Preset::default());


        assert!(!preset(Mode::Custom).is_stair(), "Custom's row: steps derived from the cap at create");
        assert_eq!(custom_stair(200_000), (12, 10_000));
        for b in [0u8, 1, 2, 3] {
            assert_eq!(Mode::try_from(b).unwrap() as u8, b);
        }

        assert_eq!(
            (Mode::Custom as u8, Mode::Moonshot as u8, Mode::Cook as u8, Mode::Bluechip as u8),
            (0, 1, 2, 3)
        );


        for b in 4u8..=255 {
            assert_eq!(Mode::try_from(b), Err(FloorMathError::UnknownMode), "{b}");
        }
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
    #[test]
    fn the_staircase_presets_are_the_owners_table() {
        let table = [
            (Mode::Moonshot, 25_000u64, 5_000u64, 15u8, 1.0f64),
            (Mode::Cook, 50_000, 10_000, 15, 1.04),
            (Mode::Bluechip, 100_000, 20_000, 14, 1.04),
        ];
        for (m, cap, raise, steps, growth) in table {
            let p = preset(m);
            assert_eq!(p.graduation_mcap_usd, cap, "{m:?} cap");
            let (sale, pol, rest) = split_tokens(p.sale_bps, p.pol_bps).unwrap();
            assert_eq!((cap as u128 * sale as u128 / SUPPLY as u128) as u64, raise, "{m:?}: the raise is 20 % of the cap");
            assert_eq!((sale, pol), (SUPPLY / 5, SUPPLY / 10), "{m:?} sale and classic POL");
            let (st, tail) = crate::stair::stair_tail_split(rest);
            assert_eq!((st, tail), (SUPPLY / 100 * 69, SUPPLY / 100), "{m:?}: 69 staircase / 1 tail");
            assert_eq!(p.stair_steps, steps, "{m:?} steps");
            assert_eq!(p.stair_growth_bps, (growth * 10_000.0).round() as u16, "{m:?} weighting");

            let top = cap as f64 * 2f64.powi(steps as i32);
            assert!((800e6..=1_700e6).contains(&top), "{m:?}: the staircase tops out at ${top}");
        }
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
    #[test]
    fn the_new_fees_are_the_owners_table() {
        let bps = |pct: f64| (pct * 100.0).round() as u16;

        let table = [

            (Mode::Moonshot, 2.0, 0.5, 0.0, 3.0),
            (Mode::Cook, 2.0, 0.75, 0.0, 3.0),
            (Mode::Bluechip, 2.0, 1.0, 0.0, 3.0),
        ];
        for (m, skim, creator, holder, at_floor) in table {
            let p = preset(m);
            assert_eq!((p.skim_buy_far_bps, p.skim_sell_far_bps), (bps(skim), bps(skim)), "{m:?} far skim");
            assert_eq!(p.creator_bps, bps(creator), "{m:?} creator");
            assert_eq!(p.holder_bps, bps(holder), "{m:?} holder");
            assert_eq!(p.skim_sell_at_floor_multiple_bps, 15_000, "{m:?}: 1.5x into the bid (2026-10-12; ~~5x~~)");
            assert_eq!(skim_sell_at_floor_multiple_for(m), 15_000);
            let at = crate::constants::skim_sell_at_floor_bps_at(p.skim_sell_far_bps, skim_sell_at_floor_multiple_for(m) as u64);
            assert_eq!(at, bps(at_floor), "{m:?} at-floor sell");
            assert!(p.skim_sell_far_bps <= crate::policy::MAX_SKIM_FAR_BPS && p.creator_bps <= crate::policy::MAX_CREATOR_BPS);
            let multiple = u64::from(p.skim_sell_at_floor_multiple_bps);
            assert!(
                (crate::constants::MIN_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS..=crate::constants::MAX_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS).contains(&multiple),
                "{m:?}'s multiple inside register_pool's window"
            );
        }

        assert_eq!(skim_sell_at_floor_multiple_for(Mode::Custom) as u64, crate::constants::SKIM_SELL_AT_FLOOR_MULTIPLE_BPS);
        assert_eq!(crate::constants::skim_sell_at_floor_bps_at(250, skim_sell_at_floor_multiple_for(Mode::Custom) as u64), 375);
        assert_eq!(preset(Mode::Custom).skim_sell_at_floor_multiple_bps, 0, "Custom's row is all zero");
    }







///
///
///
    #[test]
    fn tranche_price_matches_hand_computation() {
        let p = tranche_price_x64(5_000, 100 * 100, 9).unwrap();
        assert_eq!(p, (1u128 << 64) / 20_000);
        assert_eq!(p, 922_337_203_685_477);
        let whole_supply = (U256::from(SUPPLY) * U256::from(p)) >> 64usize;
        assert_eq!(whole_supply.as_u64(), 49_999_999_999);

        for mcap in [5_000u64, 20_000, 100_000, 50_000] {
            for sol in [1u64, 17, 100, 250, 2_400, 100_000] {
                let direct = (U256::from(mcap) << 64usize) / (U256::from(sol) * U256::from(1_000_000u64));
                assert_eq!(U256::from(tranche_price_x64(mcap, sol * 100, 9).unwrap()), direct, "{mcap}@{sol}");
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
    fn price_reproduces_the_wsol_whole_dollar_form() {
        for mcap in [1u64, 5_000, 20_000, 100_000, 8_191_999] {
            for sol in [1u64, 3, 17, 100, 101, 250, 2_400, 100_000] {
                let old = (U256::from(mcap) * U256::from(1_000_000_000u64))
                    .mul_div_floor(U256::from(Q64), U256::from(sol) * U256::from(SUPPLY))
                    .unwrap();
                let want = if old.is_zero() || old >= U256::from(MAX_TRANCHE_PRICE_X64) {
                    None
                } else {
                    Some(old.as_u128())
                };
                assert_eq!(tranche_price_x64(mcap, sol * 100, 9).ok(), want, "{mcap} at {sol} USD/SOL");
            }
        }
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
    fn the_evm_whole_dollar_shape_is_two_decimals_down() {
        for mcap in [1u64, 5_000, 20_000, 100_000, 8_191_999] {
            for usd in [1u64, 3, 17, 150, 2_400, 100_000, 1_000_000_000] {
                for d in (MIN_QUOTE_DECIMALS + 2)..=MAX_QUOTE_DECIMALS {
                    let Some(x100) = usd.checked_mul(100) else { continue };
                    assert_eq!(
                        tranche_price_x64(mcap, usd, d - 2).ok(),
                        tranche_price_x64(mcap, x100, d).ok(),
                        "{mcap} at {usd} USD, d={d}"
                    );
                }
            }
        }
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
    #[test]
    fn the_whole_supply_costs_the_cap_in_every_admissible_quote() {



        for d in MIN_QUOTE_DECIMALS..=MAX_QUOTE_DECIMALS {

            let unit: u128 = (0..d).fold(1u128, |a, _| a * 10);



            for cents in [100u64, 50, 199, 10_152, 23_000] {
                for mcap in [20_000u64, 40_000, 100_000] {
                    let p = match tranche_price_x64(mcap, cents, d) {
                        Ok(p) => p,



                        Err(e) => {
                            assert_eq!(e, FloorMathError::TranchePriceOutOfRange, "d={d} cents={cents} mcap={mcap}");
                            continue;
                        }
                    };
                    let got = ((U256::from(SUPPLY) * U256::from(p)) >> 64usize).as_u128();
                    let want = (mcap as u128) * 100 * unit / (cents as u128);




                    assert!(
                        got <= want && want - got <= 1,
                        "d={d} cents={cents} mcap={mcap}: whole supply costs {got}, cap is {want}"
                    );
                }
            }
        }




        assert_eq!(
            ((U256::from(SUPPLY) * U256::from(tranche_price_x64(20_000, 100, 0).unwrap())) >> 64usize).as_u128(),
            20_000 - 1,
            "a 0-decimal quote: the whole supply costs the cap in whole units of it, less one floor"
        );
        assert_eq!(
            ((U256::from(SUPPLY) * U256::from(tranche_price_x64(20_000, 100, 2).unwrap())) >> 64usize).as_u128(),
            2_000_000 - 1,
            "a 2-decimal quote: a hundred raw units to the whole unit"
        );
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
    #[test]
    fn one_decimal_step_is_exactly_one_factor_of_ten() {



        let mut rungs = 0usize;
        for d in MIN_QUOTE_DECIMALS..MAX_QUOTE_DECIMALS {
            let (Ok(lo), hi) = (tranche_price_x64(20_000, 100, d), tranche_price_x64(20_000, 100, d + 1)) else {
                break;
            };
            let Ok(hi) = hi else {
                assert_eq!(hi.unwrap_err(), FloorMathError::TranchePriceOutOfRange, "d={}", d + 1);
                break;
            };
            assert!(hi >= lo * 10 && hi <= lo * 10 + 9, "{d} -> {}: {lo} then {hi}", d + 1);
            rungs += 1;
        }



        assert!(tranche_price_x64(20_000, 100, 0).is_ok(), "a 0-decimal quote prices");
        assert!(rungs >= 14, "only {rungs} rungs: the 2^77 bound should allow d = 0..=14 here");
        assert!(rungs <= 20, "{rungs} rungs: the cap has stopped binding, which it cannot at $1.00");
    }

    #[test]
    fn tranche_price_bounds() {
        assert_eq!(tranche_price_x64(5_000, 0, 9), Err(FloorMathError::ZeroDenominator));
        assert_eq!(tranche_price_x64(0, 100 * 100, 9), Err(FloorMathError::TranchePriceOutOfRange));

        assert_eq!(tranche_price_x64(1, u64::MAX, 9), Err(FloorMathError::TranchePriceOutOfRange));

        assert_eq!(tranche_price_x64(u64::MAX, 100, 9), Err(FloorMathError::TranchePriceOutOfRange));

        assert!(tranche_price_x64(8_191_999_999, 100, 9).is_ok());
        assert_eq!(tranche_price_x64(8_192_000_000, 100, 9), Err(FloorMathError::TranchePriceOutOfRange));
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
    #[test]
    fn decimals_outside_the_window_are_refused_by_name() {
        assert!(tranche_price_x64(1, u64::MAX, 36).is_ok(), "the derivation's own input prices at 36");

        assert_eq!(tranche_price_x64(1, u64::MAX, 36).unwrap(), 10u128.pow(23) + 5_421);
        assert_eq!(tranche_price_x64(1, u64::MAX, 37), Err(FloorMathError::QuoteDecimalsOutOfRange));
        assert_eq!(tranche_price_x64(20_000, 100, 36), Err(FloorMathError::TranchePriceOutOfRange));
        assert_eq!(
            tranche_price_x64(20_000, 100, 20),
            Err(FloorMathError::TranchePriceOutOfRange),
            "20 is INSIDE the window (it was refused by name under the struck u64 ceiling)"
        );
        for d in 0..=255u8 {
            let r = tranche_price_x64(20_000, 15_000, d);
            if (MIN_QUOTE_DECIMALS..=MAX_QUOTE_DECIMALS).contains(&d) {
                assert_ne!(r, Err(FloorMathError::QuoteDecimalsOutOfRange), "d={d} is admissible");
            } else {
                assert_eq!(r, Err(FloorMathError::QuoteDecimalsOutOfRange), "d={d}");
            }
        }

        let p0 = tranche_price_x64(20_000, 15_000, 0).expect("a 0-decimal quote prices");
        let p2 = tranche_price_x64(20_000, 15_000, 2).expect("a 2-decimal quote prices");
        assert!(p2 >= p0 * 100 && p2 <= p0 * 100 + 99, "{p0} then {p2}");


        assert_eq!(tranche_price_x64(20_000, 0, 18), Err(FloorMathError::ZeroDenominator));
        assert_eq!(tranche_price_x64(20_000, 0, 99), Err(FloorMathError::ZeroDenominator));
    }

///
///
///
///
///
    #[test]
    fn a_sub_dollar_quote_prices_and_a_fractional_one_is_not_rounded() {

        assert!(tranche_price_x64(20_000, 99, 6).is_ok());

        let p199 = tranche_price_x64(20_000, 199, 6).unwrap();
        let p100 = tranche_price_x64(20_000, 100, 6).unwrap();
        let p200 = tranche_price_x64(20_000, 200, 6).unwrap();
        assert!(p200 < p199 && p199 < p100, "{p200} < {p199} < {p100}");
    }

///
///
///
    #[test]
    fn split_tokens_holds_each_zone_to_ten_percent_and_the_sum_to_supply() {
        let tenth = SUPPLY / 10;
        assert_eq!(split_tokens(3_000, 1_000), Ok((3 * tenth, tenth, 6 * tenth)));
        assert_eq!(split_tokens(1_000, 1_000), Ok((tenth, tenth, 8 * tenth)));
        assert_eq!(split_tokens(8_000, 1_000), Ok((8 * tenth, tenth, tenth)));
        assert_eq!(split_tokens(1_000, 8_000), Ok((tenth, 8 * tenth, tenth)));
        for (s, p) in [(999, 1_000), (1_000, 999), (8_001, 1_000), (1_000, 8_001), (4_500, 4_501), (0, 0), (9_000, 1_000)] {
            assert_eq!(split_tokens(s, p), Err(FloorMathError::BadSplit), "{s} / {p}");
        }
        for (s, p) in [(1_000, 1_000), (7_000, 1_000), (3_333, 3_333), (8_000, 1_000)] {
            let (a, b, c) = split_tokens(s, p).unwrap();
            assert_eq!(a + b + c, SUPPLY);
        }
    }
}
