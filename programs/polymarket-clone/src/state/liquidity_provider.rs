use anchor_lang::prelude::*;

#[account]

pub struct LiquidityProvider{
    pub market:Pubkey,
    pub owner : Pubkey , 
    pub lp_shares : u64,
    pub deposited_usdc: u64,
    pub bump : u8,
}