# Governance Voting Engine Design

## Purpose

Phase 2E-4C-2 implements the first complete DAO voting engine inside Alpha Protocol:

```text
Proposal -> Snapshot -> Vote -> Count -> Finalize
```

This phase does not connect passed governance decisions to the Security Layer, Execution Queue, Treasury transfers, builder payouts, program upgrades, frontend UI, or Mainnet deployment.

## Proposal Lifecycle

`GovernanceProposalV1` now records both proposal metadata and final voting results.

Core lifecycle in this phase:

```text
Draft
-> Voting
-> Passed / Rejected
```

The enum also keeps future states such as `Queued` and `Executed` for later Security Layer integration, but Phase 2E-4C-2 stops at governance decision finalization.

`GovernanceProposalV1` stores:

- proposal id
- proposer
- proposal type
- action type
- target program / target account
- payload hash
- voting window
- snapshot account
- yes / no / abstain weights
- finalized timestamp
- status

## Governance Voting Config

`GovernanceVotingConfigV1` parameterizes voting policy instead of hardcoding it in finalize logic.

Default values:

- quorum: 500 bps, or 5%
- approval threshold: 6,000 bps, or 60%
- voting period: 7 days

PDA:

```text
governance_voting_config_v1
```

## Snapshot Mechanism

`create_governance_snapshot_v1` creates a `GovernanceSnapshotV1` account and moves a proposal from `Draft` to `Voting`.

The snapshot records:

- proposal account
- total voting power
- yes / no / abstain weights
- created timestamp
- finalized flag

PDA:

```text
governance_snapshot_v1 + proposal.key()
```

The snapshot total is fixed when voting starts. Voting records require the voter position to have been last updated at or before the snapshot timestamp, so new locks after snapshot creation cannot increase voting power for the current proposal.

## Voting Flow

`cast_governance_vote_v1` creates a `VoteRecordV1` and adds the governance position's voting power to one bucket:

- `Yes`
- `No`
- `Abstain`

The instruction checks:

- proposal is in `Voting`
- current timestamp is inside the voting window
- snapshot exists and is not finalized
- governance position belongs to the signer
- governance position is active
- governance position has nonzero voting power
- governance position was not updated after the snapshot
- vote record does not already exist
- aggregate votes do not exceed snapshot total voting power

## Anti Double Vote

The vote record PDA is:

```text
vote_record_v1 + proposal.key() + governance_position.key()
```

This gives each governance position at most one vote record per proposal. If a vote record already contains a proposal / position pair, `cast_governance_vote_v1` returns `AlreadyVoted`.

## Vote Counting

Vote weights are accumulated in `GovernanceSnapshotV1`:

```text
total_votes = yes_weight + no_weight + abstain_weight
```

All additions are checked arithmetic.

## Quorum

Finalize checks:

```text
total_votes / total_voting_power >= quorum_bps / 10_000
```

The default quorum is 5%. If quorum is not reached, finalization returns `QuorumNotReached` and the proposal remains in `Voting`.

## Approval Threshold

Approval ignores abstain votes:

```text
yes_weight / (yes_weight + no_weight) >= approval_threshold_bps / 10_000
```

The default approval threshold is 60%.

If approval passes:

```text
status = Passed
```

Otherwise:

```text
status = Rejected
```

Finalize also copies the snapshot weights into `GovernanceProposalV1` and marks the snapshot finalized.

## Future Security Layer Connection

Phase 2E-4D should connect successful governance decisions to the existing Security Layer:

```text
GovernanceProposalV1 Passed
-> ProposalDecisionV1
-> ExecutionQueueItemV1
-> Timelock
-> Domain-specific execute instruction
```

This preserves the existing Security Layer as the execution layer while Governance V1 becomes the community decision layer.

## Not Implemented In This Phase

- Security Layer connection
- Execution Queue integration
- Treasury transfer
- builder payout execution
- program upgrade governance execution
- frontend
- Mainnet deployment
