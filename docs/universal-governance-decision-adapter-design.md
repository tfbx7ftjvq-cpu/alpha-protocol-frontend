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

1. A governance proposal is created with `initialize_governance_proposal_with_action_v1`.
2. The same transaction creates immutable `GovernanceProposalActionV1`.
3. A snapshot is created only if the proposal action sidecar exists and matches the proposal.
4. the module registry is validated before the proposal enters voting.
5. voters cast votes.
6. the vote is finalized.
7. if quorum and approval threshold pass, the proposal becomes `Passed`.
8. anyone may call `create_governance_decision_adapter_v1`.
9. the program verifies the proposal, sidecar, module registry, and snapshot.
10. the program creates:
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

The following fields are read from `GovernanceProposalActionV1`:

- `GovernanceActionTypeV1`
- `ProtocolModuleIdV1`
- `target_program`
- `target_account`
- `canonical_payload_hash`

The adapter maps `GovernanceActionTypeV1` to Security `ActionType` and Security `ProposalType` through exhaustive helper functions.

`GovernanceProposalV1.action_type` remains only a compatibility mirror. It must match the sidecar stable action code, but it is not the adapter's trusted source.

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
- `GovernanceProposalActionV1` exists
- sidecar proposal id and proposer match `GovernanceProposalV1`
- sidecar stable action code matches the proposal mirror field
- sidecar module id matches the action mapping
- proposal type matches the action category
- sidecar target mirrors match proposal target fields
- sidecar module is registered in `ProtocolModuleRegistryV1`
- registry is enabled, schema version 1, and bound to the current Alpha Protocol Program ID
- registry is tied to the expected Security `GovernanceConfigV1`
- canonical payload hash recomputes from sidecar fields
- target program is the current Alpha Protocol Program ID
- proposal weights match finalized snapshot weights
- quorum and approval threshold still satisfy the shared proposal-type threshold helper
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

## 9. Governance Action Framework Compatibility

Phase 2E-4D-3B introduces `GovernanceActionTypeV1`, `ProtocolModuleIdV1`, and `GovernancePayloadV1`.

The intended future path is:

```text
GovernanceActionTypeV1
-> action mapping helper
-> Security ActionType
-> UniversalGovernanceDecisionAdapterV1
-> ProposalDecisionV1
-> ExecutionQueueItemV1
```

The adapter should continue to preserve the invariant:

```text
DAO-voted payload hash == Security execution payload hash
```

Phase 2E-FINAL Stage 2 implements the typed binding: adapter creation now consumes `GovernanceProposalActionV1` and rejects legacy proposals without a sidecar.

Phase 2E-FINAL Stage 3 adds `ProtocolModuleRegistryV1` to adapter validation. Registry update / enable / disable, external program registration, Treasury transfers, Relief / Scam Registry business accounts, and Mainnet authority migration remain out of scope.

The adapter path is now:

```text
GovernanceProposalV1::Passed
-> GovernanceProposalActionV1
-> ProtocolModuleRegistryV1
-> UniversalGovernanceDecisionAdapterV1
-> ProposalDecisionV1
```

## 10. Treasury Execution Layer Consumption

Phase 2E-FINAL Stage 4 adds the first module-specific consumer of adapter output:

```text
UniversalGovernanceDecisionAdapterV1
-> ProposalDecisionV1
-> ExecutionQueueItemV1 Executed
-> Approved Treasury request
-> strict Treasury wrapper
-> TreasuryExecutionRecordV1
```

The Treasury wrappers do not accept caller-controlled action, target, or payload.
They read the typed action sidecar, adapter, decision, queue item, and real
Treasury request accounts, then recompute the Stage 4 parameters hash.

The adapter remains a bridge only. It still does not transfer USDC, mutate
Treasury requests, or choose a source vault.
