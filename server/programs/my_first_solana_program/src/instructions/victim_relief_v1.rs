use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

use crate::constants::{
    GOVERNANCE_CONFIG_V1_SEED, TREASURY_CONFIG_V2_SEED, VICTIM_RELIEF_CASE_V1_SEED,
    VICTIM_RELIEF_CLAIMANT_STATE_V1_SEED, VICTIM_RELIEF_CONFIG_V1_SEED,
    VICTIM_RELIEF_POLICY_V1_SEED, VICTIM_RELIEF_POLICY_VERSION_V1, VICTIM_RELIEF_SCHEMA_VERSION_V1,
};
use crate::error::CustomError;
use crate::state::{
    GovernanceConfigV1, TreasuryConfigV2, VictimReliefCaseStatusV1, VictimReliefCaseV1,
    VictimReliefClaimantStateV1, VictimReliefConfigV1, VictimReliefPolicyV1,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VictimReliefPolicyParametersV1 {
    pub min_claim_amount_usdc: u64,
    pub max_claim_amount_usdc: u64,
    pub max_payout_per_case_usdc: u64,
    pub evidence_window_seconds: i64,
    pub review_window_seconds: i64,
    pub appeal_window_seconds: i64,
    pub submission_cooldown_seconds: i64,
    pub max_evidence_items: u32,
    pub max_active_cases_per_claimant: u16,
}

#[derive(Accounts)]
pub struct InitializeVictimReliefConfigV1<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + VictimReliefConfigV1::INIT_SPACE,
        seeds = [VICTIM_RELIEF_CONFIG_V1_SEED],
        bump
    )]
    pub victim_relief_config: Account<'info, VictimReliefConfigV1>,

    #[account(
        seeds = [GOVERNANCE_CONFIG_V1_SEED],
        bump = security_governance_config.bump
    )]
    pub security_governance_config: Account<'info, GovernanceConfigV1>,

    #[account(
        seeds = [TREASURY_CONFIG_V2_SEED],
        bump = treasury_config.bump
    )]
    pub treasury_config: Account<'info, TreasuryConfigV2>,

    #[account(
        constraint = usdc_mint.key() == treasury_config.usdc_mint @ CustomError::InvalidMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub bootstrap_authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeVictimReliefPolicyV1<'info> {
    #[account(
        mut,
        seeds = [VICTIM_RELIEF_CONFIG_V1_SEED],
        bump = victim_relief_config.bump
    )]
    pub victim_relief_config: Account<'info, VictimReliefConfigV1>,

    #[account(
        init,
        payer = payer,
        space = 8 + VictimReliefPolicyV1::INIT_SPACE,
        seeds = [
            VICTIM_RELIEF_POLICY_V1_SEED,
            victim_relief_config.key().as_ref(),
            &VICTIM_RELIEF_POLICY_VERSION_V1.to_le_bytes()
        ],
        bump
    )]
    pub victim_relief_policy: Account<'info, VictimReliefPolicyV1>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(case_id: u64)]
pub struct SubmitVictimReliefCaseV1<'info> {
    #[account(
        mut,
        seeds = [VICTIM_RELIEF_CONFIG_V1_SEED],
        bump = victim_relief_config.bump
    )]
    pub victim_relief_config: Account<'info, VictimReliefConfigV1>,

    #[account(
        seeds = [
            VICTIM_RELIEF_POLICY_V1_SEED,
            victim_relief_config.key().as_ref(),
            &VICTIM_RELIEF_POLICY_VERSION_V1.to_le_bytes()
        ],
        bump = victim_relief_policy.bump,
        constraint = victim_relief_config.current_policy == victim_relief_policy.key() @ CustomError::InvalidVictimReliefPolicy,
        constraint = victim_relief_config.current_policy_version == victim_relief_policy.policy_version @ CustomError::InvalidVictimReliefPolicyVersion
    )]
    pub victim_relief_policy: Account<'info, VictimReliefPolicyV1>,

    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + VictimReliefClaimantStateV1::INIT_SPACE,
        seeds = [
            VICTIM_RELIEF_CLAIMANT_STATE_V1_SEED,
            victim_relief_config.key().as_ref(),
            claimant.key().as_ref()
        ],
        bump
    )]
    pub claimant_state: Account<'info, VictimReliefClaimantStateV1>,

    #[account(
        init,
        payer = payer,
        space = 8 + VictimReliefCaseV1::INIT_SPACE,
        seeds = [
            VICTIM_RELIEF_CASE_V1_SEED,
            victim_relief_config.key().as_ref(),
            &case_id.to_le_bytes()
        ],
        bump
    )]
    pub victim_relief_case: Account<'info, VictimReliefCaseV1>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub claimant: Signer<'info>,

    pub claimant_recipient_usdc_account: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = usdc_mint.key() == victim_relief_config.usdc_mint @ CustomError::InvalidMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateVictimReliefEvidenceRootV1<'info> {
    #[account(
        seeds = [VICTIM_RELIEF_CONFIG_V1_SEED],
        bump = victim_relief_config.bump
    )]
    pub victim_relief_config: Account<'info, VictimReliefConfigV1>,

    #[account(
        seeds = [
            VICTIM_RELIEF_POLICY_V1_SEED,
            victim_relief_config.key().as_ref(),
            &VICTIM_RELIEF_POLICY_VERSION_V1.to_le_bytes()
        ],
        bump = victim_relief_policy.bump
    )]
    pub victim_relief_policy: Account<'info, VictimReliefPolicyV1>,

    #[account(
        mut,
        seeds = [
            VICTIM_RELIEF_CASE_V1_SEED,
            victim_relief_config.key().as_ref(),
            &victim_relief_case.case_id.to_le_bytes()
        ],
        bump = victim_relief_case.bump
    )]
    pub victim_relief_case: Account<'info, VictimReliefCaseV1>,

    pub claimant: Signer<'info>,
}

#[derive(Accounts)]
pub struct CancelVictimReliefCaseV1<'info> {
    #[account(
        seeds = [VICTIM_RELIEF_CONFIG_V1_SEED],
        bump = victim_relief_config.bump
    )]
    pub victim_relief_config: Account<'info, VictimReliefConfigV1>,

    #[account(
        mut,
        seeds = [
            VICTIM_RELIEF_CLAIMANT_STATE_V1_SEED,
            victim_relief_config.key().as_ref(),
            claimant.key().as_ref()
        ],
        bump = claimant_state.bump
    )]
    pub claimant_state: Account<'info, VictimReliefClaimantStateV1>,

    #[account(
        mut,
        seeds = [
            VICTIM_RELIEF_CASE_V1_SEED,
            victim_relief_config.key().as_ref(),
            &victim_relief_case.case_id.to_le_bytes()
        ],
        bump = victim_relief_case.bump
    )]
    pub victim_relief_case: Account<'info, VictimReliefCaseV1>,

    pub claimant: Signer<'info>,
}

#[derive(Accounts)]
pub struct ExpireVictimReliefCaseV1<'info> {
    #[account(
        seeds = [VICTIM_RELIEF_CONFIG_V1_SEED],
        bump = victim_relief_config.bump
    )]
    pub victim_relief_config: Account<'info, VictimReliefConfigV1>,

    #[account(
        mut,
        seeds = [
            VICTIM_RELIEF_CLAIMANT_STATE_V1_SEED,
            victim_relief_config.key().as_ref(),
            victim_relief_case.claimant.as_ref()
        ],
        bump = claimant_state.bump
    )]
    pub claimant_state: Account<'info, VictimReliefClaimantStateV1>,

    #[account(
        mut,
        seeds = [
            VICTIM_RELIEF_CASE_V1_SEED,
            victim_relief_config.key().as_ref(),
            &victim_relief_case.case_id.to_le_bytes()
        ],
        bump = victim_relief_case.bump
    )]
    pub victim_relief_case: Account<'info, VictimReliefCaseV1>,

    pub executor: Signer<'info>,
}

pub fn initialize_victim_relief_config_v1_handler(
    ctx: Context<InitializeVictimReliefConfigV1>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;

    require!(
        ctx.accounts.bootstrap_authority.key() == ctx.accounts.security_governance_config.authority,
        CustomError::UnauthorizedSecurityAuthority
    );
    require!(
        ctx.accounts.treasury_config.usdc_mint == ctx.accounts.usdc_mint.key(),
        CustomError::InvalidMint
    );

    record_victim_relief_config_init_v1(
        &mut ctx.accounts.victim_relief_config,
        ctx.accounts.security_governance_config.authority,
        ctx.accounts.treasury_config.key(),
        ctx.accounts.security_governance_config.key(),
        ctx.accounts.usdc_mint.key(),
        now,
        ctx.bumps.victim_relief_config,
    )
}

pub fn initialize_victim_relief_policy_v1_handler(
    ctx: Context<InitializeVictimReliefPolicyV1>,
    min_claim_amount_usdc: u64,
    max_claim_amount_usdc: u64,
    max_payout_per_case_usdc: u64,
    evidence_window_seconds: i64,
    review_window_seconds: i64,
    appeal_window_seconds: i64,
    submission_cooldown_seconds: i64,
    max_evidence_items: u32,
    max_active_cases_per_claimant: u16,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let config_key = ctx.accounts.victim_relief_config.key();
    let policy_key = ctx.accounts.victim_relief_policy.key();
    let parameters = VictimReliefPolicyParametersV1 {
        min_claim_amount_usdc,
        max_claim_amount_usdc,
        max_payout_per_case_usdc,
        evidence_window_seconds,
        review_window_seconds,
        appeal_window_seconds,
        submission_cooldown_seconds,
        max_evidence_items,
        max_active_cases_per_claimant,
    };

    record_victim_relief_policy_init_v1(
        &mut ctx.accounts.victim_relief_config,
        &mut ctx.accounts.victim_relief_policy,
        config_key,
        policy_key,
        ctx.accounts.authority.key(),
        parameters,
        now,
        ctx.bumps.victim_relief_policy,
    )
}

pub fn submit_victim_relief_case_v1_handler(
    ctx: Context<SubmitVictimReliefCaseV1>,
    case_id: u64,
    subject_commitment: [u8; 32],
    evidence_root: [u8; 32],
    evidence_count: u32,
    claimed_amount_usdc: u64,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let config_key = ctx.accounts.victim_relief_config.key();
    let policy_key = ctx.accounts.victim_relief_policy.key();
    let claimant_key = ctx.accounts.claimant.key();
    let recipient_key = ctx.accounts.claimant_recipient_usdc_account.key();
    let recipient_owner = ctx.accounts.claimant_recipient_usdc_account.owner;
    let recipient_mint = ctx.accounts.claimant_recipient_usdc_account.mint;
    let usdc_mint = ctx.accounts.usdc_mint.key();

    ensure_victim_relief_claimant_state_v1(
        &mut ctx.accounts.claimant_state,
        config_key,
        claimant_key,
        now,
        ctx.bumps.claimant_state,
    )?;

    validate_victim_relief_case_submission_v1(
        &ctx.accounts.victim_relief_config,
        &ctx.accounts.victim_relief_policy,
        &ctx.accounts.claimant_state,
        policy_key,
        case_id,
        subject_commitment,
        evidence_root,
        evidence_count,
        claimed_amount_usdc,
        recipient_owner,
        recipient_mint,
        usdc_mint,
        now,
    )?;

    record_victim_relief_case_submission_v1(
        &mut ctx.accounts.victim_relief_config,
        &ctx.accounts.victim_relief_policy,
        &mut ctx.accounts.claimant_state,
        &mut ctx.accounts.victim_relief_case,
        config_key,
        policy_key,
        claimant_key,
        recipient_key,
        subject_commitment,
        evidence_root,
        evidence_count,
        claimed_amount_usdc,
        usdc_mint,
        now,
        ctx.bumps.victim_relief_case,
    )
}

pub fn update_victim_relief_evidence_root_v1_handler(
    ctx: Context<UpdateVictimReliefEvidenceRootV1>,
    new_evidence_root: [u8; 32],
    new_evidence_count: u32,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    validate_victim_relief_evidence_update_v1(
        &ctx.accounts.victim_relief_config,
        &ctx.accounts.victim_relief_policy,
        &ctx.accounts.victim_relief_case,
        ctx.accounts.victim_relief_config.key(),
        ctx.accounts.victim_relief_policy.key(),
        ctx.accounts.claimant.key(),
        new_evidence_root,
        new_evidence_count,
        now,
    )?;
    record_victim_relief_evidence_update_v1(
        &mut ctx.accounts.victim_relief_case,
        new_evidence_root,
        new_evidence_count,
        now,
    )
}

pub fn cancel_victim_relief_case_v1_handler(ctx: Context<CancelVictimReliefCaseV1>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    validate_victim_relief_case_claimant_link_v1(
        &ctx.accounts.victim_relief_config,
        &ctx.accounts.claimant_state,
        &ctx.accounts.victim_relief_case,
        ctx.accounts.victim_relief_config.key(),
        ctx.accounts.claimant.key(),
    )?;
    require!(
        ctx.accounts.victim_relief_case.status == VictimReliefCaseStatusV1::EvidencePeriod,
        CustomError::VictimReliefCaseStatusMismatch
    );
    record_victim_relief_case_terminal_status_v1(
        &mut ctx.accounts.claimant_state,
        &mut ctx.accounts.victim_relief_case,
        VictimReliefCaseStatusV1::Cancelled,
        now,
    )
}

pub fn expire_victim_relief_case_v1_handler(ctx: Context<ExpireVictimReliefCaseV1>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    validate_victim_relief_case_claimant_link_v1(
        &ctx.accounts.victim_relief_config,
        &ctx.accounts.claimant_state,
        &ctx.accounts.victim_relief_case,
        ctx.accounts.victim_relief_config.key(),
        ctx.accounts.victim_relief_case.claimant,
    )?;
    require!(
        ctx.accounts.victim_relief_case.status == VictimReliefCaseStatusV1::EvidencePeriod,
        CustomError::VictimReliefCaseStatusMismatch
    );
    require!(
        now > ctx.accounts.victim_relief_case.evidence_deadline,
        CustomError::VictimReliefCaseNotExpired
    );
    record_victim_relief_case_terminal_status_v1(
        &mut ctx.accounts.claimant_state,
        &mut ctx.accounts.victim_relief_case,
        VictimReliefCaseStatusV1::Expired,
        now,
    )
}

pub fn validate_victim_relief_policy_parameters_v1(
    parameters: VictimReliefPolicyParametersV1,
) -> Result<()> {
    require!(
        parameters.min_claim_amount_usdc > 0,
        CustomError::InvalidVictimReliefPolicy
    );
    require!(
        parameters.max_claim_amount_usdc >= parameters.min_claim_amount_usdc,
        CustomError::InvalidVictimReliefPolicy
    );
    require!(
        parameters.max_payout_per_case_usdc > 0,
        CustomError::InvalidVictimReliefPolicy
    );
    require!(
        parameters.max_payout_per_case_usdc <= parameters.max_claim_amount_usdc,
        CustomError::InvalidVictimReliefPolicy
    );
    require!(
        parameters.evidence_window_seconds > 0
            && parameters.review_window_seconds > 0
            && parameters.appeal_window_seconds > 0,
        CustomError::InvalidVictimReliefPolicy
    );
    require!(
        parameters.submission_cooldown_seconds >= 0,
        CustomError::InvalidVictimReliefPolicy
    );
    require!(
        parameters.max_evidence_items > 0,
        CustomError::InvalidVictimReliefPolicy
    );
    require!(
        parameters.max_active_cases_per_claimant > 0,
        CustomError::InvalidVictimReliefPolicy
    );
    Ok(())
}

pub fn record_victim_relief_config_init_v1(
    config: &mut VictimReliefConfigV1,
    authority: Pubkey,
    treasury_config: Pubkey,
    security_governance_config: Pubkey,
    usdc_mint: Pubkey,
    now: i64,
    bump: u8,
) -> Result<()> {
    require!(
        authority != Pubkey::default()
            && treasury_config != Pubkey::default()
            && security_governance_config != Pubkey::default()
            && usdc_mint != Pubkey::default(),
        CustomError::InvalidVictimReliefConfig
    );

    config.authority = authority;
    config.treasury_config = treasury_config;
    config.security_governance_config = security_governance_config;
    config.usdc_mint = usdc_mint;
    config.current_policy = Pubkey::default();
    config.current_policy_version = 0;
    config.next_case_id = 1;
    config.paused = false;
    config.created_at = now;
    config.schema_version = VICTIM_RELIEF_SCHEMA_VERSION_V1;
    config.bump = bump;
    config.reserved = [0; 32];

    Ok(())
}

pub fn record_victim_relief_policy_init_v1(
    config: &mut VictimReliefConfigV1,
    policy: &mut VictimReliefPolicyV1,
    config_key: Pubkey,
    policy_key: Pubkey,
    authority: Pubkey,
    parameters: VictimReliefPolicyParametersV1,
    now: i64,
    bump: u8,
) -> Result<()> {
    validate_victim_relief_policy_parameters_v1(parameters)?;
    require!(
        config.schema_version == VICTIM_RELIEF_SCHEMA_VERSION_V1,
        CustomError::InvalidVictimReliefConfig
    );
    require!(
        authority == config.authority,
        CustomError::UnauthorizedSecurityAuthority
    );
    require!(
        config.current_policy == Pubkey::default() && config.current_policy_version == 0,
        CustomError::VictimReliefPolicyAlreadyInitialized
    );

    policy.config = config_key;
    policy.policy_version = VICTIM_RELIEF_POLICY_VERSION_V1;
    policy.min_claim_amount_usdc = parameters.min_claim_amount_usdc;
    policy.max_claim_amount_usdc = parameters.max_claim_amount_usdc;
    policy.max_payout_per_case_usdc = parameters.max_payout_per_case_usdc;
    policy.evidence_window_seconds = parameters.evidence_window_seconds;
    policy.review_window_seconds = parameters.review_window_seconds;
    policy.appeal_window_seconds = parameters.appeal_window_seconds;
    policy.submission_cooldown_seconds = parameters.submission_cooldown_seconds;
    policy.max_evidence_items = parameters.max_evidence_items;
    policy.max_active_cases_per_claimant = parameters.max_active_cases_per_claimant;
    policy.active = true;
    policy.initialized_by = authority;
    policy.created_at = now;
    policy.schema_version = VICTIM_RELIEF_SCHEMA_VERSION_V1;
    policy.bump = bump;
    policy.reserved = [0; 32];

    config.current_policy = policy_key;
    config.current_policy_version = VICTIM_RELIEF_POLICY_VERSION_V1;

    Ok(())
}

pub fn ensure_victim_relief_claimant_state_v1(
    claimant_state: &mut VictimReliefClaimantStateV1,
    config: Pubkey,
    claimant: Pubkey,
    now: i64,
    bump: u8,
) -> Result<()> {
    if claimant_state.schema_version == 0 {
        claimant_state.config = config;
        claimant_state.claimant = claimant;
        claimant_state.active_case_count = 0;
        claimant_state.total_case_count = 0;
        claimant_state.last_case_id = 0;
        claimant_state.last_submitted_at = 0;
        claimant_state.created_at = now;
        claimant_state.updated_at = now;
        claimant_state.schema_version = VICTIM_RELIEF_SCHEMA_VERSION_V1;
        claimant_state.bump = bump;
    }

    require!(
        claimant_state.schema_version == VICTIM_RELIEF_SCHEMA_VERSION_V1
            && claimant_state.config == config
            && claimant_state.claimant == claimant,
        CustomError::VictimReliefClaimantMismatch
    );
    Ok(())
}

pub fn validate_victim_relief_case_submission_v1(
    config: &VictimReliefConfigV1,
    policy: &VictimReliefPolicyV1,
    claimant_state: &VictimReliefClaimantStateV1,
    policy_key: Pubkey,
    case_id: u64,
    subject_commitment: [u8; 32],
    evidence_root: [u8; 32],
    evidence_count: u32,
    claimed_amount_usdc: u64,
    recipient_owner: Pubkey,
    recipient_mint: Pubkey,
    usdc_mint: Pubkey,
    now: i64,
) -> Result<()> {
    require!(!config.paused, CustomError::VictimReliefPaused);
    require!(
        config.schema_version == VICTIM_RELIEF_SCHEMA_VERSION_V1,
        CustomError::InvalidVictimReliefConfig
    );
    require!(
        policy.schema_version == VICTIM_RELIEF_SCHEMA_VERSION_V1
            && policy.policy_version == VICTIM_RELIEF_POLICY_VERSION_V1
            && policy.active,
        CustomError::InvalidVictimReliefPolicy
    );
    require!(
        config.current_policy == policy_key
            && config.current_policy_version == policy.policy_version,
        CustomError::InvalidVictimReliefPolicyVersion
    );
    require!(
        case_id == config.next_case_id,
        CustomError::InvalidVictimReliefCaseId
    );
    require!(
        !is_zero_32(&subject_commitment),
        CustomError::InvalidVictimReliefSubjectCommitment
    );
    require!(
        !is_zero_32(&evidence_root),
        CustomError::InvalidVictimReliefEvidenceRoot
    );
    validate_victim_relief_evidence_count_v1(evidence_count, policy.max_evidence_items)?;
    require!(
        claimed_amount_usdc >= policy.min_claim_amount_usdc
            && claimed_amount_usdc <= policy.max_claim_amount_usdc,
        CustomError::InvalidVictimReliefClaimAmount
    );
    require!(
        claimant_state.active_case_count < policy.max_active_cases_per_claimant,
        CustomError::VictimReliefActiveCaseLimitReached
    );
    if claimant_state.last_submitted_at != 0 {
        let next_allowed = claimant_state
            .last_submitted_at
            .checked_add(policy.submission_cooldown_seconds)
            .ok_or(CustomError::MathOverflow)?;
        require!(
            now >= next_allowed,
            CustomError::VictimReliefSubmissionCooldownActive
        );
    }
    require!(
        config.usdc_mint == usdc_mint && recipient_mint == usdc_mint,
        CustomError::InvalidMint
    );
    require!(
        recipient_owner == claimant_state.claimant,
        CustomError::VictimReliefRecipientMismatch
    );
    Ok(())
}

pub fn record_victim_relief_case_submission_v1(
    config: &mut VictimReliefConfigV1,
    policy: &VictimReliefPolicyV1,
    claimant_state: &mut VictimReliefClaimantStateV1,
    victim_relief_case: &mut VictimReliefCaseV1,
    config_key: Pubkey,
    policy_key: Pubkey,
    claimant: Pubkey,
    recipient_token_account: Pubkey,
    subject_commitment: [u8; 32],
    evidence_root: [u8; 32],
    evidence_count: u32,
    claimed_amount_usdc: u64,
    usdc_mint: Pubkey,
    now: i64,
    case_bump: u8,
) -> Result<()> {
    let case_id = config.next_case_id;
    let evidence_deadline = now
        .checked_add(policy.evidence_window_seconds)
        .ok_or(CustomError::MathOverflow)?;

    victim_relief_case.case_id = case_id;
    victim_relief_case.config = config_key;
    victim_relief_case.policy = policy_key;
    victim_relief_case.policy_version = policy.policy_version;
    victim_relief_case.claimant = claimant;
    victim_relief_case.subject_commitment = subject_commitment;
    victim_relief_case.evidence_root = evidence_root;
    victim_relief_case.evidence_count = evidence_count;
    victim_relief_case.evidence_revision = 0;
    victim_relief_case.claimed_amount_usdc = claimed_amount_usdc;
    victim_relief_case.approved_amount_usdc = 0;
    victim_relief_case.recipient_owner = claimant;
    victim_relief_case.recipient_token_account = recipient_token_account;
    victim_relief_case.usdc_mint = usdc_mint;
    victim_relief_case.status = VictimReliefCaseStatusV1::EvidencePeriod;
    victim_relief_case.active_appeal = Pubkey::default();
    victim_relief_case.decision_proposal = Pubkey::default();
    victim_relief_case.decision_queue = Pubkey::default();
    victim_relief_case.submitted_at = now;
    victim_relief_case.evidence_deadline = evidence_deadline;
    victim_relief_case.review_deadline = 0;
    victim_relief_case.appeal_deadline = 0;
    victim_relief_case.updated_at = now;
    victim_relief_case.schema_version = VICTIM_RELIEF_SCHEMA_VERSION_V1;
    victim_relief_case.bump = case_bump;
    victim_relief_case.reserved = [0; 64];

    config.next_case_id = config
        .next_case_id
        .checked_add(1)
        .ok_or(CustomError::MathOverflow)?;
    update_victim_relief_claimant_state_on_submit_v1(claimant_state, case_id, now)?;

    Ok(())
}

pub fn update_victim_relief_claimant_state_on_submit_v1(
    claimant_state: &mut VictimReliefClaimantStateV1,
    case_id: u64,
    now: i64,
) -> Result<()> {
    claimant_state.active_case_count = claimant_state
        .active_case_count
        .checked_add(1)
        .ok_or(CustomError::MathOverflow)?;
    claimant_state.total_case_count = claimant_state
        .total_case_count
        .checked_add(1)
        .ok_or(CustomError::MathOverflow)?;
    claimant_state.last_case_id = case_id;
    claimant_state.last_submitted_at = now;
    claimant_state.updated_at = now;
    Ok(())
}

pub fn close_victim_relief_active_case_count_v1(
    claimant_state: &mut VictimReliefClaimantStateV1,
    now: i64,
) -> Result<()> {
    claimant_state.active_case_count = claimant_state
        .active_case_count
        .checked_sub(1)
        .ok_or(CustomError::VictimReliefActiveCaseCountUnderflow)?;
    claimant_state.updated_at = now;
    Ok(())
}

pub fn validate_victim_relief_evidence_update_v1(
    config: &VictimReliefConfigV1,
    policy: &VictimReliefPolicyV1,
    victim_relief_case: &VictimReliefCaseV1,
    config_key: Pubkey,
    policy_key: Pubkey,
    claimant: Pubkey,
    new_evidence_root: [u8; 32],
    new_evidence_count: u32,
    now: i64,
) -> Result<()> {
    require!(
        config.schema_version == VICTIM_RELIEF_SCHEMA_VERSION_V1,
        CustomError::InvalidVictimReliefConfig
    );
    require!(
        victim_relief_case.claimant == claimant,
        CustomError::VictimReliefClaimantMismatch
    );
    require!(
        victim_relief_case.config == config_key,
        CustomError::InvalidVictimReliefConfig
    );
    require!(
        victim_relief_case.policy == policy_key
            && victim_relief_case.policy_version == policy.policy_version,
        CustomError::InvalidVictimReliefPolicyVersion
    );
    require!(
        victim_relief_case.status == VictimReliefCaseStatusV1::EvidencePeriod,
        CustomError::VictimReliefCaseStatusMismatch
    );
    require!(
        now <= victim_relief_case.evidence_deadline,
        CustomError::VictimReliefEvidenceWindowClosed
    );
    require!(
        !is_zero_32(&new_evidence_root),
        CustomError::InvalidVictimReliefEvidenceRoot
    );
    validate_victim_relief_evidence_count_v1(new_evidence_count, policy.max_evidence_items)?;
    require!(
        new_evidence_root != victim_relief_case.evidence_root
            || new_evidence_count != victim_relief_case.evidence_count,
        CustomError::VictimReliefEvidenceUnchanged
    );
    Ok(())
}

pub fn record_victim_relief_evidence_update_v1(
    victim_relief_case: &mut VictimReliefCaseV1,
    new_evidence_root: [u8; 32],
    new_evidence_count: u32,
    now: i64,
) -> Result<()> {
    victim_relief_case.evidence_root = new_evidence_root;
    victim_relief_case.evidence_count = new_evidence_count;
    victim_relief_case.evidence_revision = victim_relief_case
        .evidence_revision
        .checked_add(1)
        .ok_or(CustomError::MathOverflow)?;
    victim_relief_case.updated_at = now;
    Ok(())
}

pub fn validate_victim_relief_case_claimant_link_v1(
    config: &VictimReliefConfigV1,
    claimant_state: &VictimReliefClaimantStateV1,
    victim_relief_case: &VictimReliefCaseV1,
    config_key: Pubkey,
    claimant: Pubkey,
) -> Result<()> {
    require!(
        config.schema_version == VICTIM_RELIEF_SCHEMA_VERSION_V1,
        CustomError::InvalidVictimReliefConfig
    );
    require!(
        victim_relief_case.config == config_key,
        CustomError::InvalidVictimReliefConfig
    );
    require!(
        victim_relief_case.claimant == claimant
            && claimant_state.claimant == claimant
            && claimant_state.config == config_key,
        CustomError::VictimReliefClaimantMismatch
    );
    Ok(())
}

pub fn record_victim_relief_case_terminal_status_v1(
    claimant_state: &mut VictimReliefClaimantStateV1,
    victim_relief_case: &mut VictimReliefCaseV1,
    status: VictimReliefCaseStatusV1,
    now: i64,
) -> Result<()> {
    require!(
        status == VictimReliefCaseStatusV1::Cancelled
            || status == VictimReliefCaseStatusV1::Expired,
        CustomError::VictimReliefCaseStatusMismatch
    );
    close_victim_relief_active_case_count_v1(claimant_state, now)?;
    victim_relief_case.status = status;
    victim_relief_case.updated_at = now;
    Ok(())
}

fn validate_victim_relief_evidence_count_v1(
    evidence_count: u32,
    max_evidence_items: u32,
) -> Result<()> {
    require!(
        evidence_count > 0 && evidence_count <= max_evidence_items,
        CustomError::InvalidVictimReliefEvidenceCount
    );
    Ok(())
}

fn is_zero_32(value: &[u8; 32]) -> bool {
    value.iter().all(|byte| *byte == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy_parameters() -> VictimReliefPolicyParametersV1 {
        VictimReliefPolicyParametersV1 {
            min_claim_amount_usdc: 1_000_000,
            max_claim_amount_usdc: 1_000_000_000,
            max_payout_per_case_usdc: 500_000_000,
            evidence_window_seconds: 86_400,
            review_window_seconds: 172_800,
            appeal_window_seconds: 259_200,
            submission_cooldown_seconds: 3_600,
            max_evidence_items: 32,
            max_active_cases_per_claimant: 2,
        }
    }

    fn config_fixture(policy_key: Pubkey) -> VictimReliefConfigV1 {
        VictimReliefConfigV1 {
            authority: Pubkey::new_unique(),
            treasury_config: Pubkey::new_unique(),
            security_governance_config: Pubkey::new_unique(),
            usdc_mint: Pubkey::new_unique(),
            current_policy: policy_key,
            current_policy_version: VICTIM_RELIEF_POLICY_VERSION_V1,
            next_case_id: 1,
            paused: false,
            created_at: 100,
            schema_version: VICTIM_RELIEF_SCHEMA_VERSION_V1,
            bump: 1,
            reserved: [0; 32],
        }
    }

    fn policy_fixture(config_key: Pubkey) -> VictimReliefPolicyV1 {
        let params = policy_parameters();
        VictimReliefPolicyV1 {
            config: config_key,
            policy_version: VICTIM_RELIEF_POLICY_VERSION_V1,
            min_claim_amount_usdc: params.min_claim_amount_usdc,
            max_claim_amount_usdc: params.max_claim_amount_usdc,
            max_payout_per_case_usdc: params.max_payout_per_case_usdc,
            evidence_window_seconds: params.evidence_window_seconds,
            review_window_seconds: params.review_window_seconds,
            appeal_window_seconds: params.appeal_window_seconds,
            submission_cooldown_seconds: params.submission_cooldown_seconds,
            max_evidence_items: params.max_evidence_items,
            max_active_cases_per_claimant: params.max_active_cases_per_claimant,
            active: true,
            initialized_by: Pubkey::new_unique(),
            created_at: 100,
            schema_version: VICTIM_RELIEF_SCHEMA_VERSION_V1,
            bump: 2,
            reserved: [0; 32],
        }
    }

    fn claimant_state_fixture(config: Pubkey, claimant: Pubkey) -> VictimReliefClaimantStateV1 {
        VictimReliefClaimantStateV1 {
            config,
            claimant,
            active_case_count: 0,
            total_case_count: 0,
            last_case_id: 0,
            last_submitted_at: 0,
            created_at: 100,
            updated_at: 100,
            schema_version: VICTIM_RELIEF_SCHEMA_VERSION_V1,
            bump: 3,
        }
    }

    fn case_fixture(
        config: Pubkey,
        policy: Pubkey,
        claimant: Pubkey,
        usdc_mint: Pubkey,
    ) -> VictimReliefCaseV1 {
        VictimReliefCaseV1 {
            case_id: 1,
            config,
            policy,
            policy_version: VICTIM_RELIEF_POLICY_VERSION_V1,
            claimant,
            subject_commitment: [11; 32],
            evidence_root: [22; 32],
            evidence_count: 1,
            evidence_revision: 0,
            claimed_amount_usdc: 10_000_000,
            approved_amount_usdc: 0,
            recipient_owner: claimant,
            recipient_token_account: Pubkey::new_unique(),
            usdc_mint,
            status: VictimReliefCaseStatusV1::EvidencePeriod,
            active_appeal: Pubkey::default(),
            decision_proposal: Pubkey::default(),
            decision_queue: Pubkey::default(),
            submitted_at: 1_000,
            evidence_deadline: 2_000,
            review_deadline: 0,
            appeal_deadline: 0,
            updated_at: 1_000,
            schema_version: VICTIM_RELIEF_SCHEMA_VERSION_V1,
            bump: 4,
            reserved: [0; 64],
        }
    }

    #[test]
    fn validates_policy_parameters() {
        assert!(validate_victim_relief_policy_parameters_v1(policy_parameters()).is_ok());

        let mut invalid = policy_parameters();
        invalid.min_claim_amount_usdc = 0;
        assert_eq!(
            validate_victim_relief_policy_parameters_v1(invalid).unwrap_err(),
            CustomError::InvalidVictimReliefPolicy.into()
        );

        let mut invalid = policy_parameters();
        invalid.max_payout_per_case_usdc = invalid.max_claim_amount_usdc + 1;
        assert_eq!(
            validate_victim_relief_policy_parameters_v1(invalid).unwrap_err(),
            CustomError::InvalidVictimReliefPolicy.into()
        );

        let mut invalid = policy_parameters();
        invalid.evidence_window_seconds = 0;
        assert_eq!(
            validate_victim_relief_policy_parameters_v1(invalid).unwrap_err(),
            CustomError::InvalidVictimReliefPolicy.into()
        );
    }

    #[test]
    fn records_config_defaults() {
        let mut config = VictimReliefConfigV1 {
            authority: Pubkey::default(),
            treasury_config: Pubkey::default(),
            security_governance_config: Pubkey::default(),
            usdc_mint: Pubkey::default(),
            current_policy: Pubkey::default(),
            current_policy_version: 9,
            next_case_id: 9,
            paused: true,
            created_at: 0,
            schema_version: 0,
            bump: 0,
            reserved: [9; 32],
        };
        let authority = Pubkey::new_unique();
        let treasury = Pubkey::new_unique();
        let security = Pubkey::new_unique();
        let mint = Pubkey::new_unique();

        record_victim_relief_config_init_v1(
            &mut config,
            authority,
            treasury,
            security,
            mint,
            77,
            4,
        )
        .unwrap();

        assert_eq!(config.authority, authority);
        assert_eq!(config.treasury_config, treasury);
        assert_eq!(config.security_governance_config, security);
        assert_eq!(config.usdc_mint, mint);
        assert_eq!(config.current_policy, Pubkey::default());
        assert_eq!(config.current_policy_version, 0);
        assert_eq!(config.next_case_id, 1);
        assert!(!config.paused);
        assert_eq!(config.schema_version, VICTIM_RELIEF_SCHEMA_VERSION_V1);
        assert_eq!(config.bump, 4);
    }

    #[test]
    fn records_policy_once_and_updates_config_current_policy() {
        let mut config = config_fixture(Pubkey::default());
        config.current_policy_version = 0;
        let mut policy = policy_fixture(Pubkey::new_unique());
        let config_key = Pubkey::new_unique();
        let policy_key = Pubkey::new_unique();
        let authority = config.authority;

        record_victim_relief_policy_init_v1(
            &mut config,
            &mut policy,
            config_key,
            policy_key,
            authority,
            policy_parameters(),
            111,
            5,
        )
        .unwrap();

        assert_eq!(policy.policy_version, VICTIM_RELIEF_POLICY_VERSION_V1);
        assert!(policy.active);
        assert_eq!(policy.initialized_by, authority);
        assert_eq!(policy.config, config_key);
        assert_eq!(config.current_policy, policy_key);
        assert_eq!(
            config.current_policy_version,
            VICTIM_RELIEF_POLICY_VERSION_V1
        );

        let err = record_victim_relief_policy_init_v1(
            &mut config,
            &mut policy,
            config_key,
            policy_key,
            authority,
            policy_parameters(),
            111,
            5,
        )
        .unwrap_err();
        assert_eq!(
            err,
            CustomError::VictimReliefPolicyAlreadyInitialized.into()
        );
    }

    #[test]
    fn initializes_claimant_state_once() {
        let config = Pubkey::new_unique();
        let claimant = Pubkey::new_unique();
        let mut state = VictimReliefClaimantStateV1 {
            config: Pubkey::default(),
            claimant: Pubkey::default(),
            active_case_count: 0,
            total_case_count: 0,
            last_case_id: 0,
            last_submitted_at: 0,
            created_at: 0,
            updated_at: 0,
            schema_version: 0,
            bump: 0,
        };

        ensure_victim_relief_claimant_state_v1(&mut state, config, claimant, 123, 7).unwrap();
        assert_eq!(state.config, config);
        assert_eq!(state.claimant, claimant);
        assert_eq!(state.created_at, 123);
        assert_eq!(state.bump, 7);

        ensure_victim_relief_claimant_state_v1(&mut state, config, claimant, 999, 8).unwrap();
        assert_eq!(state.created_at, 123);
        assert_eq!(state.bump, 7);
    }

    #[test]
    fn submit_case_updates_case_config_and_claimant_state() {
        let config_key = Pubkey::new_unique();
        let policy_key = Pubkey::new_unique();
        let claimant = Pubkey::new_unique();
        let usdc_mint = Pubkey::new_unique();
        let recipient = Pubkey::new_unique();
        let mut config = config_fixture(policy_key);
        config.usdc_mint = usdc_mint;
        let policy = policy_fixture(config_key);
        let mut claimant_state = claimant_state_fixture(config_key, claimant);
        let mut case = case_fixture(config_key, policy_key, claimant, usdc_mint);

        validate_victim_relief_case_submission_v1(
            &config,
            &policy,
            &claimant_state,
            policy_key,
            1,
            [1; 32],
            [2; 32],
            2,
            10_000_000,
            claimant,
            usdc_mint,
            usdc_mint,
            200,
        )
        .unwrap();

        record_victim_relief_case_submission_v1(
            &mut config,
            &policy,
            &mut claimant_state,
            &mut case,
            config_key,
            policy_key,
            claimant,
            recipient,
            [1; 32],
            [2; 32],
            2,
            10_000_000,
            usdc_mint,
            200,
            9,
        )
        .unwrap();

        assert_eq!(config.next_case_id, 2);
        assert_eq!(claimant_state.active_case_count, 1);
        assert_eq!(claimant_state.total_case_count, 1);
        assert_eq!(claimant_state.last_case_id, 1);
        assert_eq!(case.status, VictimReliefCaseStatusV1::EvidencePeriod);
        assert_eq!(case.evidence_deadline, 200 + policy.evidence_window_seconds);
        assert_eq!(case.recipient_owner, claimant);
        assert_eq!(case.recipient_token_account, recipient);
        assert_eq!(case.claimed_amount_usdc, 10_000_000);
    }

    #[test]
    fn submit_case_validation_rejects_invalid_inputs() {
        let config_key = Pubkey::new_unique();
        let policy_key = Pubkey::new_unique();
        let claimant = Pubkey::new_unique();
        let usdc_mint = Pubkey::new_unique();
        let mut config = config_fixture(policy_key);
        config.usdc_mint = usdc_mint;
        let policy = policy_fixture(config_key);
        let mut claimant_state = claimant_state_fixture(config_key, claimant);

        assert_eq!(
            validate_victim_relief_case_submission_v1(
                &config,
                &policy,
                &claimant_state,
                policy_key,
                2,
                [1; 32],
                [2; 32],
                1,
                10_000_000,
                claimant,
                usdc_mint,
                usdc_mint,
                200,
            )
            .unwrap_err(),
            CustomError::InvalidVictimReliefCaseId.into()
        );

        assert_eq!(
            validate_victim_relief_case_submission_v1(
                &config,
                &policy,
                &claimant_state,
                policy_key,
                1,
                [0; 32],
                [2; 32],
                1,
                10_000_000,
                claimant,
                usdc_mint,
                usdc_mint,
                200,
            )
            .unwrap_err(),
            CustomError::InvalidVictimReliefSubjectCommitment.into()
        );

        assert_eq!(
            validate_victim_relief_case_submission_v1(
                &config,
                &policy,
                &claimant_state,
                policy_key,
                1,
                [1; 32],
                [0; 32],
                1,
                10_000_000,
                claimant,
                usdc_mint,
                usdc_mint,
                200,
            )
            .unwrap_err(),
            CustomError::InvalidVictimReliefEvidenceRoot.into()
        );

        assert_eq!(
            validate_victim_relief_case_submission_v1(
                &config,
                &policy,
                &claimant_state,
                policy_key,
                1,
                [1; 32],
                [2; 32],
                0,
                10_000_000,
                claimant,
                usdc_mint,
                usdc_mint,
                200,
            )
            .unwrap_err(),
            CustomError::InvalidVictimReliefEvidenceCount.into()
        );

        assert_eq!(
            validate_victim_relief_case_submission_v1(
                &config,
                &policy,
                &claimant_state,
                policy_key,
                1,
                [1; 32],
                [2; 32],
                1,
                policy.min_claim_amount_usdc - 1,
                claimant,
                usdc_mint,
                usdc_mint,
                200,
            )
            .unwrap_err(),
            CustomError::InvalidVictimReliefClaimAmount.into()
        );

        assert_eq!(
            validate_victim_relief_case_submission_v1(
                &config,
                &policy,
                &claimant_state,
                policy_key,
                1,
                [1; 32],
                [2; 32],
                1,
                10_000_000,
                Pubkey::new_unique(),
                usdc_mint,
                usdc_mint,
                200,
            )
            .unwrap_err(),
            CustomError::VictimReliefRecipientMismatch.into()
        );

        claimant_state.active_case_count = policy.max_active_cases_per_claimant;
        assert_eq!(
            validate_victim_relief_case_submission_v1(
                &config,
                &policy,
                &claimant_state,
                policy_key,
                1,
                [1; 32],
                [2; 32],
                1,
                10_000_000,
                claimant,
                usdc_mint,
                usdc_mint,
                200,
            )
            .unwrap_err(),
            CustomError::VictimReliefActiveCaseLimitReached.into()
        );

        claimant_state.active_case_count = 0;
        claimant_state.last_submitted_at = 100;
        assert_eq!(
            validate_victim_relief_case_submission_v1(
                &config,
                &policy,
                &claimant_state,
                policy_key,
                1,
                [1; 32],
                [2; 32],
                1,
                10_000_000,
                claimant,
                usdc_mint,
                usdc_mint,
                101,
            )
            .unwrap_err(),
            CustomError::VictimReliefSubmissionCooldownActive.into()
        );

        config.paused = true;
        assert_eq!(
            validate_victim_relief_case_submission_v1(
                &config,
                &policy,
                &claimant_state,
                policy_key,
                1,
                [1; 32],
                [2; 32],
                1,
                10_000_000,
                claimant,
                usdc_mint,
                usdc_mint,
                4_000,
            )
            .unwrap_err(),
            CustomError::VictimReliefPaused.into()
        );
    }

    #[test]
    fn evidence_update_revises_root_only_inside_window() {
        let config_key = Pubkey::new_unique();
        let policy_key = Pubkey::new_unique();
        let claimant = Pubkey::new_unique();
        let usdc_mint = Pubkey::new_unique();
        let config = config_fixture(policy_key);
        let policy = policy_fixture(config_key);
        let mut case = case_fixture(config_key, policy_key, claimant, usdc_mint);

        validate_victim_relief_evidence_update_v1(
            &config,
            &policy,
            &case,
            config_key,
            policy_key,
            claimant,
            [33; 32],
            2,
            case.evidence_deadline,
        )
        .unwrap();
        record_victim_relief_evidence_update_v1(&mut case, [33; 32], 2, 1_500).unwrap();
        assert_eq!(case.evidence_root, [33; 32]);
        assert_eq!(case.evidence_count, 2);
        assert_eq!(case.evidence_revision, 1);

        assert_eq!(
            validate_victim_relief_evidence_update_v1(
                &config, &policy, &case, config_key, policy_key, claimant, [33; 32], 2, 1_600,
            )
            .unwrap_err(),
            CustomError::VictimReliefEvidenceUnchanged.into()
        );

        assert_eq!(
            validate_victim_relief_evidence_update_v1(
                &config,
                &policy,
                &case,
                config_key,
                policy_key,
                claimant,
                [44; 32],
                2,
                case.evidence_deadline + 1,
            )
            .unwrap_err(),
            CustomError::VictimReliefEvidenceWindowClosed.into()
        );
    }

    #[test]
    fn cancel_and_expire_decrement_active_count_once() {
        let config_key = Pubkey::new_unique();
        let policy_key = Pubkey::new_unique();
        let claimant = Pubkey::new_unique();
        let usdc_mint = Pubkey::new_unique();
        let mut claimant_state = claimant_state_fixture(config_key, claimant);
        claimant_state.active_case_count = 1;
        let mut case = case_fixture(config_key, policy_key, claimant, usdc_mint);

        record_victim_relief_case_terminal_status_v1(
            &mut claimant_state,
            &mut case,
            VictimReliefCaseStatusV1::Cancelled,
            2_000,
        )
        .unwrap();
        assert_eq!(claimant_state.active_case_count, 0);
        assert_eq!(case.status, VictimReliefCaseStatusV1::Cancelled);

        assert_eq!(
            record_victim_relief_case_terminal_status_v1(
                &mut claimant_state,
                &mut case,
                VictimReliefCaseStatusV1::Expired,
                2_001,
            )
            .unwrap_err(),
            CustomError::VictimReliefActiveCaseCountUnderflow.into()
        );
    }

    #[test]
    fn active_count_checked_add_and_sub_are_guarded() {
        let config = Pubkey::new_unique();
        let claimant = Pubkey::new_unique();
        let mut state = claimant_state_fixture(config, claimant);
        state.active_case_count = u16::MAX;
        assert_eq!(
            update_victim_relief_claimant_state_on_submit_v1(&mut state, 1, 100).unwrap_err(),
            CustomError::MathOverflow.into()
        );

        state.active_case_count = 0;
        assert_eq!(
            close_victim_relief_active_case_count_v1(&mut state, 100).unwrap_err(),
            CustomError::VictimReliefActiveCaseCountUnderflow.into()
        );
    }

    #[test]
    fn victim_relief_pdas_are_stable() {
        let (config_pda, _) =
            Pubkey::find_program_address(&[VICTIM_RELIEF_CONFIG_V1_SEED], &crate::ID);
        let (policy_pda, _) = Pubkey::find_program_address(
            &[
                VICTIM_RELIEF_POLICY_V1_SEED,
                config_pda.as_ref(),
                &VICTIM_RELIEF_POLICY_VERSION_V1.to_le_bytes(),
            ],
            &crate::ID,
        );
        let claimant = Pubkey::new_unique();
        let (claimant_state_pda, _) = Pubkey::find_program_address(
            &[
                VICTIM_RELIEF_CLAIMANT_STATE_V1_SEED,
                config_pda.as_ref(),
                claimant.as_ref(),
            ],
            &crate::ID,
        );
        let (case_pda, _) = Pubkey::find_program_address(
            &[
                VICTIM_RELIEF_CASE_V1_SEED,
                config_pda.as_ref(),
                &1u64.to_le_bytes(),
            ],
            &crate::ID,
        );

        assert_ne!(config_pda, Pubkey::default());
        assert_ne!(policy_pda, Pubkey::default());
        assert_ne!(claimant_state_pda, Pubkey::default());
        assert_ne!(case_pda, Pubkey::default());
    }
}
