use anchor_lang::prelude::*;

use crate::{
    constants::{ASKS_SEEDS, BIDS_SEED, EVENT_QUEUE_SEED, REQUEST_QUEUE_SEED},
    error::MarketError,
    state::{Event, EventQueue, EventType, Market, OrderSide, RequestQueue, RequestType, Slab},
    utils::{
        allocate_order_entry, append_order_to_price, dequeue_requests, find_best_price_node_index,
        pop_order_from_prices, push_event,
    },
};

#[derive(Accounts)]
pub struct MatchRequest<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(mut , seeds = [BIDS_SEED , &market.market_id.to_le_bytes()] , bump)]
    pub bids: Account<'info, Slab>,

    #[account(mut , seeds = [ASKS_SEEDS , &market.market_id.to_le_bytes()], bump)]
    pub asks: Account<'info, Slab>,

    #[account(mut , seeds = [REQUEST_QUEUE_SEED , &market.market_id.to_le_bytes()] , bump)]
    pub request_queue: Account<'info, RequestQueue>,

    #[account(mut , seeds = [EVENT_QUEUE_SEED , &market.market_id.to_le_bytes()] , bump)]
    pub event_queue: Account<'info, EventQueue>,
}

pub fn match_order_handler(ctx: Context<MatchRequest>, max_requests: u16) -> Result<()> {
    let req = dequeue_requests(&mut ctx.accounts.request_queue, max_requests as usize)?;
    for req in req.into_iter() {
        if req.request_type == RequestType::CancelOrder as u8 {
            let event = Event {
                event_type: EventType::Cancel as u8,
                makers_open_orders: req.owner,
                maker_slot: 0,
                taker_open_orders: Pubkey::default(),
                taker_side: req.side,
                taker_slot: 0,
                price: 0,
                quantity: 0,
                order_id: req.order_id,
                outcome: req.outcome,
                timestamp: Clock::get()?.unix_timestamp,
            };
            push_event(&mut ctx.accounts.event_queue, event)?;
            continue;
        }

        let (opposite_slab, _own_slab) = if req.side == OrderSide::Buy as u8 {
            (&mut ctx.accounts.asks, &mut ctx.accounts.bids)
        } else {
            (&mut ctx.accounts.bids, &mut ctx.accounts.asks)
        };

        let mut left_quantity = req.quantity;

        loop {
            if left_quantity == 0 {
                break;
            }
            let maybe_best_price = find_best_price_node_index(&opposite_slab);
            if maybe_best_price.is_none() {
                break;
            }
            let best_price_index = maybe_best_price.ok_or(MarketError::MathError)?;
            let best_price = opposite_slab.price_nodes[best_price_index as usize].key;

            let crossing = if req.side == OrderSide::Buy as u8 {
                best_price <= req.price
            } else {
                best_price >= req.price
            };

            if !crossing {
                break;
            }

            let head_entry_index = opposite_slab.price_nodes[best_price_index as usize].order_head;
            if head_entry_index == -1 {
                break;
            }
            let entry = opposite_slab.order_entries[head_entry_index as usize];
            let matched_quantity = if entry.quantity > left_quantity {
                left_quantity
            } else {
                entry.quantity
            };

            let event = Event {
                event_type: EventType::Fill as u8,
                makers_open_orders: entry.open_order_owner,
                maker_slot: entry.owner_slot,
                taker_open_orders: req.open_order,
                taker_slot: 0,
                taker_side: req.side,
                price: best_price,
                quantity: matched_quantity,
                order_id: entry.order_id,
                outcome: req.outcome,
                timestamp: Clock::get()?.unix_timestamp,
            };

            push_event(&mut ctx.accounts.event_queue, event)?;

            if entry.quantity > matched_quantity {
                opposite_slab.order_entries[head_entry_index as usize].quantity =
                    entry.quantity - matched_quantity;
            } else {
                pop_order_from_prices(opposite_slab, best_price_index)?;
            }

            left_quantity = left_quantity
                .checked_sub(matched_quantity)
                .ok_or(MarketError::MathError)?;
        }

        if left_quantity > 0 {
            let event = Event {
                event_type: EventType::Fill as u8,
                makers_open_orders: ctx.accounts.market.key(),
                maker_slot: 0,
                taker_open_orders: req.open_order,
                taker_slot: 0,
                taker_side: req.side,
                price: req.price,
                quantity: left_quantity,
                order_id: req.order_id,
                outcome: req.outcome,
                timestamp: Clock::get()?.unix_timestamp,
            };

            push_event(&mut ctx.accounts.event_queue, event)?;
        }
        if left_quantity > 0 {
            // Insert unmatched order into own slab
            let own_slab = if req.side == OrderSide::Buy as u8 {
                &mut ctx.accounts.bids
            } else {
                &mut ctx.accounts.asks
            };

            let price_node_index = crate::utils::insert_price_node_by_tree(own_slab, req.price)?;

            let order_entry_index = allocate_order_entry(own_slab)?;
            let order_entry = &mut own_slab.order_entries[order_entry_index as usize];
            order_entry.order_id = req.order_id;
            order_entry.open_order_owner = req.open_order;
            order_entry.quantity = left_quantity;
            order_entry.owner_slot = 0; // TODO: Find available slot in OpenOrder
            order_entry.reserved_amount = if req.side == OrderSide::Buy as u8 {
                (req.price
                    .checked_mul(left_quantity as u128)
                    .ok_or(MarketError::MathError)?
                    .checked_div(crate::constants::PRICE_PRECISION_SCALE)
                    .ok_or(MarketError::MathError)?) as u64
            } else {
                0
            };
            append_order_to_price(own_slab, price_node_index, order_entry_index)?;
        }
    }
    Ok(())
}
