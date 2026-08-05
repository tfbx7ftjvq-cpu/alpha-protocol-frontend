import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import {
  OperationsValidationError,
  validateDiscussion,
  validateGovernanceProposal,
} from '../src/features/operations/domain.ts';

const migration = readFileSync(new URL(
  '../../supabase/migrations/202608070001_governance_operations_audited_execution_preparation.sql',
  import.meta.url,
), 'utf8');
const repository = readFileSync(new URL(
  '../src/features/operations/repository.ts', import.meta.url,
), 'utf8');
const dashboard = readFileSync(new URL(
  '../src/components/OperationsDashboard.tsx', import.meta.url,
), 'utf8');
const validWallet = '11111111111111111111111111111111';

test('governance proposal validation separates private manifest data and requires sha256 binding', () => {
  const result = validateGovernanceProposal({
    title: 'Builder budget process',
    privateSummary: 'Private evidence and operational details for independent proposal review.',
    proposalKind: 'builders_spend',
    executionRequired: true,
    privateExecutionManifest: '{"asset":"USDC","amount":"10"}',
    executionManifestSha256: 'a'.repeat(64),
    publicProposalConsent: true,
    walletAddress: validWallet,
    submissionReference: 'local-domain:proposal-1',
  });
  assert.deepEqual(result.privateExecutionManifest, { asset: 'USDC', amount: '10' });
  assert.equal(result.executionManifestSha256, 'a'.repeat(64));
  assert.throws(() => validateGovernanceProposal({
    ...result,
    privateExecutionManifest: '{}',
    executionManifestSha256: 'bad',
  }), OperationsValidationError);
});

test('discussion publication consent is separate and wallet consent fails closed', () => {
  assert.throws(() => validateDiscussion({
    topic: 'Governance review',
    body: 'A substantive private governance discussion for moderation.',
    walletAddress: validWallet,
    publicBodyConsent: false,
    publicWalletConsent: true,
    submissionReference: 'local-domain:discussion-1',
  }), /公开发言钱包/);
});

test('governance migration exposes only role-controlled audited mutation RPCs', () => {
  for (const rpc of [
    'publish_governance_proposal_v1',
    'review_governance_discussion_v1',
    'finalize_governance_decision_v1',
  ]) {
    assert.match(migration, new RegExp(`create or replace function public\\.${rpc}`));
    assert.match(migration, new RegExp(`grant execute on function public\\.${rpc}`));
  }
  assert.match(migration, /v_actor_role is null/);
  assert.match(migration, /cannot review their own governance proposal/);
  assert.match(migration, /moderators cannot review their own governance discussion/);
  assert.match(migration, /finalizer must be independent/);
  assert.match(migration, /revoke insert, update, delete on table public\.treasury_execution_intents/);
  assert.match(migration, /governance_proposal_submissions_rate_limit/);
});

test('decision finalization deterministically binds sha256 and cannot create execution state', () => {
  const body = migration.match(/create or replace function public\.finalize_governance_decision_v1[\s\S]*?\$\$;/i)?.[0] ?? '';
  assert.match(body, /sha256\(convert_to\(concat_ws/);
  assert.match(body, /execution_intent_created', false/);
  assert.match(body, /execution_receipt_created', false/);
  assert.doesNotMatch(body, /(insert into|update|delete from) public\.treasury_/i);
  assert.match(repository, /p_execution_manifest_sha256/);
  assert.match(dashboard, /批准不等于付款或链上执行/);
});

test('governance staging cleanup is exact, owner-bound, and service-role-only', () => {
  assert.match(migration, /p_owner_id uuid/);
  assert.match(migration, /submitted_by = p_owner_id/);
  assert.match(migration, /phase-2e-6c-staging-e2e:/);
  assert.match(migration, /grant execute on function public\.cleanup_governance_operations_staging_e2e_v1[\s\S]*?to service_role;/);
  assert.match(migration, /revoke all on function public\.cleanup_governance_operations_staging_e2e_v1[\s\S]*?from public, anon, authenticated, service_role;/);
});

test('browser repository contains no service-role key or external sender', () => {
  assert.doesNotMatch(repository, /service[_-]?role[_-]?key/i);
  assert.doesNotMatch(repository, /sendTransaction|fetch\s*\(/);
  assert.match(repository, /submit_governance_proposal_v1/);
  assert.match(repository, /finalize_governance_decision_v1/);
});
