use anchor_lang::prelude::*;

use crate::constants::{
    BUILDER_PAYOUT_REQUEST_V1_SEED, CONTRIBUTOR_MILESTONE_DESCRIPTION_MAX_BYTES,
    CONTRIBUTOR_MILESTONE_TITLE_MAX_BYTES, CONTRIBUTOR_MILESTONE_V1_SEED,
    CONTRIBUTOR_REGISTRY_V1_SEED,
};
use crate::error::CustomError;
use crate::state::{
    BuilderPayoutRequestV1, ContributorMilestoneV1, ContributorRegistryV1, ContributorRoleV1,
    ContributorStatusV1, MilestoneStatusV1, PayoutStatusV1,
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

pub fn initialize_contributor_registry_v1_handler(
    ctx: Context<InitializeContributorRegistryV1>,
    role: ContributorRoleV1,
) -> Result<()> {
    validate_contributor_role(role)?;

    let now = Clock::get()?.unix_timestamp;
    let contributor_registry = &mut ctx.accounts.contributor_registry;
    contributor_registry.wallet = ctx.accounts.contributor_wallet.key();
    contributor_registry.role = role;
    contributor_registry.status = ContributorStatusV1::Active;
    contributor_registry.joined_at = now;
    contributor_registry.last_active_at = now;
    contributor_registry.completed_milestones = 0;
    contributor_registry.approved_payout_count = 0;
    contributor_registry.reputation_score = 0;
    contributor_registry.bump = ctx.bumps.contributor_registry;

    Ok(())
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

    fn nonzero_hash() -> [u8; 32] {
        [7u8; 32]
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
}
