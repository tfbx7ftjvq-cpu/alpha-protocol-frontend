# Green Label V1 Devnet Scripts

These scripts are Devnet-only helpers for Green Label V1 end-to-end validation.

They can send on-chain transactions and move Devnet USDC. Do not use them on Mainnet.
They do not deploy or upgrade the program, do not run `anchor keys sync`, and do not bypass
Security Layer timelocks or Green Label dispute response windows.

## Prerequisites

1. Deploy or upgrade the program separately before running these scripts.
2. Confirm Treasury V2 is initialized.
3. Confirm System Security Layer V1 governance config is initialized.
4. Confirm the wallet configured by Anchor has enough Devnet SOL and Devnet USDC.
5. Confirm the active cluster is Devnet.

Default constants:

- Program ID: `HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY`
- Devnet USDC mint: `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU`
- Green Label queue target account: the dispute PDA, matching `link_green_label_security_decision`

## Setup

```bash
npm run devnet:green-label:setup
```

If `GreenLabelConfigV1` already exists, the setup script only prints and validates it.
If it does not exist, the script initializes it after confirming Treasury V2 and Security Layer
accounts already exist.

Optional setup env:

- `USDC_MINT`, defaults to Devnet USDC.
- `BASE_BOND_TREASURY_VAULT`, defaults to Treasury V2 `builders_usdc_vault`.
- `RELIEF_OR_RISK_VAULT`, defaults to Treasury V2 `relief_usdc_vault`.

## Refund E2E

```bash
npm run devnet:green-label:refund
```

Path:

1. Submit Green Label application.
2. Initialize Green Bond Vault.
3. Lock Project Bond / Risk Bond.
4. Open dispute.
5. Wait for dispute response window if it is short enough.
6. Mark dispute ready for Security Layer decision.
7. Create Security Layer `GreenLabelRefund` decision.
8. Queue `GreenLabelRefund`.
9. Wait for timelock.
10. Link Green Label security decision.
11. Execute Green Label refund.
12. Print final project, dispute, and vault balance deltas.

## Slash E2E

```bash
npm run devnet:green-label:slash
```

Path:

1. Submit Green Label application.
2. Initialize Green Bond Vault.
3. Lock Project Bond / Risk Bond.
4. Open dispute.
5. Wait for dispute response window if it is short enough.
6. Mark dispute ready for Security Layer decision.
7. Create Security Layer `GreenLabelSlash` decision.
8. Queue `GreenLabelSlash`.
9. Wait for timelock.
10. Link Green Label security decision.
11. Execute Green Label slash.
12. Print final project, dispute, and vault balance deltas.

## Inspect

```bash
PROJECT_ID=1 npm run devnet:green-label:inspect
```

Optional:

```bash
PROJECT_ID=1 DISPUTE_ID=1 npm run devnet:green-label:inspect
```

## Common Env

- `BOND_AMOUNT_USDC`, default `299`.
- `PROJECT_NAME_HASH`, defaults to `sha256("alpha-green-label-devnet-project")`.
- `PROJECT_URL_HASH`, defaults to `sha256("https://example.dev/alpha-green-label")`.
- `TOKEN_MINT`, defaults to the system program id placeholder.
- `PROJECT_TREASURY_WALLET`, defaults to the provider wallet.
- `DISPUTE_ID`, default `1`.
- `DISPUTE_EVIDENCE_HASH`, defaults to `sha256("green-label-devnet-evidence")`.
- `REASON_CODE`, default `LiquidityRemoved`.
- `PROPOSAL_ID`, optional; by default scripts use `governance_config.proposal_count + 1`.
- `DELAY_SECONDS`, default is the Security Layer `min_execution_delay_seconds` or `60`.
- `MAX_RESPONSE_WAIT_SECONDS`, default `120`.
- `DRY_RUN`, default `false`.

If the configured dispute + response window is longer than `MAX_RESPONSE_WAIT_SECONDS`, the E2E
scripts stop before sending transactions. They will not skip contract time checks.
