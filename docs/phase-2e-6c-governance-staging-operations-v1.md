# Phase 2E-6C Governance Staging Operations v1

This runbook is for controlled Staging validation only. It does not authorize Mainnet, a Solana transaction, a payment, a program change, or a remote migration from this task.

## Preconditions

1. Review the 6C migration and run local database tests first.
2. Apply reviewed migrations to Staging only through the established manual process.
3. Keep `.env.operations-staging` ignored; pass CAPTCHA tokens through the current process only.
4. Verify the dedicated Staging project ref/URL.
5. Provision distinct temporary proposal owner, operator, moderator, and governance-admin actors.
6. Enable the wallet intake gate only for the bounded test window.

## E2E proof

Use reserved reference `phase-2e-6c-staging-e2e:<13-digit-ms>-<8-lowercase-hex>`.

1. Authenticate a temporary Solana wallet by message signature only.
2. Submit exactly two private proposals named `Staging governance publish <run-id>` and `Staging governance reject <run-id>`.
3. Submit exactly two private discussions named `Staging discussion publish <run-id>` and `Staging discussion reject <run-id>`.
4. Prove anon cannot read intake/audit data; no-role and self-review calls fail.
5. As operator, publish one sanitized proposal and reject one. Verify no owner, wallet, raw summary, raw manifest, or reviewer notes leak.
6. As moderator, publish one separately written sanitized discussion and reject one. Prove operator moderation fails.
7. As independent governance admin, prove a manifest mismatch fails, then finalize the published proposal.
8. Verify a 64-character decision hash and the documented canonical tuple.
9. Verify zero execution intents and zero execution receipts. Do not send a transaction or contact an external sender.
10. Prove direct writes and immutable rewrites fail.

## Exact cleanup

Use `cleanup_governance_operations_staging_e2e_v1` only through the isolated service-role Node tool. Pass the exact run reference, owner UUID, exactly two proposal-submission UUIDs, and exactly two discussion UUIDs. The RPC checks reserved names, owner binding, unique IDs, reference isolation, and exact counts. `anon`/`authenticated` cannot execute it, and service role has no direct governance DELETE grant.

The existing single-discussion E2E fixture uses `cleanup_governance_discussion_staging_e2e_v1` with the same reference/owner/content/count protections. Never replace either RPC with broad deletion. Delete only exact temporary Auth users after row cleanup. On any mismatch, stop and preserve evidence rather than widening filters.

## Commands

From `project/`: `npm run operations:verify`, then separately authorized `npm run operations:staging:preflight` and `npm run operations:staging:e2e`. From repository root: `git diff --check`.

Record migration version, Staging project ref, run reference, temporary actor UUIDs, cleanup counts, tests, and zero-intent/zero-receipt proof. Never record keys, CAPTCHA tokens, signing material, or private bodies.

Phase 2E-6D execution-registry preparation is a separate, later manual Staging operation. Follow `audited-treasury-execution-registry-and-reconciliation-v1.md`; do not treat a 6C approval as an execution fixture or bypass the 6D legacy-history inventory gate.
