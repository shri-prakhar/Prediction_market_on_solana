use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenAccount, TokenInterface};

use crate::constants::{ASKS_SEEDS, BIDS_SEED, EMPTY_INDEX, MAX_EVENTS, POSITION_SEED};
use crate::state::{ Event, EventQueue, EventType, Market, MarketStatus, Orderside, Slab, SlabNode, TraderPosition};
use crate::error::MarketError;
use crate::utils::{find_matching_node, free_a_node};

#[derive(Debug , AnchorSerialize ,AnchorDeserialize , Clone )]

pub struct PlaceLimitOrderParams{
    orderside: Orderside ,
    price : u64 ,
    quantity: u64,
    market_id: u64
}

#[derive(Accounts)]
#[instruction(params: PlaceLimitOrderParams)]
pub struct PlaceLimitOrder<'info>{
    #[account(mut)]
    pub trader: Signer<'info>,
    
    #[account(mut)]
    pub trader_usdc : InterfaceAccount<'info , TokenAccount>,

    #[account(init ,
        payer = trader,
        space = 8 + std::mem::size_of::<TraderPosition>(),
        seeds = [POSITION_SEED , &params.market_id.to_le_bytes() , trader.key().as_ref()],
        bump
    )]
    pub trader_postion: Account<'info , TraderPosition>,

    #[account(mut)]
    pub market : Account<'info, Market>,

    #[account(mut , seeds = [ASKS_SEEDS , &params.market_id.to_le_bytes()] , bump)]
    pub asks : Account<'info , Slab>,

    #[account(mut , seeds = [BIDS_SEED , &params.market_id.to_le_bytes()] , bump)]
    pub bids : Account<'info ,Slab>,

    #[account(mut)]
    pub event_queue : Account<'info , EventQueue>,

    pub token_program : Interface<'info , TokenInterface>,  
    pub system_program : Program<'info , System>  
}

pub fn place_limit_order_handler(ctx:Context<PlaceLimitOrder> , params: PlaceLimitOrderParams) -> Result<()>{
    let trader_position = &mut ctx.accounts.trader_postion;
    let market = &mut ctx.accounts.market;

    let queue = &mut ctx.accounts.event_queue;


    require!(market.status == MarketStatus::Open , MarketError::MarketNotOpen);
    
    let mut left_qty = params.quantity;
    let price_key = params.price as u128;
    let now = Clock::get()?.unix_timestamp;

    let ( mut opposite_slab , is_opposite_side_slab) = match params.orderside {
        Orderside::Buy => ( &mut ctx.accounts.asks , false),
        Orderside::Sell => ( &mut ctx.accounts.bids , true),
    };

    while left_qty > 0 {
        if let Some(index) = find_matching_node(&mut opposite_slab, left_qty as u128, is_opposite_side_slab){
           let mut node = opposite_slab.nodes[index as usize];
           let match_quantity = if node.quantity > left_qty { left_qty } else { left_qty };

           let ev = Event{
            event_type :  EventType::Fill,
            maker : node.owner,
            taker : ctx.accounts.trader.key(),
            quantity : match_quantity,
            price: node.key as u64,
            side : match params.orderside { Orderside::Buy => 0u8 , Orderside::Sell => 1u8 },
            order_id : node.order_id,
            time_stamp: now
           };

           if (ctx.accounts.event_queue.events.len() as usize) >= MAX_EVENTS {
            return Err(MarketError::EventQueueFull.into());
           }

           ctx.accounts.event_queue.events.push(ev);
           ctx.accounts.event_queue.count = ctx.accounts.event_queue.count.checked_add(1).ok_or(MarketError::MathError)?;

           node.quantity = node.quantity.checked_sub(match_quantity).ok_or(MarketError::MathError)?;

           if node.quantity == 0 {
            let prev = node.prev;
            let next  = node.next;

            if prev != EMPTY_INDEX {
                opposite_slab.nodes[prev as usize].next = next;
            } else {
                opposite_slab.head_index = next;
            }
            if next != EMPTY_INDEX {
                opposite_slab.nodes[next  as usize].prev = prev;

            }

            free_a_node(opposite_slab, index)?;
           }

           left_qty = left_qty.checked_sub(match_quantity).ok_or(MarketError::MathError)?;
           continue;
        } else {
            break;
        }
    }

    if left_qty > 0 {
        let target_slab = match params.orderside { Orderside::Buy => &mut ctx.accounts.bids , Orderside::Sell => &mut ctx.accounts.asks};
        
    }


    Ok(())
}