//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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
use floor_types::constants::{
    BPS, MAX_CREATOR_BPS, MAX_HOLDER_BPS, MAX_NAME_LEN, MAX_SKIM_FAR_BPS, MAX_SYMBOL_LEN,
    MAX_TICK, MAX_URI_LEN, MIN_TICK, TICKS_PER_DOUBLING,
};
pub use floor_math::modes::{preset, split_tokens, Mode, Preset, MAX_TRANCHE_PRICE_X64 as PRICE_X64_LIMIT, STANDARD_POL_BPS, STANDARD_SALE_BPS};
pub use floor_math::price::open_tick;
pub use floor_math::sale::{sale_buy_with_team, sale_tokens_for, sell_back_settle, team_fee_of};
pub use floor_math::tranche_price_x64;

use crate::errors::LaunchError;
use crate::state::CreateLaunchParams;

///
///
///
///
pub const OPEN_TICK_MIN_EXCLUSIVE: i32 = MIN_TICK + 3 * TICKS_PER_DOUBLING + 60;
pub const OPEN_TICK_MAX_EXCLUSIVE: i32 = MAX_TICK - 2 * 60;





///
///
///
///
pub fn sell_back_payout(contributed: u64, tokens: u64, allocation: u64) -> Option<u64> {
    if allocation == 0 || tokens > allocation {
        return None;
    }
    let p = (contributed as u128) * (tokens as u128) / (allocation as u128);
    u64::try_from(p).ok()
}

///
///
///
///
///
///
///
///
pub fn backing_at_sellout(price_x64: u128, platform_bps: u16, sale_tokens: u64) -> Option<u64> {
    let raised = (sale_tokens as u128).checked_mul(price_x64)? >> 64;
    let raised = u64::try_from(raised).ok()?;
    let cut = (raised as u128 * platform_bps as u128 / BPS as u128) as u64;
    raised.checked_sub(cut)
}

///
pub fn open_tick_in_bounds(tick: i32) -> bool {
    tick > OPEN_TICK_MIN_EXCLUSIVE && tick < OPEN_TICK_MAX_EXCLUSIVE
}





///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedConfig {
    pub mode: u8,
    pub mcap_usd: u64,
    pub skim_buy_far_bps: u16,
    pub skim_sell_far_bps: u16,
///
///
///
    pub creator_buy_bps: u16,
    pub creator_sell_bps: u16,
///
///
///
///
///
    pub holder_buy_bps: u16,
    pub holder_sell_bps: u16,
    pub price_x64: u128,
    pub open_tick: i32,
    pub pending: bool,

///
///
///
///
    pub sale_tokens: u64,
    pub pol_tokens: u64,
    pub stair_tokens: u64,


///
///
///
///
///
///
///
    pub skim_sell_at_floor_multiple_bps: u32,
///
///
///
///
///
///
    pub stair_steps: u8,
    pub stair_growth_bps: u16,
///
///
    pub team_buy_max_bps: u16,
///
///
    pub team_vest_days: Option<[u16; 4]>,
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
pub fn hooked_for(params: &CreateLaunchParams) -> bool {
    params.holder_buy_bps > 0 || params.holder_sell_bps > 0
}

impl ResolvedConfig {
///
    pub fn hooked(&self) -> bool {
        self.holder_buy_bps > 0 || self.holder_sell_bps > 0
    }
}






///
///
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Upline {
    pub tier1: Pubkey,
    pub tier2: Pubkey,
    pub tier3: Pubkey,
}

impl Upline {
///
    pub fn is_set(&self) -> bool {
        self.tier1 != Pubkey::default()
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
pub fn creator_upline(params: &CreateLaunchParams, creator: &Pubkey, link_owner: &Pubkey, link_data: &[u8]) -> Upline {
    use crate::state::TraderLink;

    let wallet_slot = |k: &Pubkey| params.recipients.iter().any(|s| s.platform == 0 && s.key == k.to_bytes());
    if *link_owner != crate::ID || link_data.len() != TraderLink::LEN || link_data[..8] != *TraderLink::DISCRIMINATOR
    {
        return Upline::default();
    }
    let l: TraderLink = bytemuck::pod_read_unaligned(&link_data[8..TraderLink::LEN]);
    if l.trader != *creator
        || l.tier1 == Pubkey::default()
        || !floor_types::register::referral_tiers_well_formed(
            &creator.to_bytes(),
            &l.tier1.to_bytes(),
            &l.tier2.to_bytes(),
            &l.tier3.to_bytes(),
        )
        || [&l.tier1, &l.tier2, &l.tier3].iter().any(|k| **k != Pubkey::default() && wallet_slot(k))
    {
        return Upline::default();
    }
    Upline { tier1: l.tier1, tier2: l.tier2, tier3: l.tier3 }
}

///
///
///


#[allow(deprecated)]
pub fn is_wallet_key(key: &Pubkey) -> bool {
    solana_curve25519::edwards::validate_edwards(&solana_curve25519::edwards::PodEdwardsPoint(key.to_bytes()))
}

///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigFault {
///
    Config,
///
    Recipient,
///
///
    SkimSellMultiple,
///
    SkimSellAtFloorCap,
///
///
    StairDoesNotFit,
///
///
    Recipients(u8),
///
    PresetUnknown,
///
    PresetDisabled,
}

impl From<ConfigFault> for LaunchError {
    fn from(f: ConfigFault) -> Self {
        match f {
            ConfigFault::Config => LaunchError::BadConfig,
            ConfigFault::Recipient => LaunchError::BadRecipientConfig,
            ConfigFault::SkimSellMultiple => LaunchError::SkimSellMultipleOutOfRange,
            ConfigFault::SkimSellAtFloorCap => LaunchError::SkimSellAtFloorAboveCap,
            ConfigFault::StairDoesNotFit => LaunchError::StairDoesNotFit,
            ConfigFault::Recipients(code) => {
                msg!("BadRecipients code {}", code);
                LaunchError::BadRecipients
            }
            ConfigFault::PresetUnknown => LaunchError::PresetUnknown,
            ConfigFault::PresetDisabled => LaunchError::PresetDisabled,
        }
    }
}

///
///
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TeamBuy {
    pub tokens: u64,
///
    pub step_tokens: u64,
///
    pub wallet_tokens: u64,
    pub lock_days: u16,
    pub vest_days: [u16; 4],

}

///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeamBuyFault {
///
    OutOfRange,
///
    NeedsStaircase,
///
    OverCap,
///
///
    Payees(u8),
}

impl From<TeamBuyFault> for LaunchError {
    fn from(f: TeamBuyFault) -> Self {
        match f {
            TeamBuyFault::OutOfRange => LaunchError::TeamBuyOutOfRange,
            TeamBuyFault::NeedsStaircase => LaunchError::TeamBuyNeedsStaircase,
            TeamBuyFault::OverCap => LaunchError::TeamStepsOverCap,
            TeamBuyFault::Payees(code) => {
                msg!("TeamBuyOutOfRange: team payees code {}", code);
                LaunchError::TeamBuyOutOfRange
            }
        }
    }
}





///
///
///
///
///
pub fn preset_from_registry_data(data: &[u8], mode: u8) -> core::result::Result<Option<floor_math::presets::PresetRow>, ConfigFault> {
    use floor_types::layout::Layout;
    if mode == floor_types::constants::MODE_CUSTOM {
        return Ok(None);
    }
    let reg = floor_types::layout::PresetRegistry::DISCRIMINATOR;
    if data.len() != crate::state::PresetRegistry::LEN || data[..8] != reg {
        return Err(ConfigFault::PresetUnknown);
    }
    let count = data[8 + 2];
    if mode > floor_types::constants::MAX_PRESET_ID || mode >= count {
        return Err(ConfigFault::PresetUnknown);
    }
    let at = 8 + 72 + floor_types::constants::PRESET_ROW_LEN * mode as usize;
    let row: crate::state::PresetRowZc = bytemuck::pod_read_unaligned(&data[at..at + floor_types::constants::PRESET_ROW_LEN]);
    if row.enabled == 0 {
        return Err(ConfigFault::PresetDisabled);
    }
    Ok(Some(row.numbers()))
}

///
///
///
///
///
pub fn resolve_team_buy(p: &CreateLaunchParams, r: &ResolvedConfig) -> core::result::Result<Option<TeamBuy>, TeamBuyFault> {
    use floor_types::constants::{BPS, TEAM_DAYS_MAX, TEAM_LOCK_MIN_DAYS, TEAM_STEPS_MIN_BPS, SUPPLY};
    if p.team_buy_tokens == 0 {

        let none = p.team_steps_bps == 0 && p.team_lock_days == 0 && p.team_vest_days == [0; 4] && p.team_payees.is_empty();
        return if none { Ok(None) } else { Err(TeamBuyFault::OutOfRange) };
    }


    if r.stair_steps == 0 {
        return Err(TeamBuyFault::NeedsStaircase);
    }



    let max = SUPPLY / BPS * r.team_buy_max_bps as u64;
    let steps_ok = (TEAM_STEPS_MIN_BPS..=BPS as u16).contains(&p.team_steps_bps);
    let lock_ok = (TEAM_LOCK_MIN_DAYS..=TEAM_DAYS_MAX).contains(&p.team_lock_days);
    if p.team_buy_tokens > max || p.team_buy_tokens >= r.sale_tokens || !steps_ok || !lock_ok {
        return Err(TeamBuyFault::OutOfRange);
    }


    let vest_days = match r.team_vest_days {
        Some(days) if p.team_vest_days == [0; 4] => days,
        Some(_) => return Err(TeamBuyFault::OutOfRange),
        None => {
            let d = p.team_vest_days;
            let (lo, hi) = (floor_types::constants::TEAM_CUSTOM_VEST_DAYS_MIN, floor_types::constants::TEAM_CUSTOM_VEST_DAYS_MAX);
            if d.iter().any(|&x| !(lo..=hi).contains(&x)) || d.windows(2).any(|w| w[0] >= w[1]) {
                return Err(TeamBuyFault::OutOfRange);
            }
            d
        }
    };
    let wallet = (p.team_buy_tokens as u128 * (BPS as u128 - p.team_steps_bps as u128) / BPS as u128) as u64;
    let steps = p.team_buy_tokens - wallet;
    if !floor_math::owners::team_buy_ok_max(r.team_buy_max_bps as u64, p.team_buy_tokens, steps) {
        return Err(TeamBuyFault::OutOfRange);
    }




    let (reserve_steps, _tail) = if r.mode == floor_types::constants::MODE_CUSTOM {
        floor_math::stair::stair_tail_split_supply(r.stair_tokens, &floor_math::stair::PRESET_STAIR)
    } else {
        floor_math::stair::stair_tail_split(r.stair_tokens)
    };
    let staircase = reserve_steps.checked_add(steps).ok_or(TeamBuyFault::OverCap)?;
    if !floor_math::owners::team_steps_fit(steps, staircase) {
        return Err(TeamBuyFault::OverCap);
    }
    Ok(Some(TeamBuy {
        tokens: p.team_buy_tokens,
        step_tokens: steps,
        wallet_tokens: wallet,
        lock_days: p.team_lock_days,
        vest_days,
    }))
}

///
///
///
///
///
pub fn resolve_team_payees(
    p: &CreateLaunchParams,
    creator: &Pubkey,
) -> core::result::Result<Vec<floor_types::slots::SlotArg>, TeamBuyFault> {
    if p.team_payees.is_empty() {
        return Ok(vec![floor_types::slots::SlotArg {
            bps: floor_types::slots::SLOTS_TOTAL_BPS,
            platform: 0,
            key: creator.to_bytes(),
        }]);
    }
    let slots: Vec<floor_math::Slot> = p.team_payees.iter().map(Into::into).collect();
    let code = floor_math::check_slots(&slots);
    if code != 0 {
        return Err(TeamBuyFault::Payees(code));
    }
    Ok(p.team_payees.iter().map(Into::into).collect())
}

///
///
///
pub fn register_slots(
    p: &CreateLaunchParams,
    team: Vec<floor_types::slots::SlotArg>,
) -> floor_types::slots::SlotsArg {
    floor_types::slots::SlotsArg { creator: p.recipients.iter().map(Into::into).collect(), team }
}

///
///
///
///
///
///
pub fn team_vested_slot_part(
    total: u64,
    days: &[u16; 4],
    anchor: i64,
    now: i64,
    bps: u16,
    paid_mask: u8,
) -> (u64, u8) {
    let at = |k: usize| anchor.saturating_add(days[k] as i64 * 86_400);
    let v_after = |cliffs: usize| if cliffs == 0 { 0 } else { floor_math::owners::team_vested(total, days, anchor, at(cliffs - 1)) };
    let paid = (0..4).filter(|k| paid_mask & (1 << k) != 0).map(|k| k + 1).max().unwrap_or(0);
    let cliffs = (0..4).filter(|&k| now >= at(k)).count();
    if cliffs <= paid {
        return (0, paid_mask);
    }
    let amount = floor_math::slot_cliff_part(v_after(paid), v_after(cliffs), bps);
    let mask = (paid..cliffs).fold(paid_mask, |m, k| m | (1 << k));
    (amount, mask)
}

///
///
pub fn slot_cliff_mask(by_slot: u32, slot: u8) -> u8 {
    ((by_slot >> (4 * slot as u32)) & 0xF) as u8
}

///
///
pub fn with_slot_cliff_mask(by_slot: u32, slot: u8, mask: u8) -> u32 {
    let shift = 4 * slot as u32;
    (by_slot & !(0xF << shift)) | (((mask & 0xF) as u32) << shift)
}

///
///
pub fn team_buy_cost(tokens: u64, price_x64: u128) -> Option<u64> {
    let prod = (tokens as u128).checked_mul(price_x64)?;
    u64::try_from(prod.div_ceil(1u128 << 64)).ok()
}

///
///
///
pub fn abi_encode_pending(platform: u8, handle: &[u8]) -> Vec<u8> {
    let padded = handle.len().div_ceil(32) * 32;
    let mut out = vec![0u8; 96 + padded];
    out[31] = platform;
    out[63] = 0x40;
    out[88..96].copy_from_slice(&(handle.len() as u64).to_be_bytes());
    out[96..96 + handle.len()].copy_from_slice(handle);
    out
}

///
///
pub fn pending_recipient_id(platform: u8, handle: &[u8]) -> [u8; 32] {
    solana_keccak_hasher::hashv(&[&abi_encode_pending(platform, handle)]).to_bytes()
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
pub fn resolve_config(
    params: &CreateLaunchParams,
    name: &str,
    symbol: &str,
    uri: &str,
    platform_bps: u16,
    min_bid: u64,
    quote_decimals: u8,
    preset: Option<&floor_math::presets::PresetRow>,
) -> core::result::Result<ResolvedConfig, ConfigFault> {
    resolve_config_chained(params, name, symbol, uri, platform_bps, min_bid, quote_decimals, 0, preset)
}

///
///
///
///
///
///
///
///
#[allow(clippy::too_many_arguments)]
pub fn resolve_config_chained(
    params: &CreateLaunchParams,
    name: &str,
    symbol: &str,
    uri: &str,
    platform_bps: u16,
    min_bid: u64,
    quote_decimals: u8,
    chain_ticks: i64,
    preset: Option<&floor_math::presets::PresetRow>,
) -> core::result::Result<ResolvedConfig, ConfigFault> {
    use ConfigFault::Config;

    if name.is_empty() || name.len() > MAX_NAME_LEN || symbol.is_empty() || symbol.len() > MAX_SYMBOL_LEN {
        return Err(Config);
    }




    if uri.len() > MAX_URI_LEN || params.quote_usd_cents == 0 {
        return Err(Config);
    }






    let slots: Vec<floor_math::Slot> = params.recipients.iter().map(Into::into).collect();
    let code = floor_math::check_slots(&slots);
    if code != 0 {
        return Err(ConfigFault::Recipients(code));
    }
    let pending = params.recipients.iter().any(|s| s.platform != 0);




    let custom = params.mode == floor_types::constants::MODE_CUSTOM;
    let row = if custom { None } else { Some(*preset.ok_or(ConfigFault::PresetUnknown)?) };





    let (mcap_usd, creator_buy_bps, creator_sell_bps, holder_buy_bps, holder_sell_bps, skim_buy_far_bps, skim_sell_far_bps) =
        if let Some(p) = row {





            if params.graduation_mcap_usd != 0
                || params.skim_buy_bps != 0
                || params.skim_sell_bps != 0
                || params.creator_buy_bps != 0
                || params.creator_sell_bps != 0
                || params.holder_buy_bps != 0
                || params.holder_sell_bps != 0

                || params.sale_bps != 0
                || params.pol_bps != 0

                || params.skim_sell_at_floor_multiple_bps != 0
            {
                return Err(Config);
            }

            (p.graduation_mcap_usd, p.creator_bps, p.creator_bps, p.holder_bps, p.holder_bps, p.skim_buy_far_bps, p.skim_sell_far_bps)
        } else {






            if params.graduation_mcap_usd == 0
                || params.skim_buy_bps > MAX_SKIM_FAR_BPS
                || params.skim_sell_bps > MAX_SKIM_FAR_BPS
                || params.creator_buy_bps > MAX_CREATOR_BPS
                || params.creator_sell_bps > MAX_CREATOR_BPS
                || params.holder_buy_bps > MAX_HOLDER_BPS
                || params.holder_sell_bps > MAX_HOLDER_BPS



                || ((params.sale_bps == 0) != (params.pol_bps == 0))
            {
                return Err(Config);
            }
            (
                params.graduation_mcap_usd,
                params.creator_buy_bps,
                params.creator_sell_bps,
                params.holder_buy_bps,
                params.holder_sell_bps,
                params.skim_buy_bps,
                params.skim_sell_bps,
            )
        };




    let (split_sale_bps, split_pol_bps) = match row {
        Some(p) => (p.sale_bps, p.pol_bps),
        None if params.sale_bps == 0 => (STANDARD_SALE_BPS, STANDARD_POL_BPS),
        None => (params.sale_bps, params.pol_bps),
    };
    let (sale_tokens, pol_tokens, stair_tokens) = split_tokens(split_sale_bps, split_pol_bps).map_err(|_| Config)?;



    let skim_sell_at_floor_multiple_bps = match row {
        Some(p) => p.skim_sell_at_floor_multiple_bps,
        None if params.skim_sell_at_floor_multiple_bps == 0 => floor_math::modes::skim_sell_at_floor_multiple_for(Mode::Custom),
        None => {
            let m = u32::from(params.skim_sell_at_floor_multiple_bps);
            if !(floor_types::constants::MIN_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS..=floor_types::constants::MAX_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS)
                .contains(&m)
            {
                return Err(ConfigFault::SkimSellMultiple);
            }
            m
        }
    };


    if floor_types::constants::skim_sell_at_floor_bps_at(skim_sell_far_bps, skim_sell_at_floor_multiple_bps)
        > floor_types::constants::MAX_SKIM_SELL_AT_FLOOR_BPS
    {
        return Err(ConfigFault::SkimSellAtFloorCap);
    }



    let (stair_steps, stair_growth_bps) = match row {
        Some(p) => (p.stair_steps, p.stair_growth_bps),
        None => floor_math::stair::custom_stair(mcap_usd),
    };


    let (team_buy_max_bps, team_vest_days) = match row {
        Some(p) => (p.team_buy_max_bps, Some(p.team_vest_days)),
        None => (floor_types::constants::TEAM_BUY_MAX_BPS, None),
    };








    if pending && creator_buy_bps == 0 && creator_sell_bps == 0 {
        return Err(ConfigFault::Recipients(floor_math::slots::CHECK_PENDING_WITHOUT_RATE));
    }









    let root_price_x64 = tranche_price_x64(mcap_usd, params.quote_usd_cents, quote_decimals).map_err(|_| Config)?;
    let price_x64 = floor_math::chained_price_x64(root_price_x64, chain_ticks).map_err(|_| Config)?;
    let backing = backing_at_sellout(price_x64, platform_bps, sale_tokens).ok_or(Config)?;
    if backing < min_bid {
        return Err(Config);
    }
    let open = open_tick(price_x64).map_err(|_| Config)?;
    if !open_tick_in_bounds(open) {
        return Err(Config);
    }





    let edge = floor_math::floor_to_spacing(open, floor_types::constants::FLOOR_TICK_SPACING as i32).map_err(|_| Config)?;
    if !floor_math::stair::stair_fits(edge, stair_steps, floor_types::constants::FLOOR_TICK_SPACING as i32).map_err(|_| Config)? {
        return Err(ConfigFault::StairDoesNotFit);
    }

    Ok(ResolvedConfig {
        mode: params.mode,
        mcap_usd,
        skim_buy_far_bps,
        skim_sell_far_bps,
        creator_buy_bps,
        creator_sell_bps,
        holder_buy_bps,
        holder_sell_bps,
        price_x64,
        open_tick: open,
        pending,
        sale_tokens,
        pol_tokens,
        stair_tokens,
        skim_sell_at_floor_multiple_bps,
        stair_steps,
        stair_growth_bps,
        team_buy_max_bps,
        team_vest_days,
    })
}





///
///
///
///
pub fn create_pda<'info>(
    payer: &AccountInfo<'info>,
    target: &AccountInfo<'info>,
    space: usize,
    owner: &Pubkey,
    signer_seeds: &[&[u8]],
) -> Result<()> {
    require!(
        target.owner == &system_program::ID && target.data_is_empty(),
        LaunchError::AccountInUse
    );
    let required = Rent::get()?.minimum_balance(space);
    let have = target.lamports();
    if have < required {
        system_program::transfer(
            CpiContext::new(
                system_program::ID,
                system_program::Transfer { from: payer.clone(), to: target.clone() },
            ),
            required - have,
        )?;
    }
    if space > 0 {
        system_program::allocate(
            CpiContext::new_with_signer(
                system_program::ID,
                system_program::Allocate { account_to_allocate: target.clone() },
                &[signer_seeds],
            ),
            space as u64,
        )?;
    }
    system_program::assign(
        CpiContext::new_with_signer(
            system_program::ID,
            system_program::Assign { account_to_assign: target.clone() },
            &[signer_seeds],
        ),
        owner,
    )?;
    Ok(())
}





///
///
pub struct GraduationCtx<'a, 'info> {
///
    pub payer: &'a AccountInfo<'info>,
    pub launch: &'a AccountInfo<'info>,
    pub sale_vault: &'a AccountInfo<'info>,
    pub token_vault: &'a AccountInfo<'info>,
    pub mint: &'a AccountInfo<'info>,
    pub lock_authority: &'a AccountInfo<'info>,
///
///
///
///
///
    pub launch_bump: u8,
///
    pub lock_bump: u8,
    pub token_2022: &'a AccountInfo<'info>,
    pub system_program: &'a AccountInfo<'info>,
    pub event_authority: &'a AccountInfo<'info>,
    pub program: &'a AccountInfo<'info>,
///
    pub graduation_accounts: &'a [AccountInfo<'info>],
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
pub fn graduate_hook(g: GraduationCtx<'_, '_>) -> Result<()> {
    crate::graduate::run(&crate::graduate::Graduation {
        payer: g.payer,
        launch: g.launch,
        sale_vault: g.sale_vault,
        token_vault: g.token_vault,
        mint: g.mint,
        lock_authority: g.lock_authority,
        lock_bump: g.lock_bump,
        token_2022: g.token_2022,
        system_program: g.system_program,
        event_authority: g.event_authority,
        program: g.program,
        graduation_accounts: g.graduation_accounts,
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use floor_types::constants::{
        MAX_QUOTE_DECIMALS, MIN_BID_LAMPORTS, MIN_QUOTE_DECIMALS, MODE_BLUECHIP, MODE_MOONSHOT, MODE_COOK, MODE_CUSTOM,
        PLATFORM_BPS, POL_TOKENS, SALE_TOKENS, LADDER_TOKENS,
    };

///
    const WSOL_DECIMALS: u8 = 9;
    const SOL_150: u64 = 150 * 100;

    fn params(mode: u8) -> CreateLaunchParams {
        CreateLaunchParams {
            mode,
            quote_usd_cents: SOL_150,
            recipients: vec![wallet([7; 32], 10_000)],
            ..Default::default()
        }
    }

///
    fn wallet(key: [u8; 32], bps: u16) -> crate::state::SlotArg {
        crate::state::SlotArg { bps, platform: 0, key }
    }

///
    fn handle(h: &[u8], bps: u16) -> crate::state::SlotArg {
        crate::state::SlotArg { bps, platform: 1, key: pending_recipient_id(1, h) }
    }

///
///
    fn seeded(p: &CreateLaunchParams) -> Option<floor_math::presets::PresetRow> {
        floor_math::presets::seed_row(p.mode)
    }

    fn resolve(p: &CreateLaunchParams) -> core::result::Result<ResolvedConfig, ConfigFault> {
        resolve_config(p, "Floor", "FLR", "", PLATFORM_BPS, MIN_BID_LAMPORTS, WSOL_DECIMALS, seeded(p).as_ref())
    }

    fn hex32(s: &str) -> [u8; 32] {
        let mut out = [0u8; 32];
        for (i, b) in out.iter_mut().enumerate() {
            *b = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap();
        }
        out
    }

///
///
    #[test]
    fn floor_math_gives_the_values_the_copies_pinned() {



        assert_eq!(tranche_price_x64(20_000, 150 * 100, 9), Ok(2_459_565_876_494_606));
        assert_eq!(tranche_price_x64(100_000, 100, 9), Ok(1_844_674_407_370_955_161));
        assert!(tranche_price_x64(1, 0, 9).is_err());
        assert!(tranche_price_x64(8_192_000_000, 100, 9).is_err(), "at 2^77 the price is refused");

        let p = tranche_price_x64(20_000, 150 * 100, 9).unwrap();
        for lamports in [1u64, 2, 999, 1_000_000_007, 73_333_333_334] {
            let (t, c) = sale_tokens_for(lamports, p, SALE_TOKENS).unwrap();
            assert!(c <= lamports, "cost {c} > lamports {lamports}");
            assert_eq!(c as u128, ((t as u128) * p).div_ceil(1u128 << 64), "exact ceil");
        }
        let (t, c) = sale_tokens_for(u64::MAX / 2, p, 10).unwrap();
        assert_eq!((t, c), (10, 1), "capped by the remainder; a fraction of a lamport rounds up to one");
    }

    #[test]
    fn sell_back_is_exact_across_partials() {
        let (contributed, allocation) = (1_000_000_007u64, 3_000_000_000u64);
        let a = sell_back_payout(contributed, 1_000_000_000, allocation).unwrap();
        let rest = sell_back_payout(contributed - a, 2_000_000_000, allocation - 1_000_000_000).unwrap();
        assert_eq!(a + rest, contributed);
        assert_eq!(sell_back_payout(1, 2, 1), None);
        assert_eq!(sell_back_payout(1, 0, 0), None);
    }

///
    #[test]
    fn sell_back_parts_net_their_own_fees_and_sum_to_the_contribution_less_the_fees() {
        let (contributed, allocation) = (7_777_777_777u64, 1_000_003u64);
        let first = sell_back_payout(contributed, allocation / 3, allocation).unwrap();
        let rest = sell_back_payout(contributed - first, allocation - allocation / 3, allocation - allocation / 3).unwrap();
        let (f1, r1) = floor_math::sell_back_with_team(first, 25).unwrap();
        let (f2, r2) = floor_math::sell_back_with_team(rest, 25).unwrap();
        assert_eq!(first + rest, contributed);
        assert_eq!(r1 + r2, contributed - f1 - f2);
    }

    #[test]
    fn whole_sale_sell_back_at_the_bluechip_cap_needs_u128() {
        let p = tranche_price_x64(100_000, 100, 9).unwrap();
        let (t, c) = sale_tokens_for(u64::MAX, p, SALE_TOKENS).unwrap();
        assert_eq!(t, SALE_TOKENS);
        assert!((c as u128) * (t as u128) > u64::MAX as u128, "the product overflows u64");
        assert_eq!(sell_back_payout(c, t, t), Some(c));
    }

    #[test]
    fn abi_encode_is_solidity_bytes() {
        let e = abi_encode_pending(1, b"floor_dev");
        assert_eq!(e.len(), 128);
        assert_eq!((e[31], e[63], e[95]), (1, 0x40, 9));
        assert_eq!(&e[96..105], b"floor_dev");
        assert_eq!(abi_encode_pending(2, &[b'a'; 32]).len(), 128, "a 32-byte handle adds no pad word");

        assert_eq!(
            pending_recipient_id(1, b"floor_dev"),
            hex32("abc6f518cd4baa41d37e36cd4ddc19658259e9ccd7e27ba9dca1c381b11bec4a")
        );
        assert_eq!(
            pending_recipient_id(2, b"abcdefghijklmnopqrstuvwxyz012345"),
            hex32("87a3671855d904bf1787d1622f1b54c031d8d7bb490b2e5300ab61cdc585e4ed")
        );
    }

///
///
///
///
    #[test]
    fn the_pending_handle_vectors_are_the_sdks() {
        let src = include_str!("../../../crates/floor-math/fixtures/pending-handle-vectors.json");
        let (mut accepted, mut refused) = (0, 0);
        for line in src.lines() {
            let t = line.trim().trim_end_matches(',');
            if !t.starts_with("[\"") {
                continue;
            }
            let cells: Vec<&str> = t.trim_matches(|c| c == '[' || c == ']').split(',').map(|c| c.trim().trim_matches('"')).collect();
            let (platform, committed, id) = (cells[0].parse::<u8>().unwrap(), cells[1], cells[2]);
            if committed.is_empty() {
                assert!(id.is_empty(), "a refused row carries no id");
                refused += 1;
                continue;
            }
            assert_eq!(pending_recipient_id(platform, committed.as_bytes()), hex32(id), "{platform}:{committed}");
            accepted += 1;
        }
        assert!(accepted >= 15 && refused >= 10, "{accepted} accepted / {refused} refused rows");
    }

///
///
///
///
    #[test]
    fn the_at_floor_sell_multiple_resolves_from_the_mode_2026_09_29() {
        for m in [MODE_COOK, MODE_BLUECHIP, MODE_MOONSHOT] {
            assert_eq!(resolve(&params(m)).unwrap().skim_sell_at_floor_multiple_bps, 15_000, "{m}: 1.5x (2026-10-12; ~~5x~~)");
        }
        let c = resolve(&custom(10, 10, 0, 0)).unwrap();
        assert_eq!(c.skim_sell_at_floor_multiple_bps, 15_000, "Custom sending 0: 1.5x");
        let c = resolve(&custom(500, 500, 1_000, 1_000)).unwrap();
        assert_eq!(c.skim_sell_at_floor_multiple_bps, 15_000, "Custom's own rates do not move it");
        assert_eq!(floor_types::constants::SKIM_SELL_AT_FLOOR_MULTIPLE_BPS, 15_000);
        assert_eq!(floor_types::constants::PRESET_SKIM_SELL_AT_FLOOR_MULTIPLE_BPS, 15_000);
    }

///
///
///
///
///
///
///
    #[test]
    fn the_at_floor_sell_skim_is_capped_at_fifteen_percent_2026_09_30() {
        let with = |far: u16, m: u16| CreateLaunchParams { skim_sell_bps: far, skim_sell_at_floor_multiple_bps: m, ..custom(10, 10, 0, 0) };
        assert!(resolve(&with(1_000, 15_000)).is_ok(), "10 % far at 1.5x = 1,500: exactly allowed");
        assert!(resolve(&with(1_000, 0)).is_ok(), "10 % far at the default 1.5x");
        assert_eq!(resolve(&with(1_000, 16_000)), Err(ConfigFault::SkimSellAtFloorCap));
        assert!(resolve(&with(300, 50_000)).is_ok(), "3 % far at 5x = 1,500: exactly allowed");
        assert_eq!(resolve(&with(310, 50_000)), Err(ConfigFault::SkimSellAtFloorCap));
        assert_eq!(LaunchError::from(ConfigFault::SkimSellAtFloorCap) as u32 + 6000, 6057);
        for m in [MODE_COOK, MODE_BLUECHIP, MODE_MOONSHOT] {
            assert!(resolve(&params(m)).is_ok(), "{m}: presets unaffected");
        }
    }

    #[test]
    fn custom_chooses_its_at_floor_sell_multiple_2026_09_29() {
        let with = |m: u16| CreateLaunchParams { skim_sell_at_floor_multiple_bps: m, ..custom(10, 10, 0, 0) };
        assert_eq!(resolve(&with(0)).unwrap().skim_sell_at_floor_multiple_bps, 15_000, "0 = the default 1.5x");
        for m in [10_000u16, 15_000, 32_500, 50_000] {
            assert_eq!(resolve(&with(m)).unwrap().skim_sell_at_floor_multiple_bps, u32::from(m), "{m}");
        }
        for m in [1u16, 9_999, 50_001, u16::MAX] {
            assert_eq!(resolve(&with(m)), Err(ConfigFault::SkimSellMultiple), "{m}");
        }
        assert_eq!(LaunchError::from(ConfigFault::SkimSellMultiple) as u32 + 6000, 6056);









        let wire = |n: usize| {
            let recipients = (0..n).map(|i| wallet([i as u8 + 1; 32], if i == 0 { 10_000 - (n as u16 - 1) } else { 1 })).collect();
            let p = CreateLaunchParams { recipients, skim_sell_at_floor_multiple_bps: 50_000, ..custom(10, 10, 0, 0) };
            let mut v = Vec::new();
            anchor_lang::AnchorSerialize::serialize(&p, &mut v).unwrap();
            v
        };
        for n in 1..=5usize {
            let b = wire(n);
            assert_eq!(b.len(), 63 + 35 * n, "{n} slots");
            assert_eq!(&b[37 + 35 * n..39 + 35 * n], &50_000u16.to_le_bytes(), "{n} slots: the choice's two bytes");
        }
        for m in [MODE_COOK, MODE_BLUECHIP, MODE_MOONSHOT] {
            let p = CreateLaunchParams { skim_sell_at_floor_multiple_bps: 50_000, ..params(m) };
            assert_eq!(resolve(&p), Err(ConfigFault::Config), "{m}: a preset sends 0");
            assert_eq!(resolve(&params(m)).unwrap().skim_sell_at_floor_multiple_bps, 15_000, "{m}: its own 1.5x (2026-10-12; ~~5x~~)");
        }
    }

    #[test]
    fn presets_are_floor_modes() {








        assert_eq!(preset(Mode::Cook).graduation_mcap_usd, 50_000);


        assert_eq!((preset(Mode::Cook).stair_steps, preset(Mode::Cook).stair_growth_bps), (15, 10_400));

        assert_eq!((preset(Mode::Bluechip).stair_steps, preset(Mode::Bluechip).stair_growth_bps), (14, 10_400));
        assert_eq!((preset(Mode::Moonshot).stair_steps, preset(Mode::Moonshot).stair_growth_bps), (15, 10_000));
        assert_eq!(preset(Mode::Moonshot).graduation_mcap_usd, 25_000);


        assert_eq!(preset(Mode::Cook).skim_buy_far_bps, 200);
        assert_eq!(preset(Mode::Cook).creator_bps, 75);
        assert_eq!((preset(Mode::Moonshot).creator_bps, preset(Mode::Moonshot).skim_sell_far_bps), (50, 200));
        assert_eq!(preset(Mode::Bluechip).creator_bps, 100);
        assert_eq!(preset(Mode::Bluechip).skim_sell_far_bps, 200);
        assert_eq!(preset(Mode::Bluechip).holder_bps, 0);
        assert_eq!((preset(Mode::Cook).holder_bps, preset(Mode::Moonshot).holder_bps), (0, 0));
        assert_eq!(preset(Mode::Bluechip).graduation_mcap_usd, 100_000);
        assert_eq!((MAX_CREATOR_BPS, MAX_HOLDER_BPS, MAX_SKIM_FAR_BPS), (500, 1000, 1000));
        assert_eq!(preset(Mode::Custom), Preset::default());

        assert_eq!((Mode::try_from(MODE_COOK), Mode::try_from(MODE_MOONSHOT)), (Ok(Mode::Cook), Ok(Mode::Moonshot)));
        assert_eq!((MODE_CUSTOM, MODE_MOONSHOT, MODE_COOK, MODE_BLUECHIP), (0, 1, 2, 3));
        assert!(Mode::try_from(4).is_err());
    }

///
///
///
///
    #[test]
    fn the_declared_cents_must_not_be_zero() {
        let mut p = params(MODE_COOK);
        p.quote_usd_cents = 0;
        assert_eq!(resolve(&p), Err(ConfigFault::Config), "no declared quote price");
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
    #[test]
    fn the_wire_is_short_refused_and_long_tolerated() {
        use anchor_lang::{AnchorDeserialize, AnchorSerialize};

        let mut p = params(MODE_COOK);
        p.recipients = vec![wallet([9; 32], 10_000)];
        let mut full = Vec::new();
        p.serialize(&mut full).unwrap();



        assert!(CreateLaunchParams::deserialize(&mut &full[..full.len() - 2][..]).is_err(), "a short payload");

        let short = &full[..full.len() - 8];
        assert!(
            CreateLaunchParams::deserialize(&mut &short[..]).is_err(),
            "a short payload must fail borsh, not be re-interpreted"
        );


        let mut long = full.clone();
        long.extend_from_slice(&[0xAAu8; 8]);
        let round = CreateLaunchParams::deserialize(&mut &long[..]).expect("a long payload deserialises");
        assert_eq!(round, p, "…and the trailing bytes are dropped, not read into a field");


        assert!(CreateLaunchParams::try_from_slice(&long).is_err());

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
    fn the_quote_decimals_reach_the_price_and_the_window_is_enforced() {
        let p = params(MODE_COOK);
        let one_raw_unit = 1u64;
        let mut prev: Option<u128> = None;
        let mut rungs = 0usize;
        for d in MIN_QUOTE_DECIMALS..=MAX_QUOTE_DECIMALS {


            let Ok(r) = resolve_config(&p, "Floor", "FLR", "", PLATFORM_BPS, one_raw_unit, d, seeded(&p).as_ref()) else {
                break;
            };
            if let Some(lo) = prev {



                assert!(
                    r.price_x64 >= lo * 10 && r.price_x64 <= lo * 10 + 9,
                    "one decimal is one factor of ten (d={d}): {lo} then {}",
                    r.price_x64
                );
            }
            prev = Some(r.price_x64);
            rungs += 1;
        }
        assert!(rungs >= 15, "only {rungs} rungs: d = 0..=16 should price at this cap and quote price");



        for d in [0u8, 2] {
            let r = resolve_config(&p, "Floor", "FLR", "", PLATFORM_BPS, one_raw_unit, d, seeded(&p).as_ref())
                .unwrap_or_else(|e| panic!("d={d} must resolve: {e:?}"));
            let backing = backing_at_sellout(r.price_x64, PLATFORM_BPS, r.sale_tokens).expect("backing");
            assert!(backing >= one_raw_unit, "d={d}: backing {backing} is at least one raw unit");
        }



        assert_eq!(
            resolve_config(&p, "Floor", "FLR", "", PLATFORM_BPS, MIN_BID_LAMPORTS, 0, seeded(&p).as_ref()),
            Err(ConfigFault::Config),
            "0.05 SOL as a 0-decimal quote's min_bid is 50 million whole units"
        );




        for d in [MAX_QUOTE_DECIMALS + 1, 100, 255] {
            assert_eq!(
                resolve_config(&p, "Floor", "FLR", "", PLATFORM_BPS, one_raw_unit, d, seeded(&p).as_ref()),
                Err(ConfigFault::Config),
                "d={d} is outside the window"
            );
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
    #[test]
    fn every_launch_resolves_to_a_staircase_custom_derived_from_its_cap() {
        for (mode, steps, growth) in [(MODE_COOK, 15u8, 10_400u16), (MODE_BLUECHIP, 14, 10_400), (MODE_MOONSHOT, 15, 10_000)] {
            let r = resolve(&params(mode)).unwrap();
            assert_eq!((r.stair_steps, r.stair_growth_bps), (steps, growth), "{mode}: its row");
        }
        let custom = |cap: u64| CreateLaunchParams { graduation_mcap_usd: cap, ..params(MODE_CUSTOM) };

        for (cap, steps) in [(25_000u64, 15u8), (50_000, 14), (100_000, 13), (200_000, 12), (250_000, 12), (1_000_000, 10), (10_000_000, 7)] {
            let r = resolve(&custom(cap)).unwrap();
            assert_eq!((r.stair_steps, r.stair_growth_bps), (steps, 10_000), "${cap}");
            assert_eq!(r.mcap_usd, cap);
        }

        let own = CreateLaunchParams { sale_bps: 4_000, pol_bps: 2_000, skim_sell_at_floor_multiple_bps: 30_000, ..custom(100_000) };
        let r = resolve(&own).unwrap();
        assert_eq!((r.sale_tokens, r.pol_tokens, r.stair_tokens), (floor_types::constants::SUPPLY / 10 * 4, floor_types::constants::SUPPLY / 10 * 2, floor_types::constants::SUPPLY / 10 * 4));
        assert_eq!(r.skim_sell_at_floor_multiple_bps, 30_000);
        assert_eq!((r.stair_steps, r.stair_growth_bps), (13, 10_000));
        assert_eq!(resolve(&custom(100_000)).unwrap().skim_sell_at_floor_multiple_bps, 15_000, "0 is the default 1.5x");

        for cap in [1_000u64, 25_000, 690_533, 690_534, 999_999_999] {
            let r = resolve(&custom(cap)).unwrap();
            assert!(r.stair_steps >= 1, "${cap}");
        }
    }

///
///
///
///
///
///
    #[test]
    fn the_dust_line_is_after_the_cut_and_over_the_launchs_own_sale() {
        let mut p = params(MODE_CUSTOM);
        p.graduation_mcap_usd = 50_000;
        let r = resolve(&p).expect("a $50k Custom launch resolves");
        let gross = u64::try_from((r.sale_tokens as u128 * r.price_x64) >> 64).unwrap();
        let net = backing_at_sellout(r.price_x64, PLATFORM_BPS, r.sale_tokens).unwrap();
        assert!(net < gross, "the cut is taken");
        let at = |min_bid: u64, p: &CreateLaunchParams| resolve_config(p, "Floor", "FLR", "", PLATFORM_BPS, min_bid, WSOL_DECIMALS, seeded(p).as_ref());
        assert!(at(net, &p).is_ok(), "exactly the backing passes");
        assert_eq!(at(net + 1, &p).map(|_| ()), Err(ConfigFault::Config), "one unit over the backing is refused…");
        assert!(net + 1 <= gross, "…although the GROSS raise covers it");
        assert_eq!(at(gross, &p).map(|_| ()), Err(ConfigFault::Config), "the gross raise itself is refused");


        let split = |sale: u16, pol: u16| CreateLaunchParams { sale_bps: sale, pol_bps: pol, ..p.clone() };
        let r30 = resolve(&split(3_000, 1_000)).expect("a 30 % sale resolves");
        let line = backing_at_sellout(r30.price_x64, PLATFORM_BPS, r30.sale_tokens).unwrap();
        assert!(at(line, &split(3_000, 1_000)).is_ok(), "30 % of supply clears its own line");
        assert_eq!(
            at(line, &split(1_000, 1_000)).map(|_| ()),
            Err(ConfigFault::Config),
            "10 % of the same cap raises a third of it and is refused at the same line"
        );
    }

    #[test]
    fn resolve_config_follows_floor_factory() {
        let ok = resolve(&params(MODE_COOK)).unwrap();
        assert_eq!(ok.mcap_usd, 50_000);
        assert_eq!((ok.stair_steps, ok.stair_growth_bps), (15, 10_400));
        assert!(open_tick_in_bounds(ok.open_tick));

        let mut p = params(MODE_COOK);
        p.creator_buy_bps = 1;
        assert_eq!(resolve(&p), Err(ConfigFault::Config), "a preset carries no Custom field");

        let mut p = params(MODE_CUSTOM);
        p.graduation_mcap_usd = 50_000;
        p.skim_buy_bps = 1001;
        assert_eq!(resolve(&p), Err(ConfigFault::Config));

        let mut p = params(MODE_COOK);
        p.holder_buy_bps = 1;
        assert_eq!(resolve(&p), Err(ConfigFault::Config), "a preset carries no holder rate (it was D-24 until 2026-09-22)");







        let mut p = params(MODE_CUSTOM);
        p.graduation_mcap_usd = 50_000;
        p.holder_buy_bps = MAX_HOLDER_BPS;
        p.holder_sell_bps = MAX_HOLDER_BPS;
        let r = resolve(&p).expect("1000 bps a side is the EVM's own ceiling");
        assert_eq!((r.holder_buy_bps, r.holder_sell_bps), (1000, 1000));
        p.holder_buy_bps = MAX_HOLDER_BPS + 1;
        assert_eq!(resolve(&p), Err(ConfigFault::Config), "1001 is past the cap");
        let meme = resolve(&params(MODE_COOK)).unwrap();
        assert_eq!((meme.holder_buy_bps, meme.holder_sell_bps), (0, 0), "a preset resolves to 0");


        assert_eq!(resolve(&params(4)), Err(ConfigFault::PresetUnknown), "unknown mode: 4 is not registered yet");
        assert_eq!(resolve(&params(5)), Err(ConfigFault::PresetUnknown));
        assert_eq!(resolve(&params(6)), Err(ConfigFault::PresetUnknown));
        assert!(resolve(&params(MODE_MOONSHOT)).is_ok(), "1 is Moonshot since 2026-10-10");

        assert_eq!(
            resolve_config(&params(MODE_COOK), "", "FLR", "", PLATFORM_BPS, MIN_BID_LAMPORTS, WSOL_DECIMALS, seeded(&params(MODE_COOK)).as_ref()),
            Err(ConfigFault::Config)
        );
        assert_eq!(
            resolve_config(
                &params(MODE_COOK),
                "Floor",
                "FLR",
                &"u".repeat(201),
                PLATFORM_BPS,
                MIN_BID_LAMPORTS,
                WSOL_DECIMALS,
                seeded(&params(MODE_COOK)).as_ref()
            ),
            Err(ConfigFault::Config)
        );


        let mut p = params(MODE_COOK);
        p.recipients = vec![handle(b"floor_dev", 10_000)];
        assert!(resolve(&p).unwrap().pending);
        p.recipients = vec![];
        assert_eq!(resolve(&p), Err(ConfigFault::Recipients(1)), "no recipient at all");










        let mut p = params(MODE_COOK);
        p.quote_usd_cents = 10_000_000 * 100;
        assert_eq!(resolve(&p), Err(ConfigFault::Config), "backing below min_bid");
    }

///
///
///
///
///
    #[test]
    fn the_team_buy_resolves_inside_its_bounds_and_refuses_by_name_2026_10_04() {
        use floor_types::constants::SUPPLY;
        let five = SUPPLY / 20;
        let with = |m: u8, t: u64, steps: u16, lock: u16, vest: [u16; 4]| CreateLaunchParams {
            team_buy_tokens: t,
            team_steps_bps: steps,
            team_lock_days: lock,
            team_vest_days: vest,
            ..params(m)
        };
        let deployer = |p: &CreateLaunchParams| resolve_team_buy(p, &resolve(p).expect("the preset resolves"));

        assert_eq!(deployer(&params(MODE_COOK)), Ok(None));
        assert_eq!(deployer(&with(MODE_COOK, 0, 8_000, 0, [0; 4])), Err(TeamBuyFault::OutOfRange));

        let d = deployer(&with(MODE_COOK, five, 8_000, 90, [0; 4])).unwrap().unwrap();
        assert_eq!((d.tokens, d.step_tokens, d.wallet_tokens, d.lock_days), (five, five / 10 * 8, five / 10 * 2, 90));
        assert_eq!(d.vest_days, [10, 20, 40, 80]);

        let stray = CreateLaunchParams { team_payees: vec![wallet([7; 32], 10_000)], ..params(MODE_COOK) };
        assert_eq!(deployer(&stray), Err(TeamBuyFault::OutOfRange), "a payee with no team buy");
        let b = deployer(&with(MODE_BLUECHIP, five, 10_000, 365, [0; 4])).unwrap().unwrap();
        assert_eq!((b.step_tokens, b.wallet_tokens, b.vest_days), (five, 0, [30, 60, 90, 120]), "100 % on steps is allowed");
        assert_eq!(deployer(&with(MODE_MOONSHOT, 1, 8_000, 90, [0; 4])).unwrap().unwrap().wallet_tokens, 0, "the wallet rounds DOWN");

        assert_eq!(deployer(&with(MODE_COOK, five + 1, 8_000, 90, [0; 4])), Err(TeamBuyFault::OutOfRange), "over 5 %");


        let ten = SUPPLY / 10;
        let bc = deployer(&with(MODE_BLUECHIP, ten, 8_000, 90, [0; 4])).unwrap().unwrap();
        assert_eq!((bc.tokens, bc.step_tokens, bc.wallet_tokens), (ten, ten / 10 * 8, ten / 10 * 2), "Bluechip's 10 %: 8 % steps, 2 % wallet");
        assert_eq!(deployer(&with(MODE_BLUECHIP, ten + 1, 8_000, 90, [0; 4])), Err(TeamBuyFault::OutOfRange), "Bluechip over 10 %");
        assert_eq!(deployer(&with(MODE_MOONSHOT, five + 1, 8_000, 90, [0; 4])), Err(TeamBuyFault::OutOfRange), "Moonshot over 5 %");
        assert_eq!(deployer(&with(MODE_COOK, five, 7_999, 90, [0; 4])), Err(TeamBuyFault::OutOfRange), "under 80 % on steps");
        assert_eq!(deployer(&with(MODE_COOK, five, 10_001, 90, [0; 4])), Err(TeamBuyFault::OutOfRange), "over 100 % on steps");
        assert_eq!(deployer(&with(MODE_COOK, five, 8_000, 89, [0; 4])), Err(TeamBuyFault::OutOfRange), "a lock under 90 days");
        assert_eq!(deployer(&with(MODE_COOK, five, 8_000, 1_826, [0; 4])), Err(TeamBuyFault::OutOfRange), "a lock past the EVM's 1,825 days");
        assert!(deployer(&with(MODE_COOK, five, 8_000, 1_825, [0; 4])).is_ok(), "a lock of exactly 1,825 days");
        assert_eq!(deployer(&with(MODE_COOK, five, 8_000, 90, [5, 5, 5, 5])), Err(TeamBuyFault::OutOfRange), "a preset's cliffs are its own");


        let c = |vest: [u16; 4]| CreateLaunchParams {
            team_buy_tokens: five,
            team_steps_bps: 8_000,
            team_lock_days: 90,
            team_vest_days: vest,
            ..custom(10, 10, 0, 0)
        };
        let cd = resolve_team_buy(&c([12, 30, 90, 180]), &resolve(&c([12, 30, 90, 180])).unwrap()).unwrap().unwrap();
        assert_eq!((cd.step_tokens, cd.vest_days), (five / 10 * 8, [12, 30, 90, 180]));
        assert_eq!(resolve_team_buy(&c([10, 11, 12, 365]), &resolve(&c([10, 11, 12, 365])).unwrap()).unwrap().unwrap().vest_days, [10, 11, 12, 365], "both ends");
        for bad in [[0; 4], [9, 20, 40, 80], [10, 20, 40, 366], [10, 20, 20, 80], [30, 20, 40, 80]] {
            assert_eq!(resolve_team_buy(&c(bad), &resolve(&c(bad)).unwrap()), Err(TeamBuyFault::OutOfRange), "{bad:?}");
        }

        let mut flat = resolve(&c([12, 30, 90, 180])).unwrap();
        flat.stair_steps = 0;
        assert_eq!(resolve_team_buy(&c([12, 30, 90, 180]), &flat), Err(TeamBuyFault::NeedsStaircase));
        assert_eq!(LaunchError::from(TeamBuyFault::OutOfRange) as u32 + 6000, 6061);
        assert_eq!(LaunchError::from(TeamBuyFault::NeedsStaircase) as u32 + 6000, 6062);




        let p = with(MODE_COOK, five, 10_000, 90, [0; 4]);
        let mut r = resolve(&p).unwrap();
        r.stair_tokens = five;
        assert_eq!(resolve_team_buy(&p, &r), Err(TeamBuyFault::OverCap), "the team would hold half the staircase");
        r.stair_tokens = five * 3;
        assert_eq!(resolve_team_buy(&p, &r), Err(TeamBuyFault::OverCap), "the team would hold just over a quarter");
        r.stair_tokens = five * 3 + five / 10;
        assert!(resolve_team_buy(&p, &r).unwrap().is_some(), "just under a quarter rides");
        assert_eq!(LaunchError::from(TeamBuyFault::OverCap) as u32 + 6000, 6066);

        assert_eq!(team_buy_cost(3, 1u128 << 63), Some(2));
        assert_eq!(team_buy_cost(4, 1u128 << 63), Some(2));
    }

    fn custom(cb: u16, cs: u16, hb: u16, hs: u16) -> CreateLaunchParams {
        CreateLaunchParams {
            graduation_mcap_usd: 50_000,
            creator_buy_bps: cb,
            creator_sell_bps: cs,
            holder_buy_bps: hb,
            holder_sell_bps: hs,
            ..params(MODE_CUSTOM)
        }
    }

///
    fn pending(mut p: CreateLaunchParams) -> CreateLaunchParams {
        p.recipients = vec![handle(b"floor_dev", 10_000)];
        p
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
    fn split_rates_resolve_per_side_and_each_side_is_capped() {

        let r = resolve(&custom(30, 90, 20, 150)).expect("asymmetric Custom rates resolve");
        assert_eq!((r.creator_buy_bps, r.creator_sell_bps, r.holder_buy_bps, r.holder_sell_bps), (30, 90, 20, 150));

        let r = resolve(&custom(0, 100, 200, 0)).expect("0 / 100 and 200 / 0");
        assert_eq!((r.creator_buy_bps, r.creator_sell_bps, r.holder_buy_bps, r.holder_sell_bps), (0, 100, 200, 0));



        let caps = [MAX_CREATOR_BPS, MAX_CREATOR_BPS, MAX_HOLDER_BPS, MAX_HOLDER_BPS];
        for (side, cap) in caps.into_iter().enumerate() {
            let mut v = [10u16, 20, 30, 40];
            v[side] = cap;
            let r = resolve(&custom(v[0], v[1], v[2], v[3])).unwrap_or_else(|e| panic!("side {side} at its cap {cap}: {e:?}"));
            assert_eq!([r.creator_buy_bps, r.creator_sell_bps, r.holder_buy_bps, r.holder_sell_bps], v);
            v[side] = cap + 1;
            assert_eq!(resolve(&custom(v[0], v[1], v[2], v[3])), Err(ConfigFault::Config), "side {side} one past its cap");
        }


        for side in 0..4 {
            let mut p = params(MODE_COOK);
            match side {
                0 => p.creator_buy_bps = 1,
                1 => p.creator_sell_bps = 1,
                2 => p.holder_buy_bps = 1,
                _ => p.holder_sell_bps = 1,
            }
            assert_eq!(resolve(&p), Err(ConfigFault::Config), "a preset with side {side} set");
        }


        for (mode, m) in [(MODE_COOK, Mode::Cook), (MODE_BLUECHIP, Mode::Bluechip), (MODE_MOONSHOT, Mode::Moonshot)] {
            let r = resolve(&params(mode)).unwrap();
            let (c, h) = (preset(m).creator_bps, preset(m).holder_bps);
            assert!(c > 0);
            assert_eq!((r.creator_buy_bps, r.creator_sell_bps, r.holder_buy_bps, r.holder_sell_bps), (c, c, h, h));
        }



        assert!(resolve(&pending(custom(0, 50, 0, 0))).expect("0 on buys, 0.5 % on sells").pending);
        assert!(resolve(&pending(custom(50, 0, 0, 0))).expect("0.5 % on buys, 0 on sells").pending);

        assert_eq!(resolve(&pending(custom(0, 0, 0, 0))), Err(ConfigFault::Recipients(7)), "a handle with nothing to receive");
        assert_eq!(
            resolve(&pending(custom(0, 0, 20, 150))),
            Err(ConfigFault::Recipients(7)),
            "a holder rate is not the recipient's — it cannot stand in for a creator rate"
        );
    }










///
///
    const ON_CURVE: [u8; 32] = [9; 32];
    const CREATOR: [u8; 32] = [3; 32];
    const OFF_CURVE: [u8; 32] = [7; 32];

    #[test]
    fn t_slc_is_wallet_key_is_the_curve() {
        assert!(is_wallet_key(&Pubkey::new_from_array(ON_CURVE)));
        assert!(is_wallet_key(&Pubkey::new_from_array(CREATOR)));
        assert!(!is_wallet_key(&Pubkey::new_from_array(OFF_CURVE)), "[7; 32] is not an ed25519 point");
    }

///
    fn own_fees() -> CreateLaunchParams {
        CreateLaunchParams { recipients: vec![wallet(CREATOR, 10_000)], ..custom(10, 10, 0, 0) }
    }

///
    fn link_bytes(trader: Pubkey, t1: Pubkey, t2: Pubkey, t3: Pubkey) -> Vec<u8> {
        use crate::state::TraderLink;
        let mut d = vec![0u8; TraderLink::LEN];
        d[..8].copy_from_slice(TraderLink::DISCRIMINATOR);
        let l: &mut TraderLink = bytemuck::from_bytes_mut(&mut d[8..TraderLink::LEN]);
        l.trader = trader;
        l.tier1 = t1;
        l.tier2 = t2;
        l.tier3 = t3;
        l.bump = 255;
        d
    }

///
///
///
///
///
///
///
    #[test]
    fn creator_upline_reads_the_creators_own_link_and_every_other_state_is_none() {
        let creator = Pubkey::new_from_array(CREATOR);
        let mut on = (10u8..=255).map(|b| Pubkey::new_from_array([b; 32])).filter(is_wallet_key);
        let (a, b, c) = (on.next().unwrap(), on.next().unwrap(), on.next().unwrap());
        let z = Pubkey::default();
        let none = Upline::default();
        let me = crate::ID;
        let sys = anchor_lang::solana_program::system_program::ID;
        let p = own_fees();
        let up = |p: &CreateLaunchParams, owner: &Pubkey, d: &[u8]| creator_upline(p, &creator, owner, d);


        assert_eq!(up(&p, &me, &link_bytes(creator, a, b, c)), Upline { tier1: a, tier2: b, tier3: c }, "three tiers");
        assert_eq!(up(&p, &me, &link_bytes(creator, a, b, z)), Upline { tier1: a, tier2: b, tier3: z }, "two tiers");
        assert_eq!(up(&p, &me, &link_bytes(creator, a, z, z)), Upline { tier1: a, tier2: z, tier3: z }, "one tier");
        assert!(up(&p, &me, &link_bytes(creator, a, z, z)).is_set());

        assert_eq!(up(&p, &sys, &[]), none, "unlinked");
        assert!(!none.is_set());

        assert_eq!(up(&p, &sys, &link_bytes(creator, a, b, c)), none, "System-owned bytes");
        assert_eq!(up(&p, &Pubkey::new_from_array([0x44; 32]), &link_bytes(creator, a, b, c)), none, "a foreign owner");

        let mut wrong = link_bytes(creator, a, b, c);
        wrong[0] ^= 1;
        assert_eq!(up(&p, &me, &wrong), none, "wrong discriminator");
        let mut long = link_bytes(creator, a, b, c);
        long.push(0);
        assert_eq!(up(&p, &me, &long), none, "too long");
        assert_eq!(up(&p, &me, &link_bytes(creator, a, b, c)[..100]), none, "too short");

        assert_eq!(up(&p, &me, &link_bytes(a, b, c, z)), none, "trader != creator");

        assert_eq!(up(&p, &me, &link_bytes(creator, creator, z, z)), none, "tier 1 = the creator");
        assert_eq!(up(&p, &me, &link_bytes(creator, a, creator, z)), none, "tier 2 = the creator");
        assert_eq!(up(&p, &me, &link_bytes(creator, a, b, creator)), none, "tier 3 = the creator");

        assert_eq!(up(&p, &me, &link_bytes(creator, z, z, z)), none, "no tier 1");
        assert_eq!(up(&p, &me, &link_bytes(creator, z, b, z)), none, "a gap at tier 1");
        assert_eq!(up(&p, &me, &link_bytes(creator, a, z, c)), none, "a gap at tier 2");
        assert_eq!(up(&p, &me, &link_bytes(creator, a, a, z)), none, "a repeat");



        let link = link_bytes(creator, a, b, c);
        let full = Upline { tier1: a, tier2: b, tier3: c };
        let mut q = own_fees();
        q.recipients = vec![wallet([0x11; 32], 10_000)];
        assert_eq!(up(&q, &me, &link), full, "A16: fees routed to a third party, still referred");
        let mut q = own_fees();
        q.recipients = vec![handle(b"floor_dev", 10_000)];
        assert_eq!(up(&q, &me, &link), full, "A16: a pending-handle launch, still referred");
        let mut q = own_fees();
        q.recipients = vec![crate::state::SlotArg { bps: 10_000, platform: 1, key: CREATOR }];
        assert_eq!(up(&q, &me, &link), full, "A16: a pending slot whose bytes are the signer's, still referred");
        let mut q = own_fees();
        q.recipients = vec![wallet([0x11; 32], 5_000), handle(b"floor_dev", 3_000), wallet(CREATOR, 2_000)];
        assert_eq!(up(&q, &me, &link), Upline { tier1: a, tier2: b, tier3: c }, "the deployer holds ONE of three slots");
        for (k, why) in [(a, "tier 1"), (b, "tier 2"), (c, "tier 3")] {
            let mut q = own_fees();
            q.recipients = vec![wallet(CREATOR, 6_000), wallet(k.to_bytes(), 4_000)];
            assert_eq!(up(&q, &me, &link), none, "{why} holds a creator slot");
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
    fn t_slc_the_appended_errors_are_6037_to_6040() {
        assert_eq!(LaunchError::GovernanceIsAttestor as u32 + 6000, 6035, "the tail both changes append to");
        assert_eq!(LaunchError::AttestorIsTreasuryGovernance as u32 + 6000, 6036, "main's, which landed first");
        assert_eq!(LaunchError::ReferrerNotAllowed as u32 + 6000, 6037);
        assert_eq!(LaunchError::ReferrerOffCurve as u32 + 6000, 6038);
        assert_eq!(LaunchError::HookNotArmed as u32 + 6000, 6039);
        assert_eq!(LaunchError::HolderRecordMismatch as u32 + 6000, 6040);
    }

///
///
///
    #[test]
    fn t_slc_b2_hooked_iff_a_holder_rate() {
        for (hb, hs, want) in [(0u16, 0u16, false), (1, 0, true), (0, 1, true), (200, 150, true)] {
            let p = custom(10, 10, hb, hs);
            assert_eq!(hooked_for(&p), want, "holder {hb}/{hs}");
            assert_eq!(resolve(&p).unwrap().hooked(), want, "resolved {hb}/{hs}");
        }



        for m in [MODE_COOK, MODE_BLUECHIP, MODE_MOONSHOT] {
            assert!(!hooked_for(&params(m)), "{m}: a preset's params carry no holder rate");
            assert!(!resolve(&params(m)).unwrap().hooked(), "{m}: no preset is hooked");
        }
        let r = resolve(&params(MODE_BLUECHIP)).unwrap();
        assert_eq!((r.holder_buy_bps, r.holder_sell_bps), (0, 0), "Bluechip carries no holder rate");
    }

///
///
///
///
///
    #[test]
    fn the_split_is_the_presets_or_the_custom_pick_inside_the_ten_percent_rule() {
        let tenth = floor_types::constants::SUPPLY / 10;
        for m in [MODE_COOK, MODE_BLUECHIP, MODE_MOONSHOT] {
            let r = resolve(&params(m)).unwrap();
            let row = preset(Mode::try_from(m).unwrap());
            let want = split_tokens(row.sale_bps, row.pol_bps).unwrap();
            assert_eq!((r.sale_tokens, r.pol_tokens, r.stair_tokens), want, "{m}: its own row");

            assert_eq!((r.holder_buy_bps, r.holder_sell_bps), (row.holder_bps, row.holder_bps), "{m}: its row's holder rate");
            assert!(!r.hooked(), "{m}: no preset is hooked");
            let p = CreateLaunchParams { sale_bps: 1_000, pol_bps: 1_000, ..params(m) };
            assert_eq!(resolve(&p), Err(ConfigFault::Config), "{m}: a preset sends 0 / 0");
        }
        let custom = |s: u16, p: u16| CreateLaunchParams { graduation_mcap_usd: 540_000, sale_bps: s, pol_bps: p, ..params(MODE_CUSTOM) };
        let std = resolve(&custom(0, 0)).unwrap();
        assert_eq!((std.sale_tokens, std.pol_tokens, std.stair_tokens), (SALE_TOKENS, POL_TOKENS, LADDER_TOKENS));
        let pick = resolve(&custom(4_000, 2_000)).unwrap();
        assert_eq!((pick.sale_tokens, pick.pol_tokens, pick.stair_tokens), (4 * tenth, 2 * tenth, 4 * tenth));
        let edge = resolve(&custom(8_000, 1_000)).unwrap();
        assert_eq!((edge.sale_tokens, edge.pol_tokens, edge.stair_tokens), (8 * tenth, tenth, tenth), "every zone at its 10 % floor");
        for (s, p) in [(500, 500), (999, 1_000), (1_000, 999), (8_001, 1_000), (4_500, 4_600), (3_000, 0), (0, 1_000)] {
            assert_eq!(resolve(&custom(s, p)), Err(ConfigFault::Config), "{s} / {p}");
        }

        let r10 = resolve(&custom(1_000, 1_000)).unwrap();
        let r80 = resolve(&custom(8_000, 1_000)).unwrap();
        let b10 = backing_at_sellout(r10.price_x64, PLATFORM_BPS, r10.sale_tokens).unwrap();
        let b80 = backing_at_sellout(r80.price_x64, PLATFORM_BPS, r80.sale_tokens).unwrap();
        assert!(b80 > 7 * b10, "80 % raises 8x what 10 % does at one price: {b10} vs {b80}");
    }




///
///
///
    #[test]
    fn a_staircase_that_does_not_fit_is_refused_by_name() {
        let custom = CreateLaunchParams { graduation_mcap_usd: 50_000, ..params(MODE_CUSTOM) };
        for p in [params(MODE_COOK), params(MODE_BLUECHIP), params(MODE_MOONSHOT), custom] {
            let m = p.mode;
            let r = resolve(&p).unwrap();
            let edge = floor_math::floor_to_spacing(r.open_tick, floor_types::constants::FLOOR_TICK_SPACING as i32).unwrap();
            assert!(floor_math::stair::stair_fits(edge, r.stair_steps, floor_types::constants::FLOOR_TICK_SPACING as i32).unwrap(), "{m}");
        }
        assert_eq!(LaunchError::from(ConfigFault::StairDoesNotFit) as u32 + 6000, 6059);
        assert_eq!(LaunchError::StairDropped as u32 + 6000, 6058);
    }





///
///
///
///
///
///
    #[test]
    fn mr_s_the_recipient_list_is_refused_by_its_check_code() {
        let with = |r: Vec<crate::state::SlotArg>| CreateLaunchParams { recipients: r, ..custom(10, 10, 0, 0) };

        assert!(!resolve(&with(vec![wallet([1; 32], 10_000)])).unwrap().pending);
        let five = vec![
            wallet([1; 32], 4_000),
            wallet([2; 32], 3_000),
            handle(b"floor_dev", 1_500),
            crate::state::SlotArg { bps: 1_000, platform: 2, key: pending_recipient_id(2, b"floor-gh") },
            wallet([3; 32], 500),
        ];
        assert!(resolve(&with(five.clone())).unwrap().pending, "a pending slot among five");

        assert_eq!(resolve(&with(vec![])), Err(ConfigFault::Recipients(1)));
        let mut six = five.clone();
        six[0].bps -= 1;
        six.push(wallet([4; 32], 1));
        assert_eq!(resolve(&with(six)), Err(ConfigFault::Recipients(1)));

        assert_eq!(resolve(&with(vec![wallet([1; 32], 10_000), wallet([2; 32], 0)])), Err(ConfigFault::Recipients(2)));

        assert_eq!(resolve(&with(vec![wallet([1; 32], 5_000), wallet([2; 32], 4_999)])), Err(ConfigFault::Recipients(3)));

        assert_eq!(
            resolve(&with(vec![crate::state::SlotArg { bps: 10_000, platform: 5, key: [1; 32] }])),
            Err(ConfigFault::Recipients(4))
        );
        assert_eq!(resolve(&with(vec![wallet([0; 32], 10_000)])), Err(ConfigFault::Recipients(4)));

        assert_eq!(resolve(&with(vec![wallet([1; 32], 5_000), wallet([1; 32], 5_000)])), Err(ConfigFault::Recipients(5)));
        assert_eq!(resolve(&with(vec![handle(b"x", 5_000), handle(b"x", 5_000)])), Err(ConfigFault::Recipients(6)));

        let p = CreateLaunchParams { recipients: five, ..custom(0, 0, 20, 20) };
        assert_eq!(resolve(&p), Err(ConfigFault::Recipients(floor_math::slots::CHECK_PENDING_WITHOUT_RATE)));

        assert!(resolve(&CreateLaunchParams { recipients: vec![wallet([1; 32], 10_000)], ..custom(0, 0, 0, 0) }).is_ok());

        assert_eq!(LaunchError::from(ConfigFault::Recipients(3)) as u32 + 6000, 6097);
        assert_eq!(LaunchError::BadRecipients as u32 + 6000, 6097);
        assert_eq!(LaunchError::SaleReferralAlreadyOpen as u32 + 6000, 6096, "the variant before it (TRADER REFERRALS, v5 preview)");
    }

///
///
///
///
    #[test]
    fn mr_s_the_team_payee_list_is_the_deployer_by_default_and_checked_otherwise() {
        use floor_types::constants::SUPPLY;
        let creator = Pubkey::new_from_array(CREATOR);
        let team = |payees: Vec<crate::state::SlotArg>| CreateLaunchParams {
            team_buy_tokens: SUPPLY / 20,
            team_steps_bps: 8_000,
            team_lock_days: 90,
            team_payees: payees,
            ..params(MODE_COOK)
        };

        let d = resolve_team_payees(&team(vec![]), &creator).unwrap();
        assert_eq!(d, vec![floor_types::slots::SlotArg { bps: 10_000, platform: 0, key: CREATOR }]);

        let two = vec![wallet([0x21; 32], 8_000), crate::state::SlotArg { bps: 2_000, platform: 3, key: pending_recipient_id(3, b"dc") }];
        let got = resolve_team_payees(&team(two.clone()), &creator).unwrap();
        assert_eq!(got, two.iter().map(floor_types::slots::SlotArg::from).collect::<Vec<_>>());

        assert_eq!(resolve_team_payees(&team(vec![wallet([1; 32], 9_999)]), &creator), Err(TeamBuyFault::Payees(3)));
        assert_eq!(
            resolve_team_payees(&team(vec![wallet([1; 32], 5_000), wallet([1; 32], 5_000)]), &creator),
            Err(TeamBuyFault::Payees(5))
        );
        let six = (1..=6u8).map(|k| wallet([k; 32], if k == 1 { 5_000 } else { 1_000 })).collect();
        assert_eq!(resolve_team_payees(&team(six), &creator), Err(TeamBuyFault::Payees(1)));
        assert_eq!(LaunchError::from(TeamBuyFault::Payees(1)) as u32, LaunchError::TeamBuyOutOfRange as u32);

        let stray = CreateLaunchParams { team_payees: vec![wallet([1; 32], 10_000)], ..params(MODE_COOK) };
        assert_eq!(resolve_team_buy(&stray, &resolve(&stray).unwrap()), Err(TeamBuyFault::OutOfRange));

        let p = team(two);
        assert!(resolve_team_buy(&p, &resolve(&p).unwrap()).unwrap().is_some());
    }

///
///
///
///
    #[test]
    fn mr_s_the_register_pool_slots_are_the_creator_list_then_the_team_list() {
        let p = CreateLaunchParams {
            recipients: vec![wallet([1; 32], 7_000), handle(b"floor_dev", 3_000)],
            ..custom(10, 10, 0, 0)
        };
        let none = register_slots(&p, Vec::new());
        let b = borsh::to_vec(&none).unwrap();
        assert_eq!(b.len(), 4 + 2 * 35 + 4);
        assert_eq!(&b[..4], &2u32.to_le_bytes());
        assert_eq!(&b[4..6], &7_000u16.to_le_bytes());
        assert_eq!(b[6], 0);
        assert_eq!(&b[7..39], &[1u8; 32]);
        assert_eq!(&b[39..41], &3_000u16.to_le_bytes());
        assert_eq!(b[41], 1);
        assert_eq!(&b[42..74], &pending_recipient_id(1, b"floor_dev"));
        assert_eq!(&b[74..], &0u32.to_le_bytes(), "no team buy: an empty team list");
        let team = resolve_team_payees(&CreateLaunchParams { team_payees: vec![], ..p.clone() }, &Pubkey::new_from_array(CREATOR)).unwrap();
        let with = register_slots(&p, team);
        let b = borsh::to_vec(&with).unwrap();
        assert_eq!(b.len(), 4 + 2 * 35 + 4 + 35);
        assert_eq!(&b[74..78], &1u32.to_le_bytes());
        assert_eq!(&b[78..80], &10_000u16.to_le_bytes(), "the deployer at 10,000");
        assert_eq!(&b[81..113], &CREATOR, "keyed by the SIGNER");

        let five = |base: u8| (0..5u8).map(|i| wallet([base + i; 32], 2_000)).collect::<Vec<_>>();
        let widest = register_slots(
            &CreateLaunchParams { recipients: five(10), ..custom(10, 10, 0, 0) },
            five(20).iter().map(floor_types::slots::SlotArg::from).collect(),
        );
        assert_eq!(borsh::to_vec(&widest).unwrap().len(), 358);
    }

///
///
///
    #[test]
    fn mr_s_the_wallet_part_edges_before_after_and_one_slot_is_today() {
        let days = [10u16, 20, 40, 80];
        let anchor = 1_700_000_000i64;
        let at = |k: usize| anchor + days[k] as i64 * 86_400;
        assert_eq!(team_vested_slot_part(1_000, &days, anchor, at(0) - 1, 10_000, 0), (0, 0), "before the first cliff");
        assert_eq!(team_vested_slot_part(1_000, &days, anchor, at(3) + 9, 10_000, 0b1111), (0, 0b1111), "all paid");
        assert_eq!(team_vested_slot_part(0, &days, anchor, at(1), 5_000, 0), (0, 0b11), "a sold-back team buy pays 0");

        let total = 1_000_000_003u64;
        let mut mask = 0u8;
        let mut paid = 0u64;
        for k in 0..4 {
            let (a, m) = team_vested_slot_part(total, &days, anchor, at(k), 10_000, mask);
            paid += a;
            mask = m;
            assert_eq!(paid, floor_math::owners::team_vested(total, &days, anchor, at(k)), "cliff {k}");
        }
        assert_eq!(paid, total);

        let by_slot: u32 = 0x0000_FFFF;
        let (_, m) = team_vested_slot_part(total, &days, anchor, at(0), 2_000, slot_cliff_mask(by_slot, 4));
        let next = with_slot_cliff_mask(by_slot, 4, m);
        assert_eq!(next & 0xFFFF, 0xFFFF);
        assert_eq!(next >> 16, 1);
    }

///
///
///
///
///
///
///
///
    #[test]
    fn mr_s_the_wallet_part_is_paid_per_slot_per_cliff_and_sums_to_earned_at_the_last() {
        let total: u64 = 1_000_000_003;
        let days = [10u16, 20, 40, 80];
        let anchor = 1_700_000_000i64;
        let at = |k: usize| anchor + days[k] as i64 * 86_400;
        let bps = [3_333u16, 3_333, 1_667, 1_000, 667];
        assert_eq!(bps.iter().map(|&b| b as u32).sum::<u32>(), 10_000);

        let schedules: [&[i64]; 4] = [
            &[at(0), at(1), at(2), at(3)],
            &[at(1), at(3)],
            &[at(3) + 1],
            &[at(0) - 1, at(2), at(3) + 999],
        ];
        for (n, sched) in schedules.iter().enumerate() {
            let mut by_slot: u32 = 0;
            let mut paid = [0u64; 5];
            for &now in sched.iter() {
                for (i, &b) in bps.iter().enumerate() {
                    let shift = 4 * i as u32;
                    let mask = ((by_slot >> shift) & 0xF) as u8;
                    let (amount, next) = team_vested_slot_part(total, &days, anchor, now, b, mask);
                    if next == mask {
                        assert_eq!(amount, 0, "schedule {n}: nothing new, nothing paid");
                    }
                    paid[i] += amount;
                    by_slot = (by_slot & !(0xF << shift)) | ((next as u32) << shift);
                }

                for (i, &b) in bps.iter().enumerate() {
                    let mask = ((by_slot >> (4 * i as u32)) & 0xF) as u8;
                    assert_eq!(team_vested_slot_part(total, &days, anchor, now, b, mask), (0, mask), "schedule {n}: a repeat");
                }
            }
            let v_last = floor_math::owners::team_vested(total, &days, anchor, at(3));
            assert_eq!(v_last, total);
            for (i, &b) in bps.iter().enumerate() {
                assert_eq!(paid[i], floor_math::slot_earned(v_last, b), "schedule {n}, slot {i}");
            }
            let sum: u64 = paid.iter().sum();
            assert!(sum <= total && total - sum < bps.len() as u64, "schedule {n}: dust {} under the slot count", total - sum);
            assert_eq!(by_slot, 0x000F_FFFF, "every slot's four cliffs booked, nothing past slot 4");
        }

        let mut mask = 0u8;
        let mut got = 0;
        for k in 0..4 {
            let (a, m) = team_vested_slot_part(total, &days, anchor, at(k), 10_000, mask);
            assert_eq!(a, floor_math::owners::team_vested(total, &days, anchor, at(k)) - got);
            got += a;
            mask = m;
        }
        assert_eq!(got, total);
    }

///
///
///
///
    #[test]
    fn mr_s_the_widest_create_launch_data_is_measured() {
        use anchor_lang::InstructionData;
        use floor_types::constants::{MAX_NAME_LEN, MAX_SYMBOL_LEN};
        let slot = |k: u8| crate::state::SlotArg { bps: 2_000, platform: 1, key: [k; 32] };
        let params = CreateLaunchParams {
            mode: MODE_CUSTOM,
            graduation_mcap_usd: u64::MAX,
            recipients: (1..=5).map(slot).collect(),
            quote_usd_cents: u64::MAX,
            team_buy_tokens: u64::MAX,
            team_payees: (6..=10).map(slot).collect(),
            ..Default::default()
        };
        let ix = crate::instruction::CreateLaunch {
            params,
            nonce: u64::MAX,
            name: "N".repeat(MAX_NAME_LEN),
            symbol: "S".repeat(MAX_SYMBOL_LEN),
        };
        let data = ix.data();
        println!("create_launch data at 5 + 5 slots, name {MAX_NAME_LEN} B, symbol {MAX_SYMBOL_LEN} B: {} bytes", data.len());



        assert_eq!(data.len(), 8 + 413 + 8 + (4 + MAX_NAME_LEN) + (4 + MAX_SYMBOL_LEN));
        assert_eq!(data.len(), 517);
    }

///
///
///
///
///
///
    #[test]
    fn mr_s_each_team_slot_owns_its_own_four_cliff_bits() {
        for slot in 0u8..5 {
            for mask in 0u8..16 {

                for base in [0u32, 0x000F_FFFF, 0xA5A5_A5A5] {
                    let w = with_slot_cliff_mask(base, slot, mask);
                    assert_eq!(slot_cliff_mask(w, slot), mask, "slot {slot} mask {mask:#x} base {base:#x}");
                    for other in 0u8..5 {
                        if other != slot {
                            assert_eq!(slot_cliff_mask(w, other), slot_cliff_mask(base, other), "slot {slot} moved {other}");
                        }
                    }
                    assert_eq!(w & !(0xF << (4 * slot as u32)), base & !(0xF << (4 * slot as u32)));
                }
            }

            assert_eq!(slot_cliff_mask(with_slot_cliff_mask(0x000F_FFFF, slot, 0b0001), slot), 0b0001);

            let w = with_slot_cliff_mask(0, slot, 0xFF);
            assert_eq!(w, 0xF << (4 * slot as u32));
        }

        assert_eq!(with_slot_cliff_mask(0, 0, 0b1011), 0b1011);
        assert_eq!(with_slot_cliff_mask(0, 4, 0b0001), 1 << 16);
    }

///
///
///
///
///
    #[test]
    fn mr_s_a_dust_slot_books_zero_cliffs_and_still_gets_its_whole_share() {
        let days = [10u16, 20, 40, 80];
        let anchor = 1_700_000_000i64;
        let at = |k: usize| anchor + days[k] as i64 * 86_400;
        let total = 39_999u64;
        let mut mask = 0u8;
        let mut paid = 0u64;
        let mut amounts = [0u64; 4];
        for k in 0..4 {
            let (a, m) = team_vested_slot_part(total, &days, anchor, at(k), 1, mask);
            assert_ne!(m, mask, "cliff {k} must be booked even when it pays 0");
            amounts[k] = a;
            paid += a;
            mask = m;
        }
        assert_eq!(mask, 0xF);
        assert_eq!(amounts[0], 0, "a quarter of 39,999 at 1 bps is under one unit");
        assert_eq!(paid, floor_math::slots::slot_earned(total, 1));
        assert_eq!(paid, 3, "floor(39,999 / 10,000)");

        assert_eq!(team_vested_slot_part(total, &days, anchor, at(3) + 86_400, 1, mask), (0, mask));
    }



///
///
    fn registry_data(rows: &[(u8, floor_math::presets::PresetRow, bool)]) -> Vec<u8> {
        use floor_types::layout::Layout;
        let mut r: crate::state::PresetRegistry = bytemuck::Zeroable::zeroed();
        r.version = 1;
        for &(id, row, enabled) in rows {
            r.rows[id as usize] = crate::state::PresetRowZc::from_numbers(&row, enabled);
            r.count = r.count.max(id + 1);
        }
        let mut data = floor_types::layout::PresetRegistry::DISCRIMINATOR.to_vec();
        data.extend_from_slice(bytemuck::bytes_of(&r));
        data
    }

///
    fn fresh() -> Vec<u8> {
        registry_data(&(1..=3).map(|id| (id, floor_math::presets::seed_row(id).unwrap(), true)).collect::<Vec<_>>())
    }

///
    fn monkey() -> floor_math::presets::PresetRow {
        floor_math::presets::PresetRow {
            graduation_mcap_usd: 75_000,
            creator_bps: 60,
            skim_buy_far_bps: 40,
            skim_sell_far_bps: 70,
            holder_bps: 0,
            sale_bps: 2_500,
            pol_bps: 1_200,
            skim_sell_at_floor_multiple_bps: 20_000,
            stair_steps: 12,
            stair_growth_bps: 10_200,
            team_buy_max_bps: 700,
            team_vest_days: [15, 30, 60, 90],
        }
    }

///
    fn resolve_in(data: &[u8], p: &CreateLaunchParams) -> core::result::Result<ResolvedConfig, ConfigFault> {
        let row = preset_from_registry_data(data, p.mode)?;
        resolve_config(p, "Floor", "FLR", "", PLATFORM_BPS, MIN_BID_LAMPORTS, WSOL_DECIMALS, row.as_ref())
    }

///
///
    #[test]
    fn the_registry_account_is_floor_types_bytes() {
        use anchor_lang::Discriminator;
        assert_eq!(crate::state::PresetRegistry::DISCRIMINATOR, floor_types::disc::account::PRESET_REGISTRY);
        assert_eq!(crate::state::PresetRegistry::LEN, 1_616);
        assert_eq!(core::mem::size_of::<crate::state::PresetRowZc>(), 48);
        assert_eq!(
            crate::constants::PRESET_REGISTRY_KEY,
            Pubkey::find_program_address(&[b"preset_registry"], &crate::ID).0,
            "the compile-time address is the seeds' PDA"
        );

        use floor_types::constants::{TEAM_VEST_DAYS_BLUECHIP, TEAM_VEST_DAYS_COOK_MOONSHOT};
        assert_eq!(floor_math::presets::seed_row(MODE_MOONSHOT).unwrap().team_vest_days, TEAM_VEST_DAYS_COOK_MOONSHOT);
        assert_eq!(floor_math::presets::seed_row(MODE_COOK).unwrap().team_vest_days, TEAM_VEST_DAYS_COOK_MOONSHOT);
        assert_eq!(floor_math::presets::seed_row(MODE_BLUECHIP).unwrap().team_vest_days, TEAM_VEST_DAYS_BLUECHIP);

        assert_eq!(LaunchError::from(ConfigFault::PresetUnknown) as u32 + 6000, 6088);
        assert_eq!(LaunchError::from(ConfigFault::PresetDisabled) as u32 + 6000, 6089);
        assert_eq!(LaunchError::PresetTableFull as u32 + 6000, 6090);
        assert_eq!(LaunchError::PresetInvalid as u32 + 6000, 6091);

        let a: crate::state::PresetRowArgs = monkey().into();
        let mut w = Vec::new();
        anchor_lang::AnchorSerialize::serialize(&a, &mut w).unwrap();
        assert_eq!(w.len(), 37);
        assert_eq!(&w[..8], &75_000u64.to_le_bytes());
        assert_eq!((&w[8..10], &w[18..20]), (&60u16.to_le_bytes()[..], &1_200u16.to_le_bytes()[..]), "creator, then pol @18");
        assert_eq!(&w[20..24], &20_000u32.to_le_bytes(), "the multiple @20");
        assert_eq!((w[24], &w[25..27], &w[27..29]), (12, &10_200u16.to_le_bytes()[..], &700u16.to_le_bytes()[..]));
        assert_eq!(&w[29..31], &15u16.to_le_bytes(), "the first cliff @29");
    }

///
///
    #[test]
    fn the_seeded_ids_resolve_exactly_as_the_old_presets() {
        use floor_math::modes::{preset, Mode};
        let data = fresh();
        for (id, mode) in [(MODE_MOONSHOT, Mode::Moonshot), (MODE_COOK, Mode::Cook), (MODE_BLUECHIP, Mode::Bluechip)] {
            let r = resolve_in(&data, &params(id)).unwrap();
            let p = preset(mode);
            let price = tranche_price_x64(p.graduation_mcap_usd, SOL_150, WSOL_DECIMALS).unwrap();
            let (sale, pol, stair) = split_tokens(p.sale_bps, p.pol_bps).unwrap();
            let want = ResolvedConfig {
                mode: id,
                mcap_usd: p.graduation_mcap_usd,
                skim_buy_far_bps: p.skim_buy_far_bps,
                skim_sell_far_bps: p.skim_sell_far_bps,
                creator_buy_bps: p.creator_bps,
                creator_sell_bps: p.creator_bps,
                holder_buy_bps: p.holder_bps,
                holder_sell_bps: p.holder_bps,
                price_x64: price,
                open_tick: open_tick(price).unwrap(),
                pending: false,
                sale_tokens: sale,
                pol_tokens: pol,
                stair_tokens: stair,
                skim_sell_at_floor_multiple_bps: floor_math::modes::skim_sell_at_floor_multiple_for(mode),
                stair_steps: p.stair_steps,
                stair_growth_bps: p.stair_growth_bps,
                team_buy_max_bps: floor_types::constants::team_buy_max_bps(id),
                team_vest_days: Some(if mode == Mode::Bluechip {
                    floor_types::constants::TEAM_VEST_DAYS_BLUECHIP
                } else {
                    floor_types::constants::TEAM_VEST_DAYS_COOK_MOONSHOT
                }),
            };
            assert_eq!(r, want, "{mode:?} resolves through the registry exactly as before it");
        }

        let custom = CreateLaunchParams { graduation_mcap_usd: 50_000, ..params(MODE_CUSTOM) };
        assert_eq!(resolve_in(&[], &custom), resolve_in(&data, &custom));
        let c = resolve_in(&[], &custom).unwrap();
        assert_eq!((c.team_buy_max_bps, c.team_vest_days), (floor_types::constants::TEAM_BUY_MAX_BPS, None));
    }

///
///
    #[test]
    fn a_new_row_resolves_to_exactly_its_numbers() {
        let m = monkey();
        assert_eq!(floor_math::presets::validate_preset_row(&m), Ok(()));
        let mut rows: Vec<_> = (1..=3).map(|id| (id, floor_math::presets::seed_row(id).unwrap(), true)).collect();
        rows.push((4, m, true));
        let data = registry_data(&rows);
        let r = resolve_in(&data, &params(4)).expect("id 4 is registered and enabled");
        let price = tranche_price_x64(75_000, SOL_150, WSOL_DECIMALS).unwrap();
        let (sale, pol, stair) = split_tokens(2_500, 1_200).unwrap();
        assert_eq!(
            r,
            ResolvedConfig {
                mode: 4,
                mcap_usd: 75_000,
                skim_buy_far_bps: 40,
                skim_sell_far_bps: 70,
                creator_buy_bps: 60,
                creator_sell_bps: 60,
                holder_buy_bps: 0,
                holder_sell_bps: 0,
                price_x64: price,
                open_tick: open_tick(price).unwrap(),
                pending: false,
                sale_tokens: sale,
                pol_tokens: pol,
                stair_tokens: stair,
                skim_sell_at_floor_multiple_bps: 20_000,
                stair_steps: 12,
                stair_growth_bps: 10_200,
                team_buy_max_bps: 700,
                team_vest_days: Some([15, 30, 60, 90]),
            }
        );

        assert!(!r.hooked(), "a preset row carries no holder rate, so the launch is not hooked (B2)");

        assert_eq!(resolve_in(&data, &CreateLaunchParams { creator_buy_bps: 1, ..params(4) }), Err(ConfigFault::Config));

        use floor_types::constants::{BPS, SUPPLY};
        let team = |tokens: u64| CreateLaunchParams { team_buy_tokens: tokens, team_steps_bps: 8_000, team_lock_days: 90, ..params(4) };
        let at = SUPPLY / BPS * 700;
        let t = resolve_team_buy(&team(at), &r).unwrap().unwrap();
        assert_eq!((t.tokens, t.vest_days), (at, [15, 30, 60, 90]));
        assert_eq!(resolve_team_buy(&team(at + 1), &r), Err(TeamBuyFault::OutOfRange), "one past the row's ceiling");
        let own_days = CreateLaunchParams { team_vest_days: [15, 30, 60, 90], ..team(at) };
        assert_eq!(resolve_team_buy(&own_days, &r), Err(TeamBuyFault::OutOfRange), "a preset sends no cliffs of its own");
    }

///
///
    #[test]
    fn a_disabled_preset_is_refused_by_name() {
        let mut rows: Vec<_> = (1..=3).map(|id| (id, floor_math::presets::seed_row(id).unwrap(), true)).collect();
        rows.push((4, monkey(), false));
        rows[1].2 = false;
        let off = registry_data(&rows);
        assert_eq!(resolve_in(&off, &params(4)), Err(ConfigFault::PresetDisabled));
        assert_eq!(resolve_in(&off, &params(MODE_COOK)), Err(ConfigFault::PresetDisabled), "a seeded row disables the same way");
        assert_eq!(LaunchError::from(ConfigFault::PresetDisabled) as u32, LaunchError::PresetDisabled as u32);
        assert!(resolve_in(&off, &params(MODE_MOONSHOT)).is_ok(), "the others are untouched");
        rows[1].2 = true;
        rows[3].2 = true;
        let on = registry_data(&rows);
        assert_eq!(resolve_in(&on, &params(MODE_COOK)), resolve_in(&fresh(), &params(MODE_COOK)), "re-enabled: the same numbers");
        assert!(resolve_in(&on, &params(4)).is_ok());
    }

///
///
    #[test]
    fn an_unknown_preset_is_refused_by_name() {
        let data = fresh();
        for id in [4u8, 5, 31, 32, 200, 255] {
            assert_eq!(resolve_in(&data, &params(id)), Err(ConfigFault::PresetUnknown), "{id}");
        }
        assert_eq!(resolve_in(&[], &params(MODE_COOK)), Err(ConfigFault::PresetUnknown), "no registry");
        assert_eq!(resolve_in(&data[..data.len() - 1], &params(MODE_COOK)), Err(ConfigFault::PresetUnknown), "short");
        let mut foreign = data.clone();
        foreign[0] ^= 1;
        assert_eq!(resolve_in(&foreign, &params(MODE_COOK)), Err(ConfigFault::PresetUnknown), "not a PresetRegistry");

        assert_eq!(
            resolve_config(&params(MODE_COOK), "Floor", "FLR", "", PLATFORM_BPS, MIN_BID_LAMPORTS, WSOL_DECIMALS, None),
            Err(ConfigFault::PresetUnknown)
        );
        assert_eq!(LaunchError::from(ConfigFault::PresetUnknown) as u32, LaunchError::PresetUnknown as u32);
    }

///
///
///
    #[test]
    fn adding_or_disabling_a_preset_moves_no_resolved_launch() {
        let before = resolve_in(&fresh(), &params(MODE_COOK)).unwrap();
        let mut rows: Vec<_> = (1..=3).map(|id| (id, floor_math::presets::seed_row(id).unwrap(), true)).collect();
        rows.push((4, monkey(), true));
        assert_eq!(resolve_in(&registry_data(&rows), &params(MODE_COOK)).unwrap(), before, "adding id 4 moves no other id");
        rows[1].2 = false;
        assert_eq!(resolve_in(&registry_data(&rows), &params(MODE_COOK)), Err(ConfigFault::PresetDisabled));
        assert_eq!(before.mcap_usd, floor_math::modes::COOK_GRADUATION_MCAP_USD, "the copy taken before is unchanged");
    }





    fn seeded_registry() -> crate::state::PresetRegistry {
        let mut r: crate::state::PresetRegistry = bytemuck::Zeroable::zeroed();
        r.seed(Pubkey::new_unique(), 254);
        r
    }

///
///
///
    #[test]
    fn the_seeded_registry_is_todays_presets_byte_for_byte() {
        use floor_types::constants as c;
        let r = seeded_registry();
        assert_eq!((r.version, r.bump, r.count, r.pending_authority), (1, 254, 4, Pubkey::default()));
        for (id, mode, team_max, days) in [
            (1u8, floor_math::modes::Mode::Moonshot, c::TEAM_BUY_MAX_BPS, c::TEAM_VEST_DAYS_COOK_MOONSHOT),
            (2, floor_math::modes::Mode::Cook, c::TEAM_BUY_MAX_BPS, c::TEAM_VEST_DAYS_COOK_MOONSHOT),
            (3, floor_math::modes::Mode::Bluechip, c::BLUECHIP_TEAM_BUY_MAX_BPS, c::TEAM_VEST_DAYS_BLUECHIP),
        ] {
            let p = floor_math::modes::preset(mode);
            let want = crate::state::PresetRowZc {
                graduation_mcap_usd: p.graduation_mcap_usd,
                skim_sell_at_floor_multiple_bps: p.skim_sell_at_floor_multiple_bps,
                creator_bps: p.creator_bps,
                skim_buy_far_bps: p.skim_buy_far_bps,
                skim_sell_far_bps: p.skim_sell_far_bps,
                holder_bps: p.holder_bps,
                sale_bps: p.sale_bps,
                pol_bps: p.pol_bps,
                stair_growth_bps: p.stair_growth_bps,
                team_buy_max_bps: team_max,
                team_vest_days: days,
                stair_steps: p.stair_steps,
                enabled: 1,
                pad: [0; 10],
            };
            assert_eq!(bytemuck::bytes_of(&r.rows[id as usize]), bytemuck::bytes_of(&want), "{mode:?}");
        }
        for id in core::iter::once(0usize).chain(4..32) {
            assert!(bytemuck::bytes_of(&r.rows[id]).iter().all(|b| *b == 0), "row {id} is zero");
        }
    }

    #[test]
    fn the_registry_appends_from_four_enabled_and_a_refusal_takes_no_id() {
        use crate::state::PresetAppendFault;
        let mut r = seeded_registry();
        assert_eq!(r.count, 4, "count 4 after init");
        let mut bad = monkey();
        bad.creator_bps = 501;
        assert_eq!(r.append(&bad), Err(PresetAppendFault::Invalid(floor_math::presets::PresetField::CreatorBps)));
        assert_eq!(r.count, 4, "a refused add takes no id");
        assert!(r.row(4).is_none());
        assert_eq!(r.append(&monkey()), Ok(4), "the first add is id 4");
        assert_eq!(r.rows[4].enabled, 1, "written enabled");
        assert_eq!(r.rows[4].numbers(), monkey(), "stored exactly as sent");

        let before = r.rows[4];
        assert!(r.set_enabled(4, false));
        assert_eq!(r.rows[4].enabled, 0);
        assert!(r.set_enabled(4, true));
        assert_eq!(r.rows[4], before);
        assert_eq!(r.count, 5);
        for id in [0u8, 5, 31, 32, 255] {
            assert!(!r.set_enabled(id, true), "{id} is not registered");
        }
        assert!(r.rows[0].enabled == 0, "Custom can never be enabled into a row");
    }

    #[test]
    fn the_registry_is_full_past_thirty_one() {
        use crate::state::PresetAppendFault;
        let mut r = seeded_registry();
        for want in 4..=floor_types::constants::MAX_PRESET_ID {
            assert_eq!(r.append(&monkey()), Ok(want));
        }
        assert_eq!(r.count, 32);
        assert_eq!(r.append(&monkey()), Err(PresetAppendFault::TableFull));

        let mut bad = monkey();
        bad.stair_steps = 0;
        assert_eq!(r.append(&bad), Err(PresetAppendFault::TableFull));
        assert_eq!(r.count, 32, "the count stays at 32");
        assert!(r.row(31).is_some() && r.row(32).is_none());
    }
}
