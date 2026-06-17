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
}
