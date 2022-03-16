use anchor_lang::error;

#[error]
pub enum ErrorCode {
    #[msg("the user account cannot be initialized")]
    AccountCannotInit,

    #[msg("the user account is invalid")]
    InvalidAccount,

    #[msg("the account is not owned by the payer")]
    UnauthorizedAccount,

    #[msg("the account balance is insufficient")]
    InsufficientFund,

    #[msg("Token transfer failed")]
    TokenTransferFailed,

    #[msg("the token vault is not owned by the payer")]
    UnauthorizedTokenVault,

    #[msg("the provided pda is invalid")]
    InvalidPDA,

    #[msg("Uuid must be exactly of 6 length")]
    UuidMustBeExactly6Length,

    #[msg("Numerical overflow error!")]
    NumericalOverflowError,

    #[msg("Insufficient Margin!")]
    InsufficientMargin,

    #[msg("Incorrect coin mint!")]
    IncorrectCoinMint,

    #[msg("Cannot settle fund befor markets have been stopped!")]
    CannotSettleFundBeforeMarketsStopped,

    #[msg("Incorrect oracle account")]
    IncorrectOracleAccount,

    #[msg("the working state is wrong")]
    WrongState,

    #[msg("the instrument has already been done")]
    WrongInstrument,

    #[msg("no enough orders in proposal to execute")]
    NoEnoughOrdersInProposal,

    #[msg("cannot remove the instrument for amm")]
    CannotRemoveInstrumentForAMM,

    #[msg("cannot add the instrument for amm due to duplication")]
    DuplicateInstrumentForAMM,

    #[msg("User is not in liquidation")]
    UserNotInLiquidation,

    #[msg("User is not in cancel order status")]
    UserNotInCancelOrder,

    #[msg("User was already in liquidation")]
    UserAlreadyInLiquidation,

    #[msg("Instrument was already registered for liquidation")]
    InstrumentAlreadyRegisteredForLiquidation,

    #[msg("Users cannot place manual orders while their accounts are in liquidation")]
    CannotPlaceOrdersInLiquidation,

    #[msg("Provided USDC pool is not central pool")]
    PoolNotCentralUSDCPool,

    #[msg("Invalid open orders market authority")]
    InvalidSerumAuthority,

    #[msg("Only one withdraw request allowed at one time")]
    WithdrawRequestInvalid,

    #[msg("Market maker withdraw outside of valid window")]
    MMWithdrawNotInWindow,

    #[msg("Wrong asset")]
    WrongAsset,

    #[msg("Should update the margin stress again")]
    TimeOut,
}
