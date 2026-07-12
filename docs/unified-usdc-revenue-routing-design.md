# Unified USDC Revenue Routing Design

Date: 2026-07-13

## Purpose

Phase 2E-2B adds a typed USDC revenue router on top of Treasury V2. Official protocol revenue should enter Treasury through an explicit routing instruction, then split automatically into the existing Treasury V2 vaults.

Direct transfers to ordinary wallets or vaults are not treated as protocol revenue routing. Revenue must be routed through the program instruction to be counted in Treasury accounting.

## Implemented In This Phase

- `RevenueType` enum for typed USDC protocol revenue.
- `RevenueRoutingStatsV1` account for typed revenue totals.
- `initialize_revenue_routing_stats_v1` instruction.
- `route_usdc_revenue_v1` instruction.
- Shared 50 / 20 / 20 / 10 split logic with existing `deposit_usdc_revenue`.
- Shared SPL Token `transfer_checked` pattern with existing Treasury V2 USDC deposit.

## Revenue Types

- `GreenLabelCertificationFee`
- `GreenLabelForfeitedBond`
- `ProtocolServiceFee`
- `PlatformRevenue`
- `PartnershipRevenue`
- `ManualGovernanceApprovedRevenue`

The enum is USDC-only. It does not include SOL revenue and does not include refundable Green Label bond escrow.

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

## Current Boundaries

- `deposit_usdc_revenue` remains available as the legacy/simple USDC revenue entry.
- SOL revenue split is not implemented.
- Green Label revenue integration is not implemented in this phase.
- Green Label refundable escrow is not implemented in this phase.
- Builders payout governance is not implemented in this phase.
- DAO voting is not implemented in this phase.
- Token launch remains NO-GO.

## Mainnet Notes

Before Mainnet, revenue operators and governance must define which external revenue sources are authorized to call the router and how typed revenue evidence is recorded off-chain or on-chain. Mainnet communications must not imply that direct wallet transfers are automatically split or counted as official protocol revenue.
