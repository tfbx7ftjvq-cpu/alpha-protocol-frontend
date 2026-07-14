use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};

use crate::constants::{
    BPS_DENOMINATOR, GOVERNANCE_180_DAY_MULTIPLIER_BPS, GOVERNANCE_30_DAY_MULTIPLIER_BPS,
    GOVERNANCE_365_DAY_MULTIPLIER_BPS, GOVERNANCE_90_DAY_MULTIPLIER_BPS,
    GOVERNANCE_DEFAULT_APPROVAL_THRESHOLD_BPS, GOVERNANCE_DEFAULT_MIN_LOCK_AMOUNT,
    GOVERNANCE_DEFAULT_QUORUM_BPS, GOVERNANCE_DEFAULT_VOTING_PERIOD_SECONDS,
    GOVERNANCE_LOCK_CONFIG_V1_SEED, GOVERNANCE_MAX_LOCK_DURATION_SECONDS,
    GOVERNANCE_MAX_TIME_MULTIPLIER_BPS, GOVERNANCE_MIN_LOCK_DURATION_SECONDS,
    GOVERNANCE_POSITION_V1_SEED, GOVERNANCE_POSITION_VOTE_LOCK_V1_SEED,
    GOVERNANCE_POWER_STATE_V1_SEED, GOVERNANCE_PROPOSAL_ACTION_V1_SEED,
    GOVERNANCE_PROPOSAL_V1_SEED, GOVERNANCE_SNAPSHOT_V1_SEED, GOVERNANCE_VAULT_V1_SEED,
    GOVERNANCE_VOTING_CONFIG_V1_SEED, LOCK_180_DAYS_SECONDS, LOCK_30_DAYS_SECONDS,
    LOCK_365_DAYS_SECONDS, LOCK_90_DAYS_SECONDS, VOTE_RECORD_V1_SEED,
};
use crate::error::CustomError;
use crate::instructions::governance_action_v1::{
    governance_action_stable_code_v1, governance_payload_from_action_request_v1,
    hash_governance_payload_v1, map_governance_action_to_governance_proposal_type_v1,
    map_governance_action_to_module,
};
use crate::state::{
    GovernanceActionRequestV1, GovernanceLockConfigV1, GovernancePayloadV1,
    GovernancePositionStatusV1, GovernancePositionV1, GovernancePositionVoteLockV1,
    GovernancePowerStateV1, GovernanceProposalActionV1, GovernanceProposalStatusV1,
    GovernanceProposalTypeV1, GovernanceProposalV1, GovernanceSnapshotV1, GovernanceVotingConfigV1,
    VoteChoiceV1, VoteRecordV1,
};

#[derive(Accounts)]
pub struct InitializeGovernanceConfigV1<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + GovernanceLockConfigV1::INIT_SPACE,
        seeds = [GOVERNANCE_LOCK_CONFIG_V1_SEED],
        bump
    )]
    pub governance_config: Account<'info, GovernanceLockConfigV1>,

    #[account(
        init,
        payer = authority,
        space = 8 + GovernancePowerStateV1::INIT_SPACE,
        seeds = [GOVERNANCE_POWER_STATE_V1_SEED, governance_config.key().as_ref()],
        bump
    )]
    pub governance_power_state: Account<'info, GovernancePowerStateV1>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub alpha_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = authority,
        token::mint = alpha_mint,
        token::authority = governance_config,
        seeds = [GOVERNANCE_VAULT_V1_SEED, governance_config.key().as_ref()],
        bump
    )]
    pub governance_vault: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitializeGovernanceVotingConfigV1<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + GovernanceVotingConfigV1::INIT_SPACE,
        seeds = [GOVERNANCE_VOTING_CONFIG_V1_SEED],
        bump
    )]
    pub governance_voting_config: Account<'info, GovernanceVotingConfigV1>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct InitializeGovernanceProposalV1<'info> {
    #[account(
        init,
        payer = proposer,
        space = 8 + GovernanceProposalV1::INIT_SPACE,
        seeds = [GOVERNANCE_PROPOSAL_V1_SEED, &proposal_id.to_le_bytes()],
        bump
    )]
    pub governance_proposal: Account<'info, GovernanceProposalV1>,

    #[account(mut)]
    pub proposer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct InitializeGovernanceProposalWithActionV1<'info> {
    #[account(
        init,
        payer = proposer,
        space = 8 + GovernanceProposalV1::INIT_SPACE,
        seeds = [GOVERNANCE_PROPOSAL_V1_SEED, &proposal_id.to_le_bytes()],
        bump
    )]
    pub governance_proposal: Account<'info, GovernanceProposalV1>,

    #[account(
        init,
        payer = proposer,
        space = 8 + GovernanceProposalActionV1::INIT_SPACE,
        seeds = [
            GOVERNANCE_PROPOSAL_ACTION_V1_SEED,
            governance_proposal.key().as_ref()
        ],
        bump
    )]
    pub governance_proposal_action: Account<'info, GovernanceProposalActionV1>,

    #[account(mut)]
    pub proposer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeGovernancePositionV1<'info> {
    #[account(
        seeds = [GOVERNANCE_LOCK_CONFIG_V1_SEED],
        bump = governance_config.bump
    )]
    pub governance_config: Account<'info, GovernanceLockConfigV1>,

    #[account(
        init,
        payer = owner,
        space = 8 + GovernancePositionV1::INIT_SPACE,
        seeds = [GOVERNANCE_POSITION_V1_SEED, owner.key().as_ref()],
        bump
    )]
    pub governance_position: Account<'info, GovernancePositionV1>,

    #[account(
        init,
        payer = owner,
        space = 8 + GovernancePositionVoteLockV1::INIT_SPACE,
        seeds = [
            GOVERNANCE_POSITION_VOTE_LOCK_V1_SEED,
            governance_position.key().as_ref()
        ],
        bump
    )]
    pub governance_position_vote_lock: Account<'info, GovernancePositionVoteLockV1>,

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        constraint = alpha_mint.key() == governance_config.alpha_mint @ CustomError::InvalidMint
    )]
    pub alpha_mint: Box<Account<'info, Mint>>,

    #[account(
        constraint = governance_vault.key() == governance_config.governance_vault @ CustomError::InvalidGovernanceVault,
        constraint = governance_vault.mint == governance_config.alpha_mint @ CustomError::InvalidMint,
        constraint = governance_vault.owner == governance_config.key() @ CustomError::InvalidGovernanceVault
    )]
    pub governance_vault: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct LockAlphaForGovernance<'info> {
    #[account(
        seeds = [GOVERNANCE_LOCK_CONFIG_V1_SEED],
        bump = governance_config.bump
    )]
    pub governance_config: Account<'info, GovernanceLockConfigV1>,

    #[account(
        mut,
        seeds = [GOVERNANCE_POSITION_V1_SEED, owner.key().as_ref()],
        bump = governance_position.bump,
        constraint = governance_position.owner == owner.key() @ CustomError::UnauthorizedGovernancePositionOwner,
        constraint = governance_position.alpha_mint == governance_config.alpha_mint @ CustomError::InvalidMint,
        constraint = governance_position.vault == governance_config.governance_vault @ CustomError::InvalidGovernanceVault
    )]
    pub governance_position: Account<'info, GovernancePositionV1>,

    #[account(
        mut,
        seeds = [GOVERNANCE_POWER_STATE_V1_SEED, governance_config.key().as_ref()],
        bump = governance_power_state.bump,
        constraint = governance_power_state.governance_lock_config == governance_config.key() @ CustomError::InvalidGovernancePowerState
    )]
    pub governance_power_state: Account<'info, GovernancePowerStateV1>,

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        constraint = owner_alpha_token_account.mint == governance_config.alpha_mint @ CustomError::InvalidMint,
        constraint = owner_alpha_token_account.owner == owner.key() @ CustomError::InvalidTokenAccount
    )]
    pub owner_alpha_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = alpha_mint.key() == governance_config.alpha_mint @ CustomError::InvalidMint
    )]
    pub alpha_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [GOVERNANCE_VAULT_V1_SEED, governance_config.key().as_ref()],
        bump,
        constraint = governance_vault.key() == governance_config.governance_vault @ CustomError::InvalidGovernanceVault,
        constraint = governance_vault.mint == governance_config.alpha_mint @ CustomError::InvalidMint,
        constraint = governance_vault.owner == governance_config.key() @ CustomError::InvalidGovernanceVault
    )]
    pub governance_vault: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UnlockAlphaFromGovernance<'info> {
    #[account(
        seeds = [GOVERNANCE_LOCK_CONFIG_V1_SEED],
        bump = governance_config.bump
    )]
    pub governance_config: Account<'info, GovernanceLockConfigV1>,

    #[account(
        mut,
        seeds = [GOVERNANCE_POSITION_V1_SEED, owner.key().as_ref()],
        bump = governance_position.bump,
        constraint = governance_position.owner == owner.key() @ CustomError::UnauthorizedGovernancePositionOwner,
        constraint = governance_position.alpha_mint == governance_config.alpha_mint @ CustomError::InvalidMint,
        constraint = governance_position.vault == governance_config.governance_vault @ CustomError::InvalidGovernanceVault
    )]
    pub governance_position: Account<'info, GovernancePositionV1>,

    #[account(
        mut,
        seeds = [GOVERNANCE_POWER_STATE_V1_SEED, governance_config.key().as_ref()],
        bump = governance_power_state.bump,
        constraint = governance_power_state.governance_lock_config == governance_config.key() @ CustomError::InvalidGovernancePowerState
    )]
    pub governance_power_state: Account<'info, GovernancePowerStateV1>,

    #[account(
        mut,
        seeds = [
            GOVERNANCE_POSITION_VOTE_LOCK_V1_SEED,
            governance_position.key().as_ref()
        ],
        bump = governance_position_vote_lock.bump,
        constraint = governance_position_vote_lock.governance_position == governance_position.key() @ CustomError::InvalidGovernanceVoteLock
    )]
    pub governance_position_vote_lock: Account<'info, GovernancePositionVoteLockV1>,

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        constraint = owner_alpha_token_account.mint == governance_config.alpha_mint @ CustomError::InvalidMint,
        constraint = owner_alpha_token_account.owner == owner.key() @ CustomError::InvalidTokenAccount
    )]
    pub owner_alpha_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = alpha_mint.key() == governance_config.alpha_mint @ CustomError::InvalidMint
    )]
    pub alpha_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [GOVERNANCE_VAULT_V1_SEED, governance_config.key().as_ref()],
        bump,
        constraint = governance_vault.key() == governance_config.governance_vault @ CustomError::InvalidGovernanceVault,
        constraint = governance_vault.mint == governance_config.alpha_mint @ CustomError::InvalidMint,
        constraint = governance_vault.owner == governance_config.key() @ CustomError::InvalidGovernanceVault
    )]
    pub governance_vault: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct InitializeGovernanceSnapshotV1<'info> {
    pub governance_proposal: Account<'info, GovernanceProposalV1>,

    #[account(
        init,
        payer = payer,
        space = 8 + GovernanceSnapshotV1::INIT_SPACE,
        seeds = [GOVERNANCE_SNAPSHOT_V1_SEED, governance_proposal.key().as_ref()],
        bump
    )]
    pub governance_snapshot: Account<'info, GovernanceSnapshotV1>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateGovernanceSnapshotV1<'info> {
    #[account(
        seeds = [GOVERNANCE_VOTING_CONFIG_V1_SEED],
        bump = governance_voting_config.bump
    )]
    pub governance_voting_config: Account<'info, GovernanceVotingConfigV1>,

    #[account(
        seeds = [GOVERNANCE_LOCK_CONFIG_V1_SEED],
        bump = governance_config.bump
    )]
    pub governance_config: Account<'info, GovernanceLockConfigV1>,

    #[account(
        seeds = [GOVERNANCE_POWER_STATE_V1_SEED, governance_config.key().as_ref()],
        bump = governance_power_state.bump,
        constraint = governance_power_state.governance_lock_config == governance_config.key() @ CustomError::InvalidGovernancePowerState
    )]
    pub governance_power_state: Account<'info, GovernancePowerStateV1>,

    #[account(
        mut,
        seeds = [GOVERNANCE_PROPOSAL_V1_SEED, &governance_proposal.proposal_id.to_le_bytes()],
        bump = governance_proposal.bump,
        constraint = governance_proposal.proposer == proposer.key() @ CustomError::UnauthorizedSecurityAuthority
    )]
    pub governance_proposal: Account<'info, GovernanceProposalV1>,

    #[account(
        seeds = [
            GOVERNANCE_PROPOSAL_ACTION_V1_SEED,
            governance_proposal.key().as_ref()
        ],
        bump = governance_proposal_action.bump
    )]
    pub governance_proposal_action: Account<'info, GovernanceProposalActionV1>,

    #[account(
        init,
        payer = proposer,
        space = 8 + GovernanceSnapshotV1::INIT_SPACE,
        seeds = [GOVERNANCE_SNAPSHOT_V1_SEED, governance_proposal.key().as_ref()],
        bump
    )]
    pub governance_snapshot: Account<'info, GovernanceSnapshotV1>,

    #[account(mut)]
    pub proposer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CastGovernanceVoteV1<'info> {
    #[account(
        mut,
        seeds = [GOVERNANCE_PROPOSAL_V1_SEED, &governance_proposal.proposal_id.to_le_bytes()],
        bump = governance_proposal.bump
    )]
    pub governance_proposal: Account<'info, GovernanceProposalV1>,

    #[account(
        mut,
        seeds = [GOVERNANCE_SNAPSHOT_V1_SEED, governance_proposal.key().as_ref()],
        bump = governance_snapshot.bump,
        constraint = governance_snapshot.proposal == governance_proposal.key() @ CustomError::InvalidGovernanceSnapshot
    )]
    pub governance_snapshot: Account<'info, GovernanceSnapshotV1>,

    #[account(
        seeds = [GOVERNANCE_POSITION_V1_SEED, voter.key().as_ref()],
        bump = governance_position.bump,
        constraint = governance_position.owner == voter.key() @ CustomError::UnauthorizedGovernancePositionOwner,
        constraint = governance_position.status == GovernancePositionStatusV1::Active @ CustomError::InvalidGovernancePosition
    )]
    pub governance_position: Account<'info, GovernancePositionV1>,

    #[account(
        mut,
        seeds = [
            GOVERNANCE_POSITION_VOTE_LOCK_V1_SEED,
            governance_position.key().as_ref()
        ],
        bump = governance_position_vote_lock.bump,
        constraint = governance_position_vote_lock.governance_position == governance_position.key() @ CustomError::InvalidGovernanceVoteLock
    )]
    pub governance_position_vote_lock: Account<'info, GovernancePositionVoteLockV1>,

    #[account(
        init_if_needed,
        payer = voter,
        space = 8 + VoteRecordV1::INIT_SPACE,
        seeds = [
            VOTE_RECORD_V1_SEED,
            governance_proposal.key().as_ref(),
            governance_position.key().as_ref()
        ],
        bump
    )]
    pub vote_record: Account<'info, VoteRecordV1>,

    #[account(mut)]
    pub voter: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FinalizeGovernanceVoteV1<'info> {
    #[account(
        seeds = [GOVERNANCE_VOTING_CONFIG_V1_SEED],
        bump = governance_voting_config.bump
    )]
    pub governance_voting_config: Account<'info, GovernanceVotingConfigV1>,

    #[account(
        mut,
        seeds = [GOVERNANCE_PROPOSAL_V1_SEED, &governance_proposal.proposal_id.to_le_bytes()],
        bump = governance_proposal.bump
    )]
    pub governance_proposal: Account<'info, GovernanceProposalV1>,

    #[account(
        mut,
        seeds = [GOVERNANCE_SNAPSHOT_V1_SEED, governance_proposal.key().as_ref()],
        bump = governance_snapshot.bump,
        constraint = governance_snapshot.proposal == governance_proposal.key() @ CustomError::InvalidGovernanceSnapshot
    )]
    pub governance_snapshot: Account<'info, GovernanceSnapshotV1>,
}

#[derive(Accounts)]
pub struct InitializeVoteRecordV1<'info> {
    pub governance_proposal: Account<'info, GovernanceProposalV1>,

    #[account(
        seeds = [GOVERNANCE_POSITION_V1_SEED, voter.key().as_ref()],
        bump = governance_position.bump,
        constraint = governance_position.owner == voter.key() @ CustomError::UnauthorizedGovernancePositionOwner,
        constraint = governance_position.status == GovernancePositionStatusV1::Active @ CustomError::InvalidGovernancePosition
    )]
    pub governance_position: Account<'info, GovernancePositionV1>,

    #[account(
        init,
        payer = voter,
        space = 8 + VoteRecordV1::INIT_SPACE,
        seeds = [
            VOTE_RECORD_V1_SEED,
            governance_proposal.key().as_ref(),
            governance_position.key().as_ref()
        ],
        bump
    )]
    pub vote_record: Account<'info, VoteRecordV1>,

    #[account(mut)]
    pub voter: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_governance_config_v1_handler(
    ctx: Context<InitializeGovernanceConfigV1>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    record_governance_config_init(
        &mut ctx.accounts.governance_config,
        ctx.accounts.authority.key(),
        ctx.accounts.alpha_mint.key(),
        ctx.accounts.governance_vault.key(),
        now,
        ctx.bumps.governance_config,
    )?;
    record_governance_power_state_init(
        &mut ctx.accounts.governance_power_state,
        ctx.accounts.governance_config.key(),
        now,
        ctx.bumps.governance_power_state,
    )
}

pub fn initialize_governance_voting_config_v1_handler(
    ctx: Context<InitializeGovernanceVotingConfigV1>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    record_governance_voting_config_init(
        &mut ctx.accounts.governance_voting_config,
        ctx.accounts.authority.key(),
        now,
        ctx.bumps.governance_voting_config,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn initialize_governance_proposal_v1_handler(
    ctx: Context<InitializeGovernanceProposalV1>,
    proposal_id: u64,
    proposal_type: GovernanceProposalTypeV1,
    action_type: u8,
    target_program: Pubkey,
    target_account: Pubkey,
    payload_hash: [u8; 32],
    voting_start_ts: i64,
    voting_end_ts: i64,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    record_governance_proposal_init(
        &mut ctx.accounts.governance_proposal,
        proposal_id,
        ctx.accounts.proposer.key(),
        proposal_type,
        action_type,
        target_program,
        target_account,
        payload_hash,
        voting_start_ts,
        voting_end_ts,
        now,
        ctx.bumps.governance_proposal,
    )
}

pub fn initialize_governance_proposal_with_action_v1_handler(
    ctx: Context<InitializeGovernanceProposalWithActionV1>,
    proposal_id: u64,
    request: GovernanceActionRequestV1,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let proposal_key = ctx.accounts.governance_proposal.key();
    record_governance_proposal_with_action_init(
        &mut ctx.accounts.governance_proposal,
        &mut ctx.accounts.governance_proposal_action,
        proposal_key,
        proposal_id,
        ctx.accounts.proposer.key(),
        &request,
        now,
        ctx.bumps.governance_proposal,
        ctx.bumps.governance_proposal_action,
    )
}

pub fn initialize_governance_position_v1_handler(
    ctx: Context<InitializeGovernancePositionV1>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    record_governance_position_init(
        &mut ctx.accounts.governance_position,
        ctx.accounts.owner.key(),
        ctx.accounts.alpha_mint.key(),
        ctx.accounts.governance_vault.key(),
        now,
        ctx.bumps.governance_position,
    )?;
    record_governance_position_vote_lock_init(
        &mut ctx.accounts.governance_position_vote_lock,
        ctx.accounts.governance_position.key(),
        now,
        ctx.bumps.governance_position_vote_lock,
    )
}

pub fn lock_alpha_for_governance_handler(
    ctx: Context<LockAlphaForGovernance>,
    amount: u64,
    lock_duration_seconds: i64,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let previous_locked_amount = ctx.accounts.governance_position.locked_amount;
    let previous_voting_power = ctx.accounts.governance_position.voting_power;
    record_governance_lock(
        &mut ctx.accounts.governance_position,
        &ctx.accounts.governance_config,
        amount,
        lock_duration_seconds,
        now,
    )?;
    record_governance_power_after_lock(
        &mut ctx.accounts.governance_power_state,
        ctx.accounts.governance_config.key(),
        previous_locked_amount,
        previous_voting_power,
        ctx.accounts.governance_position.locked_amount,
        ctx.accounts.governance_position.voting_power,
        now,
    )?;

    let cpi_accounts = TransferChecked {
        from: ctx.accounts.owner_alpha_token_account.to_account_info(),
        mint: ctx.accounts.alpha_mint.to_account_info(),
        to: ctx.accounts.governance_vault.to_account_info(),
        authority: ctx.accounts.owner.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.key(), cpi_accounts);

    transfer_checked(cpi_ctx, amount, ctx.accounts.alpha_mint.decimals)
}

pub fn unlock_alpha_from_governance_handler(ctx: Context<UnlockAlphaFromGovernance>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let unlock_amount = validate_governance_unlock_with_vote_lock(
        &ctx.accounts.governance_position,
        ctx.accounts.governance_position.key(),
        &ctx.accounts.governance_position_vote_lock,
        ctx.accounts.governance_vault.amount,
        now,
    )?;
    let previous_voting_power = ctx.accounts.governance_position.voting_power;

    let config_bump = ctx.accounts.governance_config.bump;
    let signer_seeds: &[&[&[u8]]] = &[&[GOVERNANCE_LOCK_CONFIG_V1_SEED, &[config_bump]]];
    let cpi_accounts = TransferChecked {
        from: ctx.accounts.governance_vault.to_account_info(),
        mint: ctx.accounts.alpha_mint.to_account_info(),
        to: ctx.accounts.owner_alpha_token_account.to_account_info(),
        authority: ctx.accounts.governance_config.to_account_info(),
    };
    let cpi_ctx =
        CpiContext::new_with_signer(ctx.accounts.token_program.key(), cpi_accounts, signer_seeds);

    transfer_checked(cpi_ctx, unlock_amount, ctx.accounts.alpha_mint.decimals)?;
    record_governance_power_after_unlock(
        &mut ctx.accounts.governance_power_state,
        ctx.accounts.governance_config.key(),
        unlock_amount,
        previous_voting_power,
        now,
    )?;
    record_governance_unlock(&mut ctx.accounts.governance_position, now)
}

pub fn initialize_governance_snapshot_v1_handler(
    ctx: Context<InitializeGovernanceSnapshotV1>,
) -> Result<()> {
    require_keys_eq!(
        ctx.accounts.payer.key(),
        ctx.accounts.governance_proposal.proposer,
        CustomError::UnauthorizedSecurityAuthority
    );

    let now = Clock::get()?.unix_timestamp;
    record_governance_snapshot_init(
        &mut ctx.accounts.governance_snapshot,
        ctx.accounts.governance_proposal.key(),
        now,
        ctx.bumps.governance_snapshot,
    )
}

pub fn create_governance_snapshot_v1_handler(
    ctx: Context<CreateGovernanceSnapshotV1>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let proposal_key = ctx.accounts.governance_proposal.key();
    let snapshot_key = ctx.accounts.governance_snapshot.key();
    record_governance_snapshot_create(
        &mut ctx.accounts.governance_proposal,
        &ctx.accounts.governance_proposal_action,
        &mut ctx.accounts.governance_snapshot,
        &ctx.accounts.governance_voting_config,
        &ctx.accounts.governance_power_state,
        proposal_key,
        snapshot_key,
        now,
        ctx.bumps.governance_snapshot,
    )
}

pub fn cast_governance_vote_v1_handler(
    ctx: Context<CastGovernanceVoteV1>,
    choice: VoteChoiceV1,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let proposal_key = ctx.accounts.governance_proposal.key();
    let position_key = ctx.accounts.governance_position.key();
    record_cast_governance_vote(
        &mut ctx.accounts.governance_proposal,
        &mut ctx.accounts.governance_snapshot,
        &ctx.accounts.governance_position,
        &mut ctx.accounts.governance_position_vote_lock,
        &mut ctx.accounts.vote_record,
        proposal_key,
        position_key,
        choice,
        now,
        ctx.bumps.vote_record,
    )
}

pub fn finalize_governance_vote_v1_handler(ctx: Context<FinalizeGovernanceVoteV1>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    record_finalize_governance_vote(
        &mut ctx.accounts.governance_proposal,
        &mut ctx.accounts.governance_snapshot,
        &ctx.accounts.governance_voting_config,
        now,
    )
}

pub fn initialize_vote_record_v1_handler(ctx: Context<InitializeVoteRecordV1>) -> Result<()> {
    record_vote_record_init(
        &mut ctx.accounts.vote_record,
        ctx.accounts.governance_proposal.key(),
        ctx.accounts.governance_position.key(),
        ctx.bumps.vote_record,
    )
}

pub fn validate_governance_lock_config(
    min_lock_amount: u64,
    min_lock_duration_seconds: i64,
    max_lock_duration_seconds: i64,
    max_time_multiplier_bps: u64,
) -> Result<()> {
    require!(
        min_lock_amount > 0,
        CustomError::InvalidGovernanceLockAmount
    );
    require!(
        min_lock_duration_seconds == GOVERNANCE_MIN_LOCK_DURATION_SECONDS,
        CustomError::InvalidGovernanceLockConfig
    );
    require!(
        max_lock_duration_seconds == GOVERNANCE_MAX_LOCK_DURATION_SECONDS,
        CustomError::InvalidGovernanceLockConfig
    );
    require!(
        max_time_multiplier_bps == GOVERNANCE_MAX_TIME_MULTIPLIER_BPS,
        CustomError::InvalidGovernanceLockConfig
    );
    require!(
        min_lock_duration_seconds > 0 && max_lock_duration_seconds >= min_lock_duration_seconds,
        CustomError::InvalidGovernanceLockDuration
    );
    Ok(())
}

pub fn record_governance_config_init(
    governance_config: &mut GovernanceLockConfigV1,
    authority: Pubkey,
    alpha_mint: Pubkey,
    governance_vault: Pubkey,
    created_at: i64,
    bump: u8,
) -> Result<()> {
    require!(
        authority != Pubkey::default(),
        CustomError::UnauthorizedSecurityAuthority
    );
    require!(
        alpha_mint != Pubkey::default(),
        CustomError::InvalidGovernanceLockConfig
    );
    require!(
        governance_vault != Pubkey::default(),
        CustomError::InvalidGovernanceVault
    );
    validate_governance_lock_config(
        GOVERNANCE_DEFAULT_MIN_LOCK_AMOUNT,
        GOVERNANCE_MIN_LOCK_DURATION_SECONDS,
        GOVERNANCE_MAX_LOCK_DURATION_SECONDS,
        GOVERNANCE_MAX_TIME_MULTIPLIER_BPS,
    )?;

    governance_config.authority = authority;
    governance_config.alpha_mint = alpha_mint;
    governance_config.governance_vault = governance_vault;
    governance_config.min_lock_amount = GOVERNANCE_DEFAULT_MIN_LOCK_AMOUNT;
    governance_config.min_lock_duration_seconds = GOVERNANCE_MIN_LOCK_DURATION_SECONDS;
    governance_config.max_lock_duration_seconds = GOVERNANCE_MAX_LOCK_DURATION_SECONDS;
    governance_config.max_time_multiplier_bps = GOVERNANCE_MAX_TIME_MULTIPLIER_BPS;
    governance_config.created_at = created_at;
    governance_config.bump = bump;

    Ok(())
}

pub fn record_governance_power_state_init(
    governance_power_state: &mut GovernancePowerStateV1,
    governance_lock_config: Pubkey,
    updated_at: i64,
    bump: u8,
) -> Result<()> {
    require!(
        governance_lock_config != Pubkey::default(),
        CustomError::InvalidGovernancePowerState
    );

    governance_power_state.governance_lock_config = governance_lock_config;
    governance_power_state.total_locked_alpha = 0;
    governance_power_state.total_voting_power = 0;
    governance_power_state.active_position_count = 0;
    governance_power_state.updated_at = updated_at;
    governance_power_state.bump = bump;

    Ok(())
}

pub fn validate_governance_voting_config(
    quorum_bps: u64,
    approval_threshold_bps: u64,
    voting_period_seconds: i64,
) -> Result<()> {
    require!(
        quorum_bps > 0 && quorum_bps <= BPS_DENOMINATOR,
        CustomError::InvalidGovernanceVotingConfig
    );
    require!(
        approval_threshold_bps > 0 && approval_threshold_bps <= BPS_DENOMINATOR,
        CustomError::InvalidGovernanceVotingConfig
    );
    require!(
        voting_period_seconds > 0,
        CustomError::InvalidGovernanceVotingConfig
    );
    Ok(())
}

pub fn record_governance_voting_config_init(
    governance_voting_config: &mut GovernanceVotingConfigV1,
    authority: Pubkey,
    created_at: i64,
    bump: u8,
) -> Result<()> {
    require!(
        authority != Pubkey::default(),
        CustomError::UnauthorizedSecurityAuthority
    );
    validate_governance_voting_config(
        GOVERNANCE_DEFAULT_QUORUM_BPS,
        GOVERNANCE_DEFAULT_APPROVAL_THRESHOLD_BPS,
        GOVERNANCE_DEFAULT_VOTING_PERIOD_SECONDS,
    )?;

    governance_voting_config.authority = authority;
    governance_voting_config.quorum_bps = GOVERNANCE_DEFAULT_QUORUM_BPS;
    governance_voting_config.approval_threshold_bps = GOVERNANCE_DEFAULT_APPROVAL_THRESHOLD_BPS;
    governance_voting_config.voting_period_seconds = GOVERNANCE_DEFAULT_VOTING_PERIOD_SECONDS;
    governance_voting_config.created_at = created_at;
    governance_voting_config.bump = bump;

    Ok(())
}

pub fn validate_governance_proposal_init(
    proposal_id: u64,
    target_program: Pubkey,
    target_account: Pubkey,
    payload_hash: [u8; 32],
    voting_start_ts: i64,
    voting_end_ts: i64,
) -> Result<()> {
    require!(proposal_id > 0, CustomError::InvalidGovernanceProposal);
    require!(
        target_program != Pubkey::default(),
        CustomError::InvalidGovernanceProposal
    );
    require!(
        target_account != Pubkey::default(),
        CustomError::InvalidGovernanceProposal
    );
    require!(
        payload_hash != [0u8; 32],
        CustomError::InvalidGovernanceProposal
    );

    if voting_start_ts == 0 && voting_end_ts == 0 {
        return Ok(());
    }

    require!(
        voting_start_ts > 0 && voting_end_ts > voting_start_ts,
        CustomError::InvalidGovernanceProposalTime
    );
    voting_end_ts
        .checked_sub(voting_start_ts)
        .ok_or(CustomError::MathOverflow)?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn record_governance_proposal_init(
    governance_proposal: &mut GovernanceProposalV1,
    proposal_id: u64,
    proposer: Pubkey,
    proposal_type: GovernanceProposalTypeV1,
    action_type: u8,
    target_program: Pubkey,
    target_account: Pubkey,
    payload_hash: [u8; 32],
    voting_start_ts: i64,
    voting_end_ts: i64,
    created_at: i64,
    bump: u8,
) -> Result<()> {
    validate_governance_proposal_init(
        proposal_id,
        target_program,
        target_account,
        payload_hash,
        voting_start_ts,
        voting_end_ts,
    )?;

    governance_proposal.proposal_id = proposal_id;
    governance_proposal.proposer = proposer;
    governance_proposal.proposal_type = proposal_type;
    governance_proposal.action_type = action_type;
    governance_proposal.target_program = target_program;
    governance_proposal.target_account = target_account;
    governance_proposal.payload_hash = payload_hash;
    governance_proposal.status = GovernanceProposalStatusV1::Draft;
    governance_proposal.voting_start_ts = voting_start_ts;
    governance_proposal.voting_end_ts = voting_end_ts;
    governance_proposal.created_at = created_at;
    governance_proposal.snapshot = Pubkey::default();
    governance_proposal.yes_weight = 0;
    governance_proposal.no_weight = 0;
    governance_proposal.abstain_weight = 0;
    governance_proposal.finalized_at = 0;
    governance_proposal.bump = bump;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn record_governance_proposal_with_action_init(
    governance_proposal: &mut GovernanceProposalV1,
    governance_proposal_action: &mut GovernanceProposalActionV1,
    governance_proposal_key: Pubkey,
    proposal_id: u64,
    proposer: Pubkey,
    request: &GovernanceActionRequestV1,
    created_at: i64,
    proposal_bump: u8,
    action_bump: u8,
) -> Result<()> {
    require!(
        governance_proposal_action.governance_proposal == Pubkey::default(),
        CustomError::GovernanceProposalActionMismatch
    );

    let payload = governance_payload_from_action_request_v1(request, created_at)?;
    let canonical_payload_hash = hash_governance_payload_v1(&payload)?;
    let proposal_type = map_governance_action_to_governance_proposal_type_v1(request.action_type);
    let action_code = governance_action_stable_code_v1(request.action_type);

    record_governance_proposal_init(
        governance_proposal,
        proposal_id,
        proposer,
        proposal_type,
        action_code,
        request.target_program,
        request.target_account,
        canonical_payload_hash,
        0,
        0,
        created_at,
        proposal_bump,
    )?;

    governance_proposal_action.governance_proposal = governance_proposal_key;
    governance_proposal_action.proposal_id = proposal_id;
    governance_proposal_action.proposer = proposer;
    governance_proposal_action.action_type = request.action_type;
    governance_proposal_action.module_id = request.module_id;
    governance_proposal_action.target_program = request.target_program;
    governance_proposal_action.target_account = request.target_account;
    governance_proposal_action.parameters_hash = request.parameters_hash;
    governance_proposal_action.evidence_hash = request.evidence_hash;
    governance_proposal_action.canonical_payload_hash = canonical_payload_hash;
    governance_proposal_action.schema_version = request.schema_version;
    governance_proposal_action.created_at = created_at;
    governance_proposal_action.bump = action_bump;

    Ok(())
}

pub fn validate_governance_proposal_action_v1(
    governance_proposal: &GovernanceProposalV1,
    governance_proposal_action: &GovernanceProposalActionV1,
    governance_proposal_key: Pubkey,
) -> Result<()> {
    require!(
        governance_proposal_action.governance_proposal != Pubkey::default(),
        CustomError::GovernanceProposalActionMissing
    );
    require_keys_eq!(
        governance_proposal_action.governance_proposal,
        governance_proposal_key,
        CustomError::GovernanceProposalActionMismatch
    );
    require!(
        governance_proposal_action.proposal_id == governance_proposal.proposal_id,
        CustomError::GovernanceProposalActionMismatch
    );
    require_keys_eq!(
        governance_proposal_action.proposer,
        governance_proposal.proposer,
        CustomError::GovernanceProposalActionMismatch
    );
    require!(
        governance_proposal_action.schema_version == 1,
        CustomError::InvalidGovernancePayloadSchema
    );

    let expected_module = map_governance_action_to_module(governance_proposal_action.action_type);
    require!(
        governance_proposal_action.module_id == expected_module,
        CustomError::GovernanceActionModuleMismatch
    );
    require!(
        governance_proposal.action_type
            == governance_action_stable_code_v1(governance_proposal_action.action_type),
        CustomError::InvalidGovernanceActionCode
    );
    require!(
        governance_proposal.proposal_type
            == map_governance_action_to_governance_proposal_type_v1(
                governance_proposal_action.action_type
            ),
        CustomError::GovernanceProposalActionMismatch
    );
    require_keys_eq!(
        governance_proposal.target_program,
        governance_proposal_action.target_program,
        CustomError::GovernanceActionTargetMismatch
    );
    require_keys_eq!(
        governance_proposal.target_account,
        governance_proposal_action.target_account,
        CustomError::GovernanceActionTargetMismatch
    );
    require!(
        governance_proposal.payload_hash == governance_proposal_action.canonical_payload_hash,
        CustomError::GovernanceProposalActionMismatch
    );
    require_keys_eq!(
        governance_proposal_action.target_program,
        crate::ID,
        CustomError::GovernanceActionTargetMismatch
    );
    require!(
        governance_proposal_action.target_account != Pubkey::default(),
        CustomError::GovernanceActionTargetMismatch
    );
    require!(
        governance_proposal_action.parameters_hash != [0u8; 32],
        CustomError::InvalidGovernanceProposal
    );

    let payload = GovernancePayloadV1 {
        schema_version: 1,
        action_type: governance_proposal_action.action_type,
        module_id: governance_proposal_action.module_id,
        target_program: governance_proposal_action.target_program,
        target_account: governance_proposal_action.target_account,
        parameters_hash: governance_proposal_action.parameters_hash,
        evidence_hash: governance_proposal_action.evidence_hash,
        created_at: governance_proposal_action.created_at,
    };
    let canonical_payload_hash = hash_governance_payload_v1(&payload)?;
    require!(
        canonical_payload_hash == governance_proposal_action.canonical_payload_hash,
        CustomError::GovernanceProposalActionMismatch
    );

    Ok(())
}

pub fn record_governance_position_init(
    governance_position: &mut GovernancePositionV1,
    owner: Pubkey,
    alpha_mint: Pubkey,
    vault: Pubkey,
    last_updated_at: i64,
    bump: u8,
) -> Result<()> {
    require!(
        owner != Pubkey::default(),
        CustomError::InvalidGovernancePosition
    );
    require!(
        alpha_mint != Pubkey::default(),
        CustomError::InvalidGovernancePosition
    );
    require!(
        vault != Pubkey::default(),
        CustomError::InvalidGovernanceVault
    );

    governance_position.owner = owner;
    governance_position.alpha_mint = alpha_mint;
    governance_position.vault = vault;
    governance_position.locked_amount = 0;
    governance_position.lock_start_time = 0;
    governance_position.lock_end_time = 0;
    governance_position.holding_multiplier_bps = 0;
    governance_position.voting_power = 0;
    governance_position.status = GovernancePositionStatusV1::Active;
    governance_position.last_updated_at = last_updated_at;
    governance_position.bump = bump;

    Ok(())
}

pub fn record_governance_position_vote_lock_init(
    governance_position_vote_lock: &mut GovernancePositionVoteLockV1,
    governance_position: Pubkey,
    updated_at: i64,
    bump: u8,
) -> Result<()> {
    require!(
        governance_position != Pubkey::default(),
        CustomError::InvalidGovernanceVoteLock
    );

    governance_position_vote_lock.governance_position = governance_position;
    governance_position_vote_lock.voting_lock_until = 0;
    governance_position_vote_lock.last_proposal = Pubkey::default();
    governance_position_vote_lock.updated_at = updated_at;
    governance_position_vote_lock.bump = bump;

    Ok(())
}

pub fn validate_governance_lock_params(
    governance_config: &GovernanceLockConfigV1,
    amount: u64,
    lock_duration_seconds: i64,
) -> Result<()> {
    require!(
        amount >= governance_config.min_lock_amount,
        CustomError::InvalidGovernanceLockAmount
    );
    require!(
        lock_duration_seconds >= governance_config.min_lock_duration_seconds
            && lock_duration_seconds <= governance_config.max_lock_duration_seconds,
        CustomError::InvalidGovernanceLockDuration
    );
    Ok(())
}

pub fn governance_time_multiplier_bps(lock_duration_seconds: i64) -> Result<u64> {
    require!(
        lock_duration_seconds >= GOVERNANCE_MIN_LOCK_DURATION_SECONDS
            && lock_duration_seconds <= GOVERNANCE_MAX_LOCK_DURATION_SECONDS,
        CustomError::InvalidGovernanceLockDuration
    );

    if lock_duration_seconds >= LOCK_365_DAYS_SECONDS {
        Ok(GOVERNANCE_365_DAY_MULTIPLIER_BPS)
    } else if lock_duration_seconds >= LOCK_180_DAYS_SECONDS {
        Ok(GOVERNANCE_180_DAY_MULTIPLIER_BPS)
    } else if lock_duration_seconds >= LOCK_90_DAYS_SECONDS {
        Ok(GOVERNANCE_90_DAY_MULTIPLIER_BPS)
    } else if lock_duration_seconds >= LOCK_30_DAYS_SECONDS {
        Ok(GOVERNANCE_30_DAY_MULTIPLIER_BPS)
    } else {
        err!(CustomError::InvalidGovernanceLockDuration)
    }
}

pub fn calculate_governance_voting_power(
    locked_amount: u64,
    lock_duration_seconds: i64,
) -> Result<u64> {
    require!(locked_amount > 0, CustomError::InvalidGovernanceLockAmount);

    let multiplier_bps = governance_time_multiplier_bps(lock_duration_seconds)?;
    calculate_governance_voting_power_with_multiplier(locked_amount, multiplier_bps)
}

pub fn calculate_governance_voting_power_with_multiplier(
    locked_amount: u64,
    multiplier_bps: u64,
) -> Result<u64> {
    require!(locked_amount > 0, CustomError::InvalidGovernanceLockAmount);
    require!(
        multiplier_bps >= BPS_DENOMINATOR && multiplier_bps <= GOVERNANCE_MAX_TIME_MULTIPLIER_BPS,
        CustomError::InvalidGovernanceLockDuration
    );

    let scaled_power = (locked_amount as u128)
        .checked_mul(multiplier_bps as u128)
        .and_then(|value| value.checked_div(BPS_DENOMINATOR as u128))
        .ok_or(CustomError::MathOverflow)?;
    require!(scaled_power <= u64::MAX as u128, CustomError::MathOverflow);
    Ok(scaled_power as u64)
}

pub fn record_governance_lock(
    governance_position: &mut GovernancePositionV1,
    governance_config: &GovernanceLockConfigV1,
    amount: u64,
    lock_duration_seconds: i64,
    now_ts: i64,
) -> Result<()> {
    validate_governance_lock_params(governance_config, amount, lock_duration_seconds)?;
    require!(
        governance_position.status == GovernancePositionStatusV1::Active,
        CustomError::InvalidGovernancePosition
    );
    require_keys_eq!(
        governance_position.alpha_mint,
        governance_config.alpha_mint,
        CustomError::InvalidMint
    );
    require_keys_eq!(
        governance_position.vault,
        governance_config.governance_vault,
        CustomError::InvalidGovernanceVault
    );

    let new_locked_amount = governance_position
        .locked_amount
        .checked_add(amount)
        .ok_or(CustomError::MathOverflow)?;
    let new_lock_end = now_ts
        .checked_add(lock_duration_seconds)
        .ok_or(CustomError::MathOverflow)?;
    let effective_lock_end = governance_position.lock_end_time.max(new_lock_end);
    let requested_multiplier_bps = governance_time_multiplier_bps(lock_duration_seconds)?;
    let holding_multiplier_bps = governance_position
        .holding_multiplier_bps
        .max(requested_multiplier_bps);
    let voting_power = calculate_governance_voting_power_with_multiplier(
        new_locked_amount,
        holding_multiplier_bps,
    )?;

    if governance_position.locked_amount == 0 {
        governance_position.lock_start_time = now_ts;
    }
    governance_position.lock_end_time = effective_lock_end;
    governance_position.locked_amount = new_locked_amount;
    governance_position.holding_multiplier_bps = holding_multiplier_bps;
    governance_position.voting_power = voting_power;
    governance_position.last_updated_at = now_ts;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn record_governance_power_after_lock(
    governance_power_state: &mut GovernancePowerStateV1,
    governance_lock_config: Pubkey,
    previous_locked_amount: u64,
    previous_voting_power: u64,
    new_locked_amount: u64,
    new_voting_power: u64,
    updated_at: i64,
) -> Result<()> {
    require_keys_eq!(
        governance_power_state.governance_lock_config,
        governance_lock_config,
        CustomError::InvalidGovernancePowerState
    );
    require!(
        new_locked_amount >= previous_locked_amount,
        CustomError::InvalidGovernancePowerState
    );

    let locked_delta = new_locked_amount
        .checked_sub(previous_locked_amount)
        .ok_or(CustomError::MathOverflow)?;
    let voting_power_without_previous = governance_power_state
        .total_voting_power
        .checked_sub(previous_voting_power)
        .ok_or(CustomError::MathOverflow)?;

    governance_power_state.total_locked_alpha = governance_power_state
        .total_locked_alpha
        .checked_add(locked_delta)
        .ok_or(CustomError::MathOverflow)?;
    governance_power_state.total_voting_power = voting_power_without_previous
        .checked_add(new_voting_power)
        .ok_or(CustomError::MathOverflow)?;
    if previous_locked_amount == 0 && new_locked_amount > 0 {
        governance_power_state.active_position_count = governance_power_state
            .active_position_count
            .checked_add(1)
            .ok_or(CustomError::MathOverflow)?;
    }
    governance_power_state.updated_at = updated_at;

    Ok(())
}

pub fn validate_governance_unlock(
    governance_position: &GovernancePositionV1,
    vault_balance: u64,
    now_ts: i64,
) -> Result<u64> {
    require!(
        governance_position.status == GovernancePositionStatusV1::Active,
        CustomError::InvalidGovernancePosition
    );
    require!(
        governance_position.locked_amount > 0,
        CustomError::InvalidGovernanceLockAmount
    );
    require!(
        now_ts >= governance_position.lock_end_time,
        CustomError::GovernanceLockStillActive
    );
    require!(
        vault_balance >= governance_position.locked_amount,
        CustomError::InsufficientVaultBalance
    );
    Ok(governance_position.locked_amount)
}

pub fn validate_governance_unlock_with_vote_lock(
    governance_position: &GovernancePositionV1,
    governance_position_key: Pubkey,
    governance_position_vote_lock: &GovernancePositionVoteLockV1,
    vault_balance: u64,
    now_ts: i64,
) -> Result<u64> {
    require_keys_eq!(
        governance_position_vote_lock.governance_position,
        governance_position_key,
        CustomError::InvalidGovernanceVoteLock
    );
    require!(
        now_ts >= governance_position_vote_lock.voting_lock_until,
        CustomError::GovernanceVoteLockActive
    );

    validate_governance_unlock(governance_position, vault_balance, now_ts)
}

pub fn record_governance_power_after_unlock(
    governance_power_state: &mut GovernancePowerStateV1,
    governance_lock_config: Pubkey,
    locked_amount: u64,
    voting_power: u64,
    updated_at: i64,
) -> Result<()> {
    require_keys_eq!(
        governance_power_state.governance_lock_config,
        governance_lock_config,
        CustomError::InvalidGovernancePowerState
    );
    require!(locked_amount > 0, CustomError::InvalidGovernanceLockAmount);

    governance_power_state.total_locked_alpha = governance_power_state
        .total_locked_alpha
        .checked_sub(locked_amount)
        .ok_or(CustomError::MathOverflow)?;
    governance_power_state.total_voting_power = governance_power_state
        .total_voting_power
        .checked_sub(voting_power)
        .ok_or(CustomError::MathOverflow)?;
    governance_power_state.active_position_count = governance_power_state
        .active_position_count
        .checked_sub(1)
        .ok_or(CustomError::MathOverflow)?;
    governance_power_state.updated_at = updated_at;

    Ok(())
}

pub fn record_governance_unlock(
    governance_position: &mut GovernancePositionV1,
    now_ts: i64,
) -> Result<()> {
    require!(
        governance_position.locked_amount > 0,
        CustomError::InvalidGovernanceLockAmount
    );

    governance_position.locked_amount = 0;
    governance_position.lock_start_time = 0;
    governance_position.lock_end_time = 0;
    governance_position.holding_multiplier_bps = 0;
    governance_position.voting_power = 0;
    governance_position.status = GovernancePositionStatusV1::Closed;
    governance_position.last_updated_at = now_ts;

    Ok(())
}

pub fn record_governance_snapshot_init(
    governance_snapshot: &mut GovernanceSnapshotV1,
    proposal: Pubkey,
    created_at: i64,
    bump: u8,
) -> Result<()> {
    require!(
        proposal != Pubkey::default(),
        CustomError::InvalidGovernanceProposal
    );

    governance_snapshot.proposal = proposal;
    governance_snapshot.total_voting_power = 0;
    governance_snapshot.yes_weight = 0;
    governance_snapshot.no_weight = 0;
    governance_snapshot.abstain_weight = 0;
    governance_snapshot.created_at = created_at;
    governance_snapshot.finalized = false;
    governance_snapshot.bump = bump;

    Ok(())
}

pub fn record_governance_snapshot_create(
    governance_proposal: &mut GovernanceProposalV1,
    governance_proposal_action: &GovernanceProposalActionV1,
    governance_snapshot: &mut GovernanceSnapshotV1,
    governance_voting_config: &GovernanceVotingConfigV1,
    governance_power_state: &GovernancePowerStateV1,
    proposal_key: Pubkey,
    snapshot_key: Pubkey,
    created_at: i64,
    bump: u8,
) -> Result<()> {
    require!(
        governance_proposal.status == GovernanceProposalStatusV1::Draft,
        CustomError::InvalidGovernanceProposal
    );
    require!(
        governance_proposal.snapshot == Pubkey::default(),
        CustomError::InvalidGovernanceSnapshot
    );
    validate_governance_proposal_action_v1(
        governance_proposal,
        governance_proposal_action,
        proposal_key,
    )?;
    require!(
        governance_power_state.total_voting_power > 0,
        CustomError::InvalidGovernanceSnapshot
    );
    require!(
        !governance_snapshot.finalized,
        CustomError::ProposalAlreadyFinalized
    );
    validate_governance_voting_config(
        governance_voting_config.quorum_bps,
        governance_voting_config.approval_threshold_bps,
        governance_voting_config.voting_period_seconds,
    )?;

    let voting_end_ts = created_at
        .checked_add(governance_voting_config.voting_period_seconds)
        .ok_or(CustomError::MathOverflow)?;

    governance_snapshot.proposal = proposal_key;
    governance_snapshot.total_voting_power = governance_power_state.total_voting_power;
    governance_snapshot.yes_weight = 0;
    governance_snapshot.no_weight = 0;
    governance_snapshot.abstain_weight = 0;
    governance_snapshot.created_at = created_at;
    governance_snapshot.finalized = false;
    governance_snapshot.bump = bump;

    governance_proposal.status = GovernanceProposalStatusV1::Voting;
    governance_proposal.voting_start_ts = created_at;
    governance_proposal.voting_end_ts = voting_end_ts;
    governance_proposal.snapshot = snapshot_key;
    governance_proposal.yes_weight = 0;
    governance_proposal.no_weight = 0;
    governance_proposal.abstain_weight = 0;
    governance_proposal.finalized_at = 0;

    Ok(())
}

pub fn record_cast_governance_vote(
    governance_proposal: &mut GovernanceProposalV1,
    governance_snapshot: &mut GovernanceSnapshotV1,
    governance_position: &GovernancePositionV1,
    governance_position_vote_lock: &mut GovernancePositionVoteLockV1,
    vote_record: &mut VoteRecordV1,
    governance_proposal_key: Pubkey,
    governance_position_key: Pubkey,
    choice: VoteChoiceV1,
    now_ts: i64,
    bump: u8,
) -> Result<()> {
    require!(
        governance_proposal.status == GovernanceProposalStatusV1::Voting,
        CustomError::ProposalNotVoting
    );
    require!(
        governance_proposal.snapshot != Pubkey::default()
            && governance_snapshot.proposal != Pubkey::default(),
        CustomError::SnapshotMissing
    );
    require!(
        !governance_snapshot.finalized,
        CustomError::ProposalAlreadyFinalized
    );
    require!(
        now_ts >= governance_proposal.voting_start_ts,
        CustomError::ProposalNotVoting
    );
    require!(
        now_ts <= governance_proposal.voting_end_ts,
        CustomError::VotingPeriodEnded
    );
    require!(
        vote_record.proposal == Pubkey::default()
            && vote_record.voter_position == Pubkey::default(),
        CustomError::AlreadyVoted
    );
    require!(
        governance_position.status == GovernancePositionStatusV1::Active,
        CustomError::InvalidGovernancePosition
    );
    require!(
        governance_position.last_updated_at <= governance_snapshot.created_at,
        CustomError::InvalidGovernancePosition
    );
    require!(
        governance_position.voting_power > 0,
        CustomError::InvalidGovernanceVote
    );

    let voting_power = governance_position.voting_power;
    match choice {
        VoteChoiceV1::Yes => {
            governance_snapshot.yes_weight = governance_snapshot
                .yes_weight
                .checked_add(voting_power)
                .ok_or(CustomError::MathOverflow)?;
        }
        VoteChoiceV1::No => {
            governance_snapshot.no_weight = governance_snapshot
                .no_weight
                .checked_add(voting_power)
                .ok_or(CustomError::MathOverflow)?;
        }
        VoteChoiceV1::Abstain => {
            governance_snapshot.abstain_weight = governance_snapshot
                .abstain_weight
                .checked_add(voting_power)
                .ok_or(CustomError::MathOverflow)?;
        }
    }

    let total_votes = checked_governance_total_votes(governance_snapshot)?;
    require!(
        total_votes <= governance_snapshot.total_voting_power,
        CustomError::InvalidGovernanceVote
    );

    vote_record.proposal = governance_proposal_key;
    vote_record.voter_position = governance_position_key;
    vote_record.choice = choice;
    vote_record.voting_power_used = voting_power;
    vote_record.timestamp = now_ts;
    vote_record.bump = bump;

    record_governance_position_vote_lock_after_vote(
        governance_position_vote_lock,
        governance_position_key,
        governance_proposal_key,
        governance_proposal.voting_end_ts,
        now_ts,
    )?;

    Ok(())
}

pub fn record_governance_position_vote_lock_after_vote(
    governance_position_vote_lock: &mut GovernancePositionVoteLockV1,
    governance_position_key: Pubkey,
    governance_proposal_key: Pubkey,
    voting_end_ts: i64,
    updated_at: i64,
) -> Result<()> {
    require_keys_eq!(
        governance_position_vote_lock.governance_position,
        governance_position_key,
        CustomError::InvalidGovernanceVoteLock
    );
    require!(
        governance_proposal_key != Pubkey::default() && voting_end_ts > 0,
        CustomError::InvalidGovernanceVoteLock
    );

    governance_position_vote_lock.voting_lock_until = governance_position_vote_lock
        .voting_lock_until
        .max(voting_end_ts);
    governance_position_vote_lock.last_proposal = governance_proposal_key;
    governance_position_vote_lock.updated_at = updated_at;

    Ok(())
}

pub fn checked_governance_total_votes(snapshot: &GovernanceSnapshotV1) -> Result<u64> {
    snapshot
        .yes_weight
        .checked_add(snapshot.no_weight)
        .and_then(|value| value.checked_add(snapshot.abstain_weight))
        .ok_or(CustomError::MathOverflow.into())
}

pub fn has_governance_quorum(
    total_votes: u64,
    total_voting_power: u64,
    quorum_bps: u64,
) -> Result<bool> {
    require!(
        total_voting_power > 0,
        CustomError::InvalidGovernanceSnapshot
    );
    let vote_bps_side = (total_votes as u128)
        .checked_mul(BPS_DENOMINATOR as u128)
        .ok_or(CustomError::MathOverflow)?;
    let quorum_side = (total_voting_power as u128)
        .checked_mul(quorum_bps as u128)
        .ok_or(CustomError::MathOverflow)?;
    Ok(vote_bps_side >= quorum_side)
}

pub fn has_governance_approval(
    yes_weight: u64,
    no_weight: u64,
    approval_threshold_bps: u64,
) -> Result<bool> {
    let decisive_votes = yes_weight
        .checked_add(no_weight)
        .ok_or(CustomError::MathOverflow)?;
    require!(decisive_votes > 0, CustomError::QuorumNotReached);

    let yes_bps_side = (yes_weight as u128)
        .checked_mul(BPS_DENOMINATOR as u128)
        .ok_or(CustomError::MathOverflow)?;
    let approval_side = (decisive_votes as u128)
        .checked_mul(approval_threshold_bps as u128)
        .ok_or(CustomError::MathOverflow)?;
    Ok(yes_bps_side >= approval_side)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GovernanceThresholdPolicyV1 {
    pub quorum_bps: u64,
    pub approval_threshold_bps: u64,
}

pub fn governance_threshold_policy_for_proposal_type(
    proposal_type: GovernanceProposalTypeV1,
) -> GovernanceThresholdPolicyV1 {
    match proposal_type {
        GovernanceProposalTypeV1::Contributor => GovernanceThresholdPolicyV1 {
            quorum_bps: 500,
            approval_threshold_bps: 6_000,
        },
        GovernanceProposalTypeV1::Treasury => GovernanceThresholdPolicyV1 {
            quorum_bps: 1_000,
            approval_threshold_bps: 6_667,
        },
        GovernanceProposalTypeV1::Parameter => GovernanceThresholdPolicyV1 {
            quorum_bps: 2_000,
            approval_threshold_bps: 7_500,
        },
        GovernanceProposalTypeV1::Upgrade => GovernanceThresholdPolicyV1 {
            quorum_bps: 2_500,
            approval_threshold_bps: 8_000,
        },
        GovernanceProposalTypeV1::Emergency => GovernanceThresholdPolicyV1 {
            quorum_bps: 1_500,
            approval_threshold_bps: 7_500,
        },
        GovernanceProposalTypeV1::GreenLabel
        | GovernanceProposalTypeV1::VictimRelief
        | GovernanceProposalTypeV1::ScamRegistry => GovernanceThresholdPolicyV1 {
            quorum_bps: 1_000,
            approval_threshold_bps: 6_667,
        },
    }
}

pub fn validate_governance_thresholds(
    governance_snapshot: &GovernanceSnapshotV1,
    proposal_type: GovernanceProposalTypeV1,
) -> Result<bool> {
    let total_votes = checked_governance_total_votes(governance_snapshot)?;
    let threshold_policy = governance_threshold_policy_for_proposal_type(proposal_type);
    require!(
        has_governance_quorum(
            total_votes,
            governance_snapshot.total_voting_power,
            threshold_policy.quorum_bps,
        )?,
        CustomError::QuorumNotReached
    );

    has_governance_approval(
        governance_snapshot.yes_weight,
        governance_snapshot.no_weight,
        threshold_policy.approval_threshold_bps,
    )
}

pub fn record_finalize_governance_vote(
    governance_proposal: &mut GovernanceProposalV1,
    governance_snapshot: &mut GovernanceSnapshotV1,
    governance_voting_config: &GovernanceVotingConfigV1,
    now_ts: i64,
) -> Result<()> {
    validate_governance_voting_config(
        governance_voting_config.quorum_bps,
        governance_voting_config.approval_threshold_bps,
        governance_voting_config.voting_period_seconds,
    )?;
    require!(
        governance_proposal.status == GovernanceProposalStatusV1::Voting,
        CustomError::ProposalNotVoting
    );
    require!(
        governance_proposal.snapshot != Pubkey::default()
            && governance_snapshot.proposal != Pubkey::default(),
        CustomError::SnapshotMissing
    );
    require!(
        !governance_snapshot.finalized,
        CustomError::ProposalAlreadyFinalized
    );
    require!(
        now_ts >= governance_proposal.voting_end_ts,
        CustomError::VotingPeriodNotEnded
    );

    let passed =
        validate_governance_thresholds(governance_snapshot, governance_proposal.proposal_type)?;

    governance_proposal.yes_weight = governance_snapshot.yes_weight;
    governance_proposal.no_weight = governance_snapshot.no_weight;
    governance_proposal.abstain_weight = governance_snapshot.abstain_weight;
    governance_proposal.finalized_at = now_ts;
    governance_snapshot.finalized = true;

    if passed {
        governance_proposal.status = GovernanceProposalStatusV1::Passed;
    } else {
        governance_proposal.status = GovernanceProposalStatusV1::Rejected;
    }

    Ok(())
}

pub fn record_vote_record_init(
    vote_record: &mut VoteRecordV1,
    proposal: Pubkey,
    voter_position: Pubkey,
    bump: u8,
) -> Result<()> {
    require!(
        proposal != Pubkey::default(),
        CustomError::InvalidGovernanceProposal
    );
    require!(
        voter_position != Pubkey::default(),
        CustomError::InvalidGovernancePosition
    );

    vote_record.proposal = proposal;
    vote_record.voter_position = voter_position;
    vote_record.choice = VoteChoiceV1::Abstain;
    vote_record.voting_power_used = 0;
    vote_record.timestamp = 0;
    vote_record.bump = bump;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{GovernanceActionTypeV1, ProtocolModuleIdV1};

    const PROPOSAL_ID: u64 = 7;
    const PROPOSER: Pubkey = Pubkey::new_from_array([1; 32]);
    const OWNER: Pubkey = Pubkey::new_from_array([2; 32]);
    const ALPHA_MINT: Pubkey = Pubkey::new_from_array([3; 32]);
    const TARGET_PROGRAM: Pubkey = Pubkey::new_from_array([4; 32]);
    const TARGET_ACCOUNT: Pubkey = Pubkey::new_from_array([5; 32]);
    const PROPOSAL_KEY: Pubkey = Pubkey::new_from_array([6; 32]);
    const POSITION_KEY: Pubkey = Pubkey::new_from_array([7; 32]);
    const GOVERNANCE_VAULT: Pubkey = Pubkey::new_from_array([8; 32]);
    const SNAPSHOT_KEY: Pubkey = Pubkey::new_from_array([10; 32]);
    const POSITION_TWO_KEY: Pubkey = Pubkey::new_from_array([11; 32]);
    const GOVERNANCE_CONFIG_KEY: Pubkey = Pubkey::new_from_array([12; 32]);
    const PAYLOAD_HASH: [u8; 32] = [9; 32];

    fn default_config() -> GovernanceLockConfigV1 {
        GovernanceLockConfigV1 {
            authority: PROPOSER,
            alpha_mint: ALPHA_MINT,
            governance_vault: GOVERNANCE_VAULT,
            min_lock_amount: GOVERNANCE_DEFAULT_MIN_LOCK_AMOUNT,
            min_lock_duration_seconds: GOVERNANCE_MIN_LOCK_DURATION_SECONDS,
            max_lock_duration_seconds: GOVERNANCE_MAX_LOCK_DURATION_SECONDS,
            max_time_multiplier_bps: GOVERNANCE_MAX_TIME_MULTIPLIER_BPS,
            created_at: 1,
            bump: 1,
        }
    }

    fn default_voting_config() -> GovernanceVotingConfigV1 {
        GovernanceVotingConfigV1 {
            authority: PROPOSER,
            quorum_bps: GOVERNANCE_DEFAULT_QUORUM_BPS,
            approval_threshold_bps: GOVERNANCE_DEFAULT_APPROVAL_THRESHOLD_BPS,
            voting_period_seconds: GOVERNANCE_DEFAULT_VOTING_PERIOD_SECONDS,
            created_at: 1,
            bump: 1,
        }
    }

    fn default_proposal() -> GovernanceProposalV1 {
        GovernanceProposalV1 {
            proposal_id: 0,
            proposer: Pubkey::default(),
            proposal_type: GovernanceProposalTypeV1::Contributor,
            action_type: 0,
            target_program: Pubkey::default(),
            target_account: Pubkey::default(),
            payload_hash: [0; 32],
            status: GovernanceProposalStatusV1::Cancelled,
            voting_start_ts: 0,
            voting_end_ts: 0,
            created_at: 0,
            snapshot: Pubkey::default(),
            yes_weight: 0,
            no_weight: 0,
            abstain_weight: 0,
            finalized_at: 0,
            bump: 0,
        }
    }

    fn default_action_request(action_type: GovernanceActionTypeV1) -> GovernanceActionRequestV1 {
        GovernanceActionRequestV1 {
            schema_version: 1,
            action_type,
            module_id: map_governance_action_to_module(action_type),
            target_program: crate::ID,
            target_account: TARGET_ACCOUNT,
            parameters_hash: PAYLOAD_HASH,
            evidence_hash: [0; 32],
        }
    }

    fn default_proposal_action() -> GovernanceProposalActionV1 {
        GovernanceProposalActionV1 {
            governance_proposal: Pubkey::default(),
            proposal_id: 0,
            proposer: Pubkey::default(),
            action_type: GovernanceActionTypeV1::ContributorAdd,
            module_id: ProtocolModuleIdV1::Contributor,
            target_program: Pubkey::default(),
            target_account: Pubkey::default(),
            parameters_hash: [0; 32],
            evidence_hash: [0; 32],
            canonical_payload_hash: [0; 32],
            schema_version: 0,
            created_at: 0,
            bump: 0,
        }
    }

    fn strict_proposal_and_action(
        action_type: GovernanceActionTypeV1,
    ) -> (GovernanceProposalV1, GovernanceProposalActionV1) {
        let mut proposal = default_proposal();
        let mut action = default_proposal_action();
        let request = default_action_request(action_type);
        record_governance_proposal_with_action_init(
            &mut proposal,
            &mut action,
            PROPOSAL_KEY,
            PROPOSAL_ID,
            PROPOSER,
            &request,
            80,
            1,
            2,
        )
        .unwrap();
        (proposal, action)
    }

    fn default_position() -> GovernancePositionV1 {
        GovernancePositionV1 {
            owner: Pubkey::default(),
            alpha_mint: Pubkey::default(),
            vault: Pubkey::default(),
            locked_amount: 1,
            lock_start_time: 2,
            lock_end_time: 3,
            holding_multiplier_bps: 4,
            voting_power: 5,
            status: GovernancePositionStatusV1::Closed,
            last_updated_at: 6,
            bump: 0,
        }
    }

    fn active_position() -> GovernancePositionV1 {
        let mut position = default_position();
        record_governance_position_init(&mut position, OWNER, ALPHA_MINT, GOVERNANCE_VAULT, 10, 2)
            .unwrap();
        position
    }

    fn active_position_with_power(voting_power: u64, last_updated_at: i64) -> GovernancePositionV1 {
        let mut position = active_position();
        position.locked_amount = voting_power;
        position.lock_start_time = 1;
        position.lock_end_time = 1_000;
        position.holding_multiplier_bps = BPS_DENOMINATOR;
        position.voting_power = voting_power;
        position.last_updated_at = last_updated_at;
        position
    }

    fn default_snapshot() -> GovernanceSnapshotV1 {
        GovernanceSnapshotV1 {
            proposal: Pubkey::default(),
            total_voting_power: 1,
            yes_weight: 1,
            no_weight: 1,
            abstain_weight: 1,
            created_at: 0,
            finalized: true,
            bump: 0,
        }
    }

    fn blank_snapshot() -> GovernanceSnapshotV1 {
        GovernanceSnapshotV1 {
            proposal: Pubkey::default(),
            total_voting_power: 0,
            yes_weight: 0,
            no_weight: 0,
            abstain_weight: 0,
            created_at: 0,
            finalized: false,
            bump: 0,
        }
    }

    fn default_vote_record() -> VoteRecordV1 {
        VoteRecordV1 {
            proposal: Pubkey::default(),
            voter_position: Pubkey::default(),
            choice: VoteChoiceV1::Yes,
            voting_power_used: 100,
            timestamp: 100,
            bump: 0,
        }
    }

    fn power_state_with_total(total_voting_power: u64) -> GovernancePowerStateV1 {
        GovernancePowerStateV1 {
            governance_lock_config: GOVERNANCE_CONFIG_KEY,
            total_locked_alpha: total_voting_power,
            total_voting_power,
            active_position_count: 1,
            updated_at: 90,
            bump: 1,
        }
    }

    fn vote_lock_for_position(position_key: Pubkey) -> GovernancePositionVoteLockV1 {
        GovernancePositionVoteLockV1 {
            governance_position: position_key,
            voting_lock_until: 0,
            last_proposal: Pubkey::default(),
            updated_at: 0,
            bump: 1,
        }
    }

    fn voting_proposal_and_snapshot(
        total_voting_power: u64,
    ) -> (GovernanceProposalV1, GovernanceSnapshotV1) {
        let (mut proposal, action) =
            strict_proposal_and_action(GovernanceActionTypeV1::ContributorAdd);
        let mut snapshot = blank_snapshot();
        let power_state = power_state_with_total(total_voting_power);
        record_governance_snapshot_create(
            &mut proposal,
            &action,
            &mut snapshot,
            &default_voting_config(),
            &power_state,
            PROPOSAL_KEY,
            SNAPSHOT_KEY,
            100,
            3,
        )
        .unwrap();
        (proposal, snapshot)
    }

    #[test]
    fn initialize_governance_config_defaults_match_constants() {
        let mut config = default_config();
        record_governance_config_init(&mut config, PROPOSER, ALPHA_MINT, GOVERNANCE_VAULT, 100, 7)
            .unwrap();

        assert_eq!(config.authority, PROPOSER);
        assert_eq!(config.alpha_mint, ALPHA_MINT);
        assert_eq!(config.governance_vault, GOVERNANCE_VAULT);
        assert_eq!(config.min_lock_amount, GOVERNANCE_DEFAULT_MIN_LOCK_AMOUNT);
        assert_eq!(
            config.min_lock_duration_seconds,
            GOVERNANCE_MIN_LOCK_DURATION_SECONDS
        );
        assert_eq!(
            config.max_lock_duration_seconds,
            GOVERNANCE_MAX_LOCK_DURATION_SECONDS
        );
        assert_eq!(
            config.max_time_multiplier_bps,
            GOVERNANCE_MAX_TIME_MULTIPLIER_BPS
        );
        assert_eq!(config.created_at, 100);
        assert_eq!(config.bump, 7);
    }

    #[test]
    fn initialize_governance_power_state_defaults_to_zero_totals() {
        let mut power_state = power_state_with_total(42);
        record_governance_power_state_init(&mut power_state, GOVERNANCE_CONFIG_KEY, 100, 8)
            .unwrap();

        assert_eq!(power_state.governance_lock_config, GOVERNANCE_CONFIG_KEY);
        assert_eq!(power_state.total_locked_alpha, 0);
        assert_eq!(power_state.total_voting_power, 0);
        assert_eq!(power_state.active_position_count, 0);
        assert_eq!(power_state.updated_at, 100);
        assert_eq!(power_state.bump, 8);
    }

    #[test]
    fn initialize_governance_position_vote_lock_defaults_unlocked() {
        let mut vote_lock = vote_lock_for_position(POSITION_KEY);
        record_governance_position_vote_lock_init(&mut vote_lock, POSITION_KEY, 100, 9).unwrap();

        assert_eq!(vote_lock.governance_position, POSITION_KEY);
        assert_eq!(vote_lock.voting_lock_until, 0);
        assert_eq!(vote_lock.last_proposal, Pubkey::default());
        assert_eq!(vote_lock.updated_at, 100);
        assert_eq!(vote_lock.bump, 9);
    }

    #[test]
    fn initialize_governance_voting_config_defaults_match_policy() {
        let mut config = default_voting_config();
        record_governance_voting_config_init(&mut config, PROPOSER, 100, 8).unwrap();

        assert_eq!(config.authority, PROPOSER);
        assert_eq!(config.quorum_bps, 500);
        assert_eq!(config.approval_threshold_bps, 6_000);
        assert_eq!(
            config.voting_period_seconds,
            GOVERNANCE_DEFAULT_VOTING_PERIOD_SECONDS
        );
        assert_eq!(config.created_at, 100);
        assert_eq!(config.bump, 8);
    }

    #[test]
    fn initialize_governance_proposal_defaults_to_draft() {
        let mut proposal = default_proposal();
        record_governance_proposal_init(
            &mut proposal,
            PROPOSAL_ID,
            PROPOSER,
            GovernanceProposalTypeV1::Contributor,
            3,
            TARGET_PROGRAM,
            TARGET_ACCOUNT,
            PAYLOAD_HASH,
            10,
            20,
            9,
            1,
        )
        .unwrap();

        assert_eq!(proposal.proposal_id, PROPOSAL_ID);
        assert_eq!(proposal.proposer, PROPOSER);
        assert_eq!(proposal.status, GovernanceProposalStatusV1::Draft);
        assert_eq!(proposal.voting_start_ts, 10);
        assert_eq!(proposal.voting_end_ts, 20);
        assert_eq!(proposal.bump, 1);
    }

    #[test]
    fn initialize_governance_proposal_allows_zero_draft_voting_window() {
        validate_governance_proposal_init(
            PROPOSAL_ID,
            TARGET_PROGRAM,
            TARGET_ACCOUNT,
            PAYLOAD_HASH,
            0,
            0,
        )
        .unwrap();
    }

    #[test]
    fn initialize_governance_proposal_rejects_invalid_time_window() {
        let err = validate_governance_proposal_init(
            PROPOSAL_ID,
            TARGET_PROGRAM,
            TARGET_ACCOUNT,
            PAYLOAD_HASH,
            20,
            10,
        )
        .unwrap_err();
        assert_eq!(err, CustomError::InvalidGovernanceProposalTime.into());
    }

    #[test]
    fn initialize_governance_proposal_rejects_zero_payload_hash() {
        let err = validate_governance_proposal_init(
            PROPOSAL_ID,
            TARGET_PROGRAM,
            TARGET_ACCOUNT,
            [0; 32],
            0,
            0,
        )
        .unwrap_err();
        assert_eq!(err, CustomError::InvalidGovernanceProposal.into());
    }

    #[test]
    fn initialize_governance_proposal_with_action_sets_sidecar_and_mirrors() {
        let (proposal, action) =
            strict_proposal_and_action(GovernanceActionTypeV1::TreasuryApproveSpending);

        assert_eq!(proposal.proposal_type, GovernanceProposalTypeV1::Treasury);
        assert_eq!(
            proposal.action_type,
            governance_action_stable_code_v1(GovernanceActionTypeV1::TreasuryApproveSpending)
        );
        assert_eq!(proposal.target_program, crate::ID);
        assert_eq!(proposal.target_account, TARGET_ACCOUNT);
        assert_eq!(proposal.payload_hash, action.canonical_payload_hash);
        assert_eq!(action.governance_proposal, PROPOSAL_KEY);
        assert_eq!(action.proposal_id, PROPOSAL_ID);
        assert_eq!(action.proposer, PROPOSER);
        assert_eq!(action.module_id, ProtocolModuleIdV1::Treasury);
        validate_governance_proposal_action_v1(&proposal, &action, PROPOSAL_KEY).unwrap();
    }

    #[test]
    fn initialize_governance_proposal_with_action_rejects_wrong_module() {
        let mut proposal = default_proposal();
        let mut action = default_proposal_action();
        let mut request = default_action_request(GovernanceActionTypeV1::ContributorAdd);
        request.module_id = ProtocolModuleIdV1::Treasury;

        let err = record_governance_proposal_with_action_init(
            &mut proposal,
            &mut action,
            PROPOSAL_KEY,
            PROPOSAL_ID,
            PROPOSER,
            &request,
            80,
            1,
            2,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::GovernanceActionModuleMismatch.into());
    }

    #[test]
    fn initialize_governance_proposal_with_action_rejects_wrong_target_program() {
        let mut proposal = default_proposal();
        let mut action = default_proposal_action();
        let mut request = default_action_request(GovernanceActionTypeV1::ContributorAdd);
        request.target_program = TARGET_PROGRAM;

        let err = record_governance_proposal_with_action_init(
            &mut proposal,
            &mut action,
            PROPOSAL_KEY,
            PROPOSAL_ID,
            PROPOSER,
            &request,
            80,
            1,
            2,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::GovernanceActionTargetMismatch.into());
    }

    #[test]
    fn initialize_governance_proposal_with_action_rejects_zero_target_account() {
        let mut proposal = default_proposal();
        let mut action = default_proposal_action();
        let mut request = default_action_request(GovernanceActionTypeV1::ContributorAdd);
        request.target_account = Pubkey::default();

        let err = record_governance_proposal_with_action_init(
            &mut proposal,
            &mut action,
            PROPOSAL_KEY,
            PROPOSAL_ID,
            PROPOSER,
            &request,
            80,
            1,
            2,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::GovernanceActionTargetMismatch.into());
    }

    #[test]
    fn initialize_governance_proposal_with_action_rejects_zero_parameters_hash() {
        let mut proposal = default_proposal();
        let mut action = default_proposal_action();
        let mut request = default_action_request(GovernanceActionTypeV1::ContributorAdd);
        request.parameters_hash = [0; 32];

        let err = record_governance_proposal_with_action_init(
            &mut proposal,
            &mut action,
            PROPOSAL_KEY,
            PROPOSAL_ID,
            PROPOSER,
            &request,
            80,
            1,
            2,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::InvalidGovernanceProposal.into());
    }

    #[test]
    fn initialize_governance_proposal_with_action_rejects_duplicate_sidecar() {
        let (mut proposal, mut action) =
            strict_proposal_and_action(GovernanceActionTypeV1::ContributorAdd);
        let request = default_action_request(GovernanceActionTypeV1::ContributorAdd);

        let err = record_governance_proposal_with_action_init(
            &mut proposal,
            &mut action,
            PROPOSAL_KEY,
            PROPOSAL_ID,
            PROPOSER,
            &request,
            80,
            1,
            2,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::GovernanceProposalActionMismatch.into());
    }

    #[test]
    fn legacy_proposal_without_action_sidecar_cannot_create_snapshot() {
        let mut proposal = default_proposal();
        record_governance_proposal_init(
            &mut proposal,
            PROPOSAL_ID,
            PROPOSER,
            GovernanceProposalTypeV1::Contributor,
            3,
            TARGET_PROGRAM,
            TARGET_ACCOUNT,
            PAYLOAD_HASH,
            0,
            0,
            80,
            1,
        )
        .unwrap();
        let action = default_proposal_action();
        let mut snapshot = blank_snapshot();
        let power_state = power_state_with_total(100);

        let err = record_governance_snapshot_create(
            &mut proposal,
            &action,
            &mut snapshot,
            &default_voting_config(),
            &power_state,
            PROPOSAL_KEY,
            SNAPSHOT_KEY,
            100,
            3,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::GovernanceProposalActionMissing.into());
    }

    #[test]
    fn governance_action_mismatch_blocks_snapshot() {
        let (mut proposal, action) =
            strict_proposal_and_action(GovernanceActionTypeV1::ContributorAdd);
        proposal.action_type =
            governance_action_stable_code_v1(GovernanceActionTypeV1::ContributorRemove);
        let mut snapshot = blank_snapshot();
        let power_state = power_state_with_total(100);

        let err = record_governance_snapshot_create(
            &mut proposal,
            &action,
            &mut snapshot,
            &default_voting_config(),
            &power_state,
            PROPOSAL_KEY,
            SNAPSHOT_KEY,
            100,
            3,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::InvalidGovernanceActionCode.into());

        let (mut proposal, mut action) =
            strict_proposal_and_action(GovernanceActionTypeV1::ContributorAdd);
        proposal.proposal_type = GovernanceProposalTypeV1::Treasury;
        action.module_id = ProtocolModuleIdV1::Contributor;
        let mut snapshot = blank_snapshot();
        let err = record_governance_snapshot_create(
            &mut proposal,
            &action,
            &mut snapshot,
            &default_voting_config(),
            &power_state,
            PROPOSAL_KEY,
            SNAPSHOT_KEY,
            100,
            3,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::GovernanceProposalActionMismatch.into());
    }

    #[test]
    fn governance_payload_hash_mismatch_blocks_snapshot() {
        let (mut proposal, mut action) =
            strict_proposal_and_action(GovernanceActionTypeV1::ContributorAdd);
        action.canonical_payload_hash = [6; 32];
        let mut snapshot = blank_snapshot();
        let power_state = power_state_with_total(100);

        let err = record_governance_snapshot_create(
            &mut proposal,
            &action,
            &mut snapshot,
            &default_voting_config(),
            &power_state,
            PROPOSAL_KEY,
            SNAPSHOT_KEY,
            100,
            3,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::GovernanceProposalActionMismatch.into());
    }

    #[test]
    fn create_governance_snapshot_sets_proposal_voting() {
        let (mut proposal, action) =
            strict_proposal_and_action(GovernanceActionTypeV1::ContributorAdd);
        let mut snapshot = blank_snapshot();
        let power_state = power_state_with_total(100);

        record_governance_snapshot_create(
            &mut proposal,
            &action,
            &mut snapshot,
            &default_voting_config(),
            &power_state,
            PROPOSAL_KEY,
            SNAPSHOT_KEY,
            100,
            3,
        )
        .unwrap();

        assert_eq!(proposal.status, GovernanceProposalStatusV1::Voting);
        assert_eq!(proposal.snapshot, SNAPSHOT_KEY);
        assert_eq!(proposal.voting_start_ts, 100);
        assert_eq!(
            proposal.voting_end_ts,
            100 + GOVERNANCE_DEFAULT_VOTING_PERIOD_SECONDS
        );
        assert_eq!(snapshot.proposal, PROPOSAL_KEY);
        assert_eq!(snapshot.total_voting_power, 100);
        assert!(!snapshot.finalized);
    }

    #[test]
    fn create_governance_snapshot_copies_power_state_total_only() {
        let (mut proposal, action) =
            strict_proposal_and_action(GovernanceActionTypeV1::ContributorAdd);
        let mut snapshot = blank_snapshot();
        snapshot.total_voting_power = 999_999;
        let power_state = power_state_with_total(321);

        record_governance_snapshot_create(
            &mut proposal,
            &action,
            &mut snapshot,
            &default_voting_config(),
            &power_state,
            PROPOSAL_KEY,
            SNAPSHOT_KEY,
            100,
            3,
        )
        .unwrap();

        assert_eq!(snapshot.total_voting_power, 321);
    }

    #[test]
    fn create_governance_snapshot_rejects_zero_total_voting_power() {
        let (mut proposal, action) =
            strict_proposal_and_action(GovernanceActionTypeV1::ContributorAdd);
        let mut snapshot = blank_snapshot();
        let power_state = power_state_with_total(0);

        let err = record_governance_snapshot_create(
            &mut proposal,
            &action,
            &mut snapshot,
            &default_voting_config(),
            &power_state,
            PROPOSAL_KEY,
            SNAPSHOT_KEY,
            100,
            3,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::InvalidGovernanceSnapshot.into());
    }

    #[test]
    fn initialize_governance_position_defaults_to_active_zero_lock() {
        let mut position = default_position();
        record_governance_position_init(&mut position, OWNER, ALPHA_MINT, GOVERNANCE_VAULT, 11, 2)
            .unwrap();

        assert_eq!(position.owner, OWNER);
        assert_eq!(position.alpha_mint, ALPHA_MINT);
        assert_eq!(position.vault, GOVERNANCE_VAULT);
        assert_eq!(position.locked_amount, 0);
        assert_eq!(position.lock_start_time, 0);
        assert_eq!(position.lock_end_time, 0);
        assert_eq!(position.holding_multiplier_bps, 0);
        assert_eq!(position.voting_power, 0);
        assert_eq!(position.status, GovernancePositionStatusV1::Active);
        assert_eq!(position.last_updated_at, 11);
        assert_eq!(position.bump, 2);
    }

    #[test]
    fn initialize_governance_position_rejects_default_owner() {
        let mut position = default_position();
        let err = record_governance_position_init(
            &mut position,
            Pubkey::default(),
            ALPHA_MINT,
            GOVERNANCE_VAULT,
            11,
            2,
        )
        .unwrap_err();
        assert_eq!(err, CustomError::InvalidGovernancePosition.into());
    }

    #[test]
    fn lock_alpha_for_governance_records_10000_alpha() {
        let config = default_config();
        let mut position = active_position();
        record_governance_lock(&mut position, &config, 10_000, LOCK_365_DAYS_SECONDS, 100).unwrap();

        assert_eq!(position.locked_amount, 10_000);
        assert_eq!(position.lock_start_time, 100);
        assert_eq!(position.lock_end_time, 100 + LOCK_365_DAYS_SECONDS);
        assert_eq!(
            position.holding_multiplier_bps,
            GOVERNANCE_365_DAY_MULTIPLIER_BPS
        );
        assert_eq!(position.voting_power, 20_000);
        assert_eq!(position.last_updated_at, 100);
    }

    #[test]
    fn governance_power_state_updates_after_lock() {
        let config = default_config();
        let mut position = active_position();
        let mut power_state = power_state_with_total(0);
        power_state.governance_lock_config = GOVERNANCE_CONFIG_KEY;
        power_state.total_locked_alpha = 0;
        power_state.active_position_count = 0;

        let previous_locked_amount = position.locked_amount;
        let previous_voting_power = position.voting_power;
        record_governance_lock(&mut position, &config, 10_000, LOCK_365_DAYS_SECONDS, 100).unwrap();
        record_governance_power_after_lock(
            &mut power_state,
            GOVERNANCE_CONFIG_KEY,
            previous_locked_amount,
            previous_voting_power,
            position.locked_amount,
            position.voting_power,
            100,
        )
        .unwrap();

        assert_eq!(power_state.total_locked_alpha, 10_000);
        assert_eq!(power_state.total_voting_power, 20_000);
        assert_eq!(power_state.active_position_count, 1);
        assert_eq!(power_state.updated_at, 100);
    }

    #[test]
    fn governance_lock_duration_multipliers_match_design() {
        assert_eq!(
            governance_time_multiplier_bps(LOCK_30_DAYS_SECONDS).unwrap(),
            GOVERNANCE_30_DAY_MULTIPLIER_BPS
        );
        assert_eq!(
            governance_time_multiplier_bps(LOCK_90_DAYS_SECONDS).unwrap(),
            GOVERNANCE_90_DAY_MULTIPLIER_BPS
        );
        assert_eq!(
            governance_time_multiplier_bps(LOCK_180_DAYS_SECONDS).unwrap(),
            GOVERNANCE_180_DAY_MULTIPLIER_BPS
        );
        assert_eq!(
            governance_time_multiplier_bps(LOCK_365_DAYS_SECONDS).unwrap(),
            GOVERNANCE_365_DAY_MULTIPLIER_BPS
        );
    }

    #[test]
    fn calculate_voting_power_uses_linear_locked_amount_and_bps() {
        assert_eq!(
            calculate_governance_voting_power(10_000, LOCK_30_DAYS_SECONDS).unwrap(),
            10_000
        );
        assert_eq!(
            calculate_governance_voting_power(10_000, LOCK_90_DAYS_SECONDS).unwrap(),
            11_000
        );
        assert_eq!(
            calculate_governance_voting_power(10_000, LOCK_180_DAYS_SECONDS).unwrap(),
            15_000
        );
        assert_eq!(
            calculate_governance_voting_power(10_000, LOCK_365_DAYS_SECONDS).unwrap(),
            20_000
        );
    }

    #[test]
    fn split_wallets_do_not_increase_linear_voting_power() {
        let single_position_power =
            calculate_governance_voting_power(100, LOCK_365_DAYS_SECONDS).unwrap();
        let split_positions_power = (0..100)
            .map(|_| calculate_governance_voting_power(1, LOCK_365_DAYS_SECONDS).unwrap())
            .try_fold(0u64, |acc, value| acc.checked_add(value))
            .unwrap();

        assert_eq!(single_position_power, 200);
        assert_eq!(split_positions_power, 200);
        assert_eq!(single_position_power, split_positions_power);
    }

    #[test]
    fn calculate_voting_power_rejects_overflow_after_multiplier() {
        let err = calculate_governance_voting_power(u64::MAX, LOCK_365_DAYS_SECONDS).unwrap_err();
        assert_eq!(err, CustomError::MathOverflow.into());
    }

    #[test]
    fn lock_alpha_rejects_below_min_lock_amount() {
        let config = default_config();
        let mut position = active_position();
        let err = record_governance_lock(&mut position, &config, 0, LOCK_30_DAYS_SECONDS, 100)
            .unwrap_err();
        assert_eq!(err, CustomError::InvalidGovernanceLockAmount.into());
    }

    #[test]
    fn lock_alpha_rejects_duration_below_minimum() {
        let config = default_config();
        let mut position = active_position();
        let err = record_governance_lock(&mut position, &config, 10_000, 1, 100).unwrap_err();
        assert_eq!(err, CustomError::InvalidGovernanceLockDuration.into());
    }

    #[test]
    fn top_up_updates_power_state_by_difference_without_incrementing_position_count() {
        let config = default_config();
        let mut position = active_position();
        let mut power_state = power_state_with_total(0);
        power_state.total_locked_alpha = 0;
        power_state.active_position_count = 0;

        let previous_locked_amount = position.locked_amount;
        let previous_voting_power = position.voting_power;
        record_governance_lock(&mut position, &config, 1_000, LOCK_30_DAYS_SECONDS, 100).unwrap();
        record_governance_power_after_lock(
            &mut power_state,
            GOVERNANCE_CONFIG_KEY,
            previous_locked_amount,
            previous_voting_power,
            position.locked_amount,
            position.voting_power,
            100,
        )
        .unwrap();

        let previous_locked_amount = position.locked_amount;
        let previous_voting_power = position.voting_power;
        record_governance_lock(&mut position, &config, 500, LOCK_30_DAYS_SECONDS, 110).unwrap();
        record_governance_power_after_lock(
            &mut power_state,
            GOVERNANCE_CONFIG_KEY,
            previous_locked_amount,
            previous_voting_power,
            position.locked_amount,
            position.voting_power,
            110,
        )
        .unwrap();

        assert_eq!(position.locked_amount, 1_500);
        assert_eq!(position.voting_power, 1_500);
        assert_eq!(power_state.total_locked_alpha, 1_500);
        assert_eq!(power_state.total_voting_power, 1_500);
        assert_eq!(power_state.active_position_count, 1);
    }

    #[test]
    fn top_up_with_longer_duration_updates_power_by_new_minus_old() {
        let config = default_config();
        let mut position = active_position();
        let mut power_state = power_state_with_total(0);
        power_state.total_locked_alpha = 0;
        power_state.active_position_count = 0;

        record_governance_lock(&mut position, &config, 1_000, LOCK_30_DAYS_SECONDS, 100).unwrap();
        record_governance_power_after_lock(
            &mut power_state,
            GOVERNANCE_CONFIG_KEY,
            0,
            0,
            position.locked_amount,
            position.voting_power,
            100,
        )
        .unwrap();
        assert_eq!(position.voting_power, 1_000);

        let previous_locked_amount = position.locked_amount;
        let previous_voting_power = position.voting_power;
        record_governance_lock(&mut position, &config, 1, LOCK_365_DAYS_SECONDS, 110).unwrap();
        record_governance_power_after_lock(
            &mut power_state,
            GOVERNANCE_CONFIG_KEY,
            previous_locked_amount,
            previous_voting_power,
            position.locked_amount,
            position.voting_power,
            110,
        )
        .unwrap();

        assert_eq!(position.locked_amount, 1_001);
        assert_eq!(
            position.holding_multiplier_bps,
            GOVERNANCE_365_DAY_MULTIPLIER_BPS
        );
        assert_eq!(position.voting_power, 2_002);
        assert_eq!(power_state.total_locked_alpha, 1_001);
        assert_eq!(power_state.total_voting_power, 2_002);
        assert_eq!(power_state.active_position_count, 1);
    }

    #[test]
    fn power_state_lock_update_rejects_missing_previous_power() {
        let mut power_state = power_state_with_total(5);
        let err = record_governance_power_after_lock(
            &mut power_state,
            GOVERNANCE_CONFIG_KEY,
            10,
            10,
            20,
            20,
            100,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::MathOverflow.into());
    }

    #[test]
    fn early_unlock_fails() {
        let config = default_config();
        let mut position = active_position();
        record_governance_lock(&mut position, &config, 10_000, LOCK_30_DAYS_SECONDS, 100).unwrap();

        let err =
            validate_governance_unlock(&position, 10_000, 99 + LOCK_30_DAYS_SECONDS).unwrap_err();
        assert_eq!(err, CustomError::GovernanceLockStillActive.into());
    }

    #[test]
    fn unlock_after_lock_end_succeeds() {
        let config = default_config();
        let mut position = active_position();
        record_governance_lock(&mut position, &config, 10_000, LOCK_30_DAYS_SECONDS, 100).unwrap();

        let amount =
            validate_governance_unlock(&position, 10_000, 100 + LOCK_30_DAYS_SECONDS).unwrap();
        assert_eq!(amount, 10_000);
        record_governance_unlock(&mut position, 100 + LOCK_30_DAYS_SECONDS).unwrap();

        assert_eq!(position.locked_amount, 0);
        assert_eq!(position.holding_multiplier_bps, 0);
        assert_eq!(position.voting_power, 0);
        assert_eq!(position.status, GovernancePositionStatusV1::Closed);
        assert_eq!(position.last_updated_at, 100 + LOCK_30_DAYS_SECONDS);
    }

    #[test]
    fn duplicate_unlock_fails_after_position_is_closed() {
        let config = default_config();
        let mut position = active_position();
        record_governance_lock(&mut position, &config, 10_000, LOCK_30_DAYS_SECONDS, 100).unwrap();
        record_governance_unlock(&mut position, 100 + LOCK_30_DAYS_SECONDS).unwrap();

        let err =
            validate_governance_unlock(&position, 10_000, 100 + LOCK_30_DAYS_SECONDS).unwrap_err();
        assert_eq!(err, CustomError::InvalidGovernancePosition.into());
    }

    #[test]
    fn governance_power_state_updates_after_unlock() {
        let mut power_state = power_state_with_total(20_000);
        power_state.total_locked_alpha = 10_000;
        power_state.active_position_count = 1;

        record_governance_power_after_unlock(
            &mut power_state,
            GOVERNANCE_CONFIG_KEY,
            10_000,
            20_000,
            100 + LOCK_365_DAYS_SECONDS,
        )
        .unwrap();

        assert_eq!(power_state.total_locked_alpha, 0);
        assert_eq!(power_state.total_voting_power, 0);
        assert_eq!(power_state.active_position_count, 0);
        assert_eq!(power_state.updated_at, 100 + LOCK_365_DAYS_SECONDS);
    }

    #[test]
    fn power_state_unlock_uses_stored_voting_power() {
        let mut power_state = power_state_with_total(1_234);
        power_state.total_locked_alpha = 1_000;
        power_state.active_position_count = 1;

        record_governance_power_after_unlock(
            &mut power_state,
            GOVERNANCE_CONFIG_KEY,
            1_000,
            1_234,
            200,
        )
        .unwrap();

        assert_eq!(power_state.total_locked_alpha, 0);
        assert_eq!(power_state.total_voting_power, 0);
        assert_eq!(power_state.active_position_count, 0);
    }

    #[test]
    fn power_state_unlock_rejects_underflow() {
        let mut power_state = power_state_with_total(99);
        power_state.total_locked_alpha = 999;
        power_state.active_position_count = 1;

        let err = record_governance_power_after_unlock(
            &mut power_state,
            GOVERNANCE_CONFIG_KEY,
            1_000,
            99,
            200,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::MathOverflow.into());
    }

    #[test]
    fn vote_lock_blocks_unlock_until_proposal_voting_ends() {
        let config = default_config();
        let mut position = active_position();
        record_governance_lock(&mut position, &config, 10_000, LOCK_30_DAYS_SECONDS, 100).unwrap();
        let mut vote_lock = vote_lock_for_position(POSITION_KEY);
        vote_lock.voting_lock_until = 100 + LOCK_30_DAYS_SECONDS + 10;

        let err = validate_governance_unlock_with_vote_lock(
            &position,
            POSITION_KEY,
            &vote_lock,
            10_000,
            100 + LOCK_30_DAYS_SECONDS,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::GovernanceVoteLockActive.into());
    }

    #[test]
    fn vote_lock_allows_unlock_after_proposal_voting_ends() {
        let config = default_config();
        let mut position = active_position();
        record_governance_lock(&mut position, &config, 10_000, LOCK_30_DAYS_SECONDS, 100).unwrap();
        let mut vote_lock = vote_lock_for_position(POSITION_KEY);
        vote_lock.voting_lock_until = 100 + LOCK_30_DAYS_SECONDS;

        let amount = validate_governance_unlock_with_vote_lock(
            &position,
            POSITION_KEY,
            &vote_lock,
            10_000,
            100 + LOCK_30_DAYS_SECONDS,
        )
        .unwrap();

        assert_eq!(amount, 10_000);
    }

    #[test]
    fn unlock_rejects_insufficient_vault_balance() {
        let config = default_config();
        let mut position = active_position();
        record_governance_lock(&mut position, &config, 10_000, LOCK_30_DAYS_SECONDS, 100).unwrap();

        let err =
            validate_governance_unlock(&position, 9_999, 100 + LOCK_30_DAYS_SECONDS).unwrap_err();
        assert_eq!(err, CustomError::InsufficientVaultBalance.into());
    }

    #[test]
    fn cast_governance_vote_records_choice_and_weight() {
        let (mut proposal, mut snapshot) = voting_proposal_and_snapshot(100);
        let position = active_position_with_power(70, 90);
        let mut vote_lock = vote_lock_for_position(POSITION_KEY);
        let mut vote_record = default_vote_record();

        record_cast_governance_vote(
            &mut proposal,
            &mut snapshot,
            &position,
            &mut vote_lock,
            &mut vote_record,
            PROPOSAL_KEY,
            POSITION_KEY,
            VoteChoiceV1::Yes,
            110,
            4,
        )
        .unwrap();

        assert_eq!(snapshot.yes_weight, 70);
        assert_eq!(snapshot.no_weight, 0);
        assert_eq!(snapshot.abstain_weight, 0);
        assert_eq!(vote_record.proposal, PROPOSAL_KEY);
        assert_eq!(vote_record.voter_position, POSITION_KEY);
        assert_eq!(vote_record.choice, VoteChoiceV1::Yes);
        assert_eq!(vote_record.voting_power_used, 70);
        assert_eq!(vote_record.timestamp, 110);
        assert_eq!(vote_record.bump, 4);
        assert_eq!(vote_lock.voting_lock_until, proposal.voting_end_ts);
        assert_eq!(vote_lock.last_proposal, PROPOSAL_KEY);
        assert_eq!(vote_lock.updated_at, 110);
    }

    #[test]
    fn snapshot_after_top_up_cannot_use_new_power_for_old_snapshot() {
        let config = default_config();
        let (mut proposal, mut snapshot) = voting_proposal_and_snapshot(100);
        let mut position = active_position_with_power(70, 90);
        let mut vote_lock = vote_lock_for_position(POSITION_KEY);
        let mut vote_record = default_vote_record();

        record_governance_lock(&mut position, &config, 30, LOCK_30_DAYS_SECONDS, 110).unwrap();
        assert_eq!(position.voting_power, 100);
        assert_eq!(position.last_updated_at, 110);

        let err = record_cast_governance_vote(
            &mut proposal,
            &mut snapshot,
            &position,
            &mut vote_lock,
            &mut vote_record,
            PROPOSAL_KEY,
            POSITION_KEY,
            VoteChoiceV1::Yes,
            120,
            4,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::InvalidGovernancePosition.into());
        assert_eq!(snapshot.yes_weight, 0);
        assert_eq!(vote_lock.voting_lock_until, 0);
    }

    #[test]
    fn vote_lock_uses_max_and_does_not_shorten() {
        let mut vote_lock = vote_lock_for_position(POSITION_KEY);
        record_governance_position_vote_lock_after_vote(
            &mut vote_lock,
            POSITION_KEY,
            PROPOSAL_KEY,
            500,
            100,
        )
        .unwrap();
        record_governance_position_vote_lock_after_vote(
            &mut vote_lock,
            POSITION_KEY,
            Pubkey::new_from_array([13; 32]),
            400,
            110,
        )
        .unwrap();

        assert_eq!(vote_lock.voting_lock_until, 500);
        assert_eq!(vote_lock.updated_at, 110);
    }

    #[test]
    fn overlapping_proposals_lock_until_latest_voting_end() {
        let mut vote_lock = vote_lock_for_position(POSITION_KEY);
        record_governance_position_vote_lock_after_vote(
            &mut vote_lock,
            POSITION_KEY,
            PROPOSAL_KEY,
            400,
            100,
        )
        .unwrap();
        record_governance_position_vote_lock_after_vote(
            &mut vote_lock,
            POSITION_KEY,
            Pubkey::new_from_array([14; 32]),
            700,
            120,
        )
        .unwrap();

        assert_eq!(vote_lock.voting_lock_until, 700);
    }

    #[test]
    fn duplicate_governance_vote_fails() {
        let (mut proposal, mut snapshot) = voting_proposal_and_snapshot(100);
        let position = active_position_with_power(70, 90);
        let mut vote_lock = vote_lock_for_position(POSITION_KEY);
        let mut vote_record = default_vote_record();

        record_cast_governance_vote(
            &mut proposal,
            &mut snapshot,
            &position,
            &mut vote_lock,
            &mut vote_record,
            PROPOSAL_KEY,
            POSITION_KEY,
            VoteChoiceV1::Yes,
            110,
            4,
        )
        .unwrap();
        let err = record_cast_governance_vote(
            &mut proposal,
            &mut snapshot,
            &position,
            &mut vote_lock,
            &mut vote_record,
            PROPOSAL_KEY,
            POSITION_KEY,
            VoteChoiceV1::No,
            111,
            4,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::AlreadyVoted.into());
        assert_eq!(snapshot.yes_weight, 70);
        assert_eq!(snapshot.no_weight, 0);
    }

    #[test]
    fn finalize_governance_vote_passes_with_sixty_percent_threshold() {
        let (mut proposal, mut snapshot) = voting_proposal_and_snapshot(100);
        let yes_position = active_position_with_power(70, 90);
        let no_position = active_position_with_power(30, 90);
        let mut yes_vote_lock = vote_lock_for_position(POSITION_KEY);
        let mut no_vote_lock = vote_lock_for_position(POSITION_TWO_KEY);
        let mut yes_record = default_vote_record();
        let mut no_record = default_vote_record();

        record_cast_governance_vote(
            &mut proposal,
            &mut snapshot,
            &yes_position,
            &mut yes_vote_lock,
            &mut yes_record,
            PROPOSAL_KEY,
            POSITION_KEY,
            VoteChoiceV1::Yes,
            110,
            4,
        )
        .unwrap();
        record_cast_governance_vote(
            &mut proposal,
            &mut snapshot,
            &no_position,
            &mut no_vote_lock,
            &mut no_record,
            PROPOSAL_KEY,
            POSITION_TWO_KEY,
            VoteChoiceV1::No,
            120,
            5,
        )
        .unwrap();

        let voting_end_ts = proposal.voting_end_ts;
        record_finalize_governance_vote(
            &mut proposal,
            &mut snapshot,
            &default_voting_config(),
            voting_end_ts,
        )
        .unwrap();

        assert_eq!(proposal.status, GovernanceProposalStatusV1::Passed);
        assert_eq!(proposal.yes_weight, 70);
        assert_eq!(proposal.no_weight, 30);
        assert_eq!(proposal.abstain_weight, 0);
        assert_eq!(proposal.finalized_at, voting_end_ts);
        assert!(snapshot.finalized);
    }

    #[test]
    fn finalize_governance_vote_fails_when_quorum_not_reached() {
        let (mut proposal, mut snapshot) = voting_proposal_and_snapshot(10_000);
        snapshot.yes_weight = 100;

        let voting_end_ts = proposal.voting_end_ts;
        let err = record_finalize_governance_vote(
            &mut proposal,
            &mut snapshot,
            &default_voting_config(),
            voting_end_ts,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::QuorumNotReached.into());
        assert_eq!(proposal.status, GovernanceProposalStatusV1::Voting);
        assert!(!snapshot.finalized);
    }

    #[test]
    fn finalize_governance_vote_fails_before_voting_end() {
        let (mut proposal, mut snapshot) = voting_proposal_and_snapshot(100);
        snapshot.yes_weight = 70;
        snapshot.no_weight = 30;

        let before_voting_end_ts = proposal.voting_end_ts - 1;
        let err = record_finalize_governance_vote(
            &mut proposal,
            &mut snapshot,
            &default_voting_config(),
            before_voting_end_ts,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::VotingPeriodNotEnded.into());
        assert_eq!(proposal.status, GovernanceProposalStatusV1::Voting);
        assert!(!snapshot.finalized);
    }

    #[test]
    fn governance_threshold_policy_matches_proposal_types() {
        assert_eq!(
            governance_threshold_policy_for_proposal_type(GovernanceProposalTypeV1::Contributor),
            GovernanceThresholdPolicyV1 {
                quorum_bps: 500,
                approval_threshold_bps: 6_000,
            }
        );
        assert_eq!(
            governance_threshold_policy_for_proposal_type(GovernanceProposalTypeV1::Treasury),
            GovernanceThresholdPolicyV1 {
                quorum_bps: 1_000,
                approval_threshold_bps: 6_667,
            }
        );
        assert_eq!(
            governance_threshold_policy_for_proposal_type(GovernanceProposalTypeV1::Parameter),
            GovernanceThresholdPolicyV1 {
                quorum_bps: 2_000,
                approval_threshold_bps: 7_500,
            }
        );
        assert_eq!(
            governance_threshold_policy_for_proposal_type(GovernanceProposalTypeV1::Upgrade),
            GovernanceThresholdPolicyV1 {
                quorum_bps: 2_500,
                approval_threshold_bps: 8_000,
            }
        );
        assert_eq!(
            governance_threshold_policy_for_proposal_type(GovernanceProposalTypeV1::Emergency),
            GovernanceThresholdPolicyV1 {
                quorum_bps: 1_500,
                approval_threshold_bps: 7_500,
            }
        );
        for proposal_type in [
            GovernanceProposalTypeV1::GreenLabel,
            GovernanceProposalTypeV1::VictimRelief,
            GovernanceProposalTypeV1::ScamRegistry,
        ] {
            assert_eq!(
                governance_threshold_policy_for_proposal_type(proposal_type),
                GovernanceThresholdPolicyV1 {
                    quorum_bps: 1_000,
                    approval_threshold_bps: 6_667,
                }
            );
        }
    }

    #[test]
    fn threshold_quorum_boundary_is_inclusive() {
        assert!(has_governance_quorum(50, 1_000, 500).unwrap());
        assert!(!has_governance_quorum(49, 1_000, 500).unwrap());
    }

    #[test]
    fn threshold_approval_boundary_is_inclusive() {
        assert!(has_governance_approval(60, 40, 6_000).unwrap());
        assert!(!has_governance_approval(59, 41, 6_000).unwrap());
        assert!(has_governance_approval(6_667, 3_333, 6_667).unwrap());
        assert!(!has_governance_approval(6_666, 3_334, 6_667).unwrap());
    }

    #[test]
    fn threshold_abstain_counts_for_quorum_not_approval() {
        let mut snapshot = blank_snapshot();
        snapshot.proposal = PROPOSAL_KEY;
        snapshot.total_voting_power = 100;
        snapshot.abstain_weight = 5;

        let err = validate_governance_thresholds(&snapshot, GovernanceProposalTypeV1::Contributor)
            .unwrap_err();

        assert_eq!(err, CustomError::QuorumNotReached.into());
    }

    #[test]
    fn threshold_rejects_one_vote_below_quorum() {
        let mut snapshot = blank_snapshot();
        snapshot.proposal = PROPOSAL_KEY;
        snapshot.total_voting_power = 1_000;
        snapshot.yes_weight = 49;

        let err = validate_governance_thresholds(&snapshot, GovernanceProposalTypeV1::Contributor)
            .unwrap_err();

        assert_eq!(err, CustomError::QuorumNotReached.into());
    }

    #[test]
    fn threshold_allows_exact_quorum_and_exact_approval() {
        let mut snapshot = blank_snapshot();
        snapshot.proposal = PROPOSAL_KEY;
        snapshot.total_voting_power = 1_000;
        snapshot.yes_weight = 30;
        snapshot.no_weight = 20;

        assert!(
            validate_governance_thresholds(&snapshot, GovernanceProposalTypeV1::Contributor)
                .unwrap()
        );
    }

    #[test]
    fn treasury_finalize_uses_higher_proposal_type_threshold() {
        let (mut proposal, mut snapshot) = voting_proposal_and_snapshot(1_000);
        proposal.proposal_type = GovernanceProposalTypeV1::Treasury;
        snapshot.yes_weight = 66;
        snapshot.no_weight = 34;

        let voting_end_ts = proposal.voting_end_ts;
        record_finalize_governance_vote(
            &mut proposal,
            &mut snapshot,
            &default_voting_config(),
            voting_end_ts,
        )
        .unwrap();

        assert_eq!(proposal.status, GovernanceProposalStatusV1::Rejected);

        let (mut proposal, mut snapshot) = voting_proposal_and_snapshot(1_000);
        proposal.proposal_type = GovernanceProposalTypeV1::Treasury;
        snapshot.yes_weight = 667;
        snapshot.no_weight = 333;
        let voting_end_ts = proposal.voting_end_ts;
        record_finalize_governance_vote(
            &mut proposal,
            &mut snapshot,
            &default_voting_config(),
            voting_end_ts,
        )
        .unwrap();

        assert_eq!(proposal.status, GovernanceProposalStatusV1::Passed);
    }

    #[test]
    fn initialize_governance_snapshot_defaults_unfinalized_zero_weights() {
        let mut snapshot = default_snapshot();
        record_governance_snapshot_init(&mut snapshot, PROPOSAL_KEY, 30, 3).unwrap();

        assert_eq!(snapshot.proposal, PROPOSAL_KEY);
        assert_eq!(snapshot.total_voting_power, 0);
        assert_eq!(snapshot.yes_weight, 0);
        assert_eq!(snapshot.no_weight, 0);
        assert_eq!(snapshot.abstain_weight, 0);
        assert_eq!(snapshot.created_at, 30);
        assert!(!snapshot.finalized);
        assert_eq!(snapshot.bump, 3);
    }

    #[test]
    fn initialize_vote_record_defaults_to_abstain_zero_power() {
        let mut vote_record = default_vote_record();
        record_vote_record_init(&mut vote_record, PROPOSAL_KEY, POSITION_KEY, 4).unwrap();

        assert_eq!(vote_record.proposal, PROPOSAL_KEY);
        assert_eq!(vote_record.voter_position, POSITION_KEY);
        assert_eq!(vote_record.choice, VoteChoiceV1::Abstain);
        assert_eq!(vote_record.voting_power_used, 0);
        assert_eq!(vote_record.timestamp, 0);
        assert_eq!(vote_record.bump, 4);
    }
}
