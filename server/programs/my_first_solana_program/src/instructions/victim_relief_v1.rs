use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

use crate::constants::{
    EXECUTION_QUEUE_ITEM_V1_SEED, GOVERNANCE_CONFIG_V1_SEED, GOVERNANCE_PROPOSAL_ACTION_V1_SEED,
    GOVERNANCE_PROPOSAL_V1_SEED, PROPOSAL_DECISION_V1_SEED, PROTOCOL_MODULE_REGISTRY_V1_SEED,
    RELIEF_PAYOUT_REQUEST_V1_SEED, RELIEF_USDC_VAULT_SEED, TREASURY_CONFIG_V2_SEED,
    UNIVERSAL_GOVERNANCE_DECISION_ADAPTER_V1_SEED, VAULT_AUTHORITY_V2_SEED,
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
    ProtocolModuleRegistryV1, ReliefPayoutRequestV1, TreasuryConfigV2,
    UniversalGovernanceDecisionAdapterV1, VictimReliefCaseStatusV1, VictimReliefCaseV1,
    VictimReliefClaimantStateV1, VictimReliefConfigV1, VictimReliefDecisionExecutionRecordV1,
    VictimReliefDecisionExecutionTypeV1, VictimReliefDecisionParametersV1,
    VictimReliefEvidenceSnapshotV1, VictimReliefPayoutStatusV1, VictimReliefPolicyV1,
};

pub const VICTIM_RELIEF_DECISION_SCHEMA_VERSION: u16 = 1;
pub const VICTIM_RELIEF_DECISION_PARAMETERS_V1_DOMAIN_BYTES: [u8; 42] =
    *b"alpha_victim_relief_decision_parameters_v1";

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct VictimReliefDecisionParametersHashEnvelopeV1 {
    pub domain_separator: [u8; 42],
    pub parameters: VictimReliefDecisionParametersV1,
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
        let queue = Pubkey::new_unique();
        let (receipt_pda, _) = Pubkey::find_program_address(
            &[
                VICTIM_RELIEF_DECISION_EXECUTION_RECORD_V1_SEED,
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
        assert_ne!(receipt_pda, Pubkey::default());
    }
}
