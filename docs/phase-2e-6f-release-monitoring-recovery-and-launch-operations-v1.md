# Phase 2E-6F: Release Evidence, Monitoring, Recovery and Launch Operations

Status: local tooling and runbook implemented; no deployment action is implied by this document.

Formal code baseline: `c80702b738f6d6414e5323ddda6045a9427bcdac`. The applied
Supabase migration baseline currently recorded for Staging is through
`202608110002_operations_access_control_compatibility_fix.sql`; every release
operation must independently confirm that parity, and no already applied
migration may be edited. Cloudflare Pages frontend deployment, Supabase Staging
operation, Devnet programs, and Mainnet are separate states.

## 1. Release evidence and Pages confirmation

Every production Vite build writes `/release.json` with only `schemaVersion`,
`commitSha`, `branch`, and `buildContext`. Cloudflare commit metadata has first
priority, followed by GitHub metadata, a deliberate non-browser
`RELEASE_COMMIT_SHA`, then the explicit local fallback. Cloudflare and GitHub
commit values must be a lowercase 40-character hexadecimal Git SHA. The file
contains neither keys nor machine paths.

Before declaring a Pages release verified, record the time, operator, Git
commit, Pages URL, deployment status, and the following evidence:

1. Cloudflare Pages Production reports the intended verified commit as deployed.
2. `GET /release.json` returns that exact commit and `Cache-Control: no-store`.
3. The Pages home response is HTTP 200, not a Cloudflare error document, and
   has `nosniff`, strict referrer policy, frame denial, the camera/microphone/
   geolocation Permissions-Policy, and `same-origin-allow-popups` COOP.
4. Run the read-only probe only after separately providing the expected SHA:

   ```bash
   cd project
   npm run operations:staging:release:inspect
   ```

The probe fails closed on redirects, network errors, bad Pages responses,
missing headers, malformed release metadata, or a commit mismatch. It performs
only GET/read RPC requests and has a stable nonzero exit code on failure, so a
Windows Task Scheduler or cron task may invoke it. It sends no email, webhook,
Slack message, or automatic remediation.

## 2. Supabase release confirmation

Use a trusted operator workstation and keep all credentials out of command
history, screenshots, logs, and this repository. Record the project ref,
migration parity result, gate mode, and time, but never a URL key or token.

1. Inspect the intended project with `supabase projects list` and confirm its
   project ref against the release record.
2. Run `supabase migration list`, then `supabase db push --dry-run`; require
   parity through `202608110002` and an empty reviewed dry run.
3. Run `supabase db lint --linked` and resolve every error before release.
4. Run the existing read-only `npm run operations:staging:preflight` and then
   `npm run operations:staging:gate:inspect` under their existing credential
   controls.
5. The release-inspect probe independently checks Auth health, seven public
   anonymous reads, seven private anonymous denials, and reports the wallet
   intake gate mode. It does not create users, grant/revoke roles, or run E2E.

## 3. Backup and recovery rehearsal

Before a release, capture a read-only inventory: project ref, migration list,
linked lint result, release commit, gate mode, and role assignment inventory.
Use the Supabase Dashboard to determine the project’s actual backup and PITR
retention capability; do not assume PITR is enabled.

If PITR is unavailable and a reviewed backup is needed, an authorized operator
may run a local `pg_dump` using a protected connection method. Do not put a
password in a shell command, save a dump in the repository, or attach it to a
ticket. Encrypt and retain it only according to the approved backup policy.

Recovery rehearsal rules:

- restore only into a newly created isolated project; never overwrite current
  Staging;
- initially keep wallet intake `disabled` in the restored project;
- after restore, prove migration parity, run linked lint, read-only preflight,
  and the separately authorized E2E before considering activation;
- retain immutable audit and role-assignment event history throughout the
  rehearsal.

## 4. Incident rollback order

Rollback is controlled recovery, not deletion of evidence.

1. First action: use the exact, separately confirmed emergency command to set
   the wallet intake gate to `disabled` and record its audit reference.
2. Use the existing controlled role tool to revoke only affected audited roles;
   record every user identifier, role, change reference, and time outside this
   repository.
3. Roll Cloudflare Pages back to a previously verified Git commit and verify
   its new `/release.json` and headers.
4. Use only a forward-fix Supabase migration after review. Never reverse-edit,
   rewrite, or delete an applied migration.
5. Preserve operations, gate, and role audit events. A rollback must not delete
   evidence or audit records.

No rollback in this document sends a Solana transaction, moves funds, changes a
Program ID, or changes an authority.

## 5. Monitoring and escalation

Schedule the read-only release-inspect probe at an approved interval. It checks
Pages availability, deployed release commit, security headers, Supabase Auth,
RLS probes, and gate mode. A failure requires human escalation; the scheduler
must not change the gate, roles, database, Cloudflare deployment, or chain
state automatically.

For every probe or escalation record: commit, project ref, migration parity,
gate mode, role-list reference, timestamp, outcome, and operator/ticket ID.
Do not record keys, JWTs, CAPTCHA values, wallets, connection strings, or
response bodies.

## 6. Administrator role initialization checklist

The authoritative role names are the nine values in migration `202608110001`:
`reviewer`, `relief_reviewer`, `operator`, `moderator`, `governance_admin`,
`treasury_preparer`, `treasury_authorizer`, `executor`, and
`treasury_reconciler`. `current_operations_role_v1()` fails closed unless a
user has exactly one active, unexpired assignment. Therefore the minimal
initialization plan assigns one distinct Auth user per approved operational
responsibility, never a shared account and never the current real Web3 user.

| Responsibility | Migration/RPC-derived role | Initial status in this phase |
| --- | --- | --- |
| Task/risk review | `reviewer` | Checklist only; no grant executed |
| Relief review | `relief_reviewer` | Checklist only; no grant executed |
| Task and governance proposal operation | `operator` | Checklist only; no grant executed |
| Governance discussion review | `moderator` | Checklist only; no grant executed |
| Governance administration | `governance_admin` | Checklist only; no grant executed |
| Treasury preparation | `treasury_preparer` | Must remain unassigned |
| Treasury authorization | `treasury_authorizer` | Must remain unassigned |
| Receipt reporting | `executor` | Must remain unassigned |
| Treasury reconciliation | `treasury_reconciler` | Must remain unassigned |

The latter four roles remain unassigned until a separately approved real
treasury-execution process exists. Any later role grant requires a distinct
Auth user, an approved change reference, the exact confirmation required by
the existing role tool, post-change inspection, and an off-repository audit
record. Command templates are intentionally non-executable placeholders:

```bash
# Do not run until separately approved; replace bracketed values outside source control.
npm run operations:staging:roles:grant -- [approved-auth-user-id] [approved-role]
npm run operations:staging:roles:inspect -- [approved-auth-user-id]
```

## 7. Launch boundary and records checklist

A Cloudflare frontend Production deployment proves only that static frontend
assets were deployed. It does not prove a Mainnet launch, asset custody,
treasury authority, a Solana program upgrade, transaction capability, or funds
movement. Devnet remains a separate test environment; Mainnet remains
unlaunched unless its independent go/no-go process says otherwise.

Before release and after rollback, complete and retain this record without
secrets:

- [ ] commit and `/release.json` match;
- [ ] project ref and migration parity through `202608110002` recorded;
- [ ] Pages headers and Production deployment status recorded;
- [ ] gate mode and role-list reference recorded;
- [ ] preflight, linked lint, and release-inspect outcomes recorded;
- [ ] backup/PITR inventory or isolated rehearsal result recorded;
- [ ] timestamp, operator, reviewer, and incident/change reference recorded.
