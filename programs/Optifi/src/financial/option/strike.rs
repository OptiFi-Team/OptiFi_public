use crate::constants::{LADDER_SIZE, STRIKES};
use crate::round_as_int;

type StrikeLadder = [i32; (STRIKES) as usize];

/// For a given spot price, reduce it to a target based on its exponent,
/// and return said target and the powers by which it was reduced.
///
/// # Examples
/// ```rust
/// use optifi::financial::option::strike::calculate_target;
///
/// assert_eq!(calculate_target(4999.3f32, 2f32), (49.993f32, 2));
/// ```
pub fn calculate_target(n: f32, dist: f32) -> (f32, i32) {
    let exponent = n.log10().round() - dist;
    let res = n / (10f32.powf(exponent));
    (res, (exponent as i32))
}

fn calculate_incr_base(incr: f32, base: f32) -> f32 {
    let (mut incr_base, exp) = calculate_target(incr, 1f32);
    incr_base *= base;
    incr_base = incr_base.max(incr_base * (incr / incr_base).round());
    let exp_pow = 10f32.powf(exp as f32);

    // Re-round again to closer 1 or 5 multiple
    incr_base = exp_pow * 5f32 * ((incr_base / exp_pow / 5f32).round());

    if incr_base == 0f32 {
        incr_base = exp_pow
    }

    incr_base
}

/// Hardcode deltas for generating strikes based on the following constant calculation (Python)
/// ```python
/// import numpy as np
/// from scipy.stats import norm
///
/// deltas = np.double([0.005, 0.1, 0.25, 0.35, 0.5, 0.65, 0.75, 0.9, 0.995])
/// inv_deltas = norm.ppf(deltas)
/// ```
const INV_DELTAS: [f32; (STRIKES as usize)] = [
    -2.576, -1.282, -0.675, -0.385, 0.000, 0.385, 0.675, 1.282, 2.576,
];

/// Calculate strikes based on spot price, volatility, and years to maturity
///
/// # Examples
/// ```rust
/// use optifi::financial::option::get_strikes;
///
/// let spot = 52000f32;
/// let vol = 0.8f32;
/// let years_to_maturity = 0.019178f32;
///
/// let generated_strikes = get_strikes(spot, vol, years_to_maturity);
///
/// assert_eq!(generated_strikes, [30000, 35000, 40000, 45000, 50000, 55000, 60000, 65000, 70000])
/// ```
pub fn get_strikes(spot: f32, volatility: f32, years_to_maturity: f32) -> StrikeLadder {
    // Reduce the spot price to a target, and figure out the base for some of
    // the rounding calculations
    let (target, _) = calculate_target(spot, 2f32);
    let base: f32;
    if target < 20f32 {
        base = 1f32;
    } else if target < 50f32 {
        base = 2.5f32;
    } else if target < 75f32 {
        base = 5f32;
    } else {
        base = 10f32;
    }

    // Adj annualized volatility to maturity
    let vol_adj = volatility * years_to_maturity.sqrt();

    // Calculate strike ranges
    let strike_ranges: [f32; (STRIKES as usize)] = INV_DELTAS
        .map(|d| d * vol_adj)
        .map(|i| i.exp())
        .map(|s| s * spot);

    // Min/max strike to ATM strike range
    let range_lower: f32 = strike_ranges[LADDER_SIZE as usize] - strike_ranges[0];
    let range_upper: f32 =
        strike_ranges[(STRIKES - 1) as usize] - strike_ranges[LADDER_SIZE as usize];

    // Strike increments
    let incr_lower = range_lower / (LADDER_SIZE as f32);
    let incr_upper = range_upper / (LADDER_SIZE as f32);

    // Bases to be used for rounding
    let base_lower = calculate_incr_base(incr_lower, base);
    let base_upper = calculate_incr_base(incr_upper, base);

    // Base ATM
    let atm = base_lower * ((spot / base_lower).round());

    let mut strikes: StrikeLadder = [0; (STRIKES as usize)];

    let upper_base_mult = base_upper.min(atm);

    for i in 0..STRIKES {
        if i <= LADDER_SIZE {
            strikes[i as usize] = round_as_int!(atm - ((LADDER_SIZE - i) as f32) * base_lower);
        } else {
            strikes[i as usize] = round_as_int!(atm + ((i - LADDER_SIZE) as f32) * upper_base_mult);
        }
    }

    strikes
}
