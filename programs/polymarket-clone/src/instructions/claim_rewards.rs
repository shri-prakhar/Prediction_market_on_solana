use anchor_lang::prelude::*;
use anchor_spl::token::{burn, transfer, Burn, Mint, Token, TokenAccount, Transfer};

use crate::{
    constants::MARKET_SEED,
    error::MarketError,
    state::{Market, MarketStatus},
};

#[derive(Accounts)]

pub struct ClaimReward<'info> {
    #[account(mut)]
    pub claimer: Signer<'info>,

    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(mut)]
    pub claimer_outcome_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub claimer_usdc_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_usdc: Account<'info, TokenAccount>,

    pub outcome_mint: Account<'info, Mint>,

    token_program: Program<'info, Token>,
}

pub fn claim_reward_handler(ctx: Context<ClaimReward>, amount: u64) -> Result<()> {
    match ctx.accounts.market.status {
        MarketStatus::ResolvedYes | MarketStatus::ResolvedNo => {}
        _ => return err!(MarketError::MarketNotOpen),
    }

    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Burn {
            mint: ctx.accounts.outcome_mint.to_account_info(),
            from: ctx.accounts.claimer_outcome_account.to_account_info(),
            authority: ctx.accounts.claimer.to_account_info(),
        },
    );

    burn(cpi_ctx, amount)?;

    let bump = ctx.accounts.market.bump;
    let signer_seeds: &[&[&[u8]]] = &[&[
        MARKET_SEED,
        &ctx.accounts.market.market_id.to_le_bytes(),
        &[bump],
    ]];

    let cpi_ctx_2 = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.vault_usdc.to_account_info(),
            to: ctx.accounts.claimer_usdc_account.to_account_info(),
            authority: ctx.accounts.market.to_account_info(),
        },
        signer_seeds,
    );

    transfer(cpi_ctx_2, amount)?;

    Ok(())
}
