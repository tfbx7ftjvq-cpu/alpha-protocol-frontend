use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

pub use constants::*;
pub use error::*;
pub use instructions::*;
pub use state::*;

declare_id!("HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY");

#[program]
pub mod my_first_solana_program {
    use super::*;

    pub fn initialize_protocol(ctx: Context<InitializeProtocol>) -> Result<()> {
        instructions::initialize_protocol::initialize_protocol_handler(ctx)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit::deposit_handler(ctx, amount)
    }

    pub fn initialize_usdc_treasury(
        ctx: Context<InitializeUsdcTreasury>,
        usdc_mint: Pubkey,
        alpha_mint: Pubkey,
    ) -> Result<()> {
        instructions::initialize_usdc_treasury::initialize_usdc_treasury_handler(
            ctx, usdc_mint, alpha_mint,
        )
    }

    pub fn deposit_usdc_revenue(ctx: Context<DepositUsdcRevenue>, amount: u64) -> Result<()> {
        instructions::deposit_usdc_revenue::deposit_usdc_revenue_handler(ctx, amount)
    }
}
