use anchor_lang::prelude::*;

use crate::constants::MAX_OPEN_ORDER_SLOTS;

#[repr(C)]
#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub struct OrderSlot {
    pub active: bool,
    pub order_id: u64,
    pub price: u64,
    pub side: u8,
    pub quantity_remaining: u64,
    pub outcome: u8,
}

#[account]
pub struct OpenOrder {
    pub owner: Pubkey,
    pub market: Pubkey,
    pub free_base: u128,
    pub free_quote: u128,
    pub locked_base: u128,
    pub locked_quote: u128,
    pub slots_bitmaps: u128,
    pub slots: [OrderSlot; MAX_OPEN_ORDER_SLOTS],
    pub bump: u8,
}
