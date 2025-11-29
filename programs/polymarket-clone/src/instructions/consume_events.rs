use anchor_lang::prelude::*;
use anchor_spl::token::{mint_to, transfer, Mint, MintTo, Token, TokenAccount, Transfer};

use crate::{
    constants::{
        EVENT_QUEUE_SEED, FEE_VAULT_USDC, MARKET_SEED, MAX_EVENTS, PRICE_PRECISION_SCALE,
        VAULT_NO_SEED, VAULT_USDC_SEED, VAULT_YES_SEED,
    },
    error::MarketError,
    instructions::{amm_execute_buy, FP_SCALE},
    state::{EventQueue, EventType, Market, OpenOrder, OrderSide, OutcomeSide},
};

#[derive(Accounts)]

pub struct ConsumeEvents<'info> {
    #[account(mut)]
    pub cranker: Signer<'info>,

    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(mut, seeds = [EVENT_QUEUE_SEED , &market.market_id.to_le_bytes()] , bump)]
    pub event_queue: Account<'info, EventQueue>,

    #[account(mut, seeds = [VAULT_USDC_SEED , &market.market_id.to_le_bytes()] , bump)]
    pub vault_usdc: Account<'info, TokenAccount>,

    #[account(mut , seeds = [VAULT_YES_SEED , &market.market_id.to_le_bytes()] , bump)]
    pub vault_yes: Account<'info, TokenAccount>,

    #[account(mut , seeds = [VAULT_NO_SEED , &market.market_id.to_le_bytes()] , bump)]
    pub vault_no: Account<'info, TokenAccount>,

    #[account(mut , seeds = [FEE_VAULT_USDC , &market.market_id.to_le_bytes()] , bump)]
    pub fee_vault_usdc: Account<'info, TokenAccount>,

    pub yes_mint: Account<'info, Mint>,
    pub no_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

pub fn consume_events_handler<'info>(
    ctx: Context<'_, '_, 'info, 'info, ConsumeEvents<'info>>,
    max_events: u16,
) -> Result<()> {
    let event_queue = &mut ctx.accounts.event_queue;
    let market = &mut ctx.accounts.market;

    let n_events = core::cmp::min(event_queue.count as usize, max_events as usize);

    if n_events == 0 {
        return Ok(());
    }

    let remaining_accounts = &ctx.remaining_accounts;
    let expected_length = 6 * n_events + 1;
    require!(
        remaining_accounts.len() >= expected_length,
        MarketError::NoMatchingOrder
    );
    let bump = market.bump;
    let seeds: &[&[&[u8]]] = &[&[MARKET_SEED, &market.market_id.to_le_bytes(), &[bump]]];

    let mut remaining_index = 0usize;

    let cranker_usdc_account: &AccountInfo<'info> = &remaining_accounts[expected_length - 1];

    for i in 0..n_events {
        let event_index = ((event_queue.head as usize + i) as usize) % MAX_EVENTS;
        let event = event_queue.events[event_index];

        let maker_oo_info = &remaining_accounts[remaining_index];
        remaining_index += 1;
        let maker_outcome_info = &remaining_accounts[remaining_index];
        remaining_index += 1;
        let maker_usdc_info = &remaining_accounts[remaining_index];
        remaining_index += 1;
        let taker_oo_info = &remaining_accounts[remaining_index];
        remaining_index += 1;
        let _taker_usdc_info = &remaining_accounts[remaining_index];
        remaining_index += 1;
        let taker_outcome_info = &remaining_accounts[remaining_index];
        remaining_index += 1;

        let mut maker_oo: Account<OpenOrder> = Account::try_from(&maker_oo_info)?;
        let mut taker_oo: Account<OpenOrder> = Account::try_from(&taker_oo_info)?;

        require!(
            maker_oo.key() == event.makers_open_orders || maker_oo.key() == market.key(),
            MarketError::NoMatchingOrder
        );
        require!(
            taker_oo.key() == event.taker_open_orders,
            MarketError::NoMatchingOrder
        );

        let maker_usdc: Account<TokenAccount> = Account::try_from(&maker_usdc_info)?;
        let maker_outcome: Account<TokenAccount> = Account::try_from(maker_outcome_info)?;
        let taker_outcome: Account<TokenAccount> = Account::try_from(&taker_outcome_info)?;
        //let taker_usdc: Account<TokenAccount> = Account::try_from(taker_usdc_info)?;

        let usdc_amount = (event.price)
            .checked_mul(event.quantity as u128)
            .ok_or(MarketError::MathError)?
            .checked_div(PRICE_PRECISION_SCALE)
            .ok_or(MarketError::MathError)? as u64;

        let fee = (usdc_amount as u128)
            .checked_mul(market.fee_bps as u128)
            .ok_or(MarketError::MathError)?
            .checked_div(10_000u128)
            .ok_or(MarketError::MathError)?;

        let cranker_reward = (fee as u128)
            .checked_mul(market.cranker_reward_bps as u128)
            .ok_or(MarketError::MathError)?
            .checked_div(10_000u128)
            .ok_or(MarketError::MathError)?;

        let remaining_fee = fee
            .checked_sub(cranker_reward)
            .ok_or(MarketError::MathError)?;

        let taker_is_buyer = event.taker_side == OrderSide::Buy as u8;
        if event.event_type == EventType::Fill as u8 {
            if event.makers_open_orders == market.key() {
                let outcome = if event.outcome == OutcomeSide::Yes as u8 {
                    OutcomeSide::Yes as u8
                } else {
                    OutcomeSide::No as u8
                };

                let mut q_yes = market.q_yes;
                let mut q_no = market.q_no;
                let b_fp = market.b_liquidity;

                let cost_fp = amm_execute_buy(
                    &mut q_yes,
                    &mut q_no,
                    b_fp as u128,
                    outcome,
                    event.quantity as u128,
                )?;
                let cost_u64 = cost_fp
                    .checked_div(FP_SCALE)
                    .ok_or(MarketError::MathError)? as u64;

                let pay_amount = cost_u64
                    .checked_sub(fee as u64)
                    .ok_or(MarketError::MathError)?;

                let cpi_ctx = CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.vault_usdc.to_account_info(),
                        to: maker_usdc.to_account_info(),
                        authority: market.to_account_info(),
                    },
                    seeds,
                );

                transfer(cpi_ctx, pay_amount)?;

                let cranker_usdc_account: Account<TokenAccount> =
                    Account::try_from(&cranker_usdc_account)?;
                if cranker_reward > 0 {
                    let cpi_ctx = CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer {
                            from: ctx.accounts.vault_usdc.to_account_info(),
                            to: cranker_usdc_account.to_account_info(),
                            authority: market.to_account_info(),
                        },
                        seeds,
                    );

                    transfer(cpi_ctx, cranker_reward as u64)?;
                }

                if remaining_fee > 0 {
                    let cpi_ctx = CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer {
                            from: ctx.accounts.vault_usdc.to_account_info(),
                            to: ctx.accounts.fee_vault_usdc.to_account_info(),
                            authority: market.to_account_info(),
                        },
                        seeds,
                    );

                    transfer(cpi_ctx, remaining_fee as u64)?;
                }

                let mint_acc = if outcome == OutcomeSide::Yes as u8 {
                    &ctx.accounts.yes_mint
                } else {
                    &ctx.accounts.no_mint
                };

                let cpi_ctx = CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    MintTo {
                        mint: mint_acc.to_account_info(),
                        to: taker_outcome.to_account_info(),
                        authority: market.to_account_info(),
                    },
                    seeds,
                );

                mint_to(cpi_ctx, event.quantity)?;

                market.q_no = q_no;
                market.q_yes = q_yes;

                if taker_is_buyer {
                    taker_oo.locked_quote = taker_oo
                        .locked_quote
                        .checked_sub(cost_u64 as u128)
                        .ok_or(MarketError::MathError)?;
                } else {
                    taker_oo.locked_base = taker_oo
                        .locked_base
                        .checked_sub(event.quantity as u128)
                        .ok_or(MarketError::MathError)?;
                }
            } else {
                let payout_amount = usdc_amount
                    .checked_sub(fee as u64)
                    .ok_or(MarketError::MathError)?;

                let cpi_ctx = CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.vault_usdc.to_account_info(),
                        to: maker_usdc.to_account_info(),
                        authority: market.to_account_info(),
                    },
                    seeds,
                );

                transfer(cpi_ctx, payout_amount)?;

                let from_vault = if event.outcome == OutcomeSide::Yes as u8 {
                    ctx.accounts.vault_yes.to_account_info()
                } else {
                    ctx.accounts.vault_no.to_account_info()
                };

                let cpi_ctx = CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: from_vault,
                        to: taker_outcome.to_account_info(),
                        authority: market.to_account_info(),
                    },
                    seeds,
                );

                transfer(cpi_ctx, event.quantity)?;

                let cranker_usdc_account: Account<TokenAccount> =
                    Account::try_from(&cranker_usdc_account)?;
                if cranker_reward > 0 {
                    let cpi_ctx = CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer {
                            from: ctx.accounts.vault_usdc.to_account_info(),
                            to: cranker_usdc_account.to_account_info(),
                            authority: market.to_account_info(),
                        },
                        seeds,
                    );

                    transfer(cpi_ctx, cranker_reward as u64)?;
                }

                if remaining_fee > 0 {
                    let cpi_ctx = CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer {
                            from: ctx.accounts.vault_usdc.to_account_info(),
                            to: ctx.accounts.fee_vault_usdc.to_account_info(),
                            authority: market.to_account_info(),
                        },
                        seeds,
                    );

                    transfer(cpi_ctx, remaining_fee as u64)?;
                }

                if taker_is_buyer {
                    taker_oo.locked_quote = taker_oo
                        .locked_quote
                        .checked_sub(usdc_amount as u128)
                        .ok_or(MarketError::MathError)?;
                    maker_oo.locked_base = maker_oo
                        .locked_base
                        .checked_sub(event.quantity as u128)
                        .ok_or(MarketError::MathError)?;
                } else {
                    taker_oo.locked_base = taker_oo
                        .locked_base
                        .checked_sub(event.quantity as u128)
                        .ok_or(MarketError::MathError)?;
                    maker_oo.locked_quote = maker_oo
                        .locked_quote
                        .checked_sub(usdc_amount as u128)
                        .ok_or(MarketError::MathError)?;
                }
            }
        } else if event.event_type == EventType::Cancel as u8 {
            let mut found_slot_index: Option<usize> = None;
            for (slot_index, slot) in maker_oo.slots.iter().enumerate() {
                if slot.active && slot.order_id == event.order_id {
                    found_slot_index = Some(slot_index);
                    break;
                }
            }
            if let Some(slot_index) = found_slot_index {
                let slot = maker_oo.slots[slot_index];

                if slot.side == OrderSide::Buy as u8 {
                    let refund_fp = (slot.price as u128)
                        .checked_mul(slot.quantity_remaining as u128)
                        .ok_or(MarketError::MathError)?;
                    let refund_amount = refund_fp
                        .checked_div(PRICE_PRECISION_SCALE)
                        .ok_or(MarketError::MathError)?
                        as u64;

                    let cpi_ctx = CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer {
                            from: ctx.accounts.vault_usdc.to_account_info(),
                            to: maker_usdc.to_account_info(),
                            authority: market.to_account_info(),
                        },
                        seeds,
                    );

                    transfer(cpi_ctx, refund_amount)?;

                    maker_oo.locked_quote = maker_oo
                        .locked_quote
                        .checked_sub(refund_amount as u128)
                        .ok_or(MarketError::MathError)?;
                    maker_oo.slots[slot_index].active = false;
                    maker_oo.slots[slot_index].quantity_remaining = 0;
                } else {
                    let refund_quantity = slot.quantity_remaining;
                    let from_vault = if slot.outcome == OutcomeSide::Yes as u8 {
                        ctx.accounts.yes_mint.to_account_info()
                    } else {
                        ctx.accounts.no_mint.to_account_info()
                    };

                    let cpi_ctx = CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer {
                            from: from_vault,
                            to: maker_outcome.to_account_info(),
                            authority: market.to_account_info(),
                        },
                        seeds,
                    );

                    transfer(cpi_ctx, refund_quantity)?;

                    maker_oo.locked_base = maker_oo
                        .locked_base
                        .checked_sub(refund_quantity as u128)
                        .ok_or(MarketError::MathError)?;
                    maker_oo.slots[slot_index].active = false;
                    maker_oo.slots[slot_index].quantity_remaining = 0;
                }
            } else {
                return err!(MarketError::OrderNotFound);
            }
        } else {
            return err!(MarketError::InvalidArgument);
        }
    }

    event_queue.head = event_queue
        .head
        .checked_add(n_events as u64)
        .ok_or(MarketError::MathError)?;
    event_queue.count = event_queue
        .count
        .checked_sub(n_events as u64)
        .ok_or(MarketError::MathError)?;

    Ok(())
}
