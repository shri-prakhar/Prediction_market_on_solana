use anchor_lang::prelude::*;

#[account]

pub struct Vault {
    pub market: Pubkey,
    pub token_mint: Pubkey,
    pub token_account: Pubkey,
    pub token_collateral: u64,
    pub bump: u8,
}
