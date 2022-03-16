use crate::constants::LIQUIDATION_SLIPPAGE;
use crate::errors::ErrorCode;
use crate::instructions::order::serum_utils::serum_settle_funds_for_user;
use crate::serum_prune_orders_for_user;
use crate::state::{
    Exchange, LiquidationState, LiquidationStatus, MarginStressAccount, OptifiMarket, UserAccount,
};
use crate::utils::pda::PREFIX_USER_ACCOUNT;
use anchor_lang::prelude::*;
use anchor_spl::token::accessor;
use anchor_spl::token::accessor::amount;
use serum_dex::matching::Side;
use serum_dex::state::Market;

#[derive(Accounts)]
pub struct RegisterLiquidationMarket<'info> {
    pub optifi_exchange: ProgramAccount<'info, Exchange>,

    // #[account(constraint = margin_stress_account.optifi_exchange == optifi_exchange.key())]
    // pub margin_stress_account: ProgramAccount<'info, MarginStressAccount>,
    #[account(mut, constraint = user_account.is_in_liquidation @ ErrorCode::UserNotInLiquidation)]
    pub user_account: ProgramAccount<'info, UserAccount>,

    #[account(
        mut,
        constraint = liquidation_state.user_account == user_account.key() @ ErrorCode::InvalidAccount,
        constraint = liquidation_state.status == LiquidationStatus::CancelOrder @ ErrorCode::UserNotInCancelOrder
    )]
    pub liquidation_state: ProgramAccount<'info, LiquidationState>,

    pub market: ProgramAccount<'info, OptifiMarket>,

    #[account(mut, constraint = serum_market.key() == market.serum_market @ ErrorCode::InvalidPDA)]
    pub serum_market: AccountInfo<'info>,
    pub serum_dex_program_id: AccountInfo<'info>,
    #[account(mut)]
    pub bids: AccountInfo<'info>,
    #[account(mut)]
    pub asks: AccountInfo<'info>,
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,

    #[account(mut,
        // constraint = open_orders.owner == &user_account.key()
    )]
    pub open_orders: AccountInfo<'info>,
    pub open_orders_owner: AccountInfo<'info>,
    pub prune_authority: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,

    #[account(
        constraint = market.instrument_long_spl_token == accessor::mint(&user_instrument_long_token_vault)? @ ErrorCode::IncorrectCoinMint,
        // constraint = user_instrument_long_token_vault.owner == &user_account.key() @ ErrorCode::InvalidPDA
    )]
    pub user_instrument_long_token_vault: AccountInfo<'info>,

    #[account(
        constraint = market.instrument_short_spl_token == accessor::mint(&user_instrument_short_token_vault)? @ ErrorCode::IncorrectCoinMint,
        // constraint = user_instrument_short_token_vault.owner == &user_account.key() @ ErrorCode::InvalidPDA
    )]
    pub user_instrument_short_token_vault: AccountInfo<'info>,
}

/// register maket to liquidate for user
pub fn handler(ctx: Context<RegisterLiquidationMarket>) -> ProgramResult {
    // ** hidden code **

    Ok(())
}
