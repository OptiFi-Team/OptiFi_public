use anchor_lang::prelude::*;

/// used to derive the address of an optifi exchange
pub const PREFIX_OPTIFI_EXCHANGE: &str = "optifi_exchange";

/// 1. used to derive the address of a user account
/// 2. this pda address has the authority over all spl token accounts and open orders accounts of the user
pub const PREFIX_USER_ACCOUNT: &str = "user_account";

/// used to derive the address of an instrument
pub const PREFIX_INSTRUMENT: &str = "instrument";

/// used to derive the address of an optifi market
pub const PREFIX_OPTIFI_MARKET: &str = "optifi_market";

/// 1. used to derive an address
/// 2. this pda has the mint authority over all instruments spl tokens for all optifi markets
pub const PREFIX_OPTIFI_MARKET_MINT_AUTH: &str = "optifi_market_mint_auth";

/// used to derive the address of a user's serum open orders
pub const PREFIX_SERUM_OPEN_ORDERS: &str = "serum_open_orders";

/// 1. used to derive an address
/// 2. this pda is the serum market authority and the serum market prune authority
pub const PREFIX_SERUM_MARKET_AUTH: &str = "serum_market_auth";

/// 1. used to derive address of the central usdc pool vault for fund settlement purpose
/// 2. this pda is the serum market authority and the serum market prune authority
pub const PREFIX_CENTRAL_USDC_POOL: &str = "central_usdc_pool";

/// 1. used to derive an address
/// 2. this pda has authority over central usdc pool vault
pub const PREFIX_CENTRAL_USDC_POOL_AUTH: &str = "central_usdc_pool_auth";

/// used to derive amm account address
pub const PREFIX_AMM: &str = "amm";

/// Used to derive market maker account address
pub const PREFIX_MARKET_MAKER: &str = "market_maker";

/// used to derive the authority address of amm's usdc vault,lp token mint and lp token vault
pub const PREFIX_AMM_LIQUIDITY_AUTH: &str = "amm_liquidity_auth";

/// used for the liquidation state account, which will record the progress of liquidations for a user
pub const PREFIX_LIQUIDATION_STATE: &str = "liquidation_state";

/// Used for market maker liquidity pool
pub const PREFIX_MM_LIQUIDITY_AUTH: &str = "mm_liquidity_auth";

/// used to derive margin stress account address
pub const PREFIX_MARGIN_STRESS: &str = "margin_stress";

/// get the user account pda and its bump seed
pub fn get_user_account_pda(
    optifi_exchange: &Pubkey,
    user: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PREFIX_USER_ACCOUNT.as_bytes(),
            optifi_exchange.as_ref(),
            user.as_ref(),
        ],
        program_id,
    )
}

/// get mint authority pda
pub fn get_optifi_market_mint_auth_pda(
    optifi_exchange: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PREFIX_OPTIFI_MARKET_MINT_AUTH.as_bytes(),
            optifi_exchange.as_ref(),
        ],
        program_id,
    )
}

/// get serum market and prune authority pda
pub fn get_serum_market_auth_pda(optifi_exchange: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PREFIX_SERUM_MARKET_AUTH.as_bytes(),
            optifi_exchange.as_ref(),
        ],
        program_id,
    )
}

/// get the central usdc pool vault address(pda)
pub fn get_central_usdc_pool_pda(optifi_exchange: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PREFIX_CENTRAL_USDC_POOL.as_bytes(),
            optifi_exchange.as_ref(),
        ],
        program_id,
    )
}

/// get the authority address (pda) of central usdc pool
pub fn get_central_usdc_pool_auth_pda(
    optifi_exchange: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PREFIX_CENTRAL_USDC_POOL_AUTH.as_bytes(),
            optifi_exchange.as_ref(),
        ],
        program_id,
    )
}

pub fn get_market_maker_pool_auth_pda(
    optifi_exchange: &Pubkey,
    market_maker_address: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PREFIX_MM_LIQUIDITY_AUTH.as_bytes(),
            optifi_exchange.as_ref(),
            market_maker_address.as_ref(),
        ],
        program_id,
    )
}

/// get the amm address (pda) with index
pub fn get_amm_pda(optifi_exchange: &Pubkey, amm_idx: u8, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[PREFIX_AMM.as_bytes(), optifi_exchange.as_ref(), &[amm_idx]],
        program_id,
    )
}

/// get the authority address (pda) of amm usdc vault, lp token mint and vault
pub fn get_amm_liquidity_auth_pda(optifi_exchange: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PREFIX_AMM_LIQUIDITY_AUTH.as_bytes(),
            optifi_exchange.as_ref(),
        ],
        program_id,
    )
}
