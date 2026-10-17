//!
//!
//!
//!
//!
//!
//!

use floor_types::constants::LAUNCH_STATE_BONDING;
use floor_types::layout::{BuyerPosition, Launch};

use crate::error::{QuoteError, QuoteResult};
use crate::mech;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SaleBuyQuote {
    pub tokens: u64,
///
    pub cost: u64,
///
    pub team_fee: u64,
///
    pub spent: u64,
///
///
    pub graduates: bool,
}

///
///
///
pub fn sale_buy(launch: &Launch, lamports_in: u64) -> QuoteResult<SaleBuyQuote> {
    if launch.state != LAUNCH_STATE_BONDING {
        return Err(QuoteError::SaleOver);
    }
    let remaining = launch.sale_tokens.checked_sub(launch.sold).ok_or(QuoteError::Overflow)?;
    let (tokens, cost, team_fee) = mech::sale_buy_with_team(lamports_in, launch.price_x64.get(), remaining, launch.team_bps)?;
    if tokens == 0 {
        return Err(QuoteError::Nothing);
    }
    let spent = cost.checked_add(team_fee).ok_or(QuoteError::Overflow)?;
    Ok(SaleBuyQuote { tokens, cost, team_fee, spent, graduates: launch.sold + tokens == launch.sale_tokens })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SaleSellBackQuote {
///
    pub payout: u64,
///
    pub team_fee: u64,
///
    pub received: u64,
}

///
///
///
///
///
///
pub fn sale_sell_back(launch: &Launch, position: &BuyerPosition, tokens: u64) -> QuoteResult<SaleSellBackQuote> {
    if tokens > position.allocation && launch.state == LAUNCH_STATE_BONDING && tokens > 0 {
        return Err(QuoteError::InsufficientAllocation);
    }
    sale_sell_back_stateless(launch, tokens)
}

///
///
pub fn sale_sell_back_stateless(launch: &Launch, tokens: u64) -> QuoteResult<SaleSellBackQuote> {
    if launch.state != LAUNCH_STATE_BONDING {
        return Err(QuoteError::SaleOver);
    }
    if tokens == 0 {
        return Err(QuoteError::Nothing);
    }
    let payout = mech::sell_back_payout_at(tokens, launch.price_x64.get())?;
    let (team_fee, received) = mech::sell_back_with_team(payout, launch.team_bps)?;
    Ok(SaleSellBackQuote { payout, team_fee, received })
}

#[cfg(test)]
mod tests {
    use super::*;
    use floor_types::U128;
    fn launch(team_bps: u16) -> Launch {
        let mut l: Launch = bytemuck::Zeroable::zeroed();
        (l.version, l.state, l.sale_tokens, l.team_bps) = (3, LAUNCH_STATE_BONDING, floor_types::constants::SALE_TOKENS, team_bps);
        l.price_x64 = U128::new(1u128 << 64);
        l
    }
    #[test]
    fn a_sale_buy_quote_spends_the_gross_and_reports_the_fee() {
        assert_eq!(sale_buy(&launch(25), 1_002_500_000).unwrap(), SaleBuyQuote { tokens: 1_000_000_000, cost: 1_000_000_000, team_fee: 2_500_000, spent: 1_002_500_000, graduates: false });
        let tiny = sale_buy(&launch(25), 400).unwrap();
        assert_eq!((tiny.tokens, tiny.cost, tiny.team_fee, tiny.spent), (399, 399, 0, 399));
        let old = sale_buy(&launch(0), 1_002_500_000).unwrap();
        assert_eq!((old.cost, old.team_fee), (1_002_500_000, 0));
    }
    #[test]
    fn a_sell_back_quote_is_stateless_and_pays_the_share_less_the_fee() {
        let mut l = launch(25);
        l.price_x64 = U128::new(3u128 << 63);
        let mut p: BuyerPosition = bytemuck::Zeroable::zeroed();
        (p.allocation, p.contributed) = (3_000, 4_507);
        assert_eq!(sale_sell_back(&l, &p, 1_000).unwrap(), SaleSellBackQuote { payout: 1_500, team_fee: 3, received: 1_497 });
        assert_eq!(sale_sell_back(&l, &p, 3_000).unwrap(), SaleSellBackQuote { payout: 4_500, team_fee: 11, received: 4_489 });
        assert_eq!(sale_sell_back(&l, &p, 1_001).unwrap().payout, 1_501, "floor(1_001 · 1.5)");

        for t in [1u64, 2, 999, 1_000, 3_000] {
            assert_eq!(sale_sell_back(&l, &p, t).unwrap(), sale_sell_back_stateless(&l, t).unwrap());
        }
        assert!(sale_sell_back(&l, &p, 3_001).is_err(), "more than the allocation");
        assert!(sale_sell_back_stateless(&l, 0).is_err(), "nothing");
        l.state = LAUNCH_STATE_BONDING + 1;
        assert!(sale_sell_back_stateless(&l, 1).is_err(), "not bonding");
    }

///
///
///
///
    #[test]
    fn a_quote_follows_each_launchs_own_rate_including_zero() {
        let gross = 1_010_000_000;
        assert_eq!(sale_buy(&launch(100), gross).unwrap(), SaleBuyQuote { tokens: 1_000_000_000, cost: 1_000_000_000, team_fee: 10_000_000, spent: 1_010_000_000, graduates: false });
        assert_eq!(sale_buy(&launch(40), gross).unwrap(), SaleBuyQuote { tokens: 1_005_976_095, cost: 1_005_976_095, team_fee: 4_023_904, spent: 1_009_999_999, graduates: false });
        assert_eq!(sale_buy(&launch(0), gross).unwrap(), SaleBuyQuote { tokens: gross, cost: gross, team_fee: 0, spent: gross, graduates: false });
        let mut p: BuyerPosition = bytemuck::Zeroable::zeroed();
        (p.allocation, p.contributed) = (1_000_000_000, 1_000_000_000);
        assert_eq!(sale_sell_back(&launch(100), &p, 1_000_000_000).unwrap(), SaleSellBackQuote { payout: 1_000_000_000, team_fee: 10_000_000, received: 990_000_000 });
        assert_eq!(sale_sell_back(&launch(40), &p, 1_000_000_000).unwrap(), SaleSellBackQuote { payout: 1_000_000_000, team_fee: 4_000_000, received: 996_000_000 });
        assert_eq!(sale_sell_back(&launch(0), &p, 1_000_000_000).unwrap(), SaleSellBackQuote { payout: 1_000_000_000, team_fee: 0, received: 1_000_000_000 });
    }
}
