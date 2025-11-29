use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
    constants::{
        ASKS_SEEDS, BIDS_SEED, EVENT_QUEUE_SEED, FEE_VAULT_USDC, MARKET_SEED, REQUEST_QUEUE_SEED,
        VAULT_NO_SEED, VAULT_USDC_SEED, VAULT_YES_SEED,
    },
    error::MarketError,
    state::{EventQueue, Market, RequestQueue, Slab},
    utils::initialize_slab,
};

#[derive(Accounts)]
#[instruction(params:InitializeMarketParams)]

pub struct InitializeMarketAccounts<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        seeds = [MARKET_SEED , &params.market_id.to_le_bytes()],
        bump,
        space = 8 + std::mem::size_of::<Market>()
    )]
    pub market: Account<'info, Market>,

    #[account(
        init,
        payer = admin,
        space = 8 + std::mem::size_of::<RequestQueue>(),
        seeds = [REQUEST_QUEUE_SEED , &params.market_id.to_le_bytes()],
        bump
    )]
    pub request_queue: Account<'info, RequestQueue>,

    #[account(
        init,
        payer = admin,
        space = 8 + std::mem::size_of::<EventQueue>(),
        seeds = [EVENT_QUEUE_SEED , &params.market_id.to_le_bytes()],
        bump
    )]
    pub event_queue: Account<'info, EventQueue>,

    #[account(
        init,
        payer = admin,
        space = 8 + std::mem::size_of::<Slab>(),
        seeds = [BIDS_SEED , &params.market_id.to_le_bytes()],
        bump
    )]
    pub bids: Account<'info, Slab>,

    #[account(
        init,
        payer = admin,
        space = 8 + std::mem::size_of::<Slab>(),
        seeds = [ASKS_SEEDS , &params.market_id.to_le_bytes()],
        bump
    )]
    pub asks: Account<'info, Slab>,

    #[account(
        init ,
        payer = admin,
        mint::decimals = 6,
        mint::authority = market,
    )]
    pub yes_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = admin,
        mint::decimals = 6,
        mint::authority = market,
    )]
    pub no_mint: Account<'info, Mint>,

    // this is usdc mint account exist already
    pub usdc_mint: Account<'info, Mint>,

    #[account(
        init ,
        payer = admin,
        token::mint = usdc_mint,
        token::authority = market,
        seeds = [VAULT_USDC_SEED , &params.market_id.to_le_bytes()],
        bump
    )]
    pub vault_usdc: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = admin,
        token::mint = yes_mint,
        token::authority = market,
        seeds = [VAULT_YES_SEED , &params.market_id.to_le_bytes()],
        bump
    )]
    pub vault_yes: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = admin ,
        token::mint = no_mint,
        token::authority = market,
        seeds = [VAULT_NO_SEED , &params.market_id.to_le_bytes()],
        bump
    )]
    pub vault_no: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = admin,
        token::mint = usdc_mint,
        token::authority = market,
        seeds = [FEE_VAULT_USDC , &params.market_id.to_le_bytes()],
        bump
    )]
    pub fee_vault_usdc: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[repr(C)]
#[derive(AnchorDeserialize, AnchorSerialize, Clone, Debug)]

pub struct InitializeMarketParams {
    pub market_id: u64,
    pub question: String,
    pub description: String,
    pub end_ts: i64,
    pub fee_bps: u16,
    pub cranker_fee_bps: u16,
}

pub fn initial_market_handler(
    ctx: Context<InitializeMarketAccounts>,
    params: InitializeMarketParams,
) -> Result<()> {
    // Validate inputs
    require!(params.fee_bps <= 10000, MarketError::InvalidArgument);
    require!(
        params.cranker_fee_bps <= 10000,
        MarketError::InvalidArgument
    );
    require!(
        params.end_ts > Clock::get()?.unix_timestamp,
        MarketError::InvalidArgument
    );
    require!(
        params.question.len() <= crate::constants::MAX_MARKET_QUESTION,
        MarketError::InvalidArgument
    );
    require!(
        params.description.len() <= crate::constants::MAX_MARKET_DESC,
        MarketError::InvalidArgument
    );
    let market = &mut ctx.accounts.market;
    market.market_id = params.market_id;
    market.creator = ctx.accounts.admin.key();
    market.question = params.question;
    market.description = params.description;
    market.end_ts = params.end_ts;
    market.status = crate::state::MarketStatus::Open;
    market.asks = ctx.accounts.asks.key();
    market.bids = ctx.accounts.bids.key();
    market.yes_mint = ctx.accounts.yes_mint.key();
    market.no_mint = ctx.accounts.no_mint.key();
    market.usdc_mint = ctx.accounts.usdc_mint.key();
    market.vault_usdc = ctx.accounts.vault_usdc.key();
    market.vault_yes = ctx.accounts.vault_yes.key();
    market.vault_no = ctx.accounts.vault_no.key();
    market.event_queue = ctx.accounts.event_queue.key();
    market.request_queue = ctx.accounts.request_queue.key();
    market.fee_vault_usdc = ctx.accounts.fee_vault_usdc.key();
    market.fee_bps = params.fee_bps;
    market.cranker_reward_bps = params.cranker_fee_bps;
    market.q_yes = 0;
    market.q_no = 0;
    market.b_liquidity = 1_000_00; // initial liquidity constant
    market.oracle = ctx.accounts.admin.key();
    market.bump = ctx.bumps.market;

    //initializing request queue
    ctx.accounts.request_queue.head = 0;
    ctx.accounts.request_queue.count = 0;
    ctx.accounts.request_queue.bump = ctx.bumps.request_queue;

    //initializing event queue
    ctx.accounts.event_queue.head = 0;
    ctx.accounts.event_queue.count = 0;
    ctx.accounts.event_queue.bump = ctx.bumps.event_queue;

    //initializing market slabs
    let asks = &mut ctx.accounts.asks;
    initialize_slab(asks, false, ctx.bumps.asks);

    let bids = &mut ctx.accounts.bids;
    initialize_slab(bids, true, ctx.bumps.bids);

    Ok(())
}
