# Phase 2E-6J — Program Identity Recovery and Reproducible Build Closure

Date: 2026-08-15  
Baseline commit: `b005bf732f460c34b036e03a50a73b4355a44a58`

## Scope and conclusion

This was a read-only identity-recovery investigation. No keypair or Program ID was generated, changed, or
copied; no program source, `Anchor.toml`, IDL, authority, deployment, upgrade, close, transaction, or fund
movement occurred. Mainnet was not contacted.

Final conclusion: **PROGRAM_KEYPAIR_UNAVAILABLE**.

The declared Devnet Program ID remains
`HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY`. Within the strictly allowed current repository,
`server/target/deploy`, explicitly related `E:\a` Alpha backups, accessible WSL project path, and Git-history
references, no program keypair that derives that public key was found.

## Read evidence

The canonical charter, Phase 2E-6I report, Anchor/Cargo configuration and lockfile, server package manifest,
generated IDL, Rust `declare_id!`, frontend Program ID constants, Devnet status, and prior upgrade manifest
were read. They consistently declare the Program ID above. Historical Devnet evidence records:

- ProgramData: `4ckMZysHvxVr6v4KjuV7NqBvsykAbNKocRJCmvaYXpu6`;
- historical upgrade authority: `CqSs2yq6Jo3gYwXBq7fGRqohcxXS7HFJNYypykZTEGa8`;
- historical local-build SHA-256: `0a6e92625dd48641c4ea2bceb22d696e64068087f99b36a1bd768590e00ddd35`.

## Toolchain inventory

| Tool | Windows / accessible WSL result |
| --- | --- |
| Anchor / AVM | Not found |
| Solana CLI / `solana-keygen` | Not found |
| Rust / Cargo | Not found |
| Node | `E:\node.exe`, `v24.16.0` |
| npm | `E:\npm.cmd`, `11.13.0` |
| WSL | distribution enumeration denied by host; no known WSL project mirror was accessible |

No tool was installed. `server/programs/my_first_solana_program/Cargo.toml` and `Cargo.lock` pin
`anchor-lang` and `anchor-spl` `1.0.2`, with `anchor-lang-idl-spec` `0.1.0`; `server/package.json` uses
`@coral-xyz/anchor` `^0.32.1` for TypeScript tooling. A future reviewed build host needs compatible Anchor
`1.0.2`, Rust/Cargo, and Solana CLI installed before running exactly one `anchor build --ignore-keys`.

Do not run these from this phase. On a separately approved build host, the Anchor requirement is explicit:

```bash
avm install 1.0.2
avm use 1.0.2
anchor --version
solana --version
rustc --version
cargo --version
```

`Anchor.toml` does not pin a Solana CLI release, so its exact installer command must be selected only after a
reviewed Anchor/Solana compatibility decision; this evidence set does not guess or install one.

## Candidate keypair search

One actual candidate was found; compiler fingerprint metadata named `solana-keypair` was excluded because it
is not a program keypair. `solana-keygen` was unavailable, so the existing local verifier derived only the
public key without printing the JSON array.

| Candidate path | SHA-256 | Derived public key | Result |
| --- | --- | --- | --- |
| `server/target/deploy/my_first_solana_program-keypair.json` | `8933a8c4f7bc1fee872eae631318dd31e25ea98ae80da693a84aec5ef2d97e6e` | `BQAiU9HDA3atHN7t6jnykBTHoAKp4R4aeHSDraR6v97A` | Does not equal declared Program ID |

Git history contains Program ID and ProgramData references, but no tracked program-keypair artifact. The
candidate therefore cannot establish the declared Devnet program identity.

## IDL binding result

Generated IDL values are:

- top-level `address`: `HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY`;
- `metadata.spec`: `0.1.0`;
- `metadata.address`: absent.

Local source for `anchor-lang-idl-spec 0.1.0` defines `Idl.address` as required and does not define
`IdlMetadata.address`. Its conversion code identifies `metadata.address` as a legacy pre-Anchor-v0.30
requirement. The artifact verifier now fails closed on any missing/mismatched top-level address or unsupported
IDL spec, while rejecting a present legacy `metadata.address` that disagrees. Absence of the legacy field is
therefore explained, not a binary-identity blocker.

## Build and deployed-byte evidence

No compatible Anchor toolchain was present, so no Phase 2E-6J build was run and no tool was installed. Current
local artifact SHA-256 values are:

- `.so`: `3b197267cec02096479b1c6eac50b2912cbafce6204d31ad66fb786d650588fe`;
- IDL: `a67b5b531c69412f9c29985adeb978ce473e5a22c711a70f521b573973ddde93`.

Solana CLI was unavailable, so no Devnet `program show` or `program dump` was run. There is no current
deployed-byte hash comparison. Historical artifact hashes remain evidence only and do not match the local
artifact above.

## Required next evidence

Recover the original, authorized program keypair through an approved custody path without copying its private
material into this repository; use `solana-keygen pubkey` on the reviewed host to establish its public key.
Then, on a compatible reviewed toolchain, run one build, record the `.so` and IDL hashes, perform read-only
Devnet show/dump, and compare bytes before considering any authority or deployment action.

Mainnet remains **NO-GO**.
