# Mainnet Go/No-Go Checklist

Date: 2026-07-08

## 1. Purpose

This document is the final Go/No-Go decision entry point before Alpha Protocol Mainnet launch. It does not replace technical checks, authority reviews, build verification, or read-only sanity scripts. It summarizes the required preconditions and decision gates.

If any BLOCKER remains unresolved, the final decision must be NO-GO.

## 2. Linked Prelaunch Documents

- `docs/mainnet-prelaunch-hardening-checklist.md`: Mainnet safety gates, blocker list, script isolation, launch protection, sanity checks, and final test requirements.
- `docs/mainnet-authority-and-parameter-migration-plan.md`: Required authority migration and Green Label parameter restoration plan before Mainnet.
- `docs/prelaunch-sanity-devnet-report.md`: Current Devnet read-only sanity baseline.

The Devnet report only records Devnet health. It is not Mainnet approval. Before Mainnet, `mainnet:prelaunch:sanity` must be run against the intended Mainnet Program ID, config, vaults, and staking pool.

## 3. Current Devnet Baseline

- Program ID: `HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY`
- Green Label config: `7hNAeoqZxqvp38giY9gZwfR5ai3ttYTrse63QNYrRBWS`
- Security governance config: `5np4fcpSP8eHVLD6dsgLHf7H11VLaGcYgdadxidt9ro3`
- Treasury USDC state V2: `5e7eyC5ViwH9GBn73cY6so7J6KpRCX6XsbxozHabk2fE`
- Staking pool V1: `91PjLExu9FCLY6KQuvuisEhTEciQyWXJGW9fMKUEHW35`
- Devnet sanity result: PASS with WARN only
- FAIL: none
- MANUAL_REVIEW: none

## 4. Mainnet Required Parameters

- Green Label `min_base_bond_usdc = 299 USDC`
- `observation_period_seconds = 2592000`
- `dispute_window_seconds = 604800`
- `response_window_seconds = 259200`
- Security timelock delay must be nonzero.
- Security config must not be paused.
- Green Label config must not be paused.
- Mainnet `STAKING_POOL` must be explicitly provided to the sanity check.

## 5. Go/No-Go Decision Table

| Area | Requirement | Status | Evidence | Decision |
| --- | --- | --- | --- | --- |
| Program / IDL | Mainnet Program ID and IDL address must match deployed program. | Pending Mainnet | `mainnet:prelaunch:sanity` required. | REVIEW |
| Green Label parameters | Mainnet must use 299 USDC / 30 days / 7 days / 3 days. | Pending Mainnet | Parameter restoration plan required. | BLOCKER |
| Green Label authority | `GreenLabelConfig` authority must not remain long-term single-wallet controlled without governance. | Manual review required | Authority migration plan required. | REVIEW |
| Security governance | Timelock nonzero, config unpaused, governance authority reviewed. | Pending Mainnet | `mainnet:prelaunch:sanity` and authority review required. | BLOCKER |
| Treasury V2 vaults | Four-pool vaults, mint, owner, and authority must be confirmed. | Pending Mainnet | Treasury V2 sanity decoder and manual review required. | REVIEW |
| Staking V1 pool | Explicit Mainnet staking pool, ALPHA mint, rewards vault, and authority must be confirmed. | Pending Mainnet | `STAKING_POOL` must be provided to sanity check. | BLOCKER |
| Devnet/Mainnet script isolation | Devnet scripts must reject Mainnet endpoints; Mainnet scripts must be separately named and confirmed. | Devnet complete | Script isolation documented in hardening checklist. | REVIEW |
| Read-only sanity check | Mainnet read-only sanity must pass with no FAIL items. | Pending Mainnet | `mainnet:prelaunch:sanity` not yet recorded here. | BLOCKER |
| Frontend cluster display | Frontend must clearly show current cluster. | Manual review required | Frontend launch protection checklist. | REVIEW |
| Frontend launch guard | Devnet parameters must not be shown as Mainnet rules; Green Label must show legal risk disclaimers. | Manual review required | Dashboard launch guard required. | REVIEW |
| Documentation | Prelaunch hardening, authority migration, Devnet sanity, and Go/No-Go docs must be reviewed. | Devnet complete | Linked documents exist. | REVIEW |
| Final build/test | `cargo test`, `anchor build --ignore-keys`, and frontend build must pass before Mainnet. | Pending Mainnet | Final commands not recorded here. | BLOCKER |
| Mainnet operational runbook | Mainnet setup and emergency procedures must be separated from Devnet and reviewed. | Not started | Mainnet runbook required. | BLOCKER |

## 6. Absolute BLOCKERS

Any one of the following means NO-GO:

- Mainnet Green Label parameters are not `299 USDC / 30 days / 7 days / 3 days`.
- Mainnet sanity check reports any FAIL.
- Program ID / IDL mismatch.
- `GreenLabelConfig` authority remains long-term single-wallet controlled without a governance plan.
- Security timelock delay is `0`.
- Security config is paused.
- Devnet scripts can run against a Mainnet endpoint.
- Mainnet `STAKING_POOL` is not explicitly provided.
- Treasury vault mint mismatch.
- Staking rewards vault missing.
- Frontend cannot clearly display cluster.
- Frontend presents Devnet test parameters as Mainnet production rules.
- Final `cargo test`, `anchor build --ignore-keys`, and `npm run build` are not complete.

## 7. Manual Review Required

The following items must be manually confirmed before any Mainnet GO decision:

- Program upgrade authority.
- `GreenLabelConfig` authority.
- Security governance authority.
- Emergency guardian is pause-only.
- Treasury vault authority is the expected PDA or governance-controlled authority.
- Mainnet USDC mint.
- ALPHA mint.
- Staking pool authority.
- Mainnet RPC provider.
- Explorer links use the correct cluster.
- Legal and messaging risk: Green Label is not insurance, not a credit rating, and not investment advice.

## 8. Final Command Checklist

Do not treat this list as already executed. These commands must be run in the appropriate final Mainnet readiness window.

Devnet final:

```bash
cd server
npm run devnet:prelaunch:sanity
```

Mainnet final:

```bash
cd server
RPC_URL=<MAINNET_RPC_URL> \
GREEN_LABEL_CONFIG=<MAINNET_GREEN_LABEL_CONFIG> \
STAKING_POOL=<MAINNET_STAKING_POOL> \
npm run mainnet:prelaunch:sanity
```

Build/test:

```bash
cd server
cargo test
anchor build --ignore-keys

cd ../project
npm run build
```

## 9. Decision Template

Decision:

- GO / NO-GO

Date:

Reviewer:

Commit:

Mainnet Program ID:

Mainnet Green Label Config:

Mainnet Staking Pool:

Reason:

Open blockers:

Manual review notes:

## 10. Operational Runbook

- Review `docs/mainnet-operational-runbook.md` before any Mainnet launch-day execution.
- After the Go/No-Go checklist reaches GO, Mainnet launch must follow the operational runbook.
- The runbook cannot bypass blockers. If any blocker appears during execution, stop and return to this Go/No-Go checklist.

## 11. DAO Governance Product Layer

- Review `docs/dao-governance-product-layer.md` before Mainnet.
- Mainnet readiness includes clear display of the DAO execution layer, not only backend safety checks.
- Until the full voting layer is complete, Alpha Protocol must not be marketed as a fully decentralized DAO.
- The read-only DAO Governance Dashboard may show Security Layer state, proposal decisions, and queue items, but must not expose voting or execution buttons.

## 12. Public MVP / Litepaper

- Review `docs/alpha-protocol-litepaper.md` before Mainnet or token launch communications.
- Mainnet / token launch must have clear public explanations of protocol scope, current status, and risk boundaries.
- The Litepaper must not promise yield, insurance, fixed compensation, dividends, or token price appreciation.
- Until the full ALPHA voting layer is complete, Alpha Protocol must not be marketed as a fully decentralized DAO.
- Public MVP pages must keep Devnet verified / Mainnet not live / read-only status visible.

## 13. Public MVP Launch Readiness Review

- Review `docs/public-mvp-launch-readiness-review.md` before public preview, community preheating, Mainnet launch, or token launch communication.
- Current Public MVP preview can be GO with warnings.
- Current community preheating can be GO with warnings.
- Mainnet production remains NO-GO.
- Immediate token launch remains NO-GO.
- Public preview must not be confused with Mainnet readiness or token launch approval.

## 14. Tokenomics Finalization

- Review `docs/alpha-tokenomics-draft.md` before any token launch decision.
- Fair Launch model, total supply, no project / team / VC allocation, no initial token bucket allocation, and Treasury revenue split are confirmed.
- Launch platform, initial liquidity setup, pairing asset, mint / freeze authority policy, liquidity custody / LP handling, official communication, anti-scam warnings, and risk disclosure review remain token launch preconditions.
- If any pending operational launch decision remains unresolved, token launch decision must be NO-GO.
- The tokenomics draft does not approve immediate token launch and must not be used as a yield, dividend, insurance, or price appreciation promise.

## 15. Tokenomics Allocation Review

- Review `docs/tokenomics-allocation-review.md` before tokenomics freeze or token launch decision.
- The old allocation model is superseded.
- Alpha Protocol will not use initial token allocation buckets at launch.
- Core Fair Launch decisions are confirmed.
- Token launch remains NO-GO until pending operational launch decisions are completed.
- Launch platform, initial liquidity setup, pairing asset, liquidity custody / LP handling, authority policy, and communication must be finalized before token launch.
- The `50 / 20 / 20 / 10` split is protocol revenue split, not token supply allocation.

## 16. Tokenomics Final Decision Draft

- Review `docs/alpha-tokenomics-final-decision-draft.md` before token launch approval.
- The current recommended final decision draft is Fair Launch with no team, VC, project-side, DAO treasury, staking reserve, builders, or airdrop token buckets at launch.

- `1,000,000,000 ALPHA` is confirmed total supply.
- Core Fair Launch decisions are recorded in `docs/fair-launch-confirmed-decisions.md`.
- Token launch requires final operational launch sign-off.
- Unresolved launch platform, initial liquidity setup, pairing asset, mint / freeze authority policy, liquidity custody / LP handling, official communication, anti-scam warnings, and legal / risk review remain blockers.
- The final decision draft does not approve immediate token launch.

## 17. Green Label Strict Forfeit Governance

- Review `docs/green-label-strict-forfeit-governance-v1.md` before any Mainnet Green Label slash / forfeit readiness decision.
- Strict forfeit requires typed governance action binding, Green Label module registry, adapter output, approved Security decision, executed queue item, recorded escrow liability, and Treasury Router routing as `GreenLabelForfeitedBond`.
- Legacy Green Label slash / forfeit public entry points are disabled in the local implementation.
- This improves the Green Label fund path, but Mainnet production and token launch remain NO-GO until Mainnet parameters, authorities, vaults, sanity checks, and final build/test are complete.

## 18. Fair Launch Decision Gates

- Review `docs/fair-launch-decision-checklist.md` before any token launch approval.
- Completed gates:
  - Fair Launch model confirmed.
  - Total supply confirmed: `1,000,000,000 ALPHA`.
  - No project / team / VC allocation confirmed.
  - Initial token bucket allocation confirmed: none.
  - Treasury revenue split confirmed: `50 / 20 / 20 / 10` applies only to future protocol revenue.
- Remaining blockers:
  - Launch platform must be confirmed.
  - Initial liquidity setup must be confirmed.
  - Pairing asset must be confirmed.
  - Mint / freeze authority policy must be handled and publicly disclosed.
  - Liquidity custody / LP handling must be confirmed if applicable.
  - Official communication must be reviewed.
  - Anti-scam warning must be ready.
  - Final Go/No-Go must be completed.
- Communication must clearly distinguish Fair Launch from Treasury protocol revenue split.
- Communication must state that `50 / 20 / 20 / 10` is future protocol revenue split, not token allocation.
- Token launch remains NO-GO until these gates are completed.

## 19. Launch Communication Review

- Review `docs/launch-communication-and-community-preheat-plan.md` before any public preview announcement, Litepaper publication, community pinned message, X thread, or token launch communication.
- Token launch requires completed communication review.
- Red-flag language is prohibited, including guaranteed returns, fixed APY, passive income guarantee, insurance payout, risk-free, guaranteed buyback, price floor, price protection, hidden project-side token reserve, Fair Launch means risk-free, Mainnet live before Mainnet is live, and official token live before token launch is approved.
- Communication must clearly distinguish ALPHA Fair Launch from Treasury protocol revenue split.
- Community preheating may proceed only with warnings: Mainnet not live, token launch pending, planned Fair Launch, read-only Public MVP, and ALPHA voting layer pending.

## 20. Public MVP Safety Cleanup

- Public preview requires Devnet write buttons hidden by default.
- Any developer-only Devnet actions must require an explicit environment flag, such as `VITE_SHOW_DEVNET_ACTIONS=true`.
- Red-flag wording must be removed or replaced before public preview.
- Public MVP must remain read-only by default with no buy button, real funds entry, or on-chain write action exposed.

## 21. Unified USDC Revenue Router

- Review `docs/unified-usdc-revenue-routing-design.md` before claiming protocol revenue routing readiness.
- `route_usdc_revenue_v1` is implemented for typed USDC protocol revenue routing.
- `RevenueRoutingStatsV1` tracks typed USDC revenue totals without changing `TreasuryUsdcStateV2`.
- `deposit_usdc_revenue` remains a legacy/simple Treasury V2 USDC deposit path.
- Green Label certification fee routing is implemented as `RevenueType::GreenLabelCertificationFee` through `route_green_label_certification_fee_once_v1`.
- `GreenLabelCertificationFeePolicyV1` is the authoritative fee amount source.
- `GreenLabelCertificationFeeReceiptV1` records one immutable receipt per project.
- The legacy caller-amount Green Label fee route is disabled with `LegacyGreenLabelCertificationFeeRouteDisabled`.
- Refundable Green Label escrow is implemented as `GreenLabelRefundableEscrowV1`.
- Refundable escrow refunds only to the original payer and does not pass through Treasury split.
- Green Label forfeited escrow routes to Treasury as `RevenueType::GreenLabelForfeitedBond`.
- Green Label forfeits must require a valid dispute, dispute-ready / decision-queued state, linked Security Layer / Green Label slash decision, and non-terminal escrow state.
- No time-only forfeit path is allowed.
- Bond lock / PendingObservation receipt gate and approve certification receipt gate are still pending Stage 5B-4B-2.
- SOL revenue split is not supported.
- Builders payout governance is not implemented.
- Token launch remains NO-GO until real revenue integrations, builders payout governance, Mainnet authorities, and final launch checks are completed.

## 22. Current Conclusion

NO-GO for Mainnet production until Mainnet parameters, authorities, vaults, staking pool, and mainnet sanity check are completed and reviewed.
