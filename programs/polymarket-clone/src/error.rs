use anchor_lang::prelude::*;

#[error_code]

pub enum MarketError{
    #[msg("Market is already Resolved")]
    MarketClosed,
    #[msg("Market is Not open for trading")]
    MarketNotOpen,
    #[msg("Invalid Outcome specified")]
    InvalidOutcome,
    #[msg("Insufficient Balance for this operation")]
    InsufficientBalnace,
    #[msg("Invalid Order limit size")]
    InvalidOrderSide,
    #[msg("No matching order available")]
    NoMatchingOrder,
    #[msg("Order Not Found")]
    OrderNotFound,
    #[msg("Exceeded max open orders per trade")]
    MaxOrderReached,
    #[msg("Oracle Not Authorised to resolve the market")]
    UnauthorizedOracle,
    #[msg("Market Already Intialized")]
    MarketAlreadyInitialized,
    #[msg("Math Overflow or Underflow Detected")]
    MathError,
    #[msg("Event Queue is full")]
    EventQueueFull,
    #[msg("Invalid PDA bump")]
    Invalidbump 
}