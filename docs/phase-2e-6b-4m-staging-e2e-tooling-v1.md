# Alpha Protocol Phase 2E-6B-4M Staging E2E Tooling V1

Status: local implementation and validation only; cleanup migration and remote E2E not deployed
Baseline commit: `73441a640413414feab0355583b2d22e770fdf07`
Phase: `2E-6B-4M`
Date: `2026-08-04`

## 1. Purpose

Commit `73441a6` introduced the audited community-task workflow and was
reported successfully deployed by Cloudflare Production. Supabase Staging
migration `202608030001` was also applied with migration parity, linked lint,
and the read-only preflight passing.

The pre-existing actor-level runner could no longer verify that workflow. It
still attempted direct staff writes to `community_tasks`, while Phase 4M
deliberately revoked those writes in favor of audited RPCs. Its old cleanup
also could not remove the new immutable public result and workflow-event rows.

This follow-up makes the Staging E2E match the deployed authorization model.
It does not alter the product workflow, enable a treasury action, or send a
Solana transaction.

## 2. Controlled workflow exercised

One confirmed run creates four temporary Auth actors:

- an operator;
- a reviewer;
- one ephemeral Solana Web3 wallet owner;
- one email-only negative-control user.

Only the Web3 actor consumes the fresh human-completed Turnstile response.
Email-based staff and negative-control actors use admin-generated one-time
magic links, avoiding CAPTCHA bypasses or persistent test passwords.

The runner then performs 31 assertions covering:

1. the independent wallet-intake gate is enabled;
2. a no-role user cannot publish a task;
3. direct operator task insertion is denied;
4. the operator publishes through `publish_community_task_v1`;
5. the public task is readable to `anon`;
6. private submissions are not readable to `anon`;
7. the verified wallet owner creates the accepted-path submission;
8. the same owner creates the rejected-path submission;
9. a switched wallet is rejected;
10. an email-only identity is rejected from wallet-bound intake;
11. the owner can read both private submissions;
12. another user cannot read them;
13. the owner cannot assert `wallet_verified=true`;
14. a newly assigned reviewer claim is present only after session refresh;
15. a no-role user cannot review;
16. the submitting wallet cannot self-review;
17. the independent reviewer accepts the consented submission;
18. terminal review replay is rejected;
19. the second submission is rejected;
20. rejection creates no public result;
21. the accepted public result is sanitized and omits private identifiers and
    the unconsented wallet;
22. `anon` cannot read private workflow audit events;
23. staff reads exactly the four expected immutable workflow events;
24. staff cannot rewrite the public result;
25. a direct service-role table delete cannot bypass controlled cleanup;
26. the operator cannot directly rewrite the task;
27. the reviewer cannot directly rewrite the submission;
28. the wallet owner creates one private governance discussion;
29. the operator can read the private moderation queue;
30. an unprivileged user cannot moderate it;
31. the operator can complete the authorized moderation transition.

Acceptance remains an off-chain review result. The E2E creates no payment
intent, transaction signature, treasury receipt, USDC movement, or SOL
movement.

## 3. Exact cleanup boundary

Migration `202608040001_operations_task_staging_e2e_cleanup.sql` adds one
`SECURITY DEFINER` RPC:

```text
cleanup_operations_task_staging_e2e_v1(text, uuid, uuid[])
```

Execution is granted only to `service_role`. The function accepts only a run
reference shaped as:

```text
phase-2e-6b-4m-staging-e2e:<13-digit timestamp>-<8 lowercase hex chars>
```

Before deleting anything, it proves that all supplied ids refer to the exact
reserved test task, the two exact example.com submissions, the exact sanitized
publication, and only the expected workflow events. It rejects unrelated rows,
duplicate ids, prefix collisions, unexpected content, and count mismatches.

The immutable publication/event trigger exception requires both:

- the validated transaction-local cleanup reference; and
- `current_user` equal to the dynamically resolved owner of the cleanup RPC.

Knowing the custom setting is therefore insufficient to bypass immutability.
The migration also revokes the obsolete Phase 4H direct service-role DELETE
grants on `community_tasks` and `task_submissions`. Direct task-workflow
`DELETE` therefore remains unavailable to `anon`, `authenticated`, and
`service_role` clients. On the complete path, the RPC must return exactly:

```text
1 task publication + 4 audit events + 2 submissions + 1 task = 8 rows
```

The legacy governance-discussion fixture is separately deleted with exact-id
proof, so the full runner reports:

```text
Cleanup: 9 rows, 4 users
```

Cleanup runs after both success and failure. If exact validation prevents a
partial cleanup, the runner exits non-zero and reports the residue category;
it never broadens the target or silently counts a missing row as deleted.

## 4. Deployment and execution gates

This local phase does not apply the migration or run remote E2E. The required
future sequence is:

1. review and commit this tooling change;
2. confirm Cloudflare Production is at `73441a6` or the reviewed descendant;
3. verify the linked Supabase project ref is exactly the dedicated Staging
   project;
4. inspect `migration list`, `db push --dry-run`, and linked schema lint;
5. explicitly confirm and apply only migration `202608040001`;
6. verify migration parity and rerun `operations:staging:preflight`;
7. complete a fresh Turnstile challenge on the reviewed Pages origin;
8. place the single-use response in the current process only;
9. explicitly confirm the mutating Staging E2E and run it once;
10. require 31 assertions, nine deleted rows, four deleted users, and exit zero;
11. inspect for residue before declaring the Staging workflow closed.

The CAPTCHA response and service-role credential must never enter Git,
dotenv examples, browser variables, screenshots, logs, or documentation.

## 5. Verification scope

Local verification completed on the baseline worktree:

- all operations Node tests, including PGlite execution of every migration;
- runtime role, RLS, immutable-record, exact-fixture, and cleanup-count tests;
- operations tooling TypeScript;
- frontend TypeScript;
- ESLint;
- production Vite build;
- `git diff --check`;
- scans for browser cleanup grants, network senders, Solana transaction calls,
  treasury mutations, private keys, and secret literals.

Exact results:

```text
operations tests: 88 passed, 0 failed
operations tooling TypeScript: passed
frontend TypeScript: passed
ESLint: passed
production Vite build: passed (1829 modules transformed)
git diff --check: passed
```

The Vite build retains the existing advisory that the main minified chunk is
larger than 500 kB. This is a performance warning, not a failed verification
or an authorization-boundary change.

The remote Supabase pgTAP file also verifies the cleanup RPC grant boundary and
the owner-bound immutable triggers. Running that file in the dashboard is not
a substitute for the actor-level E2E.

## 6. Status boundary

- implemented locally: RPC-based task workflow E2E, negative authorization
  paths, exact audit assertions, strict atomic cleanup, tests, and docs;
- locally verified: 88 operations tests, both TypeScript checks, ESLint, and
  the production Vite build passed;
- deployed before this follow-up: Cloudflare Production commit `73441a6` and
  Supabase Staging migration `202608030001`;
- not deployed: migration `202608040001` and this tooling revision;
- not executed: Phase 4M task-workflow Staging E2E;
- Devnet: unchanged; no program deploy, upgrade, initialization, or
  transaction;
- Mainnet: not entered; no program deploy or transaction;
- treasury: no authority change, intent, signature, receipt, or funds movement;
- Git: no commit or push in this local phase.

This is not a Mainnet professional audit or legal review.
