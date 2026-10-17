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

use floor_types::layout::FloorPool;

use crate::error::QuoteResult;
use crate::mech;

///
///
pub fn referral_tier_dues(floor_pool_data: &[u8]) -> QuoteResult<[u64; 3]> {
    let fp: FloorPool = crate::state::decode_floor_pool(floor_pool_data)?;

    Ok(mech::referral_tier_dues(&fp))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::QuoteError;
    use crate::state::encode_account;
    use bytemuck::Zeroable;

    #[test]
    fn the_account_bytes_quote_is_the_programs_arithmetic() {
        let mut fp = FloorPool::zeroed();
        fp.referrer = [1; 32];
        fp.referrer_t2 = [2; 32];
        fp.referrer_t3 = [3; 32];
        fp.referral_fees_paid = 500;
        fp.referral_owed = 500;
        assert_eq!(referral_tier_dues(&encode_account(&fp)).unwrap(), [450, 45, 5]);

        fp.referral_owed = 455;
        fp.referral_paid_t2 = 45;
        assert_eq!(referral_tier_dues(&encode_account(&fp)).unwrap(), [450, 0, 5]);
        assert_eq!(referral_tier_dues(&encode_account(&fp)[..100]), Err(QuoteError::QuoteBadAccount("FloorPool")));
    }
}
