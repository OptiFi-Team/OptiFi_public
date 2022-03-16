pub mod amm_state;
pub mod exchange;
pub mod liquidation_state;
pub mod market_maker_account;
pub mod position;
pub mod user_account;

pub use amm_state::*;
pub use exchange::*;
pub use liquidation_state::*;
pub use position::*;
pub use user_account::*;

use anchor_lang::{prelude::*, AnchorDeserialize, AnchorSerialize};

#[account]
#[derive(Default)]
pub struct OptifiMarket {
    /// id of the optifi market, we may have markets with id from 1 ~ 50
    pub optifi_market_id: u16,
    /// the serum orderbook market which is used to swap instrument spl token and quote token
    pub serum_market: Pubkey,
    /// the instrument which is listed on this market
    pub instrument: Pubkey,
    /// instrumnet long spl token which would be sent to instrument buyers
    pub instrument_long_spl_token: Pubkey,
    /// instrumnet short spl token which would be minted to instrument seller
    pub instrument_short_spl_token: Pubkey,
    /// whether the optitfi market is stopped, which may be updated when the listing instruments is expired
    pub is_stopped: bool,
    /// bump seed which is used to generate this optifi market address
    pub bump: u8,
}

use crate::financial::Asset;

#[account]
#[derive(Default)]
pub struct MarginStressAccount {
    /// optifi exchange which the MarginStress belongs to
    pub optifi_exchange: Pubkey,
    /// bump seed used to derive this MarginStress address
    pub bump: u8,
    /// underlying asset
    pub asset: Asset, // 1 bytes

    pub spot_price: u64,
    pub iv: u64,

    pub timestamp: u64,

    /// MarginStress's state indicator
    pub state: MarginStressState,
    /// each instrument's state flag under the current MarginStress state
    pub flags: Vec<bool>,

    /// a list of pubkeys of the instruments
    pub instruments: Vec<Pubkey>,
    pub strikes: Vec<u64>,
    pub is_call: Vec<u8>,
    pub expiry_date: Vec<u64>,

    pub option_price: Vec<u64>,
    pub intrinsic_value: Vec<u64>,
    pub option_price_delta_in_stress_price: Vec<Vec<i64>>,
}

#[derive(Clone, Copy, PartialEq, Eq, AnchorDeserialize, AnchorSerialize)]
pub enum MarginStressState {
    Sync,
    Calculate,
    Available,
}

impl Default for MarginStressState {
    fn default() -> MarginStressState {
        MarginStressState::Sync
    }
}

impl MarginStressAccount {
    /// move to next state
    pub fn move_to_next_state(&mut self) {
        match self.state {
            MarginStressState::Sync => self.state = MarginStressState::Calculate,
            MarginStressState::Calculate => self.state = MarginStressState::Available,
            MarginStressState::Available => self.state = MarginStressState::Sync,
        }
    }
    #[inline]
    pub fn get_option_price(&self, instrument: Pubkey) -> u64 {
        for (index, i) in self.instruments.iter().enumerate() {
            if i == &instrument {
                return self.option_price[index];
            }
        }
        panic!("instrument not found");
    }
}
