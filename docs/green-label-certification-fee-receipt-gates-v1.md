# Green Label Certification Fee Receipt Gates V1

Date: 2026-07-15

## Purpose

Phase 2E-FINAL Stage 5B-4B-2 closes the remaining Green Label certification fee receipt bypasses.

The strict Mainnet-intended sequence is now:

```text
submit_green_label_application
-> initialize_green_bond_vault
-> route_green_label_certification_fee_once_v1
-> GreenLabelCertificationFeeReceiptV1
-> lock_green_label_bond_with_fee_receipt_v1
-> PendingObservation
-> execute_green_label_approve_certification_v1
```

The receipt is required before a project can enter `PendingObservation` through the new strict bond lock path and before certification can be approved.

## Shared Receipt Validator

The implementation adds one shared validator:

```text
validate_green_label_certification_fee_receipt_v1
```

It validates:

- receipt account exists and is not blank
- receipt PDA is tied to the Green Label project
- receipt project id, project owner, and payer match `GreenLabelProjectV1`
- receipt config matches `GreenLabelConfigV1`
- receipt policy matches `GreenLabelCertificationFeePolicyV1`
- policy schema/version are V1
- policy is active
- receipt amount equals the active policy amount
- receipt USDC mint matches the config and Treasury config
- receipt Treasury config and routed Treasury account fields are non-empty
- receipt revenue type is `RevenueType::GreenLabelCertificationFee`
- receipt routed timestamp is non-zero
- receipt bump matches the account bump
- canonical fee parameters hash is rebuilt and equals `receipt.parameters_hash`

`GreenLabelProjectV1` does not store a config field. The binding is enforced through the project PDA, project id/owner, receipt config, fee policy config, Treasury config, and canonical receipt hash.

## Strict Bond Lock Path

The new strict path is:

```text
lock_green_label_bond_with_fee_receipt_v1
```

It requires:

- `GreenLabelCertificationFeePolicyV1`
- `GreenLabelCertificationFeeReceiptV1`
- `TreasuryConfigV2`
- the existing bond lock accounts

The old `lock_green_label_bond` entry point is retained for ABI / Devnet history compatibility but now fails closed with:

```text
LegacyGreenLabelBondLockWithoutFeeReceiptDisabled
```

This prevents public no-receipt bond lock from moving a project into `PendingObservation`.

## Approve Certification Gate

`execute_green_label_approve_certification_v1` now requires the same policy, receipt, and Treasury config accounts and calls the shared validator before approving certification.

This means approval cannot rely only on bond vault state and governance approval; it must also prove the project paid the strict one-time certification fee receipt.

Reject and revoke certification do not require a receipt gate:

- Reject may apply to projects that never paid the certification fee.
- Revoke applies to an already-approved certification.
- Neither reject nor revoke refunds the certification fee.
- Neither reject nor revoke routes Treasury revenue.

## No Devnet Receipt Forging

This phase does not forge or migrate old Devnet receipts.

Existing Devnet projects without `GreenLabelCertificationFeeReceiptV1` cannot use the strict Mainnet-intended certification path unless they are re-created or migrated by a future explicit migration plan.

## Still Not Implemented

This stage does not implement:

- fee policy update V2
- sponsor payer
- certification fee refund
- new refund / forfeit paths
- Victim Relief
- Scam Registry
- DAO Control Mode
- authority migration
- frontend changes
- deployment or chain transactions

Mainnet production and token launch remain NO-GO.

