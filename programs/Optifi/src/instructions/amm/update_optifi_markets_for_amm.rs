use crate::errors::ErrorCode;
use crate::financial::{Chain, Duration};
use crate::state::{AmmAccount, Exchange, OptifiMarket, Position, Proposal};
use anchor_lang::prelude::*;
use anchor_spl::token::accessor;
use std::convert::TryFrom;

#[derive(Accounts)]
#[instruction(instrument_index: u16)]
pub struct RemoveOptiFiMarketForAMM<'info> {
    pub optifi_exchange: ProgramAccount<'info, Exchange>,

    /// the amm
    #[account(mut, constraint = amm.optifi_exchange == optifi_exchange.key())]
    pub amm: Account<'info, AmmAccount>,
    /// the optifi_market which list the instrument
    #[account(constraint = amm.trading_instruments[instrument_index as usize] == optifi_market.instrument)]
    pub optifi_market: ProgramAccount<'info, OptifiMarket>,
    /// the instrumnet to remove from amm's trading instrument list
    #[account(constraint = amm.trading_instruments[instrument_index as usize] == instrument.key() && instrument.expiry_date as i64 <= clock.unix_timestamp)]
    pub instrument: ProgramAccount<'info, Chain>,
    pub clock: Sysvar<'info, Clock>,
}

/// remove an instrument from amm's trading instrument list due to expiration
pub fn remove_instrument_handler(
    ctx: Context<RemoveOptiFiMarketForAMM>,
    instrument_index: u16,
) -> ProgramResult {
    // ** hidden code **

    Ok(())
}

#[derive(Accounts)]
pub struct AddOptiFiMarketForAMM<'info> {
    pub optifi_exchange: ProgramAccount<'info, Exchange>,

    /// the amm
    #[account(mut)] // TODO: remove hardcoded data space
    pub amm: ProgramAccount<'info, AmmAccount>,
    /// the optifi_market which list the instrument
    //#[account(constraint = optifi_market.instrument == instrument.key())]
    pub optifi_market: ProgramAccount<'info, OptifiMarket>,
    /// the instrumnet to add into amm's trading instrument list, it must not be expired
    #[account(constraint = instrument.asset == amm.asset
        && instrument.expiry_date as i64 > clock.unix_timestamp
        && instrument.duration == Duration::try_from(amm.duration).unwrap()
        && instrument.contract_size == amm.contract_size
    )]
    pub instrument: ProgramAccount<'info, Chain>,
    #[account(constraint = optifi_market.instrument_long_spl_token == accessor::mint(&amm_long_token_vault)?)]
    pub amm_long_token_vault: AccountInfo<'info>,
    #[account(constraint = optifi_market.instrument_short_spl_token == accessor::mint(&amm_short_token_vault)?)]
    pub amm_short_token_vault: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
}

/// add an instrument to amm's trading instrument list due to expiration
pub fn add_instrument_handler(ctx: Context<AddOptiFiMarketForAMM>) -> ProgramResult {
    // ** hidden code **

    Ok(())
}
