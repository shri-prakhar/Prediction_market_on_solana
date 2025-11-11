use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{
    constants::{
        ASKS_SEEDS, BIDS_SEED, EVRNT_QUEUE_SEED, MARKET_SEED, OUTCOME_POOL_SEED, VAULT_SEED,
    },
    state::{EventQueue, Market, OutcomePool, Slab, Vault},
};

#[derive(Accounts)]
#[instruction(params:InitializeMarletParams)]

pub struct InitializeMarket<'info> {
    #[account(mut, signer)]
    pub admin: AccountInfo<'info>,

    #[account(
        init,
        payer = admin,
        seeds = [MARKET_SEED , &params.market_id.to_le_bytes()],
        bump,
        space = 8 + std::mem::size_of::<Market>()
    )]
    pub market: Account<'info, Market>,

    #[account(
        init ,
        payer = admin,
        seeds = [VAULT_SEED , &params.market_id.to_le_bytes()],
        bump,
        space = 8 + std::mem::size_of::<Vault>()
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        init ,
        payer = admin,
        mint::decimals = 6,
        mint::authority = market,
    )]
    pub yes_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = admin,
        mint::decimals = 6,
        mint::authority = market,
    )]
    pub no_mint: InterfaceAccount<'info, Mint>,

    // this is usdc mint account exist already
    pub usdc_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init ,
        payer = admin,
        token::mint = usdc_mint,
        token::authority = market,
    )]
    pub vault_usdc: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = admin,
        token::mint = yes_mint,
        token::authority = market, 
    )]
    pub vault_yes: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = admin ,
        token::mint = usdc_mint,
        token::authority = market,
    )]
    pub vault_no: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = admin, 
        space = 8 + std::mem::size_of::<OutcomePool>(),
        seeds = [OUTCOME_POOL_SEED , &params.market_id.to_le_bytes()],
        bump
    )]
    pub outcome_pool: Account<'info, OutcomePool>,

    #[account(
        init, 
        payer = admin, 
        space = 8 + std::mem::size_of::<EventQueue>(),
        seeds = [EVRNT_QUEUE_SEED , &params.market_id.to_le_bytes()],
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

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[repr(C)]
#[derive(AnchorDeserialize, AnchorSerialize, Clone, Debug)]

pub struct InitializeMarletParams {
    pub market_id: u64,
    pub question: String,
    pub description: String,
    pub end_ts: i64,
    pub fee_bps: u16,
}

pub fn initial_market_handler(
    ctx: Context<InitializeMarket>,
    params: InitializeMarletParams,
) -> Result<()> {
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
    market.vault = ctx.accounts.vault.key();
    market.usdc_mint = ctx.accounts.usdc_mint.key();
    market.vault_usdc = ctx.accounts.vault_usdc.key();
    market.vault_yes = ctx.accounts.vault_yes.key();
    market.vault_no = ctx.accounts.vault_no.key();
    market.amm_pool = ctx.accounts.outcome_pool.key();
    market.fee_bps = params.fee_bps;
    market.q_yes = 0;
    market.q_no = 0;
    market.b_liquidity = 1_000_00; // initial liquidity constant
    market.oracle = ctx.accounts.admin.key();
    market.bump = ctx.bumps.market;

    let vault = &mut ctx.accounts.vault;
    vault.market = market.key();
    vault.token_mint = ctx.accounts.usdc_mint.key();
    vault.token_account = ctx.accounts.vault_usdc.key();
    vault.token_collateral = 0;
    vault.bump = ctx.bumps.vault;

    let outcome_pool = &mut ctx.accounts.outcome_pool;
    outcome_pool.market = market.key();
    outcome_pool.yes_reserve = 0;
    outcome_pool.no_reserve = 0;
    outcome_pool.total_lp_shares = 0;
    outcome_pool.bump = ctx.bumps.outcome_pool;

    Ok(())
}
