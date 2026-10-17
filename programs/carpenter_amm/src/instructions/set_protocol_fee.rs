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
use carpenter_types::constants::MAX_PROTOCOL_FEE;
use carpenter_types::protocol_fee::pack;

use crate::constants::{AMM_CONFIG_SEED, PROTOCOL_TREASURY_SEED};
use crate::errors::AmmError;
use crate::events::ProtocolFeeSet;
use crate::state::{AmmConfig, ProtocolTreasury};

#[event_cpi]
#[derive(Accounts)]
pub struct SetProtocolFee<'info> {
    #[account(constraint = governance.key() == protocol_treasury.load()?.governance @ AmmError::Unauthorized)]
    pub governance: Signer<'info>,
    #[account(seeds = [PROTOCOL_TREASURY_SEED], bump)]
    pub protocol_treasury: AccountLoader<'info, ProtocolTreasury>,
    #[account(mut, seeds = [AMM_CONFIG_SEED], bump)]
    pub config: AccountLoader<'info, AmmConfig>,
}

///
///
pub fn validate_protocol_fee(zero_for_one: u16, one_for_zero: u16) -> Result<u32> {
    require!(
        u32::from(zero_for_one) <= MAX_PROTOCOL_FEE && u32::from(one_for_zero) <= MAX_PROTOCOL_FEE,
        AmmError::ProtocolFeeTooLarge
    );
    Ok(pack(u32::from(zero_for_one), u32::from(one_for_zero)))
}

pub fn handler(ctx: Context<SetProtocolFee>, zero_for_one: u16, one_for_zero: u16) -> Result<()> {
    let protocol_fee = validate_protocol_fee(zero_for_one, one_for_zero)?;
    let previous = {
        let mut c = ctx.accounts.config.load_mut()?;
        let previous = c.current_protocol_fee();
        c.protocol_fee = protocol_fee;
        previous
    };
    emit_cpi!(ProtocolFeeSet { previous, protocol_fee });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use carpenter_types::constants::PROTOCOL_FEE_DEFAULT_PACKED;
    use carpenter_types::protocol_fee::is_valid;

    fn name(r: Result<u32>) -> String {
        match r {
            Err(anchor_lang::error::Error::AnchorError(e)) => e.error_name,
            other => format!("{other:?}"),
        }
    }

///
///
///
///
    #[test]
    fn the_cap_is_1000_per_direction_and_zero_is_a_setting() {
        assert_eq!(validate_protocol_fee(500, 500).unwrap(), PROTOCOL_FEE_DEFAULT_PACKED);
        assert_eq!(validate_protocol_fee(1_000, 1_000).unwrap(), pack(1_000, 1_000));
        assert_eq!(validate_protocol_fee(250, 750).unwrap(), 250 | 750 << 12, "zero_for_one is the LOW nibble");
        assert_eq!(validate_protocol_fee(0, 0).unwrap(), 0, "try also 0 fee");
        assert_eq!(name(validate_protocol_fee(1_001, 0)), "ProtocolFeeTooLarge");
        assert_eq!(name(validate_protocol_fee(0, 1_001)), "ProtocolFeeTooLarge");
        assert_eq!(name(validate_protocol_fee(u16::MAX, u16::MAX)), "ProtocolFeeTooLarge");

        for (a, b) in [(0u16, 0u16), (1_000, 0), (0, 1_000), (1_000, 1_000), (999, 1)] {
            assert!(is_valid(validate_protocol_fee(a, b).unwrap()));
        }
    }
}
