# Alpha Protocol Devnet Status

Last updated: 2026-06-27

## Project Positioning

Alpha Protocol is a real-yield insurance and governance protocol for Solana. The current Devnet milestone focuses on proving that protocol USDC revenue can enter an on-chain treasury, be split into dedicated vaults, and fund Alpha Guardian staking rewards without inflationary ALPHA emissions.

Current staking positioning: Alpha Guardian Staking is not a fixed-APR mining product. Users stake ALPHA to become Alpha Guardians and can claim USDC rewards sourced from real protocol revenue.

All data in this milestone is Devnet Alpha / testnet data / not mainnet funds.

## Program Status

- Program ID: `HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY`
- Upgrade authority: `CqSs2yq6Jo3gYwXBq7fGRqohcxXS7HFJNYypykZTEGa8`
- Last deployed slot: `471640168`
- Program data length: `440936` bytes

## Mints

- Devnet USDC mint: `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU`
- Devnet ALPHA test mint: `EKgyKk34WWhd5Ry1qHvfiXqtoxUAEaiX1g869tr6bbM1`

## Completed Modules

### USDC Treasury V2

USDC Treasury V2 has been deployed, initialized, and tested on Devnet. It routes incoming USDC revenue into four dedicated vaults:

- Relief pool
- Buyback and burn pool
- DAO builders pool
- Staking rewards pool

### Alpha Guardian Staking V1 Phase 1

Alpha Guardian Staking V1 Phase 1 has been deployed and validated on Devnet. The minimal closed loop is complete:

- Initialize staking pool
- Mint Devnet ALPHA to test wallet
- Stake ALPHA
- Receive protocol USDC revenue through Treasury V2
- Claim USDC staking rewards
- Unstake ALPHA

### System Security Layer V1

System Security Layer V1 has been deployed and verified on Devnet. It introduces governance decision recording, execution queueing, timelock enforcement, emergency pause protection, and queued-action cancellation.

Phase 1 execution is intentionally limited to mock/noop execution. It does not perform CPI, does not transfer funds, and does not modify Treasury V2 or Staking V1 parameters.

## Treasury V2 Addresses

- `treasury_config_v2`: `3eLgbfNTU8CY32JNhRQdnCwfiMUEZHGrD89ek6GJREvL`
- `treasury_usdc_state_v2`: `5e7eyC5ViwH9GBn73cY6so7J6KpRCX6XsbxozHabk2fE`
- `relief_usdc_vault`: `GQSK91eQ5zwzGfYchunVqrPtxe3WLokxY88JbzTVcuRM`
- `buyback_usdc_vault`: `D9M74v2tW78EbyPZgngsrB7DGxF8RMTpejiEyugGgoiR`
- `builders_usdc_vault`: `5XXaoWVSxVzyupzSs5NGXx6c8JMPD26QE7oZNmnUBAt8`
- `staking_usdc_vault`: `9nAUb7QG3mALgEUQZ26fHRa4p9BkfvKV5xGp6NFXA8wQ`
- `vault_authority_v2`: `FovfcDDZzc8ff2Z2uxNZ1fTjpuVoLkRTPUPTLvXL8TEK`

## Staking V1 Addresses

- `staking_pool_v1`: `91PjLExu9FCLY6KQuvuisEhTEciQyWXJGW9fMKUEHW35`
- `alpha_staking_vault`: `XtfRQViE9MWwvFG3EfuQWZXxqFdZd2BejumUFCRPED7`
- `alpha_vault_authority_v1`: `J9U364xC6sNX7vKMi917s9xDDnAJdDhNZEcY6MjwM134`
- Test `user_stake_account`: `C4ztgxaru9sAXC1vYX5UUE3gWPpaob6vXaWSx8AgdkvN`

## System Security Layer V1 Addresses

- `governance_config_v1`: `5np4fcpSP8eHVLD6dsgLHf7H11VLaGcYgdadxidt9ro3`
- Happy path `proposal_decision_v1` for `proposal_id` 1: `J3sJFvtsEe2p5gU3EbFBDHhQvFf3tCqKjVNEAE9sC99g`
- Happy path `execution_queue_item_v1` for `proposal_id` 1: `H3qH313B3iX6JamKRbUGnMxC6XVtsssJ8Ms7QceA7Et7`
- Cancel path `proposal_decision_v1` for `proposal_id` 3: `5tVHAJcAH5Wr1KHmG7owdQLKAsUiTVpQXn4f7rQd7Gqy`
- Cancel path `execution_queue_item_v1` for `proposal_id` 3: `G9TPbyxDk88t4A9TiGBjgG2L1xBp65bx7wRpWUaYSnNy`

## Important Transactions

- Staking pool initialize: `2d3iWV4KaEfWsbccApNd4vDv9hxLbKnj6J9A8758hFyPVVJfgdX7WAU5Fi1JNxToa9zSrcoF64FnjeyaDZPiwFUw`
- Staking V1 program deploy: `4gXSN2GM6sa2hdytbZqqUyKRTvqCNrhSCHBPyoGaAnfsXvSXApA21K7jkuAdWHD9KWgw1bPTENT8ZyEN8K21eCTr`
- System Security Layer V1 deploy: `37qbvSPbP9hipLY5Jr8W4nT79xg63joAUKioF5LyxumxEpfP4xeYZKK5d73Qy26HqbXn1YzfTB5nFoE4L5sSJ2br`
- Security Layer happy path queue execution: `hYgm2p5FQnx5AtDwc8QKz1eFiRmLJFnS2YsBvXQb2mqPKJC8V4Ah7EZAPqXyjgYd1dGMyPcQc8q4z7mUGKjumCS`
- Security Layer happy path execute queued action: `Uq4MuNqx7qatP8czdrmNckhj99atdEXmgKjzv3N3hhJz4givdTfxuXhZmqiWFLcjg2t256PhV2bSFHjaNYPWQ8n`
- Security Layer cancel queued action: `3c2WSpecDgPdcdnzt7bJ613wSG97UcAnTiwU9ZhTzQchDehrVEhucx9ZsdtX3K768t2qsEpifSR1S2Aj8YXjSJzs`

## Completed Devnet Test Flow

1. Staking pool initialized successfully.
2. 1000 Devnet ALPHA minted to the test wallet.
3. 100 Devnet ALPHA staked successfully.
4. 20 Devnet USDC revenue deposited through Treasury V2.
5. Treasury V2 split the USDC revenue across the four vaults.
6. `staking_usdc_vault` increased from about 2 USDC to about 4 USDC.
7. The staker claimed about 2 USDC.
8. `staking_usdc_vault` returned to about 2 USDC, with tiny rounding residue.
9. 100 Devnet ALPHA was unstaked successfully.
10. The wallet ALPHA balance returned to 1000, and `alpha_staking_vault` returned to 0.

## System Security Layer V1 Verified Flows

1. Happy path: governance decision was created, execution was queued, the timelock was observed, and the queued mock/noop action executed successfully.
2. Pause blocks execute: while the security layer was paused, execute failed as expected with `SecurityLayerPaused` / error number `6018` / custom program error `0x1782`.
3. Cancel blocks execute: after cancellation, execute failed as expected with `InvalidExecutionStatus` / error number `6020` / custom program error `0x1784`.

## Known Limitations

- Devnet only. These addresses and balances do not represent mainnet funds.
- Phase 1 staking uses a minimal reward index model.
- Phase 1 does not include epoch-based reward eligibility.
- Phase 1 does not include delayed reward activation for new stake.
- Phase 1 does not include emergency unstake.
- Phase 1 does not include Guardian Score or DAO voting weight.
- Phase 1 does not include multiple staking positions per wallet.
- Phase 1 does not include whale weight caps or marginal weight decay.
- Phase 1 does not include a full frontend staking flow yet.
- System Security Layer V1 Phase 1 execution is mock/noop only.
- System Security Layer V1 Phase 1 does not perform CPI.
- System Security Layer V1 Phase 1 does not transfer funds.
- System Security Layer V1 Phase 1 does not modify Treasury V2 or Staking V1 parameters.
- Current tests validate the closed loop, but the system has not completed an external audit.

## Next Planned Phase

System Security Layer V1 is now deployed and verified on Devnet. Green Label, Payroll, richer DAO governance, and production-facing staking UX should build on top of this security layer rather than ahead of it. The next phases should continue expanding:

- Protocol risk registry and exposure tracking
- Claim and payout safety checks
- Treasury movement monitoring
- Administrative action transparency
- Parameter governance boundaries
- Frontend risk disclosure and Devnet/Mainnet labeling
- Additional adversarial tests for reward accounting and vault authority flows
