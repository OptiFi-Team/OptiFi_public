use crate::constants::{SECS_IN_STANDARD_YEAR, USDC_DECIMALS};

use crate::financial::margin_function;

use crate::state::MarginStressAccount;
use crate::state::UserAccount;
use crate::{ceil, i_to_f_repr, u_to_f_repr, Exchange};
use anchor_lang::prelude::*;
use solana_program::log::sol_log_compute_units;

#[derive(Accounts, Clone)]
pub struct MarginContext<'info> {
    /// optifi_exchange account
    pub optifi_exchange: ProgramAccount<'info, Exchange>,
    #[account(constraint = margin_stress_account.optifi_exchange == optifi_exchange.key())]
    pub margin_stress_account: ProgramAccount<'info, MarginStressAccount>,
    /// user's optifi account
    #[account(mut, constraint = user_account.optifi_exchange == optifi_exchange.key())]
    pub user_account: ProgramAccount<'info, UserAccount>,

    /// Clock to get the timestamp
    pub clock: Sysvar<'info, Clock>,
}

pub fn handle(ctx: Context<MarginContext>) -> ProgramResult {
    let optifi_exchange = &ctx.accounts.optifi_exchange;
    let margin_stress_account = &ctx.accounts.margin_stress_account;
    let user_account = &mut ctx.accounts.user_account;

    // let now = Clock::get().unwrap().unix_timestamp as u64;

    sol_log_compute_units();
    // Margin calculation
    let mut positions = vec![];

    for pubkey in &margin_stress_account.instruments {
        if let Some(p) = user_account
            .positions
            .iter()
            .find(|user_position| user_position.get_instrument() == pubkey)
        {
            positions.push(p.get_quantity());
        } else {
            positions.push(0);
        }
    }
    sol_log_compute_units();
    //

    let asset = margin_stress_account.asset;
    let now = margin_stress_account.timestamp;
    let expiry_date = optifi_exchange.get_expiry_date_with_asset(asset);

    let t = expiry_date
        .iter()
        .map(|d| (d - now) as f32 / SECS_IN_STANDARD_YEAR as f32)
        .collect::<Vec<f32>>();

    sol_log_compute_units();

    let margin_result = margin_function(
        positions,
        &t,
        &margin_stress_account.option_price,
        &margin_stress_account.intrinsic_value,
        &margin_stress_account.option_price_delta_in_stress_price,
    );
    let margin_result = margin_result.min(0);

    let amount_to_reserve = -margin_result as u64;

    sol_log_compute_units();
    msg!(
        "spot_price : {}, iv : {}, amount_to_reserve : {} ",
        i_to_f_repr!(margin_stress_account.spot_price),
        u_to_f_repr!(margin_stress_account.iv),
        amount_to_reserve
    );

    user_account.amount_to_reserve[asset as usize] = amount_to_reserve;

    Ok(())
}
