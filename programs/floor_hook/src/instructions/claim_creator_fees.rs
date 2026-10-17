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

use crate::constants::{EVENT_AUTHORITY_SEED, CARPENTER_AMM_PROGRAM, HOOK_AUTHORITY_SEED};
use crate::errors::HookError;
use crate::events::CreatorClaimed;
use crate::mech::amm_cpi::{self, AmmCpi};
use crate::mech::positions::add;
use crate::mech::ratchet;
use crate::state::{slots, FloorPool};

#[event_cpi]
#[derive(Accounts)]
pub struct ClaimCreatorFees<'info> {
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
    #[account(mut)]
    pub destination: Box<InterfaceAccount<'info, TokenAccount>>,
///
///
///
///
///
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

pub fn handler(ctx: Context<ClaimCreatorFees>, slot: u8) -> Result<()> {
    let mut s: FloorPool = *ctx.accounts.floor_pool.load()?;





    let fp = ctx.accounts.floor_pool.to_account_info();
    let mut t = slots::read(&fp)?;
    let (paid, creator) = creator_due(&s, &t, slot)?;

    if paid == 0 {
        return Ok(());
    }
    require!(s.pool_bound == 1, HookError::NotBound);

    let a = &ctx.accounts;
    let p = amm_cpi::read_pool(&a.pool.to_account_info())?;
    amm_cpi::require_pool_side(&p, 0, &a.vault0.key(), &a.quote_mint.key(), &a.spl_token.key())?;
    require_keys_eq!(a.destination.owner, creator, HookError::WrongDestination);
    require_keys_eq!(a.destination.mint, s.quote_mint, HookError::WrongDestination);

    book_creator_claim(&mut s, &mut t, slot, paid)?;
    let pool = s.amm_pool;
    slots::write(&fp, &t)?;
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
        paid,
    )?;
    require!(
        ratchet::claims_cover_liabilities(&s, w.hook_claims0, w.hook_claims1),
        HookError::InvariantViolated
    );

    emit_cpi!(CreatorClaimed { pool, slot, creator, amount: paid });
    Ok(())
}

///
///
///
pub(crate) fn creator_due(s: &FloorPool, t: &slots::PayeeSlots, slot: u8) -> Result<(u64, Pubkey)> {
    let sl = slots::slot(t, slots::SLOT_KIND_CREATOR, slot)?;
    require!(!sl.is_pending(), HookError::RecipientPendingClaim);
    let wallet = Pubkey::new_from_array(sl.key);


    let Some(lifetime) = s.creator_fees_paid.checked_add(s.creator_fees_owed) else {
        return Ok((0, wallet));
    };
    Ok((floor_math::slot_due(lifetime, sl.bps, sl.paid, s.creator_fees_owed), wallet))
}

///
///
pub(crate) fn book_creator_claim(s: &mut FloorPool, t: &mut slots::PayeeSlots, slot: u8, paid: u64) -> Result<()> {
    s.creator_fees_owed -= paid;
    s.creator_fees_paid = add(s.creator_fees_paid, paid)?;
    if let Some(m) = t.slot_mut(slots::SLOT_KIND_CREATOR, slot) {
        m.paid = add(m.paid, paid)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use slots::{PayeeSlots, SlotArg, SLOT_KIND_CREATOR};

    fn code(e: anchor_lang::error::Error) -> u32 {
        match e {
            anchor_lang::error::Error::AnchorError(a) => a.error_code_number,
            e => panic!("not an anchor error: {e:?}"),
        }
    }
    fn list(args: &[(u16, u8, u8)]) -> PayeeSlots {
        let a: Vec<SlotArg> = args.iter().map(|&(bps, platform, b)| SlotArg { bps, platform, key: [b; 32] }).collect();
        let mut t = PayeeSlots::default();
        t.fill(SLOT_KIND_CREATOR, &a);
        t
    }
    fn pool(owed: u64, paid: u64) -> FloorPool {
        let mut s: FloorPool = bytemuck::Zeroable::zeroed();
        s.creator_fees_owed = owed;
        s.creator_fees_paid = paid;
        s
    }
///
    fn claim(s: &mut FloorPool, t: &mut PayeeSlots, i: u8) -> u64 {
        let (due, _) = creator_due(s, t, i).unwrap();
        book_creator_claim(s, t, i, due).unwrap();
        due
    }

///
    #[test]
    fn mr_s_one_slot_at_ten_thousand_pays_the_whole_owed() {
        let mut t = list(&[(10_000, 0, 1)]);
        let mut s = pool(123_456_789, 0);
        let (due, who) = creator_due(&s, &t, 0).unwrap();
        assert_eq!((due, who), (123_456_789, Pubkey::new_from_array([1; 32])));
        book_creator_claim(&mut s, &mut t, 0, due).unwrap();
        assert_eq!((s.creator_fees_owed, s.creator_fees_paid, t.creator[0].paid), (0, 123_456_789, 123_456_789));
        assert_eq!(creator_due(&s, &t, 0).unwrap().0, 0, "nothing owed, nothing due");
        s.creator_fees_owed = 77;
        assert_eq!(claim(&mut s, &mut t, 0), 77);
        assert_eq!((s.creator_fees_owed, s.creator_fees_paid, t.creator[0].paid), (0, 123_456_866, 123_456_866));
    }

///
    #[test]
    fn mr_s_the_slot_due_is_its_lifetime_share_less_what_it_was_paid() {
        let mut t = list(&[(6_000, 0, 1), (4_000, 0, 2)]);
        let mut s = pool(1_000, 0);
        assert_eq!(claim(&mut s, &mut t, 1), 400);
        assert_eq!((s.creator_fees_owed, s.creator_fees_paid, t.creator[1].paid), (600, 400, 400));
        assert_eq!(creator_due(&s, &t, 1).unwrap().0, 0, "slot 1 took its whole share of L = 1,000");
        s.creator_fees_owed += 1_000;
        assert_eq!(creator_due(&s, &t, 1).unwrap().0, 400, "800 earned − 400 paid");
        assert_eq!(claim(&mut s, &mut t, 0), 1_200, "slot 0: 60 % of 2,000, never claimed");
        assert_eq!(claim(&mut s, &mut t, 1), 400);
        assert_eq!((s.creator_fees_owed, s.creator_fees_paid), (0, 2_000));

        let mut s = pool(10, 990);
        let t = list(&[(10_000, 0, 1)]);
        assert_eq!(creator_due(&s, &t, 0).unwrap().0, 10);
        s.creator_fees_owed = 0;
        assert_eq!(creator_due(&s, &t, 0).unwrap().0, 0);
    }

///
    #[test]
    fn mr_s_a_pending_slot_is_refused_alone_and_a_missing_slot_is_bad_slot() {
        let mut t = list(&[(5_000, 0, 1), (5_000, 1, 2)]);
        let mut s = pool(1_000, 0);
        let pending = 7000 + HookError::RecipientPendingClaim as u32;
        let bad_slot = 7000 + HookError::BadSlot as u32;
        assert_eq!(code(creator_due(&s, &t, 1).unwrap_err()), pending);
        assert_eq!(claim(&mut s, &mut t, 0), 500, "the address slot is unaffected");
        for i in [2u8, 4, 5, 255] {
            assert_eq!(code(creator_due(&s, &t, i).unwrap_err()), bad_slot, "slot {i}");
        }

        let id = slots::resolve_slot(&mut t, SLOT_KIND_CREATOR, 1, Pubkey::new_from_array([9; 32]), HookError::AlreadyResolved).unwrap();
        assert_eq!(id, [2; 32]);
        let (due, who) = creator_due(&s, &t, 1).unwrap();
        assert_eq!((due, who), (500, Pubkey::new_from_array([9; 32])));
    }

///
///
///
///
    #[test]
    fn mr_s_corrupt_books_whose_lifetime_overflows_pay_nothing() {
        let t = list(&[(10_000, 0, 7)]);
        let s = pool(u64::MAX, 1);
        let (due, who) = creator_due(&s, &t, 0).unwrap();
        assert_eq!((due, who), (0, Pubkey::new_from_array([7; 32])));

        assert_eq!(code(creator_due(&s, &t, 1).unwrap_err()), 7000 + HookError::BadSlot as u32);
        let p = list(&[(10_000, 2, 7)]);
        assert_eq!(code(creator_due(&s, &p, 0).unwrap_err()), 7000 + HookError::RecipientPendingClaim as u32);

        assert_eq!(creator_due(&pool(u64::MAX - 1, 1), &t, 0).unwrap().0, u64::MAX - 1);
    }

    #[test]
    fn mr_s_three_slots_leave_at_most_two_units_of_dust() {
        let mut t = list(&[(3_333, 0, 1), (3_333, 0, 2), (3_334, 0, 3)]);
        let mut s = pool(0, 0);
        let mut x = 0x9E37_79B9_7F4A_7C15u64;
        for round in 0..5_000u64 {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            s.creator_fees_owed += x % 100_003 + (round % 7);
            for k in 0..3u8 {
                let i = ((round as u8).wrapping_add(k)) % 3;
                if (x >> (8 + k)) & 1 == 1 || round % 11 == 0 {
                    claim(&mut s, &mut t, i);
                }
            }
            let l = s.creator_fees_paid + s.creator_fees_owed;
            let sum: u64 = t.creator[..3].iter().map(|c| c.paid).sum();
            assert_eq!(sum, s.creator_fees_paid, "Σ slot.paid is the pool's paid");
            assert!(sum <= l, "never more than the lifetime");
        }
        for i in 0..3 {
            claim(&mut s, &mut t, i);
        }
        let l = s.creator_fees_paid + s.creator_fees_owed;
        let sum: u64 = t.creator[..3].iter().map(|c| c.paid).sum();
        assert!(l - sum <= 2, "dust {} at L = {l}", l - sum);
        assert_eq!(s.creator_fees_owed, l - sum, "the dust stays owed on the pool");
        for (i, c) in t.creator[..3].iter().enumerate() {
            assert_eq!(c.paid, floor_math::slots::slot_earned(l, c.bps), "slot {i} holds exactly its earned share");
        }
    }
}
