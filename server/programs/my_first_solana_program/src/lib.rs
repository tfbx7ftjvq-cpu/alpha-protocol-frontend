use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

pub use constants::*;
pub use error::*;
pub use instructions::*;
pub use state::*;

declare_id!("HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY");

#[program]
pub mod my_first_solana_program {
    use super::*;

    pub fn initialize_protocol(ctx: Context<InitializeProtocol>) -> Result<()> {
        instructions::initialize_protocol::initialize_protocol_handler(ctx)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit::deposit_handler(ctx, amount)
    }

    pub fn initialize_usdc_treasury(
        ctx: Context<InitializeUsdcTreasury>,
        usdc_mint: Pubkey,
        alpha_mint: Pubkey,
    ) -> Result<()> {
        instructions::initialize_usdc_treasury::initialize_usdc_treasury_handler(
            ctx, usdc_mint, alpha_mint,
        )
    }

    pub fn deposit_usdc_revenue(ctx: Context<DepositUsdcRevenue>, amount: u64) -> Result<()> {
        instructions::deposit_usdc_revenue::deposit_usdc_revenue_handler(ctx, amount)
    }

    pub fn initialize_revenue_routing_stats_v1(
        ctx: Context<InitializeRevenueRoutingStatsV1>,
    ) -> Result<()> {
        instructions::deposit_usdc_revenue::initialize_revenue_routing_stats_v1_handler(ctx)
    }

    pub fn route_usdc_revenue_v1(
        ctx: Context<RouteUsdcRevenueV1>,
        revenue_type: RevenueType,
        amount: u64,
    ) -> Result<()> {
        instructions::deposit_usdc_revenue::route_usdc_revenue_v1_handler(ctx, revenue_type, amount)
    }

    pub fn initialize_treasury_governance_config_v1(
        ctx: Context<InitializeTreasuryGovernanceConfigV1>,
        spending_limit_usdc: u64,
        split_change_threshold_bps: u64,
    ) -> Result<()> {
        instructions::treasury_governance_v1::initialize_treasury_governance_config_v1_handler(
            ctx,
            spending_limit_usdc,
            split_change_threshold_bps,
        )
    }

    pub fn initialize_treasury_spending_request_v1(
        ctx: Context<InitializeTreasurySpendingRequestV1>,
        request_id: u64,
        recipient: Pubkey,
        amount_usdc: u64,
        purpose_hash: [u8; 32],
        proposal_id: u64,
    ) -> Result<()> {
        instructions::treasury_governance_v1::initialize_treasury_spending_request_v1_handler(
            ctx,
            request_id,
            recipient,
            amount_usdc,
            purpose_hash,
            proposal_id,
        )
    }

    pub fn initialize_treasury_builder_payout_governance_v1(
        ctx: Context<InitializeTreasuryBuilderPayoutGovernanceV1>,
        proposal_id: u64,
    ) -> Result<()> {
        instructions::treasury_governance_v1::initialize_treasury_builder_payout_governance_v1_handler(
            ctx,
            proposal_id,
        )
    }

    pub fn approve_treasury_spending_request_v1(
        ctx: Context<ApproveTreasurySpendingRequestV1>,
        proposal_id: u64,
    ) -> Result<()> {
        instructions::treasury_governance_v1::approve_treasury_spending_request_v1_handler(
            ctx,
            proposal_id,
        )
    }

    pub fn approve_treasury_builder_payout_governance_v1(
        ctx: Context<ApproveTreasuryBuilderPayoutGovernanceV1>,
        proposal_id: u64,
    ) -> Result<()> {
        instructions::treasury_governance_v1::approve_treasury_builder_payout_governance_v1_handler(
            ctx,
            proposal_id,
        )
    }

    pub fn execute_treasury_builder_payout_v1(
        ctx: Context<ExecuteTreasuryBuilderPayoutV1>,
    ) -> Result<()> {
        instructions::treasury_execution_v1::execute_treasury_builder_payout_v1_handler(ctx)
    }

    pub fn execute_treasury_spending_v1(ctx: Context<ExecuteTreasurySpendingV1>) -> Result<()> {
        instructions::treasury_execution_v1::execute_treasury_spending_v1_handler(ctx)
    }

    pub fn initialize_victim_relief_config_v1(
        ctx: Context<InitializeVictimReliefConfigV1>,
    ) -> Result<()> {
        instructions::victim_relief_v1::initialize_victim_relief_config_v1_handler(ctx)
    }

    pub fn initialize_victim_relief_policy_v1(
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
        instructions::victim_relief_v1::initialize_victim_relief_policy_v1_handler(
            ctx,
            min_claim_amount_usdc,
            max_claim_amount_usdc,
            max_payout_per_case_usdc,
            evidence_window_seconds,
            review_window_seconds,
            appeal_window_seconds,
            submission_cooldown_seconds,
            max_evidence_items,
            max_active_cases_per_claimant,
        )
    }

    pub fn submit_victim_relief_case_v1(
        ctx: Context<SubmitVictimReliefCaseV1>,
        case_id: u64,
        subject_commitment: [u8; 32],
        evidence_root: [u8; 32],
        evidence_count: u32,
        claimed_amount_usdc: u64,
    ) -> Result<()> {
        instructions::victim_relief_v1::submit_victim_relief_case_v1_handler(
            ctx,
            case_id,
            subject_commitment,
            evidence_root,
            evidence_count,
            claimed_amount_usdc,
        )
    }

    pub fn update_victim_relief_evidence_root_v1(
        ctx: Context<UpdateVictimReliefEvidenceRootV1>,
        new_evidence_root: [u8; 32],
        new_evidence_count: u32,
    ) -> Result<()> {
        instructions::victim_relief_v1::update_victim_relief_evidence_root_v1_handler(
            ctx,
            new_evidence_root,
            new_evidence_count,
        )
    }

    pub fn cancel_victim_relief_case_v1(ctx: Context<CancelVictimReliefCaseV1>) -> Result<()> {
        instructions::victim_relief_v1::cancel_victim_relief_case_v1_handler(ctx)
    }

    pub fn expire_victim_relief_case_v1(ctx: Context<ExpireVictimReliefCaseV1>) -> Result<()> {
        instructions::victim_relief_v1::expire_victim_relief_case_v1_handler(ctx)
    }

    pub fn freeze_victim_relief_evidence_v1(
        ctx: Context<FreezeVictimReliefEvidenceV1>,
    ) -> Result<()> {
        instructions::victim_relief_v1::freeze_victim_relief_evidence_v1_handler(ctx)
    }

    pub fn execute_approve_victim_relief_case_v1(
        ctx: Context<ExecuteApproveVictimReliefCaseV1>,
    ) -> Result<()> {
        instructions::victim_relief_v1::execute_approve_victim_relief_case_v1_handler(ctx)
    }

    pub fn execute_reject_victim_relief_case_v1(
        ctx: Context<ExecuteRejectVictimReliefCaseV1>,
    ) -> Result<()> {
        instructions::victim_relief_v1::execute_reject_victim_relief_case_v1_handler(ctx)
    }

    pub fn open_victim_relief_appeal_v1(
        ctx: Context<OpenVictimReliefAppealV1>,
        appeal_evidence_root: [u8; 32],
        appeal_evidence_count: u32,
    ) -> Result<()> {
        instructions::victim_relief_v1::open_victim_relief_appeal_v1_handler(
            ctx,
            appeal_evidence_root,
            appeal_evidence_count,
        )
    }

    pub fn execute_uphold_victim_relief_appeal_v1(
        ctx: Context<ExecuteUpholdVictimReliefAppealV1>,
    ) -> Result<()> {
        instructions::victim_relief_v1::execute_uphold_victim_relief_appeal_v1_handler(ctx)
    }

    pub fn execute_overturn_victim_relief_appeal_v1(
        ctx: Context<ExecuteOverturnVictimReliefAppealV1>,
    ) -> Result<()> {
        instructions::victim_relief_v1::execute_overturn_victim_relief_appeal_v1_handler(ctx)
    }

    pub fn execute_victim_relief_approved_payout_v1(
        ctx: Context<ExecuteVictimReliefApprovedPayoutV1>,
    ) -> Result<()> {
        instructions::victim_relief_v1::execute_victim_relief_approved_payout_v1_handler(ctx)
    }

    pub fn initialize_governance_config_v1(
        ctx: Context<InitializeGovernanceConfigV1>,
    ) -> Result<()> {
        instructions::governance_v1::initialize_governance_config_v1_handler(ctx)
    }

    pub fn initialize_governance_voting_config_v1(
        ctx: Context<InitializeGovernanceVotingConfigV1>,
    ) -> Result<()> {
        instructions::governance_v1::initialize_governance_voting_config_v1_handler(ctx)
    }

    pub fn initialize_protocol_module_registry_v1(
        ctx: Context<InitializeProtocolModuleRegistryV1>,
        module_id: ProtocolModuleIdV1,
        schema_version: u16,
    ) -> Result<()> {
        instructions::protocol_module_registry_v1::initialize_protocol_module_registry_v1_handler(
            ctx,
            module_id,
            schema_version,
        )
    }

    pub fn initialize_governance_proposal_v1(
        ctx: Context<InitializeGovernanceProposalV1>,
        proposal_id: u64,
        proposal_type: GovernanceProposalTypeV1,
        action_type: u8,
        target_program: Pubkey,
        target_account: Pubkey,
        payload_hash: [u8; 32],
        voting_start_ts: i64,
        voting_end_ts: i64,
    ) -> Result<()> {
        instructions::governance_v1::initialize_governance_proposal_v1_handler(
            ctx,
            proposal_id,
            proposal_type,
            action_type,
            target_program,
            target_account,
            payload_hash,
            voting_start_ts,
            voting_end_ts,
        )
    }

    pub fn initialize_governance_proposal_with_action_v1(
        ctx: Context<InitializeGovernanceProposalWithActionV1>,
        proposal_id: u64,
        request: GovernanceActionRequestV1,
    ) -> Result<()> {
        instructions::governance_v1::initialize_governance_proposal_with_action_v1_handler(
            ctx,
            proposal_id,
            request,
        )
    }

    pub fn initialize_governance_position_v1(
        ctx: Context<InitializeGovernancePositionV1>,
    ) -> Result<()> {
        instructions::governance_v1::initialize_governance_position_v1_handler(ctx)
    }

    pub fn lock_alpha_for_governance(
        ctx: Context<LockAlphaForGovernance>,
        amount: u64,
        lock_duration_seconds: i64,
    ) -> Result<()> {
        instructions::governance_v1::lock_alpha_for_governance_handler(
            ctx,
            amount,
            lock_duration_seconds,
        )
    }

    pub fn unlock_alpha_from_governance(ctx: Context<UnlockAlphaFromGovernance>) -> Result<()> {
        instructions::governance_v1::unlock_alpha_from_governance_handler(ctx)
    }

    pub fn initialize_governance_snapshot_v1(
        ctx: Context<InitializeGovernanceSnapshotV1>,
    ) -> Result<()> {
        instructions::governance_v1::initialize_governance_snapshot_v1_handler(ctx)
    }

    pub fn create_governance_snapshot_v1(ctx: Context<CreateGovernanceSnapshotV1>) -> Result<()> {
        instructions::governance_v1::create_governance_snapshot_v1_handler(ctx)
    }

    pub fn cast_governance_vote_v1(
        ctx: Context<CastGovernanceVoteV1>,
        choice: VoteChoiceV1,
    ) -> Result<()> {
        instructions::governance_v1::cast_governance_vote_v1_handler(ctx, choice)
    }

    pub fn finalize_governance_vote_v1(ctx: Context<FinalizeGovernanceVoteV1>) -> Result<()> {
        instructions::governance_v1::finalize_governance_vote_v1_handler(ctx)
    }

    pub fn create_governance_decision_adapter_v1(
        ctx: Context<CreateGovernanceDecisionAdapterV1>,
    ) -> Result<()> {
        instructions::governance_adapter_v1::create_governance_decision_adapter_v1_handler(ctx)
    }

    pub fn initialize_vote_record_v1(ctx: Context<InitializeVoteRecordV1>) -> Result<()> {
        instructions::governance_v1::initialize_vote_record_v1_handler(ctx)
    }

    pub fn initialize_contributor_registry_v1(
        ctx: Context<InitializeContributorRegistryV1>,
        role: ContributorRoleV1,
    ) -> Result<()> {
        instructions::contributor_v1::initialize_contributor_registry_v1_handler(ctx, role)
    }

    pub fn initialize_contributor_milestone_v1(
        ctx: Context<InitializeContributorMilestoneV1>,
        milestone_id: u64,
        title: String,
        description: String,
        evidence_hash: [u8; 32],
        requested_amount: u64,
    ) -> Result<()> {
        instructions::contributor_v1::initialize_contributor_milestone_v1_handler(
            ctx,
            milestone_id,
            title,
            description,
            evidence_hash,
            requested_amount,
        )
    }

    pub fn initialize_builder_payout_request_v1(
        ctx: Context<InitializeBuilderPayoutRequestV1>,
        milestone_id: u64,
        amount: u64,
        destination_wallet: Pubkey,
    ) -> Result<()> {
        instructions::contributor_v1::initialize_builder_payout_request_v1_handler(
            ctx,
            milestone_id,
            amount,
            destination_wallet,
        )
    }

    pub fn execute_add_contributor(
        ctx: Context<ExecuteAddContributor>,
        proposal_id: u64,
        contributor_role: ContributorRoleV1,
    ) -> Result<()> {
        instructions::contributor_v1::execute_add_contributor_handler(
            ctx,
            proposal_id,
            contributor_role,
        )
    }

    pub fn execute_remove_contributor(
        ctx: Context<ExecuteRemoveContributor>,
        proposal_id: u64,
        reason_hash: [u8; 32],
    ) -> Result<()> {
        instructions::contributor_v1::execute_remove_contributor_handler(
            ctx,
            proposal_id,
            reason_hash,
        )
    }

    pub fn execute_update_contributor_role(
        ctx: Context<ExecuteUpdateContributorRole>,
        proposal_id: u64,
        new_role: ContributorRoleV1,
    ) -> Result<()> {
        instructions::contributor_v1::execute_update_contributor_role_handler(
            ctx,
            proposal_id,
            new_role,
        )
    }

    pub fn execute_approve_contributor_milestone(
        ctx: Context<ExecuteApproveContributorMilestone>,
        proposal_id: u64,
        milestone_id: u64,
        approved_amount: u64,
    ) -> Result<()> {
        instructions::contributor_v1::execute_approve_contributor_milestone_handler(
            ctx,
            proposal_id,
            milestone_id,
            approved_amount,
        )
    }

    pub fn execute_approve_builder_payout(
        ctx: Context<ExecuteApproveBuilderPayout>,
        proposal_id: u64,
        milestone_id: u64,
        approved_amount: u64,
    ) -> Result<()> {
        instructions::contributor_v1::execute_approve_builder_payout_handler(
            ctx,
            proposal_id,
            milestone_id,
            approved_amount,
        )
    }

    pub fn initialize_staking_pool(
        ctx: Context<InitializeStakingPool>,
        min_claim_usdc: u64,
    ) -> Result<()> {
        instructions::staking_v1::initialize_staking_pool_handler(ctx, min_claim_usdc)
    }

    pub fn stake_alpha(ctx: Context<StakeAlpha>, amount: u64, lock_tier: u8) -> Result<()> {
        instructions::staking_v1::stake_alpha_handler(ctx, amount, lock_tier)
    }

    pub fn claim_usdc_rewards(ctx: Context<ClaimUsdcRewards>) -> Result<()> {
        instructions::staking_v1::claim_usdc_rewards_handler(ctx)
    }

    pub fn unstake_alpha(ctx: Context<UnstakeAlpha>, amount: u64) -> Result<()> {
        instructions::staking_v1::unstake_alpha_handler(ctx, amount)
    }

    pub fn initialize_governance_config(
        ctx: Context<InitializeGovernanceConfig>,
        min_execution_delay_seconds: i64,
        emergency_guardian: Pubkey,
    ) -> Result<()> {
        instructions::security_v1::initialize_governance_config_handler(
            ctx,
            min_execution_delay_seconds,
            emergency_guardian,
        )
    }

    pub fn create_proposal_decision(
        ctx: Context<CreateProposalDecision>,
        expected_proposal_id: u64,
        proposal_type: ProposalType,
        decision: ProposalDecision,
        yes_weight: u64,
        no_weight: u64,
        start_ts: i64,
        end_ts: i64,
    ) -> Result<()> {
        instructions::security_v1::create_proposal_decision_handler(
            ctx,
            expected_proposal_id,
            proposal_type,
            decision,
            yes_weight,
            no_weight,
            start_ts,
            end_ts,
        )
    }

    pub fn queue_execution(
        ctx: Context<QueueExecution>,
        proposal_id: u64,
        action_type: ActionType,
        target_program: Pubkey,
        target_account: Pubkey,
        payload_hash: [u8; 32],
    ) -> Result<()> {
        instructions::security_v1::queue_execution_handler(
            ctx,
            proposal_id,
            action_type,
            target_program,
            target_account,
            payload_hash,
        )
    }

    pub fn execute_queued_action(
        ctx: Context<ExecuteQueuedAction>,
        proposal_id: u64,
        payload_hash: [u8; 32],
    ) -> Result<()> {
        instructions::security_v1::execute_queued_action_handler(ctx, proposal_id, payload_hash)
    }

    pub fn cancel_queued_action(ctx: Context<CancelQueuedAction>, proposal_id: u64) -> Result<()> {
        instructions::security_v1::cancel_queued_action_handler(ctx, proposal_id)
    }

    pub fn pause_security_layer(ctx: Context<PauseSecurityLayer>) -> Result<()> {
        instructions::security_v1::pause_security_layer_handler(ctx)
    }

    pub fn unpause_security_layer(ctx: Context<UnpauseSecurityLayer>) -> Result<()> {
        instructions::security_v1::unpause_security_layer_handler(ctx)
    }

    pub fn initialize_green_label_config(ctx: Context<InitializeGreenLabelConfig>) -> Result<()> {
        instructions::green_label_v1::initialize_green_label_config_handler(ctx)
    }

    pub fn update_green_label_windows(
        ctx: Context<UpdateGreenLabelWindows>,
        observation_period_seconds: i64,
        dispute_window_seconds: i64,
        response_window_seconds: i64,
    ) -> Result<()> {
        instructions::green_label_v1::update_green_label_windows_handler(
            ctx,
            observation_period_seconds,
            dispute_window_seconds,
            response_window_seconds,
        )
    }

    pub fn update_green_label_min_base_bond(
        ctx: Context<UpdateGreenLabelMinBaseBond>,
        min_base_bond_usdc: u64,
    ) -> Result<()> {
        instructions::green_label_v1::update_green_label_min_base_bond_handler(
            ctx,
            min_base_bond_usdc,
        )
    }

    pub fn submit_green_label_application(
        ctx: Context<SubmitGreenLabelApplication>,
        expected_project_id: u64,
        project_name_hash: [u8; 32],
        project_url_hash: [u8; 32],
        project_treasury_wallet: Pubkey,
        total_bond_amount: u64,
    ) -> Result<()> {
        instructions::green_label_v1::submit_green_label_application_handler(
            ctx,
            expected_project_id,
            project_name_hash,
            project_url_hash,
            project_treasury_wallet,
            total_bond_amount,
        )
    }

    pub fn initialize_green_bond_vault(ctx: Context<InitializeGreenBondVault>) -> Result<()> {
        instructions::green_label_v1::initialize_green_bond_vault_handler(ctx)
    }

    pub fn lock_green_label_bond(ctx: Context<LockGreenLabelBond>) -> Result<()> {
        instructions::green_label_v1::lock_green_label_bond_handler(ctx)
    }

    pub fn lock_green_label_bond_with_fee_receipt_v1(
        ctx: Context<LockGreenLabelBondWithFeeReceiptV1>,
    ) -> Result<()> {
        instructions::green_label_v1::lock_green_label_bond_with_fee_receipt_v1_handler(ctx)
    }

    pub fn open_green_label_dispute(
        ctx: Context<OpenGreenLabelDispute>,
        expected_dispute_id: u64,
        reason_code: RugReasonCode,
        evidence_hash: [u8; 32],
    ) -> Result<()> {
        instructions::green_label_v1::open_green_label_dispute_handler(
            ctx,
            expected_dispute_id,
            reason_code,
            evidence_hash,
        )
    }

    pub fn mark_dispute_ready_for_decision(
        ctx: Context<MarkDisputeReadyForDecision>,
    ) -> Result<()> {
        instructions::green_label_v1::mark_dispute_ready_for_decision_handler(ctx)
    }

    pub fn link_green_label_security_decision(
        ctx: Context<LinkGreenLabelSecurityDecision>,
        expected_proposal_id: u64,
        expected_action_type: ActionType,
        expected_payload_hash: [u8; 32],
    ) -> Result<()> {
        instructions::green_label_v1::link_green_label_security_decision_handler(
            ctx,
            expected_proposal_id,
            expected_action_type,
            expected_payload_hash,
        )
    }

    pub fn execute_green_label_refund(ctx: Context<ExecuteGreenLabelRefund>) -> Result<()> {
        instructions::green_label_v1::execute_green_label_refund_handler(ctx)
    }

    pub fn execute_green_label_slash(ctx: Context<ExecuteGreenLabelSlash>) -> Result<()> {
        instructions::green_label_v1::execute_green_label_slash_handler(ctx)
    }

    pub fn initialize_green_label_refundable_escrow_v1(
        ctx: Context<InitializeGreenLabelRefundableEscrowV1>,
        refund_available_after: i64,
    ) -> Result<()> {
        instructions::green_label_v1::initialize_green_label_refundable_escrow_v1_handler(
            ctx,
            refund_available_after,
        )
    }

    pub fn deposit_green_label_refundable_bond_v1(
        ctx: Context<DepositGreenLabelRefundableBondV1>,
        amount: u64,
    ) -> Result<()> {
        instructions::green_label_v1::deposit_green_label_refundable_bond_v1_handler(ctx, amount)
    }

    pub fn initialize_green_label_certification_fee_policy_v1(
        ctx: Context<InitializeGreenLabelCertificationFeePolicyV1>,
        fee_amount_usdc: u64,
    ) -> Result<()> {
        instructions::green_label_v1::initialize_green_label_certification_fee_policy_v1_handler(
            ctx,
            fee_amount_usdc,
        )
    }

    pub fn route_green_label_certification_fee_v1(
        ctx: Context<RouteGreenLabelCertificationFeeV1>,
        amount: u64,
    ) -> Result<()> {
        instructions::green_label_v1::route_green_label_certification_fee_v1_handler(ctx, amount)
    }

    pub fn route_green_label_certification_fee_once_v1(
        ctx: Context<RouteGreenLabelCertificationFeeOnceV1>,
    ) -> Result<()> {
        instructions::green_label_v1::route_green_label_certification_fee_once_v1_handler(ctx)
    }

    pub fn initialize_green_label_certification_state_v1(
        ctx: Context<InitializeGreenLabelCertificationStateV1>,
    ) -> Result<()> {
        instructions::green_label_v1::initialize_green_label_certification_state_v1_handler(ctx)
    }

    pub fn execute_green_label_approve_certification_v1(
        ctx: Context<ExecuteGreenLabelApproveCertificationV1>,
    ) -> Result<()> {
        instructions::green_label_v1::execute_green_label_approve_certification_v1_handler(ctx)
    }

    pub fn execute_green_label_reject_certification_v1(
        ctx: Context<ExecuteGreenLabelRejectCertificationV1>,
    ) -> Result<()> {
        instructions::green_label_v1::execute_green_label_reject_certification_v1_handler(ctx)
    }

    pub fn execute_green_label_revoke_certification_v1(
        ctx: Context<ExecuteGreenLabelRevokeCertificationV1>,
    ) -> Result<()> {
        instructions::green_label_v1::execute_green_label_revoke_certification_v1_handler(ctx)
    }

    pub fn execute_green_label_refund_no_dispute_governance_v1(
        ctx: Context<ExecuteGreenLabelRefundNoDisputeGovernanceV1>,
    ) -> Result<()> {
        instructions::green_label_v1::execute_green_label_refund_no_dispute_governance_v1_handler(
            ctx,
        )
    }

    pub fn execute_green_label_refund_dispute_governance_v1(
        ctx: Context<ExecuteGreenLabelRefundDisputeGovernanceV1>,
    ) -> Result<()> {
        instructions::green_label_v1::execute_green_label_refund_dispute_governance_v1_handler(ctx)
    }

    pub fn refund_green_label_escrow_v1(ctx: Context<RefundGreenLabelEscrowV1>) -> Result<()> {
        instructions::green_label_v1::refund_green_label_escrow_v1_handler(ctx)
    }

    pub fn forfeit_green_label_escrow_to_treasury_v1<'info>(
        ctx: Context<'info, ForfeitGreenLabelEscrowToTreasuryV1<'info>>,
    ) -> Result<()> {
        instructions::green_label_v1::forfeit_green_label_escrow_to_treasury_v1_handler(ctx)
    }

    pub fn execute_green_label_forfeit_governance_v1<'info>(
        ctx: Context<'info, ExecuteGreenLabelForfeitGovernanceV1<'info>>,
    ) -> Result<()> {
        instructions::green_label_v1::execute_green_label_forfeit_governance_v1_handler(ctx)
    }
}
