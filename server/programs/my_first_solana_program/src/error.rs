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
}
