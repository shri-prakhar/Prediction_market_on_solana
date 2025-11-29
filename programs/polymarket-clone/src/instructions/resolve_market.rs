use anchor_lang::prelude::*;

use crate::{
    error::MarketError,
    state::{Market, MarketStatus, OutcomeSide},
};

#[derive(Accounts)]
pub struct ResolveMarket<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(mut)]
    pub admin: Signer<'info>,
}

pub fn resolve_market_handler(ctx: Context<ResolveMarket>, winner: u8) -> Result<()> {
    if ctx.accounts.market.creator != ctx.accounts.admin.key() {
        return err!(MarketError::Unauthorized);
    }

    if winner == OutcomeSide::Yes as u8 {
        ctx.accounts.market.status = MarketStatus::ResolvedYes
    } else if winner == OutcomeSide::No as u8 {
        ctx.accounts.market.status = MarketStatus::ResolvedNo
    }

    Ok(())
}
