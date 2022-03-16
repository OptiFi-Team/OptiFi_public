use crate::state::{Exchange, UserAccount, UserPosition};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CleanInstrumentForUser<'info> {
    #[account(mut, constraint = user_account.optifi_exchange == optifi_exchange.key())]
    pub user_account: ProgramAccount<'info, UserAccount>,

    pub optifi_exchange: ProgramAccount<'info, Exchange>,
}

pub fn handle(ctx: Context<CleanInstrumentForUser>) -> ProgramResult {
    let user_account = &mut ctx.accounts.user_account;

    let optifi_exchange = &ctx.accounts.optifi_exchange;

    let len_1 = user_account.positions.len();

    let instruments = optifi_exchange.get_instrument_pubkey(None);

    let user_positions = user_account
        .positions
        .clone()
        .into_iter()
        .filter(|i| i.is_valid(&instruments))
        .collect::<Vec<UserPosition>>();

    let len_2 = user_account.positions.len();

    msg!(
        "Clean {} expired instruments in user positions, remaining {} valid instruments",
        len_1 - len_2,
        len_2
    );

    user_account.positions = user_positions;

    Ok(())
}
