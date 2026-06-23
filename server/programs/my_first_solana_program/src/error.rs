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
}
