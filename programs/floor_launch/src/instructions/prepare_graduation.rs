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
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::Mint;

use floor_types::constants::{FLOOR_TICK_SPACING, LAUNCH_STATE_BONDING, LAUNCH_STATE_GENESIS_CLOSED};
use floor_types::layout::tick_array_start_index;
use floor_types::{disc, seeds as sd};

use crate::constants::{CARPENTER_AMM_PROGRAM, FLOOR_HOOK_PROGRAM, LAUNCH_SEED};
use crate::errors::LaunchError;
use crate::events::GraduationPrepared;
use crate::graduate::{self, idx};
use crate::graduation_rent;
use crate::state::Launch;



#[event_cpi]
#[derive(Accounts)]
pub struct PrepareGraduation<'info> {
///
///
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(seeds = [LAUNCH_SEED, mint.key().as_ref()], bump)]
    pub launch: AccountLoader<'info, Launch>,
    #[account(address = launch.load()?.mint @ LaunchError::BadConfig)]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    pub token_2022: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<'info, PrepareGraduation<'info>>) -> Result<()> {

    let snap: Launch = *ctx.accounts.launch.load()?;
    require!(
        snap.state == LAUNCH_STATE_BONDING || snap.state == LAUNCH_STATE_GENESIS_CLOSED,
        LaunchError::CannotGraduate
    );

    let ra = ctx.remaining_accounts;
    let launch_key = ctx.accounts.launch.key();


    let checked = graduate::check_accounts_of(ra, &launch_key, None, &snap)?;


    let rent = Rent::get()?;
    let reserve = &ra[idx::GRADUATION_RENT];


    let vault0_len = graduation_rent::quote_vault_len(&ra[idx::WSOL_MINT])?;


    let vault1_len = graduation_rent::base_vault_len(&ctx.accounts.mint.to_account_info())?;
    let quote_pools = graduation_rent::quote_pools(&snap.quote_mint());

    let stair_steps = graduate::pool_stair_steps(&ra[idx::FLOOR_POOL])?;
    let available = graduation_rent::require_funded(
        reserve,
        &rent,
        graduation_rent::outstanding(ra, &rent, vault0_len, vault1_len, quote_pools, stair_steps != 0)
            .saturating_add(graduation_rent::owner_tail_outstanding(ra, &rent, graduate::pool_stair_owner(&ra[idx::FLOOR_POOL])? != 0))

            .saturating_add(snap.stair_rent_budget as u64),
    )?;
    let payer_before = ctx.accounts.payer.lamports();

    let payer = ctx.accounts.payer.to_account_info();
    let launch = ctx.accounts.launch.to_account_info();
    let mint = ctx.accounts.mint.to_account_info();
    let token_2022 = ctx.accounts.token_2022.to_account_info();
    let system_program = ctx.accounts.system_program.to_account_info();
    let launch_bump = [snap.bump];
    let launch_seeds: &[&[u8]] = &[LAUNCH_SEED, snap.mint.as_ref(), &launch_bump];
    let open = snap.open_tick;


    let pool_created = graduation_rent::uncreated(&ra[idx::POOL]);
    if pool_created {
        let sqrt_open =
            carpenter_clmm_math::tick_math::get_sqrt_price_at_tick(open).map_err(|_| LaunchError::MathOverflow)?;
        graduate::initialize_pool_cpi(
            &payer,
            &launch,
            &mint,
            &token_2022,
            &system_program,
            ra,
            sqrt_open,
            launch_seeds,
        )?;
    }






    let (pol_lower, pol_upper) = graduate::pol_range(open).map_err(|_| error!(LaunchError::MathOverflow))?;
    let (lad_lower, lad_upper) = graduate::ladder_range(open)?;
    let mut starts = [
        tick_array_start_index(pol_lower, FLOOR_TICK_SPACING),
        tick_array_start_index(pol_upper, FLOOR_TICK_SPACING),
        tick_array_start_index(lad_lower, FLOOR_TICK_SPACING),
        tick_array_start_index(lad_upper, FLOOR_TICK_SPACING),
    ];
    starts.sort_unstable();





    let stair_starts = if stair_steps != 0 {

        let open_edge = floor_math::spacing::floor_to_spacing(open, FLOOR_TICK_SPACING as i32)
            .map_err(|_| error!(LaunchError::MathOverflow))?;
        graduate::stair_array_starts(open_edge, stair_steps)?
    } else {
        Vec::new()
    };

    let mut tick_arrays_created: u8 = 0;
    let all: Vec<(i32, bool)> = starts
        .iter()
        .map(|s| (*s, true))
        .chain(stair_starts.iter().map(|s| (*s, true)))
        .collect();
    for (k, (start, required)) in all.iter().enumerate() {
        if all[..k].iter().any(|(s, _)| s == start) {
            continue;
        }
        let key =
            Pubkey::find_program_address(&[sd::TICK_ARRAY, checked.pool.as_ref(), &start.to_le_bytes()], &CARPENTER_AMM_PROGRAM).0;
        let Some(array) = ra[idx::TICK_ARRAYS..].iter().find(|a| *a.key == key) else {
            if *required {
                return Err(error!(ErrorCode::AccountNotEnoughKeys).with_account_name("tick_array"));
            }
            continue;
        };
        if !graduation_rent::uncreated(array) {
            continue;
        }




        let mut data = Vec::with_capacity(8 + 4 + 32);
        data.extend_from_slice(&disc::amm::INITIALIZE_TICK_ARRAY);
        data.extend_from_slice(&start.to_le_bytes());
        data.extend_from_slice(reserve.key.as_ref());

        graduate::cpi(
            CARPENTER_AMM_PROGRAM,
            data,
            &[
                graduate::w_s(&payer),
                graduate::r(&ra[idx::POOL]),
                graduate::w(array),
                graduate::r(&system_program),
            ],
            &[],
        )?;
        tick_arrays_created += 1;
    }






    graduate::cpi(
        FLOOR_HOOK_PROGRAM,
        disc::hook::PREPARE_POSITIONS.to_vec(),
        &[
            graduate::r_s(&launch),
            graduate::r(&ra[idx::FLOOR_POOL]),
            graduate::r(&ra[idx::POOL]),
            graduate::w(&ra[idx::POSITION_SEED]),
            graduate::w(&ra[idx::POSITION_BID]),
            graduate::w_s(&payer),
            graduate::r(&system_program),
            graduate::r(&ra[idx::CARPENTER_AMM]),
            graduate::r(&ra[idx::HOOK_AUTHORITY]),


            graduate::w(&ra[idx::STAIR_STATE]),
            graduate::w(&ra[idx::POSITION_STAIR_STEP0]),
            graduate::w(&ra[idx::POSITION_STAIR_RESERVE]),
            graduate::w(&ra[idx::POSITION_STAIR_TAIL]),
        ],
        &[launch_seeds],
    )?;




    let spent = payer_before.saturating_sub(ctx.accounts.payer.lamports());
    let reimbursed = spent.min(available);
    graduation_rent::pay(reserve, &payer, reimbursed)?;

    emit_cpi!(GraduationPrepared {
        launch: launch_key,
        pool: checked.pool,
        pool_created,
        tick_arrays_created,
        reimbursed,
    });
    Ok(())
}
