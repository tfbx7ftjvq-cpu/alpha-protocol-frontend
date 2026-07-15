use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};

use crate::constants::{
    BASE_BOND_REFUND_BPS, BASE_BOND_TREASURY_BPS, BUILDERS_USDC_VAULT_SEED,
    BUYBACK_USDC_VAULT_SEED, DEFAULT_DISPUTE_WINDOW_SECONDS, DEFAULT_OBSERVATION_PERIOD_SECONDS,
    DEFAULT_RESPONSE_WINDOW_SECONDS, EXECUTION_QUEUE_ITEM_V1_SEED, GOVERNANCE_CONFIG_V1_SEED,
    GOVERNANCE_PROPOSAL_ACTION_V1_SEED, GOVERNANCE_PROPOSAL_V1_SEED,
    GREEN_BOND_VAULT_AUTHORITY_SEED, GREEN_BOND_VAULT_SEED, GREEN_LABEL_BRONZE_TIER_THRESHOLD_USDC,
    GREEN_LABEL_CERTIFICATION_EXECUTION_RECORD_SEED, GREEN_LABEL_CERTIFICATION_FEE_POLICY_SEED,
    GREEN_LABEL_CERTIFICATION_FEE_RECEIPT_SEED, GREEN_LABEL_CERTIFICATION_STATE_SEED,
    GREEN_LABEL_CONFIG_RESERVED_BYTES, GREEN_LABEL_CONFIG_SEED, GREEN_LABEL_CONFIG_SPACE,
    GREEN_LABEL_DISPUTE_RESERVED_BYTES, GREEN_LABEL_DISPUTE_SEED, GREEN_LABEL_DISPUTE_SPACE,
    GREEN_LABEL_FORFEIT_EXECUTION_RECORD_SEED, GREEN_LABEL_GOLD_TIER_THRESHOLD_USDC,
    GREEN_LABEL_PLATINUM_TIER_THRESHOLD_USDC, GREEN_LABEL_PROJECT_RESERVED_BYTES,
    GREEN_LABEL_PROJECT_SEED, GREEN_LABEL_PROJECT_SPACE, GREEN_LABEL_REFUNDABLE_ESCROW_SEED,
    GREEN_LABEL_REFUNDABLE_VAULT_SEED, GREEN_LABEL_REFUND_EXECUTION_RECORD_SEED,
    GREEN_LABEL_SILVER_TIER_THRESHOLD_USDC, GREEN_LABEL_USDC_DECIMALS, MAX_BPS,
    MIN_GREEN_LABEL_BASE_BOND_USDC, PROPOSAL_DECISION_V1_SEED, PROTOCOL_MODULE_REGISTRY_V1_SEED,
    RELIEF_USDC_VAULT_SEED, REVENUE_ROUTING_STATS_V1_SEED, STAKING_USDC_VAULT_SEED,
    TREASURY_CONFIG_V2_SEED, TREASURY_USDC_STATE_V2_SEED,
    UNIVERSAL_GOVERNANCE_DECISION_ADAPTER_V1_SEED, VAULT_AUTHORITY_V2_SEED,
};
use crate::error::CustomError;
use crate::instructions::contributor_v1::hash_contributor_payload;
use crate::instructions::deposit_usdc_revenue::route_usdc_revenue_from_token_account;
use crate::instructions::governance_action_v1::map_governance_action_to_security_action;
use crate::instructions::governance_v1::validate_governance_proposal_action_v1;
use crate::instructions::protocol_module_registry_v1::{
    protocol_module_stable_code_v1, validate_protocol_module_registry_v1,
};
use crate::state::{
    ActionType, BondTier, DisputeStatus, ExecutionQueueItemV1, ExecutionStatus,
    GovernanceActionTypeV1, GovernanceConfigV1, GovernanceProposalActionV1,
    GovernanceProposalStatusV1, GovernanceProposalV1, GreenLabelCertificationExecutionRecordV1,
    GreenLabelCertificationExecutionTypeV1, GreenLabelCertificationFeePolicyV1,
    GreenLabelCertificationFeeReceiptV1, GreenLabelCertificationStateV1,
    GreenLabelCertificationStatusV1, GreenLabelConfigV1, GreenLabelDisputeV1,
    GreenLabelEscrowExecutionTypeV1, GreenLabelEscrowStatusV1, GreenLabelForfeitExecutionRecordV1,
    GreenLabelProjectV1, GreenLabelRefundExecutionRecordV1, GreenLabelRefundableEscrowV1,
    GreenLabelStatus, ProposalDecision, ProposalDecisionV1, ProposalType, ProtocolModuleIdV1,
    ProtocolModuleRegistryV1, RevenueRoutingStatsV1, RevenueType, RugReasonCode, TreasuryConfigV2,
    TreasuryUsdcStateV2, UniversalGovernanceDecisionAdapterV1,
};

pub const MAX_GREEN_LABEL_WINDOW_SECONDS: i64 = 30 * 24 * 60 * 60;
pub const GREEN_LABEL_CERTIFICATION_SCHEMA_VERSION: u16 = 1;
pub const GREEN_LABEL_CERTIFICATION_DECISION_PARAMETERS_V1_DOMAIN: &[u8] =
    b"alpha_green_label_certification_decision_v1";
pub const GREEN_LABEL_CERTIFICATION_FEE_SCHEMA_VERSION: u16 = 1;
pub const GREEN_LABEL_CERTIFICATION_FEE_POLICY_VERSION: u64 = 1;
pub const GREEN_LABEL_CERTIFICATION_FEE_PARAMETERS_V1_DOMAIN: &[u8] =
    b"alpha_green_label_certification_fee_parameters_v1";
pub const GREEN_LABEL_REFUND_SCHEMA_VERSION: u16 = 1;
pub const GREEN_LABEL_REFUND_PARAMETERS_V1_DOMAIN: &[u8] =
    b"alpha_green_label_refund_parameters_v1";
pub const GREEN_LABEL_FORFEIT_SCHEMA_VERSION: u16 = 1;
pub const GREEN_LABEL_FORFEIT_PARAMETERS_V1_DOMAIN: &[u8] =
    b"alpha_green_label_forfeit_parameters_v1";

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GreenLabelCertificationDecisionParametersV1 {
    pub schema_version: u16,
    pub green_label_config: Pubkey,
    pub green_label_project: Pubkey,
    pub certification_state: Pubkey,
    pub action_type: GovernanceActionTypeV1,
    pub project_authority: Pubkey,
    pub bond_tier: BondTier,
    pub bond_vault: Pubkey,
    pub usdc_mint: Pubkey,
    pub observation_end_ts: i64,
    pub expected_project_status: GreenLabelStatus,
    pub expected_certification_status: GreenLabelCertificationStatusV1,
    pub proposal_id: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GreenLabelCertificationFeeParametersV1 {
    pub schema_version: u16,
    pub green_label_config: Pubkey,
    pub fee_policy: Pubkey,
    pub policy_version: u64,
    pub green_label_project: Pubkey,
    pub project_id: u64,
    pub project_owner: Pubkey,
    pub payer: Pubkey,
    pub payer_token_account: Pubkey,
    pub fee_amount_usdc: u64,
    pub usdc_mint: Pubkey,
    pub treasury_config: Pubkey,
    pub treasury_usdc_state: Pubkey,
    pub revenue_routing_stats: Pubkey,
    pub relief_usdc_vault: Pubkey,
    pub buyback_usdc_vault: Pubkey,
    pub builders_usdc_vault: Pubkey,
    pub staking_usdc_vault: Pubkey,
    pub revenue_type: RevenueType,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GreenLabelRefundParametersV1 {
    pub schema_version: u16,
    pub green_label_config: Pubkey,
    pub green_label_project: Pubkey,
    pub green_label_dispute: Pubkey,
    pub refundable_escrow: Pubkey,
    pub refundable_vault: Pubkey,
    pub original_payer: Pubkey,
    pub payer_destination_token_account: Pubkey,
    pub refund_amount_usdc: u64,
    pub usdc_mint: Pubkey,
    pub expected_escrow_status: GreenLabelEscrowStatusV1,
    pub proposal_id: u64,
    pub action_type: GovernanceActionTypeV1,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GreenLabelForfeitParametersV1 {
    pub schema_version: u16,
    pub green_label_config: Pubkey,
    pub green_label_project: Pubkey,
    pub green_label_dispute: Pubkey,
    pub refundable_escrow: Pubkey,
    pub refundable_vault: Pubkey,
    pub forfeited_amount_usdc: u64,
    pub treasury_config: Pubkey,
    pub treasury_usdc_state: Pubkey,
    pub revenue_routing_stats: Pubkey,
    pub relief_usdc_vault: Pubkey,
    pub buyback_usdc_vault: Pubkey,
    pub builders_usdc_vault: Pubkey,
    pub staking_usdc_vault: Pubkey,
    pub usdc_mint: Pubkey,
    pub revenue_type: RevenueType,
    pub expected_escrow_status: GreenLabelEscrowStatusV1,
    pub expected_project_status: GreenLabelStatus,
    pub expected_dispute_status: DisputeStatus,
    pub action_type: GovernanceActionTypeV1,
    pub proposal_id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GreenLabelBondSplit {
    pub base_bond_amount: u64,
    pub extra_bond_amount: u64,
    pub total_bond_amount: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GreenLabelRefundAmounts {
    pub project_refund_amount: u64,
    pub treasury_amount: u64,
    pub base_refund_amount: u64,
    pub base_treasury_amount: u64,
    pub extra_refund_amount: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GreenLabelConfigInitValues {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GreenLabelProjectInitValues {
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

#[derive(Accounts)]
pub struct InitializeGreenLabelConfig<'info> {
    #[account(
        init,
        payer = authority,
        space = GREEN_LABEL_CONFIG_SPACE,
        seeds = [GREEN_LABEL_CONFIG_SEED],
        bump
    )]
    pub green_label_config: Account<'info, GreenLabelConfigV1>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: Phase 1C stores the mint key only; token validation is added in later phases.
    pub usdc_mint: UncheckedAccount<'info>,

    /// CHECK: Phase 1C stores the Treasury V2 state key only.
    pub treasury_usdc_state_v2: UncheckedAccount<'info>,

    /// CHECK: Phase 1C stores the base bond treasury vault key only.
    pub base_bond_treasury_vault: UncheckedAccount<'info>,

    /// CHECK: Phase 1C stores the relief/risk vault key only.
    pub relief_or_risk_vault: UncheckedAccount<'info>,

    /// CHECK: Phase 1C stores the Treasury V2 vault authority key only.
    pub vault_authority_v2: UncheckedAccount<'info>,

    /// CHECK: Phase 1C stores the Security Layer governance config key only.
    pub security_governance_config: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateGreenLabelWindows<'info> {
    #[account(
        mut,
        seeds = [GREEN_LABEL_CONFIG_SEED],
        bump = green_label_config.bump
    )]
    pub green_label_config: Account<'info, GreenLabelConfigV1>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateGreenLabelMinBaseBond<'info> {
    #[account(
        mut,
        seeds = [GREEN_LABEL_CONFIG_SEED],
        bump = green_label_config.bump
    )]
    pub green_label_config: Account<'info, GreenLabelConfigV1>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(expected_project_id: u64)]
pub struct SubmitGreenLabelApplication<'info> {
    #[account(
        mut,
        seeds = [GREEN_LABEL_CONFIG_SEED],
        bump = green_label_config.bump
    )]
    pub green_label_config: Account<'info, GreenLabelConfigV1>,

    #[account(
        init,
        payer = project_owner,
        space = GREEN_LABEL_PROJECT_SPACE,
        seeds = [
            GREEN_LABEL_PROJECT_SEED,
            &expected_project_id.to_le_bytes()
        ],
        bump
    )]
    pub green_label_project: Account<'info, GreenLabelProjectV1>,

    #[account(mut)]
    pub project_owner: Signer<'info>,

    /// CHECK: Phase 1D-1 stores the mint key only; token validation is added in later phases.
    pub token_mint: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeGreenBondVault<'info> {
    #[account(
        seeds = [GREEN_LABEL_CONFIG_SEED],
        bump = green_label_config.bump
    )]
    pub green_label_config: Box<Account<'info, GreenLabelConfigV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_PROJECT_SEED,
            &green_label_project.project_id.to_le_bytes()
        ],
        bump = green_label_project.bump
    )]
    pub green_label_project: Box<Account<'info, GreenLabelProjectV1>>,

    #[account(
        init,
        payer = project_owner,
        seeds = [
            GREEN_BOND_VAULT_SEED,
            green_label_project.key().as_ref()
        ],
        bump,
        token::mint = usdc_mint,
        token::authority = green_bond_vault_authority
    )]
    pub green_bond_vault: Box<Account<'info, TokenAccount>>,

    /// CHECK: This PDA owns the project Green Bond Vault token account.
    #[account(
        seeds = [
            GREEN_BOND_VAULT_AUTHORITY_SEED,
            green_label_project.key().as_ref()
        ],
        bump
    )]
    pub green_bond_vault_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub project_owner: Signer<'info>,

    #[account(
        constraint = usdc_mint.key() == green_label_config.usdc_mint @ CustomError::InvalidGreenLabelMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct LockGreenLabelBond<'info> {
    #[account(
        seeds = [GREEN_LABEL_CONFIG_SEED],
        bump = green_label_config.bump
    )]
    pub green_label_config: Box<Account<'info, GreenLabelConfigV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_PROJECT_SEED,
            &green_label_project.project_id.to_le_bytes()
        ],
        bump = green_label_project.bump
    )]
    pub green_label_project: Box<Account<'info, GreenLabelProjectV1>>,

    pub project_owner: Signer<'info>,

    #[account(mut)]
    pub project_owner_usdc_ata: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub green_bond_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = usdc_mint.key() == green_label_config.usdc_mint @ CustomError::InvalidGreenLabelMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(expected_dispute_id: u64)]
pub struct OpenGreenLabelDispute<'info> {
    #[account(
        seeds = [GREEN_LABEL_CONFIG_SEED],
        bump = green_label_config.bump
    )]
    pub green_label_config: Box<Account<'info, GreenLabelConfigV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_PROJECT_SEED,
            &green_label_project.project_id.to_le_bytes()
        ],
        bump = green_label_project.bump
    )]
    pub green_label_project: Box<Account<'info, GreenLabelProjectV1>>,

    #[account(
        init,
        payer = disputer,
        space = GREEN_LABEL_DISPUTE_SPACE,
        seeds = [
            GREEN_LABEL_DISPUTE_SEED,
            green_label_project.key().as_ref(),
            &expected_dispute_id.to_le_bytes()
        ],
        bump
    )]
    pub green_label_dispute: Box<Account<'info, GreenLabelDisputeV1>>,

    #[account(mut)]
    pub disputer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MarkDisputeReadyForDecision<'info> {
    #[account(
        seeds = [GREEN_LABEL_CONFIG_SEED],
        bump = green_label_config.bump
    )]
    pub green_label_config: Box<Account<'info, GreenLabelConfigV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_PROJECT_SEED,
            &green_label_project.project_id.to_le_bytes()
        ],
        bump = green_label_project.bump
    )]
    pub green_label_project: Box<Account<'info, GreenLabelProjectV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_DISPUTE_SEED,
            green_label_project.key().as_ref(),
            &green_label_dispute.dispute_id.to_le_bytes()
        ],
        bump = green_label_dispute.bump
    )]
    pub green_label_dispute: Box<Account<'info, GreenLabelDisputeV1>>,

    pub caller: Signer<'info>,
}

#[derive(Accounts)]
pub struct LinkGreenLabelSecurityDecision<'info> {
    #[account(
        seeds = [GREEN_LABEL_CONFIG_SEED],
        bump = green_label_config.bump
    )]
    pub green_label_config: Box<Account<'info, GreenLabelConfigV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_PROJECT_SEED,
            &green_label_project.project_id.to_le_bytes()
        ],
        bump = green_label_project.bump
    )]
    pub green_label_project: Box<Account<'info, GreenLabelProjectV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_DISPUTE_SEED,
            green_label_project.key().as_ref(),
            &green_label_dispute.dispute_id.to_le_bytes()
        ],
        bump = green_label_dispute.bump
    )]
    pub green_label_dispute: Box<Account<'info, GreenLabelDisputeV1>>,

    pub proposal_decision: Box<Account<'info, ProposalDecisionV1>>,

    pub execution_queue_item: Box<Account<'info, ExecutionQueueItemV1>>,

    pub linker: Signer<'info>,
}

#[derive(Accounts)]
pub struct ExecuteGreenLabelRefund<'info> {
    #[account(
        seeds = [GREEN_LABEL_CONFIG_SEED],
        bump = green_label_config.bump
    )]
    pub green_label_config: Box<Account<'info, GreenLabelConfigV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_PROJECT_SEED,
            &green_label_project.project_id.to_le_bytes()
        ],
        bump = green_label_project.bump
    )]
    pub green_label_project: Box<Account<'info, GreenLabelProjectV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_DISPUTE_SEED,
            green_label_project.key().as_ref(),
            &green_label_dispute.dispute_id.to_le_bytes()
        ],
        bump = green_label_dispute.bump
    )]
    pub green_label_dispute: Box<Account<'info, GreenLabelDisputeV1>>,

    pub proposal_decision: Box<Account<'info, ProposalDecisionV1>>,

    pub execution_queue_item: Box<Account<'info, ExecutionQueueItemV1>>,

    #[account(mut)]
    pub green_bond_vault: Box<Account<'info, TokenAccount>>,

    /// CHECK: This PDA signs transfers from the project Green Bond Vault.
    #[account(
        seeds = [
            GREEN_BOND_VAULT_AUTHORITY_SEED,
            green_label_project.key().as_ref()
        ],
        bump
    )]
    pub green_bond_vault_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub project_owner_usdc_ata: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub base_bond_treasury_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = usdc_mint.key() == green_label_config.usdc_mint @ CustomError::InvalidGreenLabelMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    pub executor: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ExecuteGreenLabelSlash<'info> {
    #[account(
        seeds = [GREEN_LABEL_CONFIG_SEED],
        bump = green_label_config.bump
    )]
    pub green_label_config: Box<Account<'info, GreenLabelConfigV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_PROJECT_SEED,
            &green_label_project.project_id.to_le_bytes()
        ],
        bump = green_label_project.bump
    )]
    pub green_label_project: Box<Account<'info, GreenLabelProjectV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_DISPUTE_SEED,
            green_label_project.key().as_ref(),
            &green_label_dispute.dispute_id.to_le_bytes()
        ],
        bump = green_label_dispute.bump
    )]
    pub green_label_dispute: Box<Account<'info, GreenLabelDisputeV1>>,

    pub proposal_decision: Box<Account<'info, ProposalDecisionV1>>,

    pub execution_queue_item: Box<Account<'info, ExecutionQueueItemV1>>,

    #[account(mut)]
    pub green_bond_vault: Box<Account<'info, TokenAccount>>,

    /// CHECK: This PDA signs transfers from the project Green Bond Vault.
    #[account(
        seeds = [
            GREEN_BOND_VAULT_AUTHORITY_SEED,
            green_label_project.key().as_ref()
        ],
        bump
    )]
    pub green_bond_vault_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub relief_or_risk_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = usdc_mint.key() == green_label_config.usdc_mint @ CustomError::InvalidGreenLabelMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    pub executor: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct InitializeGreenLabelRefundableEscrowV1<'info> {
    #[account(
        seeds = [GREEN_LABEL_CONFIG_SEED],
        bump = green_label_config.bump
    )]
    pub green_label_config: Box<Account<'info, GreenLabelConfigV1>>,

    #[account(
        seeds = [
            GREEN_LABEL_PROJECT_SEED,
            &green_label_project.project_id.to_le_bytes()
        ],
        bump = green_label_project.bump
    )]
    pub green_label_project: Box<Account<'info, GreenLabelProjectV1>>,

    #[account(
        init,
        payer = payer,
        space = 8 + GreenLabelRefundableEscrowV1::INIT_SPACE,
        seeds = [
            GREEN_LABEL_REFUNDABLE_ESCROW_SEED,
            green_label_project.key().as_ref()
        ],
        bump
    )]
    pub green_label_refundable_escrow: Box<Account<'info, GreenLabelRefundableEscrowV1>>,

    #[account(
        init,
        payer = payer,
        seeds = [
            GREEN_LABEL_REFUNDABLE_VAULT_SEED,
            green_label_refundable_escrow.key().as_ref()
        ],
        bump,
        token::mint = usdc_mint,
        token::authority = green_label_refundable_escrow
    )]
    pub refundable_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        constraint = usdc_mint.key() == green_label_config.usdc_mint @ CustomError::InvalidGreenLabelMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct DepositGreenLabelRefundableBondV1<'info> {
    #[account(
        seeds = [
            GREEN_LABEL_PROJECT_SEED,
            &green_label_project.project_id.to_le_bytes()
        ],
        bump = green_label_project.bump
    )]
    pub green_label_project: Box<Account<'info, GreenLabelProjectV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_REFUNDABLE_ESCROW_SEED,
            green_label_project.key().as_ref()
        ],
        bump = green_label_refundable_escrow.bump,
        constraint = green_label_refundable_escrow.project == green_label_project.key() @ CustomError::InvalidGreenLabelTargetAccount,
        constraint = green_label_refundable_escrow.payer == payer.key() @ CustomError::InvalidGreenLabelProjectOwner
    )]
    pub green_label_refundable_escrow: Box<Account<'info, GreenLabelRefundableEscrowV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_REFUNDABLE_VAULT_SEED,
            green_label_refundable_escrow.key().as_ref()
        ],
        bump = green_label_refundable_escrow.vault_bump,
        constraint = refundable_vault.mint == green_label_refundable_escrow.usdc_mint @ CustomError::InvalidGreenLabelMint,
        constraint = refundable_vault.owner == green_label_refundable_escrow.key() @ CustomError::InvalidGreenLabelTokenAccount
    )]
    pub refundable_vault: Box<Account<'info, TokenAccount>>,

    pub payer: Signer<'info>,

    #[account(
        mut,
        constraint = payer_usdc_account.owner == payer.key() @ CustomError::InvalidGreenLabelTokenAccount,
        constraint = payer_usdc_account.mint == green_label_refundable_escrow.usdc_mint @ CustomError::InvalidGreenLabelMint
    )]
    pub payer_usdc_account: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = usdc_mint.key() == green_label_refundable_escrow.usdc_mint @ CustomError::InvalidGreenLabelMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct InitializeGreenLabelCertificationFeePolicyV1<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub authority: Signer<'info>,

    #[account(
        seeds = [GREEN_LABEL_CONFIG_SEED],
        bump = green_label_config.bump,
        constraint = green_label_config.authority == authority.key() @ CustomError::UnauthorizedGreenLabelAuthority
    )]
    pub green_label_config: Box<Account<'info, GreenLabelConfigV1>>,

    #[account(
        init,
        payer = payer,
        space = 8 + GreenLabelCertificationFeePolicyV1::INIT_SPACE,
        seeds = [
            GREEN_LABEL_CERTIFICATION_FEE_POLICY_SEED,
            green_label_config.key().as_ref()
        ],
        bump
    )]
    pub green_label_certification_fee_policy:
        Box<Account<'info, GreenLabelCertificationFeePolicyV1>>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RouteGreenLabelCertificationFeeV1<'info> {
    #[account(
        seeds = [GREEN_LABEL_CONFIG_SEED],
        bump = green_label_config.bump
    )]
    pub green_label_config: Box<Account<'info, GreenLabelConfigV1>>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        constraint = payer_usdc_account.mint == green_label_config.usdc_mint @ CustomError::InvalidGreenLabelMint,
        constraint = payer_usdc_account.owner == payer.key() @ CustomError::InvalidGreenLabelTokenAccount
    )]
    pub payer_usdc_account: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [TREASURY_CONFIG_V2_SEED],
        bump = treasury_config.bump,
        constraint = treasury_config.usdc_mint == green_label_config.usdc_mint @ CustomError::InvalidGreenLabelMint
    )]
    pub treasury_config: Box<Account<'info, TreasuryConfigV2>>,

    #[account(
        mut,
        seeds = [TREASURY_USDC_STATE_V2_SEED],
        bump = treasury_usdc_state.bump
    )]
    pub treasury_usdc_state: Box<Account<'info, TreasuryUsdcStateV2>>,

    #[account(
        mut,
        seeds = [REVENUE_ROUTING_STATS_V1_SEED, treasury_config.key().as_ref()],
        bump = revenue_routing_stats.bump,
        constraint = revenue_routing_stats.authority == treasury_config.authority @ CustomError::UnauthorizedTreasuryAuthority,
        constraint = revenue_routing_stats.usdc_mint == treasury_config.usdc_mint @ CustomError::InvalidMint
    )]
    pub revenue_routing_stats: Box<Account<'info, RevenueRoutingStatsV1>>,

    #[account(
        constraint = usdc_mint.key() == treasury_config.usdc_mint @ CustomError::InvalidGreenLabelMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    /// CHECK: This PDA only owns the USDC vault token accounts.
    #[account(
        seeds = [VAULT_AUTHORITY_V2_SEED],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [RELIEF_USDC_VAULT_SEED],
        bump,
        constraint = relief_usdc_vault.mint == treasury_config.usdc_mint @ CustomError::InvalidMint,
        constraint = relief_usdc_vault.owner == vault_authority.key() @ CustomError::InvalidVault
    )]
    pub relief_usdc_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [BUYBACK_USDC_VAULT_SEED],
        bump,
        constraint = buyback_usdc_vault.mint == treasury_config.usdc_mint @ CustomError::InvalidMint,
        constraint = buyback_usdc_vault.owner == vault_authority.key() @ CustomError::InvalidVault
    )]
    pub buyback_usdc_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [BUILDERS_USDC_VAULT_SEED],
        bump,
        constraint = builders_usdc_vault.mint == treasury_config.usdc_mint @ CustomError::InvalidMint,
        constraint = builders_usdc_vault.owner == vault_authority.key() @ CustomError::InvalidVault
    )]
    pub builders_usdc_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [STAKING_USDC_VAULT_SEED],
        bump,
        constraint = staking_usdc_vault.mint == treasury_config.usdc_mint @ CustomError::InvalidMint,
        constraint = staking_usdc_vault.owner == vault_authority.key() @ CustomError::InvalidVault
    )]
    pub staking_usdc_vault: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RouteGreenLabelCertificationFeeOnceV1<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        seeds = [GREEN_LABEL_CONFIG_SEED],
        bump = green_label_config.bump
    )]
    pub green_label_config: Box<Account<'info, GreenLabelConfigV1>>,

    #[account(
        seeds = [
            GREEN_LABEL_PROJECT_SEED,
            &green_label_project.project_id.to_le_bytes()
        ],
        bump = green_label_project.bump
    )]
    pub green_label_project: Box<Account<'info, GreenLabelProjectV1>>,

    #[account(
        seeds = [
            GREEN_LABEL_CERTIFICATION_FEE_POLICY_SEED,
            green_label_config.key().as_ref()
        ],
        bump = green_label_certification_fee_policy.bump
    )]
    pub green_label_certification_fee_policy:
        Box<Account<'info, GreenLabelCertificationFeePolicyV1>>,

    #[account(
        init,
        payer = payer,
        space = 8 + GreenLabelCertificationFeeReceiptV1::INIT_SPACE,
        seeds = [
            GREEN_LABEL_CERTIFICATION_FEE_RECEIPT_SEED,
            green_label_project.key().as_ref()
        ],
        bump
    )]
    pub green_label_certification_fee_receipt:
        Box<Account<'info, GreenLabelCertificationFeeReceiptV1>>,

    #[account(
        mut,
        constraint = payer_usdc_account.mint == green_label_config.usdc_mint @ CustomError::GreenLabelCertificationFeeMintMismatch,
        constraint = payer_usdc_account.owner == payer.key() @ CustomError::GreenLabelCertificationFeePayerMismatch
    )]
    pub payer_usdc_account: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [TREASURY_CONFIG_V2_SEED],
        bump = treasury_config.bump,
        constraint = treasury_config.usdc_mint == green_label_config.usdc_mint @ CustomError::GreenLabelCertificationFeeMintMismatch
    )]
    pub treasury_config: Box<Account<'info, TreasuryConfigV2>>,

    #[account(
        mut,
        seeds = [TREASURY_USDC_STATE_V2_SEED],
        bump = treasury_usdc_state.bump
    )]
    pub treasury_usdc_state: Box<Account<'info, TreasuryUsdcStateV2>>,

    #[account(
        mut,
        seeds = [REVENUE_ROUTING_STATS_V1_SEED, treasury_config.key().as_ref()],
        bump = revenue_routing_stats.bump,
        constraint = revenue_routing_stats.authority == treasury_config.authority @ CustomError::UnauthorizedTreasuryAuthority,
        constraint = revenue_routing_stats.usdc_mint == treasury_config.usdc_mint @ CustomError::GreenLabelCertificationFeeMintMismatch
    )]
    pub revenue_routing_stats: Box<Account<'info, RevenueRoutingStatsV1>>,

    #[account(
        constraint = usdc_mint.key() == green_label_config.usdc_mint @ CustomError::GreenLabelCertificationFeeMintMismatch
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    /// CHECK: This PDA only owns the USDC vault token accounts.
    #[account(
        seeds = [VAULT_AUTHORITY_V2_SEED],
        bump,
        constraint = vault_authority.key() == green_label_config.vault_authority_v2 @ CustomError::GreenLabelCertificationFeeTreasuryMismatch
    )]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [RELIEF_USDC_VAULT_SEED],
        bump,
        constraint = relief_usdc_vault.mint == treasury_config.usdc_mint @ CustomError::GreenLabelCertificationFeeMintMismatch,
        constraint = relief_usdc_vault.owner == vault_authority.key() @ CustomError::GreenLabelCertificationFeeTreasuryMismatch
    )]
    pub relief_usdc_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [BUYBACK_USDC_VAULT_SEED],
        bump,
        constraint = buyback_usdc_vault.mint == treasury_config.usdc_mint @ CustomError::GreenLabelCertificationFeeMintMismatch,
        constraint = buyback_usdc_vault.owner == vault_authority.key() @ CustomError::GreenLabelCertificationFeeTreasuryMismatch
    )]
    pub buyback_usdc_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [BUILDERS_USDC_VAULT_SEED],
        bump,
        constraint = builders_usdc_vault.mint == treasury_config.usdc_mint @ CustomError::GreenLabelCertificationFeeMintMismatch,
        constraint = builders_usdc_vault.owner == vault_authority.key() @ CustomError::GreenLabelCertificationFeeTreasuryMismatch
    )]
    pub builders_usdc_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [STAKING_USDC_VAULT_SEED],
        bump,
        constraint = staking_usdc_vault.mint == treasury_config.usdc_mint @ CustomError::GreenLabelCertificationFeeMintMismatch,
        constraint = staking_usdc_vault.owner == vault_authority.key() @ CustomError::GreenLabelCertificationFeeTreasuryMismatch
    )]
    pub staking_usdc_vault: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeGreenLabelCertificationStateV1<'info> {
    #[account(
        seeds = [GREEN_LABEL_CONFIG_SEED],
        bump = green_label_config.bump
    )]
    pub green_label_config: Box<Account<'info, GreenLabelConfigV1>>,

    #[account(
        seeds = [
            GREEN_LABEL_PROJECT_SEED,
            &green_label_project.project_id.to_le_bytes()
        ],
        bump = green_label_project.bump
    )]
    pub green_label_project: Box<Account<'info, GreenLabelProjectV1>>,

    #[account(
        init,
        payer = payer,
        space = 8 + GreenLabelCertificationStateV1::INIT_SPACE,
        seeds = [
            GREEN_LABEL_CERTIFICATION_STATE_SEED,
            green_label_project.key().as_ref()
        ],
        bump
    )]
    pub green_label_certification_state: Box<Account<'info, GreenLabelCertificationStateV1>>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteGreenLabelApproveCertificationV1<'info> {
    #[account(seeds = [GREEN_LABEL_CONFIG_SEED], bump = green_label_config.bump)]
    pub green_label_config: Box<Account<'info, GreenLabelConfigV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_PROJECT_SEED,
            &green_label_project.project_id.to_le_bytes()
        ],
        bump = green_label_project.bump
    )]
    pub green_label_project: Box<Account<'info, GreenLabelProjectV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_CERTIFICATION_STATE_SEED,
            green_label_project.key().as_ref()
        ],
        bump = green_label_certification_state.bump
    )]
    pub green_label_certification_state: Box<Account<'info, GreenLabelCertificationStateV1>>,

    #[account(seeds = [GOVERNANCE_CONFIG_V1_SEED], bump = security_governance_config.bump)]
    pub security_governance_config: Box<Account<'info, GovernanceConfigV1>>,

    #[account(
        seeds = [
            PROTOCOL_MODULE_REGISTRY_V1_SEED,
            &[protocol_module_stable_code_v1(ProtocolModuleIdV1::GreenLabel)]
        ],
        bump = protocol_module_registry.bump
    )]
    pub protocol_module_registry: Box<Account<'info, ProtocolModuleRegistryV1>>,

    #[account(
        seeds = [
            GOVERNANCE_PROPOSAL_V1_SEED,
            &governance_proposal.proposal_id.to_le_bytes()
        ],
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
        seeds = [
            PROPOSAL_DECISION_V1_SEED,
            &governance_proposal.proposal_id.to_le_bytes()
        ],
        bump = proposal_decision.bump
    )]
    pub proposal_decision: Box<Account<'info, ProposalDecisionV1>>,

    #[account(
        seeds = [
            EXECUTION_QUEUE_ITEM_V1_SEED,
            &governance_proposal.proposal_id.to_le_bytes()
        ],
        bump = execution_queue_item.bump
    )]
    pub execution_queue_item: Box<Account<'info, ExecutionQueueItemV1>>,

    #[account(
        init,
        payer = executor,
        space = 8 + GreenLabelCertificationExecutionRecordV1::INIT_SPACE,
        seeds = [
            GREEN_LABEL_CERTIFICATION_EXECUTION_RECORD_SEED,
            execution_queue_item.key().as_ref()
        ],
        bump
    )]
    pub green_label_certification_execution_record:
        Box<Account<'info, GreenLabelCertificationExecutionRecordV1>>,

    #[account(mut)]
    pub green_bond_vault: Box<Account<'info, TokenAccount>>,

    #[account(constraint = usdc_mint.key() == green_label_config.usdc_mint @ CustomError::InvalidGreenLabelMint)]
    pub usdc_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub executor: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteGreenLabelRejectCertificationV1<'info> {
    #[account(seeds = [GREEN_LABEL_CONFIG_SEED], bump = green_label_config.bump)]
    pub green_label_config: Box<Account<'info, GreenLabelConfigV1>>,

    #[account(
        seeds = [
            GREEN_LABEL_PROJECT_SEED,
            &green_label_project.project_id.to_le_bytes()
        ],
        bump = green_label_project.bump
    )]
    pub green_label_project: Box<Account<'info, GreenLabelProjectV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_CERTIFICATION_STATE_SEED,
            green_label_project.key().as_ref()
        ],
        bump = green_label_certification_state.bump
    )]
    pub green_label_certification_state: Box<Account<'info, GreenLabelCertificationStateV1>>,

    #[account(seeds = [GOVERNANCE_CONFIG_V1_SEED], bump = security_governance_config.bump)]
    pub security_governance_config: Box<Account<'info, GovernanceConfigV1>>,

    #[account(
        seeds = [
            PROTOCOL_MODULE_REGISTRY_V1_SEED,
            &[protocol_module_stable_code_v1(ProtocolModuleIdV1::GreenLabel)]
        ],
        bump = protocol_module_registry.bump
    )]
    pub protocol_module_registry: Box<Account<'info, ProtocolModuleRegistryV1>>,

    #[account(
        seeds = [
            GOVERNANCE_PROPOSAL_V1_SEED,
            &governance_proposal.proposal_id.to_le_bytes()
        ],
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
        seeds = [
            PROPOSAL_DECISION_V1_SEED,
            &governance_proposal.proposal_id.to_le_bytes()
        ],
        bump = proposal_decision.bump
    )]
    pub proposal_decision: Box<Account<'info, ProposalDecisionV1>>,

    #[account(
        seeds = [
            EXECUTION_QUEUE_ITEM_V1_SEED,
            &governance_proposal.proposal_id.to_le_bytes()
        ],
        bump = execution_queue_item.bump
    )]
    pub execution_queue_item: Box<Account<'info, ExecutionQueueItemV1>>,

    #[account(
        init,
        payer = executor,
        space = 8 + GreenLabelCertificationExecutionRecordV1::INIT_SPACE,
        seeds = [
            GREEN_LABEL_CERTIFICATION_EXECUTION_RECORD_SEED,
            execution_queue_item.key().as_ref()
        ],
        bump
    )]
    pub green_label_certification_execution_record:
        Box<Account<'info, GreenLabelCertificationExecutionRecordV1>>,

    #[account(mut)]
    pub executor: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteGreenLabelRevokeCertificationV1<'info> {
    #[account(seeds = [GREEN_LABEL_CONFIG_SEED], bump = green_label_config.bump)]
    pub green_label_config: Box<Account<'info, GreenLabelConfigV1>>,

    #[account(
        seeds = [
            GREEN_LABEL_PROJECT_SEED,
            &green_label_project.project_id.to_le_bytes()
        ],
        bump = green_label_project.bump
    )]
    pub green_label_project: Box<Account<'info, GreenLabelProjectV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_CERTIFICATION_STATE_SEED,
            green_label_project.key().as_ref()
        ],
        bump = green_label_certification_state.bump
    )]
    pub green_label_certification_state: Box<Account<'info, GreenLabelCertificationStateV1>>,

    #[account(seeds = [GOVERNANCE_CONFIG_V1_SEED], bump = security_governance_config.bump)]
    pub security_governance_config: Box<Account<'info, GovernanceConfigV1>>,

    #[account(
        seeds = [
            PROTOCOL_MODULE_REGISTRY_V1_SEED,
            &[protocol_module_stable_code_v1(ProtocolModuleIdV1::GreenLabel)]
        ],
        bump = protocol_module_registry.bump
    )]
    pub protocol_module_registry: Box<Account<'info, ProtocolModuleRegistryV1>>,

    #[account(
        seeds = [
            GOVERNANCE_PROPOSAL_V1_SEED,
            &governance_proposal.proposal_id.to_le_bytes()
        ],
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
        seeds = [
            PROPOSAL_DECISION_V1_SEED,
            &governance_proposal.proposal_id.to_le_bytes()
        ],
        bump = proposal_decision.bump
    )]
    pub proposal_decision: Box<Account<'info, ProposalDecisionV1>>,

    #[account(
        seeds = [
            EXECUTION_QUEUE_ITEM_V1_SEED,
            &governance_proposal.proposal_id.to_le_bytes()
        ],
        bump = execution_queue_item.bump
    )]
    pub execution_queue_item: Box<Account<'info, ExecutionQueueItemV1>>,

    #[account(
        init,
        payer = executor,
        space = 8 + GreenLabelCertificationExecutionRecordV1::INIT_SPACE,
        seeds = [
            GREEN_LABEL_CERTIFICATION_EXECUTION_RECORD_SEED,
            execution_queue_item.key().as_ref()
        ],
        bump
    )]
    pub green_label_certification_execution_record:
        Box<Account<'info, GreenLabelCertificationExecutionRecordV1>>,

    #[account(mut)]
    pub executor: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteGreenLabelRefundNoDisputeGovernanceV1<'info> {
    #[account(seeds = [GREEN_LABEL_CONFIG_SEED], bump = green_label_config.bump)]
    pub green_label_config: Box<Account<'info, GreenLabelConfigV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_PROJECT_SEED,
            &green_label_project.project_id.to_le_bytes()
        ],
        bump = green_label_project.bump
    )]
    pub green_label_project: Box<Account<'info, GreenLabelProjectV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_REFUNDABLE_ESCROW_SEED,
            green_label_project.key().as_ref()
        ],
        bump = green_label_refundable_escrow.bump,
        constraint = green_label_refundable_escrow.project == green_label_project.key() @ CustomError::GreenLabelRefundTargetMismatch,
        constraint = green_label_refundable_escrow.usdc_mint == green_label_config.usdc_mint @ CustomError::GreenLabelRefundMintMismatch
    )]
    pub green_label_refundable_escrow: Box<Account<'info, GreenLabelRefundableEscrowV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_REFUNDABLE_VAULT_SEED,
            green_label_refundable_escrow.key().as_ref()
        ],
        bump = green_label_refundable_escrow.vault_bump,
        constraint = refundable_vault.mint == green_label_refundable_escrow.usdc_mint @ CustomError::GreenLabelRefundMintMismatch,
        constraint = refundable_vault.owner == green_label_refundable_escrow.key() @ CustomError::GreenLabelRefundVaultMismatch
    )]
    pub refundable_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = payer_refund_usdc_account.owner == green_label_refundable_escrow.payer @ CustomError::GreenLabelRefundWrongDestination,
        constraint = payer_refund_usdc_account.mint == green_label_refundable_escrow.usdc_mint @ CustomError::GreenLabelRefundMintMismatch
    )]
    pub payer_refund_usdc_account: Box<Account<'info, TokenAccount>>,

    #[account(constraint = usdc_mint.key() == green_label_refundable_escrow.usdc_mint @ CustomError::GreenLabelRefundMintMismatch)]
    pub usdc_mint: Box<Account<'info, Mint>>,

    #[account(seeds = [GOVERNANCE_CONFIG_V1_SEED], bump = security_governance_config.bump)]
    pub security_governance_config: Box<Account<'info, GovernanceConfigV1>>,

    #[account(
        seeds = [
            PROTOCOL_MODULE_REGISTRY_V1_SEED,
            &[protocol_module_stable_code_v1(ProtocolModuleIdV1::GreenLabel)]
        ],
        bump = protocol_module_registry.bump
    )]
    pub protocol_module_registry: Box<Account<'info, ProtocolModuleRegistryV1>>,

    #[account(
        seeds = [
            GOVERNANCE_PROPOSAL_V1_SEED,
            &governance_proposal.proposal_id.to_le_bytes()
        ],
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
        seeds = [
            PROPOSAL_DECISION_V1_SEED,
            &governance_proposal.proposal_id.to_le_bytes()
        ],
        bump = proposal_decision.bump
    )]
    pub proposal_decision: Box<Account<'info, ProposalDecisionV1>>,

    #[account(
        seeds = [
            EXECUTION_QUEUE_ITEM_V1_SEED,
            &governance_proposal.proposal_id.to_le_bytes()
        ],
        bump = execution_queue_item.bump
    )]
    pub execution_queue_item: Box<Account<'info, ExecutionQueueItemV1>>,

    #[account(
        init,
        payer = executor,
        space = 8 + GreenLabelRefundExecutionRecordV1::INIT_SPACE,
        seeds = [
            GREEN_LABEL_REFUND_EXECUTION_RECORD_SEED,
            execution_queue_item.key().as_ref()
        ],
        bump
    )]
    pub green_label_refund_execution_record: Box<Account<'info, GreenLabelRefundExecutionRecordV1>>,

    #[account(mut)]
    pub executor: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteGreenLabelRefundDisputeGovernanceV1<'info> {
    #[account(seeds = [GREEN_LABEL_CONFIG_SEED], bump = green_label_config.bump)]
    pub green_label_config: Box<Account<'info, GreenLabelConfigV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_PROJECT_SEED,
            &green_label_project.project_id.to_le_bytes()
        ],
        bump = green_label_project.bump
    )]
    pub green_label_project: Box<Account<'info, GreenLabelProjectV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_DISPUTE_SEED,
            green_label_project.key().as_ref(),
            &green_label_dispute.dispute_id.to_le_bytes()
        ],
        bump = green_label_dispute.bump,
        constraint = green_label_dispute.project == green_label_project.key() @ CustomError::GreenLabelRefundTargetMismatch
    )]
    pub green_label_dispute: Box<Account<'info, GreenLabelDisputeV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_REFUNDABLE_ESCROW_SEED,
            green_label_project.key().as_ref()
        ],
        bump = green_label_refundable_escrow.bump,
        constraint = green_label_refundable_escrow.project == green_label_project.key() @ CustomError::GreenLabelRefundTargetMismatch,
        constraint = green_label_refundable_escrow.usdc_mint == green_label_config.usdc_mint @ CustomError::GreenLabelRefundMintMismatch
    )]
    pub green_label_refundable_escrow: Box<Account<'info, GreenLabelRefundableEscrowV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_REFUNDABLE_VAULT_SEED,
            green_label_refundable_escrow.key().as_ref()
        ],
        bump = green_label_refundable_escrow.vault_bump,
        constraint = refundable_vault.mint == green_label_refundable_escrow.usdc_mint @ CustomError::GreenLabelRefundMintMismatch,
        constraint = refundable_vault.owner == green_label_refundable_escrow.key() @ CustomError::GreenLabelRefundVaultMismatch
    )]
    pub refundable_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = payer_refund_usdc_account.owner == green_label_refundable_escrow.payer @ CustomError::GreenLabelRefundWrongDestination,
        constraint = payer_refund_usdc_account.mint == green_label_refundable_escrow.usdc_mint @ CustomError::GreenLabelRefundMintMismatch
    )]
    pub payer_refund_usdc_account: Box<Account<'info, TokenAccount>>,

    #[account(constraint = usdc_mint.key() == green_label_refundable_escrow.usdc_mint @ CustomError::GreenLabelRefundMintMismatch)]
    pub usdc_mint: Box<Account<'info, Mint>>,

    #[account(seeds = [GOVERNANCE_CONFIG_V1_SEED], bump = security_governance_config.bump)]
    pub security_governance_config: Box<Account<'info, GovernanceConfigV1>>,

    #[account(
        seeds = [
            PROTOCOL_MODULE_REGISTRY_V1_SEED,
            &[protocol_module_stable_code_v1(ProtocolModuleIdV1::GreenLabel)]
        ],
        bump = protocol_module_registry.bump
    )]
    pub protocol_module_registry: Box<Account<'info, ProtocolModuleRegistryV1>>,

    #[account(
        seeds = [
            GOVERNANCE_PROPOSAL_V1_SEED,
            &governance_proposal.proposal_id.to_le_bytes()
        ],
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
        seeds = [
            PROPOSAL_DECISION_V1_SEED,
            &governance_proposal.proposal_id.to_le_bytes()
        ],
        bump = proposal_decision.bump
    )]
    pub proposal_decision: Box<Account<'info, ProposalDecisionV1>>,

    #[account(
        seeds = [
            EXECUTION_QUEUE_ITEM_V1_SEED,
            &governance_proposal.proposal_id.to_le_bytes()
        ],
        bump = execution_queue_item.bump
    )]
    pub execution_queue_item: Box<Account<'info, ExecutionQueueItemV1>>,

    #[account(
        init,
        payer = executor,
        space = 8 + GreenLabelRefundExecutionRecordV1::INIT_SPACE,
        seeds = [
            GREEN_LABEL_REFUND_EXECUTION_RECORD_SEED,
            execution_queue_item.key().as_ref()
        ],
        bump
    )]
    pub green_label_refund_execution_record: Box<Account<'info, GreenLabelRefundExecutionRecordV1>>,

    #[account(mut)]
    pub executor: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteGreenLabelForfeitGovernanceV1<'info> {
    #[account(
        seeds = [GREEN_LABEL_CONFIG_SEED],
        bump = green_label_config.bump
    )]
    pub green_label_config: Box<Account<'info, GreenLabelConfigV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_PROJECT_SEED,
            &green_label_project.project_id.to_le_bytes()
        ],
        bump = green_label_project.bump
    )]
    pub green_label_project: Box<Account<'info, GreenLabelProjectV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_DISPUTE_SEED,
            green_label_project.key().as_ref(),
            &green_label_dispute.dispute_id.to_le_bytes()
        ],
        bump = green_label_dispute.bump,
        constraint = green_label_dispute.project == green_label_project.key() @ CustomError::GreenLabelForfeitDisputeMismatch
    )]
    pub green_label_dispute: Box<Account<'info, GreenLabelDisputeV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_REFUNDABLE_ESCROW_SEED,
            green_label_project.key().as_ref()
        ],
        bump = green_label_refundable_escrow.bump,
        constraint = green_label_refundable_escrow.project == green_label_project.key() @ CustomError::GreenLabelForfeitTargetMismatch,
        constraint = green_label_refundable_escrow.usdc_mint == green_label_config.usdc_mint @ CustomError::GreenLabelForfeitMintMismatch
    )]
    pub green_label_refundable_escrow: Box<Account<'info, GreenLabelRefundableEscrowV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_REFUNDABLE_VAULT_SEED,
            green_label_refundable_escrow.key().as_ref()
        ],
        bump = green_label_refundable_escrow.vault_bump,
        constraint = refundable_vault.mint == green_label_refundable_escrow.usdc_mint @ CustomError::GreenLabelForfeitMintMismatch,
        constraint = refundable_vault.owner == green_label_refundable_escrow.key() @ CustomError::GreenLabelForfeitVaultMismatch
    )]
    pub refundable_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [GOVERNANCE_CONFIG_V1_SEED],
        bump = security_governance_config.bump
    )]
    pub security_governance_config: Box<Account<'info, GovernanceConfigV1>>,

    #[account(
        seeds = [
            PROTOCOL_MODULE_REGISTRY_V1_SEED,
            &[protocol_module_stable_code_v1(ProtocolModuleIdV1::GreenLabel)]
        ],
        bump = protocol_module_registry.bump
    )]
    pub protocol_module_registry: Box<Account<'info, ProtocolModuleRegistryV1>>,

    #[account(
        seeds = [
            GOVERNANCE_PROPOSAL_V1_SEED,
            &governance_proposal.proposal_id.to_le_bytes()
        ],
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
        seeds = [
            PROPOSAL_DECISION_V1_SEED,
            &governance_proposal.proposal_id.to_le_bytes()
        ],
        bump = proposal_decision.bump
    )]
    pub proposal_decision: Box<Account<'info, ProposalDecisionV1>>,

    #[account(
        seeds = [
            EXECUTION_QUEUE_ITEM_V1_SEED,
            &governance_proposal.proposal_id.to_le_bytes()
        ],
        bump = execution_queue_item.bump
    )]
    pub execution_queue_item: Box<Account<'info, ExecutionQueueItemV1>>,

    /// CHECK: Loaded and validated in handler to keep the Anchor accounts stack small.
    pub treasury_config: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Loaded, validated, and serialized in handler to keep the Anchor accounts stack small.
    pub treasury_usdc_state: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Loaded, validated, and serialized in handler to keep the Anchor accounts stack small.
    pub revenue_routing_stats: UncheckedAccount<'info>,

    #[account(
        constraint = usdc_mint.key() == green_label_refundable_escrow.usdc_mint @ CustomError::GreenLabelForfeitMintMismatch
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    /// CHECK: This PDA only owns the USDC vault token accounts.
    pub vault_authority: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Loaded and validated in handler to keep the Anchor accounts stack small.
    pub relief_usdc_vault: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Loaded and validated in handler to keep the Anchor accounts stack small.
    pub buyback_usdc_vault: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Loaded and validated in handler to keep the Anchor accounts stack small.
    pub builders_usdc_vault: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Loaded and validated in handler to keep the Anchor accounts stack small.
    pub staking_usdc_vault: UncheckedAccount<'info>,

    #[account(
        init,
        payer = executor,
        space = 8 + GreenLabelForfeitExecutionRecordV1::INIT_SPACE,
        seeds = [
            GREEN_LABEL_FORFEIT_EXECUTION_RECORD_SEED,
            execution_queue_item.key().as_ref()
        ],
        bump
    )]
    pub green_label_forfeit_execution_record:
        Box<Account<'info, GreenLabelForfeitExecutionRecordV1>>,

    #[account(mut)]
    pub executor: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RefundGreenLabelEscrowV1<'info> {
    #[account(
        seeds = [
            GREEN_LABEL_PROJECT_SEED,
            &green_label_project.project_id.to_le_bytes()
        ],
        bump = green_label_project.bump
    )]
    pub green_label_project: Box<Account<'info, GreenLabelProjectV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_REFUNDABLE_ESCROW_SEED,
            green_label_project.key().as_ref()
        ],
        bump = green_label_refundable_escrow.bump,
        constraint = green_label_refundable_escrow.project == green_label_project.key() @ CustomError::InvalidGreenLabelTargetAccount
    )]
    pub green_label_refundable_escrow: Box<Account<'info, GreenLabelRefundableEscrowV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_REFUNDABLE_VAULT_SEED,
            green_label_refundable_escrow.key().as_ref()
        ],
        bump = green_label_refundable_escrow.vault_bump,
        constraint = refundable_vault.mint == green_label_refundable_escrow.usdc_mint @ CustomError::InvalidGreenLabelMint,
        constraint = refundable_vault.owner == green_label_refundable_escrow.key() @ CustomError::InvalidGreenLabelTokenAccount
    )]
    pub refundable_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = payer_refund_usdc_account.owner == green_label_refundable_escrow.payer @ CustomError::InvalidGreenLabelEscrowRefund,
        constraint = payer_refund_usdc_account.mint == green_label_refundable_escrow.usdc_mint @ CustomError::InvalidGreenLabelMint
    )]
    pub payer_refund_usdc_account: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = usdc_mint.key() == green_label_refundable_escrow.usdc_mint @ CustomError::InvalidGreenLabelMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    pub caller: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ForfeitGreenLabelEscrowToTreasuryV1<'info> {
    #[account(
        seeds = [GREEN_LABEL_CONFIG_SEED],
        bump = green_label_config.bump
    )]
    pub green_label_config: Box<Account<'info, GreenLabelConfigV1>>,

    #[account(
        seeds = [
            GREEN_LABEL_PROJECT_SEED,
            &green_label_project.project_id.to_le_bytes()
        ],
        bump = green_label_project.bump
    )]
    pub green_label_project: Box<Account<'info, GreenLabelProjectV1>>,

    #[account(
        seeds = [
            GREEN_LABEL_DISPUTE_SEED,
            green_label_project.key().as_ref(),
            &green_label_dispute.dispute_id.to_le_bytes()
        ],
        bump = green_label_dispute.bump
    )]
    pub green_label_dispute: Box<Account<'info, GreenLabelDisputeV1>>,

    pub proposal_decision: Box<Account<'info, ProposalDecisionV1>>,

    pub execution_queue_item: Box<Account<'info, ExecutionQueueItemV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_REFUNDABLE_ESCROW_SEED,
            green_label_project.key().as_ref()
        ],
        bump = green_label_refundable_escrow.bump,
        constraint = green_label_refundable_escrow.project == green_label_project.key() @ CustomError::InvalidGreenLabelTargetAccount,
        constraint = green_label_refundable_escrow.usdc_mint == green_label_config.usdc_mint @ CustomError::InvalidGreenLabelMint
    )]
    pub green_label_refundable_escrow: Box<Account<'info, GreenLabelRefundableEscrowV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_REFUNDABLE_VAULT_SEED,
            green_label_refundable_escrow.key().as_ref()
        ],
        bump = green_label_refundable_escrow.vault_bump,
        constraint = refundable_vault.mint == green_label_refundable_escrow.usdc_mint @ CustomError::InvalidGreenLabelMint,
        constraint = refundable_vault.owner == green_label_refundable_escrow.key() @ CustomError::InvalidGreenLabelTokenAccount
    )]
    pub refundable_vault: Box<Account<'info, TokenAccount>>,

    /// CHECK: Loaded and validated in handler to keep the Anchor accounts stack small.
    pub treasury_config: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Loaded, validated, and serialized in handler to keep the Anchor accounts stack small.
    pub treasury_usdc_state: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Loaded, validated, and serialized in handler to keep the Anchor accounts stack small.
    pub revenue_routing_stats: UncheckedAccount<'info>,

    #[account(
        constraint = usdc_mint.key() == green_label_config.usdc_mint @ CustomError::InvalidGreenLabelMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    /// CHECK: This PDA only owns the USDC vault token accounts.
    pub vault_authority: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Loaded and validated in handler to keep the Anchor accounts stack small.
    pub relief_usdc_vault: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Loaded and validated in handler to keep the Anchor accounts stack small.
    pub buyback_usdc_vault: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Loaded and validated in handler to keep the Anchor accounts stack small.
    pub builders_usdc_vault: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Loaded and validated in handler to keep the Anchor accounts stack small.
    pub staking_usdc_vault: UncheckedAccount<'info>,

    pub executor: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn initialize_green_label_config_handler(
    ctx: Context<InitializeGreenLabelConfig>,
) -> Result<()> {
    let values = build_default_green_label_config_values(
        ctx.accounts.authority.key(),
        ctx.accounts.usdc_mint.key(),
        ctx.accounts.treasury_usdc_state_v2.key(),
        ctx.accounts.base_bond_treasury_vault.key(),
        ctx.accounts.relief_or_risk_vault.key(),
        ctx.accounts.vault_authority_v2.key(),
        ctx.accounts.security_governance_config.key(),
        ctx.bumps.green_label_config,
    )?;

    let green_label_config = &mut ctx.accounts.green_label_config;
    green_label_config.authority = values.authority;
    green_label_config.usdc_mint = values.usdc_mint;
    green_label_config.min_base_bond_usdc = values.min_base_bond_usdc;
    green_label_config.base_refund_bps = values.base_refund_bps;
    green_label_config.base_treasury_bps = values.base_treasury_bps;
    green_label_config.observation_period_seconds = values.observation_period_seconds;
    green_label_config.dispute_window_seconds = values.dispute_window_seconds;
    green_label_config.response_window_seconds = values.response_window_seconds;
    green_label_config.project_count = values.project_count;
    green_label_config.treasury_usdc_state_v2 = values.treasury_usdc_state_v2;
    green_label_config.base_bond_treasury_vault = values.base_bond_treasury_vault;
    green_label_config.relief_or_risk_vault = values.relief_or_risk_vault;
    green_label_config.vault_authority_v2 = values.vault_authority_v2;
    green_label_config.security_governance_config = values.security_governance_config;
    green_label_config.is_paused = values.is_paused;
    green_label_config.bump = values.bump;
    green_label_config.reserved = values.reserved;

    Ok(())
}

pub fn update_green_label_windows_handler(
    ctx: Context<UpdateGreenLabelWindows>,
    observation_period_seconds: i64,
    dispute_window_seconds: i64,
    response_window_seconds: i64,
) -> Result<()> {
    validate_green_label_window_update(
        ctx.accounts.green_label_config.is_paused,
        ctx.accounts.green_label_config.authority,
        ctx.accounts.authority.key(),
        observation_period_seconds,
        dispute_window_seconds,
        response_window_seconds,
    )?;

    record_green_label_window_update(
        &mut ctx.accounts.green_label_config,
        observation_period_seconds,
        dispute_window_seconds,
        response_window_seconds,
    );

    Ok(())
}

pub fn update_green_label_min_base_bond_handler(
    ctx: Context<UpdateGreenLabelMinBaseBond>,
    min_base_bond_usdc: u64,
) -> Result<()> {
    validate_green_label_min_base_bond_update(
        ctx.accounts.green_label_config.is_paused,
        ctx.accounts.green_label_config.authority,
        ctx.accounts.authority.key(),
        min_base_bond_usdc,
    )?;

    record_green_label_min_base_bond_update(
        &mut ctx.accounts.green_label_config,
        min_base_bond_usdc,
    );

    Ok(())
}

pub fn submit_green_label_application_handler(
    ctx: Context<SubmitGreenLabelApplication>,
    expected_project_id: u64,
    project_name_hash: [u8; 32],
    project_url_hash: [u8; 32],
    project_treasury_wallet: Pubkey,
    total_bond_amount: u64,
) -> Result<()> {
    let clock = Clock::get()?;
    let values = build_pending_bond_project_values(
        ctx.accounts.green_label_config.is_paused,
        ctx.accounts.green_label_config.min_base_bond_usdc,
        ctx.accounts.green_label_config.project_count,
        expected_project_id,
        ctx.accounts.project_owner.key(),
        project_name_hash,
        project_url_hash,
        ctx.accounts.token_mint.key(),
        project_treasury_wallet,
        total_bond_amount,
        clock.unix_timestamp,
        ctx.bumps.green_label_project,
    )?;

    let green_label_project = &mut ctx.accounts.green_label_project;
    green_label_project.project_id = values.project_id;
    green_label_project.project_owner = values.project_owner;
    green_label_project.project_name_hash = values.project_name_hash;
    green_label_project.project_url_hash = values.project_url_hash;
    green_label_project.token_mint = values.token_mint;
    green_label_project.project_treasury_wallet = values.project_treasury_wallet;
    green_label_project.base_bond_amount = values.base_bond_amount;
    green_label_project.extra_bond_amount = values.extra_bond_amount;
    green_label_project.total_bond_amount = values.total_bond_amount;
    green_label_project.bond_vault = values.bond_vault;
    green_label_project.bond_vault_authority = values.bond_vault_authority;
    green_label_project.bond_tier = values.bond_tier;
    green_label_project.status = values.status;
    green_label_project.submitted_at = values.submitted_at;
    green_label_project.observation_start_ts = values.observation_start_ts;
    green_label_project.observation_end_ts = values.observation_end_ts;
    green_label_project.dispute_count = values.dispute_count;
    green_label_project.active_dispute = values.active_dispute;
    green_label_project.approved_at = values.approved_at;
    green_label_project.refunded_at = values.refunded_at;
    green_label_project.slashed_at = values.slashed_at;
    green_label_project.risk_score_snapshot = values.risk_score_snapshot;
    green_label_project.terminal_proposal_id = values.terminal_proposal_id;
    green_label_project.terminal_proposal_decision = values.terminal_proposal_decision;
    green_label_project.terminal_execution_queue_item = values.terminal_execution_queue_item;
    green_label_project.terminal_payload_hash = values.terminal_payload_hash;
    green_label_project.terminal_action_type = values.terminal_action_type;
    green_label_project.bump = values.bump;
    green_label_project.reserved = values.reserved;

    ctx.accounts.green_label_config.project_count = values.project_id;

    Ok(())
}

pub fn initialize_green_bond_vault_handler(ctx: Context<InitializeGreenBondVault>) -> Result<()> {
    validate_green_bond_vault_initialization(
        ctx.accounts.green_label_config.is_paused,
        ctx.accounts.green_label_project.project_owner,
        ctx.accounts.project_owner.key(),
        ctx.accounts.green_label_project.status,
        ctx.accounts.green_label_project.bond_vault,
        ctx.accounts.green_label_project.bond_vault_authority,
        ctx.accounts.green_label_config.usdc_mint,
        ctx.accounts.usdc_mint.key(),
    )?;

    record_green_bond_vault_initialization(
        &mut ctx.accounts.green_label_project,
        ctx.accounts.green_bond_vault.key(),
        ctx.accounts.green_bond_vault_authority.key(),
    );

    Ok(())
}

pub fn lock_green_label_bond_handler(ctx: Context<LockGreenLabelBond>) -> Result<()> {
    validate_green_label_bond_lock(
        ctx.accounts.green_label_config.is_paused,
        ctx.accounts.green_label_project.project_owner,
        ctx.accounts.project_owner.key(),
        ctx.accounts.green_label_project.status,
        ctx.accounts.green_label_project.bond_vault,
        ctx.accounts.green_label_project.bond_vault_authority,
        ctx.accounts.green_bond_vault.key(),
        ctx.accounts.green_bond_vault.mint,
        ctx.accounts.green_bond_vault.owner,
        ctx.accounts.green_label_config.usdc_mint,
        ctx.accounts.project_owner_usdc_ata.owner,
        ctx.accounts.project_owner_usdc_ata.mint,
        ctx.accounts.usdc_mint.key(),
        ctx.accounts.green_label_project.base_bond_amount,
        ctx.accounts.green_label_project.extra_bond_amount,
        ctx.accounts.green_label_project.total_bond_amount,
    )?;
    require!(
        ctx.accounts.usdc_mint.decimals == GREEN_LABEL_USDC_DECIMALS,
        CustomError::InvalidGreenLabelMint
    );

    let total_bond_amount = ctx.accounts.green_label_project.total_bond_amount;
    let cpi_accounts = TransferChecked {
        from: ctx.accounts.project_owner_usdc_ata.to_account_info(),
        mint: ctx.accounts.usdc_mint.to_account_info(),
        to: ctx.accounts.green_bond_vault.to_account_info(),
        authority: ctx.accounts.project_owner.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.key(), cpi_accounts);
    transfer_checked(cpi_ctx, total_bond_amount, GREEN_LABEL_USDC_DECIMALS)?;

    let now = Clock::get()?.unix_timestamp;
    let (_, observation_end_ts) = build_observation_window(
        now,
        ctx.accounts.green_label_config.observation_period_seconds,
    )?;

    record_green_label_bond_locked(
        &mut ctx.accounts.green_label_project,
        now,
        observation_end_ts,
    )
}

pub fn open_green_label_dispute_handler(
    ctx: Context<OpenGreenLabelDispute>,
    expected_dispute_id: u64,
    reason_code: RugReasonCode,
    evidence_hash: [u8; 32],
) -> Result<()> {
    validate_open_green_label_dispute(
        ctx.accounts.green_label_config.is_paused,
        ctx.accounts.green_label_project.status,
        ctx.accounts.green_label_project.active_dispute,
        ctx.accounts.green_label_project.dispute_count,
        expected_dispute_id,
        evidence_hash,
    )?;

    let now = Clock::get()?.unix_timestamp;
    let (evidence_end_ts, response_end_ts) = build_dispute_windows(
        now,
        ctx.accounts.green_label_config.dispute_window_seconds,
        ctx.accounts.green_label_config.response_window_seconds,
    )?;

    let dispute_key = ctx.accounts.green_label_dispute.key();
    let green_label_dispute = &mut ctx.accounts.green_label_dispute;
    green_label_dispute.project_id = ctx.accounts.green_label_project.project_id;
    green_label_dispute.dispute_id = expected_dispute_id;
    green_label_dispute.project = ctx.accounts.green_label_project.key();
    green_label_dispute.disputer = ctx.accounts.disputer.key();
    green_label_dispute.reason_code = reason_code;
    green_label_dispute.evidence_hash = evidence_hash;
    green_label_dispute.status = DisputeStatus::EvidencePeriod;
    green_label_dispute.opened_at = now;
    green_label_dispute.evidence_end_ts = evidence_end_ts;
    green_label_dispute.response_end_ts = response_end_ts;
    green_label_dispute.resolved_at = 0;
    green_label_dispute.proposal_id = 0;
    green_label_dispute.proposal_decision = Pubkey::default();
    green_label_dispute.execution_queue_item = Pubkey::default();
    green_label_dispute.payload_hash = [0; 32];
    green_label_dispute.action_type = ActionType::Noop;
    green_label_dispute.bump = ctx.bumps.green_label_dispute;
    green_label_dispute.reserved = [0; GREEN_LABEL_DISPUTE_RESERVED_BYTES];

    record_green_label_dispute_opened(
        &mut ctx.accounts.green_label_project,
        dispute_key,
        expected_dispute_id,
    )
}

pub fn mark_dispute_ready_for_decision_handler(
    ctx: Context<MarkDisputeReadyForDecision>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    validate_mark_dispute_ready(
        ctx.accounts.green_label_config.is_paused,
        ctx.accounts.green_label_project.status,
        ctx.accounts.green_label_project.active_dispute,
        ctx.accounts.green_label_dispute.key(),
        ctx.accounts.green_label_dispute.project,
        ctx.accounts.green_label_project.key(),
        ctx.accounts.green_label_dispute.status,
        now,
        ctx.accounts.green_label_dispute.response_end_ts,
    )?;

    record_dispute_ready_for_decision(&mut ctx.accounts.green_label_dispute)
}

pub fn link_green_label_security_decision_handler(
    ctx: Context<LinkGreenLabelSecurityDecision>,
    expected_proposal_id: u64,
    expected_action_type: ActionType,
    expected_payload_hash: [u8; 32],
) -> Result<()> {
    let dispute_key = ctx.accounts.green_label_dispute.key();
    validate_green_label_security_decision_link(
        ctx.accounts.green_label_config.is_paused,
        ctx.accounts.green_label_project.status,
        ctx.accounts.green_label_project.active_dispute,
        dispute_key,
        ctx.accounts.green_label_dispute.project,
        ctx.accounts.green_label_project.key(),
        ctx.accounts.green_label_dispute.status,
        expected_proposal_id,
        expected_action_type,
        expected_payload_hash,
        ctx.accounts.proposal_decision.proposal_id,
        ctx.accounts.proposal_decision.proposal_type,
        ctx.accounts.proposal_decision.decision,
        ctx.accounts.execution_queue_item.proposal_id,
        ctx.accounts.execution_queue_item.action_type,
        ctx.accounts.execution_queue_item.status,
        ctx.accounts.execution_queue_item.payload_hash,
        ctx.accounts.execution_queue_item.target_program,
        crate::ID,
        ctx.accounts.execution_queue_item.target_account,
        dispute_key,
    )?;

    record_green_label_security_decision_link(
        &mut ctx.accounts.green_label_project,
        &mut ctx.accounts.green_label_dispute,
        expected_proposal_id,
        ctx.accounts.proposal_decision.key(),
        ctx.accounts.execution_queue_item.key(),
        expected_payload_hash,
        expected_action_type,
    )
}

pub fn execute_green_label_refund_handler(ctx: Context<ExecuteGreenLabelRefund>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let project_key = ctx.accounts.green_label_project.key();
    let dispute_key = ctx.accounts.green_label_dispute.key();
    let refund_amounts = calculate_green_label_refund(
        ctx.accounts.green_label_project.base_bond_amount,
        ctx.accounts.green_label_project.extra_bond_amount,
    )?;

    validate_green_label_refund_execution(
        ctx.accounts.green_label_config.is_paused,
        ctx.accounts.green_label_project.status,
        ctx.accounts.green_label_project.active_dispute,
        dispute_key,
        ctx.accounts.green_label_project.bond_vault,
        ctx.accounts.green_label_project.bond_vault_authority,
        ctx.accounts.green_label_project.project_owner,
        ctx.accounts.green_label_project.terminal_proposal_id,
        ctx.accounts.green_label_project.terminal_proposal_decision,
        ctx.accounts
            .green_label_project
            .terminal_execution_queue_item,
        ctx.accounts.green_label_project.terminal_payload_hash,
        ctx.accounts.green_label_project.terminal_action_type,
        ctx.accounts.green_label_dispute.project,
        project_key,
        ctx.accounts.green_label_dispute.status,
        ctx.accounts.green_label_dispute.proposal_id,
        ctx.accounts.green_label_dispute.proposal_decision,
        ctx.accounts.green_label_dispute.execution_queue_item,
        ctx.accounts.green_label_dispute.payload_hash,
        ctx.accounts.green_label_dispute.action_type,
        ctx.accounts.proposal_decision.key(),
        ctx.accounts.proposal_decision.proposal_id,
        ctx.accounts.proposal_decision.decision,
        ctx.accounts.execution_queue_item.key(),
        ctx.accounts.execution_queue_item.proposal_id,
        ctx.accounts.execution_queue_item.status,
        ctx.accounts.execution_queue_item.action_type,
        ctx.accounts.execution_queue_item.payload_hash,
        ctx.accounts.execution_queue_item.target_program,
        crate::ID,
        ctx.accounts.execution_queue_item.target_account,
        dispute_key,
        now,
        ctx.accounts.execution_queue_item.execute_after,
        ctx.accounts.green_bond_vault.key(),
        ctx.accounts.green_bond_vault.mint,
        ctx.accounts.green_bond_vault.owner,
        ctx.accounts.green_bond_vault_authority.key(),
        ctx.accounts.project_owner_usdc_ata.owner,
        ctx.accounts.project_owner_usdc_ata.mint,
        ctx.accounts.base_bond_treasury_vault.key(),
        ctx.accounts.base_bond_treasury_vault.mint,
        ctx.accounts.green_label_config.base_bond_treasury_vault,
        ctx.accounts.green_label_config.usdc_mint,
        ctx.accounts.usdc_mint.key(),
        ctx.accounts.usdc_mint.decimals,
        ctx.accounts.green_bond_vault.amount,
        refund_amounts.project_refund_amount,
        refund_amounts.treasury_amount,
    )?;

    let green_bond_vault_authority_bump = ctx.bumps.green_bond_vault_authority;
    let signer_seeds: &[&[&[u8]]] = &[&[
        GREEN_BOND_VAULT_AUTHORITY_SEED,
        project_key.as_ref(),
        &[green_bond_vault_authority_bump],
    ]];

    let refund_to_project_accounts = TransferChecked {
        from: ctx.accounts.green_bond_vault.to_account_info(),
        mint: ctx.accounts.usdc_mint.to_account_info(),
        to: ctx.accounts.project_owner_usdc_ata.to_account_info(),
        authority: ctx.accounts.green_bond_vault_authority.to_account_info(),
    };
    let refund_to_project_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        refund_to_project_accounts,
        signer_seeds,
    );
    transfer_checked(
        refund_to_project_ctx,
        refund_amounts.project_refund_amount,
        GREEN_LABEL_USDC_DECIMALS,
    )?;

    let treasury_accounts = TransferChecked {
        from: ctx.accounts.green_bond_vault.to_account_info(),
        mint: ctx.accounts.usdc_mint.to_account_info(),
        to: ctx.accounts.base_bond_treasury_vault.to_account_info(),
        authority: ctx.accounts.green_bond_vault_authority.to_account_info(),
    };
    let treasury_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        treasury_accounts,
        signer_seeds,
    );
    transfer_checked(
        treasury_ctx,
        refund_amounts.treasury_amount,
        GREEN_LABEL_USDC_DECIMALS,
    )?;

    record_green_label_refunded(
        &mut ctx.accounts.green_label_project,
        Some(ctx.accounts.green_label_dispute.as_mut()),
        now,
    )
}

pub fn execute_green_label_slash_handler(_ctx: Context<ExecuteGreenLabelSlash>) -> Result<()> {
    reject_legacy_green_label_slash_v1()
}

pub fn initialize_green_label_refundable_escrow_v1_handler(
    ctx: Context<InitializeGreenLabelRefundableEscrowV1>,
    refund_available_after: i64,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    validate_green_label_refundable_escrow_initialization(
        ctx.accounts.green_label_config.is_paused,
        ctx.accounts.green_label_project.project_owner,
        ctx.accounts.payer.key(),
        ctx.accounts.green_label_config.usdc_mint,
        ctx.accounts.usdc_mint.key(),
        now,
        refund_available_after,
    )?;

    let escrow = &mut ctx.accounts.green_label_refundable_escrow;
    escrow.authority = ctx.accounts.green_label_config.authority;
    escrow.project = ctx.accounts.green_label_project.key();
    escrow.project_id = ctx.accounts.green_label_project.project_id;
    escrow.payer = ctx.accounts.payer.key();
    escrow.usdc_mint = ctx.accounts.usdc_mint.key();
    escrow.refundable_vault = ctx.accounts.refundable_vault.key();
    escrow.deposited_amount = 0;
    escrow.refundable_amount = 0;
    escrow.refunded_amount = 0;
    escrow.forfeited_amount = 0;
    escrow.deposit_ts = 0;
    escrow.refund_available_after = refund_available_after;
    escrow.status = GreenLabelEscrowStatusV1::Locked;
    escrow.bump = ctx.bumps.green_label_refundable_escrow;
    escrow.vault_bump = ctx.bumps.refundable_vault;

    Ok(())
}

pub fn deposit_green_label_refundable_bond_v1_handler(
    ctx: Context<DepositGreenLabelRefundableBondV1>,
    amount: u64,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    validate_green_label_refundable_bond_deposit(
        ctx.accounts.green_label_refundable_escrow.status,
        ctx.accounts.green_label_refundable_escrow.payer,
        ctx.accounts.payer.key(),
        ctx.accounts.green_label_refundable_escrow.usdc_mint,
        ctx.accounts.payer_usdc_account.mint,
        ctx.accounts.refundable_vault.mint,
        ctx.accounts.refundable_vault.owner,
        ctx.accounts.green_label_refundable_escrow.key(),
        ctx.accounts.usdc_mint.key(),
        ctx.accounts.usdc_mint.decimals,
        amount,
    )?;

    let cpi_accounts = TransferChecked {
        from: ctx.accounts.payer_usdc_account.to_account_info(),
        mint: ctx.accounts.usdc_mint.to_account_info(),
        to: ctx.accounts.refundable_vault.to_account_info(),
        authority: ctx.accounts.payer.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.key(), cpi_accounts);
    transfer_checked(cpi_ctx, amount, GREEN_LABEL_USDC_DECIMALS)?;

    record_green_label_refundable_bond_deposit(
        &mut ctx.accounts.green_label_refundable_escrow,
        amount,
        now,
    )
}

pub fn initialize_green_label_certification_fee_policy_v1_handler(
    ctx: Context<InitializeGreenLabelCertificationFeePolicyV1>,
    fee_amount_usdc: u64,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    validate_green_label_certification_fee_policy_init_v1(
        ctx.accounts.green_label_config.authority,
        ctx.accounts.authority.key(),
        ctx.accounts.green_label_config.usdc_mint,
        fee_amount_usdc,
    )?;

    record_green_label_certification_fee_policy_init_v1(
        &mut ctx.accounts.green_label_certification_fee_policy,
        ctx.accounts.green_label_config.key(),
        ctx.accounts.green_label_config.usdc_mint,
        fee_amount_usdc,
        ctx.accounts.authority.key(),
        now,
        ctx.bumps.green_label_certification_fee_policy,
    )
}

pub fn route_green_label_certification_fee_v1_handler(
    _ctx: Context<RouteGreenLabelCertificationFeeV1>,
    _amount: u64,
) -> Result<()> {
    reject_legacy_green_label_certification_fee_route_v1()
}

pub fn route_green_label_certification_fee_once_v1_handler(
    ctx: Context<RouteGreenLabelCertificationFeeOnceV1>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let config_key = ctx.accounts.green_label_config.key();
    let project_key = ctx.accounts.green_label_project.key();
    let fee_policy_key = ctx.accounts.green_label_certification_fee_policy.key();
    let receipt_key = ctx.accounts.green_label_certification_fee_receipt.key();

    let parameters = build_green_label_certification_fee_parameters_v1(
        config_key,
        fee_policy_key,
        &ctx.accounts.green_label_certification_fee_policy,
        project_key,
        &ctx.accounts.green_label_project,
        ctx.accounts.payer.key(),
        ctx.accounts.payer_usdc_account.key(),
        ctx.accounts.treasury_config.key(),
        ctx.accounts.treasury_usdc_state.key(),
        ctx.accounts.revenue_routing_stats.key(),
        ctx.accounts.relief_usdc_vault.key(),
        ctx.accounts.buyback_usdc_vault.key(),
        ctx.accounts.builders_usdc_vault.key(),
        ctx.accounts.staking_usdc_vault.key(),
    )?;

    let parameters_hash = validate_green_label_certification_fee_once_v1(
        &ctx.accounts.green_label_config,
        config_key,
        &ctx.accounts.green_label_project,
        project_key,
        &ctx.accounts.green_label_certification_fee_policy,
        fee_policy_key,
        &ctx.accounts.payer_usdc_account,
        ctx.accounts.payer_usdc_account.key(),
        ctx.accounts.payer.key(),
        &ctx.accounts.treasury_config,
        ctx.accounts.treasury_config.key(),
        &ctx.accounts.treasury_usdc_state,
        ctx.accounts.treasury_usdc_state.key(),
        &ctx.accounts.revenue_routing_stats,
        ctx.accounts.revenue_routing_stats.key(),
        ctx.accounts.vault_authority.key(),
        ctx.accounts.relief_usdc_vault.key(),
        &ctx.accounts.relief_usdc_vault,
        ctx.accounts.buyback_usdc_vault.key(),
        &ctx.accounts.buyback_usdc_vault,
        ctx.accounts.builders_usdc_vault.key(),
        &ctx.accounts.builders_usdc_vault,
        ctx.accounts.staking_usdc_vault.key(),
        &ctx.accounts.staking_usdc_vault,
        ctx.accounts.usdc_mint.key(),
        ctx.accounts.usdc_mint.decimals,
        &parameters,
    )?;

    route_usdc_revenue_from_token_account(
        ctx.accounts.token_program.key(),
        ctx.accounts.payer_usdc_account.to_account_info(),
        ctx.accounts.payer.to_account_info(),
        None,
        ctx.accounts.usdc_mint.to_account_info(),
        ctx.accounts.relief_usdc_vault.to_account_info(),
        ctx.accounts.buyback_usdc_vault.to_account_info(),
        ctx.accounts.builders_usdc_vault.to_account_info(),
        ctx.accounts.staking_usdc_vault.to_account_info(),
        &mut ctx.accounts.treasury_usdc_state,
        &mut ctx.accounts.revenue_routing_stats,
        ctx.accounts.usdc_mint.key(),
        RevenueType::GreenLabelCertificationFee,
        ctx.accounts
            .green_label_certification_fee_policy
            .fee_amount_usdc,
        GREEN_LABEL_USDC_DECIMALS,
    )?;

    record_green_label_certification_fee_receipt_v1(
        &mut ctx.accounts.green_label_certification_fee_receipt,
        &parameters,
        parameters_hash,
        now,
        ctx.bumps.green_label_certification_fee_receipt,
        receipt_key,
    )
}

pub fn initialize_green_label_certification_state_v1_handler(
    ctx: Context<InitializeGreenLabelCertificationStateV1>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    record_green_label_certification_state_init(
        &mut ctx.accounts.green_label_certification_state,
        ctx.accounts.green_label_project.key(),
        ctx.accounts.green_label_config.key(),
        ctx.accounts.green_label_project.status,
        now,
        ctx.bumps.green_label_certification_state,
    )
}

pub fn execute_green_label_approve_certification_v1_handler(
    ctx: Context<ExecuteGreenLabelApproveCertificationV1>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let project_key = ctx.accounts.green_label_project.key();
    let config_key = ctx.accounts.green_label_config.key();
    let state_key = ctx.accounts.green_label_certification_state.key();
    let record_key = ctx
        .accounts
        .green_label_certification_execution_record
        .key();
    let parameters = build_green_label_certification_decision_parameters_v1(
        &ctx.accounts.green_label_config,
        config_key,
        &ctx.accounts.green_label_project,
        project_key,
        state_key,
        GovernanceActionTypeV1::GreenLabelApproveCertification,
        ctx.accounts
            .green_label_certification_state
            .certification_status,
        ctx.accounts.governance_proposal.proposal_id,
    )?;

    validate_green_label_certification_execution_context_v1(
        &ctx.accounts.security_governance_config,
        ctx.accounts.security_governance_config.key(),
        &ctx.accounts.green_label_config,
        &ctx.accounts.green_label_project,
        project_key,
        &ctx.accounts.green_label_certification_state,
        state_key,
        &ctx.accounts.protocol_module_registry,
        ctx.accounts.protocol_module_registry.key(),
        &ctx.accounts.governance_proposal,
        ctx.accounts.governance_proposal.key(),
        &ctx.accounts.governance_proposal_action,
        ctx.accounts.governance_proposal_action.key(),
        &ctx.accounts.governance_decision_adapter,
        ctx.accounts.governance_decision_adapter.key(),
        &ctx.accounts.proposal_decision,
        ctx.accounts.proposal_decision.key(),
        &ctx.accounts.execution_queue_item,
        ctx.accounts.execution_queue_item.key(),
        GovernanceActionTypeV1::GreenLabelApproveCertification,
        ProposalType::GreenLabelApproveCertification,
        ActionType::GreenLabelApproveCertification,
        &parameters,
    )?;
    validate_green_label_approve_certification_business_v1(
        &ctx.accounts.green_label_config,
        &ctx.accounts.green_label_project,
        &ctx.accounts.green_label_certification_state,
        ctx.accounts.green_bond_vault.key(),
        &ctx.accounts.green_bond_vault,
        ctx.accounts.usdc_mint.key(),
        ctx.accounts.usdc_mint.decimals,
        now,
    )?;

    let project_status_before = ctx.accounts.green_label_project.status;
    let certification_status_before = ctx
        .accounts
        .green_label_certification_state
        .certification_status;
    record_green_label_approve_certification_v1(
        &mut ctx.accounts.green_label_project,
        &mut ctx.accounts.green_label_certification_state,
        &mut ctx.accounts.green_label_certification_execution_record,
        ctx.accounts.execution_queue_item.key(),
        ctx.accounts.proposal_decision.key(),
        ctx.accounts.governance_proposal.key(),
        ctx.accounts.governance_proposal_action.key(),
        project_key,
        state_key,
        ctx.accounts.protocol_module_registry.key(),
        record_key,
        parameters,
        ctx.accounts
            .governance_proposal_action
            .canonical_payload_hash,
        project_status_before,
        certification_status_before,
        ctx.accounts.executor.key(),
        now,
        ctx.bumps.green_label_certification_execution_record,
    )
}

pub fn execute_green_label_reject_certification_v1_handler(
    ctx: Context<ExecuteGreenLabelRejectCertificationV1>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let project_key = ctx.accounts.green_label_project.key();
    let config_key = ctx.accounts.green_label_config.key();
    let state_key = ctx.accounts.green_label_certification_state.key();
    let record_key = ctx
        .accounts
        .green_label_certification_execution_record
        .key();
    let parameters = build_green_label_certification_decision_parameters_v1(
        &ctx.accounts.green_label_config,
        config_key,
        &ctx.accounts.green_label_project,
        project_key,
        state_key,
        GovernanceActionTypeV1::GreenLabelRejectCertification,
        ctx.accounts
            .green_label_certification_state
            .certification_status,
        ctx.accounts.governance_proposal.proposal_id,
    )?;

    validate_green_label_certification_execution_context_v1(
        &ctx.accounts.security_governance_config,
        ctx.accounts.security_governance_config.key(),
        &ctx.accounts.green_label_config,
        &ctx.accounts.green_label_project,
        project_key,
        &ctx.accounts.green_label_certification_state,
        state_key,
        &ctx.accounts.protocol_module_registry,
        ctx.accounts.protocol_module_registry.key(),
        &ctx.accounts.governance_proposal,
        ctx.accounts.governance_proposal.key(),
        &ctx.accounts.governance_proposal_action,
        ctx.accounts.governance_proposal_action.key(),
        &ctx.accounts.governance_decision_adapter,
        ctx.accounts.governance_decision_adapter.key(),
        &ctx.accounts.proposal_decision,
        ctx.accounts.proposal_decision.key(),
        &ctx.accounts.execution_queue_item,
        ctx.accounts.execution_queue_item.key(),
        GovernanceActionTypeV1::GreenLabelRejectCertification,
        ProposalType::GreenLabelRejectCertification,
        ActionType::GreenLabelRejectCertification,
        &parameters,
    )?;
    validate_green_label_reject_certification_business_v1(
        ctx.accounts.green_label_project.status,
        ctx.accounts
            .green_label_certification_state
            .certification_status,
    )?;

    let project_status_before = ctx.accounts.green_label_project.status;
    let certification_status_before = ctx
        .accounts
        .green_label_certification_state
        .certification_status;
    record_green_label_certification_decision_v1(
        &mut ctx.accounts.green_label_certification_state,
        &mut ctx.accounts.green_label_certification_execution_record,
        ctx.accounts.execution_queue_item.key(),
        ctx.accounts.proposal_decision.key(),
        ctx.accounts.governance_proposal.key(),
        ctx.accounts.governance_proposal_action.key(),
        project_key,
        state_key,
        ctx.accounts.protocol_module_registry.key(),
        record_key,
        GreenLabelCertificationExecutionTypeV1::Reject,
        GreenLabelCertificationStatusV1::Rejected,
        GovernanceActionTypeV1::GreenLabelRejectCertification,
        parameters,
        ctx.accounts
            .governance_proposal_action
            .canonical_payload_hash,
        project_status_before,
        project_status_before,
        certification_status_before,
        ctx.accounts.executor.key(),
        now,
        ctx.bumps.green_label_certification_execution_record,
    )
}

pub fn execute_green_label_revoke_certification_v1_handler(
    ctx: Context<ExecuteGreenLabelRevokeCertificationV1>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let project_key = ctx.accounts.green_label_project.key();
    let config_key = ctx.accounts.green_label_config.key();
    let state_key = ctx.accounts.green_label_certification_state.key();
    let record_key = ctx
        .accounts
        .green_label_certification_execution_record
        .key();
    let parameters = build_green_label_certification_decision_parameters_v1(
        &ctx.accounts.green_label_config,
        config_key,
        &ctx.accounts.green_label_project,
        project_key,
        state_key,
        GovernanceActionTypeV1::GreenLabelRevokeCertification,
        ctx.accounts
            .green_label_certification_state
            .certification_status,
        ctx.accounts.governance_proposal.proposal_id,
    )?;

    validate_green_label_certification_execution_context_v1(
        &ctx.accounts.security_governance_config,
        ctx.accounts.security_governance_config.key(),
        &ctx.accounts.green_label_config,
        &ctx.accounts.green_label_project,
        project_key,
        &ctx.accounts.green_label_certification_state,
        state_key,
        &ctx.accounts.protocol_module_registry,
        ctx.accounts.protocol_module_registry.key(),
        &ctx.accounts.governance_proposal,
        ctx.accounts.governance_proposal.key(),
        &ctx.accounts.governance_proposal_action,
        ctx.accounts.governance_proposal_action.key(),
        &ctx.accounts.governance_decision_adapter,
        ctx.accounts.governance_decision_adapter.key(),
        &ctx.accounts.proposal_decision,
        ctx.accounts.proposal_decision.key(),
        &ctx.accounts.execution_queue_item,
        ctx.accounts.execution_queue_item.key(),
        GovernanceActionTypeV1::GreenLabelRevokeCertification,
        ProposalType::GreenLabelRevokeCertification,
        ActionType::GreenLabelRevokeCertification,
        &parameters,
    )?;
    validate_green_label_revoke_certification_business_v1(
        ctx.accounts.green_label_project.status,
        ctx.accounts
            .green_label_certification_state
            .certification_status,
    )?;

    let project_status_before = ctx.accounts.green_label_project.status;
    let certification_status_before = ctx
        .accounts
        .green_label_certification_state
        .certification_status;
    record_green_label_certification_decision_v1(
        &mut ctx.accounts.green_label_certification_state,
        &mut ctx.accounts.green_label_certification_execution_record,
        ctx.accounts.execution_queue_item.key(),
        ctx.accounts.proposal_decision.key(),
        ctx.accounts.governance_proposal.key(),
        ctx.accounts.governance_proposal_action.key(),
        project_key,
        state_key,
        ctx.accounts.protocol_module_registry.key(),
        record_key,
        GreenLabelCertificationExecutionTypeV1::Revoke,
        GreenLabelCertificationStatusV1::Revoked,
        GovernanceActionTypeV1::GreenLabelRevokeCertification,
        parameters,
        ctx.accounts
            .governance_proposal_action
            .canonical_payload_hash,
        project_status_before,
        project_status_before,
        certification_status_before,
        ctx.accounts.executor.key(),
        now,
        ctx.bumps.green_label_certification_execution_record,
    )
}

pub fn execute_green_label_refund_no_dispute_governance_v1_handler(
    ctx: Context<ExecuteGreenLabelRefundNoDisputeGovernanceV1>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let project_key = ctx.accounts.green_label_project.key();
    let config_key = ctx.accounts.green_label_config.key();
    let escrow_key = ctx.accounts.green_label_refundable_escrow.key();
    let refundable_vault_key = ctx.accounts.refundable_vault.key();
    let destination_key = ctx.accounts.payer_refund_usdc_account.key();
    let record_key = ctx.accounts.green_label_refund_execution_record.key();
    let refund_amount =
        derive_green_label_refund_amount_v1(&ctx.accounts.green_label_refundable_escrow)?;
    let parameters = build_green_label_refund_parameters_v1(
        config_key,
        project_key,
        Pubkey::default(),
        escrow_key,
        refundable_vault_key,
        ctx.accounts.green_label_refundable_escrow.payer,
        destination_key,
        refund_amount,
        ctx.accounts.green_label_refundable_escrow.usdc_mint,
        ctx.accounts.green_label_refundable_escrow.status,
        ctx.accounts.governance_proposal.proposal_id,
    )?;

    validate_green_label_refund_execution_context_v1(
        &ctx.accounts.security_governance_config,
        ctx.accounts.security_governance_config.key(),
        &ctx.accounts.green_label_config,
        config_key,
        &ctx.accounts.green_label_project,
        project_key,
        None,
        &ctx.accounts.green_label_refundable_escrow,
        escrow_key,
        &ctx.accounts.protocol_module_registry,
        ctx.accounts.protocol_module_registry.key(),
        &ctx.accounts.governance_proposal,
        ctx.accounts.governance_proposal.key(),
        &ctx.accounts.governance_proposal_action,
        ctx.accounts.governance_proposal_action.key(),
        &ctx.accounts.governance_decision_adapter,
        ctx.accounts.governance_decision_adapter.key(),
        &ctx.accounts.proposal_decision,
        ctx.accounts.proposal_decision.key(),
        &ctx.accounts.execution_queue_item,
        ctx.accounts.execution_queue_item.key(),
        refundable_vault_key,
        ctx.accounts.refundable_vault.mint,
        ctx.accounts.refundable_vault.owner,
        destination_key,
        ctx.accounts.payer_refund_usdc_account.owner,
        ctx.accounts.payer_refund_usdc_account.mint,
        ctx.accounts.usdc_mint.key(),
        ctx.accounts.usdc_mint.decimals,
        ctx.accounts.refundable_vault.amount,
        false,
        now,
        &parameters,
    )?;

    let escrow_status_before = ctx.accounts.green_label_refundable_escrow.status;
    let project_status_before = ctx.accounts.green_label_project.status;
    let escrow_info = ctx.accounts.green_label_refundable_escrow.to_account_info();
    let refundable_vault_info = ctx.accounts.refundable_vault.to_account_info();
    let usdc_mint_info = ctx.accounts.usdc_mint.to_account_info();
    let payer_refund_info = ctx.accounts.payer_refund_usdc_account.to_account_info();
    execute_green_label_escrow_refund_internal_v1(
        &mut ctx.accounts.green_label_refundable_escrow,
        escrow_info,
        refundable_vault_info,
        usdc_mint_info,
        payer_refund_info,
        ctx.accounts.token_program.key(),
        project_key,
        refund_amount,
    )?;
    record_green_label_refund_governance_v1(
        &mut ctx.accounts.green_label_project,
        None,
        &mut ctx.accounts.green_label_refund_execution_record,
        ctx.accounts.execution_queue_item.key(),
        ctx.accounts.proposal_decision.key(),
        ctx.accounts.governance_proposal.key(),
        ctx.accounts.governance_proposal_action.key(),
        ctx.accounts.protocol_module_registry.key(),
        config_key,
        project_key,
        Pubkey::default(),
        escrow_key,
        refundable_vault_key,
        ctx.accounts.green_label_refundable_escrow.payer,
        destination_key,
        refund_amount,
        ctx.accounts.green_label_refundable_escrow.usdc_mint,
        parameters,
        ctx.accounts
            .governance_proposal_action
            .canonical_payload_hash,
        escrow_status_before,
        project_status_before,
        ctx.accounts.executor.key(),
        now,
        ctx.bumps.green_label_refund_execution_record,
        record_key,
    )
}

pub fn execute_green_label_refund_dispute_governance_v1_handler(
    ctx: Context<ExecuteGreenLabelRefundDisputeGovernanceV1>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let project_key = ctx.accounts.green_label_project.key();
    let dispute_key = ctx.accounts.green_label_dispute.key();
    let config_key = ctx.accounts.green_label_config.key();
    let escrow_key = ctx.accounts.green_label_refundable_escrow.key();
    let refundable_vault_key = ctx.accounts.refundable_vault.key();
    let destination_key = ctx.accounts.payer_refund_usdc_account.key();
    let record_key = ctx.accounts.green_label_refund_execution_record.key();
    let refund_amount =
        derive_green_label_refund_amount_v1(&ctx.accounts.green_label_refundable_escrow)?;
    let parameters = build_green_label_refund_parameters_v1(
        config_key,
        project_key,
        dispute_key,
        escrow_key,
        refundable_vault_key,
        ctx.accounts.green_label_refundable_escrow.payer,
        destination_key,
        refund_amount,
        ctx.accounts.green_label_refundable_escrow.usdc_mint,
        ctx.accounts.green_label_refundable_escrow.status,
        ctx.accounts.governance_proposal.proposal_id,
    )?;

    validate_green_label_refund_execution_context_v1(
        &ctx.accounts.security_governance_config,
        ctx.accounts.security_governance_config.key(),
        &ctx.accounts.green_label_config,
        config_key,
        &ctx.accounts.green_label_project,
        project_key,
        Some(ctx.accounts.green_label_dispute.as_ref()),
        &ctx.accounts.green_label_refundable_escrow,
        escrow_key,
        &ctx.accounts.protocol_module_registry,
        ctx.accounts.protocol_module_registry.key(),
        &ctx.accounts.governance_proposal,
        ctx.accounts.governance_proposal.key(),
        &ctx.accounts.governance_proposal_action,
        ctx.accounts.governance_proposal_action.key(),
        &ctx.accounts.governance_decision_adapter,
        ctx.accounts.governance_decision_adapter.key(),
        &ctx.accounts.proposal_decision,
        ctx.accounts.proposal_decision.key(),
        &ctx.accounts.execution_queue_item,
        ctx.accounts.execution_queue_item.key(),
        refundable_vault_key,
        ctx.accounts.refundable_vault.mint,
        ctx.accounts.refundable_vault.owner,
        destination_key,
        ctx.accounts.payer_refund_usdc_account.owner,
        ctx.accounts.payer_refund_usdc_account.mint,
        ctx.accounts.usdc_mint.key(),
        ctx.accounts.usdc_mint.decimals,
        ctx.accounts.refundable_vault.amount,
        true,
        now,
        &parameters,
    )?;

    let escrow_status_before = ctx.accounts.green_label_refundable_escrow.status;
    let project_status_before = ctx.accounts.green_label_project.status;
    let escrow_info = ctx.accounts.green_label_refundable_escrow.to_account_info();
    let refundable_vault_info = ctx.accounts.refundable_vault.to_account_info();
    let usdc_mint_info = ctx.accounts.usdc_mint.to_account_info();
    let payer_refund_info = ctx.accounts.payer_refund_usdc_account.to_account_info();
    execute_green_label_escrow_refund_internal_v1(
        &mut ctx.accounts.green_label_refundable_escrow,
        escrow_info,
        refundable_vault_info,
        usdc_mint_info,
        payer_refund_info,
        ctx.accounts.token_program.key(),
        project_key,
        refund_amount,
    )?;
    record_green_label_refund_governance_v1(
        &mut ctx.accounts.green_label_project,
        Some(&mut ctx.accounts.green_label_dispute),
        &mut ctx.accounts.green_label_refund_execution_record,
        ctx.accounts.execution_queue_item.key(),
        ctx.accounts.proposal_decision.key(),
        ctx.accounts.governance_proposal.key(),
        ctx.accounts.governance_proposal_action.key(),
        ctx.accounts.protocol_module_registry.key(),
        config_key,
        project_key,
        dispute_key,
        escrow_key,
        refundable_vault_key,
        ctx.accounts.green_label_refundable_escrow.payer,
        destination_key,
        refund_amount,
        ctx.accounts.green_label_refundable_escrow.usdc_mint,
        parameters,
        ctx.accounts
            .governance_proposal_action
            .canonical_payload_hash,
        escrow_status_before,
        project_status_before,
        ctx.accounts.executor.key(),
        now,
        ctx.bumps.green_label_refund_execution_record,
        record_key,
    )
}

pub fn execute_green_label_forfeit_governance_v1_handler<'info>(
    ctx: Context<'info, ExecuteGreenLabelForfeitGovernanceV1<'info>>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let project_key = ctx.accounts.green_label_project.key();
    let dispute_key = ctx.accounts.green_label_dispute.key();
    let config_key = ctx.accounts.green_label_config.key();
    let escrow_key = ctx.accounts.green_label_refundable_escrow.key();
    let refundable_vault_key = ctx.accounts.refundable_vault.key();
    let record_key = ctx.accounts.green_label_forfeit_execution_record.key();

    let treasury_config = Account::<TreasuryConfigV2>::try_from(&*ctx.accounts.treasury_config)?;
    let mut treasury_usdc_state =
        Account::<TreasuryUsdcStateV2>::try_from(&*ctx.accounts.treasury_usdc_state)?;
    let mut revenue_routing_stats =
        Account::<RevenueRoutingStatsV1>::try_from(&*ctx.accounts.revenue_routing_stats)?;
    let relief_usdc_vault = Account::<TokenAccount>::try_from(&*ctx.accounts.relief_usdc_vault)?;
    let buyback_usdc_vault = Account::<TokenAccount>::try_from(&*ctx.accounts.buyback_usdc_vault)?;
    let builders_usdc_vault =
        Account::<TokenAccount>::try_from(&*ctx.accounts.builders_usdc_vault)?;
    let staking_usdc_vault = Account::<TokenAccount>::try_from(&*ctx.accounts.staking_usdc_vault)?;

    let forfeit_amount =
        derive_green_label_forfeitable_amount_v1(&ctx.accounts.green_label_refundable_escrow)?;
    let parameters = build_green_label_forfeit_parameters_v1(
        config_key,
        project_key,
        dispute_key,
        escrow_key,
        refundable_vault_key,
        forfeit_amount,
        ctx.accounts.treasury_config.key(),
        ctx.accounts.treasury_usdc_state.key(),
        ctx.accounts.revenue_routing_stats.key(),
        ctx.accounts.relief_usdc_vault.key(),
        ctx.accounts.buyback_usdc_vault.key(),
        ctx.accounts.builders_usdc_vault.key(),
        ctx.accounts.staking_usdc_vault.key(),
        ctx.accounts.green_label_refundable_escrow.usdc_mint,
        ctx.accounts.green_label_refundable_escrow.status,
        ctx.accounts.green_label_project.status,
        ctx.accounts.green_label_dispute.status,
        ctx.accounts.governance_proposal.proposal_id,
    )?;

    validate_green_label_forfeit_execution_context_v1(
        &ctx.accounts.security_governance_config,
        ctx.accounts.security_governance_config.key(),
        &ctx.accounts.green_label_config,
        config_key,
        &ctx.accounts.green_label_project,
        project_key,
        &ctx.accounts.green_label_dispute,
        dispute_key,
        &ctx.accounts.green_label_refundable_escrow,
        escrow_key,
        &ctx.accounts.protocol_module_registry,
        ctx.accounts.protocol_module_registry.key(),
        &ctx.accounts.governance_proposal,
        ctx.accounts.governance_proposal.key(),
        &ctx.accounts.governance_proposal_action,
        ctx.accounts.governance_proposal_action.key(),
        &ctx.accounts.governance_decision_adapter,
        ctx.accounts.governance_decision_adapter.key(),
        &ctx.accounts.proposal_decision,
        ctx.accounts.proposal_decision.key(),
        &ctx.accounts.execution_queue_item,
        ctx.accounts.execution_queue_item.key(),
        &treasury_config,
        ctx.accounts.treasury_config.key(),
        &treasury_usdc_state,
        ctx.accounts.treasury_usdc_state.key(),
        &revenue_routing_stats,
        ctx.accounts.revenue_routing_stats.key(),
        ctx.accounts.vault_authority.key(),
        refundable_vault_key,
        ctx.accounts.refundable_vault.mint,
        ctx.accounts.refundable_vault.owner,
        ctx.accounts.refundable_vault.amount,
        ctx.accounts.relief_usdc_vault.key(),
        relief_usdc_vault.mint,
        relief_usdc_vault.owner,
        ctx.accounts.buyback_usdc_vault.key(),
        buyback_usdc_vault.mint,
        buyback_usdc_vault.owner,
        ctx.accounts.builders_usdc_vault.key(),
        builders_usdc_vault.mint,
        builders_usdc_vault.owner,
        ctx.accounts.staking_usdc_vault.key(),
        staking_usdc_vault.mint,
        staking_usdc_vault.owner,
        ctx.accounts.usdc_mint.key(),
        ctx.accounts.usdc_mint.decimals,
        now,
        &parameters,
    )?;

    let escrow_status_before = ctx.accounts.green_label_refundable_escrow.status;
    let project_status_before = ctx.accounts.green_label_project.status;
    let dispute_status_before = ctx.accounts.green_label_dispute.status;
    let escrow_usdc_mint = ctx.accounts.green_label_refundable_escrow.usdc_mint;
    let escrow_bump = ctx.accounts.green_label_refundable_escrow.bump;
    let signer_seeds: &[&[&[u8]]] = &[&[
        GREEN_LABEL_REFUNDABLE_ESCROW_SEED,
        project_key.as_ref(),
        &[escrow_bump],
    ]];

    route_usdc_revenue_from_token_account(
        ctx.accounts.token_program.key(),
        ctx.accounts.refundable_vault.to_account_info(),
        ctx.accounts.green_label_refundable_escrow.to_account_info(),
        Some(signer_seeds),
        ctx.accounts.usdc_mint.to_account_info(),
        ctx.accounts.relief_usdc_vault.to_account_info(),
        ctx.accounts.buyback_usdc_vault.to_account_info(),
        ctx.accounts.builders_usdc_vault.to_account_info(),
        ctx.accounts.staking_usdc_vault.to_account_info(),
        &mut treasury_usdc_state,
        &mut revenue_routing_stats,
        ctx.accounts.usdc_mint.key(),
        RevenueType::GreenLabelForfeitedBond,
        forfeit_amount,
        GREEN_LABEL_USDC_DECIMALS,
    )?;

    record_green_label_forfeit_governance_v1(
        &mut ctx.accounts.green_label_project,
        &mut ctx.accounts.green_label_dispute,
        &mut ctx.accounts.green_label_refundable_escrow,
        &mut ctx.accounts.green_label_forfeit_execution_record,
        ctx.accounts.execution_queue_item.key(),
        ctx.accounts.proposal_decision.key(),
        ctx.accounts.governance_proposal.key(),
        ctx.accounts.governance_proposal_action.key(),
        ctx.accounts.protocol_module_registry.key(),
        config_key,
        project_key,
        dispute_key,
        escrow_key,
        refundable_vault_key,
        ctx.accounts.treasury_config.key(),
        ctx.accounts.treasury_usdc_state.key(),
        ctx.accounts.revenue_routing_stats.key(),
        ctx.accounts.relief_usdc_vault.key(),
        ctx.accounts.buyback_usdc_vault.key(),
        ctx.accounts.builders_usdc_vault.key(),
        ctx.accounts.staking_usdc_vault.key(),
        forfeit_amount,
        escrow_usdc_mint,
        parameters,
        ctx.accounts
            .governance_proposal_action
            .canonical_payload_hash,
        escrow_status_before,
        project_status_before,
        dispute_status_before,
        ctx.accounts.executor.key(),
        now,
        ctx.bumps.green_label_forfeit_execution_record,
        record_key,
    )?;

    treasury_usdc_state.exit(ctx.program_id)?;
    revenue_routing_stats.exit(ctx.program_id)?;

    Ok(())
}

pub fn refund_green_label_escrow_v1_handler(ctx: Context<RefundGreenLabelEscrowV1>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let refund_amount = validate_green_label_escrow_refund(
        ctx.accounts.green_label_refundable_escrow.status,
        ctx.accounts.green_label_refundable_escrow.refundable_amount,
        ctx.accounts.green_label_refundable_escrow.refunded_amount,
        ctx.accounts.green_label_refundable_escrow.forfeited_amount,
        ctx.accounts
            .green_label_refundable_escrow
            .refund_available_after,
        now,
        ctx.accounts.green_label_project.active_dispute,
        ctx.accounts.green_label_project.terminal_action_type,
        ctx.accounts.green_label_project.terminal_proposal_id,
        ctx.accounts.green_label_project.terminal_payload_hash,
        ctx.accounts.green_label_refundable_escrow.payer,
        ctx.accounts.payer_refund_usdc_account.owner,
        ctx.accounts.green_label_refundable_escrow.usdc_mint,
        ctx.accounts.payer_refund_usdc_account.mint,
        ctx.accounts.refundable_vault.mint,
        ctx.accounts.refundable_vault.owner,
        ctx.accounts.green_label_refundable_escrow.key(),
        ctx.accounts.usdc_mint.key(),
        ctx.accounts.usdc_mint.decimals,
        ctx.accounts.refundable_vault.amount,
    )?;

    let project_key = ctx.accounts.green_label_project.key();
    let escrow_info = ctx.accounts.green_label_refundable_escrow.to_account_info();
    let refundable_vault_info = ctx.accounts.refundable_vault.to_account_info();
    let usdc_mint_info = ctx.accounts.usdc_mint.to_account_info();
    let payer_refund_info = ctx.accounts.payer_refund_usdc_account.to_account_info();
    execute_green_label_escrow_refund_internal_v1(
        &mut ctx.accounts.green_label_refundable_escrow,
        escrow_info,
        refundable_vault_info,
        usdc_mint_info,
        payer_refund_info,
        ctx.accounts.token_program.key(),
        project_key,
        refund_amount,
    )
}

pub fn forfeit_green_label_escrow_to_treasury_v1_handler<'info>(
    _ctx: Context<'info, ForfeitGreenLabelEscrowToTreasuryV1<'info>>,
) -> Result<()> {
    reject_legacy_green_label_forfeit_v1()
}

pub fn build_default_green_label_config_values(
    authority: Pubkey,
    usdc_mint: Pubkey,
    treasury_usdc_state_v2: Pubkey,
    base_bond_treasury_vault: Pubkey,
    relief_or_risk_vault: Pubkey,
    vault_authority_v2: Pubkey,
    security_governance_config: Pubkey,
    bump: u8,
) -> Result<GreenLabelConfigInitValues> {
    validate_green_label_bps_config(BASE_BOND_REFUND_BPS, BASE_BOND_TREASURY_BPS)?;

    Ok(GreenLabelConfigInitValues {
        authority,
        usdc_mint,
        min_base_bond_usdc: MIN_GREEN_LABEL_BASE_BOND_USDC,
        base_refund_bps: BASE_BOND_REFUND_BPS,
        base_treasury_bps: BASE_BOND_TREASURY_BPS,
        observation_period_seconds: DEFAULT_OBSERVATION_PERIOD_SECONDS,
        dispute_window_seconds: DEFAULT_DISPUTE_WINDOW_SECONDS,
        response_window_seconds: DEFAULT_RESPONSE_WINDOW_SECONDS,
        project_count: 0,
        treasury_usdc_state_v2,
        base_bond_treasury_vault,
        relief_or_risk_vault,
        vault_authority_v2,
        security_governance_config,
        is_paused: false,
        bump,
        reserved: [0; GREEN_LABEL_CONFIG_RESERVED_BYTES],
    })
}

pub fn green_label_certification_execution_type_stable_code_v1(
    execution_type: GreenLabelCertificationExecutionTypeV1,
) -> u8 {
    match execution_type {
        GreenLabelCertificationExecutionTypeV1::Approve => 1,
        GreenLabelCertificationExecutionTypeV1::Reject => 2,
        GreenLabelCertificationExecutionTypeV1::Revoke => 3,
    }
}

pub fn green_label_certification_execution_type_from_stable_code_v1(
    code: u8,
) -> Result<GreenLabelCertificationExecutionTypeV1> {
    match code {
        1 => Ok(GreenLabelCertificationExecutionTypeV1::Approve),
        2 => Ok(GreenLabelCertificationExecutionTypeV1::Reject),
        3 => Ok(GreenLabelCertificationExecutionTypeV1::Revoke),
        _ => err!(CustomError::InvalidGreenLabelCertificationSchema),
    }
}

pub fn hash_green_label_certification_decision_parameters_v1(
    parameters: &GreenLabelCertificationDecisionParametersV1,
) -> Result<[u8; 32]> {
    require!(
        parameters.schema_version == GREEN_LABEL_CERTIFICATION_SCHEMA_VERSION,
        CustomError::InvalidGreenLabelCertificationSchema
    );
    require!(
        matches!(
            parameters.action_type,
            GovernanceActionTypeV1::GreenLabelApproveCertification
                | GovernanceActionTypeV1::GreenLabelRejectCertification
                | GovernanceActionTypeV1::GreenLabelRevokeCertification
        ),
        CustomError::GreenLabelCertificationActionMismatch
    );
    require_keys_neq!(
        parameters.green_label_config,
        Pubkey::default(),
        CustomError::GreenLabelCertificationTargetMismatch
    );
    require_keys_neq!(
        parameters.green_label_project,
        Pubkey::default(),
        CustomError::GreenLabelCertificationTargetMismatch
    );
    require_keys_neq!(
        parameters.certification_state,
        Pubkey::default(),
        CustomError::GreenLabelCertificationTargetMismatch
    );
    require_keys_neq!(
        parameters.project_authority,
        Pubkey::default(),
        CustomError::InvalidGreenLabelProjectOwner
    );
    require_keys_neq!(
        parameters.usdc_mint,
        Pubkey::default(),
        CustomError::InvalidGreenLabelMint
    );
    require!(parameters.proposal_id > 0, CustomError::InvalidProposalId);

    let mut bytes = Vec::new();
    bytes.extend_from_slice(GREEN_LABEL_CERTIFICATION_DECISION_PARAMETERS_V1_DOMAIN);
    parameters
        .serialize(&mut bytes)
        .map_err(|_| error!(CustomError::GreenLabelCertificationParametersMismatch))?;
    hash_contributor_payload(&bytes)
}

pub fn build_green_label_certification_decision_parameters_v1(
    config: &GreenLabelConfigV1,
    config_key: Pubkey,
    project: &GreenLabelProjectV1,
    project_key: Pubkey,
    certification_state_key: Pubkey,
    action_type: GovernanceActionTypeV1,
    expected_certification_status: GreenLabelCertificationStatusV1,
    proposal_id: u64,
) -> Result<GreenLabelCertificationDecisionParametersV1> {
    require!(
        matches!(
            action_type,
            GovernanceActionTypeV1::GreenLabelApproveCertification
                | GovernanceActionTypeV1::GreenLabelRejectCertification
                | GovernanceActionTypeV1::GreenLabelRevokeCertification
        ),
        CustomError::GreenLabelCertificationActionMismatch
    );
    Ok(GreenLabelCertificationDecisionParametersV1 {
        schema_version: GREEN_LABEL_CERTIFICATION_SCHEMA_VERSION,
        green_label_config: config_key,
        green_label_project: project_key,
        certification_state: certification_state_key,
        action_type,
        project_authority: project.project_owner,
        bond_tier: project.bond_tier,
        bond_vault: project.bond_vault,
        usdc_mint: config.usdc_mint,
        observation_end_ts: project.observation_end_ts,
        expected_project_status: project.status,
        expected_certification_status,
        proposal_id,
    })
}

pub fn hash_green_label_certification_fee_parameters_v1(
    parameters: &GreenLabelCertificationFeeParametersV1,
) -> Result<[u8; 32]> {
    require!(
        parameters.schema_version == GREEN_LABEL_CERTIFICATION_FEE_SCHEMA_VERSION,
        CustomError::InvalidGreenLabelCertificationFeePolicySchema
    );
    require!(
        parameters.policy_version == GREEN_LABEL_CERTIFICATION_FEE_POLICY_VERSION,
        CustomError::InvalidGreenLabelCertificationFeePolicySchema
    );
    require!(
        parameters.revenue_type == RevenueType::GreenLabelCertificationFee,
        CustomError::GreenLabelCertificationFeeParametersMismatch
    );
    require_keys_neq!(
        parameters.green_label_config,
        Pubkey::default(),
        CustomError::GreenLabelCertificationFeeParametersMismatch
    );
    require_keys_neq!(
        parameters.fee_policy,
        Pubkey::default(),
        CustomError::GreenLabelCertificationFeeParametersMismatch
    );
    require_keys_neq!(
        parameters.green_label_project,
        Pubkey::default(),
        CustomError::GreenLabelCertificationFeeProjectMismatch
    );
    require_keys_neq!(
        parameters.project_owner,
        Pubkey::default(),
        CustomError::GreenLabelCertificationFeePayerMismatch
    );
    require_keys_neq!(
        parameters.payer,
        Pubkey::default(),
        CustomError::GreenLabelCertificationFeePayerMismatch
    );
    require_keys_neq!(
        parameters.payer_token_account,
        Pubkey::default(),
        CustomError::GreenLabelCertificationFeePayerMismatch
    );
    require!(
        parameters.fee_amount_usdc > 0,
        CustomError::InvalidGreenLabelCertificationFeeAmount
    );
    require_keys_neq!(
        parameters.usdc_mint,
        Pubkey::default(),
        CustomError::GreenLabelCertificationFeeMintMismatch
    );
    require_keys_neq!(
        parameters.treasury_config,
        Pubkey::default(),
        CustomError::GreenLabelCertificationFeeTreasuryMismatch
    );
    require_keys_neq!(
        parameters.treasury_usdc_state,
        Pubkey::default(),
        CustomError::GreenLabelCertificationFeeTreasuryMismatch
    );
    require_keys_neq!(
        parameters.revenue_routing_stats,
        Pubkey::default(),
        CustomError::GreenLabelCertificationFeeTreasuryMismatch
    );
    require_keys_neq!(
        parameters.relief_usdc_vault,
        Pubkey::default(),
        CustomError::GreenLabelCertificationFeeTreasuryMismatch
    );
    require_keys_neq!(
        parameters.buyback_usdc_vault,
        Pubkey::default(),
        CustomError::GreenLabelCertificationFeeTreasuryMismatch
    );
    require_keys_neq!(
        parameters.builders_usdc_vault,
        Pubkey::default(),
        CustomError::GreenLabelCertificationFeeTreasuryMismatch
    );
    require_keys_neq!(
        parameters.staking_usdc_vault,
        Pubkey::default(),
        CustomError::GreenLabelCertificationFeeTreasuryMismatch
    );

    let mut bytes = Vec::new();
    bytes.extend_from_slice(GREEN_LABEL_CERTIFICATION_FEE_PARAMETERS_V1_DOMAIN);
    parameters
        .serialize(&mut bytes)
        .map_err(|_| error!(CustomError::GreenLabelCertificationFeeParametersMismatch))?;
    hash_contributor_payload(&bytes)
}

#[allow(clippy::too_many_arguments)]
pub fn build_green_label_certification_fee_parameters_v1(
    green_label_config: Pubkey,
    fee_policy: Pubkey,
    policy: &GreenLabelCertificationFeePolicyV1,
    green_label_project: Pubkey,
    project: &GreenLabelProjectV1,
    payer: Pubkey,
    payer_token_account: Pubkey,
    treasury_config: Pubkey,
    treasury_usdc_state: Pubkey,
    revenue_routing_stats: Pubkey,
    relief_usdc_vault: Pubkey,
    buyback_usdc_vault: Pubkey,
    builders_usdc_vault: Pubkey,
    staking_usdc_vault: Pubkey,
) -> Result<GreenLabelCertificationFeeParametersV1> {
    Ok(GreenLabelCertificationFeeParametersV1 {
        schema_version: GREEN_LABEL_CERTIFICATION_FEE_SCHEMA_VERSION,
        green_label_config,
        fee_policy,
        policy_version: policy.policy_version,
        green_label_project,
        project_id: project.project_id,
        project_owner: project.project_owner,
        payer,
        payer_token_account,
        fee_amount_usdc: policy.fee_amount_usdc,
        usdc_mint: policy.usdc_mint,
        treasury_config,
        treasury_usdc_state,
        revenue_routing_stats,
        relief_usdc_vault,
        buyback_usdc_vault,
        builders_usdc_vault,
        staking_usdc_vault,
        revenue_type: RevenueType::GreenLabelCertificationFee,
    })
}

pub fn record_green_label_certification_state_init(
    certification_state: &mut GreenLabelCertificationStateV1,
    project_key: Pubkey,
    config_key: Pubkey,
    project_status: GreenLabelStatus,
    now: i64,
    bump: u8,
) -> Result<()> {
    require!(
        certification_state.green_label_project == Pubkey::default(),
        CustomError::GreenLabelCertificationStateMismatch
    );
    require!(
        matches!(
            project_status,
            GreenLabelStatus::PendingBondDeposit | GreenLabelStatus::PendingObservation
        ),
        CustomError::InvalidGreenLabelStatus
    );
    require!(now > 0, CustomError::InvalidGreenLabelCertificationSchema);

    certification_state.green_label_project = project_key;
    certification_state.green_label_config = config_key;
    certification_state.certification_status = GreenLabelCertificationStatusV1::Pending;
    certification_state.last_governance_proposal = Pubkey::default();
    certification_state.last_execution_queue = Pubkey::default();
    certification_state.last_execution_record = Pubkey::default();
    certification_state.last_action_type = GovernanceActionTypeV1::GreenLabelApproveCertification;
    certification_state.decision_at = 0;
    certification_state.created_at = now;
    certification_state.updated_at = now;
    certification_state.schema_version = GREEN_LABEL_CERTIFICATION_SCHEMA_VERSION;
    certification_state.bump = bump;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn validate_green_label_certification_execution_context_v1(
    security_governance_config: &GovernanceConfigV1,
    security_governance_config_key: Pubkey,
    green_label_config: &GreenLabelConfigV1,
    green_label_project: &GreenLabelProjectV1,
    green_label_project_key: Pubkey,
    certification_state: &GreenLabelCertificationStateV1,
    certification_state_key: Pubkey,
    protocol_module_registry: &ProtocolModuleRegistryV1,
    protocol_module_registry_key: Pubkey,
    governance_proposal: &GovernanceProposalV1,
    governance_proposal_key: Pubkey,
    governance_proposal_action: &GovernanceProposalActionV1,
    governance_proposal_action_key: Pubkey,
    governance_decision_adapter: &UniversalGovernanceDecisionAdapterV1,
    governance_decision_adapter_key: Pubkey,
    proposal_decision: &ProposalDecisionV1,
    proposal_decision_key: Pubkey,
    execution_queue_item: &ExecutionQueueItemV1,
    execution_queue_item_key: Pubkey,
    expected_governance_action: GovernanceActionTypeV1,
    expected_proposal_type: ProposalType,
    expected_security_action: ActionType,
    parameters: &GreenLabelCertificationDecisionParametersV1,
) -> Result<()> {
    require!(
        !green_label_config.is_paused,
        CustomError::InvalidGreenLabelStatus
    );
    require!(
        !security_governance_config.is_paused,
        CustomError::SecurityLayerPaused
    );
    require_keys_eq!(
        green_label_config.security_governance_config,
        security_governance_config_key,
        CustomError::ProtocolModuleGovernanceConfigMismatch
    );
    validate_protocol_module_registry_v1(
        protocol_module_registry,
        protocol_module_registry_key,
        security_governance_config_key,
        ProtocolModuleIdV1::GreenLabel,
        crate::ID,
    )?;
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
        governance_proposal_action.action_type == expected_governance_action,
        CustomError::GreenLabelCertificationActionMismatch
    );
    require!(
        governance_proposal_action.module_id == ProtocolModuleIdV1::GreenLabel,
        CustomError::GreenLabelCertificationActionMismatch
    );
    require_keys_eq!(
        governance_proposal_action.target_program,
        crate::ID,
        CustomError::GreenLabelCertificationTargetMismatch
    );
    require_keys_eq!(
        governance_proposal_action.target_account,
        green_label_project_key,
        CustomError::GreenLabelCertificationTargetMismatch
    );
    require!(
        map_governance_action_to_security_action(governance_proposal_action.action_type)?
            == expected_security_action,
        CustomError::GreenLabelCertificationActionMismatch
    );
    require_keys_eq!(
        certification_state.green_label_project,
        green_label_project_key,
        CustomError::GreenLabelCertificationStateMismatch
    );
    require_keys_eq!(
        certification_state.green_label_config,
        parameters.green_label_config,
        CustomError::GreenLabelCertificationStateMismatch
    );
    require!(
        certification_state.schema_version == GREEN_LABEL_CERTIFICATION_SCHEMA_VERSION,
        CustomError::InvalidGreenLabelCertificationSchema
    );
    require_keys_eq!(
        parameters.green_label_project,
        green_label_project_key,
        CustomError::GreenLabelCertificationTargetMismatch
    );
    require_keys_eq!(
        parameters.certification_state,
        certification_state_key,
        CustomError::GreenLabelCertificationTargetMismatch
    );
    require_keys_eq!(
        parameters.green_label_config,
        certification_state.green_label_config,
        CustomError::GreenLabelCertificationTargetMismatch
    );
    require!(
        parameters.action_type == expected_governance_action,
        CustomError::GreenLabelCertificationActionMismatch
    );
    require_keys_eq!(
        parameters.project_authority,
        green_label_project.project_owner,
        CustomError::InvalidGreenLabelProjectOwner
    );
    require!(
        parameters.bond_tier == green_label_project.bond_tier,
        CustomError::GreenLabelCertificationParametersMismatch
    );
    require_keys_eq!(
        parameters.bond_vault,
        green_label_project.bond_vault,
        CustomError::GreenLabelCertificationParametersMismatch
    );
    require_keys_eq!(
        parameters.usdc_mint,
        green_label_config.usdc_mint,
        CustomError::InvalidGreenLabelMint
    );
    require!(
        parameters.observation_end_ts == green_label_project.observation_end_ts,
        CustomError::GreenLabelCertificationParametersMismatch
    );
    require!(
        parameters.expected_project_status == green_label_project.status,
        CustomError::GreenLabelCertificationParametersMismatch
    );
    require!(
        parameters.expected_certification_status == certification_state.certification_status,
        CustomError::GreenLabelCertificationParametersMismatch
    );
    require!(
        parameters.proposal_id == governance_proposal.proposal_id,
        CustomError::InvalidProposalId
    );

    let parameters_hash = hash_green_label_certification_decision_parameters_v1(parameters)?;
    require!(
        governance_proposal_action.parameters_hash == parameters_hash,
        CustomError::GreenLabelCertificationParametersMismatch
    );
    require_keys_eq!(
        governance_decision_adapter.governance_proposal,
        governance_proposal_key,
        CustomError::InvalidGovernanceDecisionAdapter
    );
    require_keys_eq!(
        governance_decision_adapter.proposal_decision,
        proposal_decision_key,
        CustomError::InvalidGovernanceDecisionAdapter
    );
    require!(
        governance_decision_adapter.action_type == expected_security_action,
        CustomError::GreenLabelCertificationActionMismatch
    );
    require_keys_eq!(
        governance_decision_adapter.target_program,
        governance_proposal_action.target_program,
        CustomError::GreenLabelCertificationTargetMismatch
    );
    require_keys_eq!(
        governance_decision_adapter.target_account,
        green_label_project_key,
        CustomError::GreenLabelCertificationTargetMismatch
    );
    require!(
        governance_decision_adapter.payload_hash
            == governance_proposal_action.canonical_payload_hash,
        CustomError::GreenLabelCertificationParametersMismatch
    );
    require!(
        governance_decision_adapter_key != Pubkey::default(),
        CustomError::InvalidGovernanceDecisionAdapter
    );
    require!(
        proposal_decision.proposal_id == governance_proposal.proposal_id,
        CustomError::InvalidProposalId
    );
    require!(
        proposal_decision.proposal_type == expected_proposal_type,
        CustomError::InvalidActionForProposalType
    );
    require!(
        proposal_decision.decision == ProposalDecision::Approved,
        CustomError::ProposalNotApproved
    );
    require!(
        execution_queue_item.proposal_id == governance_proposal.proposal_id,
        CustomError::InvalidProposalId
    );
    require!(
        execution_queue_item.status == ExecutionStatus::Executed,
        CustomError::InvalidExecutionStatus
    );
    require!(
        execution_queue_item.executed_at > 0,
        CustomError::InvalidExecutionStatus
    );
    require!(
        execution_queue_item.decision == ProposalDecision::Approved,
        CustomError::ProposalNotApproved
    );
    require!(
        execution_queue_item.action_type == expected_security_action,
        CustomError::GreenLabelCertificationActionMismatch
    );
    require_keys_eq!(
        execution_queue_item.target_program,
        crate::ID,
        CustomError::GreenLabelCertificationTargetMismatch
    );
    require_keys_eq!(
        execution_queue_item.target_account,
        green_label_project_key,
        CustomError::GreenLabelCertificationTargetMismatch
    );
    require!(
        execution_queue_item.payload_hash == governance_proposal_action.canonical_payload_hash,
        CustomError::GreenLabelCertificationParametersMismatch
    );
    require!(
        execution_queue_item_key != Pubkey::default(),
        CustomError::InvalidGreenLabelExecutionQueue
    );

    Ok(())
}

pub fn validate_green_label_approve_certification_business_v1(
    config: &GreenLabelConfigV1,
    project: &GreenLabelProjectV1,
    certification_state: &GreenLabelCertificationStateV1,
    provided_bond_vault: Pubkey,
    green_bond_vault: &TokenAccount,
    provided_usdc_mint: Pubkey,
    usdc_decimals: u8,
    now: i64,
) -> Result<()> {
    require!(
        certification_state.certification_status == GreenLabelCertificationStatusV1::Pending,
        CustomError::GreenLabelCertificationAlreadyFinalized
    );
    require!(
        project.status == GreenLabelStatus::PendingObservation,
        CustomError::InvalidGreenLabelStatus
    );
    require!(
        now >= project.observation_end_ts,
        CustomError::GreenLabelObservationPeriodNotComplete
    );
    require!(
        project.active_dispute == Pubkey::default(),
        CustomError::GreenLabelUnresolvedDispute
    );
    require!(
        project.bond_vault != Pubkey::default()
            && project.bond_vault_authority != Pubkey::default()
            && project.total_bond_amount > 0,
        CustomError::InvalidGreenLabelBondVaultState
    );
    require_keys_eq!(
        provided_bond_vault,
        project.bond_vault,
        CustomError::InvalidGreenLabelBondVaultState
    );
    require_keys_eq!(
        green_bond_vault.mint,
        config.usdc_mint,
        CustomError::InvalidGreenLabelMint
    );
    require_keys_eq!(
        green_bond_vault.owner,
        project.bond_vault_authority,
        CustomError::InvalidGreenLabelBondVaultState
    );
    require_keys_eq!(
        provided_usdc_mint,
        config.usdc_mint,
        CustomError::InvalidGreenLabelMint
    );
    require!(
        usdc_decimals == GREEN_LABEL_USDC_DECIMALS,
        CustomError::InvalidGreenLabelMint
    );
    require!(
        green_bond_vault.amount >= project.total_bond_amount,
        CustomError::GreenLabelInsufficientBondVaultBalance
    );

    Ok(())
}

pub fn validate_green_label_reject_certification_business_v1(
    project_status: GreenLabelStatus,
    certification_status: GreenLabelCertificationStatusV1,
) -> Result<()> {
    require!(
        certification_status == GreenLabelCertificationStatusV1::Pending,
        CustomError::GreenLabelCertificationAlreadyFinalized
    );
    require!(
        matches!(
            project_status,
            GreenLabelStatus::PendingBondDeposit | GreenLabelStatus::PendingObservation
        ),
        CustomError::InvalidGreenLabelStatus
    );
    Ok(())
}

pub fn validate_green_label_revoke_certification_business_v1(
    project_status: GreenLabelStatus,
    certification_status: GreenLabelCertificationStatusV1,
) -> Result<()> {
    require!(
        certification_status == GreenLabelCertificationStatusV1::Approved,
        CustomError::GreenLabelCertificationNotApproved
    );
    require!(
        project_status == GreenLabelStatus::ActiveGreenLabel,
        CustomError::InvalidGreenLabelStatus
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn record_green_label_approve_certification_v1(
    project: &mut GreenLabelProjectV1,
    certification_state: &mut GreenLabelCertificationStateV1,
    execution_record: &mut GreenLabelCertificationExecutionRecordV1,
    execution_queue_item: Pubkey,
    proposal_decision: Pubkey,
    governance_proposal: Pubkey,
    governance_proposal_action: Pubkey,
    project_key: Pubkey,
    certification_state_key: Pubkey,
    module_registry: Pubkey,
    execution_record_key: Pubkey,
    parameters: GreenLabelCertificationDecisionParametersV1,
    canonical_governance_payload_hash: [u8; 32],
    project_status_before: GreenLabelStatus,
    certification_status_before: GreenLabelCertificationStatusV1,
    executor: Pubkey,
    now: i64,
    bump: u8,
) -> Result<()> {
    validate_green_label_status_transition(
        project.status,
        GreenLabelStatus::ActiveGreenLabel,
        false,
    )?;
    project.status = GreenLabelStatus::ActiveGreenLabel;
    project.approved_at = now;
    record_green_label_certification_decision_v1(
        certification_state,
        execution_record,
        execution_queue_item,
        proposal_decision,
        governance_proposal,
        governance_proposal_action,
        project_key,
        certification_state_key,
        module_registry,
        execution_record_key,
        GreenLabelCertificationExecutionTypeV1::Approve,
        GreenLabelCertificationStatusV1::Approved,
        GovernanceActionTypeV1::GreenLabelApproveCertification,
        parameters,
        canonical_governance_payload_hash,
        project_status_before,
        GreenLabelStatus::ActiveGreenLabel,
        certification_status_before,
        executor,
        now,
        bump,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn record_green_label_certification_decision_v1(
    certification_state: &mut GreenLabelCertificationStateV1,
    execution_record: &mut GreenLabelCertificationExecutionRecordV1,
    execution_queue_item: Pubkey,
    proposal_decision: Pubkey,
    governance_proposal: Pubkey,
    governance_proposal_action: Pubkey,
    project_key: Pubkey,
    certification_state_key: Pubkey,
    module_registry: Pubkey,
    execution_record_key: Pubkey,
    execution_type: GreenLabelCertificationExecutionTypeV1,
    next_certification_status: GreenLabelCertificationStatusV1,
    governance_action_type: GovernanceActionTypeV1,
    parameters: GreenLabelCertificationDecisionParametersV1,
    canonical_governance_payload_hash: [u8; 32],
    project_status_before: GreenLabelStatus,
    project_status_after: GreenLabelStatus,
    certification_status_before: GreenLabelCertificationStatusV1,
    executor: Pubkey,
    executed_at: i64,
    bump: u8,
) -> Result<()> {
    require!(
        execution_record.execution_queue_item == Pubkey::default(),
        CustomError::GreenLabelCertificationExecutionAlreadyCompleted
    );
    require_keys_eq!(
        certification_state.green_label_project,
        project_key,
        CustomError::GreenLabelCertificationStateMismatch
    );
    require_keys_eq!(
        parameters.certification_state,
        certification_state_key,
        CustomError::GreenLabelCertificationStateMismatch
    );
    require!(
        certification_status_before == certification_state.certification_status,
        CustomError::GreenLabelCertificationStateMismatch
    );
    validate_green_label_certification_status_transition(
        certification_state.certification_status,
        next_certification_status,
    )?;
    let parameters_hash = hash_green_label_certification_decision_parameters_v1(&parameters)?;

    certification_state.certification_status = next_certification_status;
    certification_state.last_governance_proposal = governance_proposal;
    certification_state.last_execution_queue = execution_queue_item;
    certification_state.last_execution_record = execution_record_key;
    certification_state.last_action_type = governance_action_type;
    certification_state.decision_at = executed_at;
    certification_state.updated_at = executed_at;

    execution_record.execution_queue_item = execution_queue_item;
    execution_record.proposal_decision = proposal_decision;
    execution_record.governance_proposal = governance_proposal;
    execution_record.governance_proposal_action = governance_proposal_action;
    execution_record.green_label_project = project_key;
    execution_record.certification_state = certification_state_key;
    execution_record.module_registry = module_registry;
    execution_record.execution_type = execution_type;
    execution_record.governance_action_type = governance_action_type;
    execution_record.target_account = project_key;
    execution_record.parameters_hash = parameters_hash;
    execution_record.canonical_governance_payload_hash = canonical_governance_payload_hash;
    execution_record.project_status_before = project_status_before;
    execution_record.project_status_after = project_status_after;
    execution_record.certification_status_before = certification_status_before;
    execution_record.certification_status_after = next_certification_status;
    execution_record.executor = executor;
    execution_record.executed_at = executed_at;
    execution_record.schema_version = GREEN_LABEL_CERTIFICATION_SCHEMA_VERSION;
    execution_record.bump = bump;

    Ok(())
}

pub fn validate_green_label_certification_status_transition(
    current: GreenLabelCertificationStatusV1,
    next: GreenLabelCertificationStatusV1,
) -> Result<()> {
    let valid = matches!(
        (current, next),
        (
            GreenLabelCertificationStatusV1::Pending,
            GreenLabelCertificationStatusV1::Approved
        ) | (
            GreenLabelCertificationStatusV1::Pending,
            GreenLabelCertificationStatusV1::Rejected
        ) | (
            GreenLabelCertificationStatusV1::Approved,
            GreenLabelCertificationStatusV1::Revoked
        )
    );
    require!(valid, CustomError::GreenLabelCertificationAlreadyFinalized);
    Ok(())
}

pub fn green_label_escrow_execution_type_stable_code_v1(
    execution_type: GreenLabelEscrowExecutionTypeV1,
) -> u8 {
    match execution_type {
        GreenLabelEscrowExecutionTypeV1::Refund => 1,
        GreenLabelEscrowExecutionTypeV1::Forfeit => 2,
    }
}

pub fn green_label_escrow_execution_type_from_stable_code_v1(
    code: u8,
) -> Result<GreenLabelEscrowExecutionTypeV1> {
    match code {
        1 => Ok(GreenLabelEscrowExecutionTypeV1::Refund),
        2 => Ok(GreenLabelEscrowExecutionTypeV1::Forfeit),
        _ => err!(CustomError::InvalidGreenLabelRefundSchema),
    }
}

pub fn hash_green_label_refund_parameters_v1(
    parameters: &GreenLabelRefundParametersV1,
) -> Result<[u8; 32]> {
    require!(
        parameters.schema_version == GREEN_LABEL_REFUND_SCHEMA_VERSION,
        CustomError::InvalidGreenLabelRefundSchema
    );
    require!(
        parameters.action_type == GovernanceActionTypeV1::GreenLabelRefundBond,
        CustomError::GreenLabelRefundActionMismatch
    );
    require_keys_neq!(
        parameters.green_label_config,
        Pubkey::default(),
        CustomError::GreenLabelRefundTargetMismatch
    );
    require_keys_neq!(
        parameters.green_label_project,
        Pubkey::default(),
        CustomError::GreenLabelRefundTargetMismatch
    );
    require_keys_neq!(
        parameters.refundable_escrow,
        Pubkey::default(),
        CustomError::GreenLabelRefundTargetMismatch
    );
    require_keys_neq!(
        parameters.refundable_vault,
        Pubkey::default(),
        CustomError::GreenLabelRefundVaultMismatch
    );
    require_keys_neq!(
        parameters.original_payer,
        Pubkey::default(),
        CustomError::GreenLabelRefundWrongPayer
    );
    require_keys_neq!(
        parameters.payer_destination_token_account,
        Pubkey::default(),
        CustomError::GreenLabelRefundWrongDestination
    );
    require!(
        parameters.refund_amount_usdc > 0,
        CustomError::GreenLabelRefundAmountMismatch
    );
    require_keys_neq!(
        parameters.usdc_mint,
        Pubkey::default(),
        CustomError::GreenLabelRefundMintMismatch
    );
    require!(parameters.proposal_id > 0, CustomError::InvalidProposalId);

    let mut bytes = Vec::new();
    bytes.extend_from_slice(GREEN_LABEL_REFUND_PARAMETERS_V1_DOMAIN);
    parameters
        .serialize(&mut bytes)
        .map_err(|_| error!(CustomError::GreenLabelRefundParametersMismatch))?;
    hash_contributor_payload(&bytes)
}

#[allow(clippy::too_many_arguments)]
pub fn build_green_label_refund_parameters_v1(
    green_label_config: Pubkey,
    green_label_project: Pubkey,
    green_label_dispute: Pubkey,
    refundable_escrow: Pubkey,
    refundable_vault: Pubkey,
    original_payer: Pubkey,
    payer_destination_token_account: Pubkey,
    refund_amount_usdc: u64,
    usdc_mint: Pubkey,
    expected_escrow_status: GreenLabelEscrowStatusV1,
    proposal_id: u64,
) -> Result<GreenLabelRefundParametersV1> {
    Ok(GreenLabelRefundParametersV1 {
        schema_version: GREEN_LABEL_REFUND_SCHEMA_VERSION,
        green_label_config,
        green_label_project,
        green_label_dispute,
        refundable_escrow,
        refundable_vault,
        original_payer,
        payer_destination_token_account,
        refund_amount_usdc,
        usdc_mint,
        expected_escrow_status,
        proposal_id,
        action_type: GovernanceActionTypeV1::GreenLabelRefundBond,
    })
}

pub fn hash_green_label_forfeit_parameters_v1(
    parameters: &GreenLabelForfeitParametersV1,
) -> Result<[u8; 32]> {
    require!(
        parameters.schema_version == GREEN_LABEL_FORFEIT_SCHEMA_VERSION,
        CustomError::InvalidGreenLabelForfeitSchema
    );
    require!(
        parameters.action_type == GovernanceActionTypeV1::GreenLabelSlashBond,
        CustomError::GreenLabelForfeitActionMismatch
    );
    require!(
        parameters.revenue_type == RevenueType::GreenLabelForfeitedBond,
        CustomError::GreenLabelForfeitActionMismatch
    );
    require_keys_neq!(
        parameters.green_label_config,
        Pubkey::default(),
        CustomError::GreenLabelForfeitTargetMismatch
    );
    require_keys_neq!(
        parameters.green_label_project,
        Pubkey::default(),
        CustomError::GreenLabelForfeitTargetMismatch
    );
    require_keys_neq!(
        parameters.green_label_dispute,
        Pubkey::default(),
        CustomError::GreenLabelForfeitDisputeMismatch
    );
    require_keys_neq!(
        parameters.refundable_escrow,
        Pubkey::default(),
        CustomError::GreenLabelForfeitTargetMismatch
    );
    require_keys_neq!(
        parameters.refundable_vault,
        Pubkey::default(),
        CustomError::GreenLabelForfeitVaultMismatch
    );
    require!(
        parameters.forfeited_amount_usdc > 0,
        CustomError::GreenLabelForfeitAmountMismatch
    );
    require_keys_neq!(
        parameters.treasury_config,
        Pubkey::default(),
        CustomError::GreenLabelForfeitTargetMismatch
    );
    require_keys_neq!(
        parameters.treasury_usdc_state,
        Pubkey::default(),
        CustomError::GreenLabelForfeitTargetMismatch
    );
    require_keys_neq!(
        parameters.revenue_routing_stats,
        Pubkey::default(),
        CustomError::GreenLabelForfeitTargetMismatch
    );
    require_keys_neq!(
        parameters.relief_usdc_vault,
        Pubkey::default(),
        CustomError::GreenLabelForfeitVaultMismatch
    );
    require_keys_neq!(
        parameters.buyback_usdc_vault,
        Pubkey::default(),
        CustomError::GreenLabelForfeitVaultMismatch
    );
    require_keys_neq!(
        parameters.builders_usdc_vault,
        Pubkey::default(),
        CustomError::GreenLabelForfeitVaultMismatch
    );
    require_keys_neq!(
        parameters.staking_usdc_vault,
        Pubkey::default(),
        CustomError::GreenLabelForfeitVaultMismatch
    );
    require_keys_neq!(
        parameters.usdc_mint,
        Pubkey::default(),
        CustomError::GreenLabelForfeitMintMismatch
    );
    require!(parameters.proposal_id > 0, CustomError::InvalidProposalId);

    let mut bytes = Vec::new();
    bytes.extend_from_slice(GREEN_LABEL_FORFEIT_PARAMETERS_V1_DOMAIN);
    parameters
        .serialize(&mut bytes)
        .map_err(|_| error!(CustomError::GreenLabelForfeitParametersMismatch))?;
    hash_contributor_payload(&bytes)
}

#[allow(clippy::too_many_arguments)]
pub fn build_green_label_forfeit_parameters_v1(
    green_label_config: Pubkey,
    green_label_project: Pubkey,
    green_label_dispute: Pubkey,
    refundable_escrow: Pubkey,
    refundable_vault: Pubkey,
    forfeited_amount_usdc: u64,
    treasury_config: Pubkey,
    treasury_usdc_state: Pubkey,
    revenue_routing_stats: Pubkey,
    relief_usdc_vault: Pubkey,
    buyback_usdc_vault: Pubkey,
    builders_usdc_vault: Pubkey,
    staking_usdc_vault: Pubkey,
    usdc_mint: Pubkey,
    expected_escrow_status: GreenLabelEscrowStatusV1,
    expected_project_status: GreenLabelStatus,
    expected_dispute_status: DisputeStatus,
    proposal_id: u64,
) -> Result<GreenLabelForfeitParametersV1> {
    Ok(GreenLabelForfeitParametersV1 {
        schema_version: GREEN_LABEL_FORFEIT_SCHEMA_VERSION,
        green_label_config,
        green_label_project,
        green_label_dispute,
        refundable_escrow,
        refundable_vault,
        forfeited_amount_usdc,
        treasury_config,
        treasury_usdc_state,
        revenue_routing_stats,
        relief_usdc_vault,
        buyback_usdc_vault,
        builders_usdc_vault,
        staking_usdc_vault,
        usdc_mint,
        revenue_type: RevenueType::GreenLabelForfeitedBond,
        expected_escrow_status,
        expected_project_status,
        expected_dispute_status,
        action_type: GovernanceActionTypeV1::GreenLabelSlashBond,
        proposal_id,
    })
}

pub fn derive_green_label_refund_amount_v1(escrow: &GreenLabelRefundableEscrowV1) -> Result<u64> {
    if escrow.status == GreenLabelEscrowStatusV1::Refunded {
        return err!(CustomError::GreenLabelEscrowAlreadyRefunded);
    }
    if escrow.status == GreenLabelEscrowStatusV1::Forfeited {
        return err!(CustomError::GreenLabelEscrowAlreadyForfeited);
    }
    require!(
        matches!(
            escrow.status,
            GreenLabelEscrowStatusV1::Locked | GreenLabelEscrowStatusV1::Refundable
        ),
        CustomError::GreenLabelRefundNotEligible
    );
    require!(
        escrow.refundable_amount <= escrow.deposited_amount,
        CustomError::InvalidGreenLabelEscrowAmount
    );
    let refund_amount = calculate_green_label_escrow_remaining_amount(
        escrow.refundable_amount,
        escrow.refunded_amount,
        escrow.forfeited_amount,
    )?;
    require!(
        refund_amount > 0,
        CustomError::GreenLabelRefundAmountMismatch
    );
    Ok(refund_amount)
}

pub fn derive_green_label_forfeitable_amount_v1(
    escrow: &GreenLabelRefundableEscrowV1,
) -> Result<u64> {
    if escrow.status == GreenLabelEscrowStatusV1::Refunded {
        return err!(CustomError::GreenLabelEscrowAlreadyRefunded);
    }
    if escrow.status == GreenLabelEscrowStatusV1::Forfeited {
        return err!(CustomError::GreenLabelEscrowAlreadyForfeited);
    }
    require!(
        matches!(
            escrow.status,
            GreenLabelEscrowStatusV1::Locked | GreenLabelEscrowStatusV1::Refundable
        ),
        CustomError::GreenLabelForfeitNotEligible
    );
    require!(
        escrow.refundable_amount <= escrow.deposited_amount,
        CustomError::InvalidGreenLabelEscrowAmount
    );
    let forfeit_amount = calculate_green_label_escrow_remaining_amount(
        escrow.refundable_amount,
        escrow.refunded_amount,
        escrow.forfeited_amount,
    )?;
    require!(
        forfeit_amount > 0,
        CustomError::GreenLabelForfeitAmountMismatch
    );
    Ok(forfeit_amount)
}

pub fn validate_green_label_forfeit_vault_balance_v1(
    vault_balance: u64,
    forfeit_amount: u64,
) -> Result<()> {
    require!(
        vault_balance >= forfeit_amount,
        CustomError::GreenLabelForfeitInsufficientFunds
    );
    Ok(())
}

pub fn validate_green_label_forfeit_mint_accounts_v1(
    escrow_usdc_mint: Pubkey,
    escrow_key: Pubkey,
    refundable_vault_mint: Pubkey,
    refundable_vault_owner: Pubkey,
    provided_usdc_mint: Pubkey,
    usdc_decimals: u8,
) -> Result<()> {
    require_keys_eq!(
        refundable_vault_mint,
        escrow_usdc_mint,
        CustomError::GreenLabelForfeitMintMismatch
    );
    require_keys_eq!(
        refundable_vault_owner,
        escrow_key,
        CustomError::GreenLabelForfeitVaultMismatch
    );
    require_keys_eq!(
        provided_usdc_mint,
        escrow_usdc_mint,
        CustomError::GreenLabelForfeitMintMismatch
    );
    require!(
        usdc_decimals == GREEN_LABEL_USDC_DECIMALS,
        CustomError::GreenLabelForfeitMintMismatch
    );

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn validate_green_label_forfeit_execution_context_v1(
    security_governance_config: &GovernanceConfigV1,
    security_governance_config_key: Pubkey,
    green_label_config: &GreenLabelConfigV1,
    green_label_config_key: Pubkey,
    green_label_project: &GreenLabelProjectV1,
    green_label_project_key: Pubkey,
    green_label_dispute: &GreenLabelDisputeV1,
    green_label_dispute_key: Pubkey,
    escrow: &GreenLabelRefundableEscrowV1,
    escrow_key: Pubkey,
    protocol_module_registry: &ProtocolModuleRegistryV1,
    protocol_module_registry_key: Pubkey,
    governance_proposal: &GovernanceProposalV1,
    governance_proposal_key: Pubkey,
    governance_proposal_action: &GovernanceProposalActionV1,
    governance_proposal_action_key: Pubkey,
    governance_decision_adapter: &UniversalGovernanceDecisionAdapterV1,
    governance_decision_adapter_key: Pubkey,
    proposal_decision: &ProposalDecisionV1,
    proposal_decision_key: Pubkey,
    execution_queue_item: &ExecutionQueueItemV1,
    execution_queue_item_key: Pubkey,
    treasury_config: &TreasuryConfigV2,
    treasury_config_key: Pubkey,
    treasury_usdc_state: &TreasuryUsdcStateV2,
    treasury_usdc_state_key: Pubkey,
    revenue_routing_stats: &RevenueRoutingStatsV1,
    revenue_routing_stats_key: Pubkey,
    vault_authority_key: Pubkey,
    refundable_vault_key: Pubkey,
    refundable_vault_mint: Pubkey,
    refundable_vault_owner: Pubkey,
    vault_balance: u64,
    relief_usdc_vault_key: Pubkey,
    relief_usdc_vault_mint: Pubkey,
    relief_usdc_vault_owner: Pubkey,
    buyback_usdc_vault_key: Pubkey,
    buyback_usdc_vault_mint: Pubkey,
    buyback_usdc_vault_owner: Pubkey,
    builders_usdc_vault_key: Pubkey,
    builders_usdc_vault_mint: Pubkey,
    builders_usdc_vault_owner: Pubkey,
    staking_usdc_vault_key: Pubkey,
    staking_usdc_vault_mint: Pubkey,
    staking_usdc_vault_owner: Pubkey,
    provided_usdc_mint: Pubkey,
    usdc_decimals: u8,
    now: i64,
    parameters: &GreenLabelForfeitParametersV1,
) -> Result<()> {
    require!(
        !green_label_config.is_paused,
        CustomError::InvalidGreenLabelStatus
    );
    require!(
        !security_governance_config.is_paused,
        CustomError::SecurityLayerPaused
    );
    require_keys_eq!(
        green_label_config.security_governance_config,
        security_governance_config_key,
        CustomError::ProtocolModuleGovernanceConfigMismatch
    );
    validate_protocol_module_registry_v1(
        protocol_module_registry,
        protocol_module_registry_key,
        security_governance_config_key,
        ProtocolModuleIdV1::GreenLabel,
        crate::ID,
    )?;
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
        governance_proposal_action.action_type == GovernanceActionTypeV1::GreenLabelSlashBond,
        CustomError::GreenLabelForfeitActionMismatch
    );
    require!(
        governance_proposal_action.module_id == ProtocolModuleIdV1::GreenLabel,
        CustomError::GreenLabelForfeitActionMismatch
    );
    require_keys_eq!(
        governance_proposal_action.target_program,
        crate::ID,
        CustomError::GreenLabelForfeitTargetMismatch
    );
    require_keys_eq!(
        governance_proposal_action.target_account,
        escrow_key,
        CustomError::GreenLabelForfeitTargetMismatch
    );
    require!(
        map_governance_action_to_security_action(governance_proposal_action.action_type)?
            == ActionType::GreenLabelSlash,
        CustomError::GreenLabelForfeitActionMismatch
    );

    require_keys_eq!(
        escrow.project,
        green_label_project_key,
        CustomError::GreenLabelForfeitTargetMismatch
    );
    require!(
        escrow.project_id == green_label_project.project_id,
        CustomError::GreenLabelForfeitTargetMismatch
    );
    require_keys_eq!(
        escrow.usdc_mint,
        green_label_config.usdc_mint,
        CustomError::GreenLabelForfeitMintMismatch
    );
    require_keys_eq!(
        escrow.refundable_vault,
        refundable_vault_key,
        CustomError::GreenLabelForfeitVaultMismatch
    );
    validate_green_label_forfeit_mint_accounts_v1(
        escrow.usdc_mint,
        escrow_key,
        refundable_vault_mint,
        refundable_vault_owner,
        provided_usdc_mint,
        usdc_decimals,
    )?;

    require_keys_eq!(
        green_label_project.active_dispute,
        green_label_dispute_key,
        CustomError::GreenLabelForfeitDisputeMismatch
    );
    require_keys_eq!(
        green_label_dispute.project,
        green_label_project_key,
        CustomError::GreenLabelForfeitDisputeMismatch
    );
    require!(
        matches!(
            green_label_project.status,
            GreenLabelStatus::Disputed | GreenLabelStatus::SlashQueued
        ),
        CustomError::InvalidGreenLabelStatus
    );
    require!(
        matches!(
            green_label_dispute.status,
            DisputeStatus::ReadyForDecision | DisputeStatus::DecisionQueued
        ),
        CustomError::InvalidGreenLabelDisputeStatus
    );
    require!(
        green_label_dispute.action_type != ActionType::GreenLabelRefund,
        CustomError::GreenLabelForfeitDecisionMismatch
    );

    let forfeit_amount = derive_green_label_forfeitable_amount_v1(escrow)?;
    validate_green_label_forfeit_vault_balance_v1(vault_balance, forfeit_amount)?;

    require!(
        parameters.schema_version == GREEN_LABEL_FORFEIT_SCHEMA_VERSION,
        CustomError::InvalidGreenLabelForfeitSchema
    );
    require_keys_eq!(
        parameters.green_label_config,
        green_label_config_key,
        CustomError::GreenLabelForfeitTargetMismatch
    );
    require_keys_eq!(
        parameters.green_label_project,
        green_label_project_key,
        CustomError::GreenLabelForfeitTargetMismatch
    );
    require_keys_eq!(
        parameters.green_label_dispute,
        green_label_dispute_key,
        CustomError::GreenLabelForfeitDisputeMismatch
    );
    require_keys_eq!(
        parameters.refundable_escrow,
        escrow_key,
        CustomError::GreenLabelForfeitTargetMismatch
    );
    require_keys_eq!(
        parameters.refundable_vault,
        refundable_vault_key,
        CustomError::GreenLabelForfeitVaultMismatch
    );
    require!(
        parameters.forfeited_amount_usdc == forfeit_amount,
        CustomError::GreenLabelForfeitAmountMismatch
    );
    require_keys_eq!(
        parameters.treasury_config,
        treasury_config_key,
        CustomError::GreenLabelForfeitTargetMismatch
    );
    require_keys_eq!(
        parameters.treasury_usdc_state,
        treasury_usdc_state_key,
        CustomError::GreenLabelForfeitTargetMismatch
    );
    require_keys_eq!(
        parameters.revenue_routing_stats,
        revenue_routing_stats_key,
        CustomError::GreenLabelForfeitTargetMismatch
    );
    require_keys_eq!(
        parameters.relief_usdc_vault,
        relief_usdc_vault_key,
        CustomError::GreenLabelForfeitVaultMismatch
    );
    require_keys_eq!(
        parameters.buyback_usdc_vault,
        buyback_usdc_vault_key,
        CustomError::GreenLabelForfeitVaultMismatch
    );
    require_keys_eq!(
        parameters.builders_usdc_vault,
        builders_usdc_vault_key,
        CustomError::GreenLabelForfeitVaultMismatch
    );
    require_keys_eq!(
        parameters.staking_usdc_vault,
        staking_usdc_vault_key,
        CustomError::GreenLabelForfeitVaultMismatch
    );
    require_keys_eq!(
        parameters.usdc_mint,
        escrow.usdc_mint,
        CustomError::GreenLabelForfeitMintMismatch
    );
    require!(
        parameters.revenue_type == RevenueType::GreenLabelForfeitedBond,
        CustomError::GreenLabelForfeitActionMismatch
    );
    require!(
        parameters.expected_escrow_status == escrow.status,
        CustomError::GreenLabelForfeitParametersMismatch
    );
    require!(
        parameters.expected_project_status == green_label_project.status,
        CustomError::GreenLabelForfeitParametersMismatch
    );
    require!(
        parameters.expected_dispute_status == green_label_dispute.status,
        CustomError::GreenLabelForfeitParametersMismatch
    );
    require!(
        parameters.action_type == GovernanceActionTypeV1::GreenLabelSlashBond,
        CustomError::GreenLabelForfeitActionMismatch
    );
    require!(
        parameters.proposal_id == governance_proposal.proposal_id,
        CustomError::InvalidProposalId
    );

    let parameters_hash = hash_green_label_forfeit_parameters_v1(parameters)?;
    require!(
        governance_proposal_action.parameters_hash == parameters_hash,
        CustomError::GreenLabelForfeitParametersMismatch
    );

    validate_green_label_treasury_router_accounts(
        escrow.usdc_mint,
        treasury_config_key,
        treasury_config.usdc_mint,
        treasury_usdc_state_key,
        revenue_routing_stats_key,
        revenue_routing_stats.usdc_mint,
        vault_authority_key,
        relief_usdc_vault_key,
        relief_usdc_vault_mint,
        relief_usdc_vault_owner,
        buyback_usdc_vault_key,
        buyback_usdc_vault_mint,
        buyback_usdc_vault_owner,
        builders_usdc_vault_key,
        builders_usdc_vault_mint,
        builders_usdc_vault_owner,
        staking_usdc_vault_key,
        staking_usdc_vault_mint,
        staking_usdc_vault_owner,
    )?;
    require_keys_eq!(
        green_label_config.treasury_usdc_state_v2,
        treasury_usdc_state_key,
        CustomError::GreenLabelForfeitTargetMismatch
    );
    require_keys_eq!(
        green_label_config.vault_authority_v2,
        vault_authority_key,
        CustomError::GreenLabelForfeitVaultMismatch
    );
    require_keys_eq!(
        revenue_routing_stats.authority,
        treasury_config.authority,
        CustomError::UnauthorizedTreasuryAuthority
    );
    require_keys_eq!(
        treasury_config.usdc_mint,
        escrow.usdc_mint,
        CustomError::GreenLabelForfeitMintMismatch
    );
    let _ = treasury_usdc_state;

    require_keys_eq!(
        governance_decision_adapter.governance_proposal,
        governance_proposal_key,
        CustomError::InvalidGovernanceDecisionAdapter
    );
    require_keys_eq!(
        governance_decision_adapter.proposal_decision,
        proposal_decision_key,
        CustomError::InvalidGovernanceDecisionAdapter
    );
    require!(
        governance_decision_adapter.action_type == ActionType::GreenLabelSlash,
        CustomError::GreenLabelForfeitActionMismatch
    );
    require_keys_eq!(
        governance_decision_adapter.target_program,
        governance_proposal_action.target_program,
        CustomError::GreenLabelForfeitTargetMismatch
    );
    require_keys_eq!(
        governance_decision_adapter.target_account,
        escrow_key,
        CustomError::GreenLabelForfeitTargetMismatch
    );
    require!(
        governance_decision_adapter.payload_hash
            == governance_proposal_action.canonical_payload_hash,
        CustomError::GreenLabelForfeitParametersMismatch
    );
    require!(
        governance_decision_adapter_key != Pubkey::default(),
        CustomError::InvalidGovernanceDecisionAdapter
    );
    require!(
        proposal_decision.proposal_id == governance_proposal.proposal_id,
        CustomError::InvalidProposalId
    );
    require!(
        proposal_decision.proposal_type == ProposalType::GreenLabelSlash,
        CustomError::InvalidActionForProposalType
    );
    require!(
        proposal_decision.decision == ProposalDecision::Approved,
        CustomError::ProposalNotApproved
    );
    require!(
        execution_queue_item.proposal_id == governance_proposal.proposal_id,
        CustomError::InvalidProposalId
    );
    require!(
        execution_queue_item.status == ExecutionStatus::Executed,
        CustomError::InvalidExecutionStatus
    );
    require!(
        execution_queue_item.executed_at > 0 && execution_queue_item.executed_at <= now,
        CustomError::InvalidExecutionStatus
    );
    require!(
        execution_queue_item.decision == ProposalDecision::Approved,
        CustomError::ProposalNotApproved
    );
    require!(
        execution_queue_item.action_type == ActionType::GreenLabelSlash,
        CustomError::GreenLabelForfeitActionMismatch
    );
    require_keys_eq!(
        execution_queue_item.target_program,
        crate::ID,
        CustomError::GreenLabelForfeitTargetMismatch
    );
    require_keys_eq!(
        execution_queue_item.target_account,
        escrow_key,
        CustomError::GreenLabelForfeitTargetMismatch
    );
    require!(
        execution_queue_item.payload_hash == governance_proposal_action.canonical_payload_hash,
        CustomError::GreenLabelForfeitParametersMismatch
    );
    require!(
        execution_queue_item_key != Pubkey::default(),
        CustomError::InvalidGreenLabelExecutionQueue
    );

    Ok(())
}

pub fn validate_green_label_refund_vault_balance_v1(
    vault_balance: u64,
    refund_amount: u64,
) -> Result<()> {
    require!(
        vault_balance >= refund_amount,
        CustomError::GreenLabelRefundInsufficientFunds
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn validate_green_label_refund_execution_context_v1(
    security_governance_config: &GovernanceConfigV1,
    security_governance_config_key: Pubkey,
    green_label_config: &GreenLabelConfigV1,
    green_label_config_key: Pubkey,
    green_label_project: &GreenLabelProjectV1,
    green_label_project_key: Pubkey,
    green_label_dispute: Option<&GreenLabelDisputeV1>,
    escrow: &GreenLabelRefundableEscrowV1,
    escrow_key: Pubkey,
    protocol_module_registry: &ProtocolModuleRegistryV1,
    protocol_module_registry_key: Pubkey,
    governance_proposal: &GovernanceProposalV1,
    governance_proposal_key: Pubkey,
    governance_proposal_action: &GovernanceProposalActionV1,
    governance_proposal_action_key: Pubkey,
    governance_decision_adapter: &UniversalGovernanceDecisionAdapterV1,
    governance_decision_adapter_key: Pubkey,
    proposal_decision: &ProposalDecisionV1,
    proposal_decision_key: Pubkey,
    execution_queue_item: &ExecutionQueueItemV1,
    execution_queue_item_key: Pubkey,
    refundable_vault_key: Pubkey,
    refundable_vault_mint: Pubkey,
    refundable_vault_owner: Pubkey,
    payer_destination_token_account: Pubkey,
    payer_destination_owner: Pubkey,
    payer_destination_mint: Pubkey,
    provided_usdc_mint: Pubkey,
    usdc_decimals: u8,
    vault_balance: u64,
    requires_dispute: bool,
    now: i64,
    parameters: &GreenLabelRefundParametersV1,
) -> Result<()> {
    require!(
        !green_label_config.is_paused,
        CustomError::InvalidGreenLabelStatus
    );
    require!(
        !security_governance_config.is_paused,
        CustomError::SecurityLayerPaused
    );
    require_keys_eq!(
        green_label_config.security_governance_config,
        security_governance_config_key,
        CustomError::ProtocolModuleGovernanceConfigMismatch
    );
    validate_protocol_module_registry_v1(
        protocol_module_registry,
        protocol_module_registry_key,
        security_governance_config_key,
        ProtocolModuleIdV1::GreenLabel,
        crate::ID,
    )?;
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
        governance_proposal_action.action_type == GovernanceActionTypeV1::GreenLabelRefundBond,
        CustomError::GreenLabelRefundActionMismatch
    );
    require!(
        governance_proposal_action.module_id == ProtocolModuleIdV1::GreenLabel,
        CustomError::GreenLabelRefundActionMismatch
    );
    require_keys_eq!(
        governance_proposal_action.target_program,
        crate::ID,
        CustomError::GreenLabelRefundTargetMismatch
    );
    require_keys_eq!(
        governance_proposal_action.target_account,
        escrow_key,
        CustomError::GreenLabelRefundTargetMismatch
    );
    require!(
        map_governance_action_to_security_action(governance_proposal_action.action_type)?
            == ActionType::GreenLabelRefund,
        CustomError::GreenLabelRefundActionMismatch
    );
    require_keys_eq!(
        escrow.project,
        green_label_project_key,
        CustomError::GreenLabelRefundTargetMismatch
    );
    require!(
        escrow.project_id == green_label_project.project_id,
        CustomError::GreenLabelRefundTargetMismatch
    );
    require_keys_eq!(
        escrow.usdc_mint,
        green_label_config.usdc_mint,
        CustomError::GreenLabelRefundMintMismatch
    );
    require_keys_eq!(
        escrow.refundable_vault,
        refundable_vault_key,
        CustomError::GreenLabelRefundVaultMismatch
    );
    require_keys_eq!(
        refundable_vault_mint,
        escrow.usdc_mint,
        CustomError::GreenLabelRefundMintMismatch
    );
    require_keys_eq!(
        refundable_vault_owner,
        escrow_key,
        CustomError::GreenLabelRefundVaultMismatch
    );
    require_keys_eq!(
        payer_destination_owner,
        escrow.payer,
        CustomError::GreenLabelRefundWrongDestination
    );
    require_keys_eq!(
        payer_destination_mint,
        escrow.usdc_mint,
        CustomError::GreenLabelRefundMintMismatch
    );
    require!(
        payer_destination_token_account != refundable_vault_key,
        CustomError::GreenLabelRefundWrongDestination
    );
    require_keys_eq!(
        provided_usdc_mint,
        escrow.usdc_mint,
        CustomError::GreenLabelRefundMintMismatch
    );
    require!(
        usdc_decimals == GREEN_LABEL_USDC_DECIMALS,
        CustomError::GreenLabelRefundMintMismatch
    );

    let refund_amount = derive_green_label_refund_amount_v1(escrow)?;
    validate_green_label_refund_vault_balance_v1(vault_balance, refund_amount)?;
    require!(
        parameters.schema_version == GREEN_LABEL_REFUND_SCHEMA_VERSION,
        CustomError::InvalidGreenLabelRefundSchema
    );
    require_keys_eq!(
        parameters.green_label_config,
        green_label_config_key,
        CustomError::GreenLabelRefundTargetMismatch
    );
    require_keys_eq!(
        parameters.green_label_project,
        green_label_project_key,
        CustomError::GreenLabelRefundTargetMismatch
    );
    require_keys_eq!(
        parameters.refundable_escrow,
        escrow_key,
        CustomError::GreenLabelRefundTargetMismatch
    );
    require_keys_eq!(
        parameters.refundable_vault,
        refundable_vault_key,
        CustomError::GreenLabelRefundVaultMismatch
    );
    require_keys_eq!(
        parameters.original_payer,
        escrow.payer,
        CustomError::GreenLabelRefundWrongPayer
    );
    require_keys_eq!(
        parameters.payer_destination_token_account,
        payer_destination_token_account,
        CustomError::GreenLabelRefundWrongDestination
    );
    require!(
        parameters.refund_amount_usdc == refund_amount,
        CustomError::GreenLabelRefundAmountMismatch
    );
    require_keys_eq!(
        parameters.usdc_mint,
        escrow.usdc_mint,
        CustomError::GreenLabelRefundMintMismatch
    );
    require!(
        parameters.expected_escrow_status == escrow.status,
        CustomError::GreenLabelRefundParametersMismatch
    );
    require!(
        parameters.proposal_id == governance_proposal.proposal_id,
        CustomError::InvalidProposalId
    );
    require!(
        parameters.action_type == GovernanceActionTypeV1::GreenLabelRefundBond,
        CustomError::GreenLabelRefundActionMismatch
    );

    if requires_dispute {
        let dispute = green_label_dispute
            .ok_or_else(|| error!(CustomError::GreenLabelRefundTargetMismatch))?;
        require_keys_neq!(
            parameters.green_label_dispute,
            Pubkey::default(),
            CustomError::GreenLabelRefundTargetMismatch
        );
        require_keys_eq!(
            dispute.project,
            green_label_project_key,
            CustomError::GreenLabelRefundTargetMismatch
        );
        require_keys_eq!(
            green_label_project.active_dispute,
            parameters.green_label_dispute,
            CustomError::InvalidGreenLabelActiveDispute
        );
        require!(
            matches!(
                green_label_project.status,
                GreenLabelStatus::Disputed | GreenLabelStatus::RefundQueued
            ),
            CustomError::InvalidGreenLabelStatus
        );
        require!(
            matches!(
                dispute.status,
                DisputeStatus::ReadyForDecision | DisputeStatus::DecisionQueued
            ),
            CustomError::InvalidGreenLabelDisputeStatus
        );
        require!(
            dispute.action_type != ActionType::GreenLabelSlash,
            CustomError::GreenLabelRefundActionMismatch
        );
    } else {
        require_keys_eq!(
            parameters.green_label_dispute,
            Pubkey::default(),
            CustomError::GreenLabelRefundTargetMismatch
        );
        require_keys_eq!(
            green_label_project.active_dispute,
            Pubkey::default(),
            CustomError::GreenLabelUnresolvedDispute
        );
        require!(
            now >= escrow.refund_available_after,
            CustomError::GreenLabelRefundNotEligible
        );
        require!(
            !matches!(
                green_label_project.status,
                GreenLabelStatus::Disputed
                    | GreenLabelStatus::Refunded
                    | GreenLabelStatus::Slashed
                    | GreenLabelStatus::Cancelled
            ),
            CustomError::InvalidGreenLabelStatus
        );
    }

    let parameters_hash = hash_green_label_refund_parameters_v1(parameters)?;
    require!(
        governance_proposal_action.parameters_hash == parameters_hash,
        CustomError::GreenLabelRefundParametersMismatch
    );
    require_keys_eq!(
        governance_decision_adapter.governance_proposal,
        governance_proposal_key,
        CustomError::InvalidGovernanceDecisionAdapter
    );
    require_keys_eq!(
        governance_decision_adapter.proposal_decision,
        proposal_decision_key,
        CustomError::InvalidGovernanceDecisionAdapter
    );
    require!(
        governance_decision_adapter.action_type == ActionType::GreenLabelRefund,
        CustomError::GreenLabelRefundActionMismatch
    );
    require_keys_eq!(
        governance_decision_adapter.target_program,
        governance_proposal_action.target_program,
        CustomError::GreenLabelRefundTargetMismatch
    );
    require_keys_eq!(
        governance_decision_adapter.target_account,
        escrow_key,
        CustomError::GreenLabelRefundTargetMismatch
    );
    require!(
        governance_decision_adapter.payload_hash
            == governance_proposal_action.canonical_payload_hash,
        CustomError::GreenLabelRefundParametersMismatch
    );
    require!(
        governance_decision_adapter_key != Pubkey::default(),
        CustomError::InvalidGovernanceDecisionAdapter
    );
    require!(
        proposal_decision.proposal_id == governance_proposal.proposal_id,
        CustomError::InvalidProposalId
    );
    require!(
        proposal_decision.proposal_type == ProposalType::GreenLabelRefund,
        CustomError::InvalidActionForProposalType
    );
    require!(
        proposal_decision.decision == ProposalDecision::Approved,
        CustomError::ProposalNotApproved
    );
    require!(
        execution_queue_item.proposal_id == governance_proposal.proposal_id,
        CustomError::InvalidProposalId
    );
    require!(
        execution_queue_item.status == ExecutionStatus::Executed,
        CustomError::InvalidExecutionStatus
    );
    require!(
        execution_queue_item.executed_at > 0,
        CustomError::InvalidExecutionStatus
    );
    require!(
        execution_queue_item.decision == ProposalDecision::Approved,
        CustomError::ProposalNotApproved
    );
    require!(
        execution_queue_item.action_type == ActionType::GreenLabelRefund,
        CustomError::GreenLabelRefundActionMismatch
    );
    require_keys_eq!(
        execution_queue_item.target_program,
        crate::ID,
        CustomError::GreenLabelRefundTargetMismatch
    );
    require_keys_eq!(
        execution_queue_item.target_account,
        escrow_key,
        CustomError::GreenLabelRefundTargetMismatch
    );
    require!(
        execution_queue_item.payload_hash == governance_proposal_action.canonical_payload_hash,
        CustomError::GreenLabelRefundParametersMismatch
    );
    require!(
        execution_queue_item_key != Pubkey::default(),
        CustomError::InvalidGreenLabelExecutionQueue
    );

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn execute_green_label_escrow_refund_internal_v1<'info>(
    escrow: &mut GreenLabelRefundableEscrowV1,
    escrow_info: AccountInfo<'info>,
    refundable_vault_info: AccountInfo<'info>,
    usdc_mint_info: AccountInfo<'info>,
    payer_refund_usdc_account_info: AccountInfo<'info>,
    token_program: Pubkey,
    project_key: Pubkey,
    refund_amount: u64,
) -> Result<()> {
    let escrow_bump = escrow.bump;
    let signer_seeds: &[&[&[u8]]] = &[&[
        GREEN_LABEL_REFUNDABLE_ESCROW_SEED,
        project_key.as_ref(),
        &[escrow_bump],
    ]];
    let cpi_accounts = TransferChecked {
        from: refundable_vault_info,
        mint: usdc_mint_info,
        to: payer_refund_usdc_account_info,
        authority: escrow_info,
    };
    let cpi_ctx = CpiContext::new_with_signer(token_program, cpi_accounts, signer_seeds);
    transfer_checked(cpi_ctx, refund_amount, GREEN_LABEL_USDC_DECIMALS)?;
    record_green_label_escrow_refunded(escrow, refund_amount)
}

#[allow(clippy::too_many_arguments)]
pub fn record_green_label_refund_governance_v1(
    project: &mut GreenLabelProjectV1,
    dispute: Option<&mut GreenLabelDisputeV1>,
    execution_record: &mut GreenLabelRefundExecutionRecordV1,
    execution_queue_item: Pubkey,
    proposal_decision: Pubkey,
    governance_proposal: Pubkey,
    governance_proposal_action: Pubkey,
    module_registry: Pubkey,
    green_label_config: Pubkey,
    green_label_project: Pubkey,
    green_label_dispute: Pubkey,
    refundable_escrow: Pubkey,
    refundable_vault: Pubkey,
    original_payer: Pubkey,
    payer_destination_token_account: Pubkey,
    refund_amount_usdc: u64,
    usdc_mint: Pubkey,
    parameters: GreenLabelRefundParametersV1,
    canonical_governance_payload_hash: [u8; 32],
    escrow_status_before: GreenLabelEscrowStatusV1,
    project_status_before: GreenLabelStatus,
    executor: Pubkey,
    executed_at: i64,
    bump: u8,
    execution_record_key: Pubkey,
) -> Result<()> {
    require!(
        execution_record.execution_queue_item == Pubkey::default(),
        CustomError::GreenLabelRefundExecutionAlreadyCompleted
    );
    let parameters_hash = hash_green_label_refund_parameters_v1(&parameters)?;

    project.status = GreenLabelStatus::Refunded;
    project.active_dispute = Pubkey::default();
    project.refunded_at = executed_at;
    project.terminal_proposal_id = parameters.proposal_id;
    project.terminal_proposal_decision = proposal_decision;
    project.terminal_execution_queue_item = execution_queue_item;
    project.terminal_payload_hash = canonical_governance_payload_hash;
    project.terminal_action_type = ActionType::GreenLabelRefund;

    if let Some(dispute) = dispute {
        dispute.status = DisputeStatus::ResolvedRefund;
        dispute.resolved_at = executed_at;
        dispute.proposal_id = parameters.proposal_id;
        dispute.proposal_decision = proposal_decision;
        dispute.execution_queue_item = execution_queue_item;
        dispute.payload_hash = canonical_governance_payload_hash;
        dispute.action_type = ActionType::GreenLabelRefund;
    }

    execution_record.execution_queue_item = execution_queue_item;
    execution_record.proposal_decision = proposal_decision;
    execution_record.governance_proposal = governance_proposal;
    execution_record.governance_proposal_action = governance_proposal_action;
    execution_record.module_registry = module_registry;
    execution_record.green_label_config = green_label_config;
    execution_record.green_label_project = green_label_project;
    execution_record.green_label_dispute = green_label_dispute;
    execution_record.refundable_escrow = refundable_escrow;
    execution_record.refundable_vault = refundable_vault;
    execution_record.original_payer = original_payer;
    execution_record.payer_destination_token_account = payer_destination_token_account;
    execution_record.refund_amount_usdc = refund_amount_usdc;
    execution_record.usdc_mint = usdc_mint;
    execution_record.execution_type = GreenLabelEscrowExecutionTypeV1::Refund;
    execution_record.governance_action_type = GovernanceActionTypeV1::GreenLabelRefundBond;
    execution_record.parameters_hash = parameters_hash;
    execution_record.canonical_governance_payload_hash = canonical_governance_payload_hash;
    execution_record.escrow_status_before = escrow_status_before;
    execution_record.escrow_status_after = GreenLabelEscrowStatusV1::Refunded;
    execution_record.project_status_before = project_status_before;
    execution_record.project_status_after = GreenLabelStatus::Refunded;
    execution_record.executor = executor;
    execution_record.executed_at = executed_at;
    execution_record.schema_version = GREEN_LABEL_REFUND_SCHEMA_VERSION;
    execution_record.bump = bump;

    require_keys_neq!(
        execution_record_key,
        Pubkey::default(),
        CustomError::GreenLabelRefundExecutionAlreadyCompleted
    );

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn record_green_label_forfeit_governance_v1(
    project: &mut GreenLabelProjectV1,
    dispute: &mut GreenLabelDisputeV1,
    escrow: &mut GreenLabelRefundableEscrowV1,
    execution_record: &mut GreenLabelForfeitExecutionRecordV1,
    execution_queue_item: Pubkey,
    proposal_decision: Pubkey,
    governance_proposal: Pubkey,
    governance_proposal_action: Pubkey,
    module_registry: Pubkey,
    green_label_config: Pubkey,
    green_label_project: Pubkey,
    green_label_dispute: Pubkey,
    refundable_escrow: Pubkey,
    refundable_vault: Pubkey,
    treasury_config: Pubkey,
    treasury_usdc_state: Pubkey,
    revenue_routing_stats: Pubkey,
    relief_usdc_vault: Pubkey,
    buyback_usdc_vault: Pubkey,
    builders_usdc_vault: Pubkey,
    staking_usdc_vault: Pubkey,
    forfeited_amount_usdc: u64,
    usdc_mint: Pubkey,
    parameters: GreenLabelForfeitParametersV1,
    canonical_governance_payload_hash: [u8; 32],
    escrow_status_before: GreenLabelEscrowStatusV1,
    project_status_before: GreenLabelStatus,
    dispute_status_before: DisputeStatus,
    executor: Pubkey,
    executed_at: i64,
    bump: u8,
    execution_record_key: Pubkey,
) -> Result<()> {
    require!(
        execution_record.execution_queue_item == Pubkey::default(),
        CustomError::GreenLabelForfeitExecutionAlreadyCompleted
    );
    require!(
        forfeited_amount_usdc == derive_green_label_forfeitable_amount_v1(escrow)?,
        CustomError::GreenLabelForfeitAmountMismatch
    );
    let parameters_hash = hash_green_label_forfeit_parameters_v1(&parameters)?;
    require!(
        parameters.forfeited_amount_usdc == forfeited_amount_usdc,
        CustomError::GreenLabelForfeitAmountMismatch
    );

    record_green_label_escrow_forfeited(escrow, forfeited_amount_usdc)?;

    project.status = GreenLabelStatus::Slashed;
    project.slashed_at = executed_at;
    project.active_dispute = Pubkey::default();
    project.terminal_proposal_id = parameters.proposal_id;
    project.terminal_proposal_decision = proposal_decision;
    project.terminal_execution_queue_item = execution_queue_item;
    project.terminal_payload_hash = canonical_governance_payload_hash;
    project.terminal_action_type = ActionType::GreenLabelSlash;

    dispute.status = DisputeStatus::ResolvedSlash;
    dispute.resolved_at = executed_at;
    dispute.proposal_id = parameters.proposal_id;
    dispute.proposal_decision = proposal_decision;
    dispute.execution_queue_item = execution_queue_item;
    dispute.payload_hash = canonical_governance_payload_hash;
    dispute.action_type = ActionType::GreenLabelSlash;

    execution_record.execution_queue_item = execution_queue_item;
    execution_record.proposal_decision = proposal_decision;
    execution_record.governance_proposal = governance_proposal;
    execution_record.governance_proposal_action = governance_proposal_action;
    execution_record.module_registry = module_registry;
    execution_record.green_label_config = green_label_config;
    execution_record.green_label_project = green_label_project;
    execution_record.green_label_dispute = green_label_dispute;
    execution_record.refundable_escrow = refundable_escrow;
    execution_record.refundable_vault = refundable_vault;
    execution_record.treasury_config = treasury_config;
    execution_record.treasury_usdc_state = treasury_usdc_state;
    execution_record.revenue_routing_stats = revenue_routing_stats;
    execution_record.relief_usdc_vault = relief_usdc_vault;
    execution_record.buyback_usdc_vault = buyback_usdc_vault;
    execution_record.builders_usdc_vault = builders_usdc_vault;
    execution_record.staking_usdc_vault = staking_usdc_vault;
    execution_record.forfeited_amount_usdc = forfeited_amount_usdc;
    execution_record.usdc_mint = usdc_mint;
    execution_record.revenue_type = RevenueType::GreenLabelForfeitedBond;
    execution_record.execution_type = GreenLabelEscrowExecutionTypeV1::Forfeit;
    execution_record.governance_action_type = GovernanceActionTypeV1::GreenLabelSlashBond;
    execution_record.parameters_hash = parameters_hash;
    execution_record.canonical_governance_payload_hash = canonical_governance_payload_hash;
    execution_record.escrow_status_before = escrow_status_before;
    execution_record.escrow_status_after = GreenLabelEscrowStatusV1::Forfeited;
    execution_record.project_status_before = project_status_before;
    execution_record.project_status_after = GreenLabelStatus::Slashed;
    execution_record.dispute_status_before = dispute_status_before;
    execution_record.dispute_status_after = DisputeStatus::ResolvedSlash;
    execution_record.executor = executor;
    execution_record.executed_at = executed_at;
    execution_record.schema_version = GREEN_LABEL_FORFEIT_SCHEMA_VERSION;
    execution_record.bump = bump;

    require_keys_neq!(
        execution_record_key,
        Pubkey::default(),
        CustomError::GreenLabelForfeitExecutionAlreadyCompleted
    );

    Ok(())
}

pub fn validate_green_label_window_update(
    config_is_paused: bool,
    expected_authority: Pubkey,
    signer: Pubkey,
    observation_period_seconds: i64,
    dispute_window_seconds: i64,
    response_window_seconds: i64,
) -> Result<()> {
    require!(!config_is_paused, CustomError::InvalidGreenLabelStatus);
    require_keys_eq!(
        expected_authority,
        signer,
        CustomError::UnauthorizedGreenLabelAuthority
    );
    require!(
        observation_period_seconds > 0
            && observation_period_seconds <= MAX_GREEN_LABEL_WINDOW_SECONDS,
        CustomError::InvalidGreenLabelWindowConfig
    );
    require!(
        dispute_window_seconds > 0 && dispute_window_seconds <= MAX_GREEN_LABEL_WINDOW_SECONDS,
        CustomError::InvalidGreenLabelWindowConfig
    );
    require!(
        response_window_seconds > 0 && response_window_seconds <= MAX_GREEN_LABEL_WINDOW_SECONDS,
        CustomError::InvalidGreenLabelWindowConfig
    );

    Ok(())
}

pub fn record_green_label_window_update(
    green_label_config: &mut GreenLabelConfigV1,
    observation_period_seconds: i64,
    dispute_window_seconds: i64,
    response_window_seconds: i64,
) {
    green_label_config.observation_period_seconds = observation_period_seconds;
    green_label_config.dispute_window_seconds = dispute_window_seconds;
    green_label_config.response_window_seconds = response_window_seconds;
}

pub fn validate_green_label_min_base_bond_update(
    config_is_paused: bool,
    expected_authority: Pubkey,
    signer: Pubkey,
    min_base_bond_usdc: u64,
) -> Result<()> {
    require!(!config_is_paused, CustomError::InvalidGreenLabelStatus);
    require_keys_eq!(
        expected_authority,
        signer,
        CustomError::UnauthorizedGreenLabelAuthority
    );
    require!(
        min_base_bond_usdc > 0 && min_base_bond_usdc <= MIN_GREEN_LABEL_BASE_BOND_USDC,
        CustomError::InvalidGreenLabelBondAmount
    );

    Ok(())
}

pub fn record_green_label_min_base_bond_update(
    green_label_config: &mut GreenLabelConfigV1,
    min_base_bond_usdc: u64,
) {
    green_label_config.min_base_bond_usdc = min_base_bond_usdc;
}

pub fn validate_green_bond_vault_initialization(
    config_is_paused: bool,
    project_owner: Pubkey,
    signer: Pubkey,
    project_status: GreenLabelStatus,
    existing_bond_vault: Pubkey,
    existing_bond_vault_authority: Pubkey,
    expected_usdc_mint: Pubkey,
    provided_usdc_mint: Pubkey,
) -> Result<()> {
    require!(!config_is_paused, CustomError::InvalidGreenLabelStatus);
    require_keys_eq!(
        project_owner,
        signer,
        CustomError::InvalidGreenLabelProjectOwner
    );
    require!(
        project_status == GreenLabelStatus::PendingBondDeposit,
        CustomError::InvalidGreenLabelStatus
    );
    require!(
        existing_bond_vault == Pubkey::default(),
        CustomError::InvalidGreenLabelBondVaultState
    );
    require!(
        existing_bond_vault_authority == Pubkey::default(),
        CustomError::InvalidGreenLabelBondVaultState
    );
    require_keys_eq!(
        expected_usdc_mint,
        provided_usdc_mint,
        CustomError::InvalidGreenLabelMint
    );

    Ok(())
}

pub fn validate_green_label_bond_lock(
    config_is_paused: bool,
    project_owner: Pubkey,
    signer: Pubkey,
    project_status: GreenLabelStatus,
    bond_vault: Pubkey,
    bond_vault_authority: Pubkey,
    provided_bond_vault: Pubkey,
    provided_bond_vault_mint: Pubkey,
    provided_bond_vault_owner: Pubkey,
    expected_usdc_mint: Pubkey,
    owner_ata_owner: Pubkey,
    owner_ata_mint: Pubkey,
    usdc_mint: Pubkey,
    base_bond_amount: u64,
    extra_bond_amount: u64,
    total_bond_amount: u64,
) -> Result<()> {
    require!(!config_is_paused, CustomError::InvalidGreenLabelStatus);
    require_keys_eq!(
        project_owner,
        signer,
        CustomError::InvalidGreenLabelProjectOwner
    );
    require!(
        project_status == GreenLabelStatus::PendingBondDeposit,
        CustomError::InvalidGreenLabelStatus
    );
    require!(
        bond_vault != Pubkey::default(),
        CustomError::InvalidGreenLabelBondVaultState
    );
    require!(
        bond_vault_authority != Pubkey::default(),
        CustomError::InvalidGreenLabelBondVaultState
    );
    require_keys_eq!(
        provided_bond_vault,
        bond_vault,
        CustomError::InvalidGreenLabelBondVaultState
    );
    require_keys_eq!(
        provided_bond_vault_mint,
        expected_usdc_mint,
        CustomError::InvalidGreenLabelMint
    );
    require_keys_eq!(
        provided_bond_vault_owner,
        bond_vault_authority,
        CustomError::InvalidGreenLabelBondVaultState
    );
    require_keys_eq!(
        owner_ata_owner,
        signer,
        CustomError::InvalidGreenLabelTokenAccount
    );
    require_keys_eq!(
        owner_ata_mint,
        expected_usdc_mint,
        CustomError::InvalidGreenLabelMint
    );
    require_keys_eq!(
        usdc_mint,
        expected_usdc_mint,
        CustomError::InvalidGreenLabelMint
    );
    require!(
        base_bond_amount > 0,
        CustomError::InvalidGreenLabelBondAmount
    );
    let expected_total_bond_amount = base_bond_amount
        .checked_add(extra_bond_amount)
        .ok_or(CustomError::GreenLabelMathOverflow)?;
    require!(
        total_bond_amount == expected_total_bond_amount,
        CustomError::InvalidGreenLabelBondAmount
    );

    Ok(())
}

pub fn build_observation_window(now: i64, observation_period_seconds: i64) -> Result<(i64, i64)> {
    let observation_end_ts = now
        .checked_add(observation_period_seconds)
        .ok_or(CustomError::GreenLabelMathOverflow)?;

    Ok((now, observation_end_ts))
}

pub fn record_green_label_bond_locked(
    project: &mut GreenLabelProjectV1,
    now: i64,
    observation_end_ts: i64,
) -> Result<()> {
    project.status = GreenLabelStatus::PendingObservation;
    project.observation_start_ts = now;
    project.observation_end_ts = observation_end_ts;

    Ok(())
}

pub fn validate_open_green_label_dispute(
    config_is_paused: bool,
    project_status: GreenLabelStatus,
    active_dispute: Pubkey,
    current_dispute_count: u64,
    expected_dispute_id: u64,
    evidence_hash: [u8; 32],
) -> Result<()> {
    require!(!config_is_paused, CustomError::InvalidGreenLabelStatus);
    require!(
        matches!(
            project_status,
            GreenLabelStatus::PendingObservation | GreenLabelStatus::ActiveGreenLabel
        ),
        CustomError::InvalidGreenLabelStatus
    );
    require!(
        active_dispute == Pubkey::default(),
        CustomError::InvalidGreenLabelActiveDispute
    );

    let next_dispute_id = current_dispute_count
        .checked_add(1)
        .ok_or(CustomError::GreenLabelMathOverflow)?;
    require!(
        expected_dispute_id == next_dispute_id,
        CustomError::InvalidGreenLabelDisputeId
    );
    require!(
        evidence_hash != [0; 32],
        CustomError::InvalidGreenLabelEvidenceHash
    );

    Ok(())
}

pub fn build_dispute_windows(
    now: i64,
    dispute_window_seconds: i64,
    response_window_seconds: i64,
) -> Result<(i64, i64)> {
    let evidence_end_ts = now
        .checked_add(dispute_window_seconds)
        .ok_or(CustomError::GreenLabelMathOverflow)?;
    let response_end_ts = evidence_end_ts
        .checked_add(response_window_seconds)
        .ok_or(CustomError::GreenLabelMathOverflow)?;

    Ok((evidence_end_ts, response_end_ts))
}

pub fn record_green_label_dispute_opened(
    project: &mut GreenLabelProjectV1,
    dispute_key: Pubkey,
    expected_dispute_id: u64,
) -> Result<()> {
    project.status = GreenLabelStatus::Disputed;
    project.active_dispute = dispute_key;
    project.dispute_count = expected_dispute_id;

    Ok(())
}

pub fn validate_mark_dispute_ready(
    config_is_paused: bool,
    project_status: GreenLabelStatus,
    project_active_dispute: Pubkey,
    dispute_key: Pubkey,
    dispute_project: Pubkey,
    project_key: Pubkey,
    dispute_status: DisputeStatus,
    now: i64,
    response_end_ts: i64,
) -> Result<()> {
    require!(!config_is_paused, CustomError::InvalidGreenLabelStatus);
    require!(
        project_status == GreenLabelStatus::Disputed,
        CustomError::InvalidGreenLabelStatus
    );
    require_keys_eq!(
        project_active_dispute,
        dispute_key,
        CustomError::InvalidGreenLabelActiveDispute
    );
    require_keys_eq!(
        dispute_project,
        project_key,
        CustomError::InvalidGreenLabelTargetAccount
    );
    require!(
        matches!(
            dispute_status,
            DisputeStatus::EvidencePeriod | DisputeStatus::ProjectResponsePeriod
        ),
        CustomError::InvalidGreenLabelDisputeStatus
    );
    require!(
        now >= response_end_ts,
        CustomError::GreenLabelDisputeWindowNotEnded
    );

    Ok(())
}

pub fn record_dispute_ready_for_decision(dispute: &mut GreenLabelDisputeV1) -> Result<()> {
    dispute.status = DisputeStatus::ReadyForDecision;

    Ok(())
}

pub fn validate_green_label_security_decision_link(
    config_is_paused: bool,
    project_status: GreenLabelStatus,
    project_active_dispute: Pubkey,
    dispute_key: Pubkey,
    dispute_project: Pubkey,
    project_key: Pubkey,
    dispute_status: DisputeStatus,
    expected_proposal_id: u64,
    expected_action_type: ActionType,
    expected_payload_hash: [u8; 32],
    proposal_id: u64,
    proposal_type: ProposalType,
    proposal_decision: ProposalDecision,
    queue_proposal_id: u64,
    queue_action_type: ActionType,
    queue_status: ExecutionStatus,
    queue_payload_hash: [u8; 32],
    queue_target_program: Pubkey,
    expected_program_id: Pubkey,
    queue_target_account: Pubkey,
    expected_target_account: Pubkey,
) -> Result<()> {
    require!(!config_is_paused, CustomError::InvalidGreenLabelStatus);
    require!(
        project_status == GreenLabelStatus::Disputed,
        CustomError::InvalidGreenLabelStatus
    );
    require_keys_eq!(
        project_active_dispute,
        dispute_key,
        CustomError::InvalidGreenLabelActiveDispute
    );
    require_keys_eq!(
        dispute_project,
        project_key,
        CustomError::InvalidGreenLabelTargetAccount
    );
    require!(
        dispute_status == DisputeStatus::ReadyForDecision,
        CustomError::InvalidGreenLabelDisputeStatus
    );
    validate_payload_hash(expected_payload_hash)?;
    require!(
        matches!(
            expected_action_type,
            ActionType::GreenLabelSlash | ActionType::GreenLabelRefund
        ),
        CustomError::InvalidGreenLabelActionType
    );
    require!(
        proposal_id == expected_proposal_id,
        CustomError::InvalidGreenLabelSecurityDecision
    );
    require!(
        proposal_type_matches_action(proposal_type, expected_action_type),
        CustomError::InvalidGreenLabelSecurityDecision
    );
    require!(
        proposal_decision == ProposalDecision::Approved,
        CustomError::InvalidGreenLabelSecurityDecision
    );
    require!(
        queue_proposal_id == expected_proposal_id,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require!(
        queue_action_type == expected_action_type,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require!(
        queue_status == ExecutionStatus::Queued,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require!(
        queue_payload_hash == expected_payload_hash,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require_keys_eq!(
        queue_target_program,
        expected_program_id,
        CustomError::InvalidGreenLabelTargetProgram
    );
    require_keys_eq!(
        queue_target_account,
        expected_target_account,
        CustomError::InvalidGreenLabelTargetAccount
    );

    Ok(())
}

pub fn record_green_label_security_decision_link(
    project: &mut GreenLabelProjectV1,
    dispute: &mut GreenLabelDisputeV1,
    proposal_id: u64,
    proposal_decision_key: Pubkey,
    execution_queue_item_key: Pubkey,
    payload_hash: [u8; 32],
    action_type: ActionType,
) -> Result<()> {
    project.status = match action_type {
        ActionType::GreenLabelSlash => GreenLabelStatus::SlashQueued,
        ActionType::GreenLabelRefund => GreenLabelStatus::RefundQueued,
        _ => return err!(CustomError::InvalidGreenLabelActionType),
    };

    dispute.status = DisputeStatus::DecisionQueued;
    dispute.proposal_id = proposal_id;
    dispute.proposal_decision = proposal_decision_key;
    dispute.execution_queue_item = execution_queue_item_key;
    dispute.payload_hash = payload_hash;
    dispute.action_type = action_type;

    project.terminal_proposal_id = proposal_id;
    project.terminal_proposal_decision = proposal_decision_key;
    project.terminal_execution_queue_item = execution_queue_item_key;
    project.terminal_payload_hash = payload_hash;
    project.terminal_action_type = action_type;

    Ok(())
}

pub fn validate_green_label_refund_execution(
    config_is_paused: bool,
    project_status: GreenLabelStatus,
    project_active_dispute: Pubkey,
    dispute_key: Pubkey,
    project_bond_vault: Pubkey,
    project_bond_vault_authority: Pubkey,
    project_owner: Pubkey,
    project_terminal_proposal_id: u64,
    project_terminal_proposal_decision: Pubkey,
    project_terminal_execution_queue_item: Pubkey,
    project_terminal_payload_hash: [u8; 32],
    project_terminal_action_type: ActionType,
    dispute_project: Pubkey,
    project_key: Pubkey,
    dispute_status: DisputeStatus,
    dispute_proposal_id: u64,
    dispute_proposal_decision: Pubkey,
    dispute_execution_queue_item: Pubkey,
    dispute_payload_hash: [u8; 32],
    dispute_action_type: ActionType,
    proposal_decision_key: Pubkey,
    proposal_decision_proposal_id: u64,
    proposal_decision: ProposalDecision,
    execution_queue_item_key: Pubkey,
    queue_proposal_id: u64,
    queue_status: ExecutionStatus,
    queue_action_type: ActionType,
    queue_payload_hash: [u8; 32],
    queue_target_program: Pubkey,
    expected_program_id: Pubkey,
    queue_target_account: Pubkey,
    expected_target_account: Pubkey,
    now: i64,
    queue_execute_after: i64,
    provided_bond_vault: Pubkey,
    green_bond_vault_mint: Pubkey,
    green_bond_vault_owner: Pubkey,
    provided_bond_vault_authority: Pubkey,
    project_owner_ata_owner: Pubkey,
    project_owner_ata_mint: Pubkey,
    provided_treasury_vault: Pubkey,
    treasury_vault_mint: Pubkey,
    expected_treasury_vault: Pubkey,
    expected_usdc_mint: Pubkey,
    provided_usdc_mint: Pubkey,
    usdc_decimals: u8,
    vault_balance: u64,
    project_refund_amount: u64,
    treasury_amount: u64,
) -> Result<()> {
    require!(!config_is_paused, CustomError::InvalidGreenLabelStatus);
    require!(
        project_status == GreenLabelStatus::RefundQueued,
        CustomError::InvalidGreenLabelStatus
    );
    require_keys_eq!(
        project_active_dispute,
        dispute_key,
        CustomError::InvalidGreenLabelActiveDispute
    );
    require!(
        project_bond_vault != Pubkey::default()
            && project_bond_vault_authority != Pubkey::default(),
        CustomError::InvalidGreenLabelBondVaultState
    );
    require_keys_eq!(
        project_bond_vault,
        provided_bond_vault,
        CustomError::InvalidGreenLabelBondVaultState
    );
    require_keys_eq!(
        project_bond_vault_authority,
        provided_bond_vault_authority,
        CustomError::InvalidGreenLabelBondVaultState
    );
    require_keys_eq!(
        dispute_project,
        project_key,
        CustomError::InvalidGreenLabelTargetAccount
    );
    require!(
        dispute_status == DisputeStatus::DecisionQueued,
        CustomError::InvalidGreenLabelDisputeStatus
    );
    validate_terminal_action_for_refund(project_terminal_action_type)?;
    validate_payload_hash(project_terminal_payload_hash)?;
    require!(
        dispute_proposal_id == project_terminal_proposal_id,
        CustomError::InvalidGreenLabelSecurityDecision
    );
    require_keys_eq!(
        dispute_proposal_decision,
        project_terminal_proposal_decision,
        CustomError::InvalidGreenLabelSecurityDecision
    );
    require_keys_eq!(
        dispute_execution_queue_item,
        project_terminal_execution_queue_item,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require!(
        dispute_payload_hash == project_terminal_payload_hash,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require!(
        dispute_action_type == ActionType::GreenLabelRefund,
        CustomError::InvalidGreenLabelActionType
    );
    require_keys_eq!(
        proposal_decision_key,
        project_terminal_proposal_decision,
        CustomError::InvalidGreenLabelSecurityDecision
    );
    require_keys_eq!(
        execution_queue_item_key,
        project_terminal_execution_queue_item,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require!(
        proposal_decision_proposal_id == project_terminal_proposal_id,
        CustomError::InvalidGreenLabelSecurityDecision
    );
    require!(
        queue_proposal_id == project_terminal_proposal_id,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require!(
        proposal_decision == ProposalDecision::Approved,
        CustomError::InvalidGreenLabelSecurityDecision
    );
    require!(
        queue_status == ExecutionStatus::Queued,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require!(
        queue_action_type == ActionType::GreenLabelRefund,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require!(
        queue_payload_hash == project_terminal_payload_hash,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require_keys_eq!(
        queue_target_program,
        expected_program_id,
        CustomError::InvalidGreenLabelTargetProgram
    );
    require_keys_eq!(
        queue_target_account,
        expected_target_account,
        CustomError::InvalidGreenLabelTargetAccount
    );
    require!(
        now >= queue_execute_after,
        CustomError::GreenLabelTimelockNotSatisfied
    );
    require_keys_eq!(
        provided_usdc_mint,
        expected_usdc_mint,
        CustomError::InvalidGreenLabelMint
    );
    require!(
        usdc_decimals == GREEN_LABEL_USDC_DECIMALS,
        CustomError::InvalidGreenLabelMint
    );
    require_keys_eq!(
        green_bond_vault_mint,
        expected_usdc_mint,
        CustomError::InvalidGreenLabelTokenAccount
    );
    require_keys_eq!(
        green_bond_vault_owner,
        project_bond_vault_authority,
        CustomError::InvalidGreenLabelTokenAccount
    );
    require_keys_eq!(
        project_owner_ata_owner,
        project_owner,
        CustomError::InvalidGreenLabelTokenAccount
    );
    require_keys_eq!(
        project_owner_ata_mint,
        expected_usdc_mint,
        CustomError::InvalidGreenLabelTokenAccount
    );
    require_keys_eq!(
        provided_treasury_vault,
        expected_treasury_vault,
        CustomError::InvalidGreenLabelTokenAccount
    );
    require_keys_eq!(
        treasury_vault_mint,
        expected_usdc_mint,
        CustomError::InvalidGreenLabelTokenAccount
    );

    let required_vault_balance = project_refund_amount
        .checked_add(treasury_amount)
        .ok_or(CustomError::GreenLabelMathOverflow)?;
    require!(
        vault_balance >= required_vault_balance,
        CustomError::GreenLabelInsufficientBondVaultBalance
    );

    Ok(())
}

pub fn validate_green_label_slash_execution(
    config_is_paused: bool,
    project_status: GreenLabelStatus,
    project_active_dispute: Pubkey,
    dispute_key: Pubkey,
    project_bond_vault: Pubkey,
    project_bond_vault_authority: Pubkey,
    project_terminal_proposal_id: u64,
    project_terminal_proposal_decision: Pubkey,
    project_terminal_execution_queue_item: Pubkey,
    project_terminal_payload_hash: [u8; 32],
    project_terminal_action_type: ActionType,
    dispute_project: Pubkey,
    project_key: Pubkey,
    dispute_status: DisputeStatus,
    dispute_proposal_id: u64,
    dispute_proposal_decision: Pubkey,
    dispute_execution_queue_item: Pubkey,
    dispute_payload_hash: [u8; 32],
    dispute_action_type: ActionType,
    proposal_decision_key: Pubkey,
    proposal_decision_proposal_id: u64,
    proposal_decision: ProposalDecision,
    execution_queue_item_key: Pubkey,
    queue_proposal_id: u64,
    queue_status: ExecutionStatus,
    queue_action_type: ActionType,
    queue_payload_hash: [u8; 32],
    queue_target_program: Pubkey,
    expected_program_id: Pubkey,
    queue_target_account: Pubkey,
    expected_target_account: Pubkey,
    now: i64,
    queue_execute_after: i64,
    provided_bond_vault: Pubkey,
    green_bond_vault_mint: Pubkey,
    green_bond_vault_owner: Pubkey,
    provided_bond_vault_authority: Pubkey,
    provided_relief_or_risk_vault: Pubkey,
    relief_or_risk_vault_mint: Pubkey,
    expected_relief_or_risk_vault: Pubkey,
    expected_usdc_mint: Pubkey,
    provided_usdc_mint: Pubkey,
    usdc_decimals: u8,
    vault_balance: u64,
    slash_amount: u64,
) -> Result<()> {
    require!(!config_is_paused, CustomError::InvalidGreenLabelStatus);
    require!(
        project_status == GreenLabelStatus::SlashQueued,
        CustomError::InvalidGreenLabelStatus
    );
    require_keys_eq!(
        project_active_dispute,
        dispute_key,
        CustomError::InvalidGreenLabelActiveDispute
    );
    require!(
        project_bond_vault != Pubkey::default()
            && project_bond_vault_authority != Pubkey::default(),
        CustomError::InvalidGreenLabelBondVaultState
    );
    require_keys_eq!(
        project_bond_vault,
        provided_bond_vault,
        CustomError::InvalidGreenLabelBondVaultState
    );
    require_keys_eq!(
        project_bond_vault_authority,
        provided_bond_vault_authority,
        CustomError::InvalidGreenLabelBondVaultState
    );
    require_keys_eq!(
        dispute_project,
        project_key,
        CustomError::InvalidGreenLabelTargetAccount
    );
    require!(
        dispute_status == DisputeStatus::DecisionQueued,
        CustomError::InvalidGreenLabelDisputeStatus
    );
    validate_terminal_action_for_slash(project_terminal_action_type, true)?;
    validate_payload_hash(project_terminal_payload_hash)?;
    require!(
        dispute_proposal_id == project_terminal_proposal_id,
        CustomError::InvalidGreenLabelSecurityDecision
    );
    require_keys_eq!(
        dispute_proposal_decision,
        project_terminal_proposal_decision,
        CustomError::InvalidGreenLabelSecurityDecision
    );
    require_keys_eq!(
        dispute_execution_queue_item,
        project_terminal_execution_queue_item,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require!(
        dispute_payload_hash == project_terminal_payload_hash,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require!(
        dispute_action_type == ActionType::GreenLabelSlash,
        CustomError::InvalidGreenLabelActionType
    );
    require_keys_eq!(
        proposal_decision_key,
        project_terminal_proposal_decision,
        CustomError::InvalidGreenLabelSecurityDecision
    );
    require_keys_eq!(
        execution_queue_item_key,
        project_terminal_execution_queue_item,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require!(
        proposal_decision_proposal_id == project_terminal_proposal_id,
        CustomError::InvalidGreenLabelSecurityDecision
    );
    require!(
        queue_proposal_id == project_terminal_proposal_id,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require!(
        proposal_decision == ProposalDecision::Approved,
        CustomError::InvalidGreenLabelSecurityDecision
    );
    require!(
        queue_status == ExecutionStatus::Queued,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require!(
        queue_action_type == ActionType::GreenLabelSlash,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require!(
        queue_payload_hash == project_terminal_payload_hash,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require_keys_eq!(
        queue_target_program,
        expected_program_id,
        CustomError::InvalidGreenLabelTargetProgram
    );
    require_keys_eq!(
        queue_target_account,
        expected_target_account,
        CustomError::InvalidGreenLabelTargetAccount
    );
    require!(
        now >= queue_execute_after,
        CustomError::GreenLabelTimelockNotSatisfied
    );
    require_keys_eq!(
        provided_usdc_mint,
        expected_usdc_mint,
        CustomError::InvalidGreenLabelMint
    );
    require!(
        usdc_decimals == GREEN_LABEL_USDC_DECIMALS,
        CustomError::InvalidGreenLabelMint
    );
    require_keys_eq!(
        green_bond_vault_mint,
        expected_usdc_mint,
        CustomError::InvalidGreenLabelTokenAccount
    );
    require_keys_eq!(
        green_bond_vault_owner,
        project_bond_vault_authority,
        CustomError::InvalidGreenLabelTokenAccount
    );
    require_keys_eq!(
        provided_relief_or_risk_vault,
        expected_relief_or_risk_vault,
        CustomError::InvalidGreenLabelTokenAccount
    );
    require_keys_eq!(
        relief_or_risk_vault_mint,
        expected_usdc_mint,
        CustomError::InvalidGreenLabelTokenAccount
    );
    require!(
        vault_balance >= slash_amount,
        CustomError::GreenLabelInsufficientBondVaultBalance
    );

    Ok(())
}

pub fn record_green_label_refunded(
    project: &mut GreenLabelProjectV1,
    dispute: Option<&mut GreenLabelDisputeV1>,
    now: i64,
) -> Result<()> {
    project.status = GreenLabelStatus::Refunded;
    project.refunded_at = now;
    project.active_dispute = Pubkey::default();

    if let Some(dispute) = dispute {
        dispute.status = DisputeStatus::ResolvedRefund;
        dispute.resolved_at = now;
    }

    Ok(())
}

pub fn record_green_label_slashed(
    project: &mut GreenLabelProjectV1,
    dispute: &mut GreenLabelDisputeV1,
    now: i64,
) -> Result<()> {
    project.status = GreenLabelStatus::Slashed;
    project.slashed_at = now;
    project.active_dispute = Pubkey::default();
    dispute.status = DisputeStatus::ResolvedSlash;
    dispute.resolved_at = now;

    Ok(())
}

pub fn record_green_bond_vault_initialization(
    green_label_project: &mut GreenLabelProjectV1,
    green_bond_vault: Pubkey,
    green_bond_vault_authority: Pubkey,
) {
    green_label_project.bond_vault = green_bond_vault;
    green_label_project.bond_vault_authority = green_bond_vault_authority;
}

pub fn build_pending_bond_project_values(
    is_config_paused: bool,
    configured_min_base_bond_usdc: u64,
    current_project_count: u64,
    expected_project_id: u64,
    project_owner: Pubkey,
    project_name_hash: [u8; 32],
    project_url_hash: [u8; 32],
    token_mint: Pubkey,
    project_treasury_wallet: Pubkey,
    total_bond_amount: u64,
    submitted_at: i64,
    bump: u8,
) -> Result<GreenLabelProjectInitValues> {
    require!(!is_config_paused, CustomError::InvalidGreenLabelStatus);

    let next_project_id = current_project_count
        .checked_add(1)
        .ok_or(CustomError::GreenLabelMathOverflow)?;
    require!(
        expected_project_id == next_project_id,
        CustomError::InvalidGreenLabelProjectId
    );

    let (split, bond_tier) =
        derive_bond_split_and_tier(total_bond_amount, configured_min_base_bond_usdc)?;

    Ok(GreenLabelProjectInitValues {
        project_id: expected_project_id,
        project_owner,
        project_name_hash,
        project_url_hash,
        token_mint,
        project_treasury_wallet,
        base_bond_amount: split.base_bond_amount,
        extra_bond_amount: split.extra_bond_amount,
        total_bond_amount: split.total_bond_amount,
        bond_vault: Pubkey::default(),
        bond_vault_authority: Pubkey::default(),
        bond_tier,
        status: GreenLabelStatus::PendingBondDeposit,
        submitted_at,
        observation_start_ts: 0,
        observation_end_ts: 0,
        dispute_count: 0,
        active_dispute: Pubkey::default(),
        approved_at: 0,
        refunded_at: 0,
        slashed_at: 0,
        risk_score_snapshot: 0,
        terminal_proposal_id: 0,
        terminal_proposal_decision: Pubkey::default(),
        terminal_execution_queue_item: Pubkey::default(),
        terminal_payload_hash: [0; 32],
        terminal_action_type: ActionType::Noop,
        bump,
        reserved: [0; GREEN_LABEL_PROJECT_RESERVED_BYTES],
    })
}

pub fn validate_green_label_bps_config(base_refund_bps: u16, base_treasury_bps: u16) -> Result<()> {
    let configured_bps = base_refund_bps
        .checked_add(base_treasury_bps)
        .ok_or(CustomError::GreenLabelMathOverflow)?;

    require!(
        configured_bps == MAX_BPS,
        CustomError::InvalidGreenLabelBpsConfig
    );

    Ok(())
}

pub fn split_green_label_bond(
    total_bond_amount: u64,
    configured_min_base_bond_usdc: u64,
) -> Result<GreenLabelBondSplit> {
    require!(
        configured_min_base_bond_usdc > 0
            && configured_min_base_bond_usdc <= MIN_GREEN_LABEL_BASE_BOND_USDC,
        CustomError::InvalidGreenLabelBondAmount
    );
    require!(
        total_bond_amount >= configured_min_base_bond_usdc,
        CustomError::InvalidGreenLabelBondAmount
    );

    let extra_bond_amount = total_bond_amount
        .checked_sub(configured_min_base_bond_usdc)
        .ok_or(CustomError::GreenLabelMathOverflow)?;

    Ok(GreenLabelBondSplit {
        base_bond_amount: configured_min_base_bond_usdc,
        extra_bond_amount,
        total_bond_amount,
    })
}

pub fn calculate_green_label_refund(
    base_bond_amount: u64,
    extra_bond_amount: u64,
) -> Result<GreenLabelRefundAmounts> {
    validate_green_label_bps_config(BASE_BOND_REFUND_BPS, BASE_BOND_TREASURY_BPS)?;
    require!(
        base_bond_amount > 0,
        CustomError::InvalidGreenLabelBondAmount
    );

    let base_refund_amount = calculate_bps_amount(base_bond_amount, BASE_BOND_REFUND_BPS)?;
    let base_treasury_amount = calculate_bps_amount(base_bond_amount, BASE_BOND_TREASURY_BPS)?;

    let base_total = base_refund_amount
        .checked_add(base_treasury_amount)
        .ok_or(CustomError::GreenLabelMathOverflow)?;
    require!(
        base_total == base_bond_amount,
        CustomError::InvalidGreenLabelBondSplit
    );

    let project_refund_amount = base_refund_amount
        .checked_add(extra_bond_amount)
        .ok_or(CustomError::GreenLabelMathOverflow)?;

    Ok(GreenLabelRefundAmounts {
        project_refund_amount,
        treasury_amount: base_treasury_amount,
        base_refund_amount,
        base_treasury_amount,
        extra_refund_amount: extra_bond_amount,
    })
}

pub fn calculate_green_label_slash_amount(total_bond_amount: u64) -> Result<u64> {
    require!(
        total_bond_amount > 0,
        CustomError::InvalidGreenLabelBondAmount
    );
    Ok(total_bond_amount)
}

pub fn validate_green_label_certification_fee_policy_init_v1(
    expected_authority: Pubkey,
    authority: Pubkey,
    usdc_mint: Pubkey,
    fee_amount_usdc: u64,
) -> Result<()> {
    require_keys_eq!(
        expected_authority,
        authority,
        CustomError::UnauthorizedGreenLabelAuthority
    );
    require_keys_neq!(
        usdc_mint,
        Pubkey::default(),
        CustomError::GreenLabelCertificationFeeMintMismatch
    );
    require!(
        fee_amount_usdc > 0,
        CustomError::InvalidGreenLabelCertificationFeeAmount
    );

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn validate_green_label_certification_fee_once_v1(
    green_label_config: &GreenLabelConfigV1,
    green_label_config_key: Pubkey,
    green_label_project: &GreenLabelProjectV1,
    green_label_project_key: Pubkey,
    fee_policy: &GreenLabelCertificationFeePolicyV1,
    fee_policy_key: Pubkey,
    payer_usdc_account: &TokenAccount,
    payer_usdc_account_key: Pubkey,
    payer: Pubkey,
    treasury_config: &TreasuryConfigV2,
    treasury_config_key: Pubkey,
    _treasury_usdc_state: &TreasuryUsdcStateV2,
    treasury_usdc_state_key: Pubkey,
    revenue_routing_stats: &RevenueRoutingStatsV1,
    revenue_routing_stats_key: Pubkey,
    vault_authority_key: Pubkey,
    relief_usdc_vault_key: Pubkey,
    relief_usdc_vault: &TokenAccount,
    buyback_usdc_vault_key: Pubkey,
    buyback_usdc_vault: &TokenAccount,
    builders_usdc_vault_key: Pubkey,
    builders_usdc_vault: &TokenAccount,
    staking_usdc_vault_key: Pubkey,
    staking_usdc_vault: &TokenAccount,
    usdc_mint_key: Pubkey,
    usdc_decimals: u8,
    parameters: &GreenLabelCertificationFeeParametersV1,
) -> Result<[u8; 32]> {
    require!(
        !green_label_config.is_paused,
        CustomError::InvalidGreenLabelStatus
    );
    require!(
        fee_policy.schema_version == GREEN_LABEL_CERTIFICATION_FEE_SCHEMA_VERSION,
        CustomError::InvalidGreenLabelCertificationFeePolicySchema
    );
    require!(
        fee_policy.policy_version == GREEN_LABEL_CERTIFICATION_FEE_POLICY_VERSION,
        CustomError::InvalidGreenLabelCertificationFeePolicySchema
    );
    require!(
        fee_policy.active,
        CustomError::GreenLabelCertificationFeePolicyInactive
    );
    require!(
        fee_policy.fee_amount_usdc > 0,
        CustomError::InvalidGreenLabelCertificationFeeAmount
    );
    require_keys_eq!(
        fee_policy.green_label_config,
        green_label_config_key,
        CustomError::GreenLabelCertificationFeeParametersMismatch
    );
    require_keys_eq!(
        fee_policy.usdc_mint,
        green_label_config.usdc_mint,
        CustomError::GreenLabelCertificationFeeMintMismatch
    );
    require!(
        matches!(
            green_label_project.status,
            GreenLabelStatus::PendingBondDeposit
        ),
        CustomError::GreenLabelCertificationFeeStatusMismatch
    );
    require_keys_eq!(
        green_label_project.project_owner,
        payer,
        CustomError::GreenLabelCertificationFeePayerMismatch
    );
    require_keys_eq!(
        payer_usdc_account.owner,
        payer,
        CustomError::GreenLabelCertificationFeePayerMismatch
    );
    require_keys_eq!(
        payer_usdc_account.mint,
        fee_policy.usdc_mint,
        CustomError::GreenLabelCertificationFeeMintMismatch
    );
    require!(
        payer_usdc_account.amount >= fee_policy.fee_amount_usdc,
        CustomError::GreenLabelCertificationFeeInsufficientFunds
    );
    require_keys_eq!(
        usdc_mint_key,
        fee_policy.usdc_mint,
        CustomError::GreenLabelCertificationFeeMintMismatch
    );
    require!(
        usdc_decimals == GREEN_LABEL_USDC_DECIMALS,
        CustomError::GreenLabelCertificationFeeDecimalsMismatch
    );
    require_keys_eq!(
        treasury_config.usdc_mint,
        fee_policy.usdc_mint,
        CustomError::GreenLabelCertificationFeeMintMismatch
    );
    require_keys_eq!(
        green_label_config.treasury_usdc_state_v2,
        treasury_usdc_state_key,
        CustomError::GreenLabelCertificationFeeTreasuryMismatch
    );
    require_keys_eq!(
        revenue_routing_stats.authority,
        treasury_config.authority,
        CustomError::UnauthorizedTreasuryAuthority
    );
    require_keys_eq!(
        revenue_routing_stats.usdc_mint,
        fee_policy.usdc_mint,
        CustomError::GreenLabelCertificationFeeMintMismatch
    );
    require_keys_eq!(
        green_label_config.vault_authority_v2,
        vault_authority_key,
        CustomError::GreenLabelCertificationFeeTreasuryMismatch
    );
    require_keys_eq!(
        relief_usdc_vault.mint,
        fee_policy.usdc_mint,
        CustomError::GreenLabelCertificationFeeMintMismatch
    );
    require_keys_eq!(
        buyback_usdc_vault.mint,
        fee_policy.usdc_mint,
        CustomError::GreenLabelCertificationFeeMintMismatch
    );
    require_keys_eq!(
        builders_usdc_vault.mint,
        fee_policy.usdc_mint,
        CustomError::GreenLabelCertificationFeeMintMismatch
    );
    require_keys_eq!(
        staking_usdc_vault.mint,
        fee_policy.usdc_mint,
        CustomError::GreenLabelCertificationFeeMintMismatch
    );
    require_keys_eq!(
        relief_usdc_vault.owner,
        vault_authority_key,
        CustomError::GreenLabelCertificationFeeTreasuryMismatch
    );
    require_keys_eq!(
        buyback_usdc_vault.owner,
        vault_authority_key,
        CustomError::GreenLabelCertificationFeeTreasuryMismatch
    );
    require_keys_eq!(
        builders_usdc_vault.owner,
        vault_authority_key,
        CustomError::GreenLabelCertificationFeeTreasuryMismatch
    );
    require_keys_eq!(
        staking_usdc_vault.owner,
        vault_authority_key,
        CustomError::GreenLabelCertificationFeeTreasuryMismatch
    );
    require_keys_neq!(
        relief_usdc_vault_key,
        payer_usdc_account_key,
        CustomError::GreenLabelCertificationFeeTreasuryMismatch
    );
    require_keys_neq!(
        buyback_usdc_vault_key,
        payer_usdc_account_key,
        CustomError::GreenLabelCertificationFeeTreasuryMismatch
    );
    require_keys_neq!(
        builders_usdc_vault_key,
        payer_usdc_account_key,
        CustomError::GreenLabelCertificationFeeTreasuryMismatch
    );
    require_keys_neq!(
        staking_usdc_vault_key,
        payer_usdc_account_key,
        CustomError::GreenLabelCertificationFeeTreasuryMismatch
    );

    require!(
        parameters.schema_version == GREEN_LABEL_CERTIFICATION_FEE_SCHEMA_VERSION,
        CustomError::InvalidGreenLabelCertificationFeePolicySchema
    );
    require_keys_eq!(
        parameters.green_label_config,
        green_label_config_key,
        CustomError::GreenLabelCertificationFeeParametersMismatch
    );
    require_keys_eq!(
        parameters.fee_policy,
        fee_policy_key,
        CustomError::GreenLabelCertificationFeeParametersMismatch
    );
    require!(
        parameters.policy_version == fee_policy.policy_version,
        CustomError::GreenLabelCertificationFeeParametersMismatch
    );
    require_keys_eq!(
        parameters.green_label_project,
        green_label_project_key,
        CustomError::GreenLabelCertificationFeeProjectMismatch
    );
    require!(
        parameters.project_id == green_label_project.project_id,
        CustomError::GreenLabelCertificationFeeProjectMismatch
    );
    require_keys_eq!(
        parameters.project_owner,
        green_label_project.project_owner,
        CustomError::GreenLabelCertificationFeePayerMismatch
    );
    require_keys_eq!(
        parameters.payer,
        payer,
        CustomError::GreenLabelCertificationFeePayerMismatch
    );
    require_keys_eq!(
        parameters.payer_token_account,
        payer_usdc_account_key,
        CustomError::GreenLabelCertificationFeePayerMismatch
    );
    require!(
        parameters.fee_amount_usdc == fee_policy.fee_amount_usdc,
        CustomError::InvalidGreenLabelCertificationFeeAmount
    );
    require_keys_eq!(
        parameters.usdc_mint,
        fee_policy.usdc_mint,
        CustomError::GreenLabelCertificationFeeMintMismatch
    );
    require_keys_eq!(
        parameters.treasury_config,
        treasury_config_key,
        CustomError::GreenLabelCertificationFeeTreasuryMismatch
    );
    require_keys_eq!(
        parameters.treasury_usdc_state,
        treasury_usdc_state_key,
        CustomError::GreenLabelCertificationFeeTreasuryMismatch
    );
    require_keys_eq!(
        parameters.revenue_routing_stats,
        revenue_routing_stats_key,
        CustomError::GreenLabelCertificationFeeTreasuryMismatch
    );
    require_keys_eq!(
        parameters.relief_usdc_vault,
        relief_usdc_vault_key,
        CustomError::GreenLabelCertificationFeeTreasuryMismatch
    );
    require_keys_eq!(
        parameters.buyback_usdc_vault,
        buyback_usdc_vault_key,
        CustomError::GreenLabelCertificationFeeTreasuryMismatch
    );
    require_keys_eq!(
        parameters.builders_usdc_vault,
        builders_usdc_vault_key,
        CustomError::GreenLabelCertificationFeeTreasuryMismatch
    );
    require_keys_eq!(
        parameters.staking_usdc_vault,
        staking_usdc_vault_key,
        CustomError::GreenLabelCertificationFeeTreasuryMismatch
    );
    require!(
        parameters.revenue_type == RevenueType::GreenLabelCertificationFee,
        CustomError::GreenLabelCertificationFeeParametersMismatch
    );

    hash_green_label_certification_fee_parameters_v1(parameters)
}

pub fn validate_green_label_refundable_escrow_initialization(
    config_is_paused: bool,
    project_owner: Pubkey,
    payer: Pubkey,
    expected_usdc_mint: Pubkey,
    provided_usdc_mint: Pubkey,
    now: i64,
    refund_available_after: i64,
) -> Result<()> {
    require!(!config_is_paused, CustomError::InvalidGreenLabelStatus);
    require_keys_eq!(
        project_owner,
        payer,
        CustomError::InvalidGreenLabelProjectOwner
    );
    require_keys_eq!(
        provided_usdc_mint,
        expected_usdc_mint,
        CustomError::InvalidGreenLabelMint
    );
    require!(
        refund_available_after >= now,
        CustomError::InvalidGreenLabelEscrowRefund
    );

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn validate_green_label_refundable_bond_deposit(
    escrow_status: GreenLabelEscrowStatusV1,
    expected_payer: Pubkey,
    payer: Pubkey,
    expected_usdc_mint: Pubkey,
    payer_usdc_account_mint: Pubkey,
    refundable_vault_mint: Pubkey,
    refundable_vault_owner: Pubkey,
    escrow_key: Pubkey,
    provided_usdc_mint: Pubkey,
    usdc_decimals: u8,
    amount: u64,
) -> Result<()> {
    require!(amount > 0, CustomError::InvalidGreenLabelEscrowAmount);
    require!(
        escrow_status == GreenLabelEscrowStatusV1::Locked,
        CustomError::InvalidGreenLabelEscrowStatus
    );
    require_keys_eq!(
        expected_payer,
        payer,
        CustomError::InvalidGreenLabelProjectOwner
    );
    require_keys_eq!(
        payer_usdc_account_mint,
        expected_usdc_mint,
        CustomError::InvalidGreenLabelMint
    );
    require_keys_eq!(
        refundable_vault_mint,
        expected_usdc_mint,
        CustomError::InvalidGreenLabelMint
    );
    require_keys_eq!(
        refundable_vault_owner,
        escrow_key,
        CustomError::InvalidGreenLabelTokenAccount
    );
    require_keys_eq!(
        provided_usdc_mint,
        expected_usdc_mint,
        CustomError::InvalidGreenLabelMint
    );
    require!(
        usdc_decimals == GREEN_LABEL_USDC_DECIMALS,
        CustomError::InvalidGreenLabelMint
    );

    Ok(())
}

pub fn calculate_green_label_escrow_remaining_amount(
    refundable_amount: u64,
    refunded_amount: u64,
    forfeited_amount: u64,
) -> Result<u64> {
    let used_amount = refunded_amount
        .checked_add(forfeited_amount)
        .ok_or(CustomError::GreenLabelMathOverflow)?;

    refundable_amount
        .checked_sub(used_amount)
        .ok_or(CustomError::InvalidGreenLabelEscrowAmount.into())
}

#[allow(clippy::too_many_arguments)]
pub fn validate_green_label_escrow_refund(
    escrow_status: GreenLabelEscrowStatusV1,
    refundable_amount: u64,
    refunded_amount: u64,
    forfeited_amount: u64,
    refund_available_after: i64,
    now: i64,
    project_active_dispute: Pubkey,
    project_terminal_action_type: ActionType,
    project_terminal_proposal_id: u64,
    project_terminal_payload_hash: [u8; 32],
    escrow_payer: Pubkey,
    payer_refund_account_owner: Pubkey,
    expected_usdc_mint: Pubkey,
    payer_refund_account_mint: Pubkey,
    refundable_vault_mint: Pubkey,
    refundable_vault_owner: Pubkey,
    escrow_key: Pubkey,
    provided_usdc_mint: Pubkey,
    usdc_decimals: u8,
    vault_balance: u64,
) -> Result<u64> {
    require!(
        matches!(
            escrow_status,
            GreenLabelEscrowStatusV1::Locked | GreenLabelEscrowStatusV1::Refundable
        ),
        CustomError::InvalidGreenLabelEscrowStatus
    );

    let time_refund_allowed =
        project_active_dispute == Pubkey::default() && now >= refund_available_after;
    let decision_refund_allowed = project_terminal_action_type == ActionType::GreenLabelRefund
        && project_terminal_proposal_id > 0
        && project_terminal_payload_hash != [0; 32];

    require!(
        time_refund_allowed || decision_refund_allowed,
        CustomError::InvalidGreenLabelEscrowRefund
    );
    require_keys_eq!(
        payer_refund_account_owner,
        escrow_payer,
        CustomError::InvalidGreenLabelEscrowRefund
    );
    require_keys_eq!(
        payer_refund_account_mint,
        expected_usdc_mint,
        CustomError::InvalidGreenLabelMint
    );
    require_keys_eq!(
        refundable_vault_mint,
        expected_usdc_mint,
        CustomError::InvalidGreenLabelMint
    );
    require_keys_eq!(
        refundable_vault_owner,
        escrow_key,
        CustomError::InvalidGreenLabelTokenAccount
    );
    require_keys_eq!(
        provided_usdc_mint,
        expected_usdc_mint,
        CustomError::InvalidGreenLabelMint
    );
    require!(
        usdc_decimals == GREEN_LABEL_USDC_DECIMALS,
        CustomError::InvalidGreenLabelMint
    );

    let refund_amount = calculate_green_label_escrow_remaining_amount(
        refundable_amount,
        refunded_amount,
        forfeited_amount,
    )?;
    require!(
        refund_amount > 0,
        CustomError::InvalidGreenLabelEscrowAmount
    );
    require!(
        vault_balance >= refund_amount,
        CustomError::GreenLabelInsufficientBondVaultBalance
    );

    Ok(refund_amount)
}

#[allow(clippy::too_many_arguments)]
pub fn validate_green_label_treasury_router_accounts(
    expected_usdc_mint: Pubkey,
    treasury_config_key: Pubkey,
    treasury_config_usdc_mint: Pubkey,
    treasury_usdc_state_key: Pubkey,
    revenue_routing_stats_key: Pubkey,
    revenue_routing_stats_usdc_mint: Pubkey,
    vault_authority_key: Pubkey,
    relief_usdc_vault_key: Pubkey,
    relief_usdc_vault_mint: Pubkey,
    relief_usdc_vault_owner: Pubkey,
    buyback_usdc_vault_key: Pubkey,
    buyback_usdc_vault_mint: Pubkey,
    buyback_usdc_vault_owner: Pubkey,
    builders_usdc_vault_key: Pubkey,
    builders_usdc_vault_mint: Pubkey,
    builders_usdc_vault_owner: Pubkey,
    staking_usdc_vault_key: Pubkey,
    staking_usdc_vault_mint: Pubkey,
    staking_usdc_vault_owner: Pubkey,
) -> Result<()> {
    let (expected_treasury_config, _) =
        Pubkey::find_program_address(&[TREASURY_CONFIG_V2_SEED], &crate::ID);
    let (expected_treasury_usdc_state, _) =
        Pubkey::find_program_address(&[TREASURY_USDC_STATE_V2_SEED], &crate::ID);
    let (expected_revenue_routing_stats, _) = Pubkey::find_program_address(
        &[REVENUE_ROUTING_STATS_V1_SEED, treasury_config_key.as_ref()],
        &crate::ID,
    );
    let (expected_vault_authority, _) =
        Pubkey::find_program_address(&[VAULT_AUTHORITY_V2_SEED], &crate::ID);
    let (expected_relief_usdc_vault, _) =
        Pubkey::find_program_address(&[RELIEF_USDC_VAULT_SEED], &crate::ID);
    let (expected_buyback_usdc_vault, _) =
        Pubkey::find_program_address(&[BUYBACK_USDC_VAULT_SEED], &crate::ID);
    let (expected_builders_usdc_vault, _) =
        Pubkey::find_program_address(&[BUILDERS_USDC_VAULT_SEED], &crate::ID);
    let (expected_staking_usdc_vault, _) =
        Pubkey::find_program_address(&[STAKING_USDC_VAULT_SEED], &crate::ID);

    require_keys_eq!(
        treasury_config_key,
        expected_treasury_config,
        CustomError::InvalidVault
    );
    require_keys_eq!(
        treasury_config_usdc_mint,
        expected_usdc_mint,
        CustomError::InvalidGreenLabelMint
    );
    require_keys_eq!(
        treasury_usdc_state_key,
        expected_treasury_usdc_state,
        CustomError::InvalidVault
    );
    require_keys_eq!(
        revenue_routing_stats_key,
        expected_revenue_routing_stats,
        CustomError::InvalidVault
    );
    require_keys_eq!(
        revenue_routing_stats_usdc_mint,
        expected_usdc_mint,
        CustomError::InvalidMint
    );
    require_keys_eq!(
        vault_authority_key,
        expected_vault_authority,
        CustomError::InvalidVault
    );

    require_keys_eq!(
        relief_usdc_vault_key,
        expected_relief_usdc_vault,
        CustomError::InvalidVault
    );
    require_keys_eq!(
        relief_usdc_vault_mint,
        expected_usdc_mint,
        CustomError::InvalidMint
    );
    require_keys_eq!(
        relief_usdc_vault_owner,
        vault_authority_key,
        CustomError::InvalidVault
    );
    require_keys_eq!(
        buyback_usdc_vault_key,
        expected_buyback_usdc_vault,
        CustomError::InvalidVault
    );
    require_keys_eq!(
        buyback_usdc_vault_mint,
        expected_usdc_mint,
        CustomError::InvalidMint
    );
    require_keys_eq!(
        buyback_usdc_vault_owner,
        vault_authority_key,
        CustomError::InvalidVault
    );
    require_keys_eq!(
        builders_usdc_vault_key,
        expected_builders_usdc_vault,
        CustomError::InvalidVault
    );
    require_keys_eq!(
        builders_usdc_vault_mint,
        expected_usdc_mint,
        CustomError::InvalidMint
    );
    require_keys_eq!(
        builders_usdc_vault_owner,
        vault_authority_key,
        CustomError::InvalidVault
    );
    require_keys_eq!(
        staking_usdc_vault_key,
        expected_staking_usdc_vault,
        CustomError::InvalidVault
    );
    require_keys_eq!(
        staking_usdc_vault_mint,
        expected_usdc_mint,
        CustomError::InvalidMint
    );
    require_keys_eq!(
        staking_usdc_vault_owner,
        vault_authority_key,
        CustomError::InvalidVault
    );

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn validate_green_label_escrow_forfeit_to_treasury(
    config_is_paused: bool,
    project_status: GreenLabelStatus,
    project_active_dispute: Pubkey,
    dispute_key: Pubkey,
    project_terminal_proposal_id: u64,
    project_terminal_proposal_decision: Pubkey,
    project_terminal_execution_queue_item: Pubkey,
    project_terminal_payload_hash: [u8; 32],
    project_terminal_action_type: ActionType,
    dispute_project: Pubkey,
    project_key: Pubkey,
    dispute_status: DisputeStatus,
    dispute_proposal_id: u64,
    dispute_proposal_decision: Pubkey,
    dispute_execution_queue_item: Pubkey,
    dispute_payload_hash: [u8; 32],
    dispute_action_type: ActionType,
    proposal_decision_key: Pubkey,
    proposal_decision_proposal_id: u64,
    proposal_decision: ProposalDecision,
    execution_queue_item_key: Pubkey,
    queue_proposal_id: u64,
    queue_status: ExecutionStatus,
    queue_action_type: ActionType,
    queue_payload_hash: [u8; 32],
    queue_target_program: Pubkey,
    expected_program_id: Pubkey,
    queue_target_account: Pubkey,
    expected_target_account: Pubkey,
    now: i64,
    queue_execute_after: i64,
    escrow_status: GreenLabelEscrowStatusV1,
    refundable_amount: u64,
    refunded_amount: u64,
    forfeited_amount: u64,
    expected_usdc_mint: Pubkey,
    refundable_vault_mint: Pubkey,
    refundable_vault_owner: Pubkey,
    escrow_key: Pubkey,
    provided_usdc_mint: Pubkey,
    usdc_decimals: u8,
    vault_balance: u64,
) -> Result<u64> {
    require!(!config_is_paused, CustomError::InvalidGreenLabelStatus);
    require!(
        matches!(
            escrow_status,
            GreenLabelEscrowStatusV1::Locked | GreenLabelEscrowStatusV1::Refundable
        ),
        CustomError::InvalidGreenLabelEscrowStatus
    );
    require!(
        project_status == GreenLabelStatus::SlashQueued,
        CustomError::InvalidGreenLabelStatus
    );
    require_keys_eq!(
        project_active_dispute,
        dispute_key,
        CustomError::InvalidGreenLabelActiveDispute
    );
    require_keys_eq!(
        dispute_project,
        project_key,
        CustomError::InvalidGreenLabelTargetAccount
    );
    require!(
        dispute_status == DisputeStatus::DecisionQueued,
        CustomError::InvalidGreenLabelDisputeStatus
    );
    validate_terminal_action_for_slash(project_terminal_action_type, true)?;
    validate_payload_hash(project_terminal_payload_hash)?;
    require!(
        dispute_proposal_id == project_terminal_proposal_id,
        CustomError::InvalidGreenLabelSecurityDecision
    );
    require_keys_eq!(
        dispute_proposal_decision,
        project_terminal_proposal_decision,
        CustomError::InvalidGreenLabelSecurityDecision
    );
    require_keys_eq!(
        dispute_execution_queue_item,
        project_terminal_execution_queue_item,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require!(
        dispute_payload_hash == project_terminal_payload_hash,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require!(
        dispute_action_type == ActionType::GreenLabelSlash,
        CustomError::InvalidGreenLabelActionType
    );
    require_keys_eq!(
        proposal_decision_key,
        project_terminal_proposal_decision,
        CustomError::InvalidGreenLabelSecurityDecision
    );
    require_keys_eq!(
        execution_queue_item_key,
        project_terminal_execution_queue_item,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require!(
        proposal_decision_proposal_id == project_terminal_proposal_id,
        CustomError::InvalidGreenLabelSecurityDecision
    );
    require!(
        queue_proposal_id == project_terminal_proposal_id,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require!(
        proposal_decision == ProposalDecision::Approved,
        CustomError::InvalidGreenLabelSecurityDecision
    );
    require!(
        queue_status == ExecutionStatus::Queued,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require!(
        queue_action_type == ActionType::GreenLabelSlash,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require!(
        queue_payload_hash == project_terminal_payload_hash,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require_keys_eq!(
        queue_target_program,
        expected_program_id,
        CustomError::InvalidGreenLabelTargetProgram
    );
    require_keys_eq!(
        queue_target_account,
        expected_target_account,
        CustomError::InvalidGreenLabelTargetAccount
    );
    require!(
        now >= queue_execute_after,
        CustomError::GreenLabelTimelockNotSatisfied
    );
    require_keys_eq!(
        refundable_vault_mint,
        expected_usdc_mint,
        CustomError::InvalidGreenLabelMint
    );
    require_keys_eq!(
        refundable_vault_owner,
        escrow_key,
        CustomError::InvalidGreenLabelTokenAccount
    );
    require_keys_eq!(
        provided_usdc_mint,
        expected_usdc_mint,
        CustomError::InvalidGreenLabelMint
    );
    require!(
        usdc_decimals == GREEN_LABEL_USDC_DECIMALS,
        CustomError::InvalidGreenLabelMint
    );

    let forfeit_amount = calculate_green_label_escrow_remaining_amount(
        refundable_amount,
        refunded_amount,
        forfeited_amount,
    )?;
    require!(
        forfeit_amount > 0,
        CustomError::InvalidGreenLabelEscrowAmount
    );
    require!(
        vault_balance >= forfeit_amount,
        CustomError::GreenLabelInsufficientBondVaultBalance
    );

    Ok(forfeit_amount)
}

pub fn record_green_label_refundable_bond_deposit(
    escrow: &mut GreenLabelRefundableEscrowV1,
    amount: u64,
    now: i64,
) -> Result<()> {
    escrow.deposited_amount = escrow
        .deposited_amount
        .checked_add(amount)
        .ok_or(CustomError::GreenLabelMathOverflow)?;
    escrow.refundable_amount = escrow
        .refundable_amount
        .checked_add(amount)
        .ok_or(CustomError::GreenLabelMathOverflow)?;
    if escrow.deposit_ts == 0 {
        escrow.deposit_ts = now;
    }

    Ok(())
}

pub fn record_green_label_certification_fee_policy_init_v1(
    policy: &mut GreenLabelCertificationFeePolicyV1,
    green_label_config: Pubkey,
    usdc_mint: Pubkey,
    fee_amount_usdc: u64,
    initialized_by: Pubkey,
    now: i64,
    bump: u8,
) -> Result<()> {
    require!(
        policy.green_label_config == Pubkey::default(),
        CustomError::GreenLabelCertificationFeePolicyAlreadyInitialized
    );
    validate_green_label_certification_fee_policy_init_v1(
        initialized_by,
        initialized_by,
        usdc_mint,
        fee_amount_usdc,
    )?;

    policy.green_label_config = green_label_config;
    policy.usdc_mint = usdc_mint;
    policy.fee_amount_usdc = fee_amount_usdc;
    policy.policy_version = GREEN_LABEL_CERTIFICATION_FEE_POLICY_VERSION;
    policy.active = true;
    policy.initialized_by = initialized_by;
    policy.created_at = now;
    policy.schema_version = GREEN_LABEL_CERTIFICATION_FEE_SCHEMA_VERSION;
    policy.bump = bump;

    Ok(())
}

pub fn record_green_label_certification_fee_receipt_v1(
    receipt: &mut GreenLabelCertificationFeeReceiptV1,
    parameters: &GreenLabelCertificationFeeParametersV1,
    parameters_hash: [u8; 32],
    routed_at: i64,
    bump: u8,
    receipt_key: Pubkey,
) -> Result<()> {
    require!(
        receipt.green_label_project == Pubkey::default(),
        CustomError::GreenLabelCertificationFeeAlreadyPaid
    );
    require_keys_neq!(
        receipt_key,
        Pubkey::default(),
        CustomError::GreenLabelCertificationFeeParametersMismatch
    );
    require!(
        parameters_hash == hash_green_label_certification_fee_parameters_v1(parameters)?,
        CustomError::GreenLabelCertificationFeeParametersMismatch
    );

    receipt.green_label_config = parameters.green_label_config;
    receipt.fee_policy = parameters.fee_policy;
    receipt.policy_version = parameters.policy_version;
    receipt.green_label_project = parameters.green_label_project;
    receipt.project_id = parameters.project_id;
    receipt.project_owner = parameters.project_owner;
    receipt.payer = parameters.payer;
    receipt.payer_token_account = parameters.payer_token_account;
    receipt.amount_usdc = parameters.fee_amount_usdc;
    receipt.usdc_mint = parameters.usdc_mint;
    receipt.treasury_config = parameters.treasury_config;
    receipt.treasury_usdc_state = parameters.treasury_usdc_state;
    receipt.revenue_routing_stats = parameters.revenue_routing_stats;
    receipt.relief_usdc_vault = parameters.relief_usdc_vault;
    receipt.buyback_usdc_vault = parameters.buyback_usdc_vault;
    receipt.builders_usdc_vault = parameters.builders_usdc_vault;
    receipt.staking_usdc_vault = parameters.staking_usdc_vault;
    receipt.revenue_type = RevenueType::GreenLabelCertificationFee;
    receipt.parameters_hash = parameters_hash;
    receipt.routed_at = routed_at;
    receipt.schema_version = GREEN_LABEL_CERTIFICATION_FEE_SCHEMA_VERSION;
    receipt.bump = bump;

    Ok(())
}

pub fn record_green_label_escrow_refunded(
    escrow: &mut GreenLabelRefundableEscrowV1,
    refund_amount: u64,
) -> Result<()> {
    escrow.refunded_amount = escrow
        .refunded_amount
        .checked_add(refund_amount)
        .ok_or(CustomError::GreenLabelMathOverflow)?;
    escrow.status = GreenLabelEscrowStatusV1::Refunded;

    Ok(())
}

pub fn record_green_label_escrow_forfeited(
    escrow: &mut GreenLabelRefundableEscrowV1,
    forfeit_amount: u64,
) -> Result<()> {
    escrow.forfeited_amount = escrow
        .forfeited_amount
        .checked_add(forfeit_amount)
        .ok_or(CustomError::GreenLabelMathOverflow)?;
    escrow.status = GreenLabelEscrowStatusV1::Forfeited;

    Ok(())
}

pub fn calculate_bond_tier(
    total_bond_amount: u64,
    configured_min_base_bond_usdc: u64,
) -> Result<BondTier> {
    require!(
        configured_min_base_bond_usdc > 0
            && configured_min_base_bond_usdc <= MIN_GREEN_LABEL_BASE_BOND_USDC,
        CustomError::InvalidGreenLabelBondAmount
    );
    require!(
        total_bond_amount >= configured_min_base_bond_usdc,
        CustomError::InvalidGreenLabelBondAmount
    );

    if total_bond_amount >= GREEN_LABEL_PLATINUM_TIER_THRESHOLD_USDC {
        Ok(BondTier::Platinum)
    } else if total_bond_amount >= GREEN_LABEL_GOLD_TIER_THRESHOLD_USDC {
        Ok(BondTier::Gold)
    } else if total_bond_amount >= GREEN_LABEL_SILVER_TIER_THRESHOLD_USDC {
        Ok(BondTier::Silver)
    } else if total_bond_amount >= GREEN_LABEL_BRONZE_TIER_THRESHOLD_USDC {
        Ok(BondTier::Bronze)
    } else {
        Ok(BondTier::Base)
    }
}

pub fn validate_green_label_status_transition(
    from: GreenLabelStatus,
    to: GreenLabelStatus,
    has_linked_dispute: bool,
) -> Result<()> {
    if matches!(
        from,
        GreenLabelStatus::Refunded | GreenLabelStatus::Slashed | GreenLabelStatus::Cancelled
    ) {
        return err!(CustomError::InvalidGreenLabelStatus);
    }

    if from == GreenLabelStatus::Disputed
        && to == GreenLabelStatus::SlashQueued
        && !has_linked_dispute
    {
        return err!(CustomError::InvalidGreenLabelSlashWithoutDispute);
    }

    let is_valid = matches!(
        (from, to),
        (
            GreenLabelStatus::PendingBondDeposit,
            GreenLabelStatus::PendingObservation
        ) | (
            GreenLabelStatus::PendingBondDeposit,
            GreenLabelStatus::Cancelled
        ) | (
            GreenLabelStatus::PendingObservation,
            GreenLabelStatus::ActiveGreenLabel
        ) | (
            GreenLabelStatus::PendingObservation,
            GreenLabelStatus::Disputed
        ) | (
            GreenLabelStatus::PendingObservation,
            GreenLabelStatus::RefundQueued
        ) | (
            GreenLabelStatus::ActiveGreenLabel,
            GreenLabelStatus::Disputed
        ) | (GreenLabelStatus::Disputed, GreenLabelStatus::SlashQueued)
            | (GreenLabelStatus::Disputed, GreenLabelStatus::RefundQueued)
            | (GreenLabelStatus::RefundQueued, GreenLabelStatus::Refunded)
            | (GreenLabelStatus::SlashQueued, GreenLabelStatus::Slashed)
    );

    require!(is_valid, CustomError::InvalidGreenLabelStatus);

    Ok(())
}

pub fn validate_payload_hash(payload_hash: [u8; 32]) -> Result<()> {
    require!(
        payload_hash != [0; 32],
        CustomError::InvalidGreenLabelPayloadHash
    );

    Ok(())
}

pub fn expected_green_label_config_space() -> usize {
    GREEN_LABEL_CONFIG_SPACE
}

pub fn expected_green_label_project_space() -> usize {
    GREEN_LABEL_PROJECT_SPACE
}

pub fn expected_green_label_dispute_space() -> usize {
    GREEN_LABEL_DISPUTE_SPACE
}

pub fn derive_bond_split_and_tier(
    total_bond_amount: u64,
    configured_min_base_bond_usdc: u64,
) -> Result<(GreenLabelBondSplit, BondTier)> {
    let split = split_green_label_bond(total_bond_amount, configured_min_base_bond_usdc)?;
    let tier = calculate_bond_tier(total_bond_amount, configured_min_base_bond_usdc)?;

    Ok((split, tier))
}

pub fn validate_terminal_action_for_refund(action_type: ActionType) -> Result<()> {
    require!(
        action_type == ActionType::GreenLabelRefund,
        CustomError::InvalidGreenLabelActionType
    );

    Ok(())
}

pub fn validate_terminal_action_for_slash(
    action_type: ActionType,
    has_linked_dispute: bool,
) -> Result<()> {
    require!(
        action_type == ActionType::GreenLabelSlash,
        CustomError::InvalidGreenLabelActionType
    );
    require!(
        has_linked_dispute,
        CustomError::InvalidGreenLabelSlashWithoutDispute
    );

    Ok(())
}

pub fn reject_legacy_green_label_slash_v1() -> Result<()> {
    err!(CustomError::LegacyGreenLabelSlashDisabled)
}

pub fn reject_legacy_green_label_forfeit_v1() -> Result<()> {
    err!(CustomError::LegacyGreenLabelForfeitDisabled)
}

pub fn reject_legacy_green_label_certification_fee_route_v1() -> Result<()> {
    err!(CustomError::LegacyGreenLabelCertificationFeeRouteDisabled)
}

fn calculate_bps_amount(amount: u64, bps: u16) -> Result<u64> {
    amount
        .checked_mul(bps as u64)
        .and_then(|value| value.checked_div(MAX_BPS as u64))
        .ok_or(CustomError::GreenLabelMathOverflow.into())
}

fn proposal_type_matches_action(proposal_type: ProposalType, action_type: ActionType) -> bool {
    matches!(
        (proposal_type, action_type),
        (ProposalType::GreenLabelSlash, ActionType::GreenLabelSlash)
            | (ProposalType::GreenLabelRefund, ActionType::GreenLabelRefund)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{
        ANCHOR_ACCOUNT_DISCRIMINATOR_BYTES, DEFAULT_DISPUTE_WINDOW_SECONDS,
        DEFAULT_OBSERVATION_PERIOD_SECONDS, DEFAULT_RESPONSE_WINDOW_SECONDS,
        GREEN_BOND_VAULT_AUTHORITY_SEED, GREEN_BOND_VAULT_SEED, GREEN_LABEL_CONFIG_RESERVED_BYTES,
        GREEN_LABEL_CONFIG_SEED, GREEN_LABEL_DISPUTE_RESERVED_BYTES, GREEN_LABEL_DISPUTE_SEED,
        GREEN_LABEL_PROJECT_RESERVED_BYTES, GREEN_LABEL_PROJECT_SEED,
    };
    use crate::instructions::deposit_usdc_revenue::{
        calculate_revenue_routing_stats_after_route, calculate_usdc_treasury_split,
    };
    use crate::state::{
        DisputeStatus, GreenLabelConfigV1, GreenLabelDisputeV1, GreenLabelProjectV1, RugReasonCode,
    };
    use anchor_lang::solana_program::program_option::COption;
    use anchor_lang::solana_program::program_pack::Pack;
    use anchor_lang::AccountDeserialize;
    use anchor_spl::token::spl_token::state::{Account as SplTokenAccount, AccountState};

    fn assert_error_contains(err: anchor_lang::error::Error, expected: &str) {
        let message = format!("{err:?}");
        assert!(
            message.contains(expected),
            "expected {expected}, got {message}"
        );
    }

    fn serialized_account_data<T: AccountSerialize>(account: &T, len: usize) -> Vec<u8> {
        let mut data = Vec::new();
        account.try_serialize(&mut data).unwrap();
        data.resize(len, 0);
        data
    }

    #[test]
    fn certification_execution_type_stable_code_roundtrips() {
        for execution_type in [
            GreenLabelCertificationExecutionTypeV1::Approve,
            GreenLabelCertificationExecutionTypeV1::Reject,
            GreenLabelCertificationExecutionTypeV1::Revoke,
        ] {
            let code = green_label_certification_execution_type_stable_code_v1(execution_type);
            assert_eq!(
                green_label_certification_execution_type_from_stable_code_v1(code).unwrap(),
                execution_type
            );
        }
    }

    #[test]
    fn certification_execution_type_unknown_code_fails() {
        let err = green_label_certification_execution_type_from_stable_code_v1(99).unwrap_err();
        assert_eq!(
            err,
            CustomError::InvalidGreenLabelCertificationSchema.into()
        );
    }

    #[test]
    fn certification_state_init_accepts_pending_bond_deposit() {
        let mut state = blank_certification_state();
        record_green_label_certification_state_init(
            &mut state,
            Pubkey::new_from_array([31; 32]),
            Pubkey::new_from_array([32; 32]),
            GreenLabelStatus::PendingBondDeposit,
            100,
            7,
        )
        .unwrap();

        assert_eq!(
            state.certification_status,
            GreenLabelCertificationStatusV1::Pending
        );
        assert_eq!(
            state.schema_version,
            GREEN_LABEL_CERTIFICATION_SCHEMA_VERSION
        );
        assert_eq!(state.bump, 7);
    }

    #[test]
    fn certification_state_init_accepts_pending_observation() {
        let mut state = blank_certification_state();
        record_green_label_certification_state_init(
            &mut state,
            Pubkey::new_from_array([31; 32]),
            Pubkey::new_from_array([32; 32]),
            GreenLabelStatus::PendingObservation,
            100,
            7,
        )
        .unwrap();

        assert_eq!(
            state.certification_status,
            GreenLabelCertificationStatusV1::Pending
        );
    }

    #[test]
    fn certification_state_init_rejects_active_green_label() {
        let mut state = blank_certification_state();
        let err = record_green_label_certification_state_init(
            &mut state,
            Pubkey::new_from_array([31; 32]),
            Pubkey::new_from_array([32; 32]),
            GreenLabelStatus::ActiveGreenLabel,
            100,
            7,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::InvalidGreenLabelStatus.into());
    }

    #[test]
    fn certification_parameters_hash_is_deterministic_and_field_bound() {
        let config = green_label_config();
        let project = green_label_project();
        let base = build_green_label_certification_decision_parameters_v1(
            &config,
            Pubkey::new_from_array([30; 32]),
            &project,
            Pubkey::new_from_array([31; 32]),
            Pubkey::new_from_array([32; 32]),
            GovernanceActionTypeV1::GreenLabelApproveCertification,
            GreenLabelCertificationStatusV1::Pending,
            7,
        )
        .unwrap();
        let base_hash = hash_green_label_certification_decision_parameters_v1(&base).unwrap();
        assert_eq!(
            base_hash,
            hash_green_label_certification_decision_parameters_v1(&base).unwrap()
        );

        let mut changed_action = base;
        changed_action.action_type = GovernanceActionTypeV1::GreenLabelRejectCertification;
        assert_ne!(
            base_hash,
            hash_green_label_certification_decision_parameters_v1(&changed_action).unwrap()
        );

        let mut changed_project = base;
        changed_project.green_label_project = Pubkey::new_from_array([33; 32]);
        assert_ne!(
            base_hash,
            hash_green_label_certification_decision_parameters_v1(&changed_project).unwrap()
        );

        let mut changed_state = base;
        changed_state.certification_state = Pubkey::new_from_array([34; 32]);
        assert_ne!(
            base_hash,
            hash_green_label_certification_decision_parameters_v1(&changed_state).unwrap()
        );

        let mut changed_authority = base;
        changed_authority.project_authority = Pubkey::new_from_array([35; 32]);
        assert_ne!(
            base_hash,
            hash_green_label_certification_decision_parameters_v1(&changed_authority).unwrap()
        );

        let mut changed_vault = base;
        changed_vault.bond_vault = Pubkey::new_from_array([36; 32]);
        assert_ne!(
            base_hash,
            hash_green_label_certification_decision_parameters_v1(&changed_vault).unwrap()
        );

        let mut changed_mint = base;
        changed_mint.usdc_mint = Pubkey::new_from_array([37; 32]);
        assert_ne!(
            base_hash,
            hash_green_label_certification_decision_parameters_v1(&changed_mint).unwrap()
        );

        let mut changed_observation = base;
        changed_observation.observation_end_ts += 1;
        assert_ne!(
            base_hash,
            hash_green_label_certification_decision_parameters_v1(&changed_observation).unwrap()
        );

        let mut changed_proposal = base;
        changed_proposal.proposal_id += 1;
        assert_ne!(
            base_hash,
            hash_green_label_certification_decision_parameters_v1(&changed_proposal).unwrap()
        );

        let mut wrong_domain_bytes = Vec::new();
        wrong_domain_bytes.extend_from_slice(b"wrong_green_label_certification_domain");
        base.serialize(&mut wrong_domain_bytes).unwrap();
        assert_ne!(
            base_hash,
            hash_contributor_payload(&wrong_domain_bytes).unwrap()
        );
    }

    #[test]
    fn certification_fee_policy_and_receipt_spaces_match_layout() {
        assert_eq!(GreenLabelCertificationFeePolicyV1::INIT_SPACE, 124);
        assert_eq!(8 + GreenLabelCertificationFeePolicyV1::INIT_SPACE, 132);
        assert_eq!(GreenLabelCertificationFeeReceiptV1::INIT_SPACE, 516);
        assert_eq!(8 + GreenLabelCertificationFeeReceiptV1::INIT_SPACE, 524);
    }

    #[test]
    fn certification_fee_policy_init_records_fixed_fields() {
        let config_key = Pubkey::new_from_array([30; 32]);
        let mint = Pubkey::new_from_array([2; 32]);
        let authority = Pubkey::new_from_array([1; 32]);
        let mut policy = blank_certification_fee_policy();

        record_green_label_certification_fee_policy_init_v1(
            &mut policy,
            config_key,
            mint,
            1_000_000,
            authority,
            100,
            7,
        )
        .unwrap();

        assert_eq!(policy.green_label_config, config_key);
        assert_eq!(policy.usdc_mint, mint);
        assert_eq!(policy.fee_amount_usdc, 1_000_000);
        assert_eq!(
            policy.policy_version,
            GREEN_LABEL_CERTIFICATION_FEE_POLICY_VERSION
        );
        assert!(policy.active);
        assert_eq!(
            policy.schema_version,
            GREEN_LABEL_CERTIFICATION_FEE_SCHEMA_VERSION
        );
        assert_eq!(policy.bump, 7);

        let err = record_green_label_certification_fee_policy_init_v1(
            &mut policy,
            config_key,
            mint,
            1_000_000,
            authority,
            101,
            8,
        )
        .unwrap_err();
        assert_eq!(
            err,
            CustomError::GreenLabelCertificationFeePolicyAlreadyInitialized.into()
        );
    }

    #[test]
    fn certification_fee_policy_init_rejects_wrong_authority_and_zero_amount() {
        let authority = Pubkey::new_from_array([1; 32]);
        let wrong_authority = Pubkey::new_from_array([9; 32]);
        let mint = Pubkey::new_from_array([2; 32]);

        let wrong_authority_err = validate_green_label_certification_fee_policy_init_v1(
            authority,
            wrong_authority,
            mint,
            1_000_000,
        )
        .unwrap_err();
        assert_eq!(
            wrong_authority_err,
            CustomError::UnauthorizedGreenLabelAuthority.into()
        );

        let zero_amount_err =
            validate_green_label_certification_fee_policy_init_v1(authority, authority, mint, 0)
                .unwrap_err();
        assert_eq!(
            zero_amount_err,
            CustomError::InvalidGreenLabelCertificationFeeAmount.into()
        );
    }

    #[test]
    fn certification_fee_parameters_hash_is_deterministic_and_field_bound() {
        let fixture = fee_validation_fixture();
        let parameters = fixture.parameters();
        let base_hash = hash_green_label_certification_fee_parameters_v1(&parameters).unwrap();
        assert_eq!(
            base_hash,
            hash_green_label_certification_fee_parameters_v1(&parameters).unwrap()
        );

        let mut changed_amount = parameters;
        changed_amount.fee_amount_usdc = 2_000_000;
        assert_ne!(
            base_hash,
            hash_green_label_certification_fee_parameters_v1(&changed_amount).unwrap()
        );

        let mut changed_payer = parameters;
        changed_payer.payer = Pubkey::new_from_array([99; 32]);
        assert_ne!(
            base_hash,
            hash_green_label_certification_fee_parameters_v1(&changed_payer).unwrap()
        );

        let mut wrong_revenue_type = parameters;
        wrong_revenue_type.revenue_type = RevenueType::PlatformRevenue;
        let err =
            hash_green_label_certification_fee_parameters_v1(&wrong_revenue_type).unwrap_err();
        assert_eq!(
            err,
            CustomError::GreenLabelCertificationFeeParametersMismatch.into()
        );

        let mut wrong_domain_bytes = Vec::new();
        wrong_domain_bytes.extend_from_slice(b"wrong_domain");
        parameters.serialize(&mut wrong_domain_bytes).unwrap();
        assert_ne!(
            base_hash,
            hash_contributor_payload(&wrong_domain_bytes).unwrap()
        );
    }

    #[test]
    fn certification_fee_once_validation_accepts_policy_project_and_treasury() {
        let fixture = fee_validation_fixture();
        let expected_hash =
            hash_green_label_certification_fee_parameters_v1(&fixture.parameters()).unwrap();

        assert_eq!(fixture.validate().unwrap(), expected_hash);
    }

    #[test]
    fn certification_fee_once_validation_rejects_wrong_payer_status_and_decimals() {
        let mut wrong_payer = fee_validation_fixture();
        wrong_payer.project.project_owner = Pubkey::new_from_array([90; 32]);
        let err = wrong_payer.validate().unwrap_err();
        assert_eq!(
            err,
            CustomError::GreenLabelCertificationFeePayerMismatch.into()
        );

        let mut wrong_status = fee_validation_fixture();
        wrong_status.project.status = GreenLabelStatus::PendingObservation;
        let err = wrong_status.validate().unwrap_err();
        assert_eq!(
            err,
            CustomError::GreenLabelCertificationFeeStatusMismatch.into()
        );

        let mut wrong_decimals = fee_validation_fixture();
        wrong_decimals.usdc_decimals = 9;
        let err = wrong_decimals.validate().unwrap_err();
        assert_eq!(
            err,
            CustomError::GreenLabelCertificationFeeDecimalsMismatch.into()
        );
    }

    #[test]
    fn certification_fee_once_validation_rejects_treasury_mismatch_and_vault_alias() {
        let mut treasury_mismatch = fee_validation_fixture();
        treasury_mismatch.config.treasury_usdc_state_v2 = Pubkey::new_from_array([91; 32]);
        let err = treasury_mismatch.validate().unwrap_err();
        assert_eq!(
            err,
            CustomError::GreenLabelCertificationFeeTreasuryMismatch.into()
        );

        let mut vault_alias = fee_validation_fixture();
        vault_alias.relief_vault_key = vault_alias.payer_token_account_key;
        let err = vault_alias.validate().unwrap_err();
        assert_eq!(
            err,
            CustomError::GreenLabelCertificationFeeTreasuryMismatch.into()
        );
    }

    #[test]
    fn certification_fee_receipt_records_once_and_preserves_escrow_liability() {
        let fixture = fee_validation_fixture();
        let parameters = fixture.parameters();
        let parameters_hash =
            hash_green_label_certification_fee_parameters_v1(&parameters).unwrap();
        let mut receipt = blank_certification_fee_receipt();
        let escrow = refundable_escrow(100, GreenLabelEscrowStatusV1::Locked);
        let escrow_before = (
            escrow.deposited_amount,
            escrow.refundable_amount,
            escrow.refunded_amount,
            escrow.forfeited_amount,
            escrow.status,
        );

        record_green_label_certification_fee_receipt_v1(
            &mut receipt,
            &parameters,
            parameters_hash,
            123,
            9,
            Pubkey::new_from_array([88; 32]),
        )
        .unwrap();

        assert_eq!(receipt.green_label_config, fixture.config_key);
        assert_eq!(receipt.green_label_project, fixture.project_key);
        assert_eq!(receipt.project_id, fixture.project.project_id);
        assert_eq!(receipt.payer, fixture.payer);
        assert_eq!(receipt.amount_usdc, fixture.policy.fee_amount_usdc);
        assert_eq!(
            receipt.revenue_type,
            RevenueType::GreenLabelCertificationFee
        );
        assert_eq!(receipt.parameters_hash, parameters_hash);
        assert_eq!(receipt.routed_at, 123);
        assert_eq!(receipt.bump, 9);
        assert_eq!(
            escrow_before,
            (
                escrow.deposited_amount,
                escrow.refundable_amount,
                escrow.refunded_amount,
                escrow.forfeited_amount,
                escrow.status,
            )
        );

        let mut changed_payer = parameters;
        changed_payer.payer = Pubkey::new_from_array([99; 32]);
        let err = record_green_label_certification_fee_receipt_v1(
            &mut receipt,
            &changed_payer,
            hash_green_label_certification_fee_parameters_v1(&changed_payer).unwrap(),
            124,
            10,
            Pubkey::new_from_array([88; 32]),
        )
        .unwrap_err();
        assert_eq!(
            err,
            CustomError::GreenLabelCertificationFeeAlreadyPaid.into()
        );
    }

    #[test]
    fn legacy_green_label_certification_fee_route_is_disabled() {
        let err = reject_legacy_green_label_certification_fee_route_v1().unwrap_err();
        assert_eq!(
            err,
            CustomError::LegacyGreenLabelCertificationFeeRouteDisabled.into()
        );
    }

    #[test]
    fn approve_certification_record_sets_project_and_state() {
        let config = green_label_config();
        let mut project = green_label_project();
        project.status = GreenLabelStatus::PendingObservation;
        project.observation_end_ts = 100;
        let project_key = Pubkey::new_from_array([31; 32]);
        let state_key = Pubkey::new_from_array([32; 32]);
        let mut state = certification_state(project_key, Pubkey::new_from_array([30; 32]));
        let mut record = blank_certification_execution_record();
        let parameters = build_green_label_certification_decision_parameters_v1(
            &config,
            Pubkey::new_from_array([30; 32]),
            &project,
            project_key,
            state_key,
            GovernanceActionTypeV1::GreenLabelApproveCertification,
            state.certification_status,
            7,
        )
        .unwrap();

        record_green_label_approve_certification_v1(
            &mut project,
            &mut state,
            &mut record,
            Pubkey::new_from_array([40; 32]),
            Pubkey::new_from_array([41; 32]),
            Pubkey::new_from_array([42; 32]),
            Pubkey::new_from_array([43; 32]),
            project_key,
            state_key,
            Pubkey::new_from_array([44; 32]),
            Pubkey::new_from_array([45; 32]),
            parameters,
            [99; 32],
            GreenLabelStatus::PendingObservation,
            GreenLabelCertificationStatusV1::Pending,
            Pubkey::new_from_array([46; 32]),
            200,
            9,
        )
        .unwrap();

        assert_eq!(project.status, GreenLabelStatus::ActiveGreenLabel);
        assert_eq!(project.approved_at, 200);
        assert_eq!(
            state.certification_status,
            GreenLabelCertificationStatusV1::Approved
        );
        assert_eq!(
            record.execution_type,
            GreenLabelCertificationExecutionTypeV1::Approve
        );
        assert_eq!(
            record.project_status_after,
            GreenLabelStatus::ActiveGreenLabel
        );
        assert_eq!(
            record.certification_status_after,
            GreenLabelCertificationStatusV1::Approved
        );
    }

    #[test]
    fn reject_certification_record_does_not_change_project_status() {
        let config = green_label_config();
        let mut project = green_label_project();
        project.status = GreenLabelStatus::PendingBondDeposit;
        let project_key = Pubkey::new_from_array([31; 32]);
        let state_key = Pubkey::new_from_array([32; 32]);
        let mut state = certification_state(project_key, Pubkey::new_from_array([30; 32]));
        let mut record = blank_certification_execution_record();
        let parameters = build_green_label_certification_decision_parameters_v1(
            &config,
            Pubkey::new_from_array([30; 32]),
            &project,
            project_key,
            state_key,
            GovernanceActionTypeV1::GreenLabelRejectCertification,
            state.certification_status,
            7,
        )
        .unwrap();

        record_green_label_certification_decision_v1(
            &mut state,
            &mut record,
            Pubkey::new_from_array([40; 32]),
            Pubkey::new_from_array([41; 32]),
            Pubkey::new_from_array([42; 32]),
            Pubkey::new_from_array([43; 32]),
            project_key,
            state_key,
            Pubkey::new_from_array([44; 32]),
            Pubkey::new_from_array([45; 32]),
            GreenLabelCertificationExecutionTypeV1::Reject,
            GreenLabelCertificationStatusV1::Rejected,
            GovernanceActionTypeV1::GreenLabelRejectCertification,
            parameters,
            [99; 32],
            GreenLabelStatus::PendingBondDeposit,
            GreenLabelStatus::PendingBondDeposit,
            GreenLabelCertificationStatusV1::Pending,
            Pubkey::new_from_array([46; 32]),
            200,
            9,
        )
        .unwrap();

        assert_eq!(project.status, GreenLabelStatus::PendingBondDeposit);
        assert_eq!(
            state.certification_status,
            GreenLabelCertificationStatusV1::Rejected
        );
        assert_eq!(
            record.project_status_after,
            GreenLabelStatus::PendingBondDeposit
        );
    }

    #[test]
    fn revoke_certification_record_requires_approved_state() {
        validate_green_label_revoke_certification_business_v1(
            GreenLabelStatus::ActiveGreenLabel,
            GreenLabelCertificationStatusV1::Approved,
        )
        .unwrap();

        let err = validate_green_label_revoke_certification_business_v1(
            GreenLabelStatus::ActiveGreenLabel,
            GreenLabelCertificationStatusV1::Pending,
        )
        .unwrap_err();
        assert_eq!(err, CustomError::GreenLabelCertificationNotApproved.into());
    }

    #[test]
    fn duplicate_certification_record_fails() {
        let config = green_label_config();
        let project = green_label_project();
        let project_key = Pubkey::new_from_array([31; 32]);
        let state_key = Pubkey::new_from_array([32; 32]);
        let mut state = certification_state(project_key, Pubkey::new_from_array([30; 32]));
        let mut record = blank_certification_execution_record();
        record.execution_queue_item = Pubkey::new_from_array([40; 32]);
        let parameters = build_green_label_certification_decision_parameters_v1(
            &config,
            Pubkey::new_from_array([30; 32]),
            &project,
            project_key,
            state_key,
            GovernanceActionTypeV1::GreenLabelRejectCertification,
            state.certification_status,
            7,
        )
        .unwrap();

        let err = record_green_label_certification_decision_v1(
            &mut state,
            &mut record,
            Pubkey::new_from_array([40; 32]),
            Pubkey::new_from_array([41; 32]),
            Pubkey::new_from_array([42; 32]),
            Pubkey::new_from_array([43; 32]),
            project_key,
            state_key,
            Pubkey::new_from_array([44; 32]),
            Pubkey::new_from_array([45; 32]),
            GreenLabelCertificationExecutionTypeV1::Reject,
            GreenLabelCertificationStatusV1::Rejected,
            GovernanceActionTypeV1::GreenLabelRejectCertification,
            parameters,
            [99; 32],
            project.status,
            project.status,
            GreenLabelCertificationStatusV1::Pending,
            Pubkey::new_from_array([46; 32]),
            200,
            9,
        )
        .unwrap_err();

        assert_eq!(
            err,
            CustomError::GreenLabelCertificationExecutionAlreadyCompleted.into()
        );
    }

    #[test]
    fn refund_execution_type_stable_code_roundtrips() {
        let code = green_label_escrow_execution_type_stable_code_v1(
            GreenLabelEscrowExecutionTypeV1::Refund,
        );
        assert_eq!(code, 1);
        assert_eq!(
            green_label_escrow_execution_type_from_stable_code_v1(code).unwrap(),
            GreenLabelEscrowExecutionTypeV1::Refund
        );
    }

    #[test]
    fn forfeit_execution_type_stable_code_roundtrips() {
        let code = green_label_escrow_execution_type_stable_code_v1(
            GreenLabelEscrowExecutionTypeV1::Forfeit,
        );
        assert_eq!(code, 2);
        assert_eq!(
            green_label_escrow_execution_type_from_stable_code_v1(code).unwrap(),
            GreenLabelEscrowExecutionTypeV1::Forfeit
        );
    }

    #[test]
    fn legacy_green_label_slash_entrypoint_is_disabled() {
        let err = reject_legacy_green_label_slash_v1().unwrap_err();

        assert_eq!(err, CustomError::LegacyGreenLabelSlashDisabled.into());
    }

    #[test]
    fn legacy_green_label_forfeit_entrypoint_is_disabled() {
        let err = reject_legacy_green_label_forfeit_v1().unwrap_err();

        assert_eq!(err, CustomError::LegacyGreenLabelForfeitDisabled.into());
    }

    #[test]
    fn refund_execution_type_unknown_code_fails() {
        let err = green_label_escrow_execution_type_from_stable_code_v1(99).unwrap_err();
        assert_eq!(err, CustomError::InvalidGreenLabelRefundSchema.into());
    }

    #[test]
    fn refund_parameters_hash_is_deterministic_and_field_bound() {
        let base = build_green_label_refund_parameters_v1(
            Pubkey::new_from_array([1; 32]),
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([3; 32]),
            Pubkey::new_from_array([4; 32]),
            Pubkey::new_from_array([5; 32]),
            Pubkey::new_from_array([6; 32]),
            Pubkey::new_from_array([7; 32]),
            100,
            Pubkey::new_from_array([8; 32]),
            GreenLabelEscrowStatusV1::Locked,
            9,
        )
        .unwrap();
        let base_hash = hash_green_label_refund_parameters_v1(&base).unwrap();
        assert_eq!(
            base_hash,
            hash_green_label_refund_parameters_v1(&base).unwrap()
        );

        let mut changed_project = base;
        changed_project.green_label_project = Pubkey::new_from_array([9; 32]);
        assert_ne!(
            base_hash,
            hash_green_label_refund_parameters_v1(&changed_project).unwrap()
        );

        let mut changed_dispute = base;
        changed_dispute.green_label_dispute = Pubkey::new_from_array([10; 32]);
        assert_ne!(
            base_hash,
            hash_green_label_refund_parameters_v1(&changed_dispute).unwrap()
        );

        let mut changed_escrow = base;
        changed_escrow.refundable_escrow = Pubkey::new_from_array([11; 32]);
        assert_ne!(
            base_hash,
            hash_green_label_refund_parameters_v1(&changed_escrow).unwrap()
        );

        let mut changed_vault = base;
        changed_vault.refundable_vault = Pubkey::new_from_array([12; 32]);
        assert_ne!(
            base_hash,
            hash_green_label_refund_parameters_v1(&changed_vault).unwrap()
        );

        let mut changed_payer = base;
        changed_payer.original_payer = Pubkey::new_from_array([13; 32]);
        assert_ne!(
            base_hash,
            hash_green_label_refund_parameters_v1(&changed_payer).unwrap()
        );

        let mut changed_destination = base;
        changed_destination.payer_destination_token_account = Pubkey::new_from_array([14; 32]);
        assert_ne!(
            base_hash,
            hash_green_label_refund_parameters_v1(&changed_destination).unwrap()
        );

        let mut changed_amount = base;
        changed_amount.refund_amount_usdc += 1;
        assert_ne!(
            base_hash,
            hash_green_label_refund_parameters_v1(&changed_amount).unwrap()
        );

        let mut changed_mint = base;
        changed_mint.usdc_mint = Pubkey::new_from_array([15; 32]);
        assert_ne!(
            base_hash,
            hash_green_label_refund_parameters_v1(&changed_mint).unwrap()
        );

        let mut changed_proposal = base;
        changed_proposal.proposal_id += 1;
        assert_ne!(
            base_hash,
            hash_green_label_refund_parameters_v1(&changed_proposal).unwrap()
        );

        let mut changed_action = base;
        changed_action.action_type = GovernanceActionTypeV1::GreenLabelSlashBond;
        assert!(hash_green_label_refund_parameters_v1(&changed_action).is_err());

        let mut wrong_domain_bytes = Vec::new();
        wrong_domain_bytes.extend_from_slice(b"wrong_green_label_refund_domain");
        base.serialize(&mut wrong_domain_bytes).unwrap();
        assert_ne!(
            base_hash,
            hash_contributor_payload(&wrong_domain_bytes).unwrap()
        );
    }

    #[test]
    fn strict_refund_amount_uses_recorded_amount_and_allows_vault_dust() {
        let escrow = refundable_escrow(100, GreenLabelEscrowStatusV1::Locked);

        assert_eq!(derive_green_label_refund_amount_v1(&escrow).unwrap(), 100);
        validate_green_label_refund_vault_balance_v1(100, 100).unwrap();
        validate_green_label_refund_vault_balance_v1(101, 100).unwrap();

        let err = validate_green_label_refund_vault_balance_v1(99, 100).unwrap_err();
        assert_eq!(err, CustomError::GreenLabelRefundInsufficientFunds.into());
    }

    #[test]
    fn refund_parameters_hash_uses_recorded_amount_not_vault_balance() {
        let escrow = refundable_escrow(100, GreenLabelEscrowStatusV1::Locked);
        let recorded_amount = derive_green_label_refund_amount_v1(&escrow).unwrap();
        validate_green_label_refund_vault_balance_v1(101, recorded_amount).unwrap();

        let parameters = build_green_label_refund_parameters_v1(
            Pubkey::new_from_array([1; 32]),
            Pubkey::new_from_array([2; 32]),
            Pubkey::default(),
            Pubkey::new_from_array([4; 32]),
            Pubkey::new_from_array([5; 32]),
            Pubkey::new_from_array([6; 32]),
            Pubkey::new_from_array([7; 32]),
            recorded_amount,
            Pubkey::new_from_array([8; 32]),
            GreenLabelEscrowStatusV1::Locked,
            9,
        )
        .unwrap();
        let base_hash = hash_green_label_refund_parameters_v1(&parameters).unwrap();

        let hash_after_dust = hash_green_label_refund_parameters_v1(&parameters).unwrap();
        assert_eq!(base_hash, hash_after_dust);

        let mut balance_as_amount = parameters;
        balance_as_amount.refund_amount_usdc = 101;
        assert_ne!(
            base_hash,
            hash_green_label_refund_parameters_v1(&balance_as_amount).unwrap()
        );
    }

    #[test]
    fn strict_refund_amount_rejects_refunded_and_forfeited() {
        let refunded = refundable_escrow(100, GreenLabelEscrowStatusV1::Refunded);
        let forfeited = refundable_escrow(100, GreenLabelEscrowStatusV1::Forfeited);

        assert_eq!(
            derive_green_label_refund_amount_v1(&refunded).unwrap_err(),
            CustomError::GreenLabelEscrowAlreadyRefunded.into()
        );
        assert_eq!(
            derive_green_label_refund_amount_v1(&forfeited).unwrap_err(),
            CustomError::GreenLabelEscrowAlreadyForfeited.into()
        );
    }

    #[test]
    fn strict_refund_amount_rejects_overflow_and_over_recorded_liability() {
        let mut overflow = refundable_escrow(u64::MAX, GreenLabelEscrowStatusV1::Locked);
        overflow.refunded_amount = u64::MAX;
        overflow.forfeited_amount = 1;
        assert_eq!(
            derive_green_label_refund_amount_v1(&overflow).unwrap_err(),
            CustomError::GreenLabelMathOverflow.into()
        );

        let mut over_recorded = refundable_escrow(100, GreenLabelEscrowStatusV1::Locked);
        over_recorded.deposited_amount = 99;
        assert_eq!(
            derive_green_label_refund_amount_v1(&over_recorded).unwrap_err(),
            CustomError::InvalidGreenLabelEscrowAmount.into()
        );
    }

    #[test]
    fn refund_governance_record_sets_project_and_receipt_no_dispute() {
        let mut project = green_label_project();
        project.status = GreenLabelStatus::ActiveGreenLabel;
        let mut record = blank_refund_execution_record();
        let parameters = build_green_label_refund_parameters_v1(
            Pubkey::new_from_array([1; 32]),
            Pubkey::new_from_array([2; 32]),
            Pubkey::default(),
            Pubkey::new_from_array([4; 32]),
            Pubkey::new_from_array([5; 32]),
            Pubkey::new_from_array([6; 32]),
            Pubkey::new_from_array([7; 32]),
            100,
            Pubkey::new_from_array([8; 32]),
            GreenLabelEscrowStatusV1::Locked,
            9,
        )
        .unwrap();

        record_green_label_refund_governance_v1(
            &mut project,
            None,
            &mut record,
            Pubkey::new_from_array([20; 32]),
            Pubkey::new_from_array([21; 32]),
            Pubkey::new_from_array([22; 32]),
            Pubkey::new_from_array([23; 32]),
            Pubkey::new_from_array([24; 32]),
            parameters.green_label_config,
            parameters.green_label_project,
            Pubkey::default(),
            parameters.refundable_escrow,
            parameters.refundable_vault,
            parameters.original_payer,
            parameters.payer_destination_token_account,
            parameters.refund_amount_usdc,
            parameters.usdc_mint,
            parameters,
            [55; 32],
            GreenLabelEscrowStatusV1::Locked,
            GreenLabelStatus::ActiveGreenLabel,
            Pubkey::new_from_array([25; 32]),
            200,
            7,
            Pubkey::new_from_array([26; 32]),
        )
        .unwrap();

        assert_eq!(project.status, GreenLabelStatus::Refunded);
        assert_eq!(project.refunded_at, 200);
        assert_eq!(project.active_dispute, Pubkey::default());
        assert_eq!(
            record.execution_type,
            GreenLabelEscrowExecutionTypeV1::Refund
        );
        assert_eq!(record.refund_amount_usdc, 100);
        assert_eq!(
            record.escrow_status_after,
            GreenLabelEscrowStatusV1::Refunded
        );
        assert_eq!(record.project_status_after, GreenLabelStatus::Refunded);
    }

    #[test]
    fn refund_governance_record_sets_dispute_resolved_refund() {
        let mut project = green_label_project();
        project.status = GreenLabelStatus::Disputed;
        project.active_dispute = Pubkey::new_from_array([3; 32]);
        let mut dispute = green_label_dispute();
        dispute.status = DisputeStatus::ReadyForDecision;
        let mut record = blank_refund_execution_record();
        let parameters = build_green_label_refund_parameters_v1(
            Pubkey::new_from_array([1; 32]),
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([3; 32]),
            Pubkey::new_from_array([4; 32]),
            Pubkey::new_from_array([5; 32]),
            Pubkey::new_from_array([6; 32]),
            Pubkey::new_from_array([7; 32]),
            100,
            Pubkey::new_from_array([8; 32]),
            GreenLabelEscrowStatusV1::Locked,
            9,
        )
        .unwrap();

        record_green_label_refund_governance_v1(
            &mut project,
            Some(&mut dispute),
            &mut record,
            Pubkey::new_from_array([20; 32]),
            Pubkey::new_from_array([21; 32]),
            Pubkey::new_from_array([22; 32]),
            Pubkey::new_from_array([23; 32]),
            Pubkey::new_from_array([24; 32]),
            parameters.green_label_config,
            parameters.green_label_project,
            parameters.green_label_dispute,
            parameters.refundable_escrow,
            parameters.refundable_vault,
            parameters.original_payer,
            parameters.payer_destination_token_account,
            parameters.refund_amount_usdc,
            parameters.usdc_mint,
            parameters,
            [55; 32],
            GreenLabelEscrowStatusV1::Locked,
            GreenLabelStatus::Disputed,
            Pubkey::new_from_array([25; 32]),
            200,
            7,
            Pubkey::new_from_array([26; 32]),
        )
        .unwrap();

        assert_eq!(project.status, GreenLabelStatus::Refunded);
        assert_eq!(dispute.status, DisputeStatus::ResolvedRefund);
        assert_eq!(dispute.action_type, ActionType::GreenLabelRefund);
        assert_eq!(record.green_label_dispute, Pubkey::new_from_array([3; 32]));
    }

    #[test]
    fn duplicate_refund_record_fails() {
        let mut project = green_label_project();
        let mut record = blank_refund_execution_record();
        record.execution_queue_item = Pubkey::new_from_array([20; 32]);
        let parameters = build_green_label_refund_parameters_v1(
            Pubkey::new_from_array([1; 32]),
            Pubkey::new_from_array([2; 32]),
            Pubkey::default(),
            Pubkey::new_from_array([4; 32]),
            Pubkey::new_from_array([5; 32]),
            Pubkey::new_from_array([6; 32]),
            Pubkey::new_from_array([7; 32]),
            100,
            Pubkey::new_from_array([8; 32]),
            GreenLabelEscrowStatusV1::Locked,
            9,
        )
        .unwrap();

        let err = record_green_label_refund_governance_v1(
            &mut project,
            None,
            &mut record,
            Pubkey::new_from_array([20; 32]),
            Pubkey::new_from_array([21; 32]),
            Pubkey::new_from_array([22; 32]),
            Pubkey::new_from_array([23; 32]),
            Pubkey::new_from_array([24; 32]),
            parameters.green_label_config,
            parameters.green_label_project,
            Pubkey::default(),
            parameters.refundable_escrow,
            parameters.refundable_vault,
            parameters.original_payer,
            parameters.payer_destination_token_account,
            parameters.refund_amount_usdc,
            parameters.usdc_mint,
            parameters,
            [55; 32],
            GreenLabelEscrowStatusV1::Locked,
            GreenLabelStatus::ActiveGreenLabel,
            Pubkey::new_from_array([25; 32]),
            200,
            7,
            Pubkey::new_from_array([26; 32]),
        )
        .unwrap_err();

        assert_eq!(
            err,
            CustomError::GreenLabelRefundExecutionAlreadyCompleted.into()
        );
    }

    #[test]
    fn forfeit_parameters_hash_is_deterministic_and_field_bound() {
        let base = build_green_label_forfeit_parameters_v1(
            Pubkey::new_from_array([1; 32]),
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([3; 32]),
            Pubkey::new_from_array([4; 32]),
            Pubkey::new_from_array([5; 32]),
            100,
            Pubkey::new_from_array([6; 32]),
            Pubkey::new_from_array([7; 32]),
            Pubkey::new_from_array([8; 32]),
            Pubkey::new_from_array([9; 32]),
            Pubkey::new_from_array([10; 32]),
            Pubkey::new_from_array([11; 32]),
            Pubkey::new_from_array([12; 32]),
            Pubkey::new_from_array([13; 32]),
            GreenLabelEscrowStatusV1::Locked,
            GreenLabelStatus::Disputed,
            DisputeStatus::ReadyForDecision,
            14,
        )
        .unwrap();
        let base_hash = hash_green_label_forfeit_parameters_v1(&base).unwrap();
        assert_eq!(
            base_hash,
            hash_green_label_forfeit_parameters_v1(&base).unwrap()
        );

        let mut changed_project = base;
        changed_project.green_label_project = Pubkey::new_from_array([15; 32]);
        assert_ne!(
            base_hash,
            hash_green_label_forfeit_parameters_v1(&changed_project).unwrap()
        );

        let mut changed_dispute = base;
        changed_dispute.green_label_dispute = Pubkey::new_from_array([16; 32]);
        assert_ne!(
            base_hash,
            hash_green_label_forfeit_parameters_v1(&changed_dispute).unwrap()
        );

        let mut changed_escrow = base;
        changed_escrow.refundable_escrow = Pubkey::new_from_array([17; 32]);
        assert_ne!(
            base_hash,
            hash_green_label_forfeit_parameters_v1(&changed_escrow).unwrap()
        );

        let mut changed_vault = base;
        changed_vault.refundable_vault = Pubkey::new_from_array([18; 32]);
        assert_ne!(
            base_hash,
            hash_green_label_forfeit_parameters_v1(&changed_vault).unwrap()
        );

        let mut changed_amount = base;
        changed_amount.forfeited_amount_usdc += 1;
        assert_ne!(
            base_hash,
            hash_green_label_forfeit_parameters_v1(&changed_amount).unwrap()
        );

        let mut changed_treasury = base;
        changed_treasury.treasury_usdc_state = Pubkey::new_from_array([19; 32]);
        assert_ne!(
            base_hash,
            hash_green_label_forfeit_parameters_v1(&changed_treasury).unwrap()
        );

        let mut changed_mint = base;
        changed_mint.usdc_mint = Pubkey::new_from_array([20; 32]);
        assert_ne!(
            base_hash,
            hash_green_label_forfeit_parameters_v1(&changed_mint).unwrap()
        );

        let mut changed_revenue_type = base;
        changed_revenue_type.revenue_type = RevenueType::ProtocolServiceFee;
        assert!(hash_green_label_forfeit_parameters_v1(&changed_revenue_type).is_err());

        let mut changed_action = base;
        changed_action.action_type = GovernanceActionTypeV1::GreenLabelRefundBond;
        assert!(hash_green_label_forfeit_parameters_v1(&changed_action).is_err());

        let mut changed_proposal = base;
        changed_proposal.proposal_id += 1;
        assert_ne!(
            base_hash,
            hash_green_label_forfeit_parameters_v1(&changed_proposal).unwrap()
        );

        let mut wrong_domain_bytes = Vec::new();
        wrong_domain_bytes.extend_from_slice(b"wrong_green_label_forfeit_domain");
        base.serialize(&mut wrong_domain_bytes).unwrap();
        assert_ne!(
            base_hash,
            hash_contributor_payload(&wrong_domain_bytes).unwrap()
        );
    }

    #[test]
    fn strict_forfeit_amount_uses_recorded_amount_and_allows_vault_dust() {
        let escrow = refundable_escrow(100, GreenLabelEscrowStatusV1::Locked);

        assert_eq!(
            derive_green_label_forfeitable_amount_v1(&escrow).unwrap(),
            100
        );
        validate_green_label_forfeit_vault_balance_v1(100, 100).unwrap();
        validate_green_label_forfeit_vault_balance_v1(101, 100).unwrap();

        let err = validate_green_label_forfeit_vault_balance_v1(99, 100).unwrap_err();
        assert_eq!(err, CustomError::GreenLabelForfeitInsufficientFunds.into());
    }

    #[test]
    fn forfeit_parameters_hash_uses_recorded_amount_not_vault_balance() {
        let escrow = refundable_escrow(100, GreenLabelEscrowStatusV1::Locked);
        let recorded_amount = derive_green_label_forfeitable_amount_v1(&escrow).unwrap();
        validate_green_label_forfeit_vault_balance_v1(101, recorded_amount).unwrap();

        let parameters = build_green_label_forfeit_parameters_v1(
            Pubkey::new_from_array([1; 32]),
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([3; 32]),
            Pubkey::new_from_array([4; 32]),
            Pubkey::new_from_array([5; 32]),
            recorded_amount,
            Pubkey::new_from_array([6; 32]),
            Pubkey::new_from_array([7; 32]),
            Pubkey::new_from_array([8; 32]),
            Pubkey::new_from_array([9; 32]),
            Pubkey::new_from_array([10; 32]),
            Pubkey::new_from_array([11; 32]),
            Pubkey::new_from_array([12; 32]),
            Pubkey::new_from_array([13; 32]),
            GreenLabelEscrowStatusV1::Locked,
            GreenLabelStatus::Disputed,
            DisputeStatus::ReadyForDecision,
            14,
        )
        .unwrap();
        let base_hash = hash_green_label_forfeit_parameters_v1(&parameters).unwrap();

        let hash_after_dust = hash_green_label_forfeit_parameters_v1(&parameters).unwrap();
        assert_eq!(base_hash, hash_after_dust);

        let mut balance_as_amount = parameters;
        balance_as_amount.forfeited_amount_usdc = 101;
        assert_ne!(
            base_hash,
            hash_green_label_forfeit_parameters_v1(&balance_as_amount).unwrap()
        );
    }

    #[test]
    fn strict_forfeit_amount_rejects_refunded_and_forfeited() {
        let refunded = refundable_escrow(100, GreenLabelEscrowStatusV1::Refunded);
        let forfeited = refundable_escrow(100, GreenLabelEscrowStatusV1::Forfeited);

        assert_eq!(
            derive_green_label_forfeitable_amount_v1(&refunded).unwrap_err(),
            CustomError::GreenLabelEscrowAlreadyRefunded.into()
        );
        assert_eq!(
            derive_green_label_forfeitable_amount_v1(&forfeited).unwrap_err(),
            CustomError::GreenLabelEscrowAlreadyForfeited.into()
        );
    }

    #[test]
    fn strict_forfeit_amount_rejects_overflow_and_over_recorded_liability() {
        let mut overflow = refundable_escrow(u64::MAX, GreenLabelEscrowStatusV1::Locked);
        overflow.refunded_amount = u64::MAX;
        overflow.forfeited_amount = 1;
        assert_eq!(
            derive_green_label_forfeitable_amount_v1(&overflow).unwrap_err(),
            CustomError::GreenLabelMathOverflow.into()
        );

        let mut over_recorded = refundable_escrow(100, GreenLabelEscrowStatusV1::Locked);
        over_recorded.deposited_amount = 99;
        assert_eq!(
            derive_green_label_forfeitable_amount_v1(&over_recorded).unwrap_err(),
            CustomError::InvalidGreenLabelEscrowAmount.into()
        );
    }

    #[test]
    fn forfeit_governance_record_sets_escrow_project_dispute_and_receipt() {
        let mut project = green_label_project();
        project.status = GreenLabelStatus::SlashQueued;
        project.active_dispute = Pubkey::new_from_array([3; 32]);
        let mut dispute = green_label_dispute();
        dispute.status = DisputeStatus::DecisionQueued;
        let mut escrow = refundable_escrow(100, GreenLabelEscrowStatusV1::Locked);
        let mut record = blank_forfeit_execution_record();
        let parameters = build_green_label_forfeit_parameters_v1(
            Pubkey::new_from_array([1; 32]),
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([3; 32]),
            Pubkey::new_from_array([4; 32]),
            Pubkey::new_from_array([5; 32]),
            100,
            Pubkey::new_from_array([6; 32]),
            Pubkey::new_from_array([7; 32]),
            Pubkey::new_from_array([8; 32]),
            Pubkey::new_from_array([9; 32]),
            Pubkey::new_from_array([10; 32]),
            Pubkey::new_from_array([11; 32]),
            Pubkey::new_from_array([12; 32]),
            Pubkey::new_from_array([13; 32]),
            GreenLabelEscrowStatusV1::Locked,
            GreenLabelStatus::SlashQueued,
            DisputeStatus::DecisionQueued,
            14,
        )
        .unwrap();

        record_green_label_forfeit_governance_v1(
            &mut project,
            &mut dispute,
            &mut escrow,
            &mut record,
            Pubkey::new_from_array([20; 32]),
            Pubkey::new_from_array([21; 32]),
            Pubkey::new_from_array([22; 32]),
            Pubkey::new_from_array([23; 32]),
            Pubkey::new_from_array([24; 32]),
            parameters.green_label_config,
            parameters.green_label_project,
            parameters.green_label_dispute,
            parameters.refundable_escrow,
            parameters.refundable_vault,
            parameters.treasury_config,
            parameters.treasury_usdc_state,
            parameters.revenue_routing_stats,
            parameters.relief_usdc_vault,
            parameters.buyback_usdc_vault,
            parameters.builders_usdc_vault,
            parameters.staking_usdc_vault,
            parameters.forfeited_amount_usdc,
            parameters.usdc_mint,
            parameters,
            [55; 32],
            GreenLabelEscrowStatusV1::Locked,
            GreenLabelStatus::SlashQueued,
            DisputeStatus::DecisionQueued,
            Pubkey::new_from_array([25; 32]),
            200,
            7,
            Pubkey::new_from_array([26; 32]),
        )
        .unwrap();

        assert_eq!(escrow.status, GreenLabelEscrowStatusV1::Forfeited);
        assert_eq!(escrow.forfeited_amount, 100);
        assert_eq!(project.status, GreenLabelStatus::Slashed);
        assert_eq!(project.slashed_at, 200);
        assert_eq!(project.active_dispute, Pubkey::default());
        assert_eq!(dispute.status, DisputeStatus::ResolvedSlash);
        assert_eq!(dispute.action_type, ActionType::GreenLabelSlash);
        assert_eq!(record.forfeited_amount_usdc, 100);
        assert_eq!(record.revenue_type, RevenueType::GreenLabelForfeitedBond);
        assert_eq!(
            record.execution_type,
            GreenLabelEscrowExecutionTypeV1::Forfeit
        );
        assert_eq!(
            record.governance_action_type,
            GovernanceActionTypeV1::GreenLabelSlashBond
        );
        assert_eq!(
            record.escrow_status_after,
            GreenLabelEscrowStatusV1::Forfeited
        );
        assert_eq!(record.project_status_after, GreenLabelStatus::Slashed);
        assert_eq!(record.dispute_status_after, DisputeStatus::ResolvedSlash);
    }

    #[test]
    fn duplicate_forfeit_record_fails() {
        let mut project = green_label_project();
        let mut dispute = green_label_dispute();
        let mut escrow = refundable_escrow(100, GreenLabelEscrowStatusV1::Locked);
        let mut record = blank_forfeit_execution_record();
        record.execution_queue_item = Pubkey::new_from_array([20; 32]);
        let parameters = build_green_label_forfeit_parameters_v1(
            Pubkey::new_from_array([1; 32]),
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([3; 32]),
            Pubkey::new_from_array([4; 32]),
            Pubkey::new_from_array([5; 32]),
            100,
            Pubkey::new_from_array([6; 32]),
            Pubkey::new_from_array([7; 32]),
            Pubkey::new_from_array([8; 32]),
            Pubkey::new_from_array([9; 32]),
            Pubkey::new_from_array([10; 32]),
            Pubkey::new_from_array([11; 32]),
            Pubkey::new_from_array([12; 32]),
            Pubkey::new_from_array([13; 32]),
            GreenLabelEscrowStatusV1::Locked,
            GreenLabelStatus::SlashQueued,
            DisputeStatus::DecisionQueued,
            14,
        )
        .unwrap();

        let err = record_green_label_forfeit_governance_v1(
            &mut project,
            &mut dispute,
            &mut escrow,
            &mut record,
            Pubkey::new_from_array([20; 32]),
            Pubkey::new_from_array([21; 32]),
            Pubkey::new_from_array([22; 32]),
            Pubkey::new_from_array([23; 32]),
            Pubkey::new_from_array([24; 32]),
            parameters.green_label_config,
            parameters.green_label_project,
            parameters.green_label_dispute,
            parameters.refundable_escrow,
            parameters.refundable_vault,
            parameters.treasury_config,
            parameters.treasury_usdc_state,
            parameters.revenue_routing_stats,
            parameters.relief_usdc_vault,
            parameters.buyback_usdc_vault,
            parameters.builders_usdc_vault,
            parameters.staking_usdc_vault,
            parameters.forfeited_amount_usdc,
            parameters.usdc_mint,
            parameters,
            [55; 32],
            GreenLabelEscrowStatusV1::Locked,
            GreenLabelStatus::SlashQueued,
            DisputeStatus::DecisionQueued,
            Pubkey::new_from_array([25; 32]),
            200,
            7,
            Pubkey::new_from_array([26; 32]),
        )
        .unwrap_err();

        assert_eq!(
            err,
            CustomError::GreenLabelForfeitExecutionAlreadyCompleted.into()
        );
    }

    #[test]
    fn green_label_forfeit_execution_record_space_matches_layout() {
        let expected = (32 * 19) + 8 + (32 * 2) + 9 + 8 + 2 + 1;

        assert_eq!(GreenLabelForfeitExecutionRecordV1::INIT_SPACE, expected);
    }

    #[test]
    fn unchecked_treasury_usdc_state_exit_persists_mutations() {
        let key = Pubkey::new_from_array([91; 32]);
        let owner = crate::ID;
        let mut lamports = 1;
        let initial = TreasuryUsdcStateV2 {
            total_usdc_inflow: 0,
            relief_usdc_total: 0,
            buyback_usdc_total: 0,
            builders_usdc_total: 0,
            staking_usdc_total: 0,
            bump: 7,
        };
        let mut data = serialized_account_data(&initial, 8 + TreasuryUsdcStateV2::INIT_SPACE);
        let account_info =
            AccountInfo::new(&key, false, true, &mut lamports, &mut data, &owner, false);

        {
            let mut account = Account::<TreasuryUsdcStateV2>::try_from(&account_info).unwrap();
            account.total_usdc_inflow = 100;
            account.relief_usdc_total = 50;
            account.buyback_usdc_total = 20;
            account.builders_usdc_total = 20;
            account.staking_usdc_total = 10;
            account.exit(&owner).unwrap();
        }

        let reloaded = Account::<TreasuryUsdcStateV2>::try_from(&account_info).unwrap();
        assert_eq!(reloaded.total_usdc_inflow, 100);
        assert_eq!(reloaded.relief_usdc_total, 50);
        assert_eq!(reloaded.buyback_usdc_total, 20);
        assert_eq!(reloaded.builders_usdc_total, 20);
        assert_eq!(reloaded.staking_usdc_total, 10);
        assert_eq!(reloaded.bump, 7);
    }

    #[test]
    fn unchecked_revenue_routing_stats_exit_persists_mutations() {
        let key = Pubkey::new_from_array([92; 32]);
        let owner = crate::ID;
        let authority = Pubkey::new_from_array([93; 32]);
        let usdc_mint = Pubkey::new_from_array([94; 32]);
        let mut lamports = 1;
        let initial = RevenueRoutingStatsV1 {
            authority,
            usdc_mint,
            total_routed_usdc: 0,
            green_label_certification_fee_total: 0,
            green_label_forfeited_bond_total: 0,
            protocol_service_fee_total: 0,
            platform_revenue_total: 0,
            partnership_revenue_total: 0,
            manual_governance_approved_revenue_total: 0,
            bump: 8,
        };
        let mut data = serialized_account_data(&initial, 8 + RevenueRoutingStatsV1::INIT_SPACE);
        let account_info =
            AccountInfo::new(&key, false, true, &mut lamports, &mut data, &owner, false);

        {
            let mut account = Account::<RevenueRoutingStatsV1>::try_from(&account_info).unwrap();
            account.total_routed_usdc = 100;
            account.green_label_forfeited_bond_total = 100;
            account.platform_revenue_total = 7;
            account.exit(&owner).unwrap();
        }

        let reloaded = Account::<RevenueRoutingStatsV1>::try_from(&account_info).unwrap();
        assert_eq!(reloaded.authority, authority);
        assert_eq!(reloaded.usdc_mint, usdc_mint);
        assert_eq!(reloaded.total_routed_usdc, 100);
        assert_eq!(reloaded.green_label_forfeited_bond_total, 100);
        assert_eq!(reloaded.platform_revenue_total, 7);
        assert_eq!(reloaded.bump, 8);
    }

    #[test]
    fn forfeit_mint_accounts_validate_correct_decimals_and_reject_wrong_decimals() {
        let escrow_key = Pubkey::new_from_array([1; 32]);
        let usdc_mint = Pubkey::new_from_array([2; 32]);

        validate_green_label_forfeit_mint_accounts_v1(
            usdc_mint,
            escrow_key,
            usdc_mint,
            escrow_key,
            usdc_mint,
            GREEN_LABEL_USDC_DECIMALS,
        )
        .unwrap();

        let treasury_state = TreasuryUsdcStateV2 {
            total_usdc_inflow: 0,
            relief_usdc_total: 0,
            buyback_usdc_total: 0,
            builders_usdc_total: 0,
            staking_usdc_total: 0,
            bump: 1,
        };
        let routing_stats = RevenueRoutingStatsV1 {
            authority: Pubkey::new_from_array([3; 32]),
            usdc_mint,
            total_routed_usdc: 0,
            green_label_certification_fee_total: 0,
            green_label_forfeited_bond_total: 0,
            protocol_service_fee_total: 0,
            platform_revenue_total: 0,
            partnership_revenue_total: 0,
            manual_governance_approved_revenue_total: 0,
            bump: 2,
        };
        let record = blank_forfeit_execution_record();

        let err = validate_green_label_forfeit_mint_accounts_v1(
            usdc_mint,
            escrow_key,
            usdc_mint,
            escrow_key,
            usdc_mint,
            GREEN_LABEL_USDC_DECIMALS + 1,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::GreenLabelForfeitMintMismatch.into());
        assert_eq!(treasury_state.total_usdc_inflow, 0);
        assert_eq!(treasury_state.relief_usdc_total, 0);
        assert_eq!(treasury_state.buyback_usdc_total, 0);
        assert_eq!(treasury_state.builders_usdc_total, 0);
        assert_eq!(treasury_state.staking_usdc_total, 0);
        assert_eq!(routing_stats.total_routed_usdc, 0);
        assert_eq!(routing_stats.green_label_forfeited_bond_total, 0);
        assert_eq!(record.executed_at, 0);
        assert_eq!(record.forfeited_amount_usdc, 0);
    }

    #[test]
    fn forfeit_router_accounting_is_single_split_for_green_label_forfeited_bond() {
        let amount = 101;
        let split = calculate_usdc_treasury_split(amount).unwrap();
        assert_eq!(
            split.relief + split.buyback + split.builders + split.staking,
            amount
        );
        assert_eq!(split.relief, 50);
        assert_eq!(split.buyback, 20);
        assert_eq!(split.builders, 20);
        assert_eq!(split.staking, 11);

        let routing_stats = RevenueRoutingStatsV1 {
            authority: Pubkey::new_from_array([1; 32]),
            usdc_mint: Pubkey::new_from_array([2; 32]),
            total_routed_usdc: 0,
            green_label_certification_fee_total: 0,
            green_label_forfeited_bond_total: 0,
            protocol_service_fee_total: 0,
            platform_revenue_total: 0,
            partnership_revenue_total: 0,
            manual_governance_approved_revenue_total: 0,
            bump: 1,
        };
        let totals = calculate_revenue_routing_stats_after_route(
            &routing_stats,
            RevenueType::GreenLabelForfeitedBond,
            amount,
        )
        .unwrap();

        assert_eq!(totals.total_routed_usdc, amount);
        assert_eq!(totals.green_label_forfeited_bond_total, amount);
        assert_eq!(totals.green_label_certification_fee_total, 0);
        assert_eq!(totals.protocol_service_fee_total, 0);
        assert_eq!(totals.platform_revenue_total, 0);
        assert_eq!(totals.partnership_revenue_total, 0);
        assert_eq!(totals.manual_governance_approved_revenue_total, 0);

        let duplicate_totals = calculate_revenue_routing_stats_after_route(
            &RevenueRoutingStatsV1 {
                total_routed_usdc: totals.total_routed_usdc,
                green_label_forfeited_bond_total: totals.green_label_forfeited_bond_total,
                ..routing_stats
            },
            RevenueType::GreenLabelForfeitedBond,
            amount,
        )
        .unwrap();
        assert_eq!(duplicate_totals.total_routed_usdc, amount * 2);
        assert_ne!(
            totals.green_label_forfeited_bond_total,
            duplicate_totals.green_label_forfeited_bond_total
        );
    }

    #[test]
    fn validate_bps_config_accepts_80_20() {
        validate_green_label_bps_config(BASE_BOND_REFUND_BPS, BASE_BOND_TREASURY_BPS).unwrap();
    }

    #[test]
    fn initialize_green_label_config_defaults_match_constants() {
        let values = green_label_config_init_values();

        assert_eq!(values.min_base_bond_usdc, MIN_GREEN_LABEL_BASE_BOND_USDC);
        assert_eq!(values.base_refund_bps, BASE_BOND_REFUND_BPS);
        assert_eq!(values.base_treasury_bps, BASE_BOND_TREASURY_BPS);
        assert_eq!(
            values.observation_period_seconds,
            DEFAULT_OBSERVATION_PERIOD_SECONDS
        );
        assert_eq!(
            values.dispute_window_seconds,
            DEFAULT_DISPUTE_WINDOW_SECONDS
        );
        assert_eq!(
            values.response_window_seconds,
            DEFAULT_RESPONSE_WINDOW_SECONDS
        );
    }

    #[test]
    fn initialize_green_label_config_uses_zero_project_count() {
        let values = green_label_config_init_values();

        assert_eq!(values.project_count, 0);
    }

    #[test]
    fn initialize_green_label_config_uses_unpaused_default() {
        let values = green_label_config_init_values();

        assert!(!values.is_paused);
    }

    #[test]
    fn initialize_green_label_config_reserved_zeroed() {
        let values = green_label_config_init_values();

        assert_eq!(values.reserved, [0; GREEN_LABEL_CONFIG_RESERVED_BYTES]);
    }

    #[test]
    fn accepts_valid_window_update() {
        let authority = Pubkey::new_from_array([1; 32]);

        validate_green_label_window_update(false, authority, authority, 60, 60, 60).unwrap();
    }

    #[test]
    fn rejects_paused_config() {
        let authority = Pubkey::new_from_array([1; 32]);
        let err =
            validate_green_label_window_update(true, authority, authority, 60, 60, 60).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn rejects_wrong_authority() {
        let err = validate_green_label_window_update(
            false,
            Pubkey::new_from_array([1; 32]),
            Pubkey::new_from_array([2; 32]),
            60,
            60,
            60,
        )
        .unwrap_err();

        assert_error_contains(err, "UnauthorizedGreenLabelAuthority");
    }

    #[test]
    fn rejects_zero_observation_window() {
        let authority = Pubkey::new_from_array([1; 32]);
        let err =
            validate_green_label_window_update(false, authority, authority, 0, 60, 60).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelWindowConfig");
    }

    #[test]
    fn rejects_zero_dispute_window() {
        let authority = Pubkey::new_from_array([1; 32]);
        let err =
            validate_green_label_window_update(false, authority, authority, 60, 0, 60).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelWindowConfig");
    }

    #[test]
    fn rejects_zero_response_window() {
        let authority = Pubkey::new_from_array([1; 32]);
        let err =
            validate_green_label_window_update(false, authority, authority, 60, 60, 0).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelWindowConfig");
    }

    #[test]
    fn rejects_observation_above_max() {
        let authority = Pubkey::new_from_array([1; 32]);
        let err = validate_green_label_window_update(
            false,
            authority,
            authority,
            MAX_GREEN_LABEL_WINDOW_SECONDS + 1,
            60,
            60,
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelWindowConfig");
    }

    #[test]
    fn rejects_dispute_above_max() {
        let authority = Pubkey::new_from_array([1; 32]);
        let err = validate_green_label_window_update(
            false,
            authority,
            authority,
            60,
            MAX_GREEN_LABEL_WINDOW_SECONDS + 1,
            60,
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelWindowConfig");
    }

    #[test]
    fn rejects_response_above_max() {
        let authority = Pubkey::new_from_array([1; 32]);
        let err = validate_green_label_window_update(
            false,
            authority,
            authority,
            60,
            60,
            MAX_GREEN_LABEL_WINDOW_SECONDS + 1,
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelWindowConfig");
    }

    #[test]
    fn record_window_update_changes_only_windows() {
        let expected = green_label_config();
        let mut actual = green_label_config();

        record_green_label_window_update(&mut actual, 60, 70, 80);

        assert_eq!(actual.observation_period_seconds, 60);
        assert_eq!(actual.dispute_window_seconds, 70);
        assert_eq!(actual.response_window_seconds, 80);
        assert_eq!(actual.authority, expected.authority);
        assert_eq!(actual.usdc_mint, expected.usdc_mint);
        assert_eq!(actual.min_base_bond_usdc, expected.min_base_bond_usdc);
        assert_eq!(actual.base_refund_bps, expected.base_refund_bps);
        assert_eq!(actual.base_treasury_bps, expected.base_treasury_bps);
        assert_eq!(actual.project_count, expected.project_count);
        assert_eq!(
            actual.treasury_usdc_state_v2,
            expected.treasury_usdc_state_v2
        );
        assert_eq!(
            actual.base_bond_treasury_vault,
            expected.base_bond_treasury_vault
        );
        assert_eq!(actual.relief_or_risk_vault, expected.relief_or_risk_vault);
        assert_eq!(actual.vault_authority_v2, expected.vault_authority_v2);
        assert_eq!(
            actual.security_governance_config,
            expected.security_governance_config
        );
        assert_eq!(actual.is_paused, expected.is_paused);
        assert_eq!(actual.bump, expected.bump);
        assert_eq!(actual.reserved, expected.reserved);
    }

    #[test]
    fn record_window_update_does_not_change_project_count() {
        let mut config = green_label_config();
        config.project_count = 12;

        record_green_label_window_update(&mut config, 60, 70, 80);

        assert_eq!(config.project_count, 12);
    }

    #[test]
    fn record_window_update_does_not_change_vault_fields() {
        let expected = green_label_config();
        let mut actual = green_label_config();

        record_green_label_window_update(&mut actual, 60, 70, 80);

        assert_eq!(
            actual.treasury_usdc_state_v2,
            expected.treasury_usdc_state_v2
        );
        assert_eq!(
            actual.base_bond_treasury_vault,
            expected.base_bond_treasury_vault
        );
        assert_eq!(actual.relief_or_risk_vault, expected.relief_or_risk_vault);
        assert_eq!(actual.vault_authority_v2, expected.vault_authority_v2);
        assert_eq!(
            actual.security_governance_config,
            expected.security_governance_config
        );
    }

    mod min_base_bond_update_tests {
        use super::*;

        #[test]
        fn accepts_valid_min_base_bond_update_to_1_usdc() {
            let authority = Pubkey::new_from_array([1; 32]);

            validate_green_label_min_base_bond_update(false, authority, authority, 1_000_000)
                .unwrap();
        }

        #[test]
        fn accepts_valid_min_base_bond_update_to_299_usdc() {
            let authority = Pubkey::new_from_array([1; 32]);

            validate_green_label_min_base_bond_update(
                false,
                authority,
                authority,
                MIN_GREEN_LABEL_BASE_BOND_USDC,
            )
            .unwrap();
        }

        #[test]
        fn rejects_paused_config() {
            let authority = Pubkey::new_from_array([1; 32]);
            let err =
                validate_green_label_min_base_bond_update(true, authority, authority, 1_000_000)
                    .unwrap_err();

            assert_error_contains(err, "InvalidGreenLabelStatus");
        }

        #[test]
        fn rejects_wrong_authority() {
            let err = validate_green_label_min_base_bond_update(
                false,
                Pubkey::new_from_array([1; 32]),
                Pubkey::new_from_array([2; 32]),
                1_000_000,
            )
            .unwrap_err();

            assert_error_contains(err, "UnauthorizedGreenLabelAuthority");
        }

        #[test]
        fn rejects_zero_min_base_bond() {
            let authority = Pubkey::new_from_array([1; 32]);
            let err = validate_green_label_min_base_bond_update(false, authority, authority, 0)
                .unwrap_err();

            assert_error_contains(err, "InvalidGreenLabelBondAmount");
        }

        #[test]
        fn rejects_min_base_bond_above_299() {
            let authority = Pubkey::new_from_array([1; 32]);
            let err = validate_green_label_min_base_bond_update(
                false,
                authority,
                authority,
                MIN_GREEN_LABEL_BASE_BOND_USDC + 1,
            )
            .unwrap_err();

            assert_error_contains(err, "InvalidGreenLabelBondAmount");
        }

        #[test]
        fn record_min_base_bond_update_changes_only_min_base_bond() {
            let expected = green_label_config();
            let mut actual = green_label_config();

            record_green_label_min_base_bond_update(&mut actual, 1_000_000);

            assert_eq!(actual.min_base_bond_usdc, 1_000_000);
            assert_eq!(actual.authority, expected.authority);
            assert_eq!(actual.usdc_mint, expected.usdc_mint);
            assert_eq!(actual.base_refund_bps, expected.base_refund_bps);
            assert_eq!(actual.base_treasury_bps, expected.base_treasury_bps);
            assert_eq!(
                actual.observation_period_seconds,
                expected.observation_period_seconds
            );
            assert_eq!(
                actual.dispute_window_seconds,
                expected.dispute_window_seconds
            );
            assert_eq!(
                actual.response_window_seconds,
                expected.response_window_seconds
            );
            assert_eq!(actual.project_count, expected.project_count);
            assert_eq!(
                actual.treasury_usdc_state_v2,
                expected.treasury_usdc_state_v2
            );
            assert_eq!(
                actual.base_bond_treasury_vault,
                expected.base_bond_treasury_vault
            );
            assert_eq!(actual.relief_or_risk_vault, expected.relief_or_risk_vault);
            assert_eq!(actual.vault_authority_v2, expected.vault_authority_v2);
            assert_eq!(
                actual.security_governance_config,
                expected.security_governance_config
            );
            assert_eq!(actual.is_paused, expected.is_paused);
            assert_eq!(actual.bump, expected.bump);
            assert_eq!(actual.reserved, expected.reserved);
        }

        #[test]
        fn record_min_base_bond_update_does_not_change_windows() {
            let expected = green_label_config();
            let mut actual = green_label_config();

            record_green_label_min_base_bond_update(&mut actual, 1_000_000);

            assert_eq!(
                actual.observation_period_seconds,
                expected.observation_period_seconds
            );
            assert_eq!(
                actual.dispute_window_seconds,
                expected.dispute_window_seconds
            );
            assert_eq!(
                actual.response_window_seconds,
                expected.response_window_seconds
            );
        }

        #[test]
        fn record_min_base_bond_update_does_not_change_project_count() {
            let mut config = green_label_config();
            config.project_count = 12;

            record_green_label_min_base_bond_update(&mut config, 1_000_000);

            assert_eq!(config.project_count, 12);
        }

        #[test]
        fn record_min_base_bond_update_does_not_change_vault_fields() {
            let expected = green_label_config();
            let mut actual = green_label_config();

            record_green_label_min_base_bond_update(&mut actual, 1_000_000);

            assert_eq!(
                actual.treasury_usdc_state_v2,
                expected.treasury_usdc_state_v2
            );
            assert_eq!(
                actual.base_bond_treasury_vault,
                expected.base_bond_treasury_vault
            );
            assert_eq!(actual.relief_or_risk_vault, expected.relief_or_risk_vault);
            assert_eq!(actual.vault_authority_v2, expected.vault_authority_v2);
            assert_eq!(
                actual.security_governance_config,
                expected.security_governance_config
            );
        }
    }

    #[test]
    fn submit_project_defaults_to_pending_bond_deposit() {
        let values = pending_bond_project_values(299_000_000);

        assert_eq!(values.status, GreenLabelStatus::PendingBondDeposit);
    }

    #[test]
    fn submit_project_does_not_start_observation_period() {
        let values = pending_bond_project_values(299_000_000);

        assert_eq!(values.observation_start_ts, 0);
        assert_eq!(values.observation_end_ts, 0);
    }

    #[test]
    fn submit_project_uses_default_empty_bond_vault() {
        let values = pending_bond_project_values(299_000_000);

        assert_eq!(values.bond_vault, Pubkey::default());
        assert_eq!(values.bond_vault_authority, Pubkey::default());
    }

    #[test]
    fn submit_project_sets_base_and_extra_for_299() {
        let values = pending_bond_project_values(299_000_000);

        assert_eq!(values.base_bond_amount, 299_000_000);
        assert_eq!(values.extra_bond_amount, 0);
        assert_eq!(values.total_bond_amount, 299_000_000);
    }

    #[test]
    fn submit_project_sets_base_and_extra_for_1299() {
        let values = pending_bond_project_values(1_299_000_000);

        assert_eq!(values.base_bond_amount, 299_000_000);
        assert_eq!(values.extra_bond_amount, 1_000_000_000);
        assert_eq!(values.total_bond_amount, 1_299_000_000);
    }

    #[test]
    fn submit_project_accepts_configured_1_usdc_min_base_bond() {
        let values = pending_bond_project_values_with_min(1_000_000, 1_000_000);

        assert_eq!(values.total_bond_amount, 1_000_000);
        assert_eq!(values.status, GreenLabelStatus::PendingBondDeposit);
    }

    #[test]
    fn submit_project_rejects_below_configured_min_base_bond() {
        let err = try_pending_bond_project_values(false, 1_000_000, 0, 1, 999_999).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBondAmount");
    }

    #[test]
    fn submit_project_uses_configured_min_base_as_base_amount() {
        let values = pending_bond_project_values_with_min(1_000_000, 1_000_000);

        assert_eq!(values.base_bond_amount, 1_000_000);
        assert_eq!(values.extra_bond_amount, 0);
        assert_eq!(values.bond_tier, BondTier::Base);
    }

    #[test]
    fn submit_project_sets_extra_relative_to_configured_min_base() {
        let values = pending_bond_project_values_with_min(1_000_000, 2_000_000);

        assert_eq!(values.base_bond_amount, 1_000_000);
        assert_eq!(values.extra_bond_amount, 1_000_000);
        assert_eq!(values.total_bond_amount, 2_000_000);
    }

    #[test]
    fn submit_project_sets_bond_tier() {
        let values = pending_bond_project_values(1_299_000_000);

        assert_eq!(values.bond_tier, BondTier::Silver);
    }

    #[test]
    fn submit_project_rejects_bond_below_299() {
        let err = try_pending_bond_project_values(
            false,
            MIN_GREEN_LABEL_BASE_BOND_USDC,
            0,
            1,
            298_999_999,
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBondAmount");
    }

    #[test]
    fn submit_project_requires_next_project_id() {
        let err = try_pending_bond_project_values(
            false,
            MIN_GREEN_LABEL_BASE_BOND_USDC,
            0,
            2,
            299_000_000,
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelProjectId");
    }

    #[test]
    fn submit_project_rejects_when_config_paused() {
        let err = try_pending_bond_project_values(
            true,
            MIN_GREEN_LABEL_BASE_BOND_USDC,
            0,
            1,
            299_000_000,
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn submit_project_terminal_fields_are_empty() {
        let values = pending_bond_project_values(299_000_000);

        assert_eq!(values.terminal_proposal_id, 0);
        assert_eq!(values.terminal_proposal_decision, Pubkey::default());
        assert_eq!(values.terminal_execution_queue_item, Pubkey::default());
        assert_eq!(values.terminal_payload_hash, [0; 32]);
        assert_eq!(values.terminal_action_type, ActionType::Noop);
    }

    #[test]
    fn submit_project_reserved_zeroed() {
        let values = pending_bond_project_values(299_000_000);

        assert_eq!(values.reserved, [0; GREEN_LABEL_PROJECT_RESERVED_BYTES]);
    }

    #[test]
    fn bond_vault_init_accepts_pending_bond_deposit_project() {
        validate_green_bond_vault_initialization(
            false,
            Pubkey::new_from_array([8; 32]),
            Pubkey::new_from_array([8; 32]),
            GreenLabelStatus::PendingBondDeposit,
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([2; 32]),
        )
        .unwrap();
    }

    #[test]
    fn bond_vault_init_rejects_paused_config() {
        let err = try_validate_green_bond_vault_initialization(
            true,
            Pubkey::new_from_array([8; 32]),
            GreenLabelStatus::PendingBondDeposit,
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::new_from_array([2; 32]),
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn bond_vault_init_rejects_wrong_project_owner() {
        let err = try_validate_green_bond_vault_initialization(
            false,
            Pubkey::new_from_array([9; 32]),
            GreenLabelStatus::PendingBondDeposit,
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::new_from_array([2; 32]),
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelProjectOwner");
    }

    #[test]
    fn bond_vault_init_rejects_non_pending_bond_status() {
        let err = try_validate_green_bond_vault_initialization(
            false,
            Pubkey::new_from_array([8; 32]),
            GreenLabelStatus::PendingObservation,
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::new_from_array([2; 32]),
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn bond_vault_init_rejects_existing_bond_vault() {
        let err = try_validate_green_bond_vault_initialization(
            false,
            Pubkey::new_from_array([8; 32]),
            GreenLabelStatus::PendingBondDeposit,
            Pubkey::new_from_array([13; 32]),
            Pubkey::default(),
            Pubkey::new_from_array([2; 32]),
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBondVaultState");
    }

    #[test]
    fn bond_vault_init_rejects_existing_bond_vault_authority() {
        let err = try_validate_green_bond_vault_initialization(
            false,
            Pubkey::new_from_array([8; 32]),
            GreenLabelStatus::PendingBondDeposit,
            Pubkey::default(),
            Pubkey::new_from_array([14; 32]),
            Pubkey::new_from_array([2; 32]),
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBondVaultState");
    }

    #[test]
    fn bond_vault_init_rejects_wrong_usdc_mint() {
        let err = try_validate_green_bond_vault_initialization(
            false,
            Pubkey::new_from_array([8; 32]),
            GreenLabelStatus::PendingBondDeposit,
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::new_from_array([3; 32]),
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelMint");
    }

    #[test]
    fn bond_vault_init_keeps_status_pending_bond_deposit() {
        let mut project = pending_bond_project_for_vault_init();

        record_green_bond_vault_initialization(
            &mut project,
            Pubkey::new_from_array([13; 32]),
            Pubkey::new_from_array([14; 32]),
        );

        assert_eq!(project.status, GreenLabelStatus::PendingBondDeposit);
    }

    #[test]
    fn bond_vault_init_does_not_start_observation() {
        let mut project = pending_bond_project_for_vault_init();

        record_green_bond_vault_initialization(
            &mut project,
            Pubkey::new_from_array([13; 32]),
            Pubkey::new_from_array([14; 32]),
        );

        assert_eq!(project.observation_start_ts, 0);
        assert_eq!(project.observation_end_ts, 0);
    }

    #[test]
    fn bond_vault_init_does_not_change_bond_amounts() {
        let mut project = pending_bond_project_for_vault_init();
        let original_amounts = (
            project.base_bond_amount,
            project.extra_bond_amount,
            project.total_bond_amount,
        );

        record_green_bond_vault_initialization(
            &mut project,
            Pubkey::new_from_array([13; 32]),
            Pubkey::new_from_array([14; 32]),
        );

        assert_eq!(
            original_amounts,
            (
                project.base_bond_amount,
                project.extra_bond_amount,
                project.total_bond_amount
            )
        );
    }

    #[test]
    fn bond_lock_accepts_valid_pending_bond_project() {
        validate_bond_lock_fixture(BondLockValidationFixture::valid()).unwrap();
    }

    #[test]
    fn bond_lock_rejects_paused_config() {
        let mut fixture = BondLockValidationFixture::valid();
        fixture.config_is_paused = true;

        let err = validate_bond_lock_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn bond_lock_rejects_wrong_project_owner() {
        let mut fixture = BondLockValidationFixture::valid();
        fixture.signer = Pubkey::new_from_array([9; 32]);

        let err = validate_bond_lock_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelProjectOwner");
    }

    #[test]
    fn bond_lock_rejects_non_pending_bond_status() {
        let mut fixture = BondLockValidationFixture::valid();
        fixture.project_status = GreenLabelStatus::PendingObservation;

        let err = validate_bond_lock_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn bond_lock_rejects_missing_bond_vault() {
        let mut fixture = BondLockValidationFixture::valid();
        fixture.bond_vault = Pubkey::default();

        let err = validate_bond_lock_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBondVaultState");
    }

    #[test]
    fn bond_lock_rejects_missing_bond_vault_authority() {
        let mut fixture = BondLockValidationFixture::valid();
        fixture.bond_vault_authority = Pubkey::default();

        let err = validate_bond_lock_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBondVaultState");
    }

    #[test]
    fn bond_lock_rejects_wrong_bond_vault_account() {
        let mut fixture = BondLockValidationFixture::valid();
        fixture.provided_bond_vault = Pubkey::new_from_array([15; 32]);

        let err = validate_bond_lock_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBondVaultState");
    }

    #[test]
    fn bond_lock_rejects_wrong_bond_vault_mint() {
        let mut fixture = BondLockValidationFixture::valid();
        fixture.provided_bond_vault_mint = Pubkey::new_from_array([3; 32]);

        let err = validate_bond_lock_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelMint");
    }

    #[test]
    fn bond_lock_rejects_wrong_bond_vault_owner() {
        let mut fixture = BondLockValidationFixture::valid();
        fixture.provided_bond_vault_owner = Pubkey::new_from_array([16; 32]);

        let err = validate_bond_lock_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBondVaultState");
    }

    #[test]
    fn bond_lock_rejects_wrong_owner_ata_owner() {
        let mut fixture = BondLockValidationFixture::valid();
        fixture.owner_ata_owner = Pubkey::new_from_array([17; 32]);

        let err = validate_bond_lock_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelTokenAccount");
    }

    #[test]
    fn bond_lock_rejects_wrong_owner_ata_mint() {
        let mut fixture = BondLockValidationFixture::valid();
        fixture.owner_ata_mint = Pubkey::new_from_array([3; 32]);

        let err = validate_bond_lock_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelMint");
    }

    #[test]
    fn bond_lock_rejects_wrong_usdc_mint() {
        let mut fixture = BondLockValidationFixture::valid();
        fixture.usdc_mint = Pubkey::new_from_array([3; 32]);

        let err = validate_bond_lock_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelMint");
    }

    #[test]
    fn bond_lock_rejects_bond_below_299() {
        let mut fixture = BondLockValidationFixture::valid();
        fixture.total_bond_amount = 298_999_999;

        let err = validate_bond_lock_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBondAmount");
    }

    #[test]
    fn observation_window_sets_start_and_end() {
        let (start, end) = build_observation_window(1_000, 30).unwrap();

        assert_eq!(start, 1_000);
        assert_eq!(end, 1_030);
    }

    #[test]
    fn observation_window_rejects_overflow() {
        let err = build_observation_window(i64::MAX, 1).unwrap_err();

        assert_error_contains(err, "GreenLabelMathOverflow");
    }

    #[test]
    fn record_bond_locked_sets_pending_observation() {
        let mut project = pending_bond_project_for_lock();

        record_green_label_bond_locked(&mut project, 1_000, 2_000).unwrap();

        assert_eq!(project.status, GreenLabelStatus::PendingObservation);
    }

    #[test]
    fn record_bond_locked_sets_observation_timestamps() {
        let mut project = pending_bond_project_for_lock();

        record_green_label_bond_locked(&mut project, 1_000, 2_000).unwrap();

        assert_eq!(project.observation_start_ts, 1_000);
        assert_eq!(project.observation_end_ts, 2_000);
    }

    #[test]
    fn record_bond_locked_does_not_change_bond_amounts() {
        let mut project = pending_bond_project_for_lock();
        let original_amounts = (
            project.base_bond_amount,
            project.extra_bond_amount,
            project.total_bond_amount,
        );

        record_green_label_bond_locked(&mut project, 1_000, 2_000).unwrap();

        assert_eq!(
            original_amounts,
            (
                project.base_bond_amount,
                project.extra_bond_amount,
                project.total_bond_amount
            )
        );
    }

    #[test]
    fn record_bond_locked_does_not_change_terminal_fields() {
        let mut project = pending_bond_project_for_lock();
        let original_terminal_fields = (
            project.terminal_proposal_id,
            project.terminal_proposal_decision,
            project.terminal_execution_queue_item,
            project.terminal_payload_hash,
            project.terminal_action_type,
        );

        record_green_label_bond_locked(&mut project, 1_000, 2_000).unwrap();

        assert_eq!(
            original_terminal_fields,
            (
                project.terminal_proposal_id,
                project.terminal_proposal_decision,
                project.terminal_execution_queue_item,
                project.terminal_payload_hash,
                project.terminal_action_type
            )
        );
    }

    #[test]
    fn record_bond_locked_does_not_change_dispute_fields() {
        let mut project = pending_bond_project_for_lock();
        project.dispute_count = 7;
        project.active_dispute = Pubkey::new_from_array([19; 32]);
        let original_dispute_fields = (project.dispute_count, project.active_dispute);

        record_green_label_bond_locked(&mut project, 1_000, 2_000).unwrap();

        assert_eq!(
            original_dispute_fields,
            (project.dispute_count, project.active_dispute)
        );
    }

    #[test]
    fn open_dispute_accepts_pending_observation_project() {
        validate_open_green_label_dispute(
            false,
            GreenLabelStatus::PendingObservation,
            Pubkey::default(),
            0,
            1,
            [1; 32],
        )
        .unwrap();
    }

    #[test]
    fn open_dispute_accepts_active_green_label_project() {
        validate_open_green_label_dispute(
            false,
            GreenLabelStatus::ActiveGreenLabel,
            Pubkey::default(),
            0,
            1,
            [1; 32],
        )
        .unwrap();
    }

    #[test]
    fn open_dispute_rejects_paused_config() {
        let err = validate_open_green_label_dispute(
            true,
            GreenLabelStatus::PendingObservation,
            Pubkey::default(),
            0,
            1,
            [1; 32],
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn open_dispute_rejects_pending_bond_deposit() {
        let err = validate_open_green_label_dispute(
            false,
            GreenLabelStatus::PendingBondDeposit,
            Pubkey::default(),
            0,
            1,
            [1; 32],
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn open_dispute_rejects_refunded() {
        let err = validate_open_green_label_dispute(
            false,
            GreenLabelStatus::Refunded,
            Pubkey::default(),
            0,
            1,
            [1; 32],
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn open_dispute_rejects_slashed() {
        let err = validate_open_green_label_dispute(
            false,
            GreenLabelStatus::Slashed,
            Pubkey::default(),
            0,
            1,
            [1; 32],
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn open_dispute_rejects_existing_active_dispute() {
        let err = validate_open_green_label_dispute(
            false,
            GreenLabelStatus::PendingObservation,
            Pubkey::new_from_array([18; 32]),
            0,
            1,
            [1; 32],
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelActiveDispute");
    }

    #[test]
    fn open_dispute_requires_next_dispute_id() {
        let err = validate_open_green_label_dispute(
            false,
            GreenLabelStatus::PendingObservation,
            Pubkey::default(),
            0,
            2,
            [1; 32],
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelDisputeId");
    }

    #[test]
    fn open_dispute_rejects_zero_evidence_hash() {
        let err = validate_open_green_label_dispute(
            false,
            GreenLabelStatus::PendingObservation,
            Pubkey::default(),
            0,
            1,
            [0; 32],
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelEvidenceHash");
    }

    #[test]
    fn dispute_windows_set_evidence_and_response_end() {
        let (evidence_end_ts, response_end_ts) = build_dispute_windows(1_000, 60, 30).unwrap();

        assert_eq!(evidence_end_ts, 1_060);
        assert_eq!(response_end_ts, 1_090);
    }

    #[test]
    fn dispute_windows_reject_evidence_overflow() {
        let err = build_dispute_windows(i64::MAX, 1, 0).unwrap_err();

        assert_error_contains(err, "GreenLabelMathOverflow");
    }

    #[test]
    fn dispute_windows_reject_response_overflow() {
        let err = build_dispute_windows(i64::MAX - 1, 1, 1).unwrap_err();

        assert_error_contains(err, "GreenLabelMathOverflow");
    }

    #[test]
    fn record_dispute_opened_sets_project_disputed() {
        let mut project = project_for_open_dispute_record();

        record_green_label_dispute_opened(&mut project, Pubkey::new_from_array([18; 32]), 1)
            .unwrap();

        assert_eq!(project.status, GreenLabelStatus::Disputed);
    }

    #[test]
    fn record_dispute_opened_sets_active_dispute() {
        let mut project = project_for_open_dispute_record();
        let dispute_key = Pubkey::new_from_array([18; 32]);

        record_green_label_dispute_opened(&mut project, dispute_key, 1).unwrap();

        assert_eq!(project.active_dispute, dispute_key);
    }

    #[test]
    fn record_dispute_opened_updates_dispute_count() {
        let mut project = project_for_open_dispute_record();

        record_green_label_dispute_opened(&mut project, Pubkey::new_from_array([18; 32]), 3)
            .unwrap();

        assert_eq!(project.dispute_count, 3);
    }

    #[test]
    fn record_dispute_opened_does_not_change_bond_amounts() {
        let mut project = project_for_open_dispute_record();
        let original_amounts = (
            project.base_bond_amount,
            project.extra_bond_amount,
            project.total_bond_amount,
        );

        record_green_label_dispute_opened(&mut project, Pubkey::new_from_array([18; 32]), 1)
            .unwrap();

        assert_eq!(
            original_amounts,
            (
                project.base_bond_amount,
                project.extra_bond_amount,
                project.total_bond_amount
            )
        );
    }

    #[test]
    fn record_dispute_opened_does_not_change_observation_timestamps() {
        let mut project = project_for_open_dispute_record();
        let original_observation_fields =
            (project.observation_start_ts, project.observation_end_ts);

        record_green_label_dispute_opened(&mut project, Pubkey::new_from_array([18; 32]), 1)
            .unwrap();

        assert_eq!(
            original_observation_fields,
            (project.observation_start_ts, project.observation_end_ts)
        );
    }

    #[test]
    fn record_dispute_opened_does_not_change_terminal_fields() {
        let mut project = project_for_open_dispute_record();
        let original_terminal_fields = (
            project.terminal_proposal_id,
            project.terminal_proposal_decision,
            project.terminal_execution_queue_item,
            project.terminal_payload_hash,
            project.terminal_action_type,
        );

        record_green_label_dispute_opened(&mut project, Pubkey::new_from_array([18; 32]), 1)
            .unwrap();

        assert_eq!(
            original_terminal_fields,
            (
                project.terminal_proposal_id,
                project.terminal_proposal_decision,
                project.terminal_execution_queue_item,
                project.terminal_payload_hash,
                project.terminal_action_type
            )
        );
    }

    #[test]
    fn mark_dispute_ready_accepts_evidence_period_after_response_end() {
        validate_mark_ready_fixture(MarkReadyValidationFixture::valid()).unwrap();
    }

    #[test]
    fn mark_dispute_ready_accepts_project_response_period_after_response_end() {
        let mut fixture = MarkReadyValidationFixture::valid();
        fixture.dispute_status = DisputeStatus::ProjectResponsePeriod;

        validate_mark_ready_fixture(fixture).unwrap();
    }

    #[test]
    fn mark_dispute_ready_rejects_paused_config() {
        let mut fixture = MarkReadyValidationFixture::valid();
        fixture.config_is_paused = true;

        let err = validate_mark_ready_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn mark_dispute_ready_rejects_non_disputed_project() {
        let mut fixture = MarkReadyValidationFixture::valid();
        fixture.project_status = GreenLabelStatus::PendingObservation;

        let err = validate_mark_ready_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn mark_dispute_ready_rejects_wrong_active_dispute() {
        let mut fixture = MarkReadyValidationFixture::valid();
        fixture.project_active_dispute = Pubkey::new_from_array([22; 32]);

        let err = validate_mark_ready_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelActiveDispute");
    }

    #[test]
    fn mark_dispute_ready_rejects_wrong_dispute_project() {
        let mut fixture = MarkReadyValidationFixture::valid();
        fixture.dispute_project = Pubkey::new_from_array([23; 32]);

        let err = validate_mark_ready_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelTargetAccount");
    }

    #[test]
    fn mark_dispute_ready_rejects_invalid_dispute_status() {
        let mut fixture = MarkReadyValidationFixture::valid();
        fixture.dispute_status = DisputeStatus::ReadyForDecision;

        let err = validate_mark_ready_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelDisputeStatus");
    }

    #[test]
    fn mark_dispute_ready_rejects_before_response_end() {
        let mut fixture = MarkReadyValidationFixture::valid();
        fixture.now = 999;
        fixture.response_end_ts = 1_000;

        let err = validate_mark_ready_fixture(fixture).unwrap_err();

        assert_error_contains(err, "GreenLabelDisputeWindowNotEnded");
    }

    #[test]
    fn record_dispute_ready_sets_ready_for_decision() {
        let mut dispute = dispute_for_ready_record();

        record_dispute_ready_for_decision(&mut dispute).unwrap();

        assert_eq!(dispute.status, DisputeStatus::ReadyForDecision);
    }

    #[test]
    fn record_dispute_ready_does_not_change_security_fields() {
        let mut dispute = dispute_for_ready_record();
        dispute.proposal_id = 7;
        dispute.proposal_decision = Pubkey::new_from_array([21; 32]);
        dispute.execution_queue_item = Pubkey::new_from_array([22; 32]);
        dispute.payload_hash = [23; 32];
        dispute.action_type = ActionType::GreenLabelSlash;
        dispute.resolved_at = 99;
        let original_security_fields = (
            dispute.proposal_id,
            dispute.proposal_decision,
            dispute.execution_queue_item,
            dispute.payload_hash,
            dispute.action_type,
            dispute.resolved_at,
        );

        record_dispute_ready_for_decision(&mut dispute).unwrap();

        assert_eq!(
            original_security_fields,
            (
                dispute.proposal_id,
                dispute.proposal_decision,
                dispute.execution_queue_item,
                dispute.payload_hash,
                dispute.action_type,
                dispute.resolved_at
            )
        );
    }

    #[test]
    fn link_security_decision_accepts_green_label_slash() {
        validate_link_decision_fixture(LinkDecisionValidationFixture::valid(
            ActionType::GreenLabelSlash,
        ))
        .unwrap();
    }

    #[test]
    fn link_security_decision_accepts_green_label_refund() {
        validate_link_decision_fixture(LinkDecisionValidationFixture::valid(
            ActionType::GreenLabelRefund,
        ))
        .unwrap();
    }

    #[test]
    fn link_security_decision_rejects_paused_config() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.config_is_paused = true;

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn link_security_decision_rejects_non_disputed_project() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.project_status = GreenLabelStatus::PendingObservation;

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn link_security_decision_rejects_wrong_active_dispute() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.project_active_dispute = Pubkey::new_from_array([22; 32]);

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelActiveDispute");
    }

    #[test]
    fn link_security_decision_rejects_dispute_not_ready() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.dispute_status = DisputeStatus::EvidencePeriod;

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelDisputeStatus");
    }

    #[test]
    fn link_security_decision_rejects_zero_payload_hash() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.expected_payload_hash = [0; 32];
        fixture.queue_payload_hash = [0; 32];

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelPayloadHash");
    }

    #[test]
    fn link_security_decision_rejects_invalid_action_type() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.expected_action_type = ActionType::Noop;

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelActionType");
    }

    #[test]
    fn link_security_decision_rejects_wrong_proposal_id() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.proposal_id = 8;

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelSecurityDecision");
    }

    #[test]
    fn link_security_decision_rejects_wrong_proposal_type() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.proposal_type = ProposalType::GreenLabelRefund;

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelSecurityDecision");
    }

    #[test]
    fn link_security_decision_rejects_non_approved_decision() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.proposal_decision = ProposalDecision::Partial;

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelSecurityDecision");
    }

    #[test]
    fn link_security_decision_rejects_wrong_queue_proposal_id() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.queue_proposal_id = 8;

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelExecutionQueue");
    }

    #[test]
    fn link_security_decision_rejects_wrong_queue_action_type() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.queue_action_type = ActionType::GreenLabelRefund;

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelExecutionQueue");
    }

    #[test]
    fn link_security_decision_rejects_non_queued_status() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.queue_status = ExecutionStatus::Executed;

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelExecutionQueue");
    }

    #[test]
    fn link_security_decision_rejects_payload_hash_mismatch() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.queue_payload_hash = [24; 32];

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelExecutionQueue");
    }

    #[test]
    fn link_security_decision_rejects_wrong_target_program() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.queue_target_program = Pubkey::new_from_array([25; 32]);

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelTargetProgram");
    }

    #[test]
    fn link_security_decision_rejects_wrong_target_account() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.queue_target_account = Pubkey::new_from_array([26; 32]);

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelTargetAccount");
    }

    #[test]
    fn record_security_decision_link_sets_project_slash_queued() {
        let (mut project, mut dispute) = security_link_record_accounts();

        record_green_label_security_decision_link(
            &mut project,
            &mut dispute,
            7,
            Pubkey::new_from_array([21; 32]),
            Pubkey::new_from_array([22; 32]),
            [23; 32],
            ActionType::GreenLabelSlash,
        )
        .unwrap();

        assert_eq!(project.status, GreenLabelStatus::SlashQueued);
    }

    #[test]
    fn record_security_decision_link_sets_project_refund_queued() {
        let (mut project, mut dispute) = security_link_record_accounts();

        record_green_label_security_decision_link(
            &mut project,
            &mut dispute,
            7,
            Pubkey::new_from_array([21; 32]),
            Pubkey::new_from_array([22; 32]),
            [23; 32],
            ActionType::GreenLabelRefund,
        )
        .unwrap();

        assert_eq!(project.status, GreenLabelStatus::RefundQueued);
    }

    #[test]
    fn record_security_decision_link_sets_dispute_decision_queued() {
        let (mut project, mut dispute) = security_link_record_accounts();

        record_green_label_security_decision_link(
            &mut project,
            &mut dispute,
            7,
            Pubkey::new_from_array([21; 32]),
            Pubkey::new_from_array([22; 32]),
            [23; 32],
            ActionType::GreenLabelSlash,
        )
        .unwrap();

        assert_eq!(dispute.status, DisputeStatus::DecisionQueued);
    }

    #[test]
    fn record_security_decision_link_records_terminal_fields() {
        let (mut project, mut dispute) = security_link_record_accounts();
        let proposal_decision = Pubkey::new_from_array([21; 32]);
        let execution_queue_item = Pubkey::new_from_array([22; 32]);
        let payload_hash = [23; 32];

        record_green_label_security_decision_link(
            &mut project,
            &mut dispute,
            7,
            proposal_decision,
            execution_queue_item,
            payload_hash,
            ActionType::GreenLabelSlash,
        )
        .unwrap();

        assert_eq!(project.terminal_proposal_id, 7);
        assert_eq!(project.terminal_proposal_decision, proposal_decision);
        assert_eq!(project.terminal_execution_queue_item, execution_queue_item);
        assert_eq!(project.terminal_payload_hash, payload_hash);
        assert_eq!(project.terminal_action_type, ActionType::GreenLabelSlash);
        assert_eq!(dispute.proposal_id, 7);
        assert_eq!(dispute.proposal_decision, proposal_decision);
        assert_eq!(dispute.execution_queue_item, execution_queue_item);
        assert_eq!(dispute.payload_hash, payload_hash);
        assert_eq!(dispute.action_type, ActionType::GreenLabelSlash);
    }

    #[test]
    fn record_security_decision_link_does_not_change_bond_fields() {
        let (mut project, mut dispute) = security_link_record_accounts();
        let original_bond_fields = (
            project.bond_vault,
            project.bond_vault_authority,
            project.base_bond_amount,
            project.extra_bond_amount,
            project.total_bond_amount,
        );

        record_green_label_security_decision_link(
            &mut project,
            &mut dispute,
            7,
            Pubkey::new_from_array([21; 32]),
            Pubkey::new_from_array([22; 32]),
            [23; 32],
            ActionType::GreenLabelSlash,
        )
        .unwrap();

        assert_eq!(
            original_bond_fields,
            (
                project.bond_vault,
                project.bond_vault_authority,
                project.base_bond_amount,
                project.extra_bond_amount,
                project.total_bond_amount
            )
        );
    }

    #[test]
    fn record_security_decision_link_does_not_change_observation_timestamps() {
        let (mut project, mut dispute) = security_link_record_accounts();
        let original_observation_fields =
            (project.observation_start_ts, project.observation_end_ts);

        record_green_label_security_decision_link(
            &mut project,
            &mut dispute,
            7,
            Pubkey::new_from_array([21; 32]),
            Pubkey::new_from_array([22; 32]),
            [23; 32],
            ActionType::GreenLabelSlash,
        )
        .unwrap();

        assert_eq!(
            original_observation_fields,
            (project.observation_start_ts, project.observation_end_ts)
        );
    }

    #[test]
    fn refund_execution_accepts_valid_queued_refund() {
        validate_refund_execution_fixture(RefundExecutionValidationFixture::valid()).unwrap();
    }

    #[test]
    fn refund_execution_rejects_non_refund_queued_project() {
        let mut fixture = RefundExecutionValidationFixture::valid();
        fixture.project_status = GreenLabelStatus::Disputed;

        let err = validate_refund_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn refund_execution_rejects_zero_payload_hash() {
        let mut fixture = RefundExecutionValidationFixture::valid();
        fixture.project_terminal_payload_hash = [0; 32];

        let err = validate_refund_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelPayloadHash");
    }

    #[test]
    fn refund_execution_rejects_wrong_proposal_decision_account() {
        let mut fixture = RefundExecutionValidationFixture::valid();
        fixture.proposal_decision_key = Pubkey::new_from_array([24; 32]);

        let err = validate_refund_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelSecurityDecision");
    }

    #[test]
    fn refund_execution_rejects_wrong_queue_account() {
        let mut fixture = RefundExecutionValidationFixture::valid();
        fixture.execution_queue_item_key = Pubkey::new_from_array([24; 32]);

        let err = validate_refund_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelExecutionQueue");
    }

    #[test]
    fn refund_execution_rejects_wrong_proposal_id() {
        let mut fixture = RefundExecutionValidationFixture::valid();
        fixture.proposal_decision_proposal_id = 8;

        let err = validate_refund_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelSecurityDecision");
    }

    #[test]
    fn refund_execution_rejects_non_approved_decision() {
        let mut fixture = RefundExecutionValidationFixture::valid();
        fixture.proposal_decision = ProposalDecision::Rejected;

        let err = validate_refund_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelSecurityDecision");
    }

    #[test]
    fn refund_execution_rejects_non_queued_status() {
        let mut fixture = RefundExecutionValidationFixture::valid();
        fixture.queue_status = ExecutionStatus::Cancelled;

        let err = validate_refund_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelExecutionQueue");
    }

    #[test]
    fn refund_execution_rejects_wrong_action_type() {
        let mut fixture = RefundExecutionValidationFixture::valid();
        fixture.queue_action_type = ActionType::GreenLabelSlash;

        let err = validate_refund_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelExecutionQueue");
    }

    #[test]
    fn refund_execution_rejects_payload_hash_mismatch() {
        let mut fixture = RefundExecutionValidationFixture::valid();
        fixture.queue_payload_hash = [24; 32];

        let err = validate_refund_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelExecutionQueue");
    }

    #[test]
    fn refund_execution_rejects_wrong_target_program() {
        let mut fixture = RefundExecutionValidationFixture::valid();
        fixture.queue_target_program = Pubkey::new_from_array([24; 32]);

        let err = validate_refund_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelTargetProgram");
    }

    #[test]
    fn refund_execution_rejects_wrong_target_account() {
        let mut fixture = RefundExecutionValidationFixture::valid();
        fixture.queue_target_account = Pubkey::new_from_array([24; 32]);

        let err = validate_refund_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelTargetAccount");
    }

    #[test]
    fn refund_execution_rejects_timelock_not_satisfied() {
        let mut fixture = RefundExecutionValidationFixture::valid();
        fixture.now = fixture.queue_execute_after - 1;

        let err = validate_refund_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "GreenLabelTimelockNotSatisfied");
    }

    #[test]
    fn refund_execution_rejects_missing_bond_vault() {
        let mut fixture = RefundExecutionValidationFixture::valid();
        fixture.project_bond_vault = Pubkey::default();

        let err = validate_refund_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBondVaultState");
    }

    #[test]
    fn refund_execution_rejects_wrong_bond_vault_account() {
        let mut fixture = RefundExecutionValidationFixture::valid();
        fixture.provided_bond_vault = Pubkey::new_from_array([24; 32]);

        let err = validate_refund_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBondVaultState");
    }

    #[test]
    fn refund_execution_rejects_wrong_bond_vault_mint() {
        let mut fixture = RefundExecutionValidationFixture::valid();
        fixture.green_bond_vault_mint = Pubkey::new_from_array([24; 32]);

        let err = validate_refund_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelTokenAccount");
    }

    #[test]
    fn refund_execution_rejects_wrong_bond_vault_owner() {
        let mut fixture = RefundExecutionValidationFixture::valid();
        fixture.green_bond_vault_owner = Pubkey::new_from_array([24; 32]);

        let err = validate_refund_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelTokenAccount");
    }

    #[test]
    fn refund_execution_rejects_wrong_project_owner_ata() {
        let mut fixture = RefundExecutionValidationFixture::valid();
        fixture.project_owner_ata_owner = Pubkey::new_from_array([24; 32]);

        let err = validate_refund_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelTokenAccount");
    }

    #[test]
    fn refund_execution_rejects_wrong_treasury_vault() {
        let mut fixture = RefundExecutionValidationFixture::valid();
        fixture.provided_treasury_vault = Pubkey::new_from_array([24; 32]);

        let err = validate_refund_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelTokenAccount");
    }

    #[test]
    fn refund_execution_rejects_insufficient_vault_balance() {
        let mut fixture = RefundExecutionValidationFixture::valid();
        fixture.vault_balance = 1_298_999_999;

        let err = validate_refund_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "GreenLabelInsufficientBondVaultBalance");
    }

    #[test]
    fn record_refunded_sets_project_refunded() {
        let (mut project, mut dispute) = refund_record_accounts();

        record_green_label_refunded(&mut project, Some(&mut dispute), 3_000).unwrap();

        assert_eq!(project.status, GreenLabelStatus::Refunded);
    }

    #[test]
    fn record_refunded_sets_refunded_at() {
        let (mut project, mut dispute) = refund_record_accounts();

        record_green_label_refunded(&mut project, Some(&mut dispute), 3_000).unwrap();

        assert_eq!(project.refunded_at, 3_000);
    }

    #[test]
    fn record_refunded_clears_active_dispute() {
        let (mut project, mut dispute) = refund_record_accounts();

        record_green_label_refunded(&mut project, Some(&mut dispute), 3_000).unwrap();

        assert_eq!(project.active_dispute, Pubkey::default());
    }

    #[test]
    fn record_refunded_sets_dispute_resolved_refund() {
        let (mut project, mut dispute) = refund_record_accounts();

        record_green_label_refunded(&mut project, Some(&mut dispute), 3_000).unwrap();

        assert_eq!(dispute.status, DisputeStatus::ResolvedRefund);
        assert_eq!(dispute.resolved_at, 3_000);
    }

    #[test]
    fn record_refunded_does_not_change_bond_amounts() {
        let (mut project, mut dispute) = refund_record_accounts();
        let original_bond_amounts = (
            project.base_bond_amount,
            project.extra_bond_amount,
            project.total_bond_amount,
        );

        record_green_label_refunded(&mut project, Some(&mut dispute), 3_000).unwrap();

        assert_eq!(
            original_bond_amounts,
            (
                project.base_bond_amount,
                project.extra_bond_amount,
                project.total_bond_amount
            )
        );
    }

    #[test]
    fn slash_execution_accepts_valid_queued_slash() {
        validate_slash_execution_fixture(SlashExecutionValidationFixture::valid()).unwrap();
    }

    #[test]
    fn slash_execution_rejects_paused_config() {
        let mut fixture = SlashExecutionValidationFixture::valid();
        fixture.config_is_paused = true;

        let err = validate_slash_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn slash_execution_rejects_non_slash_queued_project() {
        let mut fixture = SlashExecutionValidationFixture::valid();
        fixture.project_status = GreenLabelStatus::Disputed;

        let err = validate_slash_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn slash_execution_rejects_wrong_active_dispute() {
        let mut fixture = SlashExecutionValidationFixture::valid();
        fixture.project_active_dispute = Pubkey::new_from_array([24; 32]);

        let err = validate_slash_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelActiveDispute");
    }

    #[test]
    fn slash_execution_rejects_wrong_dispute_project() {
        let mut fixture = SlashExecutionValidationFixture::valid();
        fixture.dispute_project = Pubkey::new_from_array([24; 32]);

        let err = validate_slash_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelTargetAccount");
    }

    #[test]
    fn slash_execution_rejects_dispute_not_decision_queued() {
        let mut fixture = SlashExecutionValidationFixture::valid();
        fixture.dispute_status = DisputeStatus::ReadyForDecision;

        let err = validate_slash_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelDisputeStatus");
    }

    #[test]
    fn slash_execution_rejects_zero_payload_hash() {
        let mut fixture = SlashExecutionValidationFixture::valid();
        fixture.project_terminal_payload_hash = [0; 32];

        let err = validate_slash_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelPayloadHash");
    }

    #[test]
    fn slash_execution_rejects_wrong_proposal_decision_account() {
        let mut fixture = SlashExecutionValidationFixture::valid();
        fixture.proposal_decision_key = Pubkey::new_from_array([24; 32]);

        let err = validate_slash_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelSecurityDecision");
    }

    #[test]
    fn slash_execution_rejects_wrong_queue_account() {
        let mut fixture = SlashExecutionValidationFixture::valid();
        fixture.execution_queue_item_key = Pubkey::new_from_array([24; 32]);

        let err = validate_slash_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelExecutionQueue");
    }

    #[test]
    fn slash_execution_rejects_wrong_proposal_id() {
        let mut fixture = SlashExecutionValidationFixture::valid();
        fixture.proposal_decision_proposal_id = 8;

        let err = validate_slash_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelSecurityDecision");
    }

    #[test]
    fn slash_execution_rejects_non_approved_decision() {
        let mut fixture = SlashExecutionValidationFixture::valid();
        fixture.proposal_decision = ProposalDecision::Rejected;

        let err = validate_slash_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelSecurityDecision");
    }

    #[test]
    fn slash_execution_rejects_non_queued_status() {
        let mut fixture = SlashExecutionValidationFixture::valid();
        fixture.queue_status = ExecutionStatus::Cancelled;

        let err = validate_slash_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelExecutionQueue");
    }

    #[test]
    fn slash_execution_rejects_wrong_action_type() {
        let mut fixture = SlashExecutionValidationFixture::valid();
        fixture.queue_action_type = ActionType::GreenLabelRefund;

        let err = validate_slash_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelExecutionQueue");
    }

    #[test]
    fn slash_execution_rejects_payload_hash_mismatch() {
        let mut fixture = SlashExecutionValidationFixture::valid();
        fixture.queue_payload_hash = [24; 32];

        let err = validate_slash_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelExecutionQueue");
    }

    #[test]
    fn slash_execution_rejects_wrong_target_program() {
        let mut fixture = SlashExecutionValidationFixture::valid();
        fixture.queue_target_program = Pubkey::new_from_array([24; 32]);

        let err = validate_slash_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelTargetProgram");
    }

    #[test]
    fn slash_execution_rejects_wrong_target_account() {
        let mut fixture = SlashExecutionValidationFixture::valid();
        fixture.queue_target_account = Pubkey::new_from_array([24; 32]);

        let err = validate_slash_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelTargetAccount");
    }

    #[test]
    fn slash_execution_rejects_timelock_not_satisfied() {
        let mut fixture = SlashExecutionValidationFixture::valid();
        fixture.now = fixture.queue_execute_after - 1;

        let err = validate_slash_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "GreenLabelTimelockNotSatisfied");
    }

    #[test]
    fn slash_execution_rejects_missing_bond_vault() {
        let mut fixture = SlashExecutionValidationFixture::valid();
        fixture.project_bond_vault = Pubkey::default();

        let err = validate_slash_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBondVaultState");
    }

    #[test]
    fn slash_execution_rejects_wrong_bond_vault_account() {
        let mut fixture = SlashExecutionValidationFixture::valid();
        fixture.provided_bond_vault = Pubkey::new_from_array([24; 32]);

        let err = validate_slash_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBondVaultState");
    }

    #[test]
    fn slash_execution_rejects_wrong_bond_vault_mint() {
        let mut fixture = SlashExecutionValidationFixture::valid();
        fixture.green_bond_vault_mint = Pubkey::new_from_array([24; 32]);

        let err = validate_slash_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelTokenAccount");
    }

    #[test]
    fn slash_execution_rejects_wrong_bond_vault_owner() {
        let mut fixture = SlashExecutionValidationFixture::valid();
        fixture.green_bond_vault_owner = Pubkey::new_from_array([24; 32]);

        let err = validate_slash_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelTokenAccount");
    }

    #[test]
    fn slash_execution_rejects_wrong_relief_or_risk_vault() {
        let mut fixture = SlashExecutionValidationFixture::valid();
        fixture.provided_relief_or_risk_vault = Pubkey::new_from_array([24; 32]);

        let err = validate_slash_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelTokenAccount");
    }

    #[test]
    fn slash_execution_rejects_insufficient_vault_balance() {
        let mut fixture = SlashExecutionValidationFixture::valid();
        fixture.vault_balance = fixture.slash_amount - 1;

        let err = validate_slash_execution_fixture(fixture).unwrap_err();

        assert_error_contains(err, "GreenLabelInsufficientBondVaultBalance");
    }

    #[test]
    fn record_slashed_sets_project_slashed() {
        let (mut project, mut dispute) = slash_record_accounts();

        record_green_label_slashed(&mut project, &mut dispute, 3_000).unwrap();

        assert_eq!(project.status, GreenLabelStatus::Slashed);
    }

    #[test]
    fn record_slashed_sets_slashed_at() {
        let (mut project, mut dispute) = slash_record_accounts();

        record_green_label_slashed(&mut project, &mut dispute, 3_000).unwrap();

        assert_eq!(project.slashed_at, 3_000);
    }

    #[test]
    fn record_slashed_clears_active_dispute() {
        let (mut project, mut dispute) = slash_record_accounts();

        record_green_label_slashed(&mut project, &mut dispute, 3_000).unwrap();

        assert_eq!(project.active_dispute, Pubkey::default());
    }

    #[test]
    fn record_slashed_sets_dispute_resolved_slash() {
        let (mut project, mut dispute) = slash_record_accounts();

        record_green_label_slashed(&mut project, &mut dispute, 3_000).unwrap();

        assert_eq!(dispute.status, DisputeStatus::ResolvedSlash);
        assert_eq!(dispute.resolved_at, 3_000);
    }

    #[test]
    fn record_slashed_does_not_change_refunded_at() {
        let (mut project, mut dispute) = slash_record_accounts();
        let original_refunded_at = project.refunded_at;

        record_green_label_slashed(&mut project, &mut dispute, 3_000).unwrap();

        assert_eq!(project.refunded_at, original_refunded_at);
    }

    #[test]
    fn record_slashed_does_not_change_bond_amounts() {
        let (mut project, mut dispute) = slash_record_accounts();
        let original_bond_amounts = (
            project.base_bond_amount,
            project.extra_bond_amount,
            project.total_bond_amount,
        );

        record_green_label_slashed(&mut project, &mut dispute, 3_000).unwrap();

        assert_eq!(
            original_bond_amounts,
            (
                project.base_bond_amount,
                project.extra_bond_amount,
                project.total_bond_amount
            )
        );
    }

    #[test]
    fn record_slashed_does_not_change_observation_timestamps() {
        let (mut project, mut dispute) = slash_record_accounts();
        let original_observation_fields =
            (project.observation_start_ts, project.observation_end_ts);

        record_green_label_slashed(&mut project, &mut dispute, 3_000).unwrap();

        assert_eq!(
            original_observation_fields,
            (project.observation_start_ts, project.observation_end_ts)
        );
    }

    #[test]
    fn record_slashed_does_not_change_terminal_fields() {
        let (mut project, mut dispute) = slash_record_accounts();
        let original_terminal_fields = (
            project.terminal_proposal_id,
            project.terminal_proposal_decision,
            project.terminal_execution_queue_item,
            project.terminal_payload_hash,
            project.terminal_action_type,
        );

        record_green_label_slashed(&mut project, &mut dispute, 3_000).unwrap();

        assert_eq!(
            original_terminal_fields,
            (
                project.terminal_proposal_id,
                project.terminal_proposal_decision,
                project.terminal_execution_queue_item,
                project.terminal_payload_hash,
                project.terminal_action_type
            )
        );
    }

    #[test]
    fn validate_bps_config_rejects_invalid_sum() {
        let err = validate_green_label_bps_config(8_000, 1_000).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBpsConfig");
    }

    #[test]
    fn split_bond_accepts_minimum_299() {
        let split = split_green_label_bond(299_000_000, MIN_GREEN_LABEL_BASE_BOND_USDC).unwrap();

        assert_eq!(split.base_bond_amount, 299_000_000);
        assert_eq!(split.extra_bond_amount, 0);
        assert_eq!(split.total_bond_amount, 299_000_000);
    }

    #[test]
    fn split_bond_rejects_below_299() {
        let err = split_green_label_bond(298_999_999, MIN_GREEN_LABEL_BASE_BOND_USDC).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBondAmount");
    }

    #[test]
    fn split_bond_separates_1299_into_299_base_1000_extra() {
        let split = split_green_label_bond(1_299_000_000, MIN_GREEN_LABEL_BASE_BOND_USDC).unwrap();

        assert_eq!(split.base_bond_amount, 299_000_000);
        assert_eq!(split.extra_bond_amount, 1_000_000_000);
        assert_eq!(split.total_bond_amount, 1_299_000_000);
    }

    #[test]
    fn split_bond_accepts_configured_minimum_1_usdc() {
        let split = split_green_label_bond(1_000_000, 1_000_000).unwrap();

        assert_eq!(split.base_bond_amount, 1_000_000);
        assert_eq!(split.extra_bond_amount, 0);
        assert_eq!(split.total_bond_amount, 1_000_000);
    }

    #[test]
    fn split_bond_rejects_below_configured_minimum() {
        let err = split_green_label_bond(999_999, 1_000_000).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBondAmount");
    }

    #[test]
    fn split_bond_with_2_usdc_and_1_usdc_min_sets_1_usdc_extra() {
        let split = split_green_label_bond(2_000_000, 1_000_000).unwrap();

        assert_eq!(split.base_bond_amount, 1_000_000);
        assert_eq!(split.extra_bond_amount, 1_000_000);
        assert_eq!(split.total_bond_amount, 2_000_000);
    }

    #[test]
    fn refund_amounts_for_299_still_match_80_20() {
        let project = pending_bond_project_values(299_000_000);
        let amounts =
            calculate_green_label_refund(project.base_bond_amount, project.extra_bond_amount)
                .unwrap();

        assert_eq!(amounts.base_refund_amount, 239_200_000);
        assert_eq!(amounts.base_treasury_amount, 59_800_000);
        assert_eq!(amounts.extra_refund_amount, 0);
        assert_eq!(amounts.project_refund_amount, 239_200_000);
        assert_eq!(amounts.treasury_amount, 59_800_000);
    }

    #[test]
    fn refund_amounts_for_1299_refund_extra_100_percent() {
        let project = pending_bond_project_values(1_299_000_000);
        let amounts =
            calculate_green_label_refund(project.base_bond_amount, project.extra_bond_amount)
                .unwrap();

        assert_eq!(amounts.base_refund_amount, 239_200_000);
        assert_eq!(amounts.base_treasury_amount, 59_800_000);
        assert_eq!(amounts.extra_refund_amount, 1_000_000_000);
        assert_eq!(amounts.project_refund_amount, 1_239_200_000);
        assert_eq!(amounts.treasury_amount, 59_800_000);
    }

    #[test]
    fn slash_amount_is_full_bond() {
        assert_eq!(
            calculate_green_label_slash_amount(1_299_000_000).unwrap(),
            1_299_000_000
        );
    }

    #[test]
    fn slash_amount_for_299_is_full_bond() {
        assert_eq!(
            calculate_green_label_slash_amount(299_000_000).unwrap(),
            299_000_000
        );
    }

    #[test]
    fn slash_amount_for_1299_includes_extra_bond() {
        assert_eq!(
            calculate_green_label_slash_amount(1_299_000_000).unwrap(),
            1_299_000_000
        );
    }

    #[test]
    fn bond_tier_base() {
        assert_eq!(
            calculate_bond_tier(299_000_000, MIN_GREEN_LABEL_BASE_BOND_USDC).unwrap(),
            BondTier::Base
        );
        assert_eq!(
            calculate_bond_tier(499_999_999, MIN_GREEN_LABEL_BASE_BOND_USDC).unwrap(),
            BondTier::Base
        );
    }

    #[test]
    fn bond_tier_bronze() {
        assert_eq!(
            calculate_bond_tier(500_000_000, MIN_GREEN_LABEL_BASE_BOND_USDC).unwrap(),
            BondTier::Bronze
        );
        assert_eq!(
            calculate_bond_tier(999_999_999, MIN_GREEN_LABEL_BASE_BOND_USDC).unwrap(),
            BondTier::Bronze
        );
    }

    #[test]
    fn bond_tier_silver() {
        assert_eq!(
            calculate_bond_tier(1_000_000_000, MIN_GREEN_LABEL_BASE_BOND_USDC).unwrap(),
            BondTier::Silver
        );
        assert_eq!(
            calculate_bond_tier(2_999_999_999, MIN_GREEN_LABEL_BASE_BOND_USDC).unwrap(),
            BondTier::Silver
        );
    }

    #[test]
    fn bond_tier_gold() {
        assert_eq!(
            calculate_bond_tier(3_000_000_000, MIN_GREEN_LABEL_BASE_BOND_USDC).unwrap(),
            BondTier::Gold
        );
        assert_eq!(
            calculate_bond_tier(9_999_999_999, MIN_GREEN_LABEL_BASE_BOND_USDC).unwrap(),
            BondTier::Gold
        );
    }

    #[test]
    fn bond_tier_platinum() {
        assert_eq!(
            calculate_bond_tier(10_000_000_000, MIN_GREEN_LABEL_BASE_BOND_USDC).unwrap(),
            BondTier::Platinum
        );
        assert_eq!(
            calculate_bond_tier(100_000_000_000, MIN_GREEN_LABEL_BASE_BOND_USDC).unwrap(),
            BondTier::Platinum
        );
    }

    #[test]
    fn bond_tier_rejects_below_minimum() {
        let err = calculate_bond_tier(298_999_999, MIN_GREEN_LABEL_BASE_BOND_USDC).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBondAmount");
    }

    #[test]
    fn status_transition_pending_bond_deposit_to_pending_observation() {
        validate_green_label_status_transition(
            GreenLabelStatus::PendingBondDeposit,
            GreenLabelStatus::PendingObservation,
            false,
        )
        .unwrap();
    }

    #[test]
    fn status_transition_pending_bond_deposit_to_cancelled() {
        validate_green_label_status_transition(
            GreenLabelStatus::PendingBondDeposit,
            GreenLabelStatus::Cancelled,
            false,
        )
        .unwrap();
    }

    #[test]
    fn status_transition_pending_bond_deposit_rejects_refund_queued() {
        let err = validate_green_label_status_transition(
            GreenLabelStatus::PendingBondDeposit,
            GreenLabelStatus::RefundQueued,
            false,
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn status_transition_pending_bond_deposit_rejects_slash_queued() {
        let err = validate_green_label_status_transition(
            GreenLabelStatus::PendingBondDeposit,
            GreenLabelStatus::SlashQueued,
            true,
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn status_transition_pending_bond_deposit_rejects_active_green_label() {
        let err = validate_green_label_status_transition(
            GreenLabelStatus::PendingBondDeposit,
            GreenLabelStatus::ActiveGreenLabel,
            false,
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn status_transition_pending_bond_deposit_rejects_disputed() {
        let err = validate_green_label_status_transition(
            GreenLabelStatus::PendingBondDeposit,
            GreenLabelStatus::Disputed,
            false,
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn status_transition_pending_to_active() {
        validate_green_label_status_transition(
            GreenLabelStatus::PendingObservation,
            GreenLabelStatus::ActiveGreenLabel,
            false,
        )
        .unwrap();
    }

    #[test]
    fn status_transition_pending_to_disputed() {
        validate_green_label_status_transition(
            GreenLabelStatus::PendingObservation,
            GreenLabelStatus::Disputed,
            false,
        )
        .unwrap();
    }

    #[test]
    fn status_transition_disputed_to_slash_requires_linked_dispute() {
        let err = validate_green_label_status_transition(
            GreenLabelStatus::Disputed,
            GreenLabelStatus::SlashQueued,
            false,
        )
        .unwrap_err();
        assert_error_contains(err, "InvalidGreenLabelSlashWithoutDispute");

        validate_green_label_status_transition(
            GreenLabelStatus::Disputed,
            GreenLabelStatus::SlashQueued,
            true,
        )
        .unwrap();
    }

    #[test]
    fn status_transition_terminal_refunded_rejects_next_transition() {
        let err = validate_green_label_status_transition(
            GreenLabelStatus::Refunded,
            GreenLabelStatus::Disputed,
            false,
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn payload_hash_rejects_zero_hash() {
        let err = validate_payload_hash([0; 32]).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelPayloadHash");
    }

    #[test]
    fn payload_hash_accepts_nonzero_hash() {
        validate_payload_hash([1; 32]).unwrap();
    }

    #[test]
    fn green_label_config_space_is_at_least_expected_minimum() {
        let minimum = ANCHOR_ACCOUNT_DISCRIMINATOR_BYTES
            + (32 * 7)
            + (8 * 2)
            + (2 * 2)
            + (8 * 3)
            + 1
            + 1
            + GREEN_LABEL_CONFIG_RESERVED_BYTES;

        assert!(expected_green_label_config_space() >= minimum);
        assert_eq!(
            GreenLabelConfigV1::INIT_SPACE + ANCHOR_ACCOUNT_DISCRIMINATOR_BYTES,
            expected_green_label_config_space()
        );
    }

    #[test]
    fn green_label_project_space_is_at_least_expected_minimum() {
        let minimum = ANCHOR_ACCOUNT_DISCRIMINATOR_BYTES
            + (32 * 11)
            + (8 * 12)
            + 1
            + 1
            + 2
            + 1
            + 1
            + GREEN_LABEL_PROJECT_RESERVED_BYTES;

        assert!(expected_green_label_project_space() >= minimum);
        assert_eq!(
            GreenLabelProjectV1::INIT_SPACE + ANCHOR_ACCOUNT_DISCRIMINATOR_BYTES,
            expected_green_label_project_space()
        );
    }

    #[test]
    fn green_label_dispute_space_is_at_least_expected_minimum() {
        let minimum = ANCHOR_ACCOUNT_DISCRIMINATOR_BYTES
            + (32 * 6)
            + (8 * 7)
            + 3
            + 1
            + GREEN_LABEL_DISPUTE_RESERVED_BYTES;

        assert!(expected_green_label_dispute_space() >= minimum);
        assert_eq!(
            GreenLabelDisputeV1::INIT_SPACE + ANCHOR_ACCOUNT_DISCRIMINATOR_BYTES,
            expected_green_label_dispute_space()
        );
    }

    #[test]
    fn green_label_config_reserved_space_is_128() {
        assert_eq!(
            green_label_config().reserved.len(),
            GREEN_LABEL_CONFIG_RESERVED_BYTES
        );
        assert_eq!(GREEN_LABEL_CONFIG_RESERVED_BYTES, 128);
    }

    #[test]
    fn green_label_project_reserved_space_is_160() {
        assert_eq!(
            green_label_project().reserved.len(),
            GREEN_LABEL_PROJECT_RESERVED_BYTES
        );
        assert_eq!(GREEN_LABEL_PROJECT_RESERVED_BYTES, 160);
    }

    #[test]
    fn green_label_dispute_reserved_space_is_128() {
        assert_eq!(
            green_label_dispute().reserved.len(),
            GREEN_LABEL_DISPUTE_RESERVED_BYTES
        );
        assert_eq!(GREEN_LABEL_DISPUTE_RESERVED_BYTES, 128);
    }

    #[test]
    fn derive_bond_split_and_tier_for_299() {
        let (split, tier) =
            derive_bond_split_and_tier(299_000_000, MIN_GREEN_LABEL_BASE_BOND_USDC).unwrap();

        assert_eq!(split.base_bond_amount, 299_000_000);
        assert_eq!(split.extra_bond_amount, 0);
        assert_eq!(tier, BondTier::Base);
    }

    #[test]
    fn derive_bond_split_and_tier_for_1299() {
        let (split, tier) =
            derive_bond_split_and_tier(1_299_000_000, MIN_GREEN_LABEL_BASE_BOND_USDC).unwrap();

        assert_eq!(split.base_bond_amount, 299_000_000);
        assert_eq!(split.extra_bond_amount, 1_000_000_000);
        assert_eq!(tier, BondTier::Silver);
    }

    #[test]
    fn validate_terminal_refund_accepts_green_label_refund() {
        validate_terminal_action_for_refund(ActionType::GreenLabelRefund).unwrap();
    }

    #[test]
    fn validate_terminal_refund_rejects_green_label_slash() {
        let err = validate_terminal_action_for_refund(ActionType::GreenLabelSlash).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelActionType");
    }

    #[test]
    fn validate_terminal_slash_accepts_green_label_slash_with_dispute() {
        validate_terminal_action_for_slash(ActionType::GreenLabelSlash, true).unwrap();
    }

    #[test]
    fn validate_terminal_slash_rejects_green_label_slash_without_dispute() {
        let err =
            validate_terminal_action_for_slash(ActionType::GreenLabelSlash, false).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelSlashWithoutDispute");
    }

    #[test]
    fn validate_terminal_slash_rejects_green_label_refund() {
        let err =
            validate_terminal_action_for_slash(ActionType::GreenLabelRefund, true).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelActionType");
    }

    #[test]
    fn account_structs_are_clone_debug_serializable_if_possible() {
        let mut config_data = Vec::new();
        green_label_config()
            .try_serialize(&mut config_data)
            .unwrap();
        assert!(!config_data.is_empty());

        let mut project_data = Vec::new();
        green_label_project()
            .try_serialize(&mut project_data)
            .unwrap();
        assert!(!project_data.is_empty());

        let mut dispute_data = Vec::new();
        green_label_dispute()
            .try_serialize(&mut dispute_data)
            .unwrap();
        assert!(!dispute_data.is_empty());

        let tier = BondTier::Base;
        let tier_copy = tier;
        assert_eq!(format!("{tier_copy:?}"), "Base");
    }

    #[test]
    fn pda_seed_constants_are_non_empty_and_distinct() {
        let seeds = [
            GREEN_LABEL_CONFIG_SEED,
            GREEN_LABEL_PROJECT_SEED,
            GREEN_LABEL_DISPUTE_SEED,
            GREEN_BOND_VAULT_SEED,
            GREEN_BOND_VAULT_AUTHORITY_SEED,
            GREEN_LABEL_REFUNDABLE_ESCROW_SEED,
            GREEN_LABEL_REFUNDABLE_VAULT_SEED,
        ];

        for seed in seeds {
            assert!(!seed.is_empty());
        }

        for (index, seed) in seeds.iter().enumerate() {
            for other in seeds.iter().skip(index + 1) {
                assert_ne!(seed, other);
            }
        }
    }

    #[test]
    fn escrow_remaining_amount_for_full_refundable_balance() {
        let remaining = calculate_green_label_escrow_remaining_amount(100, 0, 0).unwrap();

        assert_eq!(remaining, 100);
    }

    #[test]
    fn escrow_remaining_amount_after_partial_refund() {
        let remaining = calculate_green_label_escrow_remaining_amount(100, 20, 0).unwrap();

        assert_eq!(remaining, 80);
    }

    #[test]
    fn escrow_remaining_amount_after_full_forfeit_is_zero() {
        let remaining = calculate_green_label_escrow_remaining_amount(100, 0, 100).unwrap();

        assert_eq!(remaining, 0);
    }

    #[test]
    fn escrow_refund_rejects_refunded_status() {
        let err = validate_green_label_escrow_refund(
            GreenLabelEscrowStatusV1::Refunded,
            100,
            0,
            0,
            10,
            11,
            Pubkey::default(),
            ActionType::Noop,
            0,
            [0; 32],
            Pubkey::new_from_array([1; 32]),
            Pubkey::new_from_array([1; 32]),
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([3; 32]),
            Pubkey::new_from_array([3; 32]),
            Pubkey::new_from_array([2; 32]),
            GREEN_LABEL_USDC_DECIMALS,
            100,
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelEscrowStatus");
    }

    #[test]
    fn escrow_refund_allows_time_path_without_active_dispute() {
        let amount = validate_green_label_escrow_refund(
            GreenLabelEscrowStatusV1::Locked,
            100,
            0,
            0,
            10,
            10,
            Pubkey::default(),
            ActionType::Noop,
            0,
            [0; 32],
            Pubkey::new_from_array([1; 32]),
            Pubkey::new_from_array([1; 32]),
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([3; 32]),
            Pubkey::new_from_array([3; 32]),
            Pubkey::new_from_array([2; 32]),
            GREEN_LABEL_USDC_DECIMALS,
            100,
        )
        .unwrap();

        assert_eq!(amount, 100);
    }

    #[test]
    fn escrow_refund_allows_linked_refund_decision_path() {
        let amount = validate_green_label_escrow_refund(
            GreenLabelEscrowStatusV1::Locked,
            100,
            0,
            0,
            100,
            10,
            Pubkey::new_from_array([9; 32]),
            ActionType::GreenLabelRefund,
            1,
            [8; 32],
            Pubkey::new_from_array([1; 32]),
            Pubkey::new_from_array([1; 32]),
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([3; 32]),
            Pubkey::new_from_array([3; 32]),
            Pubkey::new_from_array([2; 32]),
            GREEN_LABEL_USDC_DECIMALS,
            100,
        )
        .unwrap();

        assert_eq!(amount, 100);
    }

    #[test]
    fn escrow_refund_rejects_active_dispute_without_refund_decision() {
        let err = validate_green_label_escrow_refund(
            GreenLabelEscrowStatusV1::Locked,
            100,
            0,
            0,
            10,
            11,
            Pubkey::new_from_array([9; 32]),
            ActionType::Noop,
            0,
            [0; 32],
            Pubkey::new_from_array([1; 32]),
            Pubkey::new_from_array([1; 32]),
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([3; 32]),
            Pubkey::new_from_array([3; 32]),
            Pubkey::new_from_array([2; 32]),
            GREEN_LABEL_USDC_DECIMALS,
            100,
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelEscrowRefund");
    }

    #[test]
    fn escrow_forfeit_rejects_time_only_without_slash_decision() {
        let err = validate_green_label_escrow_forfeit_to_treasury(
            false,
            GreenLabelStatus::PendingObservation,
            Pubkey::default(),
            Pubkey::new_from_array([9; 32]),
            0,
            Pubkey::default(),
            Pubkey::default(),
            [0; 32],
            ActionType::Noop,
            Pubkey::new_from_array([4; 32]),
            Pubkey::new_from_array([4; 32]),
            DisputeStatus::ReadyForDecision,
            0,
            Pubkey::default(),
            Pubkey::default(),
            [0; 32],
            ActionType::Noop,
            Pubkey::default(),
            0,
            ProposalDecision::Pending,
            Pubkey::default(),
            0,
            ExecutionStatus::Queued,
            ActionType::Noop,
            [0; 32],
            crate::ID,
            crate::ID,
            Pubkey::new_from_array([9; 32]),
            Pubkey::new_from_array([9; 32]),
            100,
            0,
            GreenLabelEscrowStatusV1::Locked,
            100,
            0,
            0,
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([3; 32]),
            Pubkey::new_from_array([3; 32]),
            Pubkey::new_from_array([2; 32]),
            GREEN_LABEL_USDC_DECIMALS,
            100,
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn escrow_forfeit_accepts_linked_slash_decision_path() {
        let project_key = Pubkey::new_from_array([4; 32]);
        let dispute_key = Pubkey::new_from_array([9; 32]);
        let proposal_decision_key = Pubkey::new_from_array([5; 32]);
        let queue_key = Pubkey::new_from_array([6; 32]);
        let payload_hash = [7; 32];

        let amount = validate_green_label_escrow_forfeit_to_treasury(
            false,
            GreenLabelStatus::SlashQueued,
            dispute_key,
            dispute_key,
            1,
            proposal_decision_key,
            queue_key,
            payload_hash,
            ActionType::GreenLabelSlash,
            project_key,
            project_key,
            DisputeStatus::DecisionQueued,
            1,
            proposal_decision_key,
            queue_key,
            payload_hash,
            ActionType::GreenLabelSlash,
            proposal_decision_key,
            1,
            ProposalDecision::Approved,
            queue_key,
            1,
            ExecutionStatus::Queued,
            ActionType::GreenLabelSlash,
            payload_hash,
            crate::ID,
            crate::ID,
            dispute_key,
            dispute_key,
            100,
            50,
            GreenLabelEscrowStatusV1::Locked,
            100,
            0,
            0,
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([3; 32]),
            Pubkey::new_from_array([3; 32]),
            Pubkey::new_from_array([2; 32]),
            GREEN_LABEL_USDC_DECIMALS,
            100,
        )
        .unwrap();

        assert_eq!(amount, 100);
    }

    fn green_label_config() -> GreenLabelConfigV1 {
        GreenLabelConfigV1 {
            authority: Pubkey::new_from_array([1; 32]),
            usdc_mint: Pubkey::new_from_array([2; 32]),
            min_base_bond_usdc: MIN_GREEN_LABEL_BASE_BOND_USDC,
            base_refund_bps: BASE_BOND_REFUND_BPS,
            base_treasury_bps: BASE_BOND_TREASURY_BPS,
            observation_period_seconds: 30,
            dispute_window_seconds: 7,
            response_window_seconds: 3,
            project_count: 0,
            treasury_usdc_state_v2: Pubkey::new_from_array([3; 32]),
            base_bond_treasury_vault: Pubkey::new_from_array([4; 32]),
            relief_or_risk_vault: Pubkey::new_from_array([5; 32]),
            vault_authority_v2: Pubkey::new_from_array([6; 32]),
            security_governance_config: Pubkey::new_from_array([7; 32]),
            is_paused: false,
            bump: 250,
            reserved: [0; GREEN_LABEL_CONFIG_RESERVED_BYTES],
        }
    }

    struct FeeValidationFixture {
        config_key: Pubkey,
        config: GreenLabelConfigV1,
        project_key: Pubkey,
        project: GreenLabelProjectV1,
        fee_policy_key: Pubkey,
        policy: GreenLabelCertificationFeePolicyV1,
        payer: Pubkey,
        payer_token_account_key: Pubkey,
        payer_token_account: TokenAccount,
        treasury_config_key: Pubkey,
        treasury_config: TreasuryConfigV2,
        treasury_usdc_state_key: Pubkey,
        treasury_usdc_state: TreasuryUsdcStateV2,
        revenue_routing_stats_key: Pubkey,
        revenue_routing_stats: RevenueRoutingStatsV1,
        vault_authority: Pubkey,
        relief_vault_key: Pubkey,
        relief_vault: TokenAccount,
        buyback_vault_key: Pubkey,
        buyback_vault: TokenAccount,
        builders_vault_key: Pubkey,
        builders_vault: TokenAccount,
        staking_vault_key: Pubkey,
        staking_vault: TokenAccount,
        usdc_decimals: u8,
    }

    impl FeeValidationFixture {
        fn parameters(&self) -> GreenLabelCertificationFeeParametersV1 {
            build_green_label_certification_fee_parameters_v1(
                self.config_key,
                self.fee_policy_key,
                &self.policy,
                self.project_key,
                &self.project,
                self.payer,
                self.payer_token_account_key,
                self.treasury_config_key,
                self.treasury_usdc_state_key,
                self.revenue_routing_stats_key,
                self.relief_vault_key,
                self.buyback_vault_key,
                self.builders_vault_key,
                self.staking_vault_key,
            )
            .unwrap()
        }

        fn validate(&self) -> Result<[u8; 32]> {
            validate_green_label_certification_fee_once_v1(
                &self.config,
                self.config_key,
                &self.project,
                self.project_key,
                &self.policy,
                self.fee_policy_key,
                &self.payer_token_account,
                self.payer_token_account_key,
                self.payer,
                &self.treasury_config,
                self.treasury_config_key,
                &self.treasury_usdc_state,
                self.treasury_usdc_state_key,
                &self.revenue_routing_stats,
                self.revenue_routing_stats_key,
                self.vault_authority,
                self.relief_vault_key,
                &self.relief_vault,
                self.buyback_vault_key,
                &self.buyback_vault,
                self.builders_vault_key,
                &self.builders_vault,
                self.staking_vault_key,
                &self.staking_vault,
                self.config.usdc_mint,
                self.usdc_decimals,
                &self.parameters(),
            )
        }
    }

    fn fee_validation_fixture() -> FeeValidationFixture {
        let mint = Pubkey::new_from_array([2; 32]);
        let config_key = Pubkey::new_from_array([30; 32]);
        let project_key = Pubkey::new_from_array([31; 32]);
        let fee_policy_key = Pubkey::new_from_array([32; 32]);
        let payer = Pubkey::new_from_array([33; 32]);
        let payer_token_account_key = Pubkey::new_from_array([34; 32]);
        let treasury_config_key = Pubkey::new_from_array([35; 32]);
        let treasury_usdc_state_key = Pubkey::new_from_array([36; 32]);
        let revenue_routing_stats_key = Pubkey::new_from_array([37; 32]);
        let vault_authority = Pubkey::new_from_array([38; 32]);
        let relief_vault_key = Pubkey::new_from_array([39; 32]);
        let buyback_vault_key = Pubkey::new_from_array([40; 32]);
        let builders_vault_key = Pubkey::new_from_array([41; 32]);
        let staking_vault_key = Pubkey::new_from_array([42; 32]);
        let treasury_authority = Pubkey::new_from_array([43; 32]);

        let mut config = green_label_config();
        config.usdc_mint = mint;
        config.treasury_usdc_state_v2 = treasury_usdc_state_key;
        config.vault_authority_v2 = vault_authority;

        let mut project = green_label_project();
        project.project_id = 7;
        project.project_owner = payer;
        project.status = GreenLabelStatus::PendingBondDeposit;

        FeeValidationFixture {
            config_key,
            config,
            project_key,
            project,
            fee_policy_key,
            policy: certification_fee_policy(config_key, mint, 1_000_000),
            payer,
            payer_token_account_key,
            payer_token_account: token_account(mint, payer, 2_000_000),
            treasury_config_key,
            treasury_config: TreasuryConfigV2 {
                authority: treasury_authority,
                usdc_mint: mint,
                alpha_mint: Pubkey::new_from_array([44; 32]),
                bump: 8,
            },
            treasury_usdc_state_key,
            treasury_usdc_state: TreasuryUsdcStateV2 {
                total_usdc_inflow: 0,
                relief_usdc_total: 0,
                buyback_usdc_total: 0,
                builders_usdc_total: 0,
                staking_usdc_total: 0,
                bump: 9,
            },
            revenue_routing_stats_key,
            revenue_routing_stats: RevenueRoutingStatsV1 {
                authority: treasury_authority,
                usdc_mint: mint,
                total_routed_usdc: 0,
                green_label_certification_fee_total: 0,
                green_label_forfeited_bond_total: 0,
                protocol_service_fee_total: 0,
                platform_revenue_total: 0,
                partnership_revenue_total: 0,
                manual_governance_approved_revenue_total: 0,
                bump: 10,
            },
            vault_authority,
            relief_vault_key,
            relief_vault: token_account(mint, vault_authority, 0),
            buyback_vault_key,
            buyback_vault: token_account(mint, vault_authority, 0),
            builders_vault_key,
            builders_vault: token_account(mint, vault_authority, 0),
            staking_vault_key,
            staking_vault: token_account(mint, vault_authority, 0),
            usdc_decimals: GREEN_LABEL_USDC_DECIMALS,
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

    fn blank_certification_fee_policy() -> GreenLabelCertificationFeePolicyV1 {
        GreenLabelCertificationFeePolicyV1 {
            green_label_config: Pubkey::default(),
            usdc_mint: Pubkey::default(),
            fee_amount_usdc: 0,
            policy_version: 0,
            active: false,
            initialized_by: Pubkey::default(),
            created_at: 0,
            schema_version: 0,
            bump: 0,
        }
    }

    fn certification_fee_policy(
        green_label_config: Pubkey,
        usdc_mint: Pubkey,
        fee_amount_usdc: u64,
    ) -> GreenLabelCertificationFeePolicyV1 {
        GreenLabelCertificationFeePolicyV1 {
            green_label_config,
            usdc_mint,
            fee_amount_usdc,
            policy_version: GREEN_LABEL_CERTIFICATION_FEE_POLICY_VERSION,
            active: true,
            initialized_by: Pubkey::new_from_array([1; 32]),
            created_at: 100,
            schema_version: GREEN_LABEL_CERTIFICATION_FEE_SCHEMA_VERSION,
            bump: 7,
        }
    }

    fn blank_certification_fee_receipt() -> GreenLabelCertificationFeeReceiptV1 {
        GreenLabelCertificationFeeReceiptV1 {
            green_label_config: Pubkey::default(),
            fee_policy: Pubkey::default(),
            policy_version: 0,
            green_label_project: Pubkey::default(),
            project_id: 0,
            project_owner: Pubkey::default(),
            payer: Pubkey::default(),
            payer_token_account: Pubkey::default(),
            amount_usdc: 0,
            usdc_mint: Pubkey::default(),
            treasury_config: Pubkey::default(),
            treasury_usdc_state: Pubkey::default(),
            revenue_routing_stats: Pubkey::default(),
            relief_usdc_vault: Pubkey::default(),
            buyback_usdc_vault: Pubkey::default(),
            builders_usdc_vault: Pubkey::default(),
            staking_usdc_vault: Pubkey::default(),
            revenue_type: RevenueType::GreenLabelCertificationFee,
            parameters_hash: [0; 32],
            routed_at: 0,
            schema_version: 0,
            bump: 0,
        }
    }

    fn blank_certification_state() -> GreenLabelCertificationStateV1 {
        GreenLabelCertificationStateV1 {
            green_label_project: Pubkey::default(),
            green_label_config: Pubkey::default(),
            certification_status: GreenLabelCertificationStatusV1::Pending,
            last_governance_proposal: Pubkey::default(),
            last_execution_queue: Pubkey::default(),
            last_execution_record: Pubkey::default(),
            last_action_type: GovernanceActionTypeV1::GreenLabelApproveCertification,
            decision_at: 0,
            created_at: 0,
            updated_at: 0,
            schema_version: 0,
            bump: 0,
        }
    }

    fn certification_state(
        project_key: Pubkey,
        config_key: Pubkey,
    ) -> GreenLabelCertificationStateV1 {
        GreenLabelCertificationStateV1 {
            green_label_project: project_key,
            green_label_config: config_key,
            certification_status: GreenLabelCertificationStatusV1::Pending,
            last_governance_proposal: Pubkey::default(),
            last_execution_queue: Pubkey::default(),
            last_execution_record: Pubkey::default(),
            last_action_type: GovernanceActionTypeV1::GreenLabelApproveCertification,
            decision_at: 0,
            created_at: 100,
            updated_at: 100,
            schema_version: GREEN_LABEL_CERTIFICATION_SCHEMA_VERSION,
            bump: 7,
        }
    }

    fn blank_certification_execution_record() -> GreenLabelCertificationExecutionRecordV1 {
        GreenLabelCertificationExecutionRecordV1 {
            execution_queue_item: Pubkey::default(),
            proposal_decision: Pubkey::default(),
            governance_proposal: Pubkey::default(),
            governance_proposal_action: Pubkey::default(),
            green_label_project: Pubkey::default(),
            certification_state: Pubkey::default(),
            module_registry: Pubkey::default(),
            execution_type: GreenLabelCertificationExecutionTypeV1::Approve,
            governance_action_type: GovernanceActionTypeV1::GreenLabelApproveCertification,
            target_account: Pubkey::default(),
            parameters_hash: [0; 32],
            canonical_governance_payload_hash: [0; 32],
            project_status_before: GreenLabelStatus::PendingBondDeposit,
            project_status_after: GreenLabelStatus::PendingBondDeposit,
            certification_status_before: GreenLabelCertificationStatusV1::Pending,
            certification_status_after: GreenLabelCertificationStatusV1::Pending,
            executor: Pubkey::default(),
            executed_at: 0,
            schema_version: 0,
            bump: 0,
        }
    }

    fn refundable_escrow(
        amount: u64,
        status: GreenLabelEscrowStatusV1,
    ) -> GreenLabelRefundableEscrowV1 {
        GreenLabelRefundableEscrowV1 {
            authority: Pubkey::new_from_array([1; 32]),
            project: Pubkey::new_from_array([2; 32]),
            project_id: 1,
            payer: Pubkey::new_from_array([3; 32]),
            usdc_mint: Pubkey::new_from_array([4; 32]),
            refundable_vault: Pubkey::new_from_array([5; 32]),
            deposited_amount: amount,
            refundable_amount: amount,
            refunded_amount: 0,
            forfeited_amount: 0,
            deposit_ts: 100,
            refund_available_after: 200,
            status,
            bump: 7,
            vault_bump: 8,
        }
    }

    fn blank_refund_execution_record() -> GreenLabelRefundExecutionRecordV1 {
        GreenLabelRefundExecutionRecordV1 {
            execution_queue_item: Pubkey::default(),
            proposal_decision: Pubkey::default(),
            governance_proposal: Pubkey::default(),
            governance_proposal_action: Pubkey::default(),
            module_registry: Pubkey::default(),
            green_label_config: Pubkey::default(),
            green_label_project: Pubkey::default(),
            green_label_dispute: Pubkey::default(),
            refundable_escrow: Pubkey::default(),
            refundable_vault: Pubkey::default(),
            original_payer: Pubkey::default(),
            payer_destination_token_account: Pubkey::default(),
            refund_amount_usdc: 0,
            usdc_mint: Pubkey::default(),
            execution_type: GreenLabelEscrowExecutionTypeV1::Refund,
            governance_action_type: GovernanceActionTypeV1::GreenLabelRefundBond,
            parameters_hash: [0; 32],
            canonical_governance_payload_hash: [0; 32],
            escrow_status_before: GreenLabelEscrowStatusV1::Locked,
            escrow_status_after: GreenLabelEscrowStatusV1::Locked,
            project_status_before: GreenLabelStatus::PendingBondDeposit,
            project_status_after: GreenLabelStatus::PendingBondDeposit,
            executor: Pubkey::default(),
            executed_at: 0,
            schema_version: 0,
            bump: 0,
        }
    }

    fn blank_forfeit_execution_record() -> GreenLabelForfeitExecutionRecordV1 {
        GreenLabelForfeitExecutionRecordV1 {
            execution_queue_item: Pubkey::default(),
            proposal_decision: Pubkey::default(),
            governance_proposal: Pubkey::default(),
            governance_proposal_action: Pubkey::default(),
            module_registry: Pubkey::default(),
            green_label_config: Pubkey::default(),
            green_label_project: Pubkey::default(),
            green_label_dispute: Pubkey::default(),
            refundable_escrow: Pubkey::default(),
            refundable_vault: Pubkey::default(),
            treasury_config: Pubkey::default(),
            treasury_usdc_state: Pubkey::default(),
            revenue_routing_stats: Pubkey::default(),
            relief_usdc_vault: Pubkey::default(),
            buyback_usdc_vault: Pubkey::default(),
            builders_usdc_vault: Pubkey::default(),
            staking_usdc_vault: Pubkey::default(),
            forfeited_amount_usdc: 0,
            usdc_mint: Pubkey::default(),
            revenue_type: RevenueType::GreenLabelForfeitedBond,
            execution_type: GreenLabelEscrowExecutionTypeV1::Forfeit,
            governance_action_type: GovernanceActionTypeV1::GreenLabelSlashBond,
            parameters_hash: [0; 32],
            canonical_governance_payload_hash: [0; 32],
            escrow_status_before: GreenLabelEscrowStatusV1::Locked,
            escrow_status_after: GreenLabelEscrowStatusV1::Locked,
            project_status_before: GreenLabelStatus::PendingBondDeposit,
            project_status_after: GreenLabelStatus::PendingBondDeposit,
            dispute_status_before: DisputeStatus::ReadyForDecision,
            dispute_status_after: DisputeStatus::ReadyForDecision,
            executor: Pubkey::default(),
            executed_at: 0,
            schema_version: 0,
            bump: 0,
        }
    }

    fn green_label_config_init_values() -> GreenLabelConfigInitValues {
        build_default_green_label_config_values(
            Pubkey::new_from_array([1; 32]),
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([3; 32]),
            Pubkey::new_from_array([4; 32]),
            Pubkey::new_from_array([5; 32]),
            Pubkey::new_from_array([6; 32]),
            Pubkey::new_from_array([7; 32]),
            250,
        )
        .unwrap()
    }

    fn pending_bond_project_values(total_bond_amount: u64) -> GreenLabelProjectInitValues {
        try_pending_bond_project_values(
            false,
            MIN_GREEN_LABEL_BASE_BOND_USDC,
            0,
            1,
            total_bond_amount,
        )
        .unwrap()
    }

    fn pending_bond_project_values_with_min(
        configured_min_base_bond_usdc: u64,
        total_bond_amount: u64,
    ) -> GreenLabelProjectInitValues {
        try_pending_bond_project_values(
            false,
            configured_min_base_bond_usdc,
            0,
            1,
            total_bond_amount,
        )
        .unwrap()
    }

    fn try_pending_bond_project_values(
        is_config_paused: bool,
        configured_min_base_bond_usdc: u64,
        current_project_count: u64,
        expected_project_id: u64,
        total_bond_amount: u64,
    ) -> Result<GreenLabelProjectInitValues> {
        build_pending_bond_project_values(
            is_config_paused,
            configured_min_base_bond_usdc,
            current_project_count,
            expected_project_id,
            Pubkey::new_from_array([8; 32]),
            [9; 32],
            [10; 32],
            Pubkey::new_from_array([11; 32]),
            Pubkey::new_from_array([12; 32]),
            total_bond_amount,
            1_717_171_717,
            249,
        )
    }

    #[derive(Clone, Copy)]
    struct BondLockValidationFixture {
        config_is_paused: bool,
        project_owner: Pubkey,
        signer: Pubkey,
        project_status: GreenLabelStatus,
        bond_vault: Pubkey,
        bond_vault_authority: Pubkey,
        provided_bond_vault: Pubkey,
        provided_bond_vault_mint: Pubkey,
        provided_bond_vault_owner: Pubkey,
        expected_usdc_mint: Pubkey,
        owner_ata_owner: Pubkey,
        owner_ata_mint: Pubkey,
        usdc_mint: Pubkey,
        base_bond_amount: u64,
        extra_bond_amount: u64,
        total_bond_amount: u64,
    }

    impl BondLockValidationFixture {
        fn valid() -> Self {
            let project_owner = Pubkey::new_from_array([8; 32]);
            let bond_vault = Pubkey::new_from_array([13; 32]);
            let bond_vault_authority = Pubkey::new_from_array([14; 32]);
            let usdc_mint = Pubkey::new_from_array([2; 32]);

            Self {
                config_is_paused: false,
                project_owner,
                signer: project_owner,
                project_status: GreenLabelStatus::PendingBondDeposit,
                bond_vault,
                bond_vault_authority,
                provided_bond_vault: bond_vault,
                provided_bond_vault_mint: usdc_mint,
                provided_bond_vault_owner: bond_vault_authority,
                expected_usdc_mint: usdc_mint,
                owner_ata_owner: project_owner,
                owner_ata_mint: usdc_mint,
                usdc_mint,
                base_bond_amount: 299_000_000,
                extra_bond_amount: 1_000_000_000,
                total_bond_amount: 1_299_000_000,
            }
        }
    }

    fn validate_bond_lock_fixture(fixture: BondLockValidationFixture) -> Result<()> {
        validate_green_label_bond_lock(
            fixture.config_is_paused,
            fixture.project_owner,
            fixture.signer,
            fixture.project_status,
            fixture.bond_vault,
            fixture.bond_vault_authority,
            fixture.provided_bond_vault,
            fixture.provided_bond_vault_mint,
            fixture.provided_bond_vault_owner,
            fixture.expected_usdc_mint,
            fixture.owner_ata_owner,
            fixture.owner_ata_mint,
            fixture.usdc_mint,
            fixture.base_bond_amount,
            fixture.extra_bond_amount,
            fixture.total_bond_amount,
        )
    }

    #[derive(Clone, Copy)]
    struct MarkReadyValidationFixture {
        config_is_paused: bool,
        project_status: GreenLabelStatus,
        project_active_dispute: Pubkey,
        dispute_key: Pubkey,
        dispute_project: Pubkey,
        project_key: Pubkey,
        dispute_status: DisputeStatus,
        now: i64,
        response_end_ts: i64,
    }

    impl MarkReadyValidationFixture {
        fn valid() -> Self {
            let project_key = Pubkey::new_from_array([17; 32]);
            let dispute_key = Pubkey::new_from_array([18; 32]);

            Self {
                config_is_paused: false,
                project_status: GreenLabelStatus::Disputed,
                project_active_dispute: dispute_key,
                dispute_key,
                dispute_project: project_key,
                project_key,
                dispute_status: DisputeStatus::EvidencePeriod,
                now: 1_000,
                response_end_ts: 1_000,
            }
        }
    }

    fn validate_mark_ready_fixture(fixture: MarkReadyValidationFixture) -> Result<()> {
        validate_mark_dispute_ready(
            fixture.config_is_paused,
            fixture.project_status,
            fixture.project_active_dispute,
            fixture.dispute_key,
            fixture.dispute_project,
            fixture.project_key,
            fixture.dispute_status,
            fixture.now,
            fixture.response_end_ts,
        )
    }

    #[derive(Clone, Copy)]
    struct LinkDecisionValidationFixture {
        config_is_paused: bool,
        project_status: GreenLabelStatus,
        project_active_dispute: Pubkey,
        dispute_key: Pubkey,
        dispute_project: Pubkey,
        project_key: Pubkey,
        dispute_status: DisputeStatus,
        expected_proposal_id: u64,
        expected_action_type: ActionType,
        expected_payload_hash: [u8; 32],
        proposal_id: u64,
        proposal_type: ProposalType,
        proposal_decision: ProposalDecision,
        queue_proposal_id: u64,
        queue_action_type: ActionType,
        queue_status: ExecutionStatus,
        queue_payload_hash: [u8; 32],
        queue_target_program: Pubkey,
        expected_program_id: Pubkey,
        queue_target_account: Pubkey,
        expected_target_account: Pubkey,
    }

    impl LinkDecisionValidationFixture {
        fn valid(action_type: ActionType) -> Self {
            let project_key = Pubkey::new_from_array([17; 32]);
            let dispute_key = Pubkey::new_from_array([18; 32]);
            let program_id = Pubkey::new_from_array([19; 32]);
            let payload_hash = [23; 32];
            let proposal_type = match action_type {
                ActionType::GreenLabelSlash => ProposalType::GreenLabelSlash,
                ActionType::GreenLabelRefund => ProposalType::GreenLabelRefund,
                _ => ProposalType::EmergencyPause,
            };

            Self {
                config_is_paused: false,
                project_status: GreenLabelStatus::Disputed,
                project_active_dispute: dispute_key,
                dispute_key,
                dispute_project: project_key,
                project_key,
                dispute_status: DisputeStatus::ReadyForDecision,
                expected_proposal_id: 7,
                expected_action_type: action_type,
                expected_payload_hash: payload_hash,
                proposal_id: 7,
                proposal_type,
                proposal_decision: ProposalDecision::Approved,
                queue_proposal_id: 7,
                queue_action_type: action_type,
                queue_status: ExecutionStatus::Queued,
                queue_payload_hash: payload_hash,
                queue_target_program: program_id,
                expected_program_id: program_id,
                queue_target_account: dispute_key,
                expected_target_account: dispute_key,
            }
        }
    }

    fn validate_link_decision_fixture(fixture: LinkDecisionValidationFixture) -> Result<()> {
        validate_green_label_security_decision_link(
            fixture.config_is_paused,
            fixture.project_status,
            fixture.project_active_dispute,
            fixture.dispute_key,
            fixture.dispute_project,
            fixture.project_key,
            fixture.dispute_status,
            fixture.expected_proposal_id,
            fixture.expected_action_type,
            fixture.expected_payload_hash,
            fixture.proposal_id,
            fixture.proposal_type,
            fixture.proposal_decision,
            fixture.queue_proposal_id,
            fixture.queue_action_type,
            fixture.queue_status,
            fixture.queue_payload_hash,
            fixture.queue_target_program,
            fixture.expected_program_id,
            fixture.queue_target_account,
            fixture.expected_target_account,
        )
    }

    #[derive(Clone, Copy)]
    struct RefundExecutionValidationFixture {
        config_is_paused: bool,
        project_status: GreenLabelStatus,
        project_active_dispute: Pubkey,
        dispute_key: Pubkey,
        project_bond_vault: Pubkey,
        project_bond_vault_authority: Pubkey,
        project_owner: Pubkey,
        project_terminal_proposal_id: u64,
        project_terminal_proposal_decision: Pubkey,
        project_terminal_execution_queue_item: Pubkey,
        project_terminal_payload_hash: [u8; 32],
        project_terminal_action_type: ActionType,
        dispute_project: Pubkey,
        project_key: Pubkey,
        dispute_status: DisputeStatus,
        dispute_proposal_id: u64,
        dispute_proposal_decision: Pubkey,
        dispute_execution_queue_item: Pubkey,
        dispute_payload_hash: [u8; 32],
        dispute_action_type: ActionType,
        proposal_decision_key: Pubkey,
        proposal_decision_proposal_id: u64,
        proposal_decision: ProposalDecision,
        execution_queue_item_key: Pubkey,
        queue_proposal_id: u64,
        queue_status: ExecutionStatus,
        queue_action_type: ActionType,
        queue_payload_hash: [u8; 32],
        queue_target_program: Pubkey,
        expected_program_id: Pubkey,
        queue_target_account: Pubkey,
        expected_target_account: Pubkey,
        now: i64,
        queue_execute_after: i64,
        provided_bond_vault: Pubkey,
        green_bond_vault_mint: Pubkey,
        green_bond_vault_owner: Pubkey,
        provided_bond_vault_authority: Pubkey,
        project_owner_ata_owner: Pubkey,
        project_owner_ata_mint: Pubkey,
        provided_treasury_vault: Pubkey,
        treasury_vault_mint: Pubkey,
        expected_treasury_vault: Pubkey,
        expected_usdc_mint: Pubkey,
        provided_usdc_mint: Pubkey,
        usdc_decimals: u8,
        vault_balance: u64,
        project_refund_amount: u64,
        treasury_amount: u64,
    }

    impl RefundExecutionValidationFixture {
        fn valid() -> Self {
            let project_owner = Pubkey::new_from_array([8; 32]);
            let bond_vault = Pubkey::new_from_array([13; 32]);
            let bond_vault_authority = Pubkey::new_from_array([14; 32]);
            let proposal_decision_key = Pubkey::new_from_array([15; 32]);
            let execution_queue_item_key = Pubkey::new_from_array([16; 32]);
            let project_key = Pubkey::new_from_array([17; 32]);
            let dispute_key = Pubkey::new_from_array([18; 32]);
            let program_id = Pubkey::new_from_array([19; 32]);
            let usdc_mint = Pubkey::new_from_array([2; 32]);
            let treasury_vault = Pubkey::new_from_array([4; 32]);
            let payload_hash = [23; 32];

            Self {
                config_is_paused: false,
                project_status: GreenLabelStatus::RefundQueued,
                project_active_dispute: dispute_key,
                dispute_key,
                project_bond_vault: bond_vault,
                project_bond_vault_authority: bond_vault_authority,
                project_owner,
                project_terminal_proposal_id: 7,
                project_terminal_proposal_decision: proposal_decision_key,
                project_terminal_execution_queue_item: execution_queue_item_key,
                project_terminal_payload_hash: payload_hash,
                project_terminal_action_type: ActionType::GreenLabelRefund,
                dispute_project: project_key,
                project_key,
                dispute_status: DisputeStatus::DecisionQueued,
                dispute_proposal_id: 7,
                dispute_proposal_decision: proposal_decision_key,
                dispute_execution_queue_item: execution_queue_item_key,
                dispute_payload_hash: payload_hash,
                dispute_action_type: ActionType::GreenLabelRefund,
                proposal_decision_key,
                proposal_decision_proposal_id: 7,
                proposal_decision: ProposalDecision::Approved,
                execution_queue_item_key,
                queue_proposal_id: 7,
                queue_status: ExecutionStatus::Queued,
                queue_action_type: ActionType::GreenLabelRefund,
                queue_payload_hash: payload_hash,
                queue_target_program: program_id,
                expected_program_id: program_id,
                queue_target_account: dispute_key,
                expected_target_account: dispute_key,
                now: 2_000,
                queue_execute_after: 1_000,
                provided_bond_vault: bond_vault,
                green_bond_vault_mint: usdc_mint,
                green_bond_vault_owner: bond_vault_authority,
                provided_bond_vault_authority: bond_vault_authority,
                project_owner_ata_owner: project_owner,
                project_owner_ata_mint: usdc_mint,
                provided_treasury_vault: treasury_vault,
                treasury_vault_mint: usdc_mint,
                expected_treasury_vault: treasury_vault,
                expected_usdc_mint: usdc_mint,
                provided_usdc_mint: usdc_mint,
                usdc_decimals: GREEN_LABEL_USDC_DECIMALS,
                vault_balance: 1_299_000_000,
                project_refund_amount: 1_239_200_000,
                treasury_amount: 59_800_000,
            }
        }
    }

    fn validate_refund_execution_fixture(fixture: RefundExecutionValidationFixture) -> Result<()> {
        validate_green_label_refund_execution(
            fixture.config_is_paused,
            fixture.project_status,
            fixture.project_active_dispute,
            fixture.dispute_key,
            fixture.project_bond_vault,
            fixture.project_bond_vault_authority,
            fixture.project_owner,
            fixture.project_terminal_proposal_id,
            fixture.project_terminal_proposal_decision,
            fixture.project_terminal_execution_queue_item,
            fixture.project_terminal_payload_hash,
            fixture.project_terminal_action_type,
            fixture.dispute_project,
            fixture.project_key,
            fixture.dispute_status,
            fixture.dispute_proposal_id,
            fixture.dispute_proposal_decision,
            fixture.dispute_execution_queue_item,
            fixture.dispute_payload_hash,
            fixture.dispute_action_type,
            fixture.proposal_decision_key,
            fixture.proposal_decision_proposal_id,
            fixture.proposal_decision,
            fixture.execution_queue_item_key,
            fixture.queue_proposal_id,
            fixture.queue_status,
            fixture.queue_action_type,
            fixture.queue_payload_hash,
            fixture.queue_target_program,
            fixture.expected_program_id,
            fixture.queue_target_account,
            fixture.expected_target_account,
            fixture.now,
            fixture.queue_execute_after,
            fixture.provided_bond_vault,
            fixture.green_bond_vault_mint,
            fixture.green_bond_vault_owner,
            fixture.provided_bond_vault_authority,
            fixture.project_owner_ata_owner,
            fixture.project_owner_ata_mint,
            fixture.provided_treasury_vault,
            fixture.treasury_vault_mint,
            fixture.expected_treasury_vault,
            fixture.expected_usdc_mint,
            fixture.provided_usdc_mint,
            fixture.usdc_decimals,
            fixture.vault_balance,
            fixture.project_refund_amount,
            fixture.treasury_amount,
        )
    }

    #[derive(Clone, Copy)]
    struct SlashExecutionValidationFixture {
        config_is_paused: bool,
        project_status: GreenLabelStatus,
        project_active_dispute: Pubkey,
        dispute_key: Pubkey,
        project_bond_vault: Pubkey,
        project_bond_vault_authority: Pubkey,
        project_terminal_proposal_id: u64,
        project_terminal_proposal_decision: Pubkey,
        project_terminal_execution_queue_item: Pubkey,
        project_terminal_payload_hash: [u8; 32],
        project_terminal_action_type: ActionType,
        dispute_project: Pubkey,
        project_key: Pubkey,
        dispute_status: DisputeStatus,
        dispute_proposal_id: u64,
        dispute_proposal_decision: Pubkey,
        dispute_execution_queue_item: Pubkey,
        dispute_payload_hash: [u8; 32],
        dispute_action_type: ActionType,
        proposal_decision_key: Pubkey,
        proposal_decision_proposal_id: u64,
        proposal_decision: ProposalDecision,
        execution_queue_item_key: Pubkey,
        queue_proposal_id: u64,
        queue_status: ExecutionStatus,
        queue_action_type: ActionType,
        queue_payload_hash: [u8; 32],
        queue_target_program: Pubkey,
        expected_program_id: Pubkey,
        queue_target_account: Pubkey,
        expected_target_account: Pubkey,
        now: i64,
        queue_execute_after: i64,
        provided_bond_vault: Pubkey,
        green_bond_vault_mint: Pubkey,
        green_bond_vault_owner: Pubkey,
        provided_bond_vault_authority: Pubkey,
        provided_relief_or_risk_vault: Pubkey,
        relief_or_risk_vault_mint: Pubkey,
        expected_relief_or_risk_vault: Pubkey,
        expected_usdc_mint: Pubkey,
        provided_usdc_mint: Pubkey,
        usdc_decimals: u8,
        vault_balance: u64,
        slash_amount: u64,
    }

    impl SlashExecutionValidationFixture {
        fn valid() -> Self {
            let bond_vault = Pubkey::new_from_array([13; 32]);
            let bond_vault_authority = Pubkey::new_from_array([14; 32]);
            let proposal_decision_key = Pubkey::new_from_array([15; 32]);
            let execution_queue_item_key = Pubkey::new_from_array([16; 32]);
            let project_key = Pubkey::new_from_array([17; 32]);
            let dispute_key = Pubkey::new_from_array([18; 32]);
            let program_id = Pubkey::new_from_array([19; 32]);
            let usdc_mint = Pubkey::new_from_array([2; 32]);
            let relief_or_risk_vault = Pubkey::new_from_array([5; 32]);
            let payload_hash = [23; 32];

            Self {
                config_is_paused: false,
                project_status: GreenLabelStatus::SlashQueued,
                project_active_dispute: dispute_key,
                dispute_key,
                project_bond_vault: bond_vault,
                project_bond_vault_authority: bond_vault_authority,
                project_terminal_proposal_id: 7,
                project_terminal_proposal_decision: proposal_decision_key,
                project_terminal_execution_queue_item: execution_queue_item_key,
                project_terminal_payload_hash: payload_hash,
                project_terminal_action_type: ActionType::GreenLabelSlash,
                dispute_project: project_key,
                project_key,
                dispute_status: DisputeStatus::DecisionQueued,
                dispute_proposal_id: 7,
                dispute_proposal_decision: proposal_decision_key,
                dispute_execution_queue_item: execution_queue_item_key,
                dispute_payload_hash: payload_hash,
                dispute_action_type: ActionType::GreenLabelSlash,
                proposal_decision_key,
                proposal_decision_proposal_id: 7,
                proposal_decision: ProposalDecision::Approved,
                execution_queue_item_key,
                queue_proposal_id: 7,
                queue_status: ExecutionStatus::Queued,
                queue_action_type: ActionType::GreenLabelSlash,
                queue_payload_hash: payload_hash,
                queue_target_program: program_id,
                expected_program_id: program_id,
                queue_target_account: dispute_key,
                expected_target_account: dispute_key,
                now: 2_000,
                queue_execute_after: 1_000,
                provided_bond_vault: bond_vault,
                green_bond_vault_mint: usdc_mint,
                green_bond_vault_owner: bond_vault_authority,
                provided_bond_vault_authority: bond_vault_authority,
                provided_relief_or_risk_vault: relief_or_risk_vault,
                relief_or_risk_vault_mint: usdc_mint,
                expected_relief_or_risk_vault: relief_or_risk_vault,
                expected_usdc_mint: usdc_mint,
                provided_usdc_mint: usdc_mint,
                usdc_decimals: GREEN_LABEL_USDC_DECIMALS,
                vault_balance: 1_299_000_000,
                slash_amount: 1_299_000_000,
            }
        }
    }

    fn validate_slash_execution_fixture(fixture: SlashExecutionValidationFixture) -> Result<()> {
        validate_green_label_slash_execution(
            fixture.config_is_paused,
            fixture.project_status,
            fixture.project_active_dispute,
            fixture.dispute_key,
            fixture.project_bond_vault,
            fixture.project_bond_vault_authority,
            fixture.project_terminal_proposal_id,
            fixture.project_terminal_proposal_decision,
            fixture.project_terminal_execution_queue_item,
            fixture.project_terminal_payload_hash,
            fixture.project_terminal_action_type,
            fixture.dispute_project,
            fixture.project_key,
            fixture.dispute_status,
            fixture.dispute_proposal_id,
            fixture.dispute_proposal_decision,
            fixture.dispute_execution_queue_item,
            fixture.dispute_payload_hash,
            fixture.dispute_action_type,
            fixture.proposal_decision_key,
            fixture.proposal_decision_proposal_id,
            fixture.proposal_decision,
            fixture.execution_queue_item_key,
            fixture.queue_proposal_id,
            fixture.queue_status,
            fixture.queue_action_type,
            fixture.queue_payload_hash,
            fixture.queue_target_program,
            fixture.expected_program_id,
            fixture.queue_target_account,
            fixture.expected_target_account,
            fixture.now,
            fixture.queue_execute_after,
            fixture.provided_bond_vault,
            fixture.green_bond_vault_mint,
            fixture.green_bond_vault_owner,
            fixture.provided_bond_vault_authority,
            fixture.provided_relief_or_risk_vault,
            fixture.relief_or_risk_vault_mint,
            fixture.expected_relief_or_risk_vault,
            fixture.expected_usdc_mint,
            fixture.provided_usdc_mint,
            fixture.usdc_decimals,
            fixture.vault_balance,
            fixture.slash_amount,
        )
    }

    fn try_validate_green_bond_vault_initialization(
        config_is_paused: bool,
        signer: Pubkey,
        project_status: GreenLabelStatus,
        existing_bond_vault: Pubkey,
        existing_bond_vault_authority: Pubkey,
        provided_usdc_mint: Pubkey,
    ) -> Result<()> {
        validate_green_bond_vault_initialization(
            config_is_paused,
            Pubkey::new_from_array([8; 32]),
            signer,
            project_status,
            existing_bond_vault,
            existing_bond_vault_authority,
            Pubkey::new_from_array([2; 32]),
            provided_usdc_mint,
        )
    }

    fn pending_bond_project_for_vault_init() -> GreenLabelProjectV1 {
        let mut project = green_label_project();
        project.status = GreenLabelStatus::PendingBondDeposit;
        project.bond_vault = Pubkey::default();
        project.bond_vault_authority = Pubkey::default();
        project.observation_start_ts = 0;
        project.observation_end_ts = 0;
        project
    }

    fn pending_bond_project_for_lock() -> GreenLabelProjectV1 {
        let mut project = green_label_project();
        project.status = GreenLabelStatus::PendingBondDeposit;
        project.observation_start_ts = 0;
        project.observation_end_ts = 0;
        project
    }

    fn project_for_open_dispute_record() -> GreenLabelProjectV1 {
        let mut project = green_label_project();
        project.status = GreenLabelStatus::PendingObservation;
        project.active_dispute = Pubkey::default();
        project.dispute_count = 0;
        project
    }

    fn dispute_for_ready_record() -> GreenLabelDisputeV1 {
        let mut dispute = green_label_dispute();
        dispute.status = DisputeStatus::EvidencePeriod;
        dispute.proposal_id = 0;
        dispute.proposal_decision = Pubkey::default();
        dispute.execution_queue_item = Pubkey::default();
        dispute.payload_hash = [0; 32];
        dispute.action_type = ActionType::Noop;
        dispute.resolved_at = 0;
        dispute
    }

    fn security_link_record_accounts() -> (GreenLabelProjectV1, GreenLabelDisputeV1) {
        let dispute_key = Pubkey::new_from_array([18; 32]);
        let mut project = green_label_project();
        project.status = GreenLabelStatus::Disputed;
        project.active_dispute = dispute_key;
        project.observation_start_ts = 1_000;
        project.observation_end_ts = 2_000;
        project.terminal_proposal_id = 0;
        project.terminal_proposal_decision = Pubkey::default();
        project.terminal_execution_queue_item = Pubkey::default();
        project.terminal_payload_hash = [0; 32];
        project.terminal_action_type = ActionType::Noop;

        let mut dispute = green_label_dispute();
        dispute.status = DisputeStatus::ReadyForDecision;
        dispute.proposal_id = 0;
        dispute.proposal_decision = Pubkey::default();
        dispute.execution_queue_item = Pubkey::default();
        dispute.payload_hash = [0; 32];
        dispute.action_type = ActionType::Noop;

        (project, dispute)
    }

    fn refund_record_accounts() -> (GreenLabelProjectV1, GreenLabelDisputeV1) {
        let dispute_key = Pubkey::new_from_array([18; 32]);
        let mut project = green_label_project();
        project.status = GreenLabelStatus::RefundQueued;
        project.active_dispute = dispute_key;
        project.refunded_at = 0;
        project.slashed_at = 0;
        project.terminal_action_type = ActionType::GreenLabelRefund;

        let mut dispute = green_label_dispute();
        dispute.status = DisputeStatus::DecisionQueued;
        dispute.resolved_at = 0;
        dispute.action_type = ActionType::GreenLabelRefund;

        (project, dispute)
    }

    fn slash_record_accounts() -> (GreenLabelProjectV1, GreenLabelDisputeV1) {
        let dispute_key = Pubkey::new_from_array([18; 32]);
        let mut project = green_label_project();
        project.status = GreenLabelStatus::SlashQueued;
        project.active_dispute = dispute_key;
        project.refunded_at = 111;
        project.slashed_at = 0;
        project.terminal_action_type = ActionType::GreenLabelSlash;

        let mut dispute = green_label_dispute();
        dispute.status = DisputeStatus::DecisionQueued;
        dispute.resolved_at = 0;
        dispute.action_type = ActionType::GreenLabelSlash;

        (project, dispute)
    }

    fn green_label_project() -> GreenLabelProjectV1 {
        GreenLabelProjectV1 {
            project_id: 1,
            project_owner: Pubkey::new_from_array([8; 32]),
            project_name_hash: [9; 32],
            project_url_hash: [10; 32],
            token_mint: Pubkey::new_from_array([11; 32]),
            project_treasury_wallet: Pubkey::new_from_array([12; 32]),
            base_bond_amount: 299_000_000,
            extra_bond_amount: 1_000_000_000,
            total_bond_amount: 1_299_000_000,
            bond_vault: Pubkey::new_from_array([13; 32]),
            bond_vault_authority: Pubkey::new_from_array([14; 32]),
            bond_tier: BondTier::Silver,
            status: GreenLabelStatus::RefundQueued,
            submitted_at: 1,
            observation_start_ts: 2,
            observation_end_ts: 3,
            dispute_count: 0,
            active_dispute: Pubkey::default(),
            approved_at: 0,
            refunded_at: 0,
            slashed_at: 0,
            risk_score_snapshot: 0,
            terminal_proposal_id: 1,
            terminal_proposal_decision: Pubkey::new_from_array([15; 32]),
            terminal_execution_queue_item: Pubkey::new_from_array([16; 32]),
            terminal_payload_hash: [17; 32],
            terminal_action_type: ActionType::GreenLabelRefund,
            bump: 249,
            reserved: [0; GREEN_LABEL_PROJECT_RESERVED_BYTES],
        }
    }

    fn green_label_dispute() -> GreenLabelDisputeV1 {
        GreenLabelDisputeV1 {
            project_id: 1,
            dispute_id: 1,
            project: Pubkey::new_from_array([18; 32]),
            disputer: Pubkey::new_from_array([19; 32]),
            reason_code: RugReasonCode::LiquidityRemoved,
            evidence_hash: [20; 32],
            status: DisputeStatus::DecisionQueued,
            opened_at: 1,
            evidence_end_ts: 2,
            response_end_ts: 3,
            resolved_at: 0,
            proposal_id: 1,
            proposal_decision: Pubkey::new_from_array([21; 32]),
            execution_queue_item: Pubkey::new_from_array([22; 32]),
            payload_hash: [23; 32],
            action_type: ActionType::GreenLabelSlash,
            bump: 248,
            reserved: [0; GREEN_LABEL_DISPUTE_RESERVED_BYTES],
        }
    }
}
