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

## 13. Current Conclusion

NO-GO for Mainnet production until Mainnet parameters, authorities, vaults, staking pool, and mainnet sanity check are completed and reviewed.
