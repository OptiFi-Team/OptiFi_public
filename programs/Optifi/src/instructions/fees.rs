use crate::constants::FEE;
use anchor_lang::CpiContext;
use anchor_spl::token;
use anchor_spl::token::Transfer;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint_deprecated::ProgramResult;

pub fn pay_fees<'a, 'b, 'c, 'info>(
    notional: u64,
    token_program: AccountInfo<'info>,
    payer_account: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    fee_account: AccountInfo<'info>,
    signer_seeds: Option<&'a [&'b [&'c [u8]]]>,
) -> ProgramResult {
    let transfer_context: CpiContext<'a, 'b, 'c, 'info, Transfer<'info>>;
    let transfer = Transfer {
        from: payer_account,
        to: fee_account,
        authority,
    };
    match signer_seeds {
        Some(s) => transfer_context = CpiContext::new_with_signer(token_program, transfer, s),
        None => transfer_context = CpiContext::new(token_program, transfer),
    }
    let amount = notional * FEE as u64;
    token::transfer(transfer_context, amount)?;
    Ok(())
}
