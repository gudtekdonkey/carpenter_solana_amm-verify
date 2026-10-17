use anchor_lang::prelude::*;

use crate::state::CodePin;

#[event]
pub struct TimelockInitialized {
    pub proposer: Pubkey,
    pub delay: u64,
}

#[event]
pub struct OperationScheduled {
    pub operation: Pubkey,
    pub id: [u8; 32],
///
    pub action: Vec<u8>,
    pub ready_at: i64,
}

#[event]
pub struct OperationExecuted {
    pub operation: Pubkey,
    pub id: [u8; 32],
}

#[event]
pub struct OperationCancelled {
    pub operation: Pubkey,
    pub id: [u8; 32],
    pub by: Pubkey,
}

#[event]
pub struct DelayChanged {
    pub previous: u64,
    pub delay: u64,
}

#[event]
pub struct ProposerChanged {
    pub previous: Pubkey,
    pub proposer: Pubkey,
///
    pub accepted: bool,
}

#[event]
pub struct GuardianChanged {
    pub previous: Pubkey,
    pub guardian: Pubkey,
}

///
///
///
#[event]
pub struct CodePinned {
    pub operation: Pubkey,
    pub id: [u8; 32],
    pub pin: CodePin,
}
