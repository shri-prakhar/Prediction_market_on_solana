use anchor_lang::prelude::*;
use anchor_spl::{token::{MintTo, Transfer, mint_to, transfer}, token_interface::{Mint, TokenAccount, TokenInterface}};

use crate::state::{Market, Vault};

#[derive(Accounts)]

pub struct SplitToken<'info>{
    #[account(signer)]
    pub trader: Signer<'info>,

    #[account(mut)]
    pub market: Account<'info , Market>,

    #[account(mut)]
    pub trader_usdc : InterfaceAccount<'info , TokenAccount>,

    #[account(mut)]
    pub trader_yes : InterfaceAccount<'info  , TokenAccount>,

    #[account(mut)]
    pub trader_no : InterfaceAccount<'info , TokenAccount>,

    #[account(mut)]
    pub vault: Account<'info , Vault>,

    pub yes_mint: InterfaceAccount<'info , Mint>,
    pub no_mint: InterfaceAccount<'info , Mint>,

    pub token_program : Interface<'info , TokenInterface>
}

pub fn split_tokens_handler( ctx:Context<SplitToken> , amount: u64) -> Result<()>{
    let transfer_ix = Transfer{
        from: ctx.accounts.trader_usdc.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
        authority: ctx.accounts.trader.to_account_info()
    } ;

    transfer(CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_ix), amount)?;

    let mint_yes = MintTo{
        mint:ctx.accounts.yes_mint.to_account_info(),
        to: ctx.accounts.trader_yes.to_account_info(),
        authority:ctx.accounts.market.to_account_info(),
    };

    mint_to(CpiContext::new(ctx.accounts.token_program.to_account_info(), mint_yes), amount)?;

    let mint_no = MintTo{
        mint:ctx.accounts.no_mint.to_account_info(),
        to: ctx.accounts.trader_no.to_account_info(),
        authority: ctx.accounts.market.to_account_info()
    };

    mint_to(CpiContext::new(ctx.accounts.token_program.to_account_info(), mint_no), amount)?;
    
    Ok(())
}