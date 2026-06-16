use anchor_lang::prelude::*;

use crate::constants::{
    BPS_DENOMINATOR, BUYBACK_BPS, PAYROLL_BPS, RELIEF_BPS, STAKING_BPS, TREASURY_STATE_SEED,
};
use crate::error::CustomError;
use crate::state::TreasuryState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreasurySplit {
    pub relief: u64,
    pub buyback: u64,
    pub payroll: u64,
    pub staking: u64,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        seeds = [TREASURY_STATE_SEED],
        bump = treasury_state.bump
    )]
    pub treasury_state: Account<'info, TreasuryState>,
}

pub fn deposit_handler(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    let split = calculate_treasury_split(amount)?;

    let treasury_state = &mut ctx.accounts.treasury_state;

    treasury_state.total_inflow = treasury_state
        .total_inflow
        .checked_add(amount)
        .ok_or(CustomError::MathOverflow)?;

    treasury_state.relief_pool = treasury_state
        .relief_pool
        .checked_add(split.relief)
        .ok_or(CustomError::MathOverflow)?;

    treasury_state.buyback_pool = treasury_state
        .buyback_pool
        .checked_add(split.buyback)
        .ok_or(CustomError::MathOverflow)?;

    treasury_state.payroll_pool = treasury_state
        .payroll_pool
        .checked_add(split.payroll)
        .ok_or(CustomError::MathOverflow)?;

    treasury_state.staking_pool = treasury_state
        .staking_pool
        .checked_add(split.staking)
        .ok_or(CustomError::MathOverflow)?;

    Ok(())
}

pub fn calculate_treasury_split(amount: u64) -> Result<TreasurySplit> {
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

    Ok(TreasurySplit {
        relief,
        buyback,
        payroll,
        staking,
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_expected_split_for_10000() {
        let split = calculate_treasury_split(10_000).unwrap();

        assert_eq!(split.relief, 5_000);
        assert_eq!(split.buyback, 2_000);
        assert_eq!(split.payroll, 2_000);
        assert_eq!(split.staking, 1_000);

        assert_eq!(
            split.relief + split.buyback + split.payroll + split.staking,
            10_000
        );
    }

    #[test]
    fn calculates_small_amount_without_losing_total() {
        let split = calculate_treasury_split(1).unwrap();

        assert_eq!(
            split.relief + split.buyback + split.payroll + split.staking,
            1
        );
    }

    #[test]
    fn rejects_zero_amount() {
        let err = calculate_treasury_split(0).unwrap_err();
        let message = format!("{err:?}");

        assert!(
            message.contains("InvalidSplitConfig") || message.contains("Invalid split config"),
            "unexpected error: {message}"
        );
    }

    #[test]
    fn rejects_overflowing_amount() {
        let err = calculate_treasury_split(u64::MAX).unwrap_err();
        let message = format!("{err:?}");

        assert!(
            message.contains("MathOverflow") || message.contains("Math overflow"),
            "unexpected error: {message}"
        );
    }
}
