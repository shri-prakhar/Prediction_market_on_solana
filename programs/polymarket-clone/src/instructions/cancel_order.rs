use anchor_lang::prelude::*;

use crate::state::{Event, EventQueue, EventType, Market, SlabHeader, TraderPosition};
use crate::error::MarketError;
#[derive(Debug , AnchorSerialize , AnchorDeserialize ,Clone)]

pub struct CancelOrderParams{
    pub order_id : u64
}


#[derive(Accounts)]

pub struct CancelOrder<'info>{
    #[account(mut)]
    pub trader : Signer<'info>,

    #[account(mut)]
    pub market : Account<'info , Market>,

    #[account(mut)]
    pub orderbook : Account<'info , SlabHeader>,

    #[account(mut)]
    pub event_queue : Account<'info , EventQueue>,

    #[account(mut)]
    pub trader_position: Account<'info , TraderPosition>,
}

pub fn cancle_order_handler(ctx:Context<CancelOrder> , params: CancelOrderParams) -> Result<()> {
    let trader_position = &mut ctx.accounts.trader_position;
    let queue = &mut ctx.accounts.event_queue;


    let mut found = false;

    for id in trader_position.order_ids.iter_mut() {
        if *id == params.order_id {
            *id = 0;
            found=true;
            break;
        }
    }

    require!(found , MarketError::OrderNotFound);

    queue.events.push(Event{
        event_type: EventType::Fill,
        trader: ctx.accounts.trader.key(),
        quantity:0,
        price:0,
        side:0,
        order_id: params.order_id,
    });
    
    Ok(())
}