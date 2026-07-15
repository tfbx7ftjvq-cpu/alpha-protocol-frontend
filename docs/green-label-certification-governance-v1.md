# Green Label Certification Governance V1

## Purpose

Phase 2E-FINAL Stage 5B-1 closes the strict DAO governance path for Green Label certification decisions.

The path is:

```text
GovernanceProposalV1
-> GovernanceProposalActionV1
-> Snapshot / Vote / Passed
-> UniversalGovernanceDecisionAdapterV1
-> ProposalDecisionV1 Approved
-> ExecutionQueueItemV1 Executed
-> strict Green Label certification wrapper
-> GreenLabelCertificationStateV1
-> GreenLabelCertificationExecutionRecordV1
```

This phase only implements certification state updates. It does not implement refund, forfeit, slash, certification fee receipt, Treasury transfer, Victim Relief, Scam Registry, DAO Control Mode, authority migration, frontend changes, deployment, or chain transactions.

## Certification State

`GreenLabelCertificationStateV1` is the strict-path source of truth for certification status.

It stores:

- `green_label_project`
- `green_label_config`
- `certification_status`
- `last_governance_proposal`
- `last_execution_queue`
- `last_execution_record`
- `last_action_type`
- `decision_at`
- `created_at`
- `updated_at`
- `schema_version`
- `bump`

The PDA is:

```text
[
  b"green_label_certification_state_v1",
  green_label_project.key().as_ref()
]
```

`schema_version` is currently `1`. One Green Label project can have only one certification state sidecar.

`GreenLabelProjectV1.status` remains a compatibility and bond lifecycle field. Strict certification queries should read `GreenLabelCertificationStateV1.certification_status`.

## Certification Status

`GreenLabelCertificationStatusV1` is append-only and currently supports:

- `Pending`
- `Approved`
- `Rejected`
- `Revoked`

`Rejected` and `Revoked` were intentionally not added to `GreenLabelStatus`. `GreenLabelStatus::Cancelled` keeps its legacy meaning and is not reused as strict certification rejection or revocation.

## Initialization

`initialize_green_label_certification_state_v1` initializes the sidecar.

The payer only pays rent. The program writes project/config links from real accounts, sets certification status to `Pending`, sets audit fields to empty defaults, reads timestamps from `Clock`, and stores the bump.

Initialization is allowed only while the project is:

- `PendingBondDeposit`
- `PendingObservation`

Legacy `ActiveGreenLabel`, terminal, cancelled, refunded, or slashed projects are not auto-migrated. Old Devnet `ActiveGreenLabel` projects require a separate migration plan if they need strict certification state.

## Execution Record

`GreenLabelCertificationExecutionRecordV1` is immutable after creation and records:

- `execution_queue_item`
- `proposal_decision`
- `governance_proposal`
- `governance_proposal_action`
- `green_label_project`
- `certification_state`
- `module_registry`
- `execution_type`
- `governance_action_type`
- `target_account`
- `parameters_hash`
- `canonical_governance_payload_hash`
- before/after project status
- before/after certification status
- executor
- executed timestamp
- schema version
- bump

The PDA is:

```text
[
  b"green_label_certification_execution_record_v1",
  execution_queue_item.key().as_ref()
]
```

One executed queue item can create only one certification execution record. The PDA plus status transition checks prevent replay.

## Execution Type Stable Codes

`GreenLabelCertificationExecutionTypeV1` supports:

| Execution Type | Stable Code |
| --- | ---: |
| Approve | 1 |
| Reject | 2 |
| Revoke | 3 |

The protocol uses exhaustive helper functions and does not rely on `enum as u8`.

## Decision Parameters Hash

`GreenLabelCertificationDecisionParametersV1` binds the DAO decision to the real certification execution context.

The domain separator is:

```text
alpha_green_label_certification_decision_v1
```

The hash includes:

- schema version
- Green Label config
- Green Label project
- certification state
- governance action type
- project authority
- bond tier
- bond vault
- USDC mint
- observation end timestamp
- expected project status
- expected certification status
- proposal id

Changing action, project, state, authority, vault, mint, observation timestamp, expected status, or proposal id changes the hash.

## Shared Governance Validation

Strict wrappers reuse a shared certification execution validator. It checks:

- `GovernanceProposalV1.status == Passed`
- `GovernanceProposalActionV1.module_id == GreenLabel`
- wrapper action matches the sidecar action
- target program is the current Alpha Protocol Program ID
- target account is the Green Label project
- `ProtocolModuleRegistryV1` is the enabled Green Label module bound to the expected Security governance config
- `UniversalGovernanceDecisionAdapterV1` matches the proposal, decision, action, target, and canonical payload hash
- `ProposalDecisionV1` is approved and uses the expected Green Label proposal type
- `ExecutionQueueItemV1` has already reached `Executed`
- queue action, target, program, and payload hash match the DAO sidecar
- certification parameters hash is recomputed from real accounts and matches the sidecar
- certification state is linked to the project/config and has schema version `1`

The executor is permissionless, but cannot control action, target, parameters hash, canonical payload hash, expected status, project authority, bond vault, or observation timestamp.

## Approve Certification

`execute_green_label_approve_certification_v1` requires:

- `GovernanceActionTypeV1::GreenLabelApproveCertification`
- certification state is `Pending`
- project status is `PendingObservation`
- current time is at or after `observation_end_ts`
- no active unresolved dispute
- Green bond vault, vault owner, USDC mint, and vault balance match the project/config
- Security decision and execution queue already completed

Success atomically:

- sets `GreenLabelProjectV1.status` to `ActiveGreenLabel`
- sets certification state to `Approved`
- writes last proposal/queue/record/action/timestamps
- creates an immutable execution record

It does not transfer tokens, collect a fee, refund bond, or slash bond.

## Reject Certification

`execute_green_label_reject_certification_v1` requires:

- `GovernanceActionTypeV1::GreenLabelRejectCertification`
- certification state is `Pending`
- project status is `PendingBondDeposit` or `PendingObservation`

Success:

- sets certification state to `Rejected`
- updates audit fields
- creates an immutable execution record
- leaves `GreenLabelProjectV1.status` unchanged

Reject is not a refund. If any user funds are in escrow, a separate refund governance action must handle them later.

## Revoke Certification

`execute_green_label_revoke_certification_v1` requires:

- `GovernanceActionTypeV1::GreenLabelRevokeCertification`
- certification state is `Approved`
- project status is `ActiveGreenLabel`

Success:

- sets certification state to `Revoked`
- updates audit fields
- creates an immutable execution record
- leaves `GreenLabelProjectV1.status` unchanged for compatibility

Revoke is not a slash. Refund or slash must be handled by separate governance actions.

## Legacy Boundaries

The legacy Green Label paths still exist, including:

- `execute_green_label_refund`
- `execute_green_label_slash`
- `refund_green_label_escrow_v1`
- `forfeit_green_label_escrow_to_treasury_v1`

Stage 5B-3 closes the legacy slash / non-strict forfeit value-moving paths while retaining the instruction names for ABI and Devnet history:

- `execute_green_label_slash` now fails closed with `LegacyGreenLabelSlashDisabled`
- `forfeit_green_label_escrow_to_treasury_v1` now fails closed with `LegacyGreenLabelForfeitDisabled`
- the Mainnet strict forfeit path is `execute_green_label_forfeit_governance_v1`

`DaoControlled` mode is not implemented.

## Mainnet Status

This is a local implementation milestone only. It is not Devnet-verified, not Mainnet-verified, and does not make token launch ready.

Mainnet and token launch remain NO-GO.

## Stage 5B-2 Refund Boundary

Phase 2E-FINAL Stage 5B-2 adds a separate strict governance path for Green Label refundable escrow refunds.

Certification decisions remain separate from refund decisions:

- reject certification does not auto refund
- revoke certification does not auto slash
- refund must use `GovernanceActionTypeV1::GreenLabelRefundBond`
- refund target account is the refundable escrow, not the project or dispute
- refund parameters bind the original payer, destination token account, vault, mint, amount, and queue

The refund path does not collect certification fees and does not route funds through Treasury 50 / 20 / 20 / 10 split.
