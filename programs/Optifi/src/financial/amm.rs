use crate::financial::option::*;
use crate::financial::*;
use crate::{ceil, constants::*};
use anchor_lang::prelude::*;
use ndarray::{Array2, Axis};
use solana_program::log::sol_log_compute_units;

// TOTAL AMM LIQUIDITY
pub fn total_amm_liquidity(
    delta: Vec<f32>,
    price_usd: Vec<f32>,
    amm_position: Vec<i64>,
    futures_position: f32,
    usdc_balance: f32,
    spot_price: f32,
) -> (f32, f32) {
    let net_delta: f32 = 0f32;
    let amm_liquidity: f32 = 0f32;
    // ** hidden code **

    (net_delta, amm_liquidity)
}

// AMM TRADING CAPACITY CALCULATIONS
pub fn calculate_amm_quote_price(
    spot_price: f32,
    side: OrderSide,
    net_delta: f32,
    amm_liquidity_btc: f32,
    quote_size: f32,
) -> (Vec<f32>, Vec<f32>) {
    // ** hidden code **

    (vec![], vec![])
}

/// Clip the vector of orders with zero size
/// `prices` should be sorted from big to small if asks , or from small to big if bids
pub fn clip_order_levels(sizes: &mut Vec<f32>, prices: &mut Vec<f32>) {
    // ** hidden code **
}

// ORDERBOOK
// transpose of python ver.
pub fn calculate_amm_price(
    is_call: bool,
    quote_price_2ask: &Vec<f32>,
    cdelta_btc_raw: &Vec<f32>,
    strikes: &Vec<f32>,
    iv: f32,
    dt: &Vec<f32>,
    spot_price: f32,
) -> Vec<Vec<f32>> {
    let cprice_usd_2ask: Vec<Vec<f32>> = vec![];
    // ** hidden code **

    cprice_usd_2ask
}

pub fn calculate_single_amm_price(
    is_call: bool,
    quote_price_2ask: &Vec<f32>,
    delta: f32,
    spot_price: f32,
    option_price: f32,
) -> Vec<f32> {
    // ** hidden code **
    vec![]
}

pub fn calculate_amm_size(
    is_call: bool,
    btc_delta_size_2ask: &Vec<f32>,
    cdelta_btc_raw: &Vec<f32>,
    quote_size: f32,
) -> Vec<Vec<f32>> {
    let mut result = vec![];

    // ** hidden code **

    result
}

pub fn calculate_amm_size_v2(
    is_call: bool,
    btc_delta_size_2ask: &Vec<f32>,
    mut delta: f32,
    quote_size: f32,
) -> Vec<f32> {
    // ** hidden code **

    vec![]
}
