use anchor_lang::prelude::*;

use crate::constants::{
    EXECUTION_QUEUE_ITEM_V1_SEED, GOVERNANCE_CONFIG_V1_SEED, MAX_EXECUTION_DELAY_SECONDS,
    MIN_EXECUTION_DELAY_SECONDS, PROPOSAL_DECISION_V1_SEED,
};
use crate::error::CustomError;
use crate::state::{
    ActionType, ExecutionQueueItemV1, ExecutionStatus, GovernanceConfigV1, ProposalDecision,
    ProposalDecisionV1, ProposalType,
};

#[derive(Accounts)]
pub struct InitializeGovernanceConfig<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + GovernanceConfigV1::INIT_SPACE,
        seeds = [GOVERNANCE_CONFIG_V1_SEED],
        bump
    )]
    pub governance_config: Account<'info, GovernanceConfigV1>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(expected_proposal_id: u64)]
pub struct CreateProposalDecision<'info> {
    #[account(
        mut,
        seeds = [GOVERNANCE_CONFIG_V1_SEED],
        bump = governance_config.bump
    )]
    pub governance_config: Account<'info, GovernanceConfigV1>,

    #[account(
        init,
        payer = authority,
        space = 8 + ProposalDecisionV1::INIT_SPACE,
        seeds = [
            PROPOSAL_DECISION_V1_SEED,
            &expected_proposal_id.to_le_bytes()
        ],
        bump
    )]
    pub proposal_decision: Account<'info, ProposalDecisionV1>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct QueueExecution<'info> {
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
        bump = proposal_decision.bump,
        constraint = proposal_decision.proposal_id == proposal_id @ CustomError::InvalidProposalId
    )]
    pub proposal_decision: Account<'info, ProposalDecisionV1>,

    #[account(
        init,
        payer = authority,
        space = 8 + ExecutionQueueItemV1::INIT_SPACE,
        seeds = [
            EXECUTION_QUEUE_ITEM_V1_SEED,
            &proposal_id.to_le_bytes()
        ],
        bump
    )]
    pub execution_queue_item: Account<'info, ExecutionQueueItemV1>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct ExecuteQueuedAction<'info> {
    #[account(
        seeds = [GOVERNANCE_CONFIG_V1_SEED],
        bump = governance_config.bump
    )]
    pub governance_config: Account<'info, GovernanceConfigV1>,

    #[account(
        mut,
        seeds = [
            EXECUTION_QUEUE_ITEM_V1_SEED,
            &proposal_id.to_le_bytes()
        ],
        bump = execution_queue_item.bump,
        constraint = execution_queue_item.proposal_id == proposal_id @ CustomError::InvalidProposalId
    )]
    pub execution_queue_item: Account<'info, ExecutionQueueItemV1>,

    pub executor: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct CancelQueuedAction<'info> {
    #[account(
        seeds = [GOVERNANCE_CONFIG_V1_SEED],
        bump = governance_config.bump
    )]
    pub governance_config: Account<'info, GovernanceConfigV1>,

    #[account(
        mut,
        seeds = [
            EXECUTION_QUEUE_ITEM_V1_SEED,
            &proposal_id.to_le_bytes()
        ],
        bump = execution_queue_item.bump,
        constraint = execution_queue_item.proposal_id == proposal_id @ CustomError::InvalidProposalId
    )]
    pub execution_queue_item: Account<'info, ExecutionQueueItemV1>,

    pub authority_or_guardian: Signer<'info>,
}

#[derive(Accounts)]
pub struct PauseSecurityLayer<'info> {
    #[account(
        mut,
        seeds = [GOVERNANCE_CONFIG_V1_SEED],
        bump = governance_config.bump
    )]
    pub governance_config: Account<'info, GovernanceConfigV1>,

    pub emergency_guardian: Signer<'info>,
}

#[derive(Accounts)]
pub struct UnpauseSecurityLayer<'info> {
    #[account(
        mut,
        seeds = [GOVERNANCE_CONFIG_V1_SEED],
        bump = governance_config.bump
    )]
    pub governance_config: Account<'info, GovernanceConfigV1>,

    pub authority: Signer<'info>,
}

pub fn initialize_governance_config_handler(
    ctx: Context<InitializeGovernanceConfig>,
    min_execution_delay_seconds: i64,
    emergency_guardian: Pubkey,
) -> Result<()> {
    initialize_governance_config_state(
        &mut ctx.accounts.governance_config,
        ctx.accounts.authority.key(),
        min_execution_delay_seconds,
        emergency_guardian,
        ctx.bumps.governance_config,
    )
}

pub fn create_proposal_decision_handler(
    ctx: Context<CreateProposalDecision>,
    expected_proposal_id: u64,
    proposal_type: ProposalType,
    decision: ProposalDecision,
    yes_weight: u64,
    no_weight: u64,
    start_ts: i64,
    end_ts: i64,
) -> Result<()> {
    let clock = Clock::get()?;

    create_proposal_decision_state(
        &mut ctx.accounts.governance_config,
        &mut ctx.accounts.proposal_decision,
        ctx.accounts.authority.key(),
        expected_proposal_id,
        proposal_type,
        decision,
        yes_weight,
        no_weight,
        start_ts,
        end_ts,
        clock.unix_timestamp,
        ctx.bumps.proposal_decision,
    )
}

pub fn queue_execution_handler(
    ctx: Context<QueueExecution>,
    proposal_id: u64,
    action_type: ActionType,
    target_program: Pubkey,
    target_account: Pubkey,
    payload_hash: [u8; 32],
) -> Result<()> {
    let clock = Clock::get()?;

    queue_execution_state(
        &ctx.accounts.governance_config,
        &ctx.accounts.proposal_decision,
        &mut ctx.accounts.execution_queue_item,
        ctx.accounts.authority.key(),
        proposal_id,
        action_type,
        target_program,
        target_account,
        payload_hash,
        clock.unix_timestamp,
        ctx.bumps.execution_queue_item,
    )
}

pub fn execute_queued_action_handler(
    ctx: Context<ExecuteQueuedAction>,
    proposal_id: u64,
    payload_hash: [u8; 32],
) -> Result<()> {
    let clock = Clock::get()?;

    execute_queued_action_state(
        &ctx.accounts.governance_config,
        &mut ctx.accounts.execution_queue_item,
        proposal_id,
        payload_hash,
        clock.unix_timestamp,
    )
}

pub fn cancel_queued_action_handler(
    ctx: Context<CancelQueuedAction>,
    proposal_id: u64,
) -> Result<()> {
    cancel_queued_action_state(
        &ctx.accounts.governance_config,
        &mut ctx.accounts.execution_queue_item,
        ctx.accounts.authority_or_guardian.key(),
        proposal_id,
    )
}

pub fn pause_security_layer_handler(ctx: Context<PauseSecurityLayer>) -> Result<()> {
    pause_security_layer_state(
        &mut ctx.accounts.governance_config,
        ctx.accounts.emergency_guardian.key(),
    )
}

pub fn unpause_security_layer_handler(ctx: Context<UnpauseSecurityLayer>) -> Result<()> {
    unpause_security_layer_state(
        &mut ctx.accounts.governance_config,
        ctx.accounts.authority.key(),
    )
}

pub fn initialize_governance_config_state(
    governance_config: &mut GovernanceConfigV1,
    authority: Pubkey,
    min_execution_delay_seconds: i64,
    emergency_guardian: Pubkey,
    bump: u8,
) -> Result<()> {
    validate_execution_delay(min_execution_delay_seconds)?;
    require!(
        emergency_guardian != Pubkey::default(),
        CustomError::InvalidEmergencyGuardian
    );

    governance_config.authority = authority;
    governance_config.min_execution_delay_seconds = min_execution_delay_seconds;
    governance_config.proposal_count = 0;
    governance_config.emergency_guardian = emergency_guardian;
    governance_config.is_paused = false;
    governance_config.bump = bump;

    Ok(())
}

pub fn create_proposal_decision_state(
    governance_config: &mut GovernanceConfigV1,
    proposal_decision: &mut ProposalDecisionV1,
    authority: Pubkey,
    expected_proposal_id: u64,
    proposal_type: ProposalType,
    decision: ProposalDecision,
    yes_weight: u64,
    no_weight: u64,
    start_ts: i64,
    end_ts: i64,
    finalized_ts: i64,
    bump: u8,
) -> Result<()> {
    require_keys_eq!(
        authority,
        governance_config.authority,
        CustomError::UnauthorizedSecurityAuthority
    );

    let next_proposal_id = governance_config
        .proposal_count
        .checked_add(1)
        .ok_or(CustomError::MathOverflow)?;
    require!(
        expected_proposal_id == next_proposal_id,
        CustomError::InvalidProposalId
    );
    require!(
        decision != ProposalDecision::Pending,
        CustomError::InvalidProposalDecision
    );
    require!(start_ts <= end_ts, CustomError::InvalidProposalTime);

    proposal_decision.proposal_id = expected_proposal_id;
    proposal_decision.proposal_type = proposal_type;
    proposal_decision.proposer = authority;
    proposal_decision.decision = decision;
    proposal_decision.yes_weight = yes_weight;
    proposal_decision.no_weight = no_weight;
    proposal_decision.start_ts = start_ts;
    proposal_decision.end_ts = end_ts;
    proposal_decision.finalized_ts = finalized_ts;
    proposal_decision.bump = bump;
    governance_config.proposal_count = expected_proposal_id;

    Ok(())
}

pub fn queue_execution_state(
    governance_config: &GovernanceConfigV1,
    proposal_decision: &ProposalDecisionV1,
    execution_queue_item: &mut ExecutionQueueItemV1,
    authority: Pubkey,
    proposal_id: u64,
    action_type: ActionType,
    target_program: Pubkey,
    target_account: Pubkey,
    payload_hash: [u8; 32],
    now_ts: i64,
    bump: u8,
) -> Result<()> {
    require_keys_eq!(
        authority,
        governance_config.authority,
        CustomError::UnauthorizedSecurityAuthority
    );
    require!(
        proposal_decision.proposal_id == proposal_id,
        CustomError::InvalidProposalId
    );
    require!(
        matches!(
            proposal_decision.decision,
            ProposalDecision::Approved | ProposalDecision::Partial
        ),
        CustomError::ProposalNotApproved
    );
    require!(
        is_action_valid_for_proposal_type(proposal_decision.proposal_type, action_type),
        CustomError::InvalidActionForProposalType
    );

    let execute_after = now_ts
        .checked_add(governance_config.min_execution_delay_seconds)
        .ok_or(CustomError::MathOverflow)?;

    execution_queue_item.proposal_id = proposal_id;
    execution_queue_item.proposer = proposal_decision.proposer;
    execution_queue_item.action_type = action_type;
    execution_queue_item.target_program = target_program;
    execution_queue_item.target_account = target_account;
    execution_queue_item.decision = proposal_decision.decision;
    execution_queue_item.created_at = now_ts;
    execution_queue_item.execute_after = execute_after;
    execution_queue_item.executed_at = 0;
    execution_queue_item.status = ExecutionStatus::Queued;
    execution_queue_item.payload_hash = payload_hash;
    execution_queue_item.bump = bump;

    Ok(())
}

pub fn execute_queued_action_state(
    governance_config: &GovernanceConfigV1,
    execution_queue_item: &mut ExecutionQueueItemV1,
    proposal_id: u64,
    payload_hash: [u8; 32],
    now_ts: i64,
) -> Result<()> {
    require!(
        !governance_config.is_paused,
        CustomError::SecurityLayerPaused
    );
    require!(
        execution_queue_item.proposal_id == proposal_id,
        CustomError::InvalidProposalId
    );
    require!(
        execution_queue_item.status == ExecutionStatus::Queued,
        CustomError::InvalidExecutionStatus
    );
    require!(
        now_ts >= execution_queue_item.execute_after,
        CustomError::ExecutionDelayNotMet
    );
    require!(
        execution_queue_item.payload_hash == payload_hash,
        CustomError::PayloadHashMismatch
    );

    execution_queue_item.executed_at = now_ts;
    execution_queue_item.status = ExecutionStatus::Executed;

    Ok(())
}

pub fn cancel_queued_action_state(
    governance_config: &GovernanceConfigV1,
    execution_queue_item: &mut ExecutionQueueItemV1,
    signer: Pubkey,
    proposal_id: u64,
) -> Result<()> {
    let is_authorized =
        signer == governance_config.authority || signer == governance_config.emergency_guardian;
    require!(is_authorized, CustomError::UnauthorizedSecurityAuthority);
    require!(
        execution_queue_item.proposal_id == proposal_id,
        CustomError::InvalidProposalId
    );
    require!(
        execution_queue_item.status == ExecutionStatus::Queued,
        CustomError::InvalidExecutionStatus
    );

    execution_queue_item.status = ExecutionStatus::Cancelled;

    Ok(())
}

pub fn pause_security_layer_state(
    governance_config: &mut GovernanceConfigV1,
    emergency_guardian: Pubkey,
) -> Result<()> {
    require_keys_eq!(
        emergency_guardian,
        governance_config.emergency_guardian,
        CustomError::UnauthorizedEmergencyGuardian
    );

    governance_config.is_paused = true;

    Ok(())
}

pub fn unpause_security_layer_state(
    governance_config: &mut GovernanceConfigV1,
    authority: Pubkey,
) -> Result<()> {
    require_keys_eq!(
        authority,
        governance_config.authority,
        CustomError::UnauthorizedSecurityAuthority
    );

    governance_config.is_paused = false;

    Ok(())
}

pub fn validate_execution_delay(min_execution_delay_seconds: i64) -> Result<()> {
    require!(
        min_execution_delay_seconds >= MIN_EXECUTION_DELAY_SECONDS
            && min_execution_delay_seconds <= MAX_EXECUTION_DELAY_SECONDS,
        CustomError::InvalidExecutionDelay
    );

    Ok(())
}

pub fn is_action_valid_for_proposal_type(
    proposal_type: ProposalType,
    action_type: ActionType,
) -> bool {
    if action_type == ActionType::Noop {
        return true;
    }

    matches!(
        (proposal_type, action_type),
        (ProposalType::GreenLabelSlash, ActionType::GreenLabelSlash)
            | (ProposalType::GreenLabelRefund, ActionType::GreenLabelRefund)
            | (
                ProposalType::PayrollEmployeeImpeach,
                ActionType::PayrollEmployeeImpeach
            )
            | (ProposalType::PayrollPayout, ActionType::PayrollPayout)
            | (
                ProposalType::TreasuryParamChange,
                ActionType::TreasuryParamChange
            )
            | (ProposalType::EmergencyPause, ActionType::EmergencyPause)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const AUTHORITY: Pubkey = Pubkey::new_from_array([1; 32]);
    const GUARDIAN: Pubkey = Pubkey::new_from_array([2; 32]);
    const OTHER: Pubkey = Pubkey::new_from_array([3; 32]);
    const TARGET_PROGRAM: Pubkey = Pubkey::new_from_array([4; 32]);
    const TARGET_ACCOUNT: Pubkey = Pubkey::new_from_array([5; 32]);
    const HASH_ONE: [u8; 32] = [8; 32];
    const HASH_TWO: [u8; 32] = [9; 32];

    fn governance_config() -> GovernanceConfigV1 {
        GovernanceConfigV1 {
            authority: AUTHORITY,
            min_execution_delay_seconds: MIN_EXECUTION_DELAY_SECONDS,
            proposal_count: 0,
            emergency_guardian: GUARDIAN,
            is_paused: false,
            bump: 250,
        }
    }

    fn proposal_decision(
        proposal_id: u64,
        proposal_type: ProposalType,
        decision: ProposalDecision,
    ) -> ProposalDecisionV1 {
        ProposalDecisionV1 {
            proposal_id,
            proposal_type,
            proposer: AUTHORITY,
            decision,
            yes_weight: 1,
            no_weight: 0,
            start_ts: 10,
            end_ts: 20,
            finalized_ts: 21,
            bump: 249,
        }
    }

    fn execution_queue_item(proposal_id: u64) -> ExecutionQueueItemV1 {
        ExecutionQueueItemV1 {
            proposal_id,
            proposer: AUTHORITY,
            action_type: ActionType::Noop,
            target_program: Pubkey::default(),
            target_account: Pubkey::default(),
            decision: ProposalDecision::Approved,
            created_at: 100,
            execute_after: 160,
            executed_at: 0,
            status: ExecutionStatus::Queued,
            payload_hash: HASH_ONE,
            bump: 248,
        }
    }

    fn assert_error_contains(err: anchor_lang::error::Error, expected: &str) {
        let message = format!("{err:?}");
        assert!(
            message.contains(expected),
            "expected {expected}, got {message}"
        );
    }

    #[test]
    fn initializes_governance_config_successfully() {
        let mut config = governance_config();
        config.authority = Pubkey::default();
        config.emergency_guardian = Pubkey::default();

        initialize_governance_config_state(
            &mut config,
            AUTHORITY,
            MIN_EXECUTION_DELAY_SECONDS,
            GUARDIAN,
            42,
        )
        .unwrap();

        assert_eq!(config.authority, AUTHORITY);
        assert_eq!(config.emergency_guardian, GUARDIAN);
        assert_eq!(config.proposal_count, 0);
        assert!(!config.is_paused);
        assert_eq!(config.bump, 42);
    }

    #[test]
    fn delay_below_minimum_fails() {
        let err = validate_execution_delay(MIN_EXECUTION_DELAY_SECONDS - 1).unwrap_err();

        assert_error_contains(err, "InvalidExecutionDelay");
    }

    #[test]
    fn delay_above_maximum_fails() {
        let err = validate_execution_delay(MAX_EXECUTION_DELAY_SECONDS + 1).unwrap_err();

        assert_error_contains(err, "InvalidExecutionDelay");
    }

    #[test]
    fn default_emergency_guardian_fails() {
        let mut config = governance_config();
        let err = initialize_governance_config_state(
            &mut config,
            AUTHORITY,
            MIN_EXECUTION_DELAY_SECONDS,
            Pubkey::default(),
            42,
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidEmergencyGuardian");
    }

    #[test]
    fn expected_proposal_id_creates_decision_successfully() {
        let mut config = governance_config();
        let mut decision =
            proposal_decision(0, ProposalType::PayrollPayout, ProposalDecision::Rejected);

        create_proposal_decision_state(
            &mut config,
            &mut decision,
            AUTHORITY,
            1,
            ProposalType::PayrollPayout,
            ProposalDecision::Approved,
            100,
            12,
            10,
            20,
            21,
            99,
        )
        .unwrap();

        assert_eq!(decision.proposal_id, 1);
        assert_eq!(decision.proposal_type, ProposalType::PayrollPayout);
        assert_eq!(decision.decision, ProposalDecision::Approved);
        assert_eq!(config.proposal_count, 1);
    }

    #[test]
    fn unexpected_proposal_id_fails() {
        let mut config = governance_config();
        let mut decision =
            proposal_decision(0, ProposalType::PayrollPayout, ProposalDecision::Rejected);
        let err = create_proposal_decision_state(
            &mut config,
            &mut decision,
            AUTHORITY,
            2,
            ProposalType::PayrollPayout,
            ProposalDecision::Approved,
            100,
            12,
            10,
            20,
            21,
            99,
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidProposalId");
    }

    #[test]
    fn proposal_id_increments_uniquely() {
        let mut config = governance_config();
        let mut first =
            proposal_decision(0, ProposalType::PayrollPayout, ProposalDecision::Rejected);
        let mut second =
            proposal_decision(0, ProposalType::EmergencyPause, ProposalDecision::Rejected);

        create_proposal_decision_state(
            &mut config,
            &mut first,
            AUTHORITY,
            1,
            ProposalType::PayrollPayout,
            ProposalDecision::Approved,
            100,
            12,
            10,
            20,
            21,
            99,
        )
        .unwrap();
        create_proposal_decision_state(
            &mut config,
            &mut second,
            AUTHORITY,
            2,
            ProposalType::EmergencyPause,
            ProposalDecision::Approved,
            100,
            12,
            30,
            40,
            41,
            98,
        )
        .unwrap();

        assert_eq!(first.proposal_id, 1);
        assert_eq!(second.proposal_id, 2);
        assert_eq!(config.proposal_count, 2);
    }

    #[test]
    fn pending_decision_creation_fails() {
        let mut config = governance_config();
        let mut decision =
            proposal_decision(0, ProposalType::PayrollPayout, ProposalDecision::Rejected);
        let err = create_proposal_decision_state(
            &mut config,
            &mut decision,
            AUTHORITY,
            1,
            ProposalType::PayrollPayout,
            ProposalDecision::Pending,
            100,
            12,
            10,
            20,
            21,
            99,
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidProposalDecision");
    }

    #[test]
    fn invalid_proposal_time_fails() {
        let mut config = governance_config();
        let mut decision =
            proposal_decision(0, ProposalType::PayrollPayout, ProposalDecision::Rejected);
        let err = create_proposal_decision_state(
            &mut config,
            &mut decision,
            AUTHORITY,
            1,
            ProposalType::PayrollPayout,
            ProposalDecision::Approved,
            100,
            12,
            20,
            10,
            21,
            99,
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidProposalTime");
    }

    #[test]
    fn rejected_proposal_cannot_queue() {
        let config = governance_config();
        let decision =
            proposal_decision(1, ProposalType::PayrollPayout, ProposalDecision::Rejected);
        let mut item = execution_queue_item(0);
        let err = queue_execution_state(
            &config,
            &decision,
            &mut item,
            AUTHORITY,
            1,
            ActionType::PayrollPayout,
            TARGET_PROGRAM,
            TARGET_ACCOUNT,
            HASH_ONE,
            100,
            44,
        )
        .unwrap_err();

        assert_error_contains(err, "ProposalNotApproved");
    }

    #[test]
    fn pending_proposal_cannot_queue() {
        let config = governance_config();
        let decision = proposal_decision(1, ProposalType::PayrollPayout, ProposalDecision::Pending);
        let mut item = execution_queue_item(0);
        let err = queue_execution_state(
            &config,
            &decision,
            &mut item,
            AUTHORITY,
            1,
            ActionType::PayrollPayout,
            TARGET_PROGRAM,
            TARGET_ACCOUNT,
            HASH_ONE,
            100,
            44,
        )
        .unwrap_err();

        assert_error_contains(err, "ProposalNotApproved");
    }

    #[test]
    fn approved_proposal_can_queue() {
        let config = governance_config();
        let decision =
            proposal_decision(1, ProposalType::PayrollPayout, ProposalDecision::Approved);
        let mut item = execution_queue_item(0);

        queue_execution_state(
            &config,
            &decision,
            &mut item,
            AUTHORITY,
            1,
            ActionType::PayrollPayout,
            TARGET_PROGRAM,
            TARGET_ACCOUNT,
            HASH_ONE,
            100,
            44,
        )
        .unwrap();

        assert_eq!(item.proposal_id, 1);
        assert_eq!(item.action_type, ActionType::PayrollPayout);
        assert_eq!(item.execute_after, 160);
        assert_eq!(item.status, ExecutionStatus::Queued);
    }

    #[test]
    fn partial_proposal_can_queue() {
        let config = governance_config();
        let decision =
            proposal_decision(1, ProposalType::GreenLabelRefund, ProposalDecision::Partial);
        let mut item = execution_queue_item(0);

        queue_execution_state(
            &config,
            &decision,
            &mut item,
            AUTHORITY,
            1,
            ActionType::GreenLabelRefund,
            TARGET_PROGRAM,
            TARGET_ACCOUNT,
            HASH_ONE,
            100,
            44,
        )
        .unwrap();

        assert_eq!(item.decision, ProposalDecision::Partial);
    }

    #[test]
    fn mismatched_proposal_type_and_action_type_fails() {
        let config = governance_config();
        let decision =
            proposal_decision(1, ProposalType::GreenLabelSlash, ProposalDecision::Approved);
        let mut item = execution_queue_item(0);
        let err = queue_execution_state(
            &config,
            &decision,
            &mut item,
            AUTHORITY,
            1,
            ActionType::PayrollPayout,
            TARGET_PROGRAM,
            TARGET_ACCOUNT,
            HASH_ONE,
            100,
            44,
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidActionForProposalType");
    }

    #[test]
    fn any_proposal_type_can_queue_noop() {
        let config = governance_config();
        let decision =
            proposal_decision(1, ProposalType::GreenLabelSlash, ProposalDecision::Approved);
        let mut item = execution_queue_item(0);

        queue_execution_state(
            &config,
            &decision,
            &mut item,
            AUTHORITY,
            1,
            ActionType::Noop,
            Pubkey::default(),
            Pubkey::default(),
            HASH_ONE,
            100,
            44,
        )
        .unwrap();

        assert_eq!(item.action_type, ActionType::Noop);
        assert_eq!(item.target_program, Pubkey::default());
        assert_eq!(item.target_account, Pubkey::default());
    }

    #[test]
    fn execute_before_delay_fails() {
        let config = governance_config();
        let mut item = execution_queue_item(1);
        let err = execute_queued_action_state(&config, &mut item, 1, HASH_ONE, 159).unwrap_err();

        assert_error_contains(err, "ExecutionDelayNotMet");
    }

    #[test]
    fn execute_after_delay_succeeds() {
        let config = governance_config();
        let mut item = execution_queue_item(1);

        execute_queued_action_state(&config, &mut item, 1, HASH_ONE, 160).unwrap();

        assert_eq!(item.status, ExecutionStatus::Executed);
        assert_eq!(item.executed_at, 160);
    }

    #[test]
    fn duplicate_execute_fails() {
        let config = governance_config();
        let mut item = execution_queue_item(1);

        execute_queued_action_state(&config, &mut item, 1, HASH_ONE, 160).unwrap();
        let err = execute_queued_action_state(&config, &mut item, 1, HASH_ONE, 161).unwrap_err();

        assert_error_contains(err, "InvalidExecutionStatus");
    }

    #[test]
    fn cancelled_item_cannot_execute() {
        let config = governance_config();
        let mut item = execution_queue_item(1);

        cancel_queued_action_state(&config, &mut item, GUARDIAN, 1).unwrap();
        let err = execute_queued_action_state(&config, &mut item, 1, HASH_ONE, 160).unwrap_err();

        assert_error_contains(err, "InvalidExecutionStatus");
    }

    #[test]
    fn guardian_can_pause() {
        let mut config = governance_config();

        pause_security_layer_state(&mut config, GUARDIAN).unwrap();

        assert!(config.is_paused);
    }

    #[test]
    fn paused_config_cannot_execute() {
        let mut config = governance_config();
        let mut item = execution_queue_item(1);
        pause_security_layer_state(&mut config, GUARDIAN).unwrap();
        let err = execute_queued_action_state(&config, &mut item, 1, HASH_ONE, 160).unwrap_err();

        assert_error_contains(err, "SecurityLayerPaused");
    }

    #[test]
    fn guardian_cannot_unpause() {
        let mut config = governance_config();
        pause_security_layer_state(&mut config, GUARDIAN).unwrap();
        let err = unpause_security_layer_state(&mut config, GUARDIAN).unwrap_err();

        assert_error_contains(err, "UnauthorizedSecurityAuthority");
    }

    #[test]
    fn authority_can_unpause() {
        let mut config = governance_config();
        pause_security_layer_state(&mut config, GUARDIAN).unwrap();

        unpause_security_layer_state(&mut config, AUTHORITY).unwrap();

        assert!(!config.is_paused);
    }

    #[test]
    fn payload_hash_mismatch_fails() {
        let config = governance_config();
        let mut item = execution_queue_item(1);
        let err = execute_queued_action_state(&config, &mut item, 1, HASH_TWO, 160).unwrap_err();

        assert_error_contains(err, "PayloadHashMismatch");
    }

    #[test]
    fn noop_action_can_execute() {
        let config = governance_config();
        let mut item = execution_queue_item(1);
        item.action_type = ActionType::Noop;

        execute_queued_action_state(&config, &mut item, 1, HASH_ONE, 160).unwrap();

        assert_eq!(item.status, ExecutionStatus::Executed);
    }

    #[test]
    fn non_noop_action_records_metadata_without_cpi() {
        let config = governance_config();
        let decision =
            proposal_decision(1, ProposalType::GreenLabelSlash, ProposalDecision::Approved);
        let mut item = execution_queue_item(0);

        queue_execution_state(
            &config,
            &decision,
            &mut item,
            AUTHORITY,
            1,
            ActionType::GreenLabelSlash,
            TARGET_PROGRAM,
            TARGET_ACCOUNT,
            HASH_ONE,
            100,
            44,
        )
        .unwrap();
        execute_queued_action_state(&config, &mut item, 1, HASH_ONE, 160).unwrap();

        assert_eq!(item.action_type, ActionType::GreenLabelSlash);
        assert_eq!(item.target_program, TARGET_PROGRAM);
        assert_eq!(item.target_account, TARGET_ACCOUNT);
        assert_eq!(item.status, ExecutionStatus::Executed);
    }

    #[test]
    fn unauthorized_cancel_fails() {
        let config = governance_config();
        let mut item = execution_queue_item(1);
        let err = cancel_queued_action_state(&config, &mut item, OTHER, 1).unwrap_err();

        assert_error_contains(err, "UnauthorizedSecurityAuthority");
    }
}
