# Victim Relief Original Approved Payout V1

## Scope

Phase 2E-6B-4B-2 implements the strict original-approve Victim Relief payout path:

```text
VictimReliefApproveCompensation
-> VictimReliefDecisionExecutionRecordV1::Approve
-> ReliefPayoutRequestV1::Approved
-> exact USDC transfer from relief_usdc_vault
-> ReliefPayoutRequestV1::Executed
-> VictimReliefCaseV1::Paid
-> claimant active case count decrement
-> ReliefPayoutExecutionRecordV1
```

This phase supports only the original DAO approve source. Appeal overturn payout is implemented separately in Stage 6B-4B-3; see [victim-relief-appeal-overturn-payout-v1.md](victim-relief-appeal-overturn-payout-v1.md).

## Public Instruction

The only public entrypoint added in this phase is:

```text
execute_victim_relief_approved_payout_v1
```

It accepts no instruction arguments. The executor signer pays transaction fees and receipt rent, but cannot choose the amount, recipient, source vault, payout origin, action, policy version, token mint, or payload hash.

The recipient does not sign. No authority or guardian signer is required.

## Fixed Origin And Action

The handler fixes:

- `VictimReliefPayoutOriginV1::OriginalApprove`
- `GovernanceActionTypeV1::VictimReliefApproveCompensation`

It calls the original-approve authorization validator and rejects reject, uphold, overturn, wrong-action, wrong-target, wrong-proposal, wrong-decision, wrong-queue, and mismatched authorization receipt paths.

## Strict Validation

Before any token transfer, the handler validates:

- Victim Relief config is not paused.
- Security governance config is not paused.
- `config.security_governance_config` is bound to the supplied governance config.
- payout request PDA belongs to the case.
- request status is `Approved`.
- request `executed_at == 0`.
- request proposal / decision / queue / parameters hash are nonzero and match authorization.
- case status is `PayoutQueued`.
- approved amount is exact and greater than zero.
- no active appeal is attached to the case.
- case, request, snapshot, config, policy, version, recipient, amount, Treasury config, relief vault, and USDC mint match.
- claimant state belongs to the claimant and has a positive active case count.
- source is the fixed Treasury relief USDC vault PDA.
- vault owner is the `vault_authority_v2` PDA.
- vault balance is at least the exact approved amount.
- recipient token account key, owner, and mint match the frozen payout request.
- recipient token account is not the relief vault or another Treasury vault PDA.
- USDC mint decimals match the protocol USDC decimals.
- payout receipt PDA is derived from the payout request and has not been written.

## Exact Transfer

The transfer uses SPL Token `transfer_checked`:

- source: fixed `relief_usdc_vault`
- authority: `vault_authority_v2` PDA signer
- destination: frozen request recipient token account
- amount: frozen `ReliefPayoutRequestV1.approved_amount_usdc`
- decimals: protocol USDC decimals

There is no partial payout, overpay, underpay, multi-step split, Revenue Router call, builders payout helper call, native SOL payout, or generic Treasury vault transfer instruction.

## State Changes

After successful transfer:

- `ReliefPayoutRequestV1.status = Executed`
- `ReliefPayoutRequestV1.executed_at = now`
- `VictimReliefCaseV1.status = Paid`
- `VictimReliefCaseV1.updated_at = now`
- claimant `active_case_count` decrements once
- claimant `total_case_count` does not change
- `ReliefPayoutExecutionRecordV1` is written immutably

`approved_amount_usdc`, recipient, proposal, decision, queue, snapshot, and policy links remain frozen.

## Payout Receipt

The payout receipt PDA is:

```text
relief_payout_rcpt_v1 + relief_payout_request
```

The receipt records the origin, authorization action, original authorization receipt, exact amount, recipient, source relief vault, authorization parameters hash, canonical payout parameters hash, executor, timestamp, schema, and bump.

The executor is recorded in the receipt but is not part of the canonical payout hash.

## Accounting Boundary

Victim Relief payout is a relief-vault outflow. It does not mutate:

- `TreasuryUsdcStateV2`
- `RevenueRoutingStatsV1`
- `relief_usdc_total`
- builders / buyback / staking revenue totals
- Green Label revenue totals

Payout is not a revenue reversal. Current available funds are represented by the SPL Token vault balance.

## Pause And Retry Semantics

If Victim Relief config or Security governance config is paused, payout fails before transfer.

Stage 6B-4B-4C-B1 adds governed module pause lifecycle in [victim-relief-module-pause-governance-v1.md](victim-relief-module-pause-governance-v1.md). Guardian emergency pause can only set the Victim Relief module to paused. DAO + Security can pause or unpause with distinct typed actions. Module unpause is blocked while Security global pause is active.

If relief vault balance is insufficient, payout fails before transfer. The request remains `Approved`, the case remains `PayoutQueued`, and the payout may be retried when the vault has sufficient USDC.

If the recipient token account is invalid or closed, payout fails without recipient migration. The frozen recipient account must be restored or a future recipient-migration design must be implemented.

## Deferred Work

Not implemented in this phase:

- generic payout instruction
- partial payout
- recipient migration
- payout cancellation is now implemented separately by Stage 6B-4B-4B
- vault reservation / fair ordering
- payout statistics
- new Governance or Security action types
- frontend integration
- Devnet or Mainnet deployment

`Queue Executed` alone is not payment. For original approve, payment completion requires request `Executed`, case `Paid`, the actual relief-vault transfer, and `ReliefPayoutExecutionRecordV1`.

Original approve and appeal overturn are intentionally separate strict wrappers. Original approve accepts only the original approval receipt; appeal overturn accepts only the appeal overturn receipt and original reject linkage.

Local tests are not Devnet or Mainnet verification. Mainnet production and token launch remain NO-GO.

## Cancellation Governance

Stage 6B-4B-4B adds `execute_cancel_original_victim_relief_payout_v1` for original approve requests that remain `Approved` and unpaid. Cancellation is a separate DAO + Security action targeting `ReliefPayoutRequestV1`; it writes `VictimReliefPayoutCancellationRecordV1`, marks request and case `Cancelled`, decrements active case count once, and transfers no USDC.

See [victim-relief-payout-cancellation-governance-v1.md](victim-relief-payout-cancellation-governance-v1.md).
