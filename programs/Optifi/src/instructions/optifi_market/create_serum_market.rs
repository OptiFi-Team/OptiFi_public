use crate::utils::get_serum_market_auth_pda;
use anchor_lang::{prelude::*, solana_program::program::invoke};
use serum_dex::error::DexError::ProgramError;
use serum_dex::error::{DexError, DexErrorCode};
use serum_dex::instruction::initialize_market;
use solana_program::program_error::ProgramError as SolanaProgramError;
use std::borrow::Borrow;
use std::error::Error;

#[derive(Accounts)]
#[instruction(bump:u8)]
pub struct InitializeSerumMarket<'info> {
    /// 0. `[writable]` the market to initialize
    /// 1. `[writable]` zeroed out request queue
    /// 2. `[writable]` zeroed out event queue
    /// 3. `[writable]` zeroed out bids
    /// 4. `[writable]` zeroed out asks
    /// 5. `[writable]` spl-token account for the coin currency
    /// 6. `[writable]` spl-token account for the price currency
    /// 7. `[]` coin currency Mint
    /// 8. `[]` price currency Mint
    /// 9. `[]` the rent sysvar
    /// 10. `[]` open orders market authority (optional)
    /// 11. `[]` prune authority (optional, requires open orders market authority)
    /// 12. `[]` crank authority (optional, requires prune authority)
    pub optifi_exchange: AccountInfo<'info>,
    #[account(mut)]
    pub market: AccountInfo<'info>,
    pub coin_mint_pk: AccountInfo<'info>,
    pub pc_mint_pk: AccountInfo<'info>,
    #[account(mut)]
    pub coin_vault_pk: AccountInfo<'info>,
    #[account(mut)]
    pub pc_vault_pk: AccountInfo<'info>,
    #[account(mut, signer)]
    pub bids_pk: AccountInfo<'info>,
    #[account(mut, signer)]
    pub asks_pk: AccountInfo<'info>,
    #[account(mut, signer)]
    pub req_q_pk: AccountInfo<'info>,
    #[account(mut, signer)]
    pub event_q_pk: AccountInfo<'info>,
    pub serum_market_authority: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub serum_dex_program: AccountInfo<'info>,
}

pub fn handler(
    ctx: Context<InitializeSerumMarket>,
    authority_pk: Option<Pubkey>,
    prune_authority_pk: Option<Pubkey>,
    coin_lot_size: u64,
    pc_lot_size: u64,
    vault_signer_nonce: u64,
    pc_dust_threshold: u64,
) -> ProgramResult {
    let optifi_exchange = &ctx.accounts.optifi_exchange;
    let market = &ctx.accounts.market;
    let coin_mint_pk = &ctx.accounts.coin_mint_pk;
    let pc_mint_pk = &ctx.accounts.pc_mint_pk;
    let coin_vault_pk = &ctx.accounts.coin_vault_pk;
    let pc_vault_pk = &ctx.accounts.pc_vault_pk;
    let bids_pk = &ctx.accounts.bids_pk;
    let asks_pk = &ctx.accounts.asks_pk;
    let req_q_pk = &ctx.accounts.req_q_pk;
    let event_q_pk = &ctx.accounts.event_q_pk;
    let serum_dex_program = &ctx.accounts.serum_dex_program;
    let rent = &ctx.accounts.rent.to_account_info();
    let serum_market_authority = &ctx.accounts.serum_market_authority;

    // let authority_pk_ptr: Option<&Pubkey>;
    // let prune_authority_pk_ptr: Option<&Pubkey>;

    // if let Some(authority_pk) = &authority_pk {
    //     authority_pk_ptr = Some(&authority_pk);
    // } else {
    //     authority_pk_ptr = None
    // }
    // if let Some(prune_authority_pk) = &prune_authority_pk {
    //     prune_authority_pk_ptr = Some(&prune_authority_pk)
    // } else {
    //     prune_authority_pk_ptr = None
    // }

    let (market_auth, _bump) = get_serum_market_auth_pda(&optifi_exchange.key(), ctx.program_id);

    let ix_res = initialize_market(
        market.key,
        serum_dex_program.key,
        coin_mint_pk.key,
        pc_mint_pk.key,
        coin_vault_pk.key,
        pc_vault_pk.key,
        Some(&market_auth),
        Some(&market_auth),
        bids_pk.key,
        asks_pk.key,
        req_q_pk.key,
        event_q_pk.key,
        coin_lot_size,
        pc_lot_size,
        vault_signer_nonce,
        pc_dust_threshold,
    );

    if ix_res.is_err() {
        msg!("Got error trying to form market creation instruction");
        let err: DexError = ix_res.expect_err("Ix res wasn't error");
        match err {
            ProgramError(i) => {
                msg!("i was program error");
            }
            DexError::ErrorCode(i) => {
                msg!("Code was {}", i);
                let source = i.source();
                match source {
                    None => {
                        msg!("Couldn't resolve source of error")
                    }
                    Some(e) => {
                        msg!("Got source {} of error", e);
                    }
                }
            }
        }
        return Err(SolanaProgramError::Custom(1));
    }
    let init_serum_market_ix = ix_res?;

    msg!("Ix was valid, Calling the serum dex program to initialize a market");
    invoke(
        &init_serum_market_ix,
        &[
            market.clone(),
            req_q_pk.clone(),
            event_q_pk.clone(),
            bids_pk.clone(),
            asks_pk.clone(),
            coin_vault_pk.clone(),
            pc_vault_pk.clone(),
            coin_mint_pk.clone(),
            pc_mint_pk.clone(),
            rent.clone(),
            serum_market_authority.clone(),
            serum_market_authority.clone(),
            serum_dex_program.clone(),
        ],
    )?;
    msg!("serum market initialized successfully");
    Ok(())
}
