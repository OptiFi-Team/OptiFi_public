use crate::i_to_f_repr;
use anchor_lang::prelude::*;
use solana_program::pubkey::Pubkey;

#[derive(Clone, Copy, PartialEq, Eq, AnchorSerialize, AnchorDeserialize)]
#[repr(u8)]
pub enum LiquidationStatus {
    Healthy,
    CancelOrder,
    ClosePositions,
}

impl Default for LiquidationStatus {
    fn default() -> Self {
        LiquidationStatus::Healthy
    }
}

#[account]
#[derive(Default)]
pub struct LiquidationState {
    pub user_account: Pubkey,      // 32 bytes
    pub status: LiquidationStatus, // 1 bytes
    pub instruments: Vec<Pubkey>,  // 36 * 32 bytes
    pub values: Vec<i64>,          // 36 * 8 bytes
}

impl LiquidationState {
    /// Get the largest position currently in the liquidation state -
    /// this will always be the next position to be liquidated
    pub fn pop_largest(&mut self) -> Pubkey {
        let mut temp = 0;
        let mut index: usize = 0;
        for i in 0..self.instruments.len() {
            let v = self.values[i];
            if v < temp {
                temp = v;
                index = i;
            }
        }
        let result = self.instruments[index];
        self.instruments.remove(index);
        self.values.remove(index);
        result
    }

    /// Reset the liquidation state for the user after liquidations are finished
    pub fn liquidation_complete(&mut self) {
        self.status = LiquidationStatus::Healthy;
        self.instruments = Vec::new();
        self.values = Vec::new();
    }

    /// Add registered position
    pub fn add_position(&mut self, value: i64, instrument: Pubkey) {
        self.instruments.push(instrument);
        self.values.push(value);
    }
}
