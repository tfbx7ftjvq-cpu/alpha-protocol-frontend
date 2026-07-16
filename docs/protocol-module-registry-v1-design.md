# Protocol Module Registry V1 Design

## Purpose

Phase 2E-FINAL Stage 3 adds `ProtocolModuleRegistryV1` as the on-chain allow-list for Alpha Protocol governance modules.

The registry closes the gap between:

```text
GovernanceActionTypeV1
-> ProtocolModuleIdV1
-> target program / target account
-> Universal Governance Decision Adapter
```

Before this stage, typed governance proposals could bind an action to a module, but there was no independent registry proving that the module was enabled and bound to the current Alpha Protocol program.

## Account

`ProtocolModuleRegistryV1` stores:

- `security_governance_config`
- `module_id`
- `module_code`
- `program_id`
- `enabled`
- `schema_version`
- `created_at`
- `updated_at`
- `bump`

The account size is fixed at 86 bytes excluding the Anchor discriminator, and tests assert the exact size.

## PDA

The registry PDA is:

```text
[
  b"protocol_module_registry_v1",
  &[protocol_module_stable_code_v1(module_id)]
]
```

The stable module codes are:

| Module | Code |
| --- | --- |
| Treasury | 1 |
| GreenLabel | 2 |
| VictimRelief | 3 |
| ScamRegistry | 4 |
| Contributor | 5 |
| Protocol | 6 |

The protocol does not use `module_id as u8` for stable encoding. Unknown module codes are rejected.

## Bootstrap Initialization

`initialize_protocol_module_registry_v1` initializes one module registry account.

It requires:

- payer signer
- bootstrap authority signer
- `GovernanceConfigV1` from the Security Layer
- `ProtocolModuleRegistryV1` init PDA
- system program

The bootstrap authority must equal `GovernanceConfigV1.authority`.

In V1:

- `schema_version` must be `1`
- `program_id` is always the current Alpha Protocol Program ID
- `enabled` is always `true`
- callers cannot choose `module_code`, `program_id`, or `enabled`

There is no update, enable, disable, or external program registration instruction in this phase.

## Shared Validation

`validate_protocol_module_registry_v1` checks:

- registry PDA and bump
- stored stable module code
- expected module id
- expected Security `GovernanceConfigV1`
- `enabled == true`
- `schema_version == 1`
- `program_id == expected target program`
- `program_id == crate::ID`

This helper is reused by strict proposal initialization, snapshot creation, and Universal Governance Decision Adapter creation.

## Governance Proposal Integration

`initialize_governance_proposal_with_action_v1` now requires the registry for the requested module.

The instruction:

- derives the expected module from `GovernanceActionTypeV1`
- validates the provided registry
- requires `request.target_program == registry.program_id`
- stores the target program from the registry, not from caller-controlled data
- keeps `GovernanceProposalV1.action_type` as a compatibility mirror only

This means a proposal cannot bind a Treasury action to an unregistered module or to an arbitrary external program.

## Snapshot Integration

`create_governance_snapshot_v1` now requires:

- Security `GovernanceConfigV1`
- `ProtocolModuleRegistryV1`
- `GovernanceProposalActionV1`

Before a proposal enters `Voting`, the program validates the sidecar and registry together. Legacy proposals without a sidecar, or proposals whose module registry is wrong, disabled, or bound to the wrong program, cannot enter the new voting path.

## Adapter Integration

`create_governance_decision_adapter_v1` now requires `ProtocolModuleRegistryV1`.

The adapter still derives action, target, and payload from `GovernanceProposalActionV1`, but it also verifies that the sidecar module is currently registered and enabled for the current Alpha Protocol program.

The adapter does not accept caller-controlled action, target, program id, or payload hash.

## Explicit Non-Goals

This phase does not implement:

- registry update / enable / disable
- external program registration
- CPI to external modules
- Treasury USDC execution
- builder payout transfer
- Green Label DAO closure
- Victim Relief accounts
- Scam Registry accounts
- DAO Control Mode
- authority migration
- frontend changes
- deployment or chain transactions

Mainnet remains NO-GO.

## Stage 4 Treasury Execution Usage

Phase 2E-FINAL Stage 4 consumes the Treasury module registry before approving
or executing Treasury spending.

The strict Treasury wrappers validate:

- registry PDA and stable module code
- `module_id == Treasury`
- registry enabled
- registry bound to the current Alpha Protocol Program ID
- registry tied to the expected Security `GovernanceConfigV1`

Treasury execution still does not mutate the registry. Registry update,
enable/disable, external program registration, and DAO Control Mode remain
out of scope.

## Stage 5B-1 Green Label Certification Usage

Phase 2E-FINAL Stage 5B-1 consumes the Green Label module registry before any
strict certification wrapper can approve, reject, or revoke certification.

The strict Green Label wrappers validate:

- registry PDA and stable module code
- `module_id == GreenLabel`
- registry enabled
- registry bound to the current Alpha Protocol Program ID
- registry tied to the expected Security `GovernanceConfigV1`

The registry is not mutated. Green Label refund, slash, Treasury routing,
Victim Relief, Scam Registry, DAO Control Mode, and authority migration remain
out of scope for this stage.

## Stage 5B-2 Green Label Refund Usage

Phase 2E-FINAL Stage 5B-2 also consumes the Green Label module registry before
any strict refundable escrow refund wrapper can run.

The strict refund wrappers validate:

- registry PDA and stable module code
- `module_id == GreenLabel`
- registry enabled
- registry bound to the current Alpha Protocol Program ID
- registry tied to the expected Security `GovernanceConfigV1`

The registry is not mutated. Refund execution reads the registry as a module
allow-list only. The refund path does not route through Treasury and does not
modify module registry state.

## Stage 6B-1 Victim Relief Foundation Usage

Phase 2E-6B-1 adds Victim Relief foundation accounts, but it does not consume
`ProtocolModuleRegistryV1` for execution yet.

The existing `VictimRelief` module id and stable code remain reserved for the
future DAO decision path:

- `VictimReliefApproveCompensation`
- `VictimReliefRejectClaim`
- `VictimReliefUpdatePolicy`

Current Victim Relief instructions only initialize config / immutable policy,
submit cases, update evidence roots during the evidence window, cancel cases,
and expire stale cases. They do not approve compensation, reject claims, create
payout requests, or transfer relief vault USDC.

The registry is expected to become the allow-list for future Victim Relief DAO
execution, but that closure is intentionally deferred.
