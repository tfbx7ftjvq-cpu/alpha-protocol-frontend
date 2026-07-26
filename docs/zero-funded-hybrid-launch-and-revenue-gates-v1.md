# Zero-Funded Hybrid Launch and Revenue Gates V1

## Status

- Baseline commit: `7b6d40aee76b20ff44596bbfe0481ee0fd3ed7fe`
- ALPHA Mainnet token: not launched
- Alpha custom Mainnet program: not deployed
- Latest full program Devnet upgrade: not completed
- Initial treasury funding: `0 USDC`
- This document does not authorize a deployment, token launch, authority migration, or transaction.

## Operating model

Alpha Protocol launches with no pre-funded treasury. Real platform Creator Rewards are received by a
public revenue wallet. Revenue is counted only after the asset, amount, recipient, and transaction are
verified. Non-USDC revenue must be converted and confirmed as USDC before it enters the distribution
ledger.

Confirmed USDC revenue is distributed:

| Pool | Ratio | Use |
| --- | ---: | --- |
| Victim Relief | 50% | DAO-approved relief payments |
| Buyback / Burn | 20% | Publicly recorded buyback or burn batches |
| Builders | 20% | Accepted community work and operating expenses |
| Staking | 10% | Balance-backed USDC staking rewards |

No pool may borrow from another pool. An unfunded pool creates no debt, guaranteed payout, salary,
APY, or minimum return.

## Architecture boundary

### On-chain or publicly verifiable

- Creator Rewards receipt address and transaction history
- USDC revenue confirmation
- pool balances and distribution transactions
- governance authority used for a payment
- unique proposal or decision identifier
- payment, buyback, burn, and reward-funding receipts

### Off-chain product workflow

- task publication, claiming, submission, and milestone review
- risk reports, evidence, discussion, and moderation
- relief applications, documents, review, and appeal discussion
- Green Label application and due-diligence documents
- proposal text, forum discussion, profiles, search, and notifications

An off-chain database record must never be sufficient to spend treasury funds.

## Revenue states

| State | Condition | Allowed action |
| --- | --- | --- |
| Unconfigured | No verified revenue wallet | Show prelaunch status only |
| Zero | Verified wallet, confirmed USDC balance is zero | Community and transparency features only |
| Accumulating | Confirmed USDC below batch threshold | Record revenue; do not distribute |
| Distributable | Confirmed USDC at or above threshold | Prepare one auditable 50/20/20/10 batch |
| Pool enabled | A pool meets its activation balance | Enable only that pool's funded operation |

Initial distribution threshold: `100 USDC`. It is an operating default, not an immutable protocol
parameter.

Suggested activation gates:

- Builders: `100 USDC` in Builders pool
- Staking: `100–250 USDC` in Staking pool
- Victim Relief: `250–500 USDC` in Relief pool
- Buyback / Burn: `100 USDC` in Buyback pool

## Frontend network boundary

The existing wallet provider and protocol dashboards remain Devnet-oriented. The new launch
transparency page uses a separate read-only Mainnet connection. It does not request a signature and
cannot move funds.

Mainnet values are configured with public environment variables:

- `VITE_MAINNET_RPC_ENDPOINT`
- `VITE_ALPHA_LAUNCH_STATUS`
- `VITE_ALPHA_MAINNET_MINT`
- `VITE_REVENUE_WALLET`
- `VITE_ALPHA_PUMP_URL`
- `VITE_REVENUE_DISTRIBUTION_THRESHOLD_USDC`

If a Mint or wallet is absent, the frontend displays `unconfigured`; it must not present an invented
zero balance. Vite environment variables are public and must never contain private keys or secrets.

## Pump launch-day gate

Before changing `VITE_ALPHA_LAUNCH_STATUS` to `live`, independently record:

1. official ALPHA Mint and total supply;
2. mint and freeze authority status;
3. exact Pump launch URL;
4. Creator Rewards asset and calculation rule;
5. Creator Rewards recipient and whether it is mutable;
6. whether any option redirects rewards to traders;
7. whether any launch mode changes the intended one-billion supply;
8. exact creation and transaction costs;
9. a small real reward receipt;
10. public disclosure that the custom Alpha program is not live on Mainnet.

Do not assume Creator Rewards arrive as USDC. The accounting path must support SOL receipt followed
by a separately recorded conversion to USDC.

## Budget gates

| Stage | Treasury seed | Estimated SOL requirement |
| --- | ---: | ---: |
| Prelaunch development | 0 USDC | 0 SOL on-chain |
| Token launch | 0 USDC | Pump's launch-day charge plus network reserve |
| Recommended solo launch reserve | 0 USDC | 1–2 SOL total available |
| Realms DAO, later | Revenue-funded | approximately 2–2.5 SOL; re-check before creation |
| Streamflow staking, later | Staking pool funded | 1.3 SOL service fee plus transactions; re-check |
| Minimal custom treasury, later | Revenue/grant-funded | calculate from actual `.so` using `solana rent` |

The existing 3,462,528-byte full program is not part of the survival launch budget.

## Existing full protocol

The existing Treasury, Staking, Governance, Security Layer, Green Label, Victim Relief, and Authority
Hardening work remains:

- a long-term trust-minimized module library;
- Devnet/local technical evidence;
- a source for a future minimal treasury core;
- supporting evidence for grant applications.

It must not be described as the currently deployed Mainnet product.

## Frontend dependency hardening

The frontend uses only the Phantom wallet adapter required by the current UI. It does not install the
aggregate all-wallet adapter package. This removes unused WalletConnect, Trezor, Keystone, Particle,
and other wallet integrations from the dependency graph.

The Mainnet transparency reader derives the public USDC associated token address directly from the
canonical Token Program and Associated Token Program IDs. It does not require the larger SPL Token
client package for this read-only operation.

Validation requirements:

- `npm run typecheck`
- `npm run build`
- `npm run lint`
- `npm audit --omit=dev`

At this revision, the production dependency audit has no Critical or High findings. Remaining
Moderate reports originate from the current Solana web3 / wallet-adapter dependency chain and must
be reassessed before enabling Mainnet transaction signing. Development-only audit reports do not
ship in the static production bundle, but should still be updated when compatible fixes become
available.
