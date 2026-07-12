# ALPHA Fair Launch Decision Checklist

## 1. Purpose

This document records the final items that must be confirmed by the project owner before any ALPHA Fair Launch.

It replaces the old token allocation / vesting / team allocation checklist.

It does not approve immediate token launch.

It does not constitute investment advice, a return promise, a price promise, a dividend promise, an insurance promise, or any guarantee of token performance.

## 2. Confirmed Design Principles

- Total supply confirmed: `1,000,000,000 ALPHA`.
- Launch model confirmed: Fair Launch.
- Project-side allocation confirmed: `0`.
- Team preallocation confirmed: `0`.
- VC allocation confirmed: `0`.
- Initial token bucket allocation confirmed: none.
- DAO Treasury token allocation at launch confirmed: none.
- Staking token reserve at launch confirmed: none.
- Builders token allocation at launch confirmed: none.
- Market access: open market after launch.
- Revenue split confirmed: `50 / 20 / 20 / 10` applies only to future protocol revenue entering Treasury.

Fair Launch means ALPHA does not launch with project-side token buckets. It does not mean risk-free trading, guaranteed liquidity, guaranteed yield, price protection, or guaranteed token appreciation.

Confirmed decision record: `docs/fair-launch-confirmed-decisions.md`.

## 3. Superseded Old Decisions

The following old decision items are no longer required under the Fair Launch model:

- Team allocation.
- Team vesting.
- 12-month cliff.
- 36-month vesting.
- Builders token allocation.
- Builders token vesting.
- Staking token reserve.
- DAO treasury token allocation.
- Community token allocation bucket.
- Airdrop bucket.
- Option A / B / C token allocation models.
- `35 / 20 / 15 / 15 / 10 / 5` allocation model.

These items are superseded and must not be treated as active ALPHA launch requirements.

## 4. Decision Area 1: Total Supply

- Confirmed supply: `1,000,000,000 ALPHA`.
- Status: CONFIRMED.

Project owner checklist:

- [x] Confirm total supply as `1,000,000,000 ALPHA`.
- [ ] Confirm no future arbitrary minting after launch.
- [ ] Confirm supply figure is consistently used in docs and communication.

Notes:

- Total supply does not mean project allocation.
- Fair Launch means market distribution after launch, not project-side token buckets.

## 5. Decision Area 2: Launch Platform

- Status: pending final decision.
- Launch platform must be selected before token launch.
- The platform choice affects liquidity setup, launch mechanics, discoverability, bot risk, user experience, and communication.

Checklist:

- [ ] Compare available Solana fair-launch / launch options.
- [ ] Confirm launch platform.
- [ ] Confirm launch mechanics.
- [ ] Confirm whether launch uses bonding curve, AMM pool, or another mechanism.
- [ ] Confirm official launch link policy.
- [ ] Confirm anti-scam warning wording.

No specific launch platform is recommended in this document. The launch platform remains TBD / pending.

## 6. Decision Area 3: Initial Liquidity Setup

- Status: pending final decision.
- Initial liquidity setup must be reviewed before launch.
- Liquidity setup is not price protection.
- Liquidity setup is not guaranteed buyback.

Checklist:

- [ ] Confirm initial liquidity mechanism.
- [ ] Confirm source of initial liquidity.
- [ ] Confirm whether liquidity is created automatically by launch platform or manually.
- [ ] Confirm whether any LP position exists.
- [ ] Confirm LP lock / burn / custody policy if applicable.
- [ ] Confirm public disclosure wording.

## 7. Decision Area 4: Pairing Asset

- Status: pending final decision.
- Possible pairing assets may include SOL or USDC depending on launch platform and liquidity route.

Checklist:

- [ ] Confirm pairing asset.
- [ ] Confirm why pairing asset was selected.
- [ ] Confirm risk wording.
- [ ] Confirm docs and public communication use the same pairing asset description.

Notes:

- SOL pair may fit Solana-native community trading better.
- USDC pair may communicate a more stable accounting reference.
- This is not a price stability guarantee.

## 8. Decision Area 5: Mint Authority Policy

- Status: pending final decision.
- Mint authority determines whether additional ALPHA can be minted after launch.

Checklist:

- [ ] Confirm whether mint authority will be closed after launch.
- [ ] If not closed immediately, confirm multisig / governance control.
- [ ] Confirm public disclosure of mint authority policy.
- [ ] Confirm token metadata / explorer verification plan.

Recommended language:

If total supply is intended to be fixed, mint authority should be closed or placed under clearly disclosed multisig / governance control before public launch.

## 9. Decision Area 6: Freeze Authority Policy

- Status: pending final decision.
- Freeze authority can affect holder trust and must be handled carefully.

Checklist:

- [ ] Confirm whether freeze authority will be closed.
- [ ] If retained, document exact purpose.
- [ ] If retained, require multisig / governance control.
- [ ] Publicly disclose the policy before launch.

Recommended language:

If there is no strong compliance or operational reason to retain freeze authority, closing it is generally simpler and clearer for a fair-launch token.

## 10. Decision Area 7: Liquidity Custody / LP Handling

- Status: pending final decision.
- Applicable only if the launch route creates LP positions or project-controlled liquidity.

Checklist:

- [ ] Confirm whether LP tokens exist.
- [ ] Confirm lock / burn / custody policy.
- [ ] Confirm lock duration if locked.
- [ ] Confirm multisig custody if retained.
- [ ] Confirm public disclosure.
- [ ] Confirm no price floor / no guaranteed liquidity support wording.

## 11. Decision Area 8: Official Communication

- Status: pending final decision.
- Communication must clearly distinguish Fair Launch from Treasury revenue split.

Checklist:

- [ ] Announce ALPHA as planned Fair Launch.
- [ ] State no team / VC / project-side reserved token allocation.
- [ ] State `50 / 20 / 20 / 10` applies to future protocol revenue, not token allocation.
- [ ] State Mainnet status accurately.
- [ ] State token launch status accurately.
- [ ] Include anti-scam warnings.
- [ ] Include not financial advice.
- [ ] Include no guaranteed yield / no fixed APY.
- [ ] Include buyback / burn is not price protection.
- [ ] Include Green Label is not insurance / not credit rating.

## 12. Decision Area 9: Community Preheating Timing

- Status: pending final decision.
- Community preheating may begin before token launch if wording remains accurate and cautious.

Checklist:

- [ ] Confirm whether to start quiet preview.
- [ ] Confirm whether to start broader community preheating.
- [ ] Confirm which pages / docs are safe to share.
- [ ] Confirm whether language / i18n is ready enough for public preview.
- [ ] Confirm feedback channels.
- [ ] Confirm no real-funds entry is exposed.

## 13. Decision Area 10: Token Launch Timing

- Status: NO-GO until checklist items are resolved.

Checklist:

- [x] Fair Launch model confirmed.
- [x] Total supply confirmed.
- [x] Project-side allocation confirmed as `0`.
- [x] Team preallocation confirmed as `0`.
- [x] VC allocation confirmed as `0`.
- [x] Initial token bucket allocation confirmed as none.
- [x] Treasury revenue split confirmed as `50 / 20 / 20 / 10` for future protocol revenue only.
- [ ] Launch platform confirmed.
- [ ] Initial liquidity setup confirmed.
- [ ] Pairing asset confirmed.
- [ ] Mint authority policy confirmed.
- [ ] Freeze authority policy confirmed.
- [ ] LP / custody policy confirmed if applicable.
- [ ] Official communication reviewed.
- [ ] Anti-scam warning ready.
- [ ] Risk / legal wording reviewed.
- [ ] Public MVP pages reviewed.
- [ ] Final Go/No-Go completed.

## 14. Current Recommendation

- Keep token launch as NO-GO.
- Continue Devnet-verified Public MVP preparation.
- Use quiet preview only until frontend copy and i18n are reviewed.
- Do not announce a launch date yet.
- Do not publish a buy link yet.
- Do not imply price support, guaranteed buyback, guaranteed yield, or insurance coverage.

## 15. Final Status

- Fair Launch core decisions: CONFIRMED.
- Fair Launch decision checklist: PARTIALLY COMPLETE; operational launch decisions remain pending.
- Token launch: NO-GO.
- Mainnet production: NO-GO.
- Public MVP quiet preview: possible after frontend review.
- Community preheating: GO with warnings after communication review.
