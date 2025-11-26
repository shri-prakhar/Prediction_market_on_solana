use anchor_lang::prelude::*;

#[error_code]

pub enum MarketError {
    #[msg("Market is Not Open")]
    MarketNotOpen,
    #[msg("Event Queue is Full")]
    EventQueueFUll,
    #[msg("Request Queue is Full")]
    RequestQueueFull,
    #[msg("Insufficient Balance for this operation")]
    InsufficientBalance,
    #[msg("Order Not Found")]
    OrderNotFound,
    #[msg("Exceeded max open orders per trade")]
    MaxOrderReached,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Math Overflow or Underflow Detected")]
    MathError,
    #[msg("No Matching Order Found")]
    NoMatchingOrder,
    #[msg("Invalid Argument")]
    InvalidArgument,
    #[msg("Slot Already Occupied")]
    SlotOccupied,
    #[msg("Invalid Order Side")]
    InvalidSide,
    #[msg("Vault Transfer Failed")]
    VaultTransferFailed,
}
