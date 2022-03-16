use crate::constants::LIQUIDATION_SLIPPAGE;
use crate::errors::ErrorCode;
use crate::instructions::order::serum_utils::serum_new_order_with_client_order_id;
use crate::state::{LiquidationState, OptifiMarket, UserAccount};
use crate::utils::{get_serum_market_auth_pda, PREFIX_SERUM_MARKET_AUTH, PREFIX_USER_ACCOUNT};
use anchor_lang::{prelude::*, ProgramAccount};
use anchor_spl::token::{self, accessor};
use serum_dex::matching::{OrderType, Side};
use serum_dex::state::Market;

#[derive(Accounts)]
// #[instruction(bump: u8)]
pub struct LiquidatePosition<'info> {
    pub optifi_exchange: AccountInfo<'info>,

    #[account(mut, constraint=user_account.is_in_liquidation)]
    pub user_account: ProgramAccount<'info, UserAccount>,

    #[account(mut)]
    pub user_margin_account: AccountInfo<'info>,

    #[account(mut,
        // seeds=[
        //     PREFIX_LIQUIDATION_STATE.as_bytes(),
        //     optifi_exchange.key().as_ref(),
        //     user_account.key().as_ref()
        // ],
        // bump,
        constraint = liquidation_state.user_account == user_account.key()
    )]
    pub liquidation_state: ProgramAccount<'info, LiquidationState>,

    #[account(mut)]
    pub user_instrument_long_token_vault: AccountInfo<'info>,
    #[account(mut)]
    pub user_instrument_short_token_vault: AccountInfo<'info>,

    pub optifi_market: ProgramAccount<'info, OptifiMarket>,

    #[account(mut, constraint = serum_market.key() == optifi_market.serum_market)]
    pub serum_market: AccountInfo<'info>,
    #[account(mut)]
    pub open_orders: AccountInfo<'info>,
    #[account(mut)]
    pub request_queue: AccountInfo<'info>,
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
    #[account(mut)]
    pub bids: AccountInfo<'info>,
    #[account(mut)]
    pub asks: AccountInfo<'info>,
    #[account(mut)]
    pub coin_vault: AccountInfo<'info>,
    #[account(mut)]
    pub pc_vault: AccountInfo<'info>,
    pub serum_dex_program_id: AccountInfo<'info>,
    #[account(address = token::ID)]
    pub token_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,

    #[account(mut, signer)]
    pub liquidator: AccountInfo<'info>,
}
/// liquidate user's positions
pub fn handler(ctx: Context<LiquidatePosition>) -> ProgramResult {
    // ** hidden code **

    Ok(())
}
