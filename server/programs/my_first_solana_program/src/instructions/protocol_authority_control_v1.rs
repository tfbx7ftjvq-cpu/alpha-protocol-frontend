use anchor_lang::prelude::*;

use crate::constants::{
    EXECUTION_QUEUE_ITEM_V1_SEED, GOVERNANCE_CONFIG_V1_SEED, GOVERNANCE_PROPOSAL_ACTION_V1_SEED,
    PROPOSAL_DECISION_V1_SEED, PROTOCOL_AUTHORITY_CONTROL_V1_SEED,
    PROTOCOL_DAO_CONTROL_ACTIVATION_RECORD_V1_SEED, PROTOCOL_MODULE_REGISTRY_V1_SEED,
    PROTOCOL_SECURITY_UNPAUSE_EXECUTION_RECORD_V1_SEED,
    UNIVERSAL_GOVERNANCE_DECISION_ADAPTER_V1_SEED,
};
use crate::error::CustomError;
use crate::instructions::contributor_v1::hash_contributor_payload;
use crate::instructions::governance_action_v1::{
    hash_governance_payload_v1, map_governance_action_to_module,
    map_governance_action_to_security_action,
};
use crate::instructions::governance_adapter_v1::security_proposal_type_for_action;
use crate::instructions::governance_v1::validate_governance_proposal_action_v1;
use crate::instructions::protocol_module_registry_v1::{
    protocol_module_stable_code_v1, validate_protocol_module_registry_v1,
};
use crate::state::{
    ActionType, ExecutionQueueItemV1, ExecutionStatus, GovernanceActionTypeV1, GovernanceConfigV1,
    GovernancePayloadV1, GovernanceProposalActionV1, GovernanceProposalStatusV1,
    GovernanceProposalV1, ProposalDecision, ProposalDecisionV1,
    ProtocolActivateDaoControlParametersV1, ProtocolAuthorityControlV1, ProtocolAuthorityModeV1,
    ProtocolDaoControlActivationRecordV1, ProtocolModuleIdV1, ProtocolModuleRegistryV1,
    ProtocolSecurityUnpauseExecutionRecordV1, ProtocolUnpauseSecurityParametersV1,
    UniversalGovernanceDecisionAdapterV1,
};

pub const PROTOCOL_AUTHORITY_SCHEMA_VERSION_V1: u16 = 1;
pub const PROTOCOL_ACTIVATE_DAO_CONTROL_DOMAIN: &[u8] = b"alpha_protocol_activate_dao_control_v1";
pub const PROTOCOL_UNPAUSE_SECURITY_DOMAIN: &[u8] = b"alpha_protocol_unpause_security_v1";

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct ProtocolActivateDaoControlHashEnvelopeV1 {
    pub domain_separator: Vec<u8>,
    pub parameters: ProtocolActivateDaoControlParametersV1,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct ProtocolUnpauseSecurityHashEnvelopeV1 {
    pub domain_separator: Vec<u8>,
    pub parameters: ProtocolUnpauseSecurityParametersV1,
}

#[derive(Accounts)]
pub struct InitializeProtocolAuthorityControlV1<'info> {
    #[account(
        seeds = [GOVERNANCE_CONFIG_V1_SEED],
        bump = governance_config.bump
    )]
    pub governance_config: Box<Account<'info, GovernanceConfigV1>>,

    #[account(
        init,
        payer = authority,
        space = 8 + ProtocolAuthorityControlV1::INIT_SPACE,
        seeds = [
            PROTOCOL_AUTHORITY_CONTROL_V1_SEED,
            governance_config.key().as_ref()
        ],
        bump
    )]
    pub authority_control: Box<Account<'info, ProtocolAuthorityControlV1>>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteActivateProtocolDaoControlV1<'info> {
    #[account(mut)]
    pub executor: Signer<'info>,

    pub current_authority: Signer<'info>,

    #[account(
        seeds = [GOVERNANCE_CONFIG_V1_SEED],
        bump = governance_config.bump
    )]
    pub governance_config: Box<Account<'info, GovernanceConfigV1>>,

    #[account(
        mut,
        seeds = [
            PROTOCOL_AUTHORITY_CONTROL_V1_SEED,
            governance_config.key().as_ref()
        ],
        bump = authority_control.bump
    )]
    pub authority_control: Box<Account<'info, ProtocolAuthorityControlV1>>,

    #[account(
        seeds = [
            PROTOCOL_MODULE_REGISTRY_V1_SEED,
            &[protocol_module_stable_code_v1(ProtocolModuleIdV1::Protocol)]
        ],
        bump = protocol_module_registry.bump
    )]
    pub protocol_module_registry: Box<Account<'info, ProtocolModuleRegistryV1>>,

    #[account(
        seeds = [
            GOVERNANCE_PROPOSAL_ACTION_V1_SEED,
            governance_proposal.key().as_ref()
        ],
        bump = governance_proposal_action.bump
    )]
    pub governance_proposal_action: Box<Account<'info, GovernanceProposalActionV1>>,

    pub governance_proposal: Box<Account<'info, GovernanceProposalV1>>,

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
        space = 8 + ProtocolDaoControlActivationRecordV1::INIT_SPACE,
        seeds = [
            PROTOCOL_DAO_CONTROL_ACTIVATION_RECORD_V1_SEED,
            authority_control.key().as_ref()
        ],
        bump
    )]
    pub activation_record: Box<Account<'info, ProtocolDaoControlActivationRecordV1>>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteProtocolUnpauseSecurityV1<'info> {
    #[account(mut)]
    pub executor: Signer<'info>,

    #[account(
        mut,
        seeds = [GOVERNANCE_CONFIG_V1_SEED],
        bump = governance_config.bump
    )]
    pub governance_config: Box<Account<'info, GovernanceConfigV1>>,

    #[account(
        seeds = [
            PROTOCOL_AUTHORITY_CONTROL_V1_SEED,
            governance_config.key().as_ref()
        ],
        bump = authority_control.bump
    )]
    pub authority_control: Box<Account<'info, ProtocolAuthorityControlV1>>,

    #[account(
        seeds = [
            PROTOCOL_MODULE_REGISTRY_V1_SEED,
            &[protocol_module_stable_code_v1(ProtocolModuleIdV1::Protocol)]
        ],
        bump = protocol_module_registry.bump
    )]
    pub protocol_module_registry: Box<Account<'info, ProtocolModuleRegistryV1>>,

    #[account(
        seeds = [
            GOVERNANCE_PROPOSAL_ACTION_V1_SEED,
            governance_proposal.key().as_ref()
        ],
        bump = governance_proposal_action.bump
    )]
    pub governance_proposal_action: Box<Account<'info, GovernanceProposalActionV1>>,

    pub governance_proposal: Box<Account<'info, GovernanceProposalV1>>,

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
        mut,
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
        space = 8 + ProtocolSecurityUnpauseExecutionRecordV1::INIT_SPACE,
        seeds = [
            PROTOCOL_SECURITY_UNPAUSE_EXECUTION_RECORD_V1_SEED,
            execution_queue_item.key().as_ref()
        ],
        bump
    )]
    pub unpause_record: Box<Account<'info, ProtocolSecurityUnpauseExecutionRecordV1>>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_protocol_authority_control_v1_handler(
    ctx: Context<InitializeProtocolAuthorityControlV1>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let authority_control_key = ctx.accounts.authority_control.key();
    initialize_protocol_authority_control_state(
        &ctx.accounts.governance_config,
        &mut ctx.accounts.authority_control,
        authority_control_key,
        ctx.accounts.authority.key(),
        now,
        ctx.bumps.authority_control,
    )
}

pub fn execute_activate_protocol_dao_control_v1_handler(
    ctx: Context<ExecuteActivateProtocolDaoControlV1>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let authority_control_key = ctx.accounts.authority_control.key();
    let governance_config_key = ctx.accounts.governance_config.key();
    let governance_proposal_key = ctx.accounts.governance_proposal.key();
    let governance_proposal_action_key = ctx.accounts.governance_proposal_action.key();
    let proposal_decision_key = ctx.accounts.proposal_decision.key();
    let execution_queue_item_key = ctx.accounts.execution_queue_item.key();
    let protocol_module_registry_key = ctx.accounts.protocol_module_registry.key();

    execute_activate_protocol_dao_control_state(
        &ctx.accounts.governance_config,
        &mut ctx.accounts.authority_control,
        &ctx.accounts.protocol_module_registry,
        &ctx.accounts.governance_proposal,
        &ctx.accounts.governance_proposal_action,
        &ctx.accounts.governance_decision_adapter,
        &ctx.accounts.proposal_decision,
        &ctx.accounts.execution_queue_item,
        &mut ctx.accounts.activation_record,
        authority_control_key,
        governance_config_key,
        protocol_module_registry_key,
        governance_proposal_key,
        governance_proposal_action_key,
        proposal_decision_key,
        execution_queue_item_key,
        ctx.accounts.current_authority.key(),
        ctx.accounts.executor.key(),
        now,
        ctx.bumps.activation_record,
    )
}

pub fn execute_protocol_unpause_security_v1_handler(
    ctx: Context<ExecuteProtocolUnpauseSecurityV1>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let authority_control_key = ctx.accounts.authority_control.key();
    let governance_config_key = ctx.accounts.governance_config.key();
    let governance_proposal_key = ctx.accounts.governance_proposal.key();
    let governance_proposal_action_key = ctx.accounts.governance_proposal_action.key();
    let proposal_decision_key = ctx.accounts.proposal_decision.key();
    let execution_queue_item_key = ctx.accounts.execution_queue_item.key();
    let protocol_module_registry_key = ctx.accounts.protocol_module_registry.key();

    execute_protocol_unpause_security_state(
        &mut ctx.accounts.governance_config,
        &ctx.accounts.authority_control,
        &ctx.accounts.protocol_module_registry,
        &ctx.accounts.governance_proposal,
        &ctx.accounts.governance_proposal_action,
        &ctx.accounts.governance_decision_adapter,
        &ctx.accounts.proposal_decision,
        &mut ctx.accounts.execution_queue_item,
        &mut ctx.accounts.unpause_record,
        authority_control_key,
        governance_config_key,
        protocol_module_registry_key,
        governance_proposal_key,
        governance_proposal_action_key,
        proposal_decision_key,
        execution_queue_item_key,
        ctx.accounts.executor.key(),
        now,
        ctx.bumps.unpause_record,
    )
}

pub fn protocol_authority_mode_stable_code_v1(mode: ProtocolAuthorityModeV1) -> u8 {
    match mode {
        ProtocolAuthorityModeV1::Bootstrap => 1,
        ProtocolAuthorityModeV1::DaoControlled => 2,
    }
}

pub fn protocol_authority_mode_from_stable_code_v1(code: u8) -> Result<ProtocolAuthorityModeV1> {
    match code {
        1 => Ok(ProtocolAuthorityModeV1::Bootstrap),
        2 => Ok(ProtocolAuthorityModeV1::DaoControlled),
        _ => err!(CustomError::InvalidProtocolAuthorityMode),
    }
}

pub fn expected_protocol_authority_control_key_and_bump(governance_config: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PROTOCOL_AUTHORITY_CONTROL_V1_SEED,
            governance_config.as_ref(),
        ],
        &crate::ID,
    )
}

pub fn validate_protocol_authority_mode_transition_v1(
    current_mode: ProtocolAuthorityModeV1,
    next_mode: ProtocolAuthorityModeV1,
) -> Result<()> {
    require!(
        current_mode == ProtocolAuthorityModeV1::Bootstrap
            && next_mode == ProtocolAuthorityModeV1::DaoControlled,
        CustomError::InvalidProtocolAuthorityModeTransition
    );
    Ok(())
}

pub fn validate_protocol_authority_control_binding_v1(
    governance_config: &GovernanceConfigV1,
    authority_control: &ProtocolAuthorityControlV1,
    authority_control_key: Pubkey,
    governance_config_key: Pubkey,
) -> Result<()> {
    require!(
        authority_control.schema_version == PROTOCOL_AUTHORITY_SCHEMA_VERSION_V1,
        CustomError::InvalidProtocolAuthoritySchema
    );
    require_keys_eq!(
        authority_control.governance_config,
        governance_config_key,
        CustomError::ProtocolAuthorityControlMismatch
    );
    require_keys_eq!(
        authority_control.bootstrap_authority,
        governance_config.authority,
        CustomError::ProtocolAuthorityControlMismatch
    );
    require_keys_eq!(
        authority_control.emergency_guardian,
        governance_config.emergency_guardian,
        CustomError::ProtocolAuthorityControlMismatch
    );

    let (expected_key, expected_bump) =
        expected_protocol_authority_control_key_and_bump(governance_config_key);
    require_keys_eq!(
        authority_control_key,
        expected_key,
        CustomError::ProtocolAuthorityControlMismatch
    );
    require!(
        authority_control.bump == expected_bump,
        CustomError::ProtocolAuthorityControlMismatch
    );
    protocol_authority_mode_stable_code_v1(authority_control.mode);

    Ok(())
}

pub fn validate_legacy_security_authority_allowed_v1(
    governance_config: &GovernanceConfigV1,
    authority_control: &ProtocolAuthorityControlV1,
    authority: Pubkey,
) -> Result<()> {
    require_keys_eq!(
        authority,
        governance_config.authority,
        CustomError::UnauthorizedSecurityAuthority
    );
    require!(
        authority_control.schema_version == PROTOCOL_AUTHORITY_SCHEMA_VERSION_V1,
        CustomError::InvalidProtocolAuthoritySchema
    );
    require_keys_eq!(
        authority_control.bootstrap_authority,
        governance_config.authority,
        CustomError::ProtocolAuthorityControlMismatch
    );
    require_keys_eq!(
        authority_control.emergency_guardian,
        governance_config.emergency_guardian,
        CustomError::ProtocolAuthorityControlMismatch
    );
    require!(
        authority_control.mode == ProtocolAuthorityModeV1::Bootstrap,
        CustomError::ProtocolAuthorityControlNotBootstrap
    );

    Ok(())
}

pub fn initialize_protocol_authority_control_state(
    governance_config: &GovernanceConfigV1,
    authority_control: &mut ProtocolAuthorityControlV1,
    authority_control_key: Pubkey,
    authority: Pubkey,
    now_ts: i64,
    bump: u8,
) -> Result<()> {
    require_keys_eq!(
        authority,
        governance_config.authority,
        CustomError::UnauthorizedSecurityAuthority
    );
    require!(
        authority_control.governance_config == Pubkey::default(),
        CustomError::ProtocolAuthorityControlMismatch
    );

    let governance_config_key =
        Pubkey::find_program_address(&[GOVERNANCE_CONFIG_V1_SEED], &crate::ID).0;
    let (expected_key, expected_bump) =
        expected_protocol_authority_control_key_and_bump(governance_config_key);
    require_keys_eq!(
        authority_control_key,
        expected_key,
        CustomError::ProtocolAuthorityControlMismatch
    );
    require!(
        bump == expected_bump,
        CustomError::ProtocolAuthorityControlMismatch
    );

    authority_control.governance_config = governance_config_key;
    authority_control.mode = ProtocolAuthorityModeV1::Bootstrap;
    authority_control.bootstrap_authority = governance_config.authority;
    authority_control.emergency_guardian = governance_config.emergency_guardian;
    authority_control.activation_governance_proposal = Pubkey::default();
    authority_control.activation_proposal_decision = Pubkey::default();
    authority_control.activation_execution_queue_item = Pubkey::default();
    authority_control.activated_at = 0;
    authority_control.schema_version = PROTOCOL_AUTHORITY_SCHEMA_VERSION_V1;
    authority_control.bump = bump;
    authority_control.reserved = [0u8; 64];

    let _ = now_ts;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn build_protocol_activate_dao_control_parameters_v1(
    authority_control: Pubkey,
    governance_config: Pubkey,
    current_authority: Pubkey,
    emergency_guardian: Pubkey,
    governance_proposal: Pubkey,
    governance_proposal_action: Pubkey,
    proposal_decision: Pubkey,
    execution_queue_item: Pubkey,
) -> Result<ProtocolActivateDaoControlParametersV1> {
    let parameters = ProtocolActivateDaoControlParametersV1 {
        schema_version: PROTOCOL_AUTHORITY_SCHEMA_VERSION_V1,
        authority_control,
        governance_config,
        expected_mode: ProtocolAuthorityModeV1::Bootstrap,
        next_mode: ProtocolAuthorityModeV1::DaoControlled,
        current_authority,
        emergency_guardian,
        governance_proposal,
        governance_proposal_action,
        proposal_decision,
        execution_queue_item,
        action_type: GovernanceActionTypeV1::ProtocolActivateDaoControl,
    };
    validate_protocol_activate_dao_control_parameters_v1(&parameters)?;
    Ok(parameters)
}

pub fn validate_protocol_activate_dao_control_parameters_v1(
    parameters: &ProtocolActivateDaoControlParametersV1,
) -> Result<()> {
    require!(
        parameters.schema_version == PROTOCOL_AUTHORITY_SCHEMA_VERSION_V1,
        CustomError::InvalidProtocolAuthoritySchema
    );
    validate_protocol_authority_mode_transition_v1(parameters.expected_mode, parameters.next_mode)?;
    require!(
        parameters.action_type == GovernanceActionTypeV1::ProtocolActivateDaoControl,
        CustomError::ProtocolAuthorityActionMismatch
    );
    require!(
        parameters.authority_control != Pubkey::default()
            && parameters.governance_config != Pubkey::default()
            && parameters.current_authority != Pubkey::default()
            && parameters.emergency_guardian != Pubkey::default()
            && parameters.governance_proposal != Pubkey::default()
            && parameters.governance_proposal_action != Pubkey::default()
            && parameters.proposal_decision != Pubkey::default()
            && parameters.execution_queue_item != Pubkey::default(),
        CustomError::ProtocolAuthorityParametersMismatch
    );
    Ok(())
}

pub fn hash_protocol_activate_dao_control_parameters_v1(
    parameters: &ProtocolActivateDaoControlParametersV1,
) -> Result<[u8; 32]> {
    validate_protocol_activate_dao_control_parameters_v1(parameters)?;
    hash_contributor_payload(&ProtocolActivateDaoControlHashEnvelopeV1 {
        domain_separator: PROTOCOL_ACTIVATE_DAO_CONTROL_DOMAIN.to_vec(),
        parameters: *parameters,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn build_protocol_unpause_security_parameters_v1(
    authority_control: Pubkey,
    governance_config: Pubkey,
    governance_proposal: Pubkey,
    governance_proposal_action: Pubkey,
    proposal_decision: Pubkey,
    execution_queue_item: Pubkey,
    proposal_id: u64,
) -> Result<ProtocolUnpauseSecurityParametersV1> {
    let parameters = ProtocolUnpauseSecurityParametersV1 {
        schema_version: PROTOCOL_AUTHORITY_SCHEMA_VERSION_V1,
        authority_control,
        governance_config,
        expected_mode: ProtocolAuthorityModeV1::DaoControlled,
        expected_paused: true,
        next_paused: false,
        action_type: GovernanceActionTypeV1::ProtocolUnpauseSecurity,
        governance_proposal,
        governance_proposal_action,
        proposal_decision,
        execution_queue_item,
        proposal_id,
    };
    validate_protocol_unpause_security_parameters_v1(&parameters)?;
    Ok(parameters)
}

pub fn validate_protocol_unpause_security_parameters_v1(
    parameters: &ProtocolUnpauseSecurityParametersV1,
) -> Result<()> {
    require!(
        parameters.schema_version == PROTOCOL_AUTHORITY_SCHEMA_VERSION_V1,
        CustomError::InvalidProtocolAuthoritySchema
    );
    require!(
        parameters.expected_mode == ProtocolAuthorityModeV1::DaoControlled
            && parameters.expected_paused
            && !parameters.next_paused,
        CustomError::ProtocolAuthorityParametersMismatch
    );
    require!(
        parameters.action_type == GovernanceActionTypeV1::ProtocolUnpauseSecurity,
        CustomError::ProtocolAuthorityActionMismatch
    );
    require!(
        parameters.authority_control != Pubkey::default()
            && parameters.governance_config != Pubkey::default()
            && parameters.governance_proposal != Pubkey::default()
            && parameters.governance_proposal_action != Pubkey::default()
            && parameters.proposal_decision != Pubkey::default()
            && parameters.execution_queue_item != Pubkey::default()
            && parameters.proposal_id > 0,
        CustomError::ProtocolAuthorityParametersMismatch
    );
    Ok(())
}

pub fn hash_protocol_unpause_security_parameters_v1(
    parameters: &ProtocolUnpauseSecurityParametersV1,
) -> Result<[u8; 32]> {
    validate_protocol_unpause_security_parameters_v1(parameters)?;
    hash_contributor_payload(&ProtocolUnpauseSecurityHashEnvelopeV1 {
        domain_separator: PROTOCOL_UNPAUSE_SECURITY_DOMAIN.to_vec(),
        parameters: *parameters,
    })
}

pub fn canonical_governance_payload_hash_for_action_v1(
    governance_proposal_action: &GovernanceProposalActionV1,
) -> Result<[u8; 32]> {
    hash_governance_payload_v1(&GovernancePayloadV1 {
        schema_version: 1,
        action_type: governance_proposal_action.action_type,
        module_id: governance_proposal_action.module_id,
        target_program: governance_proposal_action.target_program,
        target_account: governance_proposal_action.target_account,
        parameters_hash: governance_proposal_action.parameters_hash,
        evidence_hash: governance_proposal_action.evidence_hash,
        created_at: governance_proposal_action.created_at,
    })
}

#[allow(clippy::too_many_arguments)]
fn validate_common_protocol_governance_chain_v1(
    governance_config: &GovernanceConfigV1,
    authority_control: &ProtocolAuthorityControlV1,
    protocol_module_registry: &ProtocolModuleRegistryV1,
    governance_proposal: &GovernanceProposalV1,
    governance_proposal_action: &GovernanceProposalActionV1,
    governance_decision_adapter: &UniversalGovernanceDecisionAdapterV1,
    proposal_decision: &ProposalDecisionV1,
    execution_queue_item: &ExecutionQueueItemV1,
    authority_control_key: Pubkey,
    governance_config_key: Pubkey,
    protocol_module_registry_key: Pubkey,
    governance_proposal_key: Pubkey,
    governance_proposal_action_key: Pubkey,
    proposal_decision_key: Pubkey,
    execution_queue_item_key: Pubkey,
    expected_action: GovernanceActionTypeV1,
    expected_security_action: ActionType,
    expected_queue_status: ExecutionStatus,
) -> Result<[u8; 32]> {
    validate_protocol_authority_control_binding_v1(
        governance_config,
        authority_control,
        authority_control_key,
        governance_config_key,
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
        governance_proposal_action.action_type == expected_action,
        CustomError::ProtocolAuthorityActionMismatch
    );
    require!(
        governance_proposal_action.module_id == ProtocolModuleIdV1::Protocol,
        CustomError::GovernanceActionModuleMismatch
    );
    require!(
        map_governance_action_to_module(governance_proposal_action.action_type)
            == ProtocolModuleIdV1::Protocol,
        CustomError::GovernanceActionModuleMismatch
    );
    require_keys_eq!(
        governance_proposal_action.target_program,
        crate::ID,
        CustomError::ProtocolAuthorityTargetMismatch
    );

    let expected_action_from_mapping =
        map_governance_action_to_security_action(governance_proposal_action.action_type)?;
    require!(
        expected_action_from_mapping == expected_security_action,
        CustomError::ProtocolAuthorityActionMismatch
    );

    validate_protocol_module_registry_v1(
        protocol_module_registry,
        protocol_module_registry_key,
        governance_config_key,
        ProtocolModuleIdV1::Protocol,
        crate::ID,
    )?;

    let canonical_hash =
        canonical_governance_payload_hash_for_action_v1(governance_proposal_action)?;
    require!(
        canonical_hash == governance_proposal_action.canonical_payload_hash,
        CustomError::ProtocolAuthorityParametersMismatch
    );
    require!(
        governance_proposal.payload_hash == canonical_hash,
        CustomError::ProtocolAuthorityParametersMismatch
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
        CustomError::ProtocolAuthorityActionMismatch
    );
    require_keys_eq!(
        governance_decision_adapter.target_program,
        governance_proposal_action.target_program,
        CustomError::ProtocolAuthorityTargetMismatch
    );
    require_keys_eq!(
        governance_decision_adapter.target_account,
        governance_proposal_action.target_account,
        CustomError::ProtocolAuthorityTargetMismatch
    );
    require!(
        governance_decision_adapter.payload_hash == canonical_hash,
        CustomError::ProtocolAuthorityParametersMismatch
    );
    require!(
        governance_decision_adapter.executed,
        CustomError::InvalidGovernanceDecisionAdapter
    );

    let expected_proposal_type = security_proposal_type_for_action(expected_security_action)?;
    require!(
        proposal_decision.proposal_id == governance_proposal.proposal_id
            && proposal_decision.proposer == governance_proposal.proposer
            && proposal_decision.proposal_type == expected_proposal_type
            && proposal_decision.decision == ProposalDecision::Approved,
        CustomError::InvalidProposalDecision
    );
    require!(
        execution_queue_item.proposal_id == governance_proposal.proposal_id
            && execution_queue_item.proposer == governance_proposal.proposer
            && execution_queue_item.action_type == expected_security_action
            && execution_queue_item.decision == ProposalDecision::Approved
            && execution_queue_item.status == expected_queue_status,
        CustomError::InvalidExecutionStatus
    );
    require_keys_eq!(
        execution_queue_item.target_program,
        governance_proposal_action.target_program,
        CustomError::ProtocolAuthorityTargetMismatch
    );
    require_keys_eq!(
        execution_queue_item.target_account,
        governance_proposal_action.target_account,
        CustomError::ProtocolAuthorityTargetMismatch
    );
    require!(
        execution_queue_item.payload_hash == canonical_hash,
        CustomError::ProtocolAuthorityParametersMismatch
    );

    let _ = governance_proposal_action_key;
    let _ = execution_queue_item_key;

    Ok(canonical_hash)
}

#[allow(clippy::too_many_arguments)]
pub fn execute_activate_protocol_dao_control_state(
    governance_config: &GovernanceConfigV1,
    authority_control: &mut ProtocolAuthorityControlV1,
    protocol_module_registry: &ProtocolModuleRegistryV1,
    governance_proposal: &GovernanceProposalV1,
    governance_proposal_action: &GovernanceProposalActionV1,
    governance_decision_adapter: &UniversalGovernanceDecisionAdapterV1,
    proposal_decision: &ProposalDecisionV1,
    execution_queue_item: &ExecutionQueueItemV1,
    activation_record: &mut ProtocolDaoControlActivationRecordV1,
    authority_control_key: Pubkey,
    governance_config_key: Pubkey,
    protocol_module_registry_key: Pubkey,
    governance_proposal_key: Pubkey,
    governance_proposal_action_key: Pubkey,
    proposal_decision_key: Pubkey,
    execution_queue_item_key: Pubkey,
    current_authority: Pubkey,
    executor: Pubkey,
    now_ts: i64,
    record_bump: u8,
) -> Result<()> {
    require!(
        activation_record.authority_control == Pubkey::default(),
        CustomError::ProtocolAuthorityReceiptAlreadyExists
    );
    require!(
        authority_control.mode == ProtocolAuthorityModeV1::Bootstrap,
        CustomError::ProtocolAuthorityControlNotBootstrap
    );
    require_keys_eq!(
        current_authority,
        governance_config.authority,
        CustomError::UnauthorizedSecurityAuthority
    );
    require_keys_eq!(
        current_authority,
        authority_control.bootstrap_authority,
        CustomError::UnauthorizedSecurityAuthority
    );
    require_keys_eq!(
        governance_proposal_action.target_account,
        authority_control_key,
        CustomError::ProtocolAuthorityTargetMismatch
    );

    let parameters = build_protocol_activate_dao_control_parameters_v1(
        authority_control_key,
        governance_config_key,
        current_authority,
        governance_config.emergency_guardian,
        governance_proposal_key,
        governance_proposal_action_key,
        proposal_decision_key,
        execution_queue_item_key,
    )?;
    let parameters_hash = hash_protocol_activate_dao_control_parameters_v1(&parameters)?;
    require!(
        governance_proposal_action.parameters_hash == parameters_hash,
        CustomError::ProtocolAuthorityParametersMismatch
    );

    let canonical_hash = validate_common_protocol_governance_chain_v1(
        governance_config,
        authority_control,
        protocol_module_registry,
        governance_proposal,
        governance_proposal_action,
        governance_decision_adapter,
        proposal_decision,
        execution_queue_item,
        authority_control_key,
        governance_config_key,
        protocol_module_registry_key,
        governance_proposal_key,
        governance_proposal_action_key,
        proposal_decision_key,
        execution_queue_item_key,
        GovernanceActionTypeV1::ProtocolActivateDaoControl,
        ActionType::ProtocolActivateDaoControl,
        ExecutionStatus::Executed,
    )?;

    activation_record.authority_control = authority_control_key;
    activation_record.governance_config = governance_config_key;
    activation_record.governance_proposal = governance_proposal_key;
    activation_record.proposal_decision = proposal_decision_key;
    activation_record.execution_queue_item = execution_queue_item_key;
    activation_record.governance_proposal_action = governance_proposal_action_key;
    activation_record.current_authority = current_authority;
    activation_record.emergency_guardian = governance_config.emergency_guardian;
    activation_record.mode_before = ProtocolAuthorityModeV1::Bootstrap;
    activation_record.mode_after = ProtocolAuthorityModeV1::DaoControlled;
    activation_record.parameters_hash = parameters_hash;
    activation_record.canonical_governance_payload_hash = canonical_hash;
    activation_record.executor = executor;
    activation_record.executed_at = now_ts;
    activation_record.schema_version = PROTOCOL_AUTHORITY_SCHEMA_VERSION_V1;
    activation_record.bump = record_bump;
    activation_record.reserved = [0u8; 64];

    authority_control.mode = ProtocolAuthorityModeV1::DaoControlled;
    authority_control.activation_governance_proposal = governance_proposal_key;
    authority_control.activation_proposal_decision = proposal_decision_key;
    authority_control.activation_execution_queue_item = execution_queue_item_key;
    authority_control.activated_at = now_ts;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn execute_protocol_unpause_security_state(
    governance_config: &mut GovernanceConfigV1,
    authority_control: &ProtocolAuthorityControlV1,
    protocol_module_registry: &ProtocolModuleRegistryV1,
    governance_proposal: &GovernanceProposalV1,
    governance_proposal_action: &GovernanceProposalActionV1,
    governance_decision_adapter: &UniversalGovernanceDecisionAdapterV1,
    proposal_decision: &ProposalDecisionV1,
    execution_queue_item: &mut ExecutionQueueItemV1,
    unpause_record: &mut ProtocolSecurityUnpauseExecutionRecordV1,
    authority_control_key: Pubkey,
    governance_config_key: Pubkey,
    protocol_module_registry_key: Pubkey,
    governance_proposal_key: Pubkey,
    governance_proposal_action_key: Pubkey,
    proposal_decision_key: Pubkey,
    execution_queue_item_key: Pubkey,
    executor: Pubkey,
    now_ts: i64,
    record_bump: u8,
) -> Result<()> {
    require!(
        unpause_record.authority_control == Pubkey::default(),
        CustomError::ProtocolAuthorityReceiptAlreadyExists
    );
    require!(
        authority_control.mode == ProtocolAuthorityModeV1::DaoControlled,
        CustomError::ProtocolAuthorityControlNotDaoControlled
    );
    require!(
        governance_config.is_paused,
        CustomError::ProtocolSecurityUnpauseNotEligible
    );
    require!(
        now_ts >= execution_queue_item.execute_after,
        CustomError::ExecutionDelayNotMet
    );
    require_keys_eq!(
        governance_proposal_action.target_account,
        governance_config_key,
        CustomError::ProtocolAuthorityTargetMismatch
    );

    let parameters = build_protocol_unpause_security_parameters_v1(
        authority_control_key,
        governance_config_key,
        governance_proposal_key,
        governance_proposal_action_key,
        proposal_decision_key,
        execution_queue_item_key,
        governance_proposal.proposal_id,
    )?;
    let parameters_hash = hash_protocol_unpause_security_parameters_v1(&parameters)?;
    require!(
        governance_proposal_action.parameters_hash == parameters_hash,
        CustomError::ProtocolAuthorityParametersMismatch
    );

    let canonical_hash = validate_common_protocol_governance_chain_v1(
        governance_config,
        authority_control,
        protocol_module_registry,
        governance_proposal,
        governance_proposal_action,
        governance_decision_adapter,
        proposal_decision,
        execution_queue_item,
        authority_control_key,
        governance_config_key,
        protocol_module_registry_key,
        governance_proposal_key,
        governance_proposal_action_key,
        proposal_decision_key,
        execution_queue_item_key,
        GovernanceActionTypeV1::ProtocolUnpauseSecurity,
        ActionType::ProtocolUnpauseSecurity,
        ExecutionStatus::Queued,
    )?;

    unpause_record.authority_control = authority_control_key;
    unpause_record.governance_config = governance_config_key;
    unpause_record.governance_proposal = governance_proposal_key;
    unpause_record.proposal_decision = proposal_decision_key;
    unpause_record.execution_queue_item = execution_queue_item_key;
    unpause_record.governance_proposal_action = governance_proposal_action_key;
    unpause_record.paused_before = true;
    unpause_record.paused_after = false;
    unpause_record.parameters_hash = parameters_hash;
    unpause_record.canonical_governance_payload_hash = canonical_hash;
    unpause_record.executor = executor;
    unpause_record.executed_at = now_ts;
    unpause_record.schema_version = PROTOCOL_AUTHORITY_SCHEMA_VERSION_V1;
    unpause_record.bump = record_bump;
    unpause_record.reserved = [0u8; 64];

    execution_queue_item.status = ExecutionStatus::Executed;
    execution_queue_item.executed_at = now_ts;
    governance_config.is_paused = false;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const AUTHORITY: Pubkey = Pubkey::new_from_array([1; 32]);
    const GUARDIAN: Pubkey = Pubkey::new_from_array([2; 32]);
    const OTHER: Pubkey = Pubkey::new_from_array([3; 32]);
    const SECURITY_CONFIG: Pubkey = Pubkey::new_from_array([4; 32]);
    const AUTHORITY_CONTROL: Pubkey = Pubkey::new_from_array([5; 32]);
    const PROPOSAL: Pubkey = Pubkey::new_from_array([6; 32]);
    const ACTION: Pubkey = Pubkey::new_from_array([7; 32]);
    const DECISION: Pubkey = Pubkey::new_from_array([8; 32]);
    const QUEUE: Pubkey = Pubkey::new_from_array([9; 32]);

    fn governance_config(paused: bool) -> GovernanceConfigV1 {
        GovernanceConfigV1 {
            authority: AUTHORITY,
            min_execution_delay_seconds: 60,
            proposal_count: 1,
            emergency_guardian: GUARDIAN,
            is_paused: paused,
            bump: 250,
        }
    }

    fn authority_control(mode: ProtocolAuthorityModeV1) -> ProtocolAuthorityControlV1 {
        ProtocolAuthorityControlV1 {
            governance_config: SECURITY_CONFIG,
            mode,
            bootstrap_authority: AUTHORITY,
            emergency_guardian: GUARDIAN,
            activation_governance_proposal: Pubkey::default(),
            activation_proposal_decision: Pubkey::default(),
            activation_execution_queue_item: Pubkey::default(),
            activated_at: 0,
            schema_version: PROTOCOL_AUTHORITY_SCHEMA_VERSION_V1,
            bump: 1,
            reserved: [0u8; 64],
        }
    }

    #[test]
    fn protocol_authority_mode_codes_are_fixed() {
        assert_eq!(
            protocol_authority_mode_stable_code_v1(ProtocolAuthorityModeV1::Bootstrap),
            1
        );
        assert_eq!(
            protocol_authority_mode_stable_code_v1(ProtocolAuthorityModeV1::DaoControlled),
            2
        );
        assert_eq!(
            protocol_authority_mode_from_stable_code_v1(1).unwrap(),
            ProtocolAuthorityModeV1::Bootstrap
        );
        assert_eq!(
            protocol_authority_mode_from_stable_code_v1(2).unwrap(),
            ProtocolAuthorityModeV1::DaoControlled
        );
        assert_eq!(
            protocol_authority_mode_from_stable_code_v1(9).unwrap_err(),
            CustomError::InvalidProtocolAuthorityMode.into()
        );
    }

    #[test]
    fn protocol_authority_mode_transition_is_one_way() {
        validate_protocol_authority_mode_transition_v1(
            ProtocolAuthorityModeV1::Bootstrap,
            ProtocolAuthorityModeV1::DaoControlled,
        )
        .unwrap();
        assert_eq!(
            validate_protocol_authority_mode_transition_v1(
                ProtocolAuthorityModeV1::DaoControlled,
                ProtocolAuthorityModeV1::Bootstrap
            )
            .unwrap_err(),
            CustomError::InvalidProtocolAuthorityModeTransition.into()
        );
    }

    #[test]
    fn protocol_authority_account_sizes_are_exact() {
        assert_eq!(ProtocolAuthorityControlV1::INIT_SPACE, 268);
        assert_eq!(ProtocolDaoControlActivationRecordV1::INIT_SPACE, 429);
        assert_eq!(ProtocolSecurityUnpauseExecutionRecordV1::INIT_SPACE, 365);
    }

    #[test]
    fn legacy_security_authority_allowed_only_in_bootstrap() {
        let config = governance_config(false);
        let bootstrap_control = authority_control(ProtocolAuthorityModeV1::Bootstrap);
        validate_legacy_security_authority_allowed_v1(&config, &bootstrap_control, AUTHORITY)
            .unwrap();

        let dao_control = authority_control(ProtocolAuthorityModeV1::DaoControlled);
        assert_eq!(
            validate_legacy_security_authority_allowed_v1(&config, &dao_control, AUTHORITY)
                .unwrap_err(),
            CustomError::ProtocolAuthorityControlNotBootstrap.into()
        );
        assert_eq!(
            validate_legacy_security_authority_allowed_v1(&config, &bootstrap_control, OTHER)
                .unwrap_err(),
            CustomError::UnauthorizedSecurityAuthority.into()
        );
    }

    #[test]
    fn activation_parameters_hash_is_deterministic_and_bound() {
        let params = build_protocol_activate_dao_control_parameters_v1(
            AUTHORITY_CONTROL,
            SECURITY_CONFIG,
            AUTHORITY,
            GUARDIAN,
            PROPOSAL,
            ACTION,
            DECISION,
            QUEUE,
        )
        .unwrap();
        let hash = hash_protocol_activate_dao_control_parameters_v1(&params).unwrap();
        assert_eq!(
            hash,
            hash_protocol_activate_dao_control_parameters_v1(&params).unwrap()
        );

        let mut changed = params;
        changed.current_authority = OTHER;
        assert_ne!(
            hash,
            hash_protocol_activate_dao_control_parameters_v1(&changed).unwrap()
        );
    }

    #[test]
    fn unpause_parameters_hash_is_deterministic_and_bound() {
        let params = build_protocol_unpause_security_parameters_v1(
            AUTHORITY_CONTROL,
            SECURITY_CONFIG,
            PROPOSAL,
            ACTION,
            DECISION,
            QUEUE,
            7,
        )
        .unwrap();
        let hash = hash_protocol_unpause_security_parameters_v1(&params).unwrap();
        assert_eq!(
            hash,
            hash_protocol_unpause_security_parameters_v1(&params).unwrap()
        );

        let mut changed = params;
        changed.proposal_id = 8;
        assert_ne!(
            hash,
            hash_protocol_unpause_security_parameters_v1(&changed).unwrap()
        );
    }
}
