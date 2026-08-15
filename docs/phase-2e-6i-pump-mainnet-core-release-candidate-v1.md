# Phase 2E-6I — Pump Mainnet Core Release Candidate

Date: 2026-08-15
Candidate commit: `5e38e20aec59e730f940999d6314665f78269c1e`

## Scope and non-actions

This phase records a release candidate and offline evidence tooling. It did **not** deploy or upgrade a
program, create an ALPHA mint, send a transaction, move SOL or USDC, change any authority, or modify an
existing chain account. Mainnet is not deployed or launched.

The canonical product charter is present at
`docs/alpha-protocol-canonical-product-charter-and-launch-checklist-v1.md` and is the sole authoritative
product scope for this phase. `README.md` contains its canonical-charter entry. This Phase 2E-6I record
conforms to the charter's Pump / USDC / `50 / 20 / 20 / 10` scope; older phase reports or inventories do
not override it.

## Confirmed launch and revenue boundary

- ALPHA supply policy is `1,000,000,000 ALPHA`; launch model is Fair Launch.
- Project, team, VC, DAO treasury, staking reserve, builders, and other initial ALPHA allocations are `0`.
- Pump is the confirmed launch platform. This is not a launch authorization and does not settle liquidity,
  pair, authority, custody, communication, or timing decisions.
- Treasury starts at `0 USDC`.
- The sole planned automatic revenue intake is Pump Creator Fee / Platform Revenue settled in canonical USDC.
- It is represented by the existing typed USDC router as `PlatformRevenue`, with Relief / Buyback / Builders /
  Staking split `50 / 20 / 20 / 10`.
- Creator Fee is platform revenue, not a Token-2022 transfer tax. The creator-fee recipient and the later
  Treasury-route operation remain separate human-controlled boundaries.
- If settlement is not USDC, automatic routing stops. A separately recorded and approved conversion remains
  outside this phase; this phase implements no automatic SOL-to-USDC conversion, transaction construction,
  signing, or submission.

## Program identity and binary integrity evidence

| Item | Observed / fixed evidence | Result |
| --- | --- | --- |
| Declared program ID | `HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY` in Rust, Anchor configuration, and frontend constants | Consistent |
| Current IDL top-level address | `HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY` | Matches declared ID |
| Current IDL metadata address | Missing | Fail-closed blocker |
| Current local program-keypair public key | `BQAiU9HDA3atHN7t6jnykBTHoAKp4R4aeHSDraR6v97A` | Does not match declared ID; fail-closed blocker |
| Existing local `.so` SHA-256 | `3b197267cec02096479b1c6eac50b2912cbafce6204d31ad66fb786d650588fe` | Uncertified pre-build artifact |
| Existing local IDL SHA-256 | `a67b5b531c69412f9c29985adeb978ce473e5a22c711a70f521b573973ddde93` | Top-level address only |
| Historical Devnet ProgramData | `4ckMZysHvxVr6v4KjuV7NqBvsykAbNKocRJCmvaYXpu6` | Historical fixed evidence only |
| Historical upgrade authority | `CqSs2yq6Jo3gYwXBq7fGRqohcxXS7HFJNYypykZTEGa8` | Historical fixed evidence only |
| Historical local build `.so` SHA-256 | `4f2da8d84964b446c0d3dea06339aa55ea61b29eed68aa37798fe91282541fe4` | Does not match current local artifact |
| Historical alternate dump / temporary binary SHA-256 | `b9d203d02ba5416c05fd2b43af3e2adbb229df3c7ed38cc9d9b2017fe706b20a` | Does not match current local artifact |

Binary-drift verdict: **UNRESOLVED_BLOCKER**. The mismatched local keypair and missing `metadata.address`
mean the local artifacts cannot prove correspondence to the declared program. Historical hashes are retained
as evidence, not overwritten or explained away by a redeployment. No Devnet RPC dump was requested in this
phase, so current deployed-byte correspondence is not asserted.

The release verifier is local and read-only. It requires explicit artifact paths and expected SHA-256 values,
checks program ID, keypair public key, both IDL address fields, and optional supplied dump hash. It never
prints keypair bytes and has no RPC, signing, authority, deployment, or transaction capability.

## Build environment record

Observed host: Windows `10.0.19045.7548`; Git `2.54.0.windows.1`; Node `v24.16.0`; npm `11.13.0`.
`rustc`, Cargo, Solana CLI, and Anchor CLI were not available on the Windows PATH; WSL status access was
denied by the host. The required `anchor build --ignore-keys` was attempted exactly once and could not start
because `anchor` was not found. Therefore Rust/Cargo/Solana/Anchor versions and a reproducible local Anchor
build are not yet evidenced from this host. This is a release-candidate blocker, not an authorization to
install tools, change the program, or redeploy it.

## Offline Pump Creator Fee manifest

`server/scripts/revenue/pump-creator-fee-usdc.ts` supports only `inspect` and `prepare`. Its deterministic
manifest includes schema version, `pump_creator_fee` source, token mint, public revenue wallet, canonical
USDC mint and six decimals, base-unit amount, `PlatformRevenue`, four destination vaults, exact split,
source-evidence reference, and SHA-256 batch hash. It rejects Vite revenue-wallet configuration, malformed
or non-positive amounts, non-USDC settlement, duplicate vaults, and secret-like evidence text. A future
route operation is deliberately absent and must remain separately approved by humans.

## Closure classification

### A. Closed in this phase

- Pump launch-platform decision and the USDC-only platform-revenue accounting boundary are documented.
- The existing typed USDC `PlatformRevenue` route and fixed split are reused without changing program code.
- Offline artifact and batch-manifest validation is implemented and tested.

### B. Remaining code / evidence work

- Provide a reviewed build environment, run the required one-time reproducible Anchor build, record versions,
  and produce new artifact hashes.
- Resolve keypair/program-ID correspondence and add the required IDL `metadata.address` through the correct
  reviewed build/source process; do not edit deployed history.
- Obtain a separately approved read-only deployed-byte dump and compare it with the reviewed artifact.

### C. Remaining manual decisions

- Pump UI creator-fee recipient behavior, creator-fee public wallet, actual settlement asset, initial
  liquidity, pairing asset, mint/freeze policy, custody/LP handling, legal/risk review, communication, and
  final launch timing.
- A separate human approval for any later USDC Treasury route after a verified receipt.

### D. Irreversible confirmation list for the next phase

Before any irreversible action, record the reviewed commit, canonical charter, Program ID / IDL / keypair /
artifact correspondence, current deployed-byte comparison, Pump settings and recipient, mint/authority
policy, pair and liquidity decisions, legal/risk approval, exact public communication, signer identities,
and explicit human approval. Re-run the immutable evidence checks first. Only then may a separately scoped
launch operation be considered; this document grants none.

## Status statement

- **Implemented:** offline evidence verifier, offline Pump Creator Fee manifest, documentation alignment, and
  a small frontend label.
- **Tested:** targeted local TypeScript tests and typecheck (see command record in final phase report).
- **Not deployed:** all changes; no Mainnet program, ALPHA mint, Pump launch, fee route, or real-funds action.
- **Devnet:** historical program and integration evidence remain historical; no RPC query or modification was
  made in this phase.
- **Mainnet:** NO-GO and not launched.
