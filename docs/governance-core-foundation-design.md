# Governance Core Foundation Design

## Purpose

Phase 2E-4B adds the core Governance V1 data model for a future hybrid voting layer.

This phase is intentionally data-only. It does not implement ALPHA voting, voting power calculation, ALPHA token transfer, snapshot execution, proposal finalization, Security Layer connection, or frontend UI.

## New Core Accounts

### GovernanceProposalV1

Records a community governance proposal before it is converted into a Security Layer decision.

Fields:

- `proposal_id`
- `proposer`
- `proposal_type`
- `action_type`
- `target_program`
- `target_account`
- `payload_hash`
- `status`
- `voting_start_ts`
- `voting_end_ts`
- `created_at`
- `bump`

Suggested PDA seed:

```text
governance_proposal_v1 + proposal_id_le
```

### GovernancePositionV1

Records the future governance lock position for a voter.

Fields:

- `owner`
- `alpha_mint`
- `locked_amount`
- `lock_start_time`
- `lock_end_time`
- `reputation_snapshot`
- `holding_multiplier_bps`
- `reputation_multiplier_bps`
- `voting_power`
- `status`
- `bump`

Suggested PDA seed:

```text
governance_position_v1 + owner
```

This account is separate from staking reward accounting. Future phases can decide whether ALPHA staking positions can be migrated or mirrored into governance positions, but this phase does not transfer ALPHA. Initialization defaults to `Active`, `locked_amount = 0`, and 1x multipliers.

### GovernanceSnapshotV1

Records proposal-level voting totals after a future snapshot is created.

Fields:

- `proposal`
- `total_voting_power`
- `yes_weight`
- `no_weight`
- `abstain_weight`
- `created_at`
- `finalized`
- `bump`

Suggested PDA seed:

```text
governance_snapshot_v1 + proposal
```

### VoteRecordV1

Records a single vote by a governance position.

Fields:

- `proposal`
- `voter_position`
- `choice`
- `voting_power_used`
- `timestamp`
- `bump`

Suggested PDA seed:

```text
vote_record_v1 + proposal + voter_position
```

This PDA shape prevents a single governance position from voting twice on the same proposal.

## Enums

Phase 2E-4B adds:

- `GovernanceProposalTypeV1`
- `GovernanceProposalStatusV1`
- `GovernancePositionStatusV1`
- `VoteChoiceV1`

These are Anchor-compatible enums reserved for future instructions.

## Future Hybrid Voting Model

The intended future voting power model is:

```text
sqrt(ALPHA locked amount)
* holding time multiplier
* reputation multiplier
```

Limits from design review:

- holding time multiplier max: 2x
- reputation multiplier max: 1.5x

This phase only stores fields needed for that future calculation. It does not calculate or enforce voting power.

## Future Lifecycle

Target lifecycle for later phases:

```text
Draft
-> Snapshot
-> Voting
-> Finalize
-> Security Layer Review
-> Execution Queue
-> Timelock
-> Execute
```

Security Layer integration remains a future phase. The existing `ProposalDecisionV1` and `ExecutionQueueItemV1` should remain the execution path.

## Not Implemented In This Phase

- ALPHA holder voting
- quorum
- token-weighted voting calculation
- ALPHA lock or unlock transfer
- snapshot execution
- proposal finalization
- conversion to `ProposalDecisionV1`
- Security Layer queueing
- USDC payout
- frontend
