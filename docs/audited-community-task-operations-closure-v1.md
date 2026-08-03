# Alpha Protocol Audited Community Task Operations Closure V1

Status: workflow code deployed to Cloudflare Production; Staging migration applied; actor-level Staging E2E tooling pending review and deployment
Baseline commit: `5a31caeacf0e3b10f37d29ba409f7e58dfc8ff97`
Deployed workflow checkpoint: `73441a640413414feab0355583b2d22e770fdf07`
Phase: `2E-6B-4M`
Date: `2026-08-04`

## 1. Outcome

Phase 4M closes the first complete off-chain community-task workflow without
giving the operations database treasury authority:

1. an operator or governance administrator publishes a public task through a
   role-gated RPC;
2. a wallet-authenticated contributor submits a private result through the
   existing intake gate and owner RLS;
3. a reviewer, operator, or governance administrator accepts or rejects the
   submission through one role-gated RPC;
4. acceptance may publish only a reviewer-written sanitized result and only
   when the contributor explicitly consented;
5. each publication or review transition appends an immutable private audit
   event;
6. an accepted result remains an operations outcome, not a payment approval,
   treasury intent, or proof that USDC was transferred.

The migration contains no HTTP client, Solana transaction sender, private key,
treasury mutation, or automatic payout path.

## 2. Data separation

The workflow deliberately separates three data classes.

### 2.1 Public task definition

`community_tasks` contains the task title, summary, requirements, optional
declared USDC budget, declared reward source, deadline, and public status.

The budget is informational. It accepts zero through `1,000,000,000` USDC with
at most six decimals. It is not a balance reservation or payment instruction.

### 2.2 Private contributor submission

`task_submissions` remains protected by owner and staff RLS. It contains the
authenticated user link, full contributor description, submitted deliverable,
wallet, review state, and reviewer notes.

Two independent consent values are added:

- `public_result_consent`: permits publication of a separately written,
  sanitized result summary and public deliverable URL after acceptance;
- `public_wallet_consent`: separately permits publishing the wallet address.

Wallet publication consent cannot be true unless result publication consent is
also true. Consent does not make private submission text public automatically.

### 2.3 Sanitized public result

`task_submission_publications` contains only:

- task id and task title;
- sanitized result summary;
- safe HTTPS deliverable URL;
- optional wallet, only with separate consent;
- unique review reference;
- acceptance and publication timestamps.

It intentionally has no Auth user foreign key, private submission id,
`submitted_by`, `reviewed_by`, or raw reviewer notes. Rows are immutable.

### 2.4 Private workflow audit

`operations_task_workflow_events` records:

- task publication;
- submission acceptance;
- submission rejection;
- sanitized result publication.

Every event binds the entity, actor, actor role, unique audit reference,
structured event data, and timestamp. Events are immutable and are not
readable by `anon`.

## 3. Authority model

The public frontend never decides who is staff. The verified Supabase Web3
session must also carry an allowed `app_metadata.operations_role` claim.

| Action | Allowed roles |
| --- | --- |
| Publish task | `operator`, `governance_admin` |
| Read private task review queue | `reviewer`, `operator`, `governance_admin` |
| Accept or reject submission | `reviewer`, `operator`, `governance_admin` |
| Submit result | verified owner wallet while the independent intake gate is enabled |
| Read sanitized result | public |
| Read private workflow audit | staff policy scope only |

Direct authenticated `INSERT`, `UPDATE`, or `DELETE` task-management grants are
revoked where the RPC must be used. The RPCs perform validation, row locking,
state transition, publication, and audit insertion atomically.

## 4. Security review finding

### 4M-AUTH-001 — High before deployment, fixed locally

The first local draft used only this style of PL/pgSQL authorization check:

```text
role NOT IN (...)
```

When the role is `NULL`, PostgreSQL evaluates that expression to `NULL`, not
`TRUE`. A PL/pgSQL `IF` does not enter its rejection branch for `NULL`, which
could let an authenticated session without an operations role pass the initial
check of a `SECURITY DEFINER` function.

Both Phase 4M RPCs now reject when any of these are true:

- `auth.uid()` is `NULL`;
- the operations role is `NULL`;
- the role is outside the exact allowlist.

The runtime regression test calls both RPCs with an authenticated user that has
no operations role and proves they fail before reading or creating a workflow
record. This fix is included before any Phase 4M migration deployment.

No unresolved Critical or High finding remains in the reviewed Phase 4M scope.
This conclusion is not a professional Mainnet audit.

## 5. Review invariants

The review RPC enforces all of the following in one database transaction:

- reviewer identity and exact role allowlist;
- existing submission and task;
- row lock before state evaluation;
- no self-review;
- only `submitted` or `in_review` may transition;
- decision must be exactly `accepted` or `rejected`;
- reviewer notes and audit reference are mandatory;
- acceptance requires contributor result-publication consent;
- acceptance requires a fresh sanitized summary and safe HTTPS URL;
- wallet publication occurs only with separate wallet consent;
- rejection forbids all public-result fields;
- acceptance produces one sanitized result and two audit events;
- rejection produces no sanitized result and one audit event;
- terminal-state replay fails;
- immutable publications and events cannot be rewritten or deleted.

The status `accepted` never means `paid`. Any future contributor payment must
remain a separate governed treasury process with an independently reviewed
manifest and immutable on-chain or multisig receipt.

## 6. Frontend behavior

The public operations dashboard now:

- displays public tasks and sanitized accepted results separately;
- asks contributors for result-publication consent and optional wallet
  publication consent;
- keeps submission controls bound to a verified matching Web3 session and the
  independently controlled intake gate;
- shows the staff tab only when the verified session has an allowed operations
  role;
- lets operator/admin roles create public tasks through the publication RPC;
- lets allowed staff read the private queue and accept or reject through the
  review RPC;
- labels task budgets as declared budgets, not guaranteed payments;
- never exposes the service-role key or grants the browser treasury authority.

## 7. Migration and deployment sequence

The new migration is:

```text
supabase/migrations/202608030001_operations_task_moderation_closure.sql
```

The controlled actor-level Staging E2E requires one follow-up migration:

```text
supabase/migrations/202608040001_operations_task_staging_e2e_cleanup.sql
```

The follow-up does not add a product mutation path. It replaces the two task
workflow immutability triggers with an owner-bound variant and exposes one
`service_role`-only RPC that can delete only the exact reserved Phase 4M E2E
fixture graph. Direct browser and direct service-role table mutations remain
denied. The function rejects an invalid run reference, unexpected task or
submission content, unrelated rows, prefix collisions, duplicate ids, and
cleanup-count mismatches.

Applying it is a remote database mutation and requires a separate human
confirmation. Recommended order:

1. verify the deployed frontend checkpoint is exactly `73441a6` or a reviewed
   descendant;
2. verify the linked Supabase project is the dedicated Staging project;
3. confirm migration `202608030001` is already present remotely;
4. review and commit the E2E tooling and cleanup migration;
5. inspect `migration list`, run `db push --dry-run`, and run linked lint;
6. apply only migration `202608040001` after explicit confirmation;
7. confirm migration parity and rerun the read-only preflight;
8. obtain one fresh, process-only Turnstile response and separately confirm the
   mutating E2E;
9. perform controlled task publication, two owner submissions, acceptance,
   rejection, replay denial, immutable-record checks, and atomic cleanup;
10. require the runner to report 31 assertions and cleanup of nine database
    rows plus four temporary Auth users;
11. retain the independent intake gate as an emergency stop for all
    contributor writes.

Migration application does not publish a task, review a submission, send a
Solana transaction, or move funds.

## 8. Local verification

The local verification suite covers:

- client-side content, URL, wallet, budget, deadline, consent, decision, and
  audit-reference boundaries;
- exact role allowlists and missing-role rejection;
- RPC-only staff mutation privileges;
- owner wallet binding and intake-gate enforcement;
- operator task publication plus audit insertion;
- no-role task publication and review denial;
- self-review denial;
- acceptance without consent denial;
- sanitized acceptance with wallet omitted by default;
- rejected submission with no public result;
- terminal review replay denial;
- immutable result and audit rows;
- exact database table, policy, and trigger totals;
- absence of network senders or treasury mutations.

Verified locally at this checkpoint:

```text
operations tests: 83 passed, 0 failed
operations tooling TypeScript: passed
frontend TypeScript: passed
ESLint: passed
production Vite build: passed
git diff --check: passed
```

The Phase 4M Staging E2E tooling follow-up was then verified separately:

```text
operations tests: 88 passed, 0 failed
operations tooling TypeScript: passed
frontend TypeScript: passed
ESLint: passed
production Vite build: passed
git diff --check: passed
```

## 9. Current status

- implemented: task publication, private submission consent, role-gated
  review, sanitized public result, immutable audit history, staff UI, and
  adversarial tests;
- locally tested at deployed checkpoint `73441a6`: 83 operations tests, both
  TypeScript checks, ESLint, and the production build passed;
- locally tested in the E2E tooling follow-up: 88 operations tests, both
  TypeScript checks, ESLint, and the production build passed;
- Cloudflare Production: commit `73441a6` was reported successfully deployed;
- existing Supabase Staging: migrations through `202608030001` are applied,
  migration parity, linked lint, and the read-only preflight passed, and the
  wallet intake gate remains `wallet_staging`;
- Phase 4M actor-level Staging E2E: not yet executed against the task workflow;
- cleanup migration `202608040001`: implemented locally in the follow-up
  tooling change, not applied remotely;
- this local follow-up: no remote database mutation, no Auth user creation, no
  Staging E2E write, no deploy, no commit, and no push;
- Devnet: no program deployment, upgrade, initialization, or transaction in
  this phase;
- Mainnet: not entered and no transaction sent;
- treasury: no authority change, intent, signature, or funds movement.

This is not a Mainnet professional audit or legal review.
