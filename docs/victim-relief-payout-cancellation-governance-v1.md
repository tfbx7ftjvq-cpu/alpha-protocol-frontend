# Victim Relief Payout Cancellation Governance V1

Phase 2E-6B-4B-4B adds a terminal DAO-controlled cancellation path for Victim Relief payout requests that were approved but not yet paid.

This phase does not transfer USDC, does not migrate recipients, does not reserve vault liquidity, does not add a generic payout or cancel entrypoint, and does not update Treasury revenue accounting.

## Scope

Cancellation applies only to `ReliefPayoutRequestV1` accounts with:

- `status = Approved`
- `executed_at = 0`
- linked `VictimReliefCaseV1.status = PayoutQueued`

The canonical governance target is the `ReliefPayoutRequestV1` account. The case, recipient token account, relief vault, or arbitrary target cannot be used as the cancellation target.

## Governance Action

The DAO action is appended as:

```text
GovernanceActionTypeV1::VictimReliefCancelPayout
```

The matching Security Layer types are appended as:

```text
ProposalType::VictimReliefCancelPayout
ActionType::VictimReliefCancelPayout
```

The action maps to `ProtocolModuleIdV1::VictimRelief`. Existing enum variants are append-only and must not be reordered.

## Strict Wrappers

There are two source-specific wrappers:

- `execute_cancel_original_victim_relief_payout_v1`
- `execute_cancel_overturn_victim_relief_payout_v1`

The original wrapper accepts only the original DAO approve source:

```text
VictimReliefApproveCompensation
-> VictimReliefDecisionExecutionRecordV1::Approve
-> ReliefPayoutRequestV1::Approved
-> VictimReliefCancelPayout
-> ReliefPayoutRequestV1::Cancelled
```

The appeal wrapper accepts only the appeal overturn source:

```text
VictimReliefOverturnAppeal
-> VictimReliefAppealDecisionExecutionRecordV1::Overturn
-> ReliefPayoutRequestV1::Approved
-> VictimReliefCancelPayout
-> ReliefPayoutRequestV1::Cancelled
```

There is no generic cancellation wrapper. Authority and guardian wallets cannot single-sign cancellation.

## Parameters Hash

`VictimReliefPayoutCancellationParametersV1` is hashed under:

```text
alpha_victim_relief_payout_cancellation_parameters_v1
```

The hash binds:

- payout origin
- payout request
- victim relief case
- config, policy, and policy version
- original authorization action, proposal, decision, queue, action sidecar, receipt, and parameters hash
- evidence snapshot
- approved amount
- recipient owner and token account
- Treasury config, relief vault, and USDC mint
- cancellation proposal, decision, queue, and action sidecar
- expected request status `Approved`
- expected case status `PayoutQueued`

Executor address, dynamic vault balance, recipient balance, and free-text cancellation reasons are not included. Cancellation reasons should live in off-chain governance evidence and be committed through the governance action evidence hash.

## Receipt

`VictimReliefPayoutCancellationRecordV1` is immutable and one-per-request.

PDA:

```text
relief_payout_cancel_rcpt_v1 + relief_payout_request
```

The receipt records the original payout authorization source, the cancellation governance source, the frozen amount and recipient, the cancellation parameters hash, the canonical governance payload hash, executor, timestamp, schema version, and bump.

No update, close, public initializer, or receipt rewrite path exists.

## State Changes

Successful cancellation performs:

- `ReliefPayoutRequestV1.status: Approved -> Cancelled`
- `ReliefPayoutRequestV1.executed_at` remains `0`
- `VictimReliefCaseV1.status: PayoutQueued -> Cancelled`
- claimant `active_case_count` decrements once with checked arithmetic
- cancellation receipt is written

It does not:

- transfer USDC
- create a payout receipt
- mutate Treasury totals or `RevenueRoutingStatsV1`
- modify approved amount, recipient, original authorization refs, evidence snapshot, or policy refs
- reactivate the request

After cancellation, a claimant must create a new case or wait for a future V2 migration design. V1 does not support in-place recipient migration.

## Pause Semantics

Payout wrappers still fail while Victim Relief or Security governance is paused before transfer.

Cancellation is non-fund-moving and risk-reducing. If the cancellation queue has already been executed by the Security Layer, the strict cancellation wrapper can complete even while payout execution is paused. A Security pause can still prevent a not-yet-executed cancellation queue from progressing; that governance hardening remains a future phase.

Stage 6B-4B-4C-B1 formalizes Victim Relief module pause governance in [victim-relief-module-pause-governance-v1.md](victim-relief-module-pause-governance-v1.md). The cancellation pause exception remains intentional: module pause blocks new risk-increasing activity and payouts, but does not block an already-authorized cancellation that transfers no USDC and closes an unpaid request.

## Race Model

Payout and cancellation are mutually exclusive through `ReliefPayoutRequestV1.status` and atomic account writes:

```text
Approved -> Executed
Approved -> Cancelled
```

Whichever transaction lands first determines the terminal state. The second path fails on status checks. Payout receipts and cancellation receipts use different PDAs, but V1 relies on the shared mutable request and case status rather than optional receipt probing.

## Current Limits

V1 intentionally does not implement:

- recipient migration
- FIFO or reservation accounting
- payout statistics account
- generic cancellation
- authority or guardian cancellation
- USDC transfer during cancellation
- Devnet or Mainnet deployment

Local tests do not equal Devnet or Mainnet verification. Mainnet and token launch remain NO-GO until strict E2E, authority migration, operational controls, legal/risk review, and Mainnet safety checks are complete.
