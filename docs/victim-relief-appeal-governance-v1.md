# Victim Relief Appeal Governance V1

## Scope

Phase 2E-6B-3 adds a one-time DAO appeal path for rejected Victim Relief cases:

```text
Rejected
-> claimant opens appeal
-> AppealPending
-> DAO Uphold or Overturn
-> immutable appeal decision receipt
-> Overturn creates ReliefPayoutRequestV1
-> Stage 6B-4 performs any relief-vault transfer
```

This phase does not transfer USDC, does not execute payouts, does not create an appeal cancel or expiry path, and does not add a new governance action for opening an appeal. Opening an appeal is a claimant business action, not a DAO action.

## Append-Only Actions

Governance actions appended in this phase:

- `VictimReliefUpholdAppeal`
- `VictimReliefOverturnAppeal`

Both map to `ProtocolModuleIdV1::VictimRelief` and `GovernanceProposalTypeV1::VictimRelief`.

Security proposal/action variants are also append-only:

- `ProposalType::VictimReliefUpholdAppeal`
- `ProposalType::VictimReliefOverturnAppeal`
- `ActionType::VictimReliefUpholdAppeal`
- `ActionType::VictimReliefOverturnAppeal`

Stable governance action codes are explicit and do not use `enum as u8`.

## Appeal Account

`VictimReliefAppealV1` records the single V1 appeal for a case.

PDA:

```text
victim_relief_appeal_v1 + victim_relief_case
```

It stores:

- case / config / policy / policy version
- claimant
- original evidence snapshot
- original reject decision record
- original governance proposal and queue item
- appeal evidence commitment
- status: `Pending`, `Upheld`, or `Overturned`
- appeal decision proposal and queue references
- opened / deadline / resolved timestamps
- schema version and reserved bytes

One case can have at most one V1 appeal. The account has no update or close instruction. It stores no string, URI, raw evidence, or PII.

## Open Appeal

`open_victim_relief_appeal_v1` is claimant-only.

It requires:

- the case is `Rejected`
- the appeal deadline is still open
- no existing active appeal is recorded on the case
- the original evidence snapshot belongs to the case
- the original decision execution record is a reject receipt for the same case, proposal, and queue
- config, locked policy, policy version, claimant, and case references match

On success:

- creates `VictimReliefAppealV1`
- sets appeal status to `Pending`
- changes case status to `AppealPending`
- records `case.active_appeal`
- does not modify approved amount, recipient, policy, evidence snapshot, claimant active case count, Treasury state, or token balances

## Appeal Evidence

Appeal evidence is only a commitment.

Allowed forms:

- no new evidence: `root == [0; 32]` and `count == 0`
- new evidence: `root != [0; 32]`, `count > 0`, and `count <= policy.max_evidence_items`

Rejected forms:

- zero root with nonzero count
- nonzero root with zero count
- count above policy limit

## Active Case Count

The original reject path already decrements `active_case_count`.

V1 appeal rules:

- opening an appeal does not increment active case count
- uphold does not change active case count
- overturn restores the case to `PayoutQueued` and increments active case count by one with checked arithmetic
- overturn does not re-check the new-case active cap, because it restores an old case rather than admitting a new one

## Appeal Decision Parameters

`VictimReliefAppealDecisionParametersV1` binds:

- config / policy / policy version
- case and appeal
- original evidence snapshot
- original reject decision record
- claimant and subject commitment
- original and appeal evidence commitments
- claimed amount and deterministic approved amount
- recipient owner and token account
- USDC mint, Treasury config, and relief vault
- governance action and proposal id
- expected case status `AppealPending`
- expected appeal status `Pending`

Hash domain:

```text
alpha_victim_relief_appeal_decision_parameters_v1
```

Uphold uses approved amount `0`. Overturn uses the existing deterministic helper:

```text
min(case.claimed_amount_usdc, policy.max_payout_per_case_usdc)
```

Callers do not provide the approved amount or arbitrary parameters hash.

## Canonical Governance Target

Appeal governance actions target `VictimReliefAppealV1`, not the original case:

```text
GovernanceProposalActionV1.target_account == victim_relief_appeal
```

This keeps the original case decision and the appeal decision distinct, and prevents one appeal proposal from operating on another appeal.

## Appeal Decision Receipt

`VictimReliefAppealDecisionExecutionRecordV1` is immutable and one-per-queue.

PDA:

```text
victim_relief_appeal_rcpt_v1 + execution_queue_item
```

The shortened seed is intentional because Solana PDA seed components must be 32 bytes or shorter.

The receipt records the governance proposal, proposal action sidecar, adapter output, Security decision, queue item, module registry, case, appeal, original snapshot, original reject record, execution type, before/after statuses, amount, recipient, parameters hash, canonical governance payload hash, executor, and execution timestamp.

Execution type stable codes:

- `Uphold = 1`
- `Overturn = 2`

## Strict Governance Validation

Both appeal execution wrappers require:

- governance proposal status is `Passed`
- sidecar module is `VictimRelief`
- sidecar target is the appeal account
- target program is the current Alpha Protocol program id
- appeal decision parameters hash is recomputed on-chain
- canonical governance payload hash is recomputed on-chain
- Victim Relief module registry is enabled and bound to the expected Security governance config
- adapter, `ProposalDecisionV1`, and `ExecutionQueueItemV1` match the same proposal, action, target, and payload
- proposal decision is approved
- queue item is executed
- case is `AppealPending`
- appeal is `Pending`
- original snapshot and original reject record match the case
- Treasury config and relief vault are the expected accounts

The wrappers do not trust caller-supplied action, target, recipient, amount, evidence, policy, relief vault, or parameters hash.

## Uphold

`execute_uphold_victim_relief_appeal_v1`:

- marks appeal `Upheld`
- marks case `AppealUpheld`
- clears `case.active_appeal`
- keeps approved amount `0`
- creates an immutable appeal decision receipt
- does not create `ReliefPayoutRequestV1`
- does not change active case count
- does not transfer USDC
- does not update Treasury totals

## Overturn

`execute_overturn_victim_relief_appeal_v1`:

- marks appeal `Overturned`
- marks case `PayoutQueued`
- clears `case.active_appeal`
- derives the policy-capped approved amount
- increments claimant active case count by one
- creates `ReliefPayoutRequestV1`
- creates an immutable appeal decision receipt
- does not transfer USDC
- does not mark the case `Paid`
- does not update Treasury revenue totals

The payout request uses the appeal proposal / decision / queue references, keeps the original evidence snapshot reference, and stores the appeal decision parameters hash.

`PayoutRequest Approved` is not payment. `Queue Executed` is not payment. The original approve payout wrapper from Stage 6B-4B-2 does not accept appeal overturn authorization receipts. Stage 6B-4B-3 must add the dedicated appeal-aware payout wrapper before appeal overturn can move relief-vault USDC.

Stage 6B-4B-1 adds the payout foundation in [victim-relief-payout-foundation-v1.md](victim-relief-payout-foundation-v1.md). Stage 6B-4B-2 adds original approve payout in [victim-relief-original-approved-payout-v1.md](victim-relief-original-approved-payout-v1.md). The future appeal payout path must use a dedicated `execute_victim_relief_overturn_payout_v1` wrapper, validate the `VictimReliefAppealDecisionExecutionRecordV1::Overturn` receipt, and write `ReliefPayoutExecutionRecordV1`.

## Non-Goals And Residual Risk

This phase intentionally does not implement:

- appeal cancel
- appeal expiry
- appeal overturn relief-vault transfer
- arbitrary partial compensation
- additional evidence plaintext storage
- frontend
- Devnet or Mainnet transaction execution

There is a liveness residual risk: if DAO governance does not process the appeal, a case can remain in `AppealPending`. Permissionless appeal expiry is deferred because it could race with voting or timelock before proposal linkage is fully modeled.

DAO decisions are not court judgments, insurance determinations, credit ratings, or investment advice. Local tests are not Devnet or Mainnet verification. Mainnet and token launch remain NO-GO.
