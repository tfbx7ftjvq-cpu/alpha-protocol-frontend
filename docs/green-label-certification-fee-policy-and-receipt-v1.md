# Green Label Certification Fee Policy and Receipt V1

Date: 2026-07-15

## Purpose

Phase 2E-FINAL Stage 5B-4B-1 closes the unsafe legacy Green Label certification fee route. Stage 5B-4B-2 closes the remaining receipt bypasses for bond lock / `PendingObservation` and approve certification.

The previous public route accepted a caller-supplied `amount` and did not bind payment to a Green Label project or immutable receipt. The Mainnet-intended path is now a strict one-time route:

```text
GreenLabelConfigV1
-> GreenLabelCertificationFeePolicyV1
-> GreenLabelProjectV1
-> route_green_label_certification_fee_once_v1
-> Treasury Router 50 / 20 / 20 / 10
-> GreenLabelCertificationFeeReceiptV1
```

## Fee Policy

`GreenLabelCertificationFeePolicyV1` stores the authoritative non-refundable certification fee for a Green Label config.

Fields:

- `green_label_config`
- `usdc_mint`
- `fee_amount_usdc`
- `policy_version`
- `active`
- `initialized_by`
- `created_at`
- `schema_version`
- `bump`

PDA:

```text
[
  b"green_label_certification_fee_policy_v1",
  green_label_config.key().as_ref()
]
```

Size:

- `INIT_SPACE = 124`
- account space with Anchor discriminator = `132`

V1 policy rules:

- One fee policy per `GreenLabelConfigV1`.
- `policy_version` is fixed to `1`.
- `schema_version` is fixed to `1`.
- `active` is fixed to `true` on initialization.
- `usdc_mint` is derived from `GreenLabelConfigV1.usdc_mint`.
- `fee_amount_usdc` is the only business value supplied at bootstrap and must be greater than zero.
- The caller cannot choose mint, active status, schema version, policy version, or config mirror fields.
- There is no update, deactivate, or close instruction in this phase.
- `GreenLabelConfigV1` layout is unchanged.

The policy initializer is `initialize_green_label_certification_fee_policy_v1(fee_amount_usdc)`.

Authority:

- `authority` signer must equal `GreenLabelConfigV1.authority`.
- `payer` only pays rent and does not control business fields.

## Fee Parameters Hash

`GreenLabelCertificationFeeParametersV1` is the canonical audit payload for the fee route.

Domain separator:

```text
alpha_green_label_certification_fee_parameters_v1
```

The hash includes:

- schema version
- Green Label config
- fee policy
- policy version
- Green Label project
- project id
- project owner
- payer
- payer token account
- exact fee amount
- USDC mint
- Treasury config
- Treasury USDC state
- RevenueRoutingStats
- relief vault
- buyback vault
- builders vault
- staking vault
- revenue type

The hash uses deterministic Anchor/Borsh serialization with fixed field order. Dynamic vault balances are not included. Changing any key payment, project, policy, mint, Treasury, vault, amount, or revenue type field changes the hash.

## Fee Receipt

`GreenLabelCertificationFeeReceiptV1` is the immutable payment fact for one project.

Fields:

- `green_label_config`
- `fee_policy`
- `policy_version`
- `green_label_project`
- `project_id`
- `project_owner`
- `payer`
- `payer_token_account`
- `amount_usdc`
- `usdc_mint`
- `treasury_config`
- `treasury_usdc_state`
- `revenue_routing_stats`
- `relief_usdc_vault`
- `buyback_usdc_vault`
- `builders_usdc_vault`
- `staking_usdc_vault`
- `revenue_type`
- `parameters_hash`
- `routed_at`
- `schema_version`
- `bump`

PDA:

```text
[
  b"green_label_certification_fee_receipt_v1",
  green_label_project.key().as_ref()
]
```

Size:

- `INIT_SPACE = 516`
- account space with Anchor discriminator = `524`

Receipt rules:

- One receipt per Green Label project.
- Receipt creation, four-vault routing, Treasury accounting, and typed revenue stats update occur in the same instruction.
- If any transfer or accounting step fails, the receipt creation rolls back.
- The payer cannot provide receipt contents.
- Reapplication requires a new Green Label project and therefore a new receipt PDA.
- The receipt is a Green Label fee receipt, not a Treasury spending receipt.
- No `TreasuryExecutionRecordV1` is created by this route.

## Strict Once Route

Instruction:

```text
route_green_label_certification_fee_once_v1()
```

This instruction does not accept an amount argument. The exact routed amount is:

```text
GreenLabelCertificationFeePolicyV1.fee_amount_usdc
```

Required checks include:

- Green Label config is not paused.
- Project PDA and project id are correct.
- Project owner equals payer.
- Project status is `PendingBondDeposit`.
- Pending observation, active, disputed, refunded, slashed, cancelled, or other terminal states cannot pay this V1 fee route.
- Fee policy is bound to the config.
- Fee policy is active.
- Fee policy schema and policy version are both `1`.
- Fee policy mint equals config mint.
- Payer token account owner equals payer.
- Payer token account mint equals USDC mint.
- Payer token account balance is at least the exact fee.
- Payer token account is not any Treasury vault.
- USDC mint matches config and Treasury config.
- Mint decimals equal the Green Label USDC decimals constant.
- Treasury state matches the Green Label config.
- Revenue routing stats match Treasury authority and USDC mint.
- Four Treasury vaults have USDC mint and are owned by `vault_authority_v2`.
- Revenue type is fixed to `RevenueType::GreenLabelCertificationFee`.
- Parameters hash is recomputed from actual accounts and policy.

The route reuses the existing `route_usdc_revenue_from_token_account` helper. It does not duplicate the 50 / 20 / 20 / 10 split logic.

## Accounting

The fee is protocol revenue and routes through Treasury V2:

- 50% Relief
- 20% Buyback / Burn
- 20% Builders / Contributors
- 10% Staking Rewards, using remainder

The helper updates:

- `TreasuryUsdcStateV2.total_usdc_inflow`
- `TreasuryUsdcStateV2.relief_usdc_total`
- `TreasuryUsdcStateV2.buyback_usdc_total`
- `TreasuryUsdcStateV2.builders_usdc_total`
- `TreasuryUsdcStateV2.staking_usdc_total`
- `RevenueRoutingStatsV1.total_routed_usdc`
- `RevenueRoutingStatsV1.green_label_certification_fee_total`

`TreasuryUsdcStateV2` and `RevenueRoutingStatsV1` are passed as mutable Anchor accounts, so Anchor persists them when the instruction succeeds.

## Legacy Route Shutdown

The old public instruction remains for ABI and Devnet history compatibility:

```text
route_green_label_certification_fee_v1(ctx, amount)
```

It now fails closed with:

```text
LegacyGreenLabelCertificationFeeRouteDisabled
```

The legacy handler:

- ignores the caller-supplied amount
- does not transfer tokens
- does not update Treasury totals
- does not update typed revenue stats
- does not create a receipt
- does not modify the project

The Mainnet-intended fee route is `route_green_label_certification_fee_once_v1`.

## Economic Isolation

Certification fee and refundable bond are separate.

- The fee is independent non-refundable protocol revenue.
- The refundable bond is escrow liability.
- The fee does not enter `GreenLabelRefundableEscrowV1.deposited_amount`.
- The fee does not enter `GreenLabelRefundableEscrowV1.refundable_amount`.
- `Reject`, `Revoke`, and `RefundBond` do not refund the fee.
- `ForfeitBond` does not route the fee again.
- A forfeited bond becomes protocol revenue only after strict slash / forfeit governance and routes as `RevenueType::GreenLabelForfeitedBond`.

## Receipt Gates

Stage 5B-4B-2 adds a shared receipt validator and gates the downstream certification lifecycle:

- `lock_green_label_bond_with_fee_receipt_v1` requires `GreenLabelCertificationFeeReceiptV1` before moving a project into `PendingObservation`.
- the legacy `lock_green_label_bond` entry point now fails closed with `LegacyGreenLabelBondLockWithoutFeeReceiptDisabled`.
- `execute_green_label_approve_certification_v1` requires the same receipt before approving certification.
- the validator rebuilds the canonical fee parameters hash and rejects receipt/project/policy/amount/Treasury/revenue-type mismatches.
- old Devnet projects are not granted forged receipts.

This phase still does not add:

- fee policy update
- third-party sponsor payer
- fee refund
- new refund or forfeit functionality
- Victim Relief
- Scam Registry
- DAO Control Mode
- authority migration
- frontend changes
- deployment or chain transactions

Current boundaries:

- legacy no-receipt bond lock is disabled
- approve certification requires a fee receipt
- reject and revoke certification do not require a receipt and do not refund fees
- Mainnet production remains NO-GO
- token launch remains NO-GO

## Verification Status

Local tests cover policy size, receipt size, PDA seeds, policy initialization, wrong authority, zero amount, deterministic parameters hash, strict route validation, payer/status/decimals mismatch, Treasury/vault mismatch, receipt immutability, receipt gate acceptance/rejection, economic isolation from refundable escrow, legacy fee route fail-closed behavior, and legacy bond lock fail-closed behavior.

Local tests are not Devnet verification and are not Mainnet verification.
