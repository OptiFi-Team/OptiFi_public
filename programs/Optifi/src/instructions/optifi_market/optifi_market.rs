use crate::errors::ErrorCode;
use crate::financial::chain::Chain;
use crate::state::exchange::{Exchange, OptifiMarketKeyData};
use crate::state::OptifiMarket;
use crate::utils::{get_optifi_market_mint_auth_pda, PREFIX_OPTIFI_MARKET};
use anchor_lang::solana_program::program_option::COption;
use anchor_lang::{prelude::*, AnchorDeserialize};
use anchor_spl::token::Mint;
use serum_dex::state::{Market, ToAlignedBytes};
use solana_program::account_info::AccountInfo;

fn verify_mint_authority(
    exchange: &ProgramAccount<'_, Exchange>,
    mint: &Account<'_, Mint>,
    program_id: &Pubkey,
) -> bool {
    let mint_auth_pda: COption<Pubkey> =
        COption::Some(get_optifi_market_mint_auth_pda(&exchange.key(), program_id).0);
    msg!(
        "Mint authority is {}, should be {}",
        mint.mint_authority.unwrap(),
        mint_auth_pda.unwrap()
    );
    mint.mint_authority == mint_auth_pda
}

/// Create an OptiFi market and list an instrument on the market,
/// and add the market to exchange market list
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct CreateOptifiMarket<'info> {
    /// The optifi market to be created
    #[account(init,
    seeds=[PREFIX_OPTIFI_MARKET.as_bytes(),
    exchange.clone().key().as_ref(),
    &exchange.markets.len().checked_add(1).ok_or(ErrorCode::NumericalOverflowError)?.to_be_bytes(),
    ], payer=payer, bump=bump, space=std::mem::size_of::<OptifiMarket>() + 8)]
    pub optifi_market: ProgramAccount<'info, OptifiMarket>,
    /// OptiFi Exchange account
    #[account(mut)]
    pub exchange: ProgramAccount<'info, Exchange>,
    /// the serum market on which the instrument will be listed
    pub serum_market: AccountInfo<'info>,
    /// The instrument to be listed
    #[account(mut, constraint = !instrument.is_listed_on_market && instrument.expiry_date as i64 > clock.unix_timestamp)]
    pub instrument: ProgramAccount<'info, Chain>,
    /// the mint address of spl token for buyers of the instrument,
    /// it should be the base currency for the serum orderbook
    /// it's mint authority should be this optifi_market_mint_auth pda
    #[account(constraint = verify_mint_authority(&exchange, &long_spl_token_mint, program_id))]
    pub long_spl_token_mint: Account<'info, Mint>,
    /// the mint address of spl token for sellers of the instrument,
    /// it's mint authority should be this optifi_market_mint_auth pda
    #[account(constraint = verify_mint_authority(&exchange, &short_spl_token_mint, program_id))]
    pub short_spl_token_mint: Account<'info, Mint>,
    #[account(mut, signer)]
    pub payer: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handle_create_optifi_market(ctx: Context<CreateOptifiMarket>, bump: u8) -> ProgramResult {
    let exchange = &mut ctx.accounts.exchange;
    let optifi_market = &mut ctx.accounts.optifi_market;
    let serum_market = &ctx.accounts.serum_market;
    let instrument = &mut ctx.accounts.instrument;
    let long_spl_token_mint = &ctx.accounts.long_spl_token_mint;
    let short_spl_token_mint = &ctx.accounts.short_spl_token_mint;

    // validate the long_spl_token_mint the user passed is exactly the same coin mint of the serum_market
    let serum_state = Market::load(serum_market, serum_market.owner)?;
    let serum_coin_mint = serum_state.coin_mint;
    if serum_coin_mint != long_spl_token_mint.key().to_aligned_bytes() {
        return Err(ErrorCode::IncorrectCoinMint.into());
    }

    optifi_market.optifi_market_id = exchange
        .markets
        .len()
        .checked_add(1)
        .ok_or(ErrorCode::NumericalOverflowError)? as u16;

    optifi_market.serum_market = serum_market.key();
    optifi_market.instrument = instrument.key();
    optifi_market.instrument_long_spl_token = long_spl_token_mint.key();
    optifi_market.instrument_short_spl_token = short_spl_token_mint.key();

    // set the optifi market is_stopped to false, which means the market is running
    optifi_market.is_stopped = false;
    optifi_market.bump = bump;

    msg!(
        "instrument listed on market before set is {}",
        instrument.is_listed_on_market
    );
    // mark the instrument as listed, which means it cannot be listed on other markets again
    instrument.is_listed_on_market = true;

    msg!("About to push market to exchange");

    // add the new optifi market to optifi exchange's markets(key data) list
    exchange.markets.push(OptifiMarketKeyData {
        optifi_market_pubkey: optifi_market.key(),
        // optifi_market_id: optifi_market.optifi_market_id,
        // serum_market: optifi_market.serum_market,
        // instrument: optifi_market.instrument,
        expiry_date: instrument.expiry_date,
        is_stopped: optifi_market.is_stopped,
    });

    msg!("optifi market is created and the instrument is listed on the market successfully");
    Ok(())
}

/// Update a stopped OptiFi market with a new instrument which will be listed on it
#[derive(Accounts)]
#[instruction()]
pub struct UpdateOptifiMarket<'info> {
    /// OptiFi Exchange account
    #[account(mut)]
    pub exchange: Account<'info, Exchange>,
    /// The optifi market to be updated
    #[account(mut, constraint = optifi_market.is_stopped)]
    pub optifi_market: Account<'info, OptifiMarket>,
    /// the serum market on which the instrument will list
    pub serum_market: AccountInfo<'info>,
    /// The instrument to be listed
    #[account(mut, constraint = !instrument.is_listed_on_market && instrument.expiry_date as i64 > clock.unix_timestamp)]
    pub instrument: Account<'info, Chain>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handle_update_optifi_market(ctx: Context<UpdateOptifiMarket>) -> ProgramResult {
    let exchange = &mut ctx.accounts.exchange;
    let optifi_market = &mut ctx.accounts.optifi_market;

    // ==============================================================================
    // TODO: make sure the serum market(orderbook) is zeroed before re-use the market for a new instrument
    // ==============================================================================

    let instrument = &mut ctx.accounts.instrument;
    optifi_market.instrument = instrument.key();

    // set the optifi market is_stopped to false, which means the market is running
    optifi_market.is_stopped = false;
    instrument.is_listed_on_market = true;

    // also update the markets(key data) list in optifi exchange
    for market in &mut exchange.markets.clone() {
        if market.optifi_market_pubkey == optifi_market.key() {
            // market.instrument = instrument.key();
            market.expiry_date = instrument.expiry_date;
            market.is_stopped = false;
        }
    }
    msg!("the market is updated with new the instrument listed on it successfully");
    Ok(())
}
