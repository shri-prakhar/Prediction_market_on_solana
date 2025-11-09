use anchor_lang::prelude::*;

use crate::constants::MAX_ORDER_PER_TRADER;

#[account]
pub struct TraderPosition{
        pub market : Pubkey ,
        pub owner : Pubkey,
        pub yes_shares : u128,
        pub no_shares : u128 ,
        pub pending_usdc: u128,
        pub active_orders : u8, //no. of active orders
        pub slots_bitmap : u128, //bitmask , 1 means occupied  
        pub order_ids : [u64; MAX_ORDER_PER_TRADER], //defines the fixed length of the struct 
        pub bump: u8,
}