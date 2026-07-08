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

## 5. Devnet / Mainnet Script Separation

- Devnet scripts must not be used directly for Mainnet.
- Mainnet scripts must use separate names, for example `mainnet:green-label:setup`.
- Mainnet scripts must force-print cluster / program id / authority / config parameters.
- Mainnet scripts must include a human confirmation mechanism to prevent accidental transactions.

## 6. Frontend Launch Protection

- The frontend must clearly display the current cluster.
- Mainnet mode must not present Devnet test parameters as production rules.
- Green Label pages must display: not investment advice, not an insurance promise, and risk bond / commitment tier is not a credit rating.
- The frontend must show warnings when Mainnet parameters are abnormal.

## 7. Program / IDL / Explorer Checks

- Program ID and IDL address must match.
- The frontend IDL must match the deployed program version.
- Explorer links must switch by cluster.
- A read-only sanity check must be run before Mainnet launch.

## 8. Funds-Flow Checks

- Treasury V2 four-pool addresses must be confirmed.
- Green Label `base_bond_treasury_vault` must be confirmed.
- Green Label `relief_or_risk_vault` must be confirmed.
- Staking rewards vault must be confirmed.
- Every vault owner / mint / authority must be checked.

## 9. Tests That Must Not Be Skipped

- `cargo test`
- `anchor build --ignore-keys`
- Frontend `npm run build`
- Devnet final sanity check
- Security Layer pause / unpause / cancel path sanity check
- Green Label refund / slash read-only verification
- Treasury / Staking read-only verification

## 10. Mainnet Blockers

Any one of the following unresolved items blocks Mainnet launch:

- Devnet parameters are not restored or Mainnet parameters are not confirmed.
- Authority remains under single-wallet control without a governance plan.
- Devnet scripts and Mainnet scripts are not separated.
- The frontend cannot clearly display cluster.
- IDL and Program are inconsistent.
- Vault addresses are not reviewed.
- Security Layer is not enabled or timelock can be bypassed.
- Final build / test is not complete.
