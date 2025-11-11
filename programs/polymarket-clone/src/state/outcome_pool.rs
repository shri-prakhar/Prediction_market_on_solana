use anchor_lang::prelude::*;

#[account]

pub struct OutcomePool {
    pub market: Pubkey,
    pub yes_reserve: u128,
    pub no_reserve: u128,
    pub total_lp_shares: u64,
    pub bump: u8,
}
