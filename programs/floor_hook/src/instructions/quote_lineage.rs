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
//!
//!
//!
//!
//!
//!
//!
use anchor_lang::prelude::*;

use floor_types::constants::{MAX_QUOTE_DECIMALS, MIN_QUOTE_DECIMALS, MODE_CUSTOM};

use crate::constants::MAX_PRESET_ID;
use floor_types::layout::{
    FloorPlatformPool as FloorPlatformPoolLayout, FloorPool as FloorPoolLayout, Layout, PlatformTreasury as PlatformTreasuryLayout,
    Pool as PoolLayout,
};
use floor_types::twap::TwapOracle;

use crate::constants::{CARPENTER_AMM_PROGRAM, FLOOR_PLATFORM_POOL_SEED, FLOOR_POOL_SEED, PLATFORM_TREASURY_SEED};
use crate::errors::HookError;

///
pub const DERIVED_REMAINING: usize = 7;

///
pub const ADMITTED_ALLOW_ALL: u8 = 1;
pub const ADMITTED_PRESET_PARENT: u8 = 2;
pub const ADMITTED_FLOOR_PARENT: u8 = 3;

///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Derived {
    pub how: u8,
///
    pub parent: Pubkey,
    pub decimals: u8,
    pub min_bid: u64,
    pub max_bounty: u64,
///
    pub cap_tick: i32,
}

///
///
///
///
pub fn is_preset_mode(mode: u8) -> bool {
    mode != MODE_CUSTOM && mode <= MAX_PRESET_ID
}

///
///
///
pub fn mint_decimals(acct: &AccountInfo, quote_mint: &Pubkey) -> Result<u8> {
    require_keys_eq!(acct.key(), *quote_mint, HookError::QuoteNotListed);
    let owner = *acct.owner;
    require!(owner == anchor_spl::token::ID || owner == anchor_spl::token_2022::ID, HookError::QuoteNotListed);
    let data = acct.try_borrow_data()?;

    require!(data.len() >= 82 && data[45] == 1, HookError::QuoteNotListed);
    let d = data[44];
    require!((MIN_QUOTE_DECIMALS..=MAX_QUOTE_DECIMALS).contains(&d), HookError::QuoteDecimalsOutOfRange);
    Ok(d)
}

///
///
pub fn allow_all(acct: &AccountInfo) -> Result<bool> {
    let (expected, _) = Pubkey::find_program_address(&[PLATFORM_TREASURY_SEED], &crate::ID);
    if acct.key() != expected || *acct.owner != crate::ID {
        return Ok(false);
    }
    let data = acct.try_borrow_data()?;
    if data.len() < PlatformTreasuryLayout::ACCOUNT_LEN || data[..8] != PlatformTreasuryLayout::DISCRIMINATOR {
        return Ok(false);
    }
    let t: PlatformTreasuryLayout = bytemuck::pod_read_unaligned(&data[8..PlatformTreasuryLayout::ACCOUNT_LEN]);
    Ok(t.allow_all_quotes == 1)
}

///
///
///
pub fn parent_qualifies(fp: &FloorPoolLayout, quote_mint: &Pubkey, is_floor: bool) -> Option<(u8, u64)> {
    if fp.base_mint != quote_mint.to_bytes() || fp.pool_bound != 1 || fp.ladder_seeded != 1 || fp.quote_is_token1 != 0 {
        return None;
    }
    if fp.holders_hooked != 0 {
        return None;
    }
    if is_floor {
        Some((ADMITTED_FLOOR_PARENT, fp.max_bounty))
    } else if is_preset_mode(fp.mode) {
        Some((ADMITTED_PRESET_PARENT, fp.max_bounty))
    } else {
        None
    }
}

///
///
fn read_parent(acct: &AccountInfo) -> Result<Option<FloorPoolLayout>> {
    if *acct.owner != crate::ID {
        return Ok(None);
    }
    let data = acct.try_borrow_data()?;
    if data.len() < FloorPoolLayout::ACCOUNT_LEN || data[..8] != FloorPoolLayout::DISCRIMINATOR {
        return Ok(None);
    }
    let fp: FloorPoolLayout = bytemuck::pod_read_unaligned(&data[8..FloorPoolLayout::ACCOUNT_LEN]);
    let derived = Pubkey::create_program_address(&[FLOOR_POOL_SEED, fp.amm_pool.as_ref(), &[fp.bump]], &crate::ID);
    Ok(match derived {
        Ok(k) if k == acct.key() => Some(fp),
        _ => None,
    })
}

///
fn is_floor_pool(record: &AccountInfo, parent: &Pubkey) -> Result<bool> {
    let (expected, _) = Pubkey::find_program_address(&[FLOOR_PLATFORM_POOL_SEED], &crate::ID);
    if record.key() != expected || *record.owner != crate::ID {
        return Ok(false);
    }
    let data = record.try_borrow_data()?;
    if data.len() < FloorPlatformPoolLayout::ACCOUNT_LEN || data[..8] != FloorPlatformPoolLayout::DISCRIMINATOR {
        return Ok(false);
    }
    let r: FloorPlatformPoolLayout = bytemuck::pod_read_unaligned(&data[8..FloorPlatformPoolLayout::ACCOUNT_LEN]);
    Ok(r.floor_pool == parent.to_bytes())
}

///
///
fn settled_parent_tick(parent_acct: &AccountInfo, fp: &FloorPoolLayout, pool_acct: &AccountInfo, now: u32) -> Result<Option<i32>> {
    if pool_acct.key().to_bytes() != fp.amm_pool || *pool_acct.owner != CARPENTER_AMM_PROGRAM {
        return Ok(None);
    }
    let live = {
        let data = pool_acct.try_borrow_data()?;
        if data.len() < PoolLayout::ACCOUNT_LEN || data[..8] != PoolLayout::DISCRIMINATOR {
            return Ok(None);
        }
        let p: PoolLayout = bytemuck::pod_read_unaligned(&data[8..PoolLayout::ACCOUNT_LEN]);
        p.tick_current
    };
    let data = parent_acct.try_borrow_data()?;
    let Some(o) = TwapOracle::from_account(&data) else { return Ok(None) };
    Ok(o.settled_tick(now, live, floor_math::lineage::SMART_CAP_TWAP_WINDOW_SECS).ok())
}

///
pub fn admit<'info>(ra: &[AccountInfo<'info>], quote_mint: &Pubkey, now: u32) -> Result<Derived> {
    require!(ra.len() == DERIVED_REMAINING, HookError::BadConfig);
    let decimals = mint_decimals(&ra[2], quote_mint)?;
    let min_bid = floor_math::min_bid_for(decimals);
    let parent_key = ra[4].key();
    if let Some(fp) = read_parent(&ra[4])? {
        let is_floor = is_floor_pool(&ra[6], &parent_key)?;
        if let Some((how, parent_cap)) = parent_qualifies(&fp, quote_mint, is_floor) {
            let (max_bounty, cap_tick) = match settled_parent_tick(&ra[4], &fp, &ra[5], now)? {
                Some(t) => (floor_math::quote_amount_at_tick(parent_cap, t), t),
                None => (0, 0),
            };
            return Ok(Derived { how, parent: parent_key, decimals, min_bid, max_bounty, cap_tick });
        }


        require!(!(fp.base_mint == quote_mint.to_bytes() && fp.holders_hooked != 0), HookError::QuoteNotAdmitted);
    }
    require!(allow_all(&ra[3])?, HookError::QuoteNotAdmitted);
    Ok(Derived { how: ADMITTED_ALLOW_ALL, parent: Pubkey::default(), decimals, min_bid, max_bounty: 0, cap_tick: 0 })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck::Zeroable;

    fn graduated(mint: Pubkey, mode: u8) -> FloorPoolLayout {
        let mut fp = FloorPoolLayout::zeroed();
        fp.base_mint = mint.to_bytes();
        fp.pool_bound = 1;
        fp.ladder_seeded = 1;
        fp.mode = mode;
        fp.max_bounty = 50_000_000;
        fp
    }

///
///
    #[test]
    fn only_a_graduated_preset_or_floors_own_pool_qualifies() {
        let m = Pubkey::new_unique();
        use floor_types::constants::{MODE_BLUECHIP, MODE_COOK, MODE_MOONSHOT};
        for mode in [MODE_COOK, MODE_BLUECHIP, MODE_MOONSHOT, 4, MAX_PRESET_ID] {
            assert_eq!(parent_qualifies(&graduated(m, mode), &m, false), Some((ADMITTED_PRESET_PARENT, 50_000_000)));
        }
        let custom = graduated(m, floor_types::constants::MODE_CUSTOM);
        assert_eq!(parent_qualifies(&custom, &m, false), None, "Custom is not a preset");
        assert_eq!(parent_qualifies(&custom, &m, true), Some((ADMITTED_FLOOR_PARENT, 50_000_000)), "$FLOOR by its record");
        let mut bonding = graduated(m, MODE_COOK);
        bonding.ladder_seeded = 0;
        assert_eq!(parent_qualifies(&bonding, &m, false), None, "prepared but not graduated");
        bonding.ladder_seeded = 1;
        bonding.pool_bound = 0;
        assert_eq!(parent_qualifies(&bonding, &m, true), None, "unbound, even $FLOOR");
        let mut hooked = graduated(m, MODE_BLUECHIP);
        hooked.holders_hooked = 1;
        assert_eq!(parent_qualifies(&hooked, &m, false), None, "a hooked mint cannot be a quote");
        let mut flipped = graduated(m, MODE_COOK);
        flipped.quote_is_token1 = 1;
        assert_eq!(parent_qualifies(&flipped, &m, false), None);
        assert_eq!(parent_qualifies(&graduated(Pubkey::new_unique(), MODE_COOK), &m, false), None, "another mint's pool");
        assert!(!is_preset_mode(floor_types::constants::MODE_CUSTOM));
        assert_eq!(parent_qualifies(&graduated(m, MAX_PRESET_ID + 1), &m, false), None, "past the table is no preset");
    }

///
///
///
    #[test]
    fn a_preset_is_any_registered_id_up_to_the_tables_bound() {
        assert!(!is_preset_mode(MODE_CUSTOM), "Custom is never a preset");
        for id in 1..=MAX_PRESET_ID {
            assert!(is_preset_mode(id), "id {id} is a preset slot");
        }
        for id in (MAX_PRESET_ID + 1)..=u8::MAX {
            assert!(!is_preset_mode(id), "id {id} is past the table");
        }
        assert_eq!(MAX_PRESET_ID, 31);
    }
}
