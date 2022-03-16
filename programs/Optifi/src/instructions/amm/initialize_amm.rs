use crate::financial::Duration;
use crate::state::{Exchange, AmmAccount};
use crate::utils::{ PREFIX_AMM};
use anchor_lang::prelude::*;
use anchor_spl::token::{ Mint, Token};
use std::convert::TryFrom;


#[derive(Accounts)]
#[instruction(bump: u8, data: InitializeAMMData)]
pub struct InitializeAMM<'info> {
    /// the optifi exchange
    pub optifi_exchange: ProgramAccount<'info, Exchange>,

    /// the amm account to be initialized
    #[account(init, 
        seeds=[
            PREFIX_AMM.as_bytes(),
            optifi_exchange.key().as_ref(),
            &[data.amm_idx],
        ],
        payer=payer, bump=bump, space=7000)]
    pub amm: ProgramAccount<'info, AmmAccount>,

    // /// amm's base token vault (Long position)
    // #[account(constraint = data.long_token_mint.contains(&long_token_vault_0.mint) && long_token_vault_0.owner == amm.key())]
    // pub long_token_vault_0: Account<'info, TokenAccount>,

    // /// amm's base token vault (Short position)
    // #[account(constraint = data.short_token_mint.contains(&short_token_vault_0.mint)  && short_token_vault_0.owner == amm.key())]
    // pub short_token_vault_0: Account<'info, TokenAccount>,

    // /// quote token mint address
    // pub quote_token_mint: Pubkey,
    // /// quote token account address
    // pub quote_token_vault: Pubkey,
    /// amm's quote token vault (USDC) - (aka. quote token vault)
    /// its token mint must be the usdc recogonized by optifi exchange, and owner is amm's liquidity auth pda
    pub usdc_token_vault: AccountInfo<'info>,
    /// amm's lp token mint address, whose mint authority is amm's liquidity auth pda
    pub lp_token_mint: Account<'info, Mint>,
    // /// amm's lp token vault, whose authority is amm's liquidity auth pda
    // #[account(constraint = accessor::mint(&lp_token_vault)? == lp_token_mint.key()
    //  && accessor::authority(&lp_token_vault)? == get_amm_liquidity_auth_pda(&optifi_exchange.key(), program_id).0)]
    // pub lp_token_vault: AccountInfo<'info>,
    /// The user that owns the deposits
    #[account(signer)]
    pub payer: AccountInfo<'info>,

    // /// The token account with the tokens to be deposited
    // #[account(mut)]
    // pub deposit_source: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeAMMData {
    /// idx of the amm
    pub amm_idx: u8,
    // /// quote token mint address
    // pub quote_token_mint: Pubkey,
    // /// quote token account address
    // pub quote_token_vault: Pubkey,
    // /// LP tokens for liquidity providers
    // pub lp_token_mint: Pubkey,
    /// amm capacity percentage (25 is actually 25%)
    pub amm_capacity: u64,
    /// bump seed used to derive this amm address
    pub bump: u8,
    /// underlying asset
    pub asset: u8,
    /// number of trading instruments
    pub num_instruments: u8,
    /// Duration type
    pub duration: u8, // 1 bytes
    /// the contract size *10000 (f_to_u_repr)
    pub contract_size: u64,
}

/// initialize amm handler
pub fn handler(ctx: Context<InitializeAMM>, bump: u8, data: InitializeAMMData) -> ProgramResult {
    // ** hidden code **

    Ok(())
}
