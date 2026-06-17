use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};

use crate::constants::{
    BPS_DENOMINATOR, BUILDERS_USDC_VAULT_SEED, BUYBACK_BPS, BUYBACK_USDC_VAULT_SEED, PAYROLL_BPS,
    RELIEF_BPS, RELIEF_USDC_VAULT_SEED, STAKING_BPS, STAKING_USDC_VAULT_SEED,
    TREASURY_CONFIG_V2_SEED, TREASURY_USDC_STATE_V2_SEED, VAULT_AUTHORITY_V2_SEED,
};
use crate::error::CustomError;
use crate::state::{TreasuryConfigV2, TreasuryUsdcStateV2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsdcTreasurySplit {
    pub relief: u64,
    pub buyback: u64,
    pub builders: u64,
    pub staking: u64,
}

#[derive(Accounts)]
pub struct DepositUsdcRevenue<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    #[account(
        mut,
        constraint = depositor_usdc_token_account.mint == treasury_config.usdc_mint @ CustomError::InvalidMint,
        constraint = depositor_usdc_token_account.owner == depositor.key() @ CustomError::InvalidTokenAccount
    )]
    pub depositor_usdc_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [TREASURY_CONFIG_V2_SEED],
        bump = treasury_config.bump
    )]
    pub treasury_config: Account<'info, TreasuryConfigV2>,

    #[account(
        mut,
        seeds = [TREASURY_USDC_STATE_V2_SEED],
        bump = treasury_usdc_state.bump
    )]
    pub treasury_usdc_state: Account<'info, TreasuryUsdcStateV2>,

    #[account(
        constraint = usdc_mint.key() == treasury_config.usdc_mint @ CustomError::InvalidMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    /// CHECK: This PDA only owns the USDC vault token accounts.
    #[account(
        seeds = [VAULT_AUTHORITY_V2_SEED],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [RELIEF_USDC_VAULT_SEED],
        bump,
        constraint = relief_usdc_vault.mint == treasury_config.usdc_mint @ CustomError::InvalidMint,
        constraint = relief_usdc_vault.owner == vault_authority.key() @ CustomError::InvalidVault
    )]
    pub relief_usdc_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [BUYBACK_USDC_VAULT_SEED],
        bump,
        constraint = buyback_usdc_vault.mint == treasury_config.usdc_mint @ CustomError::InvalidMint,
        constraint = buyback_usdc_vault.owner == vault_authority.key() @ CustomError::InvalidVault
    )]
    pub buyback_usdc_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [BUILDERS_USDC_VAULT_SEED],
        bump,
        constraint = builders_usdc_vault.mint == treasury_config.usdc_mint @ CustomError::InvalidMint,
        constraint = builders_usdc_vault.owner == vault_authority.key() @ CustomError::InvalidVault
    )]
    pub builders_usdc_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [STAKING_USDC_VAULT_SEED],
        bump,
        constraint = staking_usdc_vault.mint == treasury_config.usdc_mint @ CustomError::InvalidMint,
        constraint = staking_usdc_vault.owner == vault_authority.key() @ CustomError::InvalidVault
    )]
    pub staking_usdc_vault: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

pub fn deposit_usdc_revenue_handler(ctx: Context<DepositUsdcRevenue>, amount: u64) -> Result<()> {
    require!(amount > 0, CustomError::InvalidAmount);

    let split = calculate_usdc_treasury_split(amount)?;

    let new_total_usdc_inflow = ctx
        .accounts
        .treasury_usdc_state
        .total_usdc_inflow
        .checked_add(amount)
        .ok_or(CustomError::MathOverflow)?;
    let new_relief_usdc_total = ctx
        .accounts
        .treasury_usdc_state
        .relief_usdc_total
        .checked_add(split.relief)
        .ok_or(CustomError::MathOverflow)?;
    let new_buyback_usdc_total = ctx
        .accounts
        .treasury_usdc_state
        .buyback_usdc_total
        .checked_add(split.buyback)
        .ok_or(CustomError::MathOverflow)?;
    let new_builders_usdc_total = ctx
        .accounts
        .treasury_usdc_state
        .builders_usdc_total
        .checked_add(split.builders)
        .ok_or(CustomError::MathOverflow)?;
    let new_staking_usdc_total = ctx
        .accounts
        .treasury_usdc_state
        .staking_usdc_total
        .checked_add(split.staking)
        .ok_or(CustomError::MathOverflow)?;

    let decimals = ctx.accounts.usdc_mint.decimals;

    transfer_usdc_to_vault(
        &ctx,
        ctx.accounts.relief_usdc_vault.to_account_info(),
        split.relief,
        decimals,
    )?;
    transfer_usdc_to_vault(
        &ctx,
        ctx.accounts.buyback_usdc_vault.to_account_info(),
        split.buyback,
        decimals,
    )?;
    transfer_usdc_to_vault(
        &ctx,
        ctx.accounts.builders_usdc_vault.to_account_info(),
        split.builders,
        decimals,
    )?;
    transfer_usdc_to_vault(
        &ctx,
        ctx.accounts.staking_usdc_vault.to_account_info(),
        split.staking,
        decimals,
    )?;

    let treasury_usdc_state = &mut ctx.accounts.treasury_usdc_state;
    treasury_usdc_state.total_usdc_inflow = new_total_usdc_inflow;
    treasury_usdc_state.relief_usdc_total = new_relief_usdc_total;
    treasury_usdc_state.buyback_usdc_total = new_buyback_usdc_total;
    treasury_usdc_state.builders_usdc_total = new_builders_usdc_total;
    treasury_usdc_state.staking_usdc_total = new_staking_usdc_total;

    Ok(())
}

pub fn calculate_usdc_treasury_split(amount: u64) -> Result<UsdcTreasurySplit> {
    require!(amount > 0, CustomError::InvalidAmount);

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
    let builders = split_amount(amount, PAYROLL_BPS)?;
    let staking = amount
        .checked_sub(relief)
        .and_then(|value| value.checked_sub(buyback))
        .and_then(|value| value.checked_sub(builders))
        .ok_or(CustomError::MathOverflow)?;

    Ok(UsdcTreasurySplit {
        relief,
        buyback,
        builders,
        staking,
    })
}

fn split_amount(amount: u64, bps: u64) -> Result<u64> {
    let multiplied = amount.checked_mul(bps).ok_or(CustomError::MathOverflow)?;

    multiplied
        .checked_div(BPS_DENOMINATOR)
        .ok_or(CustomError::MathOverflow.into())
}

fn transfer_usdc_to_vault<'info>(
    ctx: &Context<DepositUsdcRevenue<'info>>,
    vault: AccountInfo<'info>,
    amount: u64,
    decimals: u8,
) -> Result<()> {
    let cpi_accounts = TransferChecked {
        from: ctx.accounts.depositor_usdc_token_account.to_account_info(),
        mint: ctx.accounts.usdc_mint.to_account_info(),
        to: vault,
        authority: ctx.accounts.depositor.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.key(), cpi_accounts);

    transfer_checked(cpi_ctx, amount, decimals)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_usdc_revenue_amount() {
        let err = calculate_usdc_treasury_split(0).unwrap_err();
        let message = format!("{err:?}");

        assert!(
            message.contains("InvalidAmount") || message.contains("Invalid amount"),
            "unexpected error: {message}"
        );
    }

    #[test]
    fn calculates_expected_usdc_split_for_100_usdc() {
        let split = calculate_usdc_treasury_split(100_000_000).unwrap();

        assert_eq!(split.relief, 50_000_000);
        assert_eq!(split.buyback, 20_000_000);
        assert_eq!(split.builders, 20_000_000);
        assert_eq!(split.staking, 10_000_000);
    }

    #[test]
    fn usdc_split_sums_to_original_amount() {
        let amount = 100_000_000;
        let split = calculate_usdc_treasury_split(amount).unwrap();

        assert_eq!(
            split.relief + split.buyback + split.builders + split.staking,
            amount
        );
    }

    #[test]
    fn rejects_overflowing_usdc_amount() {
        let err = calculate_usdc_treasury_split(u64::MAX).unwrap_err();
        let message = format!("{err:?}");

        assert!(
            message.contains("MathOverflow") || message.contains("Math overflow"),
            "unexpected error: {message}"
        );
    }
}
