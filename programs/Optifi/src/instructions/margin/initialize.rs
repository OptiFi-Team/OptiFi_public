
use crate::{state::MarginStressAccount, financial::Asset};
use crate::Exchange;
use anchor_lang::prelude::*;
use crate::utils::{ PREFIX_MARGIN_STRESS};


use std::convert::TryFrom;


#[derive(Accounts, Clone)]
#[instruction(bump: u8,asset:u8)]
pub struct InitMarginStressContext<'info> {
    pub optifi_exchange: ProgramAccount<'info, Exchange>,

    #[account(init, 
        seeds=[
            PREFIX_MARGIN_STRESS.as_bytes(),
            optifi_exchange.key().as_ref(),
            &[asset],
        ],
        payer=payer, bump=bump, space=3000)]
    pub margin_stress_account: ProgramAccount<'info, MarginStressAccount>,


    #[account(signer)]
    pub payer: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handle(ctx: Context<InitMarginStressContext>,bump:u8,asset:u8) -> ProgramResult {
    let optifi_exchange = &ctx.accounts.optifi_exchange;
    let margin_stress_account = &mut ctx.accounts.margin_stress_account;

    margin_stress_account.optifi_exchange = optifi_exchange.key();
    margin_stress_account.bump = bump;

    let asset =  Asset::try_from(asset).unwrap();

    margin_stress_account.asset=asset;

    let (instrument_pubkey, strikes, is_call, expiry_date) = optifi_exchange.get_instrument_data_with_asset(asset);

    margin_stress_account.instruments= instrument_pubkey;
    margin_stress_account.strikes  = strikes;
    margin_stress_account.is_call = is_call;
    margin_stress_account.expiry_date= expiry_date;

    let len = margin_stress_account.instruments.len();

    margin_stress_account.flags = vec![false;len];
    margin_stress_account.option_price = vec![0;len];
    margin_stress_account.intrinsic_value = vec![0;len];
    margin_stress_account.option_price_delta_in_stress_price = vec![vec![];len];


    Ok(())
}
