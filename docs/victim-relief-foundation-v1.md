# Victim Relief Foundation V1

## Scope

Victim Relief Foundation V1 establishes the first on-chain account layer for relief cases:

- `VictimReliefConfigV1`
- immutable `VictimReliefPolicyV1`
- `VictimReliefClaimantStateV1`
- `VictimReliefCaseV1`
- evidence-root updates during the evidence window
- claimant cancellation
- permissionless expiry

This is a foundation layer only. It does not approve claims, reject claims, pay claims, transfer USDC, or execute any DAO-controlled relief payout.

## Phase 2E-6B-2 Evidence and Decision Layer

The next Victim Relief layer is documented in [victim-relief-evidence-and-decision-v1.md](victim-relief-evidence-and-decision-v1.md).

It adds immutable evidence snapshots, DAO approve/reject execution records, and `ReliefPayoutRequestV1`. It still does not transfer USDC. `PayoutQueued` is not `Paid`, and queue execution is not proof that funds were paid. Stage 6B-4 must add the actual relief-vault payout receipt.

V1 approval uses the deterministic policy-capped amount:

```text
min(case.claimed_amount_usdc, policy.max_payout_per_case_usdc)
```

V1 does not support arbitrary partial compensation. Review deadline is recorded for audit only and does not create permissionless `UnderReview` expiry in this phase.

## Phase 2E-6B-3 Appeal Governance Layer

Appeal governance is documented in [victim-relief-appeal-governance-v1.md](victim-relief-appeal-governance-v1.md).

It adds a one-time claimant appeal path for rejected cases:

```text
Rejected -> AppealPending -> DAO Uphold / Overturn
```

Opening an appeal is a claimant business action, not a DAO action. Uphold creates only an immutable appeal decision receipt. Overturn restores the deterministic payout path by creating `ReliefPayoutRequestV1`, but it still does not transfer USDC. `PayoutRequest Approved` and `Queue Executed` are not proof of payment.

V1 appeal does not implement cancel or expiry. If DAO governance does not process an appeal, the case can remain in `AppealPending`; this liveness risk is intentionally deferred until appeal-proposal linkage is modeled safely.

## Phase 2E-6B-4B-1 Payout Foundation

Victim Relief payout foundation is documented in [victim-relief-payout-foundation-v1.md](victim-relief-payout-foundation-v1.md).

It adds strict payout origin typing, canonical payout parameters hashing, authorization validators, common vault / recipient / pause checks, and the immutable `ReliefPayoutExecutionRecordV1` model. It still does not add a public payout wrapper, transfer USDC, mark a request `Executed`, mark a case `Paid`, or decrement active case count.

The only valid future payout origins are original DAO approve and appeal overturn. `Queue Executed`, `PayoutQueued`, and `PayoutRequest Approved` remain distinct from actual payment.

## Privacy Boundary

Victim Relief V1 stores only fixed-length commitments and public accounting fields on-chain.

Allowed on-chain data:

- `subject_commitment: [u8; 32]`
- `evidence_root: [u8; 32]`
- claimant public key
- claimant-owned recipient USDC token account
- requested amount
- evidence count
- timestamps
- status
- config and policy references

Not allowed on-chain:

- names
- identity documents
- phone numbers
- email addresses
- physical addresses
- bank information
- medical information
- private chat logs
- raw complaint text
- evidence plaintext
- URL or IPFS plaintext pointers

`subject_commitment` must be constructed off-chain with a high-entropy salt. The salt is never stored on-chain. The program cannot verify whether a user used a high-entropy salt. `evidence_root` is intended to be a Merkle commitment to an off-chain evidence set.

DAO review is not a court judgment, legal finding, insurance determination, or investment recommendation.

## Accounts

`VictimReliefConfigV1` uses PDA seed:

```text
victim_relief_config_v1
```

It records the Security Layer authority, Treasury config, USDC mint, current immutable policy, `next_case_id`, paused state, schema version, and reserved bytes. It has no update, pause, or unpause instruction in this phase.

`VictimReliefPolicyV1` uses PDA seeds:

```text
victim_relief_policy_v1 + config + policy_version
```

Policy V1 is immutable and bootstrap-only. Cases permanently lock the policy pubkey and policy version used at submission time.

`VictimReliefClaimantStateV1` uses PDA seeds:

```text
victim_relief_claimant_state_v1 + config + claimant
```

It tracks active case count, total case count, last case id, and submission cooldown metadata.

`VictimReliefCaseV1` uses PDA seeds:

```text
victim_relief_case_v1 + config + case_id
```

Cases contain only fixed-length commitments, amount fields, token-account references, status, deadlines, and governance reference placeholders.

## Case Lifecycle In This Phase

1. `submit_victim_relief_case_v1`
   - creates a case directly in `EvidencePeriod`
   - requires claimant signer
   - requires recipient owner to equal claimant
   - requires claimant recipient token account mint to equal the config USDC mint
   - increments claimant active and total case counters
   - does not transfer USDC
   - does not create a governance proposal

2. `update_victim_relief_evidence_root_v1`
   - allows the claimant to replace the evidence Merkle root during the evidence window
   - increments `evidence_revision`
   - does not store evidence plaintext
   - does not modify recipient, claimed amount, policy, or deadlines

3. `cancel_victim_relief_case_v1`
   - allows claimant cancellation only while status is `EvidencePeriod`
   - sets status to `Cancelled`
   - decrements active case count
   - does not close accounts or move funds

4. `expire_victim_relief_case_v1`
   - permissionless after `now > evidence_deadline`
   - sets status to `Expired`
   - decrements active case count
   - does not close accounts or move funds

Stage 6B-2 must freeze evidence before a case can enter DAO review. Expired-but-unfrozen cases cannot move into DAO decision flow.

## Anti-Spam Measures

Implemented now:

- claimant signer
- recipient must be claimant
- maximum active cases per claimant
- submission cooldown
- maximum evidence item count
- case-account rent cost
- evidence deadline
- cancellation and expiry release active case count

Not solved in this phase:

- multi-wallet Sybil submissions
- duplicate real-world events across wallets
- global per-subject deduplication
- submission bonds
- DAO reviewer information-cost defenses

Spam is partially mitigated, not eliminated.

## Not Implemented

This phase does not implement:

- DAO approve / reject domain execution
- governance decision receipt
- evidence freeze / governance snapshot binding
- appeals
- payout requests
- appeals
- relief vault transfer
- payout execution receipt writer as a public instruction
- Treasury execution
- submission bond
- third-party recipient
- frontend
- Devnet or Mainnet deployment

There is no relief vault transfer entrypoint in this phase. Authority cannot approve or pay claims through these instructions.

## Compatibility

This phase does not modify existing Treasury, Governance, Security, Green Label, Protocol Module Registry, or generated IDL files. It adds new Victim Relief accounts and one new case-status enum.

Token launch remains NO-GO. Local tests are not Devnet or Mainnet validation.
