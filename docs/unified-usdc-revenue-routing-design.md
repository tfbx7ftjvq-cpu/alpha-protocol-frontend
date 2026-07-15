# Unified USDC Revenue Routing Design

Date: 2026-07-13

## Purpose

Phase 2E-2B adds a typed USDC revenue router on top of Treasury V2. Phase 2E-2C adds Green Label refundable escrow and Green Label-specific Treasury routing. Official protocol revenue should enter Treasury through an explicit routing instruction, then split automatically into the existing Treasury V2 vaults.

Direct transfers to ordinary wallets or vaults are not treated as protocol revenue routing. Revenue must be routed through the program instruction to be counted in Treasury accounting.

## Implemented In This Phase

Phase 2E-2B:

- `RevenueType` enum for typed USDC protocol revenue.
- `RevenueRoutingStatsV1` account for typed revenue totals.
- `initialize_revenue_routing_stats_v1` instruction.
- `route_usdc_revenue_v1` instruction.
- Shared 50 / 20 / 20 / 10 split logic with existing `deposit_usdc_revenue`.
- Shared SPL Token `transfer_checked` pattern with existing Treasury V2 USDC deposit.

Phase 2E-2C:

- `GreenLabelRefundableEscrowV1` sidecar account.
- `GreenLabelEscrowStatusV1` enum.
- `initialize_green_label_refundable_escrow_v1` instruction.
- `deposit_green_label_refundable_bond_v1` instruction.
- `route_green_label_certification_fee_v1` instruction.
- `refund_green_label_escrow_v1` instruction.
- `forfeit_green_label_escrow_to_treasury_v1` instruction, retained as a legacy ABI / Devnet history entry point and now fail-closed.
- Shared internal Treasury router helper that supports either signer payer or escrow PDA signer.

Phase 2E-FINAL Stage 5B-3:

- `GreenLabelForfeitExecutionRecordV1` immutable record.
- `GreenLabelForfeitParametersV1` governance parameter hash.
- Strict DAO-governed Green Label forfeit wrapper.
- Forfeited escrow routing through the typed Treasury router as `RevenueType::GreenLabelForfeitedBond`.
- Legacy public slash / forfeit entry points disabled: `execute_green_label_slash` returns `LegacyGreenLabelSlashDisabled`, and `forfeit_green_label_escrow_to_treasury_v1` returns `LegacyGreenLabelForfeitDisabled`.

Phase 2E-FINAL Stage 5B-4B-1:

- `GreenLabelCertificationFeePolicyV1` authoritative fee policy sidecar.
- `GreenLabelCertificationFeeReceiptV1` immutable one-receipt-per-project payment fact.
- `GreenLabelCertificationFeeParametersV1` canonical fee route hash.
- `initialize_green_label_certification_fee_policy_v1` bootstrap instruction.
- `route_green_label_certification_fee_once_v1` strict route that reads the exact fee amount from policy, routes it as `RevenueType::GreenLabelCertificationFee`, and writes the receipt atomically.
- Legacy caller-amount `route_green_label_certification_fee_v1(ctx, amount)` disabled with `LegacyGreenLabelCertificationFeeRouteDisabled`.

## Revenue Types

- `GreenLabelCertificationFee`
- `GreenLabelForfeitedBond`
- `ProtocolServiceFee`
- `PlatformRevenue`
- `PartnershipRevenue`
- `ManualGovernanceApprovedRevenue`

The enum is USDC-only. It does not include SOL revenue and does not include refundable Green Label bond escrow.

The current Mainnet-intended strict forfeit path is `execute_green_label_forfeit_governance_v1`. Historical Devnet transactions and accounts remain readable, but they are not evidence that the legacy slash / non-strict forfeit instructions are currently executable.

## Stats PDA

`RevenueRoutingStatsV1` uses:

```text
seeds = [b"revenue_routing_stats_v1", treasury_config_v2.key().as_ref()]
```

The account stores:

- authority
- usdc_mint
- total_routed_usdc
- green_label_certification_fee_total
- green_label_forfeited_bond_total
- protocol_service_fee_total
- platform_revenue_total
- partnership_revenue_total
- manual_governance_approved_revenue_total
- bump

## Split Policy

`route_usdc_revenue_v1` routes USDC into Treasury V2 using the existing protocol split:

- 50% Relief Pool
- 20% Buyback / Burn vault
- 20% Builders / Contributors vault
- 10% Staking Rewards vault

Staking receives the remainder after the first three integer splits, so `relief + buyback + builders + staking == amount` for any positive amount.

## Green Label Refundable Escrow

`GreenLabelRefundableEscrowV1` uses:

```text
seeds = [b"green_label_refundable_escrow_v1", green_label_project.key().as_ref()]
```

The refundable USDC vault uses:

```text
seeds = [b"green_label_refundable_vault_v1", green_label_refundable_escrow.key().as_ref()]
```

The refundable vault authority is the escrow PDA. The escrow records the original `payer`, and normal refund must return funds only to a token account owned by that original payer.

Refundable escrow funds are not Treasury revenue while they remain refundable. They do not update `TreasuryUsdcStateV2` or `RevenueRoutingStatsV1` on deposit or normal refund.

## Current Boundaries

- `deposit_usdc_revenue` remains available as the legacy/simple USDC revenue entry.
- SOL revenue split is not implemented.
- Green Label certification fee routing is implemented as `RevenueType::GreenLabelCertificationFee` through the strict once route documented in `docs/green-label-certification-fee-policy-and-receipt-v1.md`.
- The legacy caller-amount Green Label certification fee route is disabled and no longer transfers tokens or updates accounting.
- Green Label refundable escrow is implemented as a sidecar account and independent refundable vault.
- Green Label forfeited escrow routing is implemented as `RevenueType::GreenLabelForfeitedBond`.
- Strict Green Label forfeit now requires governance sidecar, module registry, adapter, approved Security decision, executed queue item, and immutable forfeit execution record.
- Existing Green Label legacy slash / forfeit public entry points are disabled in Stage 5B-3. Legacy refund compatibility remains separate from strict governance refund.
- Stage 5B-4B-2 adds receipt gates for bond lock / `PendingObservation` and approve certification. The strict bond lock path is `lock_green_label_bond_with_fee_receipt_v1`; the legacy no-receipt bond lock path now fails closed.
- Builders payout governance is not implemented in this phase.
- DAO voting is not implemented in this phase.
- Token launch remains NO-GO.

## Green Label Certification Fee Route

The strict certification fee route is separate from refundable escrow. The fee is non-refundable protocol revenue, while refundable bond funds remain escrow liability until refund or forfeit.

`route_green_label_certification_fee_once_v1` does not accept a caller-supplied amount. It reads `GreenLabelCertificationFeePolicyV1.fee_amount_usdc`, verifies the project is still `PendingBondDeposit`, verifies the payer is the project owner, verifies Treasury state/stats/vaults, routes the fee through the shared Treasury router, and writes `GreenLabelCertificationFeeReceiptV1` in the same instruction.

One project can have only one receipt PDA:

```text
[
  b"green_label_certification_fee_receipt_v1",
  green_label_project.key().as_ref()
]
```

## Green Label Refundable Escrow Rules

Green Label refundable escrow is explicitly separate from Treasury revenue routing. Refundable escrow funds are not protocol revenue while they are refundable, and they must not pass through the Treasury 50 / 20 / 20 / 10 split unless a valid slash / forfeit decision converts them into protocol revenue.

### Refund Conditions

Refund may be allowed only when one of the following is true:

- No active valid dispute exists and the current timestamp is greater than or equal to `refund_available_after`.
- A linked Security Layer / Green Label decision is `Refund`.

Refund requirements:

- Refund must go only to the original payer.
- Refundable escrow must not pass through Treasury split.
- Refund must not update `RevenueRoutingStatsV1`.

### Forfeit Conditions

Forfeit may be allowed only when all of the following are true:

- A valid dispute exists.
- The dispute is ready for decision.
- A linked Security Layer / Green Label decision is `Slash` / `Forfeit`.
- The escrow has not already been refunded or forfeited.

Forfeit requirements:

- Forfeited funds must route as `RevenueType::GreenLabelForfeitedBond`.
- Forfeited funds must enter the Treasury router and split 50 / 20 / 20 / 10.
- No one may forfeit escrow funds by time alone.
- The strict Stage 5B-3 path requires `GovernanceActionTypeV1::GreenLabelSlashBond`, the Green Label module registry, `UniversalGovernanceDecisionAdapterV1`, an approved `ProposalDecisionV1`, an executed `ExecutionQueueItemV1`, and the exact `GreenLabelForfeitParametersV1` hash.
- The executor cannot choose the amount. The routed amount is `refundable_amount - refunded_amount - forfeited_amount` from escrow state.
- `refundable_vault.amount` is checked only for sufficiency. Extra dust does not increase routed revenue and remains in the vault.

Legacy public slash / forfeit entry points are disabled by Stage 5B-3 and return explicit legacy-disabled errors.

## Mainnet Notes

Before Mainnet, revenue operators and governance must define which external revenue sources are authorized to call the router and how typed revenue evidence is recorded off-chain or on-chain. Mainnet communications must not imply that direct wallet transfers are automatically split or counted as official protocol revenue.
