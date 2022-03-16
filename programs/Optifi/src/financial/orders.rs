/// Placeholder types, we'll probably just want to use the Serum orderbook for this instead of
/// defining it ourselves, but I want to flesh out an end-to-end anchor example with orders,
/// so going to leave it here for the moment
use anchor_lang::prelude::*;
use optifi_proc_macros::assert_size;

#[assert_size(1)]
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum OrderSide {
    Bid,
    Ask,
}

impl Default for OrderSide {
    fn default() -> OrderSide {
        OrderSide::Bid
    }
}
