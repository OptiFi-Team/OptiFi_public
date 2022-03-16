use crate::financial::Asset;
use anchor_lang::prelude::*;
use anchor_lang::{AnchorDeserialize, AnchorSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

#[derive(Clone, Default, AnchorSerialize, AnchorDeserialize)]
pub struct MarketMakerData {
    pub asset: Asset,
    // Stored information for rolling MM calculations
    pub epoch: u64,
    pub volume: u64,
    pub total_indicator: u64,
    pub time_weighted_indicator: u64,
    pub dfm_ask: u64,
}

#[account]
#[derive(Default)]
pub struct MarketMakerAccount {
    /// The user that this market maker is associated with
    pub user_account: Pubkey,
    pub active: bool,
    pub liquidity_pool: Pubkey,
    /// This is used for the market maker 24 hour withdrawl window - if this is 0, then
    /// there's no withdrawal currently registered. If it's not 0, it's the timestamp at which
    /// a withdrawal was started
    pub withdraw_ts: u64,
    pub withdrawal_amount: u64,

    pub outstanding_penalty: u64,
    pub eth_data: MarketMakerData,
    pub btc_data: MarketMakerData,
}
