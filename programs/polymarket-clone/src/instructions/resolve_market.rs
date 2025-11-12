use anchor_lang::prelude::*;

use crate::{constants::MARKET_SEED, error::MarketError, state::{Market, MarketStatus, OutcomeSide}};


#[derive(AnchorSerialize , AnchorDeserialize , Debug , Clone )]

pub struct ResolveMarketParams{
    winner: OutcomeSide,
    market_id : u64,
} 

#[derive(Accounts)]
#[instruction(params: ResolveMarketParams)]
pub struct ResolveMarket<'info>{
    #[account(mut , seeds = [MARKET_SEED , &params.market_id.to_le_bytes()] , bump)]
    pub market : Account<'info , Market>,

    pub oracle : Signer<'info>
}

pub fn resolve_market_handler(ctx: Context<ResolveMarket> , params: ResolveMarketParams) -> Result<()> {
    let market = &mut ctx.accounts.market;
    require!(matches!(market.status , MarketStatus::Open) , MarketError::MarketNotOpen);

    market.status = MarketStatus::Resolved { winner: ( params.winner ) };
    Ok(())
}
