# Victim Relief Devnet Strict E2E V1

This document records the Phase 2E-6B-4C Devnet script and upgrade readiness layer for Victim Relief. It does not mark Mainnet readiness and does not authorize token launch.

## Scope

Phase 2E-6B-4C adds Devnet-only operational scripts for:

- Victim Relief inspect and setup.
- Original approved payout strict wrapper.
- Reject plus uphold appeal path.
- Reject plus overturn payout path.
- Original payout cancellation path.
- Overturn payout cancellation path.
- Victim Relief module pause path.
- Protocol Authority inspect, bootstrap, DAO control activation manifest, DAO control activation wrapper, recovery unpause wrapper, and legacy fail-close inspection.

## Devnet Guards

The scripts must run only against Devnet. They verify:

- RPC URL does not point to Mainnet.
- Runtime genesis hash equals the known Devnet genesis hash.
- Program ID equals `HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY`.
- Local IDL address matches the fixed Program ID.
- Transaction scripts refuse to proceed unless required instruction names exist in the generated IDL.
- Transaction scripts are dry-run or locked unless `CONFIRM_DEVNET_TX=true` is explicitly set.

The scripts never create a wallet, never overwrite a keypair, never run `anchor keys sync`, and never target Mainnet.

## Commands

Victim Relief:

```bash
npm run devnet:victim-relief:inspect
npm run devnet:victim-relief:setup
npm run devnet:victim-relief:original-payout-e2e
npm run devnet:victim-relief:reject-uphold-e2e
npm run devnet:victim-relief:overturn-payout-e2e
npm run devnet:victim-relief:cancel-original-e2e
npm run devnet:victim-relief:cancel-overturn-e2e
npm run devnet:victim-relief:pause-e2e
```

Protocol Authority:

```bash
npm run devnet:authority:inspect
npm run devnet:authority:init-bootstrap
npm run devnet:authority:activation-manifest
npm run devnet:authority:activate-dao-control
npm run devnet:authority:recovery-unpause-e2e
npm run devnet:authority:verify-legacy-fail-close
```

## Required Fresh IDL

The current generated IDL must be produced from the current source before any transaction script is used. If the IDL is stale, transaction scripts fail closed before building instructions.

## E2E Environment Inputs

The strict E2E runner requires explicit environment variables. It does not guess governance artifacts or recipient accounts.

Common inputs:

- `CASE_ID`
- `CLAIMANT`
- `PROPOSAL_ID`
- `RECIPIENT_USDC_TOKEN_ACCOUNT` for payout paths
- `ORIGINAL_PROPOSAL_ID` for appeal and cancellation paths that reference the original decision
- `OVERTURN_PROPOSAL_ID` for overturn cancellation
- `PAUSE_MODE=guardian`, `PAUSE_MODE=dao-pause`, or `PAUSE_MODE=dao-unpause` for pause E2E
- `EMERGENCY_GUARDIAN` for guardian pause

Real Devnet sends require:

```bash
DRY_RUN=false CONFIRM_DEVNET_TX=true
```

## Upgrade Stop Point

This phase only prepares for a Devnet program upgrade. It must stop before deployment until the user explicitly confirms:

```text
»∑»œ÷¥––Devnet program upgrade
```

## Status

- Devnet scripts: prepared.
- Devnet program upgrade: pending explicit confirmation.
- Mainnet production: NO-GO.
- Token launch: NO-GO.