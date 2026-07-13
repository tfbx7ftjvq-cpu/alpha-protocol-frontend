use anchor_lang::prelude::*;

use crate::constants::{
    BUILDER_PAYOUT_REQUEST_V1_SEED, CONTRIBUTOR_MILESTONE_DESCRIPTION_MAX_BYTES,
    CONTRIBUTOR_MILESTONE_TITLE_MAX_BYTES, CONTRIBUTOR_MILESTONE_V1_SEED,
    CONTRIBUTOR_REGISTRY_V1_SEED, EXECUTION_QUEUE_ITEM_V1_SEED, GOVERNANCE_CONFIG_V1_SEED,
    PROPOSAL_DECISION_V1_SEED,
};
use crate::error::CustomError;
use crate::state::{
    ActionType, AddContributorPayloadV1, ApproveBuilderPayoutPayloadV1,
    ApproveContributorMilestonePayloadV1, BuilderPayoutRequestV1, ContributorMilestoneV1,
    ContributorRegistryV1, ContributorRoleV1, ContributorStatusV1, ExecutionQueueItemV1,
    ExecutionStatus, GovernanceConfigV1, MilestoneStatusV1, PayoutStatusV1, ProposalDecision,
    ProposalDecisionV1, ProposalType, RemoveContributorPayloadV1, UpdateContributorRolePayloadV1,
};

#[derive(Accounts)]
pub struct InitializeContributorRegistryV1<'info> {
    #[account(
        init,
        payer = contributor_wallet,
        space = 8 + ContributorRegistryV1::INIT_SPACE,
        seeds = [CONTRIBUTOR_REGISTRY_V1_SEED, contributor_wallet.key().as_ref()],
        bump
    )]
    pub contributor_registry: Account<'info, ContributorRegistryV1>,

    #[account(mut)]
    pub contributor_wallet: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(milestone_id: u64)]
pub struct InitializeContributorMilestoneV1<'info> {
    #[account(
        mut,
        seeds = [CONTRIBUTOR_REGISTRY_V1_SEED, contributor_wallet.key().as_ref()],
        bump = contributor_registry.bump,
        constraint = contributor_registry.wallet == contributor_wallet.key() @ CustomError::UnauthorizedContributorWallet,
        constraint = contributor_registry.status == ContributorStatusV1::Active @ CustomError::InvalidContributorStatus
    )]
    pub contributor_registry: Account<'info, ContributorRegistryV1>,

    #[account(
        init,
        payer = contributor_wallet,
        space = 8 + ContributorMilestoneV1::INIT_SPACE,
        seeds = [
            CONTRIBUTOR_MILESTONE_V1_SEED,
            contributor_registry.key().as_ref(),
            &milestone_id.to_le_bytes()
        ],
        bump
    )]
    pub contributor_milestone: Account<'info, ContributorMilestoneV1>,

    #[account(mut)]
    pub contributor_wallet: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(milestone_id: u64)]
pub struct InitializeBuilderPayoutRequestV1<'info> {
    #[account(
        mut,
        seeds = [CONTRIBUTOR_REGISTRY_V1_SEED, contributor_wallet.key().as_ref()],
        bump = contributor_registry.bump,
        constraint = contributor_registry.wallet == contributor_wallet.key() @ CustomError::UnauthorizedContributorWallet,
        constraint = contributor_registry.status == ContributorStatusV1::Active @ CustomError::InvalidContributorStatus
    )]
    pub contributor_registry: Account<'info, ContributorRegistryV1>,

    #[account(
        seeds = [
            CONTRIBUTOR_MILESTONE_V1_SEED,
            contributor_registry.key().as_ref(),
            &milestone_id.to_le_bytes()
        ],
        bump = contributor_milestone.bump,
        constraint = contributor_milestone.contributor == contributor_registry.key() @ CustomError::InvalidContributorMilestone
    )]
    pub contributor_milestone: Account<'info, ContributorMilestoneV1>,

    #[account(
        init,
        payer = contributor_wallet,
        space = 8 + BuilderPayoutRequestV1::INIT_SPACE,
        seeds = [
            BUILDER_PAYOUT_REQUEST_V1_SEED,
            contributor_registry.key().as_ref(),
            contributor_milestone.key().as_ref()
        ],
        bump
    )]
    pub builder_payout_request: Account<'info, BuilderPayoutRequestV1>,

    #[account(mut)]
    pub contributor_wallet: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct ExecuteAddContributor<'info> {
    #[account(
        seeds = [GOVERNANCE_CONFIG_V1_SEED],
        bump = governance_config.bump
    )]
    pub governance_config: Account<'info, GovernanceConfigV1>,

    #[account(
        seeds = [
            PROPOSAL_DECISION_V1_SEED,
            &proposal_id.to_le_bytes()
        ],
        bump = proposal_decision.bump,
        constraint = proposal_decision.proposal_id == proposal_id @ CustomError::InvalidProposalId
    )]
    pub proposal_decision: Account<'info, ProposalDecisionV1>,

    #[account(
        seeds = [
            EXECUTION_QUEUE_ITEM_V1_SEED,
            &proposal_id.to_le_bytes()
        ],
        bump = execution_queue_item.bump,
        constraint = execution_queue_item.proposal_id == proposal_id @ CustomError::InvalidProposalId
    )]
    pub execution_queue_item: Account<'info, ExecutionQueueItemV1>,

    #[account(
        init_if_needed,
        payer = executor,
        space = 8 + ContributorRegistryV1::INIT_SPACE,
        seeds = [CONTRIBUTOR_REGISTRY_V1_SEED, contributor_wallet.key().as_ref()],
        bump
    )]
    pub contributor_registry: Account<'info, ContributorRegistryV1>,

    /// CHECK: The contributor wallet is recorded in the governance payload.
    pub contributor_wallet: UncheckedAccount<'info>,

    #[account(mut)]
    pub executor: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct ExecuteRemoveContributor<'info> {
    #[account(
        seeds = [GOVERNANCE_CONFIG_V1_SEED],
        bump = governance_config.bump
    )]
    pub governance_config: Account<'info, GovernanceConfigV1>,

    #[account(
        seeds = [
            PROPOSAL_DECISION_V1_SEED,
            &proposal_id.to_le_bytes()
        ],
        bump = proposal_decision.bump,
        constraint = proposal_decision.proposal_id == proposal_id @ CustomError::InvalidProposalId
    )]
    pub proposal_decision: Account<'info, ProposalDecisionV1>,

    #[account(
        seeds = [
            EXECUTION_QUEUE_ITEM_V1_SEED,
            &proposal_id.to_le_bytes()
        ],
        bump = execution_queue_item.bump,
        constraint = execution_queue_item.proposal_id == proposal_id @ CustomError::InvalidProposalId
    )]
    pub execution_queue_item: Account<'info, ExecutionQueueItemV1>,

    #[account(
        mut,
        seeds = [CONTRIBUTOR_REGISTRY_V1_SEED, contributor_registry.wallet.as_ref()],
        bump = contributor_registry.bump
    )]
    pub contributor_registry: Account<'info, ContributorRegistryV1>,

    pub executor: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct ExecuteUpdateContributorRole<'info> {
    #[account(
        seeds = [GOVERNANCE_CONFIG_V1_SEED],
        bump = governance_config.bump
    )]
    pub governance_config: Account<'info, GovernanceConfigV1>,

    #[account(
        seeds = [
            PROPOSAL_DECISION_V1_SEED,
            &proposal_id.to_le_bytes()
        ],
        bump = proposal_decision.bump,
        constraint = proposal_decision.proposal_id == proposal_id @ CustomError::InvalidProposalId
    )]
    pub proposal_decision: Account<'info, ProposalDecisionV1>,

    #[account(
        seeds = [
            EXECUTION_QUEUE_ITEM_V1_SEED,
            &proposal_id.to_le_bytes()
        ],
        bump = execution_queue_item.bump,
        constraint = execution_queue_item.proposal_id == proposal_id @ CustomError::InvalidProposalId
    )]
    pub execution_queue_item: Account<'info, ExecutionQueueItemV1>,

    #[account(
        mut,
        seeds = [CONTRIBUTOR_REGISTRY_V1_SEED, contributor_registry.wallet.as_ref()],
        bump = contributor_registry.bump
    )]
    pub contributor_registry: Account<'info, ContributorRegistryV1>,

    pub executor: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(proposal_id: u64, milestone_id: u64)]
pub struct ExecuteApproveContributorMilestone<'info> {
    #[account(
        seeds = [GOVERNANCE_CONFIG_V1_SEED],
        bump = governance_config.bump
    )]
    pub governance_config: Account<'info, GovernanceConfigV1>,

    #[account(
        seeds = [
            PROPOSAL_DECISION_V1_SEED,
            &proposal_id.to_le_bytes()
        ],
        bump = proposal_decision.bump,
        constraint = proposal_decision.proposal_id == proposal_id @ CustomError::InvalidProposalId
    )]
    pub proposal_decision: Account<'info, ProposalDecisionV1>,

    #[account(
        seeds = [
            EXECUTION_QUEUE_ITEM_V1_SEED,
            &proposal_id.to_le_bytes()
        ],
        bump = execution_queue_item.bump,
        constraint = execution_queue_item.proposal_id == proposal_id @ CustomError::InvalidProposalId
    )]
    pub execution_queue_item: Account<'info, ExecutionQueueItemV1>,

    #[account(
        mut,
        seeds = [CONTRIBUTOR_REGISTRY_V1_SEED, contributor_registry.wallet.as_ref()],
        bump = contributor_registry.bump
    )]
    pub contributor_registry: Account<'info, ContributorRegistryV1>,

    #[account(
        mut,
        seeds = [
            CONTRIBUTOR_MILESTONE_V1_SEED,
            contributor_registry.key().as_ref(),
            &milestone_id.to_le_bytes()
        ],
        bump = contributor_milestone.bump,
        constraint = contributor_milestone.contributor == contributor_registry.key() @ CustomError::InvalidContributorMilestone
    )]
    pub contributor_milestone: Account<'info, ContributorMilestoneV1>,

    pub executor: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(proposal_id: u64, milestone_id: u64)]
pub struct ExecuteApproveBuilderPayout<'info> {
    #[account(
        seeds = [GOVERNANCE_CONFIG_V1_SEED],
        bump = governance_config.bump
    )]
    pub governance_config: Account<'info, GovernanceConfigV1>,

    #[account(
        seeds = [
            PROPOSAL_DECISION_V1_SEED,
            &proposal_id.to_le_bytes()
        ],
        bump = proposal_decision.bump,
        constraint = proposal_decision.proposal_id == proposal_id @ CustomError::InvalidProposalId
    )]
    pub proposal_decision: Account<'info, ProposalDecisionV1>,

    #[account(
        seeds = [
            EXECUTION_QUEUE_ITEM_V1_SEED,
            &proposal_id.to_le_bytes()
        ],
        bump = execution_queue_item.bump,
        constraint = execution_queue_item.proposal_id == proposal_id @ CustomError::InvalidProposalId
    )]
    pub execution_queue_item: Account<'info, ExecutionQueueItemV1>,

    #[account(
        mut,
        seeds = [CONTRIBUTOR_REGISTRY_V1_SEED, contributor_registry.wallet.as_ref()],
        bump = contributor_registry.bump
    )]
    pub contributor_registry: Account<'info, ContributorRegistryV1>,

    #[account(
        seeds = [
            CONTRIBUTOR_MILESTONE_V1_SEED,
            contributor_registry.key().as_ref(),
            &milestone_id.to_le_bytes()
        ],
        bump = contributor_milestone.bump,
        constraint = contributor_milestone.contributor == contributor_registry.key() @ CustomError::InvalidContributorMilestone
    )]
    pub contributor_milestone: Account<'info, ContributorMilestoneV1>,

    #[account(
        mut,
        seeds = [
            BUILDER_PAYOUT_REQUEST_V1_SEED,
            contributor_registry.key().as_ref(),
            contributor_milestone.key().as_ref()
        ],
        bump = builder_payout_request.bump,
        constraint = builder_payout_request.contributor == contributor_registry.key() @ CustomError::InvalidContributorPayoutRequest,
        constraint = builder_payout_request.milestone == contributor_milestone.key() @ CustomError::InvalidContributorPayoutRequest
    )]
    pub builder_payout_request: Account<'info, BuilderPayoutRequestV1>,

    pub executor: Signer<'info>,
}

pub fn initialize_contributor_registry_v1_handler(
    ctx: Context<InitializeContributorRegistryV1>,
    role: ContributorRoleV1,
) -> Result<()> {
    validate_contributor_role(role)?;

    let now = Clock::get()?.unix_timestamp;
    let contributor_registry = &mut ctx.accounts.contributor_registry;
    contributor_registry.wallet = ctx.accounts.contributor_wallet.key();
    contributor_registry.role = role;
    contributor_registry.status = ContributorStatusV1::Suspended;
    contributor_registry.joined_at = now;
    contributor_registry.last_active_at = now;
    contributor_registry.completed_milestones = 0;
    contributor_registry.approved_payout_count = 0;
    contributor_registry.reputation_score = 0;
    contributor_registry.bump = ctx.bumps.contributor_registry;

    Ok(())
}

pub fn execute_add_contributor_handler(
    ctx: Context<ExecuteAddContributor>,
    proposal_id: u64,
    contributor_role: ContributorRoleV1,
) -> Result<()> {
    let payload = AddContributorPayloadV1 {
        contributor_wallet: ctx.accounts.contributor_wallet.key(),
        contributor_role,
    };
    let payload_hash = hash_contributor_payload(&payload)?;

    validate_contributor_security_execution(
        &ctx.accounts.governance_config,
        &ctx.accounts.proposal_decision,
        &ctx.accounts.execution_queue_item,
        proposal_id,
        ProposalType::ContributorAddContributor,
        ActionType::ContributorAddContributor,
        ctx.accounts.contributor_registry.key(),
        payload_hash,
    )?;

    let now = Clock::get()?.unix_timestamp;
    record_add_contributor_execution(
        &mut ctx.accounts.contributor_registry,
        ctx.accounts.contributor_wallet.key(),
        contributor_role,
        ctx.bumps.contributor_registry,
        now,
    )
}

pub fn execute_remove_contributor_handler(
    ctx: Context<ExecuteRemoveContributor>,
    proposal_id: u64,
    reason_hash: [u8; 32],
) -> Result<()> {
    let payload = RemoveContributorPayloadV1 {
        contributor_registry: ctx.accounts.contributor_registry.key(),
        reason_hash,
    };
    let payload_hash = hash_contributor_payload(&payload)?;

    validate_contributor_security_execution(
        &ctx.accounts.governance_config,
        &ctx.accounts.proposal_decision,
        &ctx.accounts.execution_queue_item,
        proposal_id,
        ProposalType::ContributorRemoveContributor,
        ActionType::ContributorRemoveContributor,
        ctx.accounts.contributor_registry.key(),
        payload_hash,
    )?;

    let now = Clock::get()?.unix_timestamp;
    record_remove_contributor_execution(&mut ctx.accounts.contributor_registry, now)
}

pub fn execute_update_contributor_role_handler(
    ctx: Context<ExecuteUpdateContributorRole>,
    proposal_id: u64,
    new_role: ContributorRoleV1,
) -> Result<()> {
    let payload = UpdateContributorRolePayloadV1 {
        contributor_registry: ctx.accounts.contributor_registry.key(),
        new_role,
    };
    let payload_hash = hash_contributor_payload(&payload)?;

    validate_contributor_security_execution(
        &ctx.accounts.governance_config,
        &ctx.accounts.proposal_decision,
        &ctx.accounts.execution_queue_item,
        proposal_id,
        ProposalType::ContributorUpdateRole,
        ActionType::ContributorUpdateRole,
        ctx.accounts.contributor_registry.key(),
        payload_hash,
    )?;

    let now = Clock::get()?.unix_timestamp;
    record_update_contributor_role_execution(&mut ctx.accounts.contributor_registry, new_role, now)
}

pub fn execute_approve_contributor_milestone_handler(
    ctx: Context<ExecuteApproveContributorMilestone>,
    proposal_id: u64,
    _milestone_id: u64,
    approved_amount: u64,
) -> Result<()> {
    let payload = ApproveContributorMilestonePayloadV1 {
        milestone: ctx.accounts.contributor_milestone.key(),
        approved_amount,
    };
    let payload_hash = hash_contributor_payload(&payload)?;

    validate_contributor_security_execution(
        &ctx.accounts.governance_config,
        &ctx.accounts.proposal_decision,
        &ctx.accounts.execution_queue_item,
        proposal_id,
        ProposalType::ContributorApproveMilestone,
        ActionType::ContributorApproveMilestone,
        ctx.accounts.contributor_milestone.key(),
        payload_hash,
    )?;

    let now = Clock::get()?.unix_timestamp;
    let contributor_registry_key = ctx.accounts.contributor_registry.key();
    record_approve_contributor_milestone_execution(
        &mut ctx.accounts.contributor_registry,
        contributor_registry_key,
        &mut ctx.accounts.contributor_milestone,
        approved_amount,
        now,
    )
}

pub fn execute_approve_builder_payout_handler(
    ctx: Context<ExecuteApproveBuilderPayout>,
    proposal_id: u64,
    _milestone_id: u64,
    approved_amount: u64,
) -> Result<()> {
    let payload = ApproveBuilderPayoutPayloadV1 {
        payout_request: ctx.accounts.builder_payout_request.key(),
        approved_amount,
    };
    let payload_hash = hash_contributor_payload(&payload)?;

    validate_contributor_security_execution(
        &ctx.accounts.governance_config,
        &ctx.accounts.proposal_decision,
        &ctx.accounts.execution_queue_item,
        proposal_id,
        ProposalType::ContributorApproveBuilderPayout,
        ActionType::ContributorApproveBuilderPayout,
        ctx.accounts.builder_payout_request.key(),
        payload_hash,
    )?;

    let now = Clock::get()?.unix_timestamp;
    let contributor_registry_key = ctx.accounts.contributor_registry.key();
    record_approve_builder_payout_execution(
        &mut ctx.accounts.contributor_registry,
        contributor_registry_key,
        &mut ctx.accounts.builder_payout_request,
        approved_amount,
        now,
    )
}

pub fn initialize_contributor_milestone_v1_handler(
    ctx: Context<InitializeContributorMilestoneV1>,
    _milestone_id: u64,
    title: String,
    description: String,
    evidence_hash: [u8; 32],
    requested_amount: u64,
) -> Result<()> {
    validate_contributor_milestone_input(&title, &description, evidence_hash, requested_amount)?;

    let now = Clock::get()?.unix_timestamp;
    let contributor_milestone = &mut ctx.accounts.contributor_milestone;
    contributor_milestone.contributor = ctx.accounts.contributor_registry.key();
    contributor_milestone.title = title;
    contributor_milestone.description = description;
    contributor_milestone.evidence_hash = evidence_hash;
    contributor_milestone.requested_amount = requested_amount;
    contributor_milestone.status = MilestoneStatusV1::Pending;
    contributor_milestone.created_at = now;
    contributor_milestone.bump = ctx.bumps.contributor_milestone;

    ctx.accounts.contributor_registry.last_active_at = now;

    Ok(())
}

pub fn initialize_builder_payout_request_v1_handler(
    ctx: Context<InitializeBuilderPayoutRequestV1>,
    _milestone_id: u64,
    amount: u64,
    destination_wallet: Pubkey,
) -> Result<()> {
    validate_builder_payout_request_input(
        ctx.accounts.contributor_milestone.status,
        ctx.accounts.contributor_milestone.requested_amount,
        amount,
        destination_wallet,
    )?;

    let now = Clock::get()?.unix_timestamp;
    let builder_payout_request = &mut ctx.accounts.builder_payout_request;
    builder_payout_request.contributor = ctx.accounts.contributor_registry.key();
    builder_payout_request.milestone = ctx.accounts.contributor_milestone.key();
    builder_payout_request.amount = amount;
    builder_payout_request.destination_wallet = destination_wallet;
    builder_payout_request.status = PayoutStatusV1::Pending;
    builder_payout_request.created_at = now;
    builder_payout_request.bump = ctx.bumps.builder_payout_request;

    ctx.accounts.contributor_registry.last_active_at = now;

    Ok(())
}

pub fn validate_contributor_role(role: ContributorRoleV1) -> Result<()> {
    match role {
        ContributorRoleV1::CoreDeveloper
        | ContributorRoleV1::BackendDeveloper
        | ContributorRoleV1::FrontendDeveloper
        | ContributorRoleV1::SecurityResearcher
        | ContributorRoleV1::ProtocolResearcher
        | ContributorRoleV1::Designer
        | ContributorRoleV1::CommunityManager
        | ContributorRoleV1::Operations
        | ContributorRoleV1::TreasuryReviewer
        | ContributorRoleV1::Translator
        | ContributorRoleV1::Ambassador
        | ContributorRoleV1::Other => Ok(()),
    }
}

pub fn validate_contributor_milestone_input(
    title: &str,
    description: &str,
    evidence_hash: [u8; 32],
    requested_amount: u64,
) -> Result<()> {
    require!(
        !title.is_empty() && title.len() <= CONTRIBUTOR_MILESTONE_TITLE_MAX_BYTES,
        CustomError::InvalidContributorMilestoneText
    );
    require!(
        !description.is_empty() && description.len() <= CONTRIBUTOR_MILESTONE_DESCRIPTION_MAX_BYTES,
        CustomError::InvalidContributorMilestoneText
    );
    require!(
        evidence_hash != [0u8; 32],
        CustomError::InvalidContributorMilestone
    );
    require!(
        requested_amount > 0,
        CustomError::InvalidContributorMilestoneAmount
    );

    Ok(())
}

pub fn validate_builder_payout_request_input(
    milestone_status: MilestoneStatusV1,
    milestone_requested_amount: u64,
    amount: u64,
    destination_wallet: Pubkey,
) -> Result<()> {
    require!(
        milestone_status == MilestoneStatusV1::Pending
            || milestone_status == MilestoneStatusV1::Approved,
        CustomError::InvalidContributorMilestone
    );
    require!(amount > 0, CustomError::InvalidContributorPayoutAmount);
    require!(
        amount <= milestone_requested_amount,
        CustomError::InvalidContributorPayoutAmount
    );
    require!(
        destination_wallet != Pubkey::default(),
        CustomError::InvalidContributorPayoutDestination
    );

    Ok(())
}

pub fn hash_contributor_payload<T: AnchorSerialize>(payload: &T) -> Result<[u8; 32]> {
    let mut bytes = Vec::new();
    payload
        .serialize(&mut bytes)
        .map_err(|_| error!(CustomError::InvalidContributorPayloadHash))?;
    Ok(sha256(bytes.as_slice()))
}

fn sha256(data: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut msg = Vec::with_capacity(((data.len() + 9 + 63) / 64) * 64);
    msg.extend_from_slice(data);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            let offset = i * 4;
            w[i] = u32::from_be_bytes([
                chunk[offset],
                chunk[offset + 1],
                chunk[offset + 2],
                chunk[offset + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = [0u8; 32];
    for (i, value) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&value.to_be_bytes());
    }
    out
}

pub fn validate_contributor_security_execution(
    governance_config: &GovernanceConfigV1,
    proposal_decision: &ProposalDecisionV1,
    execution_queue_item: &ExecutionQueueItemV1,
    proposal_id: u64,
    expected_proposal_type: ProposalType,
    expected_action_type: ActionType,
    expected_target_account: Pubkey,
    expected_payload_hash: [u8; 32],
) -> Result<()> {
    require!(
        !governance_config.is_paused,
        CustomError::SecurityLayerPaused
    );
    require!(
        proposal_decision.proposal_id == proposal_id,
        CustomError::InvalidProposalId
    );
    require!(
        proposal_decision.proposal_type == expected_proposal_type,
        CustomError::InvalidActionForProposalType
    );
    require!(
        matches!(
            proposal_decision.decision,
            ProposalDecision::Approved | ProposalDecision::Partial
        ),
        CustomError::ProposalNotApproved
    );
    require!(
        execution_queue_item.proposal_id == proposal_id,
        CustomError::InvalidProposalId
    );
    require!(
        execution_queue_item.status == ExecutionStatus::Executed,
        CustomError::InvalidExecutionStatus
    );
    require!(
        execution_queue_item.action_type == expected_action_type,
        CustomError::InvalidActionForProposalType
    );
    require!(
        execution_queue_item.target_program == crate::ID,
        CustomError::InvalidContributorSecurityExecution
    );
    require!(
        execution_queue_item.target_account == expected_target_account,
        CustomError::InvalidContributorSecurityExecution
    );
    require!(
        execution_queue_item.payload_hash == expected_payload_hash,
        CustomError::PayloadHashMismatch
    );
    require!(
        matches!(
            execution_queue_item.decision,
            ProposalDecision::Approved | ProposalDecision::Partial
        ),
        CustomError::ProposalNotApproved
    );

    Ok(())
}

pub fn record_add_contributor_execution(
    contributor_registry: &mut ContributorRegistryV1,
    contributor_wallet: Pubkey,
    contributor_role: ContributorRoleV1,
    bump: u8,
    now_ts: i64,
) -> Result<()> {
    validate_contributor_role(contributor_role)?;

    let is_new_registry = contributor_registry.wallet == Pubkey::default();
    if !is_new_registry {
        require_keys_eq!(
            contributor_registry.wallet,
            contributor_wallet,
            CustomError::UnauthorizedContributorWallet
        );
        require!(
            contributor_registry.status == ContributorStatusV1::Suspended,
            CustomError::InvalidContributorStatus
        );
    }

    contributor_registry.wallet = contributor_wallet;
    contributor_registry.role = contributor_role;
    contributor_registry.status = ContributorStatusV1::Active;
    if contributor_registry.joined_at == 0 {
        contributor_registry.joined_at = now_ts;
    }
    contributor_registry.last_active_at = now_ts;
    contributor_registry.bump = bump;

    Ok(())
}

pub fn record_remove_contributor_execution(
    contributor_registry: &mut ContributorRegistryV1,
    now_ts: i64,
) -> Result<()> {
    validate_contributor_status_transition(
        contributor_registry.status,
        ContributorStatusV1::Removed,
    )?;
    contributor_registry.status = ContributorStatusV1::Removed;
    contributor_registry.last_active_at = now_ts;
    Ok(())
}

pub fn record_update_contributor_role_execution(
    contributor_registry: &mut ContributorRegistryV1,
    new_role: ContributorRoleV1,
    now_ts: i64,
) -> Result<()> {
    validate_contributor_role(new_role)?;
    require!(
        contributor_registry.status == ContributorStatusV1::Active,
        CustomError::InvalidContributorStatus
    );
    require!(
        contributor_registry.role != new_role,
        CustomError::InvalidContributorRole
    );

    contributor_registry.role = new_role;
    contributor_registry.last_active_at = now_ts;
    Ok(())
}

pub fn record_approve_contributor_milestone_execution(
    contributor_registry: &mut ContributorRegistryV1,
    contributor_registry_key: Pubkey,
    contributor_milestone: &mut ContributorMilestoneV1,
    approved_amount: u64,
    now_ts: i64,
) -> Result<()> {
    require!(
        contributor_registry.status == ContributorStatusV1::Active,
        CustomError::InvalidContributorStatus
    );
    require!(
        contributor_milestone.contributor == contributor_registry_key,
        CustomError::InvalidContributorMilestone
    );
    require!(
        approved_amount > 0 && approved_amount <= contributor_milestone.requested_amount,
        CustomError::InvalidContributorMilestoneAmount
    );
    validate_milestone_status_transition(
        contributor_milestone.status,
        MilestoneStatusV1::Approved,
    )?;

    contributor_milestone.status = MilestoneStatusV1::Approved;
    contributor_registry.completed_milestones =
        checked_increment_u32(contributor_registry.completed_milestones)?;
    contributor_registry.last_active_at = now_ts;
    Ok(())
}

pub fn record_approve_builder_payout_execution(
    contributor_registry: &mut ContributorRegistryV1,
    contributor_registry_key: Pubkey,
    builder_payout_request: &mut BuilderPayoutRequestV1,
    approved_amount: u64,
    now_ts: i64,
) -> Result<()> {
    require!(
        contributor_registry.status == ContributorStatusV1::Active,
        CustomError::InvalidContributorStatus
    );
    require!(
        builder_payout_request.contributor == contributor_registry_key,
        CustomError::InvalidContributorPayoutRequest
    );
    require!(
        approved_amount > 0 && approved_amount == builder_payout_request.amount,
        CustomError::InvalidContributorPayoutAmount
    );
    validate_payout_status_transition(builder_payout_request.status, PayoutStatusV1::Approved)?;

    builder_payout_request.status = PayoutStatusV1::Approved;
    contributor_registry.approved_payout_count =
        checked_increment_u32(contributor_registry.approved_payout_count)?;
    contributor_registry.last_active_at = now_ts;
    Ok(())
}

pub fn validate_contributor_status_transition(
    current: ContributorStatusV1,
    next: ContributorStatusV1,
) -> Result<()> {
    let is_valid = matches!(
        (current, next),
        (ContributorStatusV1::Active, ContributorStatusV1::Suspended)
            | (ContributorStatusV1::Active, ContributorStatusV1::Removed)
            | (ContributorStatusV1::Suspended, ContributorStatusV1::Active)
            | (ContributorStatusV1::Suspended, ContributorStatusV1::Removed)
    );
    require!(is_valid, CustomError::InvalidContributorStatus);
    Ok(())
}

pub fn validate_milestone_status_transition(
    current: MilestoneStatusV1,
    next: MilestoneStatusV1,
) -> Result<()> {
    let is_valid = matches!(
        (current, next),
        (MilestoneStatusV1::Pending, MilestoneStatusV1::Approved)
            | (MilestoneStatusV1::Pending, MilestoneStatusV1::Rejected)
            | (MilestoneStatusV1::Approved, MilestoneStatusV1::Paid)
    );
    require!(is_valid, CustomError::InvalidContributorMilestone);
    Ok(())
}

pub fn validate_payout_status_transition(
    current: PayoutStatusV1,
    next: PayoutStatusV1,
) -> Result<()> {
    let is_valid = matches!(
        (current, next),
        (PayoutStatusV1::Pending, PayoutStatusV1::Approved)
            | (PayoutStatusV1::Pending, PayoutStatusV1::Rejected)
            | (PayoutStatusV1::Approved, PayoutStatusV1::Executed)
    );
    require!(is_valid, CustomError::InvalidContributorPayoutRequest);
    Ok(())
}

pub fn checked_increment_u32(value: u32) -> Result<u32> {
    value.checked_add(1).ok_or(CustomError::MathOverflow.into())
}

pub fn checked_increment_u64(value: u64) -> Result<u64> {
    value.checked_add(1).ok_or(CustomError::MathOverflow.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROPOSAL_ID: u64 = 42;
    const AUTHORITY: Pubkey = Pubkey::new_from_array([1; 32]);
    const GUARDIAN: Pubkey = Pubkey::new_from_array([2; 32]);
    const CONTRIBUTOR_WALLET: Pubkey = Pubkey::new_from_array([3; 32]);
    const CONTRIBUTOR_REGISTRY_KEY: Pubkey = Pubkey::new_from_array([4; 32]);
    const MILESTONE_KEY: Pubkey = Pubkey::new_from_array([5; 32]);
    const PAYOUT_REQUEST_KEY: Pubkey = Pubkey::new_from_array([6; 32]);

    fn nonzero_hash() -> [u8; 32] {
        [7u8; 32]
    }

    fn governance_config() -> GovernanceConfigV1 {
        GovernanceConfigV1 {
            authority: AUTHORITY,
            min_execution_delay_seconds: 60,
            proposal_count: PROPOSAL_ID,
            emergency_guardian: GUARDIAN,
            is_paused: false,
            bump: 1,
        }
    }

    fn proposal_decision(proposal_type: ProposalType) -> ProposalDecisionV1 {
        ProposalDecisionV1 {
            proposal_id: PROPOSAL_ID,
            proposal_type,
            proposer: AUTHORITY,
            decision: ProposalDecision::Approved,
            yes_weight: 1,
            no_weight: 0,
            start_ts: 1,
            end_ts: 2,
            finalized_ts: 3,
            bump: 1,
        }
    }

    fn executed_queue(
        action_type: ActionType,
        target_account: Pubkey,
        payload_hash: [u8; 32],
    ) -> ExecutionQueueItemV1 {
        ExecutionQueueItemV1 {
            proposal_id: PROPOSAL_ID,
            proposer: AUTHORITY,
            action_type,
            target_program: crate::ID,
            target_account,
            decision: ProposalDecision::Approved,
            created_at: 10,
            execute_after: 20,
            executed_at: 21,
            status: ExecutionStatus::Executed,
            payload_hash,
            bump: 1,
        }
    }

    fn default_registry() -> ContributorRegistryV1 {
        ContributorRegistryV1 {
            wallet: Pubkey::default(),
            role: ContributorRoleV1::Other,
            status: ContributorStatusV1::Suspended,
            joined_at: 0,
            last_active_at: 0,
            completed_milestones: 0,
            approved_payout_count: 0,
            reputation_score: 0,
            bump: 0,
        }
    }

    fn active_registry() -> ContributorRegistryV1 {
        ContributorRegistryV1 {
            wallet: CONTRIBUTOR_WALLET,
            role: ContributorRoleV1::BackendDeveloper,
            status: ContributorStatusV1::Active,
            joined_at: 10,
            last_active_at: 10,
            completed_milestones: 0,
            approved_payout_count: 0,
            reputation_score: 0,
            bump: 1,
        }
    }

    fn pending_milestone() -> ContributorMilestoneV1 {
        ContributorMilestoneV1 {
            contributor: CONTRIBUTOR_REGISTRY_KEY,
            title: "router".to_string(),
            description: "ship typed router".to_string(),
            evidence_hash: nonzero_hash(),
            requested_amount: 1_000,
            status: MilestoneStatusV1::Pending,
            created_at: 10,
            bump: 1,
        }
    }

    fn pending_payout_request() -> BuilderPayoutRequestV1 {
        BuilderPayoutRequestV1 {
            contributor: CONTRIBUTOR_REGISTRY_KEY,
            milestone: MILESTONE_KEY,
            amount: 1_000,
            destination_wallet: Pubkey::new_from_array([9; 32]),
            status: PayoutStatusV1::Pending,
            created_at: 10,
            bump: 1,
        }
    }

    #[test]
    fn contributor_registry_space_covers_fields() {
        assert!(ContributorRegistryV1::INIT_SPACE >= 67);
    }

    #[test]
    fn contributor_milestone_space_covers_bounded_strings() {
        let expected = 32
            + 4
            + CONTRIBUTOR_MILESTONE_TITLE_MAX_BYTES
            + 4
            + CONTRIBUTOR_MILESTONE_DESCRIPTION_MAX_BYTES
            + 32
            + 8
            + 1
            + 8
            + 1;
        assert_eq!(ContributorMilestoneV1::INIT_SPACE, expected);
    }

    #[test]
    fn builder_payout_request_space_covers_fields() {
        assert!(BuilderPayoutRequestV1::INIT_SPACE >= 114);
    }

    #[test]
    fn validates_milestone_input() {
        validate_contributor_milestone_input(
            "ship router",
            "implemented typed router",
            nonzero_hash(),
            1,
        )
        .unwrap();
    }

    #[test]
    fn rejects_empty_milestone_title() {
        let err =
            validate_contributor_milestone_input("", "description", nonzero_hash(), 1).unwrap_err();
        assert_eq!(err, CustomError::InvalidContributorMilestoneText.into());
    }

    #[test]
    fn rejects_overlong_milestone_title() {
        let title = "x".repeat(CONTRIBUTOR_MILESTONE_TITLE_MAX_BYTES + 1);
        let err = validate_contributor_milestone_input(&title, "description", nonzero_hash(), 1)
            .unwrap_err();
        assert_eq!(err, CustomError::InvalidContributorMilestoneText.into());
    }

    #[test]
    fn rejects_overlong_milestone_description() {
        let description = "x".repeat(CONTRIBUTOR_MILESTONE_DESCRIPTION_MAX_BYTES + 1);
        let err = validate_contributor_milestone_input("title", &description, nonzero_hash(), 1)
            .unwrap_err();
        assert_eq!(err, CustomError::InvalidContributorMilestoneText.into());
    }

    #[test]
    fn rejects_zero_evidence_hash() {
        let err =
            validate_contributor_milestone_input("title", "description", [0u8; 32], 1).unwrap_err();
        assert_eq!(err, CustomError::InvalidContributorMilestone.into());
    }

    #[test]
    fn rejects_zero_requested_amount() {
        let err = validate_contributor_milestone_input("title", "description", nonzero_hash(), 0)
            .unwrap_err();
        assert_eq!(err, CustomError::InvalidContributorMilestoneAmount.into());
    }

    #[test]
    fn validates_pending_payout_request_for_pending_milestone() {
        validate_builder_payout_request_input(
            MilestoneStatusV1::Pending,
            1_000,
            500,
            Pubkey::new_unique(),
        )
        .unwrap();
    }

    #[test]
    fn validates_pending_payout_request_for_approved_milestone() {
        validate_builder_payout_request_input(
            MilestoneStatusV1::Approved,
            1_000,
            1_000,
            Pubkey::new_unique(),
        )
        .unwrap();
    }

    #[test]
    fn rejects_payout_request_for_rejected_milestone() {
        let err = validate_builder_payout_request_input(
            MilestoneStatusV1::Rejected,
            1_000,
            500,
            Pubkey::new_unique(),
        )
        .unwrap_err();
        assert_eq!(err, CustomError::InvalidContributorMilestone.into());
    }

    #[test]
    fn rejects_payout_above_requested_amount() {
        let err = validate_builder_payout_request_input(
            MilestoneStatusV1::Pending,
            1_000,
            1_001,
            Pubkey::new_unique(),
        )
        .unwrap_err();
        assert_eq!(err, CustomError::InvalidContributorPayoutAmount.into());
    }

    #[test]
    fn rejects_zero_payout_destination() {
        let err = validate_builder_payout_request_input(
            MilestoneStatusV1::Pending,
            1_000,
            500,
            Pubkey::default(),
        )
        .unwrap_err();
        assert_eq!(err, CustomError::InvalidContributorPayoutDestination.into());
    }

    #[test]
    fn contributor_status_allows_active_to_removed() {
        validate_contributor_status_transition(
            ContributorStatusV1::Active,
            ContributorStatusV1::Removed,
        )
        .unwrap();
    }

    #[test]
    fn contributor_status_rejects_removed_to_active() {
        let err = validate_contributor_status_transition(
            ContributorStatusV1::Removed,
            ContributorStatusV1::Active,
        )
        .unwrap_err();
        assert_eq!(err, CustomError::InvalidContributorStatus.into());
    }

    #[test]
    fn milestone_status_allows_pending_to_approved() {
        validate_milestone_status_transition(
            MilestoneStatusV1::Pending,
            MilestoneStatusV1::Approved,
        )
        .unwrap();
    }

    #[test]
    fn milestone_status_rejects_rejected_to_paid() {
        let err = validate_milestone_status_transition(
            MilestoneStatusV1::Rejected,
            MilestoneStatusV1::Paid,
        )
        .unwrap_err();
        assert_eq!(err, CustomError::InvalidContributorMilestone.into());
    }

    #[test]
    fn payout_status_allows_approved_to_executed() {
        validate_payout_status_transition(PayoutStatusV1::Approved, PayoutStatusV1::Executed)
            .unwrap();
    }

    #[test]
    fn payout_status_rejects_pending_to_executed() {
        let err =
            validate_payout_status_transition(PayoutStatusV1::Pending, PayoutStatusV1::Executed)
                .unwrap_err();
        assert_eq!(err, CustomError::InvalidContributorPayoutRequest.into());
    }

    #[test]
    fn checked_increment_u32_rejects_overflow() {
        let err = checked_increment_u32(u32::MAX).unwrap_err();
        assert_eq!(err, CustomError::MathOverflow.into());
    }

    #[test]
    fn checked_increment_u64_rejects_overflow() {
        let err = checked_increment_u64(u64::MAX).unwrap_err();
        assert_eq!(err, CustomError::MathOverflow.into());
    }

    #[test]
    fn contributor_payload_hash_is_stable_for_same_payload() {
        let payload = AddContributorPayloadV1 {
            contributor_wallet: CONTRIBUTOR_WALLET,
            contributor_role: ContributorRoleV1::BackendDeveloper,
        };
        assert_eq!(
            hash_contributor_payload(&payload).unwrap(),
            hash_contributor_payload(&payload).unwrap()
        );
    }

    #[test]
    fn security_execution_rejects_payload_mismatch() {
        let payload = AddContributorPayloadV1 {
            contributor_wallet: CONTRIBUTOR_WALLET,
            contributor_role: ContributorRoleV1::BackendDeveloper,
        };
        let payload_hash = hash_contributor_payload(&payload).unwrap();
        let queue = executed_queue(
            ActionType::ContributorAddContributor,
            CONTRIBUTOR_REGISTRY_KEY,
            [9u8; 32],
        );
        let err = validate_contributor_security_execution(
            &governance_config(),
            &proposal_decision(ProposalType::ContributorAddContributor),
            &queue,
            PROPOSAL_ID,
            ProposalType::ContributorAddContributor,
            ActionType::ContributorAddContributor,
            CONTRIBUTOR_REGISTRY_KEY,
            payload_hash,
        )
        .unwrap_err();
        assert_eq!(err, CustomError::PayloadHashMismatch.into());
    }

    #[test]
    fn execute_add_contributor_sets_registry_active() {
        let payload = AddContributorPayloadV1 {
            contributor_wallet: CONTRIBUTOR_WALLET,
            contributor_role: ContributorRoleV1::BackendDeveloper,
        };
        let payload_hash = hash_contributor_payload(&payload).unwrap();
        let queue = executed_queue(
            ActionType::ContributorAddContributor,
            CONTRIBUTOR_REGISTRY_KEY,
            payload_hash,
        );
        validate_contributor_security_execution(
            &governance_config(),
            &proposal_decision(ProposalType::ContributorAddContributor),
            &queue,
            PROPOSAL_ID,
            ProposalType::ContributorAddContributor,
            ActionType::ContributorAddContributor,
            CONTRIBUTOR_REGISTRY_KEY,
            payload_hash,
        )
        .unwrap();

        let mut registry = default_registry();
        record_add_contributor_execution(
            &mut registry,
            CONTRIBUTOR_WALLET,
            ContributorRoleV1::BackendDeveloper,
            9,
            100,
        )
        .unwrap();
        assert_eq!(registry.wallet, CONTRIBUTOR_WALLET);
        assert_eq!(registry.role, ContributorRoleV1::BackendDeveloper);
        assert_eq!(registry.status, ContributorStatusV1::Active);
        assert_eq!(registry.joined_at, 100);
    }

    #[test]
    fn execute_remove_contributor_sets_removed() {
        let payload = RemoveContributorPayloadV1 {
            contributor_registry: CONTRIBUTOR_REGISTRY_KEY,
            reason_hash: nonzero_hash(),
        };
        let payload_hash = hash_contributor_payload(&payload).unwrap();
        let queue = executed_queue(
            ActionType::ContributorRemoveContributor,
            CONTRIBUTOR_REGISTRY_KEY,
            payload_hash,
        );
        validate_contributor_security_execution(
            &governance_config(),
            &proposal_decision(ProposalType::ContributorRemoveContributor),
            &queue,
            PROPOSAL_ID,
            ProposalType::ContributorRemoveContributor,
            ActionType::ContributorRemoveContributor,
            CONTRIBUTOR_REGISTRY_KEY,
            payload_hash,
        )
        .unwrap();

        let mut registry = active_registry();
        record_remove_contributor_execution(&mut registry, 120).unwrap();
        assert_eq!(registry.status, ContributorStatusV1::Removed);
        assert_eq!(registry.last_active_at, 120);
    }

    #[test]
    fn execute_update_contributor_role_changes_role() {
        let payload = UpdateContributorRolePayloadV1 {
            contributor_registry: CONTRIBUTOR_REGISTRY_KEY,
            new_role: ContributorRoleV1::CoreDeveloper,
        };
        let payload_hash = hash_contributor_payload(&payload).unwrap();
        let queue = executed_queue(
            ActionType::ContributorUpdateRole,
            CONTRIBUTOR_REGISTRY_KEY,
            payload_hash,
        );
        validate_contributor_security_execution(
            &governance_config(),
            &proposal_decision(ProposalType::ContributorUpdateRole),
            &queue,
            PROPOSAL_ID,
            ProposalType::ContributorUpdateRole,
            ActionType::ContributorUpdateRole,
            CONTRIBUTOR_REGISTRY_KEY,
            payload_hash,
        )
        .unwrap();

        let mut registry = active_registry();
        record_update_contributor_role_execution(
            &mut registry,
            ContributorRoleV1::CoreDeveloper,
            130,
        )
        .unwrap();
        assert_eq!(registry.role, ContributorRoleV1::CoreDeveloper);
        assert_eq!(registry.last_active_at, 130);
    }

    #[test]
    fn execute_approve_contributor_milestone_sets_approved() {
        let payload = ApproveContributorMilestonePayloadV1 {
            milestone: MILESTONE_KEY,
            approved_amount: 1_000,
        };
        let payload_hash = hash_contributor_payload(&payload).unwrap();
        let queue = executed_queue(
            ActionType::ContributorApproveMilestone,
            MILESTONE_KEY,
            payload_hash,
        );
        validate_contributor_security_execution(
            &governance_config(),
            &proposal_decision(ProposalType::ContributorApproveMilestone),
            &queue,
            PROPOSAL_ID,
            ProposalType::ContributorApproveMilestone,
            ActionType::ContributorApproveMilestone,
            MILESTONE_KEY,
            payload_hash,
        )
        .unwrap();

        let mut registry = active_registry();
        let mut milestone = pending_milestone();
        record_approve_contributor_milestone_execution(
            &mut registry,
            CONTRIBUTOR_REGISTRY_KEY,
            &mut milestone,
            1_000,
            140,
        )
        .unwrap();
        assert_eq!(milestone.status, MilestoneStatusV1::Approved);
        assert_eq!(registry.completed_milestones, 1);
    }

    #[test]
    fn execute_approve_builder_payout_sets_approved() {
        let payload = ApproveBuilderPayoutPayloadV1 {
            payout_request: PAYOUT_REQUEST_KEY,
            approved_amount: 1_000,
        };
        let payload_hash = hash_contributor_payload(&payload).unwrap();
        let queue = executed_queue(
            ActionType::ContributorApproveBuilderPayout,
            PAYOUT_REQUEST_KEY,
            payload_hash,
        );
        validate_contributor_security_execution(
            &governance_config(),
            &proposal_decision(ProposalType::ContributorApproveBuilderPayout),
            &queue,
            PROPOSAL_ID,
            ProposalType::ContributorApproveBuilderPayout,
            ActionType::ContributorApproveBuilderPayout,
            PAYOUT_REQUEST_KEY,
            payload_hash,
        )
        .unwrap();

        let mut registry = active_registry();
        let mut payout = pending_payout_request();
        record_approve_builder_payout_execution(
            &mut registry,
            CONTRIBUTOR_REGISTRY_KEY,
            &mut payout,
            1_000,
            150,
        )
        .unwrap();
        assert_eq!(payout.status, PayoutStatusV1::Approved);
        assert_eq!(registry.approved_payout_count, 1);
    }
}
