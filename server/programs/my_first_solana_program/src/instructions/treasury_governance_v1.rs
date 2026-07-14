use anchor_lang::prelude::*;

use crate::constants::{
    BUILDER_PAYOUT_REQUEST_V1_SEED, CONTRIBUTOR_REGISTRY_V1_SEED, EXECUTION_QUEUE_ITEM_V1_SEED,
    GOVERNANCE_CONFIG_V1_SEED, PROPOSAL_DECISION_V1_SEED,
    TREASURY_BUILDER_PAYOUT_GOVERNANCE_V1_SEED, TREASURY_CONFIG_V2_SEED,
    TREASURY_GOVERNANCE_CONFIG_V1_SEED, TREASURY_SPENDING_REQUEST_V1_SEED,
};
use crate::error::CustomError;
use crate::instructions::contributor_v1::hash_contributor_payload;
use crate::state::{
    ActionType, BuilderPayoutRequestV1, ContributorMilestoneV1, ContributorRegistryV1,
    ExecutionQueueItemV1, ExecutionStatus, GovernanceConfigV1, PayoutStatusV1, ProposalDecision,
    ProposalDecisionV1, ProposalType, TreasuryBuilderPayoutGovernanceV1,
    TreasuryBuilderPayoutStatusV1, TreasuryConfigV2, TreasuryGovernanceConfigV1,
    TreasurySpendingRequestV1, TreasurySpendingStatusV1,
};

pub const TREASURY_GOVERNANCE_PAYLOAD_V1_DOMAIN: &[u8] = b"alpha_treasury_governance_payload_v1";

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreasurySpendingRequestPayloadV1 {
    pub spending_request: Pubkey,
    pub recipient: Pubkey,
    pub amount_usdc: u64,
    pub purpose_hash: [u8; 32],
    pub proposal_id: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreasuryBuilderPayoutGovernancePayloadV1 {
    pub payout_governance: Pubkey,
    pub payout_request: Pubkey,
    pub contributor_registry: Pubkey,
    pub milestone: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
    pub proposal_id: u64,
}

#[derive(Accounts)]
pub struct InitializeTreasuryGovernanceConfigV1<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [TREASURY_CONFIG_V2_SEED],
        bump = treasury_config.bump,
        constraint = treasury_config.authority == authority.key() @ CustomError::UnauthorizedTreasuryAuthority
    )]
    pub treasury_config: Account<'info, TreasuryConfigV2>,

    #[account(
        seeds = [GOVERNANCE_CONFIG_V1_SEED],
        bump = security_governance_config.bump
    )]
    pub security_governance_config: Account<'info, GovernanceConfigV1>,

    #[account(
        init,
        payer = authority,
        space = 8 + TreasuryGovernanceConfigV1::INIT_SPACE,
        seeds = [TREASURY_GOVERNANCE_CONFIG_V1_SEED],
        bump
    )]
    pub treasury_governance_config: Account<'info, TreasuryGovernanceConfigV1>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(request_id: u64)]
pub struct InitializeTreasurySpendingRequestV1<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,

    #[account(
        seeds = [TREASURY_CONFIG_V2_SEED],
        bump = treasury_config.bump
    )]
    pub treasury_config: Account<'info, TreasuryConfigV2>,

    #[account(
        seeds = [TREASURY_GOVERNANCE_CONFIG_V1_SEED],
        bump = treasury_governance_config.bump,
        constraint = treasury_governance_config.treasury_config == treasury_config.key() @ CustomError::InvalidTreasuryGovernanceConfig
    )]
    pub treasury_governance_config: Account<'info, TreasuryGovernanceConfigV1>,

    #[account(
        init,
        payer = proposer,
        space = 8 + TreasurySpendingRequestV1::INIT_SPACE,
        seeds = [
            TREASURY_SPENDING_REQUEST_V1_SEED,
            treasury_config.key().as_ref(),
            &request_id.to_le_bytes()
        ],
        bump
    )]
    pub treasury_spending_request: Account<'info, TreasurySpendingRequestV1>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeTreasuryBuilderPayoutGovernanceV1<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,

    #[account(
        seeds = [TREASURY_CONFIG_V2_SEED],
        bump = treasury_config.bump
    )]
    pub treasury_config: Account<'info, TreasuryConfigV2>,

    #[account(
        seeds = [TREASURY_GOVERNANCE_CONFIG_V1_SEED],
        bump = treasury_governance_config.bump,
        constraint = treasury_governance_config.treasury_config == treasury_config.key() @ CustomError::InvalidTreasuryGovernanceConfig
    )]
    pub treasury_governance_config: Account<'info, TreasuryGovernanceConfigV1>,

    #[account(
        seeds = [
            CONTRIBUTOR_REGISTRY_V1_SEED,
            contributor_registry.wallet.as_ref()
        ],
        bump = contributor_registry.bump
    )]
    pub contributor_registry: Account<'info, ContributorRegistryV1>,

    #[account(
        constraint = contributor_milestone.contributor == contributor_registry.key() @ CustomError::InvalidContributorMilestone
    )]
    pub contributor_milestone: Account<'info, ContributorMilestoneV1>,

    #[account(
        seeds = [
            BUILDER_PAYOUT_REQUEST_V1_SEED,
            contributor_registry.key().as_ref(),
            contributor_milestone.key().as_ref()
        ],
        bump = builder_payout_request.bump,
        constraint = builder_payout_request.contributor == contributor_registry.key() @ CustomError::InvalidContributorPayoutRequest,
        constraint = builder_payout_request.milestone == contributor_milestone.key() @ CustomError::InvalidContributorPayoutRequest
    )]
    pub builder_payout_request: Account<'info, BuilderPayoutRequestV1>,

    #[account(
        init,
        payer = proposer,
        space = 8 + TreasuryBuilderPayoutGovernanceV1::INIT_SPACE,
        seeds = [
            TREASURY_BUILDER_PAYOUT_GOVERNANCE_V1_SEED,
            builder_payout_request.key().as_ref()
        ],
        bump
    )]
    pub treasury_builder_payout_governance: Account<'info, TreasuryBuilderPayoutGovernanceV1>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct ApproveTreasurySpendingRequestV1<'info> {
    #[account(
        seeds = [GOVERNANCE_CONFIG_V1_SEED],
        bump = governance_config.bump
    )]
    pub governance_config: Account<'info, GovernanceConfigV1>,

    #[account(
        seeds = [
            PROPOSAL_DECISION_V1_SEED,
            &proposal_id.to_le_bytes()
        ],
        bump = proposal_decision.bump
    )]
    pub proposal_decision: Account<'info, ProposalDecisionV1>,

    #[account(
        seeds = [
            EXECUTION_QUEUE_ITEM_V1_SEED,
            &proposal_id.to_le_bytes()
        ],
        bump = execution_queue_item.bump
    )]
    pub execution_queue_item: Account<'info, ExecutionQueueItemV1>,

    #[account(
        seeds = [TREASURY_GOVERNANCE_CONFIG_V1_SEED],
        bump = treasury_governance_config.bump
    )]
    pub treasury_governance_config: Account<'info, TreasuryGovernanceConfigV1>,

    #[account(mut)]
    pub treasury_spending_request: Account<'info, TreasurySpendingRequestV1>,
}

#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct ApproveTreasuryBuilderPayoutGovernanceV1<'info> {
    #[account(
        seeds = [GOVERNANCE_CONFIG_V1_SEED],
        bump = governance_config.bump
    )]
    pub governance_config: Account<'info, GovernanceConfigV1>,

    #[account(
        seeds = [
            PROPOSAL_DECISION_V1_SEED,
            &proposal_id.to_le_bytes()
        ],
        bump = proposal_decision.bump
    )]
    pub proposal_decision: Account<'info, ProposalDecisionV1>,

    #[account(
        seeds = [
            EXECUTION_QUEUE_ITEM_V1_SEED,
            &proposal_id.to_le_bytes()
        ],
        bump = execution_queue_item.bump
    )]
    pub execution_queue_item: Account<'info, ExecutionQueueItemV1>,

    #[account(
        seeds = [TREASURY_GOVERNANCE_CONFIG_V1_SEED],
        bump = treasury_governance_config.bump
    )]
    pub treasury_governance_config: Account<'info, TreasuryGovernanceConfigV1>,

    #[account(mut)]
    pub treasury_builder_payout_governance: Account<'info, TreasuryBuilderPayoutGovernanceV1>,
}

pub fn initialize_treasury_governance_config_v1_handler(
    ctx: Context<InitializeTreasuryGovernanceConfigV1>,
    spending_limit_usdc: u64,
    split_change_threshold_bps: u64,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    initialize_treasury_governance_config_state(
        &mut ctx.accounts.treasury_governance_config,
        ctx.accounts.treasury_config.key(),
        ctx.accounts.security_governance_config.authority,
        spending_limit_usdc,
        split_change_threshold_bps,
        now,
        ctx.bumps.treasury_governance_config,
    )
}

pub fn initialize_treasury_spending_request_v1_handler(
    ctx: Context<InitializeTreasurySpendingRequestV1>,
    request_id: u64,
    recipient: Pubkey,
    amount_usdc: u64,
    purpose_hash: [u8; 32],
    proposal_id: u64,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    initialize_treasury_spending_request_state(
        &ctx.accounts.treasury_governance_config,
        &mut ctx.accounts.treasury_spending_request,
        request_id,
        ctx.accounts.treasury_config.key(),
        ctx.accounts.proposer.key(),
        recipient,
        amount_usdc,
        purpose_hash,
        proposal_id,
        now,
        ctx.bumps.treasury_spending_request,
    )
}

pub fn initialize_treasury_builder_payout_governance_v1_handler(
    ctx: Context<InitializeTreasuryBuilderPayoutGovernanceV1>,
    proposal_id: u64,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    initialize_treasury_builder_payout_governance_state(
        &ctx.accounts.treasury_governance_config,
        ctx.accounts.builder_payout_request.key(),
        &ctx.accounts.builder_payout_request,
        &mut ctx.accounts.treasury_builder_payout_governance,
        ctx.accounts.contributor_registry.key(),
        ctx.accounts.contributor_milestone.key(),
        proposal_id,
        now,
        ctx.bumps.treasury_builder_payout_governance,
    )
}

pub fn approve_treasury_spending_request_v1_handler(
    ctx: Context<ApproveTreasurySpendingRequestV1>,
    proposal_id: u64,
) -> Result<()> {
    require!(
        ctx.accounts.treasury_governance_config.dao_enabled,
        CustomError::InvalidTreasuryGovernanceConfig
    );
    let payload_hash = hash_treasury_spending_request_payload(
        ctx.accounts.treasury_spending_request.key(),
        &ctx.accounts.treasury_spending_request,
    )?;

    validate_treasury_governance_request(
        &ctx.accounts.governance_config,
        &ctx.accounts.proposal_decision,
        &ctx.accounts.execution_queue_item,
        proposal_id,
        ProposalType::TreasuryApproveSpending,
        ActionType::TreasuryApproveSpending,
        ctx.accounts.treasury_spending_request.key(),
        payload_hash,
    )?;

    record_treasury_spending_status(
        &mut ctx.accounts.treasury_spending_request,
        TreasurySpendingStatusV1::Approved,
        0,
    )
}

pub fn approve_treasury_builder_payout_governance_v1_handler(
    ctx: Context<ApproveTreasuryBuilderPayoutGovernanceV1>,
    proposal_id: u64,
) -> Result<()> {
    require!(
        ctx.accounts.treasury_governance_config.dao_enabled,
        CustomError::InvalidTreasuryGovernanceConfig
    );
    let payload_hash = hash_treasury_builder_payout_governance_payload(
        ctx.accounts.treasury_builder_payout_governance.key(),
        &ctx.accounts.treasury_builder_payout_governance,
    )?;

    validate_treasury_governance_request(
        &ctx.accounts.governance_config,
        &ctx.accounts.proposal_decision,
        &ctx.accounts.execution_queue_item,
        proposal_id,
        ProposalType::TreasuryApproveBuilderPayout,
        ActionType::TreasuryApproveBuilderPayout,
        ctx.accounts.treasury_builder_payout_governance.key(),
        payload_hash,
    )?;

    record_treasury_builder_payout_status(
        &mut ctx.accounts.treasury_builder_payout_governance,
        TreasuryBuilderPayoutStatusV1::Approved,
    )
}

pub fn initialize_treasury_governance_config_state(
    treasury_governance_config: &mut TreasuryGovernanceConfigV1,
    treasury_config: Pubkey,
    security_authority: Pubkey,
    spending_limit_usdc: u64,
    split_change_threshold_bps: u64,
    now_ts: i64,
    bump: u8,
) -> Result<()> {
    require!(
        treasury_config != Pubkey::default() && security_authority != Pubkey::default(),
        CustomError::InvalidTreasuryGovernanceConfig
    );
    require!(spending_limit_usdc > 0, CustomError::InvalidAmount);
    require!(
        split_change_threshold_bps > 0,
        CustomError::InvalidTreasuryGovernanceConfig
    );

    treasury_governance_config.treasury_config = treasury_config;
    treasury_governance_config.security_authority = security_authority;
    treasury_governance_config.dao_enabled = true;
    treasury_governance_config.spending_limit_usdc = spending_limit_usdc;
    treasury_governance_config.split_change_threshold_bps = split_change_threshold_bps;
    treasury_governance_config.emergency_mode = false;
    treasury_governance_config.created_at = now_ts;
    treasury_governance_config.updated_at = now_ts;
    treasury_governance_config.bump = bump;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn initialize_treasury_spending_request_state(
    treasury_governance_config: &TreasuryGovernanceConfigV1,
    treasury_spending_request: &mut TreasurySpendingRequestV1,
    request_id: u64,
    treasury_config: Pubkey,
    proposer: Pubkey,
    recipient: Pubkey,
    amount_usdc: u64,
    purpose_hash: [u8; 32],
    proposal_id: u64,
    now_ts: i64,
    bump: u8,
) -> Result<()> {
    validate_treasury_governance_open(treasury_governance_config)?;
    require!(
        treasury_governance_config.treasury_config == treasury_config,
        CustomError::InvalidTreasuryGovernanceConfig
    );
    require!(
        request_id > 0,
        CustomError::InvalidTreasuryGovernanceRequest
    );
    require!(
        proposer != Pubkey::default() && recipient != Pubkey::default(),
        CustomError::InvalidTreasuryGovernanceRequest
    );
    validate_treasury_spending_amount(treasury_governance_config, amount_usdc)?;
    require!(
        purpose_hash != [0u8; 32],
        CustomError::InvalidTreasuryGovernancePayloadHash
    );
    require!(proposal_id > 0, CustomError::InvalidProposalId);

    treasury_spending_request.request_id = request_id;
    treasury_spending_request.treasury_config = treasury_config;
    treasury_spending_request.proposer = proposer;
    treasury_spending_request.recipient = recipient;
    treasury_spending_request.amount_usdc = amount_usdc;
    treasury_spending_request.purpose_hash = purpose_hash;
    treasury_spending_request.proposal_id = proposal_id;
    treasury_spending_request.status = TreasurySpendingStatusV1::Pending;
    treasury_spending_request.created_at = now_ts;
    treasury_spending_request.executed_at = 0;
    treasury_spending_request.bump = bump;

    Ok(())
}

pub fn initialize_treasury_builder_payout_governance_state(
    treasury_governance_config: &TreasuryGovernanceConfigV1,
    builder_payout_request_key: Pubkey,
    builder_payout_request: &BuilderPayoutRequestV1,
    treasury_builder_payout_governance: &mut TreasuryBuilderPayoutGovernanceV1,
    contributor_registry: Pubkey,
    milestone: Pubkey,
    proposal_id: u64,
    now_ts: i64,
    bump: u8,
) -> Result<()> {
    validate_treasury_governance_open(treasury_governance_config)?;
    require!(
        builder_payout_request.status == PayoutStatusV1::Pending,
        CustomError::InvalidContributorPayoutRequest
    );
    require!(
        builder_payout_request.contributor == contributor_registry
            && builder_payout_request.milestone == milestone,
        CustomError::InvalidContributorPayoutRequest
    );
    validate_treasury_spending_amount(treasury_governance_config, builder_payout_request.amount)?;
    require!(proposal_id > 0, CustomError::InvalidProposalId);

    treasury_builder_payout_governance.payout_request = builder_payout_request_key;
    treasury_builder_payout_governance.contributor_registry = contributor_registry;
    treasury_builder_payout_governance.milestone = milestone;
    treasury_builder_payout_governance.recipient = builder_payout_request.destination_wallet;
    treasury_builder_payout_governance.amount = builder_payout_request.amount;
    treasury_builder_payout_governance.proposal_id = proposal_id;
    treasury_builder_payout_governance.status = TreasuryBuilderPayoutStatusV1::Pending;
    treasury_builder_payout_governance.created_at = now_ts;
    treasury_builder_payout_governance.bump = bump;

    Ok(())
}

pub fn validate_treasury_governance_request(
    governance_config: &GovernanceConfigV1,
    proposal_decision: &ProposalDecisionV1,
    execution_queue_item: &ExecutionQueueItemV1,
    proposal_id: u64,
    expected_proposal_type: ProposalType,
    expected_action_type: ActionType,
    expected_target_account: Pubkey,
    expected_payload_hash: [u8; 32],
) -> Result<()> {
    validate_treasury_governance_action(expected_action_type)?;
    require!(
        !governance_config.is_paused,
        CustomError::SecurityLayerPaused
    );
    require!(
        proposal_decision.proposal_id == proposal_id,
        CustomError::InvalidProposalId
    );
    require!(
        proposal_decision.proposal_type == expected_proposal_type,
        CustomError::InvalidActionForProposalType
    );
    require!(
        matches!(
            proposal_decision.decision,
            ProposalDecision::Approved | ProposalDecision::Partial
        ),
        CustomError::ProposalNotApproved
    );
    require!(
        execution_queue_item.proposal_id == proposal_id,
        CustomError::InvalidProposalId
    );
    require!(
        execution_queue_item.status == ExecutionStatus::Executed,
        CustomError::InvalidExecutionStatus
    );
    require!(
        execution_queue_item.action_type == expected_action_type,
        CustomError::InvalidActionForProposalType
    );
    require!(
        execution_queue_item.target_program == crate::ID,
        CustomError::InvalidTreasuryGovernanceRequest
    );
    require!(
        execution_queue_item.target_account == expected_target_account,
        CustomError::InvalidTreasuryGovernanceRequest
    );
    require!(
        execution_queue_item.payload_hash == expected_payload_hash,
        CustomError::PayloadHashMismatch
    );
    require!(
        matches!(
            execution_queue_item.decision,
            ProposalDecision::Approved | ProposalDecision::Partial
        ),
        CustomError::ProposalNotApproved
    );

    Ok(())
}

pub fn validate_treasury_governance_action(action_type: ActionType) -> Result<()> {
    require!(
        matches!(
            action_type,
            ActionType::TreasuryUpdateRevenueSplit
                | ActionType::TreasuryApproveSpending
                | ActionType::TreasuryApproveBuilderPayout
        ),
        CustomError::InvalidActionForProposalType
    );

    Ok(())
}

pub fn record_treasury_spending_status(
    treasury_spending_request: &mut TreasurySpendingRequestV1,
    next_status: TreasurySpendingStatusV1,
    executed_at: i64,
) -> Result<()> {
    validate_treasury_spending_status_transition(treasury_spending_request.status, next_status)?;

    treasury_spending_request.status = next_status;
    if next_status == TreasurySpendingStatusV1::Executed {
        require!(
            executed_at > 0,
            CustomError::InvalidTreasuryGovernanceRequest
        );
        treasury_spending_request.executed_at = executed_at;
    }

    Ok(())
}

pub fn record_treasury_builder_payout_status(
    treasury_builder_payout_governance: &mut TreasuryBuilderPayoutGovernanceV1,
    next_status: TreasuryBuilderPayoutStatusV1,
) -> Result<()> {
    validate_treasury_builder_payout_status_transition(
        treasury_builder_payout_governance.status,
        next_status,
    )?;

    treasury_builder_payout_governance.status = next_status;
    Ok(())
}

pub fn validate_treasury_spending_status_transition(
    current: TreasurySpendingStatusV1,
    next: TreasurySpendingStatusV1,
) -> Result<()> {
    let is_valid = matches!(
        (current, next),
        (
            TreasurySpendingStatusV1::Pending,
            TreasurySpendingStatusV1::Approved
        ) | (
            TreasurySpendingStatusV1::Pending,
            TreasurySpendingStatusV1::Rejected
        ) | (
            TreasurySpendingStatusV1::Pending,
            TreasurySpendingStatusV1::Cancelled
        ) | (
            TreasurySpendingStatusV1::Approved,
            TreasurySpendingStatusV1::Executed
        ) | (
            TreasurySpendingStatusV1::Approved,
            TreasurySpendingStatusV1::Cancelled
        )
    );
    require!(is_valid, CustomError::InvalidTreasurySpendingStatus);
    Ok(())
}

pub fn validate_treasury_builder_payout_status_transition(
    current: TreasuryBuilderPayoutStatusV1,
    next: TreasuryBuilderPayoutStatusV1,
) -> Result<()> {
    let is_valid = matches!(
        (current, next),
        (
            TreasuryBuilderPayoutStatusV1::Pending,
            TreasuryBuilderPayoutStatusV1::Approved
        ) | (
            TreasuryBuilderPayoutStatusV1::Pending,
            TreasuryBuilderPayoutStatusV1::Rejected
        ) | (
            TreasuryBuilderPayoutStatusV1::Approved,
            TreasuryBuilderPayoutStatusV1::Executed
        )
    );
    require!(is_valid, CustomError::InvalidTreasuryBuilderPayoutStatus);
    Ok(())
}

pub fn hash_treasury_spending_request_payload(
    spending_request_key: Pubkey,
    treasury_spending_request: &TreasurySpendingRequestV1,
) -> Result<[u8; 32]> {
    let payload = TreasurySpendingRequestPayloadV1 {
        spending_request: spending_request_key,
        recipient: treasury_spending_request.recipient,
        amount_usdc: treasury_spending_request.amount_usdc,
        purpose_hash: treasury_spending_request.purpose_hash,
        proposal_id: treasury_spending_request.proposal_id,
    };
    hash_treasury_payload(&payload)
}

pub fn hash_treasury_builder_payout_governance_payload(
    payout_governance_key: Pubkey,
    treasury_builder_payout_governance: &TreasuryBuilderPayoutGovernanceV1,
) -> Result<[u8; 32]> {
    let payload = TreasuryBuilderPayoutGovernancePayloadV1 {
        payout_governance: payout_governance_key,
        payout_request: treasury_builder_payout_governance.payout_request,
        contributor_registry: treasury_builder_payout_governance.contributor_registry,
        milestone: treasury_builder_payout_governance.milestone,
        recipient: treasury_builder_payout_governance.recipient,
        amount: treasury_builder_payout_governance.amount,
        proposal_id: treasury_builder_payout_governance.proposal_id,
    };
    hash_treasury_payload(&payload)
}

pub fn hash_treasury_payload<T: AnchorSerialize>(payload: &T) -> Result<[u8; 32]> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(TREASURY_GOVERNANCE_PAYLOAD_V1_DOMAIN);
    payload
        .serialize(&mut bytes)
        .map_err(|_| error!(CustomError::InvalidTreasuryGovernancePayloadHash))?;
    hash_contributor_payload(&bytes)
}

fn validate_treasury_governance_open(
    treasury_governance_config: &TreasuryGovernanceConfigV1,
) -> Result<()> {
    require!(
        treasury_governance_config.dao_enabled && !treasury_governance_config.emergency_mode,
        CustomError::InvalidTreasuryGovernanceConfig
    );
    Ok(())
}

fn validate_treasury_spending_amount(
    treasury_governance_config: &TreasuryGovernanceConfigV1,
    amount_usdc: u64,
) -> Result<()> {
    require!(amount_usdc > 0, CustomError::InvalidAmount);
    require!(
        amount_usdc <= treasury_governance_config.spending_limit_usdc,
        CustomError::InvalidTreasuryGovernanceRequest
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const HASH_ONE: [u8; 32] = [1; 32];
    const HASH_TWO: [u8; 32] = [2; 32];

    fn spending_request() -> TreasurySpendingRequestV1 {
        TreasurySpendingRequestV1 {
            request_id: 1,
            treasury_config: Pubkey::new_unique(),
            proposer: Pubkey::new_unique(),
            recipient: Pubkey::new_unique(),
            amount_usdc: 1_000_000,
            purpose_hash: HASH_ONE,
            proposal_id: 7,
            status: TreasurySpendingStatusV1::Pending,
            created_at: 20,
            executed_at: 0,
            bump: 1,
        }
    }

    fn builder_payout_request() -> BuilderPayoutRequestV1 {
        BuilderPayoutRequestV1 {
            contributor: Pubkey::new_unique(),
            milestone: Pubkey::new_unique(),
            amount: 1_000_000,
            destination_wallet: Pubkey::new_unique(),
            status: PayoutStatusV1::Pending,
            created_at: 20,
            bump: 1,
        }
    }

    fn builder_payout_governance() -> TreasuryBuilderPayoutGovernanceV1 {
        let payout = builder_payout_request();
        TreasuryBuilderPayoutGovernanceV1 {
            payout_request: Pubkey::new_unique(),
            contributor_registry: payout.contributor,
            milestone: payout.milestone,
            recipient: payout.destination_wallet,
            amount: payout.amount,
            proposal_id: 8,
            status: TreasuryBuilderPayoutStatusV1::Pending,
            created_at: 21,
            bump: 1,
        }
    }

    fn proposal_decision(proposal_id: u64, proposal_type: ProposalType) -> ProposalDecisionV1 {
        ProposalDecisionV1 {
            proposal_id,
            proposal_type,
            proposer: Pubkey::new_unique(),
            decision: ProposalDecision::Approved,
            yes_weight: 10,
            no_weight: 0,
            start_ts: 1,
            end_ts: 2,
            finalized_ts: 3,
            bump: 1,
        }
    }

    fn execution_queue_item(
        proposal_id: u64,
        action_type: ActionType,
        target_account: Pubkey,
        payload_hash: [u8; 32],
    ) -> ExecutionQueueItemV1 {
        ExecutionQueueItemV1 {
            proposal_id,
            proposer: Pubkey::new_unique(),
            action_type,
            target_program: crate::ID,
            target_account,
            decision: ProposalDecision::Approved,
            created_at: 4,
            execute_after: 5,
            executed_at: 6,
            status: ExecutionStatus::Executed,
            payload_hash,
            bump: 1,
        }
    }

    fn governance_config() -> GovernanceConfigV1 {
        GovernanceConfigV1 {
            authority: Pubkey::new_unique(),
            min_execution_delay_seconds: 1,
            proposal_count: 1,
            emergency_guardian: Pubkey::new_unique(),
            is_paused: false,
            bump: 1,
        }
    }

    #[test]
    fn initializes_treasury_governance_config_defaults() {
        let mut config = TreasuryGovernanceConfigV1 {
            treasury_config: Pubkey::default(),
            security_authority: Pubkey::default(),
            dao_enabled: false,
            spending_limit_usdc: 0,
            split_change_threshold_bps: 0,
            emergency_mode: true,
            created_at: 0,
            updated_at: 0,
            bump: 0,
        };
        let treasury_config = Pubkey::new_unique();
        let security_authority = Pubkey::new_unique();

        initialize_treasury_governance_config_state(
            &mut config,
            treasury_config,
            security_authority,
            10_000_000,
            100,
            99,
            3,
        )
        .unwrap();

        assert_eq!(config.treasury_config, treasury_config);
        assert_eq!(config.security_authority, security_authority);
        assert!(config.dao_enabled);
        assert!(!config.emergency_mode);
        assert_eq!(config.spending_limit_usdc, 10_000_000);
        assert_eq!(config.split_change_threshold_bps, 100);
        assert_eq!(config.created_at, 99);
        assert_eq!(config.updated_at, 99);
        assert_eq!(config.bump, 3);
    }

    #[test]
    fn spending_request_status_changes_to_approved() {
        let mut request = spending_request();
        record_treasury_spending_status(&mut request, TreasurySpendingStatusV1::Approved, 0)
            .unwrap();

        assert_eq!(request.status, TreasurySpendingStatusV1::Approved);
        assert_eq!(request.executed_at, 0);
    }

    #[test]
    fn builder_payout_governance_status_changes_to_approved() {
        let mut governance = builder_payout_governance();
        record_treasury_builder_payout_status(
            &mut governance,
            TreasuryBuilderPayoutStatusV1::Approved,
        )
        .unwrap();

        assert_eq!(governance.status, TreasuryBuilderPayoutStatusV1::Approved);
    }

    #[test]
    fn illegal_action_rejected() {
        let err = validate_treasury_governance_action(ActionType::GreenLabelRefund).unwrap_err();
        assert!(format!("{err:?}").contains("InvalidActionForProposalType"));
    }

    #[test]
    fn payload_mismatch_rejected() {
        let target_account = Pubkey::new_unique();
        let proposal_id = 7;
        let governance_config = governance_config();
        let proposal_decision =
            proposal_decision(proposal_id, ProposalType::TreasuryApproveSpending);
        let queue_item = execution_queue_item(
            proposal_id,
            ActionType::TreasuryApproveSpending,
            target_account,
            HASH_ONE,
        );

        let err = validate_treasury_governance_request(
            &governance_config,
            &proposal_decision,
            &queue_item,
            proposal_id,
            ProposalType::TreasuryApproveSpending,
            ActionType::TreasuryApproveSpending,
            target_account,
            HASH_TWO,
        )
        .unwrap_err();
        assert!(format!("{err:?}").contains("PayloadHashMismatch"));
    }

    #[test]
    fn executed_security_queue_validates_treasury_request() {
        let target_account = Pubkey::new_unique();
        let proposal_id = 7;
        let governance_config = governance_config();
        let proposal_decision =
            proposal_decision(proposal_id, ProposalType::TreasuryApproveSpending);
        let queue_item = execution_queue_item(
            proposal_id,
            ActionType::TreasuryApproveSpending,
            target_account,
            HASH_ONE,
        );

        validate_treasury_governance_request(
            &governance_config,
            &proposal_decision,
            &queue_item,
            proposal_id,
            ProposalType::TreasuryApproveSpending,
            ActionType::TreasuryApproveSpending,
            target_account,
            HASH_ONE,
        )
        .unwrap();
    }
}
