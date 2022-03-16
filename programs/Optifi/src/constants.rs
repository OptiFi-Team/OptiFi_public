/// Important constants used throughout the system

// The fee for each transaction on the OptiFi system, currently set at 0.05%
pub const FEE: f32 = 0.0005;

// The fee (in USD) that a cranker will receive.
pub const CRANKER_FEE: f32 = 0.002;
pub const MM_BALANCE_THRESHOLD: f32 = 0.1;

// Current version of the market schema
pub const MARKET_VERSION: i32 = 1;

// Orderbook spread limit for penalties, 1%
pub const SPREAD_LIMIT: f32 = 0.01;

/// How many strikes to generate on either side of a spot,
/// and the increment in USD they'll be generated at.
/// This value MUST be odd for the strike ladder to be generated
/// correctly
/// ```rust
/// use optifi::constants::STRIKES;
/// assert_eq!(STRIKES % 2, 1)
/// ```
pub const STRIKES: i32 = 9;
pub const BACKUP_STRIKES: i32 = 4;
// This will be the number of strikes on either side of the
// atm strike.
pub const LADDER_SIZE: i32 = (STRIKES - 1) / 2;
pub const BTC_STRIKES_INCR_USD: i32 = 500;
pub const ETH_STRIKES_INCR_USD: i32 = 50;

// Constant program addressses in devnet and mainnet for the oracles
pub const SWITCHBOARD_DEVNET_BTC_USD: &str = "74YzQPGUT9VnjrBz8MuyDLKgKpbDqGot5xZJvTtMi6Ng";
pub const SWITCHBAORD_MAINNET_BTC_USD: &str = "74YzQPGUT9VnjrBz8MuyDLKgKpbDqGot5xZJvTtMi6Ng";

pub const SWITCHBOARD_DEVNET_ETH_USD: &str = "QJc2HgGhdtW4e7zjvLB1TGRuwEpTre2agU5Lap2UqYz";
pub const SWITCHBOARD_MAINNET_ETH_USD: &str = "QJc2HgGhdtW4e7zjvLB1TGRuwEpTre2agU5Lap2UqYz";

pub const SWITCHBOARD_DEVNET_BTC_IV: &str = "CX1PvW4qUDy4PPq8egnMVCbVJt8TcPCt7WCZuwmvCfo7";
pub const SWITCHBOARD_MAINNET_BTC_IV: &str = "CX1PvW4qUDy4PPq8egnMVCbVJt8TcPCt7WCZuwmvCfo7";

pub const SWITCHBOARD_DEVNET_ETH_IV: &str = "4AGPMUEfBCSNqVd4Y6veHAep6VPtrkMa89rBhPqMYegz";
pub const SWITCHBOARD_MAINNET_ETH_IV: &str = "4AGPMUEfBCSNqVd4Y6veHAep6VPtrkMa89rBhPqMYegz";

// #[cfg(not(feature = "devnet"))]
// const BTC_SPOT_ORACLE: &str = SWITCHBAORD_MAINNET_BTC_USD;
// #[cfg(not(feature = "devnet"))]
// const BTC_IV_ORACLE: &str = SWITCHBOARD_MAINNET_BTC_IV;
// #[cfg(not(feature = "devnet"))]
// const ETH_SPOT_ORACLE: &str = SWITCHBOARD_MAINNET_ETH_USD;
// #[cfg(not(feature = "devnet"))]
// const ETH_IV_ORACLE: &str = SWITCHBOARD_MAINNET_ETH_IV;

// #[cfg(feature = "devnet")]
// const BTC_SPOT_ORACLE: &str = SWITCHBOARD_DEVNET_BTC_USD;
// #[cfg(feature = "devnet")]
// const BTC_IV_ORACLE: &str = SWITCHBOARD_DEVNET_BTC_IV;
// #[cfg(feature = "devnet")]
// const ETH_SPOT_ORACLE: &str = SWITCHBOARD_DEVNET_ETH_USD;
// #[cfg(feature = "devnet")]
// const ETH_IV_ORACLE: &str = SWITCHBOARD_DEVNET_ETH_IV;

pub const USDC_DECIMALS: u32 = 6u32;

// Constant for the AMM
pub const DELTA_LIMIT: f32 = 0.05; // delta limit for hedging
pub const TRADE_CAPACITY: f32 = 0.25;
pub const NSTEP: i64 = 100; //nStep to generate orderbook
pub const NQUOTES: i64 = 5; //to place on orderbook
pub const MAX_ORDERBOOK_SIZE: f32 = 10.0;
pub const PRICE_MOVE: f32 = 0.005; // price change tolerance for updating amm orders
pub const ORDER_LEVELS: usize = 20;

// Some useful datetime constants
pub const SECONDS_IN_MINUTE: u64 = 60;
pub const MINUTES_IN_HOUR: u64 = 60;
pub const HOURS_IN_DAY: u64 = 24;
pub const DAYS_IN_WEEK: u64 = 7;
pub const DAYS_IN_STANDARD_YEAR: u64 = 365;
pub const SECS_IN_HOUR: u64 = SECONDS_IN_MINUTE * MINUTES_IN_HOUR;
pub const SECS_IN_DAY: u64 = SECS_IN_HOUR * HOURS_IN_DAY;
pub const SECS_IN_STANDARD_YEAR: u64 = SECS_IN_DAY * DAYS_IN_STANDARD_YEAR;

// Constant for the margin calculation
pub const STRESS: f32 = 0.3;
pub const STEP: u8 = 5;

pub const LIQUIDATION: f32 = 0.9;
pub const LIQUIDATION_SLIPPAGE: f32 = 1.0;
