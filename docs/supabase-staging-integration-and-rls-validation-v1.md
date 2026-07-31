# Alpha Protocol Supabase Staging Integration and RLS Validation V1

Status: remote staging active; actor-level RLS E2E and cleanup verified
Baseline commit: `50463230bf55f6a9d14b66f6679def099e1ba9a9`
Phase: `2E-6B-4H`

Follow-on note (`2026-07-30`): this document records the completed Phase 4H
email/role RLS baseline. Phase `2E-6B-4I` locally replaces the abandoned
anonymous-intake plan with Solana Web3 Auth and wallet-bound RLS. Its migration
has not yet been applied remotely. See
`docs/wallet-authenticated-staging-intake-v1.md`.

Current-state note (`2026-07-31`): migration `202607300001` was subsequently
applied and verified on the same dedicated Staging project. The Cloudflare
Pages frontend is live in public-read-only mode. Phase `2E-6B-4J` adds the
Turnstile client path, but CAPTCHA, Web3 provider, database intake gate, and
frontend wallet-staging mode remain disabled.

## 1. Purpose

Phase 4F created the off-chain operations foundation. Phase 4G prepared a
separate Supabase staging project to validate the database authorization
boundary with real Auth users and real PostgREST RLS behavior. Phase 4H
completed remote activation and closed the E2E cleanup privilege boundary.

This phase adds:

- a read-only staging preflight;
- a deliberately confirmed, self-cleaning RLS E2E runner;
- strict separation of public and service-role credentials;
- exact Supabase project-ref binding;
- a follow-up database hardening migration;
- pgTAP database assertions;
- local PGlite execution of all operations migrations.

The staging activation does not enable public intake, send a Solana
transaction, or change any treasury authority.

## 2. Architecture boundary

The selected boundary remains:

> Smart contract = constitution + vault + cashier.
> Operations database = intake + review + discussion + publication.

Supabase records do not authorize a payout. In particular:

- an accepted task submission is not a payroll transaction;
- an approved relief application is not a relief transfer;
- an approved governance decision is not a signer;
- an execution intent is not a confirmed transaction;
- only a separately verified immutable execution receipt may record an
  externally confirmed payment.

No migration or staging script contains a Solana private key, upgrade
authority, treasury authority, transaction builder, or send-transaction call.

## 3. Staging-only environment

Create a dedicated non-production Supabase project. Do not reuse a production
project and do not point these tools at an unrelated project.

Copy:

```text
project/.env.operations-staging.example
```

to:

```text
project/.env.operations-staging
```

The real file is explicitly ignored by `project/.gitignore`.

Required read-only values:

```dotenv
OPERATIONS_STAGING_PROJECT_REF=
OPERATIONS_STAGING_SUPABASE_URL=
OPERATIONS_STAGING_PUBLIC_KEY=
```

Required only for the mutating E2E:

```dotenv
OPERATIONS_STAGING_SERVICE_ROLE_KEY=
OPERATIONS_STAGING_WEB3_URL=https://<exact-allowlisted-staging-page>
CONFIRM_OPERATIONS_STAGING_E2E=I_UNDERSTAND_THIS_CREATES_AND_DELETES_STAGING_TEST_DATA
```

The URL must exactly equal:

```text
https://<OPERATIONS_STAGING_PROJECT_REF>.supabase.co
```

The parser rejects:

- malformed or non-HTTPS URLs;
- credentials, paths, query strings, or fragments in the URL;
- a URL whose host does not match the explicit 20-character project ref;
- `sb_secret_` or `service_role` material in the public-key field;
- a publishable/anon key in the service-role field;
- the same value in both key fields;
- any service-role/secret value exposed through a `VITE_*` variable;
- missing or inexact E2E confirmation.

Never copy `OPERATIONS_STAGING_SERVICE_ROLE_KEY` into:

- `project/.env`;
- `project/.env.local`;
- `VITE_*`;
- browser code;
- Git;
- screenshots;
- issue comments;
- CI output.

The Supabase secret/service-role credential maps to the `service_role`
database role and bypasses RLS. Its presence is limited to a trusted operator
process that creates and deletes test users and performs cleanup.

## 4. Migration order

Apply migrations in filename order:

```text
supabase/migrations/202607270001_offchain_operations_foundation.sql
supabase/migrations/202607270002_operations_staging_hardening.sql
supabase/migrations/202607290001_operations_staging_e2e_cleanup_privileges.sql
supabase/migrations/202607300001_wallet_authenticated_operations_intake.sql
```

The second migration fixes two findings identified while preparing real-role
tests. The third migration fixes the table-privilege boundary found by the
first remote staging E2E cleanup attempt.
The fourth migration belongs to follow-on Phase 4I. It creates a
wallet-authenticated intake path but leaves its database-side gate disabled by
default. The Phase 4H remote status below does not claim that migration has
been applied.

### 4.1 Published-record downgrade

The original trigger protected published content but excluded
`publication_status` from its comparison. An authorized operator could first
change `published` to `draft` and then rewrite fields that were intended to
remain frozen.

The replacement function now:

1. rejects every transition away from `published`;
2. continues allowing only the reviewed lifecycle `status` and `updated_at`
   changes;
3. rejects changes to every other published field.

The change applies to the existing task and governance-proposal protection
triggers without altering their account or table interfaces.

### 4.2 Moderator read policy

The original discussion policy granted moderators UPDATE scope but omitted a
matching SELECT policy. PostgreSQL RLS requires row visibility for a reviewer
to locate the record it must moderate.

The new policy grants authenticated users with one of these server-controlled
roles private discussion SELECT scope:

```text
moderator
operator
governance_admin
```

It does not expose discussions to `anon` or ordinary authenticated users.

### 4.3 Staging E2E cleanup privilege

The first remote actor-level E2E completed its RLS assertions but exited
non-zero during cleanup. The `service_role` database role bypasses RLS, but
PostgreSQL still requires table privileges for a filtered `DELETE`. The
affected tables had no `service_role` DELETE grant.

The third migration grants only `SELECT` and `DELETE` on the three temporary
fixture tables:

```text
community_tasks
task_submissions
governance_discussions
```

`SELECT` is required for the `id` filter and returned deletion proof. `DELETE`
remains explicitly revoked from `anon` and `authenticated`. No browser-facing
role receives a cleanup path.

## 5. Read-only preflight

Run from `project`:

```bash
npm run operations:staging:preflight
```

The preflight:

1. validates local configuration and secret isolation;
2. checks the Supabase Auth health endpoint;
3. performs `GET ...?limit=1` against seven intentionally public tables;
4. confirms anonymous reads are denied on six private intake tables;
5. performs no insert, update, delete, user creation, RPC mutation, or
   transaction.

Expected public tables:

- `community_tasks`
- `risk_publications`
- `relief_public_updates`
- `governance_proposals`
- `governance_discussion_publications`
- `governance_decisions`
- `treasury_execution_receipts`

Expected anonymous-denied tables:

- `task_submissions`
- `risk_reports`
- `risk_evidence`
- `relief_applications`
- `governance_discussions`
- `treasury_execution_intents`

A missing table, unexpected HTTP response, or anonymous private read fails the
preflight closed.

## 6. Real RLS E2E

Only after reviewing the exact staging project ref and accepting temporary
staging writes, set the confirmation value and run:

```bash
npm run operations:staging:e2e
```

The E2E first reruns the read-only preflight. It then creates temporary Auth
actors:

- operator;
- moderator;
- Solana Web3 owner A;
- Solana Web3 owner B;
- one email-only negative-control owner.

It validates:

- operator creation of a published community task;
- public anonymous read of that task;
- anonymous denial on private submissions;
- confirmation that the reviewed database-side intake gate is enabled;
- owner creation and read of a private submission;
- rejection when owner A submits owner B's wallet;
- rejection of an email-only owner that cannot prove wallet control;
- owner B isolation from owner A's submission;
- rejection of self-asserted `wallet_verified=true`;
- owner creation of a private governance discussion;
- moderator SELECT visibility;
- rejection of unprivileged moderation;
- successful moderator status transition;
- allowed published-task lifecycle status transition;
- rejection of publication downgrade;
- rejection of published-content rewrite.

Cleanup runs even when an assertion fails. It deletes test rows in dependency
order and then deletes temporary Auth users. Each row deletion must return
exactly the expected id before it is counted as successful. A cleanup error is
reported and must be resolved before rerunning.

The runner does not:

- use a production project;
- enable the public frontend;
- upload user evidence;
- create execution receipts;
- call any Solana RPC;
- sign or send a transaction;
- move USDC or SOL.

## 7. pgTAP

Database assertions are stored at:

```text
supabase/tests/database/operations_schema.test.sql
supabase/tests/database/operations_cleanup_privileges.test.sql
```

With the Supabase CLI and local database available:

```bash
supabase test db
```

The database tests check:

- all 14 operations tables, including the disabled-by-default intake control;
- RLS enabled on all 14 tables;
- the reviewed total of 37 policies;
- the reviewed total of 34 non-internal triggers;
- moderator discussion SELECT policy shape;
- absence of `anon` grants on private intake;
- published-downgrade protection in the installed function;
- exact `service_role` cleanup privileges on the three E2E fixture tables;
- continued denial of DELETE to `anon` and `authenticated`;
- absence of database HTTP or transaction-sending functions.

pgTAP results complement, but do not replace, actor-level PostgREST E2E.

## 8. Local verification

Run:

```bash
cd project
npm ci
npm run operations:verify
npm audit --omit=dev
```

`operations:verify` runs:

1. Node operations tests;
2. operations tooling TypeScript check;
3. frontend TypeScript check;
4. ESLint;
5. production Vite build.

The Node schema suite executes all operations migrations in isolated PGlite
databases.
The `pgcrypto` extension installation line is omitted only inside PGlite;
Supabase supplies `pgcrypto`, and the unmodified migration must be used for
local Supabase or staging.

The PGlite checks confirm:

```text
14 operations tables
37 RLS policies
34 non-internal triggers
```

They also execute the published-downgrade rejection and a moderator RLS read.

Running either staging command without configuration must fail closed. That
failure is expected during code-only validation and must not be described as a
remote staging pass.

## 9. Remote activation sequence

For a fresh staging project:

1. record the exact project ref out of band;
2. confirm the project contains no production data;
3. keep Anonymous Sign-Ins disabled; the follow-on intake path uses Solana
   Web3 Auth;
4. apply all operations migrations in order;
5. run `supabase test db`;
6. run the read-only preflight;
7. inspect Auth rate limits and abuse controls;
8. set the exact E2E confirmation;
9. run the RLS E2E once;
10. confirm cleanup completed;
11. manually inspect RLS policies and logs;
12. keep `VITE_OPERATIONS_INTAKE_MODE=disabled` until the staging review is
    explicitly accepted.

Remote migration application and E2E mutation require explicit human
confirmation because they modify a cloud project.

## 10. Deployment status

Current staging status after final Phase 4H verification:

- dedicated staging project `alpha-protocol-staging` is active;
- migrations `202607270001`, `202607270002`, and `202607290001` are applied
  remotely and the local/remote migration lists match;
- a post-activation dry run reports the remote database is up to date;
- remote schema lint reports no errors;
- read-only preflight passed with seven public tables readable and six private
  tables denied to `anon`;
- the corrected operations-schema pgTAP suite passed `1..22`;
- a remote privilege inspection confirmed `service_role` has `SELECT` and
  `DELETE` only on the three E2E fixture tables required for cleanup, while
  `anon` and `authenticated` retain no DELETE privilege on them;
- `npm run operations:verify` passed 35 of 35 tests, both TypeScript checks,
  ESLint, and the production Vite build;
- the final actor-level RLS E2E passed 14 assertions and exited zero;
- automatic cleanup reported three rows and four temporary Auth users deleted;
- an independent read-only residue query returned zero for community tasks,
  task submissions, governance discussions, and temporary Auth users;
- frontend community intake remains disabled by default;
- no Devnet transaction was sent;
- no Mainnet transaction was sent;
- no Solana authority or keypair changed;
- the existing uploaded Devnet program buffer was not closed or modified;
- the latest full Solana program remains not upgraded on Devnet;
- no Alpha Protocol custom program is deployed on Mainnet.

Subsequent verified Staging state:

- migration `202607300001_wallet_authenticated_operations_intake.sql` is also
  applied and local/remote migration parity is confirmed;
- remote database lint reports no schema errors after that migration;
- the read-only preflight still passes;
- Cloudflare Pages serves the frontend at
  `https://alpha-protocol-frontend.pages.dev/`;
- frontend public reads are configured, while
  `VITE_OPERATIONS_INTAKE_MODE=disabled`;
- the database-side operations intake control remains disabled;
- Supabase Anonymous Sign-Ins remain disabled;
- Supabase Web3 Wallet and CAPTCHA remain disabled pending the reviewed
  Turnstile activation sequence.

This phase verifies the selected off-chain authorization paths on the dedicated
Supabase staging project. It does not activate production intake, deploy or
upgrade the Solana program, or constitute a professional Mainnet audit.
