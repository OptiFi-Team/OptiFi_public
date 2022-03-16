use crate::errors::ErrorCode;
use crate::state::user_account::{AccountState, UserAccount};
use crate::state::{LiquidationState, LiquidationStatus};
use crate::utils::{PREFIX_LIQUIDATION_STATE, PREFIX_USER_ACCOUNT};
use anchor_lang::{prelude::*, AnchorDeserialize};
use anchor_spl::token::Token;
use solana_program::program::invoke;
use std::mem::size_of;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitUserAccountBumpSeeds {
    pub user_account: u8,
    pub liquidation_account: u8,
}

#[derive(Accounts)]
#[instruction(bump: InitUserAccountBumpSeeds)]
pub struct InitializeUserAccount<'info> {
    /// the optifi_exchange account
    pub optifi_exchange: AccountInfo<'info>,
    /// the user's optifi account to be initialized
    #[account(init,
        seeds=[PREFIX_USER_ACCOUNT.as_bytes(), optifi_exchange.key().as_ref(), owner.key().as_ref()],
        payer=owner,
        bump=bump.user_account,
        space=3200 // 96+16+1+36*48+1+1+80+36*32+36
    )]
    pub user_account: ProgramAccount<'info, UserAccount>,

    #[account(init, 
        payer=owner, 
        seeds=[
            PREFIX_LIQUIDATION_STATE.as_bytes(),
            optifi_exchange.key().as_ref(),
            user_account.key().as_ref()
        ], 
        bump=bump.liquidation_account,
        space=1600 // 32+1+36*32+32*8
    )]
    pub liquidation_account: ProgramAccount<'info, LiquidationState>,

    /// the account user deposits spl token into
    #[account(mut, constraint= !user_margin_account_usdc.data_is_empty() && user_margin_account_usdc.lamports() > 0)]
    pub user_margin_account_usdc: AccountInfo<'info>,

    #[account(mut, signer, constraint= owner.data_is_empty() && owner.lamports() > 0)]
    pub owner: AccountInfo<'info>,
    #[account(mut, signer)]
    pub payer: AccountInfo<'info>,
    // #[account(address = system_program::ID)]
    // pub system_program: AccountInfo<'info>,
    // pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    // #[account(address = system_program::ID)]
    // pub system_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<InitializeUserAccount>,
    bump: InitUserAccountBumpSeeds,
) -> ProgramResult {
    let user_account = &mut ctx.accounts.user_account;

    if !matches!(user_account.state, AccountState::Uninitialized) {
        return Err(ErrorCode::AccountCannotInit.into());
    }

    ctx.accounts.liquidation_account.status = LiquidationStatus::Healthy;
    ctx.accounts.liquidation_account.user_account = user_account.key();

    user_account.optifi_exchange = ctx.accounts.optifi_exchange.key();
    user_account.owner = ctx.accounts.owner.key();
    user_account.state = AccountState::Initialized;
    user_account.bump = bump.user_account;
    user_account.is_in_liquidation = false;

    let user_margin_account_usdc = &ctx.accounts.user_margin_account_usdc;
    user_account.user_margin_account_usdc = user_margin_account_usdc.key();
    let _optifi_exchange = &ctx.accounts.optifi_exchange;

    // Transfer the ownership of user margin account to the user account (pda)
    // Note that the pda controls all spl token vaults of the user
    let token_program = &ctx.accounts.token_program;
    let payer = &ctx.accounts.payer;

    let owner_change_ix = spl_token::instruction::set_authority(
        token_program.key,
        user_margin_account_usdc.key,
        Some(&user_account.key()),
        spl_token::instruction::AuthorityType::AccountOwner,
        payer.key,
        &[&payer.key],
    )?;

    msg!("Calling the token program to transfer token account ownership...");
    invoke(
        &owner_change_ix,
        &[
            user_margin_account_usdc.clone(),
            payer.clone(),
            token_program.to_account_info(),
        ],
    )?;

    msg!("user account initialized successfully");
    Ok(())
}
