use anchor_lang::prelude::*;

#[repr(u8)]
#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]

pub enum MarketStatus {
    Open = 0,
    Paused = 1,
    ResolvedYes = 2,
    ResolvedNo = 3,
    Cancelled = 4,
}

#[account]
pub struct Market {
    pub market_id: u64,
    pub creator: Pubkey,
    pub question: String,
    pub description: String,
    pub end_ts: i64,
    pub status: MarketStatus,
    pub yes_mint: Pubkey,
    pub no_mint: Pubkey,
    pub usdc_mint: Pubkey,
    pub vault_yes: Pubkey,
    pub vault_no: Pubkey,
    pub vault_usdc: Pubkey,
    pub fee_vault_usdc: Pubkey,
    pub fee_bps: u16,
    pub cranker_reward_bps: u16,
    pub q_yes: u128,
    pub q_no: u128,
    pub b_liquidity: u64,
    pub bids: Pubkey,
    pub asks: Pubkey,
    pub request_queue: Pubkey,
    pub event_queue: Pubkey,
    pub oracle: Pubkey,
    pub bump: u8,
}
