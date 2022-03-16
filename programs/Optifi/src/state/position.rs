use anchor_lang::prelude::*;
use anchor_lang::{AnchorDeserialize, AnchorSerialize};
use solana_program::pubkey::Pubkey;

#[derive(Default, Clone, Copy, AnchorDeserialize, AnchorSerialize)]
pub struct Position {
    /// a list of instruments Pubkey corresponded to positions
    pub instruments: Pubkey,
    /// base token account address
    pub long_token_vault: Pubkey,
    /// base token account address
    pub short_token_vault: Pubkey,
    /// latest position updated by update_amm_positions
    pub latest_position: i64,
    /// the usdc remains in the vault
    pub usdc_balance: u64,
}
