use crate::constants::{FEE, SECS_IN_DAY, SECS_IN_STANDARD_YEAR, SPREAD_LIMIT};
use crate::errors::ErrorCode;
use crate::financial::instruments::{ExpiryType, InstrumentType};
use crate::financial::{delta_wrapper, get_serum_spot_price, max_bid, min_ask, Asset, Chain};
use crate::state::market_maker_account::{MarketMakerAccount, MarketMakerData};
use crate::{f_to_u_repr, u_to_f_repr};
use anchor_spl::token::accessor::amount;
use serum_dex::critbit::SlabView;
use serum_dex::state::Market;
use solana_program::account_info::AccountInfo;
use solana_program::clock::Clock;
use solana_program::entrypoint::ProgramResult;
use std::convert::TryFrom;

pub fn calculate_rewards_penalties(
    first_run: bool,
    mm_account: &mut MarketMakerAccount,
    spot: f32,
    iv: f32,
    chain: Chain,
    serum_market: &Market,
    asks_account: &AccountInfo,
    bids_account: &AccountInfo,
    contract_position: f32,
    quantity_traded: f32,
    pool_balance: f32,
    timestamp: u64,
) -> ProgramResult {
    // ** hidden code **

    Ok(())
}
