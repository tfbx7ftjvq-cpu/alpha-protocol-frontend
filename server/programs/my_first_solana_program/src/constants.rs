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

pub const BPS_DENOMINATOR: u64 = 10_000;
pub const RELIEF_BPS: u64 = 5_000;
pub const BUYBACK_BPS: u64 = 2_000;
pub const PAYROLL_BPS: u64 = 2_000;
pub const STAKING_BPS: u64 = 1_000;
