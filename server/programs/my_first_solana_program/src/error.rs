use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
    #[msg("Math operation overflowed")]
    MathOverflow,
    #[msg("Invalid treasury split configuration")]
    InvalidSplitConfig,
}
