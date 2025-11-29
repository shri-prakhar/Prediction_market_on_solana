use anchor_lang::prelude::*;
use anchor_spl::token::{mint_to, transfer, Mint, MintTo, Token, TokenAccount, Transfer};

use crate::{constants::VAULT_YES_SEED, state::Market};

#[derive(Accounts)]
#[instruction(params: SplitOrderParams)]
pub struct SplitToken<'info> {
    #[account(signer)]
    pub trader: Signer<'info>,

    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(mut)]
    pub trader_usdc: Account<'info, TokenAccount>,

    #[account(mut)]
    pub trader_yes: Account<'info, TokenAccount>,

    #[account(mut)]
    pub trader_no: Account<'info, TokenAccount>,

    #[account(mut ,  seeds = [VAULT_YES_SEED , &params.market_id.to_le_bytes()] , bump)]
    pub vault_usdc: Account<'info, TokenAccount>,

    pub yes_mint: Account<'info, Mint>,
    pub no_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]

pub struct SplitOrderParams {
    amount: u64,
    market_id: u64,
}

pub fn split_tokens_handler(ctx: Context<SplitToken>, params: SplitOrderParams) -> Result<()> {
    let transfer_ix = Transfer {
        from: ctx.accounts.trader_usdc.to_account_info(),
        to: ctx.accounts.vault_usdc.to_account_info(),
        authority: ctx.accounts.trader.to_account_info(),
    };

    transfer(
        CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_ix),
        params.amount,
    )?;

    let mint_yes = MintTo {
        mint: ctx.accounts.yes_mint.to_account_info(),
        to: ctx.accounts.trader_yes.to_account_info(),
        authority: ctx.accounts.market.to_account_info(),
    };

    mint_to(
        CpiContext::new(ctx.accounts.token_program.to_account_info(), mint_yes),
        params.amount,
    )?;

    let mint_no = MintTo {
        mint: ctx.accounts.no_mint.to_account_info(),
        to: ctx.accounts.trader_no.to_account_info(),
        authority: ctx.accounts.market.to_account_info(),
    };

    mint_to(
        CpiContext::new(ctx.accounts.token_program.to_account_info(), mint_no),
        params.amount,
    )?;

    Ok(())
}
