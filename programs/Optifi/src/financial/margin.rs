use std::borrow::Borrow;

use crate::{
    constants::{SECS_IN_STANDARD_YEAR, STEP, STRESS},
    state::{InstrumentCommon, InstrumentUnique, UserPosition},
    u_to_f_repr,
};
use anchor_lang::prelude::*;
use ndarray::{Array, Array2};
use solana_program::log::sol_log_compute_units;

use super::{option_intrinsic_value, option_price, Asset, SpotInputOption};

/// calculates a list of stressed spot prices
pub fn generate_stress_spot(spot: f32, stress: f32, step: u8) -> Vec<Vec<f32>> {
    let mut result: Vec<Vec<f32>> = vec![];
    let mut temp = vec![];
    for i in 0..(step * 2 + 1) {
        let incr = stress / step as f32 * i as f32;
        temp.push(spot * (1 as f32 - stress + incr));
    }
    result.push(temp);

    result
}

/// stress function result
#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct StressFunctionResult {
    // 'Price': price,
    // 'Regulation T Margin': reg_t_margin,
    // 'Delta': delta,
    // 'Intrinsic Value': intrinsic,
    // 'Stress Price Delta': stress_price_change

    // #[serde(rename = "Price")]
    pub price: Vec<Vec<f32>>,

    // #[serde(rename = "Regulation T Margin")]
    // pub reg_t_margin: Vec<Vec<f32>>,

    // pub delta: Vec<Vec<f32>>,
    pub intrinsic_value: Vec<Vec<f32>>,

    pub stress_price_delta: Vec<Vec<f32>>,
}

/// margin function result
#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct MarginFunctionResult {
    net_qty: i64,
    notional_qty: i64,
    net: f32,
    notional: f32,
    stress_result: f32,
    net_intrinsic: f32,
    net_premium: f32,
    maturing_net_intrinsic: f32,
    maturing_premium: f32,
    maturing_liquidity: f32,
    pub total_margin: f32,
    pub net_leverage: f32,
    notional_leverage: f32,
}

/// stress_function
pub fn stress_function(
    spot: f32,
    strike: Vec<f32>,
    iv: f32,
    r: f32,
    q: f32,
    t: &Vec<f32>,
    stress: f32,
    is_call: Vec<u8>,
    step: u8,
) -> StressFunctionResult {
    // main values: prices, reg-t margins, delta, intrinsic values
    // 23700 computing units for 1 strikes
    let spots = SpotInputOption::SingleSpot(spot);
    let price = option_price(spots.borrow(), strike.borrow(), iv, r, q, &t, &is_call);
    // let reg_t_margin = option_reg_t_margin(spots.borrow(), &strike, stress, &is_call);
    // let delta = option_delta(&spots, &strike, iv, r, q, &t, &is_call);

    // 1300 computing units for 1 strikes
    let intrinsic = option_intrinsic_value(&spots, &strike, &is_call);

    // sol_log_compute_units();
    // old version
    // let stress_spot = generate_stress_spot(spot, stress, 1);
    // let spots = SpotInputOption::MultiSpots(stress_spot);
    // let stress_price = option_price(&spots, &strike, iv, r, q, &t, &is_call);

    // let mut result: Vec<Vec<f32>> = vec![];
    // for (i, stress_p_vec) in stress_price.iter().enumerate() {
    //     let mut temp = vec![];
    //     for stress_p in stress_p_vec {
    //         temp.push(stress_p - price[i][0]);
    //     }
    //     result.push(temp);
    // }

    // 47400 computing units
    // new version
    let stress_spot_down = spot * (1.0 - stress);
    let stress_spot_up = spot * (1.0 + stress);

    let mut stress_price = option_price(
        &SpotInputOption::MultiSpots(vec![vec![stress_spot_down, stress_spot_up]]),
        &strike,
        iv,
        r,
        q,
        &t,
        &is_call,
    );

    // 2600 computing units
    for (i, option_prices_in_stress_prices) in stress_price.iter_mut().enumerate() {
        let down = option_prices_in_stress_prices[0] - price[i][0];
        let up = option_prices_in_stress_prices[1] - price[i][0];

        let new_len = step * 2;

        let range = (up - down) / new_len as f32;

        let last_index = new_len - 1;
        option_prices_in_stress_prices.resize(new_len as usize, 0.0);

        for i in 0..step {
            option_prices_in_stress_prices[i as usize] = down + range * i as f32;
            option_prices_in_stress_prices[(last_index - i) as usize] = up - range * i as f32;
        }
    }
    return StressFunctionResult {
        price,
        // reg_t_margin,
        // delta,
        intrinsic_value: intrinsic,
        stress_price_delta: stress_price,
    };
}

/// Margin function
pub fn margin_function(
    user: Vec<i64>,
    t: &Vec<f32>,
    price: &Vec<u64>,
    intrinsic: &Vec<u64>,
    stress_price_change: &Vec<Vec<i64>>,
) -> i64 {
    let user_matrix = Array::from_vec(user.clone());

    let shape = (stress_price_change.len(), stress_price_change[0].len());

    let stress_price_change_vec = stress_price_change.clone().into_iter().flatten().collect();
    let stress_price_change_matrix = Array::from_shape_vec(shape, stress_price_change_vec).unwrap();

    let new_matrix = user_matrix.dot(&stress_price_change_matrix);
    let stress_result = new_matrix.iter().copied().fold(i64::MAX, i64::min);

    let net_intrinsic = intrinsic
        .iter()
        .zip(user_matrix.iter())
        .map(|(&a, b)| a as i64 * b)
        .sum::<i64>();

    let net_premium = price
        .iter()
        .zip(user_matrix.iter())
        .map(|(&a, b)| a as i64 * b)
        .sum::<i64>();

    let mut min_t: Vec<i64> = vec![];
    let t_min = t.iter().copied().fold(f32::NAN, f32::min);

    for e in t {
        if *e == t_min {
            min_t.push(1)
        } else {
            min_t.push(0)
        }
    }

    let matrix1 = t
        .iter()
        .enumerate()
        .map(|(index, &v)| (2. / (365. * v + 1.) * (user[index] * min_t[index]) as f32))
        .collect::<Vec<f32>>();

    // #calculates net premium
    let maturing_premium = matrix1
        .iter()
        .enumerate()
        .map(|(index, v)| v * price[index] as f32 * min_t[index] as f32)
        .sum::<f32>() as i64;

    // #calcualtes liquidity add on
    let maturing_liquidity = matrix1
        .iter()
        .enumerate()
        .map(|(index, v)| v * intrinsic[index] as f32 * min_t[index] as f32)
        .sum::<f32>() as i64;

    // # 1st margin component is a sum of 1) change in value after stress, and a minimum of net_intrincic/net premium value)
    let margin_1 = (stress_result + net_intrinsic.min(net_premium)).min(0);

    // # 2nd margin component is a liquidity add on for soon maturing options
    let margin_2 = if maturing_liquidity < net_intrinsic && maturing_liquidity < 0 {
        maturing_liquidity - net_intrinsic
    } else {
        0
    };

    // # 3rd add on is premium add on for soon maturing options
    let margin_3 = if maturing_premium < 0 {
        maturing_premium
    } else {
        0
    };

    let total_margin = margin_1 + margin_2 + margin_3;

    return total_margin;
}

pub fn _calculate_margin(
    instrument_common: &Vec<InstrumentCommon>,
    instrument_unique: &Vec<Vec<InstrumentUnique>>,
    asset: Asset,
    spot_price: f32,
    iv: f32,
    now: u64,
    user_positions: Vec<UserPosition>,
) {
    // 7200 computing units
    let mut strikes = vec![];
    let mut is_call = vec![];
    let mut t = vec![];
    let mut positions = vec![];

    for (index, common) in instrument_common.iter().enumerate() {
        let time_to_maturity = common.expiry_date - now;
        let time_to_maturity = time_to_maturity as f32 / SECS_IN_STANDARD_YEAR as f32;
        for unique in &instrument_unique[index] {
            for (i, pubkey) in unique.instrument_pubkeys.iter().enumerate() {
                if let Some(p) = user_positions
                    .iter()
                    .find(|user_position| user_position.get_instrument() == pubkey)
                {
                    if p.get_quantity() != 0 {
                        positions.push(p.get_quantity());
                        strikes.push(unique.strike as f32);
                        is_call.push(i as u8);
                        t.push(time_to_maturity);
                    }
                }
                // else {
                //     positions.push(0);
                // }
            }
        }
    }
    // sol_log_compute_units();

    msg!(
        "spot_price {}, strikes {:?}, iv {}, t {:?}, is_call {:?}, positions {:?}",
        spot_price,
        strikes,
        iv,
        &t,
        is_call,
        positions
    );

    // let margins = option_reg_t_margin(
    //     &SpotInputOption::SingleSpot(spot_price),
    //     &strikes,
    //     STRESS,
    //     &is_call,
    // );

    // let margin = positions
    //     .iter()
    //     .zip(margins.iter().flatten())
    //     .map(|(&p, &m)| (p as f32 * m).min(0.0))
    //     .sum::<f32>();

    let stress_function_res =
        stress_function(spot_price, strikes, iv, 0.0, 0.0, &t, STRESS, is_call, STEP);

    // // 37000 computing units
    // let margin_result = margin_function(
    //     positions,
    //     spot_price,
    //     &t,
    //     stress_function_res.price,
    //     stress_function_res.intrinsic_value,
    //     stress_function_res.stress_price_delta,
    // );
    // // sol_log_compute_units();

    // margin_result.total_margin

    // margin
}

/// Old version Margin function
pub fn _margin_function(
    user: Vec<i64>,
    spot: f32,
    t: &Vec<f32>,
    price: Vec<Vec<f32>>,
    intrinsic: Vec<Vec<f32>>,
    stress_price_change: Vec<Vec<f32>>,
) -> MarginFunctionResult {
    // # calculates margin statistics for each user and his positions
    // # totals
    // # net contract position
    let net_qty: i64 = user.iter().sum();
    // #net notional contract position
    let notional_qty: i64 = user.iter().map(|f| f.abs()).sum();
    // # net notional position in USDT (assuming BTC/USDT or ETH/USDT spot price)
    let net = net_qty as f32 * spot;
    let notional = notional_qty as f32 * spot;

    let user_vec: Vec<f32> = user.iter().map(|f| *f as f32).collect();
    let user_matrix = Array::from_vec(user_vec.clone());

    let stress_price_change_vec = stress_price_change.clone().into_iter().flatten().collect();
    let stress_price_change_matrix = Array::from_shape_vec(
        (stress_price_change.len(), stress_price_change[0].len()),
        stress_price_change_vec,
    )
    .unwrap();

    let new_matrix = user_matrix.dot(&stress_price_change_matrix);
    let stress_result = new_matrix.iter().copied().fold(f32::NAN, f32::min);

    let intrinsic_vec = intrinsic.clone().into_iter().flatten().collect();
    let intrinsic_matrix =
        Array::from_shape_vec((intrinsic.len(), intrinsic[0].len()), intrinsic_vec).unwrap();

    let net_intrinsic_matrix = user_matrix.dot(&intrinsic_matrix);
    let net_intrinsic = net_intrinsic_matrix[0];

    let price_vec = price.clone().into_iter().flatten().collect();
    let price_matrix = Array::from_shape_vec((price.len(), price[0].len()), price_vec).unwrap();

    let net_premium_matrix = user_matrix.dot(&price_matrix);
    let net_premium = net_premium_matrix[0];

    let mut min_t: Vec<f32> = vec![];
    let t_min = t.iter().copied().fold(f32::NAN, f32::min);

    for e in t {
        if *e == t_min {
            min_t.push(1.)
        } else {
            min_t.push(0.)
        }
    }

    let min_t_matrix = Array::from(min_t.clone());

    let mut res: Vec<f32> = vec![];
    for (i, e) in user_matrix.iter().enumerate() {
        res.push(e * min_t_matrix[i])
    }

    let user_matrix = Array2::from_shape_vec((user_vec.len(), 1), user_vec).unwrap();
    let min_t_matrix = Array2::from_shape_vec((min_t.len(), 1), min_t).unwrap();
    let matrix1_1 = &user_matrix * &min_t_matrix;
    let matrix1_1_1 = matrix1_1.t();

    let matrix2 = &intrinsic_matrix * &min_t_matrix;

    let maturing_net_intrinsic = matrix1_1_1
        .iter()
        .zip(matrix2.iter())
        .map(|(a, b)| a * b)
        .sum::<f32>();

    let t_matrix = Array2::from_shape_vec((t.len(), 1), t.clone()).unwrap();
    let matrix1 = (2. / (365. * &t_matrix + 1.)) * &user_matrix * &min_t_matrix;

    println!("matrix1 {:?}", matrix1);

    let matrix2 = &price_matrix * &min_t_matrix;

    println!("matrix2 {:?}", matrix1);

    // #calculates net premium
    let maturing_premium = matrix1
        .iter()
        .zip(matrix2.iter())
        .map(|(a, b)| a * b)
        .sum::<f32>();

    // #calcualtes liquidity add on
    let matrix2 = &intrinsic_matrix * &min_t_matrix;
    let maturing_liquidity = matrix1
        .iter()
        .zip(matrix2.iter())
        .map(|(a, b)| a * b)
        .sum::<f32>();

    // # 1st margin component is a sum of 1) change in value after stress, and a minimum of net_intrincic/net premium value)
    let margin_1 = (stress_result + net_intrinsic.min(net_premium)).min(0.0);

    // # 2nd margin component is a liquidity add on for soon maturing options
    let margin_2 = if maturing_liquidity < net_intrinsic && maturing_liquidity < 0. {
        maturing_liquidity - net_intrinsic
    } else {
        0.
    };

    // # 3rd add on is premium add on for soon maturing options
    let margin_3 = if maturing_premium < 0. {
        maturing_premium
    } else {
        0.
    };

    println!("margin_1 {} ", margin_1);
    println!("margin_2 {} ", margin_2);
    println!("margin_3 {} ", margin_3);

    // # total margin
    let total_margin = margin_1 + margin_2 + margin_3;
    let net_leverage = net / total_margin;
    let notional_leverage = notional / total_margin;

    return MarginFunctionResult {
        net_qty,
        notional_qty,
        net,
        notional,
        stress_result,
        net_intrinsic,
        net_premium,
        maturing_net_intrinsic,
        maturing_premium,
        maturing_liquidity,
        total_margin,
        net_leverage,
        notional_leverage,
    };
}
