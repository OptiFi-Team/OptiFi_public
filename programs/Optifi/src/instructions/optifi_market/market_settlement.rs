use crate::errors::{Error, ErrorCode};
use crate::financial::{
    get_asset_to_usdc_spot, instruments::InstrumentType, verify_switchboard_account, Asset, Chain,
    OracleDataType,
};
use crate::instructions::order::{
    instrument_spl_token_utils::burn_instrument_token_for_user,
    serum_utils::{serum_prune_orders_for_user, serum_settle_funds_for_user},
};
use crate::state::{Exchange, OptifiMarket, UserAccount};
use crate::utils::{
    get_central_usdc_pool_auth_pda, PREFIX_CENTRAL_USDC_POOL_AUTH, PREFIX_USER_ACCOUNT,
};
use anchor_lang::{prelude::*, AnchorDeserialize};
use anchor_spl::token::{
    accessor::{amount, authority},
    Mint, Token,
};
use solana_program::program::invoke_signed;
use std::convert::TryFrom;

/// Record a user's profit and loss when an instrument get expired
#[derive(Accounts)]
#[instruction()]
pub struct RecordPnLForOneUser<'info> {
    /// optifi exchange account
    #[account(mut)]
    pub optifi_exchange: ProgramAccount<'info, Exchange>,
    /// the user's optifi account
    #[account(mut)]
    pub user_account: ProgramAccount<'info, UserAccount>,
    /// The optifi market to be settled
    #[account(mut, constraint = !optifi_market.is_stopped, has_one=serum_market, has_one = instrument)]
    pub optifi_market: ProgramAccount<'info, OptifiMarket>,
    /// the serum market which the optifi market is using
    #[account(mut)]
    pub serum_market: AccountInfo<'info>,
    /// the user's serum open orders account
    #[account(mut)]
    pub user_serum_open_orders: AccountInfo<'info>,
    /// The expired instrument
    #[account(constraint = instrument.is_listed_on_market && instrument.expiry_date as i64 <= clock.unix_timestamp)]
    pub instrument: ProgramAccount<'info, Chain>,
    #[account(mut)]
    pub bids: AccountInfo<'info>,
    #[account(mut)]
    pub asks: AccountInfo<'info>,
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
    #[account(mut)]
    pub coin_vault: AccountInfo<'info>,
    /// The vault for the "quote" currency
    #[account(mut)]
    pub pc_vault: AccountInfo<'info>,
    /// serum market vault owner (pda)
    #[account(mut)]
    pub vault_signer: AccountInfo<'info>,
    #[account(mut)]
    pub user_margin_account: AccountInfo<'info>,
    #[account(mut)]
    pub instrument_long_spl_token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub instrument_short_spl_token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user_instrument_long_token_vault: AccountInfo<'info>,
    #[account(mut)]
    pub user_instrument_short_token_vault: AccountInfo<'info>,
    pub prune_authority: AccountInfo<'info>,
    pub serum_dex_program_id: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
    // oracle account for spot price of the instrument's underlying asset
    pub asset_spot_price_oracle_feed: AccountInfo<'info>,
    // oracle account for usdc spot price
    pub usdc_spot_price_oracle_feed: AccountInfo<'info>,
}

/// fund settlement for crankers to call
pub fn record_pnl_for_one_user(ctx: Context<RecordPnLForOneUser>) -> ProgramResult {
    // TODO
    // 1. close the user's open orders
    // 2. settle fund on serum orderbook
    // 3. if user has both long tokens and short tokens, say X long tokens and Y short tokens,
    // first burn MIN(X,Y) short
    // 4. then burn |Y-X| short tokens, and use the instrument strike and spot price to calc the profit and loss
    // 5. update the user's temp_pnl in user_account based onthe result of last step

    let optifi_exchange = &mut ctx.accounts.optifi_exchange;
    let user_account = &mut ctx.accounts.user_account;
    let optifi_market = &mut ctx.accounts.optifi_market;
    let serum_market = &ctx.accounts.serum_market;
    // let coin_mint = &ctx.accounts.coin_mint;
    let user_serum_open_orders = &ctx.accounts.user_serum_open_orders;
    // let request_queue = &ctx.accounts.request_queue;
    let event_queue = &ctx.accounts.event_queue;
    let bids = &ctx.accounts.bids;
    let asks = &ctx.accounts.asks;
    // let order_payer = &ctx.accounts.order_payer_token_account;
    // let open_orders_authority = &ctx.accounts.open_orders_owner;
    let coin_vault = &ctx.accounts.coin_vault;
    let pc_vault = &ctx.accounts.pc_vault;
    let token_program = &ctx.accounts.token_program;
    // let rent = &ctx.accounts.rent.to_account_info();
    let dex_program = &ctx.accounts.serum_dex_program_id;

    let user_margin_account = &ctx.accounts.user_margin_account;
    let user_instrument_long_token_vault = &ctx.accounts.user_instrument_long_token_vault;
    let user_instrument_short_token_vault = &ctx.accounts.user_instrument_short_token_vault;
    let instrument_long_spl_token_mint = &ctx.accounts.instrument_long_spl_token_mint;
    let instrument_short_spl_token_mint = &ctx.accounts.instrument_short_spl_token_mint;

    let prune_authority = &ctx.accounts.prune_authority;
    let instrument = &ctx.accounts.instrument;
    let asset_oracle_feed = &ctx.accounts.asset_spot_price_oracle_feed;
    let usdc_oracle_feed = &ctx.accounts.usdc_spot_price_oracle_feed;

    if !(verify_switchboard_account(
        Asset::try_from(instrument.asset).unwrap(),
        OracleDataType::Spot,
        asset_oracle_feed.key,
        optifi_exchange,
    ) && verify_switchboard_account(
        Asset::USDC,
        OracleDataType::Spot,
        usdc_oracle_feed.key,
        optifi_exchange,
    )) {
        return Err(ErrorCode::IncorrectOracleAccount.into());
    }

    serum_prune_orders_for_user(
        dex_program,
        serum_market,
        bids,
        asks,
        prune_authority,
        user_serum_open_orders,
        &user_account.to_account_info(),
        user_account.bump,
        event_queue,
        &ctx.program_id,
        &optifi_exchange.key(),
    )?;

    let vault_signer = &ctx.accounts.vault_signer;
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

    let long_amount = amount(user_instrument_long_token_vault).unwrap();
    let short_amount = amount(user_instrument_short_token_vault).unwrap();
    let net_positions = long_amount as i64 - short_amount as i64;

    // burn all the long and short tokens
    burn_instrument_token_for_user(
        &instrument_short_spl_token_mint.to_account_info(),
        &user_instrument_short_token_vault,
        user_account.owner,
        &user_account.to_account_info(),
        user_account.bump,
        short_amount,
        token_program,
        &optifi_exchange.key(),
    )?;

    burn_instrument_token_for_user(
        &instrument_long_spl_token_mint.to_account_info(),
        &user_instrument_long_token_vault,
        user_account.owner,
        &user_account.to_account_info(),
        user_account.bump,
        long_amount,
        token_program,
        &optifi_exchange.key(),
    )?;

    msg!("user's net positions for this market: {}", net_positions);

    // TODO: get the spot price for the underlying of the instrument
    let spot_price_from_oracle = get_asset_to_usdc_spot(asset_oracle_feed, usdc_oracle_feed);

    msg!("instrument.strike: {}", instrument.strike);
    msg!("spot_price_from_oracle: {}", spot_price_from_oracle);

    // check the instrument if call or put
    let mut is_put: f32 = 1.0;
    match instrument.instrument_type {
        InstrumentType::Put => is_put = -1.0,
        _ => (),
    }
    // calc the pnl for the user and credit/debit to user's account
    let pnl = (spot_price_from_oracle - instrument.strike as f32) * is_put * net_positions as f32;
    user_account.temp_pnl.amount += pnl as i64;
    user_account.temp_pnl.epoch = instrument.expiry_date;
    msg!(
        "pnl for this market: {}, total temp pnl: {}, temp pnl epoch: {}",
        pnl,
        user_account.temp_pnl.amount,
        user_account.temp_pnl.epoch
    );
    // =================================================================
    // TODO: stop the optifi market if verything is settled
    // so that the optifi market can be re-used for new instrument later
    // but how do you know everything is settled?
    // =================================================================
    // stop the market if all user's pnl are calculated for this market
    // so that the optifi market can be re-used for new instrument later
    if instrument_long_spl_token_mint.supply == 0 && instrument_short_spl_token_mint.supply == 0 {
        // the optifi market account itself
        optifi_market.is_stopped = true;
        // also update the optifi market key data saved in optifi exchange
        for market in &mut optifi_exchange.markets {
            if market.optifi_market_pubkey == optifi_market.key() {
                market.is_stopped = true
            }
        }
    }

    Ok(())
}

/// Settle fund for a user on all optifi markets that have same expiry date - simultaneous fund settlement
#[derive(Accounts)]
#[instruction()]
pub struct SettleMarketFundForOneUser<'info> {
    /// optifi exchange account
    pub optifi_exchange: Account<'info, Exchange>,
    /// The optifi market to be settled
    // #[account(mut, constraint = !optifi_market.is_stopped, has_one = instrument_long_spl_token, has_one = instrument_short_spl_token)]
    // pub optifi_market: Account<'info, OptifiMarket>,
    #[account(mut, constraint = user_account.temp_pnl.amount != 0 )]
    pub user_account: Account<'info, UserAccount>,
    /// user's margin account
    #[account(mut)]
    pub user_margin_account_usdc: AccountInfo<'info>,
    /// a central fund pool for fund settlemnet purpose
    #[account(mut)]
    pub central_usdc_pool: AccountInfo<'info>,
    pub central_usdc_pool_auth: AccountInfo<'info>,
    pub usdc_mint: Account<'info, Mint>,
    // /// long token total supply should be 0
    // #[account(mut, constraint = instrument_long_spl_token.supply == 0 )]
    // pub instrument_long_spl_token: Account<'info, Mint>,
    // /// short token total supply should be 0
    // #[account(mut, constraint = instrument_short_spl_token.supply == 0 )]
    // pub instrument_short_spl_token: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    // =================== !!! IMPORTANT NOTE !!! =========================
    // pass all the accounts of those optifi_market with same exipry date
    // into ctx.remaining_accounts
    // ====================================================================
}

/// fund settlement for crankers to call
pub fn settle_market_fund_for_one_user(ctx: Context<SettleMarketFundForOneUser>) -> ProgramResult {
    // TODO
    // If the user's temp_pnl = 0, do nothing
    // If the user's temp_pnl > 0, transfer usdc from central_usdc_pool to user's margin account, and set temp_pnl to 0
    // If the user's temp_pnl < 0, transfer usdc from user's margin account to central_usdc_pool, and set temp_pnl to 0
    // after settlement, the central_usdc_pool balance should be zero
    // release the optifi market for re-use later
    let user_account = &mut ctx.accounts.user_account;
    let token_program = &ctx.accounts.token_program;
    let central_usdc_pool = &ctx.accounts.central_usdc_pool;
    let central_usdc_pool_auth = &ctx.accounts.central_usdc_pool_auth;
    let user_margin_account_usdc = &ctx.accounts.user_margin_account_usdc;
    let optifi_exchange = &ctx.accounts.optifi_exchange;
    let usdc_mint = &ctx.accounts.usdc_mint;

    // ====================================================================
    // TODO: check all optifi markets with same maturity are settled
    // ====================================================================
    let all_optifi_markets = &optifi_exchange.markets;
    // let markets_with_same_maturity = ctx.remaining_accounts;
    // ====================================================================
    // TODO:  check all the markets with same maturity are stopped
    // 1. passed remaining accounts (optifi_markets) length should be equal
    // to length of instruments with the same maturity in all_instruments
    // 2. each passed remaining account(optifi_market) should be stopped
    // 3. or not stopped, but it's current instrument expiry timestamp
    // is larger then user's pnl epoch
    // ====================================================================

    // here we only consider the markets with same expiry date that may haven't been set stopped
    // user's PnL may have not been calculated.
    // for those re-used market, user's PnL has been calculated, therefore no need to consider them
    let all_markets_with_same_maturity = all_optifi_markets
        .into_iter()
        .filter(|optifi_market| optifi_market.expiry_date == user_account.temp_pnl.epoch);

    // let mut has_not_stopped_market = false;
    // all_markets_with_same_maturity.for_each(|optifi_market| {
    //     if !optifi_market.is_stopped {
    //         has_not_stopped_market = true
    //     }
    // });
    // if has_not_stopped_market {
    //     return Err(ErrorCode::CannotSettleFundBeforeMarketsStopped.into());
    // }

    for optifi_market in all_markets_with_same_maturity {
        if !optifi_market.is_stopped {
            return Err(ErrorCode::CannotSettleFundBeforeMarketsStopped.into());
        }
    }

    let user_temp_pnl = user_account.temp_pnl.amount;
    // clear user'a temp pnl before moving funds in order to avoid re-entrancy attack
    user_account.temp_pnl.amount = 0;
    user_account.temp_pnl.epoch = 0;

    // move funds according to user's temp PnL
    if user_temp_pnl > 0 {
        let usdc_amount_to_transfer = user_temp_pnl
            .checked_mul(
                10_u32
                    .checked_pow(usdc_mint.decimals.into())
                    .ok_or(ErrorCode::NumericalOverflowError)?
                    .into(),
            )
            .ok_or(ErrorCode::NumericalOverflowError)? as u64;
        msg!(
            "usdc_amount_to_transfer: {}, with decimals: {}",
            usdc_amount_to_transfer,
            usdc_mint.decimals
        );
        let transfer_ix = spl_token::instruction::transfer(
            token_program.key,
            central_usdc_pool.key,
            user_margin_account_usdc.key,
            &central_usdc_pool_auth.key(), // change to central pool auth pda
            &[&central_usdc_pool_auth.key()], // change to central pool auth pda
            usdc_amount_to_transfer,
        )?;

        let (_pda, bump) = get_central_usdc_pool_auth_pda(&optifi_exchange.key(), ctx.program_id);

        msg!("Calling the token program to transfer tokens from central pool to user margin account...");
        invoke_signed(
            &transfer_ix,
            &[
                central_usdc_pool.clone(),
                user_margin_account_usdc.clone(),
                central_usdc_pool_auth.clone(),
                token_program.to_account_info(),
            ],
            &[&[
                PREFIX_CENTRAL_USDC_POOL_AUTH.as_bytes(),
                optifi_exchange.key().as_ref(),
                &[bump],
            ]],
        )
    } else {
        let usdc_amount_to_transfer = user_temp_pnl
            .abs()
            .checked_mul(
                10_u32
                    .checked_pow(usdc_mint.decimals.into())
                    .ok_or(ErrorCode::NumericalOverflowError)?
                    .into(),
            )
            .ok_or(ErrorCode::NumericalOverflowError)? as u64;

        msg!(
            "usdc_amount_to_transfer: {}, with decimals: {}",
            usdc_amount_to_transfer,
            usdc_mint.decimals
        );
        let transfer_ix = spl_token::instruction::transfer(
            token_program.key,
            user_margin_account_usdc.key,
            central_usdc_pool.key,
            &user_account.key(),
            &[&user_account.key()],
            usdc_amount_to_transfer,
        )?;

        msg!("Calling the token program to transfer tokens from user margin account to central pool...");
        invoke_signed(
            &transfer_ix,
            &[
                user_margin_account_usdc.clone(),
                central_usdc_pool.clone(),
                user_account.to_account_info(),
                token_program.to_account_info(),
            ],
            &[&[
                &PREFIX_USER_ACCOUNT.as_bytes(),
                optifi_exchange.key().as_ref(),
                user_account.owner.key().as_ref(),
                &[user_account.bump],
            ]],
        )
    }
}
