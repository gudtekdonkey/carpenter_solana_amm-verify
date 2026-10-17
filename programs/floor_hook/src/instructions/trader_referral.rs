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
use anchor_lang::system_program;
use anchor_spl::associated_token::{self, get_associated_token_address_with_program_id, AssociatedToken};
use anchor_spl::token_interface::{self, Mint, TokenInterface, TransferChecked};
use floor_types::layout::{Layout, TraderBucket, TraderLink};

use crate::constants::{CARPENTER_AMM_PROGRAM, EVENT_AUTHORITY_SEED, FLOOR_LAUNCH_PROGRAM, HOOK_AUTHORITY_SEED};
use crate::errors::HookError;
use crate::events::{TraderReferralClaimed, TraderReferralSwept};
use crate::mech::amm_cpi::{self, AmmCpi};
use crate::mech::ratchet;
use crate::state::FloorPool;

///
pub const VAULT_AUTHORITY_SEED: &[u8] = floor_types::seeds::TRADER_REF_VAULT;
///
pub const CLAIM_POOL_ACCOUNTS: usize = 3;
///
const TOKEN_STATE_OFFSET: usize = 108;
const TOKEN_FROZEN: u8 = 2;
const TOKEN_AMOUNT_OFFSET: usize = 64;

///
pub fn vault_address(quote_mint: &Pubkey, token_program: &Pubkey) -> Pubkey {
    let (auth, _) = Pubkey::find_program_address(&[VAULT_AUTHORITY_SEED], &crate::ID);
    get_associated_token_address_with_program_id(&auth, quote_mint, token_program)
}

///
pub fn bucket_address(trader: &Pubkey, quote_mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[floor_types::seeds::TRADER_REF, trader.as_ref(), quote_mint.as_ref()], &crate::ID)
}





#[derive(Accounts)]
pub struct OpenTraderBucket<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
///
    pub trader: UncheckedAccount<'info>,
///
    pub trader_link: UncheckedAccount<'info>,
    pub quote_mint: InterfaceAccount<'info, Mint>,
///
    #[account(mut)]
    pub trader_bucket: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

///
///
pub fn read_link(info: &AccountInfo, trader: &Pubkey) -> Result<[[u8; 32]; 3]> {
    require_keys_eq!(*info.owner, FLOOR_LAUNCH_PROGRAM, HookError::TraderNotLinked);
    let data = info.try_borrow_data()?;
    require!(
        data.len() == TraderLink::ACCOUNT_LEN && data[..8] == TraderLink::DISCRIMINATOR,
        HookError::TraderNotLinked
    );
    let l: TraderLink = bytemuck::pod_read_unaligned(&data[8..TraderLink::ACCOUNT_LEN]);
    require!(l.trader == trader.to_bytes(), HookError::TraderNotLinked);
    let key = Pubkey::create_program_address(
        &[floor_types::seeds::TRADER_LINK, trader.as_ref(), &[l.bump]],
        &FLOOR_LAUNCH_PROGRAM,
    )
    .map_err(|_| error!(HookError::TraderNotLinked))?;
    require_keys_eq!(*info.key, key, HookError::TraderNotLinked);
    Ok([l.tier1, l.tier2, l.tier3])
}

pub fn open_trader_bucket_handler(ctx: Context<OpenTraderBucket>) -> Result<()> {
    let a = &ctx.accounts;
    let trader = a.trader.key();
    let tiers = read_link(&a.trader_link.to_account_info(), &trader)?;
    let quote = a.quote_mint.key();
    let (key, bump) = bucket_address(&trader, &quote);
    require_keys_eq!(a.trader_bucket.key(), key, HookError::TraderRefBadAccount);
    let target = a.trader_bucket.to_account_info();
    require!(*target.owner != crate::ID, HookError::TraderBucketOpen);


    let space = TraderBucket::ACCOUNT_LEN;
    let required = Rent::get()?.minimum_balance(space);
    let have = target.lamports();
    let system = a.system_program.to_account_info();
    if have < required {
        system_program::transfer(
            CpiContext::new(system.key(), system_program::Transfer { from: a.payer.to_account_info(), to: target.clone() }),
            required - have,
        )?;
    }
    let seeds: &[&[u8]] = &[floor_types::seeds::TRADER_REF, trader.as_ref(), quote.as_ref(), &[bump]];
    system_program::allocate(
        CpiContext::new_with_signer(system.key(), system_program::Allocate { account_to_allocate: target.clone() }, &[seeds]),
        space as u64,
    )?;
    system_program::assign(
        CpiContext::new_with_signer(system.key(), system_program::Assign { account_to_assign: target.clone() }, &[seeds]),
        &crate::ID,
    )?;
    let mut b: TraderBucket = bytemuck::Zeroable::zeroed();
    b.trader = trader.to_bytes();
    b.quote_mint = quote.to_bytes();
    (b.tier1, b.tier2, b.tier3) = (tiers[0], tiers[1], tiers[2]);
    b.bump = bump;
    let mut data = target.try_borrow_mut_data()?;
    data[..8].copy_from_slice(&TraderBucket::DISCRIMINATOR);
    data[8..space].copy_from_slice(bytemuck::bytes_of(&b));
    Ok(())
}





#[derive(Accounts)]
pub struct OpenTraderRefVault<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
///
    #[account(seeds = [VAULT_AUTHORITY_SEED], bump)]
    pub vault_authority: UncheckedAccount<'info>,
    pub quote_mint: InterfaceAccount<'info, Mint>,
///
    #[account(mut, address = get_associated_token_address_with_program_id(&vault_authority.key(), &quote_mint.key(), &token_program.key()) @ HookError::TraderRefBadAccount)]
    pub vault: UncheckedAccount<'info>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn open_trader_ref_vault_handler(ctx: Context<OpenTraderRefVault>) -> Result<()> {
    let a = &ctx.accounts;
    require_keys_eq!(*a.quote_mint.to_account_info().owner, a.token_program.key(), HookError::TraderRefBadAccount);
    associated_token::create_idempotent(CpiContext::new(
        a.associated_token_program.key(),
        associated_token::Create {
            payer: a.payer.to_account_info(),
            associated_token: a.vault.to_account_info(),
            authority: a.vault_authority.to_account_info(),
            mint: a.quote_mint.to_account_info(),
            system_program: a.system_program.to_account_info(),
            token_program: a.token_program.to_account_info(),
        },
    ))
}





///
pub struct SweepCtx<'a, 'info> {
    pub carpenter_amm: &'a AccountInfo<'info>,
    pub amm_event_authority: &'a AccountInfo<'info>,
    pub hook_authority: &'a AccountInfo<'info>,
    pub hook_authority_bump: u8,
    pub hook_program: &'a AccountInfo<'info>,
    pub vault: &'a AccountInfo<'info>,
    pub quote_mint: &'a AccountInfo<'info>,
    pub token_program: &'a AccountInfo<'info>,
}

fn token_state(info: &AccountInfo) -> Option<u8> {
    info.try_borrow_data().ok().and_then(|d| d.get(TOKEN_STATE_OFFSET).copied())
}

///
///
pub fn vault_ok(vault: &AccountInfo, quote_mint: &Pubkey, token_program: &Pubkey) -> bool {
    vault.is_writable && vault.owner == token_program && *vault.key == vault_address(quote_mint, token_program)
}

///
pub fn token_amount(info: &AccountInfo) -> Result<u64> {
    let d = info.try_borrow_data()?;
    let b = d.get(TOKEN_AMOUNT_OFFSET..TOKEN_AMOUNT_OFFSET + 8).ok_or(error!(HookError::TraderRefBadAccount))?;
    Ok(u64::from_le_bytes(b.try_into().unwrap()))
}

///
pub fn is_floor_pool(info: &AccountInfo) -> bool {
    info.owner == &crate::ID
        && info.data_len() >= FloorPool::LEN
        && info.try_borrow_data().map(|d| d[..8] == *FloorPool::DISCRIMINATOR).unwrap_or(false)
}

///
///
#[inline(never)]
pub fn sweep_pool<'a, 'info>(
    c: &SweepCtx<'a, 'info>,
    floor_pool: &AccountInfo<'info>,
    pool: &AccountInfo<'info>,
    vault0: &AccountInfo<'info>,
) -> Result<u64> {
    if !is_floor_pool(floor_pool) || !floor_pool.is_writable || !pool.is_writable || !vault0.is_writable {
        return Ok(0);
    }
    let mut s: FloorPool = {
        let d = floor_pool.try_borrow_data()?;
        bytemuck::pod_read_unaligned(&d[8..FloorPool::LEN])
    };
    let amount = s.trader_ref_owed;
    if amount == 0 || s.pool_bound != 1 || s.quote_mint != *c.quote_mint.key || s.amm_pool != *pool.key {
        return Ok(0);
    }
    let Ok(p) = amm_cpi::read_pool(pool) else { return Ok(0) };
    if amm_cpi::require_pool_side(&p, 0, vault0.key, c.quote_mint.key, c.token_program.key).is_err() {
        return Ok(0);
    }
    if token_state(vault0) == Some(TOKEN_FROZEN) || token_state(c.vault) == Some(TOKEN_FROZEN) {
        return Ok(0);
    }




    let Ok(before) = token_amount(vault0) else { return Ok(0) };
    if amount > p.hook_claims0 || u128::from(p.hook_claims0) + u128::from(p.protocol_fees_owed0) > u128::from(before) {
        return Ok(0);
    }
    if crate::instructions::observe_quote::mint_paused(c.quote_mint).unwrap_or(true) {
        return Ok(0);
    }




    s.trader_ref_owed = 0;
    if !ratchet::claims_cover_liabilities(&s, p.hook_claims0 - amount, p.hook_claims1) {
        return Ok(0);
    }
    {
        let mut d = floor_pool.try_borrow_mut_data()?;
        d[8..FloorPool::LEN].copy_from_slice(bytemuck::bytes_of(&s));
    }
    let cpi = AmmCpi {
        carpenter_amm: c.carpenter_amm.clone(),
        amm_event_authority: c.amm_event_authority.clone(),
        hook_authority: c.hook_authority.clone(),
        hook_authority_bump: c.hook_authority_bump,
        hook_program: c.hook_program.clone(),
        floor_pool: floor_pool.clone(),
        pool: pool.clone(),
    };
    let w = amm_cpi::withdraw(&cpi, vault0, c.vault, c.quote_mint, c.token_program, 0, amount)?;
    require!(ratchet::claims_cover_liabilities(&s, w.hook_claims0, w.hook_claims1), HookError::InvariantViolated);
    Ok(amount)
}

#[event_cpi]
#[derive(Accounts)]
pub struct SweepTraderReferral<'info> {
///
    pub anyone: Signer<'info>,
///
    #[account(mut)]
    pub floor_pool: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub pool: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub vault0: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub vault: UncheckedAccount<'info>,
///
    pub quote_mint: UncheckedAccount<'info>,
///
    pub token_program: UncheckedAccount<'info>,
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

pub fn sweep_trader_referral_handler(ctx: Context<SweepTraderReferral>) -> Result<()> {
    let a = &ctx.accounts;
    let (vault, mint, tp) = (a.vault.to_account_info(), a.quote_mint.to_account_info(), a.token_program.to_account_info());
    if !vault_ok(&vault, mint.key, tp.key) {
        return Ok(());
    }
    let (amm, amm_ev, ha, prog) = (
        a.carpenter_amm.to_account_info(),
        a.amm_event_authority.to_account_info(),
        a.hook_authority.to_account_info(),
        a.program.to_account_info(),
    );
    let c = SweepCtx {
        carpenter_amm: &amm,
        amm_event_authority: &amm_ev,
        hook_authority: &ha,
        hook_authority_bump: ctx.bumps.hook_authority,
        hook_program: &prog,
        vault: &vault,
        quote_mint: &mint,
        token_program: &tp,
    };
    let amount = sweep_pool(&c, &a.floor_pool.to_account_info(), &a.pool.to_account_info(), &a.vault0.to_account_info())?;
    if amount > 0 {
        emit_cpi!(TraderReferralSwept { pool: a.pool.key(), quote_mint: mint.key(), amount });
    }
    Ok(())
}





#[event_cpi]
#[derive(Accounts)]
pub struct ClaimTraderReferral<'info> {
///
    pub anyone: Signer<'info>,
///
    pub referrer: UncheckedAccount<'info>,
    pub quote_mint: InterfaceAccount<'info, Mint>,
///
    #[account(seeds = [VAULT_AUTHORITY_SEED], bump)]
    pub vault_authority: UncheckedAccount<'info>,
///
    #[account(mut)]
    pub vault: UncheckedAccount<'info>,
///
    #[account(mut, address = get_associated_token_address_with_program_id(&referrer.key(), &quote_mint.key(), &token_program.key()) @ HookError::TraderRefBadAccount)]
    pub payee: UncheckedAccount<'info>,
    pub token_program: Interface<'info, TokenInterface>,
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
pub fn bucket_due(b: &TraderBucket, referrer: &Pubkey, quote: &Pubkey) -> [u64; 3] {
    let mut due = [0u64; 3];
    if b.quote_mint != quote.to_bytes() || *referrer == Pubkey::default() {
        return due;
    }
    let r = referrer.to_bytes();
    for (i, t) in [b.tier1, b.tier2, b.tier3].iter().enumerate() {
        if *t == r {
            due[i] = b.owed[i];
        }
    }
    due
}

///
pub fn take_from(due: [u64; 3], left: &mut u64) -> [u64; 3] {
    let mut take = [0u64; 3];
    for i in 0..3 {
        take[i] = due[i].min(*left);
        *left -= take[i];
    }
    take
}

///
#[inline(never)]
fn settle_bucket(info: &AccountInfo, referrer: &Pubkey, quote: &Pubkey, left: &mut u64) -> Result<u64> {
    require!(
        info.owner == &crate::ID && info.data_len() == TraderBucket::ACCOUNT_LEN,
        HookError::TraderRefBadAccount
    );
    let mut b: TraderBucket = {
        let data = info.try_borrow_data()?;
        require!(data[..8] == TraderBucket::DISCRIMINATOR, HookError::TraderRefBadAccount);
        bytemuck::pod_read_unaligned(&data[8..TraderBucket::ACCOUNT_LEN])
    };
    let take = take_from(bucket_due(&b, referrer, quote), left);
    let paid = take[0] + take[1] + take[2];


    if paid > 0 {
        require!(info.is_writable, HookError::TraderRefBadAccount);
        for (owed, t) in b.owed.iter_mut().zip(take) {
            *owed -= t;
        }
        let mut data = info.try_borrow_mut_data()?;
        data[8..TraderBucket::ACCOUNT_LEN].copy_from_slice(bytemuck::bytes_of(&b));
    }
    Ok(paid)
}

///
///
pub fn split_claim_accounts<T>(ra: &[T], is_pool: impl Fn(&T) -> bool) -> (&[T], &[T]) {
    let mut n = 0;
    while n + CLAIM_POOL_ACCOUNTS <= ra.len() && is_pool(&ra[n]) {
        n += CLAIM_POOL_ACCOUNTS;
    }
    ra.split_at(n)
}

pub fn claim_trader_referral_handler<'info>(ctx: Context<'info, ClaimTraderReferral<'info>>) -> Result<()> {
    let a = &ctx.accounts;
    let quote = a.quote_mint.key();
    let tp = a.token_program.to_account_info();
    let (vault, mint) = (a.vault.to_account_info(), a.quote_mint.to_account_info());
    require_keys_eq!(*mint.owner, *tp.key, HookError::TraderRefBadAccount);
    require!(vault_ok(&vault, &quote, tp.key), HookError::TraderRefBadAccount);
    let payee = a.payee.to_account_info();
    require!(payee.owner == tp.key, HookError::TraderRefBadAccount);


    require_keys_neq!(*payee.key, *vault.key, HookError::TraderRefBadAccount);
    let referrer = a.referrer.key();

    let (pools, buckets) = split_claim_accounts(ctx.remaining_accounts, is_floor_pool);
    if !pools.is_empty() {
        let (amm, amm_ev, ha, prog) = (
            a.carpenter_amm.to_account_info(),
            a.amm_event_authority.to_account_info(),
            a.hook_authority.to_account_info(),
            a.program.to_account_info(),
        );
        let c = SweepCtx {
            carpenter_amm: &amm,
            amm_event_authority: &amm_ev,
            hook_authority: &ha,
            hook_authority_bump: ctx.bumps.hook_authority,
            hook_program: &prog,
            vault: &vault,
            quote_mint: &mint,
            token_program: &tp,
        };
        for t in pools.chunks(CLAIM_POOL_ACCOUNTS) {
            let amount = sweep_pool(&c, &t[0], &t[1], &t[2])?;
            if amount > 0 {
                emit_cpi!(TraderReferralSwept { pool: t[1].key(), quote_mint: quote, amount });
            }
        }
    }

    let mut left = token_amount(&vault)?;
    let mut paid = 0u64;
    for b in buckets {
        paid += settle_bucket(b, &referrer, &quote, &mut left)?;
    }
    if paid == 0 {
        return Ok(());
    }
    let bump = [ctx.bumps.vault_authority];
    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            tp.key(),
            TransferChecked {
                from: vault,
                mint,
                to: payee,
                authority: a.vault_authority.to_account_info(),
            },
            &[&[VAULT_AUTHORITY_SEED, &bump]],
        ),
        paid,
        a.quote_mint.decimals,
    )?;
    emit_cpi!(TraderReferralClaimed { referrer, quote_mint: quote, paid });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const Q: Pubkey = Pubkey::new_from_array([7; 32]);
    const R: Pubkey = Pubkey::new_from_array([2; 32]);

    fn bucket(tiers: [[u8; 32]; 3], owed: [u64; 3], quote: Pubkey) -> TraderBucket {
        let mut b: TraderBucket = bytemuck::Zeroable::zeroed();
        (b.tier1, b.tier2, b.tier3) = (tiers[0], tiers[1], tiers[2]);
        b.owed = owed;
        b.quote_mint = quote.to_bytes();
        b
    }

///
///
    #[test]
    fn a_referrer_is_due_only_its_own_tiers_in_its_own_quote() {
        let b = bucket([[2; 32], [3; 32], [4; 32]], [300, 50, 20], Q);
        assert_eq!(bucket_due(&b, &R, &Q), [300, 0, 0]);
        assert_eq!(bucket_due(&b, &Pubkey::new_from_array([3; 32]), &Q), [0, 50, 0]);
        assert_eq!(bucket_due(&b, &Pubkey::new_from_array([4; 32]), &Q), [0, 0, 20]);
        assert_eq!(bucket_due(&b, &Pubkey::new_from_array([9; 32]), &Q), [0, 0, 0], "a wrong tier pays 0");
        assert_eq!(bucket_due(&b, &R, &Pubkey::new_from_array([8; 32])), [0, 0, 0], "another quote pays 0");
        let unset = bucket([[2; 32], [0; 32], [0; 32]], [300, 7, 7], Q);
        assert_eq!(bucket_due(&unset, &Pubkey::default(), &Q), [0, 0, 0], "the zero key is never a payee");
    }

///
    #[test]
    fn a_partial_vault_pays_what_it_holds_in_tier_order() {
        let mut left = 320;
        assert_eq!(take_from([300, 50, 20], &mut left), [300, 20, 0]);
        assert_eq!(left, 0);
        let mut left = 1_000;
        assert_eq!(take_from([300, 50, 20], &mut left), [300, 50, 20]);
        assert_eq!(left, 630);
    }

///
    #[test]
    fn the_remaining_accounts_split_into_pool_triples_then_buckets() {
        let ra = [1, 0, 0, 1, 0, 0, 2, 2];
        let (p, b) = split_claim_accounts(&ra, |x| *x == 1);
        assert_eq!((p.len(), b.len()), (6, 2));
        let (p, b) = split_claim_accounts(&[2, 2], |x| *x == 1);
        assert_eq!((p.len(), b.len()), (0, 2));
        let (p, b) = split_claim_accounts(&[1, 0], |x| *x == 1);
        assert_eq!((p.len(), b.len()), (0, 2), "a pool needs its whole triple");
    }
}
