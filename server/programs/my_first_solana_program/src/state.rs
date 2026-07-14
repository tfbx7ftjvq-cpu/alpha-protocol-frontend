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
