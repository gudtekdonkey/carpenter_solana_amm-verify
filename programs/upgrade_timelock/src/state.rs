use anchor_lang::prelude::*;

use crate::constants::{
    BUFFER_HEADER, FIXED_CODE_OWNERS, INITIAL_DELAY, LOADER_V3, LOADER_V4, MAX_DELAY, MAX_INVOKE_ACCOUNTS, MAX_INVOKE_DATA,
};
use crate::errors::TimelockError;

///
#[account]
pub struct Timelock {
    pub bump: u8,
    pub authority_bump: u8,
///
    pub proposer: Pubkey,
///
    pub pending_proposer: Pubkey,
///
    pub guardian: Pubkey,
///
    pub delay: u64,
}

impl Timelock {
    pub const LEN: usize = 8 + 1 + 1 + 32 * 3 + 8;
}

///
#[account]
pub struct Operation {
    pub bump: u8,
    pub id: [u8; 32],
///
    pub payer: Pubkey,
    pub ready_at: i64,
    pub action: Action,
///
    pub pin: CodePin,
}

impl Operation {
    pub fn space(action: &Action) -> usize {
        8 + 1 + 32 + 32 + 8 + action.encoded_len() + CodePin::MAX_LEN
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
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum CodePin {
///
///
    None,
    Buffer { buffer: Pubkey, code_hash: [u8; 32] },
    Program { program_data: Pubkey, deploy_slot: u64 },
}

impl CodePin {
    pub const MAX_LEN: usize = 1 + 32 + 32;
}

///
///
///
///
///
///
pub fn loader_v3_buffer_index(data: &[u8], n_accounts: usize) -> Result<Option<usize>> {
    require!(data.len() >= 4, TimelockError::BadAction);
    let tag = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let index = match tag {
        0 | 1 | 4 | 5 | 6 | 7 | 9 => return Ok(None),
        2 => 3,
        3 => 2,
        _ => return err!(TimelockError::CodeNotPinnable),
    };
    require!(index < n_accounts, TimelockError::BadAction);
    Ok(Some(index))
}

///
pub fn buffer_authority(data: &[u8]) -> Option<Pubkey> {
    if data.len() < BUFFER_HEADER || data[0..4] != [1, 0, 0, 0] || data[4] != 1 {
        return None;
    }
    Some(Pubkey::new_from_array(data[5..37].try_into().unwrap()))
}

///
pub fn buffer_code_hash(data: &[u8]) -> [u8; 32] {
    solana_sha256_hasher::hash(&data[BUFFER_HEADER.min(data.len())..]).to_bytes()
}

///
pub fn program_data_address(data: &[u8]) -> Option<Pubkey> {
    if data.len() < 36 || data[0..4] != [2, 0, 0, 0] {
        return None;
    }
    Some(Pubkey::new_from_array(data[4..36].try_into().unwrap()))
}

///
pub fn program_data_slot(data: &[u8]) -> Option<u64> {
    if data.len() < 12 || data[0..4] != [3, 0, 0, 0] {
        return None;
    }
    Some(u64::from_le_bytes(data[4..12].try_into().unwrap()))
}

///
pub fn check_buffer(owner: &Pubkey, data: &[u8], authority: &Pubkey, code_hash: &[u8; 32]) -> Result<()> {
    require!(
        *owner == LOADER_V3 && buffer_authority(data) == Some(*authority),
        TimelockError::BufferAuthorityNotTimelock
    );
    require!(buffer_code_hash(data) == *code_hash, TimelockError::CodeHashMismatch);
    Ok(())
}

///
pub fn pin_for<'a>(
    action: &Action,
    authority: &Pubkey,
    slot: u64,
    find: impl Fn(&Pubkey) -> Option<(Pubkey, &'a [u8])>,
) -> Result<CodePin> {
    let Action::Invoke { program_id, accounts, data } = action else { return Ok(CodePin::None) };
    if *program_id == LOADER_V3 {
        let Some(i) = loader_v3_buffer_index(data, accounts.len())? else { return Ok(CodePin::None) };
        let buffer = accounts[i].pubkey;
        let (owner, bytes) = find(&buffer).ok_or(TimelockError::PinAccountMissing)?;
        require!(
            owner == LOADER_V3 && buffer_authority(bytes) == Some(*authority),
            TimelockError::BufferAuthorityNotTimelock
        );
        return Ok(CodePin::Buffer { buffer, code_hash: buffer_code_hash(bytes) });
    }
    require_keys_neq!(*program_id, LOADER_V4, TimelockError::CodeNotPinnable);
    let (owner, bytes) = find(program_id).ok_or(TimelockError::PinAccountMissing)?;
    if FIXED_CODE_OWNERS.contains(&owner) {
        return Ok(CodePin::None);
    }
    require_keys_eq!(owner, LOADER_V3, TimelockError::CodeNotPinnable);
    let program_data = program_data_address(bytes).ok_or(TimelockError::CodeNotPinnable)?;
    let (pd_owner, pd) = find(&program_data).ok_or(TimelockError::PinAccountMissing)?;
    require_keys_eq!(pd_owner, LOADER_V3, TimelockError::CodeNotPinnable);
    let deploy_slot = program_data_slot(pd).ok_or(TimelockError::CodeNotPinnable)?;

    require!(deploy_slot < slot, TimelockError::ProgramRedeployed);
    Ok(CodePin::Program { program_data, deploy_slot })
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub struct OpMeta {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum Action {
///
    UpdateDelay { delay: u64 },
///
    SetProposer { proposer: Pubkey },
///
    SetGuardian { guardian: Pubkey },
///
///
    Invoke { program_id: Pubkey, accounts: Vec<OpMeta>, data: Vec<u8> },
}

impl Action {
    pub fn encoded_len(&self) -> usize {
        1 + match self {
            Action::UpdateDelay { .. } => 8,
            Action::SetProposer { .. } | Action::SetGuardian { .. } => 32,
            Action::Invoke { accounts, data, .. } => 32 + 4 + accounts.len() * 34 + 4 + data.len(),
        }
    }
}

///
pub fn check_delay(current: u64, requested: u64) -> Result<()> {
    require!(requested >= current, TimelockError::DelayDecrease);
    require!(requested <= MAX_DELAY, TimelockError::DelayAboveMax);
    Ok(())
}

///
///
pub fn check_action(action: &Action, delay: u64, this: &Pubkey, unsignable: &[Pubkey]) -> Result<()> {
    match action {
        Action::UpdateDelay { delay: d } => check_delay(delay, *d),
        Action::SetProposer { proposer } => {
            require_keys_neq!(*proposer, Pubkey::default(), TimelockError::BadAction);
            require!(!unsignable.contains(proposer), TimelockError::BadAction);
            Ok(())
        }
        Action::SetGuardian { guardian } => {
            require!(!unsignable.contains(guardian), TimelockError::BadAction);
            Ok(())
        }
        Action::Invoke { program_id, accounts, data } => {
            require_keys_neq!(*program_id, *this, TimelockError::BadAction);
            require_keys_neq!(*program_id, Pubkey::default(), TimelockError::BadAction);
            require!(accounts.len() <= MAX_INVOKE_ACCOUNTS, TimelockError::BadAction);
            require!(data.len() <= MAX_INVOKE_DATA, TimelockError::BadAction);
            Ok(())
        }
    }
}

const _: () = assert!(INITIAL_DELAY <= MAX_DELAY);
const _: () = assert!(CodePin::MAX_LEN == 65);

#[cfg(test)]
mod tests {
    use super::*;

    fn name(r: Result<()>) -> String {
        match r {
            Err(anchor_lang::error::Error::AnchorError(e)) => e.error_name,
            other => format!("{other:?}"),
        }
    }

///
    #[test]
    fn the_delay_only_grows_and_never_past_thirty_days() {
        assert!(check_delay(600, 600).is_ok());
        assert!(check_delay(600, 7 * 86_400).is_ok());
        assert!(check_delay(600, MAX_DELAY).is_ok());
        assert_eq!(name(check_delay(600, 599)), "DelayDecrease");
        assert_eq!(name(check_delay(7 * 86_400, 86_400)), "DelayDecrease");
        assert_eq!(name(check_delay(600, MAX_DELAY + 1)), "DelayAboveMax");
    }

///
    #[test]
    fn an_action_is_refused_by_name_when_malformed() {
        let this = Pubkey::new_unique();
        let pdas = [Pubkey::new_unique(), Pubkey::new_unique()];
        let invoke = |program_id, n, d| Action::Invoke {
            program_id,
            accounts: vec![OpMeta { pubkey: Pubkey::new_unique(), is_signer: false, is_writable: false }; n],
            data: vec![0; d],
        };
        assert!(check_action(&invoke(Pubkey::new_unique(), 7, 4), 600, &this, &pdas).is_ok());
        assert_eq!(name(check_action(&invoke(this, 1, 1), 600, &this, &pdas)), "BadAction");
        assert_eq!(name(check_action(&invoke(Pubkey::new_unique(), 33, 1), 600, &this, &pdas)), "BadAction");
        assert_eq!(name(check_action(&invoke(Pubkey::new_unique(), 1, 1025), 600, &this, &pdas)), "BadAction");
        let zero = Action::SetProposer { proposer: Pubkey::default() };
        assert_eq!(name(check_action(&zero, 600, &this, &pdas)), "BadAction");
        let pda = Action::SetProposer { proposer: pdas[1] };
        assert_eq!(name(check_action(&pda, 600, &this, &pdas)), "BadAction");
        assert!(check_action(&Action::SetGuardian { guardian: Pubkey::default() }, 600, &this, &pdas).is_ok());
        assert_eq!(name(check_action(&Action::UpdateDelay { delay: 1 }, 600, &this, &pdas)), "DelayDecrease");
    }

    fn buffer(auth: Option<Pubkey>, code: &[u8]) -> Vec<u8> {
        let mut d = vec![1, 0, 0, 0];
        match auth {
            Some(a) => {
                d.push(1);
                d.extend_from_slice(a.as_ref());
            }
            None => d.extend_from_slice(&[0; 33]),
        }
        d.extend_from_slice(code);
        d
    }
    fn loader_ix(tag: u32, n: usize) -> Action {
        Action::Invoke {
            program_id: LOADER_V3,
            accounts: (0..n).map(|_| OpMeta { pubkey: Pubkey::new_unique(), is_signer: false, is_writable: true }).collect(),
            data: tag.to_le_bytes().to_vec(),
        }
    }

///
///
    #[test]
    fn the_loader_tags_that_install_code_are_pinned() {
        assert_eq!(loader_v3_buffer_index(&3u32.to_le_bytes(), 7).unwrap(), Some(2));
        assert_eq!(loader_v3_buffer_index(&2u32.to_le_bytes(), 8).unwrap(), Some(3));
        for free in [0u32, 1, 4, 5, 6, 7, 9] {
            assert_eq!(loader_v3_buffer_index(&free.to_le_bytes(), 3).unwrap(), None, "tag {free}");
        }
        assert_eq!(name(loader_v3_buffer_index(&8u32.to_le_bytes(), 3).map(|_| ())), "CodeNotPinnable");
        assert_eq!(name(loader_v3_buffer_index(&10u32.to_le_bytes(), 3).map(|_| ())), "CodeNotPinnable");
        assert_eq!(name(loader_v3_buffer_index(&[3, 0], 7).map(|_| ())), "BadAction");
        assert_eq!(name(loader_v3_buffer_index(&3u32.to_le_bytes(), 2).map(|_| ())), "BadAction");
    }

///
///
    #[test]
    fn a_buffer_is_pinned_by_its_authority_and_its_bytes() {
        let pda = Pubkey::new_unique();
        let up = loader_ix(3, 7);
        let Action::Invoke { accounts, .. } = &up else { unreachable!() };
        let key = accounts[2].pubkey;
        let ours = buffer(Some(pda), b"\x7fELF reviewed");
        let theirs = buffer(Some(Pubkey::new_unique()), b"\x7fELF reviewed");
        let none = buffer(None, b"\x7fELF reviewed");
        let pin = |d: &[u8]| {
            let d = d.to_vec();
            pin_for(&up, &pda, 100, move |k| (*k == key).then(|| (LOADER_V3, Box::leak(d.clone().into_boxed_slice()) as &[u8])))
        };
        let got = pin(&ours).unwrap();
        let hash = solana_sha256_hasher::hash(b"\x7fELF reviewed").to_bytes();
        assert_eq!(got, CodePin::Buffer { buffer: key, code_hash: hash });
        assert_eq!(name(pin(&theirs).map(|_| ())), "BufferAuthorityNotTimelock");
        assert_eq!(name(pin(&none).map(|_| ())), "BufferAuthorityNotTimelock");
        assert_eq!(name(pin_for(&up, &pda, 100, |_| None).map(|_| ())), "PinAccountMissing");
        assert!(check_buffer(&LOADER_V3, &ours, &pda, &hash).is_ok());
        let rewritten = buffer(Some(pda), b"\x7fELF REVIEWED");
        assert_eq!(name(check_buffer(&LOADER_V3, &rewritten, &pda, &hash)), "CodeHashMismatch");
        assert_eq!(name(check_buffer(&LOADER_V3, &theirs, &pda, &hash)), "BufferAuthorityNotTimelock");
        assert_eq!(name(check_buffer(&Pubkey::new_unique(), &ours, &pda, &hash)), "BufferAuthorityNotTimelock");

        assert_eq!(pin_for(&loader_ix(4, 3), &pda, 100, |_| None).unwrap(), CodePin::None);
    }

///
///
    #[test]
    fn an_upgradeable_target_is_pinned_by_its_deploy_slot() {
        let pda = Pubkey::new_unique();
        let target = Pubkey::new_unique();
        let pd = Pubkey::new_unique();
        let call = Action::Invoke { program_id: target, accounts: vec![], data: vec![1] };
        let mut prog = vec![2, 0, 0, 0];
        prog.extend_from_slice(pd.as_ref());
        let mut pdata = vec![3, 0, 0, 0];
        pdata.extend_from_slice(&90u64.to_le_bytes());
        pdata.extend_from_slice(&[0; 33]);
        let world = |owner: Pubkey| {
            let (prog, pdata) = (prog.clone(), pdata.clone());
            move |k: &Pubkey| -> Option<(Pubkey, &'static [u8])> {
                if *k == target {
                    Some((owner, Box::leak(prog.clone().into_boxed_slice())))
                } else if *k == pd {
                    Some((LOADER_V3, Box::leak(pdata.clone().into_boxed_slice())))
                } else {
                    None
                }
            }
        };
        assert_eq!(pin_for(&call, &pda, 100, world(LOADER_V3)).unwrap(), CodePin::Program { program_data: pd, deploy_slot: 90 });
        assert_eq!(name(pin_for(&call, &pda, 90, world(LOADER_V3)).map(|_| ())), "ProgramRedeployed");
        for fixed in FIXED_CODE_OWNERS {
            assert_eq!(pin_for(&call, &pda, 100, world(fixed)).unwrap(), CodePin::None);
        }
        assert_eq!(name(pin_for(&call, &pda, 100, world(LOADER_V4)).map(|_| ())), "CodeNotPinnable");
        assert_eq!(name(pin_for(&call, &pda, 100, |_| None).map(|_| ())), "PinAccountMissing");
        let v4 = Action::Invoke { program_id: LOADER_V4, accounts: vec![], data: vec![0; 4] };
        assert_eq!(name(pin_for(&v4, &pda, 100, |_| None).map(|_| ())), "CodeNotPinnable");
        assert_eq!(pin_for(&Action::UpdateDelay { delay: 900 }, &pda, 100, |_| None).unwrap(), CodePin::None);
    }

///
    #[test]
    fn the_encoded_length_is_the_borsh_length() {
        let a = Action::Invoke {
            program_id: Pubkey::new_unique(),
            accounts: vec![OpMeta { pubkey: Pubkey::new_unique(), is_signer: true, is_writable: true }; 7],
            data: vec![3, 0, 0, 0],
        };
        for a in [a, Action::UpdateDelay { delay: 9 }, Action::SetProposer { proposer: Pubkey::new_unique() }] {
            assert_eq!(anchor_lang::prelude::borsh::to_vec(&a).unwrap().len(), a.encoded_len());
        }
        let widest = CodePin::Buffer { buffer: Pubkey::new_unique(), code_hash: [7; 32] };
        assert_eq!(anchor_lang::prelude::borsh::to_vec(&widest).unwrap().len(), CodePin::MAX_LEN);
    }
}
