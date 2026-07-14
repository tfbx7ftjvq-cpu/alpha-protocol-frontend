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

Phase 2E-FINAL Stage 2 adds `GovernanceProposalActionV1` as an immutable sidecar for new proposals. `GovernanceProposalV1.action_type` is retained as a compatibility mirror, while the sidecar is the trusted typed source for new governance execution paths.

## Governance Voting Config

`GovernanceVotingConfigV1` still stores the voting period and legacy default policy fields, but finalize now uses fixed proposal-type threshold policy. Callers cannot choose quorum or approval thresholds for a proposal at finalize time.

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

The snapshot instruction now requires `GovernanceProposalActionV1`, Security `GovernanceConfigV1`, and `ProtocolModuleRegistryV1`. Before voting starts, the program verifies that the sidecar matches the proposal id, proposer, stable action code, proposal category, target fields, module mapping, registry binding, schema version, and canonical payload hash.

Legacy proposals created without `GovernanceProposalActionV1`, or proposals whose module registry is missing, disabled, or mismatched, cannot enter the new voting path.

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

The snapshot total is copied from `GovernancePowerStateV1.total_voting_power` when voting starts. It is not supplied by the caller. Voting records require the voter position to have been last updated at or before the snapshot timestamp, so new locks after snapshot creation cannot increase voting power for the current proposal.

Governance V1 voting power is linear locked ALPHA multiplied by the committed lock-duration multiplier. It does not use square-root voting, because square-root voting rewards wallet splitting in a permissionless system without identity binding.

The typed action binding makes the voted payload immutable before the snapshot. After voting starts, callers cannot replace the action, target account, target program, or payload hash that the Universal Governance Decision Adapter will later consume.

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
- governance position vote-lock sidecar exists and is updated after voting
- vote record does not already exist
- aggregate votes do not exceed snapshot total voting power

## Anti Double Vote

The vote record PDA is:

```text
vote_record_v1 + proposal.key() + governance_position.key()
```

This gives each governance position at most one vote record per proposal. If a vote record already contains a proposal / position pair, `cast_governance_vote_v1` returns `AlreadyVoted`.

Voting also updates:

```text
governance_position_vote_lock_v1 + governance_position.key()
```

`voting_lock_until` is set to at least the proposal voting end timestamp. A voter cannot unlock the governance position until both the original lock period and this vote-lock period have passed.

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

Proposal-type policy:

| Proposal Type | Quorum | Approval |
| --- | --- | --- |
| Contributor | 5% | 60% |
| Treasury | 10% | 66.67% |
| GreenLabel | 10% | 66.67% |
| VictimRelief | 10% | 66.67% |
| ScamRegistry | 10% | 66.67% |
| Parameter | 20% | 75% |
| Upgrade | 25% | 80% |
| Emergency | 15% | 75% |

If quorum is not reached, finalization returns `QuorumNotReached` and the proposal remains in `Voting`.

## Approval Threshold

Approval ignores abstain votes:

```text
yes_weight / (yes_weight + no_weight) >= approval_threshold_bps / 10_000
```

The approval threshold is selected by proposal type and ignores abstain votes.

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

Phase 2E-FINAL Stage 3 adds `ProtocolModuleRegistryV1` as the module allow-list used by snapshot creation and adapter creation. This ensures the action voted on by the DAO is bound to an enabled Alpha Protocol module before it can move toward Security Layer execution.

## Not Implemented In This Phase

- Security Layer connection
- Execution Queue integration
- Treasury transfer
- builder payout execution
- program upgrade governance execution
- frontend
- Mainnet deployment
