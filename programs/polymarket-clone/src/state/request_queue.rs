use anchor_lang::prelude::*;

use crate::constants::MAX_REQUESTS;

#[repr(u8)]
//repr with u8 is generally used with enums and structs especially with enums
//it writes or assigns the value  from 0 to 255 ..(in case of u8) , to each value in the memory
//it is used when working with strict memory managements requirements occurs such as in this use case
#[derive(Debug, AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Clone, Copy)]

pub enum RequestType {
    NewOrder = 0,
    CancelOrder = 1,
    MarketOrder = 2,
}

#[repr(u8)]
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy, PartialEq, Eq)]

pub enum OrderSide {
    Buy = 0,
    Sell = 1,
}

#[repr(u8)]
#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]

pub enum OutcomeSide {
    Yes = 0,
    No = 1,
}

#[repr(C)]
// repr with C type aligns and assigns the memory with strict C type memory allocation
// it assigns with proper alignment and padding
// repr(packed) is also something which is used in same context but it removes the padding and alignment
//which makes it very specific to niches like where you know tha misalignment with not cause an issue and can lead to runtime and unexpected errors when working with various machines
#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub struct Request {
    pub request_type: u8,
    pub owner: Pubkey,
    pub open_order: Pubkey,
    pub side: u8,
    pub price: u128,
    pub quantity: u64,
    pub order_id: u64,
    pub client_id: u64,
    pub outcome: u8,
    pub timestamp: i64,
}

#[account]
pub struct RequestQueue {
    pub head: u64,
    pub count: u64,
    pub requests: [Request; MAX_REQUESTS],
    pub bump: u8,
}
