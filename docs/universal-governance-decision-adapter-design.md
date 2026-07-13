# Universal Governance Decision Adapter V1 Design

## 1. Why the Adapter Exists

Alpha Protocol now has two separate governance layers:

- Governance Layer: ALPHA lock, voting power, snapshot, vote, finalize, and `GovernanceProposalV1`.
- Security Layer: `ProposalDecisionV1`, `ExecutionQueueItemV1`, timelock, cancel, pause, and module execution checks.

Before this phase, a DAO vote could mark a `GovernanceProposalV1` as `Passed`, but there was no trusted on-chain bridge that converted that passed vote into a Security Layer decision.

`UniversalGovernanceDecisionAdapterV1` closes that gap.

## 2. DAO Layer and Security Layer Separation

The adapter does not merge the two systems.

The Governance Layer remains responsible for:

- proposal creation
- ALPHA governance lock
- voting power
- vote records
- quorum
- approval threshold
- final `Passed` or `Rejected` status

The Security Layer remains responsible for:

- `ProposalDecisionV1`
- `ExecutionQueueItemV1`
- timelock
- cancel
- pause
- module-level execution checks

The adapter only connects:

```text
GovernanceProposalV1::Passed
-> UniversalGovernanceDecisionAdapterV1
-> ProposalDecisionV1::Approved
```

It does not execute Treasury, Green Label, relief, scam registry, contributor payout, upgrade, or frontend actions.

## 3. Adapter Lifecycle

1. A governance proposal is created with:
   - `proposal_id`
   - `action_type`
   - `target_program`
   - `target_account`
   - `payload_hash`
2. A snapshot is created.
3. voters cast votes.
4. the vote is finalized.
5. if quorum and approval threshold pass, the proposal becomes `Passed`.
6. anyone may call `create_governance_decision_adapter_v1`.
7. the program verifies the proposal and snapshot.
8. the program creates:
   - `UniversalGovernanceDecisionAdapterV1`
   - `ProposalDecisionV1`

The adapter PDA is:

```text
[
  b"universal_governance_decision_adapter_v1",
  governance_proposal.key().as_ref()
]
```

One passed governance proposal can create only one adapter account.

## 4. Payload Hash Security Model

The adapter does not allow the caller to supply execution intent again.

The following fields are read from `GovernanceProposalV1`:

- `action_type`
- `target_program`
- `target_account`
- `payload_hash`

They are copied into `UniversalGovernanceDecisionAdapterV1`.

This preserves:

```text
Governance vote payload == Security execution payload
```

The next phase should add a queue adapter that creates `ExecutionQueueItemV1` from the adapter account instead of caller-provided payload values.

## 5. Security Checks

`create_governance_decision_adapter_v1` checks:

- proposal status is `Passed`
- snapshot is finalized
- proposal snapshot matches the provided snapshot
- snapshot proposal matches the governance proposal
- proposal weights match finalized snapshot weights
- quorum still satisfies `GovernanceVotingConfigV1`
- approval threshold still satisfies `GovernanceVotingConfigV1`
- payload hash is non-zero
- target program and target account are non-zero
- Security Layer proposal id sequence is preserved
- adapter and proposal decision accounts are empty before creation

Rejected, Draft, unfinalized, malformed, or replayed proposals cannot create a Security decision.

## 6. Future Module Expansion

The adapter is module-neutral. Future phases can connect:

- Treasury parameter changes
- Builders payout approval
- Green Label refund, slash, and forfeited bond routing
- Victim Relief case approval
- Scam Registry updates
- Contributor add, remove, role update, milestone approval, and payout approval
- Protocol upgrade governance
- emergency actions

## 7. Explicit Non-Goals

This phase does not implement:

- Treasury transfer
- Builders payout
- Green Label execution
- Victim Relief execution
- Scam Registry execution
- program upgrade
- frontend UI
- Mainnet deployment
- chain transaction scripts

## 8. Next Phase

The recommended next phase is:

```text
UniversalGovernanceDecisionAdapterV1
-> Queue Adapter
-> ExecutionQueueItemV1
-> Timelock
-> module-specific execute instruction
```

That queue adapter should consume the sealed `action_type`, `target_program`, `target_account`, and `payload_hash` from `UniversalGovernanceDecisionAdapterV1`.
