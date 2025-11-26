use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};

use crate::{
    constants::{OPEN_ORDER_SEED, VAULT_USDC_SEED},
    error::MarketError,
    state::{Market, OpenOrder},
};

#[derive(Accounts)]

pub struct DepositToOpenOrders<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(mut)]
    pub from_usdc: Account<'info, TokenAccount>,

    #[account(mut , seeds = [VAULT_USDC_SEED , &market.market_id.to_le_bytes()] , bump)]
    pub vault_usdc: Account<'info, TokenAccount>,

    #[account(mut , seeds=[OPEN_ORDER_SEED , market.key().as_ref() , owner.key().as_ref()] , bump)]
    pub open_orders: Account<'info, OpenOrder>,

    pub token_program: Program<'info, Token>,
}

pub fn deposit_handler(ctx: Context<DepositToOpenOrders>, amount: u64) -> Result<()> {
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.from_usdc.to_account_info(),
            to: ctx.accounts.vault_usdc.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        },
    );
    transfer(cpi_ctx, amount)?;

    let open_order = &mut ctx.accounts.open_orders;
    open_order.free_quote = open_order
        .free_quote
        .checked_add(amount as u128)
        .ok_or(MarketError::MathError)?;

    Ok(())
}
