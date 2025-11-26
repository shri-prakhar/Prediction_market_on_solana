use anchor_lang::prelude::*;

use crate::constants::{MAX_ORDER_ENTRIES, MAX_PRICE_NODES};

// use crate::{constants::MAX_SLAB_NODES, state::OutcomeSide};

// #[repr(C)]
// #[derive(AnchorDeserialize, AnchorSerialize, Debug, Clone, Copy, PartialEq, Eq)]

// pub enum EventType {
//     Fill,
//     Cancel,
// }

// #[repr(C)]
// #[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy, PartialEq, Eq)]

// pub struct Event {
//     pub event_type: EventType,
//     pub maker: Pubkey,
//     pub taker: Pubkey,
//     pub quantity: u64,
//     pub price: u64,
//     pub side: u8, //0: Buy , 1: Sell
//     pub order_id: u64,
//     pub outcome : OutcomeSide,
//     pub time_stamp: i64,
// }

// #[account]

// pub struct Slab {
//     pub is_bid: bool,                      //true => bid slab and false => ask slab
//     pub node_count: u32,                   //current used nodes
//     pub free_head: i32,                    //index of the first free node (-1 if none)
//     pub head_index: i32, //index of the best node (i.e. the highest bid / the lowest bid)
//     pub nodes: [SlabNode; MAX_SLAB_NODES], //fixed size pre-allocated array
//     pub bump: u8,
// }

// #[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
// pub struct SlabNode {
//     pub key: u128,     //price key used u128 because to make(   )
//     pub owner: Pubkey, //traderPosition pda owning the order
//     pub quantity: u64, // remaining quantity (in outcome shares)
//     pub order_id: u64,
//     pub reserved_usdc: u64,    // if Buy Order: then reserved usdc
//     pub reserved_outcome: u64, // if sell Order : then reserved outcome
//     pub outcome : OutcomeSide, // Yes or No
//     pub next: i32,             //index of next slot in price sorted linked-list (-1 if none)
//     pub prev: i32,
//     pub time_stamp: i64, //used for first-in-first-out ordering for managing event queues that it should serve
//     //as the first node should be processed that's arrived first or being there for the longest term
//     pub occupied: bool, // marks that is this slot contains the order or not
// }

// // the reason behind we are storing reserved usdc and reserved outcome here is because of the if maker cancels than
// // it's good to have stored the amounts in slabnodes to process refunds

// #[account]

// pub struct EventQueue {
//     pub head: u64,
//     pub count: u64,
//     pub events: Vec<Event>,
//     pub bump: u8,
// }

#[repr(C)]
#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]

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
#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]

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
