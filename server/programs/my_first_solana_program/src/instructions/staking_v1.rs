use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};

use crate::constants::{
    ALPHA_STAKING_VAULT_SEED, ALPHA_VAULT_AUTHORITY_V1_SEED, BPS_DENOMINATOR,
    DEFAULT_MIN_CLAIM_USDC, FLEXIBLE_MULTIPLIER_BPS, LOCK_180_DAYS_MULTIPLIER_BPS,
    LOCK_180_DAYS_SECONDS, LOCK_30_DAYS_MULTIPLIER_BPS, LOCK_30_DAYS_SECONDS,
    LOCK_365_DAYS_MULTIPLIER_BPS, LOCK_365_DAYS_SECONDS, LOCK_90_DAYS_MULTIPLIER_BPS,
    LOCK_90_DAYS_SECONDS, LOCK_TIER_180_DAYS, LOCK_TIER_30_DAYS, LOCK_TIER_365_DAYS,
    LOCK_TIER_90_DAYS, LOCK_TIER_FLEXIBLE, REWARD_INDEX_SCALE, STAKING_PHASE1_REWARD_RELEASE_BPS,
    STAKING_POOL_V1_SEED, USER_STAKE_ACCOUNT_SEED, VAULT_AUTHORITY_V2_SEED,
};
use crate::error::CustomError;
use crate::state::{StakingPoolV1, UserStakeAccount};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LockTierConfig {
    pub multiplier_bps: u16,
    pub duration_seconds: i64,
}

#[derive(Accounts)]
pub struct InitializeStakingPool<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + StakingPoolV1::INIT_SPACE,
        seeds = [STAKING_POOL_V1_SEED],
        bump
    )]
    pub staking_pool: Account<'info, StakingPoolV1>,

    pub alpha_mint: Box<Account<'info, Mint>>,
    pub usdc_mint: Box<Account<'info, Mint>>,

    /// CHECK: This PDA only owns the ALPHA staking vault.
    #[account(
        seeds = [ALPHA_VAULT_AUTHORITY_V1_SEED],
        bump
    )]
    pub alpha_vault_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = authority,
        token::mint = alpha_mint,
        token::authority = alpha_vault_authority,
        seeds = [ALPHA_STAKING_VAULT_SEED],
        bump
    )]
    pub alpha_vault: Box<Account<'info, TokenAccount>>,

    /// CHECK: Existing Treasury V2 PDA that owns the staking USDC vault.
    #[account(
        seeds = [VAULT_AUTHORITY_V2_SEED],
        bump
    )]
    pub vault_authority_v2: UncheckedAccount<'info>,

    #[account(
        constraint = staking_usdc_vault.mint == usdc_mint.key() @ CustomError::InvalidMint,
        constraint = staking_usdc_vault.owner == vault_authority_v2.key() @ CustomError::InvalidVault
    )]
    pub staking_usdc_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct StakeAlpha<'info> {
    #[account(
        mut,
        seeds = [STAKING_POOL_V1_SEED],
        bump = staking_pool.bump
    )]
    pub staking_pool: Account<'info, StakingPoolV1>,

    #[account(
        init_if_needed,
        payer = owner,
        space = 8 + UserStakeAccount::INIT_SPACE,
        seeds = [USER_STAKE_ACCOUNT_SEED, owner.key().as_ref()],
        bump
    )]
    pub user_stake_account: Account<'info, UserStakeAccount>,

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        constraint = owner_alpha_token_account.mint == staking_pool.alpha_mint @ CustomError::InvalidMint,
        constraint = owner_alpha_token_account.owner == owner.key() @ CustomError::InvalidTokenAccount
    )]
    pub owner_alpha_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = alpha_mint.key() == staking_pool.alpha_mint @ CustomError::InvalidMint
    )]
    pub alpha_mint: Box<Account<'info, Mint>>,

    /// CHECK: This PDA only owns the ALPHA staking vault.
    #[account(
        seeds = [ALPHA_VAULT_AUTHORITY_V1_SEED],
        bump = staking_pool.alpha_vault_authority_bump
    )]
    pub alpha_vault_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        constraint = alpha_vault.key() == staking_pool.alpha_vault @ CustomError::InvalidVault,
        constraint = alpha_vault.mint == staking_pool.alpha_mint @ CustomError::InvalidMint,
        constraint = alpha_vault.owner == alpha_vault_authority.key() @ CustomError::InvalidVault
    )]
    pub alpha_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = staking_usdc_vault.key() == staking_pool.staking_usdc_vault @ CustomError::InvalidVault,
        constraint = staking_usdc_vault.mint == staking_pool.usdc_mint @ CustomError::InvalidMint,
        constraint = staking_usdc_vault.owner == staking_pool.vault_authority_v2 @ CustomError::InvalidVault
    )]
    pub staking_usdc_vault: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ClaimUsdcRewards<'info> {
    #[account(
        mut,
        seeds = [STAKING_POOL_V1_SEED],
        bump = staking_pool.bump
    )]
    pub staking_pool: Account<'info, StakingPoolV1>,

    #[account(
        mut,
        seeds = [USER_STAKE_ACCOUNT_SEED, owner.key().as_ref()],
        bump = user_stake_account.bump,
        constraint = user_stake_account.owner == owner.key() @ CustomError::InvalidStakeOwner
    )]
    pub user_stake_account: Account<'info, UserStakeAccount>,

    pub owner: Signer<'info>,

    #[account(
        mut,
        constraint = staking_usdc_vault.key() == staking_pool.staking_usdc_vault @ CustomError::InvalidVault,
        constraint = staking_usdc_vault.mint == staking_pool.usdc_mint @ CustomError::InvalidMint,
        constraint = staking_usdc_vault.owner == vault_authority_v2.key() @ CustomError::InvalidVault
    )]
    pub staking_usdc_vault: Box<Account<'info, TokenAccount>>,

    /// CHECK: Existing Treasury V2 PDA that owns the staking USDC vault.
    #[account(
        seeds = [VAULT_AUTHORITY_V2_SEED],
        bump = staking_pool.vault_authority_v2_bump,
        constraint = vault_authority_v2.key() == staking_pool.vault_authority_v2 @ CustomError::InvalidVault
    )]
    pub vault_authority_v2: UncheckedAccount<'info>,

    #[account(
        mut,
        constraint = owner_usdc_token_account.mint == staking_pool.usdc_mint @ CustomError::InvalidMint,
        constraint = owner_usdc_token_account.owner == owner.key() @ CustomError::InvalidTokenAccount
    )]
    pub owner_usdc_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = usdc_mint.key() == staking_pool.usdc_mint @ CustomError::InvalidMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UnstakeAlpha<'info> {
    #[account(
        mut,
        seeds = [STAKING_POOL_V1_SEED],
        bump = staking_pool.bump
    )]
    pub staking_pool: Account<'info, StakingPoolV1>,

    #[account(
        mut,
        seeds = [USER_STAKE_ACCOUNT_SEED, owner.key().as_ref()],
        bump = user_stake_account.bump,
        constraint = user_stake_account.owner == owner.key() @ CustomError::InvalidStakeOwner
    )]
    pub user_stake_account: Account<'info, UserStakeAccount>,

    pub owner: Signer<'info>,

    #[account(
        mut,
        constraint = owner_alpha_token_account.mint == staking_pool.alpha_mint @ CustomError::InvalidMint,
        constraint = owner_alpha_token_account.owner == owner.key() @ CustomError::InvalidTokenAccount
    )]
    pub owner_alpha_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = alpha_mint.key() == staking_pool.alpha_mint @ CustomError::InvalidMint
    )]
    pub alpha_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        constraint = alpha_vault.key() == staking_pool.alpha_vault @ CustomError::InvalidVault,
        constraint = alpha_vault.mint == staking_pool.alpha_mint @ CustomError::InvalidMint,
        constraint = alpha_vault.owner == alpha_vault_authority.key() @ CustomError::InvalidVault
    )]
    pub alpha_vault: Box<Account<'info, TokenAccount>>,

    /// CHECK: This PDA only owns the ALPHA staking vault.
    #[account(
        seeds = [ALPHA_VAULT_AUTHORITY_V1_SEED],
        bump = staking_pool.alpha_vault_authority_bump,
        constraint = alpha_vault_authority.key() == staking_pool.alpha_vault_authority @ CustomError::InvalidVault
    )]
    pub alpha_vault_authority: UncheckedAccount<'info>,

    #[account(
        constraint = staking_usdc_vault.key() == staking_pool.staking_usdc_vault @ CustomError::InvalidVault,
        constraint = staking_usdc_vault.mint == staking_pool.usdc_mint @ CustomError::InvalidMint,
        constraint = staking_usdc_vault.owner == staking_pool.vault_authority_v2 @ CustomError::InvalidVault
    )]
    pub staking_usdc_vault: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

pub fn initialize_staking_pool_handler(
    ctx: Context<InitializeStakingPool>,
    min_claim_usdc: u64,
) -> Result<()> {
    let clock = Clock::get()?;
    let min_claim_usdc = if min_claim_usdc == 0 {
        DEFAULT_MIN_CLAIM_USDC
    } else {
        min_claim_usdc
    };

    let staking_pool = &mut ctx.accounts.staking_pool;
    staking_pool.authority = ctx.accounts.authority.key();
    staking_pool.alpha_mint = ctx.accounts.alpha_mint.key();
    staking_pool.usdc_mint = ctx.accounts.usdc_mint.key();
    staking_pool.alpha_vault = ctx.accounts.alpha_vault.key();
    staking_pool.alpha_vault_authority = ctx.accounts.alpha_vault_authority.key();
    staking_pool.staking_usdc_vault = ctx.accounts.staking_usdc_vault.key();
    staking_pool.vault_authority_v2 = ctx.accounts.vault_authority_v2.key();
    staking_pool.total_staked_alpha = 0;
    staking_pool.total_effective_weight = 0;
    staking_pool.acc_usdc_per_weight = 0;
    staking_pool.last_reward_update_ts = clock.unix_timestamp;
    staking_pool.last_observed_usdc_balance = ctx.accounts.staking_usdc_vault.amount;
    staking_pool.reward_release_bps = STAKING_PHASE1_REWARD_RELEASE_BPS;
    staking_pool.min_claim_usdc = min_claim_usdc;
    staking_pool.vault_authority_v2_bump = ctx.bumps.vault_authority_v2;
    staking_pool.alpha_vault_authority_bump = ctx.bumps.alpha_vault_authority;
    staking_pool.bump = ctx.bumps.staking_pool;

    Ok(())
}

pub fn stake_alpha_handler(ctx: Context<StakeAlpha>, amount: u64, lock_tier: u8) -> Result<()> {
    let clock = Clock::get()?;
    update_pool_rewards_from_balance(
        &mut ctx.accounts.staking_pool,
        ctx.accounts.staking_usdc_vault.amount,
        clock.unix_timestamp,
    )?;

    apply_stake_state(
        &mut ctx.accounts.staking_pool,
        &mut ctx.accounts.user_stake_account,
        ctx.accounts.owner.key(),
        amount,
        lock_tier,
        clock.unix_timestamp,
        ctx.bumps.user_stake_account,
    )?;

    let cpi_accounts = TransferChecked {
        from: ctx.accounts.owner_alpha_token_account.to_account_info(),
        mint: ctx.accounts.alpha_mint.to_account_info(),
        to: ctx.accounts.alpha_vault.to_account_info(),
        authority: ctx.accounts.owner.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.key(), cpi_accounts);

    transfer_checked(cpi_ctx, amount, ctx.accounts.alpha_mint.decimals)
}

pub fn claim_usdc_rewards_handler(ctx: Context<ClaimUsdcRewards>) -> Result<()> {
    let clock = Clock::get()?;
    update_pool_rewards_from_balance(
        &mut ctx.accounts.staking_pool,
        ctx.accounts.staking_usdc_vault.amount,
        clock.unix_timestamp,
    )?;

    let claim_amount =
        calculate_claim_amount(&ctx.accounts.staking_pool, &ctx.accounts.user_stake_account)?;
    require!(
        claim_amount >= ctx.accounts.staking_pool.min_claim_usdc,
        CustomError::ClaimAmountTooSmall
    );
    validate_vault_balance_for_claim(ctx.accounts.staking_usdc_vault.amount, claim_amount)?;

    let vault_authority_bump = ctx.accounts.staking_pool.vault_authority_v2_bump;
    let signer_seeds: &[&[&[u8]]] = &[&[VAULT_AUTHORITY_V2_SEED, &[vault_authority_bump]]];
    let cpi_accounts = TransferChecked {
        from: ctx.accounts.staking_usdc_vault.to_account_info(),
        mint: ctx.accounts.usdc_mint.to_account_info(),
        to: ctx.accounts.owner_usdc_token_account.to_account_info(),
        authority: ctx.accounts.vault_authority_v2.to_account_info(),
    };
    let cpi_ctx =
        CpiContext::new_with_signer(ctx.accounts.token_program.key(), cpi_accounts, signer_seeds);

    transfer_checked(cpi_ctx, claim_amount, ctx.accounts.usdc_mint.decimals)?;
    apply_claim_state(
        &mut ctx.accounts.staking_pool,
        &mut ctx.accounts.user_stake_account,
        claim_amount,
    )
}

pub fn unstake_alpha_handler(ctx: Context<UnstakeAlpha>, amount: u64) -> Result<()> {
    let clock = Clock::get()?;
    update_pool_rewards_from_balance(
        &mut ctx.accounts.staking_pool,
        ctx.accounts.staking_usdc_vault.amount,
        clock.unix_timestamp,
    )?;

    apply_unstake_state(
        &mut ctx.accounts.staking_pool,
        &mut ctx.accounts.user_stake_account,
        ctx.accounts.owner.key(),
        amount,
        clock.unix_timestamp,
    )?;

    let alpha_authority_bump = ctx.accounts.staking_pool.alpha_vault_authority_bump;
    let signer_seeds: &[&[&[u8]]] = &[&[ALPHA_VAULT_AUTHORITY_V1_SEED, &[alpha_authority_bump]]];
    let cpi_accounts = TransferChecked {
        from: ctx.accounts.alpha_vault.to_account_info(),
        mint: ctx.accounts.alpha_mint.to_account_info(),
        to: ctx.accounts.owner_alpha_token_account.to_account_info(),
        authority: ctx.accounts.alpha_vault_authority.to_account_info(),
    };
    let cpi_ctx =
        CpiContext::new_with_signer(ctx.accounts.token_program.key(), cpi_accounts, signer_seeds);

    transfer_checked(cpi_ctx, amount, ctx.accounts.alpha_mint.decimals)
}

pub fn lock_tier_config(lock_tier: u8) -> Result<LockTierConfig> {
    match lock_tier {
        LOCK_TIER_FLEXIBLE => Ok(LockTierConfig {
            multiplier_bps: FLEXIBLE_MULTIPLIER_BPS,
            duration_seconds: 0,
        }),
        LOCK_TIER_30_DAYS => Ok(LockTierConfig {
            multiplier_bps: LOCK_30_DAYS_MULTIPLIER_BPS,
            duration_seconds: LOCK_30_DAYS_SECONDS,
        }),
        LOCK_TIER_90_DAYS => Ok(LockTierConfig {
            multiplier_bps: LOCK_90_DAYS_MULTIPLIER_BPS,
            duration_seconds: LOCK_90_DAYS_SECONDS,
        }),
        LOCK_TIER_180_DAYS => Ok(LockTierConfig {
            multiplier_bps: LOCK_180_DAYS_MULTIPLIER_BPS,
            duration_seconds: LOCK_180_DAYS_SECONDS,
        }),
        LOCK_TIER_365_DAYS => Ok(LockTierConfig {
            multiplier_bps: LOCK_365_DAYS_MULTIPLIER_BPS,
            duration_seconds: LOCK_365_DAYS_SECONDS,
        }),
        _ => err!(CustomError::InvalidLockTier),
    }
}

pub fn update_pool_rewards_from_balance(
    pool: &mut StakingPoolV1,
    current_balance: u64,
    now_ts: i64,
) -> Result<()> {
    if pool.total_effective_weight == 0 {
        pool.last_observed_usdc_balance = current_balance;
        pool.last_reward_update_ts = now_ts;
        return Ok(());
    }

    if current_balance < pool.last_observed_usdc_balance {
        return err!(CustomError::VaultBalanceBelowObserved);
    }

    if current_balance == pool.last_observed_usdc_balance {
        pool.last_reward_update_ts = now_ts;
        return Ok(());
    }

    let new_income = current_balance
        .checked_sub(pool.last_observed_usdc_balance)
        .ok_or(CustomError::MathOverflow)?;
    let distributable = (new_income as u128)
        .checked_mul(pool.reward_release_bps as u128)
        .and_then(|value| value.checked_div(BPS_DENOMINATOR as u128))
        .ok_or(CustomError::MathOverflow)?;

    if distributable > 0 {
        let index_increment = distributable
            .checked_mul(REWARD_INDEX_SCALE)
            .and_then(|value| value.checked_div(pool.total_effective_weight))
            .ok_or(CustomError::MathOverflow)?;
        pool.acc_usdc_per_weight = pool
            .acc_usdc_per_weight
            .checked_add(index_increment)
            .ok_or(CustomError::MathOverflow)?;
    }

    pool.last_observed_usdc_balance = current_balance;
    pool.last_reward_update_ts = now_ts;

    Ok(())
}

pub fn apply_stake_state(
    pool: &mut StakingPoolV1,
    user: &mut UserStakeAccount,
    owner: Pubkey,
    amount: u64,
    lock_tier: u8,
    now_ts: i64,
    user_bump: u8,
) -> Result<()> {
    require!(amount > 0, CustomError::InvalidAmount);
    let config = lock_tier_config(lock_tier)?;

    if user.owner == Pubkey::default() {
        user.owner = owner;
        user.bump = user_bump;
    } else {
        require!(user.owner == owner, CustomError::InvalidStakeOwner);
    }

    settle_user_rewards(pool, user)?;

    let new_lock_end = now_ts
        .checked_add(config.duration_seconds)
        .ok_or(CustomError::MathOverflow)?;

    if user.staked_amount == 0 {
        user.lock_start_ts = now_ts;
        user.lock_end_ts = new_lock_end;
        user.lock_tier = lock_tier;
        user.multiplier_bps = config.multiplier_bps;
        user.next_reward_eligible_ts = now_ts;
    } else {
        require!(
            user.lock_tier == lock_tier,
            CustomError::StakeLockTierMismatch
        );
        user.lock_end_ts = user.lock_end_ts.max(new_lock_end);
    }

    let added_weight = calculate_effective_weight(amount, config.multiplier_bps)?;
    user.staked_amount = user
        .staked_amount
        .checked_add(amount)
        .ok_or(CustomError::MathOverflow)?;
    user.effective_weight = user
        .effective_weight
        .checked_add(added_weight)
        .ok_or(CustomError::MathOverflow)?;
    pool.total_staked_alpha = pool
        .total_staked_alpha
        .checked_add(amount)
        .ok_or(CustomError::MathOverflow)?;
    pool.total_effective_weight = pool
        .total_effective_weight
        .checked_add(added_weight)
        .ok_or(CustomError::MathOverflow)?;
    user.reward_debt = calculate_reward_debt(pool, user.effective_weight)?;

    Ok(())
}

pub fn apply_unstake_state(
    pool: &mut StakingPoolV1,
    user: &mut UserStakeAccount,
    owner: Pubkey,
    amount: u64,
    now_ts: i64,
) -> Result<()> {
    require!(amount > 0, CustomError::InvalidAmount);
    require!(user.owner == owner, CustomError::InvalidStakeOwner);
    require!(now_ts >= user.lock_end_ts, CustomError::LockPeriodNotEnded);
    require!(amount <= user.staked_amount, CustomError::InvalidAmount);

    settle_user_rewards(pool, user)?;

    let unstake_weight = if amount == user.staked_amount {
        user.effective_weight
    } else {
        user.effective_weight
            .checked_mul(amount as u128)
            .and_then(|value| value.checked_div(user.staked_amount as u128))
            .ok_or(CustomError::MathOverflow)?
    };

    user.staked_amount = user
        .staked_amount
        .checked_sub(amount)
        .ok_or(CustomError::MathOverflow)?;
    user.effective_weight = user
        .effective_weight
        .checked_sub(unstake_weight)
        .ok_or(CustomError::MathOverflow)?;
    pool.total_staked_alpha = pool
        .total_staked_alpha
        .checked_sub(amount)
        .ok_or(CustomError::MathOverflow)?;
    pool.total_effective_weight = pool
        .total_effective_weight
        .checked_sub(unstake_weight)
        .ok_or(CustomError::MathOverflow)?;

    if user.staked_amount == 0 {
        user.effective_weight = 0;
        user.reward_debt = 0;
    } else {
        user.reward_debt = calculate_reward_debt(pool, user.effective_weight)?;
    }

    Ok(())
}

pub fn calculate_claim_amount(pool: &StakingPoolV1, user: &UserStakeAccount) -> Result<u64> {
    let accrued = calculate_accrued_rewards(pool, user)?;
    let claim_amount = (user.pending_usdc as u128)
        .checked_add(accrued)
        .ok_or(CustomError::MathOverflow)?;

    u128_to_u64(claim_amount)
}

pub fn apply_claim_state(
    pool: &mut StakingPoolV1,
    user: &mut UserStakeAccount,
    claim_amount: u64,
) -> Result<()> {
    user.pending_usdc = 0;
    user.reward_debt = calculate_reward_debt(pool, user.effective_weight)?;
    pool.last_observed_usdc_balance = pool
        .last_observed_usdc_balance
        .checked_sub(claim_amount)
        .ok_or(CustomError::MathOverflow)?;

    Ok(())
}

pub fn validate_vault_balance_for_claim(vault_balance: u64, claim_amount: u64) -> Result<()> {
    require!(
        vault_balance >= claim_amount,
        CustomError::InsufficientVaultBalance
    );

    Ok(())
}

pub fn validate_token_account_mint_and_owner(
    actual_mint: Pubkey,
    expected_mint: Pubkey,
    actual_owner: Pubkey,
    expected_owner: Pubkey,
) -> Result<()> {
    require_keys_eq!(actual_mint, expected_mint, CustomError::InvalidMint);
    require_keys_eq!(actual_owner, expected_owner, CustomError::InvalidVault);

    Ok(())
}

pub fn calculate_effective_weight(amount: u64, multiplier_bps: u16) -> Result<u128> {
    (amount as u128)
        .checked_mul(multiplier_bps as u128)
        .and_then(|value| value.checked_div(BPS_DENOMINATOR as u128))
        .ok_or(CustomError::MathOverflow.into())
}

fn settle_user_rewards(pool: &StakingPoolV1, user: &mut UserStakeAccount) -> Result<()> {
    let accrued = calculate_accrued_rewards(pool, user)?;
    let accrued_u64 = u128_to_u64(accrued)?;
    user.pending_usdc = user
        .pending_usdc
        .checked_add(accrued_u64)
        .ok_or(CustomError::MathOverflow)?;
    user.reward_debt = calculate_reward_debt(pool, user.effective_weight)?;

    Ok(())
}

fn calculate_accrued_rewards(pool: &StakingPoolV1, user: &UserStakeAccount) -> Result<u128> {
    let accumulated = calculate_reward_debt(pool, user.effective_weight)?;

    accumulated
        .checked_sub(user.reward_debt)
        .ok_or(CustomError::MathOverflow.into())
}

fn calculate_reward_debt(pool: &StakingPoolV1, effective_weight: u128) -> Result<u128> {
    effective_weight
        .checked_mul(pool.acc_usdc_per_weight)
        .and_then(|value| value.checked_div(REWARD_INDEX_SCALE))
        .ok_or(CustomError::MathOverflow.into())
}

fn u128_to_u64(value: u128) -> Result<u64> {
    u64::try_from(value).map_err(|_| CustomError::MathOverflow.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    const OWNER_ONE: Pubkey = Pubkey::new_from_array([1; 32]);
    const OWNER_TWO: Pubkey = Pubkey::new_from_array([2; 32]);

    fn pool() -> StakingPoolV1 {
        StakingPoolV1 {
            authority: Pubkey::new_from_array([3; 32]),
            alpha_mint: Pubkey::new_from_array([4; 32]),
            usdc_mint: Pubkey::new_from_array([5; 32]),
            alpha_vault: Pubkey::new_from_array([6; 32]),
            alpha_vault_authority: Pubkey::new_from_array([7; 32]),
            staking_usdc_vault: Pubkey::new_from_array([8; 32]),
            vault_authority_v2: Pubkey::new_from_array([9; 32]),
            total_staked_alpha: 0,
            total_effective_weight: 0,
            acc_usdc_per_weight: 0,
            last_reward_update_ts: 0,
            last_observed_usdc_balance: 0,
            reward_release_bps: STAKING_PHASE1_REWARD_RELEASE_BPS,
            min_claim_usdc: DEFAULT_MIN_CLAIM_USDC,
            vault_authority_v2_bump: 250,
            alpha_vault_authority_bump: 251,
            bump: 252,
        }
    }

    fn user(owner: Pubkey) -> UserStakeAccount {
        UserStakeAccount {
            owner,
            staked_amount: 0,
            effective_weight: 0,
            lock_start_ts: 0,
            lock_end_ts: 0,
            lock_tier: LOCK_TIER_FLEXIBLE,
            multiplier_bps: FLEXIBLE_MULTIPLIER_BPS,
            reward_debt: 0,
            pending_usdc: 0,
            next_reward_eligible_ts: 0,
            bump: 200,
        }
    }

    fn assert_error_contains(err: anchor_lang::error::Error, expected: &str) {
        let message = format!("{err:?}");
        assert!(
            message.contains(expected),
            "expected {expected}, got {message}"
        );
    }

    #[test]
    fn initializes_pool_observed_balance_from_existing_vault_amount() {
        let mut pool = pool();
        pool.last_observed_usdc_balance = 2_000_000;

        assert_eq!(pool.last_observed_usdc_balance, 2_000_000);
        assert_eq!(pool.acc_usdc_per_weight, 0);
        assert_eq!(pool.reward_release_bps, 10_000);
    }

    #[test]
    fn existing_two_usdc_does_not_reward_first_staker() {
        let mut pool = pool();
        pool.last_observed_usdc_balance = 2_000_000;

        update_pool_rewards_from_balance(&mut pool, 2_000_000, 10).unwrap();
        assert_eq!(pool.acc_usdc_per_weight, 0);

        let mut user = user(Pubkey::default());
        apply_stake_state(
            &mut pool,
            &mut user,
            OWNER_ONE,
            1_000,
            LOCK_TIER_30_DAYS,
            11,
            1,
        )
        .unwrap();

        assert_eq!(calculate_claim_amount(&pool, &user).unwrap(), 0);
        assert_eq!(pool.last_observed_usdc_balance, 2_000_000);
    }

    #[test]
    fn lock_multipliers_are_correct() {
        assert_eq!(
            lock_tier_config(LOCK_TIER_FLEXIBLE).unwrap().multiplier_bps,
            6_000
        );
        assert_eq!(
            lock_tier_config(LOCK_TIER_30_DAYS).unwrap().multiplier_bps,
            10_000
        );
        assert_eq!(
            lock_tier_config(LOCK_TIER_90_DAYS).unwrap().multiplier_bps,
            13_500
        );
        assert_eq!(
            lock_tier_config(LOCK_TIER_180_DAYS).unwrap().multiplier_bps,
            18_000
        );
        assert_eq!(
            lock_tier_config(LOCK_TIER_365_DAYS).unwrap().multiplier_bps,
            25_000
        );
    }

    #[test]
    fn rejects_zero_stake_amount() {
        let mut pool = pool();
        let mut user = user(Pubkey::default());
        let err = apply_stake_state(&mut pool, &mut user, OWNER_ONE, 0, LOCK_TIER_30_DAYS, 1, 1)
            .unwrap_err();

        assert_error_contains(err, "InvalidAmount");
    }

    #[test]
    fn rejects_invalid_lock_tier() {
        let err = lock_tier_config(99).unwrap_err();

        assert_error_contains(err, "InvalidLockTier");
    }

    #[test]
    fn allows_adding_same_lock_tier_and_extends_lock_end() {
        let mut pool = pool();
        let mut user = user(Pubkey::default());

        apply_stake_state(
            &mut pool,
            &mut user,
            OWNER_ONE,
            1_000,
            LOCK_TIER_30_DAYS,
            100,
            1,
        )
        .unwrap();
        let first_lock_end = user.lock_end_ts;
        apply_stake_state(
            &mut pool,
            &mut user,
            OWNER_ONE,
            500,
            LOCK_TIER_30_DAYS,
            200,
            1,
        )
        .unwrap();

        assert_eq!(user.staked_amount, 1_500);
        assert!(user.lock_end_ts > first_lock_end);
        assert_eq!(pool.total_effective_weight, 1_500);
    }

    #[test]
    fn rejects_adding_different_lock_tier() {
        let mut pool = pool();
        let mut user = user(Pubkey::default());

        apply_stake_state(
            &mut pool,
            &mut user,
            OWNER_ONE,
            1_000,
            LOCK_TIER_30_DAYS,
            100,
            1,
        )
        .unwrap();
        let err = apply_stake_state(
            &mut pool,
            &mut user,
            OWNER_ONE,
            500,
            LOCK_TIER_90_DAYS,
            200,
            1,
        )
        .unwrap_err();

        assert_error_contains(err, "StakeLockTierMismatch");
    }

    #[test]
    fn distributes_rewards_to_multiple_users_by_effective_weight() {
        let mut pool = pool();
        let mut user_one = user(Pubkey::default());
        let mut user_two = user(Pubkey::default());

        apply_stake_state(
            &mut pool,
            &mut user_one,
            OWNER_ONE,
            1_000,
            LOCK_TIER_30_DAYS,
            100,
            1,
        )
        .unwrap();
        apply_stake_state(
            &mut pool,
            &mut user_two,
            OWNER_TWO,
            1_000,
            LOCK_TIER_365_DAYS,
            100,
            2,
        )
        .unwrap();

        update_pool_rewards_from_balance(&mut pool, 3_500_000, 200).unwrap();

        assert_eq!(calculate_claim_amount(&pool, &user_one).unwrap(), 1_000_000);
        assert_eq!(calculate_claim_amount(&pool, &user_two).unwrap(), 2_500_000);
    }

    #[test]
    fn total_weight_zero_updates_observed_without_crashing() {
        let mut pool = pool();

        update_pool_rewards_from_balance(&mut pool, 5_000_000, 10).unwrap();

        assert_eq!(pool.acc_usdc_per_weight, 0);
        assert_eq!(pool.last_observed_usdc_balance, 5_000_000);
        assert_eq!(pool.last_reward_update_ts, 10);
    }

    #[test]
    fn current_balance_below_observed_returns_error() {
        let mut pool = pool();
        pool.total_effective_weight = 1;
        pool.last_observed_usdc_balance = 5_000_000;

        let err = update_pool_rewards_from_balance(&mut pool, 4_999_999, 10).unwrap_err();

        assert_error_contains(err, "VaultBalanceBelowObserved");
    }

    #[test]
    fn claim_success_updates_observed_balance() {
        let mut pool = pool();
        let mut user = user(Pubkey::default());

        apply_stake_state(
            &mut pool,
            &mut user,
            OWNER_ONE,
            1_000,
            LOCK_TIER_30_DAYS,
            100,
            1,
        )
        .unwrap();
        update_pool_rewards_from_balance(&mut pool, 1_000_000, 200).unwrap();
        let claim_amount = calculate_claim_amount(&pool, &user).unwrap();

        validate_vault_balance_for_claim(1_000_000, claim_amount).unwrap();
        apply_claim_state(&mut pool, &mut user, claim_amount).unwrap();

        assert_eq!(claim_amount, 1_000_000);
        assert_eq!(user.pending_usdc, 0);
        assert_eq!(calculate_claim_amount(&pool, &user).unwrap(), 0);
        assert_eq!(pool.last_observed_usdc_balance, 0);
    }

    #[test]
    fn repeat_claim_is_below_minimum() {
        let pool = pool();
        let user = user(OWNER_ONE);

        let claim_amount = calculate_claim_amount(&pool, &user).unwrap();

        assert!(claim_amount < pool.min_claim_usdc);
    }

    #[test]
    fn vault_balance_insufficient_for_claim_fails() {
        let err = validate_vault_balance_for_claim(99, 100).unwrap_err();

        assert_error_contains(err, "InsufficientVaultBalance");
    }

    #[test]
    fn vault_mint_mismatch_fails() {
        let expected_mint = Pubkey::new_from_array([10; 32]);
        let actual_mint = Pubkey::new_from_array([11; 32]);
        let owner = Pubkey::new_from_array([12; 32]);

        let err = validate_token_account_mint_and_owner(actual_mint, expected_mint, owner, owner)
            .unwrap_err();

        assert_error_contains(err, "InvalidMint");
    }

    #[test]
    fn rejects_unstake_before_lock_end() {
        let mut pool = pool();
        let mut user = user(Pubkey::default());

        apply_stake_state(
            &mut pool,
            &mut user,
            OWNER_ONE,
            1_000,
            LOCK_TIER_30_DAYS,
            100,
            1,
        )
        .unwrap();
        let err = apply_unstake_state(&mut pool, &mut user, OWNER_ONE, 500, 101).unwrap_err();

        assert_error_contains(err, "LockPeriodNotEnded");
    }

    #[test]
    fn partial_unstake_after_lock_end_succeeds() {
        let mut pool = pool();
        let mut user = user(Pubkey::default());

        apply_stake_state(
            &mut pool,
            &mut user,
            OWNER_ONE,
            1_000,
            LOCK_TIER_30_DAYS,
            100,
            1,
        )
        .unwrap();
        let unlock_ts = user.lock_end_ts;
        apply_unstake_state(&mut pool, &mut user, OWNER_ONE, 400, unlock_ts).unwrap();

        assert_eq!(user.staked_amount, 600);
        assert_eq!(user.effective_weight, 600);
        assert_eq!(pool.total_staked_alpha, 600);
        assert_eq!(pool.total_effective_weight, 600);
    }

    #[test]
    fn full_unstake_after_lock_end_preserves_pending_usdc() {
        let mut pool = pool();
        let mut user = user(Pubkey::default());

        apply_stake_state(
            &mut pool,
            &mut user,
            OWNER_ONE,
            1_000,
            LOCK_TIER_30_DAYS,
            100,
            1,
        )
        .unwrap();
        update_pool_rewards_from_balance(&mut pool, 1_000_000, 200).unwrap();
        let unlock_ts = user.lock_end_ts;

        apply_unstake_state(&mut pool, &mut user, OWNER_ONE, 1_000, unlock_ts).unwrap();

        assert_eq!(user.staked_amount, 0);
        assert_eq!(user.effective_weight, 0);
        assert_eq!(user.reward_debt, 0);
        assert_eq!(user.pending_usdc, 1_000_000);
        assert_eq!(calculate_claim_amount(&pool, &user).unwrap(), 1_000_000);
    }

    #[test]
    fn overflowing_stake_total_fails() {
        let mut pool = pool();
        let mut user = user(Pubkey::default());
        pool.total_staked_alpha = u64::MAX;

        let err = apply_stake_state(&mut pool, &mut user, OWNER_ONE, 1, LOCK_TIER_30_DAYS, 1, 1)
            .unwrap_err();

        assert_error_contains(err, "MathOverflow");
    }

    #[test]
    fn overflowing_reward_index_fails() {
        let mut pool = pool();
        pool.total_effective_weight = 1;
        pool.acc_usdc_per_weight = u128::MAX;

        let err = update_pool_rewards_from_balance(&mut pool, 1, 1).unwrap_err();

        assert_error_contains(err, "MathOverflow");
    }
}
