# Audited Treasury Execution Registry & Reconciliation v1

Status: Phase 2E-6D local implementation; migration not remotely applied
Baseline: `b1b6e263cdff45c760e37a470670860ad1ab5377`

## Boundary

This phase is an off-chain registry for human preparation, authorization, external receipt reporting, and reconciliation. It contains no signer, transaction builder, Solana RPC client, broadcaster, balance mutation, payment authority, or automatic execution. Governance `approved`, intent `authorized`, receipt `reported`, and registry `reconciled` are distinct records; none alone means the protocol paid or executed a transaction. Mainnet and on-chain programs remain out of scope.

## Model and deterministic binding

An approved, execution-required `governance_decision` may produce at most one intent, and only through `prepare_treasury_execution_intent_v1`. Preparation requires an exact field-for-field match to the private approved manifest: pool, optional relief application, network, USDC mint/decimals, destination wallet, amount base units, recipient-verification reference, and purpose reference.

The database binds the decision hash and manifest SHA-256 into a deterministic intent hash. External receipt reporting copies the asset, destination, amount, decision, manifest, and intent bindings from that immutable intent; callers cannot substitute them. It computes a receipt hash over the bound tuple plus the externally supplied signature and timestamp. A report is an observation supplied by an executor—not database verification of Solana finality.

State transitions are explicit:

`prepared → authorized → reported → reconciled|failed`

`prepared|authorized → cancelled`

Direct table writes are revoked. Protected binding columns cannot be changed, receipts and private notes are immutable, and workflow events are append-only. Audit references are unique.

## Roles and separation of duties

| RPC | Required role | Independence |
| --- | --- | --- |
| `prepare_treasury_execution_intent_v1` | `treasury_preparer` | not the governance finalizer |
| `authorize_treasury_execution_intent_v1` | `treasury_authorizer` | not preparer or governance finalizer |
| `cancel_treasury_execution_intent_v1` | preparer or authorizer | only before reporting |
| `report_treasury_execution_receipt_v1` | `executor` | not preparer or authorizer |
| `reconcile_treasury_execution_v1` | `treasury_reconciler` | not preparer, authorizer, or reporter |

Missing, NULL, or unlisted roles fail closed. The browser uses the public authenticated client only. Service role appears only in isolated Staging cleanup tooling.

## Private/public separation

Private staff access covers full intents, exact destination wallets, recipient-verification evidence, private notes, receipts, actor IDs, and workflow events. `treasury_execution_public_registry` is a separate sanitized projection containing deterministic public hashes, purpose, amount/asset, a redacted wallet display, state, and external reference. It contains no private note, recipient evidence, actor ID, or full destination wallet. Anonymous users can read only this public registry; authenticated staff reads are RLS role-controlled.

## Existing-history safety gate

The migration refuses to run if either legacy execution table contains any row. Before a separately authorized application, run the read-only inventory queries in the migration header and preserve the results. Manually classify and migrate history with real source evidence and independent review. Never synthesize actor identities, hashes, recipient verification, authorization, signatures, or reconciliation evidence to make old rows fit.

## Staging verification and cleanup

The reusable `treasury-execution-e2e.ts` scenario accepts pre-created approved/rejected governance fixtures and distinct role clients. It proves rejected decisions and manifest mismatches fail, authorization creates no receipt, invalid/duplicate external reports fail, reconciliation is explicit, and a second intent can be cancelled. It never creates or sends a Solana transaction.

Use reserved reference `phase-2e-6d-staging-e2e:<13-digit-ms>-<8-lowercase-hex>`. Cleanup requires exactly two intent UUIDs and the exact preparer/fixture-owner UUID. `cleanup_treasury_execution_staging_e2e_v1` verifies purpose, owner, reference isolation, and exact counts, then removes only dependent 6D fixture rows. It is executable only by service role and grants no direct table DELETE capability. Governance decisions are intentionally retained for the 6C cleanup path.

## Migration and rollout

Local-only migration: `supabase/migrations/202608080001_audited_treasury_execution_registry_and_reconciliation.sql`.

Do not apply it remotely during code review. A future Staging change requires review of the inventory gate, distinct temporary actors, a reviewed migration application, bounded E2E execution, captured non-secret evidence, exact cleanup, and confirmation that no transaction/network sender was invoked.
