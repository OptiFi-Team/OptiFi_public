use crate::constants::USDC_DECIMALS;
use crate::errors::ErrorCode;
use crate::instructions::order::serum_utils::serum_new_order_with_client_order_id;
use crate::instructions::order::{
    mint_instrument_token_for_user,
    serum_utils::{serum_new_order, serum_prune_orders_for_user},
};
use crate::serum_utils::serum_settle_funds_for_user;
use crate::state::{AmmAccount, AmmState, OptifiMarket};
use crate::utils::{
    get_serum_market_auth_pda, PREFIX_AMM_LIQUIDITY_AUTH, PREFIX_SERUM_MARKET_AUTH,
};
use crate::{u_to_f_repr, uvec_to_fvec_repr};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, accessor::amount};
use serum_dex::matching::{OrderType, Side};
use serum_dex::state::Market;
use solana_program::log::sol_log_compute_units;

#[derive(Accounts)]
#[instruction(order_limit: u16, instrument_index: u16)]
pub struct UpdateAmmOrders<'info> {
    /// optifi exchange account
    pub optifi_exchange: AccountInfo<'info>,
    /// the amm to update oders for
    #[account(mut, constraint = amm.optifi_exchange == optifi_exchange.key())]
    pub amm: ProgramAccount<'info, AmmAccount>,
    /// amm's margin account(usdc vault) which is controlled by amm_authority (a pda)
    #[account(mut)]
    pub amm_usdc_vault: AccountInfo<'info>,
    /// the authority of amm's amm_usdc_vault
    pub amm_authority: AccountInfo<'info>,
    /// amm's instrument long spl token account
    #[account(mut)]
    pub amm_instrument_long_token_vault: AccountInfo<'info>,
    /// amm's instrument short spl token account
    #[account(mut)]
    pub amm_instrument_short_token_vault: AccountInfo<'info>,
    /// optifi market that binds an instrument with a serum market(orderbook)
    /// it's also the mint authority of the instrument spl token
    #[account(has_one = serum_market, constraint = amm.trading_instruments[instrument_index as usize] == optifi_market.instrument)]
    pub optifi_market: ProgramAccount<'info, OptifiMarket>,
    /// the serum market(orderbook)
    #[account(mut)]
    pub serum_market: AccountInfo<'info>,
    /// amm's open orders account for this optifi market,
    /// its owner is amm account(pda)
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
    /// the mint authoriity of both long and short spl tokens
    pub instrument_token_mint_authority_pda: AccountInfo<'info>,
    /// the instrument short spl token
    #[account(mut)]
    pub instrument_short_spl_token_mint: AccountInfo<'info>,
    // pub prune_authority: AccountInfo<'info>,
    pub serum_dex_program_id: AccountInfo<'info>,
    #[account(address = token::ID)]
    pub token_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
}

/// Submit orders in order proposal - for executing crankers to call
pub fn handle_place_new_order(
    ctx: Context<UpdateAmmOrders>,
    order_limit: u16,
    instrument_index: u16,
    amm_authority_bump: u8,
    market_auth_bump: u8,
) -> ProgramResult {
    // ** hidden code **

    Ok(())
}
