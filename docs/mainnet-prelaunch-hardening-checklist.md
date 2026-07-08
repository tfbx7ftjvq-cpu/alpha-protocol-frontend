# Mainnet Pre-Launch Hardening Checklist

Date: 2026-07-08

This checklist defines the required safety gates before any Alpha Protocol Mainnet launch. If any blocker in this document is unresolved, Mainnet launch is not allowed.

## 1. Current Status Overview

- Treasury V2 Devnet has completed USDC four-pool routing.
- Staking V1 Devnet has completed stake / claim / unstake validation.
- Security Layer V1 Devnet has completed decision / queue / timelock / cancel / pause validation.
- Green Label V1 Devnet has completed refund / slash E2E validation.
- Green Label Dashboard has completed read-only on-chain display for config / project / dispute accounts.

## 2. Green Label Devnet Test Parameters

These are Devnet E2E test parameters only:

- `min_base_bond_usdc = 1 USDC`
- `observation_period_seconds = 30`
- `dispute_window_seconds = 30`
- `response_window_seconds = 30`

## 3. Mainnet Production Parameters To Restore

Mainnet must restore and confirm the production parameters before launch:

- `min_base_bond_usdc = 299 USDC`
- `observation_period_seconds = 2592000` (30 days)
- `dispute_window_seconds = 604800` (7 days)
- `response_window_seconds = 259200` (3 days)

## 4. Authority Governance Required Before Mainnet

- `GreenLabelConfig` authority must not remain under long-term single-wallet control.
- Treasury / Green Label / Security Layer critical authorities should migrate to DAO / multisig / Security Layer timelock control.
- `update_green_label_windows` and `update_green_label_min_base_bond` must be gated by governance or timelock flow.
- Emergency guardian permissions and boundaries must be explicitly documented and reviewed.

## 5. Authority and Parameter Migration Plan

Before Mainnet, review `docs/mainnet-authority-and-parameter-migration-plan.md`. That document is the required companion plan for restoring Green Label production parameters and migrating critical authorities before launch.

## 6. Devnet / Mainnet Script Separation

- Devnet scripts must not be used directly for Mainnet.
- Mainnet scripts must use separate names, for example `mainnet:green-label:setup`.
- Mainnet scripts must force-print cluster / program id / authority / config parameters.
- Mainnet scripts must include a human confirmation mechanism to prevent accidental transactions.

## 7. Devnet Script Isolation Status

- Current Devnet scripts use the `devnet:...` npm script namespace.
- Green Label Devnet scripts include a devnet-only guard.
- Green Label Devnet scripts reject `mainnet` / `mainnet-beta` endpoints.
- Future Mainnet scripts must be separately named as `mainnet:...`.
- Mainnet scripts must include an explicit human confirmation mechanism.
- Devnet `1 USDC / 30s / 30s / 30s` parameters must never be reused as Mainnet default parameters.

## 8. Frontend Launch Protection

- The frontend must clearly display the current cluster.
- Mainnet mode must not present Devnet test parameters as production rules.
- Green Label pages must display: not investment advice, not an insurance promise, and risk bond / commitment tier is not a credit rating.
- The frontend must show warnings when Mainnet parameters are abnormal.

## 9. Program / IDL / Explorer Checks

- Program ID and IDL address must match.
- The frontend IDL must match the deployed program version.
- Explorer links must switch by cluster.
- A read-only sanity check must be run before Mainnet launch.

## 10. Read-only Sanity Check

- `devnet:prelaunch:sanity` runs the Devnet read-only prelaunch sanity check.
- `mainnet:prelaunch:sanity` runs the Mainnet read-only prelaunch sanity check.
- These scripts only read local files and on-chain accounts; they must not send transactions.
- The sanity check now raw decodes Security Layer `GovernanceConfigV1`.
- The sanity check also raw decodes Treasury V2 `TreasuryConfigV2` / `TreasuryUsdcStateV2` and Staking V1 `StakingPoolV1`.
- Mainnet launch requires manual confirmation of Security governance authority and emergency guardian control.
- If a Security governance timelock delay field exists and is `0`, that is a Mainnet blocker.
- Mainnet launch requires manual confirmation of Treasury four-pool vaults, USDC mint, and vault authority.
- Mainnet launch requires manual confirmation of staking pool, ALPHA mint, USDC rewards vault, and staking authority.
- `mainnet:prelaunch:sanity` must receive an explicit `STAKING_POOL`; missing `STAKING_POOL` is a blocker.
- Any Treasury / Staking decoder, vault, mint, or owner check `FAIL` blocks Mainnet launch.
- Mainnet launch requires `mainnet:prelaunch:sanity` to pass before any production action.
- Any `FAIL` result blocks Mainnet launch.
- Any `MANUAL_REVIEW` result must be manually confirmed before Mainnet launch.

## 11. Devnet Prelaunch Sanity Report

- Review `docs/prelaunch-sanity-devnet-report.md` for the current Devnet sanity baseline.
- The report records Devnet health only and cannot replace Mainnet sanity.
- Before Mainnet, `mainnet:prelaunch:sanity` must still be run against the intended Mainnet Program ID, config, vaults, and staking pool.

## 12. Mainnet Go/No-Go Checklist

- Review `docs/mainnet-go-no-go-checklist.md` as the final Mainnet launch decision entry point.
- The Go/No-Go checklist summarizes required evidence across technical checks, authority migration, sanity checks, frontend launch guard, and operations.
- If any blocker remains unresolved, the Mainnet decision must be NO-GO.

## 13. Funds-Flow Checks

- Treasury V2 four-pool addresses must be confirmed.
- Green Label `base_bond_treasury_vault` must be confirmed.
- Green Label `relief_or_risk_vault` must be confirmed.
- Staking rewards vault must be confirmed.
- Every vault owner / mint / authority must be checked.

## 14. Tests That Must Not Be Skipped

- `cargo test`
- `anchor build --ignore-keys`
- Frontend `npm run build`
- Devnet final sanity check
- Security Layer pause / unpause / cancel path sanity check
- Green Label refund / slash read-only verification
- Treasury / Staking read-only verification

## 15. Mainnet Blockers

Any one of the following unresolved items blocks Mainnet launch:

- Devnet parameters are not restored or Mainnet parameters are not confirmed.
- Authority remains under single-wallet control without a governance plan.
- Devnet scripts and Mainnet scripts are not separated.
- The frontend cannot clearly display cluster.
- IDL and Program are inconsistent.
- Vault addresses are not reviewed.
- Security Layer is not enabled or timelock can be bypassed.
- Final build / test is not complete.
