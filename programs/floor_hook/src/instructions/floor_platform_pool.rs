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
use anchor_spl::associated_token::{
    self, get_associated_token_address_with_program_id, AssociatedToken,
};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::{
    CARPENTER_AMM_PROGRAM, EVENT_AUTHORITY_SEED, FLOOR_PLATFORM_POOL_SEED, HOOK_AUTHORITY_SEED,
    HOOK_CONFIG_SEED, WSOL_MINT,
};
use crate::errors::HookError;
use crate::events::{FloorPoolDesignated, PlatformPoolCredit};
use crate::mech::amm_cpi::{self, AmmCpi};
use crate::mech::positions::add;
use crate::mech::ratchet;
use crate::state::{FloorPlatformPool, FloorPool, HookConfig};

///
pub const FLOOR_PLATFORM_POOL_VERSION: u8 = 1;



#[event_cpi]
#[derive(Accounts)]
pub struct DesignateFloorPool<'info> {
///
    #[account(constraint = launch_authority.key() == hook_config.load()?.launch_authority @ HookError::NotLauncher)]
    pub launch_authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(seeds = [HOOK_CONFIG_SEED], bump)]
    pub hook_config: AccountLoader<'info, HookConfig>,
///
    #[account(mut)]
    pub floor_pool: AccountLoader<'info, FloorPool>,
///
    #[account(init, payer = payer, space = FloorPlatformPool::LEN, seeds = [FLOOR_PLATFORM_POOL_SEED], bump)]
    pub floor_platform_pool: AccountLoader<'info, FloorPlatformPool>,
    pub system_program: Program<'info, System>,
}

///
pub fn check_designatable(s: &FloorPool) -> Result<()> {
    require_keys_eq!(s.quote_mint, WSOL_MINT, HookError::BadQuoteToken);


    require!(
        s.pool_bound == 0
            && s.platform_owed == 0
            && s.platform_paid_eth == 0
            && s.platform_swept == 0
            && s.vault_eth == 0,
        HookError::FloorPoolNotEligible
    );
    Ok(())
}

pub fn designate_handler(ctx: Context<DesignateFloorPool>) -> Result<()> {
    let mut s: FloorPool = *ctx.accounts.floor_pool.load()?;
    check_designatable(&s)?;
    s.platform_bps = 0;
    *ctx.accounts.floor_pool.load_mut()? = s;

    let mut d = ctx.accounts.floor_platform_pool.load_init()?;
    d.version = FLOOR_PLATFORM_POOL_VERSION;
    d.bump = ctx.bumps.floor_platform_pool;
    d.floor_pool = ctx.accounts.floor_pool.key();
    d.amm_pool = s.amm_pool;
    d.base_mint = s.base_mint;
    d.quote_mint = s.quote_mint;
    d.received = 0;
    d.credits = 0;
    d.designated_slot = Clock::get()?.slot;
    drop(d);

    emit_cpi!(FloorPoolDesignated {
        floor_pool: ctx.accounts.floor_pool.key(),
        amm_pool: s.amm_pool,
        base_mint: s.base_mint,
    });
    Ok(())
}



///
///
///
///
///
#[derive(Accounts)]
pub struct SetGenesisStair<'info> {
///
    #[account(constraint = launch_authority.key() == hook_config.load()?.launch_authority @ HookError::NotLauncher)]
    pub launch_authority: Signer<'info>,
    #[account(seeds = [HOOK_CONFIG_SEED], bump)]
    pub hook_config: AccountLoader<'info, HookConfig>,
    #[account(seeds = [FLOOR_PLATFORM_POOL_SEED], bump = floor_platform_pool.load()?.bump)]
    pub floor_platform_pool: AccountLoader<'info, FloorPlatformPool>,
///
    #[account(mut, address = floor_platform_pool.load()?.floor_pool @ HookError::GenesisStairRefused)]
    pub floor_pool: AccountLoader<'info, FloorPool>,
}

///
///
///
///
///
///
pub fn check_genesis_stair(s: &FloorPool, steps: u8, growth_bps: u16) -> Result<()> {
    require!(
        s.pool_bound == 0
            && s.ladder_seeded == 0
            && s.stair_kind == floor_types::constants::STAIR_KIND_PRESET
            && s.stair_ready == 0,
        HookError::GenesisStairRefused
    );
    require!(floor_math::stair::stair_shape_ok(steps, growth_bps), HookError::StairShapeOutOfRange);
    Ok(())
}

pub fn set_genesis_stair_handler(ctx: Context<SetGenesisStair>, steps: u8, growth_bps: u16, owner: u8) -> Result<()> {
    let mut s = ctx.accounts.floor_pool.load_mut()?;
    check_genesis_stair(&s, steps, growth_bps)?;


    require!(owner == 0 || owner == floor_types::constants::STAIR_OWNER_TEAM, HookError::StairOwnerBadConfig);


    require!(
        owner != floor_types::constants::STAIR_OWNER_TEAM || steps >= floor_types::constants::GENESIS_MIN_STAIR_STEPS,
        HookError::StairOwnerBadConfig
    );
    s.stair_steps = steps;
    s.stair_growth_bps = growth_bps;
    s.stair_kind = floor_types::constants::STAIR_KIND_GENESIS;
    s.stair_owner = owner;
    s.owner_owed = 0;
    s.owner_paid = 0;
    Ok(())
}



#[event_cpi]
#[derive(Accounts)]
pub struct CreditFloorPool<'info> {
///
    #[account(mut)]
    pub anyone: Signer<'info>,
    #[account(mut, seeds = [FLOOR_PLATFORM_POOL_SEED], bump = floor_platform_pool.load()?.bump)]
    pub floor_platform_pool: AccountLoader<'info, FloorPlatformPool>,
///
    #[account(mut)]
    pub source_floor_pool: AccountLoader<'info, FloorPool>,
///
    #[account(mut, address = source_floor_pool.load()?.amm_pool @ HookError::WrongPool)]
    pub source_pool: UncheckedAccount<'info>,
    #[account(mut)]
    pub source_vault0: Box<InterfaceAccount<'info, TokenAccount>>,
///
    #[account(mut, address = floor_platform_pool.load()?.floor_pool @ HookError::WrongPool)]
    pub floor_pool: AccountLoader<'info, FloorPool>,
///
    #[account(mut, address = floor_pool.load()?.amm_pool @ HookError::WrongPool)]
    pub floor_amm_pool: UncheckedAccount<'info>,
    #[account(mut)]
    pub floor_vault0: Box<InterfaceAccount<'info, TokenAccount>>,
///
///
    #[account(mut)]
    pub transit: UncheckedAccount<'info>,
    #[account(address = WSOL_MINT @ HookError::BadQuoteToken)]
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
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

///
pub fn transit_for(floor_platform_pool: &Pubkey, token_program: &Pubkey) -> Pubkey {
    get_associated_token_address_with_program_id(floor_platform_pool, &WSOL_MINT, token_program)
}

///
///
pub fn check_credit(
    source: &FloorPool,
    source_key: &Pubkey,
    target: &FloorPool,
    target_key: &Pubkey,
) -> Result<()> {
    require!(source.pool_bound == 1, HookError::NotBound);
    require_keys_neq!(*source_key, *target_key, HookError::FloorPoolNotEligible);

    require_keys_eq!(source.quote_mint, WSOL_MINT, HookError::BadQuoteToken);
    require_keys_eq!(target.quote_mint, WSOL_MINT, HookError::BadQuoteToken);


    require!(
        target.pool_bound == 1 && target.ladder_seeded == 1,
        HookError::FloorPoolNotLive
    );
    target.require_quote_live()?;
    Ok(())
}

///
///
pub fn book_credit(target: &mut FloorPool, credited: u64) -> Result<()> {
    target.vault_eth = add(target.vault_eth, credited)?;
    target.needs_ratchet = 1;
    Ok(())
}

pub fn credit_handler(ctx: Context<CreditFloorPool>) -> Result<()> {



    let amount = check_and_bind(&ctx.accounts)?;
    if amount == 0 {
        return Ok(());
    }
    open_transit(&ctx.accounts)?;
    withdraw_source(&ctx.accounts, ctx.bumps.hook_authority, amount)?;
    let (credited, source_pool, target_pool) = deposit_and_book(&ctx.accounts, amount)?;
    emit_cpi!(PlatformPoolCredit {
        source_pool,
        target_pool,
        amount: credited
    });
    Ok(())
}

///
#[inline(never)]
fn check_and_bind(a: &CreditFloorPool) -> Result<u64> {
    let s: FloorPool = *a.source_floor_pool.load()?;
    let t: FloorPool = *a.floor_pool.load()?;
    check_credit(&s, &a.source_floor_pool.key(), &t, &a.floor_pool.key())?;
    let sp = amm_cpi::read_pool(&a.source_pool.to_account_info())?;
    amm_cpi::require_pool_side(
        &sp,
        0,
        &a.source_vault0.key(),
        &a.quote_mint.key(),
        &a.spl_token.key(),
    )?;
    let tp = amm_cpi::read_pool(&a.floor_amm_pool.to_account_info())?;
    amm_cpi::require_pool_side(
        &tp,
        0,
        &a.floor_vault0.key(),
        &a.quote_mint.key(),
        &a.spl_token.key(),
    )?;
    Ok(s.platform_owed)
}

///
///
#[inline(never)]
fn open_transit<'info>(a: &CreditFloorPool<'info>) -> Result<()> {
    let designation = a.floor_platform_pool.key();
    require_keys_eq!(
        a.transit.key(),
        transit_for(&designation, &a.spl_token.key()),
        HookError::WrongDestination
    );
    if a.transit.data_is_empty() {
        associated_token::create_idempotent(CpiContext::new(
            a.associated_token_program.key(),
            associated_token::Create {
                payer: a.anyone.to_account_info(),
                associated_token: a.transit.to_account_info(),
                authority: a.floor_platform_pool.to_account_info(),
                mint: a.quote_mint.to_account_info(),
                system_program: a.system_program.to_account_info(),
                token_program: a.spl_token.to_account_info(),
            },
        ))?;
    }
    let info = a.transit.to_account_info();
    require_keys_eq!(*info.owner, a.spl_token.key(), HookError::WrongDestination);
    let acct = TokenAccount::try_deserialize(&mut &info.try_borrow_data()?[..])?;
    require_keys_eq!(acct.owner, designation, HookError::WrongDestination);
    require_keys_eq!(acct.mint, WSOL_MINT, HookError::WrongDestination);
    Ok(())
}

///
///
///
#[inline(never)]
fn withdraw_source<'info>(
    a: &CreditFloorPool<'info>,
    hook_authority_bump: u8,
    amount: u64,
) -> Result<()> {
    let mut s: FloorPool = *a.source_floor_pool.load()?;
    s.platform_owed = 0;
    s.platform_swept = add(s.platform_swept, amount)?;
    *a.source_floor_pool.load_mut()? = s;
    let cpi = AmmCpi {
        carpenter_amm: a.carpenter_amm.to_account_info(),
        amm_event_authority: a.amm_event_authority.to_account_info(),
        hook_authority: a.hook_authority.to_account_info(),
        hook_authority_bump,
        hook_program: a.program.to_account_info(),
        floor_pool: a.source_floor_pool.to_account_info(),
        pool: a.source_pool.to_account_info(),
    };
    let w = amm_cpi::withdraw(
        &cpi,
        &a.source_vault0.to_account_info(),
        &a.transit.to_account_info(),
        &a.quote_mint.to_account_info(),
        &a.spl_token.to_account_info(),
        0,
        amount,
    )?;
    require!(
        ratchet::claims_cover_liabilities(&s, w.hook_claims0, w.hook_claims1),
        HookError::InvariantViolated
    );
    Ok(())
}

///
///
///
#[inline(never)]
fn deposit_and_book<'info>(
    a: &CreditFloorPool<'info>,
    amount: u64,
) -> Result<(u64, Pubkey, Pubkey)> {
    let bump = a.floor_platform_pool.load()?.bump;
    let seeds: &[&[u8]] = &[FLOOR_PLATFORM_POOL_SEED, &[bump]];
    let r = amm_cpi::deposit_signed(
        &a.carpenter_amm.to_account_info(),
        &a.amm_event_authority.to_account_info(),
        &a.floor_amm_pool.to_account_info(),
        &a.floor_platform_pool.to_account_info(),
        &a.transit.to_account_info(),
        &a.floor_vault0.to_account_info(),
        &a.quote_mint.to_account_info(),
        &a.spl_token.to_account_info(),
        0,
        amount,
        &[seeds],
    )?;
    let credited = amm_cpi::pos_to_u64(r.delta.amount0)?;
    require!(credited == amount, HookError::InvariantViolated);
    let mut t: FloorPool = *a.floor_pool.load()?;
    book_credit(&mut t, credited)?;
    require!(
        ratchet::claims_cover_liabilities(&t, r.hook_claims0, r.hook_claims1),
        HookError::InvariantViolated
    );
    *a.floor_pool.load_mut()? = t;
    {
        let mut d = a.floor_platform_pool.load_mut()?;
        d.received = add(d.received, credited)?;
        d.credits = add(d.credits, 1)?;
    }
    let source_pool = a.source_floor_pool.load()?.amm_pool;
    Ok((credited, source_pool, t.amm_pool))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pool(quote: Pubkey, bound: u8) -> FloorPool {
        let mut p: FloorPool = bytemuck::Zeroable::zeroed();
        p.quote_mint = quote;
        p.pool_bound = bound;
        p.ladder_seeded = bound;
        p.platform_bps = 250;
        p
    }

    fn code(r: Result<()>) -> u32 {
        match r.unwrap_err() {
            anchor_lang::error::Error::AnchorError(e) => e.error_code_number,
            e => panic!("not an anchor error: {e:?}"),
        }
    }

    fn hook(e: HookError) -> u32 {
        7000 + e as u32
    }

    #[test]
    fn sgc_01_designation_needs_a_fresh_unbound_wsol_pool() {
        assert!(check_designatable(&pool(WSOL_MINT, 0)).is_ok());
        assert_eq!(
            code(check_designatable(&pool(Pubkey::new_unique(), 0))),
            hook(HookError::BadQuoteToken)
        );
        assert_eq!(
            code(check_designatable(&pool(WSOL_MINT, 1))),
            hook(HookError::FloorPoolNotEligible)
        );
        let mut p = pool(WSOL_MINT, 0);
        p.platform_paid_eth = 1;
        assert_eq!(
            code(check_designatable(&p)),
            hook(HookError::FloorPoolNotEligible)
        );
        let mut p = pool(WSOL_MINT, 0);
        p.vault_eth = 1;
        assert_eq!(
            code(check_designatable(&p)),
            hook(HookError::FloorPoolNotEligible)
        );
    }

    #[test]
    fn sgc_02_credit_refuses_by_name() {
        let (sk, tk) = (Pubkey::new_unique(), Pubkey::new_unique());
        let src = pool(WSOL_MINT, 1);
        let live = pool(WSOL_MINT, 1);
        assert!(check_credit(&src, &sk, &live, &tk).is_ok());

        assert_eq!(
            code(check_credit(&src, &sk, &pool(WSOL_MINT, 0), &tk)),
            hook(HookError::FloorPoolNotLive)
        );

        assert_eq!(
            code(check_credit(
                &pool(Pubkey::new_unique(), 1),
                &sk,
                &live,
                &tk
            )),
            hook(HookError::BadQuoteToken)
        );

        assert_eq!(
            code(check_credit(&src, &tk, &live, &tk)),
            hook(HookError::FloorPoolNotEligible)
        );

        let mut prepared = pool(WSOL_MINT, 1);
        prepared.ladder_seeded = 0;
        assert_eq!(
            code(check_credit(&src, &sk, &prepared, &tk)),
            hook(HookError::FloorPoolNotLive)
        );

        assert_eq!(
            code(check_credit(&pool(WSOL_MINT, 0), &sk, &live, &tk)),
            hook(HookError::NotBound)
        );

        let halted = FloorPool {
            quote_suspended: 1,
            ..pool(WSOL_MINT, 1)
        };
        assert_eq!(
            code(check_credit(&src, &sk, &halted, &tk)),
            hook(HookError::QuoteSuspended)
        );
    }

    #[test]
    fn sgc_03_a_credit_is_backing_and_asks_for_a_ratchet() {
        let mut t = pool(WSOL_MINT, 1);
        t.vault_eth = 7;
        book_credit(&mut t, 5).unwrap();
        assert_eq!(t.vault_eth, 12);
        assert_eq!(t.needs_ratchet, 1);
        assert_eq!(
            t.platform_owed, 0,
            "no intake: $FLOOR takes no cut of its own backing"
        );
        t.vault_eth = u64::MAX;
        assert!(book_credit(&mut t, 1).is_err());
    }

    #[test]
    fn sgc_04_the_transit_is_the_designations_wsol_ata() {
        let (d, _) = Pubkey::find_program_address(&[FLOOR_PLATFORM_POOL_SEED], &crate::ID);
        let tok = anchor_spl::token::ID;
        assert_eq!(
            transit_for(&d, &tok),
            get_associated_token_address_with_program_id(&d, &WSOL_MINT, &tok)
        );
        let (pt, _) =
            Pubkey::find_program_address(&[crate::constants::PLATFORM_TREASURY_SEED], &crate::ID);
        assert_ne!(
            transit_for(&d, &tok),
            transit_for(&pt, &tok),
            "never the treasury's account"
        );
    }

///
///
///
///
    #[test]
    fn sgc_05_platform_bps_has_exactly_two_writers() {
        fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
            for e in std::fs::read_dir(dir).expect("read src/") {
                let p = e.expect("entry").path();
                if p.is_dir() {
                    walk(&p, out);
                } else if p.extension().is_some_and(|x| x == "rs") {
                    out.push(p);
                }
            }
        }
        let mut files = Vec::new();
        walk(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
            &mut files,
        );
        assert!(files.len() > 10);
        let mut hits = Vec::new();
        for f in &files {
            let text = std::fs::read_to_string(f).expect("read");

            let prod = text.split("#[cfg(test)]").next().unwrap_or("");
            for (n, line) in prod.lines().enumerate() {
                let code = line.split("//").next().unwrap_or("");
                if let Some(i) = code.find(".platform_bps") {
                    let after = code[i + ".platform_bps".len()..].trim_start();
                    if after.starts_with('=') && !after.starts_with("==") {
                        hits.push(format!(
                            "{}:{}",
                            f.file_name().unwrap().to_string_lossy(),
                            n + 1
                        ));
                    }
                }
            }
        }
        assert_eq!(hits.len(), 2, "platform_bps writers: {hits:?}");
        assert!(
            hits.iter().any(|h| h.starts_with("register_pool.rs:")),
            "{hits:?}"
        );
        assert!(
            hits.iter()
                .any(|h| h.starts_with("floor_platform_pool.rs:")),
            "{hits:?}"
        );
    }

///
///
///
///
    #[test]
    fn sgs_01_the_genesis_staircase_is_set_once_before_the_pool_is_bound() {
        assert_eq!(hook(HookError::GenesisStairRefused), 7053);
        let fresh = pool(WSOL_MINT, 0);
        assert!(check_genesis_stair(&fresh, 10, 9_000).is_ok());
        assert_eq!(code(check_genesis_stair(&pool(WSOL_MINT, 1), 10, 9_000)), hook(HookError::GenesisStairRefused));

        let mut custom = fresh;
        custom.stair_steps = floor_math::stair::custom_stair_steps(1_000_000);
        custom.stair_growth_bps = 10_000;
        assert!(check_genesis_stair(&custom, 10, 9_000).is_ok(), "a Custom launch's unplaced staircase is re-shaped");

        let mut again = fresh;
        again.stair_steps = 10;
        again.stair_kind = floor_types::constants::STAIR_KIND_GENESIS;
        assert_eq!(code(check_genesis_stair(&again, 10, 9_000)), hook(HookError::GenesisStairRefused));
        let mut ready = fresh;
        ready.stair_ready = 1;
        assert_eq!(code(check_genesis_stair(&ready, 10, 9_000)), hook(HookError::GenesisStairRefused));
        for (n, g) in [(0u8, 9_000u16), (17, 9_000), (10, 4_999), (10, 20_001)] {
            assert_eq!(code(check_genesis_stair(&fresh, n, g)), hook(HookError::StairShapeOutOfRange), "{n} {g}");
        }
        for n in 1..=16u8 {
            assert!(check_genesis_stair(&fresh, n, floor_types::constants::GENESIS_STAIR_GROWTH_BPS).is_ok());
        }
    }
}
