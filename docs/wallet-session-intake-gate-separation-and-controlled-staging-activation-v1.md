# Alpha Protocol Wallet Session and Intake Gate Separation V1

Status: remote Staging gate active; CAPTCHA-compatible final E2E pending
Baseline commit: `a962c4e18d4aaab45570de49078397fd0cdca119`
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

Subsequent separately confirmed operations deployed the Phase 4L client,
applied migration `202608020002`, and activated the dedicated Staging gate to
`wallet_staging`. The first post-activation E2E stopped before operations-row
assertions because Supabase CAPTCHA rejected the old runner's password sign-in
without a challenge token. This follow-up changes only local test tooling,
frontend token handoff metadata, tests, and documentation; it performs no
remote mutation, deployment, Solana transaction, or treasury action.

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

### 5.1 CAPTCHA-compatible final E2E

Supabase CAPTCHA must remain enabled. After this compatibility patch is
reviewed, committed, and deployed to Pages:

1. open `https://alpha-protocol-frontend.pages.dev/` and keep the reviewed
   Phantom wallet connected;
2. if an operations session is already active, use `退出运营会话` so a new
   Turnstile widget is rendered;
3. complete Turnstile, but do not click the wallet-signature button because
   that would consume the response;
4. in browser DevTools Console, copy the current response without printing it:

```javascript
copy(window.turnstile.getResponse(document.querySelector('[data-turnstile-widget-id]').dataset.turnstileWidgetId))
```

5. within five minutes, run the following in the trusted PowerShell session:

```powershell
cd E:\a\alpha-protocol-frontend\project
$E2EExit = 1

try {
    $env:OPERATIONS_STAGING_E2E_CAPTCHA_TOKEN = Get-Clipboard
    $env:CONFIRM_OPERATIONS_STAGING_E2E = "I_UNDERSTAND_THIS_CREATES_AND_DELETES_STAGING_TEST_DATA"

    npm run operations:staging:e2e
    $E2EExit = $LASTEXITCODE
}
finally {
    Remove-Item Env:OPERATIONS_STAGING_E2E_CAPTCHA_TOKEN -ErrorAction SilentlyContinue
    Remove-Item Env:CONFIRM_OPERATIONS_STAGING_E2E -ErrorAction SilentlyContinue
    Set-Clipboard -Value ""
}

"E2E_EXIT=$E2EExit"
```

The runner rejects a token definition in `.env.operations-staging`; only the
current process may hold it. Three non-wallet test roles use admin-generated
one-time magic-link sessions. One ephemeral Solana Web3 actor consumes the one
Turnstile response. The switched-wallet and cross-user negative cases do not
require a second Web3 login, so CAPTCHA stays enabled and no test bypass is
introduced.

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

Original Phase 4L local verification result:

```text
operations tests: 64 passed, 0 failed
operations tooling TypeScript: passed
frontend TypeScript: passed
ESLint: passed
production Vite build: passed
```

CAPTCHA-compatible E2E follow-up verification result:

```text
operations tests: 67 passed, 0 failed
operations tooling TypeScript: passed
frontend TypeScript: passed
ESLint: passed
production Vite build: passed
```

At this checkpoint:

- implemented: session/gate separation, controlled gate tooling, audit schema,
  adversarial tests, and operator documentation;
- tested: local database simulation, TypeScript, lint, and production build;
- deployed: Phase 4L frontend commit `a962c4e` and migrations through
  `202608020002` on the dedicated Supabase Staging project;
- not deployed: this CAPTCHA-compatible E2E follow-up patch;
- Devnet: no transaction, deployment, upgrade, initialization, or E2E action;
- Mainnet: not entered and no transaction sent;
- treasury: no authority change and no funds moved;
- database intake gate: remotely `wallet_staging`, with one audited activation
  event;
- final post-activation E2E: not passed yet; the previous attempt failed at
  CAPTCHA-protected test authentication before operations-row assertions.

This is not a Mainnet professional audit or legal review.
