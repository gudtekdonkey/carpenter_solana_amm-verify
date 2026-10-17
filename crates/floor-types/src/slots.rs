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

use crate::twap::FLOOR_POOL_ACCOUNT_LEN_WITH_TWAP;

///
pub const MAX_SLOTS: usize = 5;
///
pub const SLOTS_TOTAL_BPS: u16 = 10_000;
///
pub const MAX_SLOT_PLATFORM: u8 = 4;

///
pub const PAYEE_SLOTS_OFFSET: usize = FLOOR_POOL_ACCOUNT_LEN_WITH_TWAP;
///
pub const PAYEE_SLOT_SIZE: usize = 56;
///
pub const PAYEE_SLOTS_SIZE: usize = 568;
///
pub const FLOOR_POOL_ACCOUNT_LEN_WITH_SLOTS: usize = PAYEE_SLOTS_OFFSET + PAYEE_SLOTS_SIZE;
///
pub const PAYEE_SLOTS_VERSION: u8 = 1;

///
pub const SLOT_STATE_WALLET: u8 = 0;
///
pub const SLOT_STATE_PENDING: u8 = 1;
///
pub const SLOT_STATE_RESOLVED: u8 = 2;

///
pub const SLOT_KIND_CREATOR: u8 = 0;
pub const SLOT_KIND_TEAM: u8 = 1;

///
///
pub const RECIPIENT_CLAIM_PREFIX_V2: &[u8; 24] = b"FLOOR_RECIPIENT_CLAIM_V2";
pub const STAIR_PAYEE_CLAIM_PREFIX_V2: &[u8; 24] = b"FLOOR_STAIR_PAYEE_CLM_V2";
///
pub const SLOT_CLAIM_MSG_LEN: usize = 24 + 32 + 1 + 32 + 1 + 32 + 32 + 8;

///
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Pod, Zeroable)]
pub struct PayeeSlot {
///
    pub key: [u8; 32],
///
    pub bps: u16,
///
    pub platform: u8,
///
    pub state: u8,
    pub pad: [u8; 4],
///
    pub paid: u64,
///
///
    pub tokens_paid: u64,
}

const _: () = assert!(core::mem::size_of::<PayeeSlot>() == PAYEE_SLOT_SIZE);
const _: () = assert!(core::mem::offset_of!(PayeeSlot, bps) == 32);
const _: () = assert!(core::mem::offset_of!(PayeeSlot, paid) == 40);
const _: () = assert!(core::mem::offset_of!(PayeeSlot, tokens_paid) == 48);

impl PayeeSlot {
///
    pub fn wallet(&self) -> Option<[u8; 32]> {
        (self.state != SLOT_STATE_PENDING && self.key != [0u8; 32]).then_some(self.key)
    }

    pub fn is_pending(&self) -> bool {
        self.state == SLOT_STATE_PENDING
    }
}

///
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Pod, Zeroable)]
pub struct PayeeSlots {
///
    pub version: u8,
///
    pub creator_count: u8,
///
    pub team_count: u8,
///
    pub creator_pending_mask: u8,
///
    pub team_pending_mask: u8,
    pub pad: [u8; 3],
    pub creator: [PayeeSlot; MAX_SLOTS],
    pub team: [PayeeSlot; MAX_SLOTS],
}

const _: () = assert!(core::mem::size_of::<PayeeSlots>() == PAYEE_SLOTS_SIZE);
const _: () = assert!(core::mem::offset_of!(PayeeSlots, creator) == 8);
const _: () = assert!(core::mem::offset_of!(PayeeSlots, team) == 8 + 5 * PAYEE_SLOT_SIZE);
const _: () = assert!(FLOOR_POOL_ACCOUNT_LEN_WITH_SLOTS == 1_576);

impl PayeeSlots {
///
    pub fn from_account(data: &[u8]) -> Option<Self> {
        data.get(PAYEE_SLOTS_OFFSET..FLOOR_POOL_ACCOUNT_LEN_WITH_SLOTS).map(bytemuck::pod_read_unaligned)
    }

///
    pub fn store(&self, data: &mut [u8]) -> bool {
        match data.get_mut(PAYEE_SLOTS_OFFSET..FLOOR_POOL_ACCOUNT_LEN_WITH_SLOTS) {
            Some(d) => {
                d.copy_from_slice(bytemuck::bytes_of(self));
                true
            }
            None => false,
        }
    }

///
///
///
    pub fn list(&self, kind: u8) -> Option<(&[PayeeSlot; MAX_SLOTS], u8)> {
        match kind {
            SLOT_KIND_CREATOR => Some((&self.creator, self.creator_count.min(MAX_SLOTS as u8))),
            SLOT_KIND_TEAM => Some((&self.team, self.team_count.min(MAX_SLOTS as u8))),
            _ => None,
        }
    }

///
    pub fn slot(&self, kind: u8, i: u8) -> Option<&PayeeSlot> {
        let (l, n) = self.list(kind)?;
        (i < n).then(|| &l[i as usize])
    }

    pub fn slot_mut(&mut self, kind: u8, i: u8) -> Option<&mut PayeeSlot> {
        let n = match kind {
            SLOT_KIND_CREATOR => self.creator_count,
            SLOT_KIND_TEAM => self.team_count,
            _ => return None,
        }
        .min(MAX_SLOTS as u8);
        if i >= n {
            return None;
        }
        Some(match kind {
            SLOT_KIND_CREATOR => &mut self.creator[i as usize],
            _ => &mut self.team[i as usize],
        })
    }

///
///
    pub fn fill(&mut self, kind: u8, args: &[SlotArg]) {
        let mut mask = 0u8;
        let mut list = [PayeeSlot::default(); MAX_SLOTS];
        for (i, a) in args.iter().take(MAX_SLOTS).enumerate() {
            let pending = a.platform != 0;
            if pending {
                mask |= 1 << i;
            }
            list[i] = PayeeSlot {
                key: a.key,
                bps: a.bps,
                platform: a.platform,
                state: if pending { SLOT_STATE_PENDING } else { SLOT_STATE_WALLET },
                ..PayeeSlot::default()
            };
        }
        let n = args.len().min(MAX_SLOTS) as u8;
        match kind {
            SLOT_KIND_CREATOR => {
                self.creator = list;
                self.creator_count = n;
                self.creator_pending_mask = mask;
            }
            _ => {
                self.team = list;
                self.team_count = n;
                self.team_pending_mask = mask;
            }
        }
        self.version = PAYEE_SLOTS_VERSION;
    }

///
///
    pub fn resolve(&mut self, kind: u8, i: u8, wallet: [u8; 32]) -> bool {
        let Some(s) = self.slot_mut(kind, i) else { return false };
        if s.state != SLOT_STATE_PENDING || wallet == [0u8; 32] {
            return false;
        }
        s.key = wallet;
        s.state = SLOT_STATE_RESOLVED;
        match kind {
            SLOT_KIND_CREATOR => self.creator_pending_mask &= !(1 << i),
            _ => self.team_pending_mask &= !(1 << i),
        }
        true
    }

///
    pub fn holds_wallet(&self, kind: u8, wallet: &[u8; 32]) -> bool {
        self.list(kind)
            .map(|(l, n)| l[..n as usize].iter().any(|s| s.wallet().as_ref() == Some(wallet)))
            .unwrap_or(false)
    }
}

impl PayeeSlots {
///
    pub fn team_bps(&self) -> ([u16; MAX_SLOTS], u8) {
        let n = self.team_count.min(MAX_SLOTS as u8);
        let mut b = [0u16; MAX_SLOTS];
        for (i, s) in self.team[..n as usize].iter().enumerate() {
            b[i] = s.bps;
        }
        (b, n)
    }
}



///
pub const TEAM_PARTS_OFFSET: usize = FLOOR_POOL_ACCOUNT_LEN_WITH_SLOTS;
///
pub const TEAM_PARTS_SIZE: usize = 152;
///
pub const FLOOR_POOL_ACCOUNT_LEN_WITH_PARTS: usize = TEAM_PARTS_OFFSET + TEAM_PARTS_SIZE;
///
pub const OUT_BITS_PER_STEP: usize = MAX_SLOTS;

///
///
///
///
///
///
///
///
///
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Pod, Zeroable)]
pub struct TeamParts {
    pub out_mask: [u64; 2],
    pub part_tokens: u64,
    pub part_quote: u64,
    pub token_credit: [u64; MAX_SLOTS],
    pub quote_credit: [u64; MAX_SLOTS],
    pub pending_ask: [u64; MAX_SLOTS],
}

const _: () = assert!(core::mem::size_of::<TeamParts>() == TEAM_PARTS_SIZE);
const _: () = assert!(FLOOR_POOL_ACCOUNT_LEN_WITH_PARTS == 1_728);
const _: () = assert!(16 * OUT_BITS_PER_STEP <= 128);

impl TeamParts {
///
    pub fn from_account(data: &[u8]) -> Self {
        data.get(TEAM_PARTS_OFFSET..FLOOR_POOL_ACCOUNT_LEN_WITH_PARTS).map(bytemuck::pod_read_unaligned).unwrap_or_default()
    }

///
    pub fn store(&self, data: &mut [u8]) -> bool {
        match data.get_mut(TEAM_PARTS_OFFSET..FLOOR_POOL_ACCOUNT_LEN_WITH_PARTS) {
            Some(d) => {
                d.copy_from_slice(bytemuck::bytes_of(self));
                true
            }
            None => false,
        }
    }

    fn mask(&self) -> u128 {
        self.out_mask[0] as u128 | (self.out_mask[1] as u128) << 64
    }

///
    pub fn out_of(&self, i: usize) -> u8 {
        if i >= 16 {
            return 0;
        }
        ((self.mask() >> (OUT_BITS_PER_STEP * i)) & 0x1f) as u8
    }

///
    pub fn mark_out(&mut self, i: usize, s: usize) {
        if i >= 16 || s >= MAX_SLOTS {
            return;
        }
        let m = self.mask() | 1u128 << (OUT_BITS_PER_STEP * i + s);
        self.out_mask = [m as u64, (m >> 64) as u64];
    }

///
    pub fn any_out(&self) -> bool {
        self.mask() != 0
    }

///
    pub fn pending_slots(&self) -> u64 {
        self.pending_ask.iter().fold(0u64, |a, &v| a.saturating_add(v))
    }
}



///
///
///
#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SlotArg {
    pub bps: u16,
    pub platform: u8,
    pub key: [u8; 32],
}

pub const SLOT_ARG_LEN: usize = 35;

///
///
///
#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct SlotsArg {
    pub creator: Vec<SlotArg>,
    pub team: Vec<SlotArg>,
}

///
pub fn slot_claim_message(
    prefix: &[u8; 24],
    program: &[u8; 32],
    cluster: u8,
    launch: &[u8; 32],
    slot: u8,
    pending_id: &[u8; 32],
    recipient: &[u8; 32],
    deadline: i64,
) -> [u8; SLOT_CLAIM_MSG_LEN] {
    let mut m = [0u8; SLOT_CLAIM_MSG_LEN];
    m[0..24].copy_from_slice(prefix);
    m[24..56].copy_from_slice(program);
    m[56] = cluster;
    m[57..89].copy_from_slice(launch);
    m[89] = slot;
    m[90..122].copy_from_slice(pending_id);
    m[122..154].copy_from_slice(recipient);
    m[154..162].copy_from_slice(&deadline.to_le_bytes());
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mr_s_the_tail_sits_after_the_twap_and_is_568_bytes() {
        assert_eq!(PAYEE_SLOTS_OFFSET, 1_008);
        assert_eq!(FLOOR_POOL_ACCOUNT_LEN_WITH_SLOTS, 1_576);
        assert_eq!(core::mem::offset_of!(PayeeSlot, state), 35);
        assert_eq!(SLOT_CLAIM_MSG_LEN, 162);
    }

    #[test]
    fn mr_s_fill_resolve_and_holds() {
        let mut t = PayeeSlots::default();
        let args = [
            SlotArg { bps: 6_000, platform: 0, key: [1; 32] },
            SlotArg { bps: 4_000, platform: 1, key: [9; 32] },
        ];
        t.fill(SLOT_KIND_CREATOR, &args);
        assert_eq!((t.version, t.creator_count, t.creator_pending_mask), (1, 2, 0b10));
        assert!(t.holds_wallet(SLOT_KIND_CREATOR, &[1; 32]));
        assert!(!t.holds_wallet(SLOT_KIND_CREATOR, &[9; 32]), "a pending id is never a wallet");
        assert!(t.slot(SLOT_KIND_CREATOR, 2).is_none());
        assert!(!t.resolve(SLOT_KIND_CREATOR, 0, [7; 32]), "an address slot never resolves");
        assert!(!t.resolve(SLOT_KIND_CREATOR, 1, [0; 32]), "never to the zero key");
        assert!(t.resolve(SLOT_KIND_CREATOR, 1, [7; 32]));
        assert_eq!(t.creator_pending_mask, 0);
        assert!(!t.resolve(SLOT_KIND_CREATOR, 1, [8; 32]), "once");
        assert_eq!(t.creator[1].platform, 1, "the platform is kept, a record");
        let mut acct = vec![0u8; FLOOR_POOL_ACCOUNT_LEN_WITH_SLOTS];
        assert!(t.store(&mut acct));
        assert_eq!(PayeeSlots::from_account(&acct), Some(t));
        assert!(PayeeSlots::from_account(&acct[..1_008]).is_none());
    }

    #[test]
    fn mr_s_a_corrupt_count_above_five_refuses_the_slot_and_never_panics() {

        let mut t = PayeeSlots::default();
        t.fill(SLOT_KIND_CREATOR, &[SlotArg { bps: 10_000, platform: 0, key: [7; 32] }]);
        for kind in [SLOT_KIND_CREATOR, SLOT_KIND_TEAM] {
            t.creator_count = 255;
            t.team_count = 255;
            assert_eq!(t.list(kind).unwrap().1, MAX_SLOTS as u8);
            for i in MAX_SLOTS as u8..=255 {
                assert!(t.slot(kind, i).is_none());
                assert!(t.slot_mut(kind, i).is_none());
                assert!(!t.resolve(kind, i, [9; 32]));
            }
            assert!(t.slot(kind, 4).is_some());
            assert!(!t.holds_wallet(kind, &[9; 32]));
        }
        assert!(t.holds_wallet(SLOT_KIND_CREATOR, &[7; 32]));
    }

    #[test]
    fn mr_s_the_slot_arg_wire_is_35_bytes() {
        let a = SlotArg { bps: 0x0102, platform: 3, key: [0xAB; 32] };
        let b = borsh::to_vec(&a).unwrap();
        assert_eq!(b.len(), SLOT_ARG_LEN);
        assert_eq!(&b[..3], &[0x02, 0x01, 3]);
        let s = SlotsArg { creator: vec![a; 5], team: vec![a; 5] };
        assert_eq!(borsh::to_vec(&s).unwrap().len(), 4 + 5 * 35 + 4 + 5 * 35);
    }

    #[test]
    fn mr_s_the_claim_message_layout() {
        let m = slot_claim_message(RECIPIENT_CLAIM_PREFIX_V2, &[1; 32], 2, &[3; 32], 4, &[5; 32], &[6; 32], 0x0102);
        assert_eq!(&m[..24], b"FLOOR_RECIPIENT_CLAIM_V2");
        assert_eq!((m[56], m[57], m[89], m[90], m[122], m[154], m[155]), (2, 3, 4, 5, 6, 2, 1));
    }

///
///
    #[test]
    fn a15_team_parts_tail_and_out_mask() {
        assert_eq!((TEAM_PARTS_OFFSET, TEAM_PARTS_SIZE, FLOOR_POOL_ACCOUNT_LEN_WITH_PARTS), (1_576, 152, 1_728));
        assert_eq!(core::mem::offset_of!(TeamParts, part_tokens), 16);
        assert_eq!(core::mem::offset_of!(TeamParts, token_credit), 32);
        assert_eq!(core::mem::offset_of!(TeamParts, quote_credit), 72);
        assert_eq!(core::mem::offset_of!(TeamParts, pending_ask), 112);
        let mut short = vec![0u8; FLOOR_POOL_ACCOUNT_LEN_WITH_SLOTS];
        assert_eq!(TeamParts::from_account(&short), TeamParts::default());
        assert!(!TeamParts::default().store(&mut short));
        let mut p = TeamParts::default();
        assert!(!p.any_out());
        for i in 0..16 {
            for s in 0..MAX_SLOTS {
                let mut q = TeamParts::default();
                q.mark_out(i, s);
                assert_eq!(q.out_of(i), 1 << s, "step {i} slot {s}");
                for j in 0..16 {
                    if j != i {
                        assert_eq!(q.out_of(j), 0);
                    }
                }
            }
        }
        p.mark_out(15, 4);
        p.mark_out(3, 0);
        p.mark_out(3, 2);
        p.mark_out(16, 0);
        assert_eq!((p.out_of(15), p.out_of(3), p.out_of(4)), (0b10000, 0b00101, 0));
        p.pending_ask = [1, 2, 3, 0, 4];
        assert_eq!(p.pending_slots(), 10);
        let mut acct = vec![0u8; FLOOR_POOL_ACCOUNT_LEN_WITH_PARTS + 8];
        assert!(p.store(&mut acct));
        assert_eq!(TeamParts::from_account(&acct), p);
        assert!(acct[..TEAM_PARTS_OFFSET].iter().all(|&b| b == 0), "the slots untouched");
    }
}
