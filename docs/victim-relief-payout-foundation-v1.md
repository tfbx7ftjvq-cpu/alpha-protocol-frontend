# Victim Relief Payout Foundation V1

## Scope

Phase 2E-6B-4B-1 adds the read-safe foundation for future Victim Relief payout execution:

- `VictimReliefPayoutOriginV1`
- `VictimReliefPayoutParametersV1`
- canonical payout parameters hash
- immutable `ReliefPayoutExecutionRecordV1`
- strict original-approve authorization validator
- strict appeal-overturn authorization validator
- common request / case / vault / recipient / pause / balance validator
- receipt write helper for future strict payout wrappers

This phase does not add a public payout instruction, does not transfer USDC, does not mark requests `Executed`, does not mark cases `Paid`, and does not decrement claimant active case count.

Stage 6B-4B-2 adds the first strict payout wrapper for the original DAO approve path. See [victim-relief-original-approved-payout-v1.md](victim-relief-original-approved-payout-v1.md).

Stage 6B-4B-3 adds the second strict payout wrapper for the appeal overturn path. See [victim-relief-appeal-overturn-payout-v1.md](victim-relief-appeal-overturn-payout-v1.md).

## Valid Payout Origins

There are only two valid V1 payout authorization origins.

Original approve:

```text
VictimReliefApproveCompensation
-> VictimReliefDecisionExecutionRecordV1::Approve
-> ReliefPayoutRequestV1
```

Appeal overturn:

```text
VictimReliefOverturnAppeal
-> VictimReliefAppealDecisionExecutionRecordV1::Overturn
-> ReliefPayoutRequestV1
```

Reject, uphold, policy update, guardian authority, generic Treasury spending, caller-selected amount, and caller-selected recipient are not payout origins.

## Strict Wrapper Model

Payout execution uses separate strict public wrappers:

- `execute_victim_relief_approved_payout_v1`
- `execute_victim_relief_overturn_payout_v1`

Stage 6B-4B-2 implements `execute_victim_relief_approved_payout_v1` for the original approve path only. Stage 6B-4B-3 implements `execute_victim_relief_overturn_payout_v1` for the appeal overturn path only. The generic public payout model is intentionally avoided because it increases optional-account, unchecked-account, receipt-type, and action-confusion risk.

## Payout Origin Stable Codes

`VictimReliefPayoutOriginV1` uses explicit stable codes:

- `OriginalApprove = 1`
- `AppealOverturn = 2`

The program does not use `enum as u8` as a protocol code. Unknown codes fail.

## Payout Parameters Hash

`VictimReliefPayoutParametersV1` is hashed under:

```text
alpha_victim_relief_payout_parameters_v1
```

The hash binds:

- payout origin
- payout request
- case / config / policy / policy version
- authorization action type
- governance proposal / decision / queue / sidecar
- authorization execution receipt
- evidence snapshot
- exact approved amount
- recipient owner and recipient token account
- Treasury config
- relief USDC vault
- vault authority V2
- USDC mint
- authorization parameters hash

The executor does not enter the canonical payout hash. Dynamic vault balances, recipient balances, and pre/post transfer balances do not enter the hash.

## Authorization Hash vs Payout Hash

`ReliefPayoutRequestV1.parameters_hash` remains the authorization hash:

- original approve uses `VictimReliefDecisionParametersV1`
- appeal overturn uses `VictimReliefAppealDecisionParametersV1`

`ReliefPayoutExecutionRecordV1.payout_parameters_hash` proves the exact payout execution binding. This separation keeps DAO authorization and token transfer evidence distinct.

## Payout Execution Receipt

`ReliefPayoutExecutionRecordV1` is the proof of actual relief payout execution once a strict payout wrapper writes it after the exact transfer.

PDA:

```text
relief_payout_rcpt_v1 + relief_payout_request
```

The receipt is one per payout request, immutable, and has no public initializer, update, or close instruction. Strict payout wrappers are the only intended writers.

## Common Validation Boundary

The common payout validator checks:

- Victim Relief config is not paused
- Security governance config is not paused
- config / policy / case / request bindings
- policy schema and version
- request status is `Approved`
- request `executed_at == 0`
- case status is `PayoutQueued`
- approved amount is exact and greater than zero
- claimant state matches and active count is greater than zero
- source vault is the Treasury relief USDC vault PDA
- vault owner is `vault_authority_v2`
- recipient token account key, owner, and mint match the frozen request
- recipient token account is not the relief vault or another Treasury vault PDA
- USDC mint and decimals match the protocol USDC expectation
- relief vault balance is sufficient for the exact payout
- payout receipt PDA is derived from the payout request and is still empty

The validator does not transfer tokens and does not mutate request, case, claimant state, or Treasury accounting.

## Accounting Boundary

Victim Relief payout is a Relief pool outflow. It must not reduce:

- `TreasuryUsdcStateV2.relief_usdc_total`
- `RevenueRoutingStatsV1`
- any protocol revenue category total

Current vault balance is determined by the SPL Token account.

## Deferred Work

Stage 6B-4B-2 implements the original-approve strict payout wrapper and actual relief-vault transfer:

- fixed origin `OriginalApprove`
- fixed action `VictimReliefApproveCompensation`
- exact `transfer_checked` from `relief_usdc_vault`
- request status moves to `Executed`
- case status moves to `Paid`
- claimant active case count decrements once
- immutable payout execution receipt is written

Stage 6B-4B-3 implements the appeal-overturn strict payout wrapper and actual relief-vault transfer:

- fixed origin `AppealOverturn`
- fixed action `VictimReliefOverturnAppeal`
- original reject receipt and appeal overturn receipt are both required
- exact `transfer_checked` from `relief_usdc_vault`
- request status moves to `Executed`
- case status moves to `Paid`
- appeal remains `Overturned`
- claimant active case count decrements once
- immutable payout execution receipt is written

Stage 6B-4B-4B implements strict cancellation governance for unpaid approved payout requests:

- fixed action `VictimReliefCancelPayout`
- canonical target `ReliefPayoutRequestV1`
- separate wrappers for original approve and appeal overturn sources
- request status moves to `Cancelled`
- case status moves to `Cancelled`
- claimant active case count decrements once
- immutable cancellation receipt is written
- no USDC transfer and no Treasury accounting mutation

Deferred beyond this phase:

- partial payouts
- recipient migration
- vault reservation / fair ordering
- payout stats account
- frontend integration
- Devnet / Mainnet deployment

For both original approve and appeal overturn, `PayoutRequest Approved`, `PayoutQueued`, and `Queue Executed` are not payment. Payment requires the dedicated wrapper, exact relief-vault transfer, request `Executed`, case `Paid`, and `ReliefPayoutExecutionRecordV1`.

Mainnet and token launch remain NO-GO.

See [victim-relief-payout-cancellation-governance-v1.md](victim-relief-payout-cancellation-governance-v1.md).
