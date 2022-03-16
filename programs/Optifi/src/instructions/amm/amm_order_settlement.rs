use crate::state::{AmmAccount, OptifiMarket};
use crate::utils::PREFIX_USER_ACCOUNT;
use crate::{serum_settle_funds_for_user, Exchange};
use anchor_lang::prelude::*;
use anchor_spl::token::{accessor, Token};
use solana_program::log::sol_log_compute_units;

use super::burn_instrument_token_for_user;

#[derive(Accounts)]
pub struct AmmOrderSettlement<'info> {
    pub optifi_exchange: ProgramAccount<'info, Exchange>,

    #[account(mut, constraint = amm.optifi_exchange == optifi_exchange.key())]
    pub amm: ProgramAccount<'info, AmmAccount>,

    #[account(mut, constraint = !optifi_market.is_stopped)]
    pub optifi_market: ProgramAccount<'info, OptifiMarket>,

    #[account(mut, constraint = serum_market.key() == optifi_market.serum_market)]
    pub serum_market: AccountInfo<'info>,

    #[account(mut)]
    pub user_serum_open_orders: AccountInfo<'info>,

    #[account(mut)]
    pub coin_vault: AccountInfo<'info>,

    #[account(mut)]
    pub pc_vault: AccountInfo<'info>,

    #[account(mut)]
    pub instrument_long_spl_token_mint: AccountInfo<'info>,
    #[account(mut)]
    pub instrument_short_spl_token_mint: AccountInfo<'info>,
    #[account(mut)]
    pub user_instrument_long_token_vault: AccountInfo<'info>,
    #[account(mut)]
    pub user_instrument_short_token_vault: AccountInfo<'info>,

    #[account(mut)]
    pub user_margin_account: AccountInfo<'info>,

    #[account(mut)]
    pub vault_signer: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,

    pub serum_dex_program_id: AccountInfo<'info>,
}

pub fn handler(ctx: Context<AmmOrderSettlement>) -> ProgramResult {
    let optifi_exchange = &ctx.accounts.optifi_exchange;
    let amm = &mut ctx.accounts.amm;
    let dex_program = &ctx.accounts.serum_dex_program_id;
    let serum_market = &ctx.accounts.serum_market;
    let token_program = &ctx.accounts.token_program;
    let user_serum_open_orders = &ctx.accounts.user_serum_open_orders;
    let coin_vault = &ctx.accounts.coin_vault;
    let pc_vault = &ctx.accounts.pc_vault;
    let vault_signer = &ctx.accounts.vault_signer;
    let user_instrument_long_token_vault = &ctx.accounts.user_instrument_long_token_vault;
    let user_instrument_short_token_vault = &ctx.accounts.user_instrument_short_token_vault;
    let instrument_long_spl_token_mint = &ctx.accounts.instrument_long_spl_token_mint;
    let instrument_short_spl_token_mint = &ctx.accounts.instrument_short_spl_token_mint;
    let user_margin_account = &ctx.accounts.user_margin_account;

    let signer_seeds = &[
        PREFIX_USER_ACCOUNT.as_bytes(),
        optifi_exchange.to_account_info().key.as_ref(),
        user_account.owner.as_ref(),
        &[user_account.bump],
    ];
    serum_settle_funds_for_user(
        // &user_account.owner,
        signer_seeds,
        dex_program,
        serum_market,
        token_program,
        user_serum_open_orders,
        &user_account.to_account_info(),
        // user_account.bump,
        coin_vault,
        user_instrument_long_token_vault,
        pc_vault,
        user_margin_account,
        vault_signer,
        &ctx.program_id,
        // &optifi_exchange.key(),
    )?;

    sol_log_compute_units();

    // Update long position to user account
    let optifi_market = &ctx.accounts.optifi_market;

    // Burn the long and short token...
    let mut long_amount = accessor::amount(user_instrument_long_token_vault).unwrap();
    let mut short_amount = accessor::amount(user_instrument_short_token_vault).unwrap();
    let net_positions = long_amount as i64 - short_amount as i64;

    msg!(
        "long_amount {}, short_amount {}, net_positions {}",
        long_amount,
        short_amount,
        net_positions
    );

    sol_log_compute_units();

    let burn_amount = long_amount.min(short_amount);

    if burn_amount > 0 {
        msg!("burn_amount {}", burn_amount);

        burn_instrument_token_for_user(
            &instrument_short_spl_token_mint,
            &user_instrument_short_token_vault,
            user_account.owner,
            &user_account.to_account_info(),
            user_account.bump,
            burn_amount,
            token_program,
            &optifi_exchange.key(),
        )?;

        burn_instrument_token_for_user(
            &instrument_long_spl_token_mint,
            &user_instrument_long_token_vault,
            user_account.owner,
            &user_account.to_account_info(),
            user_account.bump,
            burn_amount,
            token_program,
            &optifi_exchange.key(),
        )?;

        long_amount = accessor::amount(user_instrument_long_token_vault).unwrap();
        short_amount = accessor::amount(user_instrument_short_token_vault).unwrap();
    }

    sol_log_compute_units();

    msg!(
        "long_amount {}, short_amount {}, net_positions {}",
        long_amount,
        short_amount,
        net_positions
    );

    let long_position = accessor::amount(&ctx.accounts.long_token_vault).unwrap();
    let short_position = accessor::amount(&ctx.accounts.short_token_vault).unwrap();

    amm.positions[long_index].latest_position = long_position as u64 - short_position as u64;

    // TODO
    let serum_market = &ctx.accounts.serum_market;
    let open_orders_account = &ctx.accounts.open_orders_account;
    let open_orders_owner = &ctx.accounts.open_orders_owner;

    let market = Market::load(serum_market, serum_market.owner)?;

    let open_orders_account_info = market.load_orders_mut(
        open_orders_account,
        Some(open_orders_owner),
        serum_market.owner,
        None,
        None,
    )?;

    let open_orders = open_orders_account_info.deref();

    amm.positions[long_index].usdc_balance = open_orders.native_pc_total;

    Ok(())
}
