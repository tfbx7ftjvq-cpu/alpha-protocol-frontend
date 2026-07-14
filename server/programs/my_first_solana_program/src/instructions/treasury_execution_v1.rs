use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};

use crate::constants::{
    BUILDERS_USDC_VAULT_SEED, BUILDER_PAYOUT_REQUEST_V1_SEED, EXECUTION_QUEUE_ITEM_V1_SEED,
    GOVERNANCE_CONFIG_V1_SEED, GOVERNANCE_PROPOSAL_ACTION_V1_SEED, GOVERNANCE_PROPOSAL_V1_SEED,
    PROPOSAL_DECISION_V1_SEED, PROTOCOL_MODULE_REGISTRY_V1_SEED,
    TREASURY_BUILDER_PAYOUT_GOVERNANCE_V1_SEED, TREASURY_CONFIG_V2_SEED,
    TREASURY_EXECUTION_RECORD_V1_SEED, TREASURY_GOVERNANCE_CONFIG_V1_SEED,
    TREASURY_SPENDING_REQUEST_V1_SEED, UNIVERSAL_GOVERNANCE_DECISION_ADAPTER_V1_SEED,
    VAULT_AUTHORITY_V2_SEED,
};
use crate::error::CustomError;
use crate::instructions::contributor_v1::{
    hash_contributor_payload, validate_milestone_status_transition,
    validate_payout_status_transition,
};
use crate::instructions::governance_action_v1::map_governance_action_to_security_action;
use crate::instructions::governance_v1::validate_governance_proposal_action_v1;
use crate::instructions::protocol_module_registry_v1::{
    protocol_module_stable_code_v1, validate_protocol_module_registry_v1,
};
use crate::instructions::treasury_governance_v1::{
    record_treasury_builder_payout_status, record_treasury_spending_status,
    validate_treasury_governance_request,
};
use crate::state::{
    ActionType, BuilderPayoutRequestV1, ContributorMilestoneV1, ExecutionQueueItemV1,
    ExecutionStatus, GovernanceActionTypeV1, GovernanceConfigV1, GovernanceProposalActionV1,
    GovernanceProposalStatusV1, GovernanceProposalV1, MilestoneStatusV1, PayoutStatusV1,
    ProposalDecision, ProposalDecisionV1, ProposalType, ProtocolModuleIdV1,
    ProtocolModuleRegistryV1, TreasuryBuilderPayoutGovernanceV1, TreasuryBuilderPayoutStatusV1,
    TreasuryConfigV2, TreasuryExecutionRecordV1, TreasuryExecutionTypeV1,
    TreasuryGovernanceConfigV1, TreasurySpendingRequestV1, TreasurySpendingStatusV1,
    UniversalGovernanceDecisionAdapterV1,
};

pub const TREASURY_EXECUTION_SCHEMA_VERSION: u16 = 1;
pub const TREASURY_BUILDER_PAYOUT_PARAMETERS_V1_DOMAIN: &[u8] =
    b"alpha_treasury_builder_payout_parameters_v1";
pub const TREASURY_SPENDING_PARAMETERS_V1_DOMAIN: &[u8] = b"alpha_treasury_spending_parameters_v1";

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreasuryBuilderPayoutParametersV1 {
    pub schema_version: u16,
    pub treasury_config: Pubkey,
    pub treasury_builder_payout_governance: Pubkey,
    pub builder_payout_request: Pubkey,
    pub milestone: Pubkey,
    pub recipient_owner: Pubkey,
    pub recipient_token_account: Pubkey,
    pub amount_usdc: u64,
    pub source_vault: Pubkey,
    pub usdc_mint: Pubkey,
    pub proposal_id: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreasurySpendingParametersV1 {
    pub schema_version: u16,
    pub treasury_config: Pubkey,
    pub treasury_spending_request: Pubkey,
    pub recipient_owner: Pubkey,
    pub recipient_token_account: Pubkey,
    pub amount_usdc: u64,
    pub source_vault: Pubkey,
    pub usdc_mint: Pubkey,
    pub purpose_hash: [u8; 32],
    pub proposal_id: u64,
}

#[derive(Accounts)]
pub struct ExecuteTreasuryBuilderPayoutV1<'info> {
    #[account(mut)]
    pub executor: Signer<'info>,

    #[account(seeds = [GOVERNANCE_CONFIG_V1_SEED], bump = security_governance_config.bump)]
    pub security_governance_config: Box<Account<'info, GovernanceConfigV1>>,

    #[account(seeds = [TREASURY_CONFIG_V2_SEED], bump = treasury_config.bump)]
    pub treasury_config: Box<Account<'info, TreasuryConfigV2>>,

    #[account(
        seeds = [TREASURY_GOVERNANCE_CONFIG_V1_SEED],
        bump = treasury_governance_config.bump,
        constraint = treasury_governance_config.treasury_config == treasury_config.key() @ CustomError::InvalidTreasuryGovernanceConfig
    )]
    pub treasury_governance_config: Box<Account<'info, TreasuryGovernanceConfigV1>>,

    #[account(
        seeds = [
            PROTOCOL_MODULE_REGISTRY_V1_SEED,
            &[protocol_module_stable_code_v1(ProtocolModuleIdV1::Treasury)]
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
        seeds = [GOVERNANCE_PROPOSAL_ACTION_V1_SEED, governance_proposal.key().as_ref()],
        bump = governance_proposal_action.bump
    )]
    pub governance_proposal_action: Box<Account<'info, GovernanceProposalActionV1>>,

    #[account(
        seeds = [UNIVERSAL_GOVERNANCE_DECISION_ADAPTER_V1_SEED, governance_proposal.key().as_ref()],
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

    #[account(
        mut,
        seeds = [TREASURY_BUILDER_PAYOUT_GOVERNANCE_V1_SEED, builder_payout_request.key().as_ref()],
        bump = treasury_builder_payout_governance.bump
    )]
    pub treasury_builder_payout_governance: Box<Account<'info, TreasuryBuilderPayoutGovernanceV1>>,

    #[account(
        mut,
        seeds = [
            BUILDER_PAYOUT_REQUEST_V1_SEED,
            treasury_builder_payout_governance.contributor_registry.as_ref(),
            treasury_builder_payout_governance.milestone.as_ref()
        ],
        bump = builder_payout_request.bump,
        constraint = builder_payout_request.contributor == treasury_builder_payout_governance.contributor_registry @ CustomError::InvalidContributorPayoutRequest,
        constraint = builder_payout_request.milestone == treasury_builder_payout_governance.milestone @ CustomError::InvalidContributorPayoutRequest
    )]
    pub builder_payout_request: Box<Account<'info, BuilderPayoutRequestV1>>,

    #[account(
        mut,
        constraint = contributor_milestone.key() == treasury_builder_payout_governance.milestone @ CustomError::InvalidContributorMilestone
    )]
    pub contributor_milestone: Box<Account<'info, ContributorMilestoneV1>>,

    #[account(
        init,
        payer = executor,
        space = 8 + TreasuryExecutionRecordV1::INIT_SPACE,
        seeds = [TREASURY_EXECUTION_RECORD_V1_SEED, execution_queue_item.key().as_ref()],
        bump
    )]
    pub treasury_execution_record: Box<Account<'info, TreasuryExecutionRecordV1>>,

    /// CHECK: This PDA only signs for Treasury V2 vault transfers.
    #[account(seeds = [VAULT_AUTHORITY_V2_SEED], bump)]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [BUILDERS_USDC_VAULT_SEED],
        bump,
        constraint = builders_usdc_vault.mint == treasury_config.usdc_mint @ CustomError::TreasuryExecutionMintMismatch,
        constraint = builders_usdc_vault.owner == vault_authority.key() @ CustomError::TreasuryExecutionVaultMismatch
    )]
    pub builders_usdc_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub recipient_usdc_token_account: Box<Account<'info, TokenAccount>>,

    #[account(constraint = usdc_mint.key() == treasury_config.usdc_mint @ CustomError::TreasuryExecutionMintMismatch)]
    pub usdc_mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteTreasurySpendingV1<'info> {
    #[account(mut)]
    pub executor: Signer<'info>,

    #[account(seeds = [GOVERNANCE_CONFIG_V1_SEED], bump = security_governance_config.bump)]
    pub security_governance_config: Box<Account<'info, GovernanceConfigV1>>,

    #[account(seeds = [TREASURY_CONFIG_V2_SEED], bump = treasury_config.bump)]
    pub treasury_config: Box<Account<'info, TreasuryConfigV2>>,

    #[account(
        seeds = [TREASURY_GOVERNANCE_CONFIG_V1_SEED],
        bump = treasury_governance_config.bump,
        constraint = treasury_governance_config.treasury_config == treasury_config.key() @ CustomError::InvalidTreasuryGovernanceConfig
    )]
    pub treasury_governance_config: Box<Account<'info, TreasuryGovernanceConfigV1>>,

    #[account(
        seeds = [
            PROTOCOL_MODULE_REGISTRY_V1_SEED,
            &[protocol_module_stable_code_v1(ProtocolModuleIdV1::Treasury)]
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
        seeds = [GOVERNANCE_PROPOSAL_ACTION_V1_SEED, governance_proposal.key().as_ref()],
        bump = governance_proposal_action.bump
    )]
    pub governance_proposal_action: Box<Account<'info, GovernanceProposalActionV1>>,

    #[account(
        seeds = [UNIVERSAL_GOVERNANCE_DECISION_ADAPTER_V1_SEED, governance_proposal.key().as_ref()],
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

    #[account(
        mut,
        seeds = [
            TREASURY_SPENDING_REQUEST_V1_SEED,
            treasury_config.key().as_ref(),
            &treasury_spending_request.request_id.to_le_bytes()
        ],
        bump = treasury_spending_request.bump
    )]
    pub treasury_spending_request: Box<Account<'info, TreasurySpendingRequestV1>>,

    #[account(
        init,
        payer = executor,
        space = 8 + TreasuryExecutionRecordV1::INIT_SPACE,
        seeds = [TREASURY_EXECUTION_RECORD_V1_SEED, execution_queue_item.key().as_ref()],
        bump
    )]
    pub treasury_execution_record: Box<Account<'info, TreasuryExecutionRecordV1>>,

    /// CHECK: This PDA only signs for Treasury V2 vault transfers.
    #[account(seeds = [VAULT_AUTHORITY_V2_SEED], bump)]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [BUILDERS_USDC_VAULT_SEED],
        bump,
        constraint = builders_usdc_vault.mint == treasury_config.usdc_mint @ CustomError::TreasuryExecutionMintMismatch,
        constraint = builders_usdc_vault.owner == vault_authority.key() @ CustomError::TreasuryExecutionVaultMismatch
    )]
    pub builders_usdc_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub recipient_usdc_token_account: Box<Account<'info, TokenAccount>>,

    #[account(constraint = usdc_mint.key() == treasury_config.usdc_mint @ CustomError::TreasuryExecutionMintMismatch)]
    pub usdc_mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn execute_treasury_builder_payout_v1_handler(
    ctx: Context<ExecuteTreasuryBuilderPayoutV1>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let parameters = treasury_builder_payout_parameters_from_accounts(
        ctx.accounts.treasury_config.key(),
        ctx.accounts.treasury_builder_payout_governance.key(),
        &ctx.accounts.treasury_builder_payout_governance,
        ctx.accounts.builder_payout_request.key(),
        ctx.accounts.contributor_milestone.key(),
        ctx.accounts.recipient_usdc_token_account.key(),
        ctx.accounts.builders_usdc_vault.key(),
        ctx.accounts.usdc_mint.key(),
    );
    let parameters_hash = hash_treasury_builder_payout_parameters_v1(&parameters)?;

    validate_treasury_execution_context_v1(
        &ctx.accounts.treasury_governance_config,
        &ctx.accounts.security_governance_config,
        ctx.accounts.security_governance_config.key(),
        &ctx.accounts.treasury_config,
        ctx.accounts.treasury_config.key(),
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
        ctx.accounts.treasury_builder_payout_governance.key(),
        GovernanceActionTypeV1::TreasuryApproveBuilderPayout,
        ProposalType::TreasuryApproveBuilderPayout,
        ActionType::TreasuryApproveBuilderPayout,
        parameters_hash,
    )?;

    validate_treasury_builder_payout_business_v1(
        &ctx.accounts.treasury_builder_payout_governance,
        ctx.accounts.builder_payout_request.key(),
        &ctx.accounts.builder_payout_request,
        ctx.accounts.contributor_milestone.key(),
        &ctx.accounts.contributor_milestone,
        ctx.accounts.governance_proposal.proposal_id,
        ctx.accounts.treasury_config.key(),
        ctx.accounts.recipient_usdc_token_account.key(),
        &ctx.accounts.recipient_usdc_token_account,
        ctx.accounts.builders_usdc_vault.key(),
        &ctx.accounts.builders_usdc_vault,
        ctx.accounts.vault_authority.key(),
        ctx.accounts.usdc_mint.key(),
    )?;

    transfer_from_builders_vault(
        ctx.accounts.token_program.key(),
        ctx.accounts.builders_usdc_vault.to_account_info(),
        ctx.accounts.usdc_mint.to_account_info(),
        ctx.accounts.recipient_usdc_token_account.to_account_info(),
        ctx.accounts.vault_authority.to_account_info(),
        ctx.bumps.vault_authority,
        ctx.accounts.treasury_builder_payout_governance.amount,
        ctx.accounts.usdc_mint.decimals,
    )?;

    record_treasury_execution(
        &mut ctx.accounts.treasury_execution_record,
        ctx.accounts.execution_queue_item.key(),
        ctx.accounts.proposal_decision.key(),
        ctx.accounts.governance_proposal.key(),
        ctx.accounts.governance_proposal_action.key(),
        ctx.accounts.treasury_builder_payout_governance.key(),
        TreasuryExecutionTypeV1::BuilderPayout,
        ctx.accounts.builders_usdc_vault.key(),
        ctx.accounts.treasury_builder_payout_governance.recipient,
        ctx.accounts.recipient_usdc_token_account.key(),
        ctx.accounts.treasury_builder_payout_governance.amount,
        ctx.accounts.usdc_mint.key(),
        parameters_hash,
        ctx.accounts
            .governance_proposal_action
            .canonical_payload_hash,
        ctx.accounts.executor.key(),
        now,
        ctx.bumps.treasury_execution_record,
    )?;

    record_treasury_builder_payout_status(
        &mut ctx.accounts.treasury_builder_payout_governance,
        TreasuryBuilderPayoutStatusV1::Executed,
    )?;
    validate_payout_status_transition(
        ctx.accounts.builder_payout_request.status,
        PayoutStatusV1::Executed,
    )?;
    ctx.accounts.builder_payout_request.status = PayoutStatusV1::Executed;
    validate_milestone_status_transition(
        ctx.accounts.contributor_milestone.status,
        MilestoneStatusV1::Paid,
    )?;
    ctx.accounts.contributor_milestone.status = MilestoneStatusV1::Paid;

    Ok(())
}

pub fn execute_treasury_spending_v1_handler(ctx: Context<ExecuteTreasurySpendingV1>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let parameters = treasury_spending_parameters_from_accounts(
        ctx.accounts.treasury_config.key(),
        ctx.accounts.treasury_spending_request.key(),
        &ctx.accounts.treasury_spending_request,
        ctx.accounts.recipient_usdc_token_account.key(),
        ctx.accounts.builders_usdc_vault.key(),
        ctx.accounts.usdc_mint.key(),
    );
    let parameters_hash = hash_treasury_spending_parameters_v1(&parameters)?;

    validate_treasury_execution_context_v1(
        &ctx.accounts.treasury_governance_config,
        &ctx.accounts.security_governance_config,
        ctx.accounts.security_governance_config.key(),
        &ctx.accounts.treasury_config,
        ctx.accounts.treasury_config.key(),
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
        ctx.accounts.treasury_spending_request.key(),
        GovernanceActionTypeV1::TreasuryApproveSpending,
        ProposalType::TreasuryApproveSpending,
        ActionType::TreasuryApproveSpending,
        parameters_hash,
    )?;

    validate_treasury_spending_business_v1(
        &ctx.accounts.treasury_governance_config,
        &ctx.accounts.treasury_spending_request,
        ctx.accounts.governance_proposal.proposal_id,
        ctx.accounts.treasury_config.key(),
        ctx.accounts.recipient_usdc_token_account.key(),
        &ctx.accounts.recipient_usdc_token_account,
        ctx.accounts.builders_usdc_vault.key(),
        &ctx.accounts.builders_usdc_vault,
        ctx.accounts.vault_authority.key(),
        ctx.accounts.usdc_mint.key(),
    )?;

    transfer_from_builders_vault(
        ctx.accounts.token_program.key(),
        ctx.accounts.builders_usdc_vault.to_account_info(),
        ctx.accounts.usdc_mint.to_account_info(),
        ctx.accounts.recipient_usdc_token_account.to_account_info(),
        ctx.accounts.vault_authority.to_account_info(),
        ctx.bumps.vault_authority,
        ctx.accounts.treasury_spending_request.amount_usdc,
        ctx.accounts.usdc_mint.decimals,
    )?;

    record_treasury_execution(
        &mut ctx.accounts.treasury_execution_record,
        ctx.accounts.execution_queue_item.key(),
        ctx.accounts.proposal_decision.key(),
        ctx.accounts.governance_proposal.key(),
        ctx.accounts.governance_proposal_action.key(),
        ctx.accounts.treasury_spending_request.key(),
        TreasuryExecutionTypeV1::TreasurySpending,
        ctx.accounts.builders_usdc_vault.key(),
        ctx.accounts.treasury_spending_request.recipient,
        ctx.accounts.recipient_usdc_token_account.key(),
        ctx.accounts.treasury_spending_request.amount_usdc,
        ctx.accounts.usdc_mint.key(),
        parameters_hash,
        ctx.accounts
            .governance_proposal_action
            .canonical_payload_hash,
        ctx.accounts.executor.key(),
        now,
        ctx.bumps.treasury_execution_record,
    )?;

    record_treasury_spending_status(
        &mut ctx.accounts.treasury_spending_request,
        TreasurySpendingStatusV1::Executed,
        now,
    )
}

pub fn treasury_execution_type_stable_code_v1(execution_type: TreasuryExecutionTypeV1) -> u8 {
    match execution_type {
        TreasuryExecutionTypeV1::BuilderPayout => 1,
        TreasuryExecutionTypeV1::TreasurySpending => 2,
    }
}

pub fn treasury_execution_type_from_stable_code_v1(code: u8) -> Result<TreasuryExecutionTypeV1> {
    match code {
        1 => Ok(TreasuryExecutionTypeV1::BuilderPayout),
        2 => Ok(TreasuryExecutionTypeV1::TreasurySpending),
        _ => err!(CustomError::InvalidTreasuryExecutionSchema),
    }
}

pub fn treasury_builder_payout_parameters_from_accounts(
    treasury_config: Pubkey,
    payout_governance_key: Pubkey,
    payout_governance: &TreasuryBuilderPayoutGovernanceV1,
    payout_request_key: Pubkey,
    milestone_key: Pubkey,
    recipient_token_account: Pubkey,
    source_vault: Pubkey,
    usdc_mint: Pubkey,
) -> TreasuryBuilderPayoutParametersV1 {
    TreasuryBuilderPayoutParametersV1 {
        schema_version: TREASURY_EXECUTION_SCHEMA_VERSION,
        treasury_config,
        treasury_builder_payout_governance: payout_governance_key,
        builder_payout_request: payout_request_key,
        milestone: milestone_key,
        recipient_owner: payout_governance.recipient,
        recipient_token_account,
        amount_usdc: payout_governance.amount,
        source_vault,
        usdc_mint,
        proposal_id: payout_governance.proposal_id,
    }
}

pub fn treasury_spending_parameters_from_accounts(
    treasury_config: Pubkey,
    spending_request_key: Pubkey,
    spending_request: &TreasurySpendingRequestV1,
    recipient_token_account: Pubkey,
    source_vault: Pubkey,
    usdc_mint: Pubkey,
) -> TreasurySpendingParametersV1 {
    TreasurySpendingParametersV1 {
        schema_version: TREASURY_EXECUTION_SCHEMA_VERSION,
        treasury_config,
        treasury_spending_request: spending_request_key,
        recipient_owner: spending_request.recipient,
        recipient_token_account,
        amount_usdc: spending_request.amount_usdc,
        source_vault,
        usdc_mint,
        purpose_hash: spending_request.purpose_hash,
        proposal_id: spending_request.proposal_id,
    }
}

pub fn hash_treasury_builder_payout_parameters_v1(
    parameters: &TreasuryBuilderPayoutParametersV1,
) -> Result<[u8; 32]> {
    require!(
        parameters.schema_version == TREASURY_EXECUTION_SCHEMA_VERSION,
        CustomError::InvalidTreasuryExecutionSchema
    );
    hash_treasury_execution_payload(TREASURY_BUILDER_PAYOUT_PARAMETERS_V1_DOMAIN, parameters)
}

pub fn hash_treasury_spending_parameters_v1(
    parameters: &TreasurySpendingParametersV1,
) -> Result<[u8; 32]> {
    require!(
        parameters.schema_version == TREASURY_EXECUTION_SCHEMA_VERSION,
        CustomError::InvalidTreasuryExecutionSchema
    );
    hash_treasury_execution_payload(TREASURY_SPENDING_PARAMETERS_V1_DOMAIN, parameters)
}

pub fn hash_treasury_execution_payload<T: AnchorSerialize>(
    domain: &[u8],
    parameters: &T,
) -> Result<[u8; 32]> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(domain);
    parameters
        .serialize(&mut bytes)
        .map_err(|_| error!(CustomError::TreasuryExecutionParametersMismatch))?;
    hash_contributor_payload(&bytes)
}

#[allow(clippy::too_many_arguments)]
pub fn validate_treasury_approval_binding_v1(
    governance_config: &GovernanceConfigV1,
    governance_config_key: Pubkey,
    treasury_governance_config: &TreasuryGovernanceConfigV1,
    treasury_config: &TreasuryConfigV2,
    treasury_config_key: Pubkey,
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
    request_account: Pubkey,
    expected_governance_action: GovernanceActionTypeV1,
    expected_proposal_type: ProposalType,
    expected_security_action: ActionType,
    expected_parameters_hash: [u8; 32],
) -> Result<()> {
    validate_treasury_execution_context_v1(
        treasury_governance_config,
        governance_config,
        governance_config_key,
        treasury_config,
        treasury_config_key,
        protocol_module_registry,
        protocol_module_registry_key,
        governance_proposal,
        governance_proposal_key,
        governance_proposal_action,
        governance_proposal_action_key,
        governance_decision_adapter,
        governance_decision_adapter_key,
        proposal_decision,
        proposal_decision_key,
        execution_queue_item,
        execution_queue_item_key,
        request_account,
        expected_governance_action,
        expected_proposal_type,
        expected_security_action,
        expected_parameters_hash,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn validate_treasury_execution_context_v1(
    treasury_governance_config: &TreasuryGovernanceConfigV1,
    governance_config: &GovernanceConfigV1,
    governance_config_key: Pubkey,
    treasury_config: &TreasuryConfigV2,
    treasury_config_key: Pubkey,
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
    request_account: Pubkey,
    expected_governance_action: GovernanceActionTypeV1,
    expected_proposal_type: ProposalType,
    expected_security_action: ActionType,
    expected_parameters_hash: [u8; 32],
) -> Result<()> {
    require!(
        treasury_governance_config.dao_enabled,
        CustomError::TreasuryExecutionDisabled
    );
    require!(
        !treasury_governance_config.emergency_mode,
        CustomError::TreasuryEmergencyModeActive
    );
    require_keys_eq!(
        treasury_governance_config.treasury_config,
        treasury_config_key,
        CustomError::InvalidTreasuryGovernanceConfig
    );
    validate_protocol_module_registry_v1(
        protocol_module_registry,
        protocol_module_registry_key,
        governance_config_key,
        ProtocolModuleIdV1::Treasury,
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
        governance_proposal_action.action_type == expected_governance_action,
        CustomError::TreasuryExecutionActionMismatch
    );
    require!(
        governance_proposal_action.module_id == ProtocolModuleIdV1::Treasury,
        CustomError::TreasuryExecutionActionMismatch
    );
    require_keys_eq!(
        governance_proposal_action.target_account,
        request_account,
        CustomError::TreasuryExecutionTargetMismatch
    );
    require_keys_eq!(
        governance_proposal_action.target_program,
        crate::ID,
        CustomError::TreasuryExecutionTargetMismatch
    );
    require!(
        governance_proposal_action.parameters_hash == expected_parameters_hash,
        CustomError::TreasuryExecutionParametersMismatch
    );
    require!(
        map_governance_action_to_security_action(governance_proposal_action.action_type)?
            == expected_security_action,
        CustomError::TreasuryExecutionActionMismatch
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
        CustomError::TreasuryExecutionActionMismatch
    );
    require_keys_eq!(
        governance_decision_adapter.target_program,
        governance_proposal_action.target_program,
        CustomError::TreasuryExecutionTargetMismatch
    );
    require_keys_eq!(
        governance_decision_adapter.target_account,
        request_account,
        CustomError::TreasuryExecutionTargetMismatch
    );
    require!(
        governance_decision_adapter.payload_hash
            == governance_proposal_action.canonical_payload_hash,
        CustomError::TreasuryExecutionParametersMismatch
    );
    validate_treasury_governance_request(
        governance_config,
        proposal_decision,
        execution_queue_item,
        governance_proposal.proposal_id,
        expected_proposal_type,
        expected_security_action,
        request_account,
        governance_proposal_action.canonical_payload_hash,
    )?;
    require!(
        proposal_decision.decision == ProposalDecision::Approved,
        CustomError::ProposalNotApproved
    );
    require!(
        execution_queue_item.decision == ProposalDecision::Approved,
        CustomError::ProposalNotApproved
    );
    require!(
        execution_queue_item.status == ExecutionStatus::Executed,
        CustomError::InvalidExecutionStatus
    );
    require!(
        execution_queue_item_key != Pubkey::default(),
        CustomError::InvalidExecutionStatus
    );
    require!(
        governance_decision_adapter_key != Pubkey::default(),
        CustomError::InvalidGovernanceDecisionAdapter
    );
    require!(
        governance_proposal_action_key != Pubkey::default(),
        CustomError::GovernanceProposalActionMissing
    );
    require!(
        treasury_config_key != Pubkey::default() && treasury_config.usdc_mint != Pubkey::default(),
        CustomError::InvalidTreasuryGovernanceConfig
    );

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn validate_treasury_builder_payout_business_v1(
    payout_governance: &TreasuryBuilderPayoutGovernanceV1,
    payout_request_key: Pubkey,
    payout_request: &BuilderPayoutRequestV1,
    milestone_key: Pubkey,
    milestone: &ContributorMilestoneV1,
    expected_proposal_id: u64,
    treasury_config: Pubkey,
    recipient_token_account_key: Pubkey,
    recipient_token_account: &TokenAccount,
    source_vault_key: Pubkey,
    source_vault: &TokenAccount,
    vault_authority: Pubkey,
    usdc_mint: Pubkey,
) -> Result<()> {
    require_keys_eq!(
        payout_governance.payout_request,
        payout_request_key,
        CustomError::InvalidContributorPayoutRequest
    );
    require!(
        payout_governance.status == TreasuryBuilderPayoutStatusV1::Approved,
        CustomError::InvalidTreasuryBuilderPayoutStatus
    );
    require!(
        payout_governance.proposal_id == expected_proposal_id,
        CustomError::InvalidProposalId
    );
    require!(
        payout_request.status == PayoutStatusV1::Approved,
        CustomError::InvalidContributorPayoutRequest
    );
    require!(
        payout_governance.milestone == payout_request.milestone
            && payout_governance.milestone == milestone_key,
        CustomError::InvalidContributorMilestone
    );
    require!(
        milestone.status == MilestoneStatusV1::Approved,
        CustomError::InvalidContributorMilestone
    );
    require!(
        payout_governance.recipient == payout_request.destination_wallet,
        CustomError::TreasuryExecutionRecipientMismatch
    );
    require!(
        payout_governance.amount == payout_request.amount && payout_governance.amount > 0,
        CustomError::TreasuryExecutionAmountMismatch
    );
    validate_treasury_token_accounts(
        payout_governance.recipient,
        recipient_token_account_key,
        recipient_token_account,
        source_vault_key,
        source_vault,
        vault_authority,
        usdc_mint,
        payout_governance.amount,
    )?;
    require!(
        treasury_config != Pubkey::default(),
        CustomError::InvalidTreasuryGovernanceConfig
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn validate_treasury_spending_business_v1(
    treasury_governance_config: &TreasuryGovernanceConfigV1,
    spending_request: &TreasurySpendingRequestV1,
    expected_proposal_id: u64,
    treasury_config: Pubkey,
    recipient_token_account_key: Pubkey,
    recipient_token_account: &TokenAccount,
    source_vault_key: Pubkey,
    source_vault: &TokenAccount,
    vault_authority: Pubkey,
    usdc_mint: Pubkey,
) -> Result<()> {
    require!(
        spending_request.status == TreasurySpendingStatusV1::Approved,
        CustomError::InvalidTreasurySpendingStatus
    );
    require_keys_eq!(
        spending_request.treasury_config,
        treasury_config,
        CustomError::InvalidTreasuryGovernanceRequest
    );
    require!(
        spending_request.proposal_id == expected_proposal_id,
        CustomError::InvalidProposalId
    );
    require!(
        spending_request.amount_usdc > 0,
        CustomError::TreasuryExecutionAmountMismatch
    );
    require!(
        spending_request.amount_usdc <= treasury_governance_config.spending_limit_usdc,
        CustomError::TreasurySpendingLimitExceeded
    );
    require!(
        spending_request.purpose_hash != [0; 32],
        CustomError::TreasuryExecutionParametersMismatch
    );
    validate_treasury_token_accounts(
        spending_request.recipient,
        recipient_token_account_key,
        recipient_token_account,
        source_vault_key,
        source_vault,
        vault_authority,
        usdc_mint,
        spending_request.amount_usdc,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn validate_treasury_builder_payout_approval_business_v1(
    payout_governance: &TreasuryBuilderPayoutGovernanceV1,
    payout_request_key: Pubkey,
    payout_request: &BuilderPayoutRequestV1,
    milestone_key: Pubkey,
    milestone: &ContributorMilestoneV1,
    expected_proposal_id: u64,
    recipient_token_account_key: Pubkey,
    recipient_token_account: &TokenAccount,
    source_vault_key: Pubkey,
    source_vault: &TokenAccount,
    vault_authority: Pubkey,
    usdc_mint: Pubkey,
) -> Result<()> {
    require_keys_eq!(
        payout_governance.payout_request,
        payout_request_key,
        CustomError::InvalidContributorPayoutRequest
    );
    require!(
        payout_governance.status == TreasuryBuilderPayoutStatusV1::Pending,
        CustomError::InvalidTreasuryBuilderPayoutStatus
    );
    require!(
        payout_governance.proposal_id == expected_proposal_id,
        CustomError::InvalidProposalId
    );
    require!(
        payout_request.status == PayoutStatusV1::Approved,
        CustomError::InvalidContributorPayoutRequest
    );
    require!(
        payout_governance.milestone == payout_request.milestone
            && payout_governance.milestone == milestone_key,
        CustomError::InvalidContributorMilestone
    );
    require!(
        milestone.status == MilestoneStatusV1::Approved,
        CustomError::InvalidContributorMilestone
    );
    require!(
        payout_governance.recipient == payout_request.destination_wallet,
        CustomError::TreasuryExecutionRecipientMismatch
    );
    require!(
        payout_governance.amount == payout_request.amount && payout_governance.amount > 0,
        CustomError::TreasuryExecutionAmountMismatch
    );
    validate_treasury_token_account_metadata(
        payout_governance.recipient,
        recipient_token_account_key,
        recipient_token_account,
        source_vault_key,
        source_vault,
        vault_authority,
        usdc_mint,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn validate_treasury_spending_approval_business_v1(
    treasury_governance_config: &TreasuryGovernanceConfigV1,
    spending_request: &TreasurySpendingRequestV1,
    expected_proposal_id: u64,
    treasury_config: Pubkey,
    recipient_token_account_key: Pubkey,
    recipient_token_account: &TokenAccount,
    source_vault_key: Pubkey,
    source_vault: &TokenAccount,
    vault_authority: Pubkey,
    usdc_mint: Pubkey,
) -> Result<()> {
    require!(
        spending_request.status == TreasurySpendingStatusV1::Pending,
        CustomError::InvalidTreasurySpendingStatus
    );
    require_keys_eq!(
        spending_request.treasury_config,
        treasury_config,
        CustomError::InvalidTreasuryGovernanceRequest
    );
    require!(
        spending_request.proposal_id == expected_proposal_id,
        CustomError::InvalidProposalId
    );
    require!(
        spending_request.amount_usdc > 0,
        CustomError::TreasuryExecutionAmountMismatch
    );
    require!(
        spending_request.amount_usdc <= treasury_governance_config.spending_limit_usdc,
        CustomError::TreasurySpendingLimitExceeded
    );
    require!(
        spending_request.purpose_hash != [0; 32],
        CustomError::TreasuryExecutionParametersMismatch
    );
    validate_treasury_token_account_metadata(
        spending_request.recipient,
        recipient_token_account_key,
        recipient_token_account,
        source_vault_key,
        source_vault,
        vault_authority,
        usdc_mint,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn validate_treasury_token_account_metadata(
    recipient_owner: Pubkey,
    recipient_token_account_key: Pubkey,
    recipient_token_account: &TokenAccount,
    source_vault_key: Pubkey,
    source_vault: &TokenAccount,
    vault_authority: Pubkey,
    usdc_mint: Pubkey,
) -> Result<()> {
    require!(
        source_vault.owner == vault_authority,
        CustomError::TreasuryExecutionVaultMismatch
    );
    require!(
        source_vault.mint == usdc_mint,
        CustomError::TreasuryExecutionMintMismatch
    );
    require!(
        recipient_token_account.owner == recipient_owner,
        CustomError::TreasuryExecutionRecipientMismatch
    );
    require!(
        recipient_token_account.mint == usdc_mint,
        CustomError::TreasuryExecutionMintMismatch
    );
    require!(
        recipient_token_account_key != source_vault_key,
        CustomError::TreasuryExecutionRecipientMismatch
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn validate_treasury_token_accounts(
    recipient_owner: Pubkey,
    recipient_token_account_key: Pubkey,
    recipient_token_account: &TokenAccount,
    source_vault_key: Pubkey,
    source_vault: &TokenAccount,
    vault_authority: Pubkey,
    usdc_mint: Pubkey,
    amount: u64,
) -> Result<()> {
    validate_treasury_token_account_metadata(
        recipient_owner,
        recipient_token_account_key,
        recipient_token_account,
        source_vault_key,
        source_vault,
        vault_authority,
        usdc_mint,
    )?;
    require!(amount > 0, CustomError::TreasuryExecutionAmountMismatch);
    require!(
        source_vault.amount >= amount,
        CustomError::TreasuryExecutionInsufficientFunds
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn record_treasury_execution(
    record: &mut TreasuryExecutionRecordV1,
    queue_item: Pubkey,
    proposal_decision: Pubkey,
    governance_proposal: Pubkey,
    governance_proposal_action: Pubkey,
    request_account: Pubkey,
    execution_type: TreasuryExecutionTypeV1,
    source_vault: Pubkey,
    recipient_owner: Pubkey,
    recipient_token_account: Pubkey,
    amount_usdc: u64,
    usdc_mint: Pubkey,
    parameters_hash: [u8; 32],
    canonical_governance_payload_hash: [u8; 32],
    executor: Pubkey,
    executed_at: i64,
    bump: u8,
) -> Result<()> {
    require!(
        record.queue_item == Pubkey::default(),
        CustomError::TreasuryExecutionAlreadyCompleted
    );
    require!(
        amount_usdc > 0,
        CustomError::TreasuryExecutionAmountMismatch
    );
    require!(executed_at > 0, CustomError::InvalidTreasuryExecutionSchema);

    record.queue_item = queue_item;
    record.proposal_decision = proposal_decision;
    record.governance_proposal = governance_proposal;
    record.governance_proposal_action = governance_proposal_action;
    record.request_account = request_account;
    record.module_id = ProtocolModuleIdV1::Treasury;
    record.execution_type = execution_type;
    record.source_vault = source_vault;
    record.recipient_owner = recipient_owner;
    record.recipient_token_account = recipient_token_account;
    record.amount_usdc = amount_usdc;
    record.usdc_mint = usdc_mint;
    record.parameters_hash = parameters_hash;
    record.canonical_governance_payload_hash = canonical_governance_payload_hash;
    record.executor = executor;
    record.executed_at = executed_at;
    record.schema_version = TREASURY_EXECUTION_SCHEMA_VERSION;
    record.bump = bump;

    Ok(())
}

fn transfer_from_builders_vault<'info>(
    token_program: Pubkey,
    source_vault: AccountInfo<'info>,
    usdc_mint: AccountInfo<'info>,
    recipient_token_account: AccountInfo<'info>,
    vault_authority: AccountInfo<'info>,
    vault_authority_bump: u8,
    amount: u64,
    decimals: u8,
) -> Result<()> {
    let signer_seeds: &[&[&[u8]]] = &[&[VAULT_AUTHORITY_V2_SEED, &[vault_authority_bump]]];
    let cpi_accounts = TransferChecked {
        from: source_vault,
        mint: usdc_mint,
        to: recipient_token_account,
        authority: vault_authority,
    };
    let cpi_ctx = CpiContext::new_with_signer(token_program, cpi_accounts, signer_seeds);
    transfer_checked(cpi_ctx, amount, decimals)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::solana_program::program_option::COption;
    use anchor_lang::solana_program::program_pack::Pack;
    use anchor_lang::AccountDeserialize;
    use anchor_spl::token::spl_token::state::{Account as SplTokenAccount, AccountState};

    const HASH_ONE: [u8; 32] = [1; 32];
    const HASH_TWO: [u8; 32] = [2; 32];

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

    fn spending_request() -> TreasurySpendingRequestV1 {
        TreasurySpendingRequestV1 {
            request_id: 11,
            treasury_config: Pubkey::new_unique(),
            proposer: Pubkey::new_unique(),
            recipient: Pubkey::new_unique(),
            amount_usdc: 1_000_000,
            purpose_hash: HASH_ONE,
            proposal_id: 77,
            status: TreasurySpendingStatusV1::Pending,
            created_at: 100,
            executed_at: 0,
            bump: 1,
        }
    }

    fn payout_governance() -> TreasuryBuilderPayoutGovernanceV1 {
        TreasuryBuilderPayoutGovernanceV1 {
            payout_request: Pubkey::new_unique(),
            contributor_registry: Pubkey::new_unique(),
            milestone: Pubkey::new_unique(),
            recipient: Pubkey::new_unique(),
            amount: 2_000_000,
            proposal_id: 88,
            status: TreasuryBuilderPayoutStatusV1::Pending,
            created_at: 100,
            bump: 1,
        }
    }

    fn payout_request(governance: &TreasuryBuilderPayoutGovernanceV1) -> BuilderPayoutRequestV1 {
        BuilderPayoutRequestV1 {
            contributor: governance.contributor_registry,
            milestone: governance.milestone,
            amount: governance.amount,
            destination_wallet: governance.recipient,
            status: PayoutStatusV1::Approved,
            created_at: 99,
            bump: 1,
        }
    }

    fn milestone(governance: &TreasuryBuilderPayoutGovernanceV1) -> ContributorMilestoneV1 {
        ContributorMilestoneV1 {
            contributor: governance.contributor_registry,
            title: "ship execution layer".to_string(),
            description: "DAO-approved builder payout milestone".to_string(),
            evidence_hash: HASH_TWO,
            requested_amount: governance.amount,
            status: MilestoneStatusV1::Approved,
            created_at: 90,
            bump: 1,
        }
    }

    #[test]
    fn treasury_execution_type_stable_code_roundtrips() {
        for execution_type in [
            TreasuryExecutionTypeV1::BuilderPayout,
            TreasuryExecutionTypeV1::TreasurySpending,
        ] {
            let code = treasury_execution_type_stable_code_v1(execution_type);
            assert_eq!(
                treasury_execution_type_from_stable_code_v1(code).unwrap(),
                execution_type
            );
        }
    }

    #[test]
    fn unknown_treasury_execution_type_code_is_rejected() {
        let err = treasury_execution_type_from_stable_code_v1(99).unwrap_err();
        assert_eq!(err, CustomError::InvalidTreasuryExecutionSchema.into());
    }

    #[test]
    fn builder_payout_parameters_hash_is_deterministic_and_field_bound() {
        let governance = payout_governance();
        let base = treasury_builder_payout_parameters_from_accounts(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            &governance,
            governance.payout_request,
            governance.milestone,
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        );
        let base_hash = hash_treasury_builder_payout_parameters_v1(&base).unwrap();
        assert_eq!(
            base_hash,
            hash_treasury_builder_payout_parameters_v1(&base).unwrap()
        );

        let mut changed = base;
        changed.amount_usdc = changed.amount_usdc.checked_add(1).unwrap();
        assert_ne!(
            base_hash,
            hash_treasury_builder_payout_parameters_v1(&changed).unwrap()
        );

        let mut changed = base;
        changed.recipient_owner = Pubkey::new_unique();
        assert_ne!(
            base_hash,
            hash_treasury_builder_payout_parameters_v1(&changed).unwrap()
        );

        let mut changed = base;
        changed.recipient_token_account = Pubkey::new_unique();
        assert_ne!(
            base_hash,
            hash_treasury_builder_payout_parameters_v1(&changed).unwrap()
        );

        let mut changed = base;
        changed.source_vault = Pubkey::new_unique();
        assert_ne!(
            base_hash,
            hash_treasury_builder_payout_parameters_v1(&changed).unwrap()
        );

        let mut changed = base;
        changed.usdc_mint = Pubkey::new_unique();
        assert_ne!(
            base_hash,
            hash_treasury_builder_payout_parameters_v1(&changed).unwrap()
        );

        let mut changed = base;
        changed.builder_payout_request = Pubkey::new_unique();
        assert_ne!(
            base_hash,
            hash_treasury_builder_payout_parameters_v1(&changed).unwrap()
        );

        let mut changed = base;
        changed.proposal_id = changed.proposal_id.checked_add(1).unwrap();
        assert_ne!(
            base_hash,
            hash_treasury_builder_payout_parameters_v1(&changed).unwrap()
        );
    }

    #[test]
    fn spending_parameters_hash_is_deterministic_and_field_bound() {
        let request = spending_request();
        let base = treasury_spending_parameters_from_accounts(
            request.treasury_config,
            Pubkey::new_unique(),
            &request,
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        );
        let base_hash = hash_treasury_spending_parameters_v1(&base).unwrap();
        assert_eq!(
            base_hash,
            hash_treasury_spending_parameters_v1(&base).unwrap()
        );

        let mut changed = base;
        changed.amount_usdc = changed.amount_usdc.checked_add(1).unwrap();
        assert_ne!(
            base_hash,
            hash_treasury_spending_parameters_v1(&changed).unwrap()
        );

        let mut changed = base;
        changed.recipient_owner = Pubkey::new_unique();
        assert_ne!(
            base_hash,
            hash_treasury_spending_parameters_v1(&changed).unwrap()
        );

        let mut changed = base;
        changed.recipient_token_account = Pubkey::new_unique();
        assert_ne!(
            base_hash,
            hash_treasury_spending_parameters_v1(&changed).unwrap()
        );

        let mut changed = base;
        changed.source_vault = Pubkey::new_unique();
        assert_ne!(
            base_hash,
            hash_treasury_spending_parameters_v1(&changed).unwrap()
        );

        let mut changed = base;
        changed.usdc_mint = Pubkey::new_unique();
        assert_ne!(
            base_hash,
            hash_treasury_spending_parameters_v1(&changed).unwrap()
        );

        let mut changed = base;
        changed.treasury_spending_request = Pubkey::new_unique();
        assert_ne!(
            base_hash,
            hash_treasury_spending_parameters_v1(&changed).unwrap()
        );

        let mut changed = base;
        changed.proposal_id = changed.proposal_id.checked_add(1).unwrap();
        assert_ne!(
            base_hash,
            hash_treasury_spending_parameters_v1(&changed).unwrap()
        );
    }

    #[test]
    fn treasury_parameter_domain_separator_changes_hash() {
        let request = spending_request();
        let spending = treasury_spending_parameters_from_accounts(
            request.treasury_config,
            Pubkey::new_unique(),
            &request,
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        );

        let spending_hash = hash_treasury_spending_parameters_v1(&spending).unwrap();
        let wrong_domain_hash =
            hash_treasury_execution_payload(b"wrong_treasury_domain", &spending).unwrap();

        assert_ne!(spending_hash, wrong_domain_hash);
    }

    #[test]
    fn record_treasury_execution_is_immutable_after_first_write() {
        let mut record = TreasuryExecutionRecordV1 {
            queue_item: Pubkey::default(),
            proposal_decision: Pubkey::default(),
            governance_proposal: Pubkey::default(),
            governance_proposal_action: Pubkey::default(),
            request_account: Pubkey::default(),
            module_id: ProtocolModuleIdV1::Treasury,
            execution_type: TreasuryExecutionTypeV1::TreasurySpending,
            source_vault: Pubkey::default(),
            recipient_owner: Pubkey::default(),
            recipient_token_account: Pubkey::default(),
            amount_usdc: 0,
            usdc_mint: Pubkey::default(),
            parameters_hash: [0; 32],
            canonical_governance_payload_hash: [0; 32],
            executor: Pubkey::default(),
            executed_at: 0,
            schema_version: 0,
            bump: 0,
        };

        record_treasury_execution(
            &mut record,
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            TreasuryExecutionTypeV1::TreasurySpending,
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            1_000_000,
            Pubkey::new_unique(),
            HASH_ONE,
            HASH_TWO,
            Pubkey::new_unique(),
            123,
            9,
        )
        .unwrap();

        assert_eq!(record.module_id, ProtocolModuleIdV1::Treasury);
        assert_eq!(record.schema_version, TREASURY_EXECUTION_SCHEMA_VERSION);
        assert_eq!(record.bump, 9);

        let err = record_treasury_execution(
            &mut record,
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            TreasuryExecutionTypeV1::BuilderPayout,
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            1_000_000,
            Pubkey::new_unique(),
            HASH_ONE,
            HASH_TWO,
            Pubkey::new_unique(),
            124,
            9,
        )
        .unwrap_err();
        assert_eq!(err, CustomError::TreasuryExecutionAlreadyCompleted.into());
    }

    #[test]
    fn token_account_validation_rejects_substitution_and_insufficient_funds() {
        let mint = Pubkey::new_unique();
        let other_mint = Pubkey::new_unique();
        let vault_authority = Pubkey::new_unique();
        let recipient = Pubkey::new_unique();
        let vault_key = Pubkey::new_unique();
        let recipient_key = Pubkey::new_unique();
        let source = token_account(mint, vault_authority, 1_000);
        let recipient_token = token_account(mint, recipient, 0);

        validate_treasury_token_accounts(
            recipient,
            recipient_key,
            &recipient_token,
            vault_key,
            &source,
            vault_authority,
            mint,
            1_000,
        )
        .unwrap();

        let wrong_recipient = token_account(mint, Pubkey::new_unique(), 0);
        assert_eq!(
            validate_treasury_token_accounts(
                recipient,
                recipient_key,
                &wrong_recipient,
                vault_key,
                &source,
                vault_authority,
                mint,
                1_000,
            )
            .unwrap_err(),
            CustomError::TreasuryExecutionRecipientMismatch.into()
        );

        let wrong_mint_recipient = token_account(other_mint, recipient, 0);
        assert_eq!(
            validate_treasury_token_accounts(
                recipient,
                recipient_key,
                &wrong_mint_recipient,
                vault_key,
                &source,
                vault_authority,
                mint,
                1_000,
            )
            .unwrap_err(),
            CustomError::TreasuryExecutionMintMismatch.into()
        );

        let wrong_owner_source = token_account(mint, Pubkey::new_unique(), 1_000);
        assert_eq!(
            validate_treasury_token_accounts(
                recipient,
                recipient_key,
                &recipient_token,
                vault_key,
                &wrong_owner_source,
                vault_authority,
                mint,
                1_000,
            )
            .unwrap_err(),
            CustomError::TreasuryExecutionVaultMismatch.into()
        );

        assert_eq!(
            validate_treasury_token_accounts(
                recipient,
                recipient_key,
                &recipient_token,
                vault_key,
                &source,
                vault_authority,
                mint,
                1_001,
            )
            .unwrap_err(),
            CustomError::TreasuryExecutionInsufficientFunds.into()
        );

        assert_eq!(
            validate_treasury_token_accounts(
                recipient,
                vault_key,
                &recipient_token,
                vault_key,
                &source,
                vault_authority,
                mint,
                1_000,
            )
            .unwrap_err(),
            CustomError::TreasuryExecutionRecipientMismatch.into()
        );
    }

    #[test]
    fn spending_approval_business_rejects_limit_and_zero_purpose() {
        let mint = Pubkey::new_unique();
        let vault_authority = Pubkey::new_unique();
        let vault_key = Pubkey::new_unique();
        let recipient_key = Pubkey::new_unique();
        let mut request = spending_request();
        let source = token_account(mint, vault_authority, 0);
        let recipient_token = token_account(mint, request.recipient, 0);
        let config = TreasuryGovernanceConfigV1 {
            treasury_config: request.treasury_config,
            security_authority: Pubkey::new_unique(),
            dao_enabled: true,
            spending_limit_usdc: request.amount_usdc,
            split_change_threshold_bps: 100,
            emergency_mode: false,
            created_at: 1,
            updated_at: 1,
            bump: 1,
        };

        validate_treasury_spending_approval_business_v1(
            &config,
            &request,
            request.proposal_id,
            request.treasury_config,
            recipient_key,
            &recipient_token,
            vault_key,
            &source,
            vault_authority,
            mint,
        )
        .unwrap();

        request.amount_usdc = request.amount_usdc.checked_add(1).unwrap();
        assert_eq!(
            validate_treasury_spending_approval_business_v1(
                &config,
                &request,
                request.proposal_id,
                request.treasury_config,
                recipient_key,
                &recipient_token,
                vault_key,
                &source,
                vault_authority,
                mint,
            )
            .unwrap_err(),
            CustomError::TreasurySpendingLimitExceeded.into()
        );

        request.amount_usdc = config.spending_limit_usdc;
        request.purpose_hash = [0; 32];
        assert_eq!(
            validate_treasury_spending_approval_business_v1(
                &config,
                &request,
                request.proposal_id,
                request.treasury_config,
                recipient_key,
                &recipient_token,
                vault_key,
                &source,
                vault_authority,
                mint,
            )
            .unwrap_err(),
            CustomError::TreasuryExecutionParametersMismatch.into()
        );
    }

    #[test]
    fn builder_payout_approval_business_rejects_wrong_statuses() {
        let mint = Pubkey::new_unique();
        let vault_authority = Pubkey::new_unique();
        let vault_key = Pubkey::new_unique();
        let recipient_key = Pubkey::new_unique();
        let governance = payout_governance();
        let request_key = governance.payout_request;
        let milestone_key = governance.milestone;
        let request = payout_request(&governance);
        let milestone = milestone(&governance);
        let source = token_account(mint, vault_authority, 0);
        let recipient_token = token_account(mint, governance.recipient, 0);

        validate_treasury_builder_payout_approval_business_v1(
            &governance,
            request_key,
            &request,
            milestone_key,
            &milestone,
            governance.proposal_id,
            recipient_key,
            &recipient_token,
            vault_key,
            &source,
            vault_authority,
            mint,
        )
        .unwrap();

        let mut executed_governance = governance.clone();
        executed_governance.status = TreasuryBuilderPayoutStatusV1::Executed;
        assert_eq!(
            validate_treasury_builder_payout_approval_business_v1(
                &executed_governance,
                request_key,
                &request,
                milestone_key,
                &milestone,
                governance.proposal_id,
                recipient_key,
                &recipient_token,
                vault_key,
                &source,
                vault_authority,
                mint,
            )
            .unwrap_err(),
            CustomError::InvalidTreasuryBuilderPayoutStatus.into()
        );

        let mut pending_request = request;
        pending_request.status = PayoutStatusV1::Pending;
        assert_eq!(
            validate_treasury_builder_payout_approval_business_v1(
                &governance,
                request_key,
                &pending_request,
                milestone_key,
                &milestone,
                governance.proposal_id,
                recipient_key,
                &recipient_token,
                vault_key,
                &source,
                vault_authority,
                mint,
            )
            .unwrap_err(),
            CustomError::InvalidContributorPayoutRequest.into()
        );
    }
}
