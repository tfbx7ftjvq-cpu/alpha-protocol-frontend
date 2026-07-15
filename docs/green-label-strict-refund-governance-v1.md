# Green Label Strict Refund Governance V1

Phase 2E-FINAL Stage 5B-2 closes the strict DAO governance refund path for Green Label refundable escrow.

The implemented path is:

```text
GovernanceProposalV1
-> GovernanceProposalActionV1
-> GreenLabel ProtocolModuleRegistryV1
-> UniversalGovernanceDecisionAdapterV1
-> ProposalDecisionV1 Approved
-> ExecutionQueueItemV1 Executed
-> strict Green Label refund wrapper
-> refundable escrow vault
-> original payer USDC token account
-> GreenLabelRefundExecutionRecordV1
```

This phase only implements refund. It does not implement Green Label forfeit/slash, Treasury Router split, certification fee receipt, legacy slash closure, Victim Relief, Scam Registry, DAO Control Mode, authority migration, frontend changes, deployment, or chain transactions.

## Fund Path

Green Label refundable escrow funds are not protocol revenue.

Normal refund:

```text
GreenLabelRefundableEscrowV1 refundable vault
-> original payer USDC token account
```

The refund does not pass through:

- `route_usdc_revenue_v1`
- `RevenueRoutingStatsV1`
- `TreasuryUsdcStateV2`
- Treasury 50 / 20 / 20 / 10 split
- `TreasuryExecutionRecordV1`

The source is only the escrow's `refundable_vault`. The destination token account must belong to `GreenLabelRefundableEscrowV1.payer`.

The refund amount source is the escrow's recorded liability:

```text
refundable_amount - refunded_amount - forfeited_amount
```

This uses checked arithmetic and must be greater than zero. `refundable_vault.amount` is not the governance amount source.

## Execution Record

`GreenLabelRefundExecutionRecordV1` is immutable after creation and records:

- Security queue item
- Security proposal decision
- governance proposal
- governance proposal action sidecar
- Green Label module registry
- Green Label config
- Green Label project
- optional Green Label dispute
- refundable escrow
- refundable vault
- original payer
- payer destination token account
- refund amount
- USDC mint
- execution type
- governance action type
- refund parameters hash
- canonical governance payload hash
- escrow status before and after
- project status before and after
- executor
- executed timestamp
- schema version
- bump

The PDA is:

```text
[
  b"green_label_refund_execution_record_v1",
  execution_queue_item.key().as_ref()
]
```

One executed queue item can create only one refund execution record. The PDA plus escrow terminal status prevents replay.

## Refund Parameters Hash

`GreenLabelRefundParametersV1` binds the DAO decision to the real refund execution context.

The domain separator is:

```text
alpha_green_label_refund_parameters_v1
```

The hash covers:

- schema version
- Green Label config
- Green Label project
- Green Label dispute, or default pubkey for no-dispute refund
- refundable escrow
- refundable vault
- original payer
- payer destination token account
- refund amount
- USDC mint
- expected escrow status
- proposal id
- governance action type

The executor cannot pass the refund amount. The program derives the amount only from escrow stored fields. The current token account balance is checked only for sufficiency.

Third-party dust does not change the parameters hash. If extra compatible USDC is sent into the refundable vault after the proposal is created, the proposal remains valid because the voted amount is the recorded escrow liability, not the mutable token balance.

## Eligibility

The strict refund wrappers support two explicit paths.

No-dispute refund:

- project has no active dispute
- current timestamp is at or after `refund_available_after`
- escrow is `Locked` or `Refundable`
- refundable vault balance is greater than or equal to the recorded refund amount

Dispute refund:

- the project has the provided active dispute
- dispute is ready for decision or already decision queued
- Security Layer approved `GreenLabelRefund`
- execution queue has reached `Executed`
- escrow is `Locked` or `Refundable`
- refundable vault balance is greater than or equal to the recorded refund amount

If `refundable_vault.amount > recorded_refund_amount`, only the recorded amount is transferred. Excess token dust remains in the vault, is not returned to the payer, is not routed to Treasury, and is not counted in revenue stats.

If `refundable_vault.amount < recorded_refund_amount`, execution fails with insufficient funds.

V1 does not include a dust recovery instruction. Any future dust recovery must be designed as a separate governance-controlled flow and must not let an authority arbitrarily withdraw escrow excess.

Rejected certification does not auto refund. Revoked certification does not auto slash. Refund and slash are separate governance actions.

## Strict Validation

The refund wrappers reuse a shared strict governance validator. It checks:

- Security governance config is not paused
- Green Label config is not paused
- Green Label module registry is valid, enabled, and bound to this program
- `GovernanceProposalV1.status == Passed`
- `GovernanceProposalActionV1.action_type == GreenLabelRefundBond`
- sidecar module is `GreenLabel`
- sidecar target account is the refundable escrow
- sidecar target program is this Alpha Protocol program
- canonical governance payload hash recomputes from the sidecar
- refund parameters hash recomputes from real accounts
- `UniversalGovernanceDecisionAdapterV1` links the governance proposal to the Security decision
- `ProposalDecisionV1` is approved
- `ExecutionQueueItemV1` has status `Executed`
- queue action is `GreenLabelRefund`
- queue target, program, and payload match the sidecar and adapter
- escrow, vault, mint, payer, and destination token account match the bound parameters

The executor is permissionless and only pays transaction fees. The executor cannot choose the payer, destination, amount, vault, mint, action, target, or payload.

## State Changes

On success:

- escrow transfers the recorded refund amount to the original payer destination token account
- escrow status becomes `Refunded`
- escrow `refunded_amount` is updated
- project status becomes `Refunded`
- project terminal proposal, decision, queue, payload, and action fields are recorded
- if a dispute account is used, dispute status becomes `ResolvedRefund`
- an immutable refund execution record is created

The refund path does not touch Treasury revenue totals or Treasury execution records.

## Legacy Boundaries

The legacy paths remain for compatibility:

- `execute_green_label_refund`
- `refund_green_label_escrow_v1`
- `execute_green_label_slash`
- `forfeit_green_label_escrow_to_treasury_v1`

This phase does not claim legacy bypasses are solved. Legacy slash bypass remains unresolved. `DaoControlled` mode is not implemented.

## Mainnet Status

This is a local implementation milestone only. It is not Devnet-verified, not Mainnet-verified, and does not make token launch ready.

Mainnet and token launch remain NO-GO.
