# Treasury Execution Layer V1

## Purpose

Phase 2E-FINAL Stage 4 closes the first DAO-controlled Treasury spending loop for Alpha Protocol.

The supported V1 execution paths are intentionally narrow:

- Builders / contributors payout
- Treasury operating spending

Both paths move USDC from the fixed `builders_usdc_vault` to a DAO-approved recipient USDC token account after the Governance, Universal Adapter, Security Layer, and Treasury request approval path has completed.

## Full Execution Path

```text
GovernanceProposalV1
-> GovernanceProposalActionV1
-> Snapshot / Vote / Passed
-> UniversalGovernanceDecisionAdapterV1
-> ProposalDecisionV1
-> ExecutionQueueItemV1 Executed
-> Approved Treasury Request
-> Strict Treasury Wrapper
-> builders_usdc_vault
-> recipient USDC token account
-> TreasuryExecutionRecordV1
```

## No Generic Treasury Transfer

There is no public arbitrary transfer instruction.

The program does not expose:

```text
execute_treasury_transfer_v1
```

Callers cannot choose a source vault. V1 only exposes:

- `execute_treasury_builder_payout_v1`
- `execute_treasury_spending_v1`

Both wrappers hard-code the source to `builders_usdc_vault`. Relief, buyback, and staking vault substitution is rejected by account seeds and token-account validation.

## TreasuryExecutionTypeV1

`TreasuryExecutionTypeV1` currently supports:

- `BuilderPayout`
- `TreasurySpending`

Stable execution codes are helper-based:

- `BuilderPayout = 1`
- `TreasurySpending = 2`

The protocol does not use enum casting as a stable wire encoding.

## TreasuryExecutionRecordV1

Each successful Treasury execution creates an immutable receipt:

- `queue_item`
- `proposal_decision`
- `governance_proposal`
- `governance_proposal_action`
- `request_account`
- `module_id`
- `execution_type`
- `source_vault`
- `recipient_owner`
- `recipient_token_account`
- `amount_usdc`
- `usdc_mint`
- `parameters_hash`
- `canonical_governance_payload_hash`
- `executor`
- `executed_at`
- `schema_version`
- `bump`

PDA:

```text
[
  b"treasury_execution_record_v1",
  execution_queue_item.key().as_ref()
]
```

One `ExecutionQueueItemV1` can create only one Treasury execution record. If `transfer_checked` fails, the Solana transaction rolls back atomically and the record is not created.

## Parameters Hash Binding

`GovernanceProposalActionV1.parameters_hash` must bind the exact business parameters that will later be paid.

Builder payout parameters include:

- treasury config
- `TreasuryBuilderPayoutGovernanceV1`
- `BuilderPayoutRequestV1`
- milestone
- recipient owner
- recipient USDC token account
- amount
- source vault
- USDC mint
- proposal id

Treasury spending parameters include:

- treasury config
- `TreasurySpendingRequestV1`
- recipient owner
- recipient USDC token account
- amount
- source vault
- USDC mint
- purpose hash
- proposal id

Domain separators:

```text
alpha_treasury_builder_payout_parameters_v1
alpha_treasury_spending_parameters_v1
```

The approval and execution wrappers rebuild these payloads from real accounts and reject mismatches.

## Approval Tightening

`approve_treasury_spending_request_v1` and `approve_treasury_builder_payout_governance_v1` now validate the typed governance path and recomputed Stage 4 parameters hash.

Approval requires:

- Treasury module registry enabled for the current program
- passed governance proposal and typed action sidecar
- Universal adapter tied to the same proposal and decision
- `ProposalDecisionV1` approved
- `ExecutionQueueItemV1` executed
- queue action, target, and canonical governance payload hash matching the sidecar
- recomputed Treasury parameters hash matching `GovernanceProposalActionV1.parameters_hash`

The older opaque Treasury governance payload hash helpers remain for compatibility tests and historical documentation, but new approvals use the typed parameters hash.

## Builder Payout Execution

`execute_treasury_builder_payout_v1` is permissionless for the executor, but the executor does not control amount, recipient, action, target, or source vault.

It requires:

- `TreasuryBuilderPayoutGovernanceV1.status == Approved`
- linked `BuilderPayoutRequestV1.status == Approved`
- linked milestone status `Approved`
- exact request, milestone, recipient, amount, source vault, mint, and proposal id binding
- sufficient builders vault balance

On success:

- transfers USDC with `transfer_checked`
- creates `TreasuryExecutionRecordV1`
- sets Treasury payout governance to `Executed`
- sets builder payout request to `Executed`
- sets milestone to `Paid`

## Treasury Spending Execution

`execute_treasury_spending_v1` is also permissionless for the executor.

It requires:

- `TreasurySpendingRequestV1.status == Approved`
- amount greater than zero
- amount at or below `TreasuryGovernanceConfigV1.spending_limit_usdc`
- non-zero purpose hash
- exact request, recipient, amount, source vault, mint, and proposal id binding
- sufficient builders vault balance

On success:

- transfers USDC with `transfer_checked`
- creates `TreasuryExecutionRecordV1`
- sets the spending request to `Executed`
- writes `executed_at`

## Accounting Semantics

`TreasuryUsdcStateV2.builders_usdc_total` is treated as cumulative routed inflow accounting. Stage 4 does not reduce it during payout.

Current spendable balance is the SPL token balance of `builders_usdc_vault`. Execution history is represented by immutable `TreasuryExecutionRecordV1` accounts.

## Explicit Non-Goals

Stage 4 does not implement:

- public arbitrary Treasury transfer
- caller-selected source vault
- Green Label refund or slash changes
- Victim Relief payout
- buyback spending
- staking rewards spending
- revenue split update
- batch payout
- registry mutation
- DAO Control Mode
- authority migration
- external CPI
- frontend changes
- deployment or chain transactions

Token launch remains NO-GO.
