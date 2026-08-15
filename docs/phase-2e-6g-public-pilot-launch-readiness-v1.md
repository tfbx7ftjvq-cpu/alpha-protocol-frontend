# Phase 2E-6G Public Pilot Launch Readiness

This phase prepares a controlled Public Pilot candidate. It is not a Mainnet
launch, does not authorize custody or funds movement, and does not enable a
Solana execution path.

## Local controls

- `npm run dependencies:risk:verify` validates the npm 10.9.8, 26-finding risk
  register. It rejects critical findings, direct production high findings, and
  unreviewed changes to the direct production dependency set.
- `npm run operations:roles:bootstrap:validate` validates an offline JSON plan.
  It never grants or revokes roles. Every staff responsibility has a distinct
  UUID Auth user, a future expiry, a change reference, and explicit attestation
  that it is not silently upgrading an ordinary Web3 user. Treasury roles stay
  unassigned for the pilot candidate.
- `npm run operations:recovery:rehearsal:validate` accepts only an
  `isolated_restore` target, rejects current Staging project
  `neevswvhndkalxkainxo`, requires migration parity through `202608110002`,
  seven public/seven private RLS results, disabled intake, and attestations of
  no users, roles, funds, or chain operations.
  The detailed isolation procedure is in
  `phase-2e-6g-isolated-supabase-recovery-rehearsal-v1.md`.
- `npm run operations:pilot:verify` emits JSON plus a human summary. Template
  evidence intentionally reports `NO-GO`; it validates local schema without
  networking or mutations. A human supplies and reviews real evidence outside
  Git before any launch decision.

## External evidence handoff

For a human review only, each real evidence file may be passed explicitly:

```bash
npm run operations:pilot:verify -- \
  --release-evidence [external-release-json] \
  --role-plan [external-role-plan-json] \
  --recovery-evidence [external-recovery-json] \
  --staging-e2e-evidence [external-staging-e2e-json]
```

The equivalent non-browser environment variables are
`OPERATIONS_PILOT_RELEASE_EVIDENCE_PATH`,
`OPERATIONS_PILOT_ROLE_PLAN_PATH`,
`OPERATIONS_PILOT_RECOVERY_EVIDENCE_PATH`, and
`OPERATIONS_PILOT_STAGING_E2E_EVIDENCE_PATH`. Do not prefix them with `VITE_`.
Every explicit file must resolve outside the Git working tree and is rejected
if it contains a secret, JWT, token, email address, wallet private key, or
service-role key. CI supplies none of these paths and therefore always checks
only the templates and reports `NO-GO` without using secrets.

Release evidence must be downloaded `release.json` with a lowercase 40-character
commit SHA, `branch: main`, and `buildContext: cloudflare-pages`. Recovery
evidence retains `gateMode: disabled`. Wallet Staging E2E evidence instead
records `executionGateMode: wallet_staging`,
`postE2EGateMode: wallet_staging`, `cleanupPassed: true`, and
`noChainTransaction: true`.

## Pilot checklist

1. Verify the deployed Pages release commit and 2E-6F release inspect record.
2. Re-run the npm 10.9.8 audit; review every risk-register trigger before
   changing dependencies. Do not use forced audit fixes, React 19, or wallet
   adapter major upgrades for this pilot.
3. Prepare a distinct-staff role plan, obtain separate approval, and perform
   any real grant only through the existing manually confirmed role tool.
4. Complete a recovery rehearsal only in a new isolated Supabase project. Keep
   intake disabled; never restore over Staging.
5. Preserve release, migration, RLS, E2E, gate, Devnet, and Mainnet evidence
   outside source control without secrets or user data.

Public Pilot means a controlled frontend/off-chain pilot boundary. Devnet
operational rehearsal is a separate test activity. Mainnet launch remains a
separate governance, authority, security, and funds-readiness decision.
