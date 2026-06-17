use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::constants::{
    BUILDERS_USDC_VAULT_SEED, BUYBACK_USDC_VAULT_SEED, RELIEF_USDC_VAULT_SEED,
    STAKING_USDC_VAULT_SEED, TREASURY_CONFIG_V2_SEED, TREASURY_USDC_STATE_V2_SEED,
    VAULT_AUTHORITY_V2_SEED,
};
use crate::error::CustomError;
use crate::state::{TreasuryConfigV2, TreasuryUsdcStateV2};

#[derive(Accounts)]
#[instruction(usdc_mint: Pubkey, alpha_mint: Pubkey)]
pub struct InitializeUsdcTreasury<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + TreasuryConfigV2::INIT_SPACE,
        seeds = [TREASURY_CONFIG_V2_SEED],
        bump
    )]
    pub treasury_config: Account<'info, TreasuryConfigV2>,

    #[account(
        init,
        payer = authority,
        space = 8 + TreasuryUsdcStateV2::INIT_SPACE,
        seeds = [TREASURY_USDC_STATE_V2_SEED],
        bump
    )]
    pub treasury_usdc_state: Account<'info, TreasuryUsdcStateV2>,

    #[account(
        constraint = usdc_mint_account.key() == usdc_mint @ CustomError::InvalidMint
    )]
    pub usdc_mint_account: Box<Account<'info, Mint>>,

    /// CHECK: This PDA only owns the USDC vault token accounts.
    #[account(
        seeds = [VAULT_AUTHORITY_V2_SEED],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = authority,
        token::mint = usdc_mint_account,
        token::authority = vault_authority,
        seeds = [RELIEF_USDC_VAULT_SEED],
        bump
    )]
    pub relief_usdc_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = authority,
        token::mint = usdc_mint_account,
        token::authority = vault_authority,
        seeds = [BUYBACK_USDC_VAULT_SEED],
        bump
    )]
    pub buyback_usdc_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = authority,
        token::mint = usdc_mint_account,
        token::authority = vault_authority,
        seeds = [BUILDERS_USDC_VAULT_SEED],
        bump
    )]
    pub builders_usdc_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = authority,
        token::mint = usdc_mint_account,
        token::authority = vault_authority,
        seeds = [STAKING_USDC_VAULT_SEED],
        bump
    )]
    pub staking_usdc_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn initialize_usdc_treasury_handler(
    ctx: Context<InitializeUsdcTreasury>,
    usdc_mint: Pubkey,
    alpha_mint: Pubkey,
) -> Result<()> {
    let treasury_config = &mut ctx.accounts.treasury_config;
    treasury_config.authority = ctx.accounts.authority.key();
    treasury_config.usdc_mint = usdc_mint;
    treasury_config.alpha_mint = alpha_mint;
    treasury_config.bump = ctx.bumps.treasury_config;

    let treasury_usdc_state = &mut ctx.accounts.treasury_usdc_state;
    treasury_usdc_state.total_usdc_inflow = 0;
    treasury_usdc_state.relief_usdc_total = 0;
    treasury_usdc_state.buyback_usdc_total = 0;
    treasury_usdc_state.builders_usdc_total = 0;
    treasury_usdc_state.staking_usdc_total = 0;
    treasury_usdc_state.bump = ctx.bumps.treasury_usdc_state;

    Ok(())
}
