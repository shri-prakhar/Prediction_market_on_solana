use anchor_lang::prelude::*;

use crate::constants::{
    ASKS_SEEDS, BIDS_SEED, EMPTY_INDEX, EVRNT_QUEUE_SEED, MARKET_SEED, MAX_EVENTS,
    MAX_ORDER_PER_TRADER, POSITION_SEED,
};
use crate::error::MarketError;
use crate::state::{Event, EventQueue, EventType, Market, OutcomeSide, Slab, TraderPosition};
use crate::utils::free_a_node;

#[repr(C)]
#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone)]

pub struct CancelOrderParams {
    pub order_id: u64,
    pub market_id: u64,
}

#[derive(Accounts)]
#[instruction(params:CancelOrderParams)]
pub struct CancelOrder<'info> {
    #[account(mut)]
    pub trader: Signer<'info>,

    #[account(mut ,  seeds=[MARKET_SEED , &params.market_id.to_le_bytes()] , bump)]
    pub market: Account<'info, Market>,

    #[account(mut , seeds= [BIDS_SEED , &params.market_id.to_le_bytes()] , bump)]
    pub bids: Account<'info, Slab>,

    #[account(mut , seeds= [ASKS_SEEDS , &params.market_id.to_le_bytes()] , bump)]
    pub asks: Account<'info, Slab>,

    #[account(mut , seeds = [EVRNT_QUEUE_SEED , &params.market_id.to_le_bytes()] , bump)]
    pub event_queue: Account<'info, EventQueue>,

    #[account(mut , seeds= [POSITION_SEED , &params.market_id.to_le_bytes()] , bump)]
    pub trader_position: Account<'info, TraderPosition>,
}

pub fn cancle_order_handler(ctx: Context<CancelOrder>, params: CancelOrderParams) -> Result<()> {
    let trader_position = &mut ctx.accounts.trader_position;

    let mut slot_index: Option<usize> = None;

    for i in 1..MAX_ORDER_PER_TRADER {
        if trader_position.order_ids[i] == params.order_id {
            slot_index = Some(i);
            break;
        }
    }

    require!(slot_index.is_some(), MarketError::OrderNotFound);
    let i = slot_index.unwrap();
    let mut found = false;
    let mut reserved_usdc = 0;
    let mut reserved_outcome = 0;
    let mut n_outcome: OutcomeSide = OutcomeSide::Yes;
    for slab in [&mut ctx.accounts.asks, &mut ctx.accounts.bids].iter_mut() {
        let mut index = slab.head_index;
        while index != EMPTY_INDEX {
            let n = slab.nodes[index as usize];
            if n.occupied && n.order_id == params.order_id {
                reserved_outcome = n.reserved_outcome;
                reserved_usdc = n.reserved_usdc; 
                n_outcome = n.outcome;   
                let prev = n.prev;
                let next = n.next;
                

                if prev != EMPTY_INDEX {
                    slab.nodes[prev as usize].next = next
                } else {
                    slab.head_index = next;
                }

                if next != EMPTY_INDEX {
                    slab.nodes[next as usize].prev = prev;
                }
                free_a_node(slab, index)?;
                found = true;
                break;
            }

            index = n.next;
        }

        if found {
            break;
        }
    }

    require!(found, MarketError::OrderNotFound);

    trader_position.order_ids[i] = 0;
    trader_position.slots_bitmap &= !(1u128 << i);
    trader_position.active_orders = trader_position
        .active_orders
        .checked_sub(1)
        .ok_or(MarketError::MathError)?;

    if (ctx.accounts.event_queue.events.len() as usize) >= MAX_EVENTS {
        return Err(MarketError::EventQueueFull.into());
    }

    ctx.accounts.event_queue.events.push(Event {
        event_type: EventType::Cancel,
        maker: trader_position.owner,
        taker: Pubkey::default(),
        quantity: reserved_outcome,
        price: reserved_usdc,
        side: 0,
        order_id: params.order_id,
        outcome: n_outcome,
        time_stamp: Clock::get()?.unix_timestamp,
    });

    ctx.accounts.event_queue.count = ctx
        .accounts
        .event_queue
        .count
        .checked_add(1)
        .ok_or(MarketError::MathError)?;

    Ok(())
}
