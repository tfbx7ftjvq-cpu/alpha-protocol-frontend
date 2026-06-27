use anchor_lang::prelude::*;

use crate::constants::{
    BASE_BOND_REFUND_BPS, BASE_BOND_TREASURY_BPS, DEFAULT_DISPUTE_WINDOW_SECONDS,
    DEFAULT_OBSERVATION_PERIOD_SECONDS, DEFAULT_RESPONSE_WINDOW_SECONDS,
    GREEN_LABEL_BRONZE_TIER_THRESHOLD_USDC, GREEN_LABEL_CONFIG_RESERVED_BYTES,
    GREEN_LABEL_CONFIG_SEED, GREEN_LABEL_CONFIG_SPACE, GREEN_LABEL_DISPUTE_SPACE,
    GREEN_LABEL_GOLD_TIER_THRESHOLD_USDC, GREEN_LABEL_PLATINUM_TIER_THRESHOLD_USDC,
    GREEN_LABEL_PROJECT_SPACE, GREEN_LABEL_SILVER_TIER_THRESHOLD_USDC, MAX_BPS,
    MIN_GREEN_LABEL_BASE_BOND_USDC,
};
use crate::error::CustomError;
use crate::state::{ActionType, BondTier, GreenLabelConfigV1, GreenLabelStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GreenLabelBondSplit {
    pub base_bond_amount: u64,
    pub extra_bond_amount: u64,
    pub total_bond_amount: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GreenLabelRefundAmounts {
    pub project_refund_amount: u64,
    pub treasury_amount: u64,
    pub base_refund_amount: u64,
    pub base_treasury_amount: u64,
    pub extra_refund_amount: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GreenLabelConfigInitValues {
    pub authority: Pubkey,
    pub usdc_mint: Pubkey,
    pub min_base_bond_usdc: u64,
    pub base_refund_bps: u16,
    pub base_treasury_bps: u16,
    pub observation_period_seconds: i64,
    pub dispute_window_seconds: i64,
    pub response_window_seconds: i64,
    pub project_count: u64,
    pub treasury_usdc_state_v2: Pubkey,
    pub base_bond_treasury_vault: Pubkey,
    pub relief_or_risk_vault: Pubkey,
    pub vault_authority_v2: Pubkey,
    pub security_governance_config: Pubkey,
    pub is_paused: bool,
    pub bump: u8,
    pub reserved: [u8; GREEN_LABEL_CONFIG_RESERVED_BYTES],
}

#[derive(Accounts)]
pub struct InitializeGreenLabelConfig<'info> {
    #[account(
        init,
        payer = authority,
        space = GREEN_LABEL_CONFIG_SPACE,
        seeds = [GREEN_LABEL_CONFIG_SEED],
        bump
    )]
    pub green_label_config: Account<'info, GreenLabelConfigV1>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: Phase 1C stores the mint key only; token validation is added in later phases.
    pub usdc_mint: UncheckedAccount<'info>,

    /// CHECK: Phase 1C stores the Treasury V2 state key only.
    pub treasury_usdc_state_v2: UncheckedAccount<'info>,

    /// CHECK: Phase 1C stores the base bond treasury vault key only.
    pub base_bond_treasury_vault: UncheckedAccount<'info>,

    /// CHECK: Phase 1C stores the relief/risk vault key only.
    pub relief_or_risk_vault: UncheckedAccount<'info>,

    /// CHECK: Phase 1C stores the Treasury V2 vault authority key only.
    pub vault_authority_v2: UncheckedAccount<'info>,

    /// CHECK: Phase 1C stores the Security Layer governance config key only.
    pub security_governance_config: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_green_label_config_handler(
    ctx: Context<InitializeGreenLabelConfig>,
) -> Result<()> {
    let values = build_default_green_label_config_values(
        ctx.accounts.authority.key(),
        ctx.accounts.usdc_mint.key(),
        ctx.accounts.treasury_usdc_state_v2.key(),
        ctx.accounts.base_bond_treasury_vault.key(),
        ctx.accounts.relief_or_risk_vault.key(),
        ctx.accounts.vault_authority_v2.key(),
        ctx.accounts.security_governance_config.key(),
        ctx.bumps.green_label_config,
    )?;

    let green_label_config = &mut ctx.accounts.green_label_config;
    green_label_config.authority = values.authority;
    green_label_config.usdc_mint = values.usdc_mint;
    green_label_config.min_base_bond_usdc = values.min_base_bond_usdc;
    green_label_config.base_refund_bps = values.base_refund_bps;
    green_label_config.base_treasury_bps = values.base_treasury_bps;
    green_label_config.observation_period_seconds = values.observation_period_seconds;
    green_label_config.dispute_window_seconds = values.dispute_window_seconds;
    green_label_config.response_window_seconds = values.response_window_seconds;
    green_label_config.project_count = values.project_count;
    green_label_config.treasury_usdc_state_v2 = values.treasury_usdc_state_v2;
    green_label_config.base_bond_treasury_vault = values.base_bond_treasury_vault;
    green_label_config.relief_or_risk_vault = values.relief_or_risk_vault;
    green_label_config.vault_authority_v2 = values.vault_authority_v2;
    green_label_config.security_governance_config = values.security_governance_config;
    green_label_config.is_paused = values.is_paused;
    green_label_config.bump = values.bump;
    green_label_config.reserved = values.reserved;

    Ok(())
}

pub fn build_default_green_label_config_values(
    authority: Pubkey,
    usdc_mint: Pubkey,
    treasury_usdc_state_v2: Pubkey,
    base_bond_treasury_vault: Pubkey,
    relief_or_risk_vault: Pubkey,
    vault_authority_v2: Pubkey,
    security_governance_config: Pubkey,
    bump: u8,
) -> Result<GreenLabelConfigInitValues> {
    validate_green_label_bps_config(BASE_BOND_REFUND_BPS, BASE_BOND_TREASURY_BPS)?;

    Ok(GreenLabelConfigInitValues {
        authority,
        usdc_mint,
        min_base_bond_usdc: MIN_GREEN_LABEL_BASE_BOND_USDC,
        base_refund_bps: BASE_BOND_REFUND_BPS,
        base_treasury_bps: BASE_BOND_TREASURY_BPS,
        observation_period_seconds: DEFAULT_OBSERVATION_PERIOD_SECONDS,
        dispute_window_seconds: DEFAULT_DISPUTE_WINDOW_SECONDS,
        response_window_seconds: DEFAULT_RESPONSE_WINDOW_SECONDS,
        project_count: 0,
        treasury_usdc_state_v2,
        base_bond_treasury_vault,
        relief_or_risk_vault,
        vault_authority_v2,
        security_governance_config,
        is_paused: false,
        bump,
        reserved: [0; GREEN_LABEL_CONFIG_RESERVED_BYTES],
    })
}

pub fn validate_green_label_bps_config(base_refund_bps: u16, base_treasury_bps: u16) -> Result<()> {
    let configured_bps = base_refund_bps
        .checked_add(base_treasury_bps)
        .ok_or(CustomError::GreenLabelMathOverflow)?;

    require!(
        configured_bps == MAX_BPS,
        CustomError::InvalidGreenLabelBpsConfig
    );

    Ok(())
}

pub fn split_green_label_bond(total_bond_amount: u64) -> Result<GreenLabelBondSplit> {
    require!(
        total_bond_amount >= MIN_GREEN_LABEL_BASE_BOND_USDC,
        CustomError::InvalidGreenLabelBondAmount
    );

    let extra_bond_amount = total_bond_amount
        .checked_sub(MIN_GREEN_LABEL_BASE_BOND_USDC)
        .ok_or(CustomError::GreenLabelMathOverflow)?;

    Ok(GreenLabelBondSplit {
        base_bond_amount: MIN_GREEN_LABEL_BASE_BOND_USDC,
        extra_bond_amount,
        total_bond_amount,
    })
}

pub fn calculate_green_label_refund(total_bond_amount: u64) -> Result<GreenLabelRefundAmounts> {
    validate_green_label_bps_config(BASE_BOND_REFUND_BPS, BASE_BOND_TREASURY_BPS)?;

    let split = split_green_label_bond(total_bond_amount)?;
    let base_refund_amount = calculate_bps_amount(split.base_bond_amount, BASE_BOND_REFUND_BPS)?;
    let base_treasury_amount =
        calculate_bps_amount(split.base_bond_amount, BASE_BOND_TREASURY_BPS)?;

    let base_total = base_refund_amount
        .checked_add(base_treasury_amount)
        .ok_or(CustomError::GreenLabelMathOverflow)?;
    require!(
        base_total == split.base_bond_amount,
        CustomError::InvalidGreenLabelBondSplit
    );

    let project_refund_amount = base_refund_amount
        .checked_add(split.extra_bond_amount)
        .ok_or(CustomError::GreenLabelMathOverflow)?;

    Ok(GreenLabelRefundAmounts {
        project_refund_amount,
        treasury_amount: base_treasury_amount,
        base_refund_amount,
        base_treasury_amount,
        extra_refund_amount: split.extra_bond_amount,
    })
}

pub fn calculate_green_label_slash_amount(total_bond_amount: u64) -> Result<u64> {
    split_green_label_bond(total_bond_amount)?;
    Ok(total_bond_amount)
}

pub fn calculate_bond_tier(total_bond_amount: u64) -> Result<BondTier> {
    require!(
        total_bond_amount >= MIN_GREEN_LABEL_BASE_BOND_USDC,
        CustomError::InvalidGreenLabelBondAmount
    );

    if total_bond_amount >= GREEN_LABEL_PLATINUM_TIER_THRESHOLD_USDC {
        Ok(BondTier::Platinum)
    } else if total_bond_amount >= GREEN_LABEL_GOLD_TIER_THRESHOLD_USDC {
        Ok(BondTier::Gold)
    } else if total_bond_amount >= GREEN_LABEL_SILVER_TIER_THRESHOLD_USDC {
        Ok(BondTier::Silver)
    } else if total_bond_amount >= GREEN_LABEL_BRONZE_TIER_THRESHOLD_USDC {
        Ok(BondTier::Bronze)
    } else {
        Ok(BondTier::Base)
    }
}

pub fn validate_green_label_status_transition(
    from: GreenLabelStatus,
    to: GreenLabelStatus,
    has_linked_dispute: bool,
) -> Result<()> {
    if matches!(
        from,
        GreenLabelStatus::Refunded | GreenLabelStatus::Slashed | GreenLabelStatus::Cancelled
    ) {
        return err!(CustomError::InvalidGreenLabelStatus);
    }

    if to == GreenLabelStatus::SlashQueued && !has_linked_dispute {
        return err!(CustomError::InvalidGreenLabelSlashWithoutDispute);
    }

    let is_valid = matches!(
        (from, to),
        (
            GreenLabelStatus::PendingObservation,
            GreenLabelStatus::ActiveGreenLabel
        ) | (
            GreenLabelStatus::PendingObservation,
            GreenLabelStatus::Disputed
        ) | (
            GreenLabelStatus::PendingObservation,
            GreenLabelStatus::RefundQueued
        ) | (
            GreenLabelStatus::ActiveGreenLabel,
            GreenLabelStatus::Disputed
        ) | (GreenLabelStatus::Disputed, GreenLabelStatus::SlashQueued)
            | (GreenLabelStatus::Disputed, GreenLabelStatus::RefundQueued)
            | (GreenLabelStatus::RefundQueued, GreenLabelStatus::Refunded)
            | (GreenLabelStatus::SlashQueued, GreenLabelStatus::Slashed)
    );

    require!(is_valid, CustomError::InvalidGreenLabelStatus);

    Ok(())
}

pub fn validate_payload_hash(payload_hash: [u8; 32]) -> Result<()> {
    require!(
        payload_hash != [0; 32],
        CustomError::InvalidGreenLabelPayloadHash
    );

    Ok(())
}

pub fn expected_green_label_config_space() -> usize {
    GREEN_LABEL_CONFIG_SPACE
}

pub fn expected_green_label_project_space() -> usize {
    GREEN_LABEL_PROJECT_SPACE
}

pub fn expected_green_label_dispute_space() -> usize {
    GREEN_LABEL_DISPUTE_SPACE
}

pub fn derive_bond_split_and_tier(
    total_bond_amount: u64,
) -> Result<(GreenLabelBondSplit, BondTier)> {
    let split = split_green_label_bond(total_bond_amount)?;
    let tier = calculate_bond_tier(total_bond_amount)?;

    Ok((split, tier))
}

pub fn validate_terminal_action_for_refund(action_type: ActionType) -> Result<()> {
    require!(
        action_type == ActionType::GreenLabelRefund,
        CustomError::InvalidGreenLabelActionType
    );

    Ok(())
}

pub fn validate_terminal_action_for_slash(
    action_type: ActionType,
    has_linked_dispute: bool,
) -> Result<()> {
    require!(
        action_type == ActionType::GreenLabelSlash,
        CustomError::InvalidGreenLabelActionType
    );
    require!(
        has_linked_dispute,
        CustomError::InvalidGreenLabelSlashWithoutDispute
    );

    Ok(())
}

fn calculate_bps_amount(amount: u64, bps: u16) -> Result<u64> {
    amount
        .checked_mul(bps as u64)
        .and_then(|value| value.checked_div(MAX_BPS as u64))
        .ok_or(CustomError::GreenLabelMathOverflow.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{
        ANCHOR_ACCOUNT_DISCRIMINATOR_BYTES, DEFAULT_DISPUTE_WINDOW_SECONDS,
        DEFAULT_OBSERVATION_PERIOD_SECONDS, DEFAULT_RESPONSE_WINDOW_SECONDS,
        GREEN_BOND_VAULT_AUTHORITY_SEED, GREEN_BOND_VAULT_SEED, GREEN_LABEL_CONFIG_RESERVED_BYTES,
        GREEN_LABEL_CONFIG_SEED, GREEN_LABEL_DISPUTE_RESERVED_BYTES, GREEN_LABEL_DISPUTE_SEED,
        GREEN_LABEL_PROJECT_RESERVED_BYTES, GREEN_LABEL_PROJECT_SEED,
    };
    use crate::state::{
        DisputeStatus, GreenLabelConfigV1, GreenLabelDisputeV1, GreenLabelProjectV1, RugReasonCode,
    };

    fn assert_error_contains(err: anchor_lang::error::Error, expected: &str) {
        let message = format!("{err:?}");
        assert!(
            message.contains(expected),
            "expected {expected}, got {message}"
        );
    }

    #[test]
    fn validate_bps_config_accepts_80_20() {
        validate_green_label_bps_config(BASE_BOND_REFUND_BPS, BASE_BOND_TREASURY_BPS).unwrap();
    }

    #[test]
    fn initialize_green_label_config_defaults_match_constants() {
        let values = green_label_config_init_values();

        assert_eq!(values.min_base_bond_usdc, MIN_GREEN_LABEL_BASE_BOND_USDC);
        assert_eq!(values.base_refund_bps, BASE_BOND_REFUND_BPS);
        assert_eq!(values.base_treasury_bps, BASE_BOND_TREASURY_BPS);
        assert_eq!(
            values.observation_period_seconds,
            DEFAULT_OBSERVATION_PERIOD_SECONDS
        );
        assert_eq!(
            values.dispute_window_seconds,
            DEFAULT_DISPUTE_WINDOW_SECONDS
        );
        assert_eq!(
            values.response_window_seconds,
            DEFAULT_RESPONSE_WINDOW_SECONDS
        );
    }

    #[test]
    fn initialize_green_label_config_uses_zero_project_count() {
        let values = green_label_config_init_values();

        assert_eq!(values.project_count, 0);
    }

    #[test]
    fn initialize_green_label_config_uses_unpaused_default() {
        let values = green_label_config_init_values();

        assert!(!values.is_paused);
    }

    #[test]
    fn initialize_green_label_config_reserved_zeroed() {
        let values = green_label_config_init_values();

        assert_eq!(values.reserved, [0; GREEN_LABEL_CONFIG_RESERVED_BYTES]);
    }

    #[test]
    fn validate_bps_config_rejects_invalid_sum() {
        let err = validate_green_label_bps_config(8_000, 1_000).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBpsConfig");
    }

    #[test]
    fn split_bond_accepts_minimum_299() {
        let split = split_green_label_bond(299_000_000).unwrap();

        assert_eq!(split.base_bond_amount, 299_000_000);
        assert_eq!(split.extra_bond_amount, 0);
        assert_eq!(split.total_bond_amount, 299_000_000);
    }

    #[test]
    fn split_bond_rejects_below_299() {
        let err = split_green_label_bond(298_999_999).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBondAmount");
    }

    #[test]
    fn split_bond_separates_1299_into_299_base_1000_extra() {
        let split = split_green_label_bond(1_299_000_000).unwrap();

        assert_eq!(split.base_bond_amount, 299_000_000);
        assert_eq!(split.extra_bond_amount, 1_000_000_000);
        assert_eq!(split.total_bond_amount, 1_299_000_000);
    }

    #[test]
    fn refund_calculation_for_299() {
        let amounts = calculate_green_label_refund(299_000_000).unwrap();

        assert_eq!(amounts.base_refund_amount, 239_200_000);
        assert_eq!(amounts.base_treasury_amount, 59_800_000);
        assert_eq!(amounts.extra_refund_amount, 0);
        assert_eq!(amounts.project_refund_amount, 239_200_000);
        assert_eq!(amounts.treasury_amount, 59_800_000);
    }

    #[test]
    fn refund_calculation_for_1299() {
        let amounts = calculate_green_label_refund(1_299_000_000).unwrap();

        assert_eq!(amounts.base_refund_amount, 239_200_000);
        assert_eq!(amounts.base_treasury_amount, 59_800_000);
        assert_eq!(amounts.extra_refund_amount, 1_000_000_000);
        assert_eq!(amounts.project_refund_amount, 1_239_200_000);
        assert_eq!(amounts.treasury_amount, 59_800_000);
    }

    #[test]
    fn slash_amount_is_full_bond() {
        assert_eq!(
            calculate_green_label_slash_amount(1_299_000_000).unwrap(),
            1_299_000_000
        );
    }

    #[test]
    fn bond_tier_base() {
        assert_eq!(calculate_bond_tier(299_000_000).unwrap(), BondTier::Base);
        assert_eq!(calculate_bond_tier(499_999_999).unwrap(), BondTier::Base);
    }

    #[test]
    fn bond_tier_bronze() {
        assert_eq!(calculate_bond_tier(500_000_000).unwrap(), BondTier::Bronze);
        assert_eq!(calculate_bond_tier(999_999_999).unwrap(), BondTier::Bronze);
    }

    #[test]
    fn bond_tier_silver() {
        assert_eq!(
            calculate_bond_tier(1_000_000_000).unwrap(),
            BondTier::Silver
        );
        assert_eq!(
            calculate_bond_tier(2_999_999_999).unwrap(),
            BondTier::Silver
        );
    }

    #[test]
    fn bond_tier_gold() {
        assert_eq!(calculate_bond_tier(3_000_000_000).unwrap(), BondTier::Gold);
        assert_eq!(calculate_bond_tier(9_999_999_999).unwrap(), BondTier::Gold);
    }

    #[test]
    fn bond_tier_platinum() {
        assert_eq!(
            calculate_bond_tier(10_000_000_000).unwrap(),
            BondTier::Platinum
        );
        assert_eq!(
            calculate_bond_tier(100_000_000_000).unwrap(),
            BondTier::Platinum
        );
    }

    #[test]
    fn bond_tier_rejects_below_minimum() {
        let err = calculate_bond_tier(298_999_999).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBondAmount");
    }

    #[test]
    fn status_transition_pending_to_active() {
        validate_green_label_status_transition(
            GreenLabelStatus::PendingObservation,
            GreenLabelStatus::ActiveGreenLabel,
            false,
        )
        .unwrap();
    }

    #[test]
    fn status_transition_pending_to_disputed() {
        validate_green_label_status_transition(
            GreenLabelStatus::PendingObservation,
            GreenLabelStatus::Disputed,
            false,
        )
        .unwrap();
    }

    #[test]
    fn status_transition_disputed_to_slash_requires_linked_dispute() {
        let err = validate_green_label_status_transition(
            GreenLabelStatus::Disputed,
            GreenLabelStatus::SlashQueued,
            false,
        )
        .unwrap_err();
        assert_error_contains(err, "InvalidGreenLabelSlashWithoutDispute");

        validate_green_label_status_transition(
            GreenLabelStatus::Disputed,
            GreenLabelStatus::SlashQueued,
            true,
        )
        .unwrap();
    }

    #[test]
    fn status_transition_terminal_refunded_rejects_next_transition() {
        let err = validate_green_label_status_transition(
            GreenLabelStatus::Refunded,
            GreenLabelStatus::Disputed,
            false,
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn payload_hash_rejects_zero_hash() {
        let err = validate_payload_hash([0; 32]).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelPayloadHash");
    }

    #[test]
    fn payload_hash_accepts_nonzero_hash() {
        validate_payload_hash([1; 32]).unwrap();
    }

    #[test]
    fn green_label_config_space_is_at_least_expected_minimum() {
        let minimum = ANCHOR_ACCOUNT_DISCRIMINATOR_BYTES
            + (32 * 7)
            + (8 * 2)
            + (2 * 2)
            + (8 * 3)
            + 1
            + 1
            + GREEN_LABEL_CONFIG_RESERVED_BYTES;

        assert!(expected_green_label_config_space() >= minimum);
        assert_eq!(
            GreenLabelConfigV1::INIT_SPACE + ANCHOR_ACCOUNT_DISCRIMINATOR_BYTES,
            expected_green_label_config_space()
        );
    }

    #[test]
    fn green_label_project_space_is_at_least_expected_minimum() {
        let minimum = ANCHOR_ACCOUNT_DISCRIMINATOR_BYTES
            + (32 * 11)
            + (8 * 12)
            + 1
            + 1
            + 2
            + 1
            + 1
            + GREEN_LABEL_PROJECT_RESERVED_BYTES;

        assert!(expected_green_label_project_space() >= minimum);
        assert_eq!(
            GreenLabelProjectV1::INIT_SPACE + ANCHOR_ACCOUNT_DISCRIMINATOR_BYTES,
            expected_green_label_project_space()
        );
    }

    #[test]
    fn green_label_dispute_space_is_at_least_expected_minimum() {
        let minimum = ANCHOR_ACCOUNT_DISCRIMINATOR_BYTES
            + (32 * 6)
            + (8 * 7)
            + 3
            + 1
            + GREEN_LABEL_DISPUTE_RESERVED_BYTES;

        assert!(expected_green_label_dispute_space() >= minimum);
        assert_eq!(
            GreenLabelDisputeV1::INIT_SPACE + ANCHOR_ACCOUNT_DISCRIMINATOR_BYTES,
            expected_green_label_dispute_space()
        );
    }

    #[test]
    fn green_label_config_reserved_space_is_128() {
        assert_eq!(
            green_label_config().reserved.len(),
            GREEN_LABEL_CONFIG_RESERVED_BYTES
        );
        assert_eq!(GREEN_LABEL_CONFIG_RESERVED_BYTES, 128);
    }

    #[test]
    fn green_label_project_reserved_space_is_160() {
        assert_eq!(
            green_label_project().reserved.len(),
            GREEN_LABEL_PROJECT_RESERVED_BYTES
        );
        assert_eq!(GREEN_LABEL_PROJECT_RESERVED_BYTES, 160);
    }

    #[test]
    fn green_label_dispute_reserved_space_is_128() {
        assert_eq!(
            green_label_dispute().reserved.len(),
            GREEN_LABEL_DISPUTE_RESERVED_BYTES
        );
        assert_eq!(GREEN_LABEL_DISPUTE_RESERVED_BYTES, 128);
    }

    #[test]
    fn derive_bond_split_and_tier_for_299() {
        let (split, tier) = derive_bond_split_and_tier(299_000_000).unwrap();

        assert_eq!(split.base_bond_amount, 299_000_000);
        assert_eq!(split.extra_bond_amount, 0);
        assert_eq!(tier, BondTier::Base);
    }

    #[test]
    fn derive_bond_split_and_tier_for_1299() {
        let (split, tier) = derive_bond_split_and_tier(1_299_000_000).unwrap();

        assert_eq!(split.base_bond_amount, 299_000_000);
        assert_eq!(split.extra_bond_amount, 1_000_000_000);
        assert_eq!(tier, BondTier::Silver);
    }

    #[test]
    fn validate_terminal_refund_accepts_green_label_refund() {
        validate_terminal_action_for_refund(ActionType::GreenLabelRefund).unwrap();
    }

    #[test]
    fn validate_terminal_refund_rejects_green_label_slash() {
        let err = validate_terminal_action_for_refund(ActionType::GreenLabelSlash).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelActionType");
    }

    #[test]
    fn validate_terminal_slash_accepts_green_label_slash_with_dispute() {
        validate_terminal_action_for_slash(ActionType::GreenLabelSlash, true).unwrap();
    }

    #[test]
    fn validate_terminal_slash_rejects_green_label_slash_without_dispute() {
        let err =
            validate_terminal_action_for_slash(ActionType::GreenLabelSlash, false).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelSlashWithoutDispute");
    }

    #[test]
    fn validate_terminal_slash_rejects_green_label_refund() {
        let err =
            validate_terminal_action_for_slash(ActionType::GreenLabelRefund, true).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelActionType");
    }

    #[test]
    fn account_structs_are_clone_debug_serializable_if_possible() {
        let mut config_data = Vec::new();
        green_label_config()
            .try_serialize(&mut config_data)
            .unwrap();
        assert!(!config_data.is_empty());

        let mut project_data = Vec::new();
        green_label_project()
            .try_serialize(&mut project_data)
            .unwrap();
        assert!(!project_data.is_empty());

        let mut dispute_data = Vec::new();
        green_label_dispute()
            .try_serialize(&mut dispute_data)
            .unwrap();
        assert!(!dispute_data.is_empty());

        let tier = BondTier::Base;
        let tier_copy = tier;
        assert_eq!(format!("{tier_copy:?}"), "Base");
    }

    #[test]
    fn pda_seed_constants_are_non_empty_and_distinct() {
        let seeds = [
            GREEN_LABEL_CONFIG_SEED,
            GREEN_LABEL_PROJECT_SEED,
            GREEN_LABEL_DISPUTE_SEED,
            GREEN_BOND_VAULT_SEED,
            GREEN_BOND_VAULT_AUTHORITY_SEED,
        ];

        for seed in seeds {
            assert!(!seed.is_empty());
        }

        for (index, seed) in seeds.iter().enumerate() {
            for other in seeds.iter().skip(index + 1) {
                assert_ne!(seed, other);
            }
        }
    }

    fn green_label_config() -> GreenLabelConfigV1 {
        GreenLabelConfigV1 {
            authority: Pubkey::new_from_array([1; 32]),
            usdc_mint: Pubkey::new_from_array([2; 32]),
            min_base_bond_usdc: MIN_GREEN_LABEL_BASE_BOND_USDC,
            base_refund_bps: BASE_BOND_REFUND_BPS,
            base_treasury_bps: BASE_BOND_TREASURY_BPS,
            observation_period_seconds: 30,
            dispute_window_seconds: 7,
            response_window_seconds: 3,
            project_count: 0,
            treasury_usdc_state_v2: Pubkey::new_from_array([3; 32]),
            base_bond_treasury_vault: Pubkey::new_from_array([4; 32]),
            relief_or_risk_vault: Pubkey::new_from_array([5; 32]),
            vault_authority_v2: Pubkey::new_from_array([6; 32]),
            security_governance_config: Pubkey::new_from_array([7; 32]),
            is_paused: false,
            bump: 250,
            reserved: [0; GREEN_LABEL_CONFIG_RESERVED_BYTES],
        }
    }

    fn green_label_config_init_values() -> GreenLabelConfigInitValues {
        build_default_green_label_config_values(
            Pubkey::new_from_array([1; 32]),
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([3; 32]),
            Pubkey::new_from_array([4; 32]),
            Pubkey::new_from_array([5; 32]),
            Pubkey::new_from_array([6; 32]),
            Pubkey::new_from_array([7; 32]),
            250,
        )
        .unwrap()
    }

    fn green_label_project() -> GreenLabelProjectV1 {
        GreenLabelProjectV1 {
            project_id: 1,
            project_owner: Pubkey::new_from_array([8; 32]),
            project_name_hash: [9; 32],
            project_url_hash: [10; 32],
            token_mint: Pubkey::new_from_array([11; 32]),
            project_treasury_wallet: Pubkey::new_from_array([12; 32]),
            base_bond_amount: 299_000_000,
            extra_bond_amount: 1_000_000_000,
            total_bond_amount: 1_299_000_000,
            bond_vault: Pubkey::new_from_array([13; 32]),
            bond_vault_authority: Pubkey::new_from_array([14; 32]),
            bond_tier: BondTier::Silver,
            status: GreenLabelStatus::RefundQueued,
            submitted_at: 1,
            observation_start_ts: 2,
            observation_end_ts: 3,
            dispute_count: 0,
            active_dispute: Pubkey::default(),
            approved_at: 0,
            refunded_at: 0,
            slashed_at: 0,
            risk_score_snapshot: 0,
            terminal_proposal_id: 1,
            terminal_proposal_decision: Pubkey::new_from_array([15; 32]),
            terminal_execution_queue_item: Pubkey::new_from_array([16; 32]),
            terminal_payload_hash: [17; 32],
            terminal_action_type: ActionType::GreenLabelRefund,
            bump: 249,
            reserved: [0; GREEN_LABEL_PROJECT_RESERVED_BYTES],
        }
    }

    fn green_label_dispute() -> GreenLabelDisputeV1 {
        GreenLabelDisputeV1 {
            project_id: 1,
            dispute_id: 1,
            project: Pubkey::new_from_array([18; 32]),
            disputer: Pubkey::new_from_array([19; 32]),
            reason_code: RugReasonCode::LiquidityRemoved,
            evidence_hash: [20; 32],
            status: DisputeStatus::DecisionQueued,
            opened_at: 1,
            evidence_end_ts: 2,
            response_end_ts: 3,
            resolved_at: 0,
            proposal_id: 1,
            proposal_decision: Pubkey::new_from_array([21; 32]),
            execution_queue_item: Pubkey::new_from_array([22; 32]),
            payload_hash: [23; 32],
            action_type: ActionType::GreenLabelSlash,
            bump: 248,
            reserved: [0; GREEN_LABEL_DISPUTE_RESERVED_BYTES],
        }
    }
}
