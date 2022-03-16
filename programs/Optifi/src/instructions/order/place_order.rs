use crate::constants::{SECS_IN_STANDARD_YEAR, USDC_DECIMALS};
use crate::errors::ErrorCode;
use crate::financial::margin::margin_function;
use crate::instructions::order::serum_utils::serum_new_order_with_client_order_id;
use crate::instrument_spl_token_utils::mint_instrument_token_for_user;
use crate::serum_utils::serum_new_order;
use crate::utils::{
    get_central_usdc_pool_auth_pda, get_serum_market_auth_pda, PREFIX_CENTRAL_USDC_POOL_AUTH,
    PREFIX_SERUM_MARKET_AUTH, PREFIX_USER_ACCOUNT,
};

use crate::state::{MarginStressAccount, OptifiMarket};
use crate::state::{MarginStressState, UserAccount};
use crate::{ceil, i_to_f_repr, pay_fees, u_to_f_repr, Exchange, OrderSide};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, accessor};
use serum_dex::matching::{OrderType, Side};
use serum_dex::state::Market;
use solana_program::log::sol_log_compute_units;

/// Accounts used to place orders on the DEX
#[derive(Accounts, Clone)]
pub struct PlaceOrderContext<'info> {
    /// optifi_exchange account
    pub optifi_exchange: ProgramAccount<'info, Exchange>,
    #[account(constraint = margin_stress_account.optifi_exchange == optifi_exchange.key())]
    pub margin_stress_account: ProgramAccount<'info, MarginStressAccount>,
    /// the user's wallet
    #[account(signer)]
    pub user: AccountInfo<'info>,
    /// user's optifi account
    #[account(mut, constraint = user_account.optifi_exchange == optifi_exchange.key())]
    pub user_account: ProgramAccount<'info, UserAccount>,
    /// user's margin account which is controlled by a pda
    #[account(mut)]
    pub user_margin_account: AccountInfo<'info>,
    /// user's instrument long spl token account which is controlled by a the user's user account(pda)
    /// it stands for how many contracts the user sold for the instrument
    /// and it should be the same as order_payer_token_account if the order is ask order
    #[account(mut)]
    pub user_instrument_long_token_vault: AccountInfo<'info>,
    /// user's instrument short spl token account which is controlled by a the user's user account(pda)
    /// it stands for how many contracts the user bought for the instrument
    #[account(mut)]
    pub user_instrument_short_token_vault: AccountInfo<'info>,
    /// optifi market that binds an instrument with a serum market(orderbook)
    /// it's also the mint authority of the instrument spl token
    pub optifi_market: ProgramAccount<'info, OptifiMarket>,
    /// the serum market(orderbook)
    #[account(mut)]
    pub serum_market: AccountInfo<'info>,
    /// the user's open orders account
    #[account(mut)]
    pub open_orders: AccountInfo<'info>,
    // pub open_orders_owner: AccountInfo<'info>,
    #[account(mut)]
    pub request_queue: AccountInfo<'info>,
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
    #[account(mut)]
    pub bids: AccountInfo<'info>,
    #[account(mut)]
    pub asks: AccountInfo<'info>,
    /// The token mint address of "base" currency, aka the instrument long spl token
    #[account(mut)]
    pub coin_mint: AccountInfo<'info>,
    /// The vault for the "base" currency
    #[account(mut)]
    pub coin_vault: AccountInfo<'info>,
    /// The vault for the "quote" currency
    #[account(mut)]
    pub pc_vault: AccountInfo<'info>,
    /// serum market vault owner (pda)
    pub vault_signer: AccountInfo<'info>,
    /// the (coin or price currency) account paying for the order
    // #[account(mut)]
    // pub order_payer_token_account: AccountInfo<'info>,
    // pub market_authority: AccountInfo<'info>,
    /// the mint authoriity of both long and short spl tokens
    pub instrument_token_mint_authority_pda: AccountInfo<'info>,
    #[account(constraint = usdc_central_pool.key() == optifi_exchange.usdc_central_pool)]
    pub usdc_central_pool: AccountInfo<'info>,
    /// the instrument short spl token
    #[account(mut)]
    pub instrument_short_spl_token_mint: AccountInfo<'info>,
    pub serum_dex_program_id: AccountInfo<'info>,
    #[account(address = token::ID)]
    pub token_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    // // Clock to get the timestamp
    // pub clock: Sysvar<'info, Clock>,
}

pub fn handle(
    ctx: Context<PlaceOrderContext>,
    side: OrderSide,
    limit: u64,
    max_coin_qty: u64,
    max_pc_qty: u64,
    client_order_id: u64,
) -> ProgramResult {
    if ctx.accounts.margin_stress_account.state != MarginStressState::Available {
        return Err(ErrorCode::WrongState.into());
    }
    let optifi_exchange = &ctx.accounts.optifi_exchange;
    let user_account = &mut ctx.accounts.user_account;
    let serum_market = &ctx.accounts.serum_market;
    let coin_mint = &ctx.accounts.coin_mint;
    let open_orders = &ctx.accounts.open_orders;
    let request_queue = &ctx.accounts.request_queue;
    let event_queue = &ctx.accounts.event_queue;
    let market_bids = &ctx.accounts.bids;
    let market_asks = &ctx.accounts.asks;
    // let order_payer = &ctx.accounts.order_payer_token_account;
    let coin_vault = &ctx.accounts.coin_vault;
    let pc_vault = &ctx.accounts.pc_vault;
    let token_program = &ctx.accounts.token_program;
    let rent = &ctx.accounts.rent.to_account_info();
    let dex_program = &ctx.accounts.serum_dex_program_id;
    let user_instrument_long_token_vault = &ctx.accounts.user_instrument_long_token_vault;
    let user_instrument_short_token_vault = &ctx.accounts.user_instrument_short_token_vault;
    let instrument_short_spl_token_mint = &ctx.accounts.instrument_short_spl_token_mint;
    let optifi_market = &ctx.accounts.optifi_market;
    let user_margin_account = &ctx.accounts.user_margin_account;
    let margin_stress_account = &ctx.accounts.margin_stress_account;
    // order_payer account should be usdc vault(margin account) if order is Bid
    // and long token vault for
    let mut order_payer = user_margin_account;

    if user_account.is_in_liquidation {
        return Err(ErrorCode::CannotPlaceOrdersInLiquidation.into());
    }

    //pay_order_fees(&ctx, limit)?;

    // 0 is bid, 1 is ask - for the purpose of this, anything non-zero will be interpreted as ask
    let serum_side = match side {
        OrderSide::Bid => Side::Bid,
        OrderSide::Ask => {
            user_account.add_short_position(optifi_market.instrument, max_coin_qty);
            Side::Ask
        }
    };

    sol_log_compute_units();
    // Margin calculation
    let mut positions = vec![];

    for pubkey in &margin_stress_account.instruments {
        if let Some(p) = user_account
            .positions
            .iter()
            .find(|user_position| user_position.get_instrument() == pubkey)
        {
            positions.push(p.get_quantity());
        } else {
            positions.push(0);
        }
    }
    sol_log_compute_units();
    //

    let asset = margin_stress_account.asset;
    let now = margin_stress_account.timestamp;
    let expiry_date = optifi_exchange.get_expiry_date_with_asset(asset);

    let t = expiry_date
        .iter()
        .map(|d| (d - now) as f32 / SECS_IN_STANDARD_YEAR as f32)
        .collect::<Vec<f32>>();

    sol_log_compute_units();

    // 37000 computing units
    let margin_result = margin_function(
        positions,
        &t,
        &margin_stress_account.option_price,
        &margin_stress_account.intrinsic_value,
        &margin_stress_account.option_price_delta_in_stress_price,
    );

    let margin_result = margin_result.min(0);

    let amount_to_reserve = -margin_result as u64;

    sol_log_compute_units();
    msg!(
        "spot_price : {}, iv : {}, amount_to_reserve : {} ",
        u_to_f_repr!(margin_stress_account.spot_price),
        u_to_f_repr!(margin_stress_account.iv),
        amount_to_reserve
    );

    user_account.amount_to_reserve[asset as usize] = amount_to_reserve;

    if !is_margin_sufficient(&user_margin_account, &user_account.amount_to_reserve) {
        return Err(ErrorCode::InsufficientMargin.into());
    }

    // mint the instrument spl token to the seller if it's an ask order
    if serum_side == Side::Ask {
        let instrument_token_mint_authority_pda = &ctx.accounts.instrument_token_mint_authority_pda;
        let serum_market_account_info = Market::load(serum_market, serum_market.owner)?;
        let amount_to_mint = max_coin_qty
            .checked_mul(serum_market_account_info.coin_lot_size)
            .ok_or(ErrorCode::NumericalOverflowError)? as u64;

        // mint long token to user
        mint_instrument_token_for_user(
            coin_mint,
            user_instrument_long_token_vault, // order_payer is the same as user_instrument_long_token_vault when the order is ask order
            amount_to_mint,
            token_program,
            ctx.program_id,
            &optifi_exchange.key(),
            instrument_token_mint_authority_pda,
        )?;

        // mint short token to user
        mint_instrument_token_for_user(
            instrument_short_spl_token_mint,
            user_instrument_short_token_vault,
            amount_to_mint,
            token_program,
            ctx.program_id,
            &optifi_exchange.key(),
            instrument_token_mint_authority_pda,
        )?;
        // msg!(
        //     "successfully minted {} spl tokens to the seller spl token account",
        //     amount_to_mint
        // );
        order_payer = &ctx.accounts.user_instrument_long_token_vault;
    }

    let exchange_key = optifi_exchange.clone().key();

    let (_market_auth, bump) = get_serum_market_auth_pda(&exchange_key, ctx.program_id);
    let signer_seeds: &[&[&[u8]]] = &[
        &[
            PREFIX_USER_ACCOUNT.as_bytes(),
            exchange_key.as_ref(),
            user_account.owner.as_ref(),
            &[user_account.bump],
        ],
        &[
            PREFIX_SERUM_MARKET_AUTH.as_bytes(),
            exchange_key.as_ref(),
            &[bump],
        ],
    ];

    // msg!(
    //     "Open orders account owner is {}",
    //     open_orders.owner.to_string()
    // );
    sol_log_compute_units();
    serum_new_order_with_client_order_id(
        signer_seeds,
        // user.key,
        serum_market,
        open_orders,
        request_queue,
        event_queue,
        market_bids,
        market_asks,
        order_payer,
        &user_account.to_account_info(), // user account is the owner of open orders account
        // user_account.bump,
        coin_vault,
        pc_vault,
        token_program,
        rent,
        dex_program,
        serum_side,
        limit,
        max_coin_qty,
        OrderType::Limit,
        client_order_id,
        max_pc_qty,
        ctx.program_id,
        &optifi_exchange.key(),
    )
}

fn is_margin_sufficient(user_margin_account: &AccountInfo, amount_to_reserve: &[u64]) -> bool {
    let margin = accessor::amount(user_margin_account).unwrap();
    let maintenance = amount_to_reserve.iter().sum::<u64>();
    msg!("margin: {}, maintenance: {}", margin, maintenance);
    if margin >= maintenance {
        return true;
    }
    return false;
}

fn pay_order_fees(ctx: &Context<PlaceOrderContext>, notional: u64) -> ProgramResult {
    let exchange_key = &ctx.accounts.optifi_exchange.key();
    let (_pda, bump) = get_central_usdc_pool_auth_pda(&exchange_key.clone(), ctx.program_id);
    let seeds: &[&[&[u8]]] = &[&[
        PREFIX_CENTRAL_USDC_POOL_AUTH.as_bytes(),
        exchange_key.as_ref(),
        &[bump],
    ]];
    pay_fees(
        notional,
        ctx.accounts.token_program.clone(),
        ctx.accounts.user_margin_account.clone(),
        ctx.accounts.user.clone(),
        ctx.accounts.usdc_central_pool.clone(),
        Some(seeds),
    )
}
