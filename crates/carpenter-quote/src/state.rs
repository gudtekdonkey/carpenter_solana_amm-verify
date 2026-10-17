//!
//!
//!
//!
//!

use std::collections::BTreeMap;

use carpenter_clmm_math::bitmap;
use floor_types::layout::{FloorPool, Launch, Layout, Pool, TickArray, TickArrayBitmapHeader};

use crate::error::{QuoteError, QuoteResult};

///
///
pub fn decode_account<T: Layout>(data: &[u8]) -> QuoteResult<T> {
    if data.len() != T::ACCOUNT_LEN {
        return Err(QuoteError::QuoteBadAccount(T::NAME));
    }
    if data[..8] != T::DISCRIMINATOR {
        return Err(QuoteError::QuoteBadAccount(T::NAME));
    }
    Ok(bytemuck::pod_read_unaligned(&data[8..]))
}

///
pub fn encode_account<T: Layout>(value: &T) -> Vec<u8> {
    let mut out = Vec::with_capacity(T::ACCOUNT_LEN);
    out.extend_from_slice(&T::DISCRIMINATOR);
    out.extend_from_slice(bytemuck::bytes_of(value));
    out
}

pub fn decode_pool(data: &[u8]) -> QuoteResult<Pool> {
    decode_account::<Pool>(data)
}

///
///
///
pub fn decode_floor_pool(data: &[u8]) -> QuoteResult<FloorPool> {
    if data.len() < FloorPool::ACCOUNT_LEN || data[..8] != FloorPool::DISCRIMINATOR {
        return Err(QuoteError::QuoteBadAccount(FloorPool::NAME));
    }
    Ok(bytemuck::pod_read_unaligned(&data[8..FloorPool::ACCOUNT_LEN]))
}

///
///
///
///
pub fn decode_cushion_top(data: &[u8]) -> Option<i32> {
    floor_types::twap::TwapOracle::cushion_top_of_account(data)
}

pub fn decode_tick_array(data: &[u8]) -> QuoteResult<TickArray> {
    decode_account::<TickArray>(data)
}

pub fn decode_launch(data: &[u8]) -> QuoteResult<Launch> {
    decode_account::<Launch>(data)
}

///
///
pub fn decode_bitmap(data: &[u8], pool_key: &[u8; 32], tick_spacing: u16) -> QuoteResult<Vec<u64>> {
    let words = bitmap::word_count(tick_spacing)?;
    let fixed = TickArrayBitmapHeader::ACCOUNT_LEN;
    if data.len() != fixed + 8 * words || data[..8] != TickArrayBitmapHeader::DISCRIMINATOR {
        return Err(QuoteError::QuoteBadBitmap);
    }
    let header: TickArrayBitmapHeader = bytemuck::pod_read_unaligned(&data[8..fixed]);
    if &header.pool != pool_key || header.tick_spacing != tick_spacing {
        return Err(QuoteError::QuoteBadBitmap);
    }
    Ok(data[fixed..]
        .chunks_exact(8)
        .map(|c| u64::from_le_bytes(c.try_into().expect("8-byte chunk")))
        .collect())
}

///
///
///
///
///
///
///
#[derive(Clone, Debug)]
pub struct TickArraySet {
    pub tick_spacing: u16,
    pub arrays: BTreeMap<i32, TickArray>,
    pub words: Vec<u64>,
///
    pub bitmap_supplied: bool,
}

impl TickArraySet {
///
///
    pub fn new(pool_key: &[u8; 32], tick_spacing: u16, arrays: Vec<TickArray>, words: Option<Vec<u64>>) -> QuoteResult<Self> {
        let mut map = BTreeMap::new();
        for a in arrays {
            if &a.pool != pool_key || !bitmap::is_valid_start_index(a.start_index, tick_spacing)? {
                return Err(QuoteError::QuoteBadTickArray);
            }
            if map.insert(a.start_index, a).is_some() {

                return Err(QuoteError::QuoteBadTickArray);
            }
        }
        let (words, bitmap_supplied) = match words {
            Some(w) => {
                bitmap::check_words(&w, tick_spacing)?;
                (w, true)
            }
            None => {
                let mut w = vec![0u64; bitmap::word_count(tick_spacing)?];
                for (start, a) in &map {
                    if a.initialized_tick_count > 0 {
                        bitmap::set_initialized(&mut w, bitmap::array_bit(*start, tick_spacing)?, true)?;
                    }
                }
                (w, false)
            }
        };
        Ok(Self { tick_spacing, arrays: map, words, bitmap_supplied })
    }

///
    pub fn from_accounts(pool_key: &[u8; 32], tick_spacing: u16, arrays: &[&[u8]], bitmap: Option<&[u8]>) -> QuoteResult<Self> {
        let decoded = arrays.iter().map(|d| decode_tick_array(d)).collect::<QuoteResult<Vec<_>>>()?;
        let words = match bitmap {
            Some(b) => Some(decode_bitmap(b, pool_key, tick_spacing)?),
            None => None,
        };
        Self::new(pool_key, tick_spacing, decoded, words)
    }

///
    pub fn is_marked(&self, start_index: i32) -> QuoteResult<bool> {
        Ok(bitmap::is_initialized(&self.words, bitmap::array_bit(start_index, self.tick_spacing)?)?)
    }
}

///
pub fn encode_bitmap(pool_key: &[u8; 32], tick_spacing: u16, words: &[u64]) -> Vec<u8> {
    let header = TickArrayBitmapHeader {
        pool: *pool_key,
        tick_spacing,
        pad0: 0,
        arrays_per_side: floor_types::layout::bitmap_arrays_per_side(tick_spacing),
    };
    let mut out = encode_account(&header);
    for w in words {
        out.extend_from_slice(&w.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod twap_tail_tests {
    use super::*;
    use bytemuck::Zeroable;

///
///
    #[test]
    fn a_floor_pool_with_the_twap_tail_decodes_and_a_short_one_does_not() {
        let mut fp = FloorPool::zeroed();
        fp.floor_tick = 4_242;
        let bare = encode_account(&fp);
        let mut long = bare.clone();
        long.resize(floor_types::twap::FLOOR_POOL_ACCOUNT_LEN_WITH_TWAP, 0xAB);
        assert_eq!(decode_floor_pool(&bare).unwrap().floor_tick, 4_242);
        assert_eq!(decode_floor_pool(&long).unwrap().floor_tick, 4_242);
        assert!(matches!(decode_floor_pool(&bare[..bare.len() - 1]), Err(QuoteError::QuoteBadAccount("FloorPool"))));
        assert!(decode_account::<FloorPool>(&long).is_err(), "the generic decoder stays exact");
    }
}
