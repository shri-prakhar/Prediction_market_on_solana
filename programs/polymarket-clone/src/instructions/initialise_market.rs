use anchor_lang::prelude::*;
use anchor_spl::{ token_interface::{Mint, TokenInterface}};

use crate::{constants::{MARKET_SEED, OUTCOME_POOL_SEED, VAULT_SEED , EVRNT_QUEUE_SEED}, state::{EventQueue, Market, OutcomePool, Vault}};


#[derive(Accounts)]
#[instruction(params:InitializeMarletParams)] 

pub struct InitializeMarket<'info>{
    #[account(mut , signer)]
    pub admin : AccountInfo<'info>,
    #[account(
        init,
        payer = admin,
        seeds = [MARKET_SEED , params.market_id.as_bytes()],
        bump,
        space = 8 + std::mem::size_of::<Market>()
    )]
    pub market: Account<'info , Market>,

    #[account(
        init ,
        payer = admin,
        seeds = [VAULT_SEED , params.market_id.as_bytes()],
        bump,
        space = 8 + std::mem::size_of::<Vault>()
    )]
    pub vault : Account<'info , Vault>,

    #[account(
        init ,
        payer = admin,
        mint::decimals = 6,
        mint::authority = market,
    )]

    pub yes_mint : InterfaceAccount<'info , Mint>,

    #[account(
        init,
        payer = admin,
        mint::decimals = 6,
        mint::authority = market,
    )]

    pub no_mint : InterfaceAccount<'info , Mint>,
    #[account(
        init,
        payer = admin , 
        space = 8 + std::mem::size_of::<OutcomePool>(),
        seeds = [OUTCOME_POOL_SEED , params.market_id.as_bytes()],
        bump
    )]
    pub outcome_pool : Account<'info , OutcomePool>,

    #[account(
        init , 
        payer = admin , 
        space = 8 + std::mem::size_of::<EventQueue>(),
        seeds = [EVRNT_QUEUE_SEED , params.market_id.as_bytes()],
        bump
    )]
    pub event_queue: Account<'info , EventQueue>,

    pub token_program : Interface<'info , TokenInterface>,
    pub system_program: Program<'info , System>,
    pub rent : Sysvar<'info, Rent>,

}

#[repr(C)]
#[derive(AnchorDeserialize , AnchorSerialize , Clone  , Debug)]

pub struct InitializeMarletParams{
    pub market_id: String,
    pub question: String,
    pub description: String,
    pub end_ts: i64,
    pub fee_bps:u16,
}

pub fn initial_market_handler (ctx:Context<InitializeMarket> , params : InitializeMarletParams) -> Result<()>{
    
    let market = &mut ctx.accounts.market;
    market.market_id = params.market_id;
    market.creator = ctx.accounts.admin.key();
    market.question = params.question;
    market.description = params.description;
    market.end_ts = params.end_ts;
    market.status =crate::state::MarketStatus::Open;
    market.yes_mint = ctx.accounts.yes_mint.key();
    market.no_mint = ctx.accounts.no_mint.key();
    market.vault_usdc = ctx.accounts.vault.key();
    market.amm_pool = ctx.accounts.outcome_pool.key();
    market.fee_bps = params.fee_bps;
    market.q_yes = 0;
    market.q_no =0;
    market.b_liquidity = 1_000_00; // initial liquidity constant 
    market.oracle = ctx.accounts.admin.key();
    Ok(())
}