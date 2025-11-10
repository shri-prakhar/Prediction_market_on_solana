use anchor_lang::prelude::*;

use crate::{constants::{EMPTY_INDEX, MAX_SLAB_NODES}, error::MarketError, state::Slab};


pub fn checked_add(a:u64 , b:u64 ) -> Result<u64> {
    a.checked_add(b).ok_or(MarketError::MathError.into()) 
    //The .into() method is a shorthand for converting one type into another. In this case, 
    //it's converting MarketError::MathError into a type that can be used as the error type for Result, 
    //which is expected to be a type that implements the Into trait.
    //Specifically, into() will convert MarketError::MathError into a Box<dyn std::error::Error>
    // (or another type that implements std::error::Error)
}

pub fn checked_sub(a:u64 , b:u64) -> Result<u64>{
    a.checked_sub(b).ok_or(MarketError::MathError.into())
}

pub fn find_best_node_index(slab: &mut Slab) -> Result<i32> {
    Ok(slab.head_index)
}

pub fn allocate_node(slab : &mut Slab ) -> Result<i32> {
    if slab.free_head != EMPTY_INDEX {
        let index = slab.free_head;
        let node = &mut slab.nodes[index as usize];
        slab.free_head = node.next;
        node.next = EMPTY_INDEX;
        slab.node_count = slab.node_count.checked_add(1).ok_or(MarketError::MathError)?;
        Ok(index)
    }else{
        for i in 0..MAX_SLAB_NODES{
            if !slab.nodes[i].occupied {
                slab.node_count = slab.node_count.checked_add(1).ok_or(MarketError::MathError)?;
                slab.nodes[i].occupied = true;
                return Ok(i as i32)
            }
        }
        Err(MarketError::MathError.into())
    }
}


pub fn free_a_node(slab: &mut Slab , index: i32) -> Result<()> {
    let idx = index as usize;

    slab.nodes[idx].occupied = false;
    slab.nodes[idx].owner = Pubkey::default(); //when default trait is being called on Pubkey it sets all the default bytes of pubkey to zero.
    slab.nodes[idx].quantity = 0;
    slab.nodes[idx].key = 0;
    slab.nodes[idx].order_id = 0;
    slab.nodes[idx].time_stamp = 0;
    //push into free list 
    slab.nodes[idx].next = slab.free_head;
    slab.free_head = index;
    slab.node_count= slab.node_count.checked_sub(1).ok_or(MarketError::MathError)?;    
    Ok(())
}

pub fn find_matching_node(slab: &mut Slab , crossing_price : u128 , is_bid_slab : bool) -> Option<i32> {
    // so Slab is a linked list sorted by price so start traversing from head 
    let mut index =slab.head_index;
    if index == EMPTY_INDEX {
        return None;
    }

    while index != EMPTY_INDEX {
        let node = &slab.nodes[index as usize];
        if is_bid_slab {
            //so now slab is bid-side ; for matching a sell incoming at price p
            //you should return key >= p , and , choose highest bids >= p
            if node.key >= crossing_price{
                return Some(index);
            } 
        } else {
            //so now slab is ask-side : so for matching a buy incoming at proce p
            //you should return key<=p , so choose the lowest asks <= p
            if node.key <= crossing_price{
                return Some(index)
            } 
        }
        index = node.next
    } 
    None
}

pub fn slab_insert_node(slab: &mut Slab , node_index: i32) -> Result<()>{
    let node = &slab.nodes[node_index as usize];
    if slab.head_index == EMPTY_INDEX {
        slab.head_index = node_index;
        slab.nodes[node_index as usize].prev = EMPTY_INDEX;
        slab.nodes[node_index as usize].next = EMPTY_INDEX;
        return Ok(())
    }
    //find insertion point
    let mut current = slab.head_index;
    let mut previous = EMPTY_INDEX;

    while current != EMPTY_INDEX {
        let cur_key = slab.nodes[current as usize].key;
        //bid bid slab : higher keys first  ; for ask slab : low keys first
            let should_insert_before = if slab.is_bid{
                node_index_is_higher(node.key, cur_key)
            }else{
                node_index_is_lower(node.key, cur_key)
            };

            if should_insert_before {
                slab.nodes[node_index as usize].next = current;
                slab.nodes[node_index as usize].prev = slab.nodes[current as usize].prev;
                if slab.nodes[current as usize].prev != EMPTY_INDEX {
                    let p = slab.nodes[current as usize].prev;
                    slab.nodes[p as usize].next = node_index;
                } else {
                    slab.head_index = node_index;
                }
                slab.nodes[current as usize].prev = node_index;
                return Ok(());
            }
            previous = current;
            current = slab.nodes[current as usize].next;
            
    }; 

    //this is if all of these failed so append at the last 
    slab.nodes[previous as usize].next =  node_index;
    slab.nodes[node_index as usize].prev = previous;
    slab.nodes[node_index as usize].next = EMPTY_INDEX;
    
    Ok(())
}

#[inline]
pub fn node_index_is_higher(a: u128 , b: u128)  -> bool { a > b }

#[inline]
pub fn node_index_is_lower(a:u128 , b:u128) -> bool { a < b }