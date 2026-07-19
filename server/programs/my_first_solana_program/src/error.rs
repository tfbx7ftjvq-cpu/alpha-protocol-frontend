use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Math overflow")]
    MathOverflow,

    #[msg("Invalid split config")]
    InvalidSplitConfig,

    #[msg("Invalid amount")]
    InvalidAmount,

    #[msg("Invalid mint")]
    InvalidMint,

    #[msg("Invalid vault")]
    InvalidVault,

    #[msg("Invalid token account")]
    InvalidTokenAccount,

    #[msg("Invalid lock tier")]
    InvalidLockTier,

    #[msg("Lock period has not ended")]
    LockPeriodNotEnded,

    #[msg("Stake account owner mismatch")]
    InvalidStakeOwner,

    #[msg("Stake lock tier mismatch")]
    StakeLockTierMismatch,

    #[msg("Claim amount is below the minimum claim threshold")]
    ClaimAmountTooSmall,

    #[msg("Vault balance is insufficient")]
    InsufficientVaultBalance,

    #[msg("Vault balance is below the observed accounting balance")]
    VaultBalanceBelowObserved,

    #[msg("Invalid execution delay")]
    InvalidExecutionDelay,

    #[msg("Invalid proposal id")]
    InvalidProposalId,

    #[msg("Invalid proposal decision")]
    InvalidProposalDecision,

    #[msg("Invalid action for proposal type")]
    InvalidActionForProposalType,

    #[msg("Proposal is not approved for execution")]
    ProposalNotApproved,

    #[msg("Security layer is paused")]
    SecurityLayerPaused,

    #[msg("Execution delay has not been met")]
    ExecutionDelayNotMet,

    #[msg("Invalid execution status")]
    InvalidExecutionStatus,

    #[msg("Payload hash mismatch")]
    PayloadHashMismatch,

    #[msg("Unauthorized security authority")]
    UnauthorizedSecurityAuthority,

    #[msg("Unauthorized emergency guardian")]
    UnauthorizedEmergencyGuardian,

    #[msg("Invalid emergency guardian")]
    InvalidEmergencyGuardian,

    #[msg("Invalid proposal time")]
    InvalidProposalTime,

    #[msg("Invalid Green Label bond amount")]
    InvalidGreenLabelBondAmount,

    #[msg("Invalid Green Label BPS config")]
    InvalidGreenLabelBpsConfig,

    #[msg("Invalid Green Label status")]
    InvalidGreenLabelStatus,

    #[msg("Invalid Green Label bond split")]
    InvalidGreenLabelBondSplit,

    #[msg("Invalid Green Label slash without dispute")]
    InvalidGreenLabelSlashWithoutDispute,

    #[msg("Invalid Green Label payload hash")]
    InvalidGreenLabelPayloadHash,

    #[msg("Green Label math overflow")]
    GreenLabelMathOverflow,

    #[msg("Invalid Green Label action type")]
    InvalidGreenLabelActionType,

    #[msg("Invalid Green Label project id")]
    InvalidGreenLabelProjectId,

    #[msg("Invalid Green Label project owner")]
    InvalidGreenLabelProjectOwner,

    #[msg("Invalid Green Label mint")]
    InvalidGreenLabelMint,

    #[msg("Invalid Green Label bond vault state")]
    InvalidGreenLabelBondVaultState,

    #[msg("Invalid Green Label token account")]
    InvalidGreenLabelTokenAccount,

    #[msg("Invalid Green Label dispute id")]
    InvalidGreenLabelDisputeId,

    #[msg("Invalid Green Label active dispute")]
    InvalidGreenLabelActiveDispute,

    #[msg("Invalid Green Label evidence hash")]
    InvalidGreenLabelEvidenceHash,

    #[msg("Invalid Green Label dispute status")]
    InvalidGreenLabelDisputeStatus,

    #[msg("Green Label dispute window has not ended")]
    GreenLabelDisputeWindowNotEnded,

    #[msg("Invalid Green Label security decision")]
    InvalidGreenLabelSecurityDecision,

    #[msg("Invalid Green Label execution queue")]
    InvalidGreenLabelExecutionQueue,

    #[msg("Invalid Green Label target program")]
    InvalidGreenLabelTargetProgram,

    #[msg("Invalid Green Label target account")]
    InvalidGreenLabelTargetAccount,

    #[msg("Green Label timelock has not been satisfied")]
    GreenLabelTimelockNotSatisfied,

    #[msg("Green Label bond vault balance is insufficient")]
    GreenLabelInsufficientBondVaultBalance,

    #[msg("Unauthorized Green Label authority")]
    UnauthorizedGreenLabelAuthority,

    #[msg("Invalid Green Label window config")]
    InvalidGreenLabelWindowConfig,

    #[msg("Unauthorized treasury authority")]
    UnauthorizedTreasuryAuthority,

    #[msg("Invalid Green Label escrow status")]
    InvalidGreenLabelEscrowStatus,

    #[msg("Invalid Green Label escrow amount")]
    InvalidGreenLabelEscrowAmount,

    #[msg("Invalid Green Label escrow refund")]
    InvalidGreenLabelEscrowRefund,

    #[msg("Invalid Green Label escrow forfeit")]
    InvalidGreenLabelEscrowForfeit,

    #[msg("Unauthorized contributor wallet")]
    UnauthorizedContributorWallet,

    #[msg("Invalid contributor status")]
    InvalidContributorStatus,

    #[msg("Invalid contributor role")]
    InvalidContributorRole,

    #[msg("Invalid contributor milestone")]
    InvalidContributorMilestone,

    #[msg("Invalid contributor milestone text")]
    InvalidContributorMilestoneText,

    #[msg("Invalid contributor milestone amount")]
    InvalidContributorMilestoneAmount,

    #[msg("Invalid contributor payout request")]
    InvalidContributorPayoutRequest,

    #[msg("Invalid contributor payout amount")]
    InvalidContributorPayoutAmount,

    #[msg("Invalid contributor payout destination")]
    InvalidContributorPayoutDestination,

    #[msg("Invalid contributor security execution")]
    InvalidContributorSecurityExecution,

    #[msg("Invalid contributor payload hash")]
    InvalidContributorPayloadHash,

    #[msg("Invalid governance proposal")]
    InvalidGovernanceProposal,

    #[msg("Invalid governance proposal time")]
    InvalidGovernanceProposalTime,

    #[msg("Invalid governance position")]
    InvalidGovernancePosition,

    #[msg("Unauthorized governance position owner")]
    UnauthorizedGovernancePositionOwner,

    #[msg("Invalid governance lock config")]
    InvalidGovernanceLockConfig,

    #[msg("Invalid governance lock duration")]
    InvalidGovernanceLockDuration,

    #[msg("Invalid governance lock amount")]
    InvalidGovernanceLockAmount,

    #[msg("Invalid governance vault")]
    InvalidGovernanceVault,

    #[msg("Governance lock is still active")]
    GovernanceLockStillActive,

    #[msg("Invalid governance voting config")]
    InvalidGovernanceVotingConfig,

    #[msg("Proposal is not voting")]
    ProposalNotVoting,

    #[msg("Vote record already exists")]
    AlreadyVoted,

    #[msg("Voting period has ended")]
    VotingPeriodEnded,

    #[msg("Voting period has not ended")]
    VotingPeriodNotEnded,

    #[msg("Governance snapshot is missing")]
    SnapshotMissing,

    #[msg("Quorum not reached")]
    QuorumNotReached,

    #[msg("Proposal already finalized")]
    ProposalAlreadyFinalized,

    #[msg("Invalid governance snapshot")]
    InvalidGovernanceSnapshot,

    #[msg("Invalid governance vote")]
    InvalidGovernanceVote,

    #[msg("Invalid governance decision adapter")]
    InvalidGovernanceDecisionAdapter,

    #[msg("Invalid governance power state")]
    InvalidGovernancePowerState,

    #[msg("Invalid governance vote lock")]
    InvalidGovernanceVoteLock,

    #[msg("Governance position is locked by an active vote")]
    GovernanceVoteLockActive,

    #[msg("Invalid treasury governance config")]
    InvalidTreasuryGovernanceConfig,

    #[msg("Invalid treasury governance request")]
    InvalidTreasuryGovernanceRequest,

    #[msg("Invalid treasury spending status")]
    InvalidTreasurySpendingStatus,

    #[msg("Invalid treasury builder payout status")]
    InvalidTreasuryBuilderPayoutStatus,

    #[msg("Invalid treasury governance payload hash")]
    InvalidTreasuryGovernancePayloadHash,

    #[msg("Governance proposal action sidecar is missing")]
    GovernanceProposalActionMissing,

    #[msg("Governance proposal action sidecar does not match proposal")]
    GovernanceProposalActionMismatch,

    #[msg("Invalid governance action stable code")]
    InvalidGovernanceActionCode,

    #[msg("Invalid governance payload schema")]
    InvalidGovernancePayloadSchema,

    #[msg("Governance action module does not match action type")]
    GovernanceActionModuleMismatch,

    #[msg("Governance action target does not match policy")]
    GovernanceActionTargetMismatch,

    #[msg("Invalid protocol module stable code")]
    InvalidProtocolModuleCode,

    #[msg("Protocol module registry does not match expected module")]
    ProtocolModuleRegistryMismatch,

    #[msg("Protocol module registry is disabled")]
    ProtocolModuleDisabled,

    #[msg("Protocol module registry program does not match expected program")]
    ProtocolModuleProgramMismatch,

    #[msg("Protocol module registry governance config does not match")]
    ProtocolModuleGovernanceConfigMismatch,

    #[msg("Invalid protocol module registry schema")]
    InvalidProtocolModuleRegistrySchema,

    #[msg("Treasury execution already completed")]
    TreasuryExecutionAlreadyCompleted,

    #[msg("Treasury execution action mismatch")]
    TreasuryExecutionActionMismatch,

    #[msg("Treasury execution target mismatch")]
    TreasuryExecutionTargetMismatch,

    #[msg("Treasury execution parameters mismatch")]
    TreasuryExecutionParametersMismatch,

    #[msg("Treasury execution vault mismatch")]
    TreasuryExecutionVaultMismatch,

    #[msg("Treasury execution recipient mismatch")]
    TreasuryExecutionRecipientMismatch,

    #[msg("Treasury execution mint mismatch")]
    TreasuryExecutionMintMismatch,

    #[msg("Treasury execution amount mismatch")]
    TreasuryExecutionAmountMismatch,

    #[msg("Treasury execution insufficient funds")]
    TreasuryExecutionInsufficientFunds,

    #[msg("Treasury execution is disabled")]
    TreasuryExecutionDisabled,

    #[msg("Treasury emergency mode is active")]
    TreasuryEmergencyModeActive,

    #[msg("Treasury spending limit exceeded")]
    TreasurySpendingLimitExceeded,

    #[msg("Invalid treasury execution schema")]
    InvalidTreasuryExecutionSchema,

    #[msg("Green Label certification state mismatch")]
    GreenLabelCertificationStateMismatch,

    #[msg("Green Label certification is already finalized")]
    GreenLabelCertificationAlreadyFinalized,

    #[msg("Green Label certification is not approved")]
    GreenLabelCertificationNotApproved,

    #[msg("Green Label observation period is not complete")]
    GreenLabelObservationPeriodNotComplete,

    #[msg("Green Label has an unresolved dispute")]
    GreenLabelUnresolvedDispute,

    #[msg("Green Label certification action mismatch")]
    GreenLabelCertificationActionMismatch,

    #[msg("Green Label certification target mismatch")]
    GreenLabelCertificationTargetMismatch,

    #[msg("Green Label certification parameters mismatch")]
    GreenLabelCertificationParametersMismatch,

    #[msg("Green Label certification execution already completed")]
    GreenLabelCertificationExecutionAlreadyCompleted,

    #[msg("Invalid Green Label certification schema")]
    InvalidGreenLabelCertificationSchema,

    #[msg("Green Label refund action mismatch")]
    GreenLabelRefundActionMismatch,

    #[msg("Green Label refund target mismatch")]
    GreenLabelRefundTargetMismatch,

    #[msg("Green Label refund parameters mismatch")]
    GreenLabelRefundParametersMismatch,

    #[msg("Green Label refund execution already completed")]
    GreenLabelRefundExecutionAlreadyCompleted,

    #[msg("Green Label refund is not eligible")]
    GreenLabelRefundNotEligible,

    #[msg("Green Label refund payer mismatch")]
    GreenLabelRefundWrongPayer,

    #[msg("Green Label refund destination mismatch")]
    GreenLabelRefundWrongDestination,

    #[msg("Green Label refund vault mismatch")]
    GreenLabelRefundVaultMismatch,

    #[msg("Green Label refund mint mismatch")]
    GreenLabelRefundMintMismatch,

    #[msg("Green Label refund amount mismatch")]
    GreenLabelRefundAmountMismatch,

    #[msg("Green Label refund has insufficient funds")]
    GreenLabelRefundInsufficientFunds,

    #[msg("Green Label escrow is already refunded")]
    GreenLabelEscrowAlreadyRefunded,

    #[msg("Green Label escrow is already forfeited")]
    GreenLabelEscrowAlreadyForfeited,

    #[msg("Invalid Green Label refund schema")]
    InvalidGreenLabelRefundSchema,

    #[msg("Green Label forfeit action mismatch")]
    GreenLabelForfeitActionMismatch,

    #[msg("Green Label forfeit target mismatch")]
    GreenLabelForfeitTargetMismatch,

    #[msg("Green Label forfeit parameters mismatch")]
    GreenLabelForfeitParametersMismatch,

    #[msg("Green Label forfeit execution already completed")]
    GreenLabelForfeitExecutionAlreadyCompleted,

    #[msg("Green Label forfeit is not eligible")]
    GreenLabelForfeitNotEligible,

    #[msg("Green Label forfeit dispute mismatch")]
    GreenLabelForfeitDisputeMismatch,

    #[msg("Green Label forfeit decision mismatch")]
    GreenLabelForfeitDecisionMismatch,

    #[msg("Green Label forfeit vault mismatch")]
    GreenLabelForfeitVaultMismatch,

    #[msg("Green Label forfeit mint mismatch")]
    GreenLabelForfeitMintMismatch,

    #[msg("Green Label forfeit amount mismatch")]
    GreenLabelForfeitAmountMismatch,

    #[msg("Green Label forfeit has insufficient funds")]
    GreenLabelForfeitInsufficientFunds,

    #[msg("Invalid Green Label forfeit schema")]
    InvalidGreenLabelForfeitSchema,

    #[msg("Legacy Green Label slash is disabled")]
    LegacyGreenLabelSlashDisabled,

    #[msg("Legacy Green Label forfeit is disabled")]
    LegacyGreenLabelForfeitDisabled,

    #[msg("Green Label certification fee policy is already initialized")]
    GreenLabelCertificationFeePolicyAlreadyInitialized,

    #[msg("Invalid Green Label certification fee policy schema")]
    InvalidGreenLabelCertificationFeePolicySchema,

    #[msg("Green Label certification fee policy is inactive")]
    GreenLabelCertificationFeePolicyInactive,

    #[msg("Invalid Green Label certification fee amount")]
    InvalidGreenLabelCertificationFeeAmount,

    #[msg("Green Label certification fee project mismatch")]
    GreenLabelCertificationFeeProjectMismatch,

    #[msg("Green Label certification fee payer mismatch")]
    GreenLabelCertificationFeePayerMismatch,

    #[msg("Green Label certification fee status mismatch")]
    GreenLabelCertificationFeeStatusMismatch,

    #[msg("Green Label certification fee mint mismatch")]
    GreenLabelCertificationFeeMintMismatch,

    #[msg("Green Label certification fee decimals mismatch")]
    GreenLabelCertificationFeeDecimalsMismatch,

    #[msg("Green Label certification fee treasury mismatch")]
    GreenLabelCertificationFeeTreasuryMismatch,

    #[msg("Green Label certification fee parameters mismatch")]
    GreenLabelCertificationFeeParametersMismatch,

    #[msg("Green Label certification fee is already paid")]
    GreenLabelCertificationFeeAlreadyPaid,

    #[msg("Legacy Green Label certification fee route is disabled")]
    LegacyGreenLabelCertificationFeeRouteDisabled,

    #[msg("Green Label certification fee has insufficient funds")]
    GreenLabelCertificationFeeInsufficientFunds,

    #[msg("Green Label certification fee receipt is missing")]
    GreenLabelCertificationFeeReceiptMissing,

    #[msg("Green Label certification fee receipt mismatch")]
    GreenLabelCertificationFeeReceiptMismatch,

    #[msg("Green Label certification fee receipt project mismatch")]
    GreenLabelCertificationFeeReceiptProjectMismatch,

    #[msg("Green Label certification fee receipt policy mismatch")]
    GreenLabelCertificationFeeReceiptPolicyMismatch,

    #[msg("Green Label certification fee receipt amount mismatch")]
    GreenLabelCertificationFeeReceiptAmountMismatch,

    #[msg("Green Label certification fee receipt hash mismatch")]
    GreenLabelCertificationFeeReceiptHashMismatch,

    #[msg("Green Label certification fee receipt revenue type mismatch")]
    GreenLabelCertificationFeeReceiptRevenueTypeMismatch,

    #[msg("Legacy Green Label bond lock without certification fee receipt is disabled")]
    LegacyGreenLabelBondLockWithoutFeeReceiptDisabled,

    #[msg("Victim Relief is paused")]
    VictimReliefPaused,

    #[msg("Invalid Victim Relief config")]
    InvalidVictimReliefConfig,

    #[msg("Invalid Victim Relief policy")]
    InvalidVictimReliefPolicy,

    #[msg("Victim Relief policy is already initialized")]
    VictimReliefPolicyAlreadyInitialized,

    #[msg("Invalid Victim Relief policy version")]
    InvalidVictimReliefPolicyVersion,

    #[msg("Invalid Victim Relief claim amount")]
    InvalidVictimReliefClaimAmount,

    #[msg("Invalid Victim Relief subject commitment")]
    InvalidVictimReliefSubjectCommitment,

    #[msg("Invalid Victim Relief evidence root")]
    InvalidVictimReliefEvidenceRoot,

    #[msg("Invalid Victim Relief evidence count")]
    InvalidVictimReliefEvidenceCount,

    #[msg("Victim Relief active case limit reached")]
    VictimReliefActiveCaseLimitReached,

    #[msg("Victim Relief submission cooldown is active")]
    VictimReliefSubmissionCooldownActive,

    #[msg("Invalid Victim Relief case id")]
    InvalidVictimReliefCaseId,

    #[msg("Victim Relief case status mismatch")]
    VictimReliefCaseStatusMismatch,

    #[msg("Victim Relief claimant mismatch")]
    VictimReliefClaimantMismatch,

    #[msg("Victim Relief recipient mismatch")]
    VictimReliefRecipientMismatch,

    #[msg("Victim Relief evidence window is closed")]
    VictimReliefEvidenceWindowClosed,

    #[msg("Victim Relief evidence is unchanged")]
    VictimReliefEvidenceUnchanged,

    #[msg("Victim Relief case is not expired")]
    VictimReliefCaseNotExpired,

    #[msg("Victim Relief active case count underflow")]
    VictimReliefActiveCaseCountUnderflow,

    #[msg("Victim Relief evidence is already frozen")]
    VictimReliefEvidenceAlreadyFrozen,

    #[msg("Victim Relief evidence snapshot mismatch")]
    VictimReliefEvidenceSnapshotMismatch,

    #[msg("Victim Relief evidence freeze is too late")]
    VictimReliefEvidenceFreezeTooLate,

    #[msg("Victim Relief decision action mismatch")]
    VictimReliefDecisionActionMismatch,

    #[msg("Victim Relief decision target mismatch")]
    VictimReliefDecisionTargetMismatch,

    #[msg("Victim Relief decision parameters mismatch")]
    VictimReliefDecisionParametersMismatch,

    #[msg("Victim Relief decision execution already completed")]
    VictimReliefDecisionExecutionAlreadyCompleted,

    #[msg("Victim Relief approved amount mismatch")]
    VictimReliefApprovedAmountMismatch,

    #[msg("Victim Relief payout request already exists")]
    VictimReliefPayoutRequestAlreadyExists,

    #[msg("Victim Relief relief vault mismatch")]
    VictimReliefReliefVaultMismatch,

    #[msg("Victim Relief decision is not eligible")]
    VictimReliefDecisionNotEligible,

    #[msg("Invalid Victim Relief decision schema")]
    InvalidVictimReliefDecisionSchema,

    #[msg("Victim Relief appeal already exists")]
    VictimReliefAppealAlreadyExists,

    #[msg("Victim Relief appeal window is closed")]
    VictimReliefAppealWindowClosed,

    #[msg("Victim Relief appeal evidence mismatch")]
    VictimReliefAppealEvidenceMismatch,

    #[msg("Victim Relief appeal original decision mismatch")]
    VictimReliefAppealOriginalDecisionMismatch,

    #[msg("Victim Relief appeal action mismatch")]
    VictimReliefAppealActionMismatch,

    #[msg("Victim Relief appeal target mismatch")]
    VictimReliefAppealTargetMismatch,

    #[msg("Victim Relief appeal parameters mismatch")]
    VictimReliefAppealParametersMismatch,

    #[msg("Victim Relief appeal status mismatch")]
    VictimReliefAppealStatusMismatch,

    #[msg("Victim Relief appeal execution already completed")]
    VictimReliefAppealExecutionAlreadyCompleted,

    #[msg("Victim Relief appeal is not eligible")]
    VictimReliefAppealNotEligible,

    #[msg("Victim Relief appeal payout request already exists")]
    VictimReliefAppealPayoutRequestAlreadyExists,

    #[msg("Invalid Victim Relief appeal schema")]
    InvalidVictimReliefAppealSchema,

    #[msg("Invalid Victim Relief payout origin")]
    InvalidVictimReliefPayoutOrigin,

    #[msg("Victim Relief payout request mismatch")]
    VictimReliefPayoutRequestMismatch,

    #[msg("Victim Relief payout status mismatch")]
    VictimReliefPayoutStatusMismatch,

    #[msg("Victim Relief payout authorization mismatch")]
    VictimReliefPayoutAuthorizationMismatch,

    #[msg("Victim Relief payout action mismatch")]
    VictimReliefPayoutActionMismatch,

    #[msg("Victim Relief payout receipt already exists")]
    VictimReliefPayoutReceiptAlreadyExists,

    #[msg("Victim Relief payout parameters mismatch")]
    VictimReliefPayoutParametersMismatch,

    #[msg("Victim Relief payout recipient mismatch")]
    VictimReliefPayoutRecipientMismatch,

    #[msg("Victim Relief payout vault mismatch")]
    VictimReliefPayoutVaultMismatch,

    #[msg("Victim Relief payout mint mismatch")]
    VictimReliefPayoutMintMismatch,

    #[msg("Victim Relief payout decimals mismatch")]
    VictimReliefPayoutDecimalsMismatch,

    #[msg("Victim Relief payout insufficient funds")]
    VictimReliefPayoutInsufficientFunds,

    #[msg("Victim Relief payout claimant state mismatch")]
    VictimReliefPayoutClaimantStateMismatch,

    #[msg("Victim Relief payout is paused")]
    VictimReliefPayoutPaused,

    #[msg("Invalid Victim Relief payout schema")]
    InvalidVictimReliefPayoutSchema,
}
