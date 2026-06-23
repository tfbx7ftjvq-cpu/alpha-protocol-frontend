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

    pub fn initialize_staking_pool(
        ctx: Context<InitializeStakingPool>,
        min_claim_usdc: u64,
    ) -> Result<()> {
        instructions::staking_v1::initialize_staking_pool_handler(ctx, min_claim_usdc)
    }

    pub fn stake_alpha(ctx: Context<StakeAlpha>, amount: u64, lock_tier: u8) -> Result<()> {
        instructions::staking_v1::stake_alpha_handler(ctx, amount, lock_tier)
    }

    pub fn claim_usdc_rewards(ctx: Context<ClaimUsdcRewards>) -> Result<()> {
        instructions::staking_v1::claim_usdc_rewards_handler(ctx)
    }

    pub fn unstake_alpha(ctx: Context<UnstakeAlpha>, amount: u64) -> Result<()> {
        instructions::staking_v1::unstake_alpha_handler(ctx, amount)
    }
}
