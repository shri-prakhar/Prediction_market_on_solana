use anchor_lang::prelude::*;

use crate::constants::{MAX_ORDER_ENTRIES, MAX_PRICE_NODES};

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]

pub struct OrderEntry {
    pub occupied: bool,
    pub order_id: u64,
    pub open_order_owner: Pubkey,
    pub owner_slot: u16,
    pub quantity: u64,
    pub reserved_amount: u64,
    pub next_in_price: i32,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PriceNode {
    pub occupied: bool,
    pub key: u128,
    pub left: i32,
    pub right: i32,
    pub parent: i32,
    pub order_head: i32,
    pub order_tail: i32,
    pub color: u8, // 0 -> Black , 1 -> Red
}

#[account]
pub struct Slab {
    pub is_bid: bool,
    pub node_count: u64,
    pub free_price_node_head: i32,
    pub free_order_entry_head: i32,
    pub root_price_node: i32,
    pub price_nodes: [PriceNode; MAX_PRICE_NODES],
    pub order_entries: [OrderEntry; MAX_ORDER_ENTRIES],
    pub bump: u8,
}
