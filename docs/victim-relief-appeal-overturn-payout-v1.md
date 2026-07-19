# Victim Relief Appeal Overturn Payout V1

## Scope

Phase 2E-6B-4B-3 implements the strict appeal-overturn Victim Relief payout path:

```text
VictimReliefOverturnAppeal
-> VictimReliefAppealDecisionExecutionRecordV1::Overturn
-> ReliefPayoutRequestV1::Approved
-> exact USDC transfer from relief_usdc_vault
-> ReliefPayoutRequestV1::Executed
-> VictimReliefCaseV1::Paid
-> claimant active case count decrement
-> ReliefPayoutExecutionRecordV1
```

This phase adds only the appeal-overturn payout wrapper. It does not add a generic payout instruction, payout cancellation, partial payout, recipient migration, reservation / fair ordering, payout stats, frontend integration, Devnet deployment, or Mainnet deployment.

## Public Instruction

The only public entrypoint added in this phase is:

```text
execute_victim_relief_overturn_payout_v1
```

It accepts no instruction arguments. The executor signer pays transaction fees and receipt rent, but cannot choose amount, recipient, source vault, payout origin, governance action, parameters hash, policy version, appeal id, token mint, or payout source.

The claimant, recipient, authority, and guardian do not sign.

## Fixed Origin And Action

The handler fixes:

- `VictimReliefPayoutOriginV1::AppealOverturn`
- `GovernanceActionTypeV1::VictimReliefOverturnAppeal`

It calls `validate_victim_relief_appeal_overturn_authorization_v1` and rejects original approve receipts, reject receipts, uphold appeal receipts, wrong appeal accounts, wrong original reject receipts, wrong action, wrong target, wrong proposal, wrong decision, wrong queue, and caller-selected origin or action.

## Required Appeal Authorization

Appeal payout requires both:

- the original reject receipt, `VictimReliefDecisionExecutionRecordV1::Reject`
- the appeal overturn receipt, `VictimReliefAppealDecisionExecutionRecordV1::Overturn`

The appeal must be `Overturned`, the original reject receipt must belong to the same case and original evidence snapshot, and the appeal authorization receipt must bind the same proposal, decision, queue, target appeal account, parameters hash, recipient, and amount as the payout request.

## Common Payout Validation

Before transfer, the shared payout validator checks pause state, request status, case status, claimant active count, config / policy / snapshot bindings, Treasury config, fixed relief vault PDA, vault authority PDA, USDC mint and decimals, exact request amount, frozen recipient token account, sufficient vault balance, and empty payout receipt PDA.

`PayoutRequest Approved`, `PayoutQueued`, and `ExecutionQueueItemV1::Executed` are not payment. Payment completion requires the token transfer, request `Executed`, case `Paid`, and `ReliefPayoutExecutionRecordV1`.

## Exact USDC Transfer

The transfer reuses the private relief-vault transfer helper:

- source: fixed `relief_usdc_vault`
- authority: `vault_authority_v2` PDA signer
- destination: frozen request recipient token account
- amount: frozen `ReliefPayoutRequestV1.approved_amount_usdc`
- decimals: protocol USDC decimals
- method: SPL Token `transfer_checked`

There is no partial payout, overpay, underpay, split payout, native SOL payout, Revenue Router call, or generic Treasury vault transfer.

## State Changes

After successful transfer:

- `ReliefPayoutRequestV1.status = Executed`
- `ReliefPayoutRequestV1.executed_at = now`
- `VictimReliefCaseV1.status = Paid`
- `VictimReliefCaseV1.updated_at = now`
- claimant `active_case_count` decrements once
- claimant `total_case_count` does not change
- `ReliefPayoutExecutionRecordV1` is written immutably

The appeal account remains `Overturned`. Its `resolved_at`, `decision_proposal`, and `decision_queue` are not modified.

## Receipt

The payout receipt PDA is:

```text
relief_payout_rcpt_v1 + relief_payout_request
```

The receipt records origin `AppealOverturn`, action `VictimReliefOverturnAppeal`, the appeal authorization receipt, exact amount, frozen recipient, relief vault, authorization parameters hash, payout parameters hash, executor, timestamp, schema, and bump.

Executor is recorded for audit but is not part of the canonical payout hash.

## Accounting Boundary

Victim Relief payout is a relief-vault outflow. It does not mutate:

- `TreasuryUsdcStateV2`
- `RevenueRoutingStatsV1`
- `relief_usdc_total`
- builders / buyback / staking revenue totals
- Green Label accounting

Payout is not a protocol revenue reversal.

## Retry And Isolation

If paused, underfunded, or the recipient token account is invalid, execution fails before state changes. The request remains `Approved`, the case remains `PayoutQueued`, the appeal remains `Overturned`, and the payout can be retried after the blocking condition is fixed.

The original approve payout wrapper remains separate and accepts only `OriginalApprove` / `VictimReliefApproveCompensation`. The appeal wrapper accepts only `AppealOverturn` / `VictimReliefOverturnAppeal`. There is no generic payout wrapper.

Local tests are not Devnet or Mainnet verification. Mainnet production and token launch remain NO-GO.
