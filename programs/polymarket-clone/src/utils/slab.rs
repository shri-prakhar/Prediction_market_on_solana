use anchor_lang::prelude::*;

use crate::{constants::{MAX_ORDER_ENTRIES, MAX_PRICE_NODES}, error::MarketError, state::Slab, utils::slab};

pub fn initialize_slab(slab: &mut Slab , is_bid: bool , bump: u8 ) {
    slab.is_bid = is_bid;
    slab.node_count = 0;
    slab.free_order_entry_head = -1;
    slab.free_price_node_head = -1 ;
    slab.root_price_node =-1;
    slab.bump = bump;

    for i in 0..MAX_ORDER_ENTRIES{
        slab.price_nodes[i].occupied = false;
        slab.price_nodes[i].key = 0;
        slab.price_nodes[i].order_head = -1;
        slab.price_nodes[i].order_tail = -1;
        slab.price_nodes[i].left = -1;
        slab.price_nodes[i].right = -1;
        slab.price_nodes[i].parent = -1;
        slab.price_nodes[i].color = 0;  
    }
    for i in 0..MAX_PRICE_NODES {
        slab.order_entries[i].occupied = false;
        slab.order_entries[i].next_in_price = -1;
        slab.order_entries[i].order_id = 0;
        slab.order_entries[i].owner_slot = 0;
        slab.order_entries[i].quantity = 0;
        slab.order_entries[i].reserved_amount = 0;
        slab.order_entries[i].open_order_owner = Pubkey::default();
    }
}

#[inline(always)]
fn is_null(index: i32) -> bool {
    if index == -1 {
        true
    } else {
        false
    }
}

#[inline(always)]
fn is_red(slab : &Slab , index:i32) -> bool {
    if is_null(index) {
        false
    }else{
       slab.price_nodes[index as usize].color == 1 
    }
}

#[inline(always)]
fn set_color(slab : &mut Slab , index: i32 , color : u8) {
    if !is_null(index){
        slab.price_nodes[index as usize].color = color; 
    }
}

pub fn allocate_price_node(slab : &mut Slab) -> Result<i32>{
    for i in 0..(MAX_PRICE_NODES as i32) {
        if !slab.price_nodes[i as usize].occupied{
            slab.price_nodes[i as usize].right = -1;
            slab.price_nodes[i as usize].parent = -1;
            slab.price_nodes[i as usize].order_tail = -1;
            slab.price_nodes[i as usize].order_head = -1;
            slab.price_nodes[i as usize].occupied = true;
            slab.price_nodes[i as usize].left = -1;
            slab.price_nodes[i as usize].color = 1;
            slab.node_count = slab.node_count.checked_add(1).ok_or(MarketError::MathError)?;
            return Ok(i); // so in rust Ok() alone is just an expression it doesn't return the okay value to the user so return is necessary because it return this expression at the last pf the function when no further execution limit left    
        }
    }
    Err(error!(MarketError::MathError))
}

pub fn allocate_order_entry(slab : &mut Slab) -> Result<i32> {
    for i in 0..(MAX_ORDER_ENTRIES as i32){
        if !slab.order_entries[i as usize].occupied {
            slab.order_entries[i as usize].occupied = true;
            slab.order_entries[i as usize].next_in_price = -1;
            return Ok(i)
        }
    }
    Err(error!(MarketError::MathError))
}

pub fn find_price_node_index(slab: &Slab , price : u128) -> Option<i32> {
    let mut current_index = slab.root_price_node;
    while !is_null(current_index){
        let current_price = slab.price_nodes[current_index as usize].key;
        if current_price == price  {
            return Some(current_index);
        }
        if current_price < price {
            current_index = slab.price_nodes[current_index as usize].left;
        }
        else{
            current_index = slab.price_nodes[current_index as usize].right;
        }
    }
    None
}

fn left_rotate(slab: &mut Slab , x:i32){
    let y = slab.price_nodes[x as usize].right;
    if is_null(y){
        return;
    }

    slab.price_nodes[x as usize].right = slab.price_nodes[y as usize].left;
    if !is_null(slab.price_nodes[y as usize].left){
        let l = slab.price_nodes[y as usize].left;
        slab.price_nodes[l as usize].parent = x;
    }
    
    slab.price_nodes[y as usize].parent = slab.price_nodes[x as usize].parent;
    if slab.root_price_node == x {
        slab.root_price_node = y;
    }else{
        let x_parent = slab.price_nodes[x as usize].parent;
        if x == 
    }
}