use crate::utils::{get_serum_market_auth_pda, PREFIX_SERUM_MARKET_AUTH};
use anchor_lang::{prelude::*, solana_program::program::invoke_signed};
use serum_dex::instruction::{
    cancel_order, cancel_order_by_client_order_id, new_order, prune, settle_funds,
    SelfTradeBehavior,
};
use serum_dex::matching::{OrderType, Side};
use std::num::NonZeroU64;

pub fn serum_new_order<'info>(
    signers_seeds: &[&[&[u8]]],
    // user: &Pubkey,
    serum_market: &AccountInfo<'info>,
    open_orders_account: &AccountInfo<'info>,
    request_queue: &AccountInfo<'info>,
    event_queue: &AccountInfo<'info>,
    market_bids: &AccountInfo<'info>,
    market_asks: &AccountInfo<'info>,
    order_payer: &AccountInfo<'info>,
    open_orders_account_owner: &AccountInfo<'info>,
    // open_orders_account_owner_bump: u8,
    coin_vault: &AccountInfo<'info>,
    pc_vault: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    rent: &AccountInfo<'info>,
    // srm_account_referral: Option<&Pubkey>,
    dex_program: &AccountInfo<'info>,
    side: Side,
    limit_price: u64,
    max_coin_qty: u64,
    order_type: OrderType,
    // client_order_id: u64,
    // self_trade_behavior: SelfTradeBehavior,
    // limit: u16,
    max_pc_qty: u64,
    program_id: &Pubkey,
    exchange: &Pubkey,
) -> ProgramResult {
    let new_order_ix = new_order(
        serum_market.key,
        open_orders_account.key,
        request_queue.key,
        event_queue.key,
        market_bids.key,
        market_asks.key,
        order_payer.key,
        open_orders_account_owner.key,
        coin_vault.key,
        pc_vault.key,
        token_program.key,
        rent.key,
        None, // I'm not 100% sure what this parameter, srm_account_referral refers to, but it's an option, so we'll leave it blank for the time being
        dex_program.key,
        side,
        NonZeroU64::new(limit_price).unwrap(),
        NonZeroU64::new(max_coin_qty).unwrap(),
        OrderType::Limit,
        0, // Also not sure about this one, client_order_id - Jet has this as a constant 0, so we'll try that.
        SelfTradeBehavior::AbortTransaction,
        65535,
        NonZeroU64::new(max_pc_qty).unwrap(),
    )?;
    msg!(
        "Created new order instruction, side: {:?}, price: {}, size: {}, value: {}",
        side,
        limit_price,
        max_coin_qty,
        max_pc_qty
    );

    invoke_signed(
        &new_order_ix,
        &[
            serum_market.clone(),
            open_orders_account.clone(),
            request_queue.clone(),
            event_queue.clone(),
            market_bids.clone(),
            market_asks.clone(),
            order_payer.clone(),
            open_orders_account_owner.clone(),
            coin_vault.clone(),
            pc_vault.clone(),
            token_program.clone(),
            rent.clone(),
            dex_program.clone(),
        ],
        // &[&[
        //     PREFIX_USER_ACCOUNT.as_bytes(),
        //     exchange.key().as_ref(),
        //     user.key().as_ref(),
        //     &[open_orders_account_owner_bump],
        // ]],
        signers_seeds,
    )
}

pub fn serum_new_order_with_client_order_id<'info>(
    signers_seeds: &[&[&[u8]]],
    // user: &Pubkey,
    serum_market: &AccountInfo<'info>,
    open_orders_account: &AccountInfo<'info>,
    request_queue: &AccountInfo<'info>,
    event_queue: &AccountInfo<'info>,
    market_bids: &AccountInfo<'info>,
    market_asks: &AccountInfo<'info>,
    order_payer: &AccountInfo<'info>,
    open_orders_account_owner: &AccountInfo<'info>,
    // open_orders_account_owner_bump: u8,
    coin_vault: &AccountInfo<'info>,
    pc_vault: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    rent: &AccountInfo<'info>,
    // srm_account_referral: Option<&Pubkey>,
    dex_program: &AccountInfo<'info>,
    side: Side,
    limit_price: u64,
    max_coin_qty: u64,
    order_type: OrderType,
    client_order_id: u64,
    // self_trade_behavior: SelfTradeBehavior,
    // limit: u16,
    max_pc_qty: u64,
    program_id: &Pubkey,
    exchange: &Pubkey,
) -> ProgramResult {
    let new_order_ix = new_order(
        serum_market.key,
        open_orders_account.key,
        request_queue.key,
        event_queue.key,
        market_bids.key,
        market_asks.key,
        order_payer.key,
        open_orders_account_owner.key,
        coin_vault.key,
        pc_vault.key,
        token_program.key,
        rent.key,
        None, // I'm not 100% sure what this parameter, srm_account_referral refers to, but it's an option, so we'll leave it blank for the time being
        dex_program.key,
        side,
        NonZeroU64::new(limit_price).unwrap(),
        NonZeroU64::new(max_coin_qty).unwrap(),
        OrderType::Limit,
        client_order_id, // Also not sure about this one, client_order_id - Jet has this as a constant 0, so we'll try that.
        SelfTradeBehavior::AbortTransaction,
        65535,
        NonZeroU64::new(max_pc_qty).unwrap(),
    )?;
    msg!(
        "Created new order instruction, side: {:?}, price: {}, size: {}, value: {}",
        side,
        limit_price,
        max_coin_qty,
        max_pc_qty
    );
    invoke_signed(
        &new_order_ix,
        &[
            serum_market.clone(),
            open_orders_account.clone(),
            request_queue.clone(),
            event_queue.clone(),
            market_bids.clone(),
            market_asks.clone(),
            order_payer.clone(),
            open_orders_account_owner.clone(),
            coin_vault.clone(),
            pc_vault.clone(),
            token_program.clone(),
            rent.clone(),
            dex_program.clone(),
        ],
        // &[&[
        //     PREFIX_USER_ACCOUNT.as_bytes(),
        //     exchange.key().as_ref(),
        //     user.key().as_ref(),
        //     &[open_orders_account_owner_bump],
        // ]],
        signers_seeds,
    )
}

pub fn serum_cancel_order_with_client_order_id<'info>(
    signers_seeds: &[&[u8]],
    dex_program: &AccountInfo<'info>,
    serum_market: &AccountInfo<'info>,
    market_bids: &AccountInfo<'info>,
    market_asks: &AccountInfo<'info>,
    open_orders_account: &AccountInfo<'info>,
    open_orders_account_owner: &AccountInfo<'info>,
    event_queue: &AccountInfo<'info>,
    side: Side,
    client_order_id: u64,
) -> ProgramResult {
    let cancel_order_ix = cancel_order_by_client_order_id(
        dex_program.key,
        serum_market.key,
        market_bids.key,
        market_asks.key,
        open_orders_account.key,
        open_orders_account_owner.key,
        event_queue.key,
        client_order_id,
    )?;
    msg!("Created cancel order instruction, invoking...");

    invoke_signed(
        &cancel_order_ix,
        &[
            dex_program.clone(),
            serum_market.clone(),
            market_bids.clone(),
            market_asks.clone(),
            open_orders_account.clone(),
            open_orders_account_owner.clone(),
            event_queue.clone(),
        ],
        &[&signers_seeds],
    )
}

pub fn serum_cancel_order<'info>(
    signers_seeds: &[&[u8]],
    dex_program: &AccountInfo<'info>,
    serum_market: &AccountInfo<'info>,
    market_bids: &AccountInfo<'info>,
    market_asks: &AccountInfo<'info>,
    open_orders_account: &AccountInfo<'info>,
    open_orders_account_owner: &AccountInfo<'info>,
    event_queue: &AccountInfo<'info>,
    side: Side,
    order_id: u128,
) -> ProgramResult {
    let cancel_order_ix = cancel_order(
        dex_program.key,
        serum_market.key,
        market_bids.key,
        market_asks.key,
        open_orders_account.key,
        open_orders_account_owner.key,
        event_queue.key,
        side,
        order_id,
    )?;
    // msg!("Created cancel order instruction, invoking...");

    invoke_signed(
        &cancel_order_ix,
        &[
            dex_program.clone(),
            serum_market.clone(),
            market_bids.clone(),
            market_asks.clone(),
            open_orders_account.clone(),
            open_orders_account_owner.clone(),
            event_queue.clone(),
        ],
        &[&signers_seeds],
    )
}

pub fn serum_settle_funds_for_user<'info>(
    // user: &Pubkey,
    signers_seeds: &[&[u8]],
    dex_program: &AccountInfo<'info>,
    serum_market: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    open_orders_account: &AccountInfo<'info>,
    open_orders_account_owner: &AccountInfo<'info>,
    // open_orders_account_owner_bump: u8,
    coin_vault: &AccountInfo<'info>,
    coin_wallet: &AccountInfo<'info>,
    pc_vault: &AccountInfo<'info>,
    pc_wallet: &AccountInfo<'info>,
    vault_signer: &AccountInfo<'info>,
    program_id: &Pubkey,
    // exchange: &Pubkey,
) -> ProgramResult {
    let settle_funds_ix = settle_funds(
        dex_program.key,
        serum_market.key,
        token_program.key,
        open_orders_account.key,
        open_orders_account_owner.key,
        coin_vault.key,
        coin_wallet.key,
        pc_vault.key,
        pc_wallet.key,
        None,
        vault_signer.key,
    )?;
    // msg!("Created settle funds instruction");

    invoke_signed(
        &settle_funds_ix,
        &[
            dex_program.clone(),
            serum_market.clone(),
            token_program.clone(),
            open_orders_account.clone(),
            open_orders_account_owner.clone(),
            coin_vault.clone(),
            coin_wallet.clone(),
            pc_vault.clone(),
            pc_wallet.clone(),
            vault_signer.clone(),
        ],
        // &[&[
        //     PREFIX_USER_ACCOUNT.as_bytes(),
        //     exchange.key().as_ref(),
        //     user.key().as_ref(),
        //     &[open_orders_account_owner_bump],
        // ]],
        &[&signers_seeds],
    )
}

/// 0. `[writable]` market
/// 1. `[writable]` bids
/// 2. `[writable]` asks
/// 3. `[signer]` prune authority
/// 4. `[]` open orders.
/// 5. `[]` open orders owner.
/// 6. `[writable]` event queue.
pub fn serum_prune_orders_for_user<'info>(
    dex_program: &AccountInfo<'info>,
    serum_market: &AccountInfo<'info>,
    bids: &AccountInfo<'info>,
    asks: &AccountInfo<'info>,
    prune_authority: &AccountInfo<'info>,
    open_orders_account: &AccountInfo<'info>,
    open_orders_account_owner: &AccountInfo<'info>,
    open_orders_account_owner_bump: u8,
    event_queue: &AccountInfo<'info>,
    program_id: &Pubkey,
    exchange: &Pubkey,
) -> ProgramResult {
    let settle_funds_ix = prune(
        dex_program.key,
        serum_market.key,
        bids.key,
        asks.key,
        prune_authority.key,
        open_orders_account.key,
        open_orders_account_owner.key,
        event_queue.key,
        10, // u16::MAX,
    )?;
    // msg!("Created prune orders instruction");

    let (_serum_prune_auth, bump) = get_serum_market_auth_pda(exchange, program_id);

    invoke_signed(
        &settle_funds_ix,
        &[
            dex_program.clone(),
            serum_market.clone(),
            bids.clone(),
            asks.clone(),
            prune_authority.clone(),
            open_orders_account.clone(),
            open_orders_account_owner.clone(),
            event_queue.clone(),
        ],
        &[&[
            PREFIX_SERUM_MARKET_AUTH.as_bytes(),
            exchange.key().as_ref(),
            &[bump],
        ]],
    )
}
