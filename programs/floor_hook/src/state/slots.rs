//!
//!
//!

use anchor_lang::prelude::*;

pub use floor_types::slots::*;

use crate::errors::HookError;



const _: () = assert!(floor_math::slots::MAX_SLOTS == floor_types::slots::MAX_SLOTS);

///
pub fn read(floor_pool: &AccountInfo) -> Result<PayeeSlots> {
    let data = floor_pool.try_borrow_data()?;
    let t = PayeeSlots::from_account(&data).ok_or(error!(HookError::BadRecipients))?;
    require!(t.version == PAYEE_SLOTS_VERSION, HookError::BadRecipients);
    Ok(t)
}

pub fn write(floor_pool: &AccountInfo, t: &PayeeSlots) -> Result<()> {
    let mut data = floor_pool.try_borrow_mut_data()?;
    require!(t.store(&mut data), HookError::BadRecipients);
    Ok(())
}

///
///
pub fn read_parts(floor_pool: &AccountInfo) -> Result<TeamParts> {
    Ok(TeamParts::from_account(&floor_pool.try_borrow_data()?))
}

///
///
pub fn write_parts(floor_pool: &AccountInfo, p: &TeamParts) -> Result<()> {
    let mut data = floor_pool.try_borrow_mut_data()?;
    require!(p.store(&mut data), HookError::BadRecipients);
    Ok(())
}

///
pub fn check(args: &[SlotArg]) -> Result<()> {
    let v: Vec<floor_math::Slot> =
        args.iter().map(|a| floor_math::Slot { bps: a.bps, platform: a.platform, key: a.key }).collect();
    let code = floor_math::check_slots(&v);
    if code != 0 {
        msg!("BadRecipients code {}", code);
        return err!(HookError::BadRecipients);
    }
    Ok(())
}

///
pub fn slot(t: &PayeeSlots, kind: u8, i: u8) -> Result<PayeeSlot> {
    t.slot(kind, i).copied().ok_or(error!(HookError::BadSlot))
}

///
///
///
///
pub fn resolve_slot(t: &mut PayeeSlots, kind: u8, i: u8, wallet: Pubkey, already: HookError) -> Result<[u8; 32]> {
    require_keys_neq!(wallet, Pubkey::default(), HookError::BadConfig);
    let s = slot(t, kind, i)?;
    if !s.is_pending() || !t.resolve(kind, i, wallet.to_bytes()) {
        return Err(already.into());
    }
    Ok(s.key)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code(e: anchor_lang::error::Error) -> u32 {
        match e {
            anchor_lang::error::Error::AnchorError(a) => a.error_code_number,
            e => panic!("not an anchor error: {e:?}"),
        }
    }

///
///
    #[test]
    fn mr_s_resolve_only_pending_once_nonzero_bit_cleared_platform_kept() {
        for (kind, already) in [(SLOT_KIND_CREATOR, HookError::AlreadyResolved), (SLOT_KIND_TEAM, HookError::StairPayeeResolved)] {
            let mut t = PayeeSlots::default();
            let args = [
                SlotArg { bps: 4_000, platform: 0, key: [1; 32] },
                SlotArg { bps: 3_000, platform: 2, key: [2; 32] },
                SlotArg { bps: 3_000, platform: 4, key: [3; 32] },
            ];
            t.fill(kind, &args);
            let other = if kind == SLOT_KIND_CREATOR { SLOT_KIND_TEAM } else { SLOT_KIND_CREATOR };
            t.fill(other, &args);
            let before = t;
            let mask = |t: &PayeeSlots| if kind == SLOT_KIND_CREATOR { t.creator_pending_mask } else { t.team_pending_mask };
            assert_eq!(mask(&t), 0b110);
            let wallet = Pubkey::new_from_array([9; 32]);
            let err = already;
            let already = 7000 + already as u32;
            let zero = resolve_slot(&mut t, kind, 1, Pubkey::default(), err).unwrap_err();
            assert_eq!(code(zero), 7000 + HookError::BadConfig as u32, "never the zero key");
            assert_eq!(code(resolve_slot(&mut t, kind, 0, wallet, err).unwrap_err()), already, "an address slot");
            assert_eq!(code(resolve_slot(&mut t, kind, 3, wallet, err).unwrap_err()), 7000 + HookError::BadSlot as u32);
            assert_eq!(t, before, "a refusal writes nothing");
            assert_eq!(resolve_slot(&mut t, kind, 1, wallet, err).unwrap(), [2; 32], "the pending id comes back");
            assert_eq!(mask(&t), 0b100, "only slot 1's bit cleared");
            let s1 = *t.slot(kind, 1).unwrap();
            assert_eq!((s1.key, s1.state, s1.platform, s1.bps), ([9; 32], SLOT_STATE_RESOLVED, 2, 3_000));
            assert_eq!(s1.wallet(), Some([9; 32]));
            assert_eq!(code(resolve_slot(&mut t, kind, 1, Pubkey::new_from_array([8; 32]), err).unwrap_err()), already, "once");
            assert_eq!(t.list(other), before.list(other), "the other list never moves");
            assert_eq!(*t.slot(kind, 2).unwrap(), *before.slot(kind, 2).unwrap(), "slot 2 still pending");
        }
    }

///
///
///
    #[test]
    fn mr_s_a_slot_may_resolve_to_another_slots_wallet() {
        for (kind, already) in [(SLOT_KIND_CREATOR, HookError::AlreadyResolved), (SLOT_KIND_TEAM, HookError::StairPayeeResolved)] {
            let mut t = PayeeSlots::default();
            let args = [
                SlotArg { bps: 5_000, platform: 0, key: [1; 32] },
                SlotArg { bps: 2_500, platform: 1, key: [2; 32] },
                SlotArg { bps: 2_500, platform: 3, key: [3; 32] },
            ];
            t.fill(SLOT_KIND_CREATOR, &args);
            t.fill(SLOT_KIND_TEAM, &args);
            let same = Pubkey::new_from_array([1; 32]);
            assert_eq!(resolve_slot(&mut t, kind, 1, same, already).unwrap(), [2; 32], "slot 0's own wallet");
            assert_eq!(resolve_slot(&mut t, kind, 2, same, already).unwrap(), [3; 32], "twice in one list");
            assert!(t.holds_wallet(kind, &[1; 32]));
            assert_eq!(t.slot(kind, 1).unwrap().wallet(), t.slot(kind, 2).unwrap().wallet());
        }
    }
}

///
///
#[inline(never)]
pub fn read_slot(floor_pool: &AccountInfo, kind: u8, i: u8) -> Result<PayeeSlot> {
    slot(&read(floor_pool)?, kind, i)
}
