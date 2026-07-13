# Governance V1 Core Foundation

## Why Alpha Protocol Needs Governance V1

Alpha Protocol now has Treasury V2, the USDC Revenue Router, Green Label refundable escrow, the Security Layer execution queue, and contributor governance records.

Governance V1 is the missing community decision layer. Its purpose is to prepare the protocol for future ALPHA-based voting without replacing the existing Security Layer execution path.

This phase only creates core accounts and initialization instructions. It does not implement ALPHA token voting, token transfer, vote calculation, proposal finalization, Security Layer queueing, or frontend UI.

## Hybrid Governance Model

The intended long-term model is:

```text
sqrt(ALPHA locked amount)
* holding time multiplier
* reputation multiplier
```

Design limits:

- holding time multiplier max: 2x
- reputation multiplier max: 1.5x

Phase 2E-4B created the core proposal / position / snapshot / vote record skeleton. Phase 2E-4C-1 implements the lock foundation with `sqrt(locked_amount) * holding time multiplier`; reputation multiplier remains a future extension and is not stored in the current `GovernancePositionV1`.

## GovernanceProposal Lifecycle

`GovernanceProposalV1` records a community proposal.

Current status enum:

- `Draft`
- `Voting`
- `Passed`
- `Rejected`
- `Queued`
- `Executed`
- `Cancelled`

Future lifecycle:

```text
Draft
-> Voting
-> Passed / Rejected
-> Queued
-> Executed / Cancelled
```

Future passed proposals should create or map to a Security Layer `ProposalDecisionV1`, then use `ExecutionQueueItemV1` for timelock execution.

Proposal PDA:

```text
governance_proposal_v1 + proposal_id.to_le_bytes()
```

## GovernancePosition Design

`GovernancePositionV1` records a future ALPHA governance lock position.

Fields:

- owner
- alpha_mint
- vault
- locked_amount
- lock_start_time
- lock_end_time
- holding_multiplier_bps
- voting_power
- status
- last_updated_at
- bump

Current position statuses:

- `Active`
- `Unlocking`
- `Closed`

Initialization defaults:

- `status = Active`
- `locked_amount = 0`
- `voting_power = 0`
- `holding_multiplier_bps = 0` until ALPHA is locked

Position PDA:

```text
governance_position_v1 + owner.key().as_ref()
```

Phase 2E-4C-1 adds ALPHA lock / unlock transfer support through the governance vault. It still does not implement proposal voting or vote finalization.

## Snapshot Design

`GovernanceSnapshotV1` is reserved to freeze proposal-level voting power totals.

Fields:

- proposal
- total_voting_power
- yes_weight
- no_weight
- abstain_weight
- created_at
- finalized
- bump

Initialization defaults:

- all weights = 0
- `finalized = false`

Snapshot PDA:

```text
governance_snapshot_v1 + proposal.key().as_ref()
```

This phase does not execute snapshot voting power calculation.

## VoteRecord Design

`VoteRecordV1` records a future vote by one governance position.

Fields:

- proposal
- voter_position
- choice
- voting_power_used
- timestamp
- bump

Current vote choices:

- `Yes`
- `No`
- `Abstain`

Initialization defaults:

- `choice = Abstain`
- `voting_power_used = 0`
- `timestamp = 0`

Vote record PDA:

```text
vote_record_v1 + proposal.key().as_ref() + governance_position.key().as_ref()
```

This PDA design prevents one governance position from creating multiple vote records for the same proposal.

## Future Voting Layer Roadmap

Future phases should add:

1. Snapshot creation and voting power freeze logic.
2. Cast vote instruction.
3. Proposal finalization with quorum / threshold rules.
4. Conversion from passed governance proposal to `ProposalDecisionV1`.
5. Optional reputation multiplier after the contributor reputation model is finalized.

## Future Security Layer Connection

The Security Layer remains the execution layer.

Target flow:

```text
GovernanceProposalV1
-> GovernanceSnapshotV1
-> VoteRecordV1
-> Finalized voting result
-> ProposalDecisionV1
-> ExecutionQueueItemV1
-> Timelock
-> Domain-specific execute instruction
```

This phase does not connect Governance V1 to Security Layer execution.

## Not Implemented In This Phase

- ALPHA voting
- quorum
- snapshot execution
- proposal finalization
- Security Layer connection
- frontend
