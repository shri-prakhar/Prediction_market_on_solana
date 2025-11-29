use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;
use crate::instructions::*;
declare_id!("2j64V9Te3wcmWZnkZDDSd3iA5YYfwErPAGeD9ip7i5BD");

#[program]
pub mod polymarket_clone {
    use super::*;

    pub fn initialize_market(
        ctx: Context<InitializeMarketAccounts>,
        params: InitializeMarketParams,
    ) -> Result<()> {
        initial_market_handler(ctx, params)
    }

    pub fn place_order_request(ctx: Context<PlaceOrder>, params: PlaceOrderParams) -> Result<()> {
        place_order_handler(ctx, params)
    }

    pub fn create_open_orders(ctx: Context<CreateOpenOrders>) -> Result<()> {
        open_order_handler(ctx)
    }

    pub fn deposit_to_open_orders(ctx: Context<DepositToOpenOrders>, amount: u64) -> Result<()> {
        deposit_handler(ctx, amount)
    }

    pub fn cancel_order_request(ctx: Context<CancelOrder>, order_id: u64) -> Result<()> {
        cancel_order_handler(ctx, order_id)
    }

    pub fn match_request(ctx: Context<MatchRequest>, max_requests: u16) -> Result<()> {
        match_order_handler(ctx, max_requests)
    }

    pub fn consume_events<'info>(
        ctx: Context<'_, '_, 'info, 'info, ConsumeEvents<'info>>,
        max_events: u16,
    ) -> Result<()> {
        consume_events_handler(ctx, max_events)
    }

    pub fn settle_funds(ctx: Context<SettleFunds>) -> Result<()> {
        settle_funds_handler(ctx)
    }

    pub fn resolve_market(ctx: Context<ResolveMarket>, winner: u8) -> Result<()> {
        resolve_market_handler(ctx, winner)
    }

    pub fn claim_reward(ctx: Context<ClaimReward>, amount: u64) -> Result<()> {
        claim_reward_handler(ctx, amount)
    }

    pub fn split_tokens(ctx: Context<SplitToken>, params: SplitOrderParams) -> Result<()> {
        split_tokens_handler(ctx, params)
    }

    pub fn merge_tokens(ctx: Context<MergeTokens>, params: MergeTokensParams) -> Result<()> {
        merge_tokens_handler(ctx, params)
    }
}
