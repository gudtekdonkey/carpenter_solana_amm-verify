//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
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
use floor_types::disc;
use floor_types::slots::{slot_claim_message, PayeeSlots, RECIPIENT_CLAIM_PREFIX_V2, SLOT_CLAIM_MSG_LEN};

use crate::constants::{
    EVENT_AUTHORITY_SEED, FLOOR_HOOK_PROGRAM, HOOK_CONFIG_SEED, LAUNCH_AUTHORITY_SEED, LAUNCH_CONFIG_SEED,
};
use crate::errors::LaunchError;
use crate::events::RecipientClaimed;
use crate::state::{Launch, LaunchConfig};

#[event_cpi]
#[derive(Accounts)]
pub struct ClaimRecipient<'info> {
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
    #[account(mut, address = launch.load()?.floor_pool @ LaunchError::BadConfig)]
    pub floor_pool: UncheckedAccount<'info>,
///
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = FLOOR_HOOK_PROGRAM)]
    pub hook_event_authority: UncheckedAccount<'info>,
}

///
pub const MSG_LEN: usize = SLOT_CLAIM_MSG_LEN;
const _: () = assert!(MSG_LEN == 162);
///
pub const ED25519_DATA_LEN: usize = 112 + MSG_LEN;
///
pub const ED25519_PROGRAM: Pubkey = pubkey!("Ed25519SigVerify111111111111111111111111111");
///
///
///
pub const EXPECTED_OFFSETS: [u16; 7] = [48, u16::MAX, 16, u16::MAX, 112, MSG_LEN as u16, u16::MAX];
///
const TOP_LEVEL: usize = 1;

///
///
///
pub fn pending_slot_id(floor_pool: &AccountInfo, kind: u8, slot: u8, not_pending: LaunchError) -> Result<[u8; 32]> {
    require_keys_eq!(*floor_pool.owner, FLOOR_HOOK_PROGRAM, LaunchError::BadConfig);
    let data = floor_pool.try_borrow_data()?;
    let Some(t) = PayeeSlots::from_account(&data) else { return Err(not_pending.into()) };
    match t.slot(kind, slot) {
        Some(s) if s.is_pending() && s.key != [0u8; 32] => Ok(s.key),
        _ => Err(not_pending.into()),
    }
}

pub fn handler(ctx: Context<ClaimRecipient>, slot: u8, recipient: Pubkey, deadline: i64) -> Result<()> {
    require!(get_stack_height() == TOP_LEVEL, LaunchError::BadAttestation);

    let launch_key = ctx.accounts.launch.key();
    let pending_id =
        pending_slot_id(&ctx.accounts.floor_pool.to_account_info(), floor_types::slots::SLOT_KIND_CREATOR, slot, LaunchError::NotPending)?;
    require!(recipient != Pubkey::default(), LaunchError::BadRecipientConfig);
    require!(Clock::get()?.unix_timestamp <= deadline, LaunchError::Expired);

    let (attestor, cluster) = {
        let c = ctx.accounts.launch_config.load()?;
        (c.attestor, c.cluster)
    };
    let msg = recipient_claim_message(&crate::ID, cluster, &launch_key, slot, &pending_id, &recipient, deadline);
    {
        let sysvar = ctx.accounts.instructions.try_borrow_data()?;
        let data = previous_ed25519_data(&sysvar).ok_or(LaunchError::BadAttestation)?;
        require!(attestation_matches(data, &attestor.to_bytes(), &msg), LaunchError::BadAttestation);
    }


    let mut data = disc::hook::RESOLVE_RECIPIENT.to_vec();
    data.push(slot);
    data.extend_from_slice(recipient.as_ref());


    let ix = Instruction {
        program_id: FLOOR_HOOK_PROGRAM,
        accounts: vec![
            AccountMeta::new_readonly(ctx.accounts.launch_authority.key(), true),
            AccountMeta::new_readonly(ctx.accounts.hook_config.key(), false),
            AccountMeta::new(ctx.accounts.floor_pool.key(), false),
            AccountMeta::new_readonly(ctx.accounts.hook_event_authority.key(), false),
            AccountMeta::new_readonly(ctx.accounts.floor_hook.key(), false),
        ],
        data,
    };
    let bump = [ctx.bumps.launch_authority];
    invoke_signed(
        &ix,
        &[
            ctx.accounts.launch_authority.to_account_info(),
            ctx.accounts.hook_config.to_account_info(),
            ctx.accounts.floor_pool.to_account_info(),
            ctx.accounts.hook_event_authority.to_account_info(),
            ctx.accounts.floor_hook.to_account_info(),
        ],
        &[&[LAUNCH_AUTHORITY_SEED, &bump]],
    )?;

    emit_cpi!(RecipientClaimed { launch: launch_key, slot, pending_recipient_id: pending_id, recipient });
    Ok(())
}

///
///
///
pub fn recipient_claim_message(
    program_id: &Pubkey,
    cluster: u8,
    launch: &Pubkey,
    slot: u8,
    pending_id: &[u8; 32],
    recipient: &Pubkey,
    deadline: i64,
) -> [u8; MSG_LEN] {
    slot_claim_message(
        RECIPIENT_CLAIM_PREFIX_V2,
        &program_id.to_bytes(),
        cluster,
        &launch.to_bytes(),
        slot,
        pending_id,
        &recipient.to_bytes(),
        deadline,
    )
}

fn read_u16(d: &[u8], at: usize) -> Option<u16> {
    let b = d.get(at..at.checked_add(2)?)?;
    Some(u16::from_le_bytes([b[0], b[1]]))
}

///
///
///
///
///
///
///
///
pub fn previous_ed25519_data(sysvar: &[u8]) -> Option<&[u8]> {
    let current = read_u16(sysvar, sysvar.len().checked_sub(2)?)? as usize;
    let prev = current.checked_sub(1)?;
    let count = read_u16(sysvar, 0)? as usize;
    if prev >= count {
        return None;
    }
    let start = read_u16(sysvar, 2 + 2 * prev)? as usize;
    if read_u16(sysvar, start)? != 0 {
        return None;
    }
    let program_at = start + 2;
    if sysvar.get(program_at..program_at + 32)? != ED25519_PROGRAM.as_ref() {
        return None;
    }
    let len = read_u16(sysvar, program_at + 32)? as usize;
    sysvar.get(program_at + 34..program_at + 34 + len)
}

///
pub fn attestation_matches(data: &[u8], attestor: &[u8; 32], msg: &[u8; MSG_LEN]) -> bool {
    if data.len() != ED25519_DATA_LEN || data[0] != 1 || data[1] != 0 {
        return false;
    }
    for (k, want) in EXPECTED_OFFSETS.iter().enumerate() {
        if read_u16(data, 2 + 2 * k) != Some(*want) {
            return false;
        }
    }
    data[16..48] == attestor[..] && data[112..] == msg[..]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good(attestor: &[u8; 32], msg: &[u8; MSG_LEN]) -> Vec<u8> {
        let mut d = vec![1u8, 0];
        for o in EXPECTED_OFFSETS {
            d.extend_from_slice(&o.to_le_bytes());
        }
        d.extend_from_slice(attestor);
        d.extend_from_slice(&[9u8; 64]);
        d.extend_from_slice(msg);
        d
    }

    fn sysvar_with(ixs: &[(Pubkey, usize, Vec<u8>)], current: u16) -> Vec<u8> {
        let mut out = (ixs.len() as u16).to_le_bytes().to_vec();
        out.resize(2 + 2 * ixs.len(), 0);
        for (i, (pid, accounts, data)) in ixs.iter().enumerate() {
            let at = out.len() as u16;
            out[2 + 2 * i..4 + 2 * i].copy_from_slice(&at.to_le_bytes());
            out.extend_from_slice(&(*accounts as u16).to_le_bytes());
            for _ in 0..*accounts {
                out.push(0);
                out.extend_from_slice(&[7u8; 32]);
            }
            out.extend_from_slice(pid.as_ref());
            out.extend_from_slice(&(data.len() as u16).to_le_bytes());
            out.extend_from_slice(data);
        }
        out.extend_from_slice(&current.to_le_bytes());
        out
    }

    fn msg() -> [u8; MSG_LEN] {
        recipient_claim_message(
            &crate::ID,
            2,
            &Pubkey::new_from_array([3; 32]),
            7,
            &[4; 32],
            &Pubkey::new_from_array([5; 32]),
            99,
        )
    }

///
    #[test]
    fn mr_s_message_is_162_bytes_in_the_pinned_order() {
        let m = msg();
        assert_eq!(m.len(), 162);
        assert_eq!(&m[..24], b"FLOOR_RECIPIENT_CLAIM_V2");
        assert_eq!(&m[24..56], crate::ID.as_ref());
        assert_eq!(m[56], 2);
        assert_eq!(&m[57..89], &[3u8; 32]);
        assert_eq!(m[89], 7, "the slot");
        assert_eq!(&m[90..122], &[4u8; 32]);
        assert_eq!(&m[122..154], &[5u8; 32]);
        assert_eq!(&m[154..162], &99i64.to_le_bytes());
        assert_eq!(ED25519_DATA_LEN, 274);
        assert_eq!(EXPECTED_OFFSETS[5], 162);
    }

    #[test]
    fn exact_offsets_pass_and_every_single_mutation_fails() {
        let att = [8u8; 32];
        let d = good(&att, &msg());
        assert_eq!(d.len(), 274);
        assert!(attestation_matches(&d, &att, &msg()));
        for byte in 0..16 {
            let mut bad = d.clone();
            bad[byte] ^= 1;
            assert!(!attestation_matches(&bad, &att, &msg()), "header/offset byte {byte}");
        }
        let mut longer = d.clone();
        longer.extend_from_slice(&[0u8; 32]);
        assert!(!attestation_matches(&longer, &att, &msg()), "trailing bytes: a verified key could sit there");
        assert!(!attestation_matches(&d, &[1u8; 32], &msg()), "another attestor");
        let mut other = msg();
        other[161] ^= 1;
        assert!(!attestation_matches(&d, &att, &other), "another message");
    }

    #[test]
    fn previous_instruction_must_be_the_ed25519_program_with_no_accounts() {
        let data = good(&[8u8; 32], &msg());
        let me = crate::ID;
        let ok = sysvar_with(&[(ED25519_PROGRAM, 0, data.clone()), (me, 3, vec![1, 2])], 1);
        assert_eq!(previous_ed25519_data(&ok), Some(&data[..]));
        let first = sysvar_with(&[(me, 3, vec![1, 2]), (ED25519_PROGRAM, 0, data.clone())], 0);
        assert_eq!(previous_ed25519_data(&first), None, "nothing before index 0");
        let gap = sysvar_with(&[(ED25519_PROGRAM, 0, data.clone()), (me, 0, vec![]), (me, 3, vec![])], 2);
        assert_eq!(previous_ed25519_data(&gap), None, "not immediately before");
        let with_accounts = sysvar_with(&[(ED25519_PROGRAM, 1, data.clone()), (me, 3, vec![])], 1);
        assert_eq!(previous_ed25519_data(&with_accounts), None);
        let other_program = sysvar_with(&[(me, 0, data), (me, 3, vec![])], 1);
        assert_eq!(previous_ed25519_data(&other_program), None);
    }
}
