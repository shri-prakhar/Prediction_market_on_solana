use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Transfer};
use anchor_spl::token_interface::{TokenAccount, TokenInterface};

use crate::constants::{
    ASKS_SEEDS, BIDS_SEED, EMPTY_INDEX, MAX_EVENTS, MAX_ORDER_PER_TRADER, POSITION_SEED,
    PRICE_PRICISION_SCALE,
};
use crate::error::MarketError;
use crate::state::{
    Event, EventQueue, EventType, Market, MarketStatus, Orderside, OutcomeSide, Slab,
    TraderPosition,
};
use crate::utils::{allocate_node, find_matching_node, free_a_node, slab_insert_node};

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone)]

pub struct PlaceLimitOrderParams {
    orderside: Orderside, // For Buy or Sell
    outcome: OutcomeSide, // For Yes or No
    price: u64,           // price should be scaled by PRICE_PRECISION
    quantity: u64,
    market_id: u64,
}

#[derive(Accounts)]
#[instruction(params: PlaceLimitOrderParams)]
pub struct PlaceLimitOrder<'info> {
    #[account(mut)]
    pub trader: Signer<'info>,

    #[account(mut)]
    pub trader_usdc: InterfaceAccount<'info, TokenAccount>,

    /// This is only needed when working with Optional accounts that if there exists a account or not
    /// user will give or not because it's only needed here for sell order
    #[account(mut)]
    pub trader_outcome: Option<InterfaceAccount<'info, TokenAccount>>,

    #[account(init ,
        payer = trader,
        space = 8 + std::mem::size_of::<TraderPosition>(),
        seeds = [POSITION_SEED , &params.market_id.to_le_bytes() , trader.key().as_ref()],
        bump
    )]
    pub trader_postion: Account<'info, TraderPosition>,

    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(mut , seeds = [ASKS_SEEDS , &params.market_id.to_le_bytes()] , bump)]
    pub asks: Account<'info, Slab>,

    #[account(mut , seeds = [BIDS_SEED , &params.market_id.to_le_bytes()] , bump)]
    pub bids: Account<'info, Slab>,

    #[account(mut)]
    pub vault_usdc: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub vault_yes: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub vault_no: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub event_queue: Account<'info, EventQueue>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn place_limit_order_handler(
    ctx: Context<PlaceLimitOrder>,
    params: PlaceLimitOrderParams,
) -> Result<()> {
    let market = &mut ctx.accounts.market;

    require!(
        market.status == MarketStatus::Open,
        MarketError::MarketNotOpen
    );

    let mut left_qty = params.quantity;
    let price_key = params.price as u128;
    let now = Clock::get()?.unix_timestamp;

    let (mut opposite_slab, is_opposite_side_slab) = match params.orderside {
        Orderside::Buy => (&mut ctx.accounts.asks, false),
        Orderside::Sell => (&mut ctx.accounts.bids, true),
    };

    while left_qty > 0 {
        if let Some(index) =
            find_matching_node(&mut opposite_slab, left_qty as u128, is_opposite_side_slab)
        {
            let mut node = opposite_slab.nodes[index as usize];
            let match_quantity = if node.quantity > left_qty {
                left_qty
            } else {
                node.quantity
            };

            let ev = Event {
                event_type: EventType::Fill,
                maker: node.owner,
                taker: ctx.accounts.trader.key(),
                quantity: match_quantity,
                price: node.key as u64,
                side: match params.orderside {
                    Orderside::Buy => 0u8,
                    Orderside::Sell => 1u8,
                },
                order_id: node.order_id,
                outcome: params.outcome,
                time_stamp: now,
            };

            if (ctx.accounts.event_queue.events.len() as usize) >= MAX_EVENTS {
                return Err(MarketError::EventQueueFull.into());
            }

            if node.reserved_usdc > 0 {
                let reduced_usdc = (node.key)
                    .checked_mul(match_quantity as u128)
                    .ok_or(MarketError::MathError)?
                    .checked_div(PRICE_PRICISION_SCALE)
                    .ok_or(MarketError::MathError)? as u64;
                node.reserved_usdc = node
                    .reserved_usdc
                    .checked_sub(reduced_usdc)
                    .ok_or(MarketError::MathError)?;
            } else if node.reserved_outcome > 0 {
                node.reserved_outcome = node
                    .reserved_outcome
                    .checked_sub(match_quantity)
                    .ok_or(MarketError::MathError)?;
            }

            ctx.accounts.event_queue.events.push(ev);
            ctx.accounts.event_queue.count = ctx
                .accounts
                .event_queue
                .count
                .checked_add(1)
                .ok_or(MarketError::MathError)?;

            node.quantity = node
                .quantity
                .checked_sub(match_quantity)
                .ok_or(MarketError::MathError)?;

            if node.quantity == 0 {
                let prev = node.prev;
                let next = node.next;

                if prev != EMPTY_INDEX {
                    opposite_slab.nodes[prev as usize].next = next;
                } else {
                    opposite_slab.head_index = next;
                }
                if next != EMPTY_INDEX {
                    opposite_slab.nodes[next as usize].prev = prev;
                }

                free_a_node(opposite_slab, index)?;
            }

            left_qty = left_qty
                .checked_sub(match_quantity)
                .ok_or(MarketError::MathError)?;
            continue;
        } else {
            break;
        }
    }

    if left_qty > 0 {
        let target_slab = match params.orderside {
            Orderside::Buy => &mut ctx.accounts.bids,
            Orderside::Sell => &mut ctx.accounts.asks,
        };

        let mut reserved_usdc = 0;
        let mut reserved_outcome = 0;

        if params.orderside == Orderside::Buy {
            let price = params.price as u128;
            reserved_usdc = (price)
                .checked_mul(left_qty as u128)
                .ok_or(MarketError::MathError)?
                .checked_div(PRICE_PRICISION_SCALE)
                .ok_or(MarketError::MathError)? as u64;

            let cpi_context = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.trader_usdc.to_account_info(),
                    to: ctx.accounts.vault_usdc.to_account_info(),
                    authority: ctx.accounts.trader.to_account_info(),
                },
            );

            transfer(cpi_context, reserved_usdc)?;
        } else {
            let trader_outcome = ctx
                .accounts
                .trader_outcome
                .as_ref()
                .ok_or(MarketError::InsufficientBalnace)?;

            let traget_vault = match params.outcome {
                OutcomeSide::Yes => ctx.accounts.vault_yes.to_account_info(),
                OutcomeSide::No => ctx.accounts.vault_no.to_account_info(),
            };

            reserved_outcome = left_qty;

            let cpi_context = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: trader_outcome.to_account_info(),
                    to: traget_vault,
                    authority: ctx.accounts.trader.to_account_info(),
                },
            );

            transfer(cpi_context, reserved_outcome)?;
        }
        let index = allocate_node(target_slab)?;
        let node = &mut target_slab.nodes[index as usize];
        node.key = price_key;
        node.owner = ctx.accounts.trader_postion.key();
        node.quantity = left_qty;
        node.reserved_usdc = reserved_usdc;
        node.reserved_outcome = reserved_outcome;
        node.order_id = Clock::get()?.slot as u64;
        node.outcome = params.outcome;
        node.time_stamp = now;
        node.occupied = true;
        node.next = EMPTY_INDEX;
        node.prev = EMPTY_INDEX;

        let order_id = node.order_id;

        slab_insert_node(target_slab, index)?;

        let trader_position = &mut ctx.accounts.trader_postion;
        let mut free_slot: Option<usize> = None;
        for i in 1..MAX_ORDER_PER_TRADER {
            let bit = (trader_position.slots_bitmap << i) & 1u128;
            if bit == 0 {
                free_slot = Some(i);
                break;
            }
        }

        if free_slot.is_none() {
            return Err(MarketError::MaxOrderReached.into());
        }
        let i = free_slot.unwrap();

        trader_position.slots_bitmap |= 1u128 << i;
        trader_position
            .active_orders
            .checked_add(1)
            .ok_or(MarketError::MathError)?;
        trader_position.order_ids[i] = order_id;
    }
    Ok(())
}
