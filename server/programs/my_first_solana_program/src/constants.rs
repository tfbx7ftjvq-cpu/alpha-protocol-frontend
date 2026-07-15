use anchor_lang::prelude::*;

#[constant]
pub const SEED: &str = "anchor";

pub const TREASURY_STATE_SEED: &[u8] = b"treasury_state";
pub const TREASURY_CONFIG_V2_SEED: &[u8] = b"treasury_config_v2";
pub const TREASURY_USDC_STATE_V2_SEED: &[u8] = b"treasury_usdc_state_v2";
pub const REVENUE_ROUTING_STATS_V1_SEED: &[u8] = b"revenue_routing_stats_v1";
pub const TREASURY_GOVERNANCE_CONFIG_V1_SEED: &[u8] = b"treasury_governance_config_v1";
pub const TREASURY_SPENDING_REQUEST_V1_SEED: &[u8] = b"treasury_spending_request_v1";
pub const TREASURY_BUILDER_PAYOUT_GOVERNANCE_V1_SEED: &[u8] =
    b"treasury_builder_payout_governance_v1";
pub const TREASURY_EXECUTION_RECORD_V1_SEED: &[u8] = b"treasury_execution_record_v1";
pub const RELIEF_USDC_VAULT_SEED: &[u8] = b"relief_usdc_vault";
pub const BUYBACK_USDC_VAULT_SEED: &[u8] = b"buyback_usdc_vault";
pub const BUILDERS_USDC_VAULT_SEED: &[u8] = b"builders_usdc_vault";
pub const STAKING_USDC_VAULT_SEED: &[u8] = b"staking_usdc_vault";
pub const VAULT_AUTHORITY_V2_SEED: &[u8] = b"vault_authority_v2";
pub const STAKING_POOL_V1_SEED: &[u8] = b"staking_pool_v1";
pub const ALPHA_STAKING_VAULT_SEED: &[u8] = b"alpha_staking_vault";
pub const ALPHA_VAULT_AUTHORITY_V1_SEED: &[u8] = b"alpha_vault_authority_v1";
pub const USER_STAKE_ACCOUNT_SEED: &[u8] = b"user_stake_account";
pub const GOVERNANCE_CONFIG_V1_SEED: &[u8] = b"governance_config_v1";
pub const GOVERNANCE_LOCK_CONFIG_V1_SEED: &[u8] = b"governance_lock_config_v1";
pub const GOVERNANCE_VOTING_CONFIG_V1_SEED: &[u8] = b"governance_voting_config_v1";
pub const GOVERNANCE_VAULT_V1_SEED: &[u8] = b"governance_vault_v1";
pub const GOVERNANCE_POWER_STATE_V1_SEED: &[u8] = b"governance_power_state_v1";
pub const GOVERNANCE_PROPOSAL_V1_SEED: &[u8] = b"governance_proposal_v1";
pub const GOVERNANCE_PROPOSAL_ACTION_V1_SEED: &[u8] = b"governance_proposal_action_v1";
pub const PROTOCOL_MODULE_REGISTRY_V1_SEED: &[u8] = b"protocol_module_registry_v1";
pub const GOVERNANCE_POSITION_V1_SEED: &[u8] = b"governance_position_v1";
pub const GOVERNANCE_POSITION_VOTE_LOCK_V1_SEED: &[u8] = b"governance_position_vote_lock_v1";
pub const GOVERNANCE_SNAPSHOT_V1_SEED: &[u8] = b"governance_snapshot_v1";
pub const VOTE_RECORD_V1_SEED: &[u8] = b"vote_record_v1";
pub const UNIVERSAL_GOVERNANCE_DECISION_ADAPTER_V1_SEED: &[u8] =
    b"universal_governance_decision_adapter_v1";
pub const PROPOSAL_DECISION_V1_SEED: &[u8] = b"proposal_decision_v1";
pub const EXECUTION_QUEUE_ITEM_V1_SEED: &[u8] = b"execution_queue_item_v1";
pub const GREEN_LABEL_CONFIG_SEED: &[u8] = b"green_label_config_v1";
pub const GREEN_LABEL_PROJECT_SEED: &[u8] = b"green_label_project_v1";
pub const GREEN_LABEL_DISPUTE_SEED: &[u8] = b"green_label_dispute_v1";
pub const GREEN_BOND_VAULT_SEED: &[u8] = b"green_bond_vault_v1";
pub const GREEN_BOND_VAULT_AUTHORITY_SEED: &[u8] = b"green_bond_vault_authority_v1";
pub const GREEN_LABEL_REFUNDABLE_ESCROW_SEED: &[u8] = b"green_label_refundable_escrow_v1";
pub const GREEN_LABEL_REFUNDABLE_VAULT_SEED: &[u8] = b"green_label_refundable_vault_v1";
pub const GREEN_LABEL_CERTIFICATION_STATE_SEED: &[u8] = b"green_label_certification_state_v1";
pub const GREEN_LABEL_CERTIFICATION_FEE_POLICY_SEED: &[u8] =
    b"green_label_certification_fee_policy_v1";
pub const GREEN_LABEL_CERTIFICATION_FEE_RECEIPT_SEED: &[u8] =
    b"green_label_certification_fee_receipt_v1";
pub const GREEN_LABEL_CERTIFICATION_EXECUTION_RECORD_SEED: &[u8] =
    b"green_label_certification_execution_record_v1";
pub const GREEN_LABEL_REFUND_EXECUTION_RECORD_SEED: &[u8] =
    b"green_label_refund_execution_record_v1";
pub const GREEN_LABEL_FORFEIT_EXECUTION_RECORD_SEED: &[u8] =
    b"green_label_forfeit_execution_record_v1";
pub const CONTRIBUTOR_REGISTRY_V1_SEED: &[u8] = b"contributor_registry_v1";
pub const CONTRIBUTOR_MILESTONE_V1_SEED: &[u8] = b"contributor_milestone_v1";
pub const BUILDER_PAYOUT_REQUEST_V1_SEED: &[u8] = b"builder_payout_request_v1";

pub const BPS_DENOMINATOR: u64 = 10_000;
pub const RELIEF_BPS: u64 = 5_000;
pub const BUYBACK_BPS: u64 = 2_000;
pub const PAYROLL_BPS: u64 = 2_000;
pub const STAKING_BPS: u64 = 1_000;

pub const REWARD_INDEX_SCALE: u128 = 1_000_000_000_000;
pub const STAKING_PHASE1_REWARD_RELEASE_BPS: u16 = 10_000;
pub const DEFAULT_MIN_CLAIM_USDC: u64 = 100_000;

pub const LOCK_TIER_FLEXIBLE: u8 = 0;
pub const LOCK_TIER_30_DAYS: u8 = 1;
pub const LOCK_TIER_90_DAYS: u8 = 2;
pub const LOCK_TIER_180_DAYS: u8 = 3;
pub const LOCK_TIER_365_DAYS: u8 = 4;

pub const FLEXIBLE_MULTIPLIER_BPS: u16 = 6_000;
pub const LOCK_30_DAYS_MULTIPLIER_BPS: u16 = 10_000;
pub const LOCK_90_DAYS_MULTIPLIER_BPS: u16 = 13_500;
pub const LOCK_180_DAYS_MULTIPLIER_BPS: u16 = 18_000;
pub const LOCK_365_DAYS_MULTIPLIER_BPS: u16 = 25_000;

pub const SECONDS_PER_DAY: i64 = 86_400;
pub const LOCK_30_DAYS_SECONDS: i64 = 30 * SECONDS_PER_DAY;
pub const LOCK_90_DAYS_SECONDS: i64 = 90 * SECONDS_PER_DAY;
pub const LOCK_180_DAYS_SECONDS: i64 = 180 * SECONDS_PER_DAY;
pub const LOCK_365_DAYS_SECONDS: i64 = 365 * SECONDS_PER_DAY;

pub const GOVERNANCE_DEFAULT_MIN_LOCK_AMOUNT: u64 = 1;
pub const GOVERNANCE_MIN_LOCK_DURATION_SECONDS: i64 = 30 * SECONDS_PER_DAY;
pub const GOVERNANCE_MAX_LOCK_DURATION_SECONDS: i64 = 365 * SECONDS_PER_DAY;
pub const GOVERNANCE_30_DAY_MULTIPLIER_BPS: u64 = 10_000;
pub const GOVERNANCE_90_DAY_MULTIPLIER_BPS: u64 = 11_000;
pub const GOVERNANCE_180_DAY_MULTIPLIER_BPS: u64 = 15_000;
pub const GOVERNANCE_365_DAY_MULTIPLIER_BPS: u64 = 20_000;
pub const GOVERNANCE_MAX_TIME_MULTIPLIER_BPS: u64 = GOVERNANCE_365_DAY_MULTIPLIER_BPS;
pub const GOVERNANCE_DEFAULT_QUORUM_BPS: u64 = 500;
pub const GOVERNANCE_DEFAULT_APPROVAL_THRESHOLD_BPS: u64 = 6_000;
pub const GOVERNANCE_DEFAULT_VOTING_PERIOD_SECONDS: i64 = 7 * SECONDS_PER_DAY;

pub const MIN_EXECUTION_DELAY_SECONDS: i64 = 60;
pub const MAX_EXECUTION_DELAY_SECONDS: i64 = 30 * SECONDS_PER_DAY;

pub const MIN_GREEN_LABEL_BASE_BOND_USDC: u64 = 299_000_000;
pub const BASE_BOND_REFUND_BPS: u16 = 8_000;
pub const BASE_BOND_TREASURY_BPS: u16 = 2_000;
pub const DEFAULT_OBSERVATION_PERIOD_SECONDS: i64 = 30 * SECONDS_PER_DAY;
pub const DEFAULT_DISPUTE_WINDOW_SECONDS: i64 = 7 * SECONDS_PER_DAY;
pub const DEFAULT_RESPONSE_WINDOW_SECONDS: i64 = 3 * SECONDS_PER_DAY;
pub const GREEN_LABEL_USDC_DECIMALS: u8 = 6;
pub const MAX_BPS: u16 = 10_000;

pub const GREEN_LABEL_BASE_TIER_THRESHOLD_USDC: u64 = 299_000_000;
pub const GREEN_LABEL_BRONZE_TIER_THRESHOLD_USDC: u64 = 500_000_000;
pub const GREEN_LABEL_SILVER_TIER_THRESHOLD_USDC: u64 = 1_000_000_000;
pub const GREEN_LABEL_GOLD_TIER_THRESHOLD_USDC: u64 = 3_000_000_000;
pub const GREEN_LABEL_PLATINUM_TIER_THRESHOLD_USDC: u64 = 10_000_000_000;

pub const ANCHOR_ACCOUNT_DISCRIMINATOR_BYTES: usize = 8;
pub const GREEN_LABEL_CONFIG_RESERVED_BYTES: usize = 128;
pub const GREEN_LABEL_PROJECT_RESERVED_BYTES: usize = 160;
pub const GREEN_LABEL_DISPUTE_RESERVED_BYTES: usize = 128;

pub const GREEN_LABEL_CONFIG_SPACE: usize = ANCHOR_ACCOUNT_DISCRIMINATOR_BYTES
    + (32 * 7)
    + (8 * 2)
    + (2 * 2)
    + (8 * 3)
    + 1
    + 1
    + GREEN_LABEL_CONFIG_RESERVED_BYTES;

pub const GREEN_LABEL_PROJECT_SPACE: usize = ANCHOR_ACCOUNT_DISCRIMINATOR_BYTES
    + (32 * 11)
    + (8 * 12)
    + 1
    + 1
    + 2
    + 1
    + 1
    + GREEN_LABEL_PROJECT_RESERVED_BYTES;

pub const GREEN_LABEL_DISPUTE_SPACE: usize = ANCHOR_ACCOUNT_DISCRIMINATOR_BYTES
    + (32 * 6)
    + (8 * 7)
    + 3
    + 1
    + GREEN_LABEL_DISPUTE_RESERVED_BYTES;

pub const CONTRIBUTOR_MILESTONE_TITLE_MAX_BYTES: usize = 96;
pub const CONTRIBUTOR_MILESTONE_DESCRIPTION_MAX_BYTES: usize = 512;
