import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import {
  OperationsValidationError,
  type TreasuryExecutionPrepareInput,
  isBase58ByteLength,
  validateTreasuryExecutionAuthorize,
  validateTreasuryExecutionPrepare,
  validateTreasuryExecutionReport,
} from '../src/features/operations/domain.ts';

const migration = readFileSync(new URL(
  '../../supabase/migrations/202608080001_audited_treasury_execution_registry_and_reconciliation.sql',
  import.meta.url,
), 'utf8');
const repository = readFileSync(new URL('../src/features/operations/repository.ts', import.meta.url), 'utf8');
const dashboard = readFileSync(new URL('../src/components/OperationsDashboard.tsx', import.meta.url), 'utf8');
const stagingTool = readFileSync(new URL(
  '../scripts/operations-staging/treasury-execution-e2e.ts', import.meta.url,
), 'utf8');

const prepareInput: TreasuryExecutionPrepareInput = {
  governanceDecisionId: '10000000-0000-4000-8000-000000000001',
  pool: 'builders' as const,
  reliefApplicationId: '',
  network: 'devnet' as const,
  assetSymbol: 'USDC',
  assetDecimals: 6,
  assetMint: '11111111111111111111111111111111',
  destinationWallet: '11111111111111111111111111111111',
  amountBaseUnits: '1000000',
  recipientVerificationReference: 'verified recipient record',
  purposeReference: 'approved purpose',
  privateNote: 'preparation note',
  auditReference: 'phase-2e-6d-staging-e2e:1770000000000-a1b2c3d4',
};

test('execution inputs fail closed and validate Solana byte lengths', () => {
  assert.deepEqual(validateTreasuryExecutionPrepare(prepareInput), prepareInput);
  assert.equal(isBase58ByteLength('1'.repeat(32), 32), true);
  assert.equal(isBase58ByteLength('1'.repeat(64), 64), true);
  assert.equal(isBase58ByteLength('0'.repeat(64), 64), false);
  assert.throws(
    () => validateTreasuryExecutionPrepare({ ...prepareInput, network: 'mainnet' as never }),
    OperationsValidationError,
  );
  assert.throws(
    () => validateTreasuryExecutionAuthorize({ executionIntentId: prepareInput.governanceDecisionId, authorizationReference: '', privateNote: 'x', auditReference: '' }),
    OperationsValidationError,
  );
  assert.throws(
    () => validateTreasuryExecutionReport({
      executionIntentId: prepareInput.governanceDecisionId,
      transactionSignature: 'not-base58-0',
      confirmedAt: new Date().toISOString(),
      privateNote: 'external report',
      auditReference: 'external report reference',
    }),
    OperationsValidationError,
  );
});

test('migration exposes explicit role-controlled RPCs without direct writes', () => {
  for (const name of [
    'prepare_treasury_execution_intent_v1',
    'authorize_treasury_execution_intent_v1',
    'cancel_treasury_execution_intent_v1',
    'report_treasury_execution_receipt_v1',
    'reconcile_treasury_execution_v1',
  ]) {
    assert.match(migration, new RegExp(`create or replace function public\\.${name}`));
  }
  assert.match(migration, /treasury_preparer/);
  assert.match(migration, /treasury_authorizer/);
  assert.match(migration, /executor/);
  assert.match(migration, /treasury_reconciler/);
  assert.match(migration, /v_actor_role is null/);
  assert.match(migration, /cannot authorize their own intent/);
  assert.match(migration, /reporter must be independent|report their own|independent from intent preparation and authorization/);
  assert.match(migration, /reconciler must be independent/);
  assert.doesNotMatch(migration, /grant (insert|update|delete)[^;]+treasury_execution_/i);
});

test('decision, intent and receipt bindings are deterministic and approval is not execution', () => {
  assert.match(migration, /decision_hash/);
  assert.match(migration, /execution_manifest_sha256/);
  assert.match(migration, /intent_hash/);
  assert.match(migration, /receipt_hash/);
  assert.match(migration, /governance_decision_id uuid not null unique/i);
  assert.match(migration, /external_execution_reference text unique/i);
  assert.match(migration, /Authorization is not payment and sends no transaction/i);
  assert.match(migration, /database does not query or verify chain finality/i);
  assert.match(repository, /prepare_treasury_execution_intent_v1/);
  assert.doesNotMatch(repository, /sendTransaction|signTransaction/);
  assert.match(dashboard, /授权不等于付款/);
  assert.match(dashboard, /不会签名或发送 Solana 交易/);
});

test('private audit data and sanitized public registry are structurally separated', () => {
  assert.match(migration, /create table public\.treasury_execution_private_notes/);
  assert.match(migration, /create table public\.operations_treasury_execution_workflow_events/);
  assert.match(migration, /create table public\.treasury_execution_public_registry/);
  const publicTable = migration.match(/create table public\.treasury_execution_public_registry \([\s\S]*?\n\);/)?.[0] ?? '';
  assert.doesNotMatch(publicTable, /private_note|actor_id|destination_wallet\s+text/);
  assert.match(publicTable, /destination_wallet_display/);
  assert.match(migration, /operations_treasury_execution_workflow_events_immutable/);
  assert.match(migration, /treasury_execution_public_registry_immutable_delete/);
});

test('history gate and cleanup are explicit, exact, owner-bound and service-role-only', () => {
  assert.match(migration, /read-only pre-application inventory/i);
  assert.match(migration, /refusing unsafe in-place migration/i);
  assert.match(migration, /phase-2e-6d-staging-e2e:\[0-9\]\{13\}-\[0-9a-f\]\{8\}/i);
  assert.match(migration, /cardinality\(p_execution_intent_ids\) <> 2/);
  assert.match(migration, /prepared_by = p_fixture_owner_id/);
  assert.match(migration, /grant execute on function public\.cleanup_treasury_execution_staging_e2e_v1[\s\S]*?to service_role;/);
  assert.match(migration, /revoke all on function public\.cleanup_treasury_execution_staging_e2e_v1[\s\S]*?from public, anon, authenticated, service_role;/);
  assert.match(stagingTool, /approvedDecisionIds: readonly \[string, string\]/);
  assert.match(stagingTool, /p_fixture_owner_id: input\.ownerId/);
  assert.match(stagingTool, /p_execution_intent_ids: \[firstIntentId, secondIntentId\]/);
  assert.doesNotMatch(stagingTool, /Keypair|sendTransaction|signTransaction|fetch\s*\(/);
});
