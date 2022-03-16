use crate::constants::{DELTA_LIMIT, PRICE_MOVE, SECS_IN_STANDARD_YEAR};
use crate::errors::ErrorCode;
use crate::financial::amm::total_amm_liquidity;
use crate::financial::option_delta_v2;
use crate::state::{AmmAccount, AmmState, MarginStressAccount, MarginStressState};
use crate::{f_to_i_repr, f_to_u_repr, u_to_f_repr, uvec_to_fvec_repr};
use anchor_lang::prelude::*;
use anchor_spl::token;
use solana_program::log::sol_log_compute_units;

#[derive(Accounts)]
pub struct CalculateAmmDelta<'info> {
    // /// get the exchange and markets
    // pub optifi_exchange: AccountInfo<'info>,
    #[account()]
    pub margin_stress_account: ProgramAccount<'info, MarginStressAccount>,

    /// the amm to which user will deposits funds
    #[account(mut, constraint = amm.optifi_exchange == margin_stress_account.optifi_exchange)]
    pub amm: ProgramAccount<'info, AmmAccount>,

    /// amm's quote token vault to get the USDC balance
    #[account(constraint = amm.quote_token_vault == quote_token_vault.key())]
    pub quote_token_vault: AccountInfo<'info>,
    // #[account(address = token::ID)]
    // pub token_program: AccountInfo<'info>,
    // /// Clock to get the timestamp
    // pub clock: Sysvar<'info, Clock>,
}

/// Calc amm delta
pub fn handler(ctx: Context<CalculateAmmDelta>) -> ProgramResult {
    // ** hidden code **

    Ok(())
}
