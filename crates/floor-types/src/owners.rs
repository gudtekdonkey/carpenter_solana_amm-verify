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

use bytemuck::{Pod, Zeroable};

use crate::layout::{Layout, StairState};

///
pub const STAIR_OWNERS_OFFSET: usize = 8 + <StairState as Layout>::SIZE;
///
pub const STAIR_OWNERS_SIZE: usize = 440;
///
pub const STAIR_STATE_ACCOUNT_LEN_WITH_OWNERS: usize = STAIR_OWNERS_OFFSET + STAIR_OWNERS_SIZE;
///
pub const STAIR_OWNERS_VERSION: u8 = 1;

///
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Pod, Zeroable)]
pub struct StairOwners {
///
///
    pub version: u8,
///
    pub kind: u8,
///
///
///
///
    pub counted: u8,
///
    pub first_step: u8,
///
    pub payee_bps: u16,
    pub pad0: [u8; 2],
///
    pub first_unlock_secs: u32,
///
    pub every_secs: u32,
///
    pub anchor: i64,
///
///
    pub pad1: [u8; 32],
///
    pub pad2: [u8; 32],
///
    pub owner_total: u64,
///
    pub owner_sold: u64,
///
    pub owner_withdrawn: u64,
///
    pub owner_proceeds: u64,
///
    pub owed_total: u64,
///
    pub forfeited: u64,
///
    pub fees_to_floor: u64,
///
///
    pub tips_paid: u64,
///
///
    pub tokens: [u64; 16],
///
///
///
///
///
    pub cut: [u64; 16],
///
///
///
    pub last_tip_at: i64,
///
///
///
///
///
    pub reserved: [u8; 24],
}

const _: () = assert!(core::mem::size_of::<StairOwners>() == STAIR_OWNERS_SIZE);
const _: () = assert!(core::mem::offset_of!(StairOwners, last_tip_at) == 408);
const _: () = assert!(core::mem::offset_of!(StairOwners, reserved) == 416);

impl StairOwners {
///
    pub fn from_account(data: &[u8]) -> Option<Self> {
        if data.len() < STAIR_STATE_ACCOUNT_LEN_WITH_OWNERS {
            return None;
        }
        Some(bytemuck::pod_read_unaligned(&data[STAIR_OWNERS_OFFSET..STAIR_STATE_ACCOUNT_LEN_WITH_OWNERS]))
    }

///
    pub fn store(&self, data: &mut [u8]) -> bool {
        if data.len() < STAIR_STATE_ACCOUNT_LEN_WITH_OWNERS {
            return false;
        }
        data[STAIR_OWNERS_OFFSET..STAIR_STATE_ACCOUNT_LEN_WITH_OWNERS].copy_from_slice(bytemuck::bytes_of(self));
        true
    }

///
    pub fn standing(&self) -> u64 {
        self.tokens.iter().fold(0u64, |a, &t| a.saturating_add(t))
    }

    fn word(&self, i: usize) -> u64 {
        u64::from_le_bytes(self.reserved[i * 8..i * 8 + 8].try_into().unwrap())
    }

    fn set_word(&mut self, i: usize, v: u64) {
        self.reserved[i * 8..i * 8 + 8].copy_from_slice(&v.to_le_bytes());
    }

///
///
///
///
///
///
///
    pub fn held(&self) -> u64 {
        self.word(0)
    }

    pub fn set_held(&mut self, v: u64) {
        self.set_word(0, v)
    }

///
///
///
    pub fn reserve_fed(&self) -> u64 {
        self.word(1)
    }

    pub fn set_reserve_fed(&mut self, v: u64) {
        self.set_word(1, v)
    }

///
///
    pub fn clamped(&self) -> u64 {
        self.word(2)
    }

    pub fn add_clamped(&mut self, v: u64) {
        let c = self.word(2).saturating_add(v);
        self.set_word(2, c)
    }

///
///
///
///
    pub fn conserved(&self) -> bool {
        let sum = self.standing() as u128
            + self.owner_sold as u128
            + self.owner_withdrawn as u128
            + self.held() as u128
            + self.clamped() as u128;
        sum == self.owner_total as u128
    }
}



///
///
///
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StairOwnerArg {
    pub kind: u8,
    pub counted: u8,
    pub payee_bps: u16,
    pub first_unlock_secs: u32,
    pub every_secs: u32,
///
    pub anchor: i64,
///
    pub owner_tokens: u64,
///
///
///
    pub payee: [u8; 32],
    pub payee_pending_id: [u8; 32],
///
///
///
    pub payee_platform: u8,
}

pub const STAIR_OWNER_ARG_LEN: usize = 1 + 1 + 2 + 4 + 4 + 8 + 8 + 32 + 32 + 1;

impl StairOwnerArg {
    pub fn to_bytes(&self) -> [u8; STAIR_OWNER_ARG_LEN] {
        let mut b = [0u8; STAIR_OWNER_ARG_LEN];
        b[0] = self.kind;
        b[1] = self.counted;
        b[2..4].copy_from_slice(&self.payee_bps.to_le_bytes());
        b[4..8].copy_from_slice(&self.first_unlock_secs.to_le_bytes());
        b[8..12].copy_from_slice(&self.every_secs.to_le_bytes());
        b[12..20].copy_from_slice(&self.anchor.to_le_bytes());
        b[20..28].copy_from_slice(&self.owner_tokens.to_le_bytes());
        b[28..60].copy_from_slice(&self.payee);
        b[60..92].copy_from_slice(&self.payee_pending_id);
        b[92] = self.payee_platform;
        b
    }
}

#[allow(clippy::result_unit_err)]
pub fn stair_owner_from_trailing(bytes: &[u8]) -> Result<Option<StairOwnerArg>, ()> {
    match bytes.len() {
        0 => Ok(None),
        n if n < STAIR_OWNER_ARG_LEN => Err(()),
        _ => {
            let u16_at = |i: usize| u16::from_le_bytes([bytes[i], bytes[i + 1]]);
            let u32_at = |i: usize| u32::from_le_bytes(bytes[i..i + 4].try_into().unwrap());
            let u64_at = |i: usize| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
            let mut payee = [0u8; 32];
            payee.copy_from_slice(&bytes[28..60]);
            let mut pending = [0u8; 32];
            pending.copy_from_slice(&bytes[60..92]);
            Ok(Some(StairOwnerArg {
                kind: bytes[0],
                counted: bytes[1],
                payee_bps: u16_at(2),
                first_unlock_secs: u32_at(4),
                every_secs: u32_at(8),
                anchor: u64_at(12) as i64,
                owner_tokens: u64_at(20),
                payee,
                payee_pending_id: pending,
                payee_platform: bytes[92],
            }))
        }
    }
}

///
///
///
pub const STAIR_OWNER_KIND_ARG_LEN: usize = 1;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{STAIR_OWNER_TEAM_BUY, STAIR_OWNER_TEAM};

    #[test]
    fn the_tail_sits_after_the_stair_state_and_is_440_bytes() {
        assert_eq!(STAIR_OWNERS_OFFSET, 592);
        assert_eq!(STAIR_STATE_ACCOUNT_LEN_WITH_OWNERS, 1_032);
        assert_eq!(core::mem::offset_of!(StairOwners, anchor), 16);
        assert_eq!(core::mem::offset_of!(StairOwners, pad1), 24);
        assert_eq!(core::mem::offset_of!(StairOwners, owner_total), 88);
        assert_eq!(core::mem::offset_of!(StairOwners, tokens), 152);
        assert_eq!(core::mem::offset_of!(StairOwners, cut), 280);
        assert_eq!(core::mem::offset_of!(StairOwners, last_tip_at), 408);
        assert_eq!(core::mem::offset_of!(StairOwners, reserved), 416);
        assert_eq!(core::mem::size_of::<StairOwners>(), 440, "the tail never grows: last_tip_at came out of reserved");
        assert_eq!((STAIR_OWNER_TEAM, STAIR_OWNER_TEAM_BUY), (1, 2));
    }

    #[test]
    fn a_short_account_has_no_tail_and_the_arg_round_trips() {
        assert!(StairOwners::from_account(&[0u8; 592]).is_none());
        let mut data = vec![0u8; STAIR_STATE_ACCOUNT_LEN_WITH_OWNERS];
        let mut o = StairOwners { kind: 1, payee_bps: 4_000, ..Default::default() };
        o.tokens[3] = 7;
        o.last_tip_at = 1_790_000_000;
        assert!(o.store(&mut data));
        assert_eq!(&data[STAIR_OWNERS_OFFSET + 408..STAIR_OWNERS_OFFSET + 416], &1_790_000_000i64.to_le_bytes());
        assert_eq!(StairOwners::from_account(&data).unwrap(), o);
        assert_eq!(o.standing(), 7);
        let a = StairOwnerArg {
            kind: 2,
            counted: 1,
            payee_bps: 8_000,
            first_unlock_secs: 90 * 86_400,
            every_secs: 0,
            anchor: -5,
            owner_tokens: 40_000_000_000_000,
            payee: [9u8; 32],
            payee_pending_id: [8u8; 32],
            payee_platform: 3,
        };

        assert_eq!(STAIR_OWNER_ARG_LEN, 93);
        assert_eq!(a.to_bytes()[92], 3);
        assert_eq!(stair_owner_from_trailing(&a.to_bytes()), Ok(Some(a)));
        assert_eq!(stair_owner_from_trailing(&a.to_bytes()[..92]), Err(()), "the 92-byte wire is refused");
        assert_eq!(stair_owner_from_trailing(&[]), Ok(None));
        assert_eq!(stair_owner_from_trailing(&[1, 2, 3]), Err(()));
    }
}
