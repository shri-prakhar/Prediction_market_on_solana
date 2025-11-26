use anchor_lang::prelude::*;

use crate::{
    constants::REQUEST_QUEUE_SEED,
    state::{Market, Request, RequestQueue, RequestType},
    utils::enqueue_request,
};

#[derive(Accounts)]

pub struct CancelOrder<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut , seeds = [REQUEST_QUEUE_SEED, &market.market_id.to_le_bytes()] , bump)]
    pub request_queue: Account<'info, RequestQueue>,
}

pub fn cancel_order_handler(ctx: Context<CancelOrder>, order_id: u64) -> Result<()> {
    let request = Request {
        request_type: RequestType::CancelOrder as u8,
        owner: ctx.accounts.owner.key(),
        open_order: Pubkey::default(),
        side: 0,
        price: 0,
        quantity: 0,
        order_id: order_id,
        client_id: 0,
        outcome: 0,
        timestamp: Clock::get()?.unix_timestamp,
    };

    enqueue_request(&mut ctx.accounts.request_queue, request)?;
    Ok(())
}
