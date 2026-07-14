# Treasury Governance Foundation Design

## 1. Treasury Governance Goal

Phase 2E-5A-2 adds the on-chain foundation for DAO-governed Treasury requests.
It does not move USDC and does not execute builders payouts.

The goal is to create auditable request accounts that can later be connected to:

```text
DAO vote
-> Security Layer ProposalDecisionV1
-> ExecutionQueueItemV1
-> Treasury execution instruction
-> PDA-signed USDC transfer
```

This phase only establishes the request and approval-recording layer.

## 2. TreasuryConfigV2 vs TreasuryGovernanceConfigV1

`TreasuryConfigV2` remains the canonical Treasury configuration for mint identity:

- `authority`
- `usdc_mint`
- `alpha_mint`
- `bump`

`TreasuryGovernanceConfigV1` is a governance sidecar and does not replace
`TreasuryConfigV2`.

It records:

- `treasury_config`
- `security_authority`
- `dao_enabled`
- `spending_limit_usdc`
- `split_change_threshold_bps`
- `emergency_mode`
- `created_at`
- `updated_at`
- `bump`

The Treasury vault authority remains a PDA. It must not be migrated to a normal
wallet.

## 3. Spending Request Lifecycle

`TreasurySpendingRequestV1` records ordinary DAO-reviewed Treasury spending
requests.

Fields include:

- `request_id`
- `treasury_config`
- `proposer`
- `recipient`
- `amount_usdc`
- `purpose_hash`
- `proposal_id`
- `status`
- `created_at`
- `executed_at`
- `bump`

Status flow:

```text
Pending
-> Approved
-> Executed
```

Rejected and Cancelled are terminal request outcomes.

In this phase, `Approved` only means the Security Layer execution record matched
the request payload. It does not transfer funds.

## 4. Builder Payout Lifecycle

`TreasuryBuilderPayoutGovernanceV1` connects contributor work to future Treasury
payout execution.

It links:

- `BuilderPayoutRequestV1`
- `ContributorRegistryV1`
- `ContributorMilestoneV1`
- recipient wallet
- requested USDC amount
- Security Layer proposal id

Status flow:

```text
Pending
-> Approved
-> Executed
```

Rejected is a terminal governance outcome.

The foundation phase did not transfer USDC from the builders vault. Phase
2E-FINAL Stage 4 adds the strict execution wrappers that consume these approved
request accounts.

## 5. Future Execution Flow

The intended final payout path is:

```text
Contributor
-> Milestone
-> BuilderPayoutRequestV1
-> DAO vote
-> Universal Governance Decision Adapter
-> ProposalDecisionV1
-> ExecutionQueueItemV1
-> Treasury builder payout execution
-> builders_usdc_vault
-> contributor destination USDC token account
```

Future Treasury execution instructions must verify:

- `proposal_id`
- `ActionType`
- target account
- `payload_hash`
- `ExecutionStatus::Executed`
- vault mint and PDA authority
- sufficient builders vault balance
- request not already executed

## 6. Explicit Non-Goals

Not implemented in this phase:

- Treasury revenue split changes
- dynamic split config
- batch payout
- frontend UI
- Mainnet deployment

Token launch remains NO-GO until Treasury payout governance and Mainnet authority
migration are completed and reviewed.

## 7. Stage 4 Execution Layer

Phase 2E-FINAL Stage 4 adds:

- `TreasuryExecutionTypeV1`
- `TreasuryExecutionRecordV1`
- `execute_treasury_builder_payout_v1`
- `execute_treasury_spending_v1`

Both wrappers are strict and only source funds from `builders_usdc_vault`.
There is no generic public Treasury transfer instruction and no caller-selected
source vault.

Approval now binds the request to exact Stage 4 parameters:

- request account
- amount
- recipient owner
- recipient USDC token account
- source vault
- USDC mint
- proposal id

The wrappers re-check the same parameters hash before performing
`transfer_checked`. `spending_limit_usdc` is interpreted as a per-request limit
in V1.
