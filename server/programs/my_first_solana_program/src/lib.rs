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

    pub fn initialize_governance_config(
        ctx: Context<InitializeGovernanceConfig>,
        min_execution_delay_seconds: i64,
        emergency_guardian: Pubkey,
    ) -> Result<()> {
        instructions::security_v1::initialize_governance_config_handler(
            ctx,
            min_execution_delay_seconds,
            emergency_guardian,
        )
    }

    pub fn create_proposal_decision(
        ctx: Context<CreateProposalDecision>,
        expected_proposal_id: u64,
        proposal_type: ProposalType,
        decision: ProposalDecision,
        yes_weight: u64,
        no_weight: u64,
        start_ts: i64,
        end_ts: i64,
    ) -> Result<()> {
        instructions::security_v1::create_proposal_decision_handler(
            ctx,
            expected_proposal_id,
            proposal_type,
            decision,
            yes_weight,
            no_weight,
            start_ts,
            end_ts,
        )
    }

    pub fn queue_execution(
        ctx: Context<QueueExecution>,
        proposal_id: u64,
        action_type: ActionType,
        target_program: Pubkey,
        target_account: Pubkey,
        payload_hash: [u8; 32],
    ) -> Result<()> {
        instructions::security_v1::queue_execution_handler(
            ctx,
            proposal_id,
            action_type,
            target_program,
            target_account,
            payload_hash,
        )
    }

    pub fn execute_queued_action(
        ctx: Context<ExecuteQueuedAction>,
        proposal_id: u64,
        payload_hash: [u8; 32],
    ) -> Result<()> {
        instructions::security_v1::execute_queued_action_handler(ctx, proposal_id, payload_hash)
    }

    pub fn cancel_queued_action(ctx: Context<CancelQueuedAction>, proposal_id: u64) -> Result<()> {
        instructions::security_v1::cancel_queued_action_handler(ctx, proposal_id)
    }

    pub fn pause_security_layer(ctx: Context<PauseSecurityLayer>) -> Result<()> {
        instructions::security_v1::pause_security_layer_handler(ctx)
    }

    pub fn unpause_security_layer(ctx: Context<UnpauseSecurityLayer>) -> Result<()> {
        instructions::security_v1::unpause_security_layer_handler(ctx)
    }
}
