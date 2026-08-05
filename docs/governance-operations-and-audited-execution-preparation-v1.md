# Governance Operations & Audited Execution Preparation v1

Status: Phase 2E-6C local implementation; migration not remotely applied
Baseline: `1b8f00e640d9b20e0d550b57c472c5b6087fb132`

## Boundary

This phase closes the off-chain governance workflow without adding a transaction sender. An `approved` decision is an immutable decision record only. It does not mean paid, submitted, confirmed, or executed, and cannot create a `treasury_execution_intent`, `treasury_execution_receipt`, Solana signature, or external request. Program IDs, Anchor configuration, on-chain programs, authorities, and Mainnet remain out of scope.

## Private and public separation

- `governance_proposal_submissions` is private intake: authenticated owner, wallet, original text, private JSON manifest, consent, and private reviewer notes.
- `governance_proposals` contains only operator-written sanitized text, a public source reference, and an optional read-only manifest URL. It contains no Auth owner ID, raw manifest, or reviewer notes.
- `governance_discussions` is private intake with separate body and wallet publication consent. `governance_discussion_publications` contains moderator-written sanitized content only.
- `operations_governance_workflow_events` is a private append-only audit log, not an execution receipt.

## Roles and independence

| Action | Required role | Independence |
| --- | --- | --- |
| Submit proposal/discussion | verified wallet; intake enabled | owner-bound to `auth.uid()` and verified Solana identity |
| Publish/reject proposal | `operator` or `governance_admin` | cannot review own proposal |
| Publish/reject discussion | `moderator` or `governance_admin` | cannot review own discussion; operator denied |
| Finalize decision | `governance_admin` | cannot be proposal submitter or publisher |

Every RPC rejects a missing user, NULL role, and unlisted role. Direct authenticated writes to governance publication, decision, intent, and receipt tables are revoked.

## Deterministic binding

Execution-requiring private intake stores a lowercase manifest SHA-256. Publication must repeat it and provide an HTTPS manifest reference. Finalization must repeat the same hash. The database—not the caller—computes `decision_hash` as SHA-256 over a canonical newline-separated tuple: version `alpha-governance-decision-v1`, proposal UUID, decision, normalized rationale, execution-required boolean, and manifest hash (or empty string).

`finalize_governance_decision_v1` writes only the decision, one audit event, and proposal status. Its receipt returns `execution_intent_created=false` and `execution_receipt_created=false`; its body contains no treasury-table mutation.

## Risk review

- Self-review and publisher-finalizer overlap fail closed.
- Operator and moderator scopes are distinct.
- Public tables do not contain private-source fields; reviewers provide separate sanitized content.
- Manifest substitution fails exact hash comparison.
- Publication, decision, and workflow events are immutable outside exact cleanup context.
- Browser roles cannot clean up; service role has RPC-only cleanup with reserved reference, exact IDs, owner/content/isolation checks, and count checks.
- Frontend code uses only the public Supabase client and has no service-role key path.

## Migration

Local-only migration: `supabase/migrations/202608070001_governance_operations_audited_execution_preparation.sql`. Do not apply it remotely as part of code review. Review and exercise it against a disposable local database before a separately authorized Staging application.
