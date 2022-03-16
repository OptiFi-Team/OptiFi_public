use crate::errors::ErrorCode;
use crate::state::OptifiMarket;
use crate::utils::{
    get_amm_liquidity_auth_pda, get_serum_market_auth_pda, PREFIX_AMM_LIQUIDITY_AUTH,
    PREFIX_SERUM_MARKET_AUTH, PREFIX_SERUM_OPEN_ORDERS,
};
use anchor_lang::{prelude::*, AnchorDeserialize};
use serum_dex::instruction::init_open_orders;
use serum_dex::state::OpenOrders;
use solana_program::program::invoke_signed;

/// Initialize an open orders account for the amm to place orders on this optifi market
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct InitAMMOnOptifiMarket<'info> {
    pub optifi_exchange: AccountInfo<'info>,
    /// the amm to init
    // #[account(constraint = amm.optifi_exchange == optifi_exchange.key())]
    // pub amm: ProgramAccount<'info, AMM>,
    #[account(constraint = get_amm_liquidity_auth_pda(optifi_exchange.key, program_id).0 == amm_authority.key())]
    pub amm_authority: AccountInfo<'info>,
    /// The account to use for placing orders on the DEX
    #[account(init,
            seeds = [
                PREFIX_SERUM_OPEN_ORDERS.as_bytes(),
                optifi_exchange.key().as_ref(),
                serum_market.key().as_ref(),
                amm_authority.key().as_ref(),
            ],
            bump = bump,
            payer = payer,
            owner = serum_dex_program_id.key(),
            space = std::mem::size_of::<OpenOrders>() + 12,
            )] // TODO: figure out skip rent_exempt ???
    pub serum_open_orders: AccountInfo<'info>,
    /// The optifi market to initialize for
    #[account( constraint = !optifi_market.is_stopped)]
    pub optifi_market: Account<'info, OptifiMarket>,
    /// the serum market the optifi market is using
    pub serum_market: AccountInfo<'info>,
    /// serum market authority which is required when init open orders if it is Some()
    pub serum_market_authority: AccountInfo<'info>,
    // pub clock: Sysvar<'info, Clock>,
    pub serum_dex_program_id: AccountInfo<'info>,
    #[account(signer)]
    pub payer: AccountInfo<'info>,
    // pub pda: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handle_init_amm_on_optifi_market(
    ctx: Context<InitAMMOnOptifiMarket>,
    _bump: u8,
) -> ProgramResult {
    let open_orders = &ctx.accounts.serum_open_orders;
    let serum_market = &ctx.accounts.serum_market;
    let serum_market_authority = &ctx.accounts.serum_market_authority;
    let serum_dex_program_id = &ctx.accounts.serum_dex_program_id;
    let optifi_exchange = &ctx.accounts.optifi_exchange;
    let rent = &ctx.accounts.rent.to_account_info();
    let amm_authority = &ctx.accounts.amm_authority;

    let init_open_orders_ix = init_open_orders(
        serum_dex_program_id.key,
        &open_orders.key,
        &amm_authority.key(),
        &serum_market.key,
        Some(serum_market_authority.key),
    )?;

    // _market_auth should be same as serum_market_authority.key
    let (_market_auth, bump) = get_serum_market_auth_pda(&optifi_exchange.key(), ctx.program_id);
    let (_, bump2) = get_amm_liquidity_auth_pda(&optifi_exchange.key(), ctx.program_id);

    if _market_auth.key() != serum_market_authority.key() {
        msg!(
            "Market auth is {}, serum market authority is {}",
            _market_auth.key(),
            serum_market_authority.key()
        );
        return Err(ErrorCode::InvalidSerumAuthority.into());
    }
    msg!("Created init_open_orders instruction");
    invoke_signed(
        &init_open_orders_ix,
        &[
            open_orders.clone(),
            amm_authority.to_account_info(),
            serum_market.clone(),
            rent.clone(),
            serum_market_authority.clone(),
            serum_dex_program_id.clone(),
        ],
        &[
            &[
                PREFIX_AMM_LIQUIDITY_AUTH.as_bytes(),
                optifi_exchange.key().as_ref(),
                &[bump2],
                // amm.key().as_ref(),
                // &[amm.amm_idx],
                // &[amm.bump],
            ],
            &[
                PREFIX_SERUM_MARKET_AUTH.as_bytes(),
                optifi_exchange.key().as_ref(),
                &[bump],
            ],
        ],
    )
}
