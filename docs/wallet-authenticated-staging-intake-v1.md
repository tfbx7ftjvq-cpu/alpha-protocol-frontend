# Alpha Protocol Wallet-Authenticated Staging Intake V1

Status: remote wallet authentication active; database intake remains locked
Baseline commit: `3700ba73ef117fdbb66f8db10a56b9a31752f79b`
Phase: `2E-6B-4I`
Date: `2026-07-30`

Follow-on note (`2026-07-31`): Phase `2E-6B-4J` adds a fail-closed
Cloudflare Turnstile challenge in front of Supabase Solana Web3 Auth. The
wallet-auth migration is now applied to the dedicated Supabase Staging project
and the frontend is deployed read-only on Cloudflare Pages. Intake remains
disabled. See `docs/turnstile-protected-wallet-auth-staging-v1.md`.

Compatibility note (`2026-07-31`): a live Phantom sign-in showed that Supabase
stores the canonical Web3 identity as `web3:solana:<address>` in
`provider_id` and `identity_data.sub`, without separate `chain` or `address`
properties. Phase `2E-6B-4K` updates both parsers while keeping the database
intake gate disabled. See
`docs/supabase-web3-solana-identity-compatibility-v1.md`.

Current-state note (`2026-08-02`): the Phase 4L client is deployed, migrations
through `202608020002` are applied to Staging, migration parity, linked schema
lint, and the read-only preflight passed. The database intake gate was
separately and explicitly activated to `wallet_staging` with one audit event.
The first post-activation E2E stopped at CAPTCHA-protected test authentication
before operations-row assertions. A locally verified compatibility follow-up
keeps CAPTCHA enabled and supplies one human-completed response only to the
single temporary Web3 actor; final remote E2E verification remains pending. See
`docs/wallet-session-intake-gate-separation-and-controlled-staging-activation-v1.md`.

## 1. Purpose

This phase replaces the superseded anonymous operations intake with a
wallet-authenticated Staging path.

The architecture boundary remains:

> Smart contract = constitution + vault + cashier.
> Operations database = intake + review + discussion + publication.

The browser can collect an application or discussion. The database can enforce
ownership, review state, publication boundaries, and rate limits. Neither can
sign a treasury transaction or move SOL, USDC, or ALPHA.

## 2. Authentication chain

An intake mutation is accepted only when all of the following agree:

1. the user connects a Solana wallet that supports message signing;
2. Supabase Web3 Auth verifies the signed Solana authentication message;
3. the returned Auth user has exactly one `web3` identity whose canonical
   subject has the exact form `web3:solana:<32-byte-Base58-address>`;
4. the frontend identity address equals the currently connected wallet;
5. the database function `current_verified_solana_wallet()` independently
   resolves the same address from `auth.identities`;
6. the submitted row's `submitted_by` equals `auth.uid()`;
7. the submitted row's `wallet_address` equals the database-resolved address;
8. the table's owner-insert RLS policy accepts every remaining initial-state
   field.

Email-only sessions, ambiguous Web3 identities, a changed connected wallet,
and a different submitted wallet fail closed.

The signed statement explicitly says that authentication:

- creates an off-chain session only;
- does not create a Solana transaction;
- does not approve a token;
- does not authorize a transfer of funds.

## 3. Exact project and page binding

The frontend requires all of these values before intake can become enabled:

```dotenv
VITE_SUPABASE_URL=https://<project-ref>.supabase.co
VITE_SUPABASE_ANON_KEY=<browser-safe-publishable-or-anon-key>
VITE_OPERATIONS_PROJECT_REF=<exact-20-character-project-ref>
VITE_OPERATIONS_WEB3_URL=https://<exact-staging-host>/<exact-page>
VITE_TURNSTILE_SITE_KEY=<browser-safe-site-key>
VITE_OPERATIONS_INTAKE_MODE=wallet-staging
```

The configuration rejects:

- an invalid, non-HTTPS, credential-bearing, or mismatched Supabase URL;
- a service-role or secret key in browser configuration;
- a missing or mismatched project ref;
- a Web3 URL containing credentials, a query string, or a fragment;
- a current browser origin or path that differs from the configured Web3 URL;
- a missing or malformed public Turnstile site key;
- any intake mode other than the exact value `wallet-staging`.

Query strings and fragments on the current page are removed before comparison,
but the origin and path must match exactly. The normalized configured URL is
passed explicitly to `signInWithWeb3`.

If a wallet-specific requirement fails, public reads remain available while
all mutations stay locked.

## 4. Database migration

The local migration is:

```text
supabase/migrations/202607300001_wallet_authenticated_operations_intake.sql
```

It adds:

- `operations_intake_control`, initialized to `disabled`;
- `is_operations_wallet_intake_enabled()`;
- `current_verified_solana_wallet()`;
- `enforce_operations_submission_rate_limit()`;
- wallet-bound insert policies for all four direct intake tables;
- one before-insert rate-limit trigger on each intake table.

Reviewed database limits are:

| Intake table | Limit |
| --- | ---: |
| `task_submissions` | 8 per Auth user per hour |
| `risk_reports` | 6 per Auth user per hour |
| `relief_applications` | 3 per Auth user per 24 hours |
| `governance_discussions` | 20 per Auth user per hour |

An advisory transaction lock serializes concurrent submissions for the same
Auth user and table before counting recent records. The trigger also rejects a
row whose `submitted_by` does not equal `auth.uid()`. It overwrites client
supplied `created_at` and `updated_at` values with database time so a forged old
timestamp cannot bypass the rolling window.

Applying the migration does not open the API. Every insert policy and every
rate-limit trigger checks the Postgres-owned control row. Browser roles cannot
read or update that row. Enabling it requires a separately reviewed database
change with a non-empty activation reference.
The control row's `updated_at` timestamp is refreshed by a protected trigger.

These limits reduce accidental and basic automated flooding. They do not stop
an attacker who controls many wallets. Supabase Web3 Auth rate limits,
CAPTCHA, monitoring, and moderation remain separate layers.

## 5. Wallet authentication is not payout verification

This phase proves which wallet created an operations record. It deliberately
does not mark the record as payout-ready.

Every new intake row still has:

```text
wallet_verified = false
```

That flag means the later payment workflow has not yet frozen and independently
verified the destination under the relevant relief, contributor, or governance
decision. A signed login is not a payroll approval, relief approval, governance
vote, multisig signature, or treasury authorization.

## 6. Frontend behavior

The operations dashboard now provides:

- Solana Web3 sign-in and local sign-out;
- visible connected-wallet and authenticated-wallet state;
- automatic locking when the wallets differ;
- read-only authenticated wallet fields in all four forms;
- wallet-authenticated task, risk, relief, and discussion intake;
- a private `我的提交` view protected by owner RLS;
- explicit messaging that accepted or approved records do not move funds.

The former anonymous-session creation path has been removed. The literal mode
`anonymous` is treated as disabled.

## 7. Staging E2E extension

The mutating Staging runner now requires:

```dotenv
OPERATIONS_STAGING_WEB3_URL=https://<exact-allowlisted-staging-page>
```

It also requires one fresh Turnstile response in the current process only:

```text
OPERATIONS_STAGING_E2E_CAPTCHA_TOKEN
```

That token must never be written to `.env.operations-staging`, an example
file, Git, logs, screenshots, or documentation. The runner rejects a persisted
definition and redacts the transient value from reported errors.

It creates:

- one email-authenticated operator;
- one email-authenticated moderator;
- one ephemeral Solana Web3 wallet actor;
- one email-only negative-control user.

The non-wallet roles receive admin-generated, one-time magic-link sessions.
They do not perform CAPTCHA-less password sign-in. The single Web3 actor
consumes the human-completed Turnstile response through Supabase Web3 Auth.
A separately generated public key supplies the switched-wallet negative case
without creating or authenticating a second Web3 user.

In addition to the Phase 4H checks, it verifies:

- a matching Web3 wallet can create its own intake row;
- the same Auth user cannot submit a different wallet;
- an email-only user cannot satisfy the wallet-bound insert policy.

The generated wallet keys exist only in process memory for the test. They are
not funded, written to disk, or used for a Solana transaction. Cleanup still
requires exact row IDs and deletes all four temporary Auth users.

## 8. Local verification

Run:

```bash
cd project
npm ci
npm run operations:verify
```

The verification suite covers:

- Web3 identity extraction and ambiguity rejection;
- exact project and page binding;
- browser secret rejection;
- four wallet-bound owner insert policies;
- a database-side intake gate that remains disabled after migration;
- four database rate-limit triggers;
- matching-wallet success;
- switched-wallet and email-only rejection;
- the hourly risk-report limit;
- the existing publication, moderation, execution, cleanup, and RLS
  invariants;
- operations tooling and application TypeScript;
- ESLint;
- the production Vite build.

Local result for this phase:

```text
operations tests: 47 passed, 0 failed
operations tooling TypeScript: passed
frontend TypeScript: passed
ESLint: passed
production Vite build: passed
```

`npm audit --omit=dev` still reports 14 moderate findings through the existing
`@solana/web3.js -> jayson -> uuid@8.3.2` dependency chain and reports no fix
available. This phase adds no package dependency and does not modify
`package.json` or `package-lock.json`. The finding remains a tracked dependency
limitation rather than a reason to apply an unreviewed forced upgrade.

## 9. Remote activation gates

Original Phase 4I implementation did not perform remote activation. Since that
code-only phase, the following Staging preparation has been completed:

- migrations through `202608020001` are applied to the dedicated Supabase
  Staging project;
- migration parity and the read-only preflight passed;
- linked schema lint reports no issues;
- the exact Cloudflare Pages URL is configured for Auth;
- a conservative Web3 Auth rate limit is configured;
- the frontend is deployed at
  `https://alpha-protocol-frontend.pages.dev/`;
- public database reads work from the deployment.

The following gates remain closed:

- database `operations_intake_control.mode=disabled`;
- Phase 4L gate-audit migration `202608020002` has not been applied remotely;
- no public wallet-authenticated intake has run.

Turnstile, Supabase CAPTCHA, the Solana Web3 provider, and frontend
`wallet-staging` mode are configured. The browser challenge and Phantom
signature reached Supabase, and Phase 4K now parses the observed canonical
identity subject. That historical sequence kept the database gate closed
through lint cleanup and the final authentication checks. Those checks are now
complete; Phase 4L keeps the gate closed under a new, separately confirmed
audit-migration and gate-activation process. It also removes the coupling
between session validity and gate state: a verified wallet may retain its
off-chain session while writes remain locked. A service-role key or Turnstile
secret must never be placed in a browser variable.

## 10. Official references

- [Supabase Web3 authentication](https://supabase.com/docs/guides/auth/auth-web3)
- [Supabase JavaScript `signInWithWeb3`](https://supabase.com/docs/reference/javascript/auth-signinwithweb3)
- [Supabase redirect URLs](https://supabase.com/docs/guides/auth/redirect-urls)
- [Supabase CAPTCHA](https://supabase.com/docs/guides/auth/auth-captcha)
- [Supabase Auth rate limits](https://supabase.com/docs/guides/auth/rate-limits)

This phase is not a Mainnet deployment, professional security audit, or legal
review.
