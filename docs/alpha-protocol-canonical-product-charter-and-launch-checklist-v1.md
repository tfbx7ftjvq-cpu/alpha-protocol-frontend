# Alpha Protocol Canonical Product Charter & Launch Checklist V1

Status: OWNER-CONFIRMED CANONICAL SCOPE

This document is the authoritative product and launch source of truth. If an
older roadmap, phase report, README inventory, generated summary, or assistant
message conflicts with it, this document wins. Only an explicit owner decision
may change the core model below.

## 1. Mission

Alpha Protocol exists to turn transparent protocol revenue into long-lived,
community-governed protection and risk infrastructure for retail participants:

1. compensate eligible victims of scam projects through DAO decisions;
2. reduce future harm through community-assessed Green Label certification;
3. sustain protocol development and operations from real revenue, not a hidden
   token allocation or guaranteed return promise.

Alpha Protocol is not a court, insurer, credit-rating agency, price-support
scheme, or guarantee that any applicant, holder, or claimant will be paid.

## 2. Token launch decisions

The following decisions are final unless the owner explicitly amends them:

| Area | Canonical decision |
| --- | --- |
| Launch platform | Pump |
| Network | Solana Mainnet |
| Token | ALPHA |
| Total supply | 1,000,000,000 ALPHA |
| Launch model | Fair Launch through Pump |
| Project/team/VC preallocation | 0 |
| DAO/staking/builder token reserve at launch | 0 |
| Treasury opening balance | 0 USDC |
| Primary accounting asset | USDC |
| Mint/freeze policy | Must be finalized and publicly disclosed before signing the launch |

Pump Creator Fee is the technical term for trading revenue paid to the token
creator. It must not be described in code or formal disclosures as an ALPHA
Token-2022 transfer tax. The final Pump launch screen must be checked because
Pump can change fees, quote assets, routing, and recipient behavior.

## 3. Revenue flywheel

### 3.1 Pump trading revenue

```text
Pump ALPHA trades
-> Pump Creator Fee / creator trading revenue
-> dedicated revenue wallet
-> USDC (directly, or through a separately recorded conversion if received as SOL)
-> Alpha typed USDC revenue router
-> deterministic 50 / 20 / 20 / 10 Treasury split
```

A direct transfer to an arbitrary wallet or vault is not official routed
revenue. Every batch needs a source record, amount, asset, conversion evidence
when applicable, routing signature, four-vault balance reconciliation, and a
public batch identifier.

### 3.2 Green Label long-term revenue

Projects may apply for Green Label certification and pay a separate,
non-refundable USDC certification/service fee. That fee is protocol revenue and
routes through the same 50 / 20 / 20 / 10 router.

The service fee is separate from refundable bond escrow:

- certification/service fee: non-refundable protocol revenue;
- base bond: Mainnet target 299 USDC unless explicitly amended;
- voluntary extra bond: fully refundable under the published policy;
- refundable bond remains an escrow liability and is not Treasury revenue;
- a valid DAO-governed forfeit converts only the authorized forfeited amount
  into `GreenLabelForfeitedBond` revenue;
- no time-only or administrator-only forfeit path is allowed.

The current policy target for the base bond is 80% refundable and 20% Treasury
share after the applicable certification lifecycle. Final Mainnet parameters,
including the separate service-fee amount, must be frozen and disclosed before
opening applications.

### 3.3 Fixed Treasury split

Every eligible routed USDC revenue batch uses the existing deterministic split:

- 50% Victim Relief Pool;
- 20% ALPHA buyback/burn pool;
- 20% builders, contributors, payroll, audits, and operations;
- 10% staking rewards.

This is a revenue split, not token supply allocation. Empty vaults create no
payment obligation. Buyback/burn is not a price floor. Staking rewards have no
fixed APY and depend on actual routed revenue.

## 4. Community DAO powers

The community DAO determines:

1. which victim-relief cases are approved, rejected, appealed, cancelled, and
   eligible for exact USDC payout;
2. whether a Green Label application is approved, rejected, disputed, revoked,
   refunded, or forfeited under the published policy;
3. major Treasury policy, exceptional spending, protocol parameters, module
   pause/unpause, and governed execution;
4. public risk/exposure decisions and audited contributor work where enabled.

Moderators and reviewers may validate evidence, redact private information, and
prepare proposals. They do not replace the DAO decision, cannot review their
own submission, and cannot directly move Treasury funds.

A DAO decision is not payment proof. Payment requires the bound decision,
timelocked execution authorization, exact recipient/amount/mint, Treasury vault
transfer, immutable receipt, and reconciliation.

## 5. Victim Relief rules

The relief lifecycle is:

```text
private application + evidence commitment
-> independent review
-> public sanitized proposal
-> governance snapshot and vote
-> final decision and timelock
-> exact relief-vault transfer
-> immutable payout receipt and public progress record
```

Raw evidence and PII stay off-chain. On-chain/public records use wallet or
recipient references, salted subject hashes, evidence roots, amounts, policy
versions, timestamps, decision hashes, and execution references.

No automatic pro-rata holder payment comes from the 50% Relief Pool. The DAO
selects eligible cases. The payout amount cannot exceed the approved amount or
the available Relief Pool balance.

## 6. Holder staking rewards

The 10% staking-reward pool is permissionless and distributed according to the
published staking accounting, proportional to eligible staked ALPHA with lock
duration weighting. It is funded only by real routed revenue.

Existing owner decisions retained here:

- unlocking, transferring, or selling resets consecutive staking duration;
- no fixed APY or guaranteed return;
- reward withdrawal carries the published 5% maintenance fee unless explicitly
  amended through the required governance process.

## 7. Anti-flash and anti-whale governance

Green Label and Victim Relief decisions must not be controlled by a wallet that
buys immediately before a vote. Minimum V1 controls are:

- voting power comes from ALPHA locked for 30 to 365 days, not a live wallet
  balance;
- a proposal snapshot freezes eligible positions and total voting power;
- a position created or updated after the snapshot cannot vote on that proposal;
- voting extends the position vote-lock through the proposal end;
- one vote record per position prevents duplicate voting;
- linear locked-token voting makes wallet splitting neutral for an equal total
  amount and duration;
- Green Label decisions require quorum, approval threshold, and a minimum
  unique-participant threshold;
- the Mainnet Green Label voting configuration must include an owner-approved
  concentration limit or equivalent two-layer safeguard before applications
  are opened.

The current lock/snapshot model blocks flash governance capture but does not
eliminate a wealthy long-term holder or Sybil risk. The final concentration
rule is therefore a launch blocker for live Green Label decisions, not a claim
that V1 has solved every whale attack.

## 8. Scope discipline

The following support launch but may not replace the core product priorities:
release evidence, Supabase operations, moderation queues, role lifecycle,
recovery drills, CI, monitoring, hiring, community task administration, and
marketing operations.

A missing large staff team is not a token-launch blocker. A solo-dev launch may
use the minimum independently controlled multisig/reviewer set, but no single
person may silently exercise all Treasury authorization and reconciliation
powers after real funds exist.

No future phase may be treated as launch-critical unless it maps to at least one
of these pillars: Pump revenue, Treasury split, DAO governance, Victim Relief,
Green Label, staking, Mainnet authority safety, or required public disclosure.

## 9. Reconciled implementation status

### Implemented and evidenced on Devnet/local/Staging

- [x] Fair Launch model, 1B supply, and zero project/team/VC allocation recorded.
- [x] Treasury V2 USDC routing and deterministic 50 / 20 / 20 / 10 split.
- [x] Typed USDC revenue router and Green Label revenue types.
- [x] Green Label strict one-time service-fee receipt.
- [x] Green Label refundable escrow, refund, dispute, and governed forfeit paths.
- [x] Governance lock, time weighting, snapshots, vote records, finalization,
      adapter, timelock, and governed execution foundations.
- [x] Victim Relief private evidence, independent review, decision, appeal,
      strict original/overturn payouts, cancellation, pause, and receipts.
- [x] Four-vault Treasury accounting and staking reward funding source.
- [x] Public/off-chain Staging operations, RLS, cleanup, release evidence, and
      read-only release inspection.

### Launch blockers still open

- [ ] Replace stale documents that still say launch platform is pending; Pump is
      now the canonical platform.
- [ ] Resolve and document the historical deployed/local Solana program binary
      hash mismatch before any upgrade, authority change, or real-funds action.
- [ ] Produce one reproducible Mainnet release candidate: source commit, `.so`,
      IDL, Program ID, checksums, and independent review all match.
- [ ] Confirm the current Pump USDC launch path, Creator Fee recipient, fee asset,
      liquidity mechanics, and final creation cost in the live Pump UI.
- [ ] Verify the Pump revenue wallet and claim/collection path; if Pump pays SOL,
      define the separately approved SOL-to-USDC conversion and slippage limits.
- [ ] Initialize and record Mainnet Program ID, USDC mint, ALPHA mint, Treasury
      config, four vaults, staking pool, Green Label config, governance configs,
      module registries, and public explorer links.
- [ ] Freeze Mainnet Green Label parameters: 299 USDC base bond, separate
      certification/service fee, lifecycle windows, refund/forfeit policy, and
      anti-whale voting configuration.
- [ ] Finalize mint authority, freeze authority, upgrade authority, emergency
      guardian, DAO/multisig, and revenue-operator policies.
- [ ] Complete a small-value Mainnet canary covering USDC route/split, Green
