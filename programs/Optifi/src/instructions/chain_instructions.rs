use crate::constants::SECS_IN_STANDARD_YEAR;
use crate::errors::ErrorCode;
use crate::financial::instruments::{ExpiryType, InstrumentType};
use crate::financial::{
    get_asset_to_usd_spot, get_iv, get_strikes, verify_switchboard_account, Asset, Chain, Duration,
    OracleDataType,
};
use crate::state::{Exchange, InstrumentCommon, InstrumentUnique};
use crate::utils::PREFIX_INSTRUMENT;
use anchor_lang::prelude::*;
use solana_program::{log::sol_log_compute_units, pubkey::Pubkey};
use std::convert::TryFrom;
use std::mem::size_of;

/// give some buffer when allocating account space
const ACCOUNT_TAIL: usize = 10;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ChainData {
    /// underlying asset
    pub asset: u8, // 1 bytes
    /// option or future
    pub instrument_type: u8, // 1 bytes
    /// expiry date of the instrument, unix timestamp
    pub expiry_date: u64, // 8 bytes
    /// Duration type
    pub duration: u8, // 1 bytes
    /// Start date, as a unix timestamp
    pub start: u64, // 8 bytes
    /// Is this a perpetual contract? Only valid for futures
    pub expiry_type: u8, // 1 byte
    /// The market authority for this instrument, Do we need this?
    pub authority: Pubkey, // 32 bytes
    /// contract size percentage: 1 means actually 0.01
    pub contract_size: u64,
    pub instrument_idx: u8,
}

pub fn chain_data_to_seed_string(data: &ChainData) -> String {
    let str = data.asset.to_string()
        + data.instrument_type.to_string().as_str()
        + data.expiry_type.to_string().as_str()
        + data.expiry_date.to_string().as_str()
        + data.instrument_idx.to_string().as_str();
    msg!("Asset is {}, instrument type is {}, expiry type is {}, idx is {}, expiry date str is {}, seed str is {}",
    data.asset, data.instrument_type, data.expiry_type, data.instrument_idx, data.expiry_date.to_string(), str);
    str
}

#[derive(Accounts)]
#[instruction(bump: u8, data: ChainData)]
pub struct CreateInstrument<'info> {
    #[account(mut)]
    pub optifi_exchange: ProgramAccount<'info, Exchange>,
    #[account(init,
    seeds=[PREFIX_INSTRUMENT.as_bytes(),
    optifi_exchange.key().as_mut(),
    chain_data_to_seed_string(&data).as_bytes(),
    ], payer=payer, bump=bump, space=size_of::<Chain>()+ACCOUNT_TAIL)]
    pub instrument: ProgramAccount<'info, Chain>,

    #[account(mut, signer)]
    pub payer: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    // oracle feed account for spot price of the instrument's underlying asset
    pub asset_spot_price_oracle_feed: AccountInfo<'info>,
    // oracle feed account for iv of the instrument's underlying asset
    pub asset_iv_oracle_feed: AccountInfo<'info>,
    // // oracle feed account for usdc spot price
    // #[account(constraint = verify_switchboard_account(Asset::USDC, OracleDataType::Spot, usdc_spot_price_oracle_feed.key, &optifi_exchange))]
    // pub usdc_spot_price_oracle_feed: AccountInfo<'info>,
    /// Clock to get the timestamp
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<CreateInstrument>, _bump: u8, data: ChainData) -> ProgramResult {
    msg!(
        "Asset is {}, {:?}",
        data.asset,
        Asset::try_from(data.asset).unwrap()
    );

    // Calculate the duration as percentage of a year
    let now = Clock::get().unwrap().unix_timestamp as u64;
    let time_to_maturity = data.expiry_date - now;
    let time_to_maturity = time_to_maturity as f32 / SECS_IN_STANDARD_YEAR as f32;

    msg!(
        "Duration is {}, {:?}",
        data.duration as f32,
        Duration::try_from(data.asset).unwrap()
    );

    // read spot price and iv from oracle accounts
    let asset_spot_price_oracle_feed = &ctx.accounts.asset_spot_price_oracle_feed;
    let asset_iv_oracle_feed = &ctx.accounts.asset_iv_oracle_feed;

    let optifi_exchange = &mut ctx.accounts.optifi_exchange;

    if !(verify_switchboard_account(
        Asset::try_from(data.asset).unwrap(),
        OracleDataType::Spot,
        asset_spot_price_oracle_feed.key,
        optifi_exchange,
    ) && verify_switchboard_account(
        Asset::try_from(data.asset).unwrap(),
        OracleDataType::IV,
        asset_iv_oracle_feed.key,
        optifi_exchange,
    )) {
        return Err(ErrorCode::IncorrectOracleAccount.into());
    }

    let spot_price_from_oracle = get_asset_to_usd_spot(asset_spot_price_oracle_feed);
    let iv_from_oracle = get_iv(asset_iv_oracle_feed);

    // calculate the strikes
    msg!(
        "Strikes input - \nSpot: {}\nIV: {}\nYear to maturity: {}\n",
        spot_price_from_oracle as f32,
        iv_from_oracle as f32,
        time_to_maturity as f32
    );
    let strikes = get_strikes(
        spot_price_from_oracle as f32,
        iv_from_oracle as f32,
        time_to_maturity as f32,
    );
    sol_log_compute_units();
    msg!("Strikes are {} ", strikes.map(|i| i.to_string()).join(", "));
    //sol_log_compute_units();
    msg!("Before loop");
    sol_log_compute_units();

    let strike = strikes[data.instrument_idx as usize];
    msg!(
        "Creating instrument {}, with strike {}",
        data.instrument_idx + 1,
        strike
    );
    let instrument = &mut ctx.accounts.instrument;
    instrument.strike = strike as u64;
    instrument.asset = data.asset;
    msg!(
        "Set instrument.asset = {}, instrument.asset = {}",
        data.asset,
        instrument.asset
    );
    instrument.instrument_type = InstrumentType::try_from(data.instrument_type).unwrap();
    instrument.expiry_date = data.expiry_date;
    instrument.duration = Duration::try_from(data.duration).unwrap();
    instrument.start = data.start;
    instrument.expiry_type = ExpiryType::try_from(data.expiry_type).unwrap();
    instrument.authority = data.authority;
    instrument.contract_size = data.contract_size;
    sol_log_compute_units();

    msg!("instrument {} created successfully", data.instrument_idx);

    // Add the instrument to exchange
    let common = InstrumentCommon {
        asset: Asset::try_from(instrument.asset).unwrap(),
        expiry_date: instrument.expiry_date,
        expiry_type: instrument.expiry_type,
    };

    let unique = InstrumentUnique {
        strike: instrument.strike as u32,
        instrument_pubkeys: [instrument.key(), instrument.key()],
    };

    if let Some((common_index, instrument_common)) = optifi_exchange
        .instrument_common
        .iter()
        .enumerate()
        .find(|(_, &ic)| ic == common)
    {
        if let Some(instrument_unique) = optifi_exchange.instrument_unique[common_index]
            .iter_mut()
            .find(|iu| iu.strike == instrument.strike as u32)
        {
            instrument_unique.instrument_pubkeys[data.instrument_type as usize] = instrument.key();
        } else {
            optifi_exchange.instrument_unique[common_index].push(unique);
        }
    } else {
        optifi_exchange.instrument_common.push(common);
        optifi_exchange.instrument_unique.push(vec![unique]);
    }

    Ok(())
}

#[derive(Accounts)]
pub struct CleanInstrument<'info> {
    #[account(mut)]
    pub optifi_exchange: ProgramAccount<'info, Exchange>,

    /// Clock to get the timestamp
    pub clock: Sysvar<'info, Clock>,
}

pub fn clean(ctx: Context<CleanInstrument>) -> ProgramResult {
    let optifi_exchange = &mut ctx.accounts.optifi_exchange;

    let now = Clock::get().unwrap().unix_timestamp as u64;

    let len_1 = optifi_exchange.instrument_common.len();

    let mut instrument_common = vec![];
    let mut instrument_unique = vec![];

    for (index, ic) in optifi_exchange.instrument_common.iter().enumerate() {
        if ic.expiry_date > now {
            instrument_common.push(*ic);
            instrument_unique.push(optifi_exchange.instrument_unique[index].clone());
        }
    }

    let len_2 = optifi_exchange.instrument_common.len();

    msg!(
        "Clean {} expired instrument groups in exchange, remaining {} valid instrument groups",
        len_1 - len_2,
        len_2
    );

    optifi_exchange.instrument_common = instrument_common;
    optifi_exchange.instrument_unique = instrument_unique;

    Ok(())
}
