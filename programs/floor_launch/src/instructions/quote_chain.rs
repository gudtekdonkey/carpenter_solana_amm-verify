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

use floor_types::constants::{LAUNCH_STATE_GRADUATED, MAX_PRESET_ID, MAX_QUOTE_DECIMALS, MIN_QUOTE_DECIMALS, MODE_CUSTOM};
use floor_types::layout::{FloorPlatformPool, FloorPool, Launch as LaunchLayout, Layout, Pool};
use floor_types::twap::TwapOracle;

use crate::constants::{CARPENTER_AMM_PROGRAM, FLOOR_HOOK_PROGRAM, LAUNCH_SEED, WSOL_MINT};
use crate::errors::LaunchError;

///
pub const WALK_FIXED: usize = 3;

///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chain {
    pub root: Pubkey,
    pub root_decimals: u8,
    pub ticks: i64,
    pub hops: u16,
}

///
///
fn is_preset(mode: u8) -> bool {
    mode != MODE_CUSTOM && mode <= MAX_PRESET_ID
}

///
pub fn floor_platform_pool(record: &AccountInfo) -> Result<Option<[u8; 32]>> {
    let (expected, _) = Pubkey::find_program_address(&[floor_types::seeds::FLOOR_PLATFORM_POOL], &FLOOR_HOOK_PROGRAM);
    if record.key() != expected || *record.owner != FLOOR_HOOK_PROGRAM {
        return Ok(None);
    }
    let data = record.try_borrow_data()?;
    if data.len() < FloorPlatformPool::ACCOUNT_LEN || data[..8] != FloorPlatformPool::DISCRIMINATOR {
        return Ok(None);
    }
    let r: FloorPlatformPool = bytemuck::pod_read_unaligned(&data[8..FloorPlatformPool::ACCOUNT_LEN]);
    Ok(Some(r.floor_pool))
}

///
fn hop(fp_acct: &AccountInfo, pool_acct: &AccountInfo, quote: &Pubkey, floor_pool: Option<[u8; 32]>, now: u32) -> Result<(Pubkey, i32)> {
    require_keys_eq!(*fp_acct.owner, FLOOR_HOOK_PROGRAM, LaunchError::QuoteChainBadHop);
    let data = fp_acct.try_borrow_data()?;
    require!(
        data.len() >= FloorPool::ACCOUNT_LEN && data[..8] == FloorPool::DISCRIMINATOR,
        LaunchError::QuoteChainBadHop
    );
    let fp: FloorPool = bytemuck::pod_read_unaligned(&data[8..FloorPool::ACCOUNT_LEN]);
    let derived = Pubkey::create_program_address(
        &[floor_types::seeds::FLOOR_POOL, fp.amm_pool.as_ref(), &[fp.bump]],
        &FLOOR_HOOK_PROGRAM,
    )
    .map_err(|_| error!(LaunchError::QuoteChainBadHop))?;
    require_keys_eq!(derived, fp_acct.key(), LaunchError::QuoteChainBadHop);

    let is_floor = floor_pool == Some(fp_acct.key().to_bytes());
    require!(
        fp.base_mint == quote.to_bytes()
            && fp.pool_bound == 1
            && fp.ladder_seeded == 1
            && fp.quote_is_token1 == 0
            && fp.holders_hooked == 0
            && (is_floor || is_preset(fp.mode)),
        LaunchError::QuoteChainBadHop
    );

    require!(
        pool_acct.key().to_bytes() == fp.amm_pool && *pool_acct.owner == CARPENTER_AMM_PROGRAM,
        LaunchError::QuoteChainBadHop
    );
    let live = {
        let p = pool_acct.try_borrow_data()?;
        require!(p.len() >= Pool::ACCOUNT_LEN && p[..8] == Pool::DISCRIMINATOR, LaunchError::QuoteChainBadHop);
        bytemuck::pod_read_unaligned::<Pool>(&p[8..Pool::ACCOUNT_LEN]).tick_current
    };

    let oracle = TwapOracle::from_account(&data).ok_or(LaunchError::ParentTwapTooYoung)?;
    let twap = oracle
        .twap_tick(now, live, floor_math::lineage::CHAIN_PRICE_TWAP_WINDOW_SECS)
        .map_err(|_| error!(LaunchError::ParentTwapTooYoung))?;
    Ok((Pubkey::new_from_array(fp.quote_mint), floor_math::chain_hop_tick(twap, live)))
}

///
fn root_decimals(acct: &AccountInfo, root: &Pubkey) -> Result<u8> {
    require_keys_eq!(acct.key(), *root, LaunchError::QuoteChainBadHop);
    let owner = *acct.owner;
    require!(owner == anchor_spl::token::ID || owner == anchor_spl::token_2022::ID, LaunchError::QuoteChainBadHop);
    let data = acct.try_borrow_data()?;
    require!(data.len() >= 82 && data[45] == 1, LaunchError::QuoteChainBadHop);
    let d = data[44];
    require!((MIN_QUOTE_DECIMALS..=MAX_QUOTE_DECIMALS).contains(&d), LaunchError::QuoteChainBadHop);
    Ok(d)
}

///
///
fn root_ends_the_walk(acct: &AccountInfo, root: &Pubkey, floor_pool: Option<[u8; 32]>) -> Result<()> {
    let (expected, _) = Pubkey::find_program_address(&[LAUNCH_SEED, root.as_ref()], &crate::ID);
    require_keys_eq!(acct.key(), expected, LaunchError::QuoteChainBadHop);
    if *acct.owner != crate::ID {
        return Ok(());
    }
    let data = acct.try_borrow_data()?;
    if data.len() < LaunchLayout::ACCOUNT_LEN || data[..8] != LaunchLayout::DISCRIMINATOR {
        return Ok(());
    }
    let l: LaunchLayout = bytemuck::pod_read_unaligned(&data[8..LaunchLayout::ACCOUNT_LEN]);
    let hooked = l.holder_buy != 0 || l.holder_sell != 0;
    let qualifies =
        l.state == LAUNCH_STATE_GRADUATED && !hooked && (is_preset(l.mode) || floor_pool == Some(l.floor_pool));
    require!(!qualifies, LaunchError::QuoteChainCut);
    Ok(())
}

///
///
///
pub fn walk_hops(pairs: &[AccountInfo], cur: &Pubkey, floor_pool: Option<[u8; 32]>, now: u32) -> Result<(Pubkey, i64, usize)> {
    require!(pairs.len() % 2 == 0, LaunchError::QuoteChainBadHop);
    let hops = pairs.len() / 2;
    let mut cur = *cur;
    let mut ticks: i64 = 0;
    for i in 0..hops {
        let (next, t) = hop(&pairs[2 * i], &pairs[2 * i + 1], &cur, floor_pool, now)?;
        ticks = ticks.checked_add(t as i64).ok_or(LaunchError::MathOverflow)?;
        cur = next;
    }
    Ok((cur, ticks, hops))
}

///
///
///
pub fn end_at_root(tail: &[AccountInfo], cur: &Pubkey, floor_pool: Option<[u8; 32]>) -> Result<u8> {
    require!(tail.len() == WALK_FIXED - 1, LaunchError::QuoteChainBadHop);
    if *cur == WSOL_MINT {
        return Ok(9);
    }
    let d = root_decimals(&tail[0], cur)?;
    root_ends_the_walk(&tail[1], cur, floor_pool)?;
    Ok(d)
}

///
///
///
pub fn walk(w: &[AccountInfo], quote: &Pubkey, now: u32) -> Result<Chain> {
    require!(w.len() >= WALK_FIXED && (w.len() - WALK_FIXED) % 2 == 0, LaunchError::QuoteChainBadHop);
    let floor_pool = floor_platform_pool(&w[0])?;
    let (cur, ticks, hops) = walk_hops(&w[1..w.len() - (WALK_FIXED - 1)], quote, floor_pool, now)?;
    let root_decimals = end_at_root(&w[w.len() - (WALK_FIXED - 1)..], &cur, floor_pool)?;
    Ok(Chain { root: cur, root_decimals, ticks, hops: u16::try_from(hops).map_err(|_| LaunchError::MathOverflow)? })
}

#[cfg(test)]
mod tests {
    use super::*;

///
///
///
    #[test]
    fn a_preset_parent_is_any_registered_id_up_to_the_tables_bound() {
        assert!(!is_preset(MODE_CUSTOM), "Custom is never a preset parent");
        for id in 1..=MAX_PRESET_ID {
            assert!(is_preset(id), "id {id} is a preset slot");
        }
        for id in (MAX_PRESET_ID + 1)..=u8::MAX {
            assert!(!is_preset(id), "id {id} is past the table");
        }
        assert_eq!(MAX_PRESET_ID, 31);
    }
}
