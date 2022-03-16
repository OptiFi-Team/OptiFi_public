use crate::state::user_account::UserAccount;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct Deposit<'info> {
    /// user account - also the pda that controls the user's spl token accounts
    #[account(mut, constraint= user_account.owner == user.key() && user_account.user_margin_account_usdc == user_margin_account_usdc.key())]
    pub user_account: Account<'info, UserAccount>,

    /// user's margin account whose authority is user account(pda)
    #[account(mut)]
    pub user_margin_account_usdc: AccountInfo<'info>,

    /// The mint for usdc margin account
    pub deposit_token_mint: AccountInfo<'info>,

    /// The user that owns the deposits
    #[account(signer)]
    pub user: AccountInfo<'info>,

    /// The token account with the tokens to be deposited
    #[account(mut)]
    pub deposit_source: AccountInfo<'info>,

    #[account(address = token::ID)]
    pub token_program: AccountInfo<'info>,
}

impl<'info> Deposit<'info> {
    fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        // self.vault_account

        CpiContext::new(
            self.token_program.clone(),
            Transfer {
                from: self.deposit_source.to_account_info(),
                to: self.user_margin_account_usdc.clone(),
                authority: self.user.clone(),
            },
        )
    }

    // fn note_mint_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
    //     CpiContext::new(
    //         self.token_program.clone(),
    //         MintTo {
    //             to: self.deposit_account.to_account_info(),
    //             mint: self.deposit_note_mint.to_account_info(),
    //             authority: self.market_authority.clone(),
    //         },
    //     )
    // }
}

/// Deposit tokens into a reserve
pub fn handler(ctx: Context<Deposit>, amount: u64) -> ProgramResult {
    // let reserve = &mut ctx.accounts.reserve;

    // reserve.deposit(amount);

    // TODO: check the dest token account owned by the user

    token::transfer(ctx.accounts.transfer_context(), amount)?;

    Ok(())
}
