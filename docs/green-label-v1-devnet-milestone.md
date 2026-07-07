# Green Label V1 Devnet E2E Milestone

Date: 2026-07-07

This document seals the Green Label V1 Devnet refund/slash E2E milestone.

## Devnet Program

Devnet Program ID:

```text
HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY
```

Green Label config:

```text
7hNAeoqZxqvp38giY9gZwfR5ai3ttYTrse63QNYrRBWS
```

## Devnet Test Parameters

These values are Devnet-only test parameters used to make E2E validation fast and feasible with Devnet USDC constraints.

```text
min_base_bond_usdc = 1 USDC
observation_period_seconds = 30
dispute_window_seconds = 30
response_window_seconds = 30
```

Mainnet production parameters must be restored before any Mainnet launch:

```text
min_base_bond_usdc = 299 USDC
observation_period_seconds = 2592000
dispute_window_seconds = 604800
response_window_seconds = 259200
```

## Refund E2E Result

```text
project_id = 2
status = Refunded
dispute status = ResolvedRefund
bond = 1 USDC
project_owner_usdc_ata delta = -0.2 USDC
base_bond_treasury_vault delta = +0.2 USDC
green_bond_vault final = 0
```

Interpretation: the 1 USDC base bond used the configured 80% refund / 20% treasury split. The project owner net delta is -0.2 USDC because the owner first deposited 1 USDC and later received 0.8 USDC back.

## Slash E2E Result

```text
project_id = 3
status = Slashed
dispute status = ResolvedSlash
bond = 1 USDC
project_owner_usdc_ata delta = -1 USDC
relief_or_risk_vault delta = +1 USDC
green_bond_vault final = 0
```

Interpretation: the full 1 USDC bond was slashed to the configured relief/risk vault.

## Verified Flow

The Devnet E2E runs verified the following Green Label and Security Layer path:

1. `submit_green_label_application`
2. `initialize_green_bond_vault`
3. `lock_green_label_bond`
4. `open_green_label_dispute`
5. `mark_dispute_ready_for_decision`
6. Create Security Layer decision via `create_proposal_decision`
7. Queue execution via `queue_execution`
8. `link_green_label_security_decision`
9. `execute_green_label_refund`
10. `execute_green_label_slash`

## Mainnet Readiness Blockers

Before Mainnet, the following items must be handled:

1. Restore production parameters:
   - `min_base_bond_usdc = 299 USDC`
   - `observation_period_seconds = 2592000`
   - `dispute_window_seconds = 604800`
   - `response_window_seconds = 259200`
2. Migrate config authority from a single wallet to DAO / multisig / Security Layer timelock control.
3. Clearly separate Devnet scripts from Mainnet scripts.
4. Audit update-config authority risk, especially the ability to change windows or minimum bond settings.

