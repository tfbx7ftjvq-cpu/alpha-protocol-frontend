use anchor_lang::prelude::*;

use crate::constants::{
    BPS_DENOMINATOR, BUYBACK_BPS, PAYROLL_BPS, RELIEF_BPS, STAKING_BPS, TREASURY_STATE_SEED,
};
use crate::error::CustomError;
use crate::state::TreasuryState;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        seeds = [TREASURY_STATE_SEED],
        bump = treasury_state.bump
    )]
    pub treasury_state: Account<'info, TreasuryState>,
}

pub fn handler(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    require!(amount > 0, CustomError::InvalidSplitConfig);

    let configured_bps = RELIEF_BPS
        .checked_add(BUYBACK_BPS)
        .and_then(|value| value.checked_add(PAYROLL_BPS))
        .and_then(|value| value.checked_add(STAKING_BPS))
        .ok_or(CustomError::MathOverflow)?;

    require!(
        configured_bps == BPS_DENOMINATOR,
        CustomError::InvalidSplitConfig
    );

    let relief = split_amount(amount, RELIEF_BPS)?;
    let buyback = split_amount(amount, BUYBACK_BPS)?;
    let payroll = split_amount(amount, PAYROLL_BPS)?;

    let staking = amount
        .checked_sub(relief)
        .and_then(|value| value.checked_sub(buyback))
        .and_then(|value| value.checked_sub(payroll))
        .ok_or(CustomError::MathOverflow)?;

    let treasury_state = &mut ctx.accounts.treasury_state;

    treasury_state.total_inflow = treasury_state
        .total_inflow
        .checked_add(amount)
        .ok_or(CustomError::MathOverflow)?;

    treasury_state.relief_pool = treasury_state
        .relief_pool
        .checked_add(relief)
        .ok_or(CustomError::MathOverflow)?;

    treasury_state.buyback_pool = treasury_state
        .buyback_pool
        .checked_add(buyback)
        .ok_or(CustomError::MathOverflow)?;

    treasury_state.payroll_pool = treasury_state
        .payroll_pool
        .checked_add(payroll)
        .ok_or(CustomError::MathOverflow)?;

    treasury_state.staking_pool = treasury_state
        .staking_pool
        .checked_add(staking)
        .ok_or(CustomError::MathOverflow)?;

    Ok(())
}

fn split_amount(amount: u64, bps: u64) -> Result<u64> {
    let multiplied = amount
        .checked_mul(bps)
        .ok_or(CustomError::MathOverflow)?;

    let divided = multiplied
        .checked_div(BPS_DENOMINATOR)
        .ok_or(CustomError::MathOverflow)?;

    Ok(divided)
}
