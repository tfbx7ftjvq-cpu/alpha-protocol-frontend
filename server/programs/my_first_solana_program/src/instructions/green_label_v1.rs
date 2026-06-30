use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};

use crate::constants::{
    BASE_BOND_REFUND_BPS, BASE_BOND_TREASURY_BPS, DEFAULT_DISPUTE_WINDOW_SECONDS,
    DEFAULT_OBSERVATION_PERIOD_SECONDS, DEFAULT_RESPONSE_WINDOW_SECONDS,
    GREEN_BOND_VAULT_AUTHORITY_SEED, GREEN_BOND_VAULT_SEED, GREEN_LABEL_BRONZE_TIER_THRESHOLD_USDC,
    GREEN_LABEL_CONFIG_RESERVED_BYTES, GREEN_LABEL_CONFIG_SEED, GREEN_LABEL_CONFIG_SPACE,
    GREEN_LABEL_DISPUTE_RESERVED_BYTES, GREEN_LABEL_DISPUTE_SEED, GREEN_LABEL_DISPUTE_SPACE,
    GREEN_LABEL_GOLD_TIER_THRESHOLD_USDC, GREEN_LABEL_PLATINUM_TIER_THRESHOLD_USDC,
    GREEN_LABEL_PROJECT_RESERVED_BYTES, GREEN_LABEL_PROJECT_SEED, GREEN_LABEL_PROJECT_SPACE,
    GREEN_LABEL_SILVER_TIER_THRESHOLD_USDC, GREEN_LABEL_USDC_DECIMALS, MAX_BPS,
    MIN_GREEN_LABEL_BASE_BOND_USDC,
};
use crate::error::CustomError;
use crate::state::{
    ActionType, BondTier, DisputeStatus, ExecutionQueueItemV1, ExecutionStatus, GreenLabelConfigV1,
    GreenLabelDisputeV1, GreenLabelProjectV1, GreenLabelStatus, ProposalDecision,
    ProposalDecisionV1, ProposalType, RugReasonCode,
};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GreenLabelProjectInitValues {
    pub project_id: u64,
    pub project_owner: Pubkey,
    pub project_name_hash: [u8; 32],
    pub project_url_hash: [u8; 32],
    pub token_mint: Pubkey,
    pub project_treasury_wallet: Pubkey,
    pub base_bond_amount: u64,
    pub extra_bond_amount: u64,
    pub total_bond_amount: u64,
    pub bond_vault: Pubkey,
    pub bond_vault_authority: Pubkey,
    pub bond_tier: BondTier,
    pub status: GreenLabelStatus,
    pub submitted_at: i64,
    pub observation_start_ts: i64,
    pub observation_end_ts: i64,
    pub dispute_count: u64,
    pub active_dispute: Pubkey,
    pub approved_at: i64,
    pub refunded_at: i64,
    pub slashed_at: i64,
    pub risk_score_snapshot: u16,
    pub terminal_proposal_id: u64,
    pub terminal_proposal_decision: Pubkey,
    pub terminal_execution_queue_item: Pubkey,
    pub terminal_payload_hash: [u8; 32],
    pub terminal_action_type: ActionType,
    pub bump: u8,
    pub reserved: [u8; GREEN_LABEL_PROJECT_RESERVED_BYTES],
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

#[derive(Accounts)]
#[instruction(expected_project_id: u64)]
pub struct SubmitGreenLabelApplication<'info> {
    #[account(
        mut,
        seeds = [GREEN_LABEL_CONFIG_SEED],
        bump = green_label_config.bump
    )]
    pub green_label_config: Account<'info, GreenLabelConfigV1>,

    #[account(
        init,
        payer = project_owner,
        space = GREEN_LABEL_PROJECT_SPACE,
        seeds = [
            GREEN_LABEL_PROJECT_SEED,
            &expected_project_id.to_le_bytes()
        ],
        bump
    )]
    pub green_label_project: Account<'info, GreenLabelProjectV1>,

    #[account(mut)]
    pub project_owner: Signer<'info>,

    /// CHECK: Phase 1D-1 stores the mint key only; token validation is added in later phases.
    pub token_mint: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeGreenBondVault<'info> {
    #[account(
        seeds = [GREEN_LABEL_CONFIG_SEED],
        bump = green_label_config.bump
    )]
    pub green_label_config: Box<Account<'info, GreenLabelConfigV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_PROJECT_SEED,
            &green_label_project.project_id.to_le_bytes()
        ],
        bump = green_label_project.bump
    )]
    pub green_label_project: Box<Account<'info, GreenLabelProjectV1>>,

    #[account(
        init,
        payer = project_owner,
        seeds = [
            GREEN_BOND_VAULT_SEED,
            green_label_project.key().as_ref()
        ],
        bump,
        token::mint = usdc_mint,
        token::authority = green_bond_vault_authority
    )]
    pub green_bond_vault: Box<Account<'info, TokenAccount>>,

    /// CHECK: This PDA owns the project Green Bond Vault token account.
    #[account(
        seeds = [
            GREEN_BOND_VAULT_AUTHORITY_SEED,
            green_label_project.key().as_ref()
        ],
        bump
    )]
    pub green_bond_vault_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub project_owner: Signer<'info>,

    #[account(
        constraint = usdc_mint.key() == green_label_config.usdc_mint @ CustomError::InvalidGreenLabelMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct LockGreenLabelBond<'info> {
    #[account(
        seeds = [GREEN_LABEL_CONFIG_SEED],
        bump = green_label_config.bump
    )]
    pub green_label_config: Box<Account<'info, GreenLabelConfigV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_PROJECT_SEED,
            &green_label_project.project_id.to_le_bytes()
        ],
        bump = green_label_project.bump
    )]
    pub green_label_project: Box<Account<'info, GreenLabelProjectV1>>,

    pub project_owner: Signer<'info>,

    #[account(mut)]
    pub project_owner_usdc_ata: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub green_bond_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = usdc_mint.key() == green_label_config.usdc_mint @ CustomError::InvalidGreenLabelMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(expected_dispute_id: u64)]
pub struct OpenGreenLabelDispute<'info> {
    #[account(
        seeds = [GREEN_LABEL_CONFIG_SEED],
        bump = green_label_config.bump
    )]
    pub green_label_config: Box<Account<'info, GreenLabelConfigV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_PROJECT_SEED,
            &green_label_project.project_id.to_le_bytes()
        ],
        bump = green_label_project.bump
    )]
    pub green_label_project: Box<Account<'info, GreenLabelProjectV1>>,

    #[account(
        init,
        payer = disputer,
        space = GREEN_LABEL_DISPUTE_SPACE,
        seeds = [
            GREEN_LABEL_DISPUTE_SEED,
            green_label_project.key().as_ref(),
            &expected_dispute_id.to_le_bytes()
        ],
        bump
    )]
    pub green_label_dispute: Box<Account<'info, GreenLabelDisputeV1>>,

    #[account(mut)]
    pub disputer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MarkDisputeReadyForDecision<'info> {
    #[account(
        seeds = [GREEN_LABEL_CONFIG_SEED],
        bump = green_label_config.bump
    )]
    pub green_label_config: Box<Account<'info, GreenLabelConfigV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_PROJECT_SEED,
            &green_label_project.project_id.to_le_bytes()
        ],
        bump = green_label_project.bump
    )]
    pub green_label_project: Box<Account<'info, GreenLabelProjectV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_DISPUTE_SEED,
            green_label_project.key().as_ref(),
            &green_label_dispute.dispute_id.to_le_bytes()
        ],
        bump = green_label_dispute.bump
    )]
    pub green_label_dispute: Box<Account<'info, GreenLabelDisputeV1>>,

    pub caller: Signer<'info>,
}

#[derive(Accounts)]
pub struct LinkGreenLabelSecurityDecision<'info> {
    #[account(
        seeds = [GREEN_LABEL_CONFIG_SEED],
        bump = green_label_config.bump
    )]
    pub green_label_config: Box<Account<'info, GreenLabelConfigV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_PROJECT_SEED,
            &green_label_project.project_id.to_le_bytes()
        ],
        bump = green_label_project.bump
    )]
    pub green_label_project: Box<Account<'info, GreenLabelProjectV1>>,

    #[account(
        mut,
        seeds = [
            GREEN_LABEL_DISPUTE_SEED,
            green_label_project.key().as_ref(),
            &green_label_dispute.dispute_id.to_le_bytes()
        ],
        bump = green_label_dispute.bump
    )]
    pub green_label_dispute: Box<Account<'info, GreenLabelDisputeV1>>,

    pub proposal_decision: Box<Account<'info, ProposalDecisionV1>>,

    pub execution_queue_item: Box<Account<'info, ExecutionQueueItemV1>>,

    pub linker: Signer<'info>,
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

pub fn submit_green_label_application_handler(
    ctx: Context<SubmitGreenLabelApplication>,
    expected_project_id: u64,
    project_name_hash: [u8; 32],
    project_url_hash: [u8; 32],
    project_treasury_wallet: Pubkey,
    total_bond_amount: u64,
) -> Result<()> {
    let clock = Clock::get()?;
    let values = build_pending_bond_project_values(
        ctx.accounts.green_label_config.is_paused,
        ctx.accounts.green_label_config.project_count,
        expected_project_id,
        ctx.accounts.project_owner.key(),
        project_name_hash,
        project_url_hash,
        ctx.accounts.token_mint.key(),
        project_treasury_wallet,
        total_bond_amount,
        clock.unix_timestamp,
        ctx.bumps.green_label_project,
    )?;

    let green_label_project = &mut ctx.accounts.green_label_project;
    green_label_project.project_id = values.project_id;
    green_label_project.project_owner = values.project_owner;
    green_label_project.project_name_hash = values.project_name_hash;
    green_label_project.project_url_hash = values.project_url_hash;
    green_label_project.token_mint = values.token_mint;
    green_label_project.project_treasury_wallet = values.project_treasury_wallet;
    green_label_project.base_bond_amount = values.base_bond_amount;
    green_label_project.extra_bond_amount = values.extra_bond_amount;
    green_label_project.total_bond_amount = values.total_bond_amount;
    green_label_project.bond_vault = values.bond_vault;
    green_label_project.bond_vault_authority = values.bond_vault_authority;
    green_label_project.bond_tier = values.bond_tier;
    green_label_project.status = values.status;
    green_label_project.submitted_at = values.submitted_at;
    green_label_project.observation_start_ts = values.observation_start_ts;
    green_label_project.observation_end_ts = values.observation_end_ts;
    green_label_project.dispute_count = values.dispute_count;
    green_label_project.active_dispute = values.active_dispute;
    green_label_project.approved_at = values.approved_at;
    green_label_project.refunded_at = values.refunded_at;
    green_label_project.slashed_at = values.slashed_at;
    green_label_project.risk_score_snapshot = values.risk_score_snapshot;
    green_label_project.terminal_proposal_id = values.terminal_proposal_id;
    green_label_project.terminal_proposal_decision = values.terminal_proposal_decision;
    green_label_project.terminal_execution_queue_item = values.terminal_execution_queue_item;
    green_label_project.terminal_payload_hash = values.terminal_payload_hash;
    green_label_project.terminal_action_type = values.terminal_action_type;
    green_label_project.bump = values.bump;
    green_label_project.reserved = values.reserved;

    ctx.accounts.green_label_config.project_count = values.project_id;

    Ok(())
}

pub fn initialize_green_bond_vault_handler(ctx: Context<InitializeGreenBondVault>) -> Result<()> {
    validate_green_bond_vault_initialization(
        ctx.accounts.green_label_config.is_paused,
        ctx.accounts.green_label_project.project_owner,
        ctx.accounts.project_owner.key(),
        ctx.accounts.green_label_project.status,
        ctx.accounts.green_label_project.bond_vault,
        ctx.accounts.green_label_project.bond_vault_authority,
        ctx.accounts.green_label_config.usdc_mint,
        ctx.accounts.usdc_mint.key(),
    )?;

    record_green_bond_vault_initialization(
        &mut ctx.accounts.green_label_project,
        ctx.accounts.green_bond_vault.key(),
        ctx.accounts.green_bond_vault_authority.key(),
    );

    Ok(())
}

pub fn lock_green_label_bond_handler(ctx: Context<LockGreenLabelBond>) -> Result<()> {
    validate_green_label_bond_lock(
        ctx.accounts.green_label_config.is_paused,
        ctx.accounts.green_label_project.project_owner,
        ctx.accounts.project_owner.key(),
        ctx.accounts.green_label_project.status,
        ctx.accounts.green_label_project.bond_vault,
        ctx.accounts.green_label_project.bond_vault_authority,
        ctx.accounts.green_bond_vault.key(),
        ctx.accounts.green_bond_vault.mint,
        ctx.accounts.green_bond_vault.owner,
        ctx.accounts.green_label_config.usdc_mint,
        ctx.accounts.project_owner_usdc_ata.owner,
        ctx.accounts.project_owner_usdc_ata.mint,
        ctx.accounts.usdc_mint.key(),
        ctx.accounts.green_label_project.total_bond_amount,
    )?;
    require!(
        ctx.accounts.usdc_mint.decimals == GREEN_LABEL_USDC_DECIMALS,
        CustomError::InvalidGreenLabelMint
    );

    let total_bond_amount = ctx.accounts.green_label_project.total_bond_amount;
    let cpi_accounts = TransferChecked {
        from: ctx.accounts.project_owner_usdc_ata.to_account_info(),
        mint: ctx.accounts.usdc_mint.to_account_info(),
        to: ctx.accounts.green_bond_vault.to_account_info(),
        authority: ctx.accounts.project_owner.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.key(), cpi_accounts);
    transfer_checked(cpi_ctx, total_bond_amount, GREEN_LABEL_USDC_DECIMALS)?;

    let now = Clock::get()?.unix_timestamp;
    let (_, observation_end_ts) = build_observation_window(
        now,
        ctx.accounts.green_label_config.observation_period_seconds,
    )?;

    record_green_label_bond_locked(
        &mut ctx.accounts.green_label_project,
        now,
        observation_end_ts,
    )
}

pub fn open_green_label_dispute_handler(
    ctx: Context<OpenGreenLabelDispute>,
    expected_dispute_id: u64,
    reason_code: RugReasonCode,
    evidence_hash: [u8; 32],
) -> Result<()> {
    validate_open_green_label_dispute(
        ctx.accounts.green_label_config.is_paused,
        ctx.accounts.green_label_project.status,
        ctx.accounts.green_label_project.active_dispute,
        ctx.accounts.green_label_project.dispute_count,
        expected_dispute_id,
        evidence_hash,
    )?;

    let now = Clock::get()?.unix_timestamp;
    let (evidence_end_ts, response_end_ts) = build_dispute_windows(
        now,
        ctx.accounts.green_label_config.dispute_window_seconds,
        ctx.accounts.green_label_config.response_window_seconds,
    )?;

    let dispute_key = ctx.accounts.green_label_dispute.key();
    let green_label_dispute = &mut ctx.accounts.green_label_dispute;
    green_label_dispute.project_id = ctx.accounts.green_label_project.project_id;
    green_label_dispute.dispute_id = expected_dispute_id;
    green_label_dispute.project = ctx.accounts.green_label_project.key();
    green_label_dispute.disputer = ctx.accounts.disputer.key();
    green_label_dispute.reason_code = reason_code;
    green_label_dispute.evidence_hash = evidence_hash;
    green_label_dispute.status = DisputeStatus::EvidencePeriod;
    green_label_dispute.opened_at = now;
    green_label_dispute.evidence_end_ts = evidence_end_ts;
    green_label_dispute.response_end_ts = response_end_ts;
    green_label_dispute.resolved_at = 0;
    green_label_dispute.proposal_id = 0;
    green_label_dispute.proposal_decision = Pubkey::default();
    green_label_dispute.execution_queue_item = Pubkey::default();
    green_label_dispute.payload_hash = [0; 32];
    green_label_dispute.action_type = ActionType::Noop;
    green_label_dispute.bump = ctx.bumps.green_label_dispute;
    green_label_dispute.reserved = [0; GREEN_LABEL_DISPUTE_RESERVED_BYTES];

    record_green_label_dispute_opened(
        &mut ctx.accounts.green_label_project,
        dispute_key,
        expected_dispute_id,
    )
}

pub fn mark_dispute_ready_for_decision_handler(
    ctx: Context<MarkDisputeReadyForDecision>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    validate_mark_dispute_ready(
        ctx.accounts.green_label_config.is_paused,
        ctx.accounts.green_label_project.status,
        ctx.accounts.green_label_project.active_dispute,
        ctx.accounts.green_label_dispute.key(),
        ctx.accounts.green_label_dispute.project,
        ctx.accounts.green_label_project.key(),
        ctx.accounts.green_label_dispute.status,
        now,
        ctx.accounts.green_label_dispute.response_end_ts,
    )?;

    record_dispute_ready_for_decision(&mut ctx.accounts.green_label_dispute)
}

pub fn link_green_label_security_decision_handler(
    ctx: Context<LinkGreenLabelSecurityDecision>,
    expected_proposal_id: u64,
    expected_action_type: ActionType,
    expected_payload_hash: [u8; 32],
) -> Result<()> {
    let dispute_key = ctx.accounts.green_label_dispute.key();
    validate_green_label_security_decision_link(
        ctx.accounts.green_label_config.is_paused,
        ctx.accounts.green_label_project.status,
        ctx.accounts.green_label_project.active_dispute,
        dispute_key,
        ctx.accounts.green_label_dispute.project,
        ctx.accounts.green_label_project.key(),
        ctx.accounts.green_label_dispute.status,
        expected_proposal_id,
        expected_action_type,
        expected_payload_hash,
        ctx.accounts.proposal_decision.proposal_id,
        ctx.accounts.proposal_decision.proposal_type,
        ctx.accounts.proposal_decision.decision,
        ctx.accounts.execution_queue_item.proposal_id,
        ctx.accounts.execution_queue_item.action_type,
        ctx.accounts.execution_queue_item.status,
        ctx.accounts.execution_queue_item.payload_hash,
        ctx.accounts.execution_queue_item.target_program,
        crate::ID,
        ctx.accounts.execution_queue_item.target_account,
        dispute_key,
    )?;

    record_green_label_security_decision_link(
        &mut ctx.accounts.green_label_project,
        &mut ctx.accounts.green_label_dispute,
        expected_proposal_id,
        ctx.accounts.proposal_decision.key(),
        ctx.accounts.execution_queue_item.key(),
        expected_payload_hash,
        expected_action_type,
    )
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

pub fn validate_green_bond_vault_initialization(
    config_is_paused: bool,
    project_owner: Pubkey,
    signer: Pubkey,
    project_status: GreenLabelStatus,
    existing_bond_vault: Pubkey,
    existing_bond_vault_authority: Pubkey,
    expected_usdc_mint: Pubkey,
    provided_usdc_mint: Pubkey,
) -> Result<()> {
    require!(!config_is_paused, CustomError::InvalidGreenLabelStatus);
    require_keys_eq!(
        project_owner,
        signer,
        CustomError::InvalidGreenLabelProjectOwner
    );
    require!(
        project_status == GreenLabelStatus::PendingBondDeposit,
        CustomError::InvalidGreenLabelStatus
    );
    require!(
        existing_bond_vault == Pubkey::default(),
        CustomError::InvalidGreenLabelBondVaultState
    );
    require!(
        existing_bond_vault_authority == Pubkey::default(),
        CustomError::InvalidGreenLabelBondVaultState
    );
    require_keys_eq!(
        expected_usdc_mint,
        provided_usdc_mint,
        CustomError::InvalidGreenLabelMint
    );

    Ok(())
}

pub fn validate_green_label_bond_lock(
    config_is_paused: bool,
    project_owner: Pubkey,
    signer: Pubkey,
    project_status: GreenLabelStatus,
    bond_vault: Pubkey,
    bond_vault_authority: Pubkey,
    provided_bond_vault: Pubkey,
    provided_bond_vault_mint: Pubkey,
    provided_bond_vault_owner: Pubkey,
    expected_usdc_mint: Pubkey,
    owner_ata_owner: Pubkey,
    owner_ata_mint: Pubkey,
    usdc_mint: Pubkey,
    total_bond_amount: u64,
) -> Result<()> {
    require!(!config_is_paused, CustomError::InvalidGreenLabelStatus);
    require_keys_eq!(
        project_owner,
        signer,
        CustomError::InvalidGreenLabelProjectOwner
    );
    require!(
        project_status == GreenLabelStatus::PendingBondDeposit,
        CustomError::InvalidGreenLabelStatus
    );
    require!(
        bond_vault != Pubkey::default(),
        CustomError::InvalidGreenLabelBondVaultState
    );
    require!(
        bond_vault_authority != Pubkey::default(),
        CustomError::InvalidGreenLabelBondVaultState
    );
    require_keys_eq!(
        provided_bond_vault,
        bond_vault,
        CustomError::InvalidGreenLabelBondVaultState
    );
    require_keys_eq!(
        provided_bond_vault_mint,
        expected_usdc_mint,
        CustomError::InvalidGreenLabelMint
    );
    require_keys_eq!(
        provided_bond_vault_owner,
        bond_vault_authority,
        CustomError::InvalidGreenLabelBondVaultState
    );
    require_keys_eq!(
        owner_ata_owner,
        signer,
        CustomError::InvalidGreenLabelTokenAccount
    );
    require_keys_eq!(
        owner_ata_mint,
        expected_usdc_mint,
        CustomError::InvalidGreenLabelMint
    );
    require_keys_eq!(
        usdc_mint,
        expected_usdc_mint,
        CustomError::InvalidGreenLabelMint
    );
    require!(
        total_bond_amount >= MIN_GREEN_LABEL_BASE_BOND_USDC,
        CustomError::InvalidGreenLabelBondAmount
    );

    Ok(())
}

pub fn build_observation_window(now: i64, observation_period_seconds: i64) -> Result<(i64, i64)> {
    let observation_end_ts = now
        .checked_add(observation_period_seconds)
        .ok_or(CustomError::GreenLabelMathOverflow)?;

    Ok((now, observation_end_ts))
}

pub fn record_green_label_bond_locked(
    project: &mut GreenLabelProjectV1,
    now: i64,
    observation_end_ts: i64,
) -> Result<()> {
    project.status = GreenLabelStatus::PendingObservation;
    project.observation_start_ts = now;
    project.observation_end_ts = observation_end_ts;

    Ok(())
}

pub fn validate_open_green_label_dispute(
    config_is_paused: bool,
    project_status: GreenLabelStatus,
    active_dispute: Pubkey,
    current_dispute_count: u64,
    expected_dispute_id: u64,
    evidence_hash: [u8; 32],
) -> Result<()> {
    require!(!config_is_paused, CustomError::InvalidGreenLabelStatus);
    require!(
        matches!(
            project_status,
            GreenLabelStatus::PendingObservation | GreenLabelStatus::ActiveGreenLabel
        ),
        CustomError::InvalidGreenLabelStatus
    );
    require!(
        active_dispute == Pubkey::default(),
        CustomError::InvalidGreenLabelActiveDispute
    );

    let next_dispute_id = current_dispute_count
        .checked_add(1)
        .ok_or(CustomError::GreenLabelMathOverflow)?;
    require!(
        expected_dispute_id == next_dispute_id,
        CustomError::InvalidGreenLabelDisputeId
    );
    require!(
        evidence_hash != [0; 32],
        CustomError::InvalidGreenLabelEvidenceHash
    );

    Ok(())
}

pub fn build_dispute_windows(
    now: i64,
    dispute_window_seconds: i64,
    response_window_seconds: i64,
) -> Result<(i64, i64)> {
    let evidence_end_ts = now
        .checked_add(dispute_window_seconds)
        .ok_or(CustomError::GreenLabelMathOverflow)?;
    let response_end_ts = evidence_end_ts
        .checked_add(response_window_seconds)
        .ok_or(CustomError::GreenLabelMathOverflow)?;

    Ok((evidence_end_ts, response_end_ts))
}

pub fn record_green_label_dispute_opened(
    project: &mut GreenLabelProjectV1,
    dispute_key: Pubkey,
    expected_dispute_id: u64,
) -> Result<()> {
    project.status = GreenLabelStatus::Disputed;
    project.active_dispute = dispute_key;
    project.dispute_count = expected_dispute_id;

    Ok(())
}

pub fn validate_mark_dispute_ready(
    config_is_paused: bool,
    project_status: GreenLabelStatus,
    project_active_dispute: Pubkey,
    dispute_key: Pubkey,
    dispute_project: Pubkey,
    project_key: Pubkey,
    dispute_status: DisputeStatus,
    now: i64,
    response_end_ts: i64,
) -> Result<()> {
    require!(!config_is_paused, CustomError::InvalidGreenLabelStatus);
    require!(
        project_status == GreenLabelStatus::Disputed,
        CustomError::InvalidGreenLabelStatus
    );
    require_keys_eq!(
        project_active_dispute,
        dispute_key,
        CustomError::InvalidGreenLabelActiveDispute
    );
    require_keys_eq!(
        dispute_project,
        project_key,
        CustomError::InvalidGreenLabelTargetAccount
    );
    require!(
        matches!(
            dispute_status,
            DisputeStatus::EvidencePeriod | DisputeStatus::ProjectResponsePeriod
        ),
        CustomError::InvalidGreenLabelDisputeStatus
    );
    require!(
        now >= response_end_ts,
        CustomError::GreenLabelDisputeWindowNotEnded
    );

    Ok(())
}

pub fn record_dispute_ready_for_decision(dispute: &mut GreenLabelDisputeV1) -> Result<()> {
    dispute.status = DisputeStatus::ReadyForDecision;

    Ok(())
}

pub fn validate_green_label_security_decision_link(
    config_is_paused: bool,
    project_status: GreenLabelStatus,
    project_active_dispute: Pubkey,
    dispute_key: Pubkey,
    dispute_project: Pubkey,
    project_key: Pubkey,
    dispute_status: DisputeStatus,
    expected_proposal_id: u64,
    expected_action_type: ActionType,
    expected_payload_hash: [u8; 32],
    proposal_id: u64,
    proposal_type: ProposalType,
    proposal_decision: ProposalDecision,
    queue_proposal_id: u64,
    queue_action_type: ActionType,
    queue_status: ExecutionStatus,
    queue_payload_hash: [u8; 32],
    queue_target_program: Pubkey,
    expected_program_id: Pubkey,
    queue_target_account: Pubkey,
    expected_target_account: Pubkey,
) -> Result<()> {
    require!(!config_is_paused, CustomError::InvalidGreenLabelStatus);
    require!(
        project_status == GreenLabelStatus::Disputed,
        CustomError::InvalidGreenLabelStatus
    );
    require_keys_eq!(
        project_active_dispute,
        dispute_key,
        CustomError::InvalidGreenLabelActiveDispute
    );
    require_keys_eq!(
        dispute_project,
        project_key,
        CustomError::InvalidGreenLabelTargetAccount
    );
    require!(
        dispute_status == DisputeStatus::ReadyForDecision,
        CustomError::InvalidGreenLabelDisputeStatus
    );
    validate_payload_hash(expected_payload_hash)?;
    require!(
        matches!(
            expected_action_type,
            ActionType::GreenLabelSlash | ActionType::GreenLabelRefund
        ),
        CustomError::InvalidGreenLabelActionType
    );
    require!(
        proposal_id == expected_proposal_id,
        CustomError::InvalidGreenLabelSecurityDecision
    );
    require!(
        proposal_type_matches_action(proposal_type, expected_action_type),
        CustomError::InvalidGreenLabelSecurityDecision
    );
    require!(
        proposal_decision == ProposalDecision::Approved,
        CustomError::InvalidGreenLabelSecurityDecision
    );
    require!(
        queue_proposal_id == expected_proposal_id,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require!(
        queue_action_type == expected_action_type,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require!(
        queue_status == ExecutionStatus::Queued,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require!(
        queue_payload_hash == expected_payload_hash,
        CustomError::InvalidGreenLabelExecutionQueue
    );
    require_keys_eq!(
        queue_target_program,
        expected_program_id,
        CustomError::InvalidGreenLabelTargetProgram
    );
    require_keys_eq!(
        queue_target_account,
        expected_target_account,
        CustomError::InvalidGreenLabelTargetAccount
    );

    Ok(())
}

pub fn record_green_label_security_decision_link(
    project: &mut GreenLabelProjectV1,
    dispute: &mut GreenLabelDisputeV1,
    proposal_id: u64,
    proposal_decision_key: Pubkey,
    execution_queue_item_key: Pubkey,
    payload_hash: [u8; 32],
    action_type: ActionType,
) -> Result<()> {
    project.status = match action_type {
        ActionType::GreenLabelSlash => GreenLabelStatus::SlashQueued,
        ActionType::GreenLabelRefund => GreenLabelStatus::RefundQueued,
        _ => return err!(CustomError::InvalidGreenLabelActionType),
    };

    dispute.status = DisputeStatus::DecisionQueued;
    dispute.proposal_id = proposal_id;
    dispute.proposal_decision = proposal_decision_key;
    dispute.execution_queue_item = execution_queue_item_key;
    dispute.payload_hash = payload_hash;
    dispute.action_type = action_type;

    project.terminal_proposal_id = proposal_id;
    project.terminal_proposal_decision = proposal_decision_key;
    project.terminal_execution_queue_item = execution_queue_item_key;
    project.terminal_payload_hash = payload_hash;
    project.terminal_action_type = action_type;

    Ok(())
}

pub fn record_green_bond_vault_initialization(
    green_label_project: &mut GreenLabelProjectV1,
    green_bond_vault: Pubkey,
    green_bond_vault_authority: Pubkey,
) {
    green_label_project.bond_vault = green_bond_vault;
    green_label_project.bond_vault_authority = green_bond_vault_authority;
}

pub fn build_pending_bond_project_values(
    is_config_paused: bool,
    current_project_count: u64,
    expected_project_id: u64,
    project_owner: Pubkey,
    project_name_hash: [u8; 32],
    project_url_hash: [u8; 32],
    token_mint: Pubkey,
    project_treasury_wallet: Pubkey,
    total_bond_amount: u64,
    submitted_at: i64,
    bump: u8,
) -> Result<GreenLabelProjectInitValues> {
    require!(!is_config_paused, CustomError::InvalidGreenLabelStatus);

    let next_project_id = current_project_count
        .checked_add(1)
        .ok_or(CustomError::GreenLabelMathOverflow)?;
    require!(
        expected_project_id == next_project_id,
        CustomError::InvalidGreenLabelProjectId
    );

    let (split, bond_tier) = derive_bond_split_and_tier(total_bond_amount)?;

    Ok(GreenLabelProjectInitValues {
        project_id: expected_project_id,
        project_owner,
        project_name_hash,
        project_url_hash,
        token_mint,
        project_treasury_wallet,
        base_bond_amount: split.base_bond_amount,
        extra_bond_amount: split.extra_bond_amount,
        total_bond_amount: split.total_bond_amount,
        bond_vault: Pubkey::default(),
        bond_vault_authority: Pubkey::default(),
        bond_tier,
        status: GreenLabelStatus::PendingBondDeposit,
        submitted_at,
        observation_start_ts: 0,
        observation_end_ts: 0,
        dispute_count: 0,
        active_dispute: Pubkey::default(),
        approved_at: 0,
        refunded_at: 0,
        slashed_at: 0,
        risk_score_snapshot: 0,
        terminal_proposal_id: 0,
        terminal_proposal_decision: Pubkey::default(),
        terminal_execution_queue_item: Pubkey::default(),
        terminal_payload_hash: [0; 32],
        terminal_action_type: ActionType::Noop,
        bump,
        reserved: [0; GREEN_LABEL_PROJECT_RESERVED_BYTES],
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

    if from == GreenLabelStatus::Disputed
        && to == GreenLabelStatus::SlashQueued
        && !has_linked_dispute
    {
        return err!(CustomError::InvalidGreenLabelSlashWithoutDispute);
    }

    let is_valid = matches!(
        (from, to),
        (
            GreenLabelStatus::PendingBondDeposit,
            GreenLabelStatus::PendingObservation
        ) | (
            GreenLabelStatus::PendingBondDeposit,
            GreenLabelStatus::Cancelled
        ) | (
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

fn proposal_type_matches_action(proposal_type: ProposalType, action_type: ActionType) -> bool {
    matches!(
        (proposal_type, action_type),
        (ProposalType::GreenLabelSlash, ActionType::GreenLabelSlash)
            | (ProposalType::GreenLabelRefund, ActionType::GreenLabelRefund)
    )
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
    fn submit_project_defaults_to_pending_bond_deposit() {
        let values = pending_bond_project_values(299_000_000);

        assert_eq!(values.status, GreenLabelStatus::PendingBondDeposit);
    }

    #[test]
    fn submit_project_does_not_start_observation_period() {
        let values = pending_bond_project_values(299_000_000);

        assert_eq!(values.observation_start_ts, 0);
        assert_eq!(values.observation_end_ts, 0);
    }

    #[test]
    fn submit_project_uses_default_empty_bond_vault() {
        let values = pending_bond_project_values(299_000_000);

        assert_eq!(values.bond_vault, Pubkey::default());
        assert_eq!(values.bond_vault_authority, Pubkey::default());
    }

    #[test]
    fn submit_project_sets_base_and_extra_for_299() {
        let values = pending_bond_project_values(299_000_000);

        assert_eq!(values.base_bond_amount, 299_000_000);
        assert_eq!(values.extra_bond_amount, 0);
        assert_eq!(values.total_bond_amount, 299_000_000);
    }

    #[test]
    fn submit_project_sets_base_and_extra_for_1299() {
        let values = pending_bond_project_values(1_299_000_000);

        assert_eq!(values.base_bond_amount, 299_000_000);
        assert_eq!(values.extra_bond_amount, 1_000_000_000);
        assert_eq!(values.total_bond_amount, 1_299_000_000);
    }

    #[test]
    fn submit_project_sets_bond_tier() {
        let values = pending_bond_project_values(1_299_000_000);

        assert_eq!(values.bond_tier, BondTier::Silver);
    }

    #[test]
    fn submit_project_rejects_bond_below_299() {
        let err = try_pending_bond_project_values(false, 0, 1, 298_999_999).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBondAmount");
    }

    #[test]
    fn submit_project_requires_next_project_id() {
        let err = try_pending_bond_project_values(false, 0, 2, 299_000_000).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelProjectId");
    }

    #[test]
    fn submit_project_rejects_when_config_paused() {
        let err = try_pending_bond_project_values(true, 0, 1, 299_000_000).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn submit_project_terminal_fields_are_empty() {
        let values = pending_bond_project_values(299_000_000);

        assert_eq!(values.terminal_proposal_id, 0);
        assert_eq!(values.terminal_proposal_decision, Pubkey::default());
        assert_eq!(values.terminal_execution_queue_item, Pubkey::default());
        assert_eq!(values.terminal_payload_hash, [0; 32]);
        assert_eq!(values.terminal_action_type, ActionType::Noop);
    }

    #[test]
    fn submit_project_reserved_zeroed() {
        let values = pending_bond_project_values(299_000_000);

        assert_eq!(values.reserved, [0; GREEN_LABEL_PROJECT_RESERVED_BYTES]);
    }

    #[test]
    fn bond_vault_init_accepts_pending_bond_deposit_project() {
        validate_green_bond_vault_initialization(
            false,
            Pubkey::new_from_array([8; 32]),
            Pubkey::new_from_array([8; 32]),
            GreenLabelStatus::PendingBondDeposit,
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([2; 32]),
        )
        .unwrap();
    }

    #[test]
    fn bond_vault_init_rejects_paused_config() {
        let err = try_validate_green_bond_vault_initialization(
            true,
            Pubkey::new_from_array([8; 32]),
            GreenLabelStatus::PendingBondDeposit,
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::new_from_array([2; 32]),
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn bond_vault_init_rejects_wrong_project_owner() {
        let err = try_validate_green_bond_vault_initialization(
            false,
            Pubkey::new_from_array([9; 32]),
            GreenLabelStatus::PendingBondDeposit,
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::new_from_array([2; 32]),
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelProjectOwner");
    }

    #[test]
    fn bond_vault_init_rejects_non_pending_bond_status() {
        let err = try_validate_green_bond_vault_initialization(
            false,
            Pubkey::new_from_array([8; 32]),
            GreenLabelStatus::PendingObservation,
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::new_from_array([2; 32]),
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn bond_vault_init_rejects_existing_bond_vault() {
        let err = try_validate_green_bond_vault_initialization(
            false,
            Pubkey::new_from_array([8; 32]),
            GreenLabelStatus::PendingBondDeposit,
            Pubkey::new_from_array([13; 32]),
            Pubkey::default(),
            Pubkey::new_from_array([2; 32]),
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBondVaultState");
    }

    #[test]
    fn bond_vault_init_rejects_existing_bond_vault_authority() {
        let err = try_validate_green_bond_vault_initialization(
            false,
            Pubkey::new_from_array([8; 32]),
            GreenLabelStatus::PendingBondDeposit,
            Pubkey::default(),
            Pubkey::new_from_array([14; 32]),
            Pubkey::new_from_array([2; 32]),
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBondVaultState");
    }

    #[test]
    fn bond_vault_init_rejects_wrong_usdc_mint() {
        let err = try_validate_green_bond_vault_initialization(
            false,
            Pubkey::new_from_array([8; 32]),
            GreenLabelStatus::PendingBondDeposit,
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::new_from_array([3; 32]),
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelMint");
    }

    #[test]
    fn bond_vault_init_keeps_status_pending_bond_deposit() {
        let mut project = pending_bond_project_for_vault_init();

        record_green_bond_vault_initialization(
            &mut project,
            Pubkey::new_from_array([13; 32]),
            Pubkey::new_from_array([14; 32]),
        );

        assert_eq!(project.status, GreenLabelStatus::PendingBondDeposit);
    }

    #[test]
    fn bond_vault_init_does_not_start_observation() {
        let mut project = pending_bond_project_for_vault_init();

        record_green_bond_vault_initialization(
            &mut project,
            Pubkey::new_from_array([13; 32]),
            Pubkey::new_from_array([14; 32]),
        );

        assert_eq!(project.observation_start_ts, 0);
        assert_eq!(project.observation_end_ts, 0);
    }

    #[test]
    fn bond_vault_init_does_not_change_bond_amounts() {
        let mut project = pending_bond_project_for_vault_init();
        let original_amounts = (
            project.base_bond_amount,
            project.extra_bond_amount,
            project.total_bond_amount,
        );

        record_green_bond_vault_initialization(
            &mut project,
            Pubkey::new_from_array([13; 32]),
            Pubkey::new_from_array([14; 32]),
        );

        assert_eq!(
            original_amounts,
            (
                project.base_bond_amount,
                project.extra_bond_amount,
                project.total_bond_amount
            )
        );
    }

    #[test]
    fn bond_lock_accepts_valid_pending_bond_project() {
        validate_bond_lock_fixture(BondLockValidationFixture::valid()).unwrap();
    }

    #[test]
    fn bond_lock_rejects_paused_config() {
        let mut fixture = BondLockValidationFixture::valid();
        fixture.config_is_paused = true;

        let err = validate_bond_lock_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn bond_lock_rejects_wrong_project_owner() {
        let mut fixture = BondLockValidationFixture::valid();
        fixture.signer = Pubkey::new_from_array([9; 32]);

        let err = validate_bond_lock_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelProjectOwner");
    }

    #[test]
    fn bond_lock_rejects_non_pending_bond_status() {
        let mut fixture = BondLockValidationFixture::valid();
        fixture.project_status = GreenLabelStatus::PendingObservation;

        let err = validate_bond_lock_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn bond_lock_rejects_missing_bond_vault() {
        let mut fixture = BondLockValidationFixture::valid();
        fixture.bond_vault = Pubkey::default();

        let err = validate_bond_lock_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBondVaultState");
    }

    #[test]
    fn bond_lock_rejects_missing_bond_vault_authority() {
        let mut fixture = BondLockValidationFixture::valid();
        fixture.bond_vault_authority = Pubkey::default();

        let err = validate_bond_lock_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBondVaultState");
    }

    #[test]
    fn bond_lock_rejects_wrong_bond_vault_account() {
        let mut fixture = BondLockValidationFixture::valid();
        fixture.provided_bond_vault = Pubkey::new_from_array([15; 32]);

        let err = validate_bond_lock_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBondVaultState");
    }

    #[test]
    fn bond_lock_rejects_wrong_bond_vault_mint() {
        let mut fixture = BondLockValidationFixture::valid();
        fixture.provided_bond_vault_mint = Pubkey::new_from_array([3; 32]);

        let err = validate_bond_lock_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelMint");
    }

    #[test]
    fn bond_lock_rejects_wrong_bond_vault_owner() {
        let mut fixture = BondLockValidationFixture::valid();
        fixture.provided_bond_vault_owner = Pubkey::new_from_array([16; 32]);

        let err = validate_bond_lock_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBondVaultState");
    }

    #[test]
    fn bond_lock_rejects_wrong_owner_ata_owner() {
        let mut fixture = BondLockValidationFixture::valid();
        fixture.owner_ata_owner = Pubkey::new_from_array([17; 32]);

        let err = validate_bond_lock_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelTokenAccount");
    }

    #[test]
    fn bond_lock_rejects_wrong_owner_ata_mint() {
        let mut fixture = BondLockValidationFixture::valid();
        fixture.owner_ata_mint = Pubkey::new_from_array([3; 32]);

        let err = validate_bond_lock_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelMint");
    }

    #[test]
    fn bond_lock_rejects_wrong_usdc_mint() {
        let mut fixture = BondLockValidationFixture::valid();
        fixture.usdc_mint = Pubkey::new_from_array([3; 32]);

        let err = validate_bond_lock_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelMint");
    }

    #[test]
    fn bond_lock_rejects_bond_below_299() {
        let mut fixture = BondLockValidationFixture::valid();
        fixture.total_bond_amount = 298_999_999;

        let err = validate_bond_lock_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelBondAmount");
    }

    #[test]
    fn observation_window_sets_start_and_end() {
        let (start, end) = build_observation_window(1_000, 30).unwrap();

        assert_eq!(start, 1_000);
        assert_eq!(end, 1_030);
    }

    #[test]
    fn observation_window_rejects_overflow() {
        let err = build_observation_window(i64::MAX, 1).unwrap_err();

        assert_error_contains(err, "GreenLabelMathOverflow");
    }

    #[test]
    fn record_bond_locked_sets_pending_observation() {
        let mut project = pending_bond_project_for_lock();

        record_green_label_bond_locked(&mut project, 1_000, 2_000).unwrap();

        assert_eq!(project.status, GreenLabelStatus::PendingObservation);
    }

    #[test]
    fn record_bond_locked_sets_observation_timestamps() {
        let mut project = pending_bond_project_for_lock();

        record_green_label_bond_locked(&mut project, 1_000, 2_000).unwrap();

        assert_eq!(project.observation_start_ts, 1_000);
        assert_eq!(project.observation_end_ts, 2_000);
    }

    #[test]
    fn record_bond_locked_does_not_change_bond_amounts() {
        let mut project = pending_bond_project_for_lock();
        let original_amounts = (
            project.base_bond_amount,
            project.extra_bond_amount,
            project.total_bond_amount,
        );

        record_green_label_bond_locked(&mut project, 1_000, 2_000).unwrap();

        assert_eq!(
            original_amounts,
            (
                project.base_bond_amount,
                project.extra_bond_amount,
                project.total_bond_amount
            )
        );
    }

    #[test]
    fn record_bond_locked_does_not_change_terminal_fields() {
        let mut project = pending_bond_project_for_lock();
        let original_terminal_fields = (
            project.terminal_proposal_id,
            project.terminal_proposal_decision,
            project.terminal_execution_queue_item,
            project.terminal_payload_hash,
            project.terminal_action_type,
        );

        record_green_label_bond_locked(&mut project, 1_000, 2_000).unwrap();

        assert_eq!(
            original_terminal_fields,
            (
                project.terminal_proposal_id,
                project.terminal_proposal_decision,
                project.terminal_execution_queue_item,
                project.terminal_payload_hash,
                project.terminal_action_type
            )
        );
    }

    #[test]
    fn record_bond_locked_does_not_change_dispute_fields() {
        let mut project = pending_bond_project_for_lock();
        project.dispute_count = 7;
        project.active_dispute = Pubkey::new_from_array([19; 32]);
        let original_dispute_fields = (project.dispute_count, project.active_dispute);

        record_green_label_bond_locked(&mut project, 1_000, 2_000).unwrap();

        assert_eq!(
            original_dispute_fields,
            (project.dispute_count, project.active_dispute)
        );
    }

    #[test]
    fn open_dispute_accepts_pending_observation_project() {
        validate_open_green_label_dispute(
            false,
            GreenLabelStatus::PendingObservation,
            Pubkey::default(),
            0,
            1,
            [1; 32],
        )
        .unwrap();
    }

    #[test]
    fn open_dispute_accepts_active_green_label_project() {
        validate_open_green_label_dispute(
            false,
            GreenLabelStatus::ActiveGreenLabel,
            Pubkey::default(),
            0,
            1,
            [1; 32],
        )
        .unwrap();
    }

    #[test]
    fn open_dispute_rejects_paused_config() {
        let err = validate_open_green_label_dispute(
            true,
            GreenLabelStatus::PendingObservation,
            Pubkey::default(),
            0,
            1,
            [1; 32],
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn open_dispute_rejects_pending_bond_deposit() {
        let err = validate_open_green_label_dispute(
            false,
            GreenLabelStatus::PendingBondDeposit,
            Pubkey::default(),
            0,
            1,
            [1; 32],
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn open_dispute_rejects_refunded() {
        let err = validate_open_green_label_dispute(
            false,
            GreenLabelStatus::Refunded,
            Pubkey::default(),
            0,
            1,
            [1; 32],
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn open_dispute_rejects_slashed() {
        let err = validate_open_green_label_dispute(
            false,
            GreenLabelStatus::Slashed,
            Pubkey::default(),
            0,
            1,
            [1; 32],
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn open_dispute_rejects_existing_active_dispute() {
        let err = validate_open_green_label_dispute(
            false,
            GreenLabelStatus::PendingObservation,
            Pubkey::new_from_array([18; 32]),
            0,
            1,
            [1; 32],
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelActiveDispute");
    }

    #[test]
    fn open_dispute_requires_next_dispute_id() {
        let err = validate_open_green_label_dispute(
            false,
            GreenLabelStatus::PendingObservation,
            Pubkey::default(),
            0,
            2,
            [1; 32],
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelDisputeId");
    }

    #[test]
    fn open_dispute_rejects_zero_evidence_hash() {
        let err = validate_open_green_label_dispute(
            false,
            GreenLabelStatus::PendingObservation,
            Pubkey::default(),
            0,
            1,
            [0; 32],
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelEvidenceHash");
    }

    #[test]
    fn dispute_windows_set_evidence_and_response_end() {
        let (evidence_end_ts, response_end_ts) = build_dispute_windows(1_000, 60, 30).unwrap();

        assert_eq!(evidence_end_ts, 1_060);
        assert_eq!(response_end_ts, 1_090);
    }

    #[test]
    fn dispute_windows_reject_evidence_overflow() {
        let err = build_dispute_windows(i64::MAX, 1, 0).unwrap_err();

        assert_error_contains(err, "GreenLabelMathOverflow");
    }

    #[test]
    fn dispute_windows_reject_response_overflow() {
        let err = build_dispute_windows(i64::MAX - 1, 1, 1).unwrap_err();

        assert_error_contains(err, "GreenLabelMathOverflow");
    }

    #[test]
    fn record_dispute_opened_sets_project_disputed() {
        let mut project = project_for_open_dispute_record();

        record_green_label_dispute_opened(&mut project, Pubkey::new_from_array([18; 32]), 1)
            .unwrap();

        assert_eq!(project.status, GreenLabelStatus::Disputed);
    }

    #[test]
    fn record_dispute_opened_sets_active_dispute() {
        let mut project = project_for_open_dispute_record();
        let dispute_key = Pubkey::new_from_array([18; 32]);

        record_green_label_dispute_opened(&mut project, dispute_key, 1).unwrap();

        assert_eq!(project.active_dispute, dispute_key);
    }

    #[test]
    fn record_dispute_opened_updates_dispute_count() {
        let mut project = project_for_open_dispute_record();

        record_green_label_dispute_opened(&mut project, Pubkey::new_from_array([18; 32]), 3)
            .unwrap();

        assert_eq!(project.dispute_count, 3);
    }

    #[test]
    fn record_dispute_opened_does_not_change_bond_amounts() {
        let mut project = project_for_open_dispute_record();
        let original_amounts = (
            project.base_bond_amount,
            project.extra_bond_amount,
            project.total_bond_amount,
        );

        record_green_label_dispute_opened(&mut project, Pubkey::new_from_array([18; 32]), 1)
            .unwrap();

        assert_eq!(
            original_amounts,
            (
                project.base_bond_amount,
                project.extra_bond_amount,
                project.total_bond_amount
            )
        );
    }

    #[test]
    fn record_dispute_opened_does_not_change_observation_timestamps() {
        let mut project = project_for_open_dispute_record();
        let original_observation_fields =
            (project.observation_start_ts, project.observation_end_ts);

        record_green_label_dispute_opened(&mut project, Pubkey::new_from_array([18; 32]), 1)
            .unwrap();

        assert_eq!(
            original_observation_fields,
            (project.observation_start_ts, project.observation_end_ts)
        );
    }

    #[test]
    fn record_dispute_opened_does_not_change_terminal_fields() {
        let mut project = project_for_open_dispute_record();
        let original_terminal_fields = (
            project.terminal_proposal_id,
            project.terminal_proposal_decision,
            project.terminal_execution_queue_item,
            project.terminal_payload_hash,
            project.terminal_action_type,
        );

        record_green_label_dispute_opened(&mut project, Pubkey::new_from_array([18; 32]), 1)
            .unwrap();

        assert_eq!(
            original_terminal_fields,
            (
                project.terminal_proposal_id,
                project.terminal_proposal_decision,
                project.terminal_execution_queue_item,
                project.terminal_payload_hash,
                project.terminal_action_type
            )
        );
    }

    #[test]
    fn mark_dispute_ready_accepts_evidence_period_after_response_end() {
        validate_mark_ready_fixture(MarkReadyValidationFixture::valid()).unwrap();
    }

    #[test]
    fn mark_dispute_ready_accepts_project_response_period_after_response_end() {
        let mut fixture = MarkReadyValidationFixture::valid();
        fixture.dispute_status = DisputeStatus::ProjectResponsePeriod;

        validate_mark_ready_fixture(fixture).unwrap();
    }

    #[test]
    fn mark_dispute_ready_rejects_paused_config() {
        let mut fixture = MarkReadyValidationFixture::valid();
        fixture.config_is_paused = true;

        let err = validate_mark_ready_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn mark_dispute_ready_rejects_non_disputed_project() {
        let mut fixture = MarkReadyValidationFixture::valid();
        fixture.project_status = GreenLabelStatus::PendingObservation;

        let err = validate_mark_ready_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn mark_dispute_ready_rejects_wrong_active_dispute() {
        let mut fixture = MarkReadyValidationFixture::valid();
        fixture.project_active_dispute = Pubkey::new_from_array([22; 32]);

        let err = validate_mark_ready_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelActiveDispute");
    }

    #[test]
    fn mark_dispute_ready_rejects_wrong_dispute_project() {
        let mut fixture = MarkReadyValidationFixture::valid();
        fixture.dispute_project = Pubkey::new_from_array([23; 32]);

        let err = validate_mark_ready_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelTargetAccount");
    }

    #[test]
    fn mark_dispute_ready_rejects_invalid_dispute_status() {
        let mut fixture = MarkReadyValidationFixture::valid();
        fixture.dispute_status = DisputeStatus::ReadyForDecision;

        let err = validate_mark_ready_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelDisputeStatus");
    }

    #[test]
    fn mark_dispute_ready_rejects_before_response_end() {
        let mut fixture = MarkReadyValidationFixture::valid();
        fixture.now = 999;
        fixture.response_end_ts = 1_000;

        let err = validate_mark_ready_fixture(fixture).unwrap_err();

        assert_error_contains(err, "GreenLabelDisputeWindowNotEnded");
    }

    #[test]
    fn record_dispute_ready_sets_ready_for_decision() {
        let mut dispute = dispute_for_ready_record();

        record_dispute_ready_for_decision(&mut dispute).unwrap();

        assert_eq!(dispute.status, DisputeStatus::ReadyForDecision);
    }

    #[test]
    fn record_dispute_ready_does_not_change_security_fields() {
        let mut dispute = dispute_for_ready_record();
        dispute.proposal_id = 7;
        dispute.proposal_decision = Pubkey::new_from_array([21; 32]);
        dispute.execution_queue_item = Pubkey::new_from_array([22; 32]);
        dispute.payload_hash = [23; 32];
        dispute.action_type = ActionType::GreenLabelSlash;
        dispute.resolved_at = 99;
        let original_security_fields = (
            dispute.proposal_id,
            dispute.proposal_decision,
            dispute.execution_queue_item,
            dispute.payload_hash,
            dispute.action_type,
            dispute.resolved_at,
        );

        record_dispute_ready_for_decision(&mut dispute).unwrap();

        assert_eq!(
            original_security_fields,
            (
                dispute.proposal_id,
                dispute.proposal_decision,
                dispute.execution_queue_item,
                dispute.payload_hash,
                dispute.action_type,
                dispute.resolved_at
            )
        );
    }

    #[test]
    fn link_security_decision_accepts_green_label_slash() {
        validate_link_decision_fixture(LinkDecisionValidationFixture::valid(
            ActionType::GreenLabelSlash,
        ))
        .unwrap();
    }

    #[test]
    fn link_security_decision_accepts_green_label_refund() {
        validate_link_decision_fixture(LinkDecisionValidationFixture::valid(
            ActionType::GreenLabelRefund,
        ))
        .unwrap();
    }

    #[test]
    fn link_security_decision_rejects_paused_config() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.config_is_paused = true;

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn link_security_decision_rejects_non_disputed_project() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.project_status = GreenLabelStatus::PendingObservation;

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn link_security_decision_rejects_wrong_active_dispute() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.project_active_dispute = Pubkey::new_from_array([22; 32]);

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelActiveDispute");
    }

    #[test]
    fn link_security_decision_rejects_dispute_not_ready() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.dispute_status = DisputeStatus::EvidencePeriod;

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelDisputeStatus");
    }

    #[test]
    fn link_security_decision_rejects_zero_payload_hash() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.expected_payload_hash = [0; 32];
        fixture.queue_payload_hash = [0; 32];

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelPayloadHash");
    }

    #[test]
    fn link_security_decision_rejects_invalid_action_type() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.expected_action_type = ActionType::Noop;

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelActionType");
    }

    #[test]
    fn link_security_decision_rejects_wrong_proposal_id() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.proposal_id = 8;

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelSecurityDecision");
    }

    #[test]
    fn link_security_decision_rejects_wrong_proposal_type() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.proposal_type = ProposalType::GreenLabelRefund;

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelSecurityDecision");
    }

    #[test]
    fn link_security_decision_rejects_non_approved_decision() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.proposal_decision = ProposalDecision::Partial;

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelSecurityDecision");
    }

    #[test]
    fn link_security_decision_rejects_wrong_queue_proposal_id() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.queue_proposal_id = 8;

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelExecutionQueue");
    }

    #[test]
    fn link_security_decision_rejects_wrong_queue_action_type() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.queue_action_type = ActionType::GreenLabelRefund;

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelExecutionQueue");
    }

    #[test]
    fn link_security_decision_rejects_non_queued_status() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.queue_status = ExecutionStatus::Executed;

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelExecutionQueue");
    }

    #[test]
    fn link_security_decision_rejects_payload_hash_mismatch() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.queue_payload_hash = [24; 32];

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelExecutionQueue");
    }

    #[test]
    fn link_security_decision_rejects_wrong_target_program() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.queue_target_program = Pubkey::new_from_array([25; 32]);

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelTargetProgram");
    }

    #[test]
    fn link_security_decision_rejects_wrong_target_account() {
        let mut fixture = LinkDecisionValidationFixture::valid(ActionType::GreenLabelSlash);
        fixture.queue_target_account = Pubkey::new_from_array([26; 32]);

        let err = validate_link_decision_fixture(fixture).unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelTargetAccount");
    }

    #[test]
    fn record_security_decision_link_sets_project_slash_queued() {
        let (mut project, mut dispute) = security_link_record_accounts();

        record_green_label_security_decision_link(
            &mut project,
            &mut dispute,
            7,
            Pubkey::new_from_array([21; 32]),
            Pubkey::new_from_array([22; 32]),
            [23; 32],
            ActionType::GreenLabelSlash,
        )
        .unwrap();

        assert_eq!(project.status, GreenLabelStatus::SlashQueued);
    }

    #[test]
    fn record_security_decision_link_sets_project_refund_queued() {
        let (mut project, mut dispute) = security_link_record_accounts();

        record_green_label_security_decision_link(
            &mut project,
            &mut dispute,
            7,
            Pubkey::new_from_array([21; 32]),
            Pubkey::new_from_array([22; 32]),
            [23; 32],
            ActionType::GreenLabelRefund,
        )
        .unwrap();

        assert_eq!(project.status, GreenLabelStatus::RefundQueued);
    }

    #[test]
    fn record_security_decision_link_sets_dispute_decision_queued() {
        let (mut project, mut dispute) = security_link_record_accounts();

        record_green_label_security_decision_link(
            &mut project,
            &mut dispute,
            7,
            Pubkey::new_from_array([21; 32]),
            Pubkey::new_from_array([22; 32]),
            [23; 32],
            ActionType::GreenLabelSlash,
        )
        .unwrap();

        assert_eq!(dispute.status, DisputeStatus::DecisionQueued);
    }

    #[test]
    fn record_security_decision_link_records_terminal_fields() {
        let (mut project, mut dispute) = security_link_record_accounts();
        let proposal_decision = Pubkey::new_from_array([21; 32]);
        let execution_queue_item = Pubkey::new_from_array([22; 32]);
        let payload_hash = [23; 32];

        record_green_label_security_decision_link(
            &mut project,
            &mut dispute,
            7,
            proposal_decision,
            execution_queue_item,
            payload_hash,
            ActionType::GreenLabelSlash,
        )
        .unwrap();

        assert_eq!(project.terminal_proposal_id, 7);
        assert_eq!(project.terminal_proposal_decision, proposal_decision);
        assert_eq!(project.terminal_execution_queue_item, execution_queue_item);
        assert_eq!(project.terminal_payload_hash, payload_hash);
        assert_eq!(project.terminal_action_type, ActionType::GreenLabelSlash);
        assert_eq!(dispute.proposal_id, 7);
        assert_eq!(dispute.proposal_decision, proposal_decision);
        assert_eq!(dispute.execution_queue_item, execution_queue_item);
        assert_eq!(dispute.payload_hash, payload_hash);
        assert_eq!(dispute.action_type, ActionType::GreenLabelSlash);
    }

    #[test]
    fn record_security_decision_link_does_not_change_bond_fields() {
        let (mut project, mut dispute) = security_link_record_accounts();
        let original_bond_fields = (
            project.bond_vault,
            project.bond_vault_authority,
            project.base_bond_amount,
            project.extra_bond_amount,
            project.total_bond_amount,
        );

        record_green_label_security_decision_link(
            &mut project,
            &mut dispute,
            7,
            Pubkey::new_from_array([21; 32]),
            Pubkey::new_from_array([22; 32]),
            [23; 32],
            ActionType::GreenLabelSlash,
        )
        .unwrap();

        assert_eq!(
            original_bond_fields,
            (
                project.bond_vault,
                project.bond_vault_authority,
                project.base_bond_amount,
                project.extra_bond_amount,
                project.total_bond_amount
            )
        );
    }

    #[test]
    fn record_security_decision_link_does_not_change_observation_timestamps() {
        let (mut project, mut dispute) = security_link_record_accounts();
        let original_observation_fields =
            (project.observation_start_ts, project.observation_end_ts);

        record_green_label_security_decision_link(
            &mut project,
            &mut dispute,
            7,
            Pubkey::new_from_array([21; 32]),
            Pubkey::new_from_array([22; 32]),
            [23; 32],
            ActionType::GreenLabelSlash,
        )
        .unwrap();

        assert_eq!(
            original_observation_fields,
            (project.observation_start_ts, project.observation_end_ts)
        );
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
    fn status_transition_pending_bond_deposit_to_pending_observation() {
        validate_green_label_status_transition(
            GreenLabelStatus::PendingBondDeposit,
            GreenLabelStatus::PendingObservation,
            false,
        )
        .unwrap();
    }

    #[test]
    fn status_transition_pending_bond_deposit_to_cancelled() {
        validate_green_label_status_transition(
            GreenLabelStatus::PendingBondDeposit,
            GreenLabelStatus::Cancelled,
            false,
        )
        .unwrap();
    }

    #[test]
    fn status_transition_pending_bond_deposit_rejects_refund_queued() {
        let err = validate_green_label_status_transition(
            GreenLabelStatus::PendingBondDeposit,
            GreenLabelStatus::RefundQueued,
            false,
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn status_transition_pending_bond_deposit_rejects_slash_queued() {
        let err = validate_green_label_status_transition(
            GreenLabelStatus::PendingBondDeposit,
            GreenLabelStatus::SlashQueued,
            true,
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn status_transition_pending_bond_deposit_rejects_active_green_label() {
        let err = validate_green_label_status_transition(
            GreenLabelStatus::PendingBondDeposit,
            GreenLabelStatus::ActiveGreenLabel,
            false,
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
    }

    #[test]
    fn status_transition_pending_bond_deposit_rejects_disputed() {
        let err = validate_green_label_status_transition(
            GreenLabelStatus::PendingBondDeposit,
            GreenLabelStatus::Disputed,
            false,
        )
        .unwrap_err();

        assert_error_contains(err, "InvalidGreenLabelStatus");
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

    fn pending_bond_project_values(total_bond_amount: u64) -> GreenLabelProjectInitValues {
        try_pending_bond_project_values(false, 0, 1, total_bond_amount).unwrap()
    }

    fn try_pending_bond_project_values(
        is_config_paused: bool,
        current_project_count: u64,
        expected_project_id: u64,
        total_bond_amount: u64,
    ) -> Result<GreenLabelProjectInitValues> {
        build_pending_bond_project_values(
            is_config_paused,
            current_project_count,
            expected_project_id,
            Pubkey::new_from_array([8; 32]),
            [9; 32],
            [10; 32],
            Pubkey::new_from_array([11; 32]),
            Pubkey::new_from_array([12; 32]),
            total_bond_amount,
            1_717_171_717,
            249,
        )
    }

    #[derive(Clone, Copy)]
    struct BondLockValidationFixture {
        config_is_paused: bool,
        project_owner: Pubkey,
        signer: Pubkey,
        project_status: GreenLabelStatus,
        bond_vault: Pubkey,
        bond_vault_authority: Pubkey,
        provided_bond_vault: Pubkey,
        provided_bond_vault_mint: Pubkey,
        provided_bond_vault_owner: Pubkey,
        expected_usdc_mint: Pubkey,
        owner_ata_owner: Pubkey,
        owner_ata_mint: Pubkey,
        usdc_mint: Pubkey,
        total_bond_amount: u64,
    }

    impl BondLockValidationFixture {
        fn valid() -> Self {
            let project_owner = Pubkey::new_from_array([8; 32]);
            let bond_vault = Pubkey::new_from_array([13; 32]);
            let bond_vault_authority = Pubkey::new_from_array([14; 32]);
            let usdc_mint = Pubkey::new_from_array([2; 32]);

            Self {
                config_is_paused: false,
                project_owner,
                signer: project_owner,
                project_status: GreenLabelStatus::PendingBondDeposit,
                bond_vault,
                bond_vault_authority,
                provided_bond_vault: bond_vault,
                provided_bond_vault_mint: usdc_mint,
                provided_bond_vault_owner: bond_vault_authority,
                expected_usdc_mint: usdc_mint,
                owner_ata_owner: project_owner,
                owner_ata_mint: usdc_mint,
                usdc_mint,
                total_bond_amount: 1_299_000_000,
            }
        }
    }

    fn validate_bond_lock_fixture(fixture: BondLockValidationFixture) -> Result<()> {
        validate_green_label_bond_lock(
            fixture.config_is_paused,
            fixture.project_owner,
            fixture.signer,
            fixture.project_status,
            fixture.bond_vault,
            fixture.bond_vault_authority,
            fixture.provided_bond_vault,
            fixture.provided_bond_vault_mint,
            fixture.provided_bond_vault_owner,
            fixture.expected_usdc_mint,
            fixture.owner_ata_owner,
            fixture.owner_ata_mint,
            fixture.usdc_mint,
            fixture.total_bond_amount,
        )
    }

    #[derive(Clone, Copy)]
    struct MarkReadyValidationFixture {
        config_is_paused: bool,
        project_status: GreenLabelStatus,
        project_active_dispute: Pubkey,
        dispute_key: Pubkey,
        dispute_project: Pubkey,
        project_key: Pubkey,
        dispute_status: DisputeStatus,
        now: i64,
        response_end_ts: i64,
    }

    impl MarkReadyValidationFixture {
        fn valid() -> Self {
            let project_key = Pubkey::new_from_array([17; 32]);
            let dispute_key = Pubkey::new_from_array([18; 32]);

            Self {
                config_is_paused: false,
                project_status: GreenLabelStatus::Disputed,
                project_active_dispute: dispute_key,
                dispute_key,
                dispute_project: project_key,
                project_key,
                dispute_status: DisputeStatus::EvidencePeriod,
                now: 1_000,
                response_end_ts: 1_000,
            }
        }
    }

    fn validate_mark_ready_fixture(fixture: MarkReadyValidationFixture) -> Result<()> {
        validate_mark_dispute_ready(
            fixture.config_is_paused,
            fixture.project_status,
            fixture.project_active_dispute,
            fixture.dispute_key,
            fixture.dispute_project,
            fixture.project_key,
            fixture.dispute_status,
            fixture.now,
            fixture.response_end_ts,
        )
    }

    #[derive(Clone, Copy)]
    struct LinkDecisionValidationFixture {
        config_is_paused: bool,
        project_status: GreenLabelStatus,
        project_active_dispute: Pubkey,
        dispute_key: Pubkey,
        dispute_project: Pubkey,
        project_key: Pubkey,
        dispute_status: DisputeStatus,
        expected_proposal_id: u64,
        expected_action_type: ActionType,
        expected_payload_hash: [u8; 32],
        proposal_id: u64,
        proposal_type: ProposalType,
        proposal_decision: ProposalDecision,
        queue_proposal_id: u64,
        queue_action_type: ActionType,
        queue_status: ExecutionStatus,
        queue_payload_hash: [u8; 32],
        queue_target_program: Pubkey,
        expected_program_id: Pubkey,
        queue_target_account: Pubkey,
        expected_target_account: Pubkey,
    }

    impl LinkDecisionValidationFixture {
        fn valid(action_type: ActionType) -> Self {
            let project_key = Pubkey::new_from_array([17; 32]);
            let dispute_key = Pubkey::new_from_array([18; 32]);
            let program_id = Pubkey::new_from_array([19; 32]);
            let payload_hash = [23; 32];
            let proposal_type = match action_type {
                ActionType::GreenLabelSlash => ProposalType::GreenLabelSlash,
                ActionType::GreenLabelRefund => ProposalType::GreenLabelRefund,
                _ => ProposalType::EmergencyPause,
            };

            Self {
                config_is_paused: false,
                project_status: GreenLabelStatus::Disputed,
                project_active_dispute: dispute_key,
                dispute_key,
                dispute_project: project_key,
                project_key,
                dispute_status: DisputeStatus::ReadyForDecision,
                expected_proposal_id: 7,
                expected_action_type: action_type,
                expected_payload_hash: payload_hash,
                proposal_id: 7,
                proposal_type,
                proposal_decision: ProposalDecision::Approved,
                queue_proposal_id: 7,
                queue_action_type: action_type,
                queue_status: ExecutionStatus::Queued,
                queue_payload_hash: payload_hash,
                queue_target_program: program_id,
                expected_program_id: program_id,
                queue_target_account: dispute_key,
                expected_target_account: dispute_key,
            }
        }
    }

    fn validate_link_decision_fixture(fixture: LinkDecisionValidationFixture) -> Result<()> {
        validate_green_label_security_decision_link(
            fixture.config_is_paused,
            fixture.project_status,
            fixture.project_active_dispute,
            fixture.dispute_key,
            fixture.dispute_project,
            fixture.project_key,
            fixture.dispute_status,
            fixture.expected_proposal_id,
            fixture.expected_action_type,
            fixture.expected_payload_hash,
            fixture.proposal_id,
            fixture.proposal_type,
            fixture.proposal_decision,
            fixture.queue_proposal_id,
            fixture.queue_action_type,
            fixture.queue_status,
            fixture.queue_payload_hash,
            fixture.queue_target_program,
            fixture.expected_program_id,
            fixture.queue_target_account,
            fixture.expected_target_account,
        )
    }

    fn try_validate_green_bond_vault_initialization(
        config_is_paused: bool,
        signer: Pubkey,
        project_status: GreenLabelStatus,
        existing_bond_vault: Pubkey,
        existing_bond_vault_authority: Pubkey,
        provided_usdc_mint: Pubkey,
    ) -> Result<()> {
        validate_green_bond_vault_initialization(
            config_is_paused,
            Pubkey::new_from_array([8; 32]),
            signer,
            project_status,
            existing_bond_vault,
            existing_bond_vault_authority,
            Pubkey::new_from_array([2; 32]),
            provided_usdc_mint,
        )
    }

    fn pending_bond_project_for_vault_init() -> GreenLabelProjectV1 {
        let mut project = green_label_project();
        project.status = GreenLabelStatus::PendingBondDeposit;
        project.bond_vault = Pubkey::default();
        project.bond_vault_authority = Pubkey::default();
        project.observation_start_ts = 0;
        project.observation_end_ts = 0;
        project
    }

    fn pending_bond_project_for_lock() -> GreenLabelProjectV1 {
        let mut project = green_label_project();
        project.status = GreenLabelStatus::PendingBondDeposit;
        project.observation_start_ts = 0;
        project.observation_end_ts = 0;
        project
    }

    fn project_for_open_dispute_record() -> GreenLabelProjectV1 {
        let mut project = green_label_project();
        project.status = GreenLabelStatus::PendingObservation;
        project.active_dispute = Pubkey::default();
        project.dispute_count = 0;
        project
    }

    fn dispute_for_ready_record() -> GreenLabelDisputeV1 {
        let mut dispute = green_label_dispute();
        dispute.status = DisputeStatus::EvidencePeriod;
        dispute.proposal_id = 0;
        dispute.proposal_decision = Pubkey::default();
        dispute.execution_queue_item = Pubkey::default();
        dispute.payload_hash = [0; 32];
        dispute.action_type = ActionType::Noop;
        dispute.resolved_at = 0;
        dispute
    }

    fn security_link_record_accounts() -> (GreenLabelProjectV1, GreenLabelDisputeV1) {
        let dispute_key = Pubkey::new_from_array([18; 32]);
        let mut project = green_label_project();
        project.status = GreenLabelStatus::Disputed;
        project.active_dispute = dispute_key;
        project.observation_start_ts = 1_000;
        project.observation_end_ts = 2_000;
        project.terminal_proposal_id = 0;
        project.terminal_proposal_decision = Pubkey::default();
        project.terminal_execution_queue_item = Pubkey::default();
        project.terminal_payload_hash = [0; 32];
        project.terminal_action_type = ActionType::Noop;

        let mut dispute = green_label_dispute();
        dispute.status = DisputeStatus::ReadyForDecision;
        dispute.proposal_id = 0;
        dispute.proposal_decision = Pubkey::default();
        dispute.execution_queue_item = Pubkey::default();
        dispute.payload_hash = [0; 32];
        dispute.action_type = ActionType::Noop;

        (project, dispute)
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
