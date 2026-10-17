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

use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token_2022::spl_token_2022;
use floor_types::keys::{SPL_TOKEN_PROGRAM_ID, TOKEN_2022_PROGRAM_ID};
use floor_types::layout::{self as ly, bitmap_account_len, Layout};

use crate::errors::LaunchError;
use crate::graduate::idx;

///
pub const POOL_LEN: usize = 8 + ly::Pool::SIZE;
///
pub const BITMAP_LEN: usize = bitmap_account_len(floor_types::constants::FLOOR_TICK_SPACING);
///
///
///
///
pub const VAULT0_LEN: usize = 165;

///
///
///
///
///
///
///
///
///
pub fn quote_vault_len(mint: &AccountInfo) -> Result<usize> {
    if *mint.owner == Pubkey::new_from_array(SPL_TOKEN_PROGRAM_ID) {
        return Ok(VAULT0_LEN);
    }
    t22_vault_len(mint)
}

///
///
///
///
fn t22_vault_len(mint: &AccountInfo) -> Result<usize> {
    use spl_token_2022::extension::{BaseStateWithExtensions, ExtensionType, StateWithExtensions};
    require_keys_eq!(*mint.owner, Pubkey::new_from_array(TOKEN_2022_PROGRAM_ID), LaunchError::BadConfig);
    let data = mint.try_borrow_data()?;
    let state = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&data).map_err(|_| LaunchError::BadConfig)?;
    let mint_types = state.get_extension_types().map_err(|_| LaunchError::BadConfig)?;
    let account_types = ExtensionType::get_required_init_account_extensions(&mint_types);
    ExtensionType::try_calculate_account_len::<spl_token_2022::state::Account>(&account_types)
        .map_err(|_| error!(LaunchError::BadConfig))
}

///
///
///
///
///
///
pub fn base_vault_len(mint: &AccountInfo) -> Result<usize> {
    t22_vault_len(mint)
}

///
///
///
pub const VAULT1_LEN_UNHOOKED: usize = floor_types::holders::AMM_BASE_VAULT_LEN_UNHOOKED;
pub const VAULT1_LEN_HOOKED: usize = floor_types::holders::AMM_BASE_VAULT_LEN_HOOKED;
///
pub const POOL_INDEX_LEN: usize = 8 + ly::PoolIndexEntry::SIZE;
///
///
///
///
///
///
pub const MINT_POOLS_LEN: usize = 8 + ly::MintPoolCount::SIZE;
///
pub const POSITION_LEN: usize = 8 + ly::Position::SIZE;
///
///
///
///
pub const FIXED_POSITIONS: u64 = 2;
///
///
///
pub const TICK_ARRAY_LEN: usize = ly::TICK_ARRAY_LEN;
///
///
///
///
pub const STAIR_STATE_LEN: usize = 8 + ly::StairState::SIZE;
///
pub const STAIR_POSITIONS: u64 = 3;

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
pub const MAX_GRADUATION_TICK_ARRAYS: usize = 8;

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
pub fn graduation_rent_lamports(rent: &Rent, vault0_len: usize, vault1_len: usize, quote_pools: bool, stair: bool) -> u64 {
    let one = |len: usize| rent.minimum_balance(len);
    one(POOL_LEN)
        + one(BITMAP_LEN)
        + one(vault0_len)
        + one(vault1_len)
        + one(POOL_INDEX_LEN)
        + one(MINT_POOLS_LEN)
        + if quote_pools { one(MINT_POOLS_LEN) } else { 0 }
        + FIXED_POSITIONS * one(POSITION_LEN)
        + if stair { stair_rent_lamports(rent) } else { 0 }
        + (MAX_GRADUATION_TICK_ARRAYS as u64) * one(TICK_ARRAY_LEN)
}

///
///
///
///
///
///
///
///
pub fn stair_rent_budget_lamports(rent: &Rent, stair_steps: u8) -> u64 {
    if stair_steps == 0 {
        return 0;
    }
    floor_types::constants::STAIR_RENT_ROUNDS * rent.minimum_balance(TICK_ARRAY_LEN)
        + (stair_steps as u64 + 1) * rent.minimum_balance(POSITION_LEN)
}

///
///
///
pub fn stair_rent_lamports(rent: &Rent) -> u64 {
    STAIR_POSITIONS * rent.minimum_balance(POSITION_LEN) + rent.minimum_balance(STAIR_STATE_LEN)
}

///
///
///
pub fn owner_tail_rent_lamports(rent: &Rent) -> u64 {
    rent.minimum_balance(floor_types::owners::STAIR_STATE_ACCOUNT_LEN_WITH_OWNERS).saturating_sub(rent.minimum_balance(STAIR_STATE_LEN))
}

///
///
pub fn owner_tail_outstanding(ra: &[AccountInfo], rent: &Rent, owner: bool) -> u64 {
    if !owner {
        return 0;
    }
    let st = &ra[idx::STAIR_STATE];
    if !uncreated(st) && st.data_len() >= floor_types::owners::STAIR_STATE_ACCOUNT_LEN_WITH_OWNERS {
        return 0;
    }
    owner_tail_rent_lamports(rent)
}

///
///
///
///
///
pub fn genesis_cushion_rent_lamports(rent: &Rent) -> u64 {
    rent.minimum_balance(POSITION_LEN)
}

///
///
///
///
///
///
///
pub fn quote_pools(quote_mint: &Pubkey) -> bool {
    *quote_mint != Pubkey::new_from_array(floor_types::keys::WSOL_MINT)
}

///
///
///
///
///
pub fn floor_balance(rent: &Rent) -> u64 {
    rent.minimum_balance(0)
}

///
pub fn available(reserve: &AccountInfo, rent: &Rent) -> u64 {
    reserve.lamports().saturating_sub(floor_balance(rent))
}

///
///
///
///
///
///
///
pub fn fund<'info>(
    creator: &AccountInfo<'info>,
    reserve: &AccountInfo<'info>,
    signer_seeds: &[&[u8]],
    vault0_len: usize,
    vault1_len: usize,
    quote_pools: bool,
    stair: bool,
    stair_budget: u64,
) -> Result<u64> {
    let rent = Rent::get()?;
    let reserved = graduation_rent_lamports(&rent, vault0_len, vault1_len, quote_pools, stair);

    let want = floor_balance(&rent) + reserved + stair_budget;
    let have = reserve.lamports();
    if have < want {
        system_program::transfer(
            CpiContext::new(
                system_program::ID,
                system_program::Transfer { from: creator.clone(), to: reserve.clone() },
            ),
            want - have,
        )?;
    }
    crate::sale::create_pda(creator, reserve, 0, &crate::ID, signer_seeds)?;
    Ok(reserved)
}

///
///
///
///
///
///
///
pub fn uncreated(info: &AccountInfo) -> bool {
    *info.owner == system_program::ID
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
pub fn outstanding(
    ra: &[AccountInfo],
    rent: &Rent,
    vault0_len: usize,
    vault1_len: usize,
    quote_pools: bool,
    stair: bool,
) -> u64 {
    let made = |i: usize, len: usize| if uncreated(&ra[i]) { 0 } else { rent.minimum_balance(len) };
    let positions: u64 = [idx::POSITION_SEED, idx::POSITION_BID]
        .iter()
        .map(|i| made(*i, POSITION_LEN))
        .sum::<u64>()
        + if stair {
            [idx::POSITION_STAIR_STEP0, idx::POSITION_STAIR_RESERVE, idx::POSITION_STAIR_TAIL]
                .iter()
                .map(|i| made(*i, POSITION_LEN))
                .sum::<u64>()
                + made(idx::STAIR_STATE, STAIR_STATE_LEN)
        } else {
            0
        };
    let arrays = ra[idx::TICK_ARRAYS..].iter().filter(|a| !uncreated(a)).count().min(MAX_GRADUATION_TICK_ARRAYS);
    let already = made(idx::POOL, POOL_LEN)
        + made(idx::BITMAP, BITMAP_LEN)
        + made(idx::VAULT0, ra[idx::VAULT0].data_len())
        + made(idx::VAULT1, ra[idx::VAULT1].data_len())
        + made(idx::POOL_INDEX_ENTRY, POOL_INDEX_LEN)
        + made(idx::MINT_POOLS1, MINT_POOLS_LEN)
        + if quote_pools { made(idx::MINT_POOLS0, MINT_POOLS_LEN) } else { 0 }
        + positions
        + (arrays as u64) * rent.minimum_balance(TICK_ARRAY_LEN);
    graduation_rent_lamports(rent, vault0_len, vault1_len, quote_pools, stair).saturating_sub(already)
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
pub fn require_funded(reserve: &AccountInfo, rent: &Rent, owed: u64) -> Result<u64> {
    require_keys_eq!(*reserve.owner, crate::ID, LaunchError::RentReserveShort);
    let have = available(reserve, rent);
    require!(have >= owed, LaunchError::RentReserveShort);
    Ok(have)
}

///
///
///
pub fn pay<'info>(reserve: &AccountInfo<'info>, to: &AccountInfo<'info>, amount: u64) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }
    require_keys_eq!(*reserve.owner, crate::ID, LaunchError::AccountInUse);
    let mut from_l = reserve.try_borrow_mut_lamports()?;
    let mut to_l = to.try_borrow_mut_lamports()?;
    **from_l = from_l.checked_sub(amount).ok_or_else(|| error!(LaunchError::MathOverflow))?;
    **to_l = to_l.checked_add(amount).ok_or_else(|| error!(LaunchError::MathOverflow))?;
    Ok(())
}

///
///
///
pub fn close<'info>(reserve: &AccountInfo<'info>, creator: &AccountInfo<'info>, keep: u64) -> Result<u64> {
    let rent = Rent::get()?;
    let amount = available(reserve, &rent).saturating_sub(keep);
    pay(reserve, creator, amount)?;
    Ok(amount)
}

#[cfg(test)]
mod tests {
    use super::*;

///
///
///
    #[test]
    fn the_byte_table_is_the_accounts_graduation_creates() {
        assert_eq!(POOL_LEN, 640);
        assert_eq!(BITMAP_LEN, 80, "8 + 40 + 8 words of 8 bytes at spacing 60");
        assert_eq!(POOL_INDEX_LEN, 56);
        assert_eq!(MINT_POOLS_LEN, 64);






        assert_eq!(POSITION_LEN, 208);


        assert_eq!(TICK_ARRAY_LEN, 3920);

        let rent = Rent::default();
        let per_byte = |len: usize| ((128 + len) as u64) * 3480 * 2;
        assert_eq!(rent.minimum_balance(0), 890_880, "the zero-data floor the reserve keeps");
        assert_eq!(rent.minimum_balance(TICK_ARRAY_LEN), per_byte(TICK_ARRAY_LEN));
        assert_eq!(rent.minimum_balance(TICK_ARRAY_LEN), 28_174_080, "one tick array at the default rate");


        let fixed = per_byte(POOL_LEN)
            + per_byte(BITMAP_LEN)
            + per_byte(VAULT0_LEN)
            + per_byte(VAULT1_LEN_UNHOOKED)
            + per_byte(POOL_INDEX_LEN)
            + per_byte(MINT_POOLS_LEN)
            + FIXED_POSITIONS * per_byte(POSITION_LEN);
        let arrays = (MAX_GRADUATION_TICK_ARRAYS as u64) * per_byte(TICK_ARRAY_LEN);
        assert_eq!(graduation_rent_lamports(&rent, VAULT0_LEN, VAULT1_LEN_UNHOOKED, false, false), fixed + arrays);


        assert_eq!(fixed, 18_200_400, "0.01820 SOL of fixed accounts (2026-10-04: two positions, was 22,877,520 with four)");

        assert_eq!(arrays, 225_392_640, "0.22539 SOL of tick arrays — they dominate");
        assert_eq!(graduation_rent_lamports(&rent, VAULT0_LEN, VAULT1_LEN_UNHOOKED, false, false), 243_593_040, "0.24359 SOL reserved at create (was 248,270,160 with four positions)");



        assert_eq!(
            graduation_rent_lamports(&rent, 170, VAULT1_LEN_UNHOOKED, false, false) - graduation_rent_lamports(&rent, VAULT0_LEN, VAULT1_LEN_UNHOOKED, false, false),
            per_byte(170) - per_byte(165)
        );



        assert_eq!(
            graduation_rent_lamports(&rent, VAULT0_LEN, VAULT1_LEN_UNHOOKED, true, false) - graduation_rent_lamports(&rent, VAULT0_LEN, VAULT1_LEN_UNHOOKED, false, false),
            per_byte(MINT_POOLS_LEN)
        );
        assert_eq!(graduation_rent_lamports(&rent, VAULT0_LEN, VAULT1_LEN_UNHOOKED, true, false), 243_593_040 + 1_336_320, "one 64-byte account of rent more: 0.24493 SOL");
    }

///
///
///
    fn live_rent() -> Rent {

        Rent::with_lamports_per_byte(5_080)
    }

///
///
///
///
///
    #[test]
    fn tc_b7_the_table_at_the_live_rate() {
        let rent = live_rent();
        let per_byte = |len: usize| ((128 + len) as u64) * 5_080;
        assert_eq!(rent.minimum_balance(TICK_ARRAY_LEN), per_byte(TICK_ARRAY_LEN));
        assert_eq!(rent.minimum_balance(TICK_ARRAY_LEN), 20_563_840, "one tick array at 5,080 / byte");
        let total = graduation_rent_lamports(&rent, VAULT0_LEN, VAULT1_LEN_UNHOOKED, false, false);
        let arrays = (MAX_GRADUATION_TICK_ARRAYS as u64) * 20_563_840;
        assert_eq!(total - arrays, 13_284_200, "the fixed accounts");
        assert_eq!(total, 177_794_920, "0.1778 SOL reserved at create");
    }

///
///
    struct Slot {
        key: Pubkey,
        lamports: u64,
        data: Vec<u8>,
        owner: Pubkey,
    }

///
    fn list(arrays: usize) -> Vec<Slot> {
        (0..idx::TICK_ARRAYS + arrays)
            .map(|_| Slot { key: Pubkey::new_unique(), lamports: 0, data: Vec::new(), owner: system_program::ID })
            .collect()
    }

///
    fn create(slots: &mut [Slot], i: usize, len: usize) {
        slots[i].owner = Pubkey::new_unique();
        slots[i].data = vec![0u8; len];
    }

    fn infos(slots: &mut [Slot]) -> Vec<AccountInfo<'_>> {
        slots
            .iter_mut()
            .map(|s| AccountInfo::new(&s.key, false, false, &mut s.lamports, &mut s.data, &s.owner, false))
            .collect()
    }

///
///
    #[test]
    fn tc_b7_a_reserve_funded_at_the_v2_table_is_exactly_covered() {
        let rent = live_rent();
        for k in [0usize, 4] {
            let mut slots = list(11);
            let mut spent = 0u64;
            if k > 0 {
                for (i, len) in [
                    (idx::POOL, POOL_LEN),
                    (idx::BITMAP, BITMAP_LEN),
                    (idx::VAULT0, VAULT0_LEN),
                    (idx::VAULT1, VAULT1_LEN_UNHOOKED),
                    (idx::POOL_INDEX_ENTRY, POOL_INDEX_LEN),
                    (idx::MINT_POOLS1, MINT_POOLS_LEN),
                    (idx::POSITION_SEED, POSITION_LEN),
                    (idx::POSITION_BID, POSITION_LEN),
                ] {
                    create(&mut slots, i, len);
                    spent += rent.minimum_balance(len);
                }
                for j in 0..k {
                    create(&mut slots, idx::TICK_ARRAYS + j, TICK_ARRAY_LEN);
                    spent += rent.minimum_balance(TICK_ARRAY_LEN);
                }
            }
            let available = graduation_rent_lamports(&rent, VAULT0_LEN, VAULT1_LEN_UNHOOKED, false, false) - spent;
            let ra = infos(&mut slots);
            assert_eq!(outstanding(&ra, &rent, VAULT0_LEN, VAULT1_LEN_UNHOOKED, false, false), available, "k = {k}");
        }
    }

///
///
    #[test]
    fn the_quote_pools_row_is_every_quote_but_wsol() {
        let wsol = Pubkey::new_from_array(floor_types::keys::WSOL_MINT);
        assert!(!quote_pools(&wsol));
        assert!(quote_pools(&Pubkey::new_unique()));
    }

///
///
    #[test]
    fn a_higher_rent_rate_raises_the_requirement() {
        let base = Rent::default();
        let dearer = Rent { lamports_per_byte_year: base.lamports_per_byte_year * 2, ..base };
        assert_eq!(graduation_rent_lamports(&dearer, VAULT0_LEN, VAULT1_LEN_UNHOOKED, true, false), 2 * graduation_rent_lamports(&base, VAULT0_LEN, VAULT1_LEN_UNHOOKED, true, false));
    }

///
///
///
///
///
///
///
///
    #[test]
    fn the_quote_vault_is_priced_at_the_length_the_amm_will_create() {
        use spl_token_2022::extension::ExtensionType;
        let spl = Pubkey::new_from_array(SPL_TOKEN_PROGRAM_ID);
        let t22 = Pubkey::new_from_array(TOKEN_2022_PROGRAM_ID);
        let key = Pubkey::new_unique();
        let check = |owner: Pubkey, mut data: Vec<u8>| -> Result<usize> {
            let mut lamports = 1u64;
            let ai = AccountInfo::new(&key, false, false, &mut lamports, &mut data, &owner, false);
            quote_vault_len(&ai)
        };

        let mut spl_mint = vec![0u8; 82];
        spl_mint[44] = 9;
        spl_mint[45] = 1;
        assert_eq!(check(spl, spl_mint.clone()).unwrap(), VAULT0_LEN);

        let mut bare = vec![0u8; 166];
        bare[44] = 6;
        bare[45] = 1;
        bare[165] = 1;
        assert_eq!(check(t22, bare).unwrap(), 165);

        let vl = ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(&[ExtensionType::Pausable]).unwrap()
            - 166
            - 4;
        let mut pausable = vec![0u8; 166 + 4 + vl];
        pausable[44] = 6;
        pausable[45] = 1;
        pausable[165] = 1;
        pausable[166..168].copy_from_slice(&u16::from(ExtensionType::Pausable).to_le_bytes());
        pausable[168..170].copy_from_slice(&(vl as u16).to_le_bytes());
        assert_eq!(check(t22, pausable).unwrap(), 170, "PausableAccount is a zero-length TLV: 166 + 4");

        assert!(check(Pubkey::new_unique(), spl_mint).is_err());
    }
///
///
///
///
///
///
///
///
    #[test]
    fn t_slc_the_base_vault_is_priced_off_the_mint_hooked_or_not() {
        use spl_token_2022::extension::ExtensionType;
        let t22 = Pubkey::new_from_array(TOKEN_2022_PROGRAM_ID);
        let key = Pubkey::new_unique();

        let mint_of = |types: &[ExtensionType]| -> Vec<u8> {
            let len = ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(types).unwrap();
            let mut d = vec![0u8; len];
            d[44] = 6;
            d[45] = 1;
            d[165] = 1;
            let mut at = 166;
            for t in types {
                let one = ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(&[*t]).unwrap() - 166 - 4;
                d[at..at + 2].copy_from_slice(&u16::from(*t).to_le_bytes());
                d[at + 2..at + 4].copy_from_slice(&(one as u16).to_le_bytes());
                at += 4 + one;
            }
            d
        };
        let len_of = |mut data: Vec<u8>| -> usize {
            let mut lamports = 1u64;
            let ai = AccountInfo::new(&key, false, false, &mut lamports, &mut data, &t22, false);
            base_vault_len(&ai).unwrap()
        };
        let unhooked = mint_of(&crate::mint_profile::FIXED_EXTENSIONS);
        let hooked = mint_of(&crate::mint_profile::FIXED_EXTENSIONS_HOOKED);
        assert_eq!(unhooked.len(), 271, "the unhooked fixed profile (spec 2026-09-23 §1.5)");
        assert_eq!(hooked.len(), 339, "the hooked fixed profile: + TransferHook's 4 + 64");
        assert_eq!(len_of(unhooked), VAULT1_LEN_UNHOOKED);
        assert_eq!(len_of(hooked), VAULT1_LEN_HOOKED, "TransferHookAccount is a 1-byte TLV: 170 + 5");

        let rent = Rent::default();
        let per_byte = |len: usize| ((128 + len) as u64) * 3480 * 2;
        assert_eq!(
            graduation_rent_lamports(&rent, VAULT0_LEN, VAULT1_LEN_HOOKED, false, false)
                - graduation_rent_lamports(&rent, VAULT0_LEN, VAULT1_LEN_UNHOOKED, false, false),
            per_byte(175) - per_byte(170),
            "a hooked launch reserves exactly five more bytes of rent"
        );
    }



///
///
///
///
///
    #[test]
    fn a_staircase_is_three_positions_and_its_state_charged_and_credited_by_their_slots() {
        assert_eq!(STAIR_STATE_LEN, 592);
        for rent in [live_rent(), Rent::default()] {
            let one = rent.minimum_balance(POSITION_LEN);
            let state = rent.minimum_balance(STAIR_STATE_LEN);
            let base = graduation_rent_lamports(&rent, VAULT0_LEN, VAULT1_LEN_UNHOOKED, false, false);
            let stair = graduation_rent_lamports(&rent, VAULT0_LEN, VAULT1_LEN_UNHOOKED, false, true);
            assert_eq!(stair, base + 3 * one + state);
            let mut slots = list(11);
            {
                let ra = infos(&mut slots);
                assert_eq!(outstanding(&ra, &rent, VAULT0_LEN, VAULT1_LEN_UNHOOKED, false, true), stair, "nothing made");
            }
            for i in [idx::POSITION_STAIR_STEP0, idx::POSITION_STAIR_RESERVE, idx::POSITION_STAIR_TAIL] {
                create(&mut slots, i, POSITION_LEN);
            }
            create(&mut slots, idx::STAIR_STATE, STAIR_STATE_LEN);
            let ra = infos(&mut slots);
            assert_eq!(outstanding(&ra, &rent, VAULT0_LEN, VAULT1_LEN_UNHOOKED, false, true), base, "prepared: all four credited");
            assert_eq!(outstanding(&ra, &rent, VAULT0_LEN, VAULT1_LEN_UNHOOKED, false, false), base, "no staircase: neither charged nor credited");
        }
    }
}
