use anchor_lang::prelude::*;

#[account]
pub struct TreasuryState {
    pub authority: Pubkey,
    pub total_inflow: u64,
    pub relief_pool: u64,
    pub buyback_pool: u64,
    pub payroll_pool: u64,
    pub staking_pool: u64,
    pub bump: u8,
}

impl TreasuryState {
    pub const INIT_SPACE: usize = 32 + (8 * 5) + 1;
}

#[account]
pub struct TreasuryConfigV2 {
    pub authority: Pubkey,
    pub usdc_mint: Pubkey,
    pub alpha_mint: Pubkey,
    pub bump: u8,
}

impl TreasuryConfigV2 {
    pub const INIT_SPACE: usize = (32 * 3) + 1;
}

#[account]
pub struct TreasuryUsdcStateV2 {
    pub total_usdc_inflow: u64,
    pub relief_usdc_total: u64,
    pub buyback_usdc_total: u64,
    pub builders_usdc_total: u64,
    pub staking_usdc_total: u64,
    pub bump: u8,
}

impl TreasuryUsdcStateV2 {
    pub const INIT_SPACE: usize = (8 * 5) + 1;
}
