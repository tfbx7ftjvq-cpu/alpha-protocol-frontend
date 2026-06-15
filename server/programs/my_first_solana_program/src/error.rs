use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Math overflow")]
    MathOverflow,

    #[msg("Invalid split config")]
    InvalidSplitConfig,
}