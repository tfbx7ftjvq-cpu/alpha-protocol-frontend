# DAO Contributor Proposal Integration

## Purpose

Phase 2E-3B connects Contributor Governance to the existing Security Layer execution flow. It does not create a separate DAO proposal system.

Contributor actions now follow the existing path:

1. `ProposalDecisionV1`
2. `ExecutionQueueItemV1`
3. Security Layer timelock execution
4. Contributor state mutation

## Why Security Layer Is The DAO Execution Layer

The Security Layer already provides the execution safety envelope for Alpha Protocol:

- proposal id tracking
- proposal decision records
- action type checks
- execution queue
- timelock
- payload hash checks
- cancellation / pause controls

Contributor Governance reuses this layer so contributor status, roles, milestones, and payout request approvals cannot be changed by an arbitrary wallet.

## Contributor Action Types

Phase 2E-3B adds contributor action support for:

- `AddContributor`
- `RemoveContributor`
- `UpdateContributorRole`
- `ApproveMilestone`
- `ApproveBuilderPayout`

The protocol also extends the existing `ProposalType` and `ActionType` enums with contributor-specific variants so the Security Layer can validate proposal/action compatibility.

## Contributor Proposal Payloads

Each contributor action has a hashable payload:

- Add contributor: contributor wallet + contributor role
- Remove contributor: contributor registry + reason hash
- Update role: contributor registry + new role
- Approve milestone: milestone address + approved amount
- Approve builder payout: payout request address + approved amount

The payload is serialized and hashed. Execution checks the resulting hash against the `ExecutionQueueItemV1.payload_hash`.

## Contributor Action Lifecycle

### Add Contributor

`execute_add_contributor` creates or activates `ContributorRegistryV1`.

Rules:

- queue item must already be `Executed`
- action type must be `ContributorAddContributor`
- target account must be the contributor registry PDA
- payload hash must match
- final status becomes `Active`

Direct wallet initialization can only create a suspended candidate record. It does not grant active contributor status.

### Remove Contributor

`execute_remove_contributor` changes:

- `Active -> Removed`

The registry account is kept for history and auditability. It is not deleted.

### Update Contributor Role

`execute_update_contributor_role` changes the contributor role, for example:

- `BackendDeveloper -> CoreDeveloper`

The contributor must be active, and repeated no-op role updates are rejected.

### Approve Milestone

`execute_approve_contributor_milestone` changes:

- `Pending -> Approved`

It increments the contributor completed milestone counter with checked arithmetic.

No payout is made in this phase.

### Approve Builder Payout

`execute_approve_builder_payout` changes:

- `Pending -> Approved`

It increments the contributor approved payout counter with checked arithmetic.

No USDC transfer is made in this phase.

## Future ALPHA Voting Layer

Future phases should add:

- ALPHA holder voting
- quorum
- token-weighted voting power
- vote records
- proposal finalization

The voting layer should produce a `ProposalDecisionV1`, then the existing Security Layer queue/timelock flow should handle execution.

## Future Builders Treasury Payout

Builders payout remains a future phase.

Expected future flow:

1. Contributor milestone is approved.
2. Builder payout request is approved.
3. Security Layer queues a builders payout action.
4. Timelock expires.
5. Dedicated payout instruction transfers USDC from the Builders Treasury vault.

Phase 2E-3B intentionally does not transfer USDC and does not operate the Builders vault.

## Not Implemented In This Phase

- ALPHA voting
- quorum
- token voting power
- vote records
- proposal finalization
- USDC payout
- Builders vault transfer
- frontend UI
