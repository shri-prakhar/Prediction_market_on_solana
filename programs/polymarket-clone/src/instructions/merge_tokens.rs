use anchor_lang::prelude::*;
use anchor_spl::{
    token::{burn, transfer, Burn, Transfer},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{
    constants::{MARKET_SEED, VAULT_USDC_SEED},
    state::{Market, Vault},
};

#[derive(Accounts)]
#[instruction(params: MergeTokensParams)]
pub struct MergeTokens<'info> {
    #[account(mut)]
    pub trader: Signer<'info>,

    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(mut , seeds = [VAULT_USDC_SEED , &params.market_id.to_le_bytes()] , bump)]
    pub vault_usdc: Account<'info, Vault>,

    #[account(mut)]
    pub trader_usdc: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub trader_yes: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub trader_no: InterfaceAccount<'info, TokenAccount>,

    pub yes_mint: InterfaceAccount<'info, Mint>,
    pub no_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
}

#[repr(C)]
#[derive(AnchorDeserialize, AnchorSerialize, Debug, Clone)]

pub struct MergeTokensParams {
    amount: u64,
    market_id: u64,
}

pub fn merge_tokens_handler(ctx: Context<MergeTokens>, params: MergeTokensParams) -> Result<()> {
    let market = &mut ctx.accounts.market;
    let bump = market.bump;
    let seed: &[&[&[u8]]] = &[&[MARKET_SEED, &market.market_id.to_le_bytes(), &[bump]]];

    let burn_yes = Burn {
        from: ctx.accounts.trader_yes.to_account_info(),
        mint: ctx.accounts.yes_mint.to_account_info(),
        authority: ctx.accounts.trader.to_account_info(),
    };

    burn(
        CpiContext::new(ctx.accounts.token_program.to_account_info(), burn_yes),
        params.amount,
    )?;

    let burn_no = Burn {
        from: ctx.accounts.trader_no.to_account_info(),
        mint: ctx.accounts.no_mint.to_account_info(),
        authority: ctx.accounts.trader.to_account_info(),
    };

    burn(
        CpiContext::new(ctx.accounts.token_program.to_account_info(), burn_no),
        params.amount,
    )?;

    let transfer_usdc = Transfer {
        from: ctx.accounts.vault_usdc.to_account_info(),
        to: ctx.accounts.trader.to_account_info(),
        authority: ctx.accounts.market.to_account_info(),
    };

    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_usdc,
            seed,
        ),
        params.amount,
    )?;

    Ok(())
}
