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
use anchor_spl::token_2022::spl_token_2022::extension::transfer_hook::TransferHook;
use anchor_spl::token_2022::spl_token_2022::extension::{BaseStateWithExtensions, ExtensionType, StateWithExtensions};
use anchor_spl::token_2022::spl_token_2022::state::Mint as MintState;
use anchor_spl::token_interface::Mint;
use floor_types::constants::{
    DYNAMIC_FEE_FLAG, FLOOR_TICK_SPACING, MAX_CREATOR_BPS, MAX_HOLDER_BPS, MAX_SKIM_FAR_BPS, MAX_TICK,
    MAX_BOUNTY_BPS, MAX_BOUNTY_LAMPORTS_CEIL, MAX_QUOTE_DECIMALS, MIN_BID_LAMPORTS_CEIL,
    MIN_BID_LAMPORTS_FLOOR, MIN_QUOTE_DECIMALS,
    PLATFORM_BPS, DEFAULT_SKIM_BUY_AT_FLOOR_BPS, skim_sell_at_floor_bps_at, MAX_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS,
    MIN_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS, MAX_SKIM_SELL_AT_FLOOR_BPS,
    REFERRAL_CREATOR_SHARE_BPS, TEAM_POOL_FEE_BPS_MAX,
};
use floor_types::layout::{
    AmmConfig, ApprovedHook as ApprovedHookLayout, Layout, MintPoolCount, QuoteMint as QuoteMintLayout,
    QuotePolicy as QuotePolicyLayout, QUOTE_STATE_HOOK_LISTED, QUOTE_STATE_LISTED, QUOTE_STATE_RETIRED,
};

use crate::constants::{
    AMM_CONFIG_SEED, APPROVED_HOOK_SEED, EVENT_AUTHORITY_SEED, CARPENTER_AMM_PROGRAM, FLOOR_HOLDERS_PROGRAM, FLOOR_POOL_SEED,
    HOOK_AUTHORITY_SEED, HOOK_CONFIG_SEED, LOCK_AUTHORITY_SEED, MINT_EXCLUSIVITY_SEED, MINT_POOLS_SEED, QUOTE_MINT_SEED,
    MAX_PRESET_ID, QUOTE_POLICY_SEED, WSOL_MINT,
};
use crate::errors::HookError;
use crate::events::{PoolReferralTiers, PoolRegistered, PoolRewardsTerms, QuoteAdmitted};
use crate::instructions::quote_lineage;
use crate::mech::amm_cpi;
use crate::mech::positions::math;
use crate::state::{FloorPool, HookConfig, RegisterPoolArgs};
use crate::{ReferralTiersArg, SkimSellMultipleArg, StairArg, TeamFeeArg};

#[event_cpi]
#[derive(Accounts)]
#[instruction(args: RegisterPoolArgs)]
pub struct RegisterPool<'info> {
    #[account(constraint = launch_authority.key() == hook_config.load()?.launch_authority @ HookError::NotLauncher)]
    pub launch_authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(seeds = [HOOK_CONFIG_SEED], bump)]
    pub hook_config: AccountLoader<'info, HookConfig>,
    #[account(
        init,
        payer = payer,




        space = FloorPool::LEN_WITH_PARTS,
        seeds = [FLOOR_POOL_SEED, args.amm_pool.as_ref()],
        bump,
    )]
    pub floor_pool: AccountLoader<'info, FloorPool>,
///
    #[account(address = CARPENTER_AMM_PROGRAM)]
    pub carpenter_amm: UncheckedAccount<'info>,
///
    #[account(seeds = [AMM_CONFIG_SEED], bump, seeds::program = CARPENTER_AMM_PROGRAM)]
    pub amm_config: UncheckedAccount<'info>,
///
    #[account(seeds = [APPROVED_HOOK_SEED, crate::ID.as_ref()], bump, seeds::program = CARPENTER_AMM_PROGRAM)]
    pub approved_hook: UncheckedAccount<'info>,
    #[account(address = args.base_mint @ HookError::BadConfig)]
    pub base_mint: Box<InterfaceAccount<'info, Mint>>,
///
    #[account(seeds = [MINT_POOLS_SEED, base_mint.key().as_ref()], bump, seeds::program = CARPENTER_AMM_PROGRAM)]
    pub mint_pools: UncheckedAccount<'info>,
///
    #[account(mut, seeds = [MINT_EXCLUSIVITY_SEED, base_mint.key().as_ref()], bump, seeds::program = CARPENTER_AMM_PROGRAM)]
    pub mint_exclusivity: UncheckedAccount<'info>,
///
    #[account(seeds = [HOOK_AUTHORITY_SEED], bump)]
    pub hook_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
///
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = CARPENTER_AMM_PROGRAM)]
    pub amm_event_authority: UncheckedAccount<'info>,
}

///
///
///
///
///
///
///
///
///
///
///
///
///
pub fn validate(args: &RegisterPoolArgs) -> Result<()> {




    require!(args.mode <= MAX_PRESET_ID, HookError::BadConfig);
    require!(
        args.skim_buy_far_bps <= MAX_SKIM_FAR_BPS && args.skim_sell_far_bps <= MAX_SKIM_FAR_BPS,
        HookError::BadConfig
    );
    require!(
        args.creator_buy_bps <= MAX_CREATOR_BPS && args.creator_sell_bps <= MAX_CREATOR_BPS,
        HookError::BadConfig
    );









    require!(
        args.holder_buy_bps <= MAX_HOLDER_BPS && args.holder_sell_bps <= MAX_HOLDER_BPS,
        HookError::BadConfig
    );

























    require!(args.platform_bps == PLATFORM_BPS, HookError::BadConfig);
    require!(args.min_sellable > 0, HookError::BadConfig);


    require!(args.bounty_bps <= MAX_BOUNTY_BPS, HookError::BountyOutOfRange);














    require!(args.pad == [0u8; 32], HookError::BadConfig);










    require_keys_neq!(args.quote_mint, args.base_mint, HookError::BadConfig);
    require!(args.base_mint.to_bytes() > args.quote_mint.to_bytes(), HookError::BadConfig);
    require_keys_neq!(args.pool_initializer, Pubkey::default(), HookError::BadConfig);
    Ok(())
}

///
///
///
///
///
///
///
pub fn validate_team_fee(team: &crate::TeamFeeArg) -> Result<()> {
    require!(team.team_fee_bps <= TEAM_POOL_FEE_BPS_MAX, HookError::TeamFeeOutOfRange);
    Ok(())
}



///
///
///
///
///
pub fn validate_skim_sell_multiple(skim: &SkimSellMultipleArg) -> Result<()> {
    require!(
        (MIN_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS..=MAX_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS).contains(&skim.skim_sell_at_floor_multiple_bps),
        HookError::SkimSellMultipleOutOfRange
    );
    Ok(())
}

///
///
///
pub fn validate_skim_sell_at_floor_cap(args: &RegisterPoolArgs, skim: &SkimSellMultipleArg) -> Result<u16> {
    let at = skim_sell_at_floor_bps_at(args.skim_sell_far_bps, skim.skim_sell_at_floor_multiple_bps);
    require!(at <= MAX_SKIM_SELL_AT_FLOOR_BPS, HookError::SkimSellAtFloorAboveCap);
    Ok(at)
}

///
///
///
///
///
pub fn validate_stair(stair: &StairArg) -> Result<()> {
    require!(floor_math::stair::stair_shape_ok(stair.steps, stair.growth_bps), HookError::StairShapeOutOfRange);
    Ok(())
}

///
///
///
pub fn validate_stair_owner(stair: &StairArg, owner: u8) -> Result<()> {
    require!(
        owner == 0 || (owner == floor_types::constants::STAIR_OWNER_TEAM_BUY && stair.steps != 0),
        HookError::StairOwnerBadConfig
    );
    Ok(())
}



///
///
///
///
///
pub fn validate_tiers(args: &RegisterPoolArgs, tiers: &ReferralTiersArg) -> Result<()> {
    require!(
        floor_types::register::referral_tiers_well_formed(
            &args.deployer.to_bytes(),
            &args.referrer.to_bytes(),
            &tiers.referrer_t2.to_bytes(),
            &tiers.referrer_t3.to_bytes(),
        ),
        HookError::ReferrerNotAllowed
    );
    Ok(())
}

///
///
///
///
///
///
///
///
///
///
pub fn validate_slots(
    args: &RegisterPoolArgs,
    tiers: &ReferralTiersArg,
    owner: u8,
    slots: &crate::SlotsArg,
) -> Result<crate::state::slots::PayeeSlots> {
    use crate::state::slots as sl;
    let creator: Vec<sl::SlotArg> = slots.creator.iter().map(Into::into).collect();
    let team: Vec<sl::SlotArg> = slots.team.iter().map(Into::into).collect();
    sl::check(&creator)?;
    if creator.iter().any(|c| c.platform != 0) && args.creator_buy_bps == 0 && args.creator_sell_bps == 0 {
        msg!("BadRecipients code {}", floor_math::slots::CHECK_PENDING_WITHOUT_RATE);
        return err!(HookError::BadRecipients);
    }
    if owner == floor_types::constants::STAIR_OWNER_TEAM_BUY {
        sl::check(&team)?;
    } else {
        require!(team.is_empty(), HookError::BadRecipients);
    }
    let mut t = sl::PayeeSlots::default();
    t.fill(sl::SLOT_KIND_CREATOR, &creator);
    if !team.is_empty() {
        t.fill(sl::SLOT_KIND_TEAM, &team);
    }
    if args.referrer != Pubkey::default() {
        for r in [args.referrer, tiers.referrer_t2, tiers.referrer_t3] {
            if r != Pubkey::default() {
                require!(!t.holds_wallet(sl::SLOT_KIND_CREATOR, &r.to_bytes()), HookError::ReferrerNotAllowed);
            }
        }
    }
    Ok(t)
}

///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
pub use super::quote_policy::QuoteTerms;

///
///
///
///
pub fn fallback_quote() -> Pubkey {
    WSOL_MINT
}

///
///
///
///
pub fn wsol_terms() -> QuoteTerms {
    QuoteTerms {
        min_bid_floor: MIN_BID_LAMPORTS_FLOOR,
        min_bid_ceil: MIN_BID_LAMPORTS_CEIL,
        max_bounty_ceil: MAX_BOUNTY_LAMPORTS_CEIL,



        min_bid: floor_types::constants::MIN_BID_LAMPORTS,
        max_bounty: floor_types::constants::MAX_BOUNTY_LAMPORTS,
    }
}

///
pub fn validate_quote_terms(min_bid: u64, max_bounty: u64, t: &QuoteTerms) -> Result<()> {
    require!((t.min_bid_floor..=t.min_bid_ceil).contains(&min_bid), HookError::MinBidOutOfRange);
    require!(max_bounty <= t.max_bounty_ceil, HookError::BountyOutOfRange);
    Ok(())
}

///
///
///
///
///
///
///
///
pub fn referral_for(referrer: Pubkey) -> (Pubkey, u16) {
    let share = if referrer == Pubkey::default() { 0 } else { REFERRAL_CREATOR_SHARE_BPS };
    (referrer, share)
}

///
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BaseHookSlot {
///
    Absent,
///
///
    Floor,
///
    Foreign,
}

///
///
///
pub fn base_hook_slot(owner: &Pubkey, data: &[u8], lock_authority: &Pubkey) -> Result<BaseHookSlot> {
    if *owner != carpenter_mint_gate::TOKEN_2022_ID {
        return Ok(BaseHookSlot::Absent);
    }
    let state = StateWithExtensions::<MintState>::unpack(data).map_err(|_| error!(HookError::BadConfig))?;
    let types = state.get_extension_types().map_err(|_| error!(HookError::BadConfig))?;
    if !types.contains(&ExtensionType::TransferHook) {
        return Ok(BaseHookSlot::Absent);
    }
    let Ok(h) = state.get_extension::<TransferHook>() else { return Ok(BaseHookSlot::Foreign) };
    let program: Option<Pubkey> = Option::<Pubkey>::from(h.program_id);
    let authority: Option<Pubkey> = Option::<Pubkey>::from(h.authority);
    Ok(if program.is_none() && authority == Some(*lock_authority) { BaseHookSlot::Floor } else { BaseHookSlot::Foreign })
}

///
///
///
///
///
pub fn holders_hooked_for(args: &RegisterPoolArgs, slot: BaseHookSlot) -> Result<bool> {
    let rate = args.holder_buy_bps > 0 || args.holder_sell_bps > 0;
    match (rate, slot) {
        (true, BaseHookSlot::Floor) => Ok(true),
        (false, BaseHookSlot::Absent) => Ok(false),
        _ => err!(HookError::HolderRateNeedsHook),
    }
}

///
///
///
///
///
pub fn admitted_base_hook(owner: &Pubkey, data: &[u8]) -> Pubkey {
    if *owner != CARPENTER_AMM_PROGRAM
        || data.len() < ApprovedHookLayout::ACCOUNT_LEN
        || data[..8] != ApprovedHookLayout::DISCRIMINATOR
    {
        return Pubkey::default();
    }
    let h: ApprovedHookLayout = bytemuck::pod_read_unaligned(&data[8..ApprovedHookLayout::ACCOUNT_LEN]);
    Pubkey::new_from_array(h.base_transfer_hook)
}

///
///
///
pub fn predicted_amm_pool(quote_mint: &Pubkey, base_mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            floor_types::seeds::POOL,
            quote_mint.as_ref(),
            base_mint.as_ref(),
            &DYNAMIC_FEE_FLAG.to_le_bytes(),
            &FLOOR_TICK_SPACING.to_le_bytes(),
            crate::ID.as_ref(),
        ],
        &CARPENTER_AMM_PROGRAM,
    )
    .0
}

///
///
///
pub(crate) fn read_amm_config(acct: &AccountInfo) -> Result<Option<AmmConfig>> {
    if *acct.owner != CARPENTER_AMM_PROGRAM {
        return Ok(None);
    }
    let data = acct.try_borrow_data()?;
    if data.len() != 8 + AmmConfig::SIZE || data[..8] != AmmConfig::DISCRIMINATOR {
        return Ok(None);
    }
    Ok(Some(bytemuck::pod_read_unaligned(&data[8..8 + AmmConfig::SIZE])))
}

///
///
///
///
///
///
///
///
///
pub(crate) fn amm_roster_entry(acct: &AccountInfo, mint: &Pubkey) -> Result<Option<(u8, [u8; 32])>> {
    let (expected, _) = Pubkey::find_program_address(&[QUOTE_MINT_SEED, mint.as_ref()], &CARPENTER_AMM_PROGRAM);
    require_keys_eq!(acct.key(), expected, HookError::QuoteNotAmmQuote);
    if *acct.owner != CARPENTER_AMM_PROGRAM {
        return Ok(None);
    }
    let data = acct.try_borrow_data()?;
    if data.len() != 8 + QuoteMintLayout::SIZE || data[..8] != QuoteMintLayout::DISCRIMINATOR {
        return Ok(None);
    }
    let q: QuoteMintLayout = bytemuck::pod_read_unaligned(&data[8..8 + QuoteMintLayout::SIZE]);
    require!(q.mint == mint.to_bytes(), HookError::QuoteNotAmmQuote);
    Ok(Some((q.state, q.hook_program)))
}

///
///
pub(crate) fn roster_pairs_for_us(entry: Option<(u8, [u8; 32])>) -> bool {
    match entry {
        Some((QUOTE_STATE_LISTED, _)) => true,
        Some((QUOTE_STATE_HOOK_LISTED, hook)) => hook == crate::ID.to_bytes(),
        _ => false,
    }
}

///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
pub(crate) fn amm_lists_quote(config: &AccountInfo, roster: Option<&AccountInfo>, mint: &Pubkey) -> Result<bool> {
    if *mint == WSOL_MINT {
        return Ok(true);
    }

    if read_amm_config(config)?.is_none() {
        return Ok(false);
    }
    match roster {
        Some(acct) => Ok(roster_pairs_for_us(amm_roster_entry(acct, mint)?)),
        None => Ok(false),
    }
}

///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
fn quote_accounts<'a, 'info>(remaining: &'a [AccountInfo<'info>]) -> Result<(Option<&'a AccountInfo<'info>>, Option<&'a AccountInfo<'info>>)> {
    require!(remaining.len() <= 2 || remaining.len() == quote_lineage::DERIVED_REMAINING, HookError::BadConfig);
    Ok((remaining.first(), remaining.get(1)))
}

fn quote_policy_of(acct: Option<&AccountInfo>, quote_mint: &Pubkey) -> Result<Option<QuotePolicyLayout>> {
    let Some(acct) = acct else { return Ok(None) };
    let (expected, _) = Pubkey::find_program_address(&[QUOTE_POLICY_SEED, quote_mint.as_ref()], &crate::ID);
    require_keys_eq!(acct.key(), expected, HookError::QuoteNotListed);
    if *acct.owner != crate::ID {
        return Ok(None);
    }
    let data = acct.try_borrow_data()?;
    if data.len() != 8 + QuotePolicyLayout::SIZE || data[..8] != QuotePolicyLayout::DISCRIMINATOR {
        return Ok(None);
    }
    let q: QuotePolicyLayout = bytemuck::pod_read_unaligned(&data[8..8 + QuotePolicyLayout::SIZE]);


    require!(q.quote_mint == quote_mint.to_bytes(), HookError::QuoteNotListed);
    Ok(Some(q))
}

///
fn mint_pool_count(acct: &AccountInfo) -> Result<Option<u64>> {
    if *acct.owner != CARPENTER_AMM_PROGRAM {
        return Ok(None);
    }
    let data = acct.try_borrow_data()?;
    if data.len() != MintPoolCount::ACCOUNT_LEN || data[..8] != MintPoolCount::DISCRIMINATOR {
        return Ok(None);
    }
    let m: MintPoolCount = bytemuck::pod_read_unaligned(&data[8..MintPoolCount::ACCOUNT_LEN]);
    Ok(Some(m.count))
}

pub fn handler<'info>(
    ctx: Context<'info, RegisterPool<'info>>,
    args: RegisterPoolArgs,
    tiers: ReferralTiersArg,
    team: TeamFeeArg,
    skim: SkimSellMultipleArg,
    stair: StairArg,
    owner: u8,
    slots: crate::SlotsArg,
) -> Result<()> {
    validate(&args)?;
    validate_stair(&stair)?;
    validate_stair_owner(&stair, owner)?;
    validate_tiers(&args, &tiers)?;
    let slots = validate_slots(&args, &tiers, owner, &slots)?;
    validate_team_fee(&team)?;
    validate_skim_sell_multiple(&skim)?;
    let skim_sell_at_floor = validate_skim_sell_at_floor_cap(&args, &skim)?;
    require_keys_eq!(args.amm_pool, predicted_amm_pool(&args.quote_mint, &args.base_mint), HookError::BadConfig);
    if let Some(count) = mint_pool_count(&ctx.accounts.mint_pools.to_account_info())? {
        require!(count == 0, HookError::MintAlreadyPooled);
    }












    let (policy_acct, roster_acct) = quote_accounts(ctx.remaining_accounts)?;
    let policy = quote_policy_of(policy_acct, &args.quote_mint)?;



    let derived = if policy.is_none() && ctx.remaining_accounts.len() == quote_lineage::DERIVED_REMAINING {
        require_keys_neq!(args.quote_mint, fallback_quote(), HookError::BadConfig);
        let d = quote_lineage::admit(ctx.remaining_accounts, &args.quote_mint, crate::state::twap::now_secs()?)?;
        require!(args.min_bid == d.min_bid && args.max_bounty == 0, HookError::DerivedQuoteTerms);
        Some(d)
    } else {
        require!(
            amm_lists_quote(&ctx.accounts.amm_config.to_account_info(), roster_acct, &args.quote_mint)?,
            HookError::QuoteNotAmmQuote
        );
        None
    };
    let terms = match policy {
        Some(q) => {


            require!(q.retired == 0, HookError::QuoteRetired);



            require!(
                (MIN_QUOTE_DECIMALS..=MAX_QUOTE_DECIMALS).contains(&q.decimals),
                HookError::QuoteDecimalsOutOfRange
            );
            QuoteTerms {
                min_bid_floor: q.min_bid_floor,
                min_bid_ceil: q.min_bid_ceil,
                max_bounty_ceil: q.max_bounty_ceil,
                min_bid: q.min_bid,
                max_bounty: q.max_bounty,
            }
        }




        None if derived.is_some() => QuoteTerms {
            min_bid_floor: args.min_bid,
            min_bid_ceil: args.min_bid,
            max_bounty_ceil: 0,
            min_bid: args.min_bid,
            max_bounty: 0,
        },
        None => {
            require_keys_eq!(args.quote_mint, fallback_quote(), HookError::QuoteNotListed);
            wsol_terms()
        }
    };
    validate_quote_terms(args.min_bid, args.max_bounty, &terms)?;


    let lock_authority =
        Pubkey::find_program_address(&[LOCK_AUTHORITY_SEED], &ctx.accounts.hook_config.load()?.launch_program).0;
    let slot = {
        let mint = ctx.accounts.base_mint.to_account_info();
        let data = mint.try_borrow_data()?;
        base_hook_slot(mint.owner, &data, &lock_authority)?
    };
    let hooked = holders_hooked_for(&args, slot)?;









    if hooked {
        let approved = ctx.accounts.approved_hook.to_account_info();
        let data = approved.try_borrow_data()?;
        require_keys_eq!(admitted_base_hook(approved.owner, &data), FLOOR_HOLDERS_PROGRAM, HookError::HolderRateNeedsHook);
    }

    let spacing = FLOOR_TICK_SPACING as i32;
    {
        let mut s = ctx.accounts.floor_pool.load_init()?;
        s.version = 1;
        s.bump = ctx.bumps.floor_pool;
        s.pool_bound = 0;
        s.quote_is_token1 = 0;
        s.ladder_seeded = 0;
        s.needs_ratchet = 0;
        s.mode = args.mode;
        s.tick_spacing = spacing;
        s.floor_tick = math::ceil_to_spacing(MAX_TICK, spacing)?;
        s.amm_pool = args.amm_pool;
        s.pool_initializer = args.pool_initializer;
        s.base_mint = args.base_mint;
        s.quote_mint = args.quote_mint;
        s.skim_buy_far_bps = args.skim_buy_far_bps;
        s.skim_sell_far_bps = args.skim_sell_far_bps;
        s.creator_buy_bps = args.creator_buy_bps;
        s.creator_sell_bps = args.creator_sell_bps;
        s.holder_buy_bps = args.holder_buy_bps;
        s.holder_sell_bps = args.holder_sell_bps;

        s.platform_bps = args.platform_bps;
        s.min_bid = args.min_bid;
        s.min_sellable = args.min_sellable;






        s.skim_buy_at_floor_bps = DEFAULT_SKIM_BUY_AT_FLOOR_BPS;

        s.skim_sell_at_floor_bps = skim_sell_at_floor;

        s.bid_width = 0;











        let (referrer, share) = referral_for(args.referrer);
        s.referrer = referrer;
        s.referral_share_bps = share;
        s.holders_hooked = hooked as u8;
        s.referral_owed = 0;
        s.referral_fees_paid = 0;



        s.referrer_t2 = tiers.referrer_t2;
        s.referrer_t3 = tiers.referrer_t3;
        s.referral_paid_t2 = 0;
        s.referral_paid_t3 = 0;



        s.max_bounty = derived.map(|d| d.max_bounty).unwrap_or(args.max_bounty);
        s.bounty_bps = args.bounty_bps;





        s.team_fee_bps = team.team_fee_bps;
        s.team_owed = 0;
        s.team_fees_paid = 0;




        s.stair_steps = stair.steps;
        s.stair_growth_bps = stair.growth_bps;
        s.stair_ready = 0;




        s.stair_owner = owner;
        s.owner_owed = 0;
        s.owner_paid = 0;
    }

    crate::state::slots::write(&ctx.accounts.floor_pool.to_account_info(), &slots)?;

    let a = &ctx.accounts;



    if let Some(d) = derived {
        let ra = ctx.remaining_accounts;
        let entry = amm_roster_entry(&ra[1], &args.quote_mint)?;
        require!(!matches!(entry, Some((QUOTE_STATE_RETIRED, _))), HookError::QuoteNotAmmQuote);
        if !roster_pairs_for_us(entry) {
            amm_cpi::hook_list_quote_mint(
                &a.carpenter_amm.to_account_info(),
                &a.amm_event_authority.to_account_info(),
                &a.hook_authority.to_account_info(),
                ctx.bumps.hook_authority,
                &a.amm_config.to_account_info(),
                &a.approved_hook.to_account_info(),
                &ra[2],
                &ra[1],
                &a.payer.to_account_info(),
                &a.system_program.to_account_info(),
            )?;
        }
        emit_cpi!(QuoteAdmitted {
            floor_pool: ctx.accounts.floor_pool.key(),
            quote_mint: args.quote_mint,
            how: d.how,
            parent: d.parent,
            decimals: d.decimals,
            min_bid: d.min_bid,
            max_bounty: d.max_bounty,
            cap_tick: d.cap_tick,
        });
    }
    let a = &ctx.accounts;
    amm_cpi::register_mint_exclusivity(
        &a.carpenter_amm.to_account_info(),
        &a.amm_event_authority.to_account_info(),
        &a.hook_authority.to_account_info(),
        ctx.bumps.hook_authority,
        &a.amm_config.to_account_info(),
        &a.approved_hook.to_account_info(),
        &a.base_mint.to_account_info(),
        &a.mint_pools.to_account_info(),
        &a.mint_exclusivity.to_account_info(),
        &a.payer.to_account_info(),
        &a.system_program.to_account_info(),
    )?;

    emit_cpi!(PoolRegistered {
        floor_pool: ctx.accounts.floor_pool.key(),
        amm_pool: args.amm_pool,
        base_mint: args.base_mint,
        pool_initializer: args.pool_initializer,
        mode: args.mode,
    });
    let (referrer, share) = referral_for(args.referrer);
    emit_cpi!(PoolRewardsTerms {
        floor_pool: ctx.accounts.floor_pool.key(),
        amm_pool: args.amm_pool,
        base_mint: args.base_mint,
        referrer,
        referral_share_bps: share,
        holders_hooked: hooked,
    });


    if tiers.referrer_t2 != Pubkey::default() {
        emit_cpi!(PoolReferralTiers {
            floor_pool: ctx.accounts.floor_pool.key(),
            amm_pool: args.amm_pool,
            referrer,
            referrer_t2: tiers.referrer_t2,
            referrer_t3: tiers.referrer_t3,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use floor_types::constants::MODE_CUSTOM;

    const REFERRER: Pubkey = Pubkey::new_from_array([0x42u8; 32]);

///
///
///
    fn ok_args() -> RegisterPoolArgs {
        RegisterPoolArgs {
            mode: MODE_CUSTOM,




            holder_buy_bps: 0,
            holder_sell_bps: 0,
            platform_bps: PLATFORM_BPS,
            min_sellable: 1,
            deployer: Pubkey::new_from_array([7u8; 32]),
            pool_initializer: Pubkey::new_from_array([8u8; 32]),
            quote_mint: WSOL_MINT,
            base_mint: Pubkey::new_from_array([0xffu8; 32]),
            ..Default::default()
        }
    }

///
///
///
///
///
///
///
    #[test]
    fn the_referrer_is_the_args_own_and_the_share_follows_it() {
        assert_eq!(referral_for(REFERRER), (REFERRER, REFERRAL_CREATOR_SHARE_BPS));
        assert_eq!(referral_for(Pubkey::default()), (Pubkey::default(), 0));
        assert_eq!(REFERRAL_CREATOR_SHARE_BPS, 1_000, "default B6: 10 % of the gross creator fee, no dial");
    }

///
///
///
///
    #[test]
    fn the_tiers_are_refused_on_a_gap_the_creator_or_a_duplicate() {
        let (t2, t3) = (Pubkey::new_from_array([0x52; 32]), Pubkey::new_from_array([0x53; 32]));
        let referred = RegisterPoolArgs { referrer: REFERRER, ..ok_args() };
        let tiers = |a: Pubkey, b: Pubkey| ReferralTiersArg { referrer_t2: a, referrer_t3: b };
        let z = Pubkey::default();
        assert!(validate_tiers(&ok_args(), &ReferralTiersArg::default()).is_ok(), "unreferred, no tiers");
        assert!(validate_tiers(&referred, &ReferralTiersArg::default()).is_ok(), "single tier");
        assert!(validate_tiers(&referred, &tiers(t2, z)).is_ok(), "two tiers");
        assert!(validate_tiers(&referred, &tiers(t2, t3)).is_ok(), "three tiers");
        let refused = |a: &RegisterPoolArgs, t: ReferralTiersArg, why: &str| {
            assert_eq!(validate_tiers(a, &t).unwrap_err(), error!(HookError::ReferrerNotAllowed), "{why}");
        };
        refused(&ok_args(), tiers(t2, z), "gap: t2 without a referrer");
        refused(&referred, tiers(z, t3), "gap: t3 without t2");
        refused(&referred, tiers(ok_args().deployer, z), "t2 is the deployer");
        refused(&referred, tiers(t2, ok_args().deployer), "t3 is the deployer");
        refused(&referred, tiers(REFERRER, z), "t2 == t1");
        refused(&referred, tiers(t2, REFERRER), "t3 == t1");
        refused(&referred, tiers(t2, t2), "t3 == t2");
    }

///
///
///
///
    #[test]
    fn the_mode_byte_is_any_preset_id_up_to_the_tables_bound() {
        assert_eq!(MAX_PRESET_ID, 31);
        for mode in 0..=MAX_PRESET_ID {
            assert!(validate(&RegisterPoolArgs { mode, ..ok_args() }).is_ok(), "mode {mode} is a table slot");
        }
        for mode in [MAX_PRESET_ID + 1, 64, u8::MAX] {
            assert_eq!(
                validate(&RegisterPoolArgs { mode, ..ok_args() }).unwrap_err(),
                error!(HookError::BadConfig),
                "mode {mode} is past the table"
            );
        }
    }

///
///
///
///
    #[test]
    fn the_handler_takes_no_wall_and_writes_no_wall_word() {
        let src = include_str!("register_pool.rs");
        let h = src.find("pub fn handler<'info>(").expect("handler");
        let end = src[h..].find("\n}\n").expect("handler end") + h;
        let body = &src[h..end];
        for w in ["s.wall_ticks", "s.wall_band_ticks", "s.wall_tail_bps", "s.wall_list_bps", "WallShapeArg", "wall:"] {
            let code: String = body.lines().map(|l| l.split("//").next().unwrap_or("")).collect::<Vec<_>>().join("\n");
            assert!(!code.contains(w), "the handler still names `{w}`");
        }
        assert_eq!(floor_types::register::REGISTER_POOL_ARGS_LEN, 265);
    }

///
///
///
///
///
    #[test]
    fn a_default_built_args_is_unreferred_which_is_property_d() {
        let args = RegisterPoolArgs::default();
        assert_eq!(args.referrer, Pubkey::default());
        assert_eq!(referral_for(args.referrer), (Pubkey::default(), 0));
        assert!(validate(&ok_args()).is_ok(), "an unreferred launch passes Q2 trivially");
    }

///
///
///
///
    #[test]
    fn mr_s_validate_refuses_a_non_zero_pad_and_tiers_refuse_the_deployer() {
        assert!(validate(&RegisterPoolArgs { referrer: REFERRER, ..ok_args() }).is_ok());
        let padded = RegisterPoolArgs { pad: [7u8; 32], ..ok_args() };
        assert_eq!(validate(&padded).unwrap_err(), error!(HookError::BadConfig), "the dead pending id must be zero");
        let self_referred = RegisterPoolArgs { referrer: ok_args().deployer, ..ok_args() };
        assert_eq!(
            validate_tiers(&self_referred, &ReferralTiersArg::default()).unwrap_err(),
            error!(HookError::ReferrerNotAllowed),
            "the deployer never refers its own launch"
        );
    }



    fn hook_code(e: anchor_lang::error::Error) -> u32 {
        match e {
            anchor_lang::error::Error::AnchorError(a) => a.error_code_number,
            e => panic!("not an anchor error: {e:?}"),
        }
    }
    const BAD_RECIPIENTS: u32 = 7000 + HookError::BadRecipients as u32;
    fn w(b: u8, bps: u16) -> crate::SlotArg {
        crate::SlotArg { bps, platform: 0, key: [b; 32] }
    }
    fn h(b: u8, bps: u16, platform: u8) -> crate::SlotArg {
        crate::SlotArg { bps, platform, key: [b; 32] }
    }
    fn creator(c: Vec<crate::SlotArg>) -> crate::SlotsArg {
        crate::SlotsArg { creator: c, team: vec![] }
    }
///
    fn rated() -> RegisterPoolArgs {
        RegisterPoolArgs { creator_buy_bps: 50, ..ok_args() }
    }
    const NO_TIERS: ReferralTiersArg = ReferralTiersArg { referrer_t2: Pubkey::new_from_array([0; 32]), referrer_t3: Pubkey::new_from_array([0; 32]) };

    #[test]
    fn mr_s_validate_slots_one_slot_and_five_slots_lay_out_the_tail() {
        use crate::state::slots as sl;
        let t = validate_slots(&rated(), &NO_TIERS, 0, &creator(vec![w(1, 10_000)])).unwrap();
        assert_eq!((t.version, t.creator_count, t.team_count, t.creator_pending_mask), (sl::PAYEE_SLOTS_VERSION, 1, 0, 0));
        assert_eq!((t.creator[0].key, t.creator[0].bps, t.creator[0].state), ([1; 32], 10_000, sl::SLOT_STATE_WALLET));
        let five = vec![w(1, 2_000), h(2, 2_000, 1), w(3, 2_000), h(4, 2_000, 4), w(5, 2_000)];
        let t = validate_slots(&rated(), &NO_TIERS, 0, &creator(five)).unwrap();
        assert_eq!((t.creator_count, t.creator_pending_mask), (5, 0b01010));
        assert_eq!(t.creator[3].state, sl::SLOT_STATE_PENDING);
        assert_eq!(t.creator[3].platform, 4);
    }

///
    #[test]
    fn mr_s_validate_slots_refuses_every_check_code_as_bad_recipients() {
        let cases: Vec<(u8, Vec<crate::SlotArg>)> = vec![
            (1, vec![]),
            (1, vec![w(1, 2_000), w(2, 2_000), w(3, 2_000), w(4, 2_000), w(5, 1_000), w(6, 1_000)]),
            (2, vec![w(1, 10_000), w(2, 0)]),
            (3, vec![w(1, 5_000), w(2, 4_999)]),
            (4, vec![w(1, 5_000), h(2, 5_000, 5)]),
            (4, vec![w(0, 5_000), w(2, 5_000)]),
            (5, vec![w(1, 5_000), w(1, 5_000)]),
            (6, vec![h(1, 5_000, 1), h(1, 5_000, 2)]),
        ];
        for (want, c) in cases {
            let fm: Vec<floor_math::Slot> =
                c.iter().map(|a| floor_math::Slot { bps: a.bps, platform: a.platform, key: a.key }).collect();
            assert_eq!(floor_math::check_slots(&fm), want, "the case is code {want}");
            let e = validate_slots(&rated(), &NO_TIERS, 0, &creator(c)).unwrap_err();
            assert_eq!(hook_code(e), BAD_RECIPIENTS, "code {want}");
        }

        let team_buy = floor_types::constants::STAIR_OWNER_TEAM_BUY;
        let bad_team = crate::SlotsArg { creator: vec![w(1, 10_000)], team: vec![w(2, 5_000), w(3, 4_000)] };
        assert_eq!(hook_code(validate_slots(&rated(), &NO_TIERS, team_buy, &bad_team).unwrap_err()), BAD_RECIPIENTS);
    }

///
    #[test]
    fn mr_s_validate_slots_code_7_a_pending_creator_slot_needs_a_rate() {
        let pending = creator(vec![w(1, 5_000), h(2, 5_000, 1)]);
        assert_eq!(hook_code(validate_slots(&ok_args(), &NO_TIERS, 0, &pending).unwrap_err()), BAD_RECIPIENTS);
        assert!(validate_slots(&RegisterPoolArgs { creator_sell_bps: 1, ..ok_args() }, &NO_TIERS, 0, &pending).is_ok());
        assert!(validate_slots(&rated(), &NO_TIERS, 0, &pending).is_ok());

        assert!(validate_slots(&ok_args(), &NO_TIERS, 0, &creator(vec![w(1, 10_000)])).is_ok());
    }

///
    #[test]
    fn mr_s_validate_slots_team_list_only_on_a_team_buy_pool() {
        use crate::state::slots as sl;
        let team_buy = floor_types::constants::STAIR_OWNER_TEAM_BUY;
        let both = crate::SlotsArg { creator: vec![w(1, 10_000)], team: vec![w(2, 6_000), h(3, 4_000, 2)] };
        for owner in [0u8, floor_types::constants::STAIR_OWNER_TEAM] {
            assert_eq!(hook_code(validate_slots(&rated(), &NO_TIERS, owner, &both).unwrap_err()), BAD_RECIPIENTS, "owner {owner}");
        }
        let t = validate_slots(&rated(), &NO_TIERS, team_buy, &both).unwrap();
        assert_eq!((t.team_count, t.team_pending_mask), (2, 0b10));
        assert_eq!(t.team[1].state, sl::SLOT_STATE_PENDING);

        assert_eq!(hook_code(validate_slots(&rated(), &NO_TIERS, team_buy, &creator(vec![w(1, 10_000)])).unwrap_err()), BAD_RECIPIENTS);
    }

///
///
    #[test]
    fn mr_s_validate_slots_the_referrer_rule() {
        let not_allowed = 7000 + HookError::ReferrerNotAllowed as u32;
        let referred = RegisterPoolArgs { referrer: REFERRER, ..rated() };

        let all_pending = creator(vec![h(1, 5_000, 1), h(2, 5_000, 3)]);
        assert!(validate_slots(&referred, &NO_TIERS, 0, &all_pending).is_ok(), "A16: referred with no address slot");
        assert!(validate_slots(&rated(), &NO_TIERS, 0, &all_pending).is_ok(), "unreferred, all pending is fine");

        let with_ref = creator(vec![w(1, 5_000), w(0x42, 5_000)]);
        assert_eq!(hook_code(validate_slots(&referred, &NO_TIERS, 0, &with_ref).unwrap_err()), not_allowed);

        let slots = creator(vec![w(1, 5_000), w(9, 5_000)]);
        let t2 = ReferralTiersArg { referrer_t2: Pubkey::new_from_array([9; 32]), ..NO_TIERS };
        assert_eq!(hook_code(validate_slots(&referred, &t2, 0, &slots).unwrap_err()), not_allowed, "t2");
        let t3 = ReferralTiersArg { referrer_t2: Pubkey::new_from_array([8; 32]), referrer_t3: Pubkey::new_from_array([1; 32]) };
        assert_eq!(hook_code(validate_slots(&referred, &t3, 0, &slots).unwrap_err()), not_allowed, "t3");

        let ok = creator(vec![w(1, 5_000), h(0x42, 5_000, 1)]);
        assert!(validate_slots(&referred, &NO_TIERS, 0, &ok).is_ok());
        let ok3 = ReferralTiersArg { referrer_t2: Pubkey::new_from_array([8; 32]), referrer_t3: Pubkey::new_from_array([6; 32]) };
        assert!(validate_slots(&referred, &ok3, 0, &slots).is_ok());
    }



    const LOCK: Pubkey = Pubkey::new_from_array([0x10u8; 32]);
    const OTHER: Pubkey = Pubkey::new_from_array([0x20u8; 32]);

    fn token_2022() -> Pubkey {
        carpenter_mint_gate::TOKEN_2022_ID
    }

///
///
///
///
    fn t22_mint(hook: Option<(Pubkey, Pubkey)>) -> Vec<u8> {
        let mut d = vec![0u8; 82];
        d[44] = 6;
        d[45] = 1;
        let Some((authority, program)) = hook else { return d };
        d.resize(165, 0);
        d.push(1);
        d.extend_from_slice(&14u16.to_le_bytes());
        d.extend_from_slice(&64u16.to_le_bytes());
        d.extend_from_slice(authority.as_ref());
        d.extend_from_slice(program.as_ref());
        d
    }

///
///
///
///
    #[test]
    fn only_an_empty_slot_whose_authority_is_the_lock_pda_is_floors() {
        let spl = carpenter_mint_gate::SPL_TOKEN_ID;
        assert_eq!(base_hook_slot(&spl, &t22_mint(None), &LOCK).unwrap(), BaseHookSlot::Absent, "classic SPL");
        assert_eq!(base_hook_slot(&token_2022(), &t22_mint(None), &LOCK).unwrap(), BaseHookSlot::Absent, "T22, no extension");
        assert_eq!(
            base_hook_slot(&token_2022(), &t22_mint(Some((LOCK, Pubkey::default()))), &LOCK).unwrap(),
            BaseHookSlot::Floor,
            "empty slot, the lock PDA's"
        );
        assert_eq!(
            base_hook_slot(&token_2022(), &t22_mint(Some((OTHER, Pubkey::default()))), &LOCK).unwrap(),
            BaseHookSlot::Foreign,
            "empty slot, somebody else's authority"
        );
        assert_eq!(
            base_hook_slot(&token_2022(), &t22_mint(Some((LOCK, OTHER))), &LOCK).unwrap(),
            BaseHookSlot::Foreign,
            "the lock PDA's slot, but ARMED"
        );
        assert_eq!(
            base_hook_slot(&token_2022(), &t22_mint(Some((Pubkey::default(), Pubkey::default()))), &LOCK).unwrap(),
            BaseHookSlot::Foreign,
            "an empty slot with NO authority is not FLOOR's (nothing could arm it at graduation)"
        );
    }

///
///
///
///
///
///
///
    #[test]
    fn a_holder_rate_and_floors_slot_come_together_or_not_at_all() {
        let none = ok_args();
        let buy = RegisterPoolArgs { holder_buy_bps: 100, ..ok_args() };
        let sell = RegisterPoolArgs { holder_sell_bps: 1, ..ok_args() };
        assert!(!holders_hooked_for(&none, BaseHookSlot::Absent).unwrap(), "no rate, no slot: unhooked");
        assert!(holders_hooked_for(&buy, BaseHookSlot::Floor).unwrap(), "a buy rate on FLOOR's slot: hooked");
        assert!(holders_hooked_for(&sell, BaseHookSlot::Floor).unwrap(), "a sell rate alone on FLOOR's slot: hooked");
        for (args, slot, what) in [
            (&buy, BaseHookSlot::Absent, "a rate with no slot"),
            (&sell, BaseHookSlot::Absent, "a sell rate with no slot"),
            (&buy, BaseHookSlot::Foreign, "a rate on a foreign slot"),
            (&none, BaseHookSlot::Floor, "FLOOR's slot with no rate"),
            (&none, BaseHookSlot::Foreign, "a foreign slot with no rate"),
        ] {
            assert_eq!(holders_hooked_for(args, slot).unwrap_err(), error!(HookError::HolderRateNeedsHook), "{what}");
        }
    }

///
///
///
    #[test]
    fn the_admitted_base_hook_is_read_off_the_amms_approval_only() {
        let mut d = vec![0u8; ApprovedHookLayout::ACCOUNT_LEN];
        d[..8].copy_from_slice(&ApprovedHookLayout::DISCRIMINATOR);
        d[8 + 129..8 + 161].copy_from_slice(FLOOR_HOLDERS_PROGRAM.as_ref());
        assert_eq!(admitted_base_hook(&CARPENTER_AMM_PROGRAM, &d), FLOOR_HOLDERS_PROGRAM);
        assert_eq!(admitted_base_hook(&Pubkey::new_unique(), &d), Pubkey::default(), "not the AMM's account");
        let mut bad = d.clone();
        bad[0] ^= 1;
        assert_eq!(admitted_base_hook(&CARPENTER_AMM_PROGRAM, &bad), Pubkey::default(), "not an ApprovedHook");
        assert_eq!(admitted_base_hook(&CARPENTER_AMM_PROGRAM, &d[..100]), Pubkey::default(), "short");
        let mut none = d.clone();
        none[8 + 129..8 + 161].fill(0);
        assert_eq!(admitted_base_hook(&CARPENTER_AMM_PROGRAM, &none), Pubkey::default(), "nothing admitted");
    }

///
///
///
///
///
///
///
    #[test]
    fn the_base_mint_must_sort_above_its_own_quote_not_above_wsol() {
        let lo = Pubkey::new_from_array([1u8; 32]);
        let hi = Pubkey::new_from_array([2u8; 32]);
        let mut args = RegisterPoolArgs { quote_mint: lo, base_mint: hi, ..ok_args() };
        assert!(validate(&args).is_ok(), "base above quote");

        args.quote_mint = hi;
        args.base_mint = lo;
        assert_eq!(validate(&args).unwrap_err(), error!(HookError::BadConfig), "base below quote");

        args.quote_mint = hi;
        args.base_mint = hi;
        assert_eq!(validate(&args).unwrap_err(), error!(HookError::BadConfig), "one mint, both sides");



        let above_wsol = Pubkey::new_from_array([0xffu8; 32]);
        let below_wsol = Pubkey::new_from_array([0x00u8; 32]);
        assert!(above_wsol.to_bytes() > WSOL_MINT.to_bytes() && below_wsol.to_bytes() < WSOL_MINT.to_bytes());
        let args = RegisterPoolArgs { quote_mint: WSOL_MINT, base_mint: above_wsol, ..ok_args() };
        assert!(validate(&args).is_ok());
        let args = RegisterPoolArgs { quote_mint: WSOL_MINT, base_mint: below_wsol, ..ok_args() };
        assert_eq!(validate(&args).unwrap_err(), error!(HookError::BadConfig));
    }

///
///
///
///
///
///
    #[test]
    fn the_wsol_terms_are_the_constants_they_replace_and_they_bind() {
        let t = wsol_terms();
        assert_eq!(t.min_bid_floor, MIN_BID_LAMPORTS_FLOOR);
        assert_eq!(t.min_bid_ceil, MIN_BID_LAMPORTS_CEIL);
        assert_eq!(t.max_bounty_ceil, MAX_BOUNTY_LAMPORTS_CEIL);

        assert!(validate_quote_terms(t.min_bid_floor, 0, &t).is_ok());
        assert!(validate_quote_terms(t.min_bid_ceil, t.max_bounty_ceil, &t).is_ok());
        assert_eq!(
            validate_quote_terms(t.min_bid_floor - 1, 0, &t).unwrap_err(),
            error!(HookError::MinBidOutOfRange)
        );
        assert_eq!(
            validate_quote_terms(t.min_bid_ceil + 1, 0, &t).unwrap_err(),
            error!(HookError::MinBidOutOfRange)
        );
        assert_eq!(
            validate_quote_terms(t.min_bid_floor, t.max_bounty_ceil + 1, &t).unwrap_err(),
            error!(HookError::BountyOutOfRange)
        );
    }

///
///
///
    #[test]
    fn a_six_decimal_quotes_terms_are_not_admissible_under_the_wsol_range() {
        let usdc = QuoteTerms {
            min_bid_floor: 1_000,
            min_bid_ceil: 5_000_000,
            max_bounty_ceil: 500_000,
            min_bid: 50_000,
            max_bounty: 50_000,
        };

        assert!(validate_quote_terms(50_000, 50_000, &usdc).is_ok());

        assert_eq!(
            validate_quote_terms(50_000, 50_000, &wsol_terms()).unwrap_err(),
            error!(HookError::MinBidOutOfRange)
        );

        assert_eq!(
            validate_quote_terms(floor_types::constants::MIN_BID_LAMPORTS, 0, &usdc).unwrap_err(),
            error!(HookError::MinBidOutOfRange)
        );
    }

///
///
///
    #[test]
    fn the_written_share_is_the_evm_default_ten_percent() {
        assert_eq!(referral_for(REFERRER).1, 1_000);
    }

///
///
///
///
///
///
///
///
///
    #[test]
    fn the_holder_rate_is_capped_at_the_evm_ten_percent_a_side_not_pinned_to_zero() {
        assert_eq!(MAX_HOLDER_BPS, 1000, "rf:src/FloorPolicy.sol MAX_HOLDER_BPS (2026-09-25)");
        for rate in [0u16, 1, 100, 200, 1000] {
            let args = RegisterPoolArgs { holder_buy_bps: rate, holder_sell_bps: rate, ..ok_args() };
            assert!(validate(&args).is_ok(), "{rate} bps a side is inside the cap");
        }
        let args = RegisterPoolArgs { holder_buy_bps: 1001, holder_sell_bps: 0, ..ok_args() };
        assert_eq!(validate(&args).unwrap_err(), error!(HookError::BadConfig), "1001 on the buy side");
        let args = RegisterPoolArgs { holder_buy_bps: 0, holder_sell_bps: 1001, ..ok_args() };
        assert_eq!(validate(&args).unwrap_err(), error!(HookError::BadConfig), "1001 on the sell side");
        let args = RegisterPoolArgs { holder_buy_bps: 1000, holder_sell_bps: 0, ..ok_args() };
        assert!(validate(&args).is_ok(), "the two sides are independent");
    }

///
///
    #[test]
    fn the_creator_cap_is_five_percent_and_the_far_skim_cap_ten_2026_09_25() {
        assert_eq!((MAX_CREATOR_BPS, MAX_SKIM_FAR_BPS), (500, 1000));
        let args = RegisterPoolArgs { creator_buy_bps: 500, creator_sell_bps: 500, ..ok_args() };
        assert!(validate(&args).is_ok(), "500 a side is inside the cap");
        for (b, s) in [(501u16, 0u16), (0, 501)] {
            let args = RegisterPoolArgs { creator_buy_bps: b, creator_sell_bps: s, ..ok_args() };
            assert_eq!(validate(&args).unwrap_err(), error!(HookError::BadConfig), "creator {b}/{s}");
        }
        let args = RegisterPoolArgs { skim_buy_far_bps: 1000, skim_sell_far_bps: 1000, ..ok_args() };
        assert!(validate(&args).is_ok());
        let args = RegisterPoolArgs { skim_sell_far_bps: 1001, ..ok_args() };
        assert_eq!(validate(&args).unwrap_err(), error!(HookError::BadConfig));

        use floor_types::constants::skim_sell_at_floor_bps;
        assert_eq!(skim_sell_at_floor_bps(1000), 1500);
        assert_eq!(skim_sell_at_floor_bps(150), 225);
        assert_eq!(skim_sell_at_floor_bps_at(1000, 15_000), 1500, "the per-launch form at 1.5x is the same");
    }

///
///
///
///
///
    #[test]
    fn the_at_floor_sell_skim_is_capped_at_fifteen_percent_2026_09_30() {

        let at = |far: u16, m: u32| {
            validate_skim_sell_at_floor_cap(
                &RegisterPoolArgs { skim_sell_far_bps: far, ..ok_args() },
                &SkimSellMultipleArg { skim_sell_at_floor_multiple_bps: m },
            )
        };
        assert_eq!(at(1_000, 15_000).unwrap(), 1_500, "10 % far at 1.5x: exactly the cap");
        assert_eq!(at(1_000, 16_000).unwrap_err(), error!(HookError::SkimSellAtFloorAboveCap));
        assert_eq!(at(300, 50_000).unwrap(), 1_500, "3 % far at 5x: exactly the cap");
        assert_eq!(at(310, 50_000).unwrap_err(), error!(HookError::SkimSellAtFloorAboveCap));

        for (far, m) in [(200u16, 15_000u32), (250, 15_000)] {
            assert!(at(far, m).unwrap() <= 375, "presets and $FLOOR sit under it");
        }
        assert_eq!(MAX_SKIM_SELL_AT_FLOOR_BPS, 1_500);
    }

    #[test]
    fn the_at_floor_sell_multiple_is_per_launch_and_bounded_2026_09_29() {
        let ok = |m: u32| validate_skim_sell_multiple(&SkimSellMultipleArg { skim_sell_at_floor_multiple_bps: m });
        for m in [10_000u32, 15_000, 50_000] {
            assert!(ok(m).is_ok(), "{m} is inside the window");
        }
        for m in [0u32, 1, 9_999, 50_001, u32::MAX] {
            assert_eq!(ok(m).unwrap_err(), error!(HookError::SkimSellMultipleOutOfRange), "{m}");
        }



        assert_eq!(skim_sell_at_floor_bps_at(200, 15_000), 300, "every preset (2026-10-12; ~~Moonshot 125, Cook 375, Bluechip 500 at 5x~~)");
        assert_eq!(skim_sell_at_floor_bps_at(250, 15_000), 375, "$FLOOR (unchanged)");
        assert_eq!(skim_sell_at_floor_bps_at(MAX_SKIM_FAR_BPS, 50_000), 5_000, "the far cap at 5x fits the u16");
    }

///
///
///
///
///
    #[test]
    fn no_instruction_but_register_pool_writes_a_pool_rate_2026_09_25() {




        const FIELDS: [&str; 13] = [
            "skim_buy_far_bps", "skim_sell_far_bps", "skim_buy_at_floor_bps", "skim_sell_at_floor_bps",
            "creator_buy_bps", "creator_sell_bps", "holder_buy_bps", "holder_sell_bps", "fee_admin",
            "team_fee_bps", "wall_band_ticks", "wall_tail_bps", "wall_list_bps",
        ];
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
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut files = Vec::new();
        walk(&src, &mut files);
        assert!(files.len() > 10, "the scan must see the program's sources");
        let mut hits = Vec::new();
        for f in &files {
            if f.ends_with("instructions/register_pool.rs") {
                continue;
            }
            let text = std::fs::read_to_string(f).expect("read");
            for (n, line) in text.lines().enumerate() {
                let code = line.split("//").next().unwrap_or("");
                for field in FIELDS {
                    let needle = format!(".{field}");
                    let mut rest = code;
                    while let Some(i) = rest.find(&needle) {
                        let after = rest[i + needle.len()..].trim_start();
                        if after.starts_with('=') && !after.starts_with("==") {
                            hits.push(format!("{}:{}: {}", f.display(), n + 1, line.trim()));
                        }
                        rest = &rest[i + needle.len()..];
                    }
                }
            }
        }
        assert!(hits.is_empty(), "a live pool's rate is written outside register_pool:\n{}", hits.join("\n"));
    }

///
///
///
///
///
///
///
///
///
///
///
///
///
///
    #[test]
    fn a_non_wsol_quote_takes_a_holder_rate_up_to_the_cap_exactly_as_wsol_does() {
        let usdc = Pubkey::new_from_array([1u8; 32]);
        let base = Pubkey::new_from_array([0xfeu8; 32]);
        for rate in [0u16, 1, 100, MAX_HOLDER_BPS] {
            let args = RegisterPoolArgs { quote_mint: usdc, base_mint: base, holder_buy_bps: rate, holder_sell_bps: rate, ..ok_args() };
            assert!(validate(&args).is_ok(), "{rate} bps on USDC, both sides");
            let wsol = RegisterPoolArgs { holder_buy_bps: rate, holder_sell_bps: rate, ..ok_args() };
            assert_eq!(validate(&args).is_ok(), validate(&wsol).is_ok(), "{rate} bps: USDC and wSOL agree");
        }
        let args = RegisterPoolArgs { quote_mint: usdc, base_mint: base, holder_buy_bps: MAX_HOLDER_BPS + 1, holder_sell_bps: 0, ..ok_args() };
        assert_eq!(validate(&args).unwrap_err(), error!(HookError::BadConfig), "cap + 1 on USDC, buy side: the cap");
        let args = RegisterPoolArgs { quote_mint: usdc, base_mint: base, holder_buy_bps: 0, holder_sell_bps: MAX_HOLDER_BPS + 1, ..ok_args() };
        assert_eq!(validate(&args).unwrap_err(), error!(HookError::BadConfig), "cap + 1 on USDC, sell side: the cap");
    }

///
///
///
///
///
///
///
///
///
    #[test]
    fn the_pool_team_rate_is_bounded_by_its_cap_and_zero_is_legal() {
        for rate in [0u16, 1, 25, TEAM_POOL_FEE_BPS_MAX] {
            assert!(validate_team_fee(&TeamFeeArg { team_fee_bps: rate }).is_ok(), "{rate} bps is inside the cap");
        }
        assert_eq!(TEAM_POOL_FEE_BPS_MAX, 50, "the ruled cap: 0.5 %");
        for rate in [TEAM_POOL_FEE_BPS_MAX + 1, 100, u16::MAX] {
            assert_eq!(
                validate_team_fee(&TeamFeeArg { team_fee_bps: rate }).unwrap_err(),
                error!(HookError::TeamFeeOutOfRange),
                "{rate} bps is refused by name"
            );
        }
        assert_eq!(TeamFeeArg::default().team_fee_bps, 0, "a Default is a zero-fee pool");
        assert!(validate_team_fee(&TeamFeeArg::default()).is_ok());

        let src = include_str!("register_pool.rs");
        let h = src.find("pub fn handler<'info>(").expect("handler");

        let body = &src[h..h + 600];
        assert!(body.contains("validate_team_fee(&team)?;"), "the handler bounds the rate before writing it");
    }

///
///
    #[test]
    fn the_stair_arg_is_required_and_an_admitted_staircase_2026_10_04() {
        let ok = |steps: u8, growth_bps: u16| validate_stair(&StairArg { steps, growth_bps }).is_ok();
        assert!(!ok(0, 0), "no staircase: refused since the wall was deleted (2026-10-04)");
        assert!(ok(floor_math::stair::custom_stair_steps(50_000), 10_000), "a Custom launch's derived staircase");
        assert!(ok(11, 10_000) && ok(9, 10_000) && ok(8, 9_500), "the first staircase presets");
        assert!(ok(14, 10_000) && ok(13, 10_400) && ok(11, 9_880), "the three presets (OWNER RULINGS 2026-09-30, to about $1B, back-loaded)");
        assert!(ok(1, 5_000) && ok(16, 20_000), "the window's edges");
        assert!(!ok(0, 10_000), "a growth with no steps");
        assert!(!ok(17, 10_000) && !ok(8, 4_999) && !ok(8, 20_001) && !ok(8, 0));
        assert_eq!(StairArg::default(), StairArg { steps: 0, growth_bps: 0 }, "absent = no staircase, refused");
        assert!(validate_stair(&StairArg::default()).is_err());
    }
}
