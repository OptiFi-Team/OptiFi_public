use crate::errors::ErrorCode;
use crate::instructions::order::serum_utils::serum_cancel_order_with_client_order_id;
use crate::instrument_spl_token_utils::burn_instrument_token_for_user;
use crate::serum_utils::{serum_cancel_order, serum_settle_funds_for_user};
use crate::utils::PREFIX_USER_ACCOUNT;

use crate::state::OptifiMarket;
use crate::state::UserAccount;
use crate::{Exchange, OrderSide};
use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token::accessor::amount;
use serum_dex::critbit::SlabView;
use serum_dex::error::DexErrorCode;
use serum_dex::matching::Side;
use serum_dex::state::{Market, OpenOrders};
use std::num::NonZeroU64;
use std::ops::DerefMut;

/// Accounts used to place orders on the DEX
#[derive(Accounts, Clone)]
pub struct CancelOrderContext<'info> {
    /// optifi_exchange account
    pub optifi_exchange: ProgramAccount<'info, Exchange>,
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
    // /// the mint authoriity of both long and short spl tokens
    // pub instrument_token_mint_authority_pda: AccountInfo<'info>,
    #[account(constraint = usdc_central_pool.key() == optifi_exchange.usdc_central_pool)]
    pub usdc_central_pool: AccountInfo<'info>,
    /// the instrument short spl token
    #[account(mut)]
    pub instrument_short_spl_token_mint: AccountInfo<'info>,
    pub serum_dex_program_id: AccountInfo<'info>,
    #[account(address = token::ID)]
    pub token_program: AccountInfo<'info>,
    // pub rent: Sysvar<'info, Rent>,

    // Oracle to get the spot price
    // pub asset_feed: AccountInfo<'info>,
    // pub usdc_feed: AccountInfo<'info>,
    // pub iv_feed: AccountInfo<'info>,
    // // Clock to get the timestamp
    // pub clock: Sysvar<'info, Clock>,
}

pub fn handle(ctx: Context<CancelOrderContext>, side: OrderSide, order_id: u128) -> ProgramResult {
    let optifi_exchange = &ctx.accounts.optifi_exchange;
    let user = &ctx.accounts.user;
    let user_account = &mut ctx.accounts.user_account;
    let optifi_market = &ctx.accounts.optifi_market;
    let serum_market = &ctx.accounts.serum_market;
    let coin_mint = &ctx.accounts.coin_mint;
    let open_orders = &ctx.accounts.open_orders;
    let event_queue = &ctx.accounts.event_queue;
    let market_bids = &ctx.accounts.bids;
    let market_asks = &ctx.accounts.asks;
    // let order_payer = &ctx.accounts.order_payer_token_account;

    // let open_orders_authority = &ctx.accounts.open_orders_owner;
    let token_program = &ctx.accounts.token_program;
    let dex_program = &ctx.accounts.serum_dex_program_id;
    let coin_vault = &ctx.accounts.coin_vault;
    let pc_vault = &ctx.accounts.pc_vault;
    let user_margin_account = &ctx.accounts.user_margin_account;

    // order_payer account should be usdc vault(margin account) if order is Bid
    // and long token vault for
    let mut order_payer = user_margin_account;

    let user_instrument_long_token_vault = &ctx.accounts.user_instrument_long_token_vault;
    let user_instrument_short_token_vault = &ctx.accounts.user_instrument_short_token_vault;
    let instrument_short_spl_token_mint = &ctx.accounts.instrument_short_spl_token_mint;

    if user_account.is_in_liquidation {
        return Err(ErrorCode::CannotPlaceOrdersInLiquidation.into());
    }

    // 0 is bid, 1 is ask - for the purpose of this, anything non-zero,
    // will be interpreted as ask
    let serum_side: Side;
    match side {
        OrderSide::Bid => serum_side = Side::Bid,
        OrderSide::Ask => serum_side = Side::Ask,
    }

    // get the order amount
    let mut order_amount: u64 = 0;
    let mut lot_size: u64 = 0;

    if serum_side == Side::Ask {
        let program_id = dex_program.key;
        let market = Market::load(serum_market, program_id)?;
        lot_size = market.coin_lot_size;

        // let mut bids = market.load_bids_mut(market_bids)?;
        let mut asks = market.load_asks_mut(market_asks)?;

        let key = asks
            .find_by_key(order_id)
            .ok_or(DexErrorCode::OrderNotFound)
            .unwrap();

        let node = asks.deref_mut().get(key).unwrap().as_leaf().unwrap();

        order_amount = node.quantity();

        order_payer = &ctx.accounts.user_instrument_long_token_vault;
    }

    let exchange_key = optifi_exchange.key();

    let signer_seeds = &[
        PREFIX_USER_ACCOUNT.as_bytes(),
        exchange_key.as_ref(),
        user_account.owner.as_ref(),
        &[user_account.bump],
    ];
    msg!("cancelling the previous order");
    serum_cancel_order(
        signer_seeds,
        dex_program,
        serum_market,
        market_bids,
        market_asks,
        open_orders,
        &user_account.to_account_info(),
        event_queue,
        serum_side,
        order_id,
    )?;

    // let instrument_token_mint_authority_pda = &ctx.accounts.instrument_token_mint_authority_pda;
    let vault_signer = &ctx.accounts.vault_signer;
    // settle funds - get base tokens back
    serum_settle_funds_for_user(
        // user.key,
        signer_seeds,
        dex_program,
        serum_market,
        token_program,
        open_orders,
        &user_account.to_account_info(),
        // user_account.bump,
        coin_vault,
        user_instrument_long_token_vault,
        pc_vault,
        user_margin_account,
        vault_signer,
        &ctx.program_id,
        // optifi_exchange.key,
    )?;

    // burn the same amount of both instrument long and short tokens if ask side
    if serum_side == Side::Ask {
        let amount_to_burn = order_amount
            .checked_div(lot_size)
            .ok_or(ErrorCode::NumericalOverflowError)? as u64;
        // burn the long tokens
        burn_instrument_token_for_user(
            coin_mint,
            order_payer, // order_payer is the same as user_instrument_long_token_vault when the order is ask order
            user.key(),
            &user_account.to_account_info(),
            user_account.bump,
            amount_to_burn,
            token_program,
            &optifi_exchange.key(),
        )?;

        // burn the short tokens
        burn_instrument_token_for_user(
            instrument_short_spl_token_mint,
            user_instrument_short_token_vault,
            user.key(),
            &user_account.to_account_info(),
            user_account.bump,
            amount_to_burn,
            token_program,
            &optifi_exchange.key(),
        )?;

        msg!("successfully burn spl tokens to the seller spl token account");
    }

    let long_amount = amount(user_instrument_long_token_vault).unwrap();
    let short_amount = amount(user_instrument_short_token_vault).unwrap();

    user_account.update_long_position(optifi_market.instrument, long_amount);
    user_account.update_short_position(optifi_market.instrument, short_amount);

    Ok(())
}

pub fn handle2(
    ctx: Context<CancelOrderContext>,
    side: OrderSide,
    client_order_id: u64,
) -> ProgramResult {
    let optifi_exchange = &ctx.accounts.optifi_exchange;
    let user = &ctx.accounts.user;
    let user_account = &mut ctx.accounts.user_account;
    let optifi_market = &ctx.accounts.optifi_market;
    let serum_market = &ctx.accounts.serum_market;
    let coin_mint = &ctx.accounts.coin_mint;
    let open_orders = &ctx.accounts.open_orders;
    let event_queue = &ctx.accounts.event_queue;
    let market_bids = &ctx.accounts.bids;
    let market_asks = &ctx.accounts.asks;
    // let order_payer = &ctx.accounts.order_payer_token_account;

    // let open_orders_authority = &ctx.accounts.open_orders_owner;
    let token_program = &ctx.accounts.token_program;
    let dex_program = &ctx.accounts.serum_dex_program_id;
    let coin_vault = &ctx.accounts.coin_vault;
    let pc_vault = &ctx.accounts.pc_vault;
    let user_margin_account = &ctx.accounts.user_margin_account;

    // order_payer account should be usdc vault(margin account) if order is Bid
    // and long token vault for
    let mut order_payer = user_margin_account;

    let user_instrument_long_token_vault = &ctx.accounts.user_instrument_long_token_vault;
    let user_instrument_short_token_vault = &ctx.accounts.user_instrument_short_token_vault;
    let instrument_short_spl_token_mint = &ctx.accounts.instrument_short_spl_token_mint;

    if user_account.is_in_liquidation {
        return Err(ErrorCode::CannotPlaceOrdersInLiquidation.into());
    }

    // 0 is bid, 1 is ask - for the purpose of this, anything non-zero,
    // will be interpreted as ask
    let serum_side: Side;
    match side {
        OrderSide::Bid => serum_side = Side::Bid,
        OrderSide::Ask => serum_side = Side::Ask,
    }

    // get the order amount
    let mut order_amount: u64 = 0;
    let mut lot_size: u64 = 0;

    if serum_side == Side::Ask {
        let program_id = dex_program.key;
        let market = Market::load(serum_market, program_id)?;
        lot_size = market.coin_lot_size;

        // let mut bids = market.load_bids_mut(market_bids)?;
        let mut asks = market.load_asks_mut(market_asks)?;

        let open_orders = market.load_orders_mut(
            open_orders,
            Some(&user_account.to_account_info()),
            program_id,
            None,
            None,
        )?;

        let slot = open_orders
            .client_order_ids
            .iter()
            .position(|&r| r == client_order_id)
            .unwrap();
        let order_id = open_orders.orders[slot];

        let key = asks
            .find_by_key(order_id)
            .ok_or(DexErrorCode::OrderNotFound)
            .unwrap();
        let node = asks.deref_mut().get(key).unwrap().as_leaf().unwrap();

        order_amount = node.quantity();

        order_payer = &ctx.accounts.user_instrument_long_token_vault;
    }

    let exchange_key = optifi_exchange.key();

    let signer_seeds = &[
        PREFIX_USER_ACCOUNT.as_bytes(),
        exchange_key.as_ref(),
        user_account.owner.as_ref(),
        &[user_account.bump],
    ];
    msg!("cancelling the previous order");
    serum_cancel_order_with_client_order_id(
        signer_seeds,
        dex_program,
        serum_market,
        market_bids,
        market_asks,
        open_orders,
        &user_account.to_account_info(),
        event_queue,
        serum_side,
        client_order_id,
    )?;

    // let instrument_token_mint_authority_pda = &ctx.accounts.instrument_token_mint_authority_pda;
    let vault_signer = &ctx.accounts.vault_signer;
    // settle funds - get base tokens back
    serum_settle_funds_for_user(
        // user.key,
        signer_seeds,
        dex_program,
        serum_market,
        token_program,
        open_orders,
        &user_account.to_account_info(),
        // user_account.bump,
        coin_vault,
        user_instrument_long_token_vault,
        pc_vault,
        user_margin_account,
        vault_signer,
        &ctx.program_id,
        // optifi_exchange.key,
    )?;

    // burn the same amount of both instrument long and short tokens if ask side
    if serum_side == Side::Ask {
        let amount_to_burn = order_amount
            .checked_div(lot_size)
            .ok_or(ErrorCode::NumericalOverflowError)? as u64;
        // burn the long tokens
        burn_instrument_token_for_user(
            coin_mint,
            order_payer, // order_payer is the same as user_instrument_long_token_vault when the order is ask order
            user.key(),
            &user_account.to_account_info(),
            user_account.bump,
            amount_to_burn,
            token_program,
            &optifi_exchange.key(),
        )?;

        // burn the short tokens
        burn_instrument_token_for_user(
            instrument_short_spl_token_mint,
            user_instrument_short_token_vault,
            user.key(),
            &user_account.to_account_info(),
            user_account.bump,
            amount_to_burn,
            token_program,
            &optifi_exchange.key(),
        )?;

        msg!("successfully burn spl tokens to the seller spl token account");
    }

    let long_amount = amount(user_instrument_long_token_vault).unwrap();
    let short_amount = amount(user_instrument_short_token_vault).unwrap();

    user_account.update_long_position(optifi_market.instrument, long_amount);
    user_account.update_short_position(optifi_market.instrument, short_amount);

    Ok(())
}

#[inline]
fn orders_with_client_ids(
    open_orders: OpenOrders,
) -> impl Iterator<Item = (NonZeroU64, u128, Side)> {
    iter_filled_slots(open_orders).filter_map(move |slot| {
        let client_order_id = NonZeroU64::new(open_orders.client_order_ids[slot as usize])?;
        let order_id = open_orders.orders[slot as usize];
        let side = open_orders.slot_side(slot).unwrap();
        Some((client_order_id, order_id, side))
    })
}

#[inline]
fn iter_filled_slots(open_orders: OpenOrders) -> impl Iterator<Item = u8> {
    struct Iter {
        bits: u128,
    }
    impl Iterator for Iter {
        type Item = u8;
        #[inline(always)]
        fn next(&mut self) -> Option<Self::Item> {
            if self.bits == 0 {
                None
            } else {
                let next = self.bits.trailing_zeros();
                let mask = 1u128 << next;
                self.bits &= !mask;
                Some(next as u8)
            }
        }
    }
    Iter {
        bits: !open_orders.free_slot_bits,
    }
}
