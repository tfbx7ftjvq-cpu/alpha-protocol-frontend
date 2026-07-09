# Tokenomics Allocation Review

Date: 2026-07-09

## 1. Purpose

This document reviews the current ALPHA tokenomics allocation draft.

The goal is to determine whether the current allocation can move toward tokenomics freeze.

This document does not represent immediate token launch approval. It is not investment advice, a return promise, a price promise, or a dividend promise.

## 2. Current Draft Summary

- Token: ALPHA
- Network: Solana
- Proposed total supply: `1,000,000,000 ALPHA`
- Current launch status: token launch pending
- Mainnet status: not live
- Product status: Devnet-verified read-only Public MVP

Current allocation draft:

| Category | Percentage | Amount |
| --- | ---: | ---: |
| DAO Treasury / Protocol Reserve | 35% | 350,000,000 ALPHA |
| Community / Ecosystem Incentives | 20% | 200,000,000 ALPHA |
| Liquidity / Launch Pool | 15% | 150,000,000 ALPHA |
| Staking Rewards Reserve | 15% | 150,000,000 ALPHA |
| Builders / Contributors | 10% | 100,000,000 ALPHA |
| Team / Founding Contributors | 5% | 50,000,000 ALPHA |
| Total | 100% | 1,000,000,000 ALPHA |

## 3. High-Level Assessment

- `1B` total supply is acceptable for a public Solana ecosystem token.
- Team allocation at `5%` is conservative and community-friendly.
- DAO Treasury `35%` is reasonable for long-term protocol control.
- Community `20%` is reasonable but should include any airdrop allocation if used.
- Liquidity `15%` is usable but requires a clear LP lock / burn / custody policy.
- Staking `15%` is acceptable only if clearly framed as protocol-rule-based incentives, not fixed yield.
- Builders `10%` is reasonable but should have vesting or milestone-based release.

Overall recommendation:

- The allocation is suitable as a recommended freeze candidate.
- It is not final until LP policy, vesting, launch platform, and authority controls are confirmed.

## 4. Category-by-Category Review

### A. DAO Treasury / Protocol Reserve - 35%

Intended use:

- Long-term protocol reserve.
- Governance-controlled treasury.
- Future grants / audits / protocol operations.
- Risk response capacity.

Risks:

- Without governance or multisig controls, this may be viewed as centralized.
- If too much is released early, it may weaken market trust.

Recommendations:

- No immediate market sale by default.
- Governed by multisig / DAO / timelock.
- Publish treasury wallet rules before launch.

Judgment:

- Recommended: keep `35%`.

### B. Community / Ecosystem Incentives - 20%

Intended use:

- Community rewards.
- Ecosystem campaigns.
- Contribution incentives.
- Possible future airdrop.
- Risk research participation.

Risks:

- Poor airdrop design may attract sybil farming.
- Early release may create sell pressure.
- It must not be marketed as a guaranteed reward.

Recommendations:

- Any airdrop should come from this `20%`.
- Avoid a separate extra airdrop allocation unless total allocation is revised.
- Release gradually through campaigns / contribution programs.

Judgment:

- Recommended: keep `20%`.

### C. Liquidity / Launch Pool - 15%

Intended use:

- Initial liquidity.
- Launch operations.
- DEX liquidity support.

Risks:

- Unlocked LP can severely damage trust.
- Burning all LP may reduce future liquidity management flexibility.
- Single-wallet LP control is high risk.

Recommendations:

- Define LP lock / burn policy before launch.
- Define who controls LP tokens.
- Publish liquidity wallet address before launch if appropriate.
- Avoid claiming guaranteed buyback or price support.

Judgment:

- Recommended: keep `15%`, but mark LP policy as a BLOCKER before launch.

### D. Staking Rewards Reserve - 15%

Intended use:

- Staking rewards reserve.
- Protocol-rule-based incentives.
- Future governance-approved reward programs.

Risks:

- Easily misunderstood as fixed yield.
- APY promise language would create high messaging and regulatory risk.
- Long-term release rules are not finalized.

Recommendations:

- No fixed APY.
- No guaranteed returns.
- Rewards depend on pool balance and protocol rules.
- Release schedule should be governance-controlled.

Judgment:

- Recommended: keep `15%`, but require reward policy before launch.

### E. Builders / Contributors - 10%

Intended use:

- Contributors.
- Audits.
- Maintenance.
- Ecosystem operations.
- Future development.

Risks:

- Immediate release may be viewed as indirect team allocation.
- Must be distinct from Team / Founding Contributors.

Recommendations:

- Milestone-based release or vesting.
- Transparent contributor policy.
- DAO / multisig approval for large allocations.

Judgment:

- Recommended: keep `10%`.

### F. Team / Founding Contributors - 5%

Intended use:

- Founding contributor alignment.
- Long-term commitment.

Risks:

- Even at `5%`, lockup details are required.
- Without vesting, it may weaken trust.

Recommendations:

- Keep `5%` because it is conservative.
- Require strong lockup.
- Recommended: 12 month cliff / 36 month vesting.
- No immediate unlock at launch.

Judgment:

- Recommended: keep `5%`, with strong vesting.

## 5. Recommended Freeze Candidate

Recommended freeze candidate:

- Total supply: `1,000,000,000 ALPHA`
- DAO Treasury / Protocol Reserve: `35%`
- Community / Ecosystem Incentives: `20%`
- Liquidity / Launch Pool: `15%`
- Staking Rewards Reserve: `15%`
- Builders / Contributors: `10%`
- Team / Founding Contributors: `5%`

Freeze condition:

This allocation can be used as a freeze candidate, but the following must be confirmed first:

- LP lock / burn / custody policy.
- Team vesting.
- Builders vesting / release policy.
- Staking rewards release policy.
- DAO treasury wallet / authority policy.
- Launch platform.
- Final risk wording.
- Legal / regulatory review.

## 6. Alternative Allocation Options

### Option A: Current Conservative Team Model

- DAO Treasury / Protocol Reserve: `35%`
- Community / Ecosystem Incentives: `20%`
- Liquidity / Launch Pool: `15%`
- Staking Rewards Reserve: `15%`
- Builders / Contributors: `10%`
- Team / Founding Contributors: `5%`

Notes:

- Recommended option.
- Low team allocation.
- Community-friendly.
- Strong DAO treasury.

### Option B: No Explicit Team Allocation

- DAO Treasury / Protocol Reserve: `37%`
- Community / Ecosystem Incentives: `20%`
- Liquidity / Launch Pool: `15%`
- Staking Rewards Reserve: `15%`
- Builders / Contributors: `13%`
- Team / Founding Contributors: `0%`

Notes:

- More anti-team-allocation narrative.
- May weaken founding contributor incentives.
- Team incentives may become hidden inside builders, reducing transparency.

### Option C: Higher Community Model

- DAO Treasury / Protocol Reserve: `30%`
- Community / Ecosystem Incentives: `25%`
- Liquidity / Launch Pool: `15%`
- Staking Rewards Reserve: `15%`
- Builders / Contributors: `10%`
- Team / Founding Contributors: `5%`

Notes:

- Better for a strong community launch narrative.
- Smaller DAO treasury.
- Requires stronger community operations and anti-sybil design.

Conclusion:

- Recommend Option A unless the user explicitly wants stronger community narrative or no explicit team allocation.

## 7. Airdrop Review

- A separate airdrop bucket is not recommended.
- If an airdrop is used, it should come from the Community / Ecosystem `20%`, for example `3%-7%`.
- Airdrop allocation should not be fully released in one event.
- Airdrops should be tied to contribution, testing, risk research, community tasks, or early support.
- Avoid a "free money" narrative.
- Sybil / bot farming prevention is required.

Recommendation:

- Airdrop is optional, not required for launch.
- If used, start with a small retro/community allocation rather than a large no-barrier airdrop.

## 8. LP Lock / Burn Review

Must decide before launch:

- Are LP tokens locked?
- Are LP tokens burned?
- Are LP tokens controlled by multisig?
- What is the lock duration?
- What is the liquidity amount?
- What is the initial liquidity pairing asset?
- Who controls treasury-side liquidity?

Recommendations:

- For an early-stage project, LP lock is stronger than leaving all LP unlocked.
- LP burn is a stronger signal but reduces future liquidity adjustment flexibility.
- Draft recommendation:
  - Lock a meaningful portion of LP.
  - Disclose lock duration.
  - Keep any operational liquidity under multisig / governance policy.
  - Do not claim guaranteed price support.

## 9. Authority / Wallet Review

Must confirm before launch:

- Mint authority.
- Freeze authority.
- Deployer wallet.
- DAO treasury wallet.
- Liquidity wallet.
- Community rewards wallet.
- Staking rewards wallet.
- Builders wallet.
- Team vesting wallet.

Requirements:

- Long-term single hot wallet control is high risk.
- Mint authority should be closed, locked, or governance / multisig controlled.
- If freeze authority is retained, the reason and governance constraints must be explained.
- Treasury wallets should be multisig or governance controlled.

## 10. Launch Readiness Impact

Current conclusions:

- Tokenomics draft: improved.
- Allocation review: completed as draft.
- Tokenomics freeze: pending user decision.
- Immediate token launch: still NO-GO.
- Community preheating: still GO with warnings.

Formal token launch still requires:

- User confirms freeze candidate.
- LP policy chosen.
- Vesting chosen.
- Airdrop decision made.
- Launch platform chosen.
- Authority / wallet plan reviewed.
- Communication reviewed.
- Legal / risk review.

## 11. Final Decision Checklist

The user must make the final call on:

- [ ] Total supply = `1,000,000,000 ALPHA`
- [ ] Use Option A `35/20/15/15/10/5`
- [ ] Keep team allocation at `5%`
- [ ] Team vesting: 12 month cliff / 36 month vesting
- [ ] Builders vesting / milestone release
- [ ] Airdrop yes/no
- [ ] If yes, airdrop percentage within Community `20%`
- [ ] LP lock / burn / multisig policy
- [ ] Launch platform
- [ ] Mint authority / freeze authority policy
- [ ] Treasury wallet policy
- [ ] Staking rewards release policy
- [ ] Final launch communication review
- [ ] Legal / risk wording review

## 12. Recommended Current Decision

Recommended:

- Keep `1B` total supply.
- Use Option A `35/20/15/15/10/5`.
- Keep team `5%`, but with strict vesting.
- Do not add a separate airdrop bucket.
- If airdrop is desired, allocate it from Community `20%`.
- Require LP lock / multisig policy before launch.
- Keep immediate token launch as NO-GO until these are finalized.
