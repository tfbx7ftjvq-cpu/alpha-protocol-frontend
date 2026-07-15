# Green Label Strict Forfeit Governance V1

Phase 2E-FINAL Stage 5B-3 closes the strict DAO governance forfeit path for Green Label refundable escrow.

The implemented path is:

```text
GovernanceProposalV1
-> GovernanceProposalActionV1
-> GreenLabel ProtocolModuleRegistryV1
-> GovernanceProposalV1::Passed
-> UniversalGovernanceDecisionAdapterV1
-> ProposalDecisionV1 Approved
-> ExecutionQueueItemV1 Executed
-> strict Green Label forfeit wrapper
-> refundable escrow vault
-> Treasury USDC revenue router
-> 50 / 20 / 20 / 10 Treasury split
-> GreenLabelForfeitExecutionRecordV1
```

This phase implements only strict Green Label forfeit governance and legacy slash / forfeit shutdown. It does not implement new refund behavior, certification fee receipts, revenue split changes, Victim Relief, Scam Registry, DAO Control Mode, authority migration, frontend changes, deployment, or chain transactions.

## Fund Path

Green Label refundable escrow is not protocol revenue while it remains refundable. It becomes protocol revenue only after a final DAO-governed slash / forfeit decision is approved, queued, executed, and consumed by the strict forfeit wrapper.

On successful strict forfeit:

```text
GreenLabelRefundableEscrowV1 refundable vault
-> route_usdc_revenue_from_token_account
-> RevenueType::GreenLabelForfeitedBond
-> Treasury V2 relief / buyback / builders / staking USDC vaults
```

The Treasury split remains:

- 50% Relief Pool
- 20% Buyback / Burn vault
- 20% Builders / Contributors vault
- 10% Staking Rewards vault

Staking receives the integer remainder, so the four amounts always sum to the recorded forfeited amount.

## Recorded Amount Rule

The forfeit amount is derived only from escrow state:

```text
refundable_amount - refunded_amount - forfeited_amount
```

The executor cannot choose the amount. The program does not use `refundable_vault.amount` as the voted amount. The vault balance is checked only for sufficiency:

- if `vault.amount >= recorded_forfeit_amount`, execution may proceed
- if `vault.amount < recorded_forfeit_amount`, execution fails
- if `vault.amount > recorded_forfeit_amount`, only the recorded amount routes to Treasury

Third-party dust does not increase the forfeit amount, does not change the governance payload, and does not block execution. Excess dust remains in the refundable vault. V1 intentionally has no dust sweep instruction.

## Governance Parameters Hash

`GreenLabelForfeitParametersV1` binds the DAO decision to the concrete business and fund path.

The domain separator is:

```text
alpha_green_label_forfeit_parameters_v1
```

The hash covers:

- schema version
- Green Label config
- Green Label project
- Green Label dispute
- refundable escrow
- refundable vault
- recorded forfeited amount
- Treasury config
- Treasury USDC state
- revenue routing stats
- relief / buyback / builders / staking vaults
- USDC mint
- `RevenueType::GreenLabelForfeitedBond`
- expected escrow, project, and dispute statuses
- `GovernanceActionTypeV1::GreenLabelSlashBond`
- governance proposal id

The strict wrapper recomputes this hash from real accounts and requires it to equal `GovernanceProposalActionV1.parameters_hash`.

## Strict Validation

The forfeit wrapper verifies:

- Green Label config is not paused
- Security governance config is not paused
- Green Label module registry is valid, enabled, and bound to the Alpha Protocol program
- `GovernanceProposalV1.status == Passed`
- typed sidecar exists and is bound to the proposal
- sidecar action is `GreenLabelSlashBond`
- sidecar module is `GreenLabel`
- sidecar target program is the current Alpha Protocol program
- sidecar target account is the refundable escrow
- adapter links the governance proposal to the Security decision
- proposal decision is approved and has `ProposalType::GreenLabelSlash`
- execution queue item is `Executed`
- queue action is `ActionType::GreenLabelSlash`
- queue target, program, and payload match the sidecar and adapter
- a valid active Green Label dispute exists
- dispute is ready for decision or decision queued
- escrow has not already been refunded or forfeited
- refundable vault mint and owner match the escrow
- Treasury router accounts match the configured USDC mint, PDAs, and vault authority
- revenue routing stats use the same USDC mint and Treasury authority

The executor is permissionless and only pays transaction fees. The executor cannot change action, target, amount, vaults, mint, split, or revenue type.

## Execution Record

`GreenLabelForfeitExecutionRecordV1` is immutable after creation and records:

- execution queue item
- proposal decision
- governance proposal
- governance proposal action sidecar
- module registry
- Green Label config / project / dispute
- refundable escrow and vault
- Treasury config / state / revenue routing stats
- four Treasury destination vaults
- forfeited amount
- USDC mint
- `RevenueType::GreenLabelForfeitedBond`
- execution type `Forfeit`
- governance action `GreenLabelSlashBond`
- parameters hash
- canonical governance payload hash
- escrow, project, and dispute statuses before and after
- executor
- executed timestamp
- schema version
- bump

The PDA is:

```text
[
  b"green_label_forfeit_execution_record_v1",
  execution_queue_item.key().as_ref()
]
```

One executed queue item can create only one business forfeit record. The same escrow can finish as either refunded or forfeited, not both.

Certification fee receipts are separate from refundable escrow. A Green Label bond forfeit routes only remaining refundable escrow liability as `RevenueType::GreenLabelForfeitedBond`; it must not route the already-paid non-refundable certification fee a second time. See `docs/green-label-certification-fee-policy-and-receipt-v1.md`.

## Legacy Shutdown

The legacy slash / forfeit public entry points are disabled:

- `execute_green_label_slash` returns `LegacyGreenLabelSlashDisabled`
- `forfeit_green_label_escrow_to_treasury_v1` returns `LegacyGreenLabelForfeitDisabled`

Legacy unit-level validators remain in the codebase for compatibility and historical tests, but they are no longer public execution paths for Green Label slash / forfeit.

## Explicit Non-Goals

This phase does not implement:

- SOL revenue routing
- Builders payout
- DAO voting changes
- DAO Control Mode
- Victim Relief
- Scam Registry
- frontend
- Mainnet deployment
- chain transaction scripts
- dust recovery

Mainnet and token launch remain NO-GO.
