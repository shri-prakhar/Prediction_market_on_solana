use anchor_lang::prelude::*;

#[repr(C)]
#[derive(Debug , Clone  , Copy , PartialEq , Eq , AnchorSerialize , AnchorDeserialize)] 
pub enum OutcomeSide{
    Yes,
    No
}
#[repr(C)]
#[derive(AnchorSerialize , AnchorDeserialize , Debug , Clone , Copy , PartialEq , Eq)]

pub enum Orderside {
    Buy ,
    Sell 
}

#[repr(C)]
#[derive(Debug, AnchorSerialize , AnchorDeserialize , Clone , Copy , PartialEq ,Eq )]

pub enum MarketStatus{
    Open,
    Paused,
    Resolved { winner: OutcomeSide },
    Cancelled
}

#[account]
pub struct Market {
    pub market_id: u64 ,
    pub creator: Pubkey,
    pub question: String,
    pub description : String ,
    pub end_ts : i64,
    pub status :MarketStatus,
    pub yes_mint : Pubkey,
    pub no_mint :Pubkey ,
    pub vault_usdc:Pubkey,
    pub amm_pool:Pubkey,
    pub fee_bps:u16,
    pub q_yes : u128,
    pub q_no : u128 ,
    pub b_liquidity : u64,
    pub bids : Pubkey,
    pub asks: Pubkey,
    pub event_queue:Pubkey,
    pub oracle: Pubkey,
    pub bump: u8,
}