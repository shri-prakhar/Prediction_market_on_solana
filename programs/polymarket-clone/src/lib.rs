use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod utils;
pub mod state;
pub mod instructions;
declare_id!("2j64V9Te3wcmWZnkZDDSd3iA5YYfwErPAGeD9ip7i5BD");

#[program]
pub mod polymarket_clone {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
