# Alpha Protocol Phase 2E-6B-4N

## Audited private risk evidence and sanitized publication closure

Status: implementation and local verification complete; Staging deployment pending

Scope: Supabase Staging operations layer only
Explicitly out of scope: Mainnet, Solana transactions, treasury authority, token actions, automatic compensation, insurance or credit ratings

## Outcome

Phase 4N closes the risk-report workflow without moving it into a Solana program. A wallet-authenticated user may submit a private report and append private evidence. A different authorized reviewer may either dismiss the report or publish a separately written, sanitized public record. The decision and its evidence count are recorded in a private immutable audit trail.

The database remains an operations and accountability layer. A published risk record is a reviewed community record, not an automatic fraud judgment, payment instruction, insurance decision or authorization to move funds.

## Security boundaries

| Data or action | Visibility / authority | Enforced boundary |
| --- | --- | --- |
| Raw risk report | Reporter and authorized staff | RLS; no anon grant |
| Evidence URL, hash and notes | Reporter and authorized staff | Private RLS; evidence cannot be marked public by the reporter |
| Sanitized publication | Public read | Reviewer-written fields only; no reporter user ID, wallet or private report foreign key |
| Workflow audit | Authorized staff only | Append-only immutable events |
| Review decision | Reviewer, operator or governance admin | Security-definer RPC with explicit role, state and self-review checks |
| Staging fixture cleanup | Service-role tooling only | Exact run reference, exact graph and exact deletion counts |
| Funds or on-chain action | None | No transaction sender, HTTP sender, treasury mutation or Solana instruction |

## Consent model

Two independent reporter choices are stored on the private report:

1. `public_report_consent` permits an independent reviewer to create a sanitized public record.
2. `public_reference_consent` separately permits one safe HTTPS reference in that public record.

Reference consent cannot be true unless report consent is true. Consent never publishes raw reporter text automatically. The reviewer must write the public summary and publication basis, and the public schema contains no Auth user ID, reporter wallet or private evidence metadata.

## Database changes

### `202608050001_operations_risk_moderation_closure.sql`

- Adds the two consent fields and their dependency constraint.
- Adds `operations_risk_workflow_events` with RLS, staff-only reads and immutable update/delete protection.
- Restricts new evidence to the authenticated owner of a nonterminal private report.
- Adds a database-side maximum of 12 evidence rows per user per hour.
- Revokes direct authenticated review/publication mutations.
- Adds `review_risk_report_v1(...)` for one atomic terminal decision, optional sanitized publication and audit event.
- Rejects missing/unauthorized roles, self-review, terminal replay, publication without consent, and public reference without separate consent.

### `202608050002_operations_risk_staging_e2e_cleanup.sql`

- Adds an owner-bound cleanup context only for an exact Phase 4N Staging fixture reference.
- Adds `cleanup_operations_risk_staging_e2e_v1(text, uuid[])` for exactly two reports, one evidence row, one publication and two audit events.
- Grants only function execution to `service_role`; it does not grant table deletion to browser roles or the service-role client.

## Application changes

- The risk intake form exposes separate public-record and public-reference consent.
- A verified reporter can append a private evidence URL, optional SHA-256 hash and private summary to their own nonterminal report.
- The staff workspace lists pending private risk reports and evidence counts.
- The reviewer UI requires a terminal decision, private reviewer notes, an audit reference, and—only for publication—a sanitized public summary and publication basis.
- Public records continue to come only from `risk_publications`.

## Verification contract

Local verification must prove:

- all migrations apply in order to a disposable PGlite database;
- roleless review and reporter self-review fail closed;
- consented publication creates one sanitized public row and one immutable event;
- dismissal creates an audit event but no public row;
- terminal replay and direct mutation fail;
- the public row excludes reporter identity and private evidence metadata;
- all TypeScript tests, type checks, lint and production build pass.

Verified locally against baseline commit `6b99d19240e1cc3f43cc1653f6cdb752d8d5e87e`:

- `npm run operations:verify` passed;
- operations tests: `99 passed, 0 failed`;
- operations tooling type check: passed;
- application type check: passed;
- ESLint: passed;
- Vite production build: passed.

The existing large-chunk build warning remains non-blocking and is not a risk-workflow authorization issue.

Controlled Staging E2E additionally exercises two wallet-owned reports, one private evidence row, independent staff review, public/anon separation, immutable output and exact cleanup. Expected combined result after Phase 4N:

- `Assertions: 46`
- `Cleanup: 15 rows, 4 users`
- `No Solana transaction or treasury action was performed.`

## Deployment order

1. Apply the reviewed source patch on the expected clean commit.
2. Run `npm run operations:verify` from `project`.
3. Commit and push; confirm Cloudflare Pages Production is built from that commit.
4. Confirm the linked Supabase project is `neevswvhndkalxkainxo`.
5. Dry-run and apply migrations `202608050001` and `202608050002` in order.
6. Confirm `migration list`, an empty post-push dry-run, `db lint --linked`, and `operations:staging:preflight`.
7. Capture one fresh Turnstile token from the current Production page and run the controlled Staging E2E once.
8. Confirm gate mode remains `wallet_staging`, cleanup counts are exact, transient environment variables are cleared, and the Git worktree remains clean.

## Emergency boundary

If unexpected intake behavior is observed, use the existing audited intake-gate emergency-disable operation. Disabling the gate stops new wallet-authenticated writes; it does not erase prior audit evidence or mutate any treasury/on-chain state. Do not manually delete production operations records or bypass the review RPC.

## Remaining work after 4N

- Assign and operationally protect real reviewer/operator identities; test accounts are not production governance.
- Define the community policy and appeal/correction process for public risk records.
- Add monitoring for review backlog, evidence-rate-limit rejections and unusual review activity.
- Perform independent security and legal review before describing the workflow as a production fraud adjudication system.
