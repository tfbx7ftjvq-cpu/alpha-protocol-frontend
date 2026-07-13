use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};

use crate::constants::{
    BPS_DENOMINATOR, BUILDERS_USDC_VAULT_SEED, BUYBACK_BPS, BUYBACK_USDC_VAULT_SEED, PAYROLL_BPS,
    RELIEF_BPS, RELIEF_USDC_VAULT_SEED, REVENUE_ROUTING_STATS_V1_SEED, STAKING_BPS,
    STAKING_USDC_VAULT_SEED, TREASURY_CONFIG_V2_SEED, TREASURY_USDC_STATE_V2_SEED,
    VAULT_AUTHORITY_V2_SEED,
};
use crate::error::CustomError;
use crate::state::{RevenueRoutingStatsV1, RevenueType, TreasuryConfigV2, TreasuryUsdcStateV2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsdcTreasurySplit {
    pub relief: u64,
    pub buyback: u64,
    pub builders: u64,
    pub staking: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RevenueRoutingStatsTotals {
    pub total_routed_usdc: u64,
    pub green_label_certification_fee_total: u64,
    pub green_label_forfeited_bond_total: u64,
    pub protocol_service_fee_total: u64,
    pub platform_revenue_total: u64,
    pub partnership_revenue_total: u64,
    pub manual_governance_approved_revenue_total: u64,
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

#[derive(Accounts)]
pub struct InitializeRevenueRoutingStatsV1<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [TREASURY_CONFIG_V2_SEED],
        bump = treasury_config.bump
    )]
    pub treasury_config: Account<'info, TreasuryConfigV2>,

    #[account(
        init,
        payer = authority,
        space = 8 + RevenueRoutingStatsV1::INIT_SPACE,
        seeds = [REVENUE_ROUTING_STATS_V1_SEED, treasury_config.key().as_ref()],
        bump
    )]
    pub revenue_routing_stats: Account<'info, RevenueRoutingStatsV1>,

    #[account(
        constraint = usdc_mint.key() == treasury_config.usdc_mint @ CustomError::InvalidMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RouteUsdcRevenueV1<'info> {
    #[account(mut)]
    pub revenue_payer: Signer<'info>,

    #[account(
        mut,
        constraint = revenue_payer_usdc_token_account.mint == treasury_config.usdc_mint @ CustomError::InvalidMint,
        constraint = revenue_payer_usdc_token_account.owner == revenue_payer.key() @ CustomError::InvalidTokenAccount
    )]
    pub revenue_payer_usdc_token_account: Box<Account<'info, TokenAccount>>,

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
        mut,
        seeds = [REVENUE_ROUTING_STATS_V1_SEED, treasury_config.key().as_ref()],
        bump = revenue_routing_stats.bump,
        constraint = revenue_routing_stats.authority == treasury_config.authority @ CustomError::UnauthorizedTreasuryAuthority,
        constraint = revenue_routing_stats.usdc_mint == treasury_config.usdc_mint @ CustomError::InvalidMint
    )]
    pub revenue_routing_stats: Account<'info, RevenueRoutingStatsV1>,

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
    let (
        new_total_usdc_inflow,
        new_relief_usdc_total,
        new_buyback_usdc_total,
        new_builders_usdc_total,
        new_staking_usdc_total,
    ) = checked_usdc_treasury_totals_after_route(&ctx.accounts.treasury_usdc_state, amount, split)?;

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

pub fn initialize_revenue_routing_stats_v1_handler(
    ctx: Context<InitializeRevenueRoutingStatsV1>,
) -> Result<()> {
    require_keys_eq!(
        ctx.accounts.authority.key(),
        ctx.accounts.treasury_config.authority,
        CustomError::UnauthorizedTreasuryAuthority
    );

    let revenue_routing_stats = &mut ctx.accounts.revenue_routing_stats;
    revenue_routing_stats.authority = ctx.accounts.authority.key();
    revenue_routing_stats.usdc_mint = ctx.accounts.usdc_mint.key();
    revenue_routing_stats.total_routed_usdc = 0;
    revenue_routing_stats.green_label_certification_fee_total = 0;
    revenue_routing_stats.green_label_forfeited_bond_total = 0;
    revenue_routing_stats.protocol_service_fee_total = 0;
    revenue_routing_stats.platform_revenue_total = 0;
    revenue_routing_stats.partnership_revenue_total = 0;
    revenue_routing_stats.manual_governance_approved_revenue_total = 0;
    revenue_routing_stats.bump = ctx.bumps.revenue_routing_stats;

    Ok(())
}

pub fn route_usdc_revenue_v1_handler(
    ctx: Context<RouteUsdcRevenueV1>,
    revenue_type: RevenueType,
    amount: u64,
) -> Result<()> {
    route_usdc_revenue_from_token_account(
        ctx.accounts.token_program.key(),
        ctx.accounts
            .revenue_payer_usdc_token_account
            .to_account_info(),
        ctx.accounts.revenue_payer.to_account_info(),
        None,
        ctx.accounts.usdc_mint.to_account_info(),
        ctx.accounts.relief_usdc_vault.to_account_info(),
        ctx.accounts.buyback_usdc_vault.to_account_info(),
        ctx.accounts.builders_usdc_vault.to_account_info(),
        ctx.accounts.staking_usdc_vault.to_account_info(),
        &mut ctx.accounts.treasury_usdc_state,
        &mut ctx.accounts.revenue_routing_stats,
        ctx.accounts.usdc_mint.key(),
        revenue_type,
        amount,
        ctx.accounts.usdc_mint.decimals,
    )
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

#[allow(clippy::too_many_arguments)]
pub fn route_usdc_revenue_from_token_account<'info>(
    token_program: Pubkey,
    source_token_account: AccountInfo<'info>,
    source_authority: AccountInfo<'info>,
    signer_seeds: Option<&[&[&[u8]]]>,
    usdc_mint: AccountInfo<'info>,
    relief_usdc_vault: AccountInfo<'info>,
    buyback_usdc_vault: AccountInfo<'info>,
    builders_usdc_vault: AccountInfo<'info>,
    staking_usdc_vault: AccountInfo<'info>,
    treasury_usdc_state: &mut TreasuryUsdcStateV2,
    revenue_routing_stats: &mut RevenueRoutingStatsV1,
    expected_usdc_mint: Pubkey,
    revenue_type: RevenueType,
    amount: u64,
    decimals: u8,
) -> Result<()> {
    require!(amount > 0, CustomError::InvalidAmount);
    require_keys_eq!(
        expected_usdc_mint,
        revenue_routing_stats.usdc_mint,
        CustomError::InvalidMint
    );

    let split = calculate_usdc_treasury_split(amount)?;
    let (
        new_total_usdc_inflow,
        new_relief_usdc_total,
        new_buyback_usdc_total,
        new_builders_usdc_total,
        new_staking_usdc_total,
    ) = checked_usdc_treasury_totals_after_route(treasury_usdc_state, amount, split)?;

    let stats_after_route =
        calculate_revenue_routing_stats_after_route(revenue_routing_stats, revenue_type, amount)?;

    transfer_usdc_checked_with_optional_signer(
        token_program,
        source_token_account.clone(),
        usdc_mint.clone(),
        relief_usdc_vault,
        source_authority.clone(),
        signer_seeds,
        split.relief,
        decimals,
    )?;
    transfer_usdc_checked_with_optional_signer(
        token_program,
        source_token_account.clone(),
        usdc_mint.clone(),
        buyback_usdc_vault,
        source_authority.clone(),
        signer_seeds,
        split.buyback,
        decimals,
    )?;
    transfer_usdc_checked_with_optional_signer(
        token_program,
        source_token_account.clone(),
        usdc_mint.clone(),
        builders_usdc_vault,
        source_authority.clone(),
        signer_seeds,
        split.builders,
        decimals,
    )?;
    transfer_usdc_checked_with_optional_signer(
        token_program,
        source_token_account,
        usdc_mint,
        staking_usdc_vault,
        source_authority,
        signer_seeds,
        split.staking,
        decimals,
    )?;

    treasury_usdc_state.total_usdc_inflow = new_total_usdc_inflow;
    treasury_usdc_state.relief_usdc_total = new_relief_usdc_total;
    treasury_usdc_state.buyback_usdc_total = new_buyback_usdc_total;
    treasury_usdc_state.builders_usdc_total = new_builders_usdc_total;
    treasury_usdc_state.staking_usdc_total = new_staking_usdc_total;

    record_revenue_routing_stats(revenue_routing_stats, stats_after_route);

    Ok(())
}

fn split_amount(amount: u64, bps: u64) -> Result<u64> {
    let multiplied = amount.checked_mul(bps).ok_or(CustomError::MathOverflow)?;

    multiplied
        .checked_div(BPS_DENOMINATOR)
        .ok_or(CustomError::MathOverflow.into())
}

fn checked_usdc_treasury_totals_after_route(
    treasury_usdc_state: &TreasuryUsdcStateV2,
    amount: u64,
    split: UsdcTreasurySplit,
) -> Result<(u64, u64, u64, u64, u64)> {
    let new_total_usdc_inflow = treasury_usdc_state
        .total_usdc_inflow
        .checked_add(amount)
        .ok_or(CustomError::MathOverflow)?;
    let new_relief_usdc_total = treasury_usdc_state
        .relief_usdc_total
        .checked_add(split.relief)
        .ok_or(CustomError::MathOverflow)?;
    let new_buyback_usdc_total = treasury_usdc_state
        .buyback_usdc_total
        .checked_add(split.buyback)
        .ok_or(CustomError::MathOverflow)?;
    let new_builders_usdc_total = treasury_usdc_state
        .builders_usdc_total
        .checked_add(split.builders)
        .ok_or(CustomError::MathOverflow)?;
    let new_staking_usdc_total = treasury_usdc_state
        .staking_usdc_total
        .checked_add(split.staking)
        .ok_or(CustomError::MathOverflow)?;

    Ok((
        new_total_usdc_inflow,
        new_relief_usdc_total,
        new_buyback_usdc_total,
        new_builders_usdc_total,
        new_staking_usdc_total,
    ))
}

pub fn calculate_revenue_routing_stats_after_route(
    revenue_routing_stats: &RevenueRoutingStatsV1,
    revenue_type: RevenueType,
    amount: u64,
) -> Result<RevenueRoutingStatsTotals> {
    require!(amount > 0, CustomError::InvalidAmount);

    let mut totals = RevenueRoutingStatsTotals {
        total_routed_usdc: revenue_routing_stats
            .total_routed_usdc
            .checked_add(amount)
            .ok_or(CustomError::MathOverflow)?,
        green_label_certification_fee_total: revenue_routing_stats
            .green_label_certification_fee_total,
        green_label_forfeited_bond_total: revenue_routing_stats.green_label_forfeited_bond_total,
        protocol_service_fee_total: revenue_routing_stats.protocol_service_fee_total,
        platform_revenue_total: revenue_routing_stats.platform_revenue_total,
        partnership_revenue_total: revenue_routing_stats.partnership_revenue_total,
        manual_governance_approved_revenue_total: revenue_routing_stats
            .manual_governance_approved_revenue_total,
    };

    match revenue_type {
        RevenueType::GreenLabelCertificationFee => {
            totals.green_label_certification_fee_total = totals
                .green_label_certification_fee_total
                .checked_add(amount)
                .ok_or(CustomError::MathOverflow)?;
        }
        RevenueType::GreenLabelForfeitedBond => {
            totals.green_label_forfeited_bond_total = totals
                .green_label_forfeited_bond_total
                .checked_add(amount)
                .ok_or(CustomError::MathOverflow)?;
        }
        RevenueType::ProtocolServiceFee => {
            totals.protocol_service_fee_total = totals
                .protocol_service_fee_total
                .checked_add(amount)
                .ok_or(CustomError::MathOverflow)?;
        }
        RevenueType::PlatformRevenue => {
            totals.platform_revenue_total = totals
                .platform_revenue_total
                .checked_add(amount)
                .ok_or(CustomError::MathOverflow)?;
        }
        RevenueType::PartnershipRevenue => {
            totals.partnership_revenue_total = totals
                .partnership_revenue_total
                .checked_add(amount)
                .ok_or(CustomError::MathOverflow)?;
        }
        RevenueType::ManualGovernanceApprovedRevenue => {
            totals.manual_governance_approved_revenue_total = totals
                .manual_governance_approved_revenue_total
                .checked_add(amount)
                .ok_or(CustomError::MathOverflow)?;
        }
    }

    Ok(totals)
}

fn record_revenue_routing_stats(
    revenue_routing_stats: &mut RevenueRoutingStatsV1,
    totals: RevenueRoutingStatsTotals,
) {
    revenue_routing_stats.total_routed_usdc = totals.total_routed_usdc;
    revenue_routing_stats.green_label_certification_fee_total =
        totals.green_label_certification_fee_total;
    revenue_routing_stats.green_label_forfeited_bond_total =
        totals.green_label_forfeited_bond_total;
    revenue_routing_stats.protocol_service_fee_total = totals.protocol_service_fee_total;
    revenue_routing_stats.platform_revenue_total = totals.platform_revenue_total;
    revenue_routing_stats.partnership_revenue_total = totals.partnership_revenue_total;
    revenue_routing_stats.manual_governance_approved_revenue_total =
        totals.manual_governance_approved_revenue_total;
}

fn transfer_usdc_to_vault<'info>(
    ctx: &Context<DepositUsdcRevenue<'info>>,
    vault: AccountInfo<'info>,
    amount: u64,
    decimals: u8,
) -> Result<()> {
    transfer_usdc_checked(
        ctx.accounts.token_program.key(),
        ctx.accounts.depositor_usdc_token_account.to_account_info(),
        ctx.accounts.usdc_mint.to_account_info(),
        vault,
        ctx.accounts.depositor.to_account_info(),
        amount,
        decimals,
    )
}

fn transfer_usdc_checked<'info>(
    token_program: Pubkey,
    from: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    to: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    amount: u64,
    decimals: u8,
) -> Result<()> {
    let cpi_accounts = TransferChecked {
        from,
        mint,
        to,
        authority,
    };
    let cpi_ctx = CpiContext::new(token_program, cpi_accounts);

    transfer_checked(cpi_ctx, amount, decimals)
}

fn transfer_usdc_checked_with_optional_signer<'info>(
    token_program: Pubkey,
    from: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    to: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    signer_seeds: Option<&[&[&[u8]]]>,
    amount: u64,
    decimals: u8,
) -> Result<()> {
    let cpi_accounts = TransferChecked {
        from,
        mint,
        to,
        authority,
    };

    let cpi_ctx = if let Some(seeds) = signer_seeds {
        CpiContext::new_with_signer(token_program, cpi_accounts, seeds)
    } else {
        CpiContext::new(token_program, cpi_accounts)
    };

    transfer_checked(cpi_ctx, amount, decimals)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_revenue_routing_stats() -> RevenueRoutingStatsV1 {
        RevenueRoutingStatsV1 {
            authority: Pubkey::new_unique(),
            usdc_mint: Pubkey::new_unique(),
            total_routed_usdc: 0,
            green_label_certification_fee_total: 0,
            green_label_forfeited_bond_total: 0,
            protocol_service_fee_total: 0,
            platform_revenue_total: 0,
            partnership_revenue_total: 0,
            manual_governance_approved_revenue_total: 0,
            bump: 255,
        }
    }

    fn assert_split_sums_to_amount(amount: u64) {
        let split = calculate_usdc_treasury_split(amount).unwrap();

        assert_eq!(
            split.relief + split.buyback + split.builders + split.staking,
            amount
        );
    }

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
    fn usdc_split_sums_for_one_raw_unit() {
        let split = calculate_usdc_treasury_split(1).unwrap();

        assert_eq!(split.relief, 0);
        assert_eq!(split.buyback, 0);
        assert_eq!(split.builders, 0);
        assert_eq!(split.staking, 1);
        assert_split_sums_to_amount(1);
    }

    #[test]
    fn usdc_split_sums_for_three_raw_units() {
        let split = calculate_usdc_treasury_split(3).unwrap();

        assert_eq!(split.relief, 1);
        assert_eq!(split.buyback, 0);
        assert_eq!(split.builders, 0);
        assert_eq!(split.staking, 2);
        assert_split_sums_to_amount(3);
    }

    #[test]
    fn usdc_split_sums_for_twenty_raw_units() {
        let split = calculate_usdc_treasury_split(20).unwrap();

        assert_eq!(split.relief, 10);
        assert_eq!(split.buyback, 4);
        assert_eq!(split.builders, 4);
        assert_eq!(split.staking, 2);
        assert_split_sums_to_amount(20);
    }

    #[test]
    fn usdc_split_sums_for_ten_usdc() {
        let split = calculate_usdc_treasury_split(10_000_000).unwrap();

        assert_eq!(split.relief, 5_000_000);
        assert_eq!(split.buyback, 2_000_000);
        assert_eq!(split.builders, 2_000_000);
        assert_eq!(split.staking, 1_000_000);
        assert_split_sums_to_amount(10_000_000);
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

    #[test]
    fn records_green_label_certification_fee_revenue_type() {
        let stats = default_revenue_routing_stats();
        let totals = calculate_revenue_routing_stats_after_route(
            &stats,
            RevenueType::GreenLabelCertificationFee,
            1_000_000,
        )
        .unwrap();

        assert_eq!(totals.total_routed_usdc, 1_000_000);
        assert_eq!(totals.green_label_certification_fee_total, 1_000_000);
        assert_eq!(totals.green_label_forfeited_bond_total, 0);
    }

    #[test]
    fn records_green_label_forfeited_bond_revenue_type() {
        let stats = default_revenue_routing_stats();
        let totals = calculate_revenue_routing_stats_after_route(
            &stats,
            RevenueType::GreenLabelForfeitedBond,
            2_000_000,
        )
        .unwrap();

        assert_eq!(totals.total_routed_usdc, 2_000_000);
        assert_eq!(totals.green_label_forfeited_bond_total, 2_000_000);
        assert_eq!(totals.green_label_certification_fee_total, 0);
    }

    #[test]
    fn records_protocol_service_fee_revenue_type() {
        let stats = default_revenue_routing_stats();
        let totals = calculate_revenue_routing_stats_after_route(
            &stats,
            RevenueType::ProtocolServiceFee,
            3_000_000,
        )
        .unwrap();

        assert_eq!(totals.total_routed_usdc, 3_000_000);
        assert_eq!(totals.protocol_service_fee_total, 3_000_000);
    }

    #[test]
    fn records_platform_revenue_type() {
        let stats = default_revenue_routing_stats();
        let totals = calculate_revenue_routing_stats_after_route(
            &stats,
            RevenueType::PlatformRevenue,
            4_000_000,
        )
        .unwrap();

        assert_eq!(totals.total_routed_usdc, 4_000_000);
        assert_eq!(totals.platform_revenue_total, 4_000_000);
    }

    #[test]
    fn records_partnership_revenue_type() {
        let stats = default_revenue_routing_stats();
        let totals = calculate_revenue_routing_stats_after_route(
            &stats,
            RevenueType::PartnershipRevenue,
            5_000_000,
        )
        .unwrap();

        assert_eq!(totals.total_routed_usdc, 5_000_000);
        assert_eq!(totals.partnership_revenue_total, 5_000_000);
    }

    #[test]
    fn records_manual_governance_approved_revenue_type() {
        let stats = default_revenue_routing_stats();
        let totals = calculate_revenue_routing_stats_after_route(
            &stats,
            RevenueType::ManualGovernanceApprovedRevenue,
            6_000_000,
        )
        .unwrap();

        assert_eq!(totals.total_routed_usdc, 6_000_000);
        assert_eq!(totals.manual_governance_approved_revenue_total, 6_000_000);
    }

    #[test]
    fn rejects_zero_revenue_routing_stats_amount() {
        let stats = default_revenue_routing_stats();
        let err =
            calculate_revenue_routing_stats_after_route(&stats, RevenueType::ProtocolServiceFee, 0)
                .unwrap_err();
        let message = format!("{err:?}");

        assert!(
            message.contains("InvalidAmount") || message.contains("Invalid amount"),
            "unexpected error: {message}"
        );
    }

    #[test]
    fn rejects_overflowing_revenue_routing_stats_total() {
        let mut stats = default_revenue_routing_stats();
        stats.total_routed_usdc = u64::MAX;

        let err =
            calculate_revenue_routing_stats_after_route(&stats, RevenueType::ProtocolServiceFee, 1)
                .unwrap_err();
        let message = format!("{err:?}");

        assert!(
            message.contains("MathOverflow") || message.contains("Math overflow"),
            "unexpected error: {message}"
        );
    }
}
