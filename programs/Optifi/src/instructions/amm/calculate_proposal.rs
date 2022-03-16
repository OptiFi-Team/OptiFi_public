use crate::constants::SECS_IN_STANDARD_YEAR;
use crate::errors::ErrorCode;
use crate::financial::amm::{
    calculate_amm_quote_price, calculate_amm_size_v2, calculate_single_amm_price,
};
use crate::financial::{delta_wrapper, OrderSide};
use crate::state::AmmState;
use crate::state::{AmmAccount, MarginStressAccount};
use crate::{f_to_u_repr, fvec_to_uvec_repr, i_to_f_repr, u_to_f_repr};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CalculateAmmProposal<'info> {
    // /// get the exchange and markets
    // pub optifi_exchange: Account<'info, Exchange>,
    #[account()]
    pub margin_stress_account: ProgramAccount<'info, MarginStressAccount>,

    /// the amm to which user will deposits funds
    #[account(mut, constraint = amm.optifi_exchange ==  margin_stress_account.optifi_exchange)]
    pub amm: ProgramAccount<'info, AmmAccount>,
    // #[account(address = token::ID)]
    // pub token_program: AccountInfo<'info>,

    // /// Clock to get the timestamp
    // pub clock: Sysvar<'info, Clock>,
}

/// Calc amm order proposals
pub fn handler(ctx: Context<CalculateAmmProposal>) -> ProgramResult {
    // ** hidden code **

    Ok(())
}
