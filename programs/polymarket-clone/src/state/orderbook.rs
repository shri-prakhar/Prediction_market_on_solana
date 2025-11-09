use anchor_lang::prelude::*;

use crate::constants::MAX_SLAB_NODES;

#[repr(C)]
#[derive(AnchorDeserialize , AnchorSerialize ,  Debug , Clone , Copy , PartialEq ,Eq)] 

pub enum EventType{
    Fill, 
    Cancel
}

#[repr(C)]
#[derive(AnchorSerialize , AnchorDeserialize , Debug , Clone , Copy , PartialEq , Eq )]

pub struct Event {
    pub event_type : EventType,
    pub maker : Pubkey,
    pub taker: Pubkey,
    pub quantity: u64,
    pub price : u64,
    pub side : u8, //0: Buy , 1: Sell
    pub order_id:u64,
    pub time_stamp: i64,
}

#[account]

pub struct Slab{
    pub is_bid: bool,  //true => bid slab and false => ask slab
    pub node_count : u32, //current used nodes 
    pub free_head : i32, //index of the first free node (-1 if none)
    pub head_index : i32 , //index of the best node (i.e. the highest bid / the lowest bid)
    pub nodes : [SlabNode; MAX_SLAB_NODES], //fixed size pre-allocated array
    pub bump : u8 , 
}

#[derive(AnchorSerialize , AnchorDeserialize , Clone , Copy ,PartialEq , Eq)]
pub struct SlabNode{
    pub key: u128 , //price key used u128 because to make(   )
    pub owner: Pubkey , //traderPosition pda owning the order 
    pub quantity : u64, // remaining quantity (in outcome shares)
    pub order_id : u64,
    pub next : i32, //index of next slot in price sorted linked-list (-1 if none)
    pub prev : i32,
    pub time_stamp : i64 , //used for first-in-first-out ordering for managing event queues that it should serve 
    //as the first node should be proceesed that's arrrived first or being there for the longest term 
    pub occupied : bool , // marks that is this slot contains the order or not 
}


#[account]

pub struct EventQueue{
    pub head : u64 ,
    pub count : u64 ,
    pub events: Vec<Event>, 
    pub bump: u8,
}

