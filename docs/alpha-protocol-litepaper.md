# Alpha Protocol Litepaper

Date: 2026-07-09

## 1. Abstract

Alpha Protocol is a DAO-governed risk protection and accountability protocol for on-chain ecosystems. It combines a Security Layer execution guard, Green Label risk bonds, Treasury V2 revenue routing, Staking V1 incentives, and public read-only dashboards.

The current implementation is a Devnet-verified Public MVP. Mainnet is not live, the public token launch is pending, and the full ALPHA voting layer has not yet launched.

## 2. Problem

On-chain ecosystems face recurring trust and execution problems:

- On-chain fraud, rugs, and malicious project behavior.
- Lack of transparent project accountability.
- Weak treasury transparency and unclear protocol revenue use.
- DAO execution without enough safety controls.
- Lack of a structured dispute / refund / slash process.

Many users see project claims, token incentives, or DAO language without a clear way to inspect risk commitments, treasury flows, or final execution rules.

## 3. Solution

Alpha Protocol introduces a transparent protocol stack:

- DAO Security Layer for proposal decisions, queueing, timelock, cancel, pause, and guarded execution.
- Green Label risk bond flow for project accountability, observation windows, dispute windows, governance decisions, refund, and slash.
- Treasury V2 for transparent USDC revenue routing across relief, buyback / burn, builders, and staking rewards.
- Staking V1 for protocol-rule-based incentives.
- Public read-only dashboards for Devnet state visibility.

The protocol is designed to make critical state inspectable before Mainnet launch and to keep sensitive actions behind governance and execution guards.

## 4. Protocol Architecture

Alpha Protocol is organized into six product layers:

### DAO Governance / Security Layer

The Security Layer is the execution guard for sensitive protocol actions. It records proposal decisions, queues execution, enforces timelocks, supports cancel paths, and provides pause / unpause controls.

### Green Label

Green Label allows project teams to lock a risk bond and move through observation, dispute, governance decision, refund, or slash flows. It is an accountability signal, not insurance and not a credit rating.

### Treasury V2

Treasury V2 routes USDC protocol revenue into four transparent pools:

- 50% Relief Pool.
- 20% Buyback / Burn Pool.
- 20% Builders / Contributors Pool.
- 10% Staking Rewards Pool.

### Staking V1

Staking V1 supports stake / claim / unstake flows on Devnet. Staking rewards are protocol-rule-based incentives, not guaranteed yield.

### Token / Revenue

ALPHA is intended as the protocol coordination asset for governance participation, staking participation, contributor coordination, ecosystem alignment, and risk reporting / dispute governance participation.

### Public Dashboards

Public dashboards expose Devnet state in read-only mode. They do not create transactions, execute governance, buy tokens, or move funds.

## 5. DAO Governance

DAO governance is core to Alpha Protocol because refund, slash, treasury, emergency, and parameter actions affect user trust and protocol credibility.

The completed Devnet scope is the execution / security layer. The full ALPHA voting layer is pending.

Refund, slash, treasury, and emergency actions must not bypass the Security Layer because:

- Timelocks create review windows.
- Queue accounts make execution visible before completion.
- Cancel paths support stopping wrong or malicious actions.
- Payload hash and action type checks reduce wrong-execution risk.
- Pause controls support emergency containment.

Until the voting layer is complete, Alpha Protocol must not be marketed as a fully decentralized DAO.

## 6. Green Label

Green Label is a project accountability mechanism based on risk bonds.

Core concepts:

- Base bond: the minimum configured project bond.
- Extra bond: additional voluntary risk commitment.
- Observation window: the period after successful bond lock.
- Dispute window: the period in which disputes can be opened.
- Governance decision: Security Layer decision and queued execution.
- Refund: valid no-rug / no-malicious outcome can return bond according to protocol rules.
- Slash: confirmed malicious / rug behavior can slash bond according to protocol rules.

Green Label is not insurance, not a credit rating, and not investment advice.

## 7. Treasury Model

Treasury V2 uses a 50 / 20 / 20 / 10 split:

- 50% Relief Pool.
- 20% Buyback / Burn Pool.
- 20% Builders / Contributors Pool.
- 10% Staking Rewards Pool.

Future changes to treasury parameters, builders spending, relief policy, and staking reward policy should require governance review and Security Layer execution controls.

## 8. ALPHA Token Utility

ALPHA token utility may include:

- Governance participation.
- Staking participation.
- Protocol incentives.
- Contributor coordination.
- Ecosystem alignment.
- Risk reporting / dispute governance participation.

ALPHA must not be described as providing guaranteed profit, fixed yield, dividends, guaranteed payouts, or token price appreciation.

## 9. Staking

Staking V1 has been verified on Devnet for stake / claim / unstake paths.

Staking rewards are protocol-rule-based incentives. They depend on pool balance, protocol rules, and future governance.

There is no guaranteed APY, fixed yield, or guaranteed return.

## 10. Current Devnet Status

Current verified status:

- Treasury V2 verified.
- Staking V1 verified.
- Security Layer V1 verified.
- Green Label refund / slash verified.
- DAO dashboard completed.
- Token / Revenue dashboard completed.
- Mainnet safety docs and sanity checks completed.

This status is a Devnet baseline and does not approve Mainnet production launch.

## 11. Roadmap

Suggested roadmap:

1. Phase 2A: DAO product layer.
2. Phase 2B: Token / Revenue product layer.
3. Phase 2C: Public MVP / Litepaper.
4. Phase 2D: Launch readiness review.
5. Future ALPHA voting layer.
6. Future DAO-controlled treasury.
7. Future Mainnet launch.

## 12. Risk Disclosure

Alpha Protocol involves technical, governance, and regulatory risks.

Important disclosures:

- This document is not financial advice.
- Alpha Protocol is not insurance.
- Green Label is not a credit rating.
- ALPHA does not guarantee returns.
- Mainnet is not live.
- Token launch is pending.
- Smart contract risk remains.
- Governance risk remains.
- Regulatory risk remains.

Public materials must not imply guaranteed returns, insurance protection, fixed compensation, or price appreciation.
