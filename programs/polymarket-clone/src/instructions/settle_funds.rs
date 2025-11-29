use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};

use crate::{
    constants::{MARKET_SEED, OPEN_ORDER_SEED},
    state::{Market, OpenOrder},
};

#[derive(Accounts)]

pub struct SettleFunds<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(mut , seeds = [OPEN_ORDER_SEED , market.key().as_ref() , owner.key().as_ref()] , bump)]
    pub open_order: Account<'info, OpenOrder>,

    #[account(mut)]
    pub vault_usdc: Account<'info, TokenAccount>,

    #[account(mut)]
    pub owner_usdc: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn settle_funds_handler(ctx: Context<SettleFunds>) -> Result<()> {
    let open_order = &mut ctx.accounts.open_order;
    let free_quote = open_order.free_quote;
    if free_quote == 0 {
        return Ok(());
    }
    let amount = free_quote as u64;

    let bump = ctx.accounts.market.bump;
    let seeds: &[&[&[u8]]] = &[&[
        MARKET_SEED,
        &ctx.accounts.market.market_id.to_le_bytes(),
        &[bump],
    ]];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.vault_usdc.to_account_info(),
            to: ctx.accounts.owner_usdc.to_account_info(),
            authority: ctx.accounts.market.to_account_info(),
        },
        seeds,
    );

    transfer(cpi_ctx, amount)?;

    open_order.free_quote = 0;

    Ok(())
}
