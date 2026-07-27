import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

const migrationUrl = new URL(
  '../../supabase/migrations/202607270001_offchain_operations_foundation.sql',
  import.meta.url,
);
const sql = readFileSync(migrationUrl, 'utf8');

const expectedTables = [
  'community_tasks',
  'task_submissions',
  'risk_reports',
  'risk_evidence',
  'risk_publications',
  'relief_applications',
  'relief_public_updates',
  'governance_proposals',
  'governance_discussions',
  'governance_discussion_publications',
  'governance_decisions',
  'treasury_execution_intents',
  'treasury_execution_receipts',
];

test('every operations table enables row-level security', () => {
  for (const table of expectedTables) {
    assert.match(
      sql,
      new RegExp(`alter table public\\.${table} enable row level security;`),
      `${table} must enable RLS`,
    );
  }
});

test('private intake tables are never granted to anon', () => {
  for (const table of [
    'task_submissions',
    'risk_reports',
    'risk_evidence',
    'relief_applications',
    'governance_discussions',
    'treasury_execution_intents',
  ]) {
    assert.doesNotMatch(
      sql,
      new RegExp(`grant [^;]+ on table public\\.${table} to anon;`),
      `${table} must not grant anon access`,
    );
  }
});

test('sanitized publication tables contain no auth user foreign key', () => {
  for (const table of [
    'risk_publications',
    'relief_public_updates',
    'governance_discussion_publications',
  ]) {
    const definition = extractCreateTable(table);
    assert.doesNotMatch(definition, /references auth\.users/i);
    assert.doesNotMatch(definition, /submitted_by|created_by|published_by|moderated_by/i);
  }
});

test('publications, governance decisions, and receipts are append-only or immutable', () => {
  for (const trigger of [
    'risk_publications_immutable',
    'relief_public_updates_immutable',
    'governance_discussion_publications_immutable',
    'governance_decisions_immutable',
    'treasury_execution_receipts_immutable',
  ]) {
    assert.match(sql, new RegExp(`create trigger ${trigger}`));
  }

  assert.doesNotMatch(
    sql,
    /grant [^;]*update[^;]* on table public\.governance_decisions/i,
  );
  assert.doesNotMatch(
    sql,
    /grant [^;]*update[^;]* on table public\.treasury_execution_receipts/i,
  );
});

test('execution receipts bind decision, intent, manifest, status, and signature', () => {
  const policy = extractPolicy('treasury_execution_receipts_executor_insert');

  for (const requiredFragment of [
    'intent.id = treasury_execution_receipts.execution_intent_id',
    'intent.governance_decision_id = treasury_execution_receipts.governance_decision_id',
    'intent.manifest_sha256 = treasury_execution_receipts.manifest_sha256',
    'intent.network = treasury_execution_receipts.network',
    "intent.status = 'confirmed'",
    'intent.submitted_signature = treasury_execution_receipts.transaction_signature',
  ]) {
    assert.ok(policy.includes(requiredFragment), `missing receipt binding: ${requiredFragment}`);
  }
});

test('governance execution requires an approved decision and recipient verification reference', () => {
  const intent = extractCreateTable('treasury_execution_intents');
  const insertPolicy = extractPolicy('treasury_execution_intents_executor_insert');

  assert.match(intent, /governance_decision_id uuid not null unique/);
  assert.match(intent, /relief_application_id uuid unique/);
  assert.match(intent, /asset_symbol text not null default 'USDC'/);
  assert.match(intent, /asset_decimals smallint not null default 6/);
  assert.match(intent, /recipient_verification_reference text not null/);
  assert.match(intent, /manifest_sha256 text not null unique/);
  assert.ok(insertPolicy.includes("decision.decision = 'approved'"));
  assert.ok(insertPolicy.includes('decision.execution_required'));
  assert.ok(insertPolicy.includes("application.status = 'approved'"));
  assert.ok(insertPolicy.includes('application.wallet_address = treasury_execution_intents.destination_wallet'));
  assert.ok(insertPolicy.includes('<= application.requested_amount_usdc * 1000000'));
});

test('paid relief state requires a matching confirmed receipt', () => {
  const application = extractCreateTable('relief_applications');

  assert.match(application, /payment_receipt_id uuid/);
  assert.match(sql, /relief_applications_payment_receipt_fk/);
  assert.match(sql, /create trigger relief_applications_validate_payment_receipt/);
  assert.match(sql, /intent\.relief_application_id = new\.id/);
  assert.match(sql, /intent\.destination_wallet = new\.wallet_address/);
  assert.match(sql, /intent\.status = 'confirmed'/);
});

test('review and execution state machines reject unlisted transitions', () => {
  for (const trigger of [
    'community_tasks_enforce_status_transition',
    'task_submissions_enforce_status_transition',
    'risk_reports_enforce_status_transition',
    'relief_applications_enforce_status_transition',
    'governance_proposals_enforce_status_transition',
    'governance_discussions_enforce_status_transition',
    'treasury_execution_intents_enforce_status_transition',
  ]) {
    assert.match(sql, new RegExp(`create trigger ${trigger}`));
  }

  assert.match(sql, /create trigger treasury_execution_intents_lock_signature/);
});

test('V1 intake wallets cannot be marked verified without a later migration', () => {
  for (const table of [
    'task_submissions',
    'risk_reports',
    'relief_applications',
    'governance_discussions',
    'governance_discussion_publications',
  ]) {
    assert.match(
      extractCreateTable(table),
      /wallet_verified boolean not null default false check \(wallet_verified = false\)/,
    );
  }
});

test('migration contains no database network or secret-vault integration', () => {
  assert.doesNotMatch(sql, /\bhttp_post\b|\bnet\.http\b|\bvault\.secrets\b|\bpg_net\b/i);
  assert.doesNotMatch(sql, /\bsend_transaction\b|\bsendtransaction\b/i);
});

function extractCreateTable(table: string): string {
  const match = sql.match(
    new RegExp(`create table public\\.${table} \\([\\s\\S]*?\\n\\);`),
  );
  assert.ok(match, `missing create table statement for ${table}`);
  return match[0];
}

function extractPolicy(policy: string): string {
  const match = sql.match(
    new RegExp(`create policy ${policy}[\\s\\S]*?;`),
  );
  assert.ok(match, `missing policy ${policy}`);
  return match[0];
}
