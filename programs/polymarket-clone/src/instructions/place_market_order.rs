use anchor_lang::prelude::*;
use anchor_spl::{token::{Transfer, transfer}, token_interface::{TokenAccount, TokenInterface}};

use crate::state::{Market, OutcomePool, OutcomeSide, Vault};

#[derive(Accounts)]

pub struct PlaceMarketOrder<'info>{
    #[account(mut)]
    pub trader : Signer<'info>,

    #[account(mut)]
    pub trader_usdc : InterfaceAccount<'info , TokenAccount>,

    #[account(mut)]
    pub trader_yes: InterfaceAccount<'info , TokenAccount>,

    #[account(mut)]
    pub trader_no : InterfaceAccount<'info , TokenAccount>,

    #[account(mut)]
    pub market: Account<'info , Market>,

    #[account(mut)]
    pub vault : Account<'info , Vault>,

    #[account(mut)]
    pub outcome_pool : Account<'info , OutcomePool>,

    pub token_program: Interface<'info , TokenInterface>,
}

#[repr(C)]
#[derive(Debug, AnchorSerialize , AnchorDeserialize , Clone )]

pub struct MarketOrderParams{
    side : OutcomeSide,
    amount : u64,
}


pub fn place_market_order_handler(ctx: Context<PlaceMarketOrder> , params : MarketOrderParams) -> Result<()>{
    let market = &mut ctx.accounts.market;
    let pool = &mut ctx.accounts.outcome_pool;

    let b = market.b_liquidity as f64;
    let q_yes = market.q_yes as f64;
    let q_no = market.q_no as f64;

    let (final_cost  , new_yes  , new_no) = match params.side {
        OutcomeSide::Yes => {
            let old_cost = b * (((q_yes)/b).exp() + ((q_no) / b).exp()).ln();
            let new_cost = b* (((q_yes + params.amount as f64)/b).exp() + ((q_no) / b).exp()).ln();

            let final_cost = new_cost - old_cost;

            (final_cost , q_yes + params.amount as f64 , q_no )
        }
        OutcomeSide::No => {
            let old_cost  = b * (((q_no )/ b).exp() + ((q_no) / b).exp()).ln();
            let new_cost = b * (((q_yes) / b).exp() + ((q_no + params.amount as f64)/b).exp()).ln();

            let final_cost = new_cost - old_cost;

            (final_cost , q_yes , q_no + params.amount as f64)
        }
    };

    let transfer_ix = Transfer{
        from: ctx.accounts.trader_usdc.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
        authority: ctx.accounts.trader.to_account_info()
    };  

    transfer(CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_ix), final_cost as u64)?;

    if params.side == OutcomeSide::Yes {
        pool.yes_reserve = new_yes as u128;
        pool.no_reserve = new_no as u128;
    } else {
        pool.yes_reserve = new_yes as u128;
        pool.no_reserve = new_no as u128;
    }
    
    Ok(())
} 

