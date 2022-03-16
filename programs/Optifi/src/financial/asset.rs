use anchor_lang::prelude::*;
use optifi_proc_macros::assert_size;
use std::convert::TryFrom;

#[assert_size(1)]
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum Asset {
    Bitcoin,
    Ethereum,
    USDC,
}

impl Default for Asset {
    fn default() -> Asset {
        Asset::Bitcoin
    }
}

impl TryFrom<u8> for Asset {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == Asset::Bitcoin as u8 => Ok(Asset::Bitcoin),
            x if x == Asset::Ethereum as u8 => Ok(Asset::Ethereum),
            x if x == Asset::USDC as u8 => Ok(Asset::USDC),
            _ => Err(()),
        }
    }
}
