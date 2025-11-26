use anchor_lang::prelude::*;

use crate::{
    constants::OPEN_ORDER_SEED,
    state::{Market, OpenOrder},
};

#[derive(Accounts)]
pub struct CreateOpenOrders<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(
        init,
        payer = owner,
        space = std::mem::size_of::<OpenOrder>(),
        seeds = [OPEN_ORDER_SEED , market.key().as_ref() , owner.key().as_ref()],
        bump
    )]
    pub open_order: Account<'info, OpenOrder>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn open_order_handler(ctx: Context<CreateOpenOrders>) -> Result<()> {
    let open_order = &mut ctx.accounts.open_order;
    open_order.owner = ctx.accounts.owner.key();
    open_order.market = ctx.accounts.market.key();
    open_order.free_base = 0;
    open_order.free_quote = 0;
    open_order.locked_base = 0;
    open_order.locked_quote = 0;
    open_order.slots_bitmaps = 0;
    open_order.bump = ctx.bumps.open_order;

    for i in 0..open_order.slots.len() {
        open_order.slots[i].active = false;
    }

    Ok(())
}
