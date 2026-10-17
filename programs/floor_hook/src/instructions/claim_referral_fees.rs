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
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::{CARPENTER_AMM_PROGRAM, EVENT_AUTHORITY_SEED, HOOK_AUTHORITY_SEED};
use crate::errors::HookError;
use crate::events::{ReferralPaid, ReferralTierPaid};
use crate::mech::amm_cpi::{self, AmmCpi};
use crate::mech::ratchet;
use crate::state::FloorPool;

///
///
///
///
#[event_cpi]
#[derive(Accounts)]
pub struct ClaimReferralFees<'info> {
///
    pub anyone: Signer<'info>,
    #[account(mut)]
    pub floor_pool: AccountLoader<'info, FloorPool>,
///
    #[account(mut, address = floor_pool.load()?.amm_pool @ HookError::WrongPool)]
    pub pool: UncheckedAccount<'info>,
///
///
///
///
///
    #[account(mut)]
    pub vault0: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub destination: Box<InterfaceAccount<'info, TokenAccount>>,
///
    #[account(address = floor_pool.load()?.quote_mint @ HookError::WrongPool)]
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
    pub spl_token: Interface<'info, TokenInterface>,
///
    #[account(address = CARPENTER_AMM_PROGRAM)]
    pub carpenter_amm: UncheckedAccount<'info>,
///
    #[account(seeds = [HOOK_AUTHORITY_SEED], bump)]
    pub hook_authority: UncheckedAccount<'info>,
///
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = CARPENTER_AMM_PROGRAM)]
    pub amm_event_authority: UncheckedAccount<'info>,
}

///
///
pub fn tier_to_pay(s: &FloorPool, tier: u8) -> Result<(Pubkey, u64)> {
    let state: &floor_math::FloorState = bytemuck::cast_ref(s);
    let (Some(wallet), Some(due)) =
        (floor_math::referral_tier_wallet(state, tier), floor_math::referral_tier_due(state, tier))
    else {
        return err!(HookError::BadReferralTier);
    };
    let wallet = Pubkey::new_from_array(wallet);
    require_keys_neq!(wallet, Pubkey::default(), HookError::WrongDestination);
    Ok((wallet, due))
}

///
///
pub fn referrer_to_pay(s: &FloorPool) -> Result<Pubkey> {
    tier_to_pay(s, 1).map(|(w, _)| w)
}

pub fn handler(ctx: Context<ClaimReferralFees>, tier: u8) -> Result<()> {
    let mut s: FloorPool = *ctx.accounts.floor_pool.load()?;
    let (referrer, amount) = tier_to_pay(&s, tier)?;

    if amount == 0 {




        return Ok(());
    }
    require!(s.pool_bound == 1, HookError::NotBound);

    let a = &ctx.accounts;
    let p = amm_cpi::read_pool(&a.pool.to_account_info())?;
    amm_cpi::require_pool_side(&p, 0, &a.vault0.key(), &a.quote_mint.key(), &a.spl_token.key())?;
    require_keys_eq!(a.destination.owner, referrer, HookError::WrongDestination);
    require_keys_eq!(a.destination.mint, s.quote_mint, HookError::WrongDestination);



    s.referral_owed = s.referral_owed.checked_sub(amount).ok_or(HookError::MathOverflow)?;
    if tier == 2 {
        s.referral_paid_t2 = s.referral_paid_t2.checked_add(amount).ok_or(HookError::MathOverflow)?;
    }
    if tier == 3 {
        s.referral_paid_t3 = s.referral_paid_t3.checked_add(amount).ok_or(HookError::MathOverflow)?;
    }
    let pool = s.amm_pool;
    let destination = a.destination.key();
    *ctx.accounts.floor_pool.load_mut()? = s;

    let a = &ctx.accounts;
    let cpi = AmmCpi {
        carpenter_amm: a.carpenter_amm.to_account_info(),
        amm_event_authority: a.amm_event_authority.to_account_info(),
        hook_authority: a.hook_authority.to_account_info(),
        hook_authority_bump: ctx.bumps.hook_authority,
        hook_program: a.program.to_account_info(),
        floor_pool: a.floor_pool.to_account_info(),
        pool: a.pool.to_account_info(),
    };
    let w = amm_cpi::withdraw(
        &cpi,
        &a.vault0.to_account_info(),
        &a.destination.to_account_info(),
        &a.quote_mint.to_account_info(),
        &a.spl_token.to_account_info(),
        0,
        amount,
    )?;



    require!(
        ratchet::claims_cover_liabilities(&s, w.hook_claims0, w.hook_claims1),
        HookError::InvariantViolated
    );

    emit_cpi!(ReferralPaid { pool, referrer, destination, amount });
    emit_cpi!(ReferralTierPaid { pool, tier, referrer, amount });
    Ok(())
}

#[cfg(test)]
mod tests {
//!
//!
//!
    use super::*;

    const REFERRER: Pubkey = Pubkey::new_from_array([0x42; 32]);

    fn pool(referrer: Pubkey) -> FloorPool {
        let mut s: FloorPool = bytemuck::Zeroable::zeroed();
        s.referrer = referrer;
        s
    }

///
///
    #[test]
    fn an_unreferred_pool_is_refused_and_only_the_stored_referrer_is_paid() {
        assert_eq!(referrer_to_pay(&pool(Pubkey::default())).unwrap_err(), error!(HookError::WrongDestination));
        assert_eq!(referrer_to_pay(&pool(REFERRER)).unwrap(), REFERRER);
    }

///
///
///
///
///
    #[test]
    fn each_tier_pays_its_own_wallet_its_own_due() {
        let (t2, t3) = (Pubkey::new_from_array([0x52; 32]), Pubkey::new_from_array([0x53; 32]));
        let mut s = pool(REFERRER);
        s.referrer_t2 = t2;
        s.referrer_t3 = t3;
        s.referral_fees_paid = 500;
        s.referral_owed = 500;
        assert_eq!(tier_to_pay(&s, 1).unwrap(), (REFERRER, 450));
        assert_eq!(tier_to_pay(&s, 2).unwrap(), (t2, 45));
        assert_eq!(tier_to_pay(&s, 3).unwrap(), (t3, 5));
        for bad in [0u8, 4, 255] {
            assert_eq!(tier_to_pay(&s, bad).unwrap_err(), error!(HookError::BadReferralTier), "tier {bad}");
        }

        s.referrer_t3 = Pubkey::default();
        assert_eq!(tier_to_pay(&s, 3).unwrap_err(), error!(HookError::WrongDestination));
        assert_eq!(tier_to_pay(&s, 2).unwrap(), (t2, 50));
    }
}
