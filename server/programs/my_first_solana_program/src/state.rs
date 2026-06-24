use anchor_lang::prelude::*;

#[account]
pub struct TreasuryState {
    pub authority: Pubkey,
    pub total_inflow: u64,
    pub relief_pool: u64,
    pub buyback_pool: u64,
    pub payroll_pool: u64,
    pub staking_pool: u64,
    pub bump: u8,
}

impl TreasuryState {
    pub const INIT_SPACE: usize = 32 + (8 * 5) + 1;
}

#[account]
pub struct TreasuryConfigV2 {
    pub authority: Pubkey,
    pub usdc_mint: Pubkey,
    pub alpha_mint: Pubkey,
    pub bump: u8,
}

impl TreasuryConfigV2 {
    pub const INIT_SPACE: usize = (32 * 3) + 1;
}

#[account]
pub struct TreasuryUsdcStateV2 {
    pub total_usdc_inflow: u64,
    pub relief_usdc_total: u64,
    pub buyback_usdc_total: u64,
    pub builders_usdc_total: u64,
    pub staking_usdc_total: u64,
    pub bump: u8,
}

impl TreasuryUsdcStateV2 {
    pub const INIT_SPACE: usize = (8 * 5) + 1;
}

#[account]
pub struct StakingPoolV1 {
    pub authority: Pubkey,
    pub alpha_mint: Pubkey,
    pub usdc_mint: Pubkey,
    pub alpha_vault: Pubkey,
    pub alpha_vault_authority: Pubkey,
    pub staking_usdc_vault: Pubkey,
    pub vault_authority_v2: Pubkey,
    pub total_staked_alpha: u64,
    pub total_effective_weight: u128,
    pub acc_usdc_per_weight: u128,
    pub last_reward_update_ts: i64,
    pub last_observed_usdc_balance: u64,
    pub reward_release_bps: u16,
    pub min_claim_usdc: u64,
    pub vault_authority_v2_bump: u8,
    pub alpha_vault_authority_bump: u8,
    pub bump: u8,
}

impl StakingPoolV1 {
    pub const INIT_SPACE: usize = 384;
}

#[account]
pub struct UserStakeAccount {
    pub owner: Pubkey,
    pub staked_amount: u64,
    pub effective_weight: u128,
    pub lock_start_ts: i64,
    pub lock_end_ts: i64,
    pub lock_tier: u8,
    pub multiplier_bps: u16,
    pub reward_debt: u128,
    pub pending_usdc: u64,
    pub next_reward_eligible_ts: i64,
    pub bump: u8,
}

impl UserStakeAccount {
    pub const INIT_SPACE: usize = 256;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProposalType {
    GreenLabelSlash,
    GreenLabelRefund,
    PayrollEmployeeImpeach,
    PayrollPayout,
    TreasuryParamChange,
    EmergencyPause,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProposalDecision {
    Pending,
    Approved,
    Rejected,
    Partial,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionStatus {
    Queued,
    Executed,
    Cancelled,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionType {
    Noop,
    GreenLabelSlash,
    GreenLabelRefund,
    PayrollEmployeeImpeach,
    PayrollPayout,
    TreasuryParamChange,
    EmergencyPause,
}

#[account]
pub struct GovernanceConfigV1 {
    pub authority: Pubkey,
    pub min_execution_delay_seconds: i64,
    pub proposal_count: u64,
    pub emergency_guardian: Pubkey,
    pub is_paused: bool,
    pub bump: u8,
}

impl GovernanceConfigV1 {
    pub const INIT_SPACE: usize = 128;
}

#[account]
pub struct ProposalDecisionV1 {
    pub proposal_id: u64,
    pub proposal_type: ProposalType,
    pub proposer: Pubkey,
    pub decision: ProposalDecision,
    pub yes_weight: u64,
    pub no_weight: u64,
    pub start_ts: i64,
    pub end_ts: i64,
    pub finalized_ts: i64,
    pub bump: u8,
}

impl ProposalDecisionV1 {
    pub const INIT_SPACE: usize = 128;
}

#[account]
pub struct ExecutionQueueItemV1 {
    pub proposal_id: u64,
    pub proposer: Pubkey,
    pub action_type: ActionType,
    pub target_program: Pubkey,
    pub target_account: Pubkey,
    pub decision: ProposalDecision,
    pub created_at: i64,
    pub execute_after: i64,
    pub executed_at: i64,
    pub status: ExecutionStatus,
    pub payload_hash: [u8; 32],
    pub bump: u8,
}

impl ExecutionQueueItemV1 {
    pub const INIT_SPACE: usize = 256;
}
