use anchor_lang::prelude::*;

declare_id!("7b8CfU8BwgJVK3k5hUu3nCMZMJpxToWD1mVpTGfGVNGQ");

#[program]
pub mod flash_loans {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
