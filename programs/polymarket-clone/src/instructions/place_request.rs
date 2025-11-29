use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};

use crate::{
    constants::{
        OPEN_ORDER_SEED, PRICE_PRECISION_SCALE, REQUEST_QUEUE_SEED, VAULT_NO_SEED, VAULT_USDC_SEED,
        VAULT_YES_SEED,
    },
    error::MarketError,
    state::{Market, OpenOrder, OrderSide, OutcomeSide, Request, RequestQueue},
    utils::enqueue_request,
};

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]

pub struct PlaceOrderParams {
    pub req_type: u8,
    pub side: u8,
    pub price: u128,
    pub quantity: u64,
    pub client_id: u64,
    pub outcome: u8,
}

#[derive(Accounts)]
pub struct PlaceOrder<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(mut , seeds = [VAULT_USDC_SEED , &market.market_id.to_le_bytes()] , bump)]
    pub vault_usdc: Account<'info, TokenAccount>,

    #[account(mut , seeds = [VAULT_YES_SEED , &market.market_id.to_le_bytes()] , bump)]
    pub vault_yes: Account<'info, TokenAccount>,

    #[account(mut , seeds = [VAULT_NO_SEED , &market.market_id.to_le_bytes()] , bump)]
    pub vault_no: Account<'info, TokenAccount>,

    #[account(mut , seeds = [OPEN_ORDER_SEED , market.key().as_ref() , owner.key.as_ref()] , bump)]
    pub open_order: Account<'info, OpenOrder>,

    #[account(mut , seeds = [REQUEST_QUEUE_SEED , &market.market_id.to_le_bytes()] , bump)]
    pub request_queue: Account<'info, RequestQueue>,

    #[account(mut)]
    pub from_usdc: Account<'info, TokenAccount>,

    #[account(mut)]
    pub from_outcome: Option<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

pub fn place_order_handler(ctx: Context<PlaceOrder>, params: PlaceOrderParams) -> Result<()> {
    let open_order = &mut ctx.accounts.open_order;

    let order_id = Clock::get()?.slot;

    if params.side != OrderSide::Buy as u8 && params.side != OrderSide::Sell as u8 {
        return err!(MarketError::InvalidSide);
    }

    if params.price <= 0 || params.quantity <= 0 {
        return err!(MarketError::InvalidArgument);
    }

    if params.side == OrderSide::Buy as u8 {
        let reserved_quote = params
            .price
            .checked_mul(params.quantity as u128)
            .ok_or(MarketError::MathError)?
            .checked_div(PRICE_PRECISION_SCALE)
            .ok_or(MarketError::MathError)? as u64;

        if reserved_quote > ctx.accounts.from_usdc.amount {
            return err!(MarketError::InsufficientBalance);
        }

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.from_usdc.to_account_info(),
                to: ctx.accounts.vault_usdc.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        );

        transfer(cpi_ctx, reserved_quote)?;

        open_order.locked_quote = open_order
            .locked_quote
            .checked_add(reserved_quote as u128)
            .ok_or(MarketError::MathError)?;
    } else {
        let from_outcome = ctx
            .accounts
            .from_outcome
            .as_ref()
            .ok_or(MarketError::MathError)?; // we are doing this here is because account is Option<account> as_ref converts this to Option<&account> this way it doesn't pass the ownership

        if params.quantity > from_outcome.amount {
            return err!(MarketError::InsufficientBalance);
        }

        let outcome_vault = if params.outcome == OutcomeSide::Yes as u8 {
            ctx.accounts.vault_yes.to_account_info()
        } else if params.outcome == OutcomeSide::No as u8 {
            ctx.accounts.vault_no.to_account_info()
        } else {
            return err!(MarketError::InvalidSide);
        };

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: from_outcome.to_account_info(),
                to: outcome_vault,
                authority: ctx.accounts.owner.to_account_info(),
            },
        );
        transfer(cpi_ctx, params.quantity)?;

        open_order.locked_base = open_order
            .locked_base
            .checked_add(params.quantity as u128)
            .ok_or(MarketError::MathError)?;
    }

    let request = Request {
        request_type: params.req_type,
        owner: ctx.accounts.owner.key(),
        side: params.side,
        price: params.price,
        open_order: ctx.accounts.open_order.key(),
        quantity: params.quantity,
        order_id,
        client_id: params.client_id,
        outcome: params.outcome,
        timestamp: Clock::get()?.unix_timestamp,
    };

    enqueue_request(&mut ctx.accounts.request_queue, request)?;

    Ok(())
}
