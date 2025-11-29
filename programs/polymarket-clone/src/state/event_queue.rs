use anchor_lang::prelude::*;

use crate::constants::MAX_EVENTS;

#[repr(u8)]
#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]

pub enum EventType {
    Fill = 0,
    Cancel = 1,
}

#[repr(C)]
#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub struct Event {
    pub event_type: u8,
    pub makers_open_orders: Pubkey,
    pub maker_slot: u16,
    pub taker_open_orders: Pubkey,
    pub taker_side: u8, //i.e. order_side Buy or Sell
    pub taker_slot: u16,
    pub price: u128,
    pub quantity: u64,
    pub order_id: u64,
    pub outcome: u8,
    pub timestamp: i64,
}

#[account]
pub struct EventQueue {
    pub head: u64,
    pub count: u64,
    pub events: [Event; MAX_EVENTS],
    pub bump: u8,
}
