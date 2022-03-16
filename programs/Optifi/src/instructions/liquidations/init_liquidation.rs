use crate::ceil;
use crate::constants::LIQUIDATION;
use crate::errors::ErrorCode;
use crate::state::{LiquidationState, LiquidationStatus, MarginStressAccount, UserAccount};
use anchor_lang::prelude::*;
use anchor_spl::token::accessor;

#[derive(Accounts)]
pub struct InitializeLiquidation<'info> {
    pub optifi_exchange: AccountInfo<'info>,

    #[account(mut, constraint= user_account.user_margin_account_usdc == user_margin_account_usdc.key())]
    pub user_account: ProgramAccount<'info, UserAccount>,

    /// user's margin account whose authority is user account(pda)
    pub user_margin_account_usdc: AccountInfo<'info>,

    #[account(mut, constraint = liquidation_state.user_account == user_account.key())]
    pub liquidation_state: ProgramAccount<'info, LiquidationState>,
}
/// Initialize liquidation for user
pub fn handler(ctx: Context<InitializeLiquidation>) -> ProgramResult {
    // ** hidden code **

    Ok(())
}
