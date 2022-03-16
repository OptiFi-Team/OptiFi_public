//! # option pricing
//! A Black Scholes option pricing library
mod erf;
use crate::constants::DELTA_LIMIT;
use anchor_lang::prelude::*;
use erf::erf;
use solana_program::log::sol_log_compute_units;
use std::f32::consts::SQRT_2;
use std::vec;

/// returns cumulative distribution functions values
pub fn cdf(x: f32) -> f32 {
    erf(x / SQRT_2) * 0.5 + 0.5
}

/// returns a approximated cumulative distribution functions values
pub fn cdf_v2(x: f32) -> f32 {
    let sqrt2pi = 2.506628274631;
    let z = x.abs();
    let t = 1.0 / (1.0 + 0.33267 * z);
    let a1 = 0.4361836;
    let a2 = 0.1201676;
    let a3 = 0.937298;
    let cdf = 1.0 - (a1 * t - a2 * t * t + a3 * t * t * t) * ((-z * z / 2.0).exp() / sqrt2pi);

    if x > 0.0 {
        return cdf;
    } else {
        return 1.0 - cdf;
    }
}
/// Returns result for a single record
///
pub fn d1_single(spot: f32, strike: f32, iv: f32, r: f32, q: f32, t: f32) -> f32 {
    ((spot / strike).ln() + (r - q + iv * iv * 0.5) * t) / (iv * t.sqrt())
}

/// Returns result for a single record
///
pub fn d2_single(spot: f32, strike: f32, iv: f32, r: f32, q: f32, t: f32) -> f32 {
    d1_single(spot, strike, iv, r, q, t) - iv * t.sqrt()
}

/// For constructing the spot input for fn d1() and fn d2()
/// It provids two options: one for single spot, one for array of spot
///
pub enum SpotInputOption {
    /// Accept a spot price record
    SingleSpot(f32),
    /// Accept multiple spot price record
    MultiSpots(Vec<Vec<f32>>),
}

/// d1 from Black Scholes pricing
pub fn d1(
    spots: SpotInputOption,
    strikes: Vec<f32>,
    iv: f32,
    r: f32,
    q: f32,
    t: &Vec<f32>,
) -> Vec<Vec<f32>> {
    if strikes.len() != t.len() {
        // Error
        println!("the lenght of stikes is not equal to length of t")
    }

    let mut spots_final: Vec<f32> = vec![];
    match spots {
        SpotInputOption::SingleSpot(spots) => (spots_final.push(spots)),
        SpotInputOption::MultiSpots(spots) => (spots_final = spots[0].to_vec()),
    }

    let mut result = vec![];
    for (i, &strike) in strikes.iter().enumerate() {
        let mut temp = vec![];
        for spot in &spots_final {
            temp.push(d1_single(*spot, strike, iv, r, q, t[i]))
        }
        result.push(temp)
    }
    result
}

/// d2 from Black Scholes pricing
pub fn d2(
    spots: SpotInputOption,
    strikes: Vec<f32>,
    iv: f32,
    r: f32,
    q: f32,
    t: &Vec<f32>,
) -> Vec<Vec<f32>> {
    if strikes.len() != t.len() {
        // Error
        println!("the lenght of stikes is not equal to length of t")
    }

    let mut res: Vec<Vec<f32>> = vec![];
    let d1_res = d1(spots, strikes, iv, r, q, t);

    for (i, e) in d1_res.iter().enumerate() {
        let mut temp: Vec<f32> = vec![];
        for n in e {
            temp.push(n - iv * t[i].sqrt())
        }
        res.push(temp)
    }
    return res;
}

/// black scholes pricing formula
/// # atm we calculate both puts and calls for each parameter set.
pub fn option_price(
    spots: &SpotInputOption,
    strikes: &Vec<f32>,
    iv: f32,
    r: f32,
    q: f32,
    t: &Vec<f32>,
    is_call: &Vec<u8>,
) -> Vec<Vec<f32>> {
    // 200 units

    let mut spots_final: Vec<f32> = vec![];
    match spots {
        SpotInputOption::SingleSpot(spots) => (spots_final.push(*spots)),
        SpotInputOption::MultiSpots(spots) => (spots_final = spots[0].to_vec()),
    }

    // 300 units
    let mut result: Vec<Vec<f32>> = vec![];
    for (i, strike) in strikes.iter().enumerate() {
        let mut temp = vec![];
        for spot in &spots_final {
            let t_i = t[i];
            // 2859 units
            let d1 = d1_single(*spot, *strike, iv, r, q, t_i);
            // 5000 units
            let d2 = d1 - iv * t_i.sqrt();
            if is_call[i] == 1 {
                let ert = (-q * t_i).exp();
                // 7000 units for cdf
                let cdf_d1 = cdf_v2(d1);
                let cdf_d2 = cdf_v2(d2);
                let call = spot * ert * cdf_d1 - strike * ert * cdf_d2;
                temp.push(call);
            } else if is_call[i] == 0 {
                let ert = (-q * t_i).exp();
                let cdf_md2 = cdf_v2(-d2);
                let cdf_md1 = cdf_v2(-d1);
                let put = strike * ert * cdf_md2 - spot * ert * cdf_md1;
                temp.push(put);
            } else {
                panic!("Neither call or put!");
            }
        }
        result.push(temp);
    }
    return result;
}

// OPTIONS DELTA
pub fn delta_wrapper(
    spot_price: f32,
    strikes: &Vec<f32>,
    iv: f32,
    dt: &Vec<f32>,
    is_call: bool,
    clip: bool,
) -> Vec<f32> {
    // this option delta calculation is used for convenient handling of orderbook calculations

    let is_call = if is_call {
        vec![1 as u8; dt.len()]
    } else {
        vec![0 as u8; dt.len()]
    };

    let mut delta = option_delta(spot_price, &strikes, iv, 0.0, 0.0, &dt, &is_call);

    if clip {
        delta_clip(&mut delta);
    }

    // TODO: convert deltas into spot deltas needed because we can deposit only usdc.
    // let _delta_usd = delta
    //     .iter()
    //     .map(|v| v.iter().map(|&d| d * spot_price).collect::<Vec<f32>>())
    //     .collect::<Vec<Vec<f32>>>();

    delta
}

// clip
pub fn delta_clip(delta: &mut Vec<f32>) {
    for d in delta {
        if d.abs() < DELTA_LIMIT {
            *d = DELTA_LIMIT * d.signum();
        }
    }
}

// OPTIONS price
pub fn price_wrapper(
    spot_price: f32,
    strikes: Vec<f32>,
    iv: f32,
    dt: Vec<f32>,
    is_call: bool,
) -> Vec<Vec<f32>> {
    // this option delta calculation is used for convenient handling of orderbook calculations

    let spot = SpotInputOption::SingleSpot(spot_price);

    let is_call = if is_call {
        vec![1 as u8; dt.len()]
    } else {
        vec![0 as u8; dt.len()]
    };
    // calculate call and put prices
    let price_usd = option_price(&spot, &strikes, iv, 0.0, 0.0, &dt, &is_call);

    let price = price_usd
        .iter()
        .map(|v| v.iter().map(|&d| d / spot_price).collect::<Vec<f32>>())
        .collect::<Vec<Vec<f32>>>();

    price
}

/// delta of a call
pub fn option_delta(
    spot_price: f32,
    strikes: &Vec<f32>,
    iv: f32,
    r: f32,
    q: f32,
    t: &Vec<f32>,
    is_call: &Vec<u8>,
) -> Vec<f32> {
    let mut result: Vec<f32> = vec![];

    for (i, strike) in strikes.iter().enumerate() {
        let call = cdf_v2(d1_single(spot_price, *strike, iv, r, q, t[i]));
        let put = call - 1 as f32;
        let value = is_call[i] as f32 * call + (1 - is_call[i]) as f32 * put;
        result.push(value);
    }
    result
}

/// delta of a call
pub fn option_delta_v2(
    spot_price: f32,
    strikes: &Vec<f32>,
    iv: f32,
    r: f32,
    q: f32,
    t: &Vec<f32>,
) -> Vec<f32> {
    let mut result: Vec<f32> = vec![];
    for (i, strike) in strikes.iter().enumerate() {
        if i % 2 == 0 {
            let call = cdf_v2(d1_single(spot_price, *strike, iv, r, q, t[i]));
            let put = call - 1 as f32;
            result.push(put);
            result.push(call);
        }
    }
    result
}

///	calculates intrinsic value of an option
/// #.clip(0) is used as a function MAX[x,0]
pub fn option_intrinsic_value(
    spots: &SpotInputOption,
    strikes: &Vec<f32>,
    is_call: &Vec<u8>,
) -> Vec<Vec<f32>> {
    let mut spots_final: Vec<f32> = vec![];
    match spots {
        SpotInputOption::SingleSpot(spots) => (spots_final.push(*spots)),
        SpotInputOption::MultiSpots(spots) => (spots_final = spots[0].to_vec()),
    }
    let mut result: Vec<Vec<f32>> = vec![];

    for (i, strike) in strikes.iter().enumerate() {
        let mut temp = vec![];
        let call = if is_call[i] == 1 { true } else { false };
        for spot in &spots_final {
            let value = if call {
                if spot > strike {
                    spot - strike
                } else {
                    0.0
                }
            } else {
                if strike > spot {
                    strike - spot
                } else {
                    0.0
                }
            };
            temp.push(value);
        }
        result.push(temp);
    }
    result
}

/// calculates reg-t margin for each option (EXCLUDING OPTION PRREMIU)
pub fn option_reg_t_margin(
    spots: &SpotInputOption,
    strikes: &Vec<f32>,
    stress: f32,
    is_call: &Vec<u8>,
) -> Vec<Vec<f32>> {
    let mut spots_final: Vec<f32> = vec![];
    match spots {
        SpotInputOption::SingleSpot(spots) => (spots_final.push(*spots)),
        SpotInputOption::MultiSpots(spots) => (spots_final = spots[0].to_vec()),
    }
    let mut result: Vec<Vec<f32>> = vec![];

    for (i, strike) in strikes.iter().enumerate() {
        let mut temp = vec![];
        for spot in &spots_final {
            let value = if is_call[i] == 1 {
                (stress * spot - (strike - spot).max(0.0)).max(stress * spot / 2.0)
            } else {
                (stress * spot - (spot - strike).max(0.0)).max(stress * spot / 2.0)
            };
            temp.push(value);
        }
        result.push(temp);
    }
    result
}
