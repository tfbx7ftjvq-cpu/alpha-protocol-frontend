# Alpha Protocol Wallet Session and Intake Gate Separation V1

Status: local implementation verified; remote intake remains disabled
Baseline commit: `818ed9f5b75ab8212abd285534f0b301cbef1b8d`
Phase: `2E-6B-4L`
Date: `2026-08-02`

## 1. Outcome

Phase 4L separates two security facts that were previously coupled in the
frontend:

1. whether Supabase has authenticated exactly one matching Solana wallet; and
2. whether the independently controlled database intake gate currently allows
   new Staging writes.

A disabled gate is no longer treated as an authentication failure. A verified
wallet session may remain visible and may read its own previously submitted
records while all new submission forms remain locked.

No remote migration, gate transition, Solana transaction, treasury action, or
deployment is performed by this local phase.

## 2. Required behavior

| Wallet session | Database gate | Private history | New submissions |
| --- | --- | --- | --- |
| invalid or absent | any state | denied | denied |
| verified and matching | disabled | allowed by owner RLS | denied |
| verified and matching | unavailable or error | allowed only if the session remains valid | denied |
| verified and matching | enabled | allowed by owner RLS | allowed by owner RLS and database checks |

The frontend is only one of three independent write guards:

1. the browser must have a verified wallet session matching the connected
   wallet;
2. the remote intake gate must report `enabled`;
3. Supabase owner-insert RLS, wallet binding, initial-state constraints, and
   database rate limits must accept the row.

No browser environment variable can override the database gate.
`VITE_OPERATIONS_INTAKE_MODE=wallet-staging` only enables the reviewed browser
authentication capability.

## 3. Auditable gate transition migration

The new migration is:

```text
supabase/migrations/202608020002_operations_wallet_intake_gate_audit.sql
```

Applying this migration does not enable intake. It preserves the existing
singleton control row and its current `disabled` mode, then adds:

- an append-only `operations_intake_gate_events` audit table;
- a fixed-search-path, security-definer transition RPC;
- exact `disabled` and `wallet-staging` modes only;
- mandatory 10–200 character change references;
- row locking and same-mode replay rejection;
- one audit event for every successful transition;
- service-role-only RPC execution and audit inspection;
- no browser-role direct access to the control or audit tables;
- no direct service-role update of the control row or direct audit write.

The RPC does not send HTTP requests, call a Solana program, hold a private key,
or move funds.

## 4. Operator commands

Read-only inspection requires the dedicated Staging service-role key in the
ignored `project/.env.operations-staging` file:

```powershell
cd E:\a\alpha-protocol-frontend\project
npm run operations:staging:gate:inspect
```

Activation requires both an auditable reference and the exact confirmation:

```powershell
$env:OPERATIONS_STAGING_GATE_CHANGE_REFERENCE = "<review-ticket-or-change-reference>"
$env:CONFIRM_OPERATIONS_STAGING_GATE_ACTIVATION = "I_UNDERSTAND_THIS_ENABLES_WALLET_AUTHENTICATED_STAGING_WRITES"
npm run operations:staging:gate:activate
Remove-Item Env:CONFIRM_OPERATIONS_STAGING_GATE_ACTIVATION -ErrorAction SilentlyContinue
Remove-Item Env:OPERATIONS_STAGING_GATE_CHANGE_REFERENCE -ErrorAction SilentlyContinue
```

Emergency or planned disablement uses a different exact confirmation:

```powershell
$env:OPERATIONS_STAGING_GATE_CHANGE_REFERENCE = "<incident-or-change-reference>"
$env:CONFIRM_OPERATIONS_STAGING_GATE_DISABLE = "I_UNDERSTAND_THIS_DISABLES_WALLET_AUTHENTICATED_STAGING_WRITES"
npm run operations:staging:gate:disable
Remove-Item Env:CONFIRM_OPERATIONS_STAGING_GATE_DISABLE -ErrorAction SilentlyContinue
Remove-Item Env:OPERATIONS_STAGING_GATE_CHANGE_REFERENCE -ErrorAction SilentlyContinue
```

Do not put secrets, keys, personal data, or raw evidence in the change
reference. The reference is intentionally stored in the audit trail.

## 5. Controlled remote sequence

Remote work must remain split into independently confirmed checkpoints:

1. deploy the Phase 4L client while the current database gate remains
   `disabled`;
2. separately inspect and apply migration `202608020002`;
3. prove migration parity, clean lint, read-only preflight, and gate inspection
   still report `disabled` with no unexpected audit history;
4. repeat Turnstile plus Phantom authentication and verify the wallet session
   remains authenticated while submission controls stay locked;
5. only after a separate explicit decision, activate the gate with a reviewed
   change reference;
6. run the controlled wallet-authenticated Staging RLS E2E and verify cleanup;
7. disable the gate immediately if any identity, RLS, rate-limit, cleanup, or
   UI invariant fails.

Applying the migration and activating the gate are deliberately different
operations. Confirmation of one never authorizes the other.

## 6. Verification and boundaries

Local adversarial coverage includes:

- verified sessions with enabled, disabled, unavailable, and error gate states;
- signed-out and switched-wallet rejection;
- exact and distinct activation/disable confirmation strings;
- missing, short, oversized, and control-character change references;
- direct control-row update denial;
- append-only gate-event enforcement;
- same-mode replay rejection;
- exact two-event activation/disable history;
- browser-role RPC and audit-table denial.

Local verification result:

```text
operations tests: 64 passed, 0 failed
operations tooling TypeScript: passed
frontend TypeScript: passed
ESLint: passed
production Vite build: passed
```

At this checkpoint:

- implemented: session/gate separation, controlled gate tooling, audit schema,
  adversarial tests, and operator documentation;
- tested: local database simulation, TypeScript, lint, and production build;
- deployed: the earlier Phase 4K frontend and migrations through
  `202608020001` only;
- not deployed: Phase 4L frontend changes and migration `202608020002`;
- Devnet: no transaction, deployment, upgrade, initialization, or E2E action;
- Mainnet: not entered and no transaction sent;
- treasury: no authority change and no funds moved;
- database intake gate: still remotely `disabled`.

This is not a Mainnet professional audit or legal review.
