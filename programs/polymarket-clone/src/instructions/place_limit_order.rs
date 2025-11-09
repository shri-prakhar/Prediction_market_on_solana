use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenAccount, TokenInterface};

use crate::state::{ EventQueue, EventType, Market, MarketStatus, Orderside, SlabHeader, SlabNode, TraderPosition};
use crate::error::MarketError;

#[derive(Debug , AnchorSerialize ,AnchorDeserialize , Clone )]

pub struct PlaceLimitOrderParams{
    orderside: Orderside ,
    price : u64 ,
    quantity: u64,
}

#[derive(Accounts)]

pub struct PlaceLimitOrder<'info>{
    #[account(mut)]
    pub trader: Signer<'info>,
    
    #[account(mut)]
    pub trader_usdc : InterfaceAccount<'info , TokenAccount>,

    #[account(mut)]
    pub trader_postion: Account<'info , TraderPosition>,

    #[account(mut)]
    pub market : Account<'info, Market>,

    #[account(mut)]
    pub orderbook : Account<'info , SlabHeader>,

    #[account(mut)]
    pub event_queue : Account<'info , EventQueue>,

    pub token_program : Interface<'info , TokenInterface>,    
}

pub fn place_limit_order_handler(ctx:Context<PlaceLimitOrder> , params: PlaceLimitOrderParams) -> Result<()>{
    let trader_position = &mut ctx.accounts.trader_postion;
    let market = &mut ctx.accounts.market;
    let orderbook = &mut ctx.accounts.orderbook;
    let queue = &mut ctx.accounts.event_queue;


    require!(market.status == MarketStatus::Open , MarketError::MarketNotOpen);

    let order_id  = Clock::get()?.slot;
     SlabNode {
        key:params.price ,
        owner: ctx.accounts.trader.key(),
        quantitity: params.quantity,
        order_id,
        next: None,
        prev: None
    };

    orderbook.leaf_count = orderbook.leaf_count.checked_add(1).ok_or(MarketError::MathError)?;

    trader_position.active_orders += 1;
    let order_id_index  = trader_position.active_orders - 1;
    trader_position.order_ids[order_id_index as usize] = order_id;

    queue.events.push(crate::state::Event {
         event_type: EventType::Fill, 
         trader: ctx.accounts.trader.key(), 
         quantity: params.quantity, 
         price: params.price, 
         side: match params.orderside {
             Orderside::Buy => 0,
             Orderside::Sell => 1
         }, 
         order_id
        });
    
    Ok(())
}