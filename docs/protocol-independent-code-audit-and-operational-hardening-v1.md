# Protocol Independent Code Audit and Operational Hardening V1

## Scope

Baseline commit:

```text
27d7fa046009351a3127c4cd69872adc684ffc44
```

Program ID under review:

```text
HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY
```

Fixed Devnet upgrade authority:

```text
CqSs2yq6Jo3gYwXBq7fGRqohcxXS7HFJNYypykZTEGa8
```

Reviewed files and direct dependencies:

- `server/programs/my_first_solana_program/src/instructions/victim_relief_v1.rs`
- `server/programs/my_first_solana_program/src/instructions/protocol_authority_control_v1.rs`
- `server/programs/my_first_solana_program/src/instructions/governance_action_v1.rs`
- `server/programs/my_first_solana_program/src/instructions/governance_adapter_v1.rs`
- `server/programs/my_first_solana_program/src/instructions/security_v1.rs`
- `server/programs/my_first_solana_program/src/instructions/treasury_execution_v1.rs`
- `server/programs/my_first_solana_program/src/state.rs`
- `server/programs/my_first_solana_program/src/error.rs`
- Devnet tooling under `server/scripts/devnet/alpha-v1`, `server/scripts/protocol-authority`, and `server/scripts/victim-relief`

## Explicit Non-Scope

This review did not deploy, upgrade, or initialize anything on Devnet or Mainnet. The new local program is not upgraded to Devnet. Authority Control is not initialized. DaoControlled mode is not activated. Victim Relief strict E2E was not executed.

This is not a third-party professional audit and must not be described as Mainnet audited.

## Methodology

The review followed the execution chain:

```text
entrypoint
-> Accounts context
-> signer / authority
-> PDA seeds and bump
-> account type and owner
-> status preconditions
-> proposal / action sidecar / canonical hash / decision / queue binding
-> token CPI inputs
-> receipt creation
-> final state updates
```

Tooling hardening was checked before any transaction construction path:

- Devnet-only guard
- fixed Program ID guard
- explicit transaction confirmation guard
- DaoControlled activation-specific confirmation guard
- IDL account schema consistency guard
- local-only TypeScript tests

## Security Invariants Reviewed

- Governance proposal action sidecars are the typed source for action, module, target, and canonical payload hash.
- Universal Governance Decision Adapter derives Security decisions from sidecar state, not caller-provided action data.
- Security queue execution is timelocked and replay-protected by queue status and payload hash.
- Protocol Authority legacy Security creation / queue / global unpause paths are allowed only in Bootstrap mode.
- DaoControlled activation is one-way and requires the current bootstrap authority plus governance chain.
- DAO global unpause recovery consumes the correct governance chain while the Security layer is paused.
- Victim Relief payouts only transfer from the relief vault PDA.
- Victim Relief payout amount and recipient are frozen in payout request / authorization records.
- Original approved payout and appeal overturn payout cannot both settle the same payout request.
- Payout and cancellation are mutually exclusive by request status and receipt PDA.
- Cancellation is risk-reducing and does not transfer tokens or change Treasury revenue accounting.
- Module pause blocks risk-increasing Victim Relief actions and payout transfers.
- Guardian can directly pause the Victim Relief module but cannot unpause it.
- Treasury execution uses DAO-enabled governance config, rejects emergency mode, and transfers only through typed request and receipt paths.

## Findings

### F-4D-01: Missing Irreversible DaoControlled Activation Confirmation

Severity: Medium

Files:

- `server/scripts/protocol-authority/activate-dao-control.ts`
- `server/scripts/devnet/alpha-v1/common.ts`

Issue:

`activate-dao-control.ts` already depended on the generic Devnet transaction confirmation, but the action is an irreversible authority-mode transition. A second, exact activation-specific local confirmation was required to reduce operator error.

Fix:

Added `CONFIRM_DAO_CONTROL_ACTIVATION=I_UNDERSTAND_DAO_CONTROL_IS_IRREVERSIBLE` enforcement in the shared tooling layer and invoked it before Devnet context loading or transaction construction. The value is intentionally not hardcoded in `package.json`.

Tests:

`server/scripts/devnet/alpha-v1/common.test.ts` covers missing, wrong, and correct activation confirmation values.

### F-4D-02: U64 Parser Error Text and Boundary Coverage

Severity: Low

File:

- `server/scripts/devnet/alpha-v1/common.ts`

Issue:

The shared `readU64` parser allows zero, but the old error text referred to a positive integer. The parser also needed explicit upper-bound tests for `u64::MAX`.

Fix:

Updated the parser wording to "non-negative integer string" and added explicit rejection for values above `2^64 - 1`. Business fields that require `> 0` remain responsible for business-level validation.

Tests:

`server/scripts/devnet/alpha-v1/common.test.ts` covers `0`, `1`, missing env, negative values, decimals, illegal strings, and values above `u64::MAX`.

### F-4D-03: Transaction Tooling Lacked Fresh IDL Account Schema Assertion

Severity: Medium

Files:

- `server/scripts/devnet/alpha-v1/common.ts`
- `server/scripts/protocol-authority/activate-dao-control.ts`
- `server/scripts/protocol-authority/init-bootstrap.ts`
- `server/scripts/protocol-authority/recovery-unpause-e2e.ts`
- `server/scripts/victim-relief/setup.ts`
- `server/scripts/victim-relief/run-strict-e2e.ts`

Issue:

Fresh IDL checks verified instruction existence, but transaction scripts still manually maintained account order. An account order, signer, or writable mismatch could reach transaction construction and rely on later failures.

Fix:

Added centralized IDL account schema assertion that verifies instruction name, flattened account count, account name, order, signer, and writable flags before constructing each transaction instruction. Newly added Protocol Authority and Victim Relief Devnet scripts now pass named account metadata into `buildIx`.

Tests:

`server/scripts/devnet/alpha-v1/common.test.ts` covers matching schema, nested account flattening, count mismatch, name/order mismatch, signer mismatch, writable mismatch, and missing script account names.

### F-4D-04: Devnet E2E Documentation Encoding Artifact

Severity: Informational

File:

- `docs/victim-relief-devnet-strict-e2e-v1.md`

Issue:

The confirmation phrase for Devnet program upgrade contained mojibake.

Fix:

Restored the intended text:

```text
确认执行Devnet program upgrade
```

### F-4D-05: Rust Protocol Static Review

Severity: Informational

Files:

- `victim_relief_v1.rs`
- `protocol_authority_control_v1.rs`
- `governance_action_v1.rs`
- `governance_adapter_v1.rs`
- `security_v1.rs`
- `treasury_execution_v1.rs`

Result:

No credible Critical or High finding was identified in the reviewed local code path.

Evidence:

- Governance action mappings use exhaustive matches and stable codes instead of enum casts.
- Adapter derives Security action, target, and payload hash from `GovernanceProposalActionV1`.
- Victim Relief payout validators bind request, case, snapshot, authorization receipt, frozen recipient, relief vault PDA, vault authority PDA, USDC mint, module pause, global pause, and one-per-request payout receipt.
- Victim Relief cancellation validators bind original authorization, cancellation proposal/action/adapter/decision/queue, request status, case status, immutable cancellation receipt, and active case count decrement.
- Protocol Authority legacy Security authority paths require Bootstrap mode and fail after DaoControlled activation.
- DAO global unpause wrapper requires DaoControlled mode, an eligible paused Security config, the correct target, parameters hash, adapter, decision, queue, and timelock.
- Treasury execution validates DAO-enabled sidecar config, emergency mode disabled, typed governance request binding, destination token metadata, builders vault authority, and immutable execution receipt.

Residual limitations:

- This is local static review plus local tests, not Devnet verification.
- Existing Devnet still runs the old program until a successful upgrade is performed.
- Authority Control is not initialized on Devnet.
- DaoControlled mode is not activated on Devnet.
- Victim Relief strict E2E is not executed on Devnet.
- Mainnet professional audit is not complete.

## Test Additions

New local-only TypeScript adversarial tests were added for:

- missing activation confirmation
- wrong activation confirmation
- correct activation confirmation
- `readU64` zero / one / negative / non-integer / invalid string / above-u64 cases
- IDL account schema success
- nested account flattening
- account count mismatch
- account order/name mismatch
- signer mismatch
- writable mismatch
- missing named account metadata

The Rust protocol adversarial surface was reviewed against the existing unit tests. No Rust protocol change was made in this phase because no credible Critical or High issue was found and the implemented hardening target was Devnet tooling.

## Validation Requirements

Expected safe local validation for this phase:

- `cargo fmt`
- `cargo fmt --check`
- `cargo test`
- `anchor build --ignore-keys`
- fresh IDL instruction count check
- TypeScript static check for the touched scripts
- `npm run tooling:test`
- `git diff --check`
- stack / verifier keyword scan
- Program ID unchanged check
- `Anchor.toml` unchanged check
- keypair diff check

## Current Deployment Statement

The current local program was not upgraded to Devnet in this phase. Devnet still runs the previously deployed program. No Devnet transaction, Mainnet transaction, Authority Control initialization, DaoControlled activation, or Victim Relief Devnet E2E was executed.
