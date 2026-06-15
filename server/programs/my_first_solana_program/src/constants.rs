use anchor_lang::prelude::*;

#[constant]
pub const SEED: &str = "anchor";

pub const TREASURY_STATE_SEED: &[u8] = b"treasury_state";

pub const BPS_DENOMINATOR: u64 = 10_000;
pub const RELIEF_BPS: u64 = 5_000;
pub const BUYBACK_BPS: u64 = 2_000;
pub const PAYROLL_BPS: u64 = 2_000;
pub const STAKING_BPS: u64 = 1_000;
