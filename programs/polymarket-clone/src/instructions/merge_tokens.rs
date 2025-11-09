use anchor_lang::prelude::*;
use anchor_spl::{token::{Burn, Transfer, burn, transfer}, token_interface::{Mint, TokenAccount, TokenInterface}};

use crate::state::{Market, Vault};



#[derive(Accounts)]

pub struct MergeTokens<'info>{
    #[account(mut)]
    pub trader: Signer<'info>,

    #[account(mut)]
    pub market : Account<'info , Market>,

    #[account(mut)]
    pub vault : Account<'info, Vault>,
    
    #[account(mut)]
    trader_usdc : InterfaceAccount<'info , TokenAccount>,

    #[account(mut)]
    trader_yes: InterfaceAccount<'info , TokenAccount>,

    #[account(mut)]
    pub trader_no : InterfaceAccount<'info , TokenAccount>,

    pub yes_mint : InterfaceAccount<'info , Mint>,
    pub no_mint : InterfaceAccount<'info ,Mint>,

    pub token_program: Interface<'info , TokenInterface>,

    
}

pub fn merge_tokens_handler(ctx: Context<MergeTokens> , amount : u64) -> Result<()>{
    
    let burn_yes = Burn{
        from: ctx.accounts.trader_yes.to_account_info(),
        mint: ctx.accounts.yes_mint.to_account_info(),
        authority:ctx.accounts.trader.to_account_info()
    };

    burn(CpiContext::new(ctx.accounts.token_program.to_account_info(), burn_yes), amount)?;

    let burn_no = Burn{
        from:ctx.accounts.trader_no.to_account_info(),
        mint: ctx.accounts.no_mint.to_account_info(),
        authority: ctx.accounts.trader.to_account_info()
    };

    burn(CpiContext::new(ctx.accounts.token_program.to_account_info(), burn_no), amount)?;

    let transfer_usdc = Transfer{
        from: ctx.accounts.vault.to_account_info(),
        to:ctx.accounts.trader.to_account_info(),
        authority: ctx.accounts.vault.to_account_info(),
    };

    transfer(CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_usdc), amount)?;

    
    Ok(())
}