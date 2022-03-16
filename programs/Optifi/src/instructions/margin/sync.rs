use crate::errors::ErrorCode;
use crate::financial::{
    get_asset_to_usdc_spot, get_iv, verify_switchboard_account, Asset, OracleDataType,
};

use crate::f_to_u_repr;
use crate::state::MarginStressAccount;
use crate::state::MarginStressState;
use crate::Exchange;
use anchor_lang::prelude::*;

#[derive(Accounts, Clone)]
pub struct SyncMarginStressContext<'info> {
    /// optifi_exchange account
    pub optifi_exchange: ProgramAccount<'info, Exchange>,

    #[account(mut, constraint = margin_stress_account.optifi_exchange == optifi_exchange.key())]
    pub margin_stress_account: ProgramAccount<'info, MarginStressAccount>,

    // Oracle to get the spot price
    pub asset_feed: AccountInfo<'info>,
    pub usdc_feed: AccountInfo<'info>,
    pub iv_feed: AccountInfo<'info>,

    // Clock to get the timestamp
    pub clock: Sysvar<'info, Clock>,
}

pub fn handle(ctx: Context<SyncMarginStressContext>) -> ProgramResult {
    if ctx.accounts.margin_stress_account.state == MarginStressState::Sync {
    } else if ctx.accounts.margin_stress_account.state == MarginStressState::Available {
    } else {
        return Err(ErrorCode::WrongState.into());
    }

    let optifi_exchange = &ctx.accounts.optifi_exchange;
    let margin_stress_account = &mut ctx.accounts.margin_stress_account;

    let asset_feed = &ctx.accounts.asset_feed;
    let usdc_feed = &ctx.accounts.usdc_feed;
    let iv_feed = &ctx.accounts.iv_feed;

    let asset = margin_stress_account.asset;

    if !(verify_switchboard_account(asset, OracleDataType::Spot, asset_feed.key, optifi_exchange)
        && verify_switchboard_account(
            Asset::USDC,
            OracleDataType::Spot,
            usdc_feed.key,
            optifi_exchange,
        )
        && verify_switchboard_account(asset, OracleDataType::IV, iv_feed.key, optifi_exchange))
    {
        return Err(ErrorCode::IncorrectOracleAccount.into());
    }

    let spot_price = get_asset_to_usdc_spot(asset_feed, usdc_feed);
    let iv = get_iv(iv_feed);
    let now = Clock::get().unwrap().unix_timestamp as u64;

    margin_stress_account.spot_price = f_to_u_repr!(spot_price);
    margin_stress_account.iv = f_to_u_repr!(iv);
    margin_stress_account.timestamp = now;

    for flag in margin_stress_account.flags.iter_mut() {
        *flag = false;
    }
    margin_stress_account.state = MarginStressState::Calculate;

    Ok(())
}
