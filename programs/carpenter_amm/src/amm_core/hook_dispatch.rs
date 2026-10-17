//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::{get_return_data, invoke_signed};
use carpenter_types::constants::*;
use carpenter_types::hook::{
    callback_discriminator, decode_record, validate_record, ArgsHeader, HookRecordV1, RecordError,
    RecordExpect, HOOK_INTERFACE_VERSION, PHASE_AFTER_ACTIONS, PHASE_AFTER_ADD_LIQUIDITY,
    PHASE_AFTER_DONATE, PHASE_AFTER_INITIALIZE, PHASE_AFTER_REMOVE_LIQUIDITY, PHASE_AFTER_SWAP,
    PHASE_BEFORE_ADD_LIQUIDITY, PHASE_BEFORE_DONATE, PHASE_BEFORE_INITIALIZE,
    PHASE_BEFORE_REMOVE_LIQUIDITY, PHASE_BEFORE_SWAP,
};

use crate::errors::AmmError;

///
pub struct HookCallContext<'a, 'info> {
///
    pub hook_program: &'a AccountInfo<'info>,
///
    pub pool: &'a AccountInfo<'info>,
///
    pub pool_signer_seeds: &'a [&'a [u8]],
///
///
///
///
///
    pub slice: &'a [AccountInfo<'info>],
///
    pub callback_accounts: u8,
///
///
    pub amm_accounts: &'a [Pubkey],
///
    pub pool_hook: Pubkey,
///
    pub flags: u16,
///
    pub max_rounds: u8,
///
///
///
///
///
///
    pub settle_accounts: &'a [AccountInfo<'info>],
}

///
///
pub fn is_self_call(pool_hook: &Pubkey, pool_hook_authority: &Pubkey, acting_signer: &Pubkey) -> bool {
    *pool_hook != Pubkey::default() && acting_signer == pool_hook_authority
}

///
///
pub const fn phase_flag(phase: u8) -> u16 {
    match phase {
        PHASE_BEFORE_INITIALIZE => BEFORE_INITIALIZE_FLAG,
        PHASE_AFTER_INITIALIZE => AFTER_INITIALIZE_FLAG,
        PHASE_BEFORE_SWAP => BEFORE_SWAP_FLAG,
        PHASE_AFTER_SWAP => AFTER_SWAP_FLAG,
        PHASE_AFTER_ACTIONS => REQUESTS_ACTIONS_FLAG,
        PHASE_BEFORE_ADD_LIQUIDITY => BEFORE_ADD_LIQUIDITY_FLAG,
        PHASE_AFTER_ADD_LIQUIDITY => AFTER_ADD_LIQUIDITY_FLAG,
        PHASE_BEFORE_REMOVE_LIQUIDITY => BEFORE_REMOVE_LIQUIDITY_FLAG,
        PHASE_AFTER_REMOVE_LIQUIDITY => AFTER_REMOVE_LIQUIDITY_FLAG,
        PHASE_BEFORE_DONATE => BEFORE_DONATE_FLAG,
        PHASE_AFTER_DONATE => AFTER_DONATE_FLAG,
        _ => 0,
    }
}

///
pub fn should_call(flags: u16, phase: u8, self_call: bool) -> bool {
    !self_call && flags & phase_flag(phase) != 0
}

///
pub fn map_record_error(e: RecordError) -> AmmError {
    match e {
        RecordError::BadReturn => AmmError::HookBadReturn,
        RecordError::FieldNotAllowed => AmmError::HookFieldNotAllowed,
        RecordError::FeeTooLarge => AmmError::FeeInvalid,
        RecordError::RoundsExceeded => AmmError::HookRoundsExceeded,
        RecordError::DeltaExceedsSwapAmount => AmmError::HookDeltaExceedsSwapAmount,
        RecordError::Overflow => AmmError::MathOverflow,
        RecordError::FeesInconsistent => AmmError::HookFeesInconsistent,
    }
}

///
pub const fn is_simple_phase(phase: u8) -> bool {
    matches!(
        phase,
        PHASE_BEFORE_INITIALIZE
            | PHASE_AFTER_INITIALIZE
            | PHASE_BEFORE_ADD_LIQUIDITY
            | PHASE_AFTER_ADD_LIQUIDITY
            | PHASE_BEFORE_REMOVE_LIQUIDITY
            | PHASE_AFTER_REMOVE_LIQUIDITY
            | PHASE_BEFORE_DONATE
            | PHASE_AFTER_DONATE
    )
}

///
///
///
///
///
///
pub fn call_simple<A: AnchorSerialize>(ctx: &HookCallContext, phase: u8, args: &A) -> Result<HookRecordV1> {
    require!(is_simple_phase(phase), AmmError::HookFieldNotAllowed);
    dispatch(ctx, phase, args)
}

///
pub fn dispatch<A: AnchorSerialize>(ctx: &HookCallContext, phase: u8, args: &A) -> Result<HookRecordV1> {

    require!(ctx.pool_hook != Pubkey::default(), AmmError::NoHook);
    require_keys_eq!(ctx.hook_program.key(), ctx.pool_hook, AmmError::HookBadReturn);


    require!(ctx.slice.len() == ctx.callback_accounts as usize, AmmError::HookSliceLengthMismatch);


    for acc in ctx.slice {
        require!(
            acc.key() != ctx.pool.key() && !ctx.amm_accounts.contains(acc.key),
            AmmError::HookSliceAliasesAmmAccount
        );
    }


    let args_bytes = borsh::to_vec(args).map_err(|_| AmmError::MathOverflow)?;
    let header = ArgsHeader::peek(&args_bytes).ok_or(AmmError::HookBadReturn)?;
    require!(
        header.version == HOOK_INTERFACE_VERSION && header.phase == phase,
        AmmError::HookBadReturn
    );
    let disc = callback_discriminator(phase).ok_or(AmmError::HookBadReturn)?;

    let mut data = Vec::with_capacity(8 + args_bytes.len());
    data.extend_from_slice(&disc);
    data.extend_from_slice(&args_bytes);





    let settles = ctx.flags & SETTLES_FLAG != 0;
    let settle_bump = if settles {
        let (pda, bump) = Pubkey::find_program_address(&[carpenter_types::seeds::HOOK_SETTLE, ctx.pool.key.as_ref()], &crate::ID);
        require!(ctx.slice.first().map(|a| *a.key) == Some(pda), AmmError::HookSettleSignerMissing);
        bump
    } else {
        0
    };
    let extra = if settles { ctx.settle_accounts } else { &ctx.settle_accounts[..0] };

    let mut metas = Vec::with_capacity(1 + ctx.slice.len() + extra.len());
    metas.push(AccountMeta::new_readonly(ctx.pool.key(), !settles));
    for (i, acc) in ctx.slice.iter().enumerate() {
        let signer = settles && i == 0;
        metas.push(if acc.is_writable {
            AccountMeta::new(acc.key(), signer)
        } else {
            AccountMeta::new_readonly(acc.key(), signer)
        });
    }
    for acc in extra {
        metas.push(if acc.is_writable {
            AccountMeta::new(acc.key(), false)
        } else {
            AccountMeta::new_readonly(acc.key(), false)
        });
    }
    let ix = Instruction { program_id: ctx.pool_hook, accounts: metas, data };

    let mut infos = Vec::with_capacity(2 + ctx.slice.len() + extra.len());
    infos.push(ctx.pool.clone());
    infos.extend(ctx.slice.iter().cloned());
    infos.extend(extra.iter().cloned());
    infos.push(ctx.hook_program.clone());

    if settles {
        let bump = [settle_bump];
        let seeds: [&[u8]; 3] = [carpenter_types::seeds::HOOK_SETTLE, ctx.pool.key.as_ref(), &bump];
        invoke_signed(&ix, &infos, &[&seeds])?;
    } else {
        invoke_signed(&ix, &infos, &[ctx.pool_signer_seeds])?;
    }


    let (from, bytes) = get_return_data().ok_or(AmmError::HookBadReturn)?;
    require_keys_eq!(from, ctx.pool_hook, AmmError::HookBadReturn);
    require!(!bytes.is_empty(), AmmError::HookBadReturn);

    let rec = decode_record(&bytes).map_err(map_record_error)?;
    let expect = RecordExpect {
        version: header.version,
        phase: header.phase,
        round: header.round,
        flags: ctx.flags,
        max_rounds: ctx.max_rounds,
    };
    validate_record(&rec, &expect).map_err(map_record_error)?;
    Ok(rec)
}





///
///
pub struct PoolSeeds {
    pub mint0: Pubkey,
    pub mint1: Pubkey,
    pub fee: [u8; 4],
    pub tick_spacing: [u8; 2],
    pub hook: Pubkey,
    pub bump: [u8; 1],
}

impl PoolSeeds {
    pub fn new(mint0: Pubkey, mint1: Pubkey, fee: u32, tick_spacing: u16, hook: Pubkey, bump: u8) -> Self {
        Self { mint0, mint1, fee: fee.to_le_bytes(), tick_spacing: tick_spacing.to_le_bytes(), hook, bump: [bump] }
    }

///
    pub fn of(pool: &crate::state::Pool) -> Self {
        Self::new(pool.mint0, pool.mint1, pool.fee, pool.tick_spacing, pool.hook, pool.bump)
    }

///
    pub fn seeds(&self) -> [&[u8]; 7] {
        [
            carpenter_types::seeds::POOL,
            self.mint0.as_ref(),
            self.mint1.as_ref(),
            &self.fee,
            &self.tick_spacing,
            self.hook.as_ref(),
            &self.bump,
        ]
    }
}

///
pub fn snapshot(pool: &crate::state::Pool) -> carpenter_types::hook::PoolSnapshot {
    carpenter_types::hook::PoolSnapshot {
        sqrt_price_x64: pool.sqrt_price_x64.get(),
        tick: pool.tick_current,
        liquidity: pool.liquidity.get(),
        lp_fee_pips: pool.lp_fee,
    }
}
