# Victim Relief Evidence and Decision V1

## Scope

Phase 2E-6B-2 adds the read/decision closure layer for Victim Relief cases:

- immutable `VictimReliefEvidenceSnapshotV1`
- claimant-only `freeze_victim_relief_evidence_v1`
- deterministic `VictimReliefDecisionParametersV1`
- immutable `VictimReliefDecisionExecutionRecordV1`
- `ReliefPayoutRequestV1`
- strict DAO / Module Registry / Adapter / Security validation for approve and reject

This phase does not transfer USDC, does not execute payouts, does not implement appeals, and does not add a review-expiry path.

## Evidence Freeze

Evidence is frozen before DAO review by creating `VictimReliefEvidenceSnapshotV1`.

PDA seeds:

```text
victim_relief_evidence_snap_v1 + victim_relief_case
```

The shortened seed is used because Solana PDA seed components must be 32 bytes or shorter.

`freeze_victim_relief_evidence_v1` is claimant-only and requires:

- case status is `EvidencePeriod`
- `now <= evidence_deadline`
- nonzero evidence root
- evidence count is within the locked policy limit
- case, config, and locked policy match
- claimant is the original claimant
- recipient owner remains the claimant

On success:

- the immutable snapshot copies the final evidence/payment commitments
- case status changes to `UnderReview`
- `review_deadline` is recorded from the locked policy
- no governance proposal is created
- no USDC is transferred

Freeze and evidence expiry do not overlap. Freeze is allowed up to and including `evidence_deadline`; expiry remains allowed only after `evidence_deadline`.

## Decision Parameters

`VictimReliefDecisionParametersV1` binds the DAO decision to the frozen case:

- config / policy / policy version
- case / case id
- evidence snapshot
- claimant
- subject commitment
- evidence root / count / revision
- claimed amount
- derived approved amount
- recipient owner and token account
- USDC mint
- Treasury config
- relief vault
- governance action type
- expected status `UnderReview`
- proposal id

The hash domain is:

```text
alpha_victim_relief_decision_parameters_v1
```

V1 approved amount is deterministic:

```text
approved_amount_usdc = min(case.claimed_amount_usdc, policy.max_payout_per_case_usdc)
```

V1 supports only approve with the policy-capped amount or reject with amount `0`. Arbitrary partial compensation is intentionally deferred to a future typed compensation request design.

## DAO Decision Execution

The canonical governance target for both Victim Relief actions is the `VictimReliefCaseV1` account.

Supported actions:

- `VictimReliefApproveCompensation`
- `VictimReliefRejectClaim`

Both execution paths validate:

- Governance proposal status is `Passed`
- `GovernanceProposalActionV1` is present and targets the case
- module id is `VictimRelief`
- target program is the current Alpha Protocol program id
- parameters hash is recomputed by the program
- canonical governance payload hash is recomputed by the program
- Victim Relief module registry is enabled
- adapter, `ProposalDecisionV1`, and `ExecutionQueueItemV1` match the same action, target, and payload
- proposal decision is approved
- execution queue item is executed
- case is `UnderReview`
- snapshot matches the case
- policy and version match the locked case policy
- relief vault is the expected Treasury relief USDC vault PDA

## Approve

`execute_approve_victim_relief_case_v1` creates:

- `ReliefPayoutRequestV1`
- immutable `VictimReliefDecisionExecutionRecordV1`

Approve changes the case to `PayoutQueued` and freezes:

- approved amount
- recipient owner
- recipient token account
- Treasury config
- relief vault
- proposal / decision / queue / snapshot links

`PayoutQueued` is not `Paid`. Queue execution is not proof that funds were paid. Stage 6B-4 must add the payout receipt and the actual relief-vault transfer.

Stage 6B-4B-1 is documented in [victim-relief-payout-foundation-v1.md](victim-relief-payout-foundation-v1.md). It adds payout origin typing, payout parameters hashing, a future immutable payout receipt account, and strict validation helpers. It still does not expose a public payout wrapper and still does not transfer USDC.

## Reject

`execute_reject_victim_relief_case_v1` creates only the immutable decision execution record.

Reject changes the case to `Rejected`, sets `approved_amount_usdc = 0`, records the appeal deadline from the locked policy, and decrements claimant active case count once.

Reject does not create a payout request, does not transfer USDC, and does not automatically open an appeal.

## Stage 6B-3 Appeal Governance

Stage 6B-3 adds [Victim Relief Appeal Governance V1](victim-relief-appeal-governance-v1.md).

Appeal rules:

- only the claimant can open an appeal
- only rejected cases can be appealed
- V1 allows one appeal per case
- appeal evidence stores only a commitment
- appeal governance targets the `VictimReliefAppealV1` account
- `VictimReliefUpholdAppeal` creates no payout request
- `VictimReliefOverturnAppeal` creates `ReliefPayoutRequestV1`
- overturn uses the same deterministic policy-capped amount as initial approve
- no USDC is transferred in the appeal phase

`PayoutQueued` remains distinct from `Paid`. Stage 6B-4 must perform the actual relief-vault transfer and write a payment receipt before a claim can be described as paid on-chain.

Future payout execution must use strict wrappers for original approve and appeal overturn. A generic caller-selected payout path is not part of V1.

## Review Deadline Residual Risk

`review_deadline` is an audit field in this phase. There is no permissionless `UnderReview` expiry because the case is not yet linked to an active governance proposal lifecycle. Adding expiry before reliable proposal linkage could allow a case to be expired during DAO voting or timelock.

## Privacy and Safety

The chain stores only commitments, hashes, counters, amounts, pubkeys, and timestamps. It does not store names, PII, raw evidence, plaintext URLs, or evidence text.

DAO decisions do not represent court judgments, legal determinations, insurance coverage, credit ratings, or investment advice.

Local tests are not Devnet or Mainnet verification. Token launch remains NO-GO.
