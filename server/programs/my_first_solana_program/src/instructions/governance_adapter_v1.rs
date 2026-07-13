use anchor_lang::prelude::*;

use crate::constants::{
    GOVERNANCE_CONFIG_V1_SEED, GOVERNANCE_PROPOSAL_V1_SEED, GOVERNANCE_SNAPSHOT_V1_SEED,
    GOVERNANCE_VOTING_CONFIG_V1_SEED, PROPOSAL_DECISION_V1_SEED,
    UNIVERSAL_GOVERNANCE_DECISION_ADAPTER_V1_SEED,
};
use crate::error::CustomError;
use crate::instructions::governance_v1::{
    checked_governance_total_votes, has_governance_approval, has_governance_quorum,
    validate_governance_voting_config,
};
use crate::state::{
    ActionType, GovernanceConfigV1, GovernanceProposalStatusV1, GovernanceProposalV1,
    GovernanceSnapshotV1, GovernanceVotingConfigV1, ProposalDecision, ProposalDecisionV1,
    ProposalType, UniversalGovernanceDecisionAdapterV1,
};

#[derive(Accounts)]
pub struct CreateGovernanceDecisionAdapterV1<'info> {
    #[account(
        mut,
        seeds = [GOVERNANCE_CONFIG_V1_SEED],
        bump = security_governance_config.bump
    )]
    pub security_governance_config: Account<'info, GovernanceConfigV1>,

    #[account(
        seeds = [GOVERNANCE_VOTING_CONFIG_V1_SEED],
        bump = governance_voting_config.bump
    )]
    pub governance_voting_config: Account<'info, GovernanceVotingConfigV1>,

    #[account(
        seeds = [GOVERNANCE_PROPOSAL_V1_SEED, &governance_proposal.proposal_id.to_le_bytes()],
        bump = governance_proposal.bump
    )]
    pub governance_proposal: Account<'info, GovernanceProposalV1>,

    #[account(
        seeds = [GOVERNANCE_SNAPSHOT_V1_SEED, governance_proposal.key().as_ref()],
        bump = governance_snapshot.bump,
        constraint = governance_snapshot.proposal == governance_proposal.key() @ CustomError::InvalidGovernanceSnapshot
    )]
    pub governance_snapshot: Account<'info, GovernanceSnapshotV1>,

    #[account(
        init,
        payer = payer,
        space = 8 + UniversalGovernanceDecisionAdapterV1::INIT_SPACE,
        seeds = [
            UNIVERSAL_GOVERNANCE_DECISION_ADAPTER_V1_SEED,
            governance_proposal.key().as_ref()
        ],
        bump
    )]
    pub governance_decision_adapter: Account<'info, UniversalGovernanceDecisionAdapterV1>,

    #[account(
        init,
        payer = payer,
        space = 8 + ProposalDecisionV1::INIT_SPACE,
        seeds = [PROPOSAL_DECISION_V1_SEED, &governance_proposal.proposal_id.to_le_bytes()],
        bump
    )]
    pub proposal_decision: Account<'info, ProposalDecisionV1>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn create_governance_decision_adapter_v1_handler(
    ctx: Context<CreateGovernanceDecisionAdapterV1>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let governance_proposal_key = ctx.accounts.governance_proposal.key();
    let governance_snapshot_key = ctx.accounts.governance_snapshot.key();
    let proposal_decision_key = ctx.accounts.proposal_decision.key();

    create_governance_decision_adapter_state(
        &mut ctx.accounts.security_governance_config,
        &ctx.accounts.governance_voting_config,
        &ctx.accounts.governance_proposal,
        &ctx.accounts.governance_snapshot,
        &mut ctx.accounts.governance_decision_adapter,
        &mut ctx.accounts.proposal_decision,
        governance_proposal_key,
        governance_snapshot_key,
        proposal_decision_key,
        now,
        ctx.bumps.governance_decision_adapter,
        ctx.bumps.proposal_decision,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn create_governance_decision_adapter_state(
    security_governance_config: &mut GovernanceConfigV1,
    governance_voting_config: &GovernanceVotingConfigV1,
    governance_proposal: &GovernanceProposalV1,
    governance_snapshot: &GovernanceSnapshotV1,
    governance_decision_adapter: &mut UniversalGovernanceDecisionAdapterV1,
    proposal_decision: &mut ProposalDecisionV1,
    governance_proposal_key: Pubkey,
    governance_snapshot_key: Pubkey,
    proposal_decision_key: Pubkey,
    created_at: i64,
    adapter_bump: u8,
    proposal_decision_bump: u8,
) -> Result<()> {
    require!(
        governance_decision_adapter.governance_proposal == Pubkey::default()
            && governance_decision_adapter.proposal_decision == Pubkey::default(),
        CustomError::InvalidGovernanceDecisionAdapter
    );
    require!(
        proposal_decision.proposal_id == 0,
        CustomError::InvalidProposalDecision
    );

    validate_governance_decision_adapter_inputs(
        security_governance_config,
        governance_voting_config,
        governance_proposal,
        governance_snapshot,
        governance_proposal_key,
        governance_snapshot_key,
    )?;

    let action_type = security_action_type_from_u8(governance_proposal.action_type)?;
    let proposal_type = security_proposal_type_for_action(action_type)?;

    governance_decision_adapter.governance_proposal = governance_proposal_key;
    governance_decision_adapter.proposal_decision = proposal_decision_key;
    governance_decision_adapter.action_type = action_type;
    governance_decision_adapter.target_program = governance_proposal.target_program;
    governance_decision_adapter.target_account = governance_proposal.target_account;
    governance_decision_adapter.payload_hash = governance_proposal.payload_hash;
    governance_decision_adapter.created_at = created_at;
    governance_decision_adapter.executed = true;
    governance_decision_adapter.bump = adapter_bump;

    proposal_decision.proposal_id = governance_proposal.proposal_id;
    proposal_decision.proposal_type = proposal_type;
    proposal_decision.proposer = governance_proposal.proposer;
    proposal_decision.decision = ProposalDecision::Approved;
    proposal_decision.yes_weight = governance_proposal.yes_weight;
    proposal_decision.no_weight = governance_proposal.no_weight;
    proposal_decision.start_ts = governance_proposal.voting_start_ts;
    proposal_decision.end_ts = governance_proposal.voting_end_ts;
    proposal_decision.finalized_ts = governance_proposal.finalized_at;
    proposal_decision.bump = proposal_decision_bump;

    security_governance_config.proposal_count = governance_proposal.proposal_id;

    Ok(())
}

pub fn validate_governance_decision_adapter_inputs(
    security_governance_config: &GovernanceConfigV1,
    governance_voting_config: &GovernanceVotingConfigV1,
    governance_proposal: &GovernanceProposalV1,
    governance_snapshot: &GovernanceSnapshotV1,
    governance_proposal_key: Pubkey,
    governance_snapshot_key: Pubkey,
) -> Result<()> {
    validate_governance_voting_config(
        governance_voting_config.quorum_bps,
        governance_voting_config.approval_threshold_bps,
        governance_voting_config.voting_period_seconds,
    )?;
    require!(
        governance_proposal.status == GovernanceProposalStatusV1::Passed,
        CustomError::InvalidGovernanceProposal
    );
    require!(
        governance_snapshot.finalized,
        CustomError::InvalidGovernanceSnapshot
    );
    require!(
        governance_proposal.snapshot == governance_snapshot_key,
        CustomError::InvalidGovernanceSnapshot
    );
    require!(
        governance_snapshot.proposal == governance_proposal_key,
        CustomError::InvalidGovernanceSnapshot
    );
    require!(
        governance_proposal.finalized_at > 0,
        CustomError::ProposalAlreadyFinalized
    );
    require!(
        governance_proposal.payload_hash != [0u8; 32],
        CustomError::InvalidGovernanceProposal
    );
    require!(
        governance_proposal.target_program != Pubkey::default()
            && governance_proposal.target_account != Pubkey::default(),
        CustomError::InvalidGovernanceProposal
    );

    let expected_proposal_id = security_governance_config
        .proposal_count
        .checked_add(1)
        .ok_or(CustomError::MathOverflow)?;
    require!(
        governance_proposal.proposal_id == expected_proposal_id,
        CustomError::InvalidProposalId
    );

    require!(
        governance_proposal.yes_weight == governance_snapshot.yes_weight
            && governance_proposal.no_weight == governance_snapshot.no_weight
            && governance_proposal.abstain_weight == governance_snapshot.abstain_weight,
        CustomError::InvalidGovernanceVote
    );

    let total_votes = checked_governance_total_votes(governance_snapshot)?;
    require!(
        has_governance_quorum(
            total_votes,
            governance_snapshot.total_voting_power,
            governance_voting_config.quorum_bps,
        )?,
        CustomError::QuorumNotReached
    );
    require!(
        has_governance_approval(
            governance_snapshot.yes_weight,
            governance_snapshot.no_weight,
            governance_voting_config.approval_threshold_bps,
        )?,
        CustomError::InvalidGovernanceVote
    );

    Ok(())
}

pub fn security_action_type_from_u8(action_type: u8) -> Result<ActionType> {
    match action_type {
        0 => Ok(ActionType::Noop),
        1 => Ok(ActionType::GreenLabelSlash),
        2 => Ok(ActionType::GreenLabelRefund),
        3 => Ok(ActionType::PayrollEmployeeImpeach),
        4 => Ok(ActionType::PayrollPayout),
        5 => Ok(ActionType::TreasuryParamChange),
        6 => Ok(ActionType::EmergencyPause),
        7 => Ok(ActionType::ContributorAddContributor),
        8 => Ok(ActionType::ContributorRemoveContributor),
        9 => Ok(ActionType::ContributorUpdateRole),
        10 => Ok(ActionType::ContributorApproveMilestone),
        11 => Ok(ActionType::ContributorApproveBuilderPayout),
        _ => err!(CustomError::InvalidActionForProposalType),
    }
}

pub fn security_proposal_type_for_action(action_type: ActionType) -> Result<ProposalType> {
    match action_type {
        ActionType::Noop => err!(CustomError::InvalidActionForProposalType),
        ActionType::GreenLabelSlash => Ok(ProposalType::GreenLabelSlash),
        ActionType::GreenLabelRefund => Ok(ProposalType::GreenLabelRefund),
        ActionType::PayrollEmployeeImpeach => Ok(ProposalType::PayrollEmployeeImpeach),
        ActionType::PayrollPayout => Ok(ProposalType::PayrollPayout),
        ActionType::TreasuryParamChange => Ok(ProposalType::TreasuryParamChange),
        ActionType::EmergencyPause => Ok(ProposalType::EmergencyPause),
        ActionType::ContributorAddContributor => Ok(ProposalType::ContributorAddContributor),
        ActionType::ContributorRemoveContributor => Ok(ProposalType::ContributorRemoveContributor),
        ActionType::ContributorUpdateRole => Ok(ProposalType::ContributorUpdateRole),
        ActionType::ContributorApproveMilestone => Ok(ProposalType::ContributorApproveMilestone),
        ActionType::ContributorApproveBuilderPayout => {
            Ok(ProposalType::ContributorApproveBuilderPayout)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{
        GOVERNANCE_DEFAULT_APPROVAL_THRESHOLD_BPS, GOVERNANCE_DEFAULT_QUORUM_BPS,
        GOVERNANCE_DEFAULT_VOTING_PERIOD_SECONDS,
    };
    use crate::state::GovernanceProposalTypeV1;

    const AUTHORITY: Pubkey = Pubkey::new_from_array([1; 32]);
    const GUARDIAN: Pubkey = Pubkey::new_from_array([2; 32]);
    const PROPOSER: Pubkey = Pubkey::new_from_array([3; 32]);
    const PROPOSAL_KEY: Pubkey = Pubkey::new_from_array([4; 32]);
    const SNAPSHOT_KEY: Pubkey = Pubkey::new_from_array([5; 32]);
    const DECISION_KEY: Pubkey = Pubkey::new_from_array([6; 32]);
    const TARGET_PROGRAM: Pubkey = Pubkey::new_from_array([7; 32]);
    const TARGET_ACCOUNT: Pubkey = Pubkey::new_from_array([8; 32]);
    const PAYLOAD_HASH: [u8; 32] = [9; 32];

    fn security_config(proposal_count: u64) -> GovernanceConfigV1 {
        GovernanceConfigV1 {
            authority: AUTHORITY,
            min_execution_delay_seconds: 60,
            proposal_count,
            emergency_guardian: GUARDIAN,
            is_paused: false,
            bump: 1,
        }
    }

    fn voting_config() -> GovernanceVotingConfigV1 {
        GovernanceVotingConfigV1 {
            authority: AUTHORITY,
            quorum_bps: GOVERNANCE_DEFAULT_QUORUM_BPS,
            approval_threshold_bps: GOVERNANCE_DEFAULT_APPROVAL_THRESHOLD_BPS,
            voting_period_seconds: GOVERNANCE_DEFAULT_VOTING_PERIOD_SECONDS,
            created_at: 10,
            bump: 2,
        }
    }

    fn passed_proposal() -> GovernanceProposalV1 {
        GovernanceProposalV1 {
            proposal_id: 1,
            proposer: PROPOSER,
            proposal_type: GovernanceProposalTypeV1::Contributor,
            action_type: 7,
            target_program: TARGET_PROGRAM,
            target_account: TARGET_ACCOUNT,
            payload_hash: PAYLOAD_HASH,
            status: GovernanceProposalStatusV1::Passed,
            voting_start_ts: 100,
            voting_end_ts: 200,
            created_at: 90,
            snapshot: SNAPSHOT_KEY,
            yes_weight: 70,
            no_weight: 30,
            abstain_weight: 0,
            finalized_at: 220,
            bump: 3,
        }
    }

    fn finalized_snapshot() -> GovernanceSnapshotV1 {
        GovernanceSnapshotV1 {
            proposal: PROPOSAL_KEY,
            total_voting_power: 100,
            yes_weight: 70,
            no_weight: 30,
            abstain_weight: 0,
            created_at: 100,
            finalized: true,
            bump: 4,
        }
    }

    fn empty_adapter() -> UniversalGovernanceDecisionAdapterV1 {
        UniversalGovernanceDecisionAdapterV1 {
            governance_proposal: Pubkey::default(),
            proposal_decision: Pubkey::default(),
            action_type: ActionType::Noop,
            target_program: Pubkey::default(),
            target_account: Pubkey::default(),
            payload_hash: [0; 32],
            created_at: 0,
            executed: false,
            bump: 0,
        }
    }

    fn empty_decision() -> ProposalDecisionV1 {
        ProposalDecisionV1 {
            proposal_id: 0,
            proposal_type: ProposalType::TreasuryParamChange,
            proposer: Pubkey::default(),
            decision: ProposalDecision::Pending,
            yes_weight: 0,
            no_weight: 0,
            start_ts: 0,
            end_ts: 0,
            finalized_ts: 0,
            bump: 0,
        }
    }

    fn create_adapter(
        security_config: &mut GovernanceConfigV1,
        proposal: &GovernanceProposalV1,
        snapshot: &GovernanceSnapshotV1,
        adapter: &mut UniversalGovernanceDecisionAdapterV1,
        decision: &mut ProposalDecisionV1,
    ) -> Result<()> {
        create_governance_decision_adapter_state(
            security_config,
            &voting_config(),
            proposal,
            snapshot,
            adapter,
            decision,
            PROPOSAL_KEY,
            SNAPSHOT_KEY,
            DECISION_KEY,
            300,
            5,
            6,
        )
    }

    #[test]
    fn passed_proposal_generates_adapter() {
        let mut security_config = security_config(0);
        let proposal = passed_proposal();
        let snapshot = finalized_snapshot();
        let mut adapter = empty_adapter();
        let mut decision = empty_decision();

        create_adapter(
            &mut security_config,
            &proposal,
            &snapshot,
            &mut adapter,
            &mut decision,
        )
        .unwrap();

        assert_eq!(adapter.governance_proposal, PROPOSAL_KEY);
        assert_eq!(adapter.proposal_decision, DECISION_KEY);
        assert_eq!(adapter.action_type, ActionType::ContributorAddContributor);
        assert!(adapter.executed);
    }

    #[test]
    fn passed_proposal_generates_proposal_decision() {
        let mut security_config = security_config(0);
        let proposal = passed_proposal();
        let snapshot = finalized_snapshot();
        let mut adapter = empty_adapter();
        let mut decision = empty_decision();

        create_adapter(
            &mut security_config,
            &proposal,
            &snapshot,
            &mut adapter,
            &mut decision,
        )
        .unwrap();

        assert_eq!(decision.proposal_id, proposal.proposal_id);
        assert_eq!(
            decision.proposal_type,
            ProposalType::ContributorAddContributor
        );
        assert_eq!(decision.decision, ProposalDecision::Approved);
        assert_eq!(decision.yes_weight, proposal.yes_weight);
        assert_eq!(security_config.proposal_count, proposal.proposal_id);
    }

    #[test]
    fn rejected_proposal_fails() {
        let mut security_config = security_config(0);
        let mut proposal = passed_proposal();
        proposal.status = GovernanceProposalStatusV1::Rejected;
        let snapshot = finalized_snapshot();
        let mut adapter = empty_adapter();
        let mut decision = empty_decision();

        let err = create_adapter(
            &mut security_config,
            &proposal,
            &snapshot,
            &mut adapter,
            &mut decision,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::InvalidGovernanceProposal.into());
    }

    #[test]
    fn unfinalized_snapshot_fails() {
        let mut security_config = security_config(0);
        let proposal = passed_proposal();
        let mut snapshot = finalized_snapshot();
        snapshot.finalized = false;
        let mut adapter = empty_adapter();
        let mut decision = empty_decision();

        let err = create_adapter(
            &mut security_config,
            &proposal,
            &snapshot,
            &mut adapter,
            &mut decision,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::InvalidGovernanceSnapshot.into());
    }

    #[test]
    fn duplicate_adapter_fails() {
        let mut security_config = security_config(0);
        let proposal = passed_proposal();
        let snapshot = finalized_snapshot();
        let mut adapter = empty_adapter();
        let mut decision = empty_decision();

        create_adapter(
            &mut security_config,
            &proposal,
            &snapshot,
            &mut adapter,
            &mut decision,
        )
        .unwrap();

        let mut second_decision = empty_decision();
        let err = create_adapter(
            &mut security_config,
            &proposal,
            &snapshot,
            &mut adapter,
            &mut second_decision,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::InvalidGovernanceDecisionAdapter.into());
    }

    #[test]
    fn payload_hash_is_preserved_in_adapter() {
        let mut security_config = security_config(0);
        let proposal = passed_proposal();
        let snapshot = finalized_snapshot();
        let mut adapter = empty_adapter();
        let mut decision = empty_decision();

        create_adapter(
            &mut security_config,
            &proposal,
            &snapshot,
            &mut adapter,
            &mut decision,
        )
        .unwrap();

        assert_eq!(adapter.payload_hash, proposal.payload_hash);
        assert_eq!(adapter.target_program, proposal.target_program);
        assert_eq!(adapter.target_account, proposal.target_account);
    }

    #[test]
    fn proposal_association_is_correct() {
        let mut security_config = security_config(0);
        let proposal = passed_proposal();
        let snapshot = finalized_snapshot();
        let mut adapter = empty_adapter();
        let mut decision = empty_decision();

        create_adapter(
            &mut security_config,
            &proposal,
            &snapshot,
            &mut adapter,
            &mut decision,
        )
        .unwrap();

        assert_eq!(adapter.governance_proposal, PROPOSAL_KEY);
        assert_eq!(adapter.proposal_decision, DECISION_KEY);
        assert_eq!(decision.proposer, proposal.proposer);
    }
}
