use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};

use crate::constants::{
    BUILDERS_USDC_VAULT_SEED, BUYBACK_USDC_VAULT_SEED, EXECUTION_QUEUE_ITEM_V1_SEED,
    GOVERNANCE_CONFIG_V1_SEED, GOVERNANCE_PROPOSAL_ACTION_V1_SEED, GOVERNANCE_PROPOSAL_V1_SEED,
    GREEN_LABEL_USDC_DECIMALS, PROPOSAL_DECISION_V1_SEED, PROTOCOL_MODULE_REGISTRY_V1_SEED,
    RELIEF_PAYOUT_EXECUTION_RECORD_V1_SEED, RELIEF_PAYOUT_REQUEST_V1_SEED, RELIEF_USDC_VAULT_SEED,
    STAKING_USDC_VAULT_SEED, TREASURY_CONFIG_V2_SEED,
    UNIVERSAL_GOVERNANCE_DECISION_ADAPTER_V1_SEED, VAULT_AUTHORITY_V2_SEED,
    VICTIM_RELIEF_APPEAL_DECISION_EXECUTION_RECORD_V1_SEED, VICTIM_RELIEF_APPEAL_V1_SEED,
    VICTIM_RELIEF_CASE_V1_SEED, VICTIM_RELIEF_CLAIMANT_STATE_V1_SEED, VICTIM_RELIEF_CONFIG_V1_SEED,
    VICTIM_RELIEF_DECISION_EXECUTION_RECORD_V1_SEED, VICTIM_RELIEF_EVIDENCE_SNAPSHOT_V1_SEED,
    VICTIM_RELIEF_POLICY_V1_SEED, VICTIM_RELIEF_POLICY_VERSION_V1, VICTIM_RELIEF_SCHEMA_VERSION_V1,
};
use crate::error::CustomError;
use crate::instructions::contributor_v1::hash_contributor_payload;
use crate::instructions::governance_action_v1::{
    hash_governance_payload_v1, map_governance_action_to_security_action,
    GOVERNANCE_PAYLOAD_V1_SCHEMA_VERSION,
};
use crate::instructions::governance_adapter_v1::security_proposal_type_for_action;
use crate::instructions::governance_v1::validate_governance_proposal_action_v1;
use crate::instructions::protocol_module_registry_v1::{
    protocol_module_stable_code_v1, validate_protocol_module_registry_v1,
};
use crate::state::{
    ExecutionQueueItemV1, ExecutionStatus, GovernanceActionTypeV1, GovernanceConfigV1,
    GovernancePayloadV1, GovernanceProposalActionV1, GovernanceProposalStatusV1,
    GovernanceProposalV1, ProposalDecision, ProposalDecisionV1, ProtocolModuleIdV1,
    ProtocolModuleRegistryV1, ReliefPayoutExecutionRecordV1, ReliefPayoutRequestV1,
    TreasuryConfigV2, UniversalGovernanceDecisionAdapterV1,
    VictimReliefAppealDecisionExecutionRecordV1, VictimReliefAppealDecisionParametersV1,
    VictimReliefAppealExecutionTypeV1, VictimReliefAppealStatusV1, VictimReliefAppealV1,
    VictimReliefCaseStatusV1, VictimReliefCaseV1, VictimReliefClaimantStateV1,
    VictimReliefConfigV1, VictimReliefDecisionExecutionRecordV1,
    VictimReliefDecisionExecutionTypeV1, VictimReliefDecisionParametersV1,
    VictimReliefEvidenceSnapshotV1, VictimReliefPayoutOriginV1, VictimReliefPayoutParametersV1,
    VictimReliefPayoutStatusV1, VictimReliefPolicyV1,
};

pub const VICTIM_RELIEF_DECISION_SCHEMA_VERSION: u16 = 1;
pub const VICTIM_RELIEF_DECISION_PARAMETERS_V1_DOMAIN_BYTES: [u8; 42] =
    *b"alpha_victim_relief_decision_parameters_v1";
pub const VICTIM_RELIEF_APPEAL_DECISION_PARAMETERS_V1_DOMAIN_BYTES: [u8; 49] =
    *b"alpha_victim_relief_appeal_decision_parameters_v1";
pub const VICTIM_RELIEF_PAYOUT_PARAMETERS_V1_DOMAIN_BYTES: [u8; 40] =
    *b"alpha_victim_relief_payout_parameters_v1";

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct VictimReliefDecisionParametersHashEnvelopeV1 {
    pub domain_separator: [u8; 42],
    pub parameters: VictimReliefDecisionParametersV1,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct VictimReliefAppealDecisionParametersHashEnvelopeV1 {
    pub domain_separator: [u8; 49],
    pub parameters: VictimReliefAppealDecisionParametersV1,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct VictimReliefPayoutParametersHashEnvelopeV1 {
    pub domain_separator: [u8; 40],
    pub parameters: VictimReliefPayoutParametersV1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VictimReliefPolicyParametersV1 {
    pub min_claim_amount_usdc: u64,
    pub max_claim_amount_usdc: u64,
    pub max_payout_per_case_usdc: u64,
    pub evidence_window_seconds: i64,
    pub review_window_seconds: i64,
    pub appeal_window_seconds: i64,
    pub submission_cooldown_seconds: i64,
    pub max_evidence_items: u32,
    pub max_active_cases_per_claimant: u16,
}

#[derive(Accounts)]
pub struct InitializeVictimReliefConfigV1<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + VictimReliefConfigV1::INIT_SPACE,
        seeds = [VICTIM_RELIEF_CONFIG_V1_SEED],
        bump
    )]
    pub victim_relief_config: Account<'info, VictimReliefConfigV1>,

    #[account(
        seeds = [GOVERNANCE_CONFIG_V1_SEED],
        bump = security_governance_config.bump
    )]
    pub security_governance_config: Account<'info, GovernanceConfigV1>,

    #[account(
        seeds = [TREASURY_CONFIG_V2_SEED],
        bump = treasury_config.bump
    )]
    pub treasury_config: Account<'info, TreasuryConfigV2>,

    #[account(
        constraint = usdc_mint.key() == treasury_config.usdc_mint @ CustomError::InvalidMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub bootstrap_authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeVictimReliefPolicyV1<'info> {
    #[account(
        mut,
        seeds = [VICTIM_RELIEF_CONFIG_V1_SEED],
        bump = victim_relief_config.bump
    )]
    pub victim_relief_config: Account<'info, VictimReliefConfigV1>,

    #[account(
        init,
        payer = payer,
        space = 8 + VictimReliefPolicyV1::INIT_SPACE,
        seeds = [
            VICTIM_RELIEF_POLICY_V1_SEED,
            victim_relief_config.key().as_ref(),
            &VICTIM_RELIEF_POLICY_VERSION_V1.to_le_bytes()
        ],
        bump
    )]
    pub victim_relief_policy: Account<'info, VictimReliefPolicyV1>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(case_id: u64)]
pub struct SubmitVictimReliefCaseV1<'info> {
    #[account(
        mut,
        seeds = [VICTIM_RELIEF_CONFIG_V1_SEED],
        bump = victim_relief_config.bump
    )]
    pub victim_relief_config: Account<'info, VictimReliefConfigV1>,

    #[account(
        seeds = [
            VICTIM_RELIEF_POLICY_V1_SEED,
            victim_relief_config.key().as_ref(),
            &VICTIM_RELIEF_POLICY_VERSION_V1.to_le_bytes()
        ],
        bump = victim_relief_policy.bump,
        constraint = victim_relief_config.current_policy == victim_relief_policy.key() @ CustomError::InvalidVictimReliefPolicy,
        constraint = victim_relief_config.current_policy_version == victim_relief_policy.policy_version @ CustomError::InvalidVictimReliefPolicyVersion
    )]
    pub victim_relief_policy: Account<'info, VictimReliefPolicyV1>,

    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + VictimReliefClaimantStateV1::INIT_SPACE,
        seeds = [
            VICTIM_RELIEF_CLAIMANT_STATE_V1_SEED,
            victim_relief_config.key().as_ref(),
            claimant.key().as_ref()
        ],
        bump
    )]
    pub claimant_state: Account<'info, VictimReliefClaimantStateV1>,

    #[account(
        init,
        payer = payer,
        space = 8 + VictimReliefCaseV1::INIT_SPACE,
        seeds = [
            VICTIM_RELIEF_CASE_V1_SEED,
            victim_relief_config.key().as_ref(),
            &case_id.to_le_bytes()
        ],
        bump
    )]
    pub victim_relief_case: Account<'info, VictimReliefCaseV1>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub claimant: Signer<'info>,

    pub claimant_recipient_usdc_account: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = usdc_mint.key() == victim_relief_config.usdc_mint @ CustomError::InvalidMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateVictimReliefEvidenceRootV1<'info> {
    #[account(
        seeds = [VICTIM_RELIEF_CONFIG_V1_SEED],
        bump = victim_relief_config.bump
    )]
    pub victim_relief_config: Box<Account<'info, VictimReliefConfigV1>>,

    #[account(
        seeds = [
            VICTIM_RELIEF_POLICY_V1_SEED,
            victim_relief_config.key().as_ref(),
            &VICTIM_RELIEF_POLICY_VERSION_V1.to_le_bytes()
        ],
        bump = victim_relief_policy.bump
    )]
    pub victim_relief_policy: Account<'info, VictimReliefPolicyV1>,

    #[account(
        mut,
        seeds = [
            VICTIM_RELIEF_CASE_V1_SEED,
            victim_relief_config.key().as_ref(),
            &victim_relief_case.case_id.to_le_bytes()
        ],
        bump = victim_relief_case.bump
    )]
    pub victim_relief_case: Box<Account<'info, VictimReliefCaseV1>>,

    pub claimant: Signer<'info>,
}

#[derive(Accounts)]
pub struct CancelVictimReliefCaseV1<'info> {
    #[account(
        seeds = [VICTIM_RELIEF_CONFIG_V1_SEED],
        bump = victim_relief_config.bump
    )]
    pub victim_relief_config: Account<'info, VictimReliefConfigV1>,

    #[account(
        mut,
        seeds = [
            VICTIM_RELIEF_CLAIMANT_STATE_V1_SEED,
            victim_relief_config.key().as_ref(),
            claimant.key().as_ref()
        ],
        bump = claimant_state.bump
    )]
    pub claimant_state: Account<'info, VictimReliefClaimantStateV1>,

    #[account(
        mut,
        seeds = [
            VICTIM_RELIEF_CASE_V1_SEED,
            victim_relief_config.key().as_ref(),
            &victim_relief_case.case_id.to_le_bytes()
        ],
        bump = victim_relief_case.bump
    )]
    pub victim_relief_case: Account<'info, VictimReliefCaseV1>,

    pub claimant: Signer<'info>,
}

#[derive(Accounts)]
pub struct ExpireVictimReliefCaseV1<'info> {
    #[account(
        seeds = [VICTIM_RELIEF_CONFIG_V1_SEED],
        bump = victim_relief_config.bump
    )]
    pub victim_relief_config: Account<'info, VictimReliefConfigV1>,

    #[account(
        mut,
        seeds = [
            VICTIM_RELIEF_CLAIMANT_STATE_V1_SEED,
            victim_relief_config.key().as_ref(),
            victim_relief_case.claimant.as_ref()
        ],
        bump = claimant_state.bump
    )]
    pub claimant_state: Account<'info, VictimReliefClaimantStateV1>,

    #[account(
        mut,
        seeds = [
            VICTIM_RELIEF_CASE_V1_SEED,
            victim_relief_config.key().as_ref(),
            &victim_relief_case.case_id.to_le_bytes()
        ],
        bump = victim_relief_case.bump
    )]
    pub victim_relief_case: Account<'info, VictimReliefCaseV1>,

    pub executor: Signer<'info>,
}

#[derive(Accounts)]
pub struct FreezeVictimReliefEvidenceV1<'info> {
    #[account(
        seeds = [VICTIM_RELIEF_CONFIG_V1_SEED],
        bump = victim_relief_config.bump
    )]
    pub victim_relief_config: Account<'info, VictimReliefConfigV1>,

    #[account(
        mut,
        seeds = [
            VICTIM_RELIEF_CASE_V1_SEED,
            victim_relief_config.key().as_ref(),
            &victim_relief_case.case_id.to_le_bytes()
        ],
        bump = victim_relief_case.bump
    )]
    pub victim_relief_case: Account<'info, VictimReliefCaseV1>,

    #[account(
        seeds = [
            VICTIM_RELIEF_POLICY_V1_SEED,
            victim_relief_config.key().as_ref(),
            &victim_relief_case.policy_version.to_le_bytes()
        ],
        bump = victim_relief_policy.bump
    )]
    pub victim_relief_policy: Box<Account<'info, VictimReliefPolicyV1>>,

    #[account(
        init,
        payer = claimant,
        space = 8 + VictimReliefEvidenceSnapshotV1::INIT_SPACE,
        seeds = [
            VICTIM_RELIEF_EVIDENCE_SNAPSHOT_V1_SEED,
            victim_relief_case.key().as_ref()
        ],
        bump
    )]
    pub evidence_snapshot: Box<Account<'info, VictimReliefEvidenceSnapshotV1>>,

    #[account(mut)]
    pub claimant: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteApproveVictimReliefCaseV1<'info> {
    #[account(seeds = [GOVERNANCE_CONFIG_V1_SEED], bump = security_governance_config.bump)]
    pub security_governance_config: Box<Account<'info, GovernanceConfigV1>>,

    #[account(
        seeds = [
            PROTOCOL_MODULE_REGISTRY_V1_SEED,
            &[protocol_module_stable_code_v1(ProtocolModuleIdV1::VictimRelief)]
        ],
        bump = protocol_module_registry.bump
    )]
    pub protocol_module_registry: Box<Account<'info, ProtocolModuleRegistryV1>>,

    #[account(
        seeds = [GOVERNANCE_PROPOSAL_V1_SEED, &governance_proposal.proposal_id.to_le_bytes()],
        bump = governance_proposal.bump
    )]
    pub governance_proposal: Box<Account<'info, GovernanceProposalV1>>,

    #[account(
        seeds = [
            GOVERNANCE_PROPOSAL_ACTION_V1_SEED,
            governance_proposal.key().as_ref()
        ],
        bump = governance_proposal_action.bump
    )]
    pub governance_proposal_action: Box<Account<'info, GovernanceProposalActionV1>>,

    #[account(
        seeds = [
            UNIVERSAL_GOVERNANCE_DECISION_ADAPTER_V1_SEED,
            governance_proposal.key().as_ref()
        ],
        bump = governance_decision_adapter.bump
    )]
    pub governance_decision_adapter: Box<Account<'info, UniversalGovernanceDecisionAdapterV1>>,

    #[account(
        seeds = [PROPOSAL_DECISION_V1_SEED, &governance_proposal.proposal_id.to_le_bytes()],
        bump = proposal_decision.bump
    )]
    pub proposal_decision: Box<Account<'info, ProposalDecisionV1>>,

    #[account(
        seeds = [EXECUTION_QUEUE_ITEM_V1_SEED, &governance_proposal.proposal_id.to_le_bytes()],
        bump = execution_queue_item.bump
    )]
    pub execution_queue_item: Box<Account<'info, ExecutionQueueItemV1>>,

    #[account(seeds = [VICTIM_RELIEF_CONFIG_V1_SEED], bump = victim_relief_config.bump)]
    pub victim_relief_config: Box<Account<'info, VictimReliefConfigV1>>,

    #[account(
        mut,
        seeds = [
            VICTIM_RELIEF_CASE_V1_SEED,
            victim_relief_config.key().as_ref(),
            &victim_relief_case.case_id.to_le_bytes()
        ],
        bump = victim_relief_case.bump
    )]
    pub victim_relief_case: Box<Account<'info, VictimReliefCaseV1>>,

    #[account(
        seeds = [
            VICTIM_RELIEF_POLICY_V1_SEED,
            victim_relief_config.key().as_ref(),
            &victim_relief_case.policy_version.to_le_bytes()
        ],
        bump = victim_relief_policy.bump
    )]
    pub victim_relief_policy: Box<Account<'info, VictimReliefPolicyV1>>,

    #[account(
        seeds = [
            VICTIM_RELIEF_CLAIMANT_STATE_V1_SEED,
            victim_relief_config.key().as_ref(),
            victim_relief_case.claimant.as_ref()
        ],
        bump = claimant_state.bump
    )]
    pub claimant_state: Box<Account<'info, VictimReliefClaimantStateV1>>,

    #[account(
        seeds = [
            VICTIM_RELIEF_EVIDENCE_SNAPSHOT_V1_SEED,
            victim_relief_case.key().as_ref()
        ],
        bump = evidence_snapshot.bump
    )]
    pub evidence_snapshot: Box<Account<'info, VictimReliefEvidenceSnapshotV1>>,

    #[account(seeds = [TREASURY_CONFIG_V2_SEED], bump = treasury_config.bump)]
    pub treasury_config: Box<Account<'info, TreasuryConfigV2>>,

    #[account(seeds = [VAULT_AUTHORITY_V2_SEED], bump)]
    /// CHECK: PDA-only vault authority.
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        seeds = [RELIEF_USDC_VAULT_SEED],
        bump,
        constraint = relief_usdc_vault.mint == treasury_config.usdc_mint @ CustomError::InvalidMint,
        constraint = relief_usdc_vault.owner == vault_authority.key() @ CustomError::VictimReliefReliefVaultMismatch
    )]
    pub relief_usdc_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = usdc_mint.key() == treasury_config.usdc_mint @ CustomError::InvalidMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = executor,
        space = 8 + ReliefPayoutRequestV1::INIT_SPACE,
        seeds = [
            RELIEF_PAYOUT_REQUEST_V1_SEED,
            victim_relief_case.key().as_ref()
        ],
        bump
    )]
    pub relief_payout_request: Box<Account<'info, ReliefPayoutRequestV1>>,

    #[account(
        init,
        payer = executor,
        space = 8 + VictimReliefDecisionExecutionRecordV1::INIT_SPACE,
        seeds = [
            VICTIM_RELIEF_DECISION_EXECUTION_RECORD_V1_SEED,
            execution_queue_item.key().as_ref()
        ],
        bump
    )]
    pub decision_execution_record: Box<Account<'info, VictimReliefDecisionExecutionRecordV1>>,

    #[account(mut)]
    pub executor: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteRejectVictimReliefCaseV1<'info> {
    #[account(seeds = [GOVERNANCE_CONFIG_V1_SEED], bump = security_governance_config.bump)]
    pub security_governance_config: Box<Account<'info, GovernanceConfigV1>>,

    #[account(
        seeds = [
            PROTOCOL_MODULE_REGISTRY_V1_SEED,
            &[protocol_module_stable_code_v1(ProtocolModuleIdV1::VictimRelief)]
        ],
        bump = protocol_module_registry.bump
    )]
    pub protocol_module_registry: Box<Account<'info, ProtocolModuleRegistryV1>>,

    #[account(
        seeds = [GOVERNANCE_PROPOSAL_V1_SEED, &governance_proposal.proposal_id.to_le_bytes()],
        bump = governance_proposal.bump
    )]
    pub governance_proposal: Box<Account<'info, GovernanceProposalV1>>,

    #[account(
        seeds = [
            GOVERNANCE_PROPOSAL_ACTION_V1_SEED,
            governance_proposal.key().as_ref()
        ],
        bump = governance_proposal_action.bump
    )]
    pub governance_proposal_action: Box<Account<'info, GovernanceProposalActionV1>>,

    #[account(
        seeds = [
            UNIVERSAL_GOVERNANCE_DECISION_ADAPTER_V1_SEED,
            governance_proposal.key().as_ref()
        ],
        bump = governance_decision_adapter.bump
    )]
    pub governance_decision_adapter: Box<Account<'info, UniversalGovernanceDecisionAdapterV1>>,

    #[account(
        seeds = [PROPOSAL_DECISION_V1_SEED, &governance_proposal.proposal_id.to_le_bytes()],
        bump = proposal_decision.bump
    )]
    pub proposal_decision: Box<Account<'info, ProposalDecisionV1>>,

    #[account(
        seeds = [EXECUTION_QUEUE_ITEM_V1_SEED, &governance_proposal.proposal_id.to_le_bytes()],
        bump = execution_queue_item.bump
    )]
    pub execution_queue_item: Box<Account<'info, ExecutionQueueItemV1>>,

    #[account(seeds = [VICTIM_RELIEF_CONFIG_V1_SEED], bump = victim_relief_config.bump)]
    pub victim_relief_config: Box<Account<'info, VictimReliefConfigV1>>,

    #[account(
        mut,
        seeds = [
            VICTIM_RELIEF_CASE_V1_SEED,
            victim_relief_config.key().as_ref(),
            &victim_relief_case.case_id.to_le_bytes()
        ],
        bump = victim_relief_case.bump
    )]
    pub victim_relief_case: Box<Account<'info, VictimReliefCaseV1>>,

    #[account(
        seeds = [
            VICTIM_RELIEF_POLICY_V1_SEED,
            victim_relief_config.key().as_ref(),
            &victim_relief_case.policy_version.to_le_bytes()
        ],
        bump = victim_relief_policy.bump
    )]
    pub victim_relief_policy: Box<Account<'info, VictimReliefPolicyV1>>,

    #[account(
        mut,
        seeds = [
            VICTIM_RELIEF_CLAIMANT_STATE_V1_SEED,
            victim_relief_config.key().as_ref(),
            victim_relief_case.claimant.as_ref()
        ],
        bump = claimant_state.bump
    )]
    pub claimant_state: Box<Account<'info, VictimReliefClaimantStateV1>>,

    #[account(
        seeds = [
            VICTIM_RELIEF_EVIDENCE_SNAPSHOT_V1_SEED,
            victim_relief_case.key().as_ref()
        ],
        bump = evidence_snapshot.bump
    )]
    pub evidence_snapshot: Box<Account<'info, VictimReliefEvidenceSnapshotV1>>,

    #[account(seeds = [TREASURY_CONFIG_V2_SEED], bump = treasury_config.bump)]
    pub treasury_config: Box<Account<'info, TreasuryConfigV2>>,

    #[account(seeds = [VAULT_AUTHORITY_V2_SEED], bump)]
    /// CHECK: PDA-only vault authority.
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        seeds = [RELIEF_USDC_VAULT_SEED],
        bump,
        constraint = relief_usdc_vault.mint == treasury_config.usdc_mint @ CustomError::InvalidMint,
        constraint = relief_usdc_vault.owner == vault_authority.key() @ CustomError::VictimReliefReliefVaultMismatch
    )]
    pub relief_usdc_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = usdc_mint.key() == treasury_config.usdc_mint @ CustomError::InvalidMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = executor,
        space = 8 + VictimReliefDecisionExecutionRecordV1::INIT_SPACE,
        seeds = [
            VICTIM_RELIEF_DECISION_EXECUTION_RECORD_V1_SEED,
            execution_queue_item.key().as_ref()
        ],
        bump
    )]
    pub decision_execution_record: Box<Account<'info, VictimReliefDecisionExecutionRecordV1>>,

    #[account(mut)]
    pub executor: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct OpenVictimReliefAppealV1<'info> {
    #[account(seeds = [VICTIM_RELIEF_CONFIG_V1_SEED], bump = victim_relief_config.bump)]
    pub victim_relief_config: Box<Account<'info, VictimReliefConfigV1>>,

    #[account(
        seeds = [
            VICTIM_RELIEF_POLICY_V1_SEED,
            victim_relief_config.key().as_ref(),
            &victim_relief_case.policy_version.to_le_bytes()
        ],
        bump = victim_relief_policy.bump
    )]
    pub victim_relief_policy: Box<Account<'info, VictimReliefPolicyV1>>,

    #[account(
        mut,
        seeds = [
            VICTIM_RELIEF_CASE_V1_SEED,
            victim_relief_config.key().as_ref(),
            &victim_relief_case.case_id.to_le_bytes()
        ],
        bump = victim_relief_case.bump
    )]
    pub victim_relief_case: Box<Account<'info, VictimReliefCaseV1>>,

    #[account(
        seeds = [
            VICTIM_RELIEF_EVIDENCE_SNAPSHOT_V1_SEED,
            victim_relief_case.key().as_ref()
        ],
        bump = original_evidence_snapshot.bump
    )]
    pub original_evidence_snapshot: Box<Account<'info, VictimReliefEvidenceSnapshotV1>>,

    #[account(
        seeds = [
            VICTIM_RELIEF_DECISION_EXECUTION_RECORD_V1_SEED,
            victim_relief_case.decision_queue.as_ref()
        ],
        bump = original_decision_record.bump
    )]
    pub original_decision_record: Box<Account<'info, VictimReliefDecisionExecutionRecordV1>>,

    #[account(
        init,
        payer = claimant,
        space = 8 + VictimReliefAppealV1::INIT_SPACE,
        seeds = [
            VICTIM_RELIEF_APPEAL_V1_SEED,
            victim_relief_case.key().as_ref()
        ],
        bump
    )]
    pub victim_relief_appeal: Box<Account<'info, VictimReliefAppealV1>>,

    #[account(mut)]
    pub claimant: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteUpholdVictimReliefAppealV1<'info> {
    #[account(seeds = [GOVERNANCE_CONFIG_V1_SEED], bump = security_governance_config.bump)]
    pub security_governance_config: Box<Account<'info, GovernanceConfigV1>>,

    #[account(
        seeds = [
            PROTOCOL_MODULE_REGISTRY_V1_SEED,
            &[protocol_module_stable_code_v1(ProtocolModuleIdV1::VictimRelief)]
        ],
        bump = protocol_module_registry.bump
    )]
    pub protocol_module_registry: Box<Account<'info, ProtocolModuleRegistryV1>>,

    #[account(
        seeds = [GOVERNANCE_PROPOSAL_V1_SEED, &governance_proposal.proposal_id.to_le_bytes()],
        bump = governance_proposal.bump
    )]
    pub governance_proposal: Box<Account<'info, GovernanceProposalV1>>,

    #[account(
        seeds = [
            GOVERNANCE_PROPOSAL_ACTION_V1_SEED,
            governance_proposal.key().as_ref()
        ],
        bump = governance_proposal_action.bump
    )]
    pub governance_proposal_action: Box<Account<'info, GovernanceProposalActionV1>>,

    #[account(
        seeds = [
            UNIVERSAL_GOVERNANCE_DECISION_ADAPTER_V1_SEED,
            governance_proposal.key().as_ref()
        ],
        bump = governance_decision_adapter.bump
    )]
    pub governance_decision_adapter: Box<Account<'info, UniversalGovernanceDecisionAdapterV1>>,

    #[account(
        seeds = [PROPOSAL_DECISION_V1_SEED, &governance_proposal.proposal_id.to_le_bytes()],
        bump = proposal_decision.bump
    )]
    pub proposal_decision: Box<Account<'info, ProposalDecisionV1>>,

    #[account(
        seeds = [EXECUTION_QUEUE_ITEM_V1_SEED, &governance_proposal.proposal_id.to_le_bytes()],
        bump = execution_queue_item.bump
    )]
    pub execution_queue_item: Box<Account<'info, ExecutionQueueItemV1>>,

    #[account(seeds = [VICTIM_RELIEF_CONFIG_V1_SEED], bump = victim_relief_config.bump)]
    pub victim_relief_config: Box<Account<'info, VictimReliefConfigV1>>,

    #[account(
        mut,
        seeds = [
            VICTIM_RELIEF_CASE_V1_SEED,
            victim_relief_config.key().as_ref(),
            &victim_relief_case.case_id.to_le_bytes()
        ],
        bump = victim_relief_case.bump
    )]
    pub victim_relief_case: Box<Account<'info, VictimReliefCaseV1>>,

    #[account(
        mut,
        seeds = [
            VICTIM_RELIEF_APPEAL_V1_SEED,
            victim_relief_case.key().as_ref()
        ],
        bump = victim_relief_appeal.bump
    )]
    pub victim_relief_appeal: Box<Account<'info, VictimReliefAppealV1>>,

    #[account(
        seeds = [
            VICTIM_RELIEF_POLICY_V1_SEED,
            victim_relief_config.key().as_ref(),
            &victim_relief_case.policy_version.to_le_bytes()
        ],
        bump = victim_relief_policy.bump
    )]
    pub victim_relief_policy: Box<Account<'info, VictimReliefPolicyV1>>,

    #[account(
        seeds = [
            VICTIM_RELIEF_EVIDENCE_SNAPSHOT_V1_SEED,
            victim_relief_case.key().as_ref()
        ],
        bump = original_evidence_snapshot.bump
    )]
    pub original_evidence_snapshot: Box<Account<'info, VictimReliefEvidenceSnapshotV1>>,

    #[account(
        seeds = [
            VICTIM_RELIEF_DECISION_EXECUTION_RECORD_V1_SEED,
            victim_relief_appeal.original_execution_queue_item.as_ref()
        ],
        bump = original_decision_record.bump
    )]
    pub original_decision_record: Box<Account<'info, VictimReliefDecisionExecutionRecordV1>>,

    #[account(seeds = [TREASURY_CONFIG_V2_SEED], bump = treasury_config.bump)]
    pub treasury_config: Box<Account<'info, TreasuryConfigV2>>,

    #[account(seeds = [VAULT_AUTHORITY_V2_SEED], bump)]
    /// CHECK: PDA-only vault authority.
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        seeds = [RELIEF_USDC_VAULT_SEED],
        bump,
        constraint = relief_usdc_vault.mint == treasury_config.usdc_mint @ CustomError::InvalidMint,
        constraint = relief_usdc_vault.owner == vault_authority.key() @ CustomError::VictimReliefReliefVaultMismatch
    )]
    pub relief_usdc_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = usdc_mint.key() == treasury_config.usdc_mint @ CustomError::InvalidMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = executor,
        space = 8 + VictimReliefAppealDecisionExecutionRecordV1::INIT_SPACE,
        seeds = [
            VICTIM_RELIEF_APPEAL_DECISION_EXECUTION_RECORD_V1_SEED,
            execution_queue_item.key().as_ref()
        ],
        bump
    )]
    pub appeal_decision_execution_record:
        Box<Account<'info, VictimReliefAppealDecisionExecutionRecordV1>>,

    #[account(mut)]
    pub executor: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteOverturnVictimReliefAppealV1<'info> {
    #[account(seeds = [GOVERNANCE_CONFIG_V1_SEED], bump = security_governance_config.bump)]
    pub security_governance_config: Box<Account<'info, GovernanceConfigV1>>,

    #[account(
        seeds = [
            PROTOCOL_MODULE_REGISTRY_V1_SEED,
            &[protocol_module_stable_code_v1(ProtocolModuleIdV1::VictimRelief)]
        ],
        bump = protocol_module_registry.bump
    )]
    pub protocol_module_registry: Box<Account<'info, ProtocolModuleRegistryV1>>,

    #[account(
        seeds = [GOVERNANCE_PROPOSAL_V1_SEED, &governance_proposal.proposal_id.to_le_bytes()],
        bump = governance_proposal.bump
    )]
    pub governance_proposal: Box<Account<'info, GovernanceProposalV1>>,

    #[account(
        seeds = [
            GOVERNANCE_PROPOSAL_ACTION_V1_SEED,
            governance_proposal.key().as_ref()
        ],
        bump = governance_proposal_action.bump
    )]
    pub governance_proposal_action: Box<Account<'info, GovernanceProposalActionV1>>,

    #[account(
        seeds = [
            UNIVERSAL_GOVERNANCE_DECISION_ADAPTER_V1_SEED,
            governance_proposal.key().as_ref()
        ],
        bump = governance_decision_adapter.bump
    )]
    pub governance_decision_adapter: Box<Account<'info, UniversalGovernanceDecisionAdapterV1>>,

    #[account(
        seeds = [PROPOSAL_DECISION_V1_SEED, &governance_proposal.proposal_id.to_le_bytes()],
        bump = proposal_decision.bump
    )]
    pub proposal_decision: Box<Account<'info, ProposalDecisionV1>>,

    #[account(
        seeds = [EXECUTION_QUEUE_ITEM_V1_SEED, &governance_proposal.proposal_id.to_le_bytes()],
        bump = execution_queue_item.bump
    )]
    pub execution_queue_item: Box<Account<'info, ExecutionQueueItemV1>>,

    #[account(seeds = [VICTIM_RELIEF_CONFIG_V1_SEED], bump = victim_relief_config.bump)]
    pub victim_relief_config: Box<Account<'info, VictimReliefConfigV1>>,

    #[account(
        mut,
        seeds = [
            VICTIM_RELIEF_CASE_V1_SEED,
            victim_relief_config.key().as_ref(),
            &victim_relief_case.case_id.to_le_bytes()
        ],
        bump = victim_relief_case.bump
    )]
    pub victim_relief_case: Box<Account<'info, VictimReliefCaseV1>>,

    #[account(
        mut,
        seeds = [
            VICTIM_RELIEF_APPEAL_V1_SEED,
            victim_relief_case.key().as_ref()
        ],
        bump = victim_relief_appeal.bump
    )]
    pub victim_relief_appeal: Box<Account<'info, VictimReliefAppealV1>>,

    #[account(
        seeds = [
            VICTIM_RELIEF_POLICY_V1_SEED,
            victim_relief_config.key().as_ref(),
            &victim_relief_case.policy_version.to_le_bytes()
        ],
        bump = victim_relief_policy.bump
    )]
    pub victim_relief_policy: Box<Account<'info, VictimReliefPolicyV1>>,

    #[account(
        mut,
        seeds = [
            VICTIM_RELIEF_CLAIMANT_STATE_V1_SEED,
            victim_relief_config.key().as_ref(),
            victim_relief_case.claimant.as_ref()
        ],
        bump = claimant_state.bump
    )]
    pub claimant_state: Box<Account<'info, VictimReliefClaimantStateV1>>,

    #[account(
        seeds = [
            VICTIM_RELIEF_EVIDENCE_SNAPSHOT_V1_SEED,
            victim_relief_case.key().as_ref()
        ],
        bump = original_evidence_snapshot.bump
    )]
    pub original_evidence_snapshot: Box<Account<'info, VictimReliefEvidenceSnapshotV1>>,

    #[account(
        seeds = [
            VICTIM_RELIEF_DECISION_EXECUTION_RECORD_V1_SEED,
            victim_relief_appeal.original_execution_queue_item.as_ref()
        ],
        bump = original_decision_record.bump
    )]
    pub original_decision_record: Box<Account<'info, VictimReliefDecisionExecutionRecordV1>>,

    #[account(seeds = [TREASURY_CONFIG_V2_SEED], bump = treasury_config.bump)]
    pub treasury_config: Box<Account<'info, TreasuryConfigV2>>,

    #[account(seeds = [VAULT_AUTHORITY_V2_SEED], bump)]
    /// CHECK: PDA-only vault authority.
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        seeds = [RELIEF_USDC_VAULT_SEED],
        bump,
        constraint = relief_usdc_vault.mint == treasury_config.usdc_mint @ CustomError::InvalidMint,
        constraint = relief_usdc_vault.owner == vault_authority.key() @ CustomError::VictimReliefReliefVaultMismatch
    )]
    pub relief_usdc_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = usdc_mint.key() == treasury_config.usdc_mint @ CustomError::InvalidMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = executor,
        space = 8 + ReliefPayoutRequestV1::INIT_SPACE,
        seeds = [
            RELIEF_PAYOUT_REQUEST_V1_SEED,
            victim_relief_case.key().as_ref()
        ],
        bump
    )]
    pub relief_payout_request: Box<Account<'info, ReliefPayoutRequestV1>>,

    #[account(
        init,
        payer = executor,
        space = 8 + VictimReliefAppealDecisionExecutionRecordV1::INIT_SPACE,
        seeds = [
            VICTIM_RELIEF_APPEAL_DECISION_EXECUTION_RECORD_V1_SEED,
            execution_queue_item.key().as_ref()
        ],
        bump
    )]
    pub appeal_decision_execution_record:
        Box<Account<'info, VictimReliefAppealDecisionExecutionRecordV1>>,

    #[account(mut)]
    pub executor: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteVictimReliefApprovedPayoutV1<'info> {
    #[account(seeds = [GOVERNANCE_CONFIG_V1_SEED], bump = security_governance_config.bump)]
    pub security_governance_config: Box<Account<'info, GovernanceConfigV1>>,

    #[account(
        seeds = [GOVERNANCE_PROPOSAL_V1_SEED, &governance_proposal.proposal_id.to_le_bytes()],
        bump = governance_proposal.bump
    )]
    pub governance_proposal: Box<Account<'info, GovernanceProposalV1>>,

    #[account(
        seeds = [
            GOVERNANCE_PROPOSAL_ACTION_V1_SEED,
            governance_proposal.key().as_ref()
        ],
        bump = governance_proposal_action.bump
    )]
    pub governance_proposal_action: Box<Account<'info, GovernanceProposalActionV1>>,

    #[account(
        seeds = [PROPOSAL_DECISION_V1_SEED, &governance_proposal.proposal_id.to_le_bytes()],
        bump = proposal_decision.bump
    )]
    pub proposal_decision: Box<Account<'info, ProposalDecisionV1>>,

    #[account(
        seeds = [EXECUTION_QUEUE_ITEM_V1_SEED, &governance_proposal.proposal_id.to_le_bytes()],
        bump = execution_queue_item.bump
    )]
    pub execution_queue_item: Box<Account<'info, ExecutionQueueItemV1>>,

    #[account(seeds = [VICTIM_RELIEF_CONFIG_V1_SEED], bump = victim_relief_config.bump)]
    pub victim_relief_config: Box<Account<'info, VictimReliefConfigV1>>,

    #[account(
        seeds = [
            VICTIM_RELIEF_POLICY_V1_SEED,
            victim_relief_config.key().as_ref(),
            &victim_relief_case.policy_version.to_le_bytes()
        ],
        bump = victim_relief_policy.bump
    )]
    pub victim_relief_policy: Box<Account<'info, VictimReliefPolicyV1>>,

    #[account(
        mut,
        seeds = [
            VICTIM_RELIEF_CLAIMANT_STATE_V1_SEED,
            victim_relief_config.key().as_ref(),
            victim_relief_case.claimant.as_ref()
        ],
        bump = claimant_state.bump
    )]
    pub claimant_state: Box<Account<'info, VictimReliefClaimantStateV1>>,

    #[account(
        mut,
        seeds = [
            VICTIM_RELIEF_CASE_V1_SEED,
            victim_relief_config.key().as_ref(),
            &victim_relief_case.case_id.to_le_bytes()
        ],
        bump = victim_relief_case.bump
    )]
    pub victim_relief_case: Box<Account<'info, VictimReliefCaseV1>>,

    #[account(
        seeds = [
            VICTIM_RELIEF_EVIDENCE_SNAPSHOT_V1_SEED,
            victim_relief_case.key().as_ref()
        ],
        bump = evidence_snapshot.bump
    )]
    pub evidence_snapshot: Box<Account<'info, VictimReliefEvidenceSnapshotV1>>,

    #[account(
        mut,
        seeds = [
            RELIEF_PAYOUT_REQUEST_V1_SEED,
            victim_relief_case.key().as_ref()
        ],
        bump = relief_payout_request.bump
    )]
    pub relief_payout_request: Box<Account<'info, ReliefPayoutRequestV1>>,

    #[account(
        seeds = [
            VICTIM_RELIEF_DECISION_EXECUTION_RECORD_V1_SEED,
            execution_queue_item.key().as_ref()
        ],
        bump = decision_execution_record.bump
    )]
    pub decision_execution_record: Box<Account<'info, VictimReliefDecisionExecutionRecordV1>>,

    #[account(seeds = [TREASURY_CONFIG_V2_SEED], bump = treasury_config.bump)]
    pub treasury_config: Box<Account<'info, TreasuryConfigV2>>,

    #[account(seeds = [VAULT_AUTHORITY_V2_SEED], bump)]
    /// CHECK: PDA-only vault authority.
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [RELIEF_USDC_VAULT_SEED],
        bump,
        constraint = relief_usdc_vault.mint == treasury_config.usdc_mint @ CustomError::InvalidMint,
        constraint = relief_usdc_vault.owner == vault_authority.key() @ CustomError::VictimReliefReliefVaultMismatch
    )]
    pub relief_usdc_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = recipient_usdc_token_account.key() == relief_payout_request.recipient_token_account @ CustomError::VictimReliefPayoutRecipientMismatch,
        constraint = recipient_usdc_token_account.owner == relief_payout_request.recipient_owner @ CustomError::VictimReliefPayoutRecipientMismatch,
        constraint = recipient_usdc_token_account.mint == treasury_config.usdc_mint @ CustomError::VictimReliefPayoutRecipientMismatch
    )]
    pub recipient_usdc_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = usdc_mint.key() == treasury_config.usdc_mint @ CustomError::InvalidMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = executor,
        space = 8 + ReliefPayoutExecutionRecordV1::INIT_SPACE,
        seeds = [
            RELIEF_PAYOUT_EXECUTION_RECORD_V1_SEED,
            relief_payout_request.key().as_ref()
        ],
        bump
    )]
    pub relief_payout_execution_record: Box<Account<'info, ReliefPayoutExecutionRecordV1>>,

    #[account(mut)]
    pub executor: Signer<'info>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_victim_relief_config_v1_handler(
    ctx: Context<InitializeVictimReliefConfigV1>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;

    require!(
        ctx.accounts.bootstrap_authority.key() == ctx.accounts.security_governance_config.authority,
        CustomError::UnauthorizedSecurityAuthority
    );
    require!(
        ctx.accounts.treasury_config.usdc_mint == ctx.accounts.usdc_mint.key(),
        CustomError::InvalidMint
    );

    record_victim_relief_config_init_v1(
        &mut ctx.accounts.victim_relief_config,
        ctx.accounts.security_governance_config.authority,
        ctx.accounts.treasury_config.key(),
        ctx.accounts.security_governance_config.key(),
        ctx.accounts.usdc_mint.key(),
        now,
        ctx.bumps.victim_relief_config,
    )
}

pub fn initialize_victim_relief_policy_v1_handler(
    ctx: Context<InitializeVictimReliefPolicyV1>,
    min_claim_amount_usdc: u64,
    max_claim_amount_usdc: u64,
    max_payout_per_case_usdc: u64,
    evidence_window_seconds: i64,
    review_window_seconds: i64,
    appeal_window_seconds: i64,
    submission_cooldown_seconds: i64,
    max_evidence_items: u32,
    max_active_cases_per_claimant: u16,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let config_key = ctx.accounts.victim_relief_config.key();
    let policy_key = ctx.accounts.victim_relief_policy.key();
    let parameters = VictimReliefPolicyParametersV1 {
        min_claim_amount_usdc,
        max_claim_amount_usdc,
        max_payout_per_case_usdc,
        evidence_window_seconds,
        review_window_seconds,
        appeal_window_seconds,
        submission_cooldown_seconds,
        max_evidence_items,
        max_active_cases_per_claimant,
    };

    record_victim_relief_policy_init_v1(
        &mut ctx.accounts.victim_relief_config,
        &mut ctx.accounts.victim_relief_policy,
        config_key,
        policy_key,
        ctx.accounts.authority.key(),
        parameters,
        now,
        ctx.bumps.victim_relief_policy,
    )
}

pub fn submit_victim_relief_case_v1_handler(
    ctx: Context<SubmitVictimReliefCaseV1>,
    case_id: u64,
    subject_commitment: [u8; 32],
    evidence_root: [u8; 32],
    evidence_count: u32,
    claimed_amount_usdc: u64,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let config_key = ctx.accounts.victim_relief_config.key();
    let policy_key = ctx.accounts.victim_relief_policy.key();
    let claimant_key = ctx.accounts.claimant.key();
    let recipient_key = ctx.accounts.claimant_recipient_usdc_account.key();
    let recipient_owner = ctx.accounts.claimant_recipient_usdc_account.owner;
    let recipient_mint = ctx.accounts.claimant_recipient_usdc_account.mint;
    let usdc_mint = ctx.accounts.usdc_mint.key();

    ensure_victim_relief_claimant_state_v1(
        &mut ctx.accounts.claimant_state,
        config_key,
        claimant_key,
        now,
        ctx.bumps.claimant_state,
    )?;

    validate_victim_relief_case_submission_v1(
        &ctx.accounts.victim_relief_config,
        &ctx.accounts.victim_relief_policy,
        &ctx.accounts.claimant_state,
        policy_key,
        case_id,
        subject_commitment,
        evidence_root,
        evidence_count,
        claimed_amount_usdc,
        recipient_owner,
        recipient_mint,
        usdc_mint,
        now,
    )?;

    record_victim_relief_case_submission_v1(
        &mut ctx.accounts.victim_relief_config,
        &ctx.accounts.victim_relief_policy,
        &mut ctx.accounts.claimant_state,
        &mut ctx.accounts.victim_relief_case,
        config_key,
        policy_key,
        claimant_key,
        recipient_key,
        subject_commitment,
        evidence_root,
        evidence_count,
        claimed_amount_usdc,
        usdc_mint,
        now,
        ctx.bumps.victim_relief_case,
    )
}

pub fn update_victim_relief_evidence_root_v1_handler(
    ctx: Context<UpdateVictimReliefEvidenceRootV1>,
    new_evidence_root: [u8; 32],
    new_evidence_count: u32,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    validate_victim_relief_evidence_update_v1(
        &ctx.accounts.victim_relief_config,
        &ctx.accounts.victim_relief_policy,
        &ctx.accounts.victim_relief_case,
        ctx.accounts.victim_relief_config.key(),
        ctx.accounts.victim_relief_policy.key(),
        ctx.accounts.claimant.key(),
        new_evidence_root,
        new_evidence_count,
        now,
    )?;
    record_victim_relief_evidence_update_v1(
        &mut ctx.accounts.victim_relief_case,
        new_evidence_root,
        new_evidence_count,
        now,
    )
}

pub fn cancel_victim_relief_case_v1_handler(ctx: Context<CancelVictimReliefCaseV1>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    validate_victim_relief_case_claimant_link_v1(
        &ctx.accounts.victim_relief_config,
        &ctx.accounts.claimant_state,
        &ctx.accounts.victim_relief_case,
        ctx.accounts.victim_relief_config.key(),
        ctx.accounts.claimant.key(),
    )?;
    require!(
        ctx.accounts.victim_relief_case.status == VictimReliefCaseStatusV1::EvidencePeriod,
        CustomError::VictimReliefCaseStatusMismatch
    );
    record_victim_relief_case_terminal_status_v1(
        &mut ctx.accounts.claimant_state,
        &mut ctx.accounts.victim_relief_case,
        VictimReliefCaseStatusV1::Cancelled,
        now,
    )
}

pub fn expire_victim_relief_case_v1_handler(ctx: Context<ExpireVictimReliefCaseV1>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    validate_victim_relief_case_claimant_link_v1(
        &ctx.accounts.victim_relief_config,
        &ctx.accounts.claimant_state,
        &ctx.accounts.victim_relief_case,
        ctx.accounts.victim_relief_config.key(),
        ctx.accounts.victim_relief_case.claimant,
    )?;
    require!(
        ctx.accounts.victim_relief_case.status == VictimReliefCaseStatusV1::EvidencePeriod,
        CustomError::VictimReliefCaseStatusMismatch
    );
    require!(
        now > ctx.accounts.victim_relief_case.evidence_deadline,
        CustomError::VictimReliefCaseNotExpired
    );
    record_victim_relief_case_terminal_status_v1(
        &mut ctx.accounts.claimant_state,
        &mut ctx.accounts.victim_relief_case,
        VictimReliefCaseStatusV1::Expired,
        now,
    )
}

pub fn freeze_victim_relief_evidence_v1_handler(
    ctx: Context<FreezeVictimReliefEvidenceV1>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let config_key = ctx.accounts.victim_relief_config.key();
    let policy_key = ctx.accounts.victim_relief_policy.key();
    let case_key = ctx.accounts.victim_relief_case.key();
    let claimant_key = ctx.accounts.claimant.key();
    let review_deadline = validate_victim_relief_evidence_freeze_v1(
        &ctx.accounts.victim_relief_config,
        &ctx.accounts.victim_relief_policy,
        &ctx.accounts.victim_relief_case,
        config_key,
        policy_key,
        claimant_key,
        now,
    )?;

    record_victim_relief_evidence_snapshot_v1(
        &mut ctx.accounts.evidence_snapshot,
        &mut ctx.accounts.victim_relief_case,
        case_key,
        config_key,
        policy_key,
        now,
        review_deadline,
        ctx.bumps.evidence_snapshot,
    )
}

pub fn execute_approve_victim_relief_case_v1_handler(
    ctx: Context<ExecuteApproveVictimReliefCaseV1>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let case_key = ctx.accounts.victim_relief_case.key();
    let config_key = ctx.accounts.victim_relief_config.key();
    let policy_key = ctx.accounts.victim_relief_policy.key();
    let snapshot_key = ctx.accounts.evidence_snapshot.key();
    let proposal_key = ctx.accounts.governance_proposal.key();
    let proposal_action_key = ctx.accounts.governance_proposal_action.key();
    let proposal_decision_key = ctx.accounts.proposal_decision.key();
    let queue_key = ctx.accounts.execution_queue_item.key();
    let module_registry_key = ctx.accounts.protocol_module_registry.key();
    let security_governance_config_key = ctx.accounts.security_governance_config.key();
    let treasury_config_key = ctx.accounts.treasury_config.key();
    let relief_vault_key = ctx.accounts.relief_usdc_vault.key();
    let executor_key = ctx.accounts.executor.key();

    let validation = validate_victim_relief_decision_execution_context_v1(
        &ctx.accounts.security_governance_config,
        security_governance_config_key,
        &ctx.accounts.protocol_module_registry,
        module_registry_key,
        &ctx.accounts.governance_proposal,
        proposal_key,
        &ctx.accounts.governance_proposal_action,
        proposal_action_key,
        &ctx.accounts.governance_decision_adapter,
        &ctx.accounts.proposal_decision,
        proposal_decision_key,
        &ctx.accounts.execution_queue_item,
        queue_key,
        &ctx.accounts.victim_relief_config,
        config_key,
        &ctx.accounts.victim_relief_policy,
        policy_key,
        &ctx.accounts.victim_relief_case,
        case_key,
        &ctx.accounts.claimant_state,
        &ctx.accounts.evidence_snapshot,
        snapshot_key,
        &ctx.accounts.treasury_config,
        treasury_config_key,
        relief_vault_key,
        &ctx.accounts.relief_usdc_vault,
        ctx.accounts.vault_authority.key(),
        ctx.accounts.usdc_mint.key(),
        GovernanceActionTypeV1::VictimReliefApproveCompensation,
    )?;

    record_approve_victim_relief_decision_v1(
        &mut ctx.accounts.victim_relief_case,
        &mut ctx.accounts.relief_payout_request,
        &mut ctx.accounts.decision_execution_record,
        case_key,
        config_key,
        policy_key,
        proposal_key,
        proposal_action_key,
        proposal_decision_key,
        queue_key,
        module_registry_key,
        snapshot_key,
        treasury_config_key,
        relief_vault_key,
        executor_key,
        validation,
        now,
        ctx.bumps.relief_payout_request,
        ctx.bumps.decision_execution_record,
    )
}

pub fn execute_reject_victim_relief_case_v1_handler(
    ctx: Context<ExecuteRejectVictimReliefCaseV1>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let case_key = ctx.accounts.victim_relief_case.key();
    let config_key = ctx.accounts.victim_relief_config.key();
    let policy_key = ctx.accounts.victim_relief_policy.key();
    let snapshot_key = ctx.accounts.evidence_snapshot.key();
    let proposal_key = ctx.accounts.governance_proposal.key();
    let proposal_action_key = ctx.accounts.governance_proposal_action.key();
    let proposal_decision_key = ctx.accounts.proposal_decision.key();
    let queue_key = ctx.accounts.execution_queue_item.key();
    let module_registry_key = ctx.accounts.protocol_module_registry.key();
    let security_governance_config_key = ctx.accounts.security_governance_config.key();
    let treasury_config_key = ctx.accounts.treasury_config.key();
    let relief_vault_key = ctx.accounts.relief_usdc_vault.key();
    let executor_key = ctx.accounts.executor.key();

    let validation = validate_victim_relief_decision_execution_context_v1(
        &ctx.accounts.security_governance_config,
        security_governance_config_key,
        &ctx.accounts.protocol_module_registry,
        module_registry_key,
        &ctx.accounts.governance_proposal,
        proposal_key,
        &ctx.accounts.governance_proposal_action,
        proposal_action_key,
        &ctx.accounts.governance_decision_adapter,
        &ctx.accounts.proposal_decision,
        proposal_decision_key,
        &ctx.accounts.execution_queue_item,
        queue_key,
        &ctx.accounts.victim_relief_config,
        config_key,
        &ctx.accounts.victim_relief_policy,
        policy_key,
        &ctx.accounts.victim_relief_case,
        case_key,
        &ctx.accounts.claimant_state,
        &ctx.accounts.evidence_snapshot,
        snapshot_key,
        &ctx.accounts.treasury_config,
        treasury_config_key,
        relief_vault_key,
        &ctx.accounts.relief_usdc_vault,
        ctx.accounts.vault_authority.key(),
        ctx.accounts.usdc_mint.key(),
        GovernanceActionTypeV1::VictimReliefRejectClaim,
    )?;

    record_reject_victim_relief_decision_v1(
        &mut ctx.accounts.claimant_state,
        &mut ctx.accounts.victim_relief_case,
        &mut ctx.accounts.decision_execution_record,
        config_key,
        policy_key,
        proposal_key,
        proposal_action_key,
        proposal_decision_key,
        queue_key,
        module_registry_key,
        case_key,
        snapshot_key,
        executor_key,
        ctx.accounts.victim_relief_policy.appeal_window_seconds,
        validation,
        now,
        ctx.bumps.decision_execution_record,
    )
}

pub fn open_victim_relief_appeal_v1_handler(
    ctx: Context<OpenVictimReliefAppealV1>,
    appeal_evidence_root: [u8; 32],
    appeal_evidence_count: u32,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let config_key = ctx.accounts.victim_relief_config.key();
    let policy_key = ctx.accounts.victim_relief_policy.key();
    let case_key = ctx.accounts.victim_relief_case.key();
    let snapshot_key = ctx.accounts.original_evidence_snapshot.key();
    let record_key = ctx.accounts.original_decision_record.key();
    let appeal_key = ctx.accounts.victim_relief_appeal.key();
    let claimant_key = ctx.accounts.claimant.key();

    validate_open_victim_relief_appeal_v1(
        &ctx.accounts.victim_relief_config,
        &ctx.accounts.victim_relief_policy,
        &ctx.accounts.victim_relief_case,
        case_key,
        &ctx.accounts.original_evidence_snapshot,
        snapshot_key,
        &ctx.accounts.original_decision_record,
        record_key,
        config_key,
        policy_key,
        claimant_key,
        appeal_evidence_root,
        appeal_evidence_count,
        now,
    )?;

    record_open_victim_relief_appeal_v1(
        &mut ctx.accounts.victim_relief_appeal,
        &mut ctx.accounts.victim_relief_case,
        appeal_key,
        case_key,
        config_key,
        policy_key,
        snapshot_key,
        record_key,
        appeal_evidence_root,
        appeal_evidence_count,
        now,
        ctx.bumps.victim_relief_appeal,
    )
}

pub fn execute_uphold_victim_relief_appeal_v1_handler(
    ctx: Context<ExecuteUpholdVictimReliefAppealV1>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let case_key = ctx.accounts.victim_relief_case.key();
    let appeal_key = ctx.accounts.victim_relief_appeal.key();
    let config_key = ctx.accounts.victim_relief_config.key();
    let policy_key = ctx.accounts.victim_relief_policy.key();
    let snapshot_key = ctx.accounts.original_evidence_snapshot.key();
    let original_record_key = ctx.accounts.original_decision_record.key();
    let proposal_key = ctx.accounts.governance_proposal.key();
    let proposal_action_key = ctx.accounts.governance_proposal_action.key();
    let proposal_decision_key = ctx.accounts.proposal_decision.key();
    let queue_key = ctx.accounts.execution_queue_item.key();
    let module_registry_key = ctx.accounts.protocol_module_registry.key();
    let security_governance_config_key = ctx.accounts.security_governance_config.key();
    let treasury_config_key = ctx.accounts.treasury_config.key();
    let relief_vault_key = ctx.accounts.relief_usdc_vault.key();
    let executor_key = ctx.accounts.executor.key();

    let validation = validate_victim_relief_appeal_execution_context_v1(
        &ctx.accounts.security_governance_config,
        security_governance_config_key,
        &ctx.accounts.protocol_module_registry,
        module_registry_key,
        &ctx.accounts.governance_proposal,
        proposal_key,
        &ctx.accounts.governance_proposal_action,
        proposal_action_key,
        &ctx.accounts.governance_decision_adapter,
        &ctx.accounts.proposal_decision,
        proposal_decision_key,
        &ctx.accounts.execution_queue_item,
        queue_key,
        &ctx.accounts.victim_relief_config,
        config_key,
        &ctx.accounts.victim_relief_policy,
        policy_key,
        &ctx.accounts.victim_relief_case,
        case_key,
        &ctx.accounts.victim_relief_appeal,
        appeal_key,
        &ctx.accounts.original_evidence_snapshot,
        snapshot_key,
        &ctx.accounts.original_decision_record,
        original_record_key,
        &ctx.accounts.treasury_config,
        treasury_config_key,
        relief_vault_key,
        &ctx.accounts.relief_usdc_vault,
        ctx.accounts.vault_authority.key(),
        ctx.accounts.usdc_mint.key(),
        GovernanceActionTypeV1::VictimReliefUpholdAppeal,
    )?;

    record_uphold_victim_relief_appeal_v1(
        &mut ctx.accounts.victim_relief_case,
        &mut ctx.accounts.victim_relief_appeal,
        &mut ctx.accounts.appeal_decision_execution_record,
        config_key,
        policy_key,
        proposal_key,
        proposal_action_key,
        proposal_decision_key,
        queue_key,
        module_registry_key,
        case_key,
        appeal_key,
        snapshot_key,
        original_record_key,
        executor_key,
        validation,
        now,
        ctx.bumps.appeal_decision_execution_record,
    )
}

pub fn execute_overturn_victim_relief_appeal_v1_handler(
    ctx: Context<ExecuteOverturnVictimReliefAppealV1>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let case_key = ctx.accounts.victim_relief_case.key();
    let appeal_key = ctx.accounts.victim_relief_appeal.key();
    let config_key = ctx.accounts.victim_relief_config.key();
    let policy_key = ctx.accounts.victim_relief_policy.key();
    let snapshot_key = ctx.accounts.original_evidence_snapshot.key();
    let original_record_key = ctx.accounts.original_decision_record.key();
    let proposal_key = ctx.accounts.governance_proposal.key();
    let proposal_action_key = ctx.accounts.governance_proposal_action.key();
    let proposal_decision_key = ctx.accounts.proposal_decision.key();
    let queue_key = ctx.accounts.execution_queue_item.key();
    let module_registry_key = ctx.accounts.protocol_module_registry.key();
    let security_governance_config_key = ctx.accounts.security_governance_config.key();
    let treasury_config_key = ctx.accounts.treasury_config.key();
    let relief_vault_key = ctx.accounts.relief_usdc_vault.key();
    let executor_key = ctx.accounts.executor.key();

    require!(
        ctx.accounts.claimant_state.config == config_key
            && ctx.accounts.claimant_state.claimant == ctx.accounts.victim_relief_case.claimant,
        CustomError::VictimReliefClaimantMismatch
    );

    let validation = validate_victim_relief_appeal_execution_context_v1(
        &ctx.accounts.security_governance_config,
        security_governance_config_key,
        &ctx.accounts.protocol_module_registry,
        module_registry_key,
        &ctx.accounts.governance_proposal,
        proposal_key,
        &ctx.accounts.governance_proposal_action,
        proposal_action_key,
        &ctx.accounts.governance_decision_adapter,
        &ctx.accounts.proposal_decision,
        proposal_decision_key,
        &ctx.accounts.execution_queue_item,
        queue_key,
        &ctx.accounts.victim_relief_config,
        config_key,
        &ctx.accounts.victim_relief_policy,
        policy_key,
        &ctx.accounts.victim_relief_case,
        case_key,
        &ctx.accounts.victim_relief_appeal,
        appeal_key,
        &ctx.accounts.original_evidence_snapshot,
        snapshot_key,
        &ctx.accounts.original_decision_record,
        original_record_key,
        &ctx.accounts.treasury_config,
        treasury_config_key,
        relief_vault_key,
        &ctx.accounts.relief_usdc_vault,
        ctx.accounts.vault_authority.key(),
        ctx.accounts.usdc_mint.key(),
        GovernanceActionTypeV1::VictimReliefOverturnAppeal,
    )?;

    record_overturn_victim_relief_appeal_v1(
        &mut ctx.accounts.claimant_state,
        &mut ctx.accounts.victim_relief_case,
        &mut ctx.accounts.victim_relief_appeal,
        &mut ctx.accounts.relief_payout_request,
        &mut ctx.accounts.appeal_decision_execution_record,
        config_key,
        policy_key,
        proposal_key,
        proposal_action_key,
        proposal_decision_key,
        queue_key,
        module_registry_key,
        case_key,
        appeal_key,
        snapshot_key,
        original_record_key,
        treasury_config_key,
        relief_vault_key,
        executor_key,
        validation,
        now,
        ctx.bumps.relief_payout_request,
        ctx.bumps.appeal_decision_execution_record,
    )
}

pub fn execute_victim_relief_approved_payout_v1_handler(
    ctx: Context<ExecuteVictimReliefApprovedPayoutV1>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;

    validate_victim_relief_original_approve_authorization_v1(
        ctx.accounts.governance_proposal.key(),
        &ctx.accounts.governance_proposal,
        ctx.accounts.governance_proposal_action.key(),
        &ctx.accounts.governance_proposal_action,
        ctx.accounts.proposal_decision.key(),
        &ctx.accounts.proposal_decision,
        ctx.accounts.execution_queue_item.key(),
        &ctx.accounts.execution_queue_item,
        ctx.accounts.decision_execution_record.key(),
        &ctx.accounts.decision_execution_record,
        ctx.accounts.evidence_snapshot.key(),
        &ctx.accounts.evidence_snapshot,
        ctx.accounts.victim_relief_case.key(),
        &ctx.accounts.victim_relief_case,
        ctx.accounts.relief_payout_request.key(),
        &ctx.accounts.relief_payout_request,
    )?;

    let payout_parameters = validate_victim_relief_payout_common_v1(
        ctx.accounts.security_governance_config.key(),
        &ctx.accounts.security_governance_config,
        ctx.accounts.victim_relief_config.key(),
        &ctx.accounts.victim_relief_config,
        ctx.accounts.victim_relief_policy.key(),
        &ctx.accounts.victim_relief_policy,
        &ctx.accounts.claimant_state,
        ctx.accounts.victim_relief_case.key(),
        &ctx.accounts.victim_relief_case,
        ctx.accounts.evidence_snapshot.key(),
        &ctx.accounts.evidence_snapshot,
        ctx.accounts.relief_payout_request.key(),
        &ctx.accounts.relief_payout_request,
        ctx.accounts.treasury_config.key(),
        &ctx.accounts.treasury_config,
        ctx.accounts.relief_usdc_vault.key(),
        &ctx.accounts.relief_usdc_vault,
        ctx.accounts.vault_authority.key(),
        ctx.accounts.recipient_usdc_token_account.key(),
        &ctx.accounts.recipient_usdc_token_account,
        ctx.accounts.usdc_mint.key(),
        &ctx.accounts.usdc_mint,
        ctx.accounts.relief_payout_execution_record.key(),
        &ctx.accounts.relief_payout_execution_record,
        VictimReliefPayoutOriginV1::OriginalApprove,
        GovernanceActionTypeV1::VictimReliefApproveCompensation,
        ctx.accounts.governance_proposal_action.key(),
        ctx.accounts.decision_execution_record.key(),
        ctx.accounts.decision_execution_record.approved_amount_usdc,
        ctx.accounts.decision_execution_record.parameters_hash,
    )?;

    let payout_parameters_hash = hash_victim_relief_payout_parameters_v1(&payout_parameters)?;

    transfer_from_relief_usdc_vault_v1(
        ctx.accounts.token_program.key(),
        ctx.accounts.relief_usdc_vault.to_account_info(),
        ctx.accounts.usdc_mint.to_account_info(),
        ctx.accounts.recipient_usdc_token_account.to_account_info(),
        ctx.accounts.vault_authority.to_account_info(),
        ctx.bumps.vault_authority,
        payout_parameters.approved_amount_usdc,
        ctx.accounts.usdc_mint.decimals,
    )?;

    let recorded_hash = record_original_approved_victim_relief_payout_v1(
        &mut ctx.accounts.claimant_state,
        &mut ctx.accounts.victim_relief_case,
        &mut ctx.accounts.relief_payout_request,
        &mut ctx.accounts.relief_payout_execution_record,
        payout_parameters,
        ctx.accounts.executor.key(),
        now,
        ctx.bumps.relief_payout_execution_record,
    )?;
    require!(
        recorded_hash == payout_parameters_hash,
        CustomError::VictimReliefPayoutParametersMismatch
    );
    Ok(())
}

pub fn validate_victim_relief_policy_parameters_v1(
    parameters: VictimReliefPolicyParametersV1,
) -> Result<()> {
    require!(
        parameters.min_claim_amount_usdc > 0,
        CustomError::InvalidVictimReliefPolicy
    );
    require!(
        parameters.max_claim_amount_usdc >= parameters.min_claim_amount_usdc,
        CustomError::InvalidVictimReliefPolicy
    );
    require!(
        parameters.max_payout_per_case_usdc > 0,
        CustomError::InvalidVictimReliefPolicy
    );
    require!(
        parameters.max_payout_per_case_usdc <= parameters.max_claim_amount_usdc,
        CustomError::InvalidVictimReliefPolicy
    );
    require!(
        parameters.evidence_window_seconds > 0
            && parameters.review_window_seconds > 0
            && parameters.appeal_window_seconds > 0,
        CustomError::InvalidVictimReliefPolicy
    );
    require!(
        parameters.submission_cooldown_seconds >= 0,
        CustomError::InvalidVictimReliefPolicy
    );
    require!(
        parameters.max_evidence_items > 0,
        CustomError::InvalidVictimReliefPolicy
    );
    require!(
        parameters.max_active_cases_per_claimant > 0,
        CustomError::InvalidVictimReliefPolicy
    );
    Ok(())
}

pub fn record_victim_relief_config_init_v1(
    config: &mut VictimReliefConfigV1,
    authority: Pubkey,
    treasury_config: Pubkey,
    security_governance_config: Pubkey,
    usdc_mint: Pubkey,
    now: i64,
    bump: u8,
) -> Result<()> {
    require!(
        authority != Pubkey::default()
            && treasury_config != Pubkey::default()
            && security_governance_config != Pubkey::default()
            && usdc_mint != Pubkey::default(),
        CustomError::InvalidVictimReliefConfig
    );

    config.authority = authority;
    config.treasury_config = treasury_config;
    config.security_governance_config = security_governance_config;
    config.usdc_mint = usdc_mint;
    config.current_policy = Pubkey::default();
    config.current_policy_version = 0;
    config.next_case_id = 1;
    config.paused = false;
    config.created_at = now;
    config.schema_version = VICTIM_RELIEF_SCHEMA_VERSION_V1;
    config.bump = bump;
    config.reserved = [0; 32];

    Ok(())
}

pub fn record_victim_relief_policy_init_v1(
    config: &mut VictimReliefConfigV1,
    policy: &mut VictimReliefPolicyV1,
    config_key: Pubkey,
    policy_key: Pubkey,
    authority: Pubkey,
    parameters: VictimReliefPolicyParametersV1,
    now: i64,
    bump: u8,
) -> Result<()> {
    validate_victim_relief_policy_parameters_v1(parameters)?;
    require!(
        config.schema_version == VICTIM_RELIEF_SCHEMA_VERSION_V1,
        CustomError::InvalidVictimReliefConfig
    );
    require!(
        authority == config.authority,
        CustomError::UnauthorizedSecurityAuthority
    );
    require!(
        config.current_policy == Pubkey::default() && config.current_policy_version == 0,
        CustomError::VictimReliefPolicyAlreadyInitialized
    );

    policy.config = config_key;
    policy.policy_version = VICTIM_RELIEF_POLICY_VERSION_V1;
    policy.min_claim_amount_usdc = parameters.min_claim_amount_usdc;
    policy.max_claim_amount_usdc = parameters.max_claim_amount_usdc;
    policy.max_payout_per_case_usdc = parameters.max_payout_per_case_usdc;
    policy.evidence_window_seconds = parameters.evidence_window_seconds;
    policy.review_window_seconds = parameters.review_window_seconds;
    policy.appeal_window_seconds = parameters.appeal_window_seconds;
    policy.submission_cooldown_seconds = parameters.submission_cooldown_seconds;
    policy.max_evidence_items = parameters.max_evidence_items;
    policy.max_active_cases_per_claimant = parameters.max_active_cases_per_claimant;
    policy.active = true;
    policy.initialized_by = authority;
    policy.created_at = now;
    policy.schema_version = VICTIM_RELIEF_SCHEMA_VERSION_V1;
    policy.bump = bump;
    policy.reserved = [0; 32];

    config.current_policy = policy_key;
    config.current_policy_version = VICTIM_RELIEF_POLICY_VERSION_V1;

    Ok(())
}

pub fn ensure_victim_relief_claimant_state_v1(
    claimant_state: &mut VictimReliefClaimantStateV1,
    config: Pubkey,
    claimant: Pubkey,
    now: i64,
    bump: u8,
) -> Result<()> {
    if claimant_state.schema_version == 0 {
        claimant_state.config = config;
        claimant_state.claimant = claimant;
        claimant_state.active_case_count = 0;
        claimant_state.total_case_count = 0;
        claimant_state.last_case_id = 0;
        claimant_state.last_submitted_at = 0;
        claimant_state.created_at = now;
        claimant_state.updated_at = now;
        claimant_state.schema_version = VICTIM_RELIEF_SCHEMA_VERSION_V1;
        claimant_state.bump = bump;
    }

    require!(
        claimant_state.schema_version == VICTIM_RELIEF_SCHEMA_VERSION_V1
            && claimant_state.config == config
            && claimant_state.claimant == claimant,
        CustomError::VictimReliefClaimantMismatch
    );
    Ok(())
}

pub fn validate_victim_relief_case_submission_v1(
    config: &VictimReliefConfigV1,
    policy: &VictimReliefPolicyV1,
    claimant_state: &VictimReliefClaimantStateV1,
    policy_key: Pubkey,
    case_id: u64,
    subject_commitment: [u8; 32],
    evidence_root: [u8; 32],
    evidence_count: u32,
    claimed_amount_usdc: u64,
    recipient_owner: Pubkey,
    recipient_mint: Pubkey,
    usdc_mint: Pubkey,
    now: i64,
) -> Result<()> {
    require!(!config.paused, CustomError::VictimReliefPaused);
    require!(
        config.schema_version == VICTIM_RELIEF_SCHEMA_VERSION_V1,
        CustomError::InvalidVictimReliefConfig
    );
    require!(
        policy.schema_version == VICTIM_RELIEF_SCHEMA_VERSION_V1
            && policy.policy_version == VICTIM_RELIEF_POLICY_VERSION_V1
            && policy.active,
        CustomError::InvalidVictimReliefPolicy
    );
    require!(
        config.current_policy == policy_key
            && config.current_policy_version == policy.policy_version,
        CustomError::InvalidVictimReliefPolicyVersion
    );
    require!(
        case_id == config.next_case_id,
        CustomError::InvalidVictimReliefCaseId
    );
    require!(
        !is_zero_32(&subject_commitment),
        CustomError::InvalidVictimReliefSubjectCommitment
    );
    require!(
        !is_zero_32(&evidence_root),
        CustomError::InvalidVictimReliefEvidenceRoot
    );
    validate_victim_relief_evidence_count_v1(evidence_count, policy.max_evidence_items)?;
    require!(
        claimed_amount_usdc >= policy.min_claim_amount_usdc
            && claimed_amount_usdc <= policy.max_claim_amount_usdc,
        CustomError::InvalidVictimReliefClaimAmount
    );
    require!(
        claimant_state.active_case_count < policy.max_active_cases_per_claimant,
        CustomError::VictimReliefActiveCaseLimitReached
    );
    if claimant_state.last_submitted_at != 0 {
        let next_allowed = claimant_state
            .last_submitted_at
            .checked_add(policy.submission_cooldown_seconds)
            .ok_or(CustomError::MathOverflow)?;
        require!(
            now >= next_allowed,
            CustomError::VictimReliefSubmissionCooldownActive
        );
    }
    require!(
        config.usdc_mint == usdc_mint && recipient_mint == usdc_mint,
        CustomError::InvalidMint
    );
    require!(
        recipient_owner == claimant_state.claimant,
        CustomError::VictimReliefRecipientMismatch
    );
    Ok(())
}

pub fn record_victim_relief_case_submission_v1(
    config: &mut VictimReliefConfigV1,
    policy: &VictimReliefPolicyV1,
    claimant_state: &mut VictimReliefClaimantStateV1,
    victim_relief_case: &mut VictimReliefCaseV1,
    config_key: Pubkey,
    policy_key: Pubkey,
    claimant: Pubkey,
    recipient_token_account: Pubkey,
    subject_commitment: [u8; 32],
    evidence_root: [u8; 32],
    evidence_count: u32,
    claimed_amount_usdc: u64,
    usdc_mint: Pubkey,
    now: i64,
    case_bump: u8,
) -> Result<()> {
    let case_id = config.next_case_id;
    let evidence_deadline = now
        .checked_add(policy.evidence_window_seconds)
        .ok_or(CustomError::MathOverflow)?;

    victim_relief_case.case_id = case_id;
    victim_relief_case.config = config_key;
    victim_relief_case.policy = policy_key;
    victim_relief_case.policy_version = policy.policy_version;
    victim_relief_case.claimant = claimant;
    victim_relief_case.subject_commitment = subject_commitment;
    victim_relief_case.evidence_root = evidence_root;
    victim_relief_case.evidence_count = evidence_count;
    victim_relief_case.evidence_revision = 0;
    victim_relief_case.claimed_amount_usdc = claimed_amount_usdc;
    victim_relief_case.approved_amount_usdc = 0;
    victim_relief_case.recipient_owner = claimant;
    victim_relief_case.recipient_token_account = recipient_token_account;
    victim_relief_case.usdc_mint = usdc_mint;
    victim_relief_case.status = VictimReliefCaseStatusV1::EvidencePeriod;
    victim_relief_case.active_appeal = Pubkey::default();
    victim_relief_case.decision_proposal = Pubkey::default();
    victim_relief_case.decision_queue = Pubkey::default();
    victim_relief_case.submitted_at = now;
    victim_relief_case.evidence_deadline = evidence_deadline;
    victim_relief_case.review_deadline = 0;
    victim_relief_case.appeal_deadline = 0;
    victim_relief_case.updated_at = now;
    victim_relief_case.schema_version = VICTIM_RELIEF_SCHEMA_VERSION_V1;
    victim_relief_case.bump = case_bump;
    victim_relief_case.reserved = [0; 64];

    config.next_case_id = config
        .next_case_id
        .checked_add(1)
        .ok_or(CustomError::MathOverflow)?;
    update_victim_relief_claimant_state_on_submit_v1(claimant_state, case_id, now)?;

    Ok(())
}

pub fn update_victim_relief_claimant_state_on_submit_v1(
    claimant_state: &mut VictimReliefClaimantStateV1,
    case_id: u64,
    now: i64,
) -> Result<()> {
    claimant_state.active_case_count = claimant_state
        .active_case_count
        .checked_add(1)
        .ok_or(CustomError::MathOverflow)?;
    claimant_state.total_case_count = claimant_state
        .total_case_count
        .checked_add(1)
        .ok_or(CustomError::MathOverflow)?;
    claimant_state.last_case_id = case_id;
    claimant_state.last_submitted_at = now;
    claimant_state.updated_at = now;
    Ok(())
}

pub fn close_victim_relief_active_case_count_v1(
    claimant_state: &mut VictimReliefClaimantStateV1,
    now: i64,
) -> Result<()> {
    claimant_state.active_case_count = claimant_state
        .active_case_count
        .checked_sub(1)
        .ok_or(CustomError::VictimReliefActiveCaseCountUnderflow)?;
    claimant_state.updated_at = now;
    Ok(())
}

pub fn validate_victim_relief_evidence_update_v1(
    config: &VictimReliefConfigV1,
    policy: &VictimReliefPolicyV1,
    victim_relief_case: &VictimReliefCaseV1,
    config_key: Pubkey,
    policy_key: Pubkey,
    claimant: Pubkey,
    new_evidence_root: [u8; 32],
    new_evidence_count: u32,
    now: i64,
) -> Result<()> {
    require!(
        config.schema_version == VICTIM_RELIEF_SCHEMA_VERSION_V1,
        CustomError::InvalidVictimReliefConfig
    );
    require!(
        victim_relief_case.claimant == claimant,
        CustomError::VictimReliefClaimantMismatch
    );
    require!(
        victim_relief_case.config == config_key,
        CustomError::InvalidVictimReliefConfig
    );
    require!(
        victim_relief_case.policy == policy_key
            && victim_relief_case.policy_version == policy.policy_version,
        CustomError::InvalidVictimReliefPolicyVersion
    );
    require!(
        victim_relief_case.status == VictimReliefCaseStatusV1::EvidencePeriod,
        CustomError::VictimReliefCaseStatusMismatch
    );
    require!(
        now <= victim_relief_case.evidence_deadline,
        CustomError::VictimReliefEvidenceWindowClosed
    );
    require!(
        !is_zero_32(&new_evidence_root),
        CustomError::InvalidVictimReliefEvidenceRoot
    );
    validate_victim_relief_evidence_count_v1(new_evidence_count, policy.max_evidence_items)?;
    require!(
        new_evidence_root != victim_relief_case.evidence_root
            || new_evidence_count != victim_relief_case.evidence_count,
        CustomError::VictimReliefEvidenceUnchanged
    );
    Ok(())
}

pub fn record_victim_relief_evidence_update_v1(
    victim_relief_case: &mut VictimReliefCaseV1,
    new_evidence_root: [u8; 32],
    new_evidence_count: u32,
    now: i64,
) -> Result<()> {
    victim_relief_case.evidence_root = new_evidence_root;
    victim_relief_case.evidence_count = new_evidence_count;
    victim_relief_case.evidence_revision = victim_relief_case
        .evidence_revision
        .checked_add(1)
        .ok_or(CustomError::MathOverflow)?;
    victim_relief_case.updated_at = now;
    Ok(())
}

pub fn validate_victim_relief_case_claimant_link_v1(
    config: &VictimReliefConfigV1,
    claimant_state: &VictimReliefClaimantStateV1,
    victim_relief_case: &VictimReliefCaseV1,
    config_key: Pubkey,
    claimant: Pubkey,
) -> Result<()> {
    require!(
        config.schema_version == VICTIM_RELIEF_SCHEMA_VERSION_V1,
        CustomError::InvalidVictimReliefConfig
    );
    require!(
        victim_relief_case.config == config_key,
        CustomError::InvalidVictimReliefConfig
    );
    require!(
        victim_relief_case.claimant == claimant
            && claimant_state.claimant == claimant
            && claimant_state.config == config_key,
        CustomError::VictimReliefClaimantMismatch
    );
    Ok(())
}

pub fn record_victim_relief_case_terminal_status_v1(
    claimant_state: &mut VictimReliefClaimantStateV1,
    victim_relief_case: &mut VictimReliefCaseV1,
    status: VictimReliefCaseStatusV1,
    now: i64,
) -> Result<()> {
    require!(
        status == VictimReliefCaseStatusV1::Cancelled
            || status == VictimReliefCaseStatusV1::Expired,
        CustomError::VictimReliefCaseStatusMismatch
    );
    close_victim_relief_active_case_count_v1(claimant_state, now)?;
    victim_relief_case.status = status;
    victim_relief_case.updated_at = now;
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VictimReliefDecisionValidationResultV1 {
    pub approved_amount_usdc: u64,
    pub parameters_hash: [u8; 32],
    pub canonical_governance_payload_hash: [u8; 32],
    pub action_type: GovernanceActionTypeV1,
    pub execution_type: VictimReliefDecisionExecutionTypeV1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VictimReliefAppealDecisionValidationResultV1 {
    pub approved_amount_usdc: u64,
    pub parameters_hash: [u8; 32],
    pub canonical_governance_payload_hash: [u8; 32],
    pub action_type: GovernanceActionTypeV1,
    pub execution_type: VictimReliefAppealExecutionTypeV1,
}

pub fn validate_victim_relief_evidence_freeze_v1(
    config: &VictimReliefConfigV1,
    policy: &VictimReliefPolicyV1,
    victim_relief_case: &VictimReliefCaseV1,
    config_key: Pubkey,
    policy_key: Pubkey,
    claimant: Pubkey,
    now: i64,
) -> Result<i64> {
    require!(!config.paused, CustomError::VictimReliefPaused);
    require!(
        config.schema_version == VICTIM_RELIEF_SCHEMA_VERSION_V1,
        CustomError::InvalidVictimReliefConfig
    );
    require!(
        policy.schema_version == VICTIM_RELIEF_SCHEMA_VERSION_V1 && policy.active,
        CustomError::InvalidVictimReliefPolicy
    );
    require_keys_eq!(
        policy.config,
        config_key,
        CustomError::InvalidVictimReliefPolicy
    );
    require!(
        victim_relief_case.schema_version == VICTIM_RELIEF_SCHEMA_VERSION_V1,
        CustomError::InvalidVictimReliefConfig
    );
    require_keys_eq!(
        victim_relief_case.config,
        config_key,
        CustomError::InvalidVictimReliefConfig
    );
    require_keys_eq!(
        victim_relief_case.policy,
        policy_key,
        CustomError::InvalidVictimReliefPolicyVersion
    );
    require!(
        victim_relief_case.policy_version == policy.policy_version,
        CustomError::InvalidVictimReliefPolicyVersion
    );
    require!(
        victim_relief_case.claimant == claimant,
        CustomError::VictimReliefClaimantMismatch
    );
    require!(
        victim_relief_case.recipient_owner == claimant,
        CustomError::VictimReliefRecipientMismatch
    );
    require!(
        victim_relief_case.status == VictimReliefCaseStatusV1::EvidencePeriod,
        CustomError::VictimReliefCaseStatusMismatch
    );
    require!(
        now <= victim_relief_case.evidence_deadline,
        CustomError::VictimReliefEvidenceFreezeTooLate
    );
    require!(
        !is_zero_32(&victim_relief_case.evidence_root),
        CustomError::InvalidVictimReliefEvidenceRoot
    );
    validate_victim_relief_evidence_count_v1(
        victim_relief_case.evidence_count,
        policy.max_evidence_items,
    )?;
    require!(
        victim_relief_case.claimed_amount_usdc >= policy.min_claim_amount_usdc
            && victim_relief_case.claimed_amount_usdc <= policy.max_claim_amount_usdc,
        CustomError::InvalidVictimReliefClaimAmount
    );
    require!(
        victim_relief_case.usdc_mint == config.usdc_mint,
        CustomError::InvalidMint
    );
    let review_deadline = now
        .checked_add(policy.review_window_seconds)
        .ok_or(CustomError::MathOverflow)?;
    Ok(review_deadline)
}

#[allow(clippy::too_many_arguments)]
pub fn record_victim_relief_evidence_snapshot_v1(
    snapshot: &mut VictimReliefEvidenceSnapshotV1,
    victim_relief_case: &mut VictimReliefCaseV1,
    case_key: Pubkey,
    config_key: Pubkey,
    policy_key: Pubkey,
    now: i64,
    review_deadline: i64,
    bump: u8,
) -> Result<()> {
    require!(
        snapshot.victim_relief_case == Pubkey::default(),
        CustomError::VictimReliefEvidenceAlreadyFrozen
    );

    snapshot.victim_relief_case = case_key;
    snapshot.config = config_key;
    snapshot.policy = policy_key;
    snapshot.policy_version = victim_relief_case.policy_version;
    snapshot.claimant = victim_relief_case.claimant;
    snapshot.subject_commitment = victim_relief_case.subject_commitment;
    snapshot.evidence_root = victim_relief_case.evidence_root;
    snapshot.evidence_count = victim_relief_case.evidence_count;
    snapshot.evidence_revision = victim_relief_case.evidence_revision;
    snapshot.claimed_amount_usdc = victim_relief_case.claimed_amount_usdc;
    snapshot.recipient_owner = victim_relief_case.recipient_owner;
    snapshot.recipient_token_account = victim_relief_case.recipient_token_account;
    snapshot.usdc_mint = victim_relief_case.usdc_mint;
    snapshot.frozen_at = now;
    snapshot.review_deadline = review_deadline;
    snapshot.schema_version = VICTIM_RELIEF_SCHEMA_VERSION_V1;
    snapshot.bump = bump;
    snapshot.reserved = [0; 32];

    victim_relief_case.status = VictimReliefCaseStatusV1::UnderReview;
    victim_relief_case.review_deadline = review_deadline;
    victim_relief_case.updated_at = now;
    Ok(())
}

pub fn derive_victim_relief_approved_amount_v1(
    victim_relief_case: &VictimReliefCaseV1,
    policy: &VictimReliefPolicyV1,
    policy_key: Pubkey,
) -> Result<u64> {
    require_keys_eq!(
        victim_relief_case.policy,
        policy_key,
        CustomError::InvalidVictimReliefPolicyVersion
    );
    require!(
        victim_relief_case.policy_version == policy.policy_version,
        CustomError::InvalidVictimReliefPolicyVersion
    );
    require!(
        victim_relief_case.claimed_amount_usdc > 0 && policy.max_payout_per_case_usdc > 0,
        CustomError::InvalidVictimReliefClaimAmount
    );
    let approved_amount = core::cmp::min(
        victim_relief_case.claimed_amount_usdc,
        policy.max_payout_per_case_usdc,
    );
    require!(
        approved_amount > 0
            && approved_amount <= victim_relief_case.claimed_amount_usdc
            && approved_amount <= policy.max_payout_per_case_usdc,
        CustomError::VictimReliefApprovedAmountMismatch
    );
    Ok(approved_amount)
}

pub fn hash_victim_relief_decision_parameters_v1(
    parameters: &VictimReliefDecisionParametersV1,
) -> Result<[u8; 32]> {
    require!(
        parameters.schema_version == VICTIM_RELIEF_DECISION_SCHEMA_VERSION,
        CustomError::InvalidVictimReliefDecisionSchema
    );
    let envelope = VictimReliefDecisionParametersHashEnvelopeV1 {
        domain_separator: VICTIM_RELIEF_DECISION_PARAMETERS_V1_DOMAIN_BYTES,
        parameters: *parameters,
    };
    hash_contributor_payload(&envelope)
        .map_err(|_| error!(CustomError::VictimReliefDecisionParametersMismatch))
}

pub fn victim_relief_decision_execution_type_stable_code_v1(
    execution_type: VictimReliefDecisionExecutionTypeV1,
) -> u8 {
    match execution_type {
        VictimReliefDecisionExecutionTypeV1::Approve => 1,
        VictimReliefDecisionExecutionTypeV1::Reject => 2,
    }
}

pub fn victim_relief_decision_execution_type_from_stable_code_v1(
    code: u8,
) -> Result<VictimReliefDecisionExecutionTypeV1> {
    match code {
        1 => Ok(VictimReliefDecisionExecutionTypeV1::Approve),
        2 => Ok(VictimReliefDecisionExecutionTypeV1::Reject),
        _ => err!(CustomError::InvalidVictimReliefDecisionSchema),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build_victim_relief_decision_parameters_v1(
    config_key: Pubkey,
    policy_key: Pubkey,
    victim_relief_case_key: Pubkey,
    evidence_snapshot_key: Pubkey,
    victim_relief_case: &VictimReliefCaseV1,
    evidence_snapshot: &VictimReliefEvidenceSnapshotV1,
    treasury_config: Pubkey,
    relief_usdc_vault: Pubkey,
    action_type: GovernanceActionTypeV1,
    approved_amount_usdc: u64,
    proposal_id: u64,
) -> VictimReliefDecisionParametersV1 {
    VictimReliefDecisionParametersV1 {
        schema_version: VICTIM_RELIEF_DECISION_SCHEMA_VERSION,
        config: config_key,
        policy: policy_key,
        policy_version: victim_relief_case.policy_version,
        victim_relief_case: victim_relief_case_key,
        evidence_snapshot: evidence_snapshot_key,
        case_id: victim_relief_case.case_id,
        claimant: victim_relief_case.claimant,
        subject_commitment: evidence_snapshot.subject_commitment,
        evidence_root: evidence_snapshot.evidence_root,
        evidence_count: evidence_snapshot.evidence_count,
        evidence_revision: evidence_snapshot.evidence_revision,
        claimed_amount_usdc: victim_relief_case.claimed_amount_usdc,
        approved_amount_usdc,
        recipient_owner: victim_relief_case.recipient_owner,
        recipient_token_account: victim_relief_case.recipient_token_account,
        usdc_mint: victim_relief_case.usdc_mint,
        treasury_config,
        relief_usdc_vault,
        action_type,
        expected_case_status: VictimReliefCaseStatusV1::UnderReview,
        proposal_id,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn validate_victim_relief_snapshot_matches_case_v1(
    snapshot: &VictimReliefEvidenceSnapshotV1,
    snapshot_key: Pubkey,
    victim_relief_case: &VictimReliefCaseV1,
    victim_relief_case_key: Pubkey,
    config_key: Pubkey,
    policy_key: Pubkey,
) -> Result<()> {
    require!(
        snapshot_key != Pubkey::default(),
        CustomError::VictimReliefEvidenceSnapshotMismatch
    );
    require_keys_eq!(
        snapshot.victim_relief_case,
        victim_relief_case_key,
        CustomError::VictimReliefEvidenceSnapshotMismatch
    );
    require_keys_eq!(
        snapshot.config,
        config_key,
        CustomError::VictimReliefEvidenceSnapshotMismatch
    );
    require_keys_eq!(
        snapshot.policy,
        policy_key,
        CustomError::VictimReliefEvidenceSnapshotMismatch
    );
    require!(
        snapshot.policy_version == victim_relief_case.policy_version
            && snapshot.claimant == victim_relief_case.claimant
            && snapshot.subject_commitment == victim_relief_case.subject_commitment
            && snapshot.evidence_root == victim_relief_case.evidence_root
            && snapshot.evidence_count == victim_relief_case.evidence_count
            && snapshot.evidence_revision == victim_relief_case.evidence_revision
            && snapshot.claimed_amount_usdc == victim_relief_case.claimed_amount_usdc
            && snapshot.recipient_owner == victim_relief_case.recipient_owner
            && snapshot.recipient_token_account == victim_relief_case.recipient_token_account
            && snapshot.usdc_mint == victim_relief_case.usdc_mint
            && snapshot.schema_version == VICTIM_RELIEF_SCHEMA_VERSION_V1,
        CustomError::VictimReliefEvidenceSnapshotMismatch
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn validate_victim_relief_decision_execution_context_v1(
    security_governance_config: &GovernanceConfigV1,
    security_governance_config_key: Pubkey,
    protocol_module_registry: &ProtocolModuleRegistryV1,
    protocol_module_registry_key: Pubkey,
    governance_proposal: &GovernanceProposalV1,
    governance_proposal_key: Pubkey,
    governance_proposal_action: &GovernanceProposalActionV1,
    governance_proposal_action_key: Pubkey,
    governance_decision_adapter: &UniversalGovernanceDecisionAdapterV1,
    proposal_decision: &ProposalDecisionV1,
    proposal_decision_key: Pubkey,
    execution_queue_item: &ExecutionQueueItemV1,
    execution_queue_item_key: Pubkey,
    config: &VictimReliefConfigV1,
    config_key: Pubkey,
    policy: &VictimReliefPolicyV1,
    policy_key: Pubkey,
    victim_relief_case: &VictimReliefCaseV1,
    victim_relief_case_key: Pubkey,
    claimant_state: &VictimReliefClaimantStateV1,
    evidence_snapshot: &VictimReliefEvidenceSnapshotV1,
    evidence_snapshot_key: Pubkey,
    treasury_config: &TreasuryConfigV2,
    treasury_config_key: Pubkey,
    relief_usdc_vault_key: Pubkey,
    relief_usdc_vault: &TokenAccount,
    vault_authority: Pubkey,
    usdc_mint: Pubkey,
    expected_action: GovernanceActionTypeV1,
) -> Result<VictimReliefDecisionValidationResultV1> {
    require!(
        expected_action == GovernanceActionTypeV1::VictimReliefApproveCompensation
            || expected_action == GovernanceActionTypeV1::VictimReliefRejectClaim,
        CustomError::VictimReliefDecisionActionMismatch
    );
    require!(
        governance_proposal.status == GovernanceProposalStatusV1::Passed,
        CustomError::InvalidGovernanceProposal
    );
    validate_governance_proposal_action_v1(
        governance_proposal,
        governance_proposal_action,
        governance_proposal_key,
    )?;
    require!(
        governance_proposal_action_key != Pubkey::default(),
        CustomError::GovernanceProposalActionMissing
    );
    require!(
        governance_proposal_action.action_type == expected_action,
        CustomError::VictimReliefDecisionActionMismatch
    );
    require!(
        governance_proposal_action.module_id == ProtocolModuleIdV1::VictimRelief,
        CustomError::VictimReliefDecisionActionMismatch
    );
    require_keys_eq!(
        governance_proposal_action.target_program,
        crate::ID,
        CustomError::VictimReliefDecisionTargetMismatch
    );
    require_keys_eq!(
        governance_proposal_action.target_account,
        victim_relief_case_key,
        CustomError::VictimReliefDecisionTargetMismatch
    );
    validate_protocol_module_registry_v1(
        protocol_module_registry,
        protocol_module_registry_key,
        security_governance_config_key,
        ProtocolModuleIdV1::VictimRelief,
        crate::ID,
    )?;
    require_keys_eq!(
        config.security_governance_config,
        security_governance_config_key,
        CustomError::ProtocolModuleGovernanceConfigMismatch
    );
    require!(
        !security_governance_config.is_paused,
        CustomError::SecurityLayerPaused
    );
    require!(!config.paused, CustomError::VictimReliefPaused);

    require!(
        victim_relief_case.status == VictimReliefCaseStatusV1::UnderReview,
        CustomError::VictimReliefDecisionNotEligible
    );
    require_keys_eq!(
        victim_relief_case.config,
        config_key,
        CustomError::InvalidVictimReliefConfig
    );
    require_keys_eq!(
        victim_relief_case.policy,
        policy_key,
        CustomError::InvalidVictimReliefPolicyVersion
    );
    require!(
        victim_relief_case.policy_version == policy.policy_version
            && policy.schema_version == VICTIM_RELIEF_SCHEMA_VERSION_V1
            && policy.config == config_key,
        CustomError::InvalidVictimReliefPolicyVersion
    );
    require!(
        claimant_state.config == config_key
            && claimant_state.claimant == victim_relief_case.claimant,
        CustomError::VictimReliefClaimantMismatch
    );
    validate_victim_relief_snapshot_matches_case_v1(
        evidence_snapshot,
        evidence_snapshot_key,
        victim_relief_case,
        victim_relief_case_key,
        config_key,
        policy_key,
    )?;
    require_keys_eq!(
        config.treasury_config,
        treasury_config_key,
        CustomError::InvalidVictimReliefConfig
    );
    require!(
        config.usdc_mint == treasury_config.usdc_mint
            && config.usdc_mint == victim_relief_case.usdc_mint
            && usdc_mint == treasury_config.usdc_mint,
        CustomError::InvalidMint
    );
    let (expected_relief_vault, _) =
        Pubkey::find_program_address(&[RELIEF_USDC_VAULT_SEED], &crate::ID);
    require_keys_eq!(
        relief_usdc_vault_key,
        expected_relief_vault,
        CustomError::VictimReliefReliefVaultMismatch
    );
    let (expected_vault_authority, _) =
        Pubkey::find_program_address(&[VAULT_AUTHORITY_V2_SEED], &crate::ID);
    require_keys_eq!(
        vault_authority,
        expected_vault_authority,
        CustomError::VictimReliefReliefVaultMismatch
    );
    require!(
        relief_usdc_vault.mint == treasury_config.usdc_mint
            && relief_usdc_vault.owner == expected_vault_authority,
        CustomError::VictimReliefReliefVaultMismatch
    );

    let approved_amount_usdc = match expected_action {
        GovernanceActionTypeV1::VictimReliefApproveCompensation => {
            derive_victim_relief_approved_amount_v1(victim_relief_case, policy, policy_key)?
        }
        GovernanceActionTypeV1::VictimReliefRejectClaim => 0,
        _ => return err!(CustomError::VictimReliefDecisionActionMismatch),
    };
    let execution_type = match expected_action {
        GovernanceActionTypeV1::VictimReliefApproveCompensation => {
            VictimReliefDecisionExecutionTypeV1::Approve
        }
        GovernanceActionTypeV1::VictimReliefRejectClaim => {
            VictimReliefDecisionExecutionTypeV1::Reject
        }
        _ => return err!(CustomError::VictimReliefDecisionActionMismatch),
    };
    let parameters = build_victim_relief_decision_parameters_v1(
        config_key,
        policy_key,
        victim_relief_case_key,
        evidence_snapshot_key,
        victim_relief_case,
        evidence_snapshot,
        treasury_config_key,
        relief_usdc_vault_key,
        expected_action,
        approved_amount_usdc,
        governance_proposal.proposal_id,
    );
    require!(
        parameters.expected_case_status == VictimReliefCaseStatusV1::UnderReview,
        CustomError::InvalidVictimReliefDecisionSchema
    );
    let parameters_hash = hash_victim_relief_decision_parameters_v1(&parameters)?;
    require!(
        governance_proposal_action.parameters_hash == parameters_hash,
        CustomError::VictimReliefDecisionParametersMismatch
    );

    let governance_payload = GovernancePayloadV1 {
        schema_version: GOVERNANCE_PAYLOAD_V1_SCHEMA_VERSION,
        action_type: governance_proposal_action.action_type,
        module_id: governance_proposal_action.module_id,
        target_program: governance_proposal_action.target_program,
        target_account: governance_proposal_action.target_account,
        parameters_hash: governance_proposal_action.parameters_hash,
        evidence_hash: governance_proposal_action.evidence_hash,
        created_at: governance_proposal_action.created_at,
    };
    let canonical_governance_payload_hash = hash_governance_payload_v1(&governance_payload)?;
    require!(
        canonical_governance_payload_hash == governance_proposal_action.canonical_payload_hash,
        CustomError::VictimReliefDecisionParametersMismatch
    );
    require!(
        governance_proposal.payload_hash == canonical_governance_payload_hash,
        CustomError::VictimReliefDecisionParametersMismatch
    );

    let expected_security_action = map_governance_action_to_security_action(expected_action)?;
    require!(
        governance_decision_adapter.governance_proposal == governance_proposal_key
            && governance_decision_adapter.proposal_decision == proposal_decision_key
            && governance_decision_adapter.action_type == expected_security_action
            && governance_decision_adapter.target_program == crate::ID
            && governance_decision_adapter.target_account == victim_relief_case_key
            && governance_decision_adapter.payload_hash == canonical_governance_payload_hash
            && governance_decision_adapter.executed,
        CustomError::InvalidGovernanceDecisionAdapter
    );
    require!(
        proposal_decision.proposal_id == governance_proposal.proposal_id
            && proposal_decision.proposer == governance_proposal.proposer
            && proposal_decision.proposal_type
                == security_proposal_type_for_action(expected_security_action)?
            && proposal_decision.decision == ProposalDecision::Approved,
        CustomError::ProposalNotApproved
    );
    require!(
        execution_queue_item.proposal_id == governance_proposal.proposal_id
            && execution_queue_item.action_type == expected_security_action
            && execution_queue_item.target_program == crate::ID
            && execution_queue_item.target_account == victim_relief_case_key
            && execution_queue_item.decision == ProposalDecision::Approved
            && execution_queue_item.status == ExecutionStatus::Executed
            && execution_queue_item.payload_hash == canonical_governance_payload_hash
            && execution_queue_item_key != Pubkey::default(),
        CustomError::InvalidExecutionStatus
    );

    Ok(VictimReliefDecisionValidationResultV1 {
        approved_amount_usdc,
        parameters_hash,
        canonical_governance_payload_hash,
        action_type: expected_action,
        execution_type,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn record_victim_relief_decision_execution_record_v1(
    record: &mut VictimReliefDecisionExecutionRecordV1,
    queue_key: Pubkey,
    proposal_decision_key: Pubkey,
    governance_proposal_key: Pubkey,
    governance_proposal_action_key: Pubkey,
    module_registry_key: Pubkey,
    config_key: Pubkey,
    policy_key: Pubkey,
    case_key: Pubkey,
    snapshot_key: Pubkey,
    case_status_before: VictimReliefCaseStatusV1,
    case_status_after: VictimReliefCaseStatusV1,
    victim_relief_case: &VictimReliefCaseV1,
    validation: VictimReliefDecisionValidationResultV1,
    executor: Pubkey,
    now: i64,
    bump: u8,
) -> Result<()> {
    require!(
        record.execution_queue_item == Pubkey::default(),
        CustomError::VictimReliefDecisionExecutionAlreadyCompleted
    );
    record.execution_queue_item = queue_key;
    record.proposal_decision = proposal_decision_key;
    record.governance_proposal = governance_proposal_key;
    record.governance_proposal_action = governance_proposal_action_key;
    record.module_registry = module_registry_key;
    record.config = config_key;
    record.policy = policy_key;
    record.victim_relief_case = case_key;
    record.evidence_snapshot = snapshot_key;
    record.execution_type = validation.execution_type;
    record.governance_action_type = validation.action_type;
    record.case_status_before = case_status_before;
    record.case_status_after = case_status_after;
    record.claimed_amount_usdc = victim_relief_case.claimed_amount_usdc;
    record.approved_amount_usdc = validation.approved_amount_usdc;
    record.recipient_owner = victim_relief_case.recipient_owner;
    record.recipient_token_account = victim_relief_case.recipient_token_account;
    record.parameters_hash = validation.parameters_hash;
    record.canonical_governance_payload_hash = validation.canonical_governance_payload_hash;
    record.executor = executor;
    record.executed_at = now;
    record.schema_version = VICTIM_RELIEF_DECISION_SCHEMA_VERSION;
    record.bump = bump;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn record_relief_payout_request_v1(
    payout_request: &mut ReliefPayoutRequestV1,
    case_key: Pubkey,
    config_key: Pubkey,
    policy_key: Pubkey,
    victim_relief_case: &VictimReliefCaseV1,
    governance_proposal_key: Pubkey,
    proposal_decision_key: Pubkey,
    queue_key: Pubkey,
    snapshot_key: Pubkey,
    treasury_config_key: Pubkey,
    relief_vault_key: Pubkey,
    parameters_hash: [u8; 32],
    now: i64,
    bump: u8,
) -> Result<()> {
    require!(
        payout_request.victim_relief_case == Pubkey::default(),
        CustomError::VictimReliefPayoutRequestAlreadyExists
    );
    payout_request.victim_relief_case = case_key;
    payout_request.config = config_key;
    payout_request.policy = policy_key;
    payout_request.policy_version = victim_relief_case.policy_version;
    payout_request.governance_proposal = governance_proposal_key;
    payout_request.proposal_decision = proposal_decision_key;
    payout_request.execution_queue_item = queue_key;
    payout_request.evidence_snapshot = snapshot_key;
    payout_request.approved_amount_usdc = victim_relief_case.approved_amount_usdc;
    payout_request.recipient_owner = victim_relief_case.recipient_owner;
    payout_request.recipient_token_account = victim_relief_case.recipient_token_account;
    payout_request.treasury_config = treasury_config_key;
    payout_request.relief_usdc_vault = relief_vault_key;
    payout_request.usdc_mint = victim_relief_case.usdc_mint;
    payout_request.status = VictimReliefPayoutStatusV1::Approved;
    payout_request.parameters_hash = parameters_hash;
    payout_request.created_at = now;
    payout_request.executed_at = 0;
    payout_request.schema_version = VICTIM_RELIEF_DECISION_SCHEMA_VERSION;
    payout_request.bump = bump;
    payout_request.reserved = [0; 32];
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn record_approve_victim_relief_decision_v1(
    victim_relief_case: &mut VictimReliefCaseV1,
    payout_request: &mut ReliefPayoutRequestV1,
    record: &mut VictimReliefDecisionExecutionRecordV1,
    case_key: Pubkey,
    config_key: Pubkey,
    policy_key: Pubkey,
    governance_proposal_key: Pubkey,
    governance_proposal_action_key: Pubkey,
    proposal_decision_key: Pubkey,
    queue_key: Pubkey,
    module_registry_key: Pubkey,
    snapshot_key: Pubkey,
    treasury_config_key: Pubkey,
    relief_vault_key: Pubkey,
    executor: Pubkey,
    validation: VictimReliefDecisionValidationResultV1,
    now: i64,
    payout_bump: u8,
    record_bump: u8,
) -> Result<()> {
    require!(
        validation.execution_type == VictimReliefDecisionExecutionTypeV1::Approve,
        CustomError::VictimReliefDecisionActionMismatch
    );
    require!(
        validation.approved_amount_usdc > 0,
        CustomError::VictimReliefApprovedAmountMismatch
    );
    let case_status_before = victim_relief_case.status;
    require!(
        case_status_before == VictimReliefCaseStatusV1::UnderReview,
        CustomError::VictimReliefDecisionNotEligible
    );

    victim_relief_case.approved_amount_usdc = validation.approved_amount_usdc;
    victim_relief_case.status = VictimReliefCaseStatusV1::PayoutQueued;
    victim_relief_case.decision_proposal = governance_proposal_key;
    victim_relief_case.decision_queue = queue_key;
    victim_relief_case.updated_at = now;

    record_relief_payout_request_v1(
        payout_request,
        case_key,
        config_key,
        policy_key,
        victim_relief_case,
        governance_proposal_key,
        proposal_decision_key,
        queue_key,
        snapshot_key,
        treasury_config_key,
        relief_vault_key,
        validation.parameters_hash,
        now,
        payout_bump,
    )?;
    record_victim_relief_decision_execution_record_v1(
        record,
        queue_key,
        proposal_decision_key,
        governance_proposal_key,
        governance_proposal_action_key,
        module_registry_key,
        config_key,
        policy_key,
        case_key,
        snapshot_key,
        case_status_before,
        VictimReliefCaseStatusV1::PayoutQueued,
        victim_relief_case,
        validation,
        executor,
        now,
        record_bump,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn record_reject_victim_relief_decision_v1(
    claimant_state: &mut VictimReliefClaimantStateV1,
    victim_relief_case: &mut VictimReliefCaseV1,
    record: &mut VictimReliefDecisionExecutionRecordV1,
    config_key: Pubkey,
    policy_key: Pubkey,
    governance_proposal_key: Pubkey,
    governance_proposal_action_key: Pubkey,
    proposal_decision_key: Pubkey,
    queue_key: Pubkey,
    module_registry_key: Pubkey,
    case_key: Pubkey,
    snapshot_key: Pubkey,
    executor: Pubkey,
    appeal_window_seconds: i64,
    validation: VictimReliefDecisionValidationResultV1,
    now: i64,
    record_bump: u8,
) -> Result<()> {
    require!(
        validation.execution_type == VictimReliefDecisionExecutionTypeV1::Reject,
        CustomError::VictimReliefDecisionActionMismatch
    );
    require!(
        validation.approved_amount_usdc == 0,
        CustomError::VictimReliefApprovedAmountMismatch
    );
    let case_status_before = victim_relief_case.status;
    require!(
        case_status_before == VictimReliefCaseStatusV1::UnderReview,
        CustomError::VictimReliefDecisionNotEligible
    );

    let appeal_deadline = now
        .checked_add(appeal_window_seconds)
        .ok_or(CustomError::MathOverflow)?;
    close_victim_relief_active_case_count_v1(claimant_state, now)?;

    victim_relief_case.approved_amount_usdc = 0;
    victim_relief_case.status = VictimReliefCaseStatusV1::Rejected;
    victim_relief_case.decision_proposal = governance_proposal_key;
    victim_relief_case.decision_queue = queue_key;
    victim_relief_case.appeal_deadline = appeal_deadline;
    victim_relief_case.updated_at = now;

    record_victim_relief_decision_execution_record_v1(
        record,
        queue_key,
        proposal_decision_key,
        governance_proposal_key,
        governance_proposal_action_key,
        module_registry_key,
        config_key,
        policy_key,
        case_key,
        snapshot_key,
        case_status_before,
        VictimReliefCaseStatusV1::Rejected,
        victim_relief_case,
        validation,
        executor,
        now,
        record_bump,
    )
}

pub fn validate_victim_relief_appeal_evidence_v1(
    appeal_evidence_root: [u8; 32],
    appeal_evidence_count: u32,
    max_evidence_items: u32,
) -> Result<()> {
    let root_is_zero = is_zero_32(&appeal_evidence_root);
    if root_is_zero {
        require!(
            appeal_evidence_count == 0,
            CustomError::VictimReliefAppealEvidenceMismatch
        );
    } else {
        require!(
            appeal_evidence_count > 0 && appeal_evidence_count <= max_evidence_items,
            CustomError::VictimReliefAppealEvidenceMismatch
        );
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn validate_original_victim_relief_reject_record_v1(
    original_decision_record: &VictimReliefDecisionExecutionRecordV1,
    original_decision_record_key: Pubkey,
    victim_relief_case: &VictimReliefCaseV1,
    victim_relief_case_key: Pubkey,
    original_evidence_snapshot_key: Pubkey,
) -> Result<()> {
    require!(
        original_decision_record_key != Pubkey::default()
            && original_decision_record.execution_queue_item == victim_relief_case.decision_queue
            && original_decision_record.governance_proposal == victim_relief_case.decision_proposal
            && original_decision_record.victim_relief_case == victim_relief_case_key
            && original_decision_record.evidence_snapshot == original_evidence_snapshot_key
            && original_decision_record.execution_type
                == VictimReliefDecisionExecutionTypeV1::Reject
            && original_decision_record.governance_action_type
                == GovernanceActionTypeV1::VictimReliefRejectClaim
            && original_decision_record.case_status_after == VictimReliefCaseStatusV1::Rejected
            && original_decision_record.approved_amount_usdc == 0
            && original_decision_record.parameters_hash != [0; 32]
            && original_decision_record.canonical_governance_payload_hash != [0; 32]
            && original_decision_record.schema_version == VICTIM_RELIEF_DECISION_SCHEMA_VERSION,
        CustomError::VictimReliefAppealOriginalDecisionMismatch
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn validate_open_victim_relief_appeal_v1(
    config: &VictimReliefConfigV1,
    policy: &VictimReliefPolicyV1,
    victim_relief_case: &VictimReliefCaseV1,
    victim_relief_case_key: Pubkey,
    original_evidence_snapshot: &VictimReliefEvidenceSnapshotV1,
    original_evidence_snapshot_key: Pubkey,
    original_decision_record: &VictimReliefDecisionExecutionRecordV1,
    original_decision_record_key: Pubkey,
    config_key: Pubkey,
    policy_key: Pubkey,
    claimant: Pubkey,
    appeal_evidence_root: [u8; 32],
    appeal_evidence_count: u32,
    now: i64,
) -> Result<()> {
    require!(!config.paused, CustomError::VictimReliefPaused);
    require!(
        config.schema_version == VICTIM_RELIEF_SCHEMA_VERSION_V1,
        CustomError::InvalidVictimReliefConfig
    );
    require!(
        policy.schema_version == VICTIM_RELIEF_SCHEMA_VERSION_V1
            && policy.active
            && policy.config == config_key,
        CustomError::InvalidVictimReliefPolicy
    );
    require!(
        victim_relief_case.claimant == claimant,
        CustomError::VictimReliefClaimantMismatch
    );
    require!(
        victim_relief_case.status == VictimReliefCaseStatusV1::Rejected,
        CustomError::VictimReliefAppealNotEligible
    );
    require!(
        victim_relief_case.appeal_deadline > 0 && now <= victim_relief_case.appeal_deadline,
        CustomError::VictimReliefAppealWindowClosed
    );
    require!(
        victim_relief_case.active_appeal == Pubkey::default(),
        CustomError::VictimReliefAppealAlreadyExists
    );
    require_keys_eq!(
        victim_relief_case.config,
        config_key,
        CustomError::InvalidVictimReliefConfig
    );
    require_keys_eq!(
        victim_relief_case.policy,
        policy_key,
        CustomError::InvalidVictimReliefPolicyVersion
    );
    require!(
        victim_relief_case.policy_version == policy.policy_version,
        CustomError::InvalidVictimReliefPolicyVersion
    );
    validate_victim_relief_snapshot_matches_case_v1(
        original_evidence_snapshot,
        original_evidence_snapshot_key,
        victim_relief_case,
        victim_relief_case_key,
        config_key,
        policy_key,
    )?;
    validate_original_victim_relief_reject_record_v1(
        original_decision_record,
        original_decision_record_key,
        victim_relief_case,
        victim_relief_case_key,
        original_evidence_snapshot_key,
    )?;
    validate_victim_relief_appeal_evidence_v1(
        appeal_evidence_root,
        appeal_evidence_count,
        policy.max_evidence_items,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn record_open_victim_relief_appeal_v1(
    appeal: &mut VictimReliefAppealV1,
    victim_relief_case: &mut VictimReliefCaseV1,
    appeal_key: Pubkey,
    victim_relief_case_key: Pubkey,
    config_key: Pubkey,
    policy_key: Pubkey,
    original_evidence_snapshot_key: Pubkey,
    original_decision_record_key: Pubkey,
    appeal_evidence_root: [u8; 32],
    appeal_evidence_count: u32,
    now: i64,
    bump: u8,
) -> Result<()> {
    require!(
        appeal.victim_relief_case == Pubkey::default(),
        CustomError::VictimReliefAppealAlreadyExists
    );
    let original_governance_proposal = victim_relief_case.decision_proposal;
    let original_execution_queue_item = victim_relief_case.decision_queue;

    appeal.victim_relief_case = victim_relief_case_key;
    appeal.config = config_key;
    appeal.policy = policy_key;
    appeal.policy_version = victim_relief_case.policy_version;
    appeal.claimant = victim_relief_case.claimant;
    appeal.original_evidence_snapshot = original_evidence_snapshot_key;
    appeal.original_decision_record = original_decision_record_key;
    appeal.original_governance_proposal = original_governance_proposal;
    appeal.original_execution_queue_item = original_execution_queue_item;
    appeal.appeal_evidence_root = appeal_evidence_root;
    appeal.appeal_evidence_count = appeal_evidence_count;
    appeal.status = VictimReliefAppealStatusV1::Pending;
    appeal.decision_proposal = Pubkey::default();
    appeal.decision_queue = Pubkey::default();
    appeal.opened_at = now;
    appeal.appeal_deadline = victim_relief_case.appeal_deadline;
    appeal.resolved_at = 0;
    appeal.schema_version = VICTIM_RELIEF_SCHEMA_VERSION_V1;
    appeal.bump = bump;
    appeal.reserved = [0; 32];

    victim_relief_case.status = VictimReliefCaseStatusV1::AppealPending;
    victim_relief_case.active_appeal = appeal_key;
    victim_relief_case.updated_at = now;
    Ok(())
}

pub fn hash_victim_relief_appeal_decision_parameters_v1(
    parameters: &VictimReliefAppealDecisionParametersV1,
) -> Result<[u8; 32]> {
    require!(
        parameters.schema_version == VICTIM_RELIEF_DECISION_SCHEMA_VERSION
            && parameters.expected_case_status == VictimReliefCaseStatusV1::AppealPending
            && parameters.expected_appeal_status == VictimReliefAppealStatusV1::Pending,
        CustomError::InvalidVictimReliefAppealSchema
    );
    let envelope = VictimReliefAppealDecisionParametersHashEnvelopeV1 {
        domain_separator: VICTIM_RELIEF_APPEAL_DECISION_PARAMETERS_V1_DOMAIN_BYTES,
        parameters: *parameters,
    };
    hash_contributor_payload(&envelope)
        .map_err(|_| error!(CustomError::VictimReliefAppealParametersMismatch))
}

pub fn victim_relief_appeal_execution_type_stable_code_v1(
    execution_type: VictimReliefAppealExecutionTypeV1,
) -> u8 {
    match execution_type {
        VictimReliefAppealExecutionTypeV1::Uphold => 1,
        VictimReliefAppealExecutionTypeV1::Overturn => 2,
    }
}

pub fn victim_relief_appeal_execution_type_from_stable_code_v1(
    code: u8,
) -> Result<VictimReliefAppealExecutionTypeV1> {
    match code {
        1 => Ok(VictimReliefAppealExecutionTypeV1::Uphold),
        2 => Ok(VictimReliefAppealExecutionTypeV1::Overturn),
        _ => err!(CustomError::InvalidVictimReliefAppealSchema),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build_victim_relief_appeal_decision_parameters_v1(
    config_key: Pubkey,
    policy_key: Pubkey,
    victim_relief_case_key: Pubkey,
    victim_relief_appeal_key: Pubkey,
    original_evidence_snapshot_key: Pubkey,
    original_decision_record_key: Pubkey,
    victim_relief_case: &VictimReliefCaseV1,
    appeal: &VictimReliefAppealV1,
    original_evidence_snapshot: &VictimReliefEvidenceSnapshotV1,
    treasury_config: Pubkey,
    relief_usdc_vault: Pubkey,
    action_type: GovernanceActionTypeV1,
    approved_amount_usdc: u64,
    proposal_id: u64,
) -> VictimReliefAppealDecisionParametersV1 {
    VictimReliefAppealDecisionParametersV1 {
        schema_version: VICTIM_RELIEF_DECISION_SCHEMA_VERSION,
        config: config_key,
        policy: policy_key,
        policy_version: victim_relief_case.policy_version,
        victim_relief_case: victim_relief_case_key,
        victim_relief_appeal: victim_relief_appeal_key,
        original_evidence_snapshot: original_evidence_snapshot_key,
        original_decision_record: original_decision_record_key,
        case_id: victim_relief_case.case_id,
        claimant: victim_relief_case.claimant,
        subject_commitment: original_evidence_snapshot.subject_commitment,
        original_evidence_root: original_evidence_snapshot.evidence_root,
        original_evidence_count: original_evidence_snapshot.evidence_count,
        original_evidence_revision: original_evidence_snapshot.evidence_revision,
        appeal_evidence_root: appeal.appeal_evidence_root,
        appeal_evidence_count: appeal.appeal_evidence_count,
        claimed_amount_usdc: victim_relief_case.claimed_amount_usdc,
        approved_amount_usdc,
        recipient_owner: victim_relief_case.recipient_owner,
        recipient_token_account: victim_relief_case.recipient_token_account,
        usdc_mint: victim_relief_case.usdc_mint,
        treasury_config,
        relief_usdc_vault,
        action_type,
        expected_case_status: VictimReliefCaseStatusV1::AppealPending,
        expected_appeal_status: VictimReliefAppealStatusV1::Pending,
        proposal_id,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn validate_victim_relief_appeal_matches_case_v1(
    appeal: &VictimReliefAppealV1,
    appeal_key: Pubkey,
    victim_relief_case: &VictimReliefCaseV1,
    victim_relief_case_key: Pubkey,
    config_key: Pubkey,
    policy_key: Pubkey,
    original_evidence_snapshot_key: Pubkey,
    original_decision_record_key: Pubkey,
    max_evidence_items: u32,
) -> Result<()> {
    require!(
        appeal_key != Pubkey::default(),
        CustomError::VictimReliefAppealTargetMismatch
    );
    require!(
        appeal.victim_relief_case == victim_relief_case_key
            && appeal.config == config_key
            && appeal.policy == policy_key
            && appeal.policy_version == victim_relief_case.policy_version
            && appeal.claimant == victim_relief_case.claimant
            && appeal.original_evidence_snapshot == original_evidence_snapshot_key
            && appeal.original_decision_record == original_decision_record_key
            && appeal.original_governance_proposal == victim_relief_case.decision_proposal
            && appeal.original_execution_queue_item == victim_relief_case.decision_queue
            && appeal.schema_version == VICTIM_RELIEF_SCHEMA_VERSION_V1,
        CustomError::VictimReliefAppealTargetMismatch
    );
    validate_victim_relief_appeal_evidence_v1(
        appeal.appeal_evidence_root,
        appeal.appeal_evidence_count,
        max_evidence_items,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn validate_victim_relief_appeal_execution_context_v1(
    security_governance_config: &GovernanceConfigV1,
    security_governance_config_key: Pubkey,
    protocol_module_registry: &ProtocolModuleRegistryV1,
    protocol_module_registry_key: Pubkey,
    governance_proposal: &GovernanceProposalV1,
    governance_proposal_key: Pubkey,
    governance_proposal_action: &GovernanceProposalActionV1,
    governance_proposal_action_key: Pubkey,
    governance_decision_adapter: &UniversalGovernanceDecisionAdapterV1,
    proposal_decision: &ProposalDecisionV1,
    proposal_decision_key: Pubkey,
    execution_queue_item: &ExecutionQueueItemV1,
    execution_queue_item_key: Pubkey,
    config: &VictimReliefConfigV1,
    config_key: Pubkey,
    policy: &VictimReliefPolicyV1,
    policy_key: Pubkey,
    victim_relief_case: &VictimReliefCaseV1,
    victim_relief_case_key: Pubkey,
    appeal: &VictimReliefAppealV1,
    appeal_key: Pubkey,
    original_evidence_snapshot: &VictimReliefEvidenceSnapshotV1,
    original_evidence_snapshot_key: Pubkey,
    original_decision_record: &VictimReliefDecisionExecutionRecordV1,
    original_decision_record_key: Pubkey,
    treasury_config: &TreasuryConfigV2,
    treasury_config_key: Pubkey,
    relief_usdc_vault_key: Pubkey,
    relief_usdc_vault: &TokenAccount,
    vault_authority: Pubkey,
    usdc_mint: Pubkey,
    expected_action: GovernanceActionTypeV1,
) -> Result<VictimReliefAppealDecisionValidationResultV1> {
    require!(
        expected_action == GovernanceActionTypeV1::VictimReliefUpholdAppeal
            || expected_action == GovernanceActionTypeV1::VictimReliefOverturnAppeal,
        CustomError::VictimReliefAppealActionMismatch
    );
    require!(
        governance_proposal.status == GovernanceProposalStatusV1::Passed,
        CustomError::InvalidGovernanceProposal
    );
    validate_governance_proposal_action_v1(
        governance_proposal,
        governance_proposal_action,
        governance_proposal_key,
    )?;
    require!(
        governance_proposal_action_key != Pubkey::default(),
        CustomError::GovernanceProposalActionMissing
    );
    require!(
        governance_proposal_action.action_type == expected_action,
        CustomError::VictimReliefAppealActionMismatch
    );
    require!(
        governance_proposal_action.module_id == ProtocolModuleIdV1::VictimRelief,
        CustomError::VictimReliefAppealActionMismatch
    );
    require_keys_eq!(
        governance_proposal_action.target_program,
        crate::ID,
        CustomError::VictimReliefAppealTargetMismatch
    );
    require_keys_eq!(
        governance_proposal_action.target_account,
        appeal_key,
        CustomError::VictimReliefAppealTargetMismatch
    );
    validate_protocol_module_registry_v1(
        protocol_module_registry,
        protocol_module_registry_key,
        security_governance_config_key,
        ProtocolModuleIdV1::VictimRelief,
        crate::ID,
    )?;
    require_keys_eq!(
        config.security_governance_config,
        security_governance_config_key,
        CustomError::ProtocolModuleGovernanceConfigMismatch
    );
    require!(
        !security_governance_config.is_paused,
        CustomError::SecurityLayerPaused
    );
    require!(!config.paused, CustomError::VictimReliefPaused);

    require!(
        victim_relief_case.status == VictimReliefCaseStatusV1::AppealPending
            && appeal.status == VictimReliefAppealStatusV1::Pending,
        CustomError::VictimReliefAppealStatusMismatch
    );
    require_keys_eq!(
        victim_relief_case.active_appeal,
        appeal_key,
        CustomError::VictimReliefAppealTargetMismatch
    );
    require_keys_eq!(
        victim_relief_case.config,
        config_key,
        CustomError::InvalidVictimReliefConfig
    );
    require_keys_eq!(
        victim_relief_case.policy,
        policy_key,
        CustomError::InvalidVictimReliefPolicyVersion
    );
    require!(
        victim_relief_case.policy_version == policy.policy_version
            && policy.schema_version == VICTIM_RELIEF_SCHEMA_VERSION_V1
            && policy.config == config_key,
        CustomError::InvalidVictimReliefPolicyVersion
    );
    validate_victim_relief_snapshot_matches_case_v1(
        original_evidence_snapshot,
        original_evidence_snapshot_key,
        victim_relief_case,
        victim_relief_case_key,
        config_key,
        policy_key,
    )?;
    validate_original_victim_relief_reject_record_v1(
        original_decision_record,
        original_decision_record_key,
        victim_relief_case,
        victim_relief_case_key,
        original_evidence_snapshot_key,
    )?;
    validate_victim_relief_appeal_matches_case_v1(
        appeal,
        appeal_key,
        victim_relief_case,
        victim_relief_case_key,
        config_key,
        policy_key,
        original_evidence_snapshot_key,
        original_decision_record_key,
        policy.max_evidence_items,
    )?;
    require_keys_eq!(
        config.treasury_config,
        treasury_config_key,
        CustomError::InvalidVictimReliefConfig
    );
    require!(
        config.usdc_mint == treasury_config.usdc_mint
            && config.usdc_mint == victim_relief_case.usdc_mint
            && usdc_mint == treasury_config.usdc_mint,
        CustomError::InvalidMint
    );
    let (expected_relief_vault, _) =
        Pubkey::find_program_address(&[RELIEF_USDC_VAULT_SEED], &crate::ID);
    require_keys_eq!(
        relief_usdc_vault_key,
        expected_relief_vault,
        CustomError::VictimReliefReliefVaultMismatch
    );
    let (expected_vault_authority, _) =
        Pubkey::find_program_address(&[VAULT_AUTHORITY_V2_SEED], &crate::ID);
    require_keys_eq!(
        vault_authority,
        expected_vault_authority,
        CustomError::VictimReliefReliefVaultMismatch
    );
    require!(
        relief_usdc_vault.mint == treasury_config.usdc_mint
            && relief_usdc_vault.owner == expected_vault_authority,
        CustomError::VictimReliefReliefVaultMismatch
    );

    let approved_amount_usdc = match expected_action {
        GovernanceActionTypeV1::VictimReliefUpholdAppeal => 0,
        GovernanceActionTypeV1::VictimReliefOverturnAppeal => {
            derive_victim_relief_approved_amount_v1(victim_relief_case, policy, policy_key)?
        }
        _ => return err!(CustomError::VictimReliefAppealActionMismatch),
    };
    let execution_type = match expected_action {
        GovernanceActionTypeV1::VictimReliefUpholdAppeal => {
            VictimReliefAppealExecutionTypeV1::Uphold
        }
        GovernanceActionTypeV1::VictimReliefOverturnAppeal => {
            VictimReliefAppealExecutionTypeV1::Overturn
        }
        _ => return err!(CustomError::VictimReliefAppealActionMismatch),
    };
    let parameters = build_victim_relief_appeal_decision_parameters_v1(
        config_key,
        policy_key,
        victim_relief_case_key,
        appeal_key,
        original_evidence_snapshot_key,
        original_decision_record_key,
        victim_relief_case,
        appeal,
        original_evidence_snapshot,
        treasury_config_key,
        relief_usdc_vault_key,
        expected_action,
        approved_amount_usdc,
        governance_proposal.proposal_id,
    );
    let parameters_hash = hash_victim_relief_appeal_decision_parameters_v1(&parameters)?;
    require!(
        governance_proposal_action.parameters_hash == parameters_hash,
        CustomError::VictimReliefAppealParametersMismatch
    );

    let governance_payload = GovernancePayloadV1 {
        schema_version: GOVERNANCE_PAYLOAD_V1_SCHEMA_VERSION,
        action_type: governance_proposal_action.action_type,
        module_id: governance_proposal_action.module_id,
        target_program: governance_proposal_action.target_program,
        target_account: governance_proposal_action.target_account,
        parameters_hash: governance_proposal_action.parameters_hash,
        evidence_hash: governance_proposal_action.evidence_hash,
        created_at: governance_proposal_action.created_at,
    };
    let canonical_governance_payload_hash = hash_governance_payload_v1(&governance_payload)?;
    require!(
        canonical_governance_payload_hash == governance_proposal_action.canonical_payload_hash
            && governance_proposal.payload_hash == canonical_governance_payload_hash,
        CustomError::VictimReliefAppealParametersMismatch
    );

    let expected_security_action = map_governance_action_to_security_action(expected_action)?;
    require!(
        governance_decision_adapter.governance_proposal == governance_proposal_key
            && governance_decision_adapter.proposal_decision == proposal_decision_key
            && governance_decision_adapter.action_type == expected_security_action
            && governance_decision_adapter.target_program == crate::ID
            && governance_decision_adapter.target_account == appeal_key
            && governance_decision_adapter.payload_hash == canonical_governance_payload_hash
            && governance_decision_adapter.executed,
        CustomError::InvalidGovernanceDecisionAdapter
    );
    require!(
        proposal_decision.proposal_id == governance_proposal.proposal_id
            && proposal_decision.proposer == governance_proposal.proposer
            && proposal_decision.proposal_type
                == security_proposal_type_for_action(expected_security_action)?
            && proposal_decision.decision == ProposalDecision::Approved,
        CustomError::ProposalNotApproved
    );
    require!(
        execution_queue_item.proposal_id == governance_proposal.proposal_id
            && execution_queue_item.action_type == expected_security_action
            && execution_queue_item.target_program == crate::ID
            && execution_queue_item.target_account == appeal_key
            && execution_queue_item.decision == ProposalDecision::Approved
            && execution_queue_item.status == ExecutionStatus::Executed
            && execution_queue_item.payload_hash == canonical_governance_payload_hash
            && execution_queue_item_key != Pubkey::default(),
        CustomError::InvalidExecutionStatus
    );

    Ok(VictimReliefAppealDecisionValidationResultV1 {
        approved_amount_usdc,
        parameters_hash,
        canonical_governance_payload_hash,
        action_type: expected_action,
        execution_type,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn record_victim_relief_appeal_decision_execution_record_v1(
    record: &mut VictimReliefAppealDecisionExecutionRecordV1,
    queue_key: Pubkey,
    proposal_decision_key: Pubkey,
    governance_proposal_key: Pubkey,
    governance_proposal_action_key: Pubkey,
    module_registry_key: Pubkey,
    config_key: Pubkey,
    policy_key: Pubkey,
    case_key: Pubkey,
    appeal_key: Pubkey,
    original_decision_record_key: Pubkey,
    original_evidence_snapshot_key: Pubkey,
    case_status_before: VictimReliefCaseStatusV1,
    case_status_after: VictimReliefCaseStatusV1,
    appeal_status_before: VictimReliefAppealStatusV1,
    appeal_status_after: VictimReliefAppealStatusV1,
    victim_relief_case: &VictimReliefCaseV1,
    validation: VictimReliefAppealDecisionValidationResultV1,
    executor: Pubkey,
    now: i64,
    bump: u8,
) -> Result<()> {
    require!(
        record.execution_queue_item == Pubkey::default(),
        CustomError::VictimReliefAppealExecutionAlreadyCompleted
    );
    record.execution_queue_item = queue_key;
    record.proposal_decision = proposal_decision_key;
    record.governance_proposal = governance_proposal_key;
    record.governance_proposal_action = governance_proposal_action_key;
    record.module_registry = module_registry_key;
    record.config = config_key;
    record.policy = policy_key;
    record.victim_relief_case = case_key;
    record.victim_relief_appeal = appeal_key;
    record.original_decision_record = original_decision_record_key;
    record.original_evidence_snapshot = original_evidence_snapshot_key;
    record.execution_type = validation.execution_type;
    record.governance_action_type = validation.action_type;
    record.case_status_before = case_status_before;
    record.case_status_after = case_status_after;
    record.appeal_status_before = appeal_status_before;
    record.appeal_status_after = appeal_status_after;
    record.claimed_amount_usdc = victim_relief_case.claimed_amount_usdc;
    record.approved_amount_usdc = validation.approved_amount_usdc;
    record.recipient_owner = victim_relief_case.recipient_owner;
    record.recipient_token_account = victim_relief_case.recipient_token_account;
    record.parameters_hash = validation.parameters_hash;
    record.canonical_governance_payload_hash = validation.canonical_governance_payload_hash;
    record.executor = executor;
    record.executed_at = now;
    record.schema_version = VICTIM_RELIEF_DECISION_SCHEMA_VERSION;
    record.bump = bump;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn record_uphold_victim_relief_appeal_v1(
    victim_relief_case: &mut VictimReliefCaseV1,
    appeal: &mut VictimReliefAppealV1,
    record: &mut VictimReliefAppealDecisionExecutionRecordV1,
    config_key: Pubkey,
    policy_key: Pubkey,
    governance_proposal_key: Pubkey,
    governance_proposal_action_key: Pubkey,
    proposal_decision_key: Pubkey,
    queue_key: Pubkey,
    module_registry_key: Pubkey,
    case_key: Pubkey,
    appeal_key: Pubkey,
    snapshot_key: Pubkey,
    original_decision_record_key: Pubkey,
    executor: Pubkey,
    validation: VictimReliefAppealDecisionValidationResultV1,
    now: i64,
    record_bump: u8,
) -> Result<()> {
    require!(
        validation.execution_type == VictimReliefAppealExecutionTypeV1::Uphold
            && validation.approved_amount_usdc == 0,
        CustomError::VictimReliefAppealActionMismatch
    );
    let case_status_before = victim_relief_case.status;
    let appeal_status_before = appeal.status;
    require!(
        case_status_before == VictimReliefCaseStatusV1::AppealPending
            && appeal_status_before == VictimReliefAppealStatusV1::Pending,
        CustomError::VictimReliefAppealStatusMismatch
    );
    require!(
        record.execution_queue_item == Pubkey::default(),
        CustomError::VictimReliefAppealExecutionAlreadyCompleted
    );

    appeal.status = VictimReliefAppealStatusV1::Upheld;
    appeal.decision_proposal = governance_proposal_key;
    appeal.decision_queue = queue_key;
    appeal.resolved_at = now;

    victim_relief_case.status = VictimReliefCaseStatusV1::AppealUpheld;
    victim_relief_case.active_appeal = Pubkey::default();
    victim_relief_case.approved_amount_usdc = 0;
    victim_relief_case.updated_at = now;

    record_victim_relief_appeal_decision_execution_record_v1(
        record,
        queue_key,
        proposal_decision_key,
        governance_proposal_key,
        governance_proposal_action_key,
        module_registry_key,
        config_key,
        policy_key,
        case_key,
        appeal_key,
        original_decision_record_key,
        snapshot_key,
        case_status_before,
        VictimReliefCaseStatusV1::AppealUpheld,
        appeal_status_before,
        VictimReliefAppealStatusV1::Upheld,
        victim_relief_case,
        validation,
        executor,
        now,
        record_bump,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn record_overturn_victim_relief_appeal_v1(
    claimant_state: &mut VictimReliefClaimantStateV1,
    victim_relief_case: &mut VictimReliefCaseV1,
    appeal: &mut VictimReliefAppealV1,
    payout_request: &mut ReliefPayoutRequestV1,
    record: &mut VictimReliefAppealDecisionExecutionRecordV1,
    config_key: Pubkey,
    policy_key: Pubkey,
    governance_proposal_key: Pubkey,
    governance_proposal_action_key: Pubkey,
    proposal_decision_key: Pubkey,
    queue_key: Pubkey,
    module_registry_key: Pubkey,
    case_key: Pubkey,
    appeal_key: Pubkey,
    snapshot_key: Pubkey,
    original_decision_record_key: Pubkey,
    treasury_config_key: Pubkey,
    relief_vault_key: Pubkey,
    executor: Pubkey,
    validation: VictimReliefAppealDecisionValidationResultV1,
    now: i64,
    payout_bump: u8,
    record_bump: u8,
) -> Result<()> {
    require!(
        validation.execution_type == VictimReliefAppealExecutionTypeV1::Overturn
            && validation.approved_amount_usdc > 0,
        CustomError::VictimReliefApprovedAmountMismatch
    );
    require!(
        payout_request.victim_relief_case == Pubkey::default(),
        CustomError::VictimReliefAppealPayoutRequestAlreadyExists
    );
    let case_status_before = victim_relief_case.status;
    let appeal_status_before = appeal.status;
    require!(
        case_status_before == VictimReliefCaseStatusV1::AppealPending
            && appeal_status_before == VictimReliefAppealStatusV1::Pending,
        CustomError::VictimReliefAppealStatusMismatch
    );
    require!(
        record.execution_queue_item == Pubkey::default(),
        CustomError::VictimReliefAppealExecutionAlreadyCompleted
    );

    claimant_state.active_case_count = claimant_state
        .active_case_count
        .checked_add(1)
        .ok_or(CustomError::MathOverflow)?;
    claimant_state.updated_at = now;

    appeal.status = VictimReliefAppealStatusV1::Overturned;
    appeal.decision_proposal = governance_proposal_key;
    appeal.decision_queue = queue_key;
    appeal.resolved_at = now;

    victim_relief_case.approved_amount_usdc = validation.approved_amount_usdc;
    victim_relief_case.status = VictimReliefCaseStatusV1::PayoutQueued;
    victim_relief_case.active_appeal = Pubkey::default();
    victim_relief_case.decision_proposal = governance_proposal_key;
    victim_relief_case.decision_queue = queue_key;
    victim_relief_case.updated_at = now;

    record_relief_payout_request_v1(
        payout_request,
        case_key,
        config_key,
        policy_key,
        victim_relief_case,
        governance_proposal_key,
        proposal_decision_key,
        queue_key,
        snapshot_key,
        treasury_config_key,
        relief_vault_key,
        validation.parameters_hash,
        now,
        payout_bump,
    )?;
    record_victim_relief_appeal_decision_execution_record_v1(
        record,
        queue_key,
        proposal_decision_key,
        governance_proposal_key,
        governance_proposal_action_key,
        module_registry_key,
        config_key,
        policy_key,
        case_key,
        appeal_key,
        original_decision_record_key,
        snapshot_key,
        case_status_before,
        VictimReliefCaseStatusV1::PayoutQueued,
        appeal_status_before,
        VictimReliefAppealStatusV1::Overturned,
        victim_relief_case,
        validation,
        executor,
        now,
        record_bump,
    )
}

pub fn victim_relief_payout_origin_stable_code_v1(origin: VictimReliefPayoutOriginV1) -> u8 {
    match origin {
        VictimReliefPayoutOriginV1::OriginalApprove => 1,
        VictimReliefPayoutOriginV1::AppealOverturn => 2,
    }
}

pub fn victim_relief_payout_origin_from_stable_code_v1(
    code: u8,
) -> Result<VictimReliefPayoutOriginV1> {
    match code {
        1 => Ok(VictimReliefPayoutOriginV1::OriginalApprove),
        2 => Ok(VictimReliefPayoutOriginV1::AppealOverturn),
        _ => err!(CustomError::InvalidVictimReliefPayoutOrigin),
    }
}

pub fn hash_victim_relief_payout_parameters_v1(
    parameters: &VictimReliefPayoutParametersV1,
) -> Result<[u8; 32]> {
    require!(
        parameters.schema_version == VICTIM_RELIEF_DECISION_SCHEMA_VERSION,
        CustomError::InvalidVictimReliefPayoutSchema
    );
    validate_victim_relief_payout_origin_action_v1(
        parameters.payout_origin,
        parameters.authorization_action_type,
    )?;
    require!(
        parameters.approved_amount_usdc > 0,
        CustomError::VictimReliefPayoutParametersMismatch
    );
    require!(
        parameters.authorization_parameters_hash != [0; 32],
        CustomError::VictimReliefPayoutParametersMismatch
    );

    hash_contributor_payload(&VictimReliefPayoutParametersHashEnvelopeV1 {
        domain_separator: VICTIM_RELIEF_PAYOUT_PARAMETERS_V1_DOMAIN_BYTES,
        parameters: *parameters,
    })
}

fn validate_victim_relief_payout_origin_action_v1(
    origin: VictimReliefPayoutOriginV1,
    action: GovernanceActionTypeV1,
) -> Result<()> {
    match origin {
        VictimReliefPayoutOriginV1::OriginalApprove => require!(
            action == GovernanceActionTypeV1::VictimReliefApproveCompensation,
            CustomError::VictimReliefPayoutActionMismatch
        ),
        VictimReliefPayoutOriginV1::AppealOverturn => require!(
            action == GovernanceActionTypeV1::VictimReliefOverturnAppeal,
            CustomError::VictimReliefPayoutActionMismatch
        ),
    }
    Ok(())
}

#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
fn build_victim_relief_payout_parameters_v1(
    payout_origin: VictimReliefPayoutOriginV1,
    authorization_action_type: GovernanceActionTypeV1,
    payout_request_key: Pubkey,
    payout_request: &ReliefPayoutRequestV1,
    governance_proposal_action_key: Pubkey,
    authorization_execution_record_key: Pubkey,
    vault_authority_v2: Pubkey,
) -> Result<VictimReliefPayoutParametersV1> {
    validate_victim_relief_payout_origin_action_v1(payout_origin, authorization_action_type)?;
    Ok(VictimReliefPayoutParametersV1 {
        schema_version: VICTIM_RELIEF_DECISION_SCHEMA_VERSION,
        payout_origin,
        payout_request: payout_request_key,
        victim_relief_case: payout_request.victim_relief_case,
        config: payout_request.config,
        policy: payout_request.policy,
        policy_version: payout_request.policy_version,
        authorization_action_type,
        governance_proposal: payout_request.governance_proposal,
        proposal_decision: payout_request.proposal_decision,
        execution_queue_item: payout_request.execution_queue_item,
        governance_proposal_action: governance_proposal_action_key,
        authorization_execution_record: authorization_execution_record_key,
        evidence_snapshot: payout_request.evidence_snapshot,
        approved_amount_usdc: payout_request.approved_amount_usdc,
        recipient_owner: payout_request.recipient_owner,
        recipient_token_account: payout_request.recipient_token_account,
        treasury_config: payout_request.treasury_config,
        relief_usdc_vault: payout_request.relief_usdc_vault,
        vault_authority_v2,
        usdc_mint: payout_request.usdc_mint,
        authorization_parameters_hash: payout_request.parameters_hash,
    })
}

#[allow(dead_code)]
fn build_original_approve_payout_parameters_v1(
    payout_request_key: Pubkey,
    payout_request: &ReliefPayoutRequestV1,
    governance_proposal_action_key: Pubkey,
    authorization_execution_record_key: Pubkey,
    vault_authority_v2: Pubkey,
) -> Result<VictimReliefPayoutParametersV1> {
    build_victim_relief_payout_parameters_v1(
        VictimReliefPayoutOriginV1::OriginalApprove,
        GovernanceActionTypeV1::VictimReliefApproveCompensation,
        payout_request_key,
        payout_request,
        governance_proposal_action_key,
        authorization_execution_record_key,
        vault_authority_v2,
    )
}

#[allow(dead_code)]
fn build_appeal_overturn_payout_parameters_v1(
    payout_request_key: Pubkey,
    payout_request: &ReliefPayoutRequestV1,
    governance_proposal_action_key: Pubkey,
    authorization_execution_record_key: Pubkey,
    vault_authority_v2: Pubkey,
) -> Result<VictimReliefPayoutParametersV1> {
    build_victim_relief_payout_parameters_v1(
        VictimReliefPayoutOriginV1::AppealOverturn,
        GovernanceActionTypeV1::VictimReliefOverturnAppeal,
        payout_request_key,
        payout_request,
        governance_proposal_action_key,
        authorization_execution_record_key,
        vault_authority_v2,
    )
}

#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
fn validate_victim_relief_original_approve_authorization_v1(
    governance_proposal_key: Pubkey,
    governance_proposal: &GovernanceProposalV1,
    governance_proposal_action_key: Pubkey,
    governance_proposal_action: &GovernanceProposalActionV1,
    proposal_decision_key: Pubkey,
    proposal_decision: &ProposalDecisionV1,
    execution_queue_item_key: Pubkey,
    execution_queue_item: &ExecutionQueueItemV1,
    authorization_record_key: Pubkey,
    authorization_record: &VictimReliefDecisionExecutionRecordV1,
    evidence_snapshot_key: Pubkey,
    evidence_snapshot: &VictimReliefEvidenceSnapshotV1,
    victim_relief_case_key: Pubkey,
    victim_relief_case: &VictimReliefCaseV1,
    payout_request_key: Pubkey,
    payout_request: &ReliefPayoutRequestV1,
) -> Result<()> {
    validate_victim_relief_governance_payout_authorization_v1(
        governance_proposal_key,
        governance_proposal,
        governance_proposal_action_key,
        governance_proposal_action,
        proposal_decision_key,
        proposal_decision,
        execution_queue_item_key,
        execution_queue_item,
        GovernanceActionTypeV1::VictimReliefApproveCompensation,
        victim_relief_case_key,
    )?;

    let (expected_record, expected_bump) = Pubkey::find_program_address(
        &[
            VICTIM_RELIEF_DECISION_EXECUTION_RECORD_V1_SEED,
            execution_queue_item_key.as_ref(),
        ],
        &crate::ID,
    );
    require_keys_eq!(
        expected_record,
        authorization_record_key,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require!(
        authorization_record.bump == expected_bump,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require!(
        authorization_record.execution_type == VictimReliefDecisionExecutionTypeV1::Approve,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require!(
        authorization_record.governance_action_type
            == GovernanceActionTypeV1::VictimReliefApproveCompensation,
        CustomError::VictimReliefPayoutActionMismatch
    );
    require_keys_eq!(
        authorization_record.victim_relief_case,
        victim_relief_case_key,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require_keys_eq!(
        authorization_record.evidence_snapshot,
        evidence_snapshot_key,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require!(
        authorization_record.case_status_after == VictimReliefCaseStatusV1::PayoutQueued,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require!(
        authorization_record.approved_amount_usdc == payout_request.approved_amount_usdc,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    validate_victim_relief_authorization_receipt_common_v1(
        authorization_record.proposal_decision,
        authorization_record.governance_proposal,
        authorization_record.governance_proposal_action,
        authorization_record.execution_queue_item,
        authorization_record.config,
        authorization_record.policy,
        authorization_record.recipient_owner,
        authorization_record.recipient_token_account,
        authorization_record.parameters_hash,
        authorization_record.canonical_governance_payload_hash,
        proposal_decision_key,
        governance_proposal_key,
        governance_proposal_action_key,
        execution_queue_item_key,
        victim_relief_case,
        payout_request,
        governance_proposal_action,
    )?;
    validate_victim_relief_payout_request_and_snapshot_v1(
        payout_request_key,
        payout_request,
        victim_relief_case_key,
        victim_relief_case,
        evidence_snapshot_key,
        evidence_snapshot,
    )
}

#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
fn validate_victim_relief_appeal_overturn_authorization_v1(
    governance_proposal_key: Pubkey,
    governance_proposal: &GovernanceProposalV1,
    governance_proposal_action_key: Pubkey,
    governance_proposal_action: &GovernanceProposalActionV1,
    proposal_decision_key: Pubkey,
    proposal_decision: &ProposalDecisionV1,
    execution_queue_item_key: Pubkey,
    execution_queue_item: &ExecutionQueueItemV1,
    appeal_key: Pubkey,
    appeal: &VictimReliefAppealV1,
    authorization_record_key: Pubkey,
    authorization_record: &VictimReliefAppealDecisionExecutionRecordV1,
    original_decision_record_key: Pubkey,
    original_decision_record: &VictimReliefDecisionExecutionRecordV1,
    original_evidence_snapshot_key: Pubkey,
    original_evidence_snapshot: &VictimReliefEvidenceSnapshotV1,
    victim_relief_case_key: Pubkey,
    victim_relief_case: &VictimReliefCaseV1,
    payout_request_key: Pubkey,
    payout_request: &ReliefPayoutRequestV1,
) -> Result<()> {
    validate_victim_relief_governance_payout_authorization_v1(
        governance_proposal_key,
        governance_proposal,
        governance_proposal_action_key,
        governance_proposal_action,
        proposal_decision_key,
        proposal_decision,
        execution_queue_item_key,
        execution_queue_item,
        GovernanceActionTypeV1::VictimReliefOverturnAppeal,
        appeal_key,
    )?;

    let (expected_record, expected_bump) = Pubkey::find_program_address(
        &[
            VICTIM_RELIEF_APPEAL_DECISION_EXECUTION_RECORD_V1_SEED,
            execution_queue_item_key.as_ref(),
        ],
        &crate::ID,
    );
    require_keys_eq!(
        expected_record,
        authorization_record_key,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require!(
        authorization_record.bump == expected_bump,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require!(
        authorization_record.execution_type == VictimReliefAppealExecutionTypeV1::Overturn,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require!(
        authorization_record.governance_action_type
            == GovernanceActionTypeV1::VictimReliefOverturnAppeal,
        CustomError::VictimReliefPayoutActionMismatch
    );
    require_keys_eq!(
        authorization_record.victim_relief_case,
        victim_relief_case_key,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require_keys_eq!(
        authorization_record.victim_relief_appeal,
        appeal_key,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require_keys_eq!(
        authorization_record.original_decision_record,
        original_decision_record_key,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require_keys_eq!(
        authorization_record.original_evidence_snapshot,
        original_evidence_snapshot_key,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require!(
        authorization_record.case_status_after == VictimReliefCaseStatusV1::PayoutQueued
            && authorization_record.appeal_status_after == VictimReliefAppealStatusV1::Overturned,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require!(
        authorization_record.approved_amount_usdc == payout_request.approved_amount_usdc,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require_keys_eq!(
        appeal.victim_relief_case,
        victim_relief_case_key,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require!(
        appeal.status == VictimReliefAppealStatusV1::Overturned,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require_keys_eq!(
        appeal.decision_proposal,
        governance_proposal_key,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require_keys_eq!(
        appeal.decision_queue,
        execution_queue_item_key,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require_keys_eq!(
        appeal.original_decision_record,
        original_decision_record_key,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require_keys_eq!(
        appeal.original_evidence_snapshot,
        original_evidence_snapshot_key,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require!(
        victim_relief_case.active_appeal == Pubkey::default(),
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require!(
        original_decision_record.execution_type == VictimReliefDecisionExecutionTypeV1::Reject
            && original_decision_record.case_status_after == VictimReliefCaseStatusV1::Rejected
            && original_decision_record.approved_amount_usdc == 0,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require_keys_eq!(
        original_decision_record.victim_relief_case,
        victim_relief_case_key,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require_keys_eq!(
        original_decision_record.evidence_snapshot,
        original_evidence_snapshot_key,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require!(
        original_evidence_snapshot.victim_relief_case == victim_relief_case_key,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    validate_victim_relief_authorization_receipt_common_v1(
        authorization_record.proposal_decision,
        authorization_record.governance_proposal,
        authorization_record.governance_proposal_action,
        authorization_record.execution_queue_item,
        authorization_record.config,
        authorization_record.policy,
        authorization_record.recipient_owner,
        authorization_record.recipient_token_account,
        authorization_record.parameters_hash,
        authorization_record.canonical_governance_payload_hash,
        proposal_decision_key,
        governance_proposal_key,
        governance_proposal_action_key,
        execution_queue_item_key,
        victim_relief_case,
        payout_request,
        governance_proposal_action,
    )?;
    validate_victim_relief_payout_request_and_snapshot_v1(
        payout_request_key,
        payout_request,
        victim_relief_case_key,
        victim_relief_case,
        original_evidence_snapshot_key,
        original_evidence_snapshot,
    )
}

#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
fn validate_victim_relief_governance_payout_authorization_v1(
    governance_proposal_key: Pubkey,
    governance_proposal: &GovernanceProposalV1,
    governance_proposal_action_key: Pubkey,
    governance_proposal_action: &GovernanceProposalActionV1,
    proposal_decision_key: Pubkey,
    proposal_decision: &ProposalDecisionV1,
    execution_queue_item_key: Pubkey,
    execution_queue_item: &ExecutionQueueItemV1,
    expected_action: GovernanceActionTypeV1,
    expected_target: Pubkey,
) -> Result<()> {
    validate_governance_proposal_action_v1(
        governance_proposal,
        governance_proposal_action,
        governance_proposal_key,
    )?;
    require!(
        governance_proposal.status == GovernanceProposalStatusV1::Passed,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require!(
        governance_proposal_action.action_type == expected_action,
        CustomError::VictimReliefPayoutActionMismatch
    );
    require!(
        governance_proposal_action.module_id == ProtocolModuleIdV1::VictimRelief,
        CustomError::VictimReliefPayoutActionMismatch
    );
    require_keys_eq!(
        governance_proposal_action.target_program,
        crate::ID,
        CustomError::VictimReliefPayoutActionMismatch
    );
    require_keys_eq!(
        governance_proposal_action.target_account,
        expected_target,
        CustomError::VictimReliefPayoutActionMismatch
    );
    let expected_security_action = map_governance_action_to_security_action(expected_action)?;
    require!(
        proposal_decision.proposal_id == governance_proposal.proposal_id
            && proposal_decision.proposer == governance_proposal.proposer
            && proposal_decision.decision == ProposalDecision::Approved
            && proposal_decision.proposal_type
                == security_proposal_type_for_action(expected_security_action)?,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require!(
        execution_queue_item.proposal_id == governance_proposal.proposal_id
            && execution_queue_item.proposer == governance_proposal.proposer
            && execution_queue_item.decision == ProposalDecision::Approved
            && execution_queue_item.status == ExecutionStatus::Executed,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require!(
        execution_queue_item.action_type == expected_security_action,
        CustomError::VictimReliefPayoutActionMismatch
    );
    require_keys_eq!(
        execution_queue_item.target_program,
        governance_proposal_action.target_program,
        CustomError::VictimReliefPayoutActionMismatch
    );
    require_keys_eq!(
        execution_queue_item.target_account,
        governance_proposal_action.target_account,
        CustomError::VictimReliefPayoutActionMismatch
    );
    require!(
        execution_queue_item.payload_hash == governance_proposal_action.canonical_payload_hash,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require!(
        proposal_decision_key != Pubkey::default()
            && execution_queue_item_key != Pubkey::default()
            && governance_proposal_action_key != Pubkey::default(),
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    Ok(())
}

#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
fn validate_victim_relief_authorization_receipt_common_v1(
    receipt_proposal_decision: Pubkey,
    receipt_governance_proposal: Pubkey,
    receipt_governance_proposal_action: Pubkey,
    receipt_execution_queue_item: Pubkey,
    receipt_config: Pubkey,
    receipt_policy: Pubkey,
    receipt_recipient_owner: Pubkey,
    receipt_recipient_token_account: Pubkey,
    receipt_parameters_hash: [u8; 32],
    receipt_canonical_governance_payload_hash: [u8; 32],
    proposal_decision_key: Pubkey,
    governance_proposal_key: Pubkey,
    governance_proposal_action_key: Pubkey,
    execution_queue_item_key: Pubkey,
    victim_relief_case: &VictimReliefCaseV1,
    payout_request: &ReliefPayoutRequestV1,
    governance_proposal_action: &GovernanceProposalActionV1,
) -> Result<()> {
    require_keys_eq!(
        receipt_proposal_decision,
        proposal_decision_key,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require_keys_eq!(
        receipt_governance_proposal,
        governance_proposal_key,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require_keys_eq!(
        receipt_governance_proposal_action,
        governance_proposal_action_key,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require_keys_eq!(
        receipt_execution_queue_item,
        execution_queue_item_key,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require_keys_eq!(
        receipt_config,
        victim_relief_case.config,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require_keys_eq!(
        receipt_policy,
        victim_relief_case.policy,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require_keys_eq!(
        receipt_recipient_owner,
        payout_request.recipient_owner,
        CustomError::VictimReliefPayoutRecipientMismatch
    );
    require_keys_eq!(
        receipt_recipient_token_account,
        payout_request.recipient_token_account,
        CustomError::VictimReliefPayoutRecipientMismatch
    );
    require!(
        receipt_parameters_hash == payout_request.parameters_hash
            && receipt_parameters_hash != [0; 32],
        CustomError::VictimReliefPayoutParametersMismatch
    );
    require!(
        receipt_canonical_governance_payload_hash
            == governance_proposal_action.canonical_payload_hash,
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    Ok(())
}

#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
fn validate_victim_relief_payout_request_and_snapshot_v1(
    payout_request_key: Pubkey,
    payout_request: &ReliefPayoutRequestV1,
    case_key: Pubkey,
    victim_relief_case: &VictimReliefCaseV1,
    evidence_snapshot_key: Pubkey,
    evidence_snapshot: &VictimReliefEvidenceSnapshotV1,
) -> Result<()> {
    let (expected_request, expected_bump) = Pubkey::find_program_address(
        &[RELIEF_PAYOUT_REQUEST_V1_SEED, case_key.as_ref()],
        &crate::ID,
    );
    require_keys_eq!(
        expected_request,
        payout_request_key,
        CustomError::VictimReliefPayoutRequestMismatch
    );
    require!(
        payout_request.bump == expected_bump,
        CustomError::VictimReliefPayoutRequestMismatch
    );
    require_keys_eq!(
        payout_request.victim_relief_case,
        case_key,
        CustomError::VictimReliefPayoutRequestMismatch
    );
    require!(
        payout_request.status == VictimReliefPayoutStatusV1::Approved
            && payout_request.executed_at == 0
            && payout_request.schema_version == VICTIM_RELIEF_DECISION_SCHEMA_VERSION,
        CustomError::VictimReliefPayoutStatusMismatch
    );
    require!(
        victim_relief_case.status == VictimReliefCaseStatusV1::PayoutQueued
            && victim_relief_case.approved_amount_usdc > 0,
        CustomError::VictimReliefPayoutStatusMismatch
    );
    require!(
        victim_relief_case.active_appeal == Pubkey::default(),
        CustomError::VictimReliefPayoutStatusMismatch
    );
    require!(
        payout_request.approved_amount_usdc == victim_relief_case.approved_amount_usdc,
        CustomError::VictimReliefPayoutParametersMismatch
    );
    require_keys_eq!(
        payout_request.recipient_owner,
        victim_relief_case.recipient_owner,
        CustomError::VictimReliefPayoutRecipientMismatch
    );
    require_keys_eq!(
        payout_request.recipient_token_account,
        victim_relief_case.recipient_token_account,
        CustomError::VictimReliefPayoutRecipientMismatch
    );
    require_keys_eq!(
        payout_request.config,
        victim_relief_case.config,
        CustomError::VictimReliefPayoutRequestMismatch
    );
    require_keys_eq!(
        payout_request.policy,
        victim_relief_case.policy,
        CustomError::VictimReliefPayoutRequestMismatch
    );
    require!(
        payout_request.policy_version == victim_relief_case.policy_version,
        CustomError::VictimReliefPayoutRequestMismatch
    );
    require_keys_eq!(
        payout_request.evidence_snapshot,
        evidence_snapshot_key,
        CustomError::VictimReliefPayoutRequestMismatch
    );
    require!(
        evidence_snapshot.victim_relief_case == case_key
            && evidence_snapshot.config == victim_relief_case.config
            && evidence_snapshot.policy == victim_relief_case.policy
            && evidence_snapshot.policy_version == victim_relief_case.policy_version
            && evidence_snapshot.claimant == victim_relief_case.claimant
            && evidence_snapshot.subject_commitment == victim_relief_case.subject_commitment
            && evidence_snapshot.evidence_root == victim_relief_case.evidence_root
            && evidence_snapshot.evidence_count == victim_relief_case.evidence_count
            && evidence_snapshot.evidence_revision == victim_relief_case.evidence_revision
            && evidence_snapshot.claimed_amount_usdc == victim_relief_case.claimed_amount_usdc
            && evidence_snapshot.recipient_owner == victim_relief_case.recipient_owner
            && evidence_snapshot.recipient_token_account
                == victim_relief_case.recipient_token_account
            && evidence_snapshot.usdc_mint == victim_relief_case.usdc_mint,
        CustomError::VictimReliefEvidenceSnapshotMismatch
    );
    Ok(())
}

#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
fn validate_victim_relief_payout_common_v1(
    security_governance_config_key: Pubkey,
    security_governance_config: &GovernanceConfigV1,
    victim_relief_config_key: Pubkey,
    victim_relief_config: &VictimReliefConfigV1,
    policy_key: Pubkey,
    policy: &VictimReliefPolicyV1,
    claimant_state: &VictimReliefClaimantStateV1,
    case_key: Pubkey,
    victim_relief_case: &VictimReliefCaseV1,
    evidence_snapshot_key: Pubkey,
    evidence_snapshot: &VictimReliefEvidenceSnapshotV1,
    payout_request_key: Pubkey,
    payout_request: &ReliefPayoutRequestV1,
    treasury_config_key: Pubkey,
    treasury_config: &TreasuryConfigV2,
    relief_usdc_vault_key: Pubkey,
    relief_usdc_vault: &TokenAccount,
    vault_authority_v2_key: Pubkey,
    recipient_token_account_key: Pubkey,
    recipient_token_account: &TokenAccount,
    usdc_mint_key: Pubkey,
    usdc_mint: &Mint,
    payout_receipt_key: Pubkey,
    payout_receipt: &ReliefPayoutExecutionRecordV1,
    payout_origin: VictimReliefPayoutOriginV1,
    authorization_action_type: GovernanceActionTypeV1,
    governance_proposal_action_key: Pubkey,
    authorization_execution_record_key: Pubkey,
    authorization_amount_usdc: u64,
    authorization_parameters_hash: [u8; 32],
) -> Result<VictimReliefPayoutParametersV1> {
    validate_victim_relief_payout_origin_action_v1(payout_origin, authorization_action_type)?;
    require!(
        !victim_relief_config.paused && !security_governance_config.is_paused,
        CustomError::VictimReliefPayoutPaused
    );
    require_keys_eq!(
        victim_relief_config.security_governance_config,
        security_governance_config_key,
        CustomError::VictimReliefPayoutRequestMismatch
    );
    require_keys_eq!(
        victim_relief_case.config,
        victim_relief_config_key,
        CustomError::VictimReliefPayoutRequestMismatch
    );
    require_keys_eq!(
        payout_request.config,
        victim_relief_config_key,
        CustomError::VictimReliefPayoutRequestMismatch
    );
    require_keys_eq!(
        victim_relief_case.policy,
        policy_key,
        CustomError::VictimReliefPayoutRequestMismatch
    );
    require_keys_eq!(
        payout_request.policy,
        policy_key,
        CustomError::VictimReliefPayoutRequestMismatch
    );
    require!(
        victim_relief_case.policy_version == policy.policy_version
            && payout_request.policy_version == policy.policy_version
            && policy.schema_version == VICTIM_RELIEF_SCHEMA_VERSION_V1
            && policy.active,
        CustomError::InvalidVictimReliefPolicy
    );
    require_keys_eq!(
        policy.config,
        victim_relief_config_key,
        CustomError::InvalidVictimReliefPolicy
    );
    require_keys_eq!(
        victim_relief_config.treasury_config,
        treasury_config_key,
        CustomError::VictimReliefPayoutRequestMismatch
    );
    require_keys_eq!(
        victim_relief_config.usdc_mint,
        treasury_config.usdc_mint,
        CustomError::VictimReliefPayoutMintMismatch
    );
    validate_victim_relief_payout_request_and_snapshot_v1(
        payout_request_key,
        payout_request,
        case_key,
        victim_relief_case,
        evidence_snapshot_key,
        evidence_snapshot,
    )?;
    require!(
        authorization_amount_usdc == payout_request.approved_amount_usdc
            && authorization_parameters_hash == payout_request.parameters_hash
            && authorization_parameters_hash != [0; 32],
        CustomError::VictimReliefPayoutAuthorizationMismatch
    );
    require!(
        payout_request.governance_proposal != Pubkey::default()
            && payout_request.proposal_decision != Pubkey::default()
            && payout_request.execution_queue_item != Pubkey::default()
            && payout_request.parameters_hash != [0; 32],
        CustomError::VictimReliefPayoutRequestMismatch
    );
    require!(
        claimant_state.config == victim_relief_config_key
            && claimant_state.claimant == victim_relief_case.claimant
            && claimant_state.active_case_count > 0,
        CustomError::VictimReliefPayoutClaimantStateMismatch
    );
    require_keys_eq!(
        payout_request.treasury_config,
        treasury_config_key,
        CustomError::VictimReliefPayoutRequestMismatch
    );
    require_keys_eq!(
        payout_request.usdc_mint,
        treasury_config.usdc_mint,
        CustomError::VictimReliefPayoutMintMismatch
    );
    require_keys_eq!(
        usdc_mint_key,
        treasury_config.usdc_mint,
        CustomError::VictimReliefPayoutMintMismatch
    );
    require_keys_eq!(
        usdc_mint_key,
        victim_relief_config.usdc_mint,
        CustomError::VictimReliefPayoutMintMismatch
    );
    require_keys_eq!(
        usdc_mint_key,
        payout_request.usdc_mint,
        CustomError::VictimReliefPayoutMintMismatch
    );
    require!(
        usdc_mint.decimals == GREEN_LABEL_USDC_DECIMALS,
        CustomError::VictimReliefPayoutDecimalsMismatch
    );

    let (expected_relief_vault, _) =
        Pubkey::find_program_address(&[RELIEF_USDC_VAULT_SEED], &crate::ID);
    require_keys_eq!(
        relief_usdc_vault_key,
        expected_relief_vault,
        CustomError::VictimReliefPayoutVaultMismatch
    );
    require_keys_eq!(
        payout_request.relief_usdc_vault,
        relief_usdc_vault_key,
        CustomError::VictimReliefPayoutVaultMismatch
    );
    let (expected_vault_authority, _) =
        Pubkey::find_program_address(&[VAULT_AUTHORITY_V2_SEED], &crate::ID);
    require_keys_eq!(
        vault_authority_v2_key,
        expected_vault_authority,
        CustomError::VictimReliefPayoutVaultMismatch
    );
    require!(
        relief_usdc_vault.mint == treasury_config.usdc_mint
            && relief_usdc_vault.owner == vault_authority_v2_key,
        CustomError::VictimReliefPayoutVaultMismatch
    );
    require_keys_eq!(
        recipient_token_account_key,
        payout_request.recipient_token_account,
        CustomError::VictimReliefPayoutRecipientMismatch
    );
    require!(
        recipient_token_account.owner == payout_request.recipient_owner
            && recipient_token_account.mint == payout_request.usdc_mint,
        CustomError::VictimReliefPayoutRecipientMismatch
    );
    require!(
        recipient_token_account_key != relief_usdc_vault_key,
        CustomError::VictimReliefPayoutRecipientMismatch
    );
    let (builders_vault, _) = Pubkey::find_program_address(&[BUILDERS_USDC_VAULT_SEED], &crate::ID);
    let (buyback_vault, _) = Pubkey::find_program_address(&[BUYBACK_USDC_VAULT_SEED], &crate::ID);
    let (staking_vault, _) = Pubkey::find_program_address(&[STAKING_USDC_VAULT_SEED], &crate::ID);
    require!(
        recipient_token_account_key != builders_vault
            && recipient_token_account_key != buyback_vault
            && recipient_token_account_key != staking_vault,
        CustomError::VictimReliefPayoutRecipientMismatch
    );
    require!(
        relief_usdc_vault.amount >= payout_request.approved_amount_usdc,
        CustomError::VictimReliefPayoutInsufficientFunds
    );

    let (expected_receipt, _) = Pubkey::find_program_address(
        &[
            RELIEF_PAYOUT_EXECUTION_RECORD_V1_SEED,
            payout_request_key.as_ref(),
        ],
        &crate::ID,
    );
    require_keys_eq!(
        payout_receipt_key,
        expected_receipt,
        CustomError::VictimReliefPayoutRequestMismatch
    );
    require!(
        payout_receipt.payout_request == Pubkey::default() && payout_receipt.bump == 0,
        CustomError::VictimReliefPayoutReceiptAlreadyExists
    );

    match payout_origin {
        VictimReliefPayoutOriginV1::OriginalApprove => {
            require!(
                authorization_action_type
                    == GovernanceActionTypeV1::VictimReliefApproveCompensation,
                CustomError::VictimReliefPayoutActionMismatch
            );
            build_original_approve_payout_parameters_v1(
                payout_request_key,
                payout_request,
                governance_proposal_action_key,
                authorization_execution_record_key,
                vault_authority_v2_key,
            )
        }
        VictimReliefPayoutOriginV1::AppealOverturn => {
            require!(
                authorization_action_type == GovernanceActionTypeV1::VictimReliefOverturnAppeal,
                CustomError::VictimReliefPayoutActionMismatch
            );
            build_appeal_overturn_payout_parameters_v1(
                payout_request_key,
                payout_request,
                governance_proposal_action_key,
                authorization_execution_record_key,
                vault_authority_v2_key,
            )
        }
    }
}

#[allow(dead_code)]
fn record_relief_payout_execution_v1(
    receipt: &mut ReliefPayoutExecutionRecordV1,
    parameters: VictimReliefPayoutParametersV1,
    executor: Pubkey,
    executed_at: i64,
    bump: u8,
) -> Result<[u8; 32]> {
    require!(
        receipt.payout_request == Pubkey::default(),
        CustomError::VictimReliefPayoutReceiptAlreadyExists
    );
    require!(
        executed_at > 0,
        CustomError::InvalidVictimReliefPayoutSchema
    );
    let payout_parameters_hash = hash_victim_relief_payout_parameters_v1(&parameters)?;

    receipt.payout_request = parameters.payout_request;
    receipt.victim_relief_case = parameters.victim_relief_case;
    receipt.config = parameters.config;
    receipt.policy = parameters.policy;
    receipt.policy_version = parameters.policy_version;
    receipt.payout_origin = parameters.payout_origin;
    receipt.authorization_action_type = parameters.authorization_action_type;
    receipt.governance_proposal = parameters.governance_proposal;
    receipt.proposal_decision = parameters.proposal_decision;
    receipt.execution_queue_item = parameters.execution_queue_item;
    receipt.governance_proposal_action = parameters.governance_proposal_action;
    receipt.authorization_execution_record = parameters.authorization_execution_record;
    receipt.evidence_snapshot = parameters.evidence_snapshot;
    receipt.relief_usdc_vault = parameters.relief_usdc_vault;
    receipt.vault_authority_v2 = parameters.vault_authority_v2;
    receipt.recipient_owner = parameters.recipient_owner;
    receipt.recipient_token_account = parameters.recipient_token_account;
    receipt.amount_usdc = parameters.approved_amount_usdc;
    receipt.usdc_mint = parameters.usdc_mint;
    receipt.authorization_parameters_hash = parameters.authorization_parameters_hash;
    receipt.payout_parameters_hash = payout_parameters_hash;
    receipt.executor = executor;
    receipt.executed_at = executed_at;
    receipt.schema_version = VICTIM_RELIEF_DECISION_SCHEMA_VERSION;
    receipt.bump = bump;
    receipt.reserved = [0; 32];
    Ok(payout_parameters_hash)
}

#[allow(clippy::too_many_arguments)]
fn record_original_approved_victim_relief_payout_v1(
    claimant_state: &mut VictimReliefClaimantStateV1,
    victim_relief_case: &mut VictimReliefCaseV1,
    payout_request: &mut ReliefPayoutRequestV1,
    receipt: &mut ReliefPayoutExecutionRecordV1,
    parameters: VictimReliefPayoutParametersV1,
    executor: Pubkey,
    now: i64,
    receipt_bump: u8,
) -> Result<[u8; 32]> {
    require!(
        parameters.payout_origin == VictimReliefPayoutOriginV1::OriginalApprove
            && parameters.authorization_action_type
                == GovernanceActionTypeV1::VictimReliefApproveCompensation,
        CustomError::VictimReliefPayoutActionMismatch
    );
    require!(
        payout_request.status == VictimReliefPayoutStatusV1::Approved
            && payout_request.executed_at == 0,
        CustomError::VictimReliefPayoutStatusMismatch
    );
    require!(
        victim_relief_case.status == VictimReliefCaseStatusV1::PayoutQueued
            && victim_relief_case.active_appeal == Pubkey::default(),
        CustomError::VictimReliefPayoutStatusMismatch
    );
    require!(
        payout_request.approved_amount_usdc > 0
            && payout_request.approved_amount_usdc == victim_relief_case.approved_amount_usdc
            && parameters.approved_amount_usdc == payout_request.approved_amount_usdc,
        CustomError::VictimReliefPayoutParametersMismatch
    );
    require!(
        payout_request.recipient_owner == victim_relief_case.recipient_owner
            && payout_request.recipient_token_account == victim_relief_case.recipient_token_account
            && parameters.recipient_owner == payout_request.recipient_owner
            && parameters.recipient_token_account == payout_request.recipient_token_account,
        CustomError::VictimReliefPayoutRecipientMismatch
    );
    require!(
        claimant_state.config == victim_relief_case.config
            && claimant_state.claimant == victim_relief_case.claimant
            && claimant_state.active_case_count > 0,
        CustomError::VictimReliefPayoutClaimantStateMismatch
    );

    payout_request.status = VictimReliefPayoutStatusV1::Executed;
    payout_request.executed_at = now;

    victim_relief_case.status = VictimReliefCaseStatusV1::Paid;
    victim_relief_case.updated_at = now;

    close_victim_relief_active_case_count_v1(claimant_state, now)?;
    record_relief_payout_execution_v1(receipt, parameters, executor, now, receipt_bump)
}

fn transfer_from_relief_usdc_vault_v1<'info>(
    token_program: Pubkey,
    relief_usdc_vault: AccountInfo<'info>,
    usdc_mint: AccountInfo<'info>,
    recipient_usdc_token_account: AccountInfo<'info>,
    vault_authority: AccountInfo<'info>,
    vault_authority_bump: u8,
    amount: u64,
    decimals: u8,
) -> Result<()> {
    require!(
        amount > 0,
        CustomError::VictimReliefPayoutParametersMismatch
    );
    require!(
        decimals == GREEN_LABEL_USDC_DECIMALS,
        CustomError::VictimReliefPayoutDecimalsMismatch
    );
    let signer_seeds: &[&[&[u8]]] = &[&[VAULT_AUTHORITY_V2_SEED, &[vault_authority_bump]]];
    let cpi_accounts = TransferChecked {
        from: relief_usdc_vault,
        mint: usdc_mint,
        to: recipient_usdc_token_account,
        authority: vault_authority,
    };
    let cpi_ctx = CpiContext::new_with_signer(token_program, cpi_accounts, signer_seeds);
    transfer_checked(cpi_ctx, amount, decimals)
}

fn validate_victim_relief_evidence_count_v1(
    evidence_count: u32,
    max_evidence_items: u32,
) -> Result<()> {
    require!(
        evidence_count > 0 && evidence_count <= max_evidence_items,
        CustomError::InvalidVictimReliefEvidenceCount
    );
    Ok(())
}

fn is_zero_32(value: &[u8; 32]) -> bool {
    value.iter().all(|byte| *byte == 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::governance_action_v1::{
        governance_action_stable_code_v1, map_governance_action_to_governance_proposal_type_v1,
        map_governance_action_to_module,
    };
    use anchor_lang::solana_program::program_option::COption;
    use anchor_lang::solana_program::program_pack::Pack;
    use anchor_lang::AccountDeserialize;
    use anchor_spl::token::spl_token::state::{
        Account as SplTokenAccount, AccountState, Mint as SplMint,
    };

    fn policy_parameters() -> VictimReliefPolicyParametersV1 {
        VictimReliefPolicyParametersV1 {
            min_claim_amount_usdc: 1_000_000,
            max_claim_amount_usdc: 1_000_000_000,
            max_payout_per_case_usdc: 500_000_000,
            evidence_window_seconds: 86_400,
            review_window_seconds: 172_800,
            appeal_window_seconds: 259_200,
            submission_cooldown_seconds: 3_600,
            max_evidence_items: 32,
            max_active_cases_per_claimant: 2,
        }
    }

    fn config_fixture(policy_key: Pubkey) -> VictimReliefConfigV1 {
        VictimReliefConfigV1 {
            authority: Pubkey::new_unique(),
            treasury_config: Pubkey::new_unique(),
            security_governance_config: Pubkey::new_unique(),
            usdc_mint: Pubkey::new_unique(),
            current_policy: policy_key,
            current_policy_version: VICTIM_RELIEF_POLICY_VERSION_V1,
            next_case_id: 1,
            paused: false,
            created_at: 100,
            schema_version: VICTIM_RELIEF_SCHEMA_VERSION_V1,
            bump: 1,
            reserved: [0; 32],
        }
    }

    fn policy_fixture(config_key: Pubkey) -> VictimReliefPolicyV1 {
        let params = policy_parameters();
        VictimReliefPolicyV1 {
            config: config_key,
            policy_version: VICTIM_RELIEF_POLICY_VERSION_V1,
            min_claim_amount_usdc: params.min_claim_amount_usdc,
            max_claim_amount_usdc: params.max_claim_amount_usdc,
            max_payout_per_case_usdc: params.max_payout_per_case_usdc,
            evidence_window_seconds: params.evidence_window_seconds,
            review_window_seconds: params.review_window_seconds,
            appeal_window_seconds: params.appeal_window_seconds,
            submission_cooldown_seconds: params.submission_cooldown_seconds,
            max_evidence_items: params.max_evidence_items,
            max_active_cases_per_claimant: params.max_active_cases_per_claimant,
            active: true,
            initialized_by: Pubkey::new_unique(),
            created_at: 100,
            schema_version: VICTIM_RELIEF_SCHEMA_VERSION_V1,
            bump: 2,
            reserved: [0; 32],
        }
    }

    fn claimant_state_fixture(config: Pubkey, claimant: Pubkey) -> VictimReliefClaimantStateV1 {
        VictimReliefClaimantStateV1 {
            config,
            claimant,
            active_case_count: 0,
            total_case_count: 0,
            last_case_id: 0,
            last_submitted_at: 0,
            created_at: 100,
            updated_at: 100,
            schema_version: VICTIM_RELIEF_SCHEMA_VERSION_V1,
            bump: 3,
        }
    }

    fn case_fixture(
        config: Pubkey,
        policy: Pubkey,
        claimant: Pubkey,
        usdc_mint: Pubkey,
    ) -> VictimReliefCaseV1 {
        VictimReliefCaseV1 {
            case_id: 1,
            config,
            policy,
            policy_version: VICTIM_RELIEF_POLICY_VERSION_V1,
            claimant,
            subject_commitment: [11; 32],
            evidence_root: [22; 32],
            evidence_count: 1,
            evidence_revision: 0,
            claimed_amount_usdc: 10_000_000,
            approved_amount_usdc: 0,
            recipient_owner: claimant,
            recipient_token_account: Pubkey::new_unique(),
            usdc_mint,
            status: VictimReliefCaseStatusV1::EvidencePeriod,
            active_appeal: Pubkey::default(),
            decision_proposal: Pubkey::default(),
            decision_queue: Pubkey::default(),
            submitted_at: 1_000,
            evidence_deadline: 2_000,
            review_deadline: 0,
            appeal_deadline: 0,
            updated_at: 1_000,
            schema_version: VICTIM_RELIEF_SCHEMA_VERSION_V1,
            bump: 4,
            reserved: [0; 64],
        }
    }

    fn empty_snapshot() -> VictimReliefEvidenceSnapshotV1 {
        VictimReliefEvidenceSnapshotV1 {
            victim_relief_case: Pubkey::default(),
            config: Pubkey::default(),
            policy: Pubkey::default(),
            policy_version: 0,
            claimant: Pubkey::default(),
            subject_commitment: [0; 32],
            evidence_root: [0; 32],
            evidence_count: 0,
            evidence_revision: 0,
            claimed_amount_usdc: 0,
            recipient_owner: Pubkey::default(),
            recipient_token_account: Pubkey::default(),
            usdc_mint: Pubkey::default(),
            frozen_at: 0,
            review_deadline: 0,
            schema_version: 0,
            bump: 0,
            reserved: [0; 32],
        }
    }

    fn snapshot_from_case(
        case_key: Pubkey,
        config_key: Pubkey,
        policy_key: Pubkey,
        case: &VictimReliefCaseV1,
    ) -> VictimReliefEvidenceSnapshotV1 {
        VictimReliefEvidenceSnapshotV1 {
            victim_relief_case: case_key,
            config: config_key,
            policy: policy_key,
            policy_version: case.policy_version,
            claimant: case.claimant,
            subject_commitment: case.subject_commitment,
            evidence_root: case.evidence_root,
            evidence_count: case.evidence_count,
            evidence_revision: case.evidence_revision,
            claimed_amount_usdc: case.claimed_amount_usdc,
            recipient_owner: case.recipient_owner,
            recipient_token_account: case.recipient_token_account,
            usdc_mint: case.usdc_mint,
            frozen_at: 1_999,
            review_deadline: 3_000,
            schema_version: VICTIM_RELIEF_SCHEMA_VERSION_V1,
            bump: 8,
            reserved: [0; 32],
        }
    }

    fn empty_decision_record() -> VictimReliefDecisionExecutionRecordV1 {
        VictimReliefDecisionExecutionRecordV1 {
            execution_queue_item: Pubkey::default(),
            proposal_decision: Pubkey::default(),
            governance_proposal: Pubkey::default(),
            governance_proposal_action: Pubkey::default(),
            module_registry: Pubkey::default(),
            config: Pubkey::default(),
            policy: Pubkey::default(),
            victim_relief_case: Pubkey::default(),
            evidence_snapshot: Pubkey::default(),
            execution_type: VictimReliefDecisionExecutionTypeV1::Reject,
            governance_action_type: GovernanceActionTypeV1::VictimReliefRejectClaim,
            case_status_before: VictimReliefCaseStatusV1::EvidencePeriod,
            case_status_after: VictimReliefCaseStatusV1::EvidencePeriod,
            claimed_amount_usdc: 0,
            approved_amount_usdc: 0,
            recipient_owner: Pubkey::default(),
            recipient_token_account: Pubkey::default(),
            parameters_hash: [0; 32],
            canonical_governance_payload_hash: [0; 32],
            executor: Pubkey::default(),
            executed_at: 0,
            schema_version: 0,
            bump: 0,
        }
    }

    fn original_reject_record_from_case(
        case_key: Pubkey,
        snapshot_key: Pubkey,
        case: &VictimReliefCaseV1,
    ) -> VictimReliefDecisionExecutionRecordV1 {
        VictimReliefDecisionExecutionRecordV1 {
            execution_queue_item: case.decision_queue,
            proposal_decision: Pubkey::new_unique(),
            governance_proposal: case.decision_proposal,
            governance_proposal_action: Pubkey::new_unique(),
            module_registry: Pubkey::new_unique(),
            config: case.config,
            policy: case.policy,
            victim_relief_case: case_key,
            evidence_snapshot: snapshot_key,
            execution_type: VictimReliefDecisionExecutionTypeV1::Reject,
            governance_action_type: GovernanceActionTypeV1::VictimReliefRejectClaim,
            case_status_before: VictimReliefCaseStatusV1::UnderReview,
            case_status_after: VictimReliefCaseStatusV1::Rejected,
            claimed_amount_usdc: case.claimed_amount_usdc,
            approved_amount_usdc: 0,
            recipient_owner: case.recipient_owner,
            recipient_token_account: case.recipient_token_account,
            parameters_hash: [7; 32],
            canonical_governance_payload_hash: [8; 32],
            executor: Pubkey::new_unique(),
            executed_at: 2_500,
            schema_version: VICTIM_RELIEF_DECISION_SCHEMA_VERSION,
            bump: 9,
        }
    }

    fn empty_payout_request() -> ReliefPayoutRequestV1 {
        ReliefPayoutRequestV1 {
            victim_relief_case: Pubkey::default(),
            config: Pubkey::default(),
            policy: Pubkey::default(),
            policy_version: 0,
            governance_proposal: Pubkey::default(),
            proposal_decision: Pubkey::default(),
            execution_queue_item: Pubkey::default(),
            evidence_snapshot: Pubkey::default(),
            approved_amount_usdc: 0,
            recipient_owner: Pubkey::default(),
            recipient_token_account: Pubkey::default(),
            treasury_config: Pubkey::default(),
            relief_usdc_vault: Pubkey::default(),
            usdc_mint: Pubkey::default(),
            status: VictimReliefPayoutStatusV1::Cancelled,
            parameters_hash: [0; 32],
            created_at: 0,
            executed_at: 0,
            schema_version: 0,
            bump: 0,
            reserved: [0; 32],
        }
    }

    fn empty_payout_execution_record() -> ReliefPayoutExecutionRecordV1 {
        ReliefPayoutExecutionRecordV1 {
            payout_request: Pubkey::default(),
            victim_relief_case: Pubkey::default(),
            config: Pubkey::default(),
            policy: Pubkey::default(),
            policy_version: 0,
            payout_origin: VictimReliefPayoutOriginV1::OriginalApprove,
            authorization_action_type: GovernanceActionTypeV1::VictimReliefApproveCompensation,
            governance_proposal: Pubkey::default(),
            proposal_decision: Pubkey::default(),
            execution_queue_item: Pubkey::default(),
            governance_proposal_action: Pubkey::default(),
            authorization_execution_record: Pubkey::default(),
            evidence_snapshot: Pubkey::default(),
            relief_usdc_vault: Pubkey::default(),
            vault_authority_v2: Pubkey::default(),
            recipient_owner: Pubkey::default(),
            recipient_token_account: Pubkey::default(),
            amount_usdc: 0,
            usdc_mint: Pubkey::default(),
            authorization_parameters_hash: [0; 32],
            payout_parameters_hash: [0; 32],
            executor: Pubkey::default(),
            executed_at: 0,
            schema_version: 0,
            bump: 0,
            reserved: [0; 32],
        }
    }

    fn empty_appeal() -> VictimReliefAppealV1 {
        VictimReliefAppealV1 {
            victim_relief_case: Pubkey::default(),
            config: Pubkey::default(),
            policy: Pubkey::default(),
            policy_version: 0,
            claimant: Pubkey::default(),
            original_evidence_snapshot: Pubkey::default(),
            original_decision_record: Pubkey::default(),
            original_governance_proposal: Pubkey::default(),
            original_execution_queue_item: Pubkey::default(),
            appeal_evidence_root: [0; 32],
            appeal_evidence_count: 0,
            status: VictimReliefAppealStatusV1::Pending,
            decision_proposal: Pubkey::default(),
            decision_queue: Pubkey::default(),
            opened_at: 0,
            appeal_deadline: 0,
            resolved_at: 0,
            schema_version: 0,
            bump: 0,
            reserved: [0; 32],
        }
    }

    fn empty_appeal_record() -> VictimReliefAppealDecisionExecutionRecordV1 {
        VictimReliefAppealDecisionExecutionRecordV1 {
            execution_queue_item: Pubkey::default(),
            proposal_decision: Pubkey::default(),
            governance_proposal: Pubkey::default(),
            governance_proposal_action: Pubkey::default(),
            module_registry: Pubkey::default(),
            config: Pubkey::default(),
            policy: Pubkey::default(),
            victim_relief_case: Pubkey::default(),
            victim_relief_appeal: Pubkey::default(),
            original_decision_record: Pubkey::default(),
            original_evidence_snapshot: Pubkey::default(),
            execution_type: VictimReliefAppealExecutionTypeV1::Uphold,
            governance_action_type: GovernanceActionTypeV1::VictimReliefUpholdAppeal,
            case_status_before: VictimReliefCaseStatusV1::EvidencePeriod,
            case_status_after: VictimReliefCaseStatusV1::EvidencePeriod,
            appeal_status_before: VictimReliefAppealStatusV1::Pending,
            appeal_status_after: VictimReliefAppealStatusV1::Pending,
            claimed_amount_usdc: 0,
            approved_amount_usdc: 0,
            recipient_owner: Pubkey::default(),
            recipient_token_account: Pubkey::default(),
            parameters_hash: [0; 32],
            canonical_governance_payload_hash: [0; 32],
            executor: Pubkey::default(),
            executed_at: 0,
            schema_version: 0,
            bump: 0,
        }
    }

    fn validation_result(
        action: GovernanceActionTypeV1,
        execution_type: VictimReliefDecisionExecutionTypeV1,
        approved_amount_usdc: u64,
    ) -> VictimReliefDecisionValidationResultV1 {
        VictimReliefDecisionValidationResultV1 {
            approved_amount_usdc,
            parameters_hash: [7; 32],
            canonical_governance_payload_hash: [8; 32],
            action_type: action,
            execution_type,
        }
    }

    fn appeal_validation_result(
        action: GovernanceActionTypeV1,
        execution_type: VictimReliefAppealExecutionTypeV1,
        approved_amount_usdc: u64,
    ) -> VictimReliefAppealDecisionValidationResultV1 {
        VictimReliefAppealDecisionValidationResultV1 {
            approved_amount_usdc,
            parameters_hash: [9; 32],
            canonical_governance_payload_hash: [10; 32],
            action_type: action,
            execution_type,
        }
    }

    fn token_account(mint: Pubkey, owner: Pubkey, amount: u64) -> TokenAccount {
        let account = SplTokenAccount {
            mint,
            owner,
            amount,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        };
        let mut data = vec![0; SplTokenAccount::LEN];
        SplTokenAccount::pack(account, &mut data).unwrap();
        TokenAccount::try_deserialize_unchecked(&mut data.as_slice()).unwrap()
    }

    fn mint_account(decimals: u8) -> Mint {
        let mint = SplMint {
            mint_authority: COption::None,
            supply: 1_000_000_000,
            decimals,
            is_initialized: true,
            freeze_authority: COption::None,
        };
        let mut data = vec![0; SplMint::LEN];
        SplMint::pack(mint, &mut data).unwrap();
        Mint::try_deserialize_unchecked(&mut data.as_slice()).unwrap()
    }

    fn treasury_config_fixture(usdc_mint: Pubkey) -> TreasuryConfigV2 {
        TreasuryConfigV2 {
            authority: Pubkey::new_unique(),
            usdc_mint,
            alpha_mint: Pubkey::new_unique(),
            bump: 1,
        }
    }

    fn security_config_fixture() -> GovernanceConfigV1 {
        GovernanceConfigV1 {
            authority: Pubkey::new_unique(),
            min_execution_delay_seconds: 60,
            proposal_count: 1,
            emergency_guardian: Pubkey::new_unique(),
            is_paused: false,
            bump: 1,
        }
    }

    fn governance_accounts_fixture(
        action_type: GovernanceActionTypeV1,
        target_account: Pubkey,
        proposer: Pubkey,
        proposal_id: u64,
        created_at: i64,
    ) -> (
        GovernanceProposalV1,
        GovernanceProposalActionV1,
        ProposalDecisionV1,
        ExecutionQueueItemV1,
    ) {
        let module_id = map_governance_action_to_module(action_type);
        let parameters_hash = [71; 32];
        let evidence_hash = [72; 32];
        let payload = GovernancePayloadV1 {
            schema_version: GOVERNANCE_PAYLOAD_V1_SCHEMA_VERSION,
            action_type,
            module_id,
            target_program: crate::ID,
            target_account,
            parameters_hash,
            evidence_hash,
            created_at,
        };
        let canonical_payload_hash = hash_governance_payload_v1(&payload).unwrap();
        let security_action = map_governance_action_to_security_action(action_type).unwrap();
        (
            GovernanceProposalV1 {
                proposal_id,
                proposer,
                proposal_type: map_governance_action_to_governance_proposal_type_v1(action_type),
                action_type: governance_action_stable_code_v1(action_type),
                target_program: crate::ID,
                target_account,
                payload_hash: canonical_payload_hash,
                status: GovernanceProposalStatusV1::Passed,
                voting_start_ts: created_at,
                voting_end_ts: created_at + 100,
                created_at,
                snapshot: Pubkey::new_unique(),
                yes_weight: 100,
                no_weight: 0,
                abstain_weight: 0,
                finalized_at: created_at + 100,
                bump: 1,
            },
            GovernanceProposalActionV1 {
                governance_proposal: Pubkey::default(),
                proposal_id,
                proposer,
                action_type,
                module_id,
                target_program: crate::ID,
                target_account,
                parameters_hash,
                evidence_hash,
                canonical_payload_hash,
                schema_version: 1,
                created_at,
                bump: 2,
            },
            ProposalDecisionV1 {
                proposal_id,
                proposal_type: security_proposal_type_for_action(security_action).unwrap(),
                proposer,
                decision: ProposalDecision::Approved,
                yes_weight: 100,
                no_weight: 0,
                start_ts: created_at,
                end_ts: created_at + 100,
                finalized_ts: created_at + 100,
                bump: 3,
            },
            ExecutionQueueItemV1 {
                proposal_id,
                proposer,
                action_type: security_action,
                target_program: crate::ID,
                target_account,
                decision: ProposalDecision::Approved,
                created_at: created_at + 101,
                execute_after: created_at + 161,
                executed_at: created_at + 162,
                status: ExecutionStatus::Executed,
                payload_hash: canonical_payload_hash,
                bump: 4,
            },
        )
    }

    struct OriginalPayoutFixture {
        security_config_key: Pubkey,
        security_config: GovernanceConfigV1,
        config_key: Pubkey,
        config: VictimReliefConfigV1,
        policy_key: Pubkey,
        policy: VictimReliefPolicyV1,
        claimant_state: VictimReliefClaimantStateV1,
        case_key: Pubkey,
        victim_relief_case: VictimReliefCaseV1,
        snapshot_key: Pubkey,
        snapshot: VictimReliefEvidenceSnapshotV1,
        payout_request_key: Pubkey,
        payout_request: ReliefPayoutRequestV1,
        governance_proposal_key: Pubkey,
        governance_proposal: GovernanceProposalV1,
        governance_proposal_action_key: Pubkey,
        governance_proposal_action: GovernanceProposalActionV1,
        proposal_decision_key: Pubkey,
        proposal_decision: ProposalDecisionV1,
        execution_queue_item_key: Pubkey,
        execution_queue_item: ExecutionQueueItemV1,
        authorization_record_key: Pubkey,
        authorization_record: VictimReliefDecisionExecutionRecordV1,
        treasury_config_key: Pubkey,
        treasury_config: TreasuryConfigV2,
        relief_vault_key: Pubkey,
        relief_vault: TokenAccount,
        vault_authority_key: Pubkey,
        recipient_token_account_key: Pubkey,
        recipient_token_account: TokenAccount,
        usdc_mint_key: Pubkey,
        usdc_mint: Mint,
        payout_receipt_key: Pubkey,
        payout_receipt: ReliefPayoutExecutionRecordV1,
    }

    fn original_payout_fixture() -> OriginalPayoutFixture {
        let (security_config_key, _) =
            Pubkey::find_program_address(&[GOVERNANCE_CONFIG_V1_SEED], &crate::ID);
        let security_config = security_config_fixture();
        let (treasury_config_key, treasury_bump) =
            Pubkey::find_program_address(&[TREASURY_CONFIG_V2_SEED], &crate::ID);
        let (relief_vault_key, _) =
            Pubkey::find_program_address(&[RELIEF_USDC_VAULT_SEED], &crate::ID);
        let (vault_authority_key, _) =
            Pubkey::find_program_address(&[VAULT_AUTHORITY_V2_SEED], &crate::ID);
        let (config_key, _) =
            Pubkey::find_program_address(&[VICTIM_RELIEF_CONFIG_V1_SEED], &crate::ID);
        let (policy_key, _) = Pubkey::find_program_address(
            &[
                VICTIM_RELIEF_POLICY_V1_SEED,
                config_key.as_ref(),
                &VICTIM_RELIEF_POLICY_VERSION_V1.to_le_bytes(),
            ],
            &crate::ID,
        );
        let claimant = Pubkey::new_unique();
        let usdc_mint_key = Pubkey::new_unique();
        let treasury_config = TreasuryConfigV2 {
            bump: treasury_bump,
            ..treasury_config_fixture(usdc_mint_key)
        };
        let mut config = config_fixture(policy_key);
        config.treasury_config = treasury_config_key;
        config.security_governance_config = security_config_key;
        config.usdc_mint = usdc_mint_key;
        let policy = policy_fixture(config_key);
        let (case_key, _) = Pubkey::find_program_address(
            &[
                VICTIM_RELIEF_CASE_V1_SEED,
                config_key.as_ref(),
                &1u64.to_le_bytes(),
            ],
            &crate::ID,
        );
        let (snapshot_key, _) = Pubkey::find_program_address(
            &[VICTIM_RELIEF_EVIDENCE_SNAPSHOT_V1_SEED, case_key.as_ref()],
            &crate::ID,
        );
        let (payout_request_key, payout_bump) = Pubkey::find_program_address(
            &[RELIEF_PAYOUT_REQUEST_V1_SEED, case_key.as_ref()],
            &crate::ID,
        );
        let mut victim_relief_case = case_fixture(config_key, policy_key, claimant, usdc_mint_key);
        victim_relief_case.status = VictimReliefCaseStatusV1::PayoutQueued;
        victim_relief_case.approved_amount_usdc = 5_000_000;
        let snapshot = snapshot_from_case(case_key, config_key, policy_key, &victim_relief_case);
        let proposer = Pubkey::new_unique();
        let proposal_id = 51;
        let governance_proposal_key = Pubkey::new_unique();
        let governance_proposal_action_key = Pubkey::new_unique();
        let proposal_decision_key = Pubkey::new_unique();
        let execution_queue_item_key = Pubkey::new_unique();
        let (
            mut governance_proposal,
            mut governance_proposal_action,
            proposal_decision,
            execution_queue_item,
        ) = governance_accounts_fixture(
            GovernanceActionTypeV1::VictimReliefApproveCompensation,
            case_key,
            proposer,
            proposal_id,
            2_000,
        );
        governance_proposal_action.governance_proposal = governance_proposal_key;
        victim_relief_case.decision_proposal = governance_proposal_key;
        victim_relief_case.decision_queue = execution_queue_item_key;
        governance_proposal.payload_hash = governance_proposal_action.canonical_payload_hash;
        let mut payout_request = empty_payout_request();
        payout_request.victim_relief_case = case_key;
        payout_request.config = config_key;
        payout_request.policy = policy_key;
        payout_request.policy_version = victim_relief_case.policy_version;
        payout_request.governance_proposal = governance_proposal_key;
        payout_request.proposal_decision = proposal_decision_key;
        payout_request.execution_queue_item = execution_queue_item_key;
        payout_request.evidence_snapshot = snapshot_key;
        payout_request.approved_amount_usdc = victim_relief_case.approved_amount_usdc;
        payout_request.recipient_owner = victim_relief_case.recipient_owner;
        payout_request.recipient_token_account = victim_relief_case.recipient_token_account;
        payout_request.treasury_config = treasury_config_key;
        payout_request.relief_usdc_vault = relief_vault_key;
        payout_request.usdc_mint = usdc_mint_key;
        payout_request.status = VictimReliefPayoutStatusV1::Approved;
        payout_request.parameters_hash = governance_proposal_action.parameters_hash;
        payout_request.created_at = 2_100;
        payout_request.executed_at = 0;
        payout_request.schema_version = VICTIM_RELIEF_DECISION_SCHEMA_VERSION;
        payout_request.bump = payout_bump;
        let (authorization_record_key, authorization_bump) = Pubkey::find_program_address(
            &[
                VICTIM_RELIEF_DECISION_EXECUTION_RECORD_V1_SEED,
                execution_queue_item_key.as_ref(),
            ],
            &crate::ID,
        );
        let authorization_record = VictimReliefDecisionExecutionRecordV1 {
            execution_queue_item: execution_queue_item_key,
            proposal_decision: proposal_decision_key,
            governance_proposal: governance_proposal_key,
            governance_proposal_action: governance_proposal_action_key,
            module_registry: Pubkey::new_unique(),
            config: config_key,
            policy: policy_key,
            victim_relief_case: case_key,
            evidence_snapshot: snapshot_key,
            execution_type: VictimReliefDecisionExecutionTypeV1::Approve,
            governance_action_type: GovernanceActionTypeV1::VictimReliefApproveCompensation,
            case_status_before: VictimReliefCaseStatusV1::UnderReview,
            case_status_after: VictimReliefCaseStatusV1::PayoutQueued,
            claimed_amount_usdc: victim_relief_case.claimed_amount_usdc,
            approved_amount_usdc: victim_relief_case.approved_amount_usdc,
            recipient_owner: victim_relief_case.recipient_owner,
            recipient_token_account: victim_relief_case.recipient_token_account,
            parameters_hash: payout_request.parameters_hash,
            canonical_governance_payload_hash: governance_proposal_action.canonical_payload_hash,
            executor: Pubkey::new_unique(),
            executed_at: 2_200,
            schema_version: VICTIM_RELIEF_DECISION_SCHEMA_VERSION,
            bump: authorization_bump,
        };
        let claimant_state = VictimReliefClaimantStateV1 {
            active_case_count: 1,
            ..claimant_state_fixture(config_key, claimant)
        };
        let relief_vault = token_account(
            usdc_mint_key,
            vault_authority_key,
            victim_relief_case.approved_amount_usdc,
        );
        let recipient_token_account_key = victim_relief_case.recipient_token_account;
        let recipient_token_account = token_account(usdc_mint_key, claimant, 0);
        let usdc_mint = mint_account(GREEN_LABEL_USDC_DECIMALS);
        let (payout_receipt_key, _) = Pubkey::find_program_address(
            &[
                RELIEF_PAYOUT_EXECUTION_RECORD_V1_SEED,
                payout_request_key.as_ref(),
            ],
            &crate::ID,
        );

        OriginalPayoutFixture {
            security_config_key,
            security_config,
            config_key,
            config,
            policy_key,
            policy,
            claimant_state,
            case_key,
            victim_relief_case,
            snapshot_key,
            snapshot,
            payout_request_key,
            payout_request,
            governance_proposal_key,
            governance_proposal,
            governance_proposal_action_key,
            governance_proposal_action,
            proposal_decision_key,
            proposal_decision,
            execution_queue_item_key,
            execution_queue_item,
            authorization_record_key,
            authorization_record,
            treasury_config_key,
            treasury_config,
            relief_vault_key,
            relief_vault,
            vault_authority_key,
            recipient_token_account_key,
            recipient_token_account,
            usdc_mint_key,
            usdc_mint,
            payout_receipt_key,
            payout_receipt: empty_payout_execution_record(),
        }
    }

    #[test]
    fn validates_policy_parameters() {
        assert!(validate_victim_relief_policy_parameters_v1(policy_parameters()).is_ok());

        let mut invalid = policy_parameters();
        invalid.min_claim_amount_usdc = 0;
        assert_eq!(
            validate_victim_relief_policy_parameters_v1(invalid).unwrap_err(),
            CustomError::InvalidVictimReliefPolicy.into()
        );

        let mut invalid = policy_parameters();
        invalid.max_payout_per_case_usdc = invalid.max_claim_amount_usdc + 1;
        assert_eq!(
            validate_victim_relief_policy_parameters_v1(invalid).unwrap_err(),
            CustomError::InvalidVictimReliefPolicy.into()
        );

        let mut invalid = policy_parameters();
        invalid.evidence_window_seconds = 0;
        assert_eq!(
            validate_victim_relief_policy_parameters_v1(invalid).unwrap_err(),
            CustomError::InvalidVictimReliefPolicy.into()
        );
    }

    #[test]
    fn records_config_defaults() {
        let mut config = VictimReliefConfigV1 {
            authority: Pubkey::default(),
            treasury_config: Pubkey::default(),
            security_governance_config: Pubkey::default(),
            usdc_mint: Pubkey::default(),
            current_policy: Pubkey::default(),
            current_policy_version: 9,
            next_case_id: 9,
            paused: true,
            created_at: 0,
            schema_version: 0,
            bump: 0,
            reserved: [9; 32],
        };
        let authority = Pubkey::new_unique();
        let treasury = Pubkey::new_unique();
        let security = Pubkey::new_unique();
        let mint = Pubkey::new_unique();

        record_victim_relief_config_init_v1(
            &mut config,
            authority,
            treasury,
            security,
            mint,
            77,
            4,
        )
        .unwrap();

        assert_eq!(config.authority, authority);
        assert_eq!(config.treasury_config, treasury);
        assert_eq!(config.security_governance_config, security);
        assert_eq!(config.usdc_mint, mint);
        assert_eq!(config.current_policy, Pubkey::default());
        assert_eq!(config.current_policy_version, 0);
        assert_eq!(config.next_case_id, 1);
        assert!(!config.paused);
        assert_eq!(config.schema_version, VICTIM_RELIEF_SCHEMA_VERSION_V1);
        assert_eq!(config.bump, 4);
    }

    #[test]
    fn records_policy_once_and_updates_config_current_policy() {
        let mut config = config_fixture(Pubkey::default());
        config.current_policy_version = 0;
        let mut policy = policy_fixture(Pubkey::new_unique());
        let config_key = Pubkey::new_unique();
        let policy_key = Pubkey::new_unique();
        let authority = config.authority;

        record_victim_relief_policy_init_v1(
            &mut config,
            &mut policy,
            config_key,
            policy_key,
            authority,
            policy_parameters(),
            111,
            5,
        )
        .unwrap();

        assert_eq!(policy.policy_version, VICTIM_RELIEF_POLICY_VERSION_V1);
        assert!(policy.active);
        assert_eq!(policy.initialized_by, authority);
        assert_eq!(policy.config, config_key);
        assert_eq!(config.current_policy, policy_key);
        assert_eq!(
            config.current_policy_version,
            VICTIM_RELIEF_POLICY_VERSION_V1
        );

        let err = record_victim_relief_policy_init_v1(
            &mut config,
            &mut policy,
            config_key,
            policy_key,
            authority,
            policy_parameters(),
            111,
            5,
        )
        .unwrap_err();
        assert_eq!(
            err,
            CustomError::VictimReliefPolicyAlreadyInitialized.into()
        );
    }

    #[test]
    fn initializes_claimant_state_once() {
        let config = Pubkey::new_unique();
        let claimant = Pubkey::new_unique();
        let mut state = VictimReliefClaimantStateV1 {
            config: Pubkey::default(),
            claimant: Pubkey::default(),
            active_case_count: 0,
            total_case_count: 0,
            last_case_id: 0,
            last_submitted_at: 0,
            created_at: 0,
            updated_at: 0,
            schema_version: 0,
            bump: 0,
        };

        ensure_victim_relief_claimant_state_v1(&mut state, config, claimant, 123, 7).unwrap();
        assert_eq!(state.config, config);
        assert_eq!(state.claimant, claimant);
        assert_eq!(state.created_at, 123);
        assert_eq!(state.bump, 7);

        ensure_victim_relief_claimant_state_v1(&mut state, config, claimant, 999, 8).unwrap();
        assert_eq!(state.created_at, 123);
        assert_eq!(state.bump, 7);
    }

    #[test]
    fn submit_case_updates_case_config_and_claimant_state() {
        let config_key = Pubkey::new_unique();
        let policy_key = Pubkey::new_unique();
        let claimant = Pubkey::new_unique();
        let usdc_mint = Pubkey::new_unique();
        let recipient = Pubkey::new_unique();
        let mut config = config_fixture(policy_key);
        config.usdc_mint = usdc_mint;
        let policy = policy_fixture(config_key);
        let mut claimant_state = claimant_state_fixture(config_key, claimant);
        let mut case = case_fixture(config_key, policy_key, claimant, usdc_mint);

        validate_victim_relief_case_submission_v1(
            &config,
            &policy,
            &claimant_state,
            policy_key,
            1,
            [1; 32],
            [2; 32],
            2,
            10_000_000,
            claimant,
            usdc_mint,
            usdc_mint,
            200,
        )
        .unwrap();

        record_victim_relief_case_submission_v1(
            &mut config,
            &policy,
            &mut claimant_state,
            &mut case,
            config_key,
            policy_key,
            claimant,
            recipient,
            [1; 32],
            [2; 32],
            2,
            10_000_000,
            usdc_mint,
            200,
            9,
        )
        .unwrap();

        assert_eq!(config.next_case_id, 2);
        assert_eq!(claimant_state.active_case_count, 1);
        assert_eq!(claimant_state.total_case_count, 1);
        assert_eq!(claimant_state.last_case_id, 1);
        assert_eq!(case.status, VictimReliefCaseStatusV1::EvidencePeriod);
        assert_eq!(case.evidence_deadline, 200 + policy.evidence_window_seconds);
        assert_eq!(case.recipient_owner, claimant);
        assert_eq!(case.recipient_token_account, recipient);
        assert_eq!(case.claimed_amount_usdc, 10_000_000);
    }

    #[test]
    fn submit_case_validation_rejects_invalid_inputs() {
        let config_key = Pubkey::new_unique();
        let policy_key = Pubkey::new_unique();
        let claimant = Pubkey::new_unique();
        let usdc_mint = Pubkey::new_unique();
        let mut config = config_fixture(policy_key);
        config.usdc_mint = usdc_mint;
        let policy = policy_fixture(config_key);
        let mut claimant_state = claimant_state_fixture(config_key, claimant);

        assert_eq!(
            validate_victim_relief_case_submission_v1(
                &config,
                &policy,
                &claimant_state,
                policy_key,
                2,
                [1; 32],
                [2; 32],
                1,
                10_000_000,
                claimant,
                usdc_mint,
                usdc_mint,
                200,
            )
            .unwrap_err(),
            CustomError::InvalidVictimReliefCaseId.into()
        );

        assert_eq!(
            validate_victim_relief_case_submission_v1(
                &config,
                &policy,
                &claimant_state,
                policy_key,
                1,
                [0; 32],
                [2; 32],
                1,
                10_000_000,
                claimant,
                usdc_mint,
                usdc_mint,
                200,
            )
            .unwrap_err(),
            CustomError::InvalidVictimReliefSubjectCommitment.into()
        );

        assert_eq!(
            validate_victim_relief_case_submission_v1(
                &config,
                &policy,
                &claimant_state,
                policy_key,
                1,
                [1; 32],
                [0; 32],
                1,
                10_000_000,
                claimant,
                usdc_mint,
                usdc_mint,
                200,
            )
            .unwrap_err(),
            CustomError::InvalidVictimReliefEvidenceRoot.into()
        );

        assert_eq!(
            validate_victim_relief_case_submission_v1(
                &config,
                &policy,
                &claimant_state,
                policy_key,
                1,
                [1; 32],
                [2; 32],
                0,
                10_000_000,
                claimant,
                usdc_mint,
                usdc_mint,
                200,
            )
            .unwrap_err(),
            CustomError::InvalidVictimReliefEvidenceCount.into()
        );

        assert_eq!(
            validate_victim_relief_case_submission_v1(
                &config,
                &policy,
                &claimant_state,
                policy_key,
                1,
                [1; 32],
                [2; 32],
                1,
                policy.min_claim_amount_usdc - 1,
                claimant,
                usdc_mint,
                usdc_mint,
                200,
            )
            .unwrap_err(),
            CustomError::InvalidVictimReliefClaimAmount.into()
        );

        assert_eq!(
            validate_victim_relief_case_submission_v1(
                &config,
                &policy,
                &claimant_state,
                policy_key,
                1,
                [1; 32],
                [2; 32],
                1,
                10_000_000,
                Pubkey::new_unique(),
                usdc_mint,
                usdc_mint,
                200,
            )
            .unwrap_err(),
            CustomError::VictimReliefRecipientMismatch.into()
        );

        claimant_state.active_case_count = policy.max_active_cases_per_claimant;
        assert_eq!(
            validate_victim_relief_case_submission_v1(
                &config,
                &policy,
                &claimant_state,
                policy_key,
                1,
                [1; 32],
                [2; 32],
                1,
                10_000_000,
                claimant,
                usdc_mint,
                usdc_mint,
                200,
            )
            .unwrap_err(),
            CustomError::VictimReliefActiveCaseLimitReached.into()
        );

        claimant_state.active_case_count = 0;
        claimant_state.last_submitted_at = 100;
        assert_eq!(
            validate_victim_relief_case_submission_v1(
                &config,
                &policy,
                &claimant_state,
                policy_key,
                1,
                [1; 32],
                [2; 32],
                1,
                10_000_000,
                claimant,
                usdc_mint,
                usdc_mint,
                101,
            )
            .unwrap_err(),
            CustomError::VictimReliefSubmissionCooldownActive.into()
        );

        config.paused = true;
        assert_eq!(
            validate_victim_relief_case_submission_v1(
                &config,
                &policy,
                &claimant_state,
                policy_key,
                1,
                [1; 32],
                [2; 32],
                1,
                10_000_000,
                claimant,
                usdc_mint,
                usdc_mint,
                4_000,
            )
            .unwrap_err(),
            CustomError::VictimReliefPaused.into()
        );
    }

    #[test]
    fn evidence_update_revises_root_only_inside_window() {
        let config_key = Pubkey::new_unique();
        let policy_key = Pubkey::new_unique();
        let claimant = Pubkey::new_unique();
        let usdc_mint = Pubkey::new_unique();
        let config = config_fixture(policy_key);
        let policy = policy_fixture(config_key);
        let mut case = case_fixture(config_key, policy_key, claimant, usdc_mint);

        validate_victim_relief_evidence_update_v1(
            &config,
            &policy,
            &case,
            config_key,
            policy_key,
            claimant,
            [33; 32],
            2,
            case.evidence_deadline,
        )
        .unwrap();
        record_victim_relief_evidence_update_v1(&mut case, [33; 32], 2, 1_500).unwrap();
        assert_eq!(case.evidence_root, [33; 32]);
        assert_eq!(case.evidence_count, 2);
        assert_eq!(case.evidence_revision, 1);

        assert_eq!(
            validate_victim_relief_evidence_update_v1(
                &config, &policy, &case, config_key, policy_key, claimant, [33; 32], 2, 1_600,
            )
            .unwrap_err(),
            CustomError::VictimReliefEvidenceUnchanged.into()
        );

        assert_eq!(
            validate_victim_relief_evidence_update_v1(
                &config,
                &policy,
                &case,
                config_key,
                policy_key,
                claimant,
                [44; 32],
                2,
                case.evidence_deadline + 1,
            )
            .unwrap_err(),
            CustomError::VictimReliefEvidenceWindowClosed.into()
        );
    }

    #[test]
    fn cancel_and_expire_decrement_active_count_once() {
        let config_key = Pubkey::new_unique();
        let policy_key = Pubkey::new_unique();
        let claimant = Pubkey::new_unique();
        let usdc_mint = Pubkey::new_unique();
        let mut claimant_state = claimant_state_fixture(config_key, claimant);
        claimant_state.active_case_count = 1;
        let mut case = case_fixture(config_key, policy_key, claimant, usdc_mint);

        record_victim_relief_case_terminal_status_v1(
            &mut claimant_state,
            &mut case,
            VictimReliefCaseStatusV1::Cancelled,
            2_000,
        )
        .unwrap();
        assert_eq!(claimant_state.active_case_count, 0);
        assert_eq!(case.status, VictimReliefCaseStatusV1::Cancelled);

        assert_eq!(
            record_victim_relief_case_terminal_status_v1(
                &mut claimant_state,
                &mut case,
                VictimReliefCaseStatusV1::Expired,
                2_001,
            )
            .unwrap_err(),
            CustomError::VictimReliefActiveCaseCountUnderflow.into()
        );
    }

    #[test]
    fn active_count_checked_add_and_sub_are_guarded() {
        let config = Pubkey::new_unique();
        let claimant = Pubkey::new_unique();
        let mut state = claimant_state_fixture(config, claimant);
        state.active_case_count = u16::MAX;
        assert_eq!(
            update_victim_relief_claimant_state_on_submit_v1(&mut state, 1, 100).unwrap_err(),
            CustomError::MathOverflow.into()
        );

        state.active_case_count = 0;
        assert_eq!(
            close_victim_relief_active_case_count_v1(&mut state, 100).unwrap_err(),
            CustomError::VictimReliefActiveCaseCountUnderflow.into()
        );
    }

    #[test]
    fn freeze_evidence_records_snapshot_and_sets_under_review() {
        let config_key = Pubkey::new_unique();
        let policy_key = Pubkey::new_unique();
        let case_key = Pubkey::new_unique();
        let claimant = Pubkey::new_unique();
        let usdc_mint = Pubkey::new_unique();
        let mut config = config_fixture(policy_key);
        config.usdc_mint = usdc_mint;
        let policy = policy_fixture(config_key);
        let mut case = case_fixture(config_key, policy_key, claimant, usdc_mint);
        let mut snapshot = empty_snapshot();

        let review_deadline = validate_victim_relief_evidence_freeze_v1(
            &config,
            &policy,
            &case,
            config_key,
            policy_key,
            claimant,
            case.evidence_deadline,
        )
        .unwrap();
        assert_eq!(
            review_deadline,
            case.evidence_deadline + policy.review_window_seconds
        );
        let evidence_deadline = case.evidence_deadline;

        record_victim_relief_evidence_snapshot_v1(
            &mut snapshot,
            &mut case,
            case_key,
            config_key,
            policy_key,
            evidence_deadline,
            review_deadline,
            9,
        )
        .unwrap();

        assert_eq!(case.status, VictimReliefCaseStatusV1::UnderReview);
        assert_eq!(case.review_deadline, review_deadline);
        assert_eq!(snapshot.victim_relief_case, case_key);
        assert_eq!(snapshot.evidence_root, case.evidence_root);
        assert_eq!(snapshot.evidence_count, case.evidence_count);
        assert_eq!(snapshot.recipient_owner, claimant);
        assert_eq!(snapshot.bump, 9);
    }

    #[test]
    fn freeze_evidence_rejects_after_deadline_and_wrong_status() {
        let config_key = Pubkey::new_unique();
        let policy_key = Pubkey::new_unique();
        let claimant = Pubkey::new_unique();
        let usdc_mint = Pubkey::new_unique();
        let config = config_fixture(policy_key);
        let policy = policy_fixture(config_key);
        let mut case = case_fixture(config_key, policy_key, claimant, usdc_mint);

        assert_eq!(
            validate_victim_relief_evidence_freeze_v1(
                &config,
                &policy,
                &case,
                config_key,
                policy_key,
                claimant,
                case.evidence_deadline + 1,
            )
            .unwrap_err(),
            CustomError::VictimReliefEvidenceFreezeTooLate.into()
        );

        case.status = VictimReliefCaseStatusV1::UnderReview;
        assert_eq!(
            validate_victim_relief_evidence_freeze_v1(
                &config,
                &policy,
                &case,
                config_key,
                policy_key,
                claimant,
                case.evidence_deadline,
            )
            .unwrap_err(),
            CustomError::VictimReliefCaseStatusMismatch.into()
        );
    }

    #[test]
    fn snapshot_validation_rejects_mutated_case_fields() {
        let config_key = Pubkey::new_unique();
        let policy_key = Pubkey::new_unique();
        let claimant = Pubkey::new_unique();
        let usdc_mint = Pubkey::new_unique();
        let case_key = Pubkey::new_unique();
        let mut case = case_fixture(config_key, policy_key, claimant, usdc_mint);
        let snapshot = snapshot_from_case(case_key, config_key, policy_key, &case);

        validate_victim_relief_snapshot_matches_case_v1(
            &snapshot,
            Pubkey::new_unique(),
            &case,
            case_key,
            config_key,
            policy_key,
        )
        .unwrap();

        case.evidence_count = case.evidence_count.checked_add(1).unwrap();
        assert_eq!(
            validate_victim_relief_snapshot_matches_case_v1(
                &snapshot,
                Pubkey::new_unique(),
                &case,
                case_key,
                config_key,
                policy_key,
            )
            .unwrap_err(),
            CustomError::VictimReliefEvidenceSnapshotMismatch.into()
        );
    }

    #[test]
    fn approved_amount_is_policy_capped_and_rejects_invalid_inputs() {
        let config_key = Pubkey::new_unique();
        let policy_key = Pubkey::new_unique();
        let claimant = Pubkey::new_unique();
        let usdc_mint = Pubkey::new_unique();
        let mut policy = policy_fixture(config_key);
        let mut case = case_fixture(config_key, policy_key, claimant, usdc_mint);

        case.claimed_amount_usdc = policy.max_payout_per_case_usdc - 1;
        assert_eq!(
            derive_victim_relief_approved_amount_v1(&case, &policy, policy_key).unwrap(),
            case.claimed_amount_usdc
        );

        case.claimed_amount_usdc = policy.max_payout_per_case_usdc;
        assert_eq!(
            derive_victim_relief_approved_amount_v1(&case, &policy, policy_key).unwrap(),
            policy.max_payout_per_case_usdc
        );

        case.claimed_amount_usdc = policy.max_payout_per_case_usdc + 1;
        assert_eq!(
            derive_victim_relief_approved_amount_v1(&case, &policy, policy_key).unwrap(),
            policy.max_payout_per_case_usdc
        );

        case.claimed_amount_usdc = 0;
        assert_eq!(
            derive_victim_relief_approved_amount_v1(&case, &policy, policy_key).unwrap_err(),
            CustomError::InvalidVictimReliefClaimAmount.into()
        );

        case.claimed_amount_usdc = 1;
        policy.max_payout_per_case_usdc = 0;
        assert_eq!(
            derive_victim_relief_approved_amount_v1(&case, &policy, policy_key).unwrap_err(),
            CustomError::InvalidVictimReliefClaimAmount.into()
        );
    }

    #[test]
    fn victim_relief_decision_execution_type_stable_codes_roundtrip() {
        assert_eq!(
            victim_relief_decision_execution_type_stable_code_v1(
                VictimReliefDecisionExecutionTypeV1::Approve
            ),
            1
        );
        assert_eq!(
            victim_relief_decision_execution_type_stable_code_v1(
                VictimReliefDecisionExecutionTypeV1::Reject
            ),
            2
        );
        assert_eq!(
            victim_relief_decision_execution_type_from_stable_code_v1(1).unwrap(),
            VictimReliefDecisionExecutionTypeV1::Approve
        );
        assert_eq!(
            victim_relief_decision_execution_type_from_stable_code_v1(2).unwrap(),
            VictimReliefDecisionExecutionTypeV1::Reject
        );
        assert_eq!(
            victim_relief_decision_execution_type_from_stable_code_v1(99).unwrap_err(),
            CustomError::InvalidVictimReliefDecisionSchema.into()
        );
    }

    #[test]
    fn decision_parameters_hash_is_deterministic_and_field_bound() {
        let config_key = Pubkey::new_unique();
        let policy_key = Pubkey::new_unique();
        let case_key = Pubkey::new_unique();
        let snapshot_key = Pubkey::new_unique();
        let treasury_key = Pubkey::new_unique();
        let relief_vault_key = Pubkey::new_unique();
        let claimant = Pubkey::new_unique();
        let usdc_mint = Pubkey::new_unique();
        let policy = policy_fixture(config_key);
        let case = case_fixture(config_key, policy_key, claimant, usdc_mint);
        let snapshot = snapshot_from_case(case_key, config_key, policy_key, &case);
        let approved_amount =
            derive_victim_relief_approved_amount_v1(&case, &policy, policy_key).unwrap();
        let params = build_victim_relief_decision_parameters_v1(
            config_key,
            policy_key,
            case_key,
            snapshot_key,
            &case,
            &snapshot,
            treasury_key,
            relief_vault_key,
            GovernanceActionTypeV1::VictimReliefApproveCompensation,
            approved_amount,
            77,
        );
        let hash_a = hash_victim_relief_decision_parameters_v1(&params).unwrap();
        let hash_b = hash_victim_relief_decision_parameters_v1(&params).unwrap();
        assert_eq!(hash_a, hash_b);

        let mut changed = params;
        changed.evidence_revision = changed.evidence_revision.checked_add(1).unwrap();
        assert_ne!(
            hash_a,
            hash_victim_relief_decision_parameters_v1(&changed).unwrap()
        );

        let mut changed = params;
        changed.approved_amount_usdc = changed.approved_amount_usdc.checked_add(1).unwrap();
        assert_ne!(
            hash_a,
            hash_victim_relief_decision_parameters_v1(&changed).unwrap()
        );

        let mut changed = params;
        changed.relief_usdc_vault = Pubkey::new_unique();
        assert_ne!(
            hash_a,
            hash_victim_relief_decision_parameters_v1(&changed).unwrap()
        );

        let mut changed = params;
        changed.action_type = GovernanceActionTypeV1::VictimReliefRejectClaim;
        assert_ne!(
            hash_a,
            hash_victim_relief_decision_parameters_v1(&changed).unwrap()
        );
    }

    #[test]
    fn approve_decision_records_payout_request_and_receipt_without_closing_case() {
        let config_key = Pubkey::new_unique();
        let policy_key = Pubkey::new_unique();
        let case_key = Pubkey::new_unique();
        let proposal_key = Pubkey::new_unique();
        let action_key = Pubkey::new_unique();
        let decision_key = Pubkey::new_unique();
        let queue_key = Pubkey::new_unique();
        let registry_key = Pubkey::new_unique();
        let snapshot_key = Pubkey::new_unique();
        let treasury_key = Pubkey::new_unique();
        let relief_vault_key = Pubkey::new_unique();
        let executor = Pubkey::new_unique();
        let claimant = Pubkey::new_unique();
        let usdc_mint = Pubkey::new_unique();
        let mut case = case_fixture(config_key, policy_key, claimant, usdc_mint);
        case.status = VictimReliefCaseStatusV1::UnderReview;
        let approved_amount = 5_000_000;
        let validation = validation_result(
            GovernanceActionTypeV1::VictimReliefApproveCompensation,
            VictimReliefDecisionExecutionTypeV1::Approve,
            approved_amount,
        );
        let mut payout_request = empty_payout_request();
        let mut record = empty_decision_record();

        record_approve_victim_relief_decision_v1(
            &mut case,
            &mut payout_request,
            &mut record,
            case_key,
            config_key,
            policy_key,
            proposal_key,
            action_key,
            decision_key,
            queue_key,
            registry_key,
            snapshot_key,
            treasury_key,
            relief_vault_key,
            executor,
            validation,
            3_000,
            4,
            5,
        )
        .unwrap();

        assert_eq!(case.status, VictimReliefCaseStatusV1::PayoutQueued);
        assert_eq!(case.approved_amount_usdc, approved_amount);
        assert_eq!(payout_request.status, VictimReliefPayoutStatusV1::Approved);
        assert_eq!(payout_request.executed_at, 0);
        assert_eq!(payout_request.relief_usdc_vault, relief_vault_key);
        assert_eq!(
            record.execution_type,
            VictimReliefDecisionExecutionTypeV1::Approve
        );
        assert_eq!(
            record.case_status_before,
            VictimReliefCaseStatusV1::UnderReview
        );
        assert_eq!(
            record.case_status_after,
            VictimReliefCaseStatusV1::PayoutQueued
        );
    }

    #[test]
    fn reject_decision_records_receipt_and_decrements_active_count_once() {
        let config_key = Pubkey::new_unique();
        let policy_key = Pubkey::new_unique();
        let case_key = Pubkey::new_unique();
        let proposal_key = Pubkey::new_unique();
        let action_key = Pubkey::new_unique();
        let decision_key = Pubkey::new_unique();
        let queue_key = Pubkey::new_unique();
        let registry_key = Pubkey::new_unique();
        let snapshot_key = Pubkey::new_unique();
        let executor = Pubkey::new_unique();
        let claimant = Pubkey::new_unique();
        let usdc_mint = Pubkey::new_unique();
        let mut claimant_state = claimant_state_fixture(config_key, claimant);
        claimant_state.active_case_count = 1;
        let mut case = case_fixture(config_key, policy_key, claimant, usdc_mint);
        case.status = VictimReliefCaseStatusV1::UnderReview;
        let validation = validation_result(
            GovernanceActionTypeV1::VictimReliefRejectClaim,
            VictimReliefDecisionExecutionTypeV1::Reject,
            0,
        );
        let mut record = empty_decision_record();

        record_reject_victim_relief_decision_v1(
            &mut claimant_state,
            &mut case,
            &mut record,
            config_key,
            policy_key,
            proposal_key,
            action_key,
            decision_key,
            queue_key,
            registry_key,
            case_key,
            snapshot_key,
            executor,
            123,
            validation,
            3_000,
            5,
        )
        .unwrap();

        assert_eq!(case.status, VictimReliefCaseStatusV1::Rejected);
        assert_eq!(case.approved_amount_usdc, 0);
        assert_eq!(case.appeal_deadline, 3_123);
        assert_eq!(claimant_state.active_case_count, 0);
        assert_eq!(
            record.execution_type,
            VictimReliefDecisionExecutionTypeV1::Reject
        );
        assert_eq!(record.case_status_after, VictimReliefCaseStatusV1::Rejected);

        assert_eq!(
            record_reject_victim_relief_decision_v1(
                &mut claimant_state,
                &mut case,
                &mut record,
                config_key,
                policy_key,
                proposal_key,
                action_key,
                decision_key,
                queue_key,
                registry_key,
                case_key,
                snapshot_key,
                executor,
                123,
                validation,
                3_001,
                5,
            )
            .unwrap_err(),
            CustomError::VictimReliefDecisionNotEligible.into()
        );
    }

    #[test]
    fn appeal_evidence_rules_accept_none_or_valid_new_evidence() {
        assert!(validate_victim_relief_appeal_evidence_v1([0; 32], 0, 3).is_ok());
        assert!(validate_victim_relief_appeal_evidence_v1([1; 32], 1, 3).is_ok());
        assert_eq!(
            validate_victim_relief_appeal_evidence_v1([0; 32], 1, 3).unwrap_err(),
            CustomError::VictimReliefAppealEvidenceMismatch.into()
        );
        assert_eq!(
            validate_victim_relief_appeal_evidence_v1([1; 32], 0, 3).unwrap_err(),
            CustomError::VictimReliefAppealEvidenceMismatch.into()
        );
        assert_eq!(
            validate_victim_relief_appeal_evidence_v1([1; 32], 4, 3).unwrap_err(),
            CustomError::VictimReliefAppealEvidenceMismatch.into()
        );
    }

    #[test]
    fn open_appeal_records_pending_appeal_without_active_count_change() {
        let config_key = Pubkey::new_unique();
        let policy_key = Pubkey::new_unique();
        let case_key = Pubkey::new_unique();
        let snapshot_key = Pubkey::new_unique();
        let record_key = Pubkey::new_unique();
        let appeal_key = Pubkey::new_unique();
        let claimant = Pubkey::new_unique();
        let usdc_mint = Pubkey::new_unique();
        let config = config_fixture(policy_key);
        let policy = policy_fixture(config_key);
        let mut case = case_fixture(config_key, policy_key, claimant, usdc_mint);
        case.status = VictimReliefCaseStatusV1::Rejected;
        case.appeal_deadline = 3_000;
        case.decision_proposal = Pubkey::new_unique();
        case.decision_queue = Pubkey::new_unique();
        let snapshot = snapshot_from_case(case_key, config_key, policy_key, &case);
        let original_record = original_reject_record_from_case(case_key, snapshot_key, &case);

        validate_open_victim_relief_appeal_v1(
            &config,
            &policy,
            &case,
            case_key,
            &snapshot,
            snapshot_key,
            &original_record,
            record_key,
            config_key,
            policy_key,
            claimant,
            [0; 32],
            0,
            3_000,
        )
        .unwrap();

        let mut appeal = empty_appeal();
        record_open_victim_relief_appeal_v1(
            &mut appeal,
            &mut case,
            appeal_key,
            case_key,
            config_key,
            policy_key,
            snapshot_key,
            record_key,
            [0; 32],
            0,
            3_000,
            6,
        )
        .unwrap();

        assert_eq!(case.status, VictimReliefCaseStatusV1::AppealPending);
        assert_eq!(case.active_appeal, appeal_key);
        assert_eq!(appeal.status, VictimReliefAppealStatusV1::Pending);
        assert_eq!(appeal.original_decision_record, record_key);
        assert_eq!(appeal.appeal_deadline, 3_000);

        assert_eq!(
            validate_open_victim_relief_appeal_v1(
                &config,
                &policy,
                &case,
                case_key,
                &snapshot,
                snapshot_key,
                &original_record,
                record_key,
                config_key,
                policy_key,
                claimant,
                [2; 32],
                1,
                3_001,
            )
            .unwrap_err(),
            CustomError::VictimReliefAppealNotEligible.into()
        );
    }

    #[test]
    fn appeal_decision_parameters_hash_is_field_bound() {
        let config_key = Pubkey::new_unique();
        let policy_key = Pubkey::new_unique();
        let case_key = Pubkey::new_unique();
        let appeal_key = Pubkey::new_unique();
        let snapshot_key = Pubkey::new_unique();
        let record_key = Pubkey::new_unique();
        let treasury_key = Pubkey::new_unique();
        let relief_vault_key = Pubkey::new_unique();
        let claimant = Pubkey::new_unique();
        let usdc_mint = Pubkey::new_unique();
        let policy = policy_fixture(config_key);
        let mut case = case_fixture(config_key, policy_key, claimant, usdc_mint);
        case.status = VictimReliefCaseStatusV1::AppealPending;
        let snapshot = snapshot_from_case(case_key, config_key, policy_key, &case);
        let mut appeal = empty_appeal();
        appeal.victim_relief_case = case_key;
        appeal.config = config_key;
        appeal.policy = policy_key;
        appeal.policy_version = case.policy_version;
        appeal.claimant = claimant;
        appeal.original_evidence_snapshot = snapshot_key;
        appeal.original_decision_record = record_key;
        appeal.original_governance_proposal = Pubkey::new_unique();
        appeal.original_execution_queue_item = Pubkey::new_unique();
        appeal.appeal_evidence_root = [3; 32];
        appeal.appeal_evidence_count = 2;
        appeal.schema_version = VICTIM_RELIEF_SCHEMA_VERSION_V1;

        let approved_amount =
            derive_victim_relief_approved_amount_v1(&case, &policy, policy_key).unwrap();
        let params = build_victim_relief_appeal_decision_parameters_v1(
            config_key,
            policy_key,
            case_key,
            appeal_key,
            snapshot_key,
            record_key,
            &case,
            &appeal,
            &snapshot,
            treasury_key,
            relief_vault_key,
            GovernanceActionTypeV1::VictimReliefOverturnAppeal,
            approved_amount,
            88,
        );
        let hash_a = hash_victim_relief_appeal_decision_parameters_v1(&params).unwrap();
        assert_eq!(
            hash_a,
            hash_victim_relief_appeal_decision_parameters_v1(&params).unwrap()
        );

        let mut changed = params;
        changed.appeal_evidence_count = changed.appeal_evidence_count.checked_add(1).unwrap();
        assert_ne!(
            hash_a,
            hash_victim_relief_appeal_decision_parameters_v1(&changed).unwrap()
        );

        let mut changed = params;
        changed.approved_amount_usdc = changed.approved_amount_usdc.checked_add(1).unwrap();
        assert_ne!(
            hash_a,
            hash_victim_relief_appeal_decision_parameters_v1(&changed).unwrap()
        );

        let mut changed = params;
        changed.relief_usdc_vault = Pubkey::new_unique();
        assert_ne!(
            hash_a,
            hash_victim_relief_appeal_decision_parameters_v1(&changed).unwrap()
        );

        let mut changed = params;
        changed.action_type = GovernanceActionTypeV1::VictimReliefUpholdAppeal;
        assert_ne!(
            hash_a,
            hash_victim_relief_appeal_decision_parameters_v1(&changed).unwrap()
        );
    }

    #[test]
    fn victim_relief_appeal_execution_type_stable_codes_roundtrip() {
        assert_eq!(
            victim_relief_appeal_execution_type_stable_code_v1(
                VictimReliefAppealExecutionTypeV1::Uphold
            ),
            1
        );
        assert_eq!(
            victim_relief_appeal_execution_type_stable_code_v1(
                VictimReliefAppealExecutionTypeV1::Overturn
            ),
            2
        );
        assert_eq!(
            victim_relief_appeal_execution_type_from_stable_code_v1(1).unwrap(),
            VictimReliefAppealExecutionTypeV1::Uphold
        );
        assert_eq!(
            victim_relief_appeal_execution_type_from_stable_code_v1(2).unwrap(),
            VictimReliefAppealExecutionTypeV1::Overturn
        );
        assert_eq!(
            victim_relief_appeal_execution_type_from_stable_code_v1(99).unwrap_err(),
            CustomError::InvalidVictimReliefAppealSchema.into()
        );
    }

    #[test]
    fn uphold_appeal_records_receipt_without_payout_or_active_count_change() {
        let config_key = Pubkey::new_unique();
        let policy_key = Pubkey::new_unique();
        let case_key = Pubkey::new_unique();
        let appeal_key = Pubkey::new_unique();
        let proposal_key = Pubkey::new_unique();
        let action_key = Pubkey::new_unique();
        let decision_key = Pubkey::new_unique();
        let queue_key = Pubkey::new_unique();
        let registry_key = Pubkey::new_unique();
        let snapshot_key = Pubkey::new_unique();
        let record_key = Pubkey::new_unique();
        let executor = Pubkey::new_unique();
        let claimant = Pubkey::new_unique();
        let usdc_mint = Pubkey::new_unique();
        let mut case = case_fixture(config_key, policy_key, claimant, usdc_mint);
        case.status = VictimReliefCaseStatusV1::AppealPending;
        case.active_appeal = appeal_key;
        let mut appeal = empty_appeal();
        appeal.status = VictimReliefAppealStatusV1::Pending;
        let mut record = empty_appeal_record();
        let validation = appeal_validation_result(
            GovernanceActionTypeV1::VictimReliefUpholdAppeal,
            VictimReliefAppealExecutionTypeV1::Uphold,
            0,
        );

        record_uphold_victim_relief_appeal_v1(
            &mut case,
            &mut appeal,
            &mut record,
            config_key,
            policy_key,
            proposal_key,
            action_key,
            decision_key,
            queue_key,
            registry_key,
            case_key,
            appeal_key,
            snapshot_key,
            record_key,
            executor,
            validation,
            4_000,
            7,
        )
        .unwrap();

        assert_eq!(case.status, VictimReliefCaseStatusV1::AppealUpheld);
        assert_eq!(case.active_appeal, Pubkey::default());
        assert_eq!(case.approved_amount_usdc, 0);
        assert_eq!(appeal.status, VictimReliefAppealStatusV1::Upheld);
        assert_eq!(
            record.execution_type,
            VictimReliefAppealExecutionTypeV1::Uphold
        );
        assert_eq!(
            record.case_status_after,
            VictimReliefCaseStatusV1::AppealUpheld
        );
    }

    #[test]
    fn overturn_appeal_creates_payout_request_and_restores_active_count() {
        let config_key = Pubkey::new_unique();
        let policy_key = Pubkey::new_unique();
        let case_key = Pubkey::new_unique();
        let appeal_key = Pubkey::new_unique();
        let proposal_key = Pubkey::new_unique();
        let action_key = Pubkey::new_unique();
        let decision_key = Pubkey::new_unique();
        let queue_key = Pubkey::new_unique();
        let registry_key = Pubkey::new_unique();
        let snapshot_key = Pubkey::new_unique();
        let record_key = Pubkey::new_unique();
        let treasury_key = Pubkey::new_unique();
        let relief_vault_key = Pubkey::new_unique();
        let executor = Pubkey::new_unique();
        let claimant = Pubkey::new_unique();
        let usdc_mint = Pubkey::new_unique();
        let mut claimant_state = claimant_state_fixture(config_key, claimant);
        claimant_state.active_case_count = 0;
        let mut case = case_fixture(config_key, policy_key, claimant, usdc_mint);
        case.status = VictimReliefCaseStatusV1::AppealPending;
        case.active_appeal = appeal_key;
        let approved_amount = 5_000_000;
        let mut appeal = empty_appeal();
        appeal.status = VictimReliefAppealStatusV1::Pending;
        let mut payout_request = empty_payout_request();
        let mut record = empty_appeal_record();
        let validation = appeal_validation_result(
            GovernanceActionTypeV1::VictimReliefOverturnAppeal,
            VictimReliefAppealExecutionTypeV1::Overturn,
            approved_amount,
        );

        record_overturn_victim_relief_appeal_v1(
            &mut claimant_state,
            &mut case,
            &mut appeal,
            &mut payout_request,
            &mut record,
            config_key,
            policy_key,
            proposal_key,
            action_key,
            decision_key,
            queue_key,
            registry_key,
            case_key,
            appeal_key,
            snapshot_key,
            record_key,
            treasury_key,
            relief_vault_key,
            executor,
            validation,
            4_000,
            6,
            7,
        )
        .unwrap();

        assert_eq!(appeal.status, VictimReliefAppealStatusV1::Overturned);
        assert_eq!(case.status, VictimReliefCaseStatusV1::PayoutQueued);
        assert_eq!(case.active_appeal, Pubkey::default());
        assert_eq!(case.approved_amount_usdc, approved_amount);
        assert_eq!(claimant_state.active_case_count, 1);
        assert_eq!(payout_request.status, VictimReliefPayoutStatusV1::Approved);
        assert_eq!(payout_request.governance_proposal, proposal_key);
        assert_eq!(payout_request.execution_queue_item, queue_key);
        assert_eq!(payout_request.evidence_snapshot, snapshot_key);
        assert_eq!(payout_request.parameters_hash, validation.parameters_hash);
        assert_eq!(
            record.execution_type,
            VictimReliefAppealExecutionTypeV1::Overturn
        );
        assert_eq!(
            record.case_status_after,
            VictimReliefCaseStatusV1::PayoutQueued
        );
    }

    #[test]
    fn victim_relief_payout_origin_stable_codes_roundtrip_and_reject_unknown() {
        assert_eq!(
            victim_relief_payout_origin_stable_code_v1(VictimReliefPayoutOriginV1::OriginalApprove),
            1
        );
        assert_eq!(
            victim_relief_payout_origin_stable_code_v1(VictimReliefPayoutOriginV1::AppealOverturn),
            2
        );
        assert_eq!(
            victim_relief_payout_origin_from_stable_code_v1(1).unwrap(),
            VictimReliefPayoutOriginV1::OriginalApprove
        );
        assert_eq!(
            victim_relief_payout_origin_from_stable_code_v1(2).unwrap(),
            VictimReliefPayoutOriginV1::AppealOverturn
        );
        assert_eq!(
            victim_relief_payout_origin_from_stable_code_v1(99).unwrap_err(),
            CustomError::InvalidVictimReliefPayoutOrigin.into()
        );
    }

    #[test]
    fn payout_parameters_hash_is_deterministic_field_bound_and_excludes_executor() {
        let f = original_payout_fixture();
        let params = build_original_approve_payout_parameters_v1(
            f.payout_request_key,
            &f.payout_request,
            f.governance_proposal_action_key,
            f.authorization_record_key,
            f.vault_authority_key,
        )
        .unwrap();
        let hash_a = hash_victim_relief_payout_parameters_v1(&params).unwrap();
        let hash_b = hash_victim_relief_payout_parameters_v1(&params).unwrap();
        assert_eq!(hash_a, hash_b);

        let mut changed = params;
        changed.payout_origin = VictimReliefPayoutOriginV1::AppealOverturn;
        changed.authorization_action_type = GovernanceActionTypeV1::VictimReliefOverturnAppeal;
        assert_ne!(
            hash_a,
            hash_victim_relief_payout_parameters_v1(&changed).unwrap()
        );

        let mut changed = params;
        changed.payout_request = Pubkey::new_unique();
        assert_ne!(
            hash_a,
            hash_victim_relief_payout_parameters_v1(&changed).unwrap()
        );

        let mut changed = params;
        changed.approved_amount_usdc = changed.approved_amount_usdc.checked_add(1).unwrap();
        assert_ne!(
            hash_a,
            hash_victim_relief_payout_parameters_v1(&changed).unwrap()
        );

        let mut changed = params;
        changed.recipient_token_account = Pubkey::new_unique();
        assert_ne!(
            hash_a,
            hash_victim_relief_payout_parameters_v1(&changed).unwrap()
        );

        let mut changed = params;
        changed.relief_usdc_vault = Pubkey::new_unique();
        assert_ne!(
            hash_a,
            hash_victim_relief_payout_parameters_v1(&changed).unwrap()
        );

        let mut changed = params;
        changed.authorization_parameters_hash = [3; 32];
        assert_ne!(
            hash_a,
            hash_victim_relief_payout_parameters_v1(&changed).unwrap()
        );

        let mut receipt_a = empty_payout_execution_record();
        let mut receipt_b = empty_payout_execution_record();
        let hash_from_a = record_relief_payout_execution_v1(
            &mut receipt_a,
            params,
            Pubkey::new_unique(),
            5_000,
            7,
        )
        .unwrap();
        let hash_from_b = record_relief_payout_execution_v1(
            &mut receipt_b,
            params,
            Pubkey::new_unique(),
            5_000,
            7,
        )
        .unwrap();
        assert_eq!(hash_from_a, hash_a);
        assert_eq!(hash_from_b, hash_a);
        assert_ne!(receipt_a.executor, receipt_b.executor);
        assert_eq!(
            receipt_a.payout_parameters_hash,
            receipt_b.payout_parameters_hash
        );
    }

    #[test]
    fn payout_execution_receipt_records_verified_fields() {
        let f = original_payout_fixture();
        let params = build_original_approve_payout_parameters_v1(
            f.payout_request_key,
            &f.payout_request,
            f.governance_proposal_action_key,
            f.authorization_record_key,
            f.vault_authority_key,
        )
        .unwrap();
        let mut receipt = empty_payout_execution_record();
        let executor = Pubkey::new_unique();
        let payout_hash =
            record_relief_payout_execution_v1(&mut receipt, params, executor, 5_000, 9).unwrap();

        assert_eq!(receipt.payout_request, f.payout_request_key);
        assert_eq!(receipt.victim_relief_case, f.case_key);
        assert_eq!(
            receipt.payout_origin,
            VictimReliefPayoutOriginV1::OriginalApprove
        );
        assert_eq!(
            receipt.authorization_action_type,
            GovernanceActionTypeV1::VictimReliefApproveCompensation
        );
        assert_eq!(
            receipt.authorization_execution_record,
            f.authorization_record_key
        );
        assert_eq!(receipt.amount_usdc, f.payout_request.approved_amount_usdc);
        assert_eq!(
            receipt.recipient_token_account,
            f.payout_request.recipient_token_account
        );
        assert_eq!(receipt.executor, executor);
        assert_eq!(receipt.payout_parameters_hash, payout_hash);
        assert_eq!(
            receipt.schema_version,
            VICTIM_RELIEF_DECISION_SCHEMA_VERSION
        );
        assert_eq!(receipt.bump, 9);

        assert_eq!(
            record_relief_payout_execution_v1(&mut receipt, params, executor, 5_001, 10)
                .unwrap_err(),
            CustomError::VictimReliefPayoutReceiptAlreadyExists.into()
        );
    }

    #[test]
    fn original_approved_payout_record_marks_request_case_and_claimant_state() {
        let f = original_payout_fixture();
        let params = build_original_approve_payout_parameters_v1(
            f.payout_request_key,
            &f.payout_request,
            f.governance_proposal_action_key,
            f.authorization_record_key,
            f.vault_authority_key,
        )
        .unwrap();
        let mut claimant_state = f.claimant_state;
        let mut victim_relief_case = f.victim_relief_case;
        let mut payout_request = f.payout_request;
        let mut receipt = empty_payout_execution_record();
        let total_cases_before = claimant_state.total_case_count;
        let executor = Pubkey::new_unique();
        let now = 7_777;
        let payout_hash = hash_victim_relief_payout_parameters_v1(&params).unwrap();

        let recorded_hash = record_original_approved_victim_relief_payout_v1(
            &mut claimant_state,
            &mut victim_relief_case,
            &mut payout_request,
            &mut receipt,
            params,
            executor,
            now,
            6,
        )
        .unwrap();

        assert_eq!(recorded_hash, payout_hash);
        assert_eq!(payout_request.status, VictimReliefPayoutStatusV1::Executed);
        assert_eq!(payout_request.executed_at, now);
        assert_eq!(victim_relief_case.status, VictimReliefCaseStatusV1::Paid);
        assert_eq!(victim_relief_case.updated_at, now);
        assert_eq!(
            victim_relief_case.approved_amount_usdc,
            params.approved_amount_usdc
        );
        assert_eq!(claimant_state.active_case_count, 0);
        assert_eq!(claimant_state.total_case_count, total_cases_before);
        assert_eq!(claimant_state.updated_at, now);
        assert_eq!(receipt.payout_request, f.payout_request_key);
        assert_eq!(
            receipt.payout_origin,
            VictimReliefPayoutOriginV1::OriginalApprove
        );
        assert_eq!(
            receipt.authorization_action_type,
            GovernanceActionTypeV1::VictimReliefApproveCompensation
        );
        assert_eq!(receipt.amount_usdc, params.approved_amount_usdc);
        assert_eq!(receipt.executor, executor);
        assert_eq!(receipt.executed_at, now);
    }

    #[test]
    fn original_approved_payout_record_rejects_wrong_origin_and_replay() {
        let f = original_payout_fixture();
        let original_params = build_original_approve_payout_parameters_v1(
            f.payout_request_key,
            &f.payout_request,
            f.governance_proposal_action_key,
            f.authorization_record_key,
            f.vault_authority_key,
        )
        .unwrap();
        let mut params = original_params;
        params.payout_origin = VictimReliefPayoutOriginV1::AppealOverturn;
        params.authorization_action_type = GovernanceActionTypeV1::VictimReliefOverturnAppeal;

        let mut claimant_state = f.claimant_state;
        let mut victim_relief_case = f.victim_relief_case;
        let mut payout_request = f.payout_request;
        let mut receipt = empty_payout_execution_record();

        assert_eq!(
            record_original_approved_victim_relief_payout_v1(
                &mut claimant_state,
                &mut victim_relief_case,
                &mut payout_request,
                &mut receipt,
                params,
                Pubkey::new_unique(),
                7_777,
                6,
            )
            .unwrap_err(),
            CustomError::VictimReliefPayoutActionMismatch.into()
        );

        payout_request.status = VictimReliefPayoutStatusV1::Executed;
        payout_request.executed_at = 8_000;
        assert_eq!(
            record_original_approved_victim_relief_payout_v1(
                &mut claimant_state,
                &mut victim_relief_case,
                &mut payout_request,
                &mut receipt,
                original_params,
                Pubkey::new_unique(),
                8_001,
                6,
            )
            .unwrap_err(),
            CustomError::VictimReliefPayoutStatusMismatch.into()
        );
    }

    #[test]
    fn original_approve_authorization_validator_accepts_only_approve_receipt() {
        let mut f = original_payout_fixture();
        validate_victim_relief_original_approve_authorization_v1(
            f.governance_proposal_key,
            &f.governance_proposal,
            f.governance_proposal_action_key,
            &f.governance_proposal_action,
            f.proposal_decision_key,
            &f.proposal_decision,
            f.execution_queue_item_key,
            &f.execution_queue_item,
            f.authorization_record_key,
            &f.authorization_record,
            f.snapshot_key,
            &f.snapshot,
            f.case_key,
            &f.victim_relief_case,
            f.payout_request_key,
            &f.payout_request,
        )
        .unwrap();

        f.authorization_record.execution_type = VictimReliefDecisionExecutionTypeV1::Reject;
        assert_eq!(
            validate_victim_relief_original_approve_authorization_v1(
                f.governance_proposal_key,
                &f.governance_proposal,
                f.governance_proposal_action_key,
                &f.governance_proposal_action,
                f.proposal_decision_key,
                &f.proposal_decision,
                f.execution_queue_item_key,
                &f.execution_queue_item,
                f.authorization_record_key,
                &f.authorization_record,
                f.snapshot_key,
                &f.snapshot,
                f.case_key,
                &f.victim_relief_case,
                f.payout_request_key,
                &f.payout_request,
            )
            .unwrap_err(),
            CustomError::VictimReliefPayoutAuthorizationMismatch.into()
        );

        f.authorization_record.execution_type = VictimReliefDecisionExecutionTypeV1::Approve;
        f.authorization_record.governance_action_type =
            GovernanceActionTypeV1::VictimReliefOverturnAppeal;
        assert_eq!(
            validate_victim_relief_original_approve_authorization_v1(
                f.governance_proposal_key,
                &f.governance_proposal,
                f.governance_proposal_action_key,
                &f.governance_proposal_action,
                f.proposal_decision_key,
                &f.proposal_decision,
                f.execution_queue_item_key,
                &f.execution_queue_item,
                f.authorization_record_key,
                &f.authorization_record,
                f.snapshot_key,
                &f.snapshot,
                f.case_key,
                &f.victim_relief_case,
                f.payout_request_key,
                &f.payout_request,
            )
            .unwrap_err(),
            CustomError::VictimReliefPayoutActionMismatch.into()
        );
    }

    #[test]
    fn common_payout_validator_checks_pause_vault_recipient_balance_and_receipt() {
        let mut f = original_payout_fixture();
        let params = validate_victim_relief_payout_common_v1(
            f.security_config_key,
            &f.security_config,
            f.config_key,
            &f.config,
            f.policy_key,
            &f.policy,
            &f.claimant_state,
            f.case_key,
            &f.victim_relief_case,
            f.snapshot_key,
            &f.snapshot,
            f.payout_request_key,
            &f.payout_request,
            f.treasury_config_key,
            &f.treasury_config,
            f.relief_vault_key,
            &f.relief_vault,
            f.vault_authority_key,
            f.recipient_token_account_key,
            &f.recipient_token_account,
            f.usdc_mint_key,
            &f.usdc_mint,
            f.payout_receipt_key,
            &f.payout_receipt,
            VictimReliefPayoutOriginV1::OriginalApprove,
            GovernanceActionTypeV1::VictimReliefApproveCompensation,
            f.governance_proposal_action_key,
            f.authorization_record_key,
            f.authorization_record.approved_amount_usdc,
            f.authorization_record.parameters_hash,
        )
        .unwrap();
        assert_eq!(
            params.payout_origin,
            VictimReliefPayoutOriginV1::OriginalApprove
        );
        assert_eq!(
            params.approved_amount_usdc,
            f.payout_request.approved_amount_usdc
        );

        f.config.paused = true;
        assert_eq!(
            validate_victim_relief_payout_common_v1(
                f.security_config_key,
                &f.security_config,
                f.config_key,
                &f.config,
                f.policy_key,
                &f.policy,
                &f.claimant_state,
                f.case_key,
                &f.victim_relief_case,
                f.snapshot_key,
                &f.snapshot,
                f.payout_request_key,
                &f.payout_request,
                f.treasury_config_key,
                &f.treasury_config,
                f.relief_vault_key,
                &f.relief_vault,
                f.vault_authority_key,
                f.recipient_token_account_key,
                &f.recipient_token_account,
                f.usdc_mint_key,
                &f.usdc_mint,
                f.payout_receipt_key,
                &f.payout_receipt,
                VictimReliefPayoutOriginV1::OriginalApprove,
                GovernanceActionTypeV1::VictimReliefApproveCompensation,
                f.governance_proposal_action_key,
                f.authorization_record_key,
                f.authorization_record.approved_amount_usdc,
                f.authorization_record.parameters_hash,
            )
            .unwrap_err(),
            CustomError::VictimReliefPayoutPaused.into()
        );
        f.config.paused = false;

        f.victim_relief_case.active_appeal = Pubkey::new_unique();
        assert_eq!(
            validate_victim_relief_payout_common_v1(
                f.security_config_key,
                &f.security_config,
                f.config_key,
                &f.config,
                f.policy_key,
                &f.policy,
                &f.claimant_state,
                f.case_key,
                &f.victim_relief_case,
                f.snapshot_key,
                &f.snapshot,
                f.payout_request_key,
                &f.payout_request,
                f.treasury_config_key,
                &f.treasury_config,
                f.relief_vault_key,
                &f.relief_vault,
                f.vault_authority_key,
                f.recipient_token_account_key,
                &f.recipient_token_account,
                f.usdc_mint_key,
                &f.usdc_mint,
                f.payout_receipt_key,
                &f.payout_receipt,
                VictimReliefPayoutOriginV1::OriginalApprove,
                GovernanceActionTypeV1::VictimReliefApproveCompensation,
                f.governance_proposal_action_key,
                f.authorization_record_key,
                f.authorization_record.approved_amount_usdc,
                f.authorization_record.parameters_hash,
            )
            .unwrap_err(),
            CustomError::VictimReliefPayoutStatusMismatch.into()
        );
        f.victim_relief_case.active_appeal = Pubkey::default();

        let low_balance_vault = token_account(f.usdc_mint_key, f.vault_authority_key, 1);
        assert_eq!(
            validate_victim_relief_payout_common_v1(
                f.security_config_key,
                &f.security_config,
                f.config_key,
                &f.config,
                f.policy_key,
                &f.policy,
                &f.claimant_state,
                f.case_key,
                &f.victim_relief_case,
                f.snapshot_key,
                &f.snapshot,
                f.payout_request_key,
                &f.payout_request,
                f.treasury_config_key,
                &f.treasury_config,
                f.relief_vault_key,
                &low_balance_vault,
                f.vault_authority_key,
                f.recipient_token_account_key,
                &f.recipient_token_account,
                f.usdc_mint_key,
                &f.usdc_mint,
                f.payout_receipt_key,
                &f.payout_receipt,
                VictimReliefPayoutOriginV1::OriginalApprove,
                GovernanceActionTypeV1::VictimReliefApproveCompensation,
                f.governance_proposal_action_key,
                f.authorization_record_key,
                f.authorization_record.approved_amount_usdc,
                f.authorization_record.parameters_hash,
            )
            .unwrap_err(),
            CustomError::VictimReliefPayoutInsufficientFunds.into()
        );

        let occupied_receipt = ReliefPayoutExecutionRecordV1 {
            payout_request: f.payout_request_key,
            ..empty_payout_execution_record()
        };
        assert_eq!(
            validate_victim_relief_payout_common_v1(
                f.security_config_key,
                &f.security_config,
                f.config_key,
                &f.config,
                f.policy_key,
                &f.policy,
                &f.claimant_state,
                f.case_key,
                &f.victim_relief_case,
                f.snapshot_key,
                &f.snapshot,
                f.payout_request_key,
                &f.payout_request,
                f.treasury_config_key,
                &f.treasury_config,
                f.relief_vault_key,
                &f.relief_vault,
                f.vault_authority_key,
                f.recipient_token_account_key,
                &f.recipient_token_account,
                f.usdc_mint_key,
                &f.usdc_mint,
                f.payout_receipt_key,
                &occupied_receipt,
                VictimReliefPayoutOriginV1::OriginalApprove,
                GovernanceActionTypeV1::VictimReliefApproveCompensation,
                f.governance_proposal_action_key,
                f.authorization_record_key,
                f.authorization_record.approved_amount_usdc,
                f.authorization_record.parameters_hash,
            )
            .unwrap_err(),
            CustomError::VictimReliefPayoutReceiptAlreadyExists.into()
        );
    }

    #[test]
    fn appeal_overturn_authorization_validator_accepts_only_overturn_receipt() {
        let mut f = original_payout_fixture();
        let appeal_key = Pubkey::new_unique();
        let proposer = Pubkey::new_unique();
        let governance_proposal_key = Pubkey::new_unique();
        let governance_proposal_action_key = Pubkey::new_unique();
        let proposal_decision_key = Pubkey::new_unique();
        let execution_queue_item_key = Pubkey::new_unique();
        let (
            mut governance_proposal,
            mut governance_proposal_action,
            proposal_decision,
            execution_queue_item,
        ) = governance_accounts_fixture(
            GovernanceActionTypeV1::VictimReliefOverturnAppeal,
            appeal_key,
            proposer,
            99,
            5_000,
        );
        governance_proposal_action.governance_proposal = governance_proposal_key;
        governance_proposal.payload_hash = governance_proposal_action.canonical_payload_hash;
        f.payout_request.governance_proposal = governance_proposal_key;
        f.payout_request.proposal_decision = proposal_decision_key;
        f.payout_request.execution_queue_item = execution_queue_item_key;
        f.payout_request.parameters_hash = governance_proposal_action.parameters_hash;
        f.victim_relief_case.decision_proposal = governance_proposal_key;
        f.victim_relief_case.decision_queue = execution_queue_item_key;
        let original_queue = Pubkey::new_unique();
        let (original_decision_record_key, original_bump) = Pubkey::find_program_address(
            &[
                VICTIM_RELIEF_DECISION_EXECUTION_RECORD_V1_SEED,
                original_queue.as_ref(),
            ],
            &crate::ID,
        );
        let original_decision_record = VictimReliefDecisionExecutionRecordV1 {
            execution_queue_item: original_queue,
            proposal_decision: Pubkey::new_unique(),
            governance_proposal: Pubkey::new_unique(),
            governance_proposal_action: Pubkey::new_unique(),
            module_registry: Pubkey::new_unique(),
            config: f.config_key,
            policy: f.policy_key,
            victim_relief_case: f.case_key,
            evidence_snapshot: f.snapshot_key,
            execution_type: VictimReliefDecisionExecutionTypeV1::Reject,
            governance_action_type: GovernanceActionTypeV1::VictimReliefRejectClaim,
            case_status_before: VictimReliefCaseStatusV1::UnderReview,
            case_status_after: VictimReliefCaseStatusV1::Rejected,
            claimed_amount_usdc: f.victim_relief_case.claimed_amount_usdc,
            approved_amount_usdc: 0,
            recipient_owner: f.victim_relief_case.recipient_owner,
            recipient_token_account: f.victim_relief_case.recipient_token_account,
            parameters_hash: [8; 32],
            canonical_governance_payload_hash: [9; 32],
            executor: Pubkey::new_unique(),
            executed_at: 4_000,
            schema_version: VICTIM_RELIEF_DECISION_SCHEMA_VERSION,
            bump: original_bump,
        };
        let mut appeal = empty_appeal();
        appeal.victim_relief_case = f.case_key;
        appeal.config = f.config_key;
        appeal.policy = f.policy_key;
        appeal.policy_version = f.victim_relief_case.policy_version;
        appeal.claimant = f.victim_relief_case.claimant;
        appeal.original_evidence_snapshot = f.snapshot_key;
        appeal.original_decision_record = original_decision_record_key;
        appeal.original_governance_proposal = original_decision_record.governance_proposal;
        appeal.original_execution_queue_item = original_queue;
        appeal.status = VictimReliefAppealStatusV1::Overturned;
        appeal.decision_proposal = governance_proposal_key;
        appeal.decision_queue = execution_queue_item_key;
        appeal.schema_version = VICTIM_RELIEF_SCHEMA_VERSION_V1;
        let (authorization_record_key, authorization_bump) = Pubkey::find_program_address(
            &[
                VICTIM_RELIEF_APPEAL_DECISION_EXECUTION_RECORD_V1_SEED,
                execution_queue_item_key.as_ref(),
            ],
            &crate::ID,
        );
        let mut authorization_record = VictimReliefAppealDecisionExecutionRecordV1 {
            execution_queue_item: execution_queue_item_key,
            proposal_decision: proposal_decision_key,
            governance_proposal: governance_proposal_key,
            governance_proposal_action: governance_proposal_action_key,
            module_registry: Pubkey::new_unique(),
            config: f.config_key,
            policy: f.policy_key,
            victim_relief_case: f.case_key,
            victim_relief_appeal: appeal_key,
            original_decision_record: original_decision_record_key,
            original_evidence_snapshot: f.snapshot_key,
            execution_type: VictimReliefAppealExecutionTypeV1::Overturn,
            governance_action_type: GovernanceActionTypeV1::VictimReliefOverturnAppeal,
            case_status_before: VictimReliefCaseStatusV1::AppealPending,
            case_status_after: VictimReliefCaseStatusV1::PayoutQueued,
            appeal_status_before: VictimReliefAppealStatusV1::Pending,
            appeal_status_after: VictimReliefAppealStatusV1::Overturned,
            claimed_amount_usdc: f.victim_relief_case.claimed_amount_usdc,
            approved_amount_usdc: f.payout_request.approved_amount_usdc,
            recipient_owner: f.payout_request.recipient_owner,
            recipient_token_account: f.payout_request.recipient_token_account,
            parameters_hash: f.payout_request.parameters_hash,
            canonical_governance_payload_hash: governance_proposal_action.canonical_payload_hash,
            executor: Pubkey::new_unique(),
            executed_at: 5_200,
            schema_version: VICTIM_RELIEF_DECISION_SCHEMA_VERSION,
            bump: authorization_bump,
        };

        validate_victim_relief_appeal_overturn_authorization_v1(
            governance_proposal_key,
            &governance_proposal,
            governance_proposal_action_key,
            &governance_proposal_action,
            proposal_decision_key,
            &proposal_decision,
            execution_queue_item_key,
            &execution_queue_item,
            appeal_key,
            &appeal,
            authorization_record_key,
            &authorization_record,
            original_decision_record_key,
            &original_decision_record,
            f.snapshot_key,
            &f.snapshot,
            f.case_key,
            &f.victim_relief_case,
            f.payout_request_key,
            &f.payout_request,
        )
        .unwrap();

        authorization_record.execution_type = VictimReliefAppealExecutionTypeV1::Uphold;
        assert_eq!(
            validate_victim_relief_appeal_overturn_authorization_v1(
                governance_proposal_key,
                &governance_proposal,
                governance_proposal_action_key,
                &governance_proposal_action,
                proposal_decision_key,
                &proposal_decision,
                execution_queue_item_key,
                &execution_queue_item,
                appeal_key,
                &appeal,
                authorization_record_key,
                &authorization_record,
                original_decision_record_key,
                &original_decision_record,
                f.snapshot_key,
                &f.snapshot,
                f.case_key,
                &f.victim_relief_case,
                f.payout_request_key,
                &f.payout_request,
            )
            .unwrap_err(),
            CustomError::VictimReliefPayoutAuthorizationMismatch.into()
        );
    }

    #[test]
    fn victim_relief_pdas_are_stable() {
        let (config_pda, _) =
            Pubkey::find_program_address(&[VICTIM_RELIEF_CONFIG_V1_SEED], &crate::ID);
        let (policy_pda, _) = Pubkey::find_program_address(
            &[
                VICTIM_RELIEF_POLICY_V1_SEED,
                config_pda.as_ref(),
                &VICTIM_RELIEF_POLICY_VERSION_V1.to_le_bytes(),
            ],
            &crate::ID,
        );
        let claimant = Pubkey::new_unique();
        let (claimant_state_pda, _) = Pubkey::find_program_address(
            &[
                VICTIM_RELIEF_CLAIMANT_STATE_V1_SEED,
                config_pda.as_ref(),
                claimant.as_ref(),
            ],
            &crate::ID,
        );
        let (case_pda, _) = Pubkey::find_program_address(
            &[
                VICTIM_RELIEF_CASE_V1_SEED,
                config_pda.as_ref(),
                &1u64.to_le_bytes(),
            ],
            &crate::ID,
        );
        let (snapshot_pda, _) = Pubkey::find_program_address(
            &[VICTIM_RELIEF_EVIDENCE_SNAPSHOT_V1_SEED, case_pda.as_ref()],
            &crate::ID,
        );
        let (payout_request_pda, _) = Pubkey::find_program_address(
            &[RELIEF_PAYOUT_REQUEST_V1_SEED, case_pda.as_ref()],
            &crate::ID,
        );
        let (payout_execution_receipt_pda, _) = Pubkey::find_program_address(
            &[
                RELIEF_PAYOUT_EXECUTION_RECORD_V1_SEED,
                payout_request_pda.as_ref(),
            ],
            &crate::ID,
        );
        let (appeal_pda, _) = Pubkey::find_program_address(
            &[VICTIM_RELIEF_APPEAL_V1_SEED, case_pda.as_ref()],
            &crate::ID,
        );
        let queue = Pubkey::new_unique();
        let (receipt_pda, _) = Pubkey::find_program_address(
            &[
                VICTIM_RELIEF_DECISION_EXECUTION_RECORD_V1_SEED,
                queue.as_ref(),
            ],
            &crate::ID,
        );
        let (appeal_receipt_pda, _) = Pubkey::find_program_address(
            &[
                VICTIM_RELIEF_APPEAL_DECISION_EXECUTION_RECORD_V1_SEED,
                queue.as_ref(),
            ],
            &crate::ID,
        );

        assert_ne!(config_pda, Pubkey::default());
        assert_ne!(policy_pda, Pubkey::default());
        assert_ne!(claimant_state_pda, Pubkey::default());
        assert_ne!(case_pda, Pubkey::default());
        assert_ne!(snapshot_pda, Pubkey::default());
        assert_ne!(payout_request_pda, Pubkey::default());
        assert_ne!(payout_execution_receipt_pda, Pubkey::default());
        assert_ne!(appeal_pda, Pubkey::default());
        assert_ne!(receipt_pda, Pubkey::default());
        assert_ne!(appeal_receipt_pda, Pubkey::default());
    }
}
