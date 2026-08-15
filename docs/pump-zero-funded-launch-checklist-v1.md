# Pump Zero-Funded Launch Checklist V1

This checklist is a manual, irreversible-operation gate. Completing development does not authorize
the launch.

## Preflight — no transaction

- [ ] Windows, GitHub, and WSL are synced to the reviewed commit.
- [ ] Worktrees are clean.
- [ ] Frontend build and typecheck pass.
- [ ] Official name, ticker, metadata, links, and one-billion supply are reviewed.
- [ ] Mayhem or any supply-changing mode is disabled.
- [ ] Creator Rewards recipient behavior is confirmed in the live Pump UI.
- [ ] “Rewards go to traders” or an equivalent redirect is disabled unless deliberately approved.
- [ ] Revenue wallet is a dedicated public address.
- [ ] Revenue wallet private key is not present in frontend, repository, database, CI, or Vite env.
- [ ] Pump creation cost and wallet SOL balance are rechecked.
- [ ] Mainnet RPC is private/redundant for production use.
- [ ] Public risk disclosure says the treasury starts at zero.
- [ ] Public risk disclosure says rewards, relief, salaries, and buybacks depend on actual pool balance.

## Human confirmation — token launch

Record before signing:

- date and time;
- wallet public key;
- Pump creation settings;
- fee recipient;
- quoted total cost;
- final metadata;
- explicit human approval.

## Post-launch verification

- [ ] Record official Mint.
- [ ] Verify supply.
- [ ] Verify mint authority status.
- [ ] Verify freeze authority status.
- [ ] Record Pump URL.
- [ ] Record Creator Rewards recipient.
- [ ] Execute no treasury distribution until a real reward receipt is observed.
- [ ] Verify the first receipt asset and amount.
- [ ] If settlement is not USDC, stop automatic revenue routing; record it as manual-review evidence only.
- [ ] Configure frontend public values.
- [ ] Change launch status to `live` only after all values are independently checked.
- [ ] Publish explorer links.

## Revenue batch

- [ ] Confirm eligible canonical-USDC Pump Creator Fee / Platform Revenue.
- [ ] Exclude refundable Green Label escrow.
- [ ] Assign a unique batch ID.
- [ ] Calculate 50/20/20/10 using deterministic rounding.
- [ ] Ensure all four destinations are correct.
- [ ] Record transaction signatures.
- [ ] Reconcile pre/post balances.
- [ ] Publish any rounding remainder.

No automatic SOL-to-USDC conversion is authorized. The offline manifest helper only inspects and prepares
the fixed `50 / 20 / 20 / 10` USDC routing evidence; it does not construct or submit a transaction.

## Explicitly not authorized

- full Alpha Mainnet program deployment;
- Devnet full-program upgrade;
- closing buffer `7dzjX9Dtwk2KsyvWhLCdMw5Au1bThDhpFc18Z3dyRGZi`;
- authority migration;
- DaoControlled activation;
- Realms creation;
- Streamflow pool creation.
