use anchor_lang::prelude::*;

///
#[error_code]
pub enum TimelockError {
    #[msg("signer is not allowed to do this")]
    Unauthorized,
    #[msg("the delay can only be raised, never lowered")]
    DelayDecrease,
    #[msg("the delay can never exceed 30 days")]
    DelayAboveMax,
    #[msg("the operation is not ready yet")]
    NotReady,
    #[msg("the action is malformed (zero key, too large, or targets this program)")]
    BadAction,
    #[msg("the accounts passed do not match the stored operation")]
    AccountMismatch,

    #[msg("the buffer's authority is not the timelock's authority PDA")]
    BufferAuthorityNotTimelock,
    #[msg("the buffer's bytes changed since the upgrade was scheduled")]
    CodeHashMismatch,
    #[msg("the target program was redeployed since it was scheduled (or in the scheduling slot)")]
    ProgramRedeployed,
    #[msg("an account the code pin reads was not passed")]
    PinAccountMissing,
    #[msg("the instruction moves code the timelock cannot pin (loader v4, Migrate, an unknown loader)")]
    CodeNotPinnable,
}
