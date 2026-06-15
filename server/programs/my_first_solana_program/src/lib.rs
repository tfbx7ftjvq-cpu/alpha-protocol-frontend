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
}
