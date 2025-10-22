use anchor_lang::prelude::*;

declare_id!("minpgGyXRDcG6KAMujbA9CF8GhTqfNKREQqUqsW83oy");

#[program]
pub mod relay_cost_protection {
    use super::*;

    #[instruction(discriminator = 1)]
    pub fn check_balance(ctx: Context<CheckBalance>, min_balance: u64) -> Result<()> {
        require!(
            ctx.accounts.account.lamports() >= min_balance,
            BalanceError::InsufficientBalance
        );
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CheckBalance<'info> {
    /// CHECK: We're only reading the balance, no other validation needed
    pub account: AccountInfo<'info>,
}

#[error_code]
pub enum BalanceError {
    #[msg("Account balance is less than required minimum")]
    InsufficientBalance,
}
