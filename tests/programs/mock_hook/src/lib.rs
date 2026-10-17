//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::{invoke, invoke_signed, set_return_data};
use carpenter_types::hook::{ArgsHeader, HookAction as TypesAction, HookRecordV1 as TypesRecord};

declare_id!("GCZ8cGz5X7Na5BWV663aLumnkzuEhWchwqsb4ZRmdT6F");

///
pub mod transfer_log;

///
pub const SENTINEL: u64 = 0x5EA1_ED0F_F100_2A6E;

///
pub const RETURN_NORMAL: u8 = 0;
///
pub const RETURN_NONE: u8 = 1;
///
///
pub const RETURN_FROM_TOKEN_PROGRAM: u8 = 2;
///
pub const RETURN_TRAILING_BYTE: u8 = 3;

///
pub const DELTA_FIXED: u8 = 0;
///
pub const DELTA_EQUALS_SPECIFIED: u8 = 1;
///
pub const DELTA_OUTPUT_PLUS_ONE: u8 = 1;

///
///
pub const SETTLE_OFF: u8 = 0;
pub const SETTLE_REMAINDER: u8 = 1;
pub const SETTLE_REMAINDER_PLUS_ONE: u8 = 2;
///
pub const MINT_AUTHORITY_SEED: &[u8] = b"mint_authority";

///
pub const ACTION_MODIFY_POSITION: u8 = 0;
pub const ACTION_SWAP: u8 = 1;

#[program]
pub mod mock_hook {
    use super::*;

///
    pub fn initialize_hook_config(
        ctx: Context<InitializeHookConfig>,
        flags: u16,
        max_rounds: u8,
        callback_accounts: u8,
    ) -> Result<()> {
        let mut hc = ctx.accounts.hook_config.load_init()?;
        hc.version = 1;
        hc.bump = ctx.bumps.hook_config;
        hc.max_rounds = max_rounds;
        hc.callback_accounts = callback_accounts;
        hc.flags = flags;
        hc.authority_bump = Pubkey::find_program_address(&[carpenter_types::seeds::HOOK_AUTHORITY], &crate::ID).1;
        hc.authority = ctx.accounts.authority.key();
        Ok(())
    }

    pub fn before_initialize(ctx: Context<Callback>, version: u8, phase: u8, round: u8) -> Result<HookRecordV1> {
        pass_through(&ctx, ArgsHeader { version, phase, round })
    }

    pub fn after_initialize(ctx: Context<Callback>, version: u8, phase: u8, round: u8) -> Result<HookRecordV1> {
        pass_through(&ctx, ArgsHeader { version, phase, round })
    }

    pub fn before_add_liquidity(ctx: Context<Callback>, version: u8, phase: u8, round: u8) -> Result<HookRecordV1> {
        pass_through(&ctx, ArgsHeader { version, phase, round })
    }

    pub fn after_add_liquidity(ctx: Context<Callback>, version: u8, phase: u8, round: u8) -> Result<HookRecordV1> {
        pass_through(&ctx, ArgsHeader { version, phase, round })
    }

    pub fn before_remove_liquidity(ctx: Context<Callback>, version: u8, phase: u8, round: u8) -> Result<HookRecordV1> {
        pass_through(&ctx, ArgsHeader { version, phase, round })
    }

    pub fn after_remove_liquidity(ctx: Context<Callback>, version: u8, phase: u8, round: u8) -> Result<HookRecordV1> {
        pass_through(&ctx, ArgsHeader { version, phase, round })
    }

    pub fn before_swap<'info>(ctx: Context<'info, Callback<'info>>, args: BeforeSwapArgs) -> Result<()> {
        let hdr = ArgsHeader { version: args.version, phase: args.phase, round: args.round };
        scripted(&ctx, hdr, View::Before(&args))
    }

    pub fn after_swap<'info>(ctx: Context<'info, Callback<'info>>, args: AfterSwapArgs) -> Result<()> {
        let hdr = ArgsHeader { version: args.version, phase: args.phase, round: args.round };
        scripted(&ctx, hdr, View::After(&args))
    }

    pub fn before_donate(ctx: Context<Callback>, version: u8, phase: u8, round: u8) -> Result<HookRecordV1> {
        pass_through(&ctx, ArgsHeader { version, phase, round })
    }

    pub fn after_donate(ctx: Context<Callback>, version: u8, phase: u8, round: u8) -> Result<HookRecordV1> {
        pass_through(&ctx, ArgsHeader { version, phase, round })
    }

    pub fn after_actions<'info>(ctx: Context<'info, Callback<'info>>, args: AfterActionsArgs) -> Result<()> {
        let hdr = ArgsHeader { version: args.version, phase: args.phase, round: args.round };
        scripted(&ctx, hdr, View::Actions(&args))
    }

///
///
///
///
///
///
///
///
    pub fn forward<'info>(ctx: Context<'info, Forward<'info>>, data: Vec<u8>) -> Result<()> {
        let (authority, bump) = Pubkey::find_program_address(&[carpenter_types::seeds::HOOK_AUTHORITY], &crate::ID);
        require_keys_eq!(ctx.accounts.hook_authority.key(), authority, MockHookError::WrongAuthority);
        let metas: Vec<AccountMeta> = ctx
            .remaining_accounts
            .iter()
            .map(|a| AccountMeta { pubkey: *a.key, is_signer: a.is_signer || *a.key == authority, is_writable: a.is_writable })
            .collect();
        let ix = Instruction { program_id: ctx.accounts.target_program.key(), accounts: metas, data };
        let mut infos: Vec<AccountInfo<'info>> = ctx.remaining_accounts.to_vec();
        infos.push(ctx.accounts.hook_authority.to_account_info());
        infos.push(ctx.accounts.target_program.to_account_info());
        invoke_signed(&ix, &infos, &[&[carpenter_types::seeds::HOOK_AUTHORITY, &[bump]]])?;
        Ok(())
    }

///
///
///
///
///
    pub fn fallback<'info>(program_id: &Pubkey, accounts: &'info [AccountInfo<'info>], data: &[u8]) -> Result<()> {
        crate::transfer_log::execute(program_id, accounts, data)
    }

///
///
///
///
///
///
///
///
    pub fn set_reported_fees(ctx: Context<SetReportedFees>, fees: [u64; 5], fees_after: [u64; 5]) -> Result<()> {
        for f in [&fees, &fees_after] {
            require!(f[carpenter_types::hook::FEE_PLATFORM] <= f[carpenter_types::hook::FEE_SKIM], MockHookError::FeesInconsistent);
            charged(f)?;
        }
        let mut hc = ctx.accounts.hook_config.load_mut()?;
        hc.reported_fees = fees;
        hc.reported_fees_after = fees_after;
        Ok(())
    }
}

///
///
fn charged(fees: &[u64; 5]) -> Result<u64> {
    use carpenter_types::hook::{FEE_CREATOR, FEE_HOLDER, FEE_SKIM, FEE_TEAM};
    fees[FEE_SKIM]
        .checked_add(fees[FEE_CREATOR])
        .and_then(|s| s.checked_add(fees[FEE_HOLDER]))
        .and_then(|s| s.checked_add(fees[FEE_TEAM]))
        .ok_or_else(|| error!(MockHookError::FeesInconsistent))
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
fn reported_fees<'info>(remaining: &'info [AccountInfo<'info>]) -> Result<Option<([u64; 5], [u64; 5])>> {
    for ai in remaining {
        if ai.owner != &crate::ID {
            continue;
        }
        let Ok(loader) = AccountLoader::<HookConfig>::try_from(ai) else { continue };
        let hc = loader.load()?;
        let want = Pubkey::create_program_address(&[carpenter_types::seeds::HOOK_CONFIG, &[hc.bump]], &crate::ID)
            .map_err(|_| error!(MockHookError::ConfigMismatch))?;
        require_keys_eq!(*ai.key, want, MockHookError::ConfigMismatch);
        let (before, after) = (hc.reported_fees, hc.reported_fees_after);
        return Ok(if before == [0; 5] && after == [0; 5] { None } else { Some((before, after)) });
    }
    Ok(None)
}

///
///
///
///
///
///
fn pass_through(ctx: &Context<Callback>, hdr: ArgsHeader) -> Result<HookRecordV1> {
    require!(ctx.accounts.pool_signer.is_signer, MockHookError::PoolNotSigner);
    Ok(TypesRecord::pass_through(hdr.version, hdr.phase, hdr.round).into())
}

enum View<'a> {
    Before(&'a BeforeSwapArgs),
    After(&'a AfterSwapArgs),
    Actions(&'a AfterActionsArgs),
}

///
///
///
fn mock_state<'info>(remaining: &'info [AccountInfo<'info>]) -> Option<&'info AccountInfo<'info>> {
    remaining.iter().take(2).find(|a| {
        *a.owner == crate::ID
            && a.try_borrow_data().map(|d| d.len() == MockState::LEN && d[..8] == *MockState::DISCRIMINATOR).unwrap_or(false)
    })
}

///
fn mock_script<'info>(remaining: &'info [AccountInfo<'info>]) -> Option<&'info AccountInfo<'info>> {
    let a = remaining.get(1)?;

    if *a.owner != crate::ID {
        return None;
    }
    let data = a.try_borrow_data().ok()?;
    (data.len() == MockScript::LEN && data[..8] == *MockScript::DISCRIMINATOR).then_some(a)
}

fn i128_of(limbs: [u64; 2]) -> i128 {
    ((limbs[0] as u128) | ((limbs[1] as u128) << 64)) as i128
}

fn u128_of(limbs: [u64; 2]) -> u128 {
    (limbs[0] as u128) | ((limbs[1] as u128) << 64)
}

fn scripted<'info>(ctx: &Context<'info, Callback<'info>>, hdr: ArgsHeader, view: View<'_>) -> Result<()> {


    let settle_signed = ctx.remaining_accounts.first().map(|a| a.is_signer).unwrap_or(false);
    require!(ctx.accounts.pool_signer.is_signer || settle_signed, MockHookError::PoolNotSigner);
    let mut rec = TypesRecord::pass_through(hdr.version, hdr.phase, hdr.round);
    let mut return_mode = RETURN_NORMAL;

    let (is_before, is_after) = (matches!(view, View::Before(_)), matches!(view, View::After(_)));

    if let Some(info) = mock_state(ctx.remaining_accounts) {
        let mut s: MockState = {
            let data = info.try_borrow_data()?;
            *bytemuck::from_bytes::<MockState>(&data[8..MockState::LEN])
        };
        if s.pool != Pubkey::default() {
            require_keys_eq!(ctx.accounts.pool_signer.key(), s.pool, MockHookError::WrongPool);
        }
        let bit = 1u16 << hdr.phase;

        match view {
            View::Before(a) => {
                s.sentinel = SENTINEL;
                s.last_claims0 = a.hook_claims0;
                s.last_claims1 = a.hook_claims1;
                rec.delta_specified = match s.before_delta_mode {
                    DELTA_EQUALS_SPECIFIED => a.params.amount_specified.checked_neg().ok_or(MockHookError::Overflow)?,
                    _ => i128_of(s.before_delta_specified),
                };
                rec.delta_unspecified = i128_of(s.before_delta_unspecified);
                if s.settle_mode != SETTLE_OFF {
                    rec.sqrt_price_limit_x64 = u128_of(s.settle_price_limit);
                }
            }
            View::After(a) => {
                s.last_claims0 = a.hook_claims0;
                s.last_claims1 = a.hook_claims1;
                rec.delta_unspecified = match s.after_delta_mode {
                    DELTA_OUTPUT_PLUS_ONE => {
                        let out = if a.params.zero_for_one { a.swap_delta.amount1 } else { a.swap_delta.amount0 };
                        out.checked_add(1).ok_or(MockHookError::Overflow)?
                    }
                    _ => i128_of(s.after_delta_unspecified),
                };
                rec.signals = s.signals;
                if s.settle_mode != SETTLE_OFF {
                    settle_after(ctx.remaining_accounts, &s, a, &mut rec)?;
                }
            }
            View::Actions(a) => {
                s.last_claims0 = a.hook_claims0;
                s.last_claims1 = a.hook_claims1;
                s.last_results = a.results.len() as u32;
            }
        }
        if s.fee_override_mask & bit != 0 {
            rec.fee_override_pips = Some(s.fee_override_pips);
        }
        if s.want_callback_mask & bit != 0 {
            rec.want_callback = true;
        }
        if s.actions_mask & bit != 0 {
            let action = if s.action_kind == ACTION_MODIFY_POSITION {
                TypesAction::ModifyPosition {
                    salt: s.action_salt,
                    tick_lower: s.action_tick_lower,
                    tick_upper: s.action_tick_upper,
                    liquidity_delta: i128_of(s.action_liquidity_delta),
                }
            } else {
                TypesAction::Swap {
                    zero_for_one: s.action_zero_for_one != 0,
                    amount_specified: i128_of(s.action_amount_specified),
                    sqrt_price_limit_x64: u128_of(s.action_sqrt_price_limit),
                }
            };
            rec.actions = vec![action; s.action_count as usize];
        }
        if s.bad_version_mask & bit != 0 {
            rec.version = rec.version.wrapping_add(1);
        }
        if s.bad_round_mask & bit != 0 {
            rec.round = rec.round.wrapping_add(1);
        }
        if s.return_mode_mask & bit != 0 {
            return_mode = s.return_mode;
        }
        s.calls += 1;
        s.last_phase = hdr.phase;
        s.last_round = hdr.round;
        if info.is_writable {
            let mut data = info.try_borrow_mut_data()?;
            *bytemuck::from_bytes_mut::<MockState>(&mut data[8..MockState::LEN]) = s;
        }
    }


    if let Some(info) = mock_script(ctx.remaining_accounts) {
        if hdr.round >= 1 && (hdr.round as usize) <= SCRIPT_ROUNDS {
            let data = info.try_borrow_data()?;
            let script = bytemuck::from_bytes::<MockScript>(&data[8..MockScript::LEN]);
            let r = &script.rounds[hdr.round as usize - 1];
            if r.want_callback != 0 {
                rec.want_callback = true;
            }
            if r.action_count > 0 {
                let n = (r.action_count as usize).min(SCRIPT_ACTIONS);
                rec.actions = r.actions[..n]
                    .iter()
                    .map(|a| {
                        if a.kind == ACTION_MODIFY_POSITION {
                            TypesAction::ModifyPosition {
                                salt: a.salt,
                                tick_lower: a.tick_lower,
                                tick_upper: a.tick_upper,
                                liquidity_delta: i128_of(a.liquidity_delta),
                            }
                        } else {
                            TypesAction::Swap {
                                zero_for_one: a.zero_for_one != 0,
                                amount_specified: i128_of(a.amount_specified),
                                sqrt_price_limit_x64: u128_of(a.sqrt_price_limit),
                            }
                        }
                    })
                    .collect();
            }
        }
    }














    if let Some((before, after)) = reported_fees(ctx.remaining_accounts)? {
        let leg = if is_before {
            Some(before)
        } else if is_after {
            Some(after)
        } else {
            None
        };
        if let Some(fees) = leg.filter(|f| *f != [0; 5]) {
            let charge = i128::from(charged(&fees)?);
            if is_before {
                rec.delta_specified = charge;
            } else {
                rec.delta_unspecified = charge;
            }
            rec.fees = fees;
        }
    }


    let mut bytes = carpenter_types::hook::encode_record(&rec);
    match return_mode {
        RETURN_NONE => {}
        RETURN_FROM_TOKEN_PROGRAM => {
            let mint = ctx.remaining_accounts.get(1).ok_or(ErrorCode::AccountNotEnoughKeys)?;
            let token_program = ctx.remaining_accounts.get(2).ok_or(ErrorCode::AccountNotEnoughKeys)?;

            let ix = Instruction {
                program_id: *token_program.key,
                accounts: vec![AccountMeta::new_readonly(*mint.key, false)],
                data: vec![21],
            };
            invoke(&ix, &[mint.clone(), token_program.clone()])?;
        }
        RETURN_TRAILING_BYTE => {
            bytes.push(0);
            set_return_data(&bytes);
        }
        _ => set_return_data(&bytes),
    }
    Ok(())
}


///
///
///
///
///
///
fn settle_after<'info>(remaining: &'info [AccountInfo<'info>], s: &MockState, a: &AfterSwapArgs, rec: &mut TypesRecord) -> Result<()> {
    let n = remaining.len();
    require!(n >= 6, ErrorCode::AccountNotEnoughKeys);
    let c = s.settle_currency as usize & 1;
    let (vault, mint, token_program) = (&remaining[n - 6 + c], &remaining[n - 4 + c], &remaining[n - 2 + c]);
    if s.settle_mint_amount != 0 {
        let (authority, bump) = Pubkey::find_program_address(&[MINT_AUTHORITY_SEED], &crate::ID);
        let auth_info = remaining.iter().find(|x| *x.key == authority).ok_or(ErrorCode::AccountNotEnoughKeys)?;
        let mut data = vec![7u8];
        data.extend_from_slice(&s.settle_mint_amount.to_le_bytes());
        let ix = Instruction {
            program_id: *token_program.key,
            accounts: vec![
                AccountMeta::new(*mint.key, false),
                AccountMeta::new(*vault.key, false),
                AccountMeta::new_readonly(authority, true),
            ],
            data,
        };
        invoke_signed(&ix, &[mint.clone(), vault.clone(), auth_info.clone(), token_program.clone()], &[&[MINT_AUTHORITY_SEED, &[bump]]])?;
    }


    let consumed = if a.params.zero_for_one { a.swap_delta.amount0 } else { a.swap_delta.amount1 }.unsigned_abs() as i128;
    let remainder = if a.params.amount_specified < 0 {
        a.params.amount_specified.checked_neg().ok_or(MockHookError::Overflow)?.checked_sub(consumed).ok_or(MockHookError::Overflow)?
    } else {
        0
    };
    rec.delta_specified = match s.settle_mode {
        SETTLE_REMAINDER_PLUS_ONE => remainder.checked_add(1).ok_or(MockHookError::Overflow)?,
        _ => remainder,
    };
    rec.delta_unspecified = -i128::from(s.settle_pay_out);
    Ok(())
}

#[derive(Accounts)]
pub struct SetReportedFees<'info> {
    pub authority: Signer<'info>,
    #[account(mut, seeds = [carpenter_types::seeds::HOOK_CONFIG], bump = hook_config.load()?.bump,
              constraint = hook_config.load()?.authority == authority.key() @ MockHookError::NotAuthority)]
    pub hook_config: AccountLoader<'info, HookConfig>,
}

#[derive(Accounts)]
pub struct InitializeHookConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, payer = authority, space = 8 + 128, seeds = [carpenter_types::seeds::HOOK_CONFIG], bump)]
    pub hook_config: AccountLoader<'info, HookConfig>,
    pub system_program: Program<'info, System>,
}

///
#[derive(Accounts)]
pub struct Callback<'info> {
///
    pub pool_signer: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct Forward<'info> {
///
///
    pub hook_authority: UncheckedAccount<'info>,
///
    pub target_program: UncheckedAccount<'info>,
}

///
#[account(zero_copy)]
pub struct HookConfig {
    pub version: u8,
    pub bump: u8,
    pub max_rounds: u8,
    pub callback_accounts: u8,
    pub flags: u16,
    pub authority_bump: u8,
    pub pad0: u8,
    pub authority: Pubkey,
///
///
///
///
    pub reported_fees: [u64; 5],
///
///
///
    pub reported_fees_after: [u64; 5],
///
    pub reserved: [u8; 8],
}
const _: () = assert!(core::mem::size_of::<HookConfig>() == 128);
const _: () = assert!(core::mem::offset_of!(HookConfig, authority) == 8);
const _: () = assert!(core::mem::offset_of!(HookConfig, reported_fees) == 40);
const _: () = assert!(core::mem::offset_of!(HookConfig, reported_fees_after) == 80);

///
///
///
///
#[account(zero_copy)]
#[derive(Debug)]
pub struct MockState {
///
    pub pool: Pubkey,
///
    pub sentinel: u64,
///
    pub calls: u64,
    pub bad_version_mask: u16,
    pub bad_round_mask: u16,
    pub fee_override_mask: u16,
    pub want_callback_mask: u16,
    pub actions_mask: u16,
    pub return_mode_mask: u16,
    pub return_mode: u8,
    pub before_delta_mode: u8,
    pub after_delta_mode: u8,
    pub action_count: u8,
    pub fee_override_pips: u32,
    pub action_kind: u8,
    pub action_zero_for_one: u8,
///
    pub signals: u8,
///
    pub last_phase: u8,
    pub before_delta_specified: [u64; 2],
    pub before_delta_unspecified: [u64; 2],
    pub after_delta_unspecified: [u64; 2],
    pub action_salt: u64,
    pub action_tick_lower: i32,
    pub action_tick_upper: i32,
    pub action_liquidity_delta: [u64; 2],
    pub action_amount_specified: [u64; 2],
    pub action_sqrt_price_limit: [u64; 2],
///
    pub last_round: u8,
    pub pad0: [u8; 3],
///
    pub last_results: u32,
///
    pub last_claims0: u64,
    pub last_claims1: u64,
///
    pub settle_price_limit: [u64; 2],
///
    pub settle_mint_amount: u64,
///
    pub settle_pay_out: u64,
    pub settle_mode: u8,
    pub settle_currency: u8,
    pub pad1: [u8; 14],
}
impl MockState {
    pub const LEN: usize = 8 + 256;
}
const _: () = assert!(core::mem::size_of::<MockState>() == 256);
const _: () = assert!(core::mem::offset_of!(MockState, before_delta_specified) == 72);
const _: () = assert!(core::mem::offset_of!(MockState, last_claims0) == 192);
const _: () = assert!(core::mem::offset_of!(MockState, settle_price_limit) == 208);
const _: () = assert!(core::mem::offset_of!(MockState, settle_mode) == 240);

///
///
pub const SCRIPT_ROUNDS: usize = 4;
///
pub const SCRIPT_ACTIONS: usize = 3;

///
///
#[zero_copy]
#[derive(Debug)]
pub struct ScriptAction {
    pub kind: u8,
    pub zero_for_one: u8,
    pub pad0: [u8; 6],
    pub salt: u64,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub liquidity_delta: [u64; 2],
    pub amount_specified: [u64; 2],
    pub sqrt_price_limit: [u64; 2],
}
const _: () = assert!(core::mem::size_of::<ScriptAction>() == 72);

///
///
#[zero_copy]
#[derive(Debug)]
pub struct ScriptRound {
    pub want_callback: u8,
    pub action_count: u8,
    pub pad0: [u8; 6],
    pub actions: [ScriptAction; SCRIPT_ACTIONS],
}
const _: () = assert!(core::mem::size_of::<ScriptRound>() == 224);

///
///
///
#[account(zero_copy)]
#[derive(Debug)]
pub struct MockScript {
    pub rounds: [ScriptRound; SCRIPT_ROUNDS],
}
impl MockScript {
    pub const LEN: usize = 8 + 896;
}
const _: () = assert!(core::mem::size_of::<MockScript>() == 896);



#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PoolSnapshot {
    pub sqrt_price_x64: u128,
    pub tick: i32,
    pub liquidity: u128,
    pub lp_fee_pips: u32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BalanceDelta {
    pub amount0: i128,
    pub amount1: i128,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SwapParams {
    pub zero_for_one: bool,
    pub amount_specified: i128,
    pub sqrt_price_limit_x64: u128,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct BeforeSwapArgs {
    pub version: u8,
    pub phase: u8,
    pub round: u8,
    pub pool: [u8; 32],
    pub sender: [u8; 32],
    pub params: SwapParams,
    pub pre: PoolSnapshot,
    pub supply0: u64,
    pub supply1: u64,
    pub hook_claims0: u64,
    pub hook_claims1: u64,
    pub hook_data: Vec<u8>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct AfterSwapArgs {
    pub version: u8,
    pub phase: u8,
    pub round: u8,
    pub pool: [u8; 32],
    pub sender: [u8; 32],
    pub params: SwapParams,
    pub pre: PoolSnapshot,
    pub post: PoolSnapshot,
    pub swap_delta: BalanceDelta,
    pub before_delta_specified: i128,
    pub before_delta_unspecified: i128,
    pub supply0: u64,
    pub supply1: u64,
    pub hook_claims0: u64,
    pub hook_claims1: u64,
    pub hook_data: Vec<u8>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ActionResult {
    pub index: u8,
    pub kind: u8,
    pub delta: BalanceDelta,
    pub fees0: u64,
    pub fees1: u64,
    pub post_sqrt_price_x64: u128,
    pub post_tick: i32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct AfterActionsArgs {
    pub version: u8,
    pub phase: u8,
    pub round: u8,
    pub pool: [u8; 32],
    pub post: PoolSnapshot,
    pub supply0: u64,
    pub supply1: u64,
    pub hook_claims0: u64,
    pub hook_claims1: u64,
    pub results: Vec<ActionResult>,
}

///
///
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct HookRecordV1 {
    pub version: u8,
    pub phase: u8,
    pub round: u8,
    pub want_callback: bool,
    pub fee_override_pips: Option<u32>,
    pub delta_specified: i128,
    pub delta_unspecified: i128,
    pub signals: u8,











    pub fees: [u64; 5],
    pub actions: Vec<HookAction>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum HookAction {
    ModifyPosition { salt: u64, tick_lower: i32, tick_upper: i32, liquidity_delta: i128 },
    Swap { zero_for_one: bool, amount_specified: i128, sqrt_price_limit_x64: u128 },
}

impl From<TypesRecord> for HookRecordV1 {
    fn from(r: TypesRecord) -> Self {
        Self {
            version: r.version,
            phase: r.phase,
            round: r.round,
            want_callback: r.want_callback,
            fee_override_pips: r.fee_override_pips,
            delta_specified: r.delta_specified,
            delta_unspecified: r.delta_unspecified,
            signals: r.signals,
            fees: r.fees,
            actions: r
                .actions
                .into_iter()
                .map(|a| match a {
                    TypesAction::ModifyPosition { salt, tick_lower, tick_upper, liquidity_delta } => {
                        HookAction::ModifyPosition { salt, tick_lower, tick_upper, liquidity_delta }
                    }
                    TypesAction::Swap { zero_for_one, amount_specified, sqrt_price_limit_x64 } => {
                        HookAction::Swap { zero_for_one, amount_specified, sqrt_price_limit_x64 }
                    }
                })
                .collect(),
        }
    }
}

///
#[error_code]
pub enum MockHookError {
    #[msg("pool_signer did not sign")]
    PoolNotSigner,
    #[msg("pool_signer is not the pool this mock state is bound to (D-14, mock form)")]
    WrongPool,
    #[msg("arithmetic overflow in a scripted delta")]
    Overflow,
    #[msg("forward: account 0 is not this program's [\"hook_authority\"] PDA")]
    WrongAuthority,
    #[msg("signer is not the hook_config authority")]
    NotAuthority,
    #[msg("reported fees: platform cut above the skim, or the charge overflows u64")]
    FeesInconsistent,
    #[msg("a HookConfig in the hook slice is not this program's [\"hook_config\"] PDA")]
    ConfigMismatch,

    #[msg("the stub transfer hook answers Execute only")]
    NotExecute,
    #[msg("Execute carried fewer than the nine accounts the list names")]
    MissingExtraAccounts,
    #[msg("Execute account 5 is not this program's TransferLog")]
    NotALog,
    #[msg("the stub transfer hook was told to fail this transfer (fail_next)")]
    Refused,
}
