use anchor_lang::prelude::*;

#[account]
pub struct Manager {
    pub admin: Pubkey,
    pub total_markets: u64,
    pub bump: u8,
}
