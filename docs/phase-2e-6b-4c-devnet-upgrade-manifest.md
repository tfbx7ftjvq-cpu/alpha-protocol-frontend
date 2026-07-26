# Phase 2E-6B-4C Devnet Program Upgrade Manifest

Status: Devnet upgrade attempted but not completed; recovery required before any retry.

This manifest was prepared after local script additions, `cargo test`, a temporary `anchor build --ignore-keys`, and a formal deployment build. The Devnet program upgrade was attempted after explicit user confirmation, but it did not complete. Do not retry upload, close buffers, or send any Devnet transaction until read-only RPC health and buffer state have been inspected.

## Fixed Values

- Cluster: Devnet
- RPC URL: `https://api.devnet.solana.com`
- Program ID: `HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY`
- Expected upgrade authority: `CqSs2yq6Jo3gYwXBq7fGRqohcxXS7HFJNYypykZTEGa8`
- Devnet USDC mint: `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU`
- Relief USDC vault: `GQSK91eQ5zwzGfYchunVqrPtxe3WLokxY88JbzTVcuRM`
- Vault authority V2: `FovfcDDZzc8ff2Z2uxNZ1fTjpuVoLkRTPUPTLvXL8TEK`

## Preflight Read-only Result

- Latest baseline commit: `11d3760 add protocol authority hardening and dao unpause v1`
- Wallet: `CqSs2yq6Jo3gYwXBq7fGRqohcxXS7HFJNYypykZTEGa8`
- Wallet balance before first upgrade attempt: `26.21362872 SOL`
- ProgramData: `4ckMZysHvxVr6v4KjuV7NqBvsykAbNKocRJCmvaYXpu6`
- Program executable: yes
- ProgramData authority matched wallet: yes
- Last confirmed ProgramData slot: `474538762`
- Last confirmed ProgramData length: `812568` bytes
- Current ProgramData balance at preflight: `5.65667736 SOL`

## Build Result

- `cargo fmt`: passed
- `cargo test`: passed, `595 passed; 0 failed`
- temporary `anchor build --ignore-keys`: Anchor exit code `0`
- temporary stack/verifier scan: `0` matches
- formal `anchor build --ignore-keys`: exit code `0`
- fresh IDL instruction count: `94`
- local binary: `server/target/deploy/my_first_solana_program.so`
- artifact size: `3462528`
- artifact SHA-256: `0a6e92625dd48641c4ea2bceb22d696e64068087f99b36a1bd768590e00ddd35`
- rent-exempt minimum for new binary size: `24.10008576 SOL`

## Local Tooling Hardening

- Devnet transaction scripts assert generated IDL account schema before instruction construction.
- The schema assertion checks instruction name, account count, account name, account order, signer flag, and writable flag.
- `execute_activate_protocol_dao_control_v1` requires both the common Devnet transaction gate and `CONFIRM_DAO_CONTROL_ACTIVATION=I_UNDERSTAND_DAO_CONTROL_IS_IRREVERSIBLE`.
- The package commands do not hardcode either confirmation value.
- The added tooling tests are local-only and do not access Devnet RPC.

## Failed Devnet Upgrade Attempt / Recovery Required

- Program ID: `HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY`
- Wallet / upgrade authority: `CqSs2yq6Jo3gYwXBq7fGRqohcxXS7HFJNYypykZTEGa8`
- Artifact size: `3462528`
- Artifact SHA-256: `0a6e92625dd48641c4ea2bceb22d696e64068087f99b36a1bd768590e00ddd35`
- Program upgrade completed: no
- Last confirmed ProgramData slot: `474538762`
- Last confirmed ProgramData length: `812568`
- Write-buffer result: failed with `40 write transactions failed`
- Later RPC/TLS failure prevented final orphan-buffer inspection.
- Mainnet transaction: none

Recovery must begin with read-only RPC and buffer inspection:

1. Confirm Devnet RPC health without sending transactions.
2. Inspect ProgramData and deployed program state read-only.
3. Inspect wallet-owned buffer accounts read-only.
4. If an orphan buffer exists, decide recovery or close only after RPC is healthy and the buffer state is understood.
5. Retry upload / upgrade only after the above recovery checks are complete.

## Working Tree Note

The working tree is intentionally dirty because this phase adds Devnet scripts and documents. Rust program source was not modified in this phase. The formal build output was generated from the existing Rust code baseline.

## Recovery Gate

The upgrade command has already been attempted and did not complete. The next operational step is not another upload attempt. The next step is read-only recovery inspection of Devnet RPC health, ProgramData, wallet-owned buffers, and any possible orphan buffer state.

## Explicit Non-Claims

- Devnet program upgrade has not completed.
- Protocol Authority Control has not been initialized on Devnet in this phase.
- Victim Relief strict E2E scripts have not been executed on Devnet in this phase.
- No Mainnet transaction was sent.

## Forbidden Actions Still Forbidden

- Do not run `anchor keys sync`.
- Do not create a new wallet.
- Do not overwrite keypairs.
- Do not modify Program ID.
- Do not enter Mainnet.
- Do not run Mainnet scripts.
- Do not retry upload or close buffers until read-only recovery inspection is complete.

## Current GO / NO-GO

- Devnet script/build readiness: READY for recovery inspection.
- Devnet program upgrade: NOT COMPLETED.
- Authority Control initialization: NOT EXECUTED.
- Victim Relief strict E2E: NOT EXECUTED.
- Mainnet production: NO-GO.
- Token launch: NO-GO.
