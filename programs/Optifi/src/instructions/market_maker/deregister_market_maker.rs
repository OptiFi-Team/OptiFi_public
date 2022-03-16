use crate::state::market_maker_account::MarketMakerAccount;
use crate::state::UserAccount;
use crate::Exchange;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct DeregisterMarketMaker<'info> {
    pub optifi_exchange: ProgramAccount<'info, Exchange>,
    pub user_account: ProgramAccount<'info, UserAccount>,

    #[account(mut, constraint = market_maker.user_account == user_account.key())]
    pub market_maker: ProgramAccount<'info, MarketMakerAccount>,

    #[account(signer, constraint = owner.key() == user_account.owner)]
    pub owner: AccountInfo<'info>,
}

pub fn handler(ctx: Context<DeregisterMarketMaker>) -> ProgramResult {
    // ** hidden code **

    Ok(())
}
