# Mainnet Operational Runbook

Date: 2026-07-08

## 1. Purpose

This document is the operational runbook for Alpha Protocol Mainnet launch day. It does not replace the Go/No-Go checklist.

Only execute this runbook after `docs/mainnet-go-no-go-checklist.md` reaches a GO decision. If any blocker appears during execution, stop the launch process and return to the Go/No-Go checklist.

## 2. Roles and Responsibilities

Release Lead:

- Gives the final Go/No-Go confirmation.
- Controls launch pacing.
- Decides whether to pause, stop, or terminate the launch process.

Contract Operator:

- Executes contract-related commands.
- Must not independently change parameters.
- Must have every command reviewed by a second person before execution.

Frontend Operator:

- Runs frontend build and deployment steps.
- Confirms cluster display and Launch Guard behavior.

Security Reviewer:

- Reviews authorities, timelock, emergency guardian, and vault authority.
- Reviews sanity check results.

Treasury Reviewer:

- Reviews Treasury vaults, USDC mint, relief/risk vault, and staking rewards vault.

Communications Lead:

- Prepares announcement channels.
- Confirms Green Label messaging states that it is not insurance, not a credit rating, and not investment advice.

## 3. Pre-Launch Freeze

The following freeze rules apply before launch:

- No last-minute contract changes.
- No last-minute IDL changes.
- No last-minute Program ID changes.
- No last-minute keypair or deploy authority changes.
- No last-minute Mainnet parameter changes.
- Do not directly reuse Devnet scripts.
- Do not modify frontend cluster or config without review.
- Any last-minute change must return to the Go/No-Go checklist for renewed evaluation.

## 4. Required Inputs

Launch day requires the following values:

- `MAINNET_RPC_URL`
- `MAINNET_PROGRAM_ID`
- `MAINNET_GREEN_LABEL_CONFIG`
- `MAINNET_TREASURY_USDC_STATE_V2`
- `MAINNET_STAKING_POOL`
- `MAINNET_USDC_MINT`
- `MAINNET_ALPHA_MINT`
- Program upgrade authority
- `GreenLabelConfig` authority
- Security governance authority
- Emergency guardian
- Treasury vault addresses
- Staking vault / rewards vault addresses
- Frontend deployment target

Do not use Devnet addresses as Mainnet addresses. Every address must be reviewed by two people.

## 5. Pre-Launch Command Sequence

Do not run these commands until the final launch window.

Step 1: confirm clean git state

```bash
cd <repo>
git status
git log --oneline -10
```

Step 2: server tests

```bash
cd server
cargo test
anchor build --ignore-keys
```

Step 3: frontend build

```bash
cd ../project
npm run build
```

Step 4: Devnet final sanity

```bash
cd ../server
npm run devnet:prelaunch:sanity
```

Step 5: Mainnet read-only sanity

```bash
RPC_URL=<MAINNET_RPC_URL> \
GREEN_LABEL_CONFIG=<MAINNET_GREEN_LABEL_CONFIG> \
TREASURY_USDC_STATE_V2=<MAINNET_TREASURY_USDC_STATE_V2> \
STAKING_POOL=<MAINNET_STAKING_POOL> \
PROGRAM_ID=<MAINNET_PROGRAM_ID> \
npm run mainnet:prelaunch:sanity
```

Step 6: review `PASS / WARN / FAIL / MANUAL_REVIEW`.

Step 7: if `FAIL` is non-empty, STOP and mark NO-GO.

Step 8: if `MANUAL_REVIEW` is non-empty, complete signed review before continuing.

## 6. Mainnet Parameter Verification

Green Label:

- `min_base_bond_usdc = 299 USDC`
- `observation_period_seconds = 2592000`
- `dispute_window_seconds = 604800`
- `response_window_seconds = 259200`
- `is_paused = false`

Security Layer:

- `min_execution_delay_seconds > 0`
- `is_paused = false`
- Emergency guardian permissions have been manually confirmed.

Staking:

- Mainnet `STAKING_POOL` is explicitly provided.
- ALPHA mint has been confirmed.
- USDC rewards mint has been confirmed.
- Staking vault and rewards vault have been confirmed.

Treasury:

- Mainnet USDC mint has been confirmed.
- Four-pool vaults have been confirmed.
- Vault authority has been confirmed.

## 7. Launch Execution Rules

- Before every command, verbally repeat cluster, Program ID, authority, and target account.
- Any command that sends a transaction must first have dry-run output or explicit human confirmation.
- Any parameter update must go through governance / multisig / timelock.
- Emergency guardian must not be used to bypass governance.
- Do not use Devnet `1 USDC / 30s` parameters on Mainnet.
- Do not use Devnet-only scripts on Mainnet.

## 8. Failure Handling

If sanity check reports FAIL:

- Stop immediately.
- Save the full output.
- Do not execute later commands.
- Return to the Go/No-Go checklist.

If Mainnet parameters are wrong:

- Do not launch the frontend.
- Do not open user interaction.
- Correct through governance / multisig / timelock, then rerun sanity.

If frontend cluster display is wrong:

- Do not publish the frontend.
- Fix and rebuild.

If authority migration is incomplete:

- Mark NO-GO.
- Do not accept "migrate after launch" as the default plan.

If a transaction fails:

- Record the tx signature.
- Check whether partial state changed.
- Do not repeat execution unless idempotency is confirmed.

## 9. Emergency Pause / Stop Policy

Security Layer pause is emergency protection, not a governance replacement.

Emergency guardian can only be used to limit risk. It must not transfer funds, change parameters, or execute refund/slash actions.

Trigger conditions include:

- Abnormal parameters.
- Abnormal vault or mint.
- Frontend misdisplay.
- Exploit risk.
- Incorrect authority.

After pause, publish an announcement and enter post-incident review.

## 10. Frontend Release Checklist

- Cluster clearly displays Mainnet.
- Green Label Launch Guard displays correctly.
- Not insurance / not credit rating / not investment advice copy exists.
- Devnet test data is not presented as Mainnet production data.
- Explorer links point to Mainnet.
- Frontend build succeeds.
- Read-only pages do not expose erroneous write buttons.

## 11. Communication Checklist

Announcements must include:

- Current Alpha Protocol feature scope.
- Green Label meaning and limits.
- Green Label is not insurance.
- Green Label is not a credit rating.
- Green Label is not investment advice.
- Bond rule explanation.
- Refund/slash requires governance flow.
- Risk notices.
- Whether the current launch is beta.

## 12. Post-Launch Monitoring

Monitor after launch:

- Program account.
- `GreenLabelConfig`.
- Security governance config.
- Treasury vault balances.
- Staking pool / rewards vault.
- Frontend cluster.
- User feedback.
- Failed transactions and abnormal logs.

## 13. Rollback / Mitigation

Solana program state cannot be simply "rolled back". Mitigation should prioritize pause, frontend takedown, parameter correction, announcements, and governance remediation.

- Frontend can roll back to a previous version.
- Contract upgrades must go through authority / governance approval.
- Fund-related incidents must prioritize vault and user asset protection.

## 14. Final Sign-Off Template

Release date:

Release commit:

Mainnet Program ID:

Mainnet Green Label Config:

Mainnet Treasury State:

Mainnet Staking Pool:

Release Lead:

Contract Operator:

Frontend Operator:

Security Reviewer:

Treasury Reviewer:

Communications Lead:

Go/No-Go decision:

Open blockers:

Manual review completed:

Final notes:
