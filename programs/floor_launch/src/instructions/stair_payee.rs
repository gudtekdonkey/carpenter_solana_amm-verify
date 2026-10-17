//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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
use anchor_lang::solana_program::instruction::{get_stack_height, Instruction};
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};
use floor_types::constants::{LAUNCH_STATE_GRADUATED, TEAM_PAYEE_PENDING, TEAM_PAYEE_RESOLVED, TOKEN_DECIMALS};
use floor_types::slots::{slot_claim_message, PayeeSlots, SLOT_KIND_TEAM, STAIR_PAYEE_CLAIM_PREFIX_V2};
use floor_types::disc;

use crate::constants::{
    EVENT_AUTHORITY_SEED, FLOOR_HOOK_PROGRAM, GENESIS_SALE_SEED, HOOK_CONFIG_SEED, LAUNCH_AUTHORITY_SEED,
    LAUNCH_CONFIG_SEED, LAUNCH_SEED,
};
use crate::errors::LaunchError;
use crate::events::{TeamVestedClaimed, StairPayeeClaimed};
use crate::instructions::claim_recipient::{attestation_matches, pending_slot_id, previous_ed25519_data, MSG_LEN};
use crate::state::{GenesisSale, Launch, LaunchConfig};



#[event_cpi]
#[derive(Accounts)]
pub struct ClaimTeamVested<'info> {
    pub anyone: Signer<'info>,
    #[account(mut, seeds = [LAUNCH_SEED, mint.key().as_ref()], bump)]
    pub launch: AccountLoader<'info, Launch>,
    #[account(address = launch.load()?.mint @ LaunchError::BadConfig)]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, address = launch.load()?.token_vault @ LaunchError::BadConfig)]
    pub token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
///
    #[account(mut)]
    pub destination: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_2022: Program<'info, Token2022>,
///
///
    #[account(address = launch.load()?.floor_pool @ LaunchError::BadConfig)]
    pub floor_pool: UncheckedAccount<'info>,
}

///
///
fn team_slot_wallet(floor_pool: &AccountInfo, slot: u8) -> Result<Option<([u8; 32], u16)>> {
    require_keys_eq!(*floor_pool.owner, FLOOR_HOOK_PROGRAM, LaunchError::BadConfig);
    let data = floor_pool.try_borrow_data()?;
    let t = PayeeSlots::from_account(&data).ok_or(error!(LaunchError::TeamNothingVested))?;
    let s = t.slot(SLOT_KIND_TEAM, slot).ok_or(error!(LaunchError::TeamNothingVested))?;
    Ok(s.wallet().map(|w| (w, s.bps)))
}

pub fn claim_team_vested<'info>(ctx: Context<'info, ClaimTeamVested<'info>>, slot: u8) -> Result<()> {
    require!(slot < floor_types::slots::MAX_SLOTS as u8, LaunchError::TeamNothingVested);
    let now = Clock::get()?.unix_timestamp;
    let (total, days, anchor, by_slot, mint, bump) = {
        let l = ctx.accounts.launch.load()?;
        require!(l.state == LAUNCH_STATE_GRADUATED && l.team_anchor != 0, LaunchError::TeamNothingVested);
        (l.team_wallet_tokens, l.team_vest_days, l.team_anchor, l.team_cliffs_paid_by_slot, l.mint, l.bump)
    };

    let (wallet, bps) =
        team_slot_wallet(&ctx.accounts.floor_pool.to_account_info(), slot)?.ok_or(error!(LaunchError::TeamNothingVested))?;
    require_keys_eq!(
        ctx.accounts.destination.key(),
        get_associated_token_address_with_program_id(&Pubkey::new_from_array(wallet), &mint, &ctx.accounts.token_2022.key()),
        LaunchError::NotAssociatedTokenAccount
    );


    let paid_mask = crate::sale::slot_cliff_mask(by_slot, slot);
    let (amount, mask) = crate::sale::team_vested_slot_part(total, &days, anchor, now, bps, paid_mask);
    require!(mask != paid_mask, LaunchError::TeamNothingVested);
    let cliffs = mask.count_ones() as u8;
    ctx.accounts.launch.load_mut()?.team_cliffs_paid_by_slot = crate::sale::with_slot_cliff_mask(by_slot, slot, mask);
    if amount == 0 {


        emit_cpi!(TeamVestedClaimed {
            launch: ctx.accounts.launch.key(),
            slot,
            to: ctx.accounts.destination.key(),
            amount,
            cliffs_paid: cliffs,
        });
        return Ok(());
    }
    let bump = [bump];
    let seeds: &[&[u8]] = &[LAUNCH_SEED, mint.as_ref(), &bump];




    let hooked = matches!(
        crate::lock::hook_slot(&ctx.accounts.mint.to_account_info())?,
        Some(crate::lock::HookSlot { program_id: Some(_), .. })
    );
    if hooked {
        let floor_pool = ctx.accounts.launch.load()?.floor_pool;
        let leg = VestedLeg {
            token_2022: ctx.accounts.token_2022.to_account_info(),
            token_vault: ctx.accounts.token_vault.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            destination: ctx.accounts.destination.to_account_info(),
            launch: ctx.accounts.launch.to_account_info(),
        };
        hooked_transfer_from_vault(&leg, ctx.remaining_accounts, &floor_pool, seeds, amount)?;
    } else {
        anchor_spl::token_2022::transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_2022.key(),
                anchor_spl::token_2022::TransferChecked {
                    from: ctx.accounts.token_vault.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.destination.to_account_info(),
                    authority: ctx.accounts.launch.to_account_info(),
                },
                &[seeds],
            ),
            amount,
            TOKEN_DECIMALS,
        )?;
    }
    emit_cpi!(TeamVestedClaimed {
        launch: ctx.accounts.launch.key(),
        slot,
        to: ctx.accounts.destination.key(),
        amount,
        cliffs_paid: cliffs,
    });
    Ok(())
}

///
struct VestedLeg<'info> {
    token_2022: AccountInfo<'info>,
    token_vault: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    destination: AccountInfo<'info>,
    launch: AccountInfo<'info>,
}

///
///
///
///
///
fn hooked_transfer_from_vault<'info>(
    a: &VestedLeg<'info>,
    remaining: &[AccountInfo<'info>],
    floor_pool: &Pubkey,
    seeds: &[&[u8]],
    amount: u64,
) -> Result<()> {
    use crate::constants::{FLOOR_HOLDERS_PROGRAM as H, HOLDER_BOOK_SEED, HOLDER_RECORD_SEED};
    let mint = a.mint.key();
    let pda = |s: &[&[u8]]| Pubkey::find_program_address(s, &H).0;
    let wanted: [(Pubkey, bool); 6] = [
        (pda(&[HOLDER_BOOK_SEED, mint.as_ref()]), true),
        (*floor_pool, false),
        (pda(&[HOLDER_RECORD_SEED, mint.as_ref(), a.token_vault.key.as_ref()]), true),
        (pda(&[HOLDER_RECORD_SEED, mint.as_ref(), a.destination.key.as_ref()]), true),
        (pda(&[floor_types::holders::EXTRA_ACCOUNT_METAS, mint.as_ref()]), false),
        (H, false),
    ];
    let mut ix = anchor_spl::token_2022::spl_token_2022::instruction::transfer_checked(
        a.token_2022.key,
        a.token_vault.key,
        a.mint.key,
        a.destination.key,
        a.launch.key,
        &[],
        amount,
        TOKEN_DECIMALS,
    )?;
    let mut infos = vec![a.token_vault.clone(), a.mint.clone(), a.destination.clone(), a.launch.clone()];
    for (key, writable) in wanted {
        let info = remaining.iter().find(|r| *r.key == key).ok_or(LaunchError::HolderRecordMismatch)?;
        require!(!writable || info.is_writable, LaunchError::HolderRecordMismatch);
        ix.accounts.push(if writable { AccountMeta::new(key, false) } else { AccountMeta::new_readonly(key, false) });
        infos.push(info.clone());
    }
    infos.push(a.token_2022.clone());
    invoke_signed(&ix, &infos, &[seeds])?;
    Ok(())
}



///
///
///
pub fn stair_payee_claim_message(
    program_id: &Pubkey,
    cluster: u8,
    launch: &Pubkey,
    slot: u8,
    pending_id: &[u8; 32],
    payee: &Pubkey,
    deadline: i64,
) -> [u8; MSG_LEN] {
    slot_claim_message(
        STAIR_PAYEE_CLAIM_PREFIX_V2,
        &program_id.to_bytes(),
        cluster,
        &launch.to_bytes(),
        slot,
        pending_id,
        &payee.to_bytes(),
        deadline,
    )
}

#[event_cpi]
#[derive(Accounts)]
pub struct ClaimStairPayee<'info> {
    pub payer: Signer<'info>,
    #[account(mut)]
    pub launch: AccountLoader<'info, Launch>,
    #[account(seeds = [LAUNCH_CONFIG_SEED], bump)]
    pub launch_config: AccountLoader<'info, LaunchConfig>,
///
    #[account(address = pubkey!("Sysvar1nstructions1111111111111111111111111"))]
    pub instructions: UncheckedAccount<'info>,
///
    #[account(seeds = [LAUNCH_AUTHORITY_SEED], bump)]
    pub launch_authority: UncheckedAccount<'info>,
///
    #[account(address = FLOOR_HOOK_PROGRAM)]
    pub floor_hook: UncheckedAccount<'info>,
///
    #[account(seeds = [HOOK_CONFIG_SEED], bump, seeds::program = FLOOR_HOOK_PROGRAM)]
    pub hook_config: UncheckedAccount<'info>,
///
///
///
    #[account(mut, address = launch.load()?.floor_pool @ LaunchError::BadConfig)]
    pub floor_pool: UncheckedAccount<'info>,
///
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = FLOOR_HOOK_PROGRAM)]
    pub hook_event_authority: UncheckedAccount<'info>,
}

///
fn require_attested(ctx: &Context<ClaimStairPayee>, slot: u8, pending_id: &[u8; 32], payee: &Pubkey, deadline: i64) -> Result<()> {
    require!(get_stack_height() == 1, LaunchError::BadAttestation);
    require!(*payee != Pubkey::default(), LaunchError::BadRecipientConfig);
    require!(Clock::get()?.unix_timestamp <= deadline, LaunchError::Expired);
    let (attestor, cluster) = {
        let c = ctx.accounts.launch_config.load()?;
        (c.attestor, c.cluster)
    };
    let msg = stair_payee_claim_message(&crate::ID, cluster, &ctx.accounts.launch.key(), slot, pending_id, payee, deadline);
    let sysvar = ctx.accounts.instructions.try_borrow_data()?;
    let data = previous_ed25519_data(&sysvar).ok_or(LaunchError::BadAttestation)?;
    require!(attestation_matches(data, &attestor.to_bytes(), &msg), LaunchError::BadAttestation);
    Ok(())
}

///
///
fn resolve_in_hook(ctx: &Context<ClaimStairPayee>, slot: u8, payee: &Pubkey) -> Result<()> {
    let a = &ctx.accounts;
    let mut data = disc::hook::RESOLVE_STAIR_PAYEE.to_vec();
    data.push(slot);
    data.extend_from_slice(payee.as_ref());
    let ix = Instruction {
        program_id: FLOOR_HOOK_PROGRAM,
        accounts: vec![
            AccountMeta::new_readonly(a.launch_authority.key(), true),
            AccountMeta::new_readonly(a.hook_config.key(), false),
            AccountMeta::new(a.floor_pool.key(), false),
            AccountMeta::new_readonly(a.hook_event_authority.key(), false),
            AccountMeta::new_readonly(a.floor_hook.key(), false),
        ],
        data,
    };
    let bump = [ctx.bumps.launch_authority];
    invoke_signed(
        &ix,
        &[
            a.launch_authority.to_account_info(),
            a.hook_config.to_account_info(),
            a.floor_pool.to_account_info(),
            a.hook_event_authority.to_account_info(),
            a.floor_hook.to_account_info(),
        ],
        &[&[LAUNCH_AUTHORITY_SEED, &bump]],
    )?;
    Ok(())
}

///
///
///
pub fn claim_team_payee(ctx: Context<ClaimStairPayee>, slot: u8, payee: Pubkey, deadline: i64) -> Result<()> {



    let genesis = super::close_graduation_rent::is_genesis_launch(&*ctx.accounts.launch.load()?);
    require!(!(genesis && slot == 0), LaunchError::GenesisPayeeUseGenesisClaim);
    let pending = pending_slot_id(&ctx.accounts.floor_pool.to_account_info(), SLOT_KIND_TEAM, slot, LaunchError::PayeeNotPending)?;
    require_attested(&ctx, slot, &pending, &payee, deadline)?;
    resolve_in_hook(&ctx, slot, &payee)?;
    emit_cpi!(StairPayeeClaimed { launch: ctx.accounts.launch.key(), slot, pending_id: pending, payee });
    Ok(())
}

///
///
///
pub fn claim_genesis_payee<'info>(
    ctx: Context<'info, ClaimStairPayee<'info>>,
    payee: Pubkey,
    deadline: i64,
) -> Result<()> {
    let info = ctx.remaining_accounts.first().ok_or(LaunchError::BadConfig)?;
    let (pda, _) = Pubkey::find_program_address(&[GENESIS_SALE_SEED], &crate::ID);
    require_keys_eq!(info.key(), pda, LaunchError::BadConfig);
    require_keys_eq!(*info.owner, crate::ID, LaunchError::BadConfig);
    require!(info.is_writable, LaunchError::BadConfig);
    let gs = AccountLoader::<GenesisSale>::try_from(info)?;
    let (pending, graduated) = {
        let g = gs.load()?;
        require_keys_eq!(g.launch, ctx.accounts.launch.key(), LaunchError::BadConfig);
        require!(g.payee_state == TEAM_PAYEE_PENDING, LaunchError::PayeeNotPending);
        (g.payee.to_bytes(), g.graduated == 1)
    };

    require_attested(&ctx, 0, &pending, &payee, deadline)?;
    {
        let mut g = gs.load_mut()?;
        g.payee_state = TEAM_PAYEE_RESOLVED;
        g.payee = payee;
    }
    if graduated {
        resolve_in_hook(&ctx, 0, &payee)?;
    }
    emit_cpi!(StairPayeeClaimed { launch: ctx.accounts.launch.key(), slot: 0, pending_id: pending, payee });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

///
    fn pool_bytes(t: &PayeeSlots) -> Vec<u8> {
        let mut d = vec![0u8; floor_types::slots::FLOOR_POOL_ACCOUNT_LEN_WITH_SLOTS];
        assert!(t.store(&mut d));
        d
    }

    fn with_pool<R>(data: &mut [u8], owner: Pubkey, f: impl FnOnce(&AccountInfo) -> R) -> R {
        let key = Pubkey::new_unique();
        let mut lamports = 1u64;
        let ai = AccountInfo::new(&key, false, false, &mut lamports, data, &owner, false);
        f(&ai)
    }

    fn code(e: anchor_lang::error::Error) -> u32 {
        match e {
            anchor_lang::error::Error::AnchorError(a) => a.error_code_number,
            other => panic!("not an AnchorError: {other:?}"),
        }
    }

///
///
///
///
///
    #[test]
    fn mr_s_the_slot_reads_are_bounded_and_pending_pays_nothing() {
        use floor_types::slots::*;
        let w = |k: u8, bps: u16| SlotArg { bps, platform: 0, key: [k; 32] };
        let h = |k: u8, bps: u16| SlotArg { bps, platform: 2, key: [k; 32] };
        let mut t = PayeeSlots::default();
        t.fill(SLOT_KIND_CREATOR, &[w(1, 6_000), h(2, 4_000)]);
        t.fill(SLOT_KIND_TEAM, &[w(3, 5_000), h(4, 3_000), w(5, 2_000)]);
        let mut d = pool_bytes(&t);
        let hook = FLOOR_HOOK_PROGRAM;
        let nv = LaunchError::TeamNothingVested as u32 + 6000;
        let np = LaunchError::NotPending as u32 + 6000;
        let pnp = LaunchError::PayeeNotPending as u32 + 6000;
        with_pool(&mut d, hook, |ai| {
            assert_eq!(team_slot_wallet(ai, 0).unwrap(), Some(([3; 32], 5_000)));
            assert_eq!(team_slot_wallet(ai, 1).unwrap(), None, "pending: paid nothing");
            assert_eq!(team_slot_wallet(ai, 2).unwrap(), Some(([5; 32], 2_000)));
            for past in [3u8, 4, 5, 255] {
                assert_eq!(code(team_slot_wallet(ai, past).unwrap_err()), nv, "team slot {past}");
            }
            assert_eq!(pending_slot_id(ai, SLOT_KIND_CREATOR, 1, LaunchError::NotPending).unwrap(), [2; 32]);
            assert_eq!(code(pending_slot_id(ai, SLOT_KIND_CREATOR, 0, LaunchError::NotPending).unwrap_err()), np, "an address slot");
            assert_eq!(code(pending_slot_id(ai, SLOT_KIND_CREATOR, 2, LaunchError::NotPending).unwrap_err()), np, "past the count");
            assert_eq!(pending_slot_id(ai, SLOT_KIND_TEAM, 1, LaunchError::PayeeNotPending).unwrap(), [4; 32]);
            assert_eq!(code(pending_slot_id(ai, SLOT_KIND_TEAM, 2, LaunchError::PayeeNotPending).unwrap_err()), pnp);
            assert_eq!(code(pending_slot_id(ai, SLOT_KIND_TEAM, 5, LaunchError::PayeeNotPending).unwrap_err()), pnp);
        });

        let mut r = t;
        assert!(r.resolve(SLOT_KIND_TEAM, 1, [9; 32]));
        let mut d = pool_bytes(&r);
        with_pool(&mut d, hook, |ai| {
            assert_eq!(team_slot_wallet(ai, 1).unwrap(), Some(([9; 32], 3_000)));
            assert_eq!(code(pending_slot_id(ai, SLOT_KIND_TEAM, 1, LaunchError::PayeeNotPending).unwrap_err()), pnp);
        });

        let bad = LaunchError::BadConfig as u32 + 6000;
        let mut d = pool_bytes(&t);
        with_pool(&mut d, Pubkey::new_unique(), |ai| {
            assert_eq!(code(team_slot_wallet(ai, 0).unwrap_err()), bad);
            assert_eq!(code(pending_slot_id(ai, SLOT_KIND_CREATOR, 1, LaunchError::NotPending).unwrap_err()), bad);
        });
        let mut short = vec![0u8; floor_types::slots::PAYEE_SLOTS_OFFSET];
        with_pool(&mut short, hook, |ai| {
            assert_eq!(code(team_slot_wallet(ai, 0).unwrap_err()), nv);
            assert_eq!(code(pending_slot_id(ai, SLOT_KIND_CREATOR, 0, LaunchError::NotPending).unwrap_err()), np);
        });
    }

    #[test]
    fn mr_s_the_payee_message_is_the_recipients_shape_under_its_own_prefix() {
        let m = stair_payee_claim_message(&crate::ID, 2, &Pubkey::new_from_array([3; 32]), 1, &[4; 32], &Pubkey::new_from_array([5; 32]), 99);
        assert_eq!(m.len(), 162);
        assert_eq!(&m[..24], b"FLOOR_STAIR_PAYEE_CLM_V2");
        assert_eq!(&m[24..56], crate::ID.as_ref());
        assert_eq!(m[56], 2);
        assert_eq!(m[89], 1, "the slot");
        assert_eq!(&m[122..154], &[5u8; 32]);
        let r = crate::instructions::claim_recipient::recipient_claim_message(
            &crate::ID,
            2,
            &Pubkey::new_from_array([3; 32]),
            1,
            &[4; 32],
            &Pubkey::new_from_array([5; 32]),
            99,
        );
        assert_ne!(m, r, "a fee-recipient attestation is never a payee one");
        assert_eq!(m[24..], r[24..], "the same fields after the prefix");
        let other_slot = stair_payee_claim_message(&crate::ID, 2, &Pubkey::new_from_array([3; 32]), 2, &[4; 32], &Pubkey::new_from_array([5; 32]), 99);
        assert_ne!(m, other_slot, "an attestation for one slot is never another's");
    }

///
///
///
///
///
    #[test]
    fn mr_s_the_payee_message_vector_is_pinned_for_the_site() {
        let m = stair_payee_claim_message(
            &crate::ID,
            2,
            &Pubkey::new_from_array([3; 32]),
            1,
            &[4; 32],
            &Pubkey::new_from_array([5; 32]),
            1_900_000_000,
        );
        let hex: String = m.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(
            hex,
            concat!(
                "464c4f4f525f53544149525f50415945455f434c4d5f5632",
                "75c14479e54e7dadf9e032d800a7c484f17d1011ea6a761d30475c9424cca011",
                "02",
                "0303030303030303030303030303030303030303030303030303030303030303",
                "01",
                "0404040404040404040404040404040404040404040404040404040404040404",
                "0505050505050505050505050505050505050505050505050505050505050505",
                "00b33f7100000000",
            )
        );
        assert_eq!(m.len(), 162);
    }

///
///
///
///
///
    #[test]
    fn mr_s_a_recipient_attestation_never_matches_a_payee_claim() {
        let program = Pubkey::new_from_array([0x11; 32]);
        let launch = Pubkey::new_from_array([0x22; 32]);
        let id = [0x33u8; 32];
        let who = Pubkey::new_from_array([0x44; 32]);
        let payee = stair_payee_claim_message(&program, 1, &launch, 2, &id, &who, 1_800_000_000);
        let recipient = crate::instructions::claim_recipient::recipient_claim_message(&program, 1, &launch, 2, &id, &who, 1_800_000_000);
        assert_ne!(payee, recipient);
        assert_eq!(payee[24..], recipient[24..]);
        assert_eq!(&payee[..24], STAIR_PAYEE_CLAIM_PREFIX_V2);
        let other_slot = stair_payee_claim_message(&program, 1, &launch, 3, &id, &who, 1_800_000_000);
        let diff: Vec<usize> = (0..MSG_LEN).filter(|&j| payee[j] != other_slot[j]).collect();
        assert_eq!(diff, vec![89]);
    }
}
