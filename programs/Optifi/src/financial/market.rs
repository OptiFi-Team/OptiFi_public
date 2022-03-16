use crate::financial::Asset;
use anchor_lang::prelude::*;
use optifi_proc_macros::assert_size;

/// A Market represents a pair of an asset, a contract type, and an underlying
///asset  - for example BTC Options UDSC, or ETH Futures USCD.
/// Within a given market there a number of chains of different instruments

#[assert_size(40)]
#[account]
pub struct Market {
    // In case we decide to ever upgrade the market code. This will always be set to the
    // MARKET_VERSION constant
    pub version: i32,             // 4 bytes
    pub asset: Asset,             // 4 bytes
    pub market_authority: Pubkey, // 32 bytes
}
