# Typed Governance Proposal Action Binding

## Purpose

Phase 2E-FINAL Stage 2 adds an immutable typed action sidecar for each new `GovernanceProposalV1`.

The goal is to make the voted governance intent the same intent that the Universal Governance Decision Adapter later converts into a Security Layer `ProposalDecisionV1`.

## New Trusted Source

`GovernanceProposalV1.action_type` remains a legacy compatibility mirror field.

The trusted source for new DAO-controlled execution paths is now:

```text
GovernanceProposalActionV1
```

It permanently binds a proposal to:

- `GovernanceActionTypeV1`
- `ProtocolModuleIdV1`
- `target_program`
- `target_account`
- `parameters_hash`
- `evidence_hash`
- `canonical_payload_hash`
- `schema_version`

The sidecar PDA is:

```text
[
  b"governance_proposal_action_v1",
  governance_proposal.key().as_ref()
]
```

One proposal can have only one action sidecar. There is no update instruction.

## Strict Proposal Initialization

New proposals should use:

```text
initialize_governance_proposal_with_action_v1
```

The caller supplies a typed `GovernanceActionRequestV1`. The program derives:

- `GovernanceProposalTypeV1`
- stable `action_type` code mirrored into `GovernanceProposalV1.action_type`
- canonical payload hash
- proposal target mirror fields

The caller cannot override the derived proposal type, raw action code, or canonical hash.

`target_program` is restricted to the current Alpha Protocol Program ID. Phase 2E-FINAL Stage 3 adds `ProtocolModuleRegistryV1`, so the strict initializer now validates the requested module against the registry and stores the target program from the registry.

The caller cannot bind a typed proposal to an unregistered module, a disabled module, or an arbitrary external program.

## Stable Action Codes

The strict path uses:

```text
governance_action_stable_code_v1(action)
governance_action_from_stable_code_v1(code)
```

The protocol does not rely on `action as u8`. Existing action codes mirror the current `GovernanceActionTypeV1` order, and future variants must be appended only.

Unknown stable codes are rejected.

## Snapshot Binding

`create_governance_snapshot_v1` now requires `GovernanceProposalActionV1`, Security `GovernanceConfigV1`, and `ProtocolModuleRegistryV1`.

Before a proposal can enter `Voting`, the program verifies:

- sidecar proposal id and proposer match the proposal
- stable action code matches the proposal mirror field
- proposal type matches the action category
- target mirrors match
- action-to-module mapping is correct
- module registry PDA, stable code, schema, enabled flag, Security governance config, and program id are valid
- canonical payload hash recomputes from sidecar fields
- schema version is valid

Legacy proposals without a sidecar cannot enter the new voting path.

## Adapter Enforcement

`create_governance_decision_adapter_v1` now requires `GovernanceProposalActionV1` and `ProtocolModuleRegistryV1`.

The adapter derives these fields from the sidecar:

- `GovernanceActionTypeV1`
- `ProtocolModuleIdV1`
- Security `ActionType`
- Security `ProposalType`
- `target_program`
- `target_account`
- `canonical_payload_hash`

The adapter validates the sidecar module against `ProtocolModuleRegistryV1` before creating `ProposalDecisionV1`. It does not accept caller-controlled action, target, program id, or payload data. It also does not trust mutated proposal mirror fields unless they match the sidecar.

## Canonical Payload Hash

The canonical payload uses:

```text
alpha_governance_payload_v1
```

as the domain separator and hashes the serialized `GovernancePayloadV1` fields in fixed order.

Snapshot and adapter validation both use the same helper to avoid rule drift.

## Legacy Proposal Path

`initialize_governance_proposal_v1` remains for compatibility and bootstrap testing.

Legacy proposals without `GovernanceProposalActionV1`:

- cannot create a governance snapshot
- cannot enter the Universal Governance Decision Adapter
- cannot enter future DAO-controlled execution

Devnet governance proposals created before this phase must be reinitialized or migrated before they can use the new typed execution path.

## Stage 4 Treasury Parameters Binding

Phase 2E-FINAL Stage 4 uses `GovernanceProposalActionV1.parameters_hash` as the
trusted binding for Treasury spending and builder payout execution.

For Treasury actions, the parameters hash is no longer an opaque caller claim.
The program rebuilds the typed parameters from real accounts during both
approval and execution:

- request account
- amount
- recipient owner
- recipient token account
- fixed `builders_usdc_vault`
- USDC mint
- proposal id
- purpose hash or milestone / payout links, depending on action

If any real account differs from the DAO-voted sidecar parameters hash, approval
or execution fails. This keeps:

```text
DAO-voted parameters == Security queue payload == Treasury transfer parameters
```

Stage 4 still does not implement generic Treasury transfers, batch payout,
registry mutation, Green Label execution changes, or frontend writes.

## Stage 5B-1 Green Label Certification Parameters Binding

Phase 2E-FINAL Stage 5B-1 uses `GovernanceProposalActionV1.parameters_hash`
as the trusted binding for Green Label certification decisions.

For approve, reject, and revoke certification wrappers, the program rebuilds a
typed `GreenLabelCertificationDecisionParametersV1` from real accounts:

- Green Label config
- Green Label project
- certification state sidecar
- governance action type
- project authority
- bond tier
- bond vault
- USDC mint
- observation end timestamp
- expected project status
- expected certification status
- proposal id

If the rebuilt parameters hash differs from the DAO-voted sidecar, execution
fails. Reject and revoke do not imply refund or slash. The certification wrappers
do not transfer tokens.

## Explicit Non-Goals

This phase does not implement:

- Protocol Module Registry update / enable / disable
- external program registration
- Treasury USDC execution
- builder payout transfer
- Green Label DAO closure
- Victim Relief accounts
- Scam Registry accounts
- DAO Control Mode
- authority migration

Mainnet remains NO-GO.
