use crate::{financial::Duration, state::position::Position};
use anchor_lang::prelude::*;
use solana_program::pubkey::Pubkey;

/// max number of instruments, can be used to limit the length of positions and proposals in AMM
const MAX_INSTRUMENT_LENGTH: u8 = 18;

#[account]
#[derive(Default)]
pub struct AmmAccount {
    /// optifi exchange which the AMM belongs to
    pub optifi_exchange: Pubkey,
    /// index of the amm
    pub amm_idx: u8,
    /// quote token mint address
    pub quote_token_mint: Pubkey,
    /// quote token account address
    pub quote_token_vault: Pubkey,
    /// LP tokens for liquidity providers
    pub lp_token_mint: Pubkey,
    pub amm_capacity: u64,
    /// bump seed used to derive this amm address
    pub bump: u8,
    /// underlying asset
    pub asset: u8, // 1 bytes
    // /// a list of AMM's active trading markets, it should be updated when AMM is updated
    // pub trading_optifi_markets: Vec<Pubkey>,
    // // a list of AMM's active trading innstruments in trading_markets, it should be updated when AMM is updated
    // pub instruments: Vec<InstrumentKeyData>,
    /// a list of pubkeys of the instruments AMM will trade
    pub trading_instruments: Vec<Pubkey>,
    /// a list of Position struct by instruments
    // pub positions: [Position; (STRIKES + BACKUP_STRIKES) as usize * 2],
    pub positions: Vec<Position>,
    /// a list of proposals of orders to excute for each instrument
    // pub proposals: [Proposal; (STRIKES + BACKUP_STRIKES) as usize * 2],
    pub proposals: Vec<Proposal>,
    /// amm's state indicator
    pub state: AmmState,
    /// each instrument's state flag under the current AMM state
    // pub flags: [bool; (STRIKES + BACKUP_STRIKES) as usize * 2],
    pub flags: Vec<bool>,
    /// the implied volatility
    pub iv: u64,
    /// the underlying asset price based on USDC
    pub price: u64,
    /// the net delta
    pub net_delta: i64,
    /// the amm total liquidity based on USDC
    pub total_liquidity: u64,
    /// the duration type (Weekly/Monthly)
    pub duration: Duration,
    /// the expiry date
    pub expiry_date: u64,
    /// the contract size *10000 (f_to_u_repr)
    pub contract_size: u64,
}

/// 1. When AMM state is Sync. a syncing cranker finds instrument with false flag to sync positions.
///    If all flag are true, AMM state will change to Calculate, and flags will be reset to all false.
/// 2. When AMM state is Calculate, a calculating crankers instrument finds instrument with false to calc
///    and save the proposal, and set this instrument's flag to true.
///    If all flags are true, AMM state will change to Execute, and flags will be reset to all false.
/// 3. When AMM state is Execute, the first executing cranker find those instuments with false flags to
///    execute the orders in proposal.
///    In a propsal, if the flag is_started is false, so the cranker, as the first cranker will need
///    to cancel the previous orders of this instrumnet, and then submit some orders in orders_to_execute of the proposal.
///    If the length of orders_to_execute is 0, which means the proposal is finished. so the cranker will
///    set instrument's flag in AMM as true.
///    If all flags in AMM are true, the executing cranker will change AMM state into Sync and
///    reset all flags in AMM to false, which means next round of AMM update can be started.
#[derive(Clone, Copy, PartialEq, Eq, AnchorDeserialize, AnchorSerialize)]
pub enum AmmState {
    Sync,
    CalculateDelta,
    CalculateProposal,
    Execute,
}

impl Default for AmmState {
    fn default() -> AmmState {
        AmmState::Sync
    }
}

impl AmmAccount {
    /// move the amm to next state
    pub fn move_to_next_state(&mut self) {
        match self.state {
            AmmState::Sync => self.state = AmmState::CalculateDelta,
            AmmState::CalculateDelta => self.state = AmmState::CalculateProposal,
            AmmState::CalculateProposal => self.state = AmmState::Execute,
            AmmState::Execute => self.state = AmmState::Sync,
        }
    }

    pub fn move_to_first_state(&mut self) {
        self.state = AmmState::Sync
    }
}

// #[assert_size(40)]
#[derive(Default, Clone, AnchorDeserialize, AnchorSerialize)]
pub struct Proposal {
    /// instrument pubkey
    pub instrument: Pubkey,
    /// if the orders execution is started
    pub is_started: bool,
    /// all ask_orders_size
    pub ask_orders_size: Vec<u64>, // 100 orders // 99
    /// all orders to execute
    pub bid_orders_size: Vec<u64>, // 100 orders // 99
    /// all orders to execute
    pub ask_orders_price: Vec<u64>, // 100 orders // 99
    /// all orders to execute
    pub bid_orders_price: Vec<u64>, // 100 orders // 99
}

// // #[assert_size(40)]
// #[derive(Default, Clone, Copy, AnchorDeserialize, AnchorSerialize)]
// pub struct ProposedOrder {
//     /// order size
//     pub size: f32,
//     /// order price - usdc as quote
//     pub price: f32,
// }
