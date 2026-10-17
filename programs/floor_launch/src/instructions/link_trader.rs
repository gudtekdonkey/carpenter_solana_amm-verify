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
use anchor_lang::solana_program::program::invoke;
use floor_types::constants::{trader_link_message, TRADER_LINK_MSG_LEN};
use floor_types::disc;

use crate::constants::{FLOOR_HOOK_PROGRAM, LAUNCH_CONFIG_SEED, TRADER_LINK_SEED, TRADER_REF_SEED, WSOL_MINT};
use crate::errors::LaunchError;
use crate::events::TraderLinked;
use crate::instructions::claim_recipient::previous_ed25519_data;
use crate::sale::{create_pda, is_wallet_key};
use crate::state::{LaunchConfig, TraderLink};

#[event_cpi]
#[derive(Accounts)]
pub struct LinkTrader<'info> {
///
    #[account(mut)]
    pub trader: Signer<'info>,
    #[account(seeds = [LAUNCH_CONFIG_SEED], bump)]
    pub launch_config: AccountLoader<'info, LaunchConfig>,
///
    #[account(mut, seeds = [TRADER_LINK_SEED, trader.key().as_ref()], bump)]
    pub trader_link: UncheckedAccount<'info>,
///
    #[account(address = pubkey!("Sysvar1nstructions1111111111111111111111111"))]
    pub instructions: UncheckedAccount<'info>,
///
    #[account(address = WSOL_MINT)]
    pub wsol_mint: UncheckedAccount<'info>,
///
    #[account(mut, seeds = [TRADER_REF_SEED, trader.key().as_ref(), WSOL_MINT.as_ref()], bump, seeds::program = FLOOR_HOOK_PROGRAM)]
    pub wsol_bucket: UncheckedAccount<'info>,
///
    #[account(address = FLOOR_HOOK_PROGRAM)]
    pub floor_hook: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

///
pub const MSG_LEN: usize = TRADER_LINK_MSG_LEN;
const _: () = assert!(MSG_LEN == 193);
///
pub const ED25519_DATA_LEN: usize = 112 + MSG_LEN;
///
pub const EXPECTED_OFFSETS: [u16; 7] = [48, u16::MAX, 16, u16::MAX, 112, MSG_LEN as u16, u16::MAX];
const TOP_LEVEL: usize = 1;

pub fn handler(ctx: Context<LinkTrader>, tier1: Pubkey, tier2: Pubkey, tier3: Pubkey, deadline: i64) -> Result<()> {
    require!(get_stack_height() == TOP_LEVEL, LaunchError::BadAttestation);
    require!(Clock::get()?.unix_timestamp <= deadline, LaunchError::Expired);

    let trader = ctx.accounts.trader.key();
    require!(
        floor_math::trader_link_tiers_well_formed(&trader.to_bytes(), [&tier1.to_bytes(), &tier2.to_bytes(), &tier3.to_bytes()]),
        LaunchError::BadTraderLinkShape
    );


    for t in [&tier1, &tier2, &tier3] {
        require!(*t == Pubkey::default() || is_wallet_key(t), LaunchError::TraderTierOffCurve);
    }

    let (attestor, cluster) = {
        let c = ctx.accounts.launch_config.load()?;
        (c.attestor, c.cluster)
    };
    let msg = trader_link_message(
        &crate::ID.to_bytes(),
        cluster,
        &trader.to_bytes(),
        [&tier1.to_bytes(), &tier2.to_bytes(), &tier3.to_bytes()],
        deadline,
    );
    {
        let sysvar = ctx.accounts.instructions.try_borrow_data()?;
        let data = previous_ed25519_data(&sysvar).ok_or(LaunchError::BadAttestation)?;
        require!(attestation_matches(data, &attestor.to_bytes(), &msg), LaunchError::BadAttestation);
    }


    let link_info = ctx.accounts.trader_link.to_account_info();
    require!(link_info.owner != &crate::ID, LaunchError::AlreadyLinked);
    let bump = ctx.bumps.trader_link;
    create_pda(
        &ctx.accounts.trader.to_account_info(),
        &link_info,
        TraderLink::LEN,
        &crate::ID,
        &[TRADER_LINK_SEED, trader.as_ref(), &[bump]],
    )?;
    {
        let mut data = link_info.try_borrow_mut_data()?;
        data[..8].copy_from_slice(TraderLink::DISCRIMINATOR);
        let l: &mut TraderLink = bytemuck::from_bytes_mut(&mut data[8..TraderLink::LEN]);
        l.trader = trader;
        l.tier1 = tier1;
        l.tier2 = tier2;
        l.tier3 = tier3;
        l.bump = bump;
    }



    let ix = Instruction {
        program_id: FLOOR_HOOK_PROGRAM,
        accounts: vec![
            AccountMeta::new(trader, true),
            AccountMeta::new_readonly(trader, false),
            AccountMeta::new_readonly(link_info.key(), false),
            AccountMeta::new_readonly(ctx.accounts.wsol_mint.key(), false),
            AccountMeta::new(ctx.accounts.wsol_bucket.key(), false),
            AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
        ],
        data: disc::hook::OPEN_TRADER_BUCKET.to_vec(),
    };
    invoke(
        &ix,
        &[
            ctx.accounts.trader.to_account_info(),
            link_info.clone(),
            ctx.accounts.wsol_mint.to_account_info(),
            ctx.accounts.wsol_bucket.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.floor_hook.to_account_info(),
        ],
    )?;

    emit_cpi!(TraderLinked { trader, tier1, tier2, tier3 });
    Ok(())
}

fn read_u16(d: &[u8], at: usize) -> Option<u16> {
    let b = d.get(at..at.checked_add(2)?)?;
    Some(u16::from_le_bytes([b[0], b[1]]))
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

    fn msg() -> [u8; MSG_LEN] {
        trader_link_message(&crate::ID.to_bytes(), 2, &[1; 32], [&[3; 32], &[4; 32], &[5; 32]], 1_900_000_000)
    }

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

///
    #[test]
    fn the_link_message_vector_is_pinned_for_the_site() {
        let m = msg();
        assert_eq!(m.len(), 193);
        assert_eq!(&m[..24], b"FLOOR_TRADER_REF_LINK_V1");
        assert_eq!(&m[24..56], crate::ID.as_ref());
        assert_eq!(m[56], 2);
        assert_eq!(&m[57..89], &[1u8; 32]);
        assert_eq!(&m[89..185], &[[3u8; 32], [4u8; 32], [5u8; 32]].concat()[..]);
        assert_eq!(&m[185..], &1_900_000_000i64.to_le_bytes());
    }

    #[test]
    fn exact_offsets_pass_and_every_single_mutation_fails() {
        let att = [8u8; 32];
        let d = good(&att, &msg());
        assert_eq!(d.len(), 305);
        assert!(attestation_matches(&d, &att, &msg()));
        for byte in 0..16 {
            let mut bad = d.clone();
            bad[byte] ^= 1;
            assert!(!attestation_matches(&bad, &att, &msg()), "header/offset byte {byte}");
        }
        let mut longer = d.clone();
        longer.extend_from_slice(&[0u8; 32]);
        assert!(!attestation_matches(&longer, &att, &msg()), "trailing bytes");
        assert!(!attestation_matches(&d, &[1u8; 32], &msg()), "another attestor");
        let mut other = msg();
        other[192] ^= 1;
        assert!(!attestation_matches(&d, &att, &other), "another deadline");

        let mut short = d.clone();
        short.truncate(112 + 161);
        assert!(!attestation_matches(&short, &att, &msg()));
    }
}
