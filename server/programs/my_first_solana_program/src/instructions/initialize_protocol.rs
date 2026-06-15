use anchor_lang::prelude::*;

use crate::constants::TREASURY_STATE_SEED;
use crate::state::TreasuryState;

#[derive(Accounts)]
pub struct InitializeProtocol<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + TreasuryState::INIT_SPACE,
        seeds = [TREASURY_STATE_SEED],
        bump
    )]
    pub treasury_state: Account<'info, TreasuryState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeProtocol>) -> Result<()> {
    let treasury_state = &mut ctx.accounts.treasury_state;

    treasury_state.authority = ctx.accounts.authority.key();
    treasury_state.total_inflow = 0;
    treasury_state.relief_pool = 0;
    treasury_state.buyback_pool = 0;
    treasury_state.payroll_pool = 0;
    treasury_state.staking_pool = 0;
    treasury_state.bump = ctx.bumps.treasury_state;

    Ok(())
}
