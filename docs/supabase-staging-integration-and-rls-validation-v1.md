# Alpha Protocol Supabase Staging Integration and RLS Validation V1

Status: local tooling and migration hardening only
Baseline commit: `db06008123d0de851053bd4578f33fa7dc1047ce`
Phase: `2E-6B-4G`

## 1. Purpose

Phase 4F created the off-chain operations foundation. Phase 4G prepares a
separate Supabase staging project to validate the database authorization
boundary with real Auth users and real PostgREST RLS behavior.

This phase adds:

- a read-only staging preflight;
- a deliberately confirmed, self-cleaning RLS E2E runner;
- strict separation of public and service-role credentials;
- exact Supabase project-ref binding;
- a follow-up database hardening migration;
- pgTAP database assertions;
- local PGlite execution of both migrations.

It does not create a Supabase project, apply a remote migration, enable public
intake, send a Solana transaction, or change any treasury authority.

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

The service-role key bypasses RLS. Its presence is limited to a trusted
operator process that creates and deletes test users and performs cleanup.

## 4. Migration order

Apply migrations in filename order:

```text
supabase/migrations/202607270001_offchain_operations_foundation.sql
supabase/migrations/202607270002_operations_staging_hardening.sql
```

The second migration fixes two findings identified while preparing real-role
tests.

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
- owner A;
- owner B.

It validates:

- operator creation of a published community task;
- public anonymous read of that task;
- anonymous denial on private submissions;
- owner creation and read of a private submission;
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
order and then deletes temporary Auth users. A cleanup error is reported and
must be resolved before rerunning.

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
```

With the Supabase CLI and local database available:

```bash
supabase test db
```

The test checks:

- all 13 operations tables;
- RLS enabled on all 13 tables;
- the reviewed total of 37 policies;
- the reviewed total of 29 non-internal triggers;
- moderator discussion SELECT policy shape;
- absence of `anon` grants on private intake;
- published-downgrade protection in the installed function;
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

The Node schema suite executes both migrations in isolated PGlite databases.
The `pgcrypto` extension installation line is omitted only inside PGlite;
Supabase supplies `pgcrypto`, and the unmodified migration must be used for
local Supabase or staging.

The PGlite checks confirm:

```text
13 operations tables
37 RLS policies
29 non-internal triggers
```

They also execute the published-downgrade rejection and a moderator RLS read.

Running either staging command without configuration must fail closed. That
failure is expected during code-only validation and must not be described as a
remote staging pass.

## 9. Remote activation sequence

When a real staging project is available:

1. record the exact project ref out of band;
2. confirm the project contains no production data;
3. enable Anonymous Sign-Ins only if anonymous intake will be tested;
4. apply both migrations in order;
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

At completion of the local 4G implementation:

- the second migration exists only in code;
- no Supabase cloud project was created;
- no migration was applied remotely;
- no staging Auth user was created;
- no remote RLS E2E ran;
- frontend community intake remains disabled by default;
- no Devnet transaction was sent;
- no Mainnet transaction was sent;
- no Solana authority or keypair changed;
- the existing uploaded Devnet program buffer was not closed or modified;
- the latest full Solana program remains not upgraded on Devnet;
- no Alpha Protocol custom program is deployed on Mainnet.

This phase validates local implementation and prepares real staging checks. It
is not a professional Mainnet audit and is not evidence that remote Supabase
RLS has already passed.
