# Alpha Protocol Off-chain Operations Foundation V1

Status: local implementation only
Baseline commit: `83711210c8e3818a17036a4f3a56eb08afa2fa50`
Phase: `2E-6B-4F`

## 1. Purpose

Alpha Protocol does not need to place its entire administration inside one
Solana program.

The selected boundary is:

> Smart contract = constitution + vault + cashier.
> Operations database = intake + review + discussion + publication.

This phase creates an off-chain operating layer for:

- community tasks and contributor submissions;
- risk reports and evidence references;
- private victim-relief applications;
- moderated governance discussions;
- public governance decisions;
- separately recorded treasury execution manifests and receipts.

The database has no Solana private key, upgrade authority, treasury authority,
or transaction-sending function.

## 2. Non-negotiable money boundary

A database state must never be interpreted as treasury authorization.

The following states are administrative records only:

- task submission `accepted`;
- relief application `approved`;
- governance decision `approved`;
- execution intent `prepared`;
- execution intent `submitted`.

A financial action is complete only after the separately controlled execution
process has:

1. bound one immutable governance decision;
2. generated one deterministic execution manifest;
3. independently verified pool, mint, destination, and base-unit amount;
4. obtained the required wallet or multisig signatures;
5. submitted and confirmed the Solana transaction;
6. verified the confirmed transaction against the manifest;
7. inserted one immutable public execution receipt;
8. for relief, bound that receipt back to the same frozen application and
   destination wallet before marking the application `paid`.

There is no SQL trigger, RPC function, Edge Function, browser function, or
database webhook in this phase that moves assets.

## 3. Data classification

### 3.1 Private intake tables

The following records are private by default:

- `task_submissions`
- `risk_reports`
- `risk_evidence`
- `relief_applications`
- `governance_discussions`
- `treasury_execution_intents`

The submitting user can read their own intake record. Reviewer roles can read
or update records in their assigned scope. The browser cannot approve its own
record.

### 3.2 Sanitized public tables

Public records are copied into separate, deliberately sanitized tables:

- `risk_publications`
- `relief_public_updates`
- `governance_discussion_publications`

These tables do not contain an auth user ID or a foreign key to the private
source record. This prevents a public publication from revealing the original
reporter or claimant merely because someone queries all accessible columns.
They are append-only: a correction creates a new row with a
`supersedes_*` reference instead of rewriting public history.

The following protocol records are public by design:

- published `community_tasks`;
- published `governance_proposals`;
- immutable `governance_decisions`;
- immutable `treasury_execution_receipts`.

## 4. Identity and wallet meaning

The initial intake path uses a Supabase anonymous auth session. It gives RLS a
stable `auth.uid()` for ownership checks, but it is not proof of legal identity
or wallet ownership.

A wallet copied from Phantom into a form is stored with:

```text
wallet_verified = false
```

Connecting a wallet without signing a challenge does not change that flag.
V1 database constraints keep intake and published discussion wallet flags
false; enabling verified wallets requires a later reviewed migration.

Before any payout, a separate workflow must:

1. issue a nonce;
2. request a wallet signature;
3. verify the signature server-side;
4. bind the verified wallet to the frozen recipient record;
5. record the verification time and verifier;
6. reject any later payout-destination substitution.

That signature workflow is intentionally not claimed as complete in V1.

## 5. Authorization roles

Privileged roles are read from server-controlled Supabase Auth
`app_metadata.operations_role`.

Supported roles:

| Role | Scope |
| --- | --- |
| `reviewer` | task and risk review |
| `relief_reviewer` | private relief review and sanitized relief updates |
| `moderator` | discussion moderation and public discussion copies |
| `operator` | task/proposal/publication operations |
| `governance_admin` | governance decisions and cross-module administration |
| `executor` | private execution intents and immutable execution receipts |

Users must never be allowed to edit their own `app_metadata`. Role assignment
must use the Supabase admin API from a protected operator environment.

Do not place a service-role key in:

- Vite environment files;
- browser JavaScript;
- Git;
- public CI logs;
- screenshots;
- community operator devices without an explicit secrets policy.

The browser configuration helper rejects `sb_secret_` keys and JWTs whose role
is `service_role`.

## 6. State boundaries

### 6.1 Community task

```text
draft -> open -> under_review -> closed
                  \-> cancelled
```

A user may submit only to a published `open` task whose deadline has not
passed.

Task submission:

```text
submitted -> in_review -> accepted | rejected
          \-> withdrawn
```

`accepted` means the work passed the operating review. It does not create a
payment.

### 6.2 Risk report

Private review:

```text
submitted -> triaged -> investigating -> resolved | dismissed
```

A public risk record is created separately. A report is an allegation until
the published basis and governance outcome say otherwise.

### 6.3 Relief application

```text
submitted
  -> triaged
  -> evidence_requested
  -> under_review
  -> approved | rejected | cancelled
  -> paid
```

The `paid` status must only be applied after a matching confirmed execution
receipt exists. The current migration deliberately does not automate that
transition.

### 6.4 Governance and execution

Proposal:

```text
draft -> discussion -> voting -> decided
                           \-> cancelled
```

Decision records are immutable and unique per proposal.

Execution:

```text
prepared -> submitted -> confirmed
       \-> cancelled | failed
```

Execution receipts are immutable and require:

- one unique execution intent;
- one unique governance decision;
- an exact manifest SHA-256 match;
- an exact network match;
- an exact transaction-signature match;
- a confirmed intent state.

A relief execution intent additionally requires:

- an `approved` relief application;
- the exact application wallet as destination;
- a USDC amount no greater than the requested amount;
- a recipient-verification evidence reference;
- a unique application-to-intent binding.

## 7. Supabase migration

Migration:

```text
supabase/migrations/202607270001_offchain_operations_foundation.sql
```

The migration:

- creates the operations tables;
- enables RLS on every new table;
- grants only the required table actions;
- separates private intake from public publication;
- freezes submitted evidence/content while allowing reviewer state changes;
- makes sanitized publications append-only;
- rejects unlisted task, review, proposal, moderation, relief, and execution
  state transitions;
- creates immutable decision and receipt triggers;
- creates unique decision, intent, signature, and receipt bindings;
- creates no asset-transfer integration.

Before applying it to any remote project:

1. create a separate non-production Supabase project;
2. inspect the SQL diff in the Supabase CLI;
3. run database linting and RLS tests locally;
4. confirm no existing public schema object uses the same names;
5. apply to a staging project;
6. test as `anon`, anonymous `authenticated`, reviewer, governance admin, and
   executor;
7. keep the production project empty until staging behavior is accepted.

This phase did not apply the migration to any Supabase project.

## 8. Frontend configuration

The public frontend uses only:

```dotenv
VITE_SUPABASE_URL=
VITE_SUPABASE_ANON_KEY=
VITE_OPERATIONS_INTAKE_MODE=disabled
```

Fail-closed behavior:

- missing URL/key: public reads and forms are locked;
- malformed or non-HTTPS URL: locked;
- suspected service-role/secret key: locked;
- configured public key with intake `disabled`: public reads only;
- intake `anonymous`: anonymous session creation and form submission enabled.

The default is `disabled`.

Before switching to `anonymous`:

- enable Supabase Anonymous Sign-Ins;
- apply and verify all RLS policies;
- set project-level Auth rate limits;
- add abuse monitoring and alerting;
- define evidence retention and takedown procedures;
- decide whether CAPTCHA or an Edge Function intake gateway is required;
- verify that published records contain no private claimant information.

Anonymous sessions can be created repeatedly by an attacker. RLS is an
authorization boundary, not a complete anti-spam system.

## 9. User-facing functions

The new `社区运营与申请` page provides:

- public task list and task-result intake;
- sanitized public risk list and private risk intake;
- anonymized relief updates and private relief intake;
- moderated public discussion and private discussion intake;
- immutable public decision records;
- explicit database-versus-treasury boundaries.

When the backend is unconfigured, it displays a locked setup state. It does not
invent sample records or convert missing data into false zeros.

## 10. Validation

Frontend domain validation covers:

- Solana 32-byte Base58 address structure;
- UUID task identifiers;
- HTTPS-only links without embedded credentials;
- localhost rejection;
- required and optional wallet fields;
- text length boundaries;
- positive USDC amounts;
- six-decimal USDC precision;
- maximum requested amount.

Run:

```bash
cd project
npm run operations:test
npm run typecheck
npm run lint
npm run build
npm audit --omit=dev
```

Database validation still requires Supabase CLI or a disposable staging
project. A TypeScript test does not prove PostgreSQL policy behavior.

Local phase validation also executed the migration in an isolated in-memory
PostgreSQL runtime with minimal Supabase `auth.uid()`, `auth.jwt()`, `anon`, and
`authenticated` stubs:

```text
13 tables created
36 RLS policies created
34 triggers created
```

Runtime checks passed for:

- anonymous reads of explicitly public rows;
- owner-only private task submission reads;
- cross-user isolation;
- direct `wallet_verified=true` rejection;
- unprivileged review update isolation;
- rejected skipped state transitions;
- valid reviewer transitions;
- immutable sanitized publications.

The in-memory runtime does not package the `pgcrypto` extension, so only the
`create extension if not exists pgcrypto` line was omitted in that local
execution. Supabase provides `pgcrypto`; staging must still execute the
unmodified migration and repeat the checks. No remote Supabase project was
used.

## 11. Cost posture

This foundation can start with:

- a free/starter Supabase project;
- static frontend hosting;
- no protocol-funded treasury balance;
- no Mainnet program deployment;
- no automated keeper;
- no custody service.

Costs rise with file storage, egress, abuse controls, custom domains,
monitoring, and operational staffing. Evidence files should initially remain
in external HTTPS references; do not accept unlimited uploads before retention
and moderation rules exist.

## 12. Explicit deployment status

At completion of this local phase:

- the operations schema is code only;
- no Supabase migration has been applied remotely;
- no production backend is configured;
- community intake remains disabled by default;
- no Devnet transaction was sent;
- no Mainnet transaction was sent;
- the latest full Solana program was not upgraded on Devnet;
- no Alpha Protocol custom program is deployed on Mainnet;
- the existing uploaded Devnet buffer remains untouched.

This is an operational security foundation, not a professional Mainnet audit.
