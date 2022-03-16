use crate::constants::{SECS_IN_STANDARD_YEAR, STEP, STRESS};
use crate::errors::ErrorCode;
use crate::financial::stress_function;

use crate::state::MarginStressAccount;
use crate::state::MarginStressState;
use crate::Exchange;
use crate::{f_to_i_repr, f_to_u_repr, fvec_to_ivec_repr, u_to_f_repr};
use anchor_lang::prelude::*;
use solana_program::log::sol_log_compute_units;

#[derive(Accounts, Clone)]
pub struct CalculateMarginStressContext<'info> {
    /// optifi_exchange account
    pub optifi_exchange: ProgramAccount<'info, Exchange>,

    #[account(mut, constraint = margin_stress_account.optifi_exchange == optifi_exchange.key())]
    pub margin_stress_account: ProgramAccount<'info, MarginStressAccount>,
}

pub fn handle(ctx: Context<CalculateMarginStressContext>) -> ProgramResult {
    if ctx.accounts.margin_stress_account.state != MarginStressState::Calculate {
        return Err(ErrorCode::WrongState.into());
    }

    let optifi_exchange = &ctx.accounts.optifi_exchange;
    let margin_stress_account = &mut ctx.accounts.margin_stress_account;

    let now = margin_stress_account.timestamp;
    let iv = u_to_f_repr!(margin_stress_account.iv);
    let spot_price = u_to_f_repr!(margin_stress_account.spot_price);

    sol_log_compute_units();

    for _ in 0..2 {
        let index = margin_stress_account
            .flags
            .iter()
            .position(|&x| x == false)
            .unwrap();
        //
        let instrument = margin_stress_account.instruments[index];
        let (instrument_data, strike, is_call) =
            optifi_exchange.get_instrument_data(&instrument).unwrap();

        let time_to_maturity = instrument_data.expiry_date - now;
        let time_to_maturity = time_to_maturity * 10_u64.pow(6) / SECS_IN_STANDARD_YEAR;
        let time_to_maturity = time_to_maturity as f32 / 10_u64.pow(6) as f32;

        let strikes = vec![strike as f32];
        let is_call = vec![is_call as u8];
        let t = vec![time_to_maturity];

        msg!(
            "spot_price {}, strikes {:?}, iv {}, t {:?}, is_call {:?}",
            spot_price,
            strikes,
            iv,
            &t,
            is_call
        );

        //
        let stress_function_res =
            stress_function(spot_price, strikes, iv, 0.0, 0.0, &t, STRESS, is_call, STEP);

        // Done
        margin_stress_account.flags[index] = true;
        margin_stress_account.option_price[index] =
            f_to_u_repr!(stress_function_res.price[0][0].to_owned());
        margin_stress_account.intrinsic_value[index] =
            f_to_u_repr!(stress_function_res.intrinsic_value[0][0].to_owned());
        margin_stress_account.option_price_delta_in_stress_price[index] =
            fvec_to_ivec_repr!(stress_function_res.stress_price_delta[0].to_owned());

        sol_log_compute_units();
    }
    //
    if margin_stress_account.flags.iter().all(|&flag| flag == true) {
        msg!("The proposal calculation is finished, changing the margin_stress_account state...");

        for flag in margin_stress_account.flags.iter_mut() {
            *flag = false;
        }
        margin_stress_account.move_to_next_state();
    }

    Ok(())
}
