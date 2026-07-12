# Treasury / DAO Revenue Readiness Audit

Date: 2026-07-12

Scope: read-only audit of current Treasury V2, revenue split, Builders 20%, DAO Governance / Security Layer, Mainnet treasury readiness, and token-launch credibility gaps.

This report does not modify contracts, IDL, frontend code, scripts, Program ID, `Anchor.toml`, `target/deploy`, or keypairs. It does not approve token launch, Mainnet launch, or any on-chain transaction.

## 1. Executive Summary

Alpha Protocol has a credible Devnet foundation for Treasury V2 accounting, USDC four-pool revenue routing, staking reward funding, Green Label refund / slash E2E, and Security Layer execution gating.

However, the current system is not yet a complete "DAO-governed protocol revenue economy." The strongest completed pieces are:

- Fair Launch token principles.
- Treasury V2 USDC deposit and automatic 50 / 20 / 20 / 10 split.
- Devnet staking reward funding from the staking USDC vault.
- Security Layer decision / queue / timelock / cancel / pause.
- Green Label refund / slash E2E linked to Security Layer state.

The largest missing or partial pieces are:

- Real production revenue routing from external sources into `deposit_usdc_revenue`.
- Builders / contributors payout governance from the 20% builders vault.
- Full ALPHA voting layer: proposal creation by users, votes, quorum, threshold, voting period, and automatic finalization.
- Mainnet treasury addresses, authorities, vaults, staking pool, mints, and multisig / governance migration.
- A clear governance path for Treasury spending, especially builders payouts.

Conclusion:

- If the goal is only to prepare a Fair Launch token narrative, the token principles are now much clearer.
- If the goal is to claim complete Treasury / DAO revenue governance, the current protocol is not complete.
- Token launch should remain NO-GO until at least revenue routing and builders payout governance are clarified.

## 2. Current Confirmed Strengths

### Devnet-verified strengths

- Treasury V2 has a real USDC deposit instruction: `deposit_usdc_revenue`.
- `deposit_usdc_revenue` transfers SPL USDC into four PDA-owned vaults.
- The split constants are `RELIEF_BPS = 5000`, `BUYBACK_BPS = 2000`, `PAYROLL_BPS = 2000`, and `STAKING_BPS = 1000`.
- Treasury V2 tracks `total_usdc_inflow`, `relief_usdc_total`, `buyback_usdc_total`, `builders_usdc_total`, and `staking_usdc_total`.
- Devnet status docs record a completed 20 Devnet USDC deposit through Treasury V2.
- Staking V1 can observe USDC reward vault balance changes and allow stake / claim / unstake.
- Security Layer V1 has GovernanceConfig, ProposalDecision, ExecutionQueueItem, timelock, cancellation, pause, and unpause.
- Green Label refund / slash paths verify Security Layer decisions, queue items, timelock, action type, payload hash, and target account before moving funds.
- Public frontend surfaces Treasury / DAO / Green Label as read-only Devnet views.
- Prelaunch sanity scripts decode Green Label, Security Governance, Treasury V2, and Staking V1 accounts.

### Important boundary

Security Layer V1 is currently an execution guard / queue record layer, not a full DAO voting layer. The generic `execute_queued_action` marks queued actions executed after checks; it does not perform a generic CPI or token transfer. Green Label has separate refund / slash handlers that validate Security Layer queue state before transferring funds.

## 3. Treasury Revenue Split Findings

### Findings

1. Current revenue entry instruction exists: YES.

   `deposit_usdc_revenue(ctx, amount)` exists and calls `deposit_usdc_revenue_handler`.

2. 50 / 20 / 20 / 10 auto split exists in that instruction: YES.

   `calculate_usdc_treasury_split` splits an amount into:

   - 50% relief
   - 20% buyback / burn
   - 20% builders / payroll
   - 10% staking

   The handler transfers each split to a dedicated vault and updates Treasury V2 totals.

3. The split is USDC-only: YES.

   The instruction checks `depositor_usdc_token_account.mint == treasury_config.usdc_mint`, requires a `Mint`, and uses SPL Token `transfer_checked`.

4. SOL revenue support: MISSING.

   No SOL-native revenue split instruction was found. Legacy `deposit(amount)` is accounting-only and does not move SOL or SPL tokens.

5. Green Label revenue automatically enters Treasury V2 accounting: PARTIAL / NO.

   Green Label refund moves the base bond treasury share directly from the project Green Bond Vault to `base_bond_treasury_vault`. Slash moves the bond to `relief_or_risk_vault`. These flows do not call `deposit_usdc_revenue` and do not update `TreasuryUsdcStateV2` revenue totals.

6. External platform revenue automatically enters Treasury: MISSING.

   External platforms must explicitly call `deposit_usdc_revenue` or route USDC into an integration that calls it. There is no automatic listener, oracle, webhook, or cross-program revenue router.

7. Direct payment to an ordinary wallet auto-splits: NO.

   If revenue is sent directly to a wallet or token account without calling `deposit_usdc_revenue`, the protocol will not automatically split or account for it.

8. Current Devnet verified split path:

   Devnet docs record a 20 Devnet USDC deposit through Treasury V2, split across relief, buyback, builders, and staking vaults. Staking V1 then observed the staking vault and claim / unstake was validated.

9. Mainnet revenue integration gaps:

   - Select and document real revenue sources.
   - Define who calls `deposit_usdc_revenue`.
   - Build source-specific deposit scripts or CPI integrations.
   - Decide whether Green Label treasury share should update Treasury V2 accounting.
   - Decide whether Green Label slash proceeds are revenue, risk reserve, or separate relief assets.
   - Add Mainnet-specific scripts with confirmation safeguards.
   - Confirm Mainnet USDC mint and vault addresses.
   - Migrate authorities to multisig / governance.

### Status

Treasury revenue split: DEVNET VERIFIED for explicit USDC deposits; PARTIAL for real revenue operations.

## 4. Builders 20% Findings

### Findings

1. Builders vault exists: YES.

   `builders_usdc_vault` is initialized by Treasury V2 and receives the 20% builders / payroll split from `deposit_usdc_revenue`.

2. Builders 20% accounting exists: YES.

   `TreasuryUsdcStateV2.builders_usdc_total` records cumulative builders allocation.

3. Builders 20% payout is governance-complete: NO.

   No builders payout / withdraw instruction was found that transfers USDC from `builders_usdc_vault` to a concrete recipient.

4. Current state of builders 20%:

   It is accounting and vault funding complete, not governable payout complete.

5. Who can start a builders payout today?

   No dedicated on-chain builders payout request flow exists.

6. Who can approve a builders payout today?

   Security Layer has `ProposalType::PayrollPayout` and `ActionType::PayrollPayout`, and it can queue / execute metadata for that type. But there is no payout execution handler that spends from the builders vault.

7. Need `builder_payout` instruction: YES.

   A minimal production path likely needs an instruction that validates:

   - Security Layer queue item status and timelock.
   - `ActionType::PayrollPayout`.
   - payload hash.
   - recipient.
   - amount.
   - builders vault.
   - vault authority.
   - approved proposal / queue item.

8. Need `BuilderPayoutProposal` / payout request account: LIKELY YES.

   A separate payout request account would make recipient, amount, memo hash, milestone hash, status, and execution metadata auditable.

9. Can Security Layer be reused?

   YES, but only as the execution guard. It can validate decision / queue / timelock / action type / payload hash. A dedicated payout instruction is still needed to move funds.

10. Missing modules for "community DAO decides builders 20%":

   - DAO voting accounts and instructions.
   - Proposal creation UI / instruction.
   - Vote casting and delegation / voting power model.
   - Quorum, threshold, voting period.
   - Vote finalization into ProposalDecision.
   - Builder payout request account.
   - Builders vault payout execution handler.
   - Frontend read-only and, later, controlled write flows.

### Status

Builders 20%: PARTIAL. Accounting and vault allocation are present; payout governance is MISSING.

## 5. DAO / Security Layer Findings

### Current Security Layer

The current on-chain DAO-related layer includes:

- `GovernanceConfigV1`
- `ProposalDecisionV1`
- `ExecutionQueueItemV1`
- `initialize_governance_config`
- `create_proposal_decision`
- `queue_execution`
- `execute_queued_action`
- `cancel_queued_action`
- `pause_security_layer`
- `unpause_security_layer`

### Answers

1. Current layer type:

   The current system is a DAO execution layer / Security Layer, not a complete DAO voting layer.

2. ALPHA holder voting exists:

   MISSING. No ALPHA-token voting instruction or voting power account was found.

3. User proposal creation flow exists:

   MISSING / PARTIAL. `create_proposal_decision` exists, but it is authority-only decision recording, not an open proposal creation flow.

4. Yes / no vote instruction exists:

   MISSING.

5. Quorum / threshold / voting period exists:

   MISSING as enforced voting logic. `ProposalDecisionV1` stores `yes_weight`, `no_weight`, `start_ts`, and `end_ts`, but those values are supplied by authority during decision creation rather than produced by on-chain vote aggregation.

6. Voting result automatically creates ProposalDecision:

   MISSING.

7. Builders payout proposal type exists:

   PARTIAL. `ProposalType::PayrollPayout` and `ActionType::PayrollPayout` exist, but no builders payout execution instruction exists.

8. DAO controls Treasury spending today:

   MISSING / PARTIAL. Security Layer can record and queue a `PayrollPayout` action, but no Treasury payout handler spends from a Treasury vault after queue execution.

9. Minimal DAO Voting MVP needs:

   - `DaoConfigV1` or extension of GovernanceConfig with voting parameters.
   - `ProposalV1` account for proposer, proposal type, payload hash, description / metadata hash, voting window, status.
   - `VoteRecordV1` account to prevent double voting.
   - `VotingPowerSnapshotV1` or deterministic staking / token balance snapshot rule.
   - `create_proposal`.
   - `cast_vote`.
   - `finalize_vote`.
   - `create_proposal_decision_from_vote_result`.
   - quorum / threshold checks.
   - optional delegation.
   - frontend read-only and later guarded write flows.

### Status

DAO execution layer: DEVNET VERIFIED.  
Full DAO voting layer: MISSING.

## 6. Mainnet Treasury Readiness Findings

### Findings

1. Mainnet treasury addresses determined:

   NO. Current docs list Devnet addresses and require Mainnet confirmation.

2. Devnet and Mainnet addresses clearly separated:

   PARTIAL. Devnet docs and sanity scripts distinguish Devnet from Mainnet and require explicit Mainnet inputs. Actual Mainnet addresses are not recorded yet.

3. Authority / guardian / vault authority still deploy-wallet-like:

   DEVNET yes. Devnet sanity report lists the same test wallet for Security authority and emergency guardian. Mainnet authority migration remains pending.

4. Mainnet multisig needed:

   YES. Docs require migration of Treasury / Green Label / Security Layer critical authorities to DAO / multisig / timelock control.

5. Authority migration needed:

   YES. Mainnet prelaunch docs explicitly require authority migration and emergency guardian review.

6. Read-only sanity script exists:

   YES. `devnet:prelaunch:sanity` and `mainnet:prelaunch:sanity` exist and decode Green Label, Security governance, Treasury V2, and Staking V1.

7. Mainnet NO-GO checklist exists:

   YES. `docs/mainnet-go-no-go-checklist.md` exists and keeps Mainnet production as NO-GO.

8. Minimum Mainnet treasury requirements:

   - Mainnet Program ID / IDL confirmation.
   - Mainnet USDC mint confirmation.
   - TreasuryConfigV2 / TreasuryUsdcStateV2 initialization.
   - Four Treasury vault addresses confirmed.
   - Vault authority confirmed as expected PDA / governed authority.
   - Staking pool explicitly provided.
   - ALPHA mint confirmed.
   - Governance / multisig authority migration.
   - Mainnet read-only sanity passes with no FAIL.
   - Final build/test evidence recorded.

### Status

Mainnet Treasury readiness: PARTIAL / BLOCKER for production.

## 7. Token Launch Readiness Assessment

| Area | Status | Assessment |
| --- | --- | --- |
| Fair Launch token principles | READY | Core decisions are confirmed: 1B supply, Fair Launch, no project / team / VC allocation, no initial token buckets. |
| Treasury revenue split | DEVNET VERIFIED | Explicit USDC deposit through `deposit_usdc_revenue` splits 50 / 20 / 20 / 10. |
| Real revenue routing | PARTIAL | No production revenue sources or external integrations are wired. Direct transfers do not auto-split. |
| Builders 20% payout | MISSING | Builders vault and accounting exist; recipient payout governance and payout execution do not. |
| DAO execution layer | DEVNET VERIFIED | Security Layer decision / queue / timelock / cancel / pause is verified. |
| DAO voting layer | MISSING | No ALPHA holder voting, quorum, threshold, vote records, or vote finalization. |
| Green Label revenue routing | PARTIAL | Refund/slash moves funds to configured vaults, but does not update Treasury V2 revenue accounting. |
| Staking rewards funding source | DEVNET VERIFIED | Staking rewards can be funded by the staking USDC vault fed by Treasury V2 deposits. |
| Mainnet treasury addresses | MISSING | Devnet addresses exist; Mainnet addresses not finalized. |
| Authority / multisig readiness | BLOCKER | Docs require migration; current Devnet authority is test-wallet based. |
| Public frontend accuracy | PARTIAL | Public MVP is read-only and mostly accurate, but claims around revenue governance should avoid implying builders payout governance is complete. |
| Token launch readiness | BLOCKER | Fair Launch principles are ready, but revenue routing, builders payout governance, authority migration, and operational launch decisions remain unresolved. |

## 8. Required Work Before Token Launch

### Minimum protocol credibility work

- Clarify whether token launch can happen before complete DAO voting, or explicitly position DAO voting as pending.
- Define real revenue sources and how each source calls `deposit_usdc_revenue`.
- Decide how Green Label base treasury share and slash proceeds should be represented in Treasury accounting.
- Add or plan a production-grade builders payout flow.
- Decide whether Builders 20% is held until DAO voting exists or can be paid by multisig under published policy.
- Confirm Mainnet Treasury addresses and vault authorities.
- Confirm Mainnet ALPHA mint / USDC mint / staking pool.
- Complete authority migration plan.
- Run and record Mainnet read-only sanity.

### Minimum builders payout work

- Add `BuilderPayoutRequestV1` or equivalent payout request account.
- Add `execute_builder_payout` or equivalent instruction.
- Gate payout execution through Security Layer decision / queue / timelock / payload hash.
- Require recipient, amount, builders vault, and memo / milestone hash verification.
- Add frontend read-only display of payout requests and execution state.
- Add docs clarifying builders payout is governance-reviewed, not arbitrary.

### Minimum DAO voting MVP work

- Define voting power source: ALPHA balance, staking weight, or snapshot.
- Add proposal account and vote record account.
- Add create / vote / finalize instructions.
- Implement quorum, threshold, and voting period.
- Generate ProposalDecision from finalized vote result.
- Integrate Security Layer queue execution after vote finalization.

## 9. Recommended Next Phases

### Phase 2E-2: Revenue Routing Design

Document and implement how real project revenue enters Treasury:

- Green Label fees / bond treasury share.
- Protocol service fees.
- External platform / launch partner revenue.
- Manual revenue deposit fallback.
- Direct transfer warning policy.

### Phase 2E-3: Builders Payout Governance Design

Design the builders payout path:

- Payout request account.
- Payout metadata hash.
- Recipient / amount validation.
- Security Layer integration.
- Multisig fallback policy before full DAO voting.

### Phase 2E-4: Minimal DAO Voting MVP Design

Define and implement voting primitives:

- Proposal creation.
- Vote records.
- Voting power.
- Quorum / threshold.
- Finalization.
- ProposalDecision generation.

### Phase 2E-5: Mainnet Treasury Operations Plan

Finalize:

- Mainnet treasury addresses.
- Vault owner / mint / authority checks.
- Revenue operator policy.
- Multisig authority migration.
- Mainnet sanity report format.

### Phase 2E-6: Public Messaging Tightening

Update public language to distinguish:

- Treasury split verified vs real revenue integrations pending.
- Builders vault funded vs builders payout governance pending.
- Security Layer execution guard vs full DAO voting pending.
- Fair Launch token principles ready vs token launch still NO-GO.

## Final Conclusion

Alpha Protocol is credible as a Devnet-verified Public MVP with strong Treasury V2, Green Label, staking, and Security Layer foundations.

It is not yet credible to claim complete DAO-governed revenue distribution or community-controlled builders payouts.

Token launch should remain NO-GO until at least:

- real revenue routing is defined,
- builders payout governance is clarified,
- authority / multisig migration plan is actionable,
- Mainnet treasury addresses are confirmed,
- communication avoids overstating DAO voting or revenue governance completion.
