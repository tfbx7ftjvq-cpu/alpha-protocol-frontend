use anchor_lang::prelude::*;

use crate::constants::{
    ANCHOR_ACCOUNT_DISCRIMINATOR_BYTES, CONTRIBUTOR_MILESTONE_DESCRIPTION_MAX_BYTES,
    CONTRIBUTOR_MILESTONE_TITLE_MAX_BYTES, GREEN_LABEL_CONFIG_RESERVED_BYTES,
    GREEN_LABEL_CONFIG_SPACE, GREEN_LABEL_DISPUTE_RESERVED_BYTES, GREEN_LABEL_DISPUTE_SPACE,
    GREEN_LABEL_PROJECT_RESERVED_BYTES, GREEN_LABEL_PROJECT_SPACE,
};

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

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum RevenueType {
    GreenLabelCertificationFee,
    GreenLabelForfeitedBond,
    ProtocolServiceFee,
    PlatformRevenue,
    PartnershipRevenue,
    ManualGovernanceApprovedRevenue,
}

#[account]
pub struct RevenueRoutingStatsV1 {
    pub authority: Pubkey,
    pub usdc_mint: Pubkey,
    pub total_routed_usdc: u64,
    pub green_label_certification_fee_total: u64,
    pub green_label_forfeited_bond_total: u64,
    pub protocol_service_fee_total: u64,
    pub platform_revenue_total: u64,
    pub partnership_revenue_total: u64,
    pub manual_governance_approved_revenue_total: u64,
    pub bump: u8,
}

impl RevenueRoutingStatsV1 {
    pub const INIT_SPACE: usize = (32 * 2) + (8 * 7) + 1;
}

#[account]
pub struct TreasuryGovernanceConfigV1 {
    pub treasury_config: Pubkey,
    pub security_authority: Pubkey,
    pub dao_enabled: bool,
    pub spending_limit_usdc: u64,
    pub split_change_threshold_bps: u64,
    pub emergency_mode: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub bump: u8,
}

impl TreasuryGovernanceConfigV1 {
    pub const INIT_SPACE: usize = (32 * 2) + 1 + (8 * 4) + 1 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreasurySpendingStatusV1 {
    Pending,
    Approved,
    Rejected,
    Executed,
    Cancelled,
}

#[account]
pub struct TreasurySpendingRequestV1 {
    pub request_id: u64,
    pub treasury_config: Pubkey,
    pub proposer: Pubkey,
    pub recipient: Pubkey,
    pub amount_usdc: u64,
    pub purpose_hash: [u8; 32],
    pub proposal_id: u64,
    pub status: TreasurySpendingStatusV1,
    pub created_at: i64,
    pub executed_at: i64,
    pub bump: u8,
}

impl TreasurySpendingRequestV1 {
    pub const INIT_SPACE: usize = 8 + (32 * 4) + (8 * 4) + 1 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreasuryBuilderPayoutStatusV1 {
    Pending,
    Approved,
    Executed,
    Rejected,
}

#[account]
pub struct TreasuryBuilderPayoutGovernanceV1 {
    pub payout_request: Pubkey,
    pub contributor_registry: Pubkey,
    pub milestone: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
    pub proposal_id: u64,
    pub status: TreasuryBuilderPayoutStatusV1,
    pub created_at: i64,
    pub bump: u8,
}

impl TreasuryBuilderPayoutGovernanceV1 {
    pub const INIT_SPACE: usize = (32 * 4) + (8 * 3) + 1 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreasuryExecutionTypeV1 {
    BuilderPayout,
    TreasurySpending,
}

#[account]
pub struct TreasuryExecutionRecordV1 {
    pub queue_item: Pubkey,
    pub proposal_decision: Pubkey,
    pub governance_proposal: Pubkey,
    pub governance_proposal_action: Pubkey,
    pub request_account: Pubkey,
    pub module_id: ProtocolModuleIdV1,
    pub execution_type: TreasuryExecutionTypeV1,
    pub source_vault: Pubkey,
    pub recipient_owner: Pubkey,
    pub recipient_token_account: Pubkey,
    pub amount_usdc: u64,
    pub usdc_mint: Pubkey,
    pub parameters_hash: [u8; 32],
    pub canonical_governance_payload_hash: [u8; 32],
    pub executor: Pubkey,
    pub executed_at: i64,
    pub schema_version: u16,
    pub bump: u8,
}

impl TreasuryExecutionRecordV1 {
    pub const INIT_SPACE: usize = (32 * 10) + 2 + 8 + (32 * 2) + 8 + 2 + 1;
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
    ContributorAddContributor,
    ContributorRemoveContributor,
    ContributorUpdateRole,
    ContributorApproveMilestone,
    ContributorApproveBuilderPayout,
    TreasuryUpdateRevenueSplit,
    TreasuryApproveSpending,
    TreasuryApproveBuilderPayout,
    GreenLabelApproveCertification,
    GreenLabelRejectCertification,
    GreenLabelRevokeCertification,
    VictimReliefApproveCompensation,
    VictimReliefRejectClaim,
    VictimReliefUpdatePolicy,
    ScamRegistryPublishReport,
    ScamRegistryRemoveReport,
    ScamRegistryAppealDecision,
    ProtocolUpdateParameter,
    ProtocolUpgradeProgram,
    ProtocolEmergencyAction,
    VictimReliefUpholdAppeal,
    VictimReliefOverturnAppeal,
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
    ContributorAddContributor,
    ContributorRemoveContributor,
    ContributorUpdateRole,
    ContributorApproveMilestone,
    ContributorApproveBuilderPayout,
    TreasuryUpdateRevenueSplit,
    TreasuryApproveSpending,
    TreasuryApproveBuilderPayout,
    GreenLabelApproveCertification,
    GreenLabelRejectCertification,
    GreenLabelRevokeCertification,
    VictimReliefApproveCompensation,
    VictimReliefRejectClaim,
    VictimReliefUpdatePolicy,
    ScamRegistryPublishReport,
    ScamRegistryRemoveReport,
    ScamRegistryAppealDecision,
    ProtocolUpdateParameter,
    ProtocolUpgradeProgram,
    ProtocolEmergencyAction,
    VictimReliefUpholdAppeal,
    VictimReliefOverturnAppeal,
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

#[account]
pub struct GovernanceLockConfigV1 {
    pub authority: Pubkey,
    pub alpha_mint: Pubkey,
    pub governance_vault: Pubkey,
    pub min_lock_amount: u64,
    pub min_lock_duration_seconds: i64,
    pub max_lock_duration_seconds: i64,
    pub max_time_multiplier_bps: u64,
    pub created_at: i64,
    pub bump: u8,
}

impl GovernanceLockConfigV1 {
    pub const INIT_SPACE: usize = (32 * 3) + (8 * 5) + 1;
}

#[account]
pub struct GovernancePowerStateV1 {
    pub governance_lock_config: Pubkey,
    pub total_locked_alpha: u64,
    pub total_voting_power: u64,
    pub active_position_count: u64,
    pub updated_at: i64,
    pub bump: u8,
}

impl GovernancePowerStateV1 {
    pub const INIT_SPACE: usize = 32 + (8 * 4) + 1;
}

#[account]
pub struct GovernancePositionVoteLockV1 {
    pub governance_position: Pubkey,
    pub voting_lock_until: i64,
    pub last_proposal: Pubkey,
    pub updated_at: i64,
    pub bump: u8,
}

impl GovernancePositionVoteLockV1 {
    pub const INIT_SPACE: usize = (32 * 2) + (8 * 2) + 1;
}

#[account]
pub struct GovernanceVotingConfigV1 {
    pub authority: Pubkey,
    pub quorum_bps: u64,
    pub approval_threshold_bps: u64,
    pub voting_period_seconds: i64,
    pub created_at: i64,
    pub bump: u8,
}

impl GovernanceVotingConfigV1 {
    pub const INIT_SPACE: usize = 32 + (8 * 4) + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GovernanceProposalTypeV1 {
    Contributor,
    Treasury,
    Parameter,
    Upgrade,
    Emergency,
    GreenLabel,
    VictimRelief,
    ScamRegistry,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GovernanceProposalStatusV1 {
    Draft,
    Voting,
    Passed,
    Rejected,
    Queued,
    Executed,
    Cancelled,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoteChoiceV1 {
    Yes,
    No,
    Abstain,
}

/// DAO-layer semantic action language for governance proposals.
///
/// Variant order is part of the serialized governance payload format. Do not
/// reorder existing variants; append new variants only.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GovernanceActionTypeV1 {
    TreasuryUpdateRevenueSplit,
    TreasuryApproveSpending,
    TreasuryApproveBuilderPayout,
    GreenLabelApproveCertification,
    GreenLabelRejectCertification,
    GreenLabelRevokeCertification,
    GreenLabelRefundBond,
    GreenLabelSlashBond,
    VictimReliefApproveCompensation,
    VictimReliefRejectClaim,
    VictimReliefUpdatePolicy,
    ScamRegistryPublishReport,
    ScamRegistryRemoveReport,
    ScamRegistryAppealDecision,
    ContributorAdd,
    ContributorRemove,
    ContributorUpdateRole,
    ContributorApproveMilestone,
    ContributorApprovePayout,
    ProtocolUpdateParameter,
    ProtocolUpgradeProgram,
    ProtocolEmergencyAction,
    VictimReliefUpholdAppeal,
    VictimReliefOverturnAppeal,
}

/// Stable identifier for the protocol module targeted by a governance action.
///
/// Variant order is part of the serialized governance payload format. Do not
/// reorder existing variants; append new variants only.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtocolModuleIdV1 {
    Treasury,
    GreenLabel,
    VictimRelief,
    ScamRegistry,
    Contributor,
    Protocol,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GovernancePayloadV1 {
    pub schema_version: u8,
    pub action_type: GovernanceActionTypeV1,
    pub module_id: ProtocolModuleIdV1,
    pub target_program: Pubkey,
    pub target_account: Pubkey,
    pub parameters_hash: [u8; 32],
    pub evidence_hash: [u8; 32],
    pub created_at: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GovernanceActionRequestV1 {
    pub schema_version: u16,
    pub action_type: GovernanceActionTypeV1,
    pub module_id: ProtocolModuleIdV1,
    pub target_program: Pubkey,
    pub target_account: Pubkey,
    pub parameters_hash: [u8; 32],
    pub evidence_hash: [u8; 32],
}

#[account]
pub struct GovernanceProposalV1 {
    pub proposal_id: u64,
    pub proposer: Pubkey,
    pub proposal_type: GovernanceProposalTypeV1,
    pub action_type: u8,
    pub target_program: Pubkey,
    pub target_account: Pubkey,
    pub payload_hash: [u8; 32],
    pub status: GovernanceProposalStatusV1,
    pub voting_start_ts: i64,
    pub voting_end_ts: i64,
    pub created_at: i64,
    pub snapshot: Pubkey,
    pub yes_weight: u64,
    pub no_weight: u64,
    pub abstain_weight: u64,
    pub finalized_at: i64,
    pub bump: u8,
}

impl GovernanceProposalV1 {
    pub const INIT_SPACE: usize = 8 + 32 + 1 + 1 + 32 + 32 + 32 + 1 + (8 * 3) + 32 + (8 * 4) + 1;
}

#[account]
pub struct GovernanceProposalActionV1 {
    pub governance_proposal: Pubkey,
    pub proposal_id: u64,
    pub proposer: Pubkey,
    pub action_type: GovernanceActionTypeV1,
    pub module_id: ProtocolModuleIdV1,
    pub target_program: Pubkey,
    pub target_account: Pubkey,
    pub parameters_hash: [u8; 32],
    pub evidence_hash: [u8; 32],
    pub canonical_payload_hash: [u8; 32],
    pub schema_version: u16,
    pub created_at: i64,
    pub bump: u8,
}

impl GovernanceProposalActionV1 {
    pub const INIT_SPACE: usize = (32 * 4) + 8 + 1 + 1 + (32 * 3) + 2 + 8 + 1;
}

#[account]
pub struct ProtocolModuleRegistryV1 {
    pub security_governance_config: Pubkey,
    pub module_id: ProtocolModuleIdV1,
    pub module_code: u8,
    pub program_id: Pubkey,
    pub enabled: bool,
    pub schema_version: u16,
    pub created_at: i64,
    pub updated_at: i64,
    pub bump: u8,
}

impl ProtocolModuleRegistryV1 {
    pub const INIT_SPACE: usize = 32 + 1 + 1 + 32 + 1 + 2 + 8 + 8 + 1;
}

#[account]
pub struct GovernancePositionV1 {
    pub owner: Pubkey,
    pub alpha_mint: Pubkey,
    pub vault: Pubkey,
    pub locked_amount: u64,
    pub lock_start_time: i64,
    pub lock_end_time: i64,
    pub holding_multiplier_bps: u64,
    pub voting_power: u64,
    pub status: GovernancePositionStatusV1,
    pub last_updated_at: i64,
    pub bump: u8,
}

impl GovernancePositionV1 {
    pub const INIT_SPACE: usize = (32 * 3) + (8 * 6) + 1 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GovernancePositionStatusV1 {
    Active,
    Unlocking,
    Closed,
}

#[account]
pub struct GovernanceSnapshotV1 {
    pub proposal: Pubkey,
    pub total_voting_power: u64,
    pub yes_weight: u64,
    pub no_weight: u64,
    pub abstain_weight: u64,
    pub created_at: i64,
    pub finalized: bool,
    pub bump: u8,
}

impl GovernanceSnapshotV1 {
    pub const INIT_SPACE: usize = 32 + (8 * 5) + 1 + 1;
}

#[account]
pub struct VoteRecordV1 {
    pub proposal: Pubkey,
    pub voter_position: Pubkey,
    pub choice: VoteChoiceV1,
    pub voting_power_used: u64,
    pub timestamp: i64,
    pub bump: u8,
}

impl VoteRecordV1 {
    pub const INIT_SPACE: usize = (32 * 2) + 1 + (8 * 2) + 1;
}

#[account]
pub struct UniversalGovernanceDecisionAdapterV1 {
    pub governance_proposal: Pubkey,
    pub proposal_decision: Pubkey,
    pub action_type: ActionType,
    pub target_program: Pubkey,
    pub target_account: Pubkey,
    pub payload_hash: [u8; 32],
    pub created_at: i64,
    pub executed: bool,
    pub bump: u8,
}

impl UniversalGovernanceDecisionAdapterV1 {
    pub const INIT_SPACE: usize = (32 * 5) + 1 + 8 + 1 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum VictimReliefCaseStatusV1 {
    EvidencePeriod,
    UnderReview,
    Approved,
    Rejected,
    AppealPending,
    AppealUpheld,
    AppealOverturned,
    PayoutQueued,
    Paid,
    Cancelled,
    Expired,
}

#[account]
pub struct VictimReliefConfigV1 {
    pub authority: Pubkey,
    pub treasury_config: Pubkey,
    pub security_governance_config: Pubkey,
    pub usdc_mint: Pubkey,
    pub current_policy: Pubkey,
    pub current_policy_version: u64,
    pub next_case_id: u64,
    pub paused: bool,
    pub created_at: i64,
    pub schema_version: u16,
    pub bump: u8,
    pub reserved: [u8; 32],
}

impl VictimReliefConfigV1 {
    pub const INIT_SPACE: usize = (32 * 5) + (8 * 2) + 1 + 8 + 2 + 1 + 32;
}

#[account]
pub struct VictimReliefPolicyV1 {
    pub config: Pubkey,
    pub policy_version: u64,
    pub min_claim_amount_usdc: u64,
    pub max_claim_amount_usdc: u64,
    pub max_payout_per_case_usdc: u64,
    pub evidence_window_seconds: i64,
    pub review_window_seconds: i64,
    pub appeal_window_seconds: i64,
    pub submission_cooldown_seconds: i64,
    pub max_evidence_items: u32,
    pub max_active_cases_per_claimant: u16,
    pub active: bool,
    pub initialized_by: Pubkey,
    pub created_at: i64,
    pub schema_version: u16,
    pub bump: u8,
    pub reserved: [u8; 32],
}

impl VictimReliefPolicyV1 {
    pub const INIT_SPACE: usize = 32 + (8 * 8) + 4 + 2 + 1 + 32 + 8 + 2 + 1 + 32;
}

#[account]
pub struct VictimReliefClaimantStateV1 {
    pub config: Pubkey,
    pub claimant: Pubkey,
    pub active_case_count: u16,
    pub total_case_count: u64,
    pub last_case_id: u64,
    pub last_submitted_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub schema_version: u16,
    pub bump: u8,
}

impl VictimReliefClaimantStateV1 {
    pub const INIT_SPACE: usize = (32 * 2) + 2 + (8 * 5) + 2 + 1;
}

#[account]
pub struct VictimReliefCaseV1 {
    pub case_id: u64,
    pub config: Pubkey,
    pub policy: Pubkey,
    pub policy_version: u64,
    pub claimant: Pubkey,
    pub subject_commitment: [u8; 32],
    pub evidence_root: [u8; 32],
    pub evidence_count: u32,
    pub evidence_revision: u32,
    pub claimed_amount_usdc: u64,
    pub approved_amount_usdc: u64,
    pub recipient_owner: Pubkey,
    pub recipient_token_account: Pubkey,
    pub usdc_mint: Pubkey,
    pub status: VictimReliefCaseStatusV1,
    pub active_appeal: Pubkey,
    pub decision_proposal: Pubkey,
    pub decision_queue: Pubkey,
    pub submitted_at: i64,
    pub evidence_deadline: i64,
    pub review_deadline: i64,
    pub appeal_deadline: i64,
    pub updated_at: i64,
    pub schema_version: u16,
    pub bump: u8,
    pub reserved: [u8; 64],
}

impl VictimReliefCaseV1 {
    pub const INIT_SPACE: usize = 8
        + (32 * 2)
        + 8
        + 32
        + (32 * 2)
        + (4 * 2)
        + (8 * 2)
        + (32 * 3)
        + 1
        + (32 * 3)
        + (8 * 5)
        + 2
        + 1
        + 64;
}

#[account]
pub struct VictimReliefEvidenceSnapshotV1 {
    pub victim_relief_case: Pubkey,
    pub config: Pubkey,
    pub policy: Pubkey,
    pub policy_version: u64,
    pub claimant: Pubkey,
    pub subject_commitment: [u8; 32],
    pub evidence_root: [u8; 32],
    pub evidence_count: u32,
    pub evidence_revision: u32,
    pub claimed_amount_usdc: u64,
    pub recipient_owner: Pubkey,
    pub recipient_token_account: Pubkey,
    pub usdc_mint: Pubkey,
    pub frozen_at: i64,
    pub review_deadline: i64,
    pub schema_version: u16,
    pub bump: u8,
    pub reserved: [u8; 32],
}

impl VictimReliefEvidenceSnapshotV1 {
    pub const INIT_SPACE: usize = (32 * 9) + (8 * 4) + (4 * 2) + 2 + 1 + 32;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct VictimReliefDecisionParametersV1 {
    pub schema_version: u16,
    pub config: Pubkey,
    pub policy: Pubkey,
    pub policy_version: u64,
    pub victim_relief_case: Pubkey,
    pub evidence_snapshot: Pubkey,
    pub case_id: u64,
    pub claimant: Pubkey,
    pub subject_commitment: [u8; 32],
    pub evidence_root: [u8; 32],
    pub evidence_count: u32,
    pub evidence_revision: u32,
    pub claimed_amount_usdc: u64,
    pub approved_amount_usdc: u64,
    pub recipient_owner: Pubkey,
    pub recipient_token_account: Pubkey,
    pub usdc_mint: Pubkey,
    pub treasury_config: Pubkey,
    pub relief_usdc_vault: Pubkey,
    pub action_type: GovernanceActionTypeV1,
    pub expected_case_status: VictimReliefCaseStatusV1,
    pub proposal_id: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum VictimReliefDecisionExecutionTypeV1 {
    Approve,
    Reject,
}

#[account]
pub struct VictimReliefDecisionExecutionRecordV1 {
    pub execution_queue_item: Pubkey,
    pub proposal_decision: Pubkey,
    pub governance_proposal: Pubkey,
    pub governance_proposal_action: Pubkey,
    pub module_registry: Pubkey,
    pub config: Pubkey,
    pub policy: Pubkey,
    pub victim_relief_case: Pubkey,
    pub evidence_snapshot: Pubkey,
    pub execution_type: VictimReliefDecisionExecutionTypeV1,
    pub governance_action_type: GovernanceActionTypeV1,
    pub case_status_before: VictimReliefCaseStatusV1,
    pub case_status_after: VictimReliefCaseStatusV1,
    pub claimed_amount_usdc: u64,
    pub approved_amount_usdc: u64,
    pub recipient_owner: Pubkey,
    pub recipient_token_account: Pubkey,
    pub parameters_hash: [u8; 32],
    pub canonical_governance_payload_hash: [u8; 32],
    pub executor: Pubkey,
    pub executed_at: i64,
    pub schema_version: u16,
    pub bump: u8,
}

impl VictimReliefDecisionExecutionRecordV1 {
    pub const INIT_SPACE: usize = (32 * 14) + 4 + (8 * 3) + 2 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum VictimReliefPayoutStatusV1 {
    Approved,
    Executed,
    Cancelled,
}

#[account]
pub struct ReliefPayoutRequestV1 {
    pub victim_relief_case: Pubkey,
    pub config: Pubkey,
    pub policy: Pubkey,
    pub policy_version: u64,
    pub governance_proposal: Pubkey,
    pub proposal_decision: Pubkey,
    pub execution_queue_item: Pubkey,
    pub evidence_snapshot: Pubkey,
    pub approved_amount_usdc: u64,
    pub recipient_owner: Pubkey,
    pub recipient_token_account: Pubkey,
    pub treasury_config: Pubkey,
    pub relief_usdc_vault: Pubkey,
    pub usdc_mint: Pubkey,
    pub status: VictimReliefPayoutStatusV1,
    pub parameters_hash: [u8; 32],
    pub created_at: i64,
    pub executed_at: i64,
    pub schema_version: u16,
    pub bump: u8,
    pub reserved: [u8; 32],
}

impl ReliefPayoutRequestV1 {
    pub const INIT_SPACE: usize = (32 * 13) + (8 * 4) + 1 + 2 + 1 + 32;
}

/// Strict source path for a Victim Relief payout.
///
/// Variant order is part of serialized payout receipts. Do not reorder
/// existing variants; append new variants only.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum VictimReliefPayoutOriginV1 {
    OriginalApprove,
    AppealOverturn,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct VictimReliefPayoutParametersV1 {
    pub schema_version: u16,
    pub payout_origin: VictimReliefPayoutOriginV1,
    pub payout_request: Pubkey,
    pub victim_relief_case: Pubkey,
    pub config: Pubkey,
    pub policy: Pubkey,
    pub policy_version: u64,
    pub authorization_action_type: GovernanceActionTypeV1,
    pub governance_proposal: Pubkey,
    pub proposal_decision: Pubkey,
    pub execution_queue_item: Pubkey,
    pub governance_proposal_action: Pubkey,
    pub authorization_execution_record: Pubkey,
    pub evidence_snapshot: Pubkey,
    pub approved_amount_usdc: u64,
    pub recipient_owner: Pubkey,
    pub recipient_token_account: Pubkey,
    pub treasury_config: Pubkey,
    pub relief_usdc_vault: Pubkey,
    pub vault_authority_v2: Pubkey,
    pub usdc_mint: Pubkey,
    pub authorization_parameters_hash: [u8; 32],
}

#[account]
pub struct ReliefPayoutExecutionRecordV1 {
    pub payout_request: Pubkey,
    pub victim_relief_case: Pubkey,
    pub config: Pubkey,
    pub policy: Pubkey,
    pub policy_version: u64,
    pub payout_origin: VictimReliefPayoutOriginV1,
    pub authorization_action_type: GovernanceActionTypeV1,
    pub governance_proposal: Pubkey,
    pub proposal_decision: Pubkey,
    pub execution_queue_item: Pubkey,
    pub governance_proposal_action: Pubkey,
    pub authorization_execution_record: Pubkey,
    pub evidence_snapshot: Pubkey,
    pub relief_usdc_vault: Pubkey,
    pub vault_authority_v2: Pubkey,
    pub recipient_owner: Pubkey,
    pub recipient_token_account: Pubkey,
    pub amount_usdc: u64,
    pub usdc_mint: Pubkey,
    pub authorization_parameters_hash: [u8; 32],
    pub payout_parameters_hash: [u8; 32],
    pub executor: Pubkey,
    pub executed_at: i64,
    pub schema_version: u16,
    pub bump: u8,
    pub reserved: [u8; 32],
}

impl ReliefPayoutExecutionRecordV1 {
    pub const INIT_SPACE: usize = (32 * 16) + (8 * 3) + 2 + (32 * 3) + 2 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum VictimReliefAppealStatusV1 {
    Pending,
    Upheld,
    Overturned,
}

#[account]
pub struct VictimReliefAppealV1 {
    pub victim_relief_case: Pubkey,
    pub config: Pubkey,
    pub policy: Pubkey,
    pub policy_version: u64,
    pub claimant: Pubkey,
    pub original_evidence_snapshot: Pubkey,
    pub original_decision_record: Pubkey,
    pub original_governance_proposal: Pubkey,
    pub original_execution_queue_item: Pubkey,
    pub appeal_evidence_root: [u8; 32],
    pub appeal_evidence_count: u32,
    pub status: VictimReliefAppealStatusV1,
    pub decision_proposal: Pubkey,
    pub decision_queue: Pubkey,
    pub opened_at: i64,
    pub appeal_deadline: i64,
    pub resolved_at: i64,
    pub schema_version: u16,
    pub bump: u8,
    pub reserved: [u8; 32],
}

impl VictimReliefAppealV1 {
    pub const INIT_SPACE: usize = (32 * 10) + 32 + (8 * 4) + 4 + 1 + 2 + 1 + 32;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct VictimReliefAppealDecisionParametersV1 {
    pub schema_version: u16,
    pub config: Pubkey,
    pub policy: Pubkey,
    pub policy_version: u64,
    pub victim_relief_case: Pubkey,
    pub victim_relief_appeal: Pubkey,
    pub original_evidence_snapshot: Pubkey,
    pub original_decision_record: Pubkey,
    pub case_id: u64,
    pub claimant: Pubkey,
    pub subject_commitment: [u8; 32],
    pub original_evidence_root: [u8; 32],
    pub original_evidence_count: u32,
    pub original_evidence_revision: u32,
    pub appeal_evidence_root: [u8; 32],
    pub appeal_evidence_count: u32,
    pub claimed_amount_usdc: u64,
    pub approved_amount_usdc: u64,
    pub recipient_owner: Pubkey,
    pub recipient_token_account: Pubkey,
    pub usdc_mint: Pubkey,
    pub treasury_config: Pubkey,
    pub relief_usdc_vault: Pubkey,
    pub action_type: GovernanceActionTypeV1,
    pub expected_case_status: VictimReliefCaseStatusV1,
    pub expected_appeal_status: VictimReliefAppealStatusV1,
    pub proposal_id: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum VictimReliefAppealExecutionTypeV1 {
    Uphold,
    Overturn,
}

#[account]
pub struct VictimReliefAppealDecisionExecutionRecordV1 {
    pub execution_queue_item: Pubkey,
    pub proposal_decision: Pubkey,
    pub governance_proposal: Pubkey,
    pub governance_proposal_action: Pubkey,
    pub module_registry: Pubkey,
    pub config: Pubkey,
    pub policy: Pubkey,
    pub victim_relief_case: Pubkey,
    pub victim_relief_appeal: Pubkey,
    pub original_decision_record: Pubkey,
    pub original_evidence_snapshot: Pubkey,
    pub execution_type: VictimReliefAppealExecutionTypeV1,
    pub governance_action_type: GovernanceActionTypeV1,
    pub case_status_before: VictimReliefCaseStatusV1,
    pub case_status_after: VictimReliefCaseStatusV1,
    pub appeal_status_before: VictimReliefAppealStatusV1,
    pub appeal_status_after: VictimReliefAppealStatusV1,
    pub claimed_amount_usdc: u64,
    pub approved_amount_usdc: u64,
    pub recipient_owner: Pubkey,
    pub recipient_token_account: Pubkey,
    pub parameters_hash: [u8; 32],
    pub canonical_governance_payload_hash: [u8; 32],
    pub executor: Pubkey,
    pub executed_at: i64,
    pub schema_version: u16,
    pub bump: u8,
}

impl VictimReliefAppealDecisionExecutionRecordV1 {
    pub const INIT_SPACE: usize = (32 * 14) + 6 + (8 * 3) + (32 * 2) + 2 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GreenLabelStatus {
    PendingBondDeposit,
    PendingObservation,
    ActiveGreenLabel,
    Disputed,
    RefundQueued,
    SlashQueued,
    Refunded,
    Slashed,
    Cancelled,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum BondTier {
    Base,
    Bronze,
    Silver,
    Gold,
    Platinum,
    Custom,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum RugReasonCode {
    LiquidityRemoved,
    DeveloperDump,
    WebsiteOrCommunityAbandoned,
    MintOrFreezeAuthorityAbuse,
    TreasuryMisuse,
    FalseDisclosure,
    MaliciousContractUpgrade,
    Other,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisputeStatus {
    Open,
    EvidencePeriod,
    ProjectResponsePeriod,
    ReadyForDecision,
    DecisionQueued,
    ResolvedRefund,
    ResolvedSlash,
    Rejected,
    Cancelled,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GreenLabelEscrowStatusV1 {
    Locked,
    Refundable,
    Refunded,
    Forfeited,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GreenLabelCertificationStatusV1 {
    Pending,
    Approved,
    Rejected,
    Revoked,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GreenLabelCertificationExecutionTypeV1 {
    Approve,
    Reject,
    Revoke,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GreenLabelEscrowExecutionTypeV1 {
    Refund,
    Forfeit,
}

#[account]
pub struct GreenLabelConfigV1 {
    pub authority: Pubkey,
    pub usdc_mint: Pubkey,
    pub min_base_bond_usdc: u64,
    pub base_refund_bps: u16,
    pub base_treasury_bps: u16,
    pub observation_period_seconds: i64,
    pub dispute_window_seconds: i64,
    pub response_window_seconds: i64,
    pub project_count: u64,
    pub treasury_usdc_state_v2: Pubkey,
    pub base_bond_treasury_vault: Pubkey,
    pub relief_or_risk_vault: Pubkey,
    pub vault_authority_v2: Pubkey,
    pub security_governance_config: Pubkey,
    pub is_paused: bool,
    pub bump: u8,
    pub reserved: [u8; GREEN_LABEL_CONFIG_RESERVED_BYTES],
}

impl GreenLabelConfigV1 {
    pub const INIT_SPACE: usize = GREEN_LABEL_CONFIG_SPACE - ANCHOR_ACCOUNT_DISCRIMINATOR_BYTES;
}

#[account]
pub struct GreenLabelProjectV1 {
    pub project_id: u64,
    pub project_owner: Pubkey,
    pub project_name_hash: [u8; 32],
    pub project_url_hash: [u8; 32],
    pub token_mint: Pubkey,
    pub project_treasury_wallet: Pubkey,
    pub base_bond_amount: u64,
    pub extra_bond_amount: u64,
    pub total_bond_amount: u64,
    pub bond_vault: Pubkey,
    pub bond_vault_authority: Pubkey,
    pub bond_tier: BondTier,
    pub status: GreenLabelStatus,
    pub submitted_at: i64,
    pub observation_start_ts: i64,
    pub observation_end_ts: i64,
    pub dispute_count: u64,
    pub active_dispute: Pubkey,
    pub approved_at: i64,
    pub refunded_at: i64,
    pub slashed_at: i64,
    pub risk_score_snapshot: u16,
    pub terminal_proposal_id: u64,
    pub terminal_proposal_decision: Pubkey,
    pub terminal_execution_queue_item: Pubkey,
    pub terminal_payload_hash: [u8; 32],
    pub terminal_action_type: ActionType,
    pub bump: u8,
    pub reserved: [u8; GREEN_LABEL_PROJECT_RESERVED_BYTES],
}

impl GreenLabelProjectV1 {
    pub const INIT_SPACE: usize = GREEN_LABEL_PROJECT_SPACE - ANCHOR_ACCOUNT_DISCRIMINATOR_BYTES;
}

#[account]
pub struct GreenLabelDisputeV1 {
    pub project_id: u64,
    pub dispute_id: u64,
    pub project: Pubkey,
    pub disputer: Pubkey,
    pub reason_code: RugReasonCode,
    pub evidence_hash: [u8; 32],
    pub status: DisputeStatus,
    pub opened_at: i64,
    pub evidence_end_ts: i64,
    pub response_end_ts: i64,
    pub resolved_at: i64,
    pub proposal_id: u64,
    pub proposal_decision: Pubkey,
    pub execution_queue_item: Pubkey,
    pub payload_hash: [u8; 32],
    pub action_type: ActionType,
    pub bump: u8,
    pub reserved: [u8; GREEN_LABEL_DISPUTE_RESERVED_BYTES],
}

impl GreenLabelDisputeV1 {
    pub const INIT_SPACE: usize = GREEN_LABEL_DISPUTE_SPACE - ANCHOR_ACCOUNT_DISCRIMINATOR_BYTES;
}

#[account]
pub struct GreenLabelRefundableEscrowV1 {
    pub authority: Pubkey,
    pub project: Pubkey,
    pub project_id: u64,
    pub payer: Pubkey,
    pub usdc_mint: Pubkey,
    pub refundable_vault: Pubkey,
    pub deposited_amount: u64,
    pub refundable_amount: u64,
    pub refunded_amount: u64,
    pub forfeited_amount: u64,
    pub deposit_ts: i64,
    pub refund_available_after: i64,
    pub status: GreenLabelEscrowStatusV1,
    pub bump: u8,
    pub vault_bump: u8,
}

impl GreenLabelRefundableEscrowV1 {
    pub const INIT_SPACE: usize = (32 * 5) + (8 * 5) + (8 * 2) + 3;
}

#[account]
pub struct GreenLabelCertificationStateV1 {
    pub green_label_project: Pubkey,
    pub green_label_config: Pubkey,
    pub certification_status: GreenLabelCertificationStatusV1,
    pub last_governance_proposal: Pubkey,
    pub last_execution_queue: Pubkey,
    pub last_execution_record: Pubkey,
    pub last_action_type: GovernanceActionTypeV1,
    pub decision_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub schema_version: u16,
    pub bump: u8,
}

impl GreenLabelCertificationStateV1 {
    pub const INIT_SPACE: usize = (32 * 5) + 1 + 1 + (8 * 3) + 2 + 1;
}

#[account]
pub struct GreenLabelCertificationFeePolicyV1 {
    pub green_label_config: Pubkey,
    pub usdc_mint: Pubkey,
    pub fee_amount_usdc: u64,
    pub policy_version: u64,
    pub active: bool,
    pub initialized_by: Pubkey,
    pub created_at: i64,
    pub schema_version: u16,
    pub bump: u8,
}

impl GreenLabelCertificationFeePolicyV1 {
    pub const INIT_SPACE: usize = (32 * 3) + (8 * 3) + 1 + 2 + 1;
}

#[account]
pub struct GreenLabelCertificationFeeReceiptV1 {
    pub green_label_config: Pubkey,
    pub fee_policy: Pubkey,
    pub policy_version: u64,
    pub green_label_project: Pubkey,
    pub project_id: u64,
    pub project_owner: Pubkey,
    pub payer: Pubkey,
    pub payer_token_account: Pubkey,
    pub amount_usdc: u64,
    pub usdc_mint: Pubkey,
    pub treasury_config: Pubkey,
    pub treasury_usdc_state: Pubkey,
    pub revenue_routing_stats: Pubkey,
    pub relief_usdc_vault: Pubkey,
    pub buyback_usdc_vault: Pubkey,
    pub builders_usdc_vault: Pubkey,
    pub staking_usdc_vault: Pubkey,
    pub revenue_type: RevenueType,
    pub parameters_hash: [u8; 32],
    pub routed_at: i64,
    pub schema_version: u16,
    pub bump: u8,
}

impl GreenLabelCertificationFeeReceiptV1 {
    pub const INIT_SPACE: usize = (32 * 14) + (8 * 4) + 1 + 32 + 2 + 1;
}

#[account]
pub struct GreenLabelCertificationExecutionRecordV1 {
    pub execution_queue_item: Pubkey,
    pub proposal_decision: Pubkey,
    pub governance_proposal: Pubkey,
    pub governance_proposal_action: Pubkey,
    pub green_label_project: Pubkey,
    pub certification_state: Pubkey,
    pub module_registry: Pubkey,
    pub execution_type: GreenLabelCertificationExecutionTypeV1,
    pub governance_action_type: GovernanceActionTypeV1,
    pub target_account: Pubkey,
    pub parameters_hash: [u8; 32],
    pub canonical_governance_payload_hash: [u8; 32],
    pub project_status_before: GreenLabelStatus,
    pub project_status_after: GreenLabelStatus,
    pub certification_status_before: GreenLabelCertificationStatusV1,
    pub certification_status_after: GreenLabelCertificationStatusV1,
    pub executor: Pubkey,
    pub executed_at: i64,
    pub schema_version: u16,
    pub bump: u8,
}

impl GreenLabelCertificationExecutionRecordV1 {
    pub const INIT_SPACE: usize = (32 * 8) + 1 + 1 + (32 * 2) + 1 + 1 + 1 + 1 + 32 + 8 + 2 + 1;
}

#[account]
pub struct GreenLabelRefundExecutionRecordV1 {
    pub execution_queue_item: Pubkey,
    pub proposal_decision: Pubkey,
    pub governance_proposal: Pubkey,
    pub governance_proposal_action: Pubkey,
    pub module_registry: Pubkey,
    pub green_label_config: Pubkey,
    pub green_label_project: Pubkey,
    pub green_label_dispute: Pubkey,
    pub refundable_escrow: Pubkey,
    pub refundable_vault: Pubkey,
    pub original_payer: Pubkey,
    pub payer_destination_token_account: Pubkey,
    pub refund_amount_usdc: u64,
    pub usdc_mint: Pubkey,
    pub execution_type: GreenLabelEscrowExecutionTypeV1,
    pub governance_action_type: GovernanceActionTypeV1,
    pub parameters_hash: [u8; 32],
    pub canonical_governance_payload_hash: [u8; 32],
    pub escrow_status_before: GreenLabelEscrowStatusV1,
    pub escrow_status_after: GreenLabelEscrowStatusV1,
    pub project_status_before: GreenLabelStatus,
    pub project_status_after: GreenLabelStatus,
    pub executor: Pubkey,
    pub executed_at: i64,
    pub schema_version: u16,
    pub bump: u8,
}

impl GreenLabelRefundExecutionRecordV1 {
    pub const INIT_SPACE: usize = (32 * 14) + 8 + 1 + 1 + (32 * 2) + 1 + 1 + 1 + 1 + 8 + 2 + 1;
}

#[account]
pub struct GreenLabelForfeitExecutionRecordV1 {
    pub execution_queue_item: Pubkey,
    pub proposal_decision: Pubkey,
    pub governance_proposal: Pubkey,
    pub governance_proposal_action: Pubkey,
    pub module_registry: Pubkey,
    pub green_label_config: Pubkey,
    pub green_label_project: Pubkey,
    pub green_label_dispute: Pubkey,
    pub refundable_escrow: Pubkey,
    pub refundable_vault: Pubkey,
    pub treasury_config: Pubkey,
    pub treasury_usdc_state: Pubkey,
    pub revenue_routing_stats: Pubkey,
    pub relief_usdc_vault: Pubkey,
    pub buyback_usdc_vault: Pubkey,
    pub builders_usdc_vault: Pubkey,
    pub staking_usdc_vault: Pubkey,
    pub forfeited_amount_usdc: u64,
    pub usdc_mint: Pubkey,
    pub revenue_type: RevenueType,
    pub execution_type: GreenLabelEscrowExecutionTypeV1,
    pub governance_action_type: GovernanceActionTypeV1,
    pub parameters_hash: [u8; 32],
    pub canonical_governance_payload_hash: [u8; 32],
    pub escrow_status_before: GreenLabelEscrowStatusV1,
    pub escrow_status_after: GreenLabelEscrowStatusV1,
    pub project_status_before: GreenLabelStatus,
    pub project_status_after: GreenLabelStatus,
    pub dispute_status_before: DisputeStatus,
    pub dispute_status_after: DisputeStatus,
    pub executor: Pubkey,
    pub executed_at: i64,
    pub schema_version: u16,
    pub bump: u8,
}

impl GreenLabelForfeitExecutionRecordV1 {
    pub const INIT_SPACE: usize = (32 * 19) + 8 + (32 * 2) + 9 + 8 + 2 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContributorRoleV1 {
    CoreDeveloper,
    BackendDeveloper,
    FrontendDeveloper,
    SecurityResearcher,
    ProtocolResearcher,
    Designer,
    CommunityManager,
    Operations,
    TreasuryReviewer,
    Translator,
    Ambassador,
    Other,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContributorStatusV1 {
    Active,
    Suspended,
    Removed,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MilestoneStatusV1 {
    Pending,
    Approved,
    Rejected,
    Paid,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PayoutStatusV1 {
    Pending,
    Approved,
    Rejected,
    Executed,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContributorProposalTypeV1 {
    AddContributor,
    RemoveContributor,
    UpdateContributorRole,
    ApproveMilestone,
    ApproveBuilderPayout,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContributorActionTypeV1 {
    AddContributor,
    RemoveContributor,
    UpdateContributorRole,
    ApproveMilestone,
    ApproveBuilderPayout,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct AddContributorPayloadV1 {
    pub contributor_wallet: Pubkey,
    pub contributor_role: ContributorRoleV1,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemoveContributorPayloadV1 {
    pub contributor_registry: Pubkey,
    pub reason_hash: [u8; 32],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct UpdateContributorRolePayloadV1 {
    pub contributor_registry: Pubkey,
    pub new_role: ContributorRoleV1,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApproveContributorMilestonePayloadV1 {
    pub milestone: Pubkey,
    pub approved_amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApproveBuilderPayoutPayloadV1 {
    pub payout_request: Pubkey,
    pub approved_amount: u64,
}

#[account]
pub struct ContributorRegistryV1 {
    pub wallet: Pubkey,
    pub role: ContributorRoleV1,
    pub status: ContributorStatusV1,
    pub joined_at: i64,
    pub last_active_at: i64,
    pub completed_milestones: u32,
    pub approved_payout_count: u32,
    pub reputation_score: u64,
    pub bump: u8,
}

impl ContributorRegistryV1 {
    pub const INIT_SPACE: usize = 32 + 1 + 1 + (8 * 2) + (4 * 2) + 8 + 1;
}

#[account]
pub struct ContributorMilestoneV1 {
    pub contributor: Pubkey,
    pub title: String,
    pub description: String,
    pub evidence_hash: [u8; 32],
    pub requested_amount: u64,
    pub status: MilestoneStatusV1,
    pub created_at: i64,
    pub bump: u8,
}

impl ContributorMilestoneV1 {
    pub const INIT_SPACE: usize = 32
        + 4
        + CONTRIBUTOR_MILESTONE_TITLE_MAX_BYTES
        + 4
        + CONTRIBUTOR_MILESTONE_DESCRIPTION_MAX_BYTES
        + 32
        + 8
        + 1
        + 8
        + 1;
}

#[account]
pub struct BuilderPayoutRequestV1 {
    pub contributor: Pubkey,
    pub milestone: Pubkey,
    pub amount: u64,
    pub destination_wallet: Pubkey,
    pub status: PayoutStatusV1,
    pub created_at: i64,
    pub bump: u8,
}

impl BuilderPayoutRequestV1 {
    pub const INIT_SPACE: usize = (32 * 3) + 8 + 1 + 8 + 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn governance_proposal_space_covers_fields() {
        let minimum = 8 + 32 + 1 + 1 + 32 + 32 + 32 + 1 + (8 * 3) + 32 + (8 * 4) + 1;
        assert!(GovernanceProposalV1::INIT_SPACE >= minimum);
    }

    #[test]
    fn governance_proposal_action_space_is_exact() {
        let minimum = (32 * 4) + 8 + 1 + 1 + (32 * 3) + 2 + 8 + 1;
        assert_eq!(GovernanceProposalActionV1::INIT_SPACE, minimum);
        assert_eq!(GovernanceProposalActionV1::INIT_SPACE, 245);
    }

    #[test]
    fn protocol_module_registry_space_is_exact() {
        let minimum = 32 + 1 + 1 + 32 + 1 + 2 + 8 + 8 + 1;
        assert_eq!(ProtocolModuleRegistryV1::INIT_SPACE, minimum);
        assert_eq!(ProtocolModuleRegistryV1::INIT_SPACE, 86);
    }

    #[test]
    fn governance_position_space_covers_fields() {
        let minimum = (32 * 3) + (8 * 6) + 1 + 1;
        assert!(GovernancePositionV1::INIT_SPACE >= minimum);
    }

    #[test]
    fn governance_lock_config_space_covers_fields() {
        let minimum = (32 * 3) + (8 * 5) + 1;
        assert!(GovernanceLockConfigV1::INIT_SPACE >= minimum);
    }

    #[test]
    fn governance_power_state_space_covers_fields() {
        let minimum = 32 + (8 * 4) + 1;
        assert!(GovernancePowerStateV1::INIT_SPACE >= minimum);
    }

    #[test]
    fn governance_position_vote_lock_space_covers_fields() {
        let minimum = (32 * 2) + (8 * 2) + 1;
        assert!(GovernancePositionVoteLockV1::INIT_SPACE >= minimum);
    }

    #[test]
    fn governance_voting_config_space_covers_fields() {
        let minimum = 32 + (8 * 4) + 1;
        assert!(GovernanceVotingConfigV1::INIT_SPACE >= minimum);
    }

    #[test]
    fn governance_snapshot_space_covers_fields() {
        let minimum = 32 + (8 * 5) + 1 + 1;
        assert!(GovernanceSnapshotV1::INIT_SPACE >= minimum);
    }

    #[test]
    fn vote_record_space_covers_fields() {
        let minimum = (32 * 2) + 1 + (8 * 2) + 1;
        assert!(VoteRecordV1::INIT_SPACE >= minimum);
    }

    #[test]
    fn universal_governance_decision_adapter_space_covers_fields() {
        let minimum = (32 * 5) + 1 + 8 + 1 + 1;
        assert!(UniversalGovernanceDecisionAdapterV1::INIT_SPACE >= minimum);
    }

    #[test]
    fn victim_relief_config_space_is_exact() {
        let minimum = (32 * 5) + (8 * 2) + 1 + 8 + 2 + 1 + 32;
        assert_eq!(VictimReliefConfigV1::INIT_SPACE, minimum);
        assert_eq!(VictimReliefConfigV1::INIT_SPACE, 220);
    }

    #[test]
    fn victim_relief_policy_space_is_exact() {
        let minimum = 32 + (8 * 8) + 4 + 2 + 1 + 32 + 8 + 2 + 1 + 32;
        assert_eq!(VictimReliefPolicyV1::INIT_SPACE, minimum);
        assert_eq!(VictimReliefPolicyV1::INIT_SPACE, 178);
    }

    #[test]
    fn victim_relief_claimant_state_space_is_exact() {
        let minimum = (32 * 2) + 2 + (8 * 5) + 2 + 1;
        assert_eq!(VictimReliefClaimantStateV1::INIT_SPACE, minimum);
        assert_eq!(VictimReliefClaimantStateV1::INIT_SPACE, 109);
    }

    #[test]
    fn victim_relief_case_space_is_exact() {
        let minimum = 8
            + (32 * 2)
            + 8
            + 32
            + (32 * 2)
            + (4 * 2)
            + (8 * 2)
            + (32 * 3)
            + 1
            + (32 * 3)
            + (8 * 5)
            + 2
            + 1
            + 64;
        assert_eq!(VictimReliefCaseV1::INIT_SPACE, minimum);
        assert_eq!(VictimReliefCaseV1::INIT_SPACE, 500);
    }

    #[test]
    fn victim_relief_evidence_snapshot_space_is_exact() {
        let minimum = (32 * 9) + (8 * 4) + (4 * 2) + 2 + 1 + 32;
        assert_eq!(VictimReliefEvidenceSnapshotV1::INIT_SPACE, minimum);
        assert_eq!(VictimReliefEvidenceSnapshotV1::INIT_SPACE, 363);
    }

    #[test]
    fn victim_relief_decision_execution_record_space_is_exact() {
        let minimum = (32 * 14) + 4 + (8 * 3) + 2 + 1;
        assert_eq!(VictimReliefDecisionExecutionRecordV1::INIT_SPACE, minimum);
        assert_eq!(VictimReliefDecisionExecutionRecordV1::INIT_SPACE, 479);
    }

    #[test]
    fn relief_payout_request_space_is_exact() {
        let minimum = (32 * 13) + (8 * 4) + 1 + 2 + 1 + 32;
        assert_eq!(ReliefPayoutRequestV1::INIT_SPACE, minimum);
        assert_eq!(ReliefPayoutRequestV1::INIT_SPACE, 484);
    }

    #[test]
    fn relief_payout_execution_record_space_is_exact() {
        let minimum = (32 * 16) + (8 * 3) + 2 + (32 * 3) + 2 + 1;
        assert_eq!(ReliefPayoutExecutionRecordV1::INIT_SPACE, minimum);
        assert_eq!(ReliefPayoutExecutionRecordV1::INIT_SPACE, 637);
    }

    #[test]
    fn victim_relief_appeal_space_is_exact() {
        let minimum = (32 * 10) + 32 + (8 * 4) + 4 + 1 + 2 + 1 + 32;
        assert_eq!(VictimReliefAppealV1::INIT_SPACE, minimum);
        assert_eq!(VictimReliefAppealV1::INIT_SPACE, 424);
    }

    #[test]
    fn victim_relief_appeal_decision_execution_record_space_is_exact() {
        let minimum = (32 * 14) + 6 + (8 * 3) + (32 * 2) + 2 + 1;
        assert_eq!(
            VictimReliefAppealDecisionExecutionRecordV1::INIT_SPACE,
            minimum
        );
        assert_eq!(VictimReliefAppealDecisionExecutionRecordV1::INIT_SPACE, 545);
    }

    #[test]
    fn treasury_governance_config_space_covers_fields() {
        let minimum = (32 * 2) + 1 + (8 * 4) + 1 + 1;
        assert!(TreasuryGovernanceConfigV1::INIT_SPACE >= minimum);
    }

    #[test]
    fn treasury_spending_request_space_covers_fields() {
        let minimum = 8 + (32 * 4) + (8 * 4) + 1 + 1;
        assert!(TreasurySpendingRequestV1::INIT_SPACE >= minimum);
    }

    #[test]
    fn treasury_builder_payout_governance_space_covers_fields() {
        let minimum = (32 * 4) + (8 * 3) + 1 + 1;
        assert!(TreasuryBuilderPayoutGovernanceV1::INIT_SPACE >= minimum);
    }

    #[test]
    fn treasury_execution_record_space_is_exact() {
        let minimum = (32 * 10) + 2 + 8 + (32 * 2) + 8 + 2 + 1;
        assert_eq!(TreasuryExecutionRecordV1::INIT_SPACE, minimum);
        assert_eq!(TreasuryExecutionRecordV1::INIT_SPACE, 405);
    }

    #[test]
    fn green_label_certification_state_space_is_exact() {
        let minimum = (32 * 5) + 1 + 1 + (8 * 3) + 2 + 1;
        assert_eq!(GreenLabelCertificationStateV1::INIT_SPACE, minimum);
        assert_eq!(GreenLabelCertificationStateV1::INIT_SPACE, 189);
    }

    #[test]
    fn green_label_certification_execution_record_space_is_exact() {
        let minimum = (32 * 8) + 1 + 1 + (32 * 2) + 1 + 1 + 1 + 1 + 32 + 8 + 2 + 1;
        assert_eq!(
            GreenLabelCertificationExecutionRecordV1::INIT_SPACE,
            minimum
        );
        assert_eq!(GreenLabelCertificationExecutionRecordV1::INIT_SPACE, 369);
    }

    #[test]
    fn green_label_refund_execution_record_space_is_exact() {
        let minimum = (32 * 14) + 8 + 1 + 1 + (32 * 2) + 1 + 1 + 1 + 1 + 8 + 2 + 1;
        assert_eq!(GreenLabelRefundExecutionRecordV1::INIT_SPACE, minimum);
        assert_eq!(GreenLabelRefundExecutionRecordV1::INIT_SPACE, 537);
    }

    #[test]
    fn governance_core_enums_are_copy_and_comparable() {
        assert_eq!(
            GovernanceProposalStatusV1::Draft,
            GovernanceProposalStatusV1::Draft
        );
        assert_ne!(VoteChoiceV1::Yes, VoteChoiceV1::No);
        assert_eq!(
            GovernanceProposalTypeV1::Contributor,
            GovernanceProposalTypeV1::Contributor
        );
        assert_eq!(
            GovernancePositionStatusV1::Active,
            GovernancePositionStatusV1::Active
        );
    }
}
