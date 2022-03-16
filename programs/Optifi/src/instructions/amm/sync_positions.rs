use crate::errors::ErrorCode;
use crate::state::{AmmAccount, AmmState, OptifiMarket};
use anchor_lang::prelude::*;
use anchor_spl::token::accessor;
use lazy_static::__Deref;
use serum_dex::state::Market;
use solana_program::log::sol_log_compute_units;

#[derive(Accounts)]
#[instruction(instrument_index: u16)]
pub struct SyncPositions<'info> {
    pub optifi_exchange: AccountInfo<'info>,
    /// the amm to which user will deposits funds
    #[account(mut, constraint = amm.optifi_exchange == optifi_exchange.key())]
    pub amm: ProgramAccount<'info, AmmAccount>,
    /// the optifi market where the instrumnet to be synced is listed
    #[account(constraint = amm.trading_instruments[instrument_index as usize] == optifi_market.instrument)]
    pub optifi_market: ProgramAccount<'info, OptifiMarket>,

    /// amm's base token vault (Long position)
    #[account(constraint = optifi_market.instrument_long_spl_token == accessor::mint(&long_token_vault)?)]
    pub long_token_vault: AccountInfo<'info>,

    /// amm's base token vault (Short position)
    #[account(constraint = optifi_market.instrument_short_spl_token == accessor::mint(&short_token_vault)?)]
    pub short_token_vault: AccountInfo<'info>,

    /// the serum market(orderbook)
    #[account(constraint = optifi_market.serum_market == serum_market.key())]
    pub serum_market: AccountInfo<'info>,

    /// the open orders account
    pub open_orders_account: AccountInfo<'info>,

    pub open_orders_owner: AccountInfo<'info>,
}

/// Update AMM Positions
pub fn handler(ctx: Context<SyncPositions>, instrument_index: u16) -> ProgramResult {
    // ** hidden code **

    Ok(())
}
