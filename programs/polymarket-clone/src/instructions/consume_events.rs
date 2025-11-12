use anchor_lang::prelude::*;
use anchor_spl::{token::{Transfer, transfer}, token_interface::{Mint, TokenAccount, TokenInterface}};

use crate::{constants::{CRANKER_REWARD_BPS, EVRNT_QUEUE_SEED, MARKET_SEED, PRICE_PRICISION_SCALE}, error::MarketError, state::{EventQueue, EventType, Market, OutcomeSide, vault}};

#[derive(Debug , AnchorSerialize ,AnchorDeserialize, Clone)]
pub struct ConsumeEventParams{
    pub market_id : u64
}

#[derive(Accounts)]
#[instruction(params: ConsumeEventParams)]
pub struct ConsumeEvents<'info> {
    #[account(mut , seeds = [EVRNT_QUEUE_SEED , &params.market_id.to_be_bytes()] , bump)]
    pub event_queue: Account<'info, EventQueue>,

    
    pub usdc_mint: InterfaceAccount<'info ,Mint>,

    
    pub yes_mint: InterfaceAccount<'info ,Mint>,

    
    pub no_mint: InterfaceAccount<'info ,Mint>,

    #[account(mut)]
    pub vault_usdc: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub vault_yes : InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub vault_no : InterfaceAccount<'info , TokenAccount>,

    #[account(mut)]
    pub cranker_usdc : InterfaceAccount<'info , TokenAccount>,
    
    #[account(mut , seeds = [MARKET_SEED , &params.market_id.to_be_bytes()] , bump)]
    pub market: Account<'info, Market>,

    pub cranker : Signer<'info>,

    pub system_program : Program<'info, System>,
    pub token_program : Interface<'info , TokenInterface>,

}

pub fn consume_events_handler<'info>(ctx: Context<'_, '_, 'info, 'info, ConsumeEvents<'info>> , params : ConsumeEventParams ) -> Result<()> {
    let queue: &mut Account<'info, EventQueue> = &mut ctx.accounts.event_queue;
    let market = &mut ctx.accounts.market;
    let rem =  &ctx.remaining_accounts;

    let n_events = queue.events.len();

    if n_events == 0 { return  Ok(()); }

    let expected = n_events.checked_mul(6).ok_or(MarketError::MathError)?;

    require!(rem.len() == expected , MarketError::NoMatchingOrder); 

    let market_id =  market.market_id.to_be_bytes();
    let bump = market.bump;
    let seeds: &[&[u8]]= &[MARKET_SEED , &market_id , &[bump]];

    let mut rem_index = 0usize ;
    for event in  queue.events.iter() {
        
        let maker_usdc = &rem[rem_index]; rem_index += 1;
        let maker_outcome = &rem[rem_index]; rem_index += 1;
        let taker_usdc = &rem[rem_index]; rem_index += 1;
        let taker_outcome = &rem[rem_index]; rem_index += 1;
        let maker_trader_position = &rem[rem_index]; rem_index += 1;
        let taker_trader_position = &rem[rem_index]; rem_index += 1;  

        let maker_usdc_ac : InterfaceAccount<'info,TokenAccount> = InterfaceAccount::try_from(maker_usdc)?;
        let maker_outcome_ac : InterfaceAccount<'info,TokenAccount> = InterfaceAccount::try_from(maker_outcome)?;
        let taker_usdc_ac : InterfaceAccount<'info,TokenAccount> = InterfaceAccount::try_from(taker_usdc)?;
        let taker_outcome_ac : InterfaceAccount<'info,TokenAccount> = InterfaceAccount::try_from(taker_outcome)?;
 

        match event.event_type {
            EventType::Fill => {
                let seeds: &[&[&[u8]]] = &[seeds];
                let usdc_amount = (event.price as u128).checked_mul(event.quantity as u128).ok_or(MarketError::MathError)?
                                        .checked_div(PRICE_PRICISION_SCALE).ok_or(MarketError::MathError)? as u64;
                
                let fee = (usdc_amount * (market.fee_bps as u64)) / 10_000;
                let cranker_reward = (fee) * (CRANKER_REWARD_BPS as u64)/10_000; // 1bps = 0.01%
                let amount_after_fee = usdc_amount.checked_sub(fee).ok_or(MarketError::MathError)?;

                let outcome_vault_info = match event.outcome {
                    OutcomeSide::Yes => ctx.accounts.vault_yes.to_account_info(),
                    OutcomeSide::No => ctx.accounts.vault_no.to_account_info()
                };

                if event.side == 0 {
                    let cpi_context1 = CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info()
                    , Transfer{
                        from : outcome_vault_info,
                        to : taker_outcome_ac.to_account_info(),
                        authority: market.to_account_info(),
                    },
                seeds
            );

                    transfer(cpi_context1, event.quantity)?;

                    let cpi_context2 = CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(),
                    
                     Transfer{
                        from:ctx.accounts.vault_usdc.to_account_info(),
                        to: maker_usdc_ac.to_account_info(),
                        authority: market.to_account_info(),
                     },
                    seeds
                );
                    
                    transfer(cpi_context2, amount_after_fee)?;


                } else {
                    let cpi_context1 = CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info()
                    , Transfer{
                        from : outcome_vault_info,
                        to : maker_outcome_ac.to_account_info(),
                        authority: market.to_account_info(),
                    },
                seeds
                );

                    transfer(cpi_context1, event.quantity)?;
                let cpi_context2 = CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(),
                
                    Transfer{
                    from:ctx.accounts.vault_usdc.to_account_info(),
                    to: maker_usdc_ac.to_account_info(),
                    authority: market.to_account_info(),
                    },
                seeds
                );
                
                transfer(cpi_context2, amount_after_fee)?;
                }

                if cranker_reward > 0 {
                let cpi_context2 = CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(),
                
                    Transfer{
                    from:ctx.accounts.vault_usdc.to_account_info(),
                    to: ctx.accounts.cranker_usdc.to_account_info(),
                    authority: market.to_account_info(),
                    },
                seeds
                );
                
                transfer(cpi_context2, cranker_reward)?;                    
                }
            },
            EventType::Cancel => {
                let seeds: &[&[&[u8]]] = &[seeds];
                let reserved_usdc = event.price;
                let reserved_outcome = event.quantity;

                if reserved_usdc > 0 {
                    let cpi_refund = CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer{
                            from : ctx.accounts.vault_usdc.to_account_info(),
                            to: maker_usdc_ac.to_account_info(),
                            authority: market.to_account_info()
                        }, seeds);

                        transfer(cpi_refund, reserved_usdc)?;
                };
                if reserved_outcome > 0 {
                    let outcome_vault_info = match event.outcome {
                        OutcomeSide::Yes => ctx.accounts.vault_yes.to_account_info(),
                        OutcomeSide::No => ctx.accounts.vault_no.to_account_info()
                    };
                    let cpi_refund = CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer{
                            from: outcome_vault_info.to_account_info(),
                            to: maker_outcome_ac.to_account_info(),
                            authority: market.to_account_info()
                        }, seeds);

                        transfer(cpi_refund, reserved_outcome)?;
                }
            }
        }
    }

    queue.events.clear();
    queue.count = 0;
    Ok(())
}
