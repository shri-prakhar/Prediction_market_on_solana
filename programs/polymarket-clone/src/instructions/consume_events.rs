use anchor_lang::prelude::*;

use crate::state::{EventQueue, Market, TraderPosition, Vault};

#[derive(Accounts)]

pub struct ConsumeEvents<'info>{
    #[account(mut)]
    pub trader_position : Account<'info , TraderPosition>,

    #[account(mut)]
    pub market : Account<'info , Market>,

    #[account(mut)]
    pub event_queue : Account<'info , EventQueue>,

    #[account(mut)]
    pub vault : Account<'info , Vault>,

}

pub fn consume_events_handler(ctx:Context<ConsumeEvents> ) -> Result<()>{
    let queue = &mut ctx.accounts.event_queue;
    let trader_position = &mut ctx.accounts.trader_position;
    let vault = &mut ctx.accounts.vault;
    
    
    
    Ok(())
}