use crate::utils::{
    get_optifi_market_mint_auth_pda, PREFIX_OPTIFI_MARKET_MINT_AUTH, PREFIX_USER_ACCOUNT,
};
use anchor_lang::{prelude::*, solana_program::program::invoke_signed};
use solana_program::log::sol_log_compute_units;

pub fn mint_instrument_token_for_user<'info>(
    token_mint: &AccountInfo<'info>,
    to_account: &AccountInfo<'info>,
    amount: u64,
    token_program: &AccountInfo<'info>,
    program_id: &Pubkey,
    exchange: &Pubkey,
    coin_mint_authority: &AccountInfo<'info>,
) -> ProgramResult {
    // TODO
    // let seeds = [
    //     PREFIX.as_bytes(),
    //     exchange.key().as_ref(),
    //     &optifi_market.optifi_market_id.to_be_bytes(),
    // ];
    let (coin_mint_authority_key, bump) = get_optifi_market_mint_auth_pda(exchange, program_id);
    let mint_to_ix = spl_token::instruction::mint_to(
        token_program.key,
        token_mint.key,
        to_account.key,
        &coin_mint_authority.key(),
        &[&coin_mint_authority_key],
        amount,
    )?;
    // msg uses about 7000 computing units, so better not use it
    // msg!("passed coin_mint_authority: {}", coin_mint_authority.key);
    // msg!("gened coin_mint_authority_key: {}", coin_mint_authority_key);
    // msg!("Created mint_to instruction");
    invoke_signed(
        &mint_to_ix,
        &[
            token_mint.clone(),
            to_account.clone(),
            coin_mint_authority.to_account_info(), // optifi_market is the pda that has mint authority
            token_program.clone(),
        ],
        &[&[
            PREFIX_OPTIFI_MARKET_MINT_AUTH.as_bytes(),
            exchange.as_ref(),
            &[bump],
        ]],
    )
}

pub fn burn_instrument_token_for_user<'info>(
    token_mint: &AccountInfo<'info>,
    to_account: &AccountInfo<'info>,
    user: Pubkey,
    user_account: &AccountInfo<'info>,
    user_account_bump: u8,
    amount: u64,
    token_program: &AccountInfo<'info>,
    exchange: &Pubkey,
) -> ProgramResult {
    let burn_ix = spl_token::instruction::burn(
        token_program.key,
        to_account.key,
        token_mint.key,
        &user_account.key(),
        &[&user_account.key()],
        amount,
    )?;

    msg!("Created burn instruction");
    invoke_signed(
        &burn_ix,
        &[
            to_account.clone(),
            token_mint.clone(),
            user_account.clone(), // optifi_market is the pda that has mint authority
            token_program.clone(),
        ],
        &[&[
            PREFIX_USER_ACCOUNT.as_bytes(),
            exchange.as_ref(),
            user.key().as_ref(),
            &[user_account_bump],
        ]],
    )
}
