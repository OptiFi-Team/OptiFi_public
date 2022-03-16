use crate::financial::instruments::*;
use crate::financial::*;
use anchor_lang::prelude::*;
use solana_program::pubkey::Pubkey;

#[account]
#[derive(Default)]
pub struct Exchange {
    /// id of the OptiFi Exchange
    pub uuid: String,
    /// OptiFi Exchange version
    pub version: u32,
    /// the authority address
    pub exchange_authority: Pubkey,
    pub owner: Pubkey, //TODO: do we need this??
    /// the recognized usdc token mint
    pub usdc_mint: Pubkey,
    /// usdc central pool for fund settlement
    pub usdc_central_pool: Pubkey,
    /// oracle data by assets
    pub oracle: Vec<OracleData>,
    /// a list of all created serum markets, it should be updated when new market is created
    pub markets: Vec<OptifiMarketKeyData>,
    // a list of all created instruments, it should be updated when new instrument is created
    pub instrument_common: Vec<InstrumentCommon>,
    // a list of all created instruments, it should be updated when new instrument is created
    pub instrument_unique: Vec<Vec<InstrumentUnique>>,
}

impl Exchange {
    pub fn get_instrument_data(
        &self,
        instrument_pubkey: &Pubkey,
    ) -> Option<(InstrumentCommon, u32, bool)> {
        for (index, uniques) in self.instrument_unique.iter().enumerate() {
            for unique in uniques {
                for (is_call, k) in unique.instrument_pubkeys.iter().enumerate() {
                    if k == instrument_pubkey {
                        return Some((self.instrument_common[index], unique.strike, is_call != 0));
                    }
                }
            }
        }

        return None;
    }

    pub fn get_instrument_pubkey(&self, asset: Option<Asset>) -> Vec<Pubkey> {
        let mut instrument_pubkey = vec![];

        for (index, ic) in self.instrument_common.iter().enumerate() {
            if let Some(asset) = asset {
                if asset != ic.asset {
                    continue;
                }
            }
            for uniques in self.instrument_unique[index].iter() {
                instrument_pubkey =
                    [instrument_pubkey, uniques.instrument_pubkeys.to_vec()].concat()
            }
        }
        instrument_pubkey
    }

    pub fn get_expiry_date_with_asset(&self, asset: Asset) -> Vec<u64> {
        let mut expiry_date: Vec<u64> = vec![];

        for (index, ic) in self.instrument_common.iter().enumerate() {
            if asset != ic.asset {
                continue;
            }
            for uniques in self.instrument_unique[index].iter() {
                expiry_date = [expiry_date, [ic.expiry_date; 2].to_vec()].concat();
            }
        }

        expiry_date
    }

    pub fn get_instrument_data_with_asset(
        &self,
        asset: Asset,
    ) -> (Vec<Pubkey>, Vec<u64>, Vec<u8>, Vec<u64>) {
        let mut instrument_pubkey: Vec<Pubkey> = vec![];
        let mut strikes: Vec<u64> = vec![];
        let mut is_call: Vec<u8> = vec![];
        let mut expiry_date: Vec<u64> = vec![];

        for (index, ic) in self.instrument_common.iter().enumerate() {
            if asset != ic.asset {
                continue;
            }
            for uniques in self.instrument_unique[index].iter() {
                instrument_pubkey =
                    [instrument_pubkey, uniques.instrument_pubkeys.to_vec()].concat();

                strikes = [strikes, [uniques.strike as u64; 2].to_vec()].concat();

                is_call = [is_call, [0, 1].to_vec()].concat();

                expiry_date = [expiry_date, [ic.expiry_date; 2].to_vec()].concat();
            }
        }

        (instrument_pubkey, strikes, is_call, expiry_date)
    }
}

/// only keep the key data for a created Instrument
#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct OracleData {
    pub asset: Asset,

    /// trusted oracle account for sopt price
    pub spot_oracle: Option<Pubkey>,
    /// trusted oracle account for iv
    pub iv_oracle: Option<Pubkey>,
    // pub spot_price: u64,

    // pub iv: u64,

    // pub latest_update_timestamp: u64,
}

/// keep the common data for an instrument group
#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub struct InstrumentCommon {
    /// underlying asset
    pub asset: Asset, // 1 bytes
    /// expiry date of the instrument, unix timestamp
    pub expiry_date: u64, // 8 bytes

    pub expiry_type: ExpiryType, // 1 byte
}

/// keep the unique data for an instrument
#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub struct InstrumentUnique {
    /// strike price of the instrument
    pub strike: u32, // 4 bytes
    /// instrument pubkey (0: put 1: call)
    pub instrument_pubkeys: [Pubkey; 2],
}

/// only keep the key data for a created OptiFi Market
#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct OptifiMarketKeyData {
    /// pubkey of created optifi market
    pub optifi_market_pubkey: Pubkey,
    // /// id of the optifi market, we may have markets with id from 1 ~ 50
    // pub optifi_market_id: u16,
    // /// the serum orderbook market which is used to swap instrument spl token and quote token
    // pub serum_market: Pubkey,
    // /// the instrument which is listed on this market
    // pub instrument: Pubkey,
    /// expiry date of the instrument which is listed on this market
    pub expiry_date: u64,
    /// whether the optitfi market is stopped, which may be updated when the listing instruments is expired
    pub is_stopped: bool,
}
