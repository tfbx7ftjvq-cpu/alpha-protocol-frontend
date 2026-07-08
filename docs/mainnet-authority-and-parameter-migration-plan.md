# Mainnet Authority and Parameter Migration Plan

Date: 2026-07-08

This document defines the Mainnet parameter restoration plan and authority migration order for Alpha Protocol Treasury V2, Green Label V1, Security Layer V1, and Staking V1.

## 1. Current Devnet Status Summary

- Program ID: `HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY`
- Green Label config: `7hNAeoqZxqvp38giY9gZwfR5ai3ttYTrse63QNYrRBWS`

Current Green Label Devnet test parameters:

- `min_base_bond_usdc = 1 USDC`
- `observation_period_seconds = 30`
- `dispute_window_seconds = 30`
- `response_window_seconds = 30`

These parameters exist only to support fast Devnet E2E validation. They are not allowed as Mainnet default parameters.

## 2. Mainnet Production Parameter Restoration Plan

Mainnet must restore the formal Green Label parameters before launch:

- `min_base_bond_usdc = 299 USDC`
- `observation_period_seconds = 2592000` (30 days)
- `dispute_window_seconds = 604800` (7 days)
- `response_window_seconds = 259200` (3 days)

Devnet may keep `1 USDC / 30s / 30s / 30s` as a test environment configuration. Mainnet setup must initialize with the production parameters by default. If Mainnet parameters are abnormal, the frontend must show a Launch Guard warning. Mainnet parameter changes must go through governance and must not be instantly controlled by a single wallet.

## 3. Authority Inventory

The following authority types require governance review or migration before Mainnet:

- Program upgrade authority
- Treasury V2 authority
- `GreenLabelConfig` authority
- Security Layer governance authority
- Emergency guardian
- Staking pool authority, if present in the current deployed implementation
- Vault authority / PDA authority

Signer authority and PDA vault authority must be treated differently. A signer authority is controlled by a private key or signing governance system and can authorize privileged actions. A PDA vault authority is a program-derived address, not a private key, and must not be migrated into a normal wallet. The real governance risk is attached to authorities that can modify configuration, upgrade the program, pause / unpause, queue execution, execute actions, or otherwise control critical protocol behavior.

## 4. Recommended Mainnet Governance Structure

### Short-Term Mainnet Beta

- Use multisig control for critical authorities.
- Use Security Layer timelock control for parameter changes.
- Emergency guardian may pause only.
- Emergency guardian must not unpause.
- Emergency guardian must not transfer funds.

### Medium-Term

- Move toward DAO governance plus Security Layer timelock.
- Parameter changes must go through proposal / queue / timelock.
- Green Label refund / slash execution must go through proposal / queue / timelock.
- Major treasury actions must go through proposal / queue / timelock.

### Long-Term

- Migrate program upgrade authority to governance / multisig.
- Govern all protocol-critical parameters.
- Minimize emergency powers and keep them narrowly scoped.

## 5. Green Label Update Instruction Risk

The following Green Label instructions are sensitive:

- `update_green_label_windows`
- `update_green_label_min_base_bond`

If these instructions remain under single-wallet control, the wallet could lower Mainnet parameters to `1 USDC / 30s`, weaken observation / dispute / response windows, and damage Green Label credibility.

Mainnet requirements:

- Both instructions must be controlled by governance / multisig / timelock.
- Parameter updates must have event or documentation records.
- The frontend must display whether current parameters deviate from formal rules.
- Devnet scripts must not operate Mainnet config.

## 6. Mainnet Parameter Restoration Execution Order

Step 1: Confirm Mainnet Program ID / IDL / deploy authority.

Step 2: Initialize Mainnet `GreenLabelConfig` with `299 USDC / 30 days / 7 days / 3 days`.

Step 3: Confirm Treasury vault / `relief_or_risk_vault` / `base_bond_treasury_vault`.

Step 4: Configure Security Layer governance config.

Step 5: Migrate `GreenLabelConfig` authority to multisig / governance / timelock.

Step 6: Migrate or lock Program upgrade authority.

Step 7: Run read-only sanity check.

Step 8: Recheck parameters before switching the frontend to Mainnet cluster.

## 7. Blocker Checklist

Any one of the following unresolved items blocks Mainnet launch:

- Green Label parameters are not `299 USDC / 30 days / 7 days / 3 days`.
- `GreenLabelConfig` authority remains under long-term single-wallet control.
- Update config instructions are not governed or timelocked.
- Devnet scripts are not isolated.
- Mainnet scripts lack a human confirmation mechanism.
- Program ID / IDL are inconsistent.
- Vault address / mint / owner / authority are not reviewed.
- Security Layer timelock is not enabled.
- Emergency guardian permissions are too broad.
- Frontend cannot display cluster and parameter warnings.

## 8. Relationship To Existing Checklist

This migration plan is a required companion document for `docs/mainnet-prelaunch-hardening-checklist.md`. The checklist defines launch gates; this document defines the parameter and authority migration plan that must be reviewed before Mainnet.
