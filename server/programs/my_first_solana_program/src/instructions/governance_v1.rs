use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::constants::{
    BPS_DENOMINATOR, GOVERNANCE_POSITION_V1_SEED, GOVERNANCE_PROPOSAL_V1_SEED,
    GOVERNANCE_SNAPSHOT_V1_SEED, VOTE_RECORD_V1_SEED,
};
use crate::error::CustomError;
use crate::state::{
    GovernancePositionStatusV1, GovernancePositionV1, GovernanceProposalStatusV1,
    GovernanceProposalTypeV1, GovernanceProposalV1, GovernanceSnapshotV1, VoteChoiceV1,
    VoteRecordV1,
};

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
pub struct InitializeGovernancePositionV1<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + GovernancePositionV1::INIT_SPACE,
        seeds = [GOVERNANCE_POSITION_V1_SEED, owner.key().as_ref()],
        bump
    )]
    pub governance_position: Account<'info, GovernancePositionV1>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub alpha_mint: Box<Account<'info, Mint>>,

    pub system_program: Program<'info, System>,
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

pub fn initialize_governance_position_v1_handler(
    ctx: Context<InitializeGovernancePositionV1>,
) -> Result<()> {
    record_governance_position_init(
        &mut ctx.accounts.governance_position,
        ctx.accounts.owner.key(),
        ctx.accounts.alpha_mint.key(),
        ctx.bumps.governance_position,
    )
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

pub fn initialize_vote_record_v1_handler(ctx: Context<InitializeVoteRecordV1>) -> Result<()> {
    record_vote_record_init(
        &mut ctx.accounts.vote_record,
        ctx.accounts.governance_proposal.key(),
        ctx.accounts.governance_position.key(),
        ctx.bumps.vote_record,
    )
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
    governance_proposal.bump = bump;

    Ok(())
}

pub fn record_governance_position_init(
    governance_position: &mut GovernancePositionV1,
    owner: Pubkey,
    alpha_mint: Pubkey,
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

    governance_position.owner = owner;
    governance_position.alpha_mint = alpha_mint;
    governance_position.locked_amount = 0;
    governance_position.lock_start_time = 0;
    governance_position.lock_end_time = 0;
    governance_position.reputation_snapshot = 0;
    governance_position.holding_multiplier_bps = BPS_DENOMINATOR;
    governance_position.reputation_multiplier_bps = BPS_DENOMINATOR;
    governance_position.voting_power = 0;
    governance_position.status = GovernancePositionStatusV1::Active;
    governance_position.bump = bump;

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

    const PROPOSAL_ID: u64 = 7;
    const PROPOSER: Pubkey = Pubkey::new_from_array([1; 32]);
    const OWNER: Pubkey = Pubkey::new_from_array([2; 32]);
    const ALPHA_MINT: Pubkey = Pubkey::new_from_array([3; 32]);
    const TARGET_PROGRAM: Pubkey = Pubkey::new_from_array([4; 32]);
    const TARGET_ACCOUNT: Pubkey = Pubkey::new_from_array([5; 32]);
    const PROPOSAL_KEY: Pubkey = Pubkey::new_from_array([6; 32]);
    const POSITION_KEY: Pubkey = Pubkey::new_from_array([7; 32]);
    const PAYLOAD_HASH: [u8; 32] = [8; 32];

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
            bump: 0,
        }
    }

    fn default_position() -> GovernancePositionV1 {
        GovernancePositionV1 {
            owner: Pubkey::default(),
            alpha_mint: Pubkey::default(),
            locked_amount: 1,
            lock_start_time: 2,
            lock_end_time: 3,
            reputation_snapshot: 4,
            holding_multiplier_bps: 5,
            reputation_multiplier_bps: 6,
            voting_power: 7,
            status: GovernancePositionStatusV1::Closed,
            bump: 0,
        }
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
    fn initialize_governance_position_defaults_to_active_zero_lock() {
        let mut position = default_position();
        record_governance_position_init(&mut position, OWNER, ALPHA_MINT, 2).unwrap();

        assert_eq!(position.owner, OWNER);
        assert_eq!(position.alpha_mint, ALPHA_MINT);
        assert_eq!(position.locked_amount, 0);
        assert_eq!(position.lock_start_time, 0);
        assert_eq!(position.lock_end_time, 0);
        assert_eq!(position.reputation_snapshot, 0);
        assert_eq!(position.holding_multiplier_bps, BPS_DENOMINATOR);
        assert_eq!(position.reputation_multiplier_bps, BPS_DENOMINATOR);
        assert_eq!(position.voting_power, 0);
        assert_eq!(position.status, GovernancePositionStatusV1::Active);
        assert_eq!(position.bump, 2);
    }

    #[test]
    fn initialize_governance_position_rejects_default_owner() {
        let mut position = default_position();
        let err = record_governance_position_init(&mut position, Pubkey::default(), ALPHA_MINT, 2)
            .unwrap_err();
        assert_eq!(err, CustomError::InvalidGovernancePosition.into());
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
