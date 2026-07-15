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
}
