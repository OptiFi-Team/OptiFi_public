use anchor_lang::prelude::*;
use anchor_lang::{AnchorDeserialize, AnchorSerialize};
use anchor_spl::token::accessor::amount;
use solana_program::{program_error::ProgramError, program_pack::IsInitialized, pubkey::Pubkey};
use std::{cmp::min, fmt::Debug};

use crate::financial::Asset;

#[account]
pub struct UserAccount {
    /// optifi exchange which the user account belongs to
    pub optifi_exchange: Pubkey, // 32 bytes

    /// The owner of this account.
    pub owner: Pubkey,

    /// The margin account which user deposits usdc token into
    /// it's a spl token account
    pub user_margin_account_usdc: Pubkey,

    /// temp PnL record for fund settlment purpose
    pub temp_pnl: TempPnL,

    // /// The total amount of tokens the user deposited into this account.
    // pub reserve: u64,
    /// The account's state
    pub state: AccountState,

    /// a list of instrument Pubkey and position
    pub positions: Vec<UserPosition>,

    pub is_in_liquidation: bool,

    /// the bump seed to get the address of this user account
    pub bump: u8,

    /// maintanance margin
    pub amount_to_reserve: [u64; 10],
}

#[derive(Clone, Copy, Debug, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct UserPosition {
    instrument: Pubkey,
    long_qty: u64,
    short_qty: u64,
}

impl UserPosition {
    /// get the instrument pubkey
    pub fn get_instrument(&self) -> &Pubkey {
        return &self.instrument;
    }

    /// get the net quantity
    pub fn get_quantity(&self) -> i64 {
        return self.long_qty as i64 - self.short_qty as i64;
    }

    /// check if the instrument is still valid on the Optifi exchange
    pub fn is_valid(&self, instruments: &Vec<Pubkey>) -> bool {
        instruments.contains(&self.instrument)
    }
}

impl UserAccount {
    // pub fn initialize_account() -> Result<Self, ProgramError> {
    //     Ok(self)
    // }

    /// Checks if account is frozen
    fn is_frozen(&self) -> bool {
        self.state == AccountState::Frozen
    }
    /// returns the available margin amount of the user
    /// if the user has negative temp pnl, should get take the loss into account
    pub fn get_available_margin(&self, user_margin_account: &AccountInfo) -> u64 {
        let margin_balance = amount(user_margin_account).unwrap();
        min(
            (margin_balance as i64 + self.temp_pnl.amount) as u64,
            margin_balance,
        )
    }

    /// add the short position amount to user
    pub fn add_short_position(&mut self, instrument: Pubkey, qty: u64) {
        if let Some(p) = self
            .positions
            .iter_mut()
            .find(|position| position.instrument == instrument)
        {
            p.short_qty += qty;
        } else {
            self.positions.push(UserPosition {
                instrument,
                long_qty: 0,
                short_qty: qty,
            });
        }
    }

    /// add the long position amount to user
    pub fn add_long_position(&mut self, instrument: Pubkey, qty: u64) {
        if let Some(p) = self
            .positions
            .iter_mut()
            .find(|position| position.instrument == instrument)
        {
            p.long_qty += qty;
        } else {
            self.positions.push(UserPosition {
                instrument,
                long_qty: qty,
                short_qty: 0,
            });
        }
    }

    /// update the long position amount to user
    pub fn update_long_position(&mut self, instrument: Pubkey, qty: u64) {
        if let Some(p) = self
            .positions
            .iter_mut()
            .find(|position| position.instrument == instrument)
        {
            p.long_qty = qty;
        } else {
            self.positions.push(UserPosition {
                instrument,
                long_qty: qty,
                short_qty: 0,
            });
        }
    }

    /// update the short position amount to user
    pub fn update_short_position(&mut self, instrument: Pubkey, qty: u64) {
        if let Some(p) = self
            .positions
            .iter_mut()
            .find(|position| position.instrument == instrument)
        {
            p.short_qty = qty;
        } else {
            self.positions.push(UserPosition {
                instrument,
                long_qty: 0,
                short_qty: qty,
            });
        }
    }

    /// get the net quantity
    pub fn get_quantity(&self, instrument: Pubkey) -> i64 {
        if let Some(p) = self
            .positions
            .iter()
            .find(|position| position.instrument == instrument)
        {
            return p.get_quantity();
        } else {
            return 0;
        }
    }

    /// get the total margin reserve
    pub fn get_maintanance_margin(&self) -> u64 {
        self.amount_to_reserve.iter().sum()
    }
}

impl IsInitialized for UserAccount {
    fn is_initialized(&self) -> bool {
        self.state != AccountState::Uninitialized
    }
}

/// Account state.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub enum AccountState {
    /// Account is not yet initialized
    Uninitialized,
    /// Account is initialized; the account owner and/or delegate may perform permitted operations
    /// on this account
    Initialized,
    /// Account has been frozen by the mint freeze authority. Neither the account owner nor
    /// the delegate are able to perform operations on this account.
    Frozen,
}

impl Default for AccountState {
    fn default() -> Self {
        AccountState::Uninitialized
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct TempPnL {
    pub amount: i64,
    pub epoch: u64,
}
