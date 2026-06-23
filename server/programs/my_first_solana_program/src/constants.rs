use anchor_lang::prelude::*;

#[constant]
pub const SEED: &str = "anchor";

pub const TREASURY_STATE_SEED: &[u8] = b"treasury_state";
pub const TREASURY_CONFIG_V2_SEED: &[u8] = b"treasury_config_v2";
pub const TREASURY_USDC_STATE_V2_SEED: &[u8] = b"treasury_usdc_state_v2";
pub const RELIEF_USDC_VAULT_SEED: &[u8] = b"relief_usdc_vault";
pub const BUYBACK_USDC_VAULT_SEED: &[u8] = b"buyback_usdc_vault";
pub const BUILDERS_USDC_VAULT_SEED: &[u8] = b"builders_usdc_vault";
pub const STAKING_USDC_VAULT_SEED: &[u8] = b"staking_usdc_vault";
pub const VAULT_AUTHORITY_V2_SEED: &[u8] = b"vault_authority_v2";
pub const STAKING_POOL_V1_SEED: &[u8] = b"staking_pool_v1";
pub const ALPHA_STAKING_VAULT_SEED: &[u8] = b"alpha_staking_vault";
pub const ALPHA_VAULT_AUTHORITY_V1_SEED: &[u8] = b"alpha_vault_authority_v1";
pub const USER_STAKE_ACCOUNT_SEED: &[u8] = b"user_stake_account";

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
