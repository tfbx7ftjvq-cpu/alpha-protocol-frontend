import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { PGlite } from '@electric-sql/pglite';

const foundationMigrationUrl = new URL(
  '../../supabase/migrations/202607270001_offchain_operations_foundation.sql',
  import.meta.url,
);
const hardeningMigrationUrl = new URL(
  '../../supabase/migrations/202607270002_operations_staging_hardening.sql',
  import.meta.url,
);
const cleanupPrivilegesMigrationUrl = new URL(
  '../../supabase/migrations/202607290001_operations_staging_e2e_cleanup_privileges.sql',
  import.meta.url,
);
const walletIntakeMigrationUrl = new URL(
  '../../supabase/migrations/202607300001_wallet_authenticated_operations_intake.sql',
  import.meta.url,
);
const identityCompatibilityMigrationUrl = new URL(
  '../../supabase/migrations/202607310001_web3_solana_identity_subject_compatibility.sql',
  import.meta.url,
);
const walletResolverLintCleanupMigrationUrl = new URL(
  '../../supabase/migrations/202608020001_web3_solana_wallet_resolver_lint_cleanup.sql',
  import.meta.url,
);
const intakeGateAuditMigrationUrl = new URL(
  '../../supabase/migrations/202608020002_operations_wallet_intake_gate_audit.sql',
  import.meta.url,
);
const taskModerationClosureMigrationUrl = new URL(
  '../../supabase/migrations/202608030001_operations_task_moderation_closure.sql',
  import.meta.url,
);
const taskStagingE2ECleanupMigrationUrl = new URL(
  '../../supabase/migrations/202608040001_operations_task_staging_e2e_cleanup.sql',
  import.meta.url,
);
const foundationSql = readFileSync(foundationMigrationUrl, 'utf8');
const hardeningSql = readFileSync(hardeningMigrationUrl, 'utf8');
const cleanupPrivilegesSql = readFileSync(cleanupPrivilegesMigrationUrl, 'utf8');
const walletIntakeSql = readFileSync(walletIntakeMigrationUrl, 'utf8');
const identityCompatibilitySql = readFileSync(identityCompatibilityMigrationUrl, 'utf8');
const walletResolverLintCleanupSql = readFileSync(
  walletResolverLintCleanupMigrationUrl,
  'utf8',
);
const intakeGateAuditSql = readFileSync(intakeGateAuditMigrationUrl, 'utf8');
const taskModerationClosureSql = readFileSync(taskModerationClosureMigrationUrl, 'utf8');
const taskStagingE2ECleanupSql = readFileSync(
  taskStagingE2ECleanupMigrationUrl,
  'utf8',
);
const sql = [
  foundationSql,
  hardeningSql,
  cleanupPrivilegesSql,
  walletIntakeSql,
  identityCompatibilitySql,
  walletResolverLintCleanupSql,
  intakeGateAuditSql,
  taskModerationClosureSql,
  taskStagingE2ECleanupSql,
].join('\n');

const expectedTables = [
  'operations_intake_control',
  'operations_intake_gate_events',
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
  'task_submission_publications',
  'operations_task_workflow_events',
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
    'operations_intake_control',
    'operations_intake_gate_events',
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
    'task_submission_publications',
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
    'task_submission_publications_immutable',
    'operations_task_workflow_events_immutable',
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

test('staging hardening prevents publication downgrade and gives moderators read scope', () => {
  assert.match(
    hardeningSql,
    /new\.publication_status is distinct from old\.publication_status/,
  );
  assert.match(
    hardeningSql,
    /published record on % cannot be unpublished/,
  );

  const policy = extractPolicy(
    'governance_discussions_moderator_read',
    hardeningSql,
  );
  assert.match(policy, /for select/);
  assert.match(policy, /'moderator', 'operator', 'governance_admin'/);
});

test('wallet intake binds all owner inserts to a verified Solana Web3 identity', () => {
  assert.match(
    walletIntakeSql,
    /mode text not null default 'disabled'/,
  );
  assert.match(
    walletIntakeSql,
    /Applying this migration leaves wallet intake disabled/,
  );
  assert.match(
    walletIntakeSql,
    /from auth\.identities identity/,
  );
  assert.match(
    walletIntakeSql,
    /identity\.provider = 'web3'/,
  );

  for (const policyName of [
    'task_submissions_owner_insert',
    'risk_reports_owner_insert',
    'relief_applications_owner_insert',
    'governance_discussions_owner_insert',
  ]) {
    const policy = extractPolicy(policyName, walletIntakeSql);
    assert.match(
      policy,
      /wallet_address = public\.current_verified_solana_wallet\(\)/,
      `${policyName} must bind the submitted wallet to the Auth identity`,
    );
    assert.match(
      policy,
      /public\.is_operations_wallet_intake_enabled\(\)/,
      `${policyName} must respect the database-side intake gate`,
    );
  }
});

test('wallet identity compatibility matches the observed Supabase provider subject', () => {
  assert.match(
    identityCompatibilitySql,
    /min\(identity\.provider_id\)/,
  );
  assert.match(
    identityCompatibilitySql,
    /min\(identity\.identity_data ->> 'sub'\)/,
  );
  assert.match(
    identityCompatibilitySql,
    /provider_identifier is distinct from identity_subject/,
  );
  assert.match(
    identityCompatibilitySql,
    /identity_prefix constant text := 'web3:solana:';/,
  );
  assert.match(
    identityCompatibilitySql,
    /wallet_address !~ '\^\[1-9A-HJ-NP-Za-km-z\]\+\$'/,
  );
  assert.match(
    identityCompatibilitySql,
    /leading_zero_bytes \+ non_zero_bytes <> 32/,
  );
  assert.doesNotMatch(
    identityCompatibilitySql,
    /update public\.operations_intake_control/,
  );
});

test('wallet resolver lint cleanup preserves identity checks without a shadowed declaration', () => {
  for (const requiredFragment of [
    /create or replace function public\.current_verified_solana_wallet\(\)/,
    /security definer/,
    /set search_path = ''/,
    /provider_identifier is distinct from identity_subject/,
    /identity_prefix constant text := 'web3:solana:';/,
    /for character_index in 1\.\.char_length\(wallet_address\) loop/,
    /leading_zero_bytes \+ non_zero_bytes <> 32/,
    /grant execute on function public\.current_verified_solana_wallet\(\) to authenticated/,
  ]) {
    assert.match(walletResolverLintCleanupSql, requiredFragment);
  }

  assert.doesNotMatch(
    walletResolverLintCleanupSql,
    /character_index integer;/,
  );
  assert.doesNotMatch(
    walletResolverLintCleanupSql,
    /update public\.operations_intake_control/,
  );
});

test('wallet intake adds database-side rate limits to every direct intake table', () => {
  for (const trigger of [
    'task_submissions_rate_limit',
    'risk_reports_rate_limit',
    'relief_applications_rate_limit',
    'governance_discussions_rate_limit',
  ]) {
    assert.match(walletIntakeSql, new RegExp(`create trigger ${trigger}`));
  }

  assert.match(walletIntakeSql, /pg_advisory_xact_lock/);
  assert.match(walletIntakeSql, /new\.created_at := now\(\)/);
  assert.match(walletIntakeSql, /new\.updated_at := now\(\)/);
  assert.match(walletIntakeSql, /relief_applications' then[\s\S]*interval '24 hours'/);
});

test('intake gate changes use a service-role-only RPC and append-only audit history', () => {
  const auditTable = extractCreateTable('operations_intake_gate_events');
  assert.match(auditTable, /previous_mode text not null/);
  assert.match(auditTable, /new_mode text not null/);
  assert.match(auditTable, /change_reference text not null/);
  assert.match(auditTable, /change_reference !~ '\[\[:cntrl:\]\]'/);
  assert.match(auditTable, /previous_mode <> new_mode/);
  assert.match(
    intakeGateAuditSql,
    /create trigger operations_intake_gate_events_immutable/,
  );
  assert.match(
    intakeGateAuditSql,
    /grant execute on function public\.set_operations_wallet_intake_mode\(text, text\) to service_role/,
  );
  assert.match(
    intakeGateAuditSql,
    /revoke all on table public\.operations_intake_control from service_role/,
  );
  assert.doesNotMatch(
    intakeGateAuditSql,
    /grant (?:insert|update|delete)[^;]*operations_intake_gate_events/i,
  );
});

test('public task results exclude private submission and Auth user identifiers', () => {
  const publication = extractCreateTable('task_submission_publications');

  assert.match(publication, /task_id uuid not null references public\.community_tasks\(id\)/);
  assert.match(publication, /result_summary text not null/);
  assert.match(publication, /wallet_address text check/);
  assert.doesNotMatch(publication, /references auth\.users/i);
  assert.doesNotMatch(publication, /submission_id|submitted_by|reviewed_by|actor_id/i);
});

test('task publications and private workflow events are immutable and separately readable', () => {
  assert.match(taskModerationClosureSql, /create trigger task_submission_publications_immutable/);
  assert.match(taskModerationClosureSql, /create trigger operations_task_workflow_events_immutable/);
  assert.match(
    extractPolicy('task_submission_publications_public_read', taskModerationClosureSql),
    /to anon, authenticated[\s\S]*using \(true\)/,
  );
  assert.match(
    extractPolicy('operations_task_workflow_events_staff_read', taskModerationClosureSql),
    /'reviewer', 'operator', 'governance_admin'/,
  );
  assert.doesNotMatch(
    taskModerationClosureSql,
    /grant select on table public\.operations_task_workflow_events to anon/,
  );
});

test('task workflow RPCs are denied to anon and granted only to authenticated sessions', () => {
  for (const signature of [
    'public\\.publish_community_task_v1\\(text, text, text, numeric, text, timestamptz, text\\)',
    'public\\.review_task_submission_v1\\(uuid, text, text, text, text, text\\)',
  ]) {
    assert.match(taskModerationClosureSql, new RegExp(`revoke all on function ${signature} from anon;`));
    assert.match(taskModerationClosureSql, new RegExp(`grant execute on function ${signature} to authenticated;`));
    assert.doesNotMatch(taskModerationClosureSql, new RegExp(`grant execute on function ${signature} to anon;`));
  }
});

test('task workflow RPC role checks explicitly reject missing users and NULL roles', () => {
  const explicitRoleCheck = /if v_actor_id is null\s+or v_actor_role is null\s+or v_actor_role not in/g;
  assert.equal([...taskModerationClosureSql.matchAll(explicitRoleCheck)].length, 2);
  assert.match(
    taskModerationClosureSql,
    /NULL NOT IN \(\.\.\.\) evaluates to NULL and can[\s\S]*security-critical/,
  );
});

test('direct staff mutations are revoked so task workflow changes must use RPCs', () => {
  assert.match(
    taskModerationClosureSql,
    /revoke insert, update, delete on table public\.community_tasks from authenticated;/,
  );
  assert.match(
    taskModerationClosureSql,
    /revoke update, delete on table public\.task_submissions from authenticated;/,
  );
  assert.doesNotMatch(
    taskModerationClosureSql,
    /grant (?:insert|update|delete)[^;]*operations_task_workflow_events/i,
  );
});

test('accepted task review requires consent and rejection cannot publish a result', () => {
  assert.match(
    taskModerationClosureSql,
    /if not v_submission\.public_result_consent then[\s\S]*requires contributor public result consent/,
  );
  assert.match(
    taskModerationClosureSql,
    /rejected task submissions cannot publish a public result/,
  );
  assert.match(
    taskModerationClosureSql,
    /case when v_submission\.public_wallet_consent then v_submission\.wallet_address else null end/,
  );
  assert.match(taskModerationClosureSql, /accepted never means paid/);
});

test('task moderation closure contains no network sender or treasury mutation', () => {
  assert.doesNotMatch(
    taskModerationClosureSql,
    /\bhttp_post\b|\bnet\.http\b|\bvault\.secrets\b|\bpg_net\b|\bsend_transaction\b|\bsendtransaction\b/i,
  );
  assert.doesNotMatch(
    taskModerationClosureSql,
    /\b(?:insert into|update|delete from)\s+public\.treasury_/i,
  );
});

test('Phase 4M Staging cleanup is a narrow service-role RPC, not table access', () => {
  assert.match(
    taskStagingE2ECleanupSql,
    /create or replace function public\.cleanup_operations_task_staging_e2e_v1\(/,
  );
  assert.match(taskStagingE2ECleanupSql, /security definer/);
  assert.match(
    taskStagingE2ECleanupSql,
    /\^phase-2e-6b-4m-staging-e2e:\[0-9\]\{13\}-\[0-9a-f\]\{8\}\$/,
  );
  assert.match(
    taskStagingE2ECleanupSql,
    /pg_catalog\.pg_get_userbyid\(procedure\.proowner\)/,
  );
  assert.match(
    taskStagingE2ECleanupSql,
    /current_user = cleanup_owner/,
  );
  assert.match(
    taskStagingE2ECleanupSql,
    /revoke all on function public\.cleanup_operations_task_staging_e2e_v1\(text, uuid, uuid\[\]\)[\s\S]*from public, anon, authenticated, service_role;/,
  );
  assert.match(
    taskStagingE2ECleanupSql,
    /grant execute on function public\.cleanup_operations_task_staging_e2e_v1\(text, uuid, uuid\[\]\)[\s\S]*to service_role;/,
  );
  assert.doesNotMatch(
    taskStagingE2ECleanupSql,
    /grant (?:insert|update|delete)[^;]* on table/i,
  );
  assert.match(
    taskStagingE2ECleanupSql,
    /revoke delete on table[\s\S]*public\.task_submissions,[\s\S]*public\.community_tasks[\s\S]*from service_role;/,
  );
  assert.doesNotMatch(
    taskStagingE2ECleanupSql,
    /\bhttp_post\b|\bnet\.http\b|\bvault\.secrets\b|\bpg_net\b|\bsend_transaction\b|\bsendtransaction\b/i,
  );
  assert.doesNotMatch(
    taskStagingE2ECleanupSql,
    /\b(?:insert into|update|delete from)\s+public\.treasury_/i,
  );
});

test('staging E2E cleanup privilege is narrow and excludes browser roles', async () => {
  assert.match(
    cleanupPrivilegesSql,
    /grant select, delete on table[\s\S]*?to service_role;/,
  );
  assert.match(
    cleanupPrivilegesSql,
    /revoke delete on table[\s\S]*?from anon, authenticated;/,
  );

  const database = await createOperationsDatabase();
  try {
    const serviceRolePrivileges = await database.query<{
      table_name: string;
      privilege_type: string;
    }>(`
      select table_name, privilege_type
      from information_schema.role_table_grants
      where table_schema = 'public'
        and grantee = 'service_role'
        and table_name = any(array[
          'community_tasks',
          'task_submissions',
          'governance_discussions'
        ])
        and privilege_type in ('SELECT', 'DELETE')
      order by table_name, privilege_type;
    `);
    assert.deepEqual(serviceRolePrivileges.rows, [
      { table_name: 'community_tasks', privilege_type: 'SELECT' },
      { table_name: 'governance_discussions', privilege_type: 'DELETE' },
      { table_name: 'governance_discussions', privilege_type: 'SELECT' },
      { table_name: 'task_submissions', privilege_type: 'SELECT' },
    ]);

    const browserDeletePrivileges = await database.query<{ count: number }>(`
      select count(*)::integer as count
      from information_schema.role_table_grants
      where table_schema = 'public'
        and grantee in ('anon', 'authenticated')
        and table_name = any(array[
          'community_tasks',
          'task_submissions',
          'governance_discussions'
        ])
        and privilege_type = 'DELETE';
    `);
    assert.equal(browserDeletePrivileges.rows[0]?.count, 0);
  } finally {
    await database.close();
  }
});

test('runtime task RPCs reject authenticated users without an operations role', async () => {
  const database = await createOperationsDatabase();
  const userId = '88888888-8888-4888-8888-888888888800';

  try {
    await database.exec(`
      insert into auth.users (id) values ('${userId}');
      select set_config('request.jwt.claim.sub', '${userId}', false);
      select set_config(
        'request.jwt.claims',
        '{"sub":"${userId}","role":"authenticated"}',
        false
      );
      set role authenticated;
    `);

    await assert.rejects(
      database.query(`
        select public.publish_community_task_v1(
          'Unauthorized task',
          'This task must not be published by an authenticated user without an operations role.',
          'The runtime test expects this function call to fail before any task row is created.',
          0,
          'none',
          null,
          'phase-4m-no-role-publish'
        );
      `),
      /operations role is not authorized to publish tasks/,
    );

    await assert.rejects(
      database.query(`
        select * from public.review_task_submission_v1(
          '88888888-8888-4888-8888-888888888899',
          'rejected',
          'Unauthorized review attempt.',
          null,
          null,
          'phase-4m-no-role-review'
        );
      `),
      /operations role is not authorized to review task submissions/,
    );
  } finally {
    await database.close();
  }
});

test('runtime operator publishes a public task and one immutable audit event atomically', async () => {
  const database = await createOperationsDatabase();
  const operatorId = '88888888-8888-4888-8888-888888888801';

  try {
    await database.exec(`
      insert into auth.users (id) values ('${operatorId}');
      select set_config('request.jwt.claim.sub', '${operatorId}', false);
      select set_config(
        'request.jwt.claims',
        '{"sub":"${operatorId}","role":"authenticated","app_metadata":{"operations_role":"operator"}}',
        false
      );
      set role authenticated;
    `);

    const published = await database.query<{ task_id: string }>(`
      select public.publish_community_task_v1(
        'Audited runtime task',
        'A public task summary created through the role-gated atomic publication function.',
        'Contributors must provide a substantive result and a safe HTTPS deliverable reference.',
        25.125,
        'builders_pool',
        null,
        'phase-4m-runtime-publish-001'
      ) as task_id;
    `);
    const taskId = published.rows[0]?.task_id;
    assert.ok(taskId);

    const task = await database.query<{
      status: string;
      publication_status: string;
      reward_source: string;
    }>(`
      select status, publication_status, reward_source
      from public.community_tasks
      where id = '${taskId}';
    `);
    assert.deepEqual(task.rows, [{
      status: 'open',
      publication_status: 'published',
      reward_source: 'builders_pool',
    }]);

    const events = await database.query<{
      action: string;
      actor_role: string;
      event_reference: string;
    }>(`
      select action, actor_role, event_reference
      from public.operations_task_workflow_events
      where entity_reference = '${taskId}';
    `);
    assert.deepEqual(events.rows, [{
      action: 'task_published',
      actor_role: 'operator',
      event_reference: 'phase-4m-runtime-publish-001',
    }]);

    await assert.rejects(
      database.exec(`
        insert into public.community_tasks (title, summary, requirements)
        values (
          'Direct task write',
          'This direct authenticated insert must remain unavailable after migration.',
          'The publication RPC is the only supported task creation path for staff.'
        );
      `),
      /permission denied/,
    );
  } finally {
    await database.close();
  }
});

test('runtime accepted review publishes only the consented sanitized result and blocks replay', async () => {
  const database = await createOperationsDatabase();
  const operatorId = '88888888-8888-4888-8888-888888888802';
  const ownerId = '88888888-8888-4888-8888-888888888803';
  const reviewerId = '88888888-8888-4888-8888-888888888804';
  const verifiedWallet = '11111111111111111111111111111111';

  try {
    await database.exec(`
      insert into auth.users (id) values
        ('${operatorId}'),
        ('${ownerId}'),
        ('${reviewerId}');
      insert into auth.identities (id, user_id, provider, provider_id, identity_data)
      values (
        'identity-phase-4m-accepted-owner',
        '${ownerId}',
        'web3',
        'web3:solana:${verifiedWallet}',
        '{"sub":"web3:solana:${verifiedWallet}"}'
      );
      update public.operations_intake_control
      set mode = 'wallet_staging', activation_reference = 'phase-4m accepted runtime fixture';
      select set_config('request.jwt.claim.sub', '${operatorId}', false);
      select set_config(
        'request.jwt.claims',
        '{"sub":"${operatorId}","role":"authenticated","app_metadata":{"operations_role":"operator"}}',
        false
      );
      set role authenticated;
    `);

    const task = await database.query<{ task_id: string }>(`
      select public.publish_community_task_v1(
        'Consent-controlled task',
        'This task is used to verify consent-controlled sanitized result publication.',
        'The accepted result must omit the private submission id, user id, and wallet by default.',
        0,
        'none',
        null,
        'phase-4m-accepted-task'
      ) as task_id;
    `);
    const taskId = task.rows[0]?.task_id;
    assert.ok(taskId);

    await database.exec(`
      reset role;
      select set_config('request.jwt.claim.sub', '${ownerId}', false);
      select set_config(
        'request.jwt.claims',
        '{"sub":"${ownerId}","role":"authenticated"}',
        false
      );
      set role authenticated;
    `);
    const submitted = await database.query<{ submission_id: string }>(`
      insert into public.task_submissions (
        task_id,
        submitted_by,
        summary,
        deliverable_url,
        wallet_address,
        public_result_consent,
        public_wallet_consent
      ) values (
        '${taskId}',
        '${ownerId}',
        'Private contributor details that must never be copied automatically into the public result.',
        'https://private.example.com/phase-4m-accepted',
        '${verifiedWallet}',
        true,
        false
      )
      returning id as submission_id;
    `);
    const submissionId = submitted.rows[0]?.submission_id;
    assert.ok(submissionId);

    await database.exec(`
      reset role;
      select set_config('request.jwt.claim.sub', '${reviewerId}', false);
      select set_config(
        'request.jwt.claims',
        '{"sub":"${reviewerId}","role":"authenticated","app_metadata":{"operations_role":"reviewer"}}',
        false
      );
      set role authenticated;
    `);
    const reviewed = await database.query<{
      submission_id: string;
      submission_status: string;
      publication_id: string;
    }>(`
      select * from public.review_task_submission_v1(
        '${submissionId}',
        'accepted',
        'The evidence was reviewed and the sanitized public description is appropriate.',
        'A sanitized accepted result that contains no private contributor identity or payment promise.',
        'https://public.example.com/phase-4m-accepted',
        'phase-4m-accepted-review'
      );
    `);
    assert.equal(reviewed.rows[0]?.submission_status, 'accepted');
    assert.ok(reviewed.rows[0]?.publication_id);

    const publication = await database.query<{
      result_summary: string;
      deliverable_url: string;
      wallet_address: string | null;
      review_reference: string;
    }>(`
      select result_summary, deliverable_url, wallet_address, review_reference
      from public.task_submission_publications
      where id = '${reviewed.rows[0]?.publication_id}';
    `);
    assert.deepEqual(publication.rows, [{
      result_summary: 'A sanitized accepted result that contains no private contributor identity or payment promise.',
      deliverable_url: 'https://public.example.com/phase-4m-accepted',
      wallet_address: null,
      review_reference: 'phase-4m-accepted-review',
    }]);

    const state = await database.query<{ status: string; event_count: number }>(`
      select
        submission.status,
        (
          select count(*)::integer
          from public.operations_task_workflow_events event
          where event.entity_reference = submission.id
        ) as event_count
      from public.task_submissions submission
      where submission.id = '${submissionId}';
    `);
    assert.deepEqual(state.rows, [{ status: 'accepted', event_count: 2 }]);

    await assert.rejects(
      database.query(`
        select * from public.review_task_submission_v1(
          '${submissionId}',
          'accepted',
          'Replay attempt.',
          'A replayed public summary that must never be published a second time.',
          'https://public.example.com/replay',
          'phase-4m-replay-review'
        );
      `),
      /already in a terminal review state/,
    );

    await database.exec('reset role;');
    await assert.rejects(
      database.exec(`
        update public.task_submission_publications
        set result_summary = 'Mutated result summary that must be rejected.'
        where id = '${reviewed.rows[0]?.publication_id}';
      `),
      /immutable operations record/,
    );
  } finally {
    await database.close();
  }
});

test('runtime self-review is denied and rejection creates no public result', async () => {
  const database = await createOperationsDatabase();
  const operatorId = '88888888-8888-4888-8888-888888888805';
  const ownerId = '88888888-8888-4888-8888-888888888806';
  const reviewerId = '88888888-8888-4888-8888-888888888807';
  const verifiedWallet = '11111111111111111111111111111111';

  try {
    await database.exec(`
      insert into auth.users (id) values
        ('${operatorId}'),
        ('${ownerId}'),
        ('${reviewerId}');
      insert into auth.identities (id, user_id, provider, provider_id, identity_data)
      values (
        'identity-phase-4m-rejected-owner',
        '${ownerId}',
        'web3',
        'web3:solana:${verifiedWallet}',
        '{"sub":"web3:solana:${verifiedWallet}"}'
      );
      update public.operations_intake_control
      set mode = 'wallet_staging', activation_reference = 'phase-4m rejected runtime fixture';
      select set_config('request.jwt.claim.sub', '${operatorId}', false);
      select set_config(
        'request.jwt.claims',
        '{"sub":"${operatorId}","role":"authenticated","app_metadata":{"operations_role":"operator"}}',
        false
      );
      set role authenticated;
    `);

    const task = await database.query<{ task_id: string }>(`
      select public.publish_community_task_v1(
        'Rejected result task',
        'This task is used to verify self-review denial and the rejected review path.',
        'A rejected submission must remain private and must never create a public task result.',
        null,
        'none',
        null,
        'phase-4m-rejected-task'
      ) as task_id;
    `);
    const taskId = task.rows[0]?.task_id;
    assert.ok(taskId);

    await database.exec(`
      reset role;
      select set_config('request.jwt.claim.sub', '${ownerId}', false);
      select set_config(
        'request.jwt.claims',
        '{"sub":"${ownerId}","role":"authenticated","app_metadata":{"operations_role":"reviewer"}}',
        false
      );
      set role authenticated;
    `);
    const submitted = await database.query<{ submission_id: string }>(`
      insert into public.task_submissions (
        task_id,
        submitted_by,
        summary,
        deliverable_url,
        wallet_address,
        public_result_consent,
        public_wallet_consent
      ) values (
        '${taskId}',
        '${ownerId}',
        'A private rejected result that is intentionally not consented for public publication.',
        'https://private.example.com/phase-4m-rejected',
        '${verifiedWallet}',
        false,
        false
      )
      returning id as submission_id;
    `);
    const submissionId = submitted.rows[0]?.submission_id;
    assert.ok(submissionId);

    await assert.rejects(
      database.query(`
        select * from public.review_task_submission_v1(
          '${submissionId}',
          'rejected',
          'A contributor cannot review their own result.',
          null,
          null,
          'phase-4m-self-review'
        );
      `),
      /reviewers cannot review their own task submission/,
    );

    await database.exec(`
      reset role;
      select set_config('request.jwt.claim.sub', '${reviewerId}', false);
      select set_config(
        'request.jwt.claims',
        '{"sub":"${reviewerId}","role":"authenticated","app_metadata":{"operations_role":"reviewer"}}',
        false
      );
      set role authenticated;
    `);
    await assert.rejects(
      database.query(`
        select * from public.review_task_submission_v1(
          '${submissionId}',
          'accepted',
          'This acceptance must fail because the contributor did not consent to publication.',
          'A sanitized summary is present but cannot override the contributor consent boundary.',
          'https://public.example.com/no-consent',
          'phase-4m-no-consent-review'
        );
      `),
      /requires contributor public result consent/,
    );

    const rejected = await database.query<{
      submission_status: string;
      publication_id: string | null;
    }>(`
      select submission_status, publication_id
      from public.review_task_submission_v1(
        '${submissionId}',
        'rejected',
        'The submitted evidence did not satisfy the published task requirements.',
        null,
        null,
        'phase-4m-rejected-review'
      );
    `);
    assert.deepEqual(rejected.rows, [{ submission_status: 'rejected', publication_id: null }]);

    const outcome = await database.query<{
      publication_count: number;
      rejection_event_count: number;
    }>(`
      select
        (select count(*)::integer from public.task_submission_publications where task_id = '${taskId}') as publication_count,
        (
          select count(*)::integer
          from public.operations_task_workflow_events
          where entity_reference = '${submissionId}'
            and action = 'submission_rejected'
        ) as rejection_event_count;
    `);
    assert.deepEqual(outcome.rows, [{ publication_count: 0, rejection_event_count: 1 }]);

    await assert.rejects(
      database.query(`
        select * from public.review_task_submission_v1(
          '${submissionId}',
          'rejected',
          'Replay attempt.',
          null,
          null,
          'phase-4m-rejected-replay'
        );
      `),
      /already in a terminal review state/,
    );
  } finally {
    await database.close();
  }
});

test('runtime Phase 4M cleanup removes only the exact audited Staging workflow', async () => {
  const database = await createOperationsDatabase();
  const operatorId = '99999999-9999-4999-8999-999999999901';
  const ownerId = '99999999-9999-4999-8999-999999999902';
  const reviewerId = '99999999-9999-4999-8999-999999999903';
  const verifiedWallet = '11111111111111111111111111111111';
  const runId = '1785800000000-deadbeef';
  const runReference = `phase-2e-6b-4m-staging-e2e:${runId}`;

  try {
    await database.exec(`
      insert into auth.users (id) values
        ('${operatorId}'),
        ('${ownerId}'),
        ('${reviewerId}');
      insert into auth.identities (id, user_id, provider, provider_id, identity_data)
      values (
        'identity-phase-4m-cleanup-owner',
        '${ownerId}',
        'web3',
        'web3:solana:${verifiedWallet}',
        '{"sub":"web3:solana:${verifiedWallet}"}'
      );
      update public.operations_intake_control
      set mode = 'wallet_staging', activation_reference = 'phase-4m cleanup runtime fixture';
      select set_config('request.jwt.claim.sub', '${operatorId}', false);
      select set_config(
        'request.jwt.claims',
        '{"sub":"${operatorId}","role":"authenticated","app_metadata":{"operations_role":"operator"}}',
        false
      );
      set role authenticated;
    `);

    const task = await database.query<{ task_id: string }>(`
      select public.publish_community_task_v1(
        'Staging task workflow ${runId}',
        'Temporary Phase 4M Staging task for audited publication and review E2E ${runId}.',
        'Submit only the two reserved example.com fixtures for this controlled Staging E2E run ${runId}.',
        0,
        'none',
        '2099-01-01T00:00:00Z',
        '${runReference}:task:publish'
      ) as task_id;
    `);
    const taskId = task.rows[0]?.task_id;
    assert.ok(taskId);

    await database.exec(`
      reset role;
      select set_config('request.jwt.claim.sub', '${ownerId}', false);
      select set_config(
        'request.jwt.claims',
        '{"sub":"${ownerId}","role":"authenticated"}',
        false
      );
      set role authenticated;
    `);
    const submissions = await database.query<{ id: string; summary: string }>(`
      insert into public.task_submissions (
        task_id,
        submitted_by,
        summary,
        deliverable_url,
        wallet_address,
        public_result_consent,
        public_wallet_consent
      ) values
        (
          '${taskId}',
          '${ownerId}',
          'Accepted Phase 4M Staging submission ${runId} for sanitized publication and immutable audit verification.',
          'https://example.com/alpha-staging-task-${runId}-accepted',
          '${verifiedWallet}',
          true,
          false
        ),
        (
          '${taskId}',
          '${ownerId}',
          'Rejected Phase 4M Staging submission ${runId} for terminal-state and no-publication verification.',
          'https://example.com/alpha-staging-task-${runId}-rejected',
          '${verifiedWallet}',
          false,
          false
        )
      returning id, summary;
    `);
    const acceptedId = submissions.rows.find((row) => row.summary.startsWith('Accepted'))?.id;
    const rejectedId = submissions.rows.find((row) => row.summary.startsWith('Rejected'))?.id;
    assert.ok(acceptedId);
    assert.ok(rejectedId);

    await database.exec(`
      reset role;
      select set_config('request.jwt.claim.sub', '${reviewerId}', false);
      select set_config(
        'request.jwt.claims',
        '{"sub":"${reviewerId}","role":"authenticated","app_metadata":{"operations_role":"reviewer"}}',
        false
      );
      set role authenticated;
    `);
    const accepted = await database.query<{ publication_id: string }>(`
      select publication_id
      from public.review_task_submission_v1(
        '${acceptedId}',
        'accepted',
        'The controlled Staging fixture satisfies the published task requirements.',
        'Sanitized accepted result for Phase 4M Staging workflow ${runId}; no payment or treasury action occurred.',
        'https://example.com/alpha-staging-task-${runId}-accepted',
        '${runReference}:accepted'
      );
    `);
    assert.ok(accepted.rows[0]?.publication_id);

    await database.query(`
      select *
      from public.review_task_submission_v1(
        '${rejectedId}',
        'rejected',
        'The second controlled Staging fixture is intentionally rejected.',
        null,
        null,
        '${runReference}:rejected'
      );
    `);

    await database.exec('reset role;');
    await assert.rejects(
      database.exec(`
        delete from public.task_submission_publications
        where id = '${accepted.rows[0]?.publication_id}';
      `),
      /immutable operations record/,
    );

    await database.exec('set role authenticated;');
    await assert.rejects(
      database.query(`
        select * from public.cleanup_operations_task_staging_e2e_v1(
          '${runReference}',
          '${taskId}',
          array['${acceptedId}'::uuid, '${rejectedId}'::uuid]
        );
      `),
      /permission denied/,
    );

    await database.exec('reset role; set role service_role;');
    const cleanup = await database.query<{
      publications_deleted: number;
      events_deleted: number;
      submissions_deleted: number;
      tasks_deleted: number;
    }>(`
      select * from public.cleanup_operations_task_staging_e2e_v1(
        '${runReference}',
        '${taskId}',
        array['${acceptedId}'::uuid, '${rejectedId}'::uuid]
      );
    `);
    assert.deepEqual(cleanup.rows, [{
      publications_deleted: 1,
      events_deleted: 4,
      submissions_deleted: 2,
      tasks_deleted: 1,
    }]);

    await database.exec('reset role;');
    const remaining = await database.query<{ count: number }>(`
      select (
        (select count(*) from public.community_tasks where id = '${taskId}')
        + (select count(*) from public.task_submissions where task_id = '${taskId}')
        + (select count(*) from public.task_submission_publications where task_id = '${taskId}')
        + (
          select count(*)
          from public.operations_task_workflow_events
          where event_reference like '${runReference}:%'
        )
      )::integer as count;
    `);
    assert.equal(remaining.rows[0]?.count, 0);
  } finally {
    await database.close();
  }
});

test('all migrations create the expected schema, policy, and trigger totals', async () => {
  const database = await createOperationsDatabase();
  try {
    const tableCount = await database.query<{ count: number }>(`
      select count(*)::integer as count
      from pg_catalog.pg_tables
      where schemaname = 'public'
        and tablename = any(array[${expectedTables.map(quoteSql).join(', ')}]);
    `);
    const policyCount = await database.query<{ count: number }>(`
      select count(*)::integer as count
      from pg_catalog.pg_policies
      where schemaname = 'public'
        and tablename = any(array[${expectedTables.map(quoteSql).join(', ')}]);
    `);
    const triggerCount = await database.query<{ count: number }>(`
      select count(*)::integer as count
      from pg_catalog.pg_trigger trigger
      join pg_catalog.pg_class relation on relation.oid = trigger.tgrelid
      join pg_catalog.pg_namespace namespace on namespace.oid = relation.relnamespace
      where not trigger.tgisinternal
        and namespace.nspname = 'public'
        and relation.relname = any(array[${expectedTables.map(quoteSql).join(', ')}]);
    `);

    assert.equal(tableCount.rows[0]?.count, 17);
    assert.equal(policyCount.rows[0]?.count, 39);
    assert.equal(triggerCount.rows[0]?.count, 37);
  } finally {
    await database.close();
  }
});

test('runtime requires an auditable server-gate activation and hides its control row', async () => {
  const database = await createOperationsDatabase();

  try {
    await assert.rejects(
      database.exec(`
        update public.operations_intake_control
        set mode = 'wallet_staging';
      `),
      /check constraint/,
    );

    await database.exec(`
      update public.operations_intake_control
      set
        mode = 'wallet_staging',
        activation_reference = 'local auditable activation test';
      set role authenticated;
    `);
    const gateResult = await database.query<{ enabled: boolean }>(`
      select public.is_operations_wallet_intake_enabled() as enabled;
    `);
    assert.equal(gateResult.rows[0]?.enabled, true);
    await assert.rejects(
      database.query('select * from public.operations_intake_control;'),
      /permission denied/,
    );
    await database.exec('reset role;');
  } finally {
    await database.close();
  }
});

test('runtime gate RPC audits activation and emergency disable without direct service-role mutation', async () => {
  const database = await createOperationsDatabase();

  try {
    const initial = await database.query<{ mode: string; event_count: number }>(`
      select
        control.mode,
        (select count(*)::integer from public.operations_intake_gate_events) as event_count
      from public.operations_intake_control control
      where control.singleton;
    `);
    assert.deepEqual(initial.rows, [{ mode: 'disabled', event_count: 0 }]);

    await database.exec('set role service_role;');
    await assert.rejects(
      database.exec(`
        update public.operations_intake_control
        set mode = 'wallet_staging', activation_reference = 'direct service update';
      `),
      /permission denied/,
    );

    await assert.rejects(
      database.query(`
        select * from public.set_operations_wallet_intake_mode(
          'wallet_staging',
          E'phase-4l invalid\\ncontrol reference'
        );
      `),
      /without control characters/,
    );

    const activated = await database.query<{
      mode: string;
      activation_reference: string | null;
      event_id: number;
    }>(`
      select mode, activation_reference, event_id
      from public.set_operations_wallet_intake_mode(
        'wallet_staging',
        'phase-4l isolated activation test'
      );
    `);
    assert.equal(activated.rows[0]?.mode, 'wallet_staging');
    assert.equal(
      activated.rows[0]?.activation_reference,
      'phase-4l isolated activation test',
    );
    assert.equal(activated.rows[0]?.event_id, 1);

    await assert.rejects(
      database.query(`
        select * from public.set_operations_wallet_intake_mode(
          'wallet_staging',
          'duplicate activation attempt'
        );
      `),
      /already in requested mode/,
    );

    const disabled = await database.query<{
      mode: string;
      activation_reference: string | null;
      event_id: number;
    }>(`
      select mode, activation_reference, event_id
      from public.set_operations_wallet_intake_mode(
        'disabled',
        'phase-4l emergency disable test'
      );
    `);
    assert.equal(disabled.rows[0]?.mode, 'disabled');
    assert.equal(disabled.rows[0]?.activation_reference, null);
    assert.equal(disabled.rows[0]?.event_id, 2);

    const events = await database.query<{
      previous_mode: string;
      new_mode: string;
      change_reference: string;
    }>(`
      select previous_mode, new_mode, change_reference
      from public.operations_intake_gate_events
      order by event_id;
    `);
    assert.deepEqual(events.rows, [
      {
        previous_mode: 'disabled',
        new_mode: 'wallet_staging',
        change_reference: 'phase-4l isolated activation test',
      },
      {
        previous_mode: 'wallet_staging',
        new_mode: 'disabled',
        change_reference: 'phase-4l emergency disable test',
      },
    ]);

    await assert.rejects(
      database.exec(`
        update public.operations_intake_gate_events
        set change_reference = 'rewritten audit reference';
      `),
      /permission denied/,
    );

    await database.exec('reset role;');
    await assert.rejects(
      database.exec(`
        delete from public.operations_intake_gate_events where event_id = 1;
      `),
      /immutable operations record/,
    );

    await database.exec('set role authenticated;');
    await assert.rejects(
      database.query(`
        select * from public.set_operations_wallet_intake_mode(
          'wallet_staging',
          'unauthorized browser activation'
        );
      `),
      /permission denied/,
    );
  } finally {
    await database.close();
  }
});

test('runtime resolves only one matching observed Supabase Solana Web3 subject', async () => {
  const database = await createOperationsDatabase();
  const validUserId = '77777777-7777-4777-8777-777777777700';
  const mismatchUserId = '77777777-7777-4777-8777-777777777701';
  const wrongChainUserId = '77777777-7777-4777-8777-777777777702';
  const malformedUserId = '77777777-7777-4777-8777-777777777703';
  const legacyUserId = '77777777-7777-4777-8777-777777777704';
  const ambiguousUserId = '77777777-7777-4777-8777-777777777705';
  const verifiedWallet = '11111111111111111111111111111111';
  const otherWallet = 'HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY';

  try {
    await database.exec(`
      insert into auth.users (id) values
        ('${validUserId}'),
        ('${mismatchUserId}'),
        ('${wrongChainUserId}'),
        ('${malformedUserId}'),
        ('${legacyUserId}'),
        ('${ambiguousUserId}');

      insert into auth.identities (
        id,
        user_id,
        provider,
        provider_id,
        identity_data
      ) values
        (
          'identity-valid',
          '${validUserId}',
          'web3',
          'web3:solana:${verifiedWallet}',
          '{"sub":"web3:solana:${verifiedWallet}"}'
        ),
        (
          'identity-mismatch',
          '${mismatchUserId}',
          'web3',
          'web3:solana:${verifiedWallet}',
          '{"sub":"web3:solana:${otherWallet}"}'
        ),
        (
          'identity-wrong-chain',
          '${wrongChainUserId}',
          'web3',
          'web3:ethereum:${verifiedWallet}',
          '{"sub":"web3:ethereum:${verifiedWallet}"}'
        ),
        (
          'identity-malformed',
          '${malformedUserId}',
          'web3',
          'web3:solana:1111111111111111111111111111111',
          '{"sub":"web3:solana:1111111111111111111111111111111"}'
        ),
        (
          'identity-legacy',
          '${legacyUserId}',
          'web3',
          '${verifiedWallet}',
          '{"chain":"solana","address":"${verifiedWallet}"}'
        ),
        (
          'identity-ambiguous-a',
          '${ambiguousUserId}',
          'web3',
          'web3:solana:${verifiedWallet}',
          '{"sub":"web3:solana:${verifiedWallet}"}'
        ),
        (
          'identity-ambiguous-b',
          '${ambiguousUserId}',
          'web3',
          'web3:solana:${otherWallet}',
          '{"sub":"web3:solana:${otherWallet}"}'
        );
    `);

    assert.equal(
      await resolveCurrentWallet(database, validUserId),
      verifiedWallet,
    );
    for (const userId of [
      mismatchUserId,
      wrongChainUserId,
      malformedUserId,
      legacyUserId,
      ambiguousUserId,
    ]) {
      assert.equal(await resolveCurrentWallet(database, userId), null);
    }
  } finally {
    await database.close();
  }
});

test('runtime keeps wallet intake closed after the migration is applied', async () => {
  const database = await createOperationsDatabase();
  const walletUserId = '66666666-6666-4666-8666-666666666666';
  const verifiedWallet = '11111111111111111111111111111111';

  try {
    await database.exec(`
      insert into auth.users (id) values ('${walletUserId}');
      insert into auth.identities (id, user_id, provider, provider_id, identity_data)
      values (
        'identity-disabled-gate',
        '${walletUserId}',
        'web3',
        'web3:solana:${verifiedWallet}',
        '{"sub":"web3:solana:${verifiedWallet}"}'
      );
      select set_config('request.jwt.claim.sub', '${walletUserId}', false);
      select set_config(
        'request.jwt.claims',
        '{"sub":"${walletUserId}","role":"authenticated"}',
        false
      );
      set role authenticated;
    `);

    await assert.rejects(
      database.exec(`
        insert into public.risk_reports (
          submitted_by,
          project_identifier,
          summary,
          reference_url,
          wallet_address
        ) values (
          '${walletUserId}',
          'Disabled server gate report',
          'This sufficiently detailed report must fail while the database intake gate is disabled.',
          'https://example.com/disabled-server-gate',
          '${verifiedWallet}'
        );
      `),
      /operations wallet intake is disabled/,
    );
  } finally {
    await database.close();
  }
});

test('runtime accepts the matching Web3 wallet and rejects email or switched-wallet intake', async () => {
  const database = await createOperationsDatabase();
  const walletUserId = '33333333-3333-4333-8333-333333333333';
  const emailUserId = '44444444-4444-4444-8444-444444444444';
  const verifiedWallet = '11111111111111111111111111111111';
  const otherWallet = 'HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY';

  try {
    await database.exec(`
      insert into auth.users (id) values
        ('${walletUserId}'),
        ('${emailUserId}');

      insert into auth.identities (id, user_id, provider, provider_id, identity_data)
      values (
        'identity-matching-wallet',
        '${walletUserId}',
        'web3',
        'web3:solana:${verifiedWallet}',
        '{"sub":"web3:solana:${verifiedWallet}"}'
      );

      update public.operations_intake_control
      set
        mode = 'wallet_staging',
        activation_reference = 'local matching-wallet runtime test';

      select set_config('request.jwt.claim.sub', '${walletUserId}', false);
      select set_config(
        'request.jwt.claims',
        '{"sub":"${walletUserId}","role":"authenticated"}',
        false
      );
      set role authenticated;
    `);

    await database.exec(`
      insert into public.risk_reports (
        submitted_by,
        project_identifier,
        summary,
        reference_url,
        wallet_address
      ) values (
        '${walletUserId}',
        'Verified wallet report',
        'A sufficiently detailed risk report submitted by the matching wallet identity.',
        'https://example.com/verified-wallet-report',
        '${verifiedWallet}'
      );
    `);

    await assert.rejects(
      database.exec(`
        insert into public.risk_reports (
          submitted_by,
          project_identifier,
          summary,
          reference_url,
          wallet_address
        ) values (
          '${walletUserId}',
          'Switched wallet report',
          'This report must fail because its wallet differs from the verified identity.',
          'https://example.com/switched-wallet-report',
          '${otherWallet}'
        );
      `),
      /row-level security policy/,
    );

    await assert.rejects(
      database.exec(`
        insert into public.risk_reports (
          submitted_by,
          project_identifier,
          summary,
          reference_url,
          wallet_address
        ) values (
          '${emailUserId}',
          'Forged owner report',
          'This report must fail because submitted_by differs from the authenticated user.',
          'https://example.com/forged-owner-report',
          '${verifiedWallet}'
        );
      `),
      /operations submission owner does not match auth\.uid/,
    );

    await database.exec('reset role;');
    await database.exec(`
      select set_config('request.jwt.claim.sub', '${emailUserId}', false);
      select set_config(
        'request.jwt.claims',
        '{"sub":"${emailUserId}","role":"authenticated"}',
        false
      );
      set role authenticated;
    `);

    await assert.rejects(
      database.exec(`
        insert into public.risk_reports (
          submitted_by,
          project_identifier,
          summary,
          reference_url,
          wallet_address
        ) values (
          '${emailUserId}',
          'Email-only report',
          'This report must fail because email authentication does not prove wallet control.',
          'https://example.com/email-only-report',
          '${verifiedWallet}'
        );
      `),
      /row-level security policy/,
    );

    await database.exec('reset role;');
  } finally {
    await database.close();
  }
});

test('runtime database throttle rejects the seventh hourly risk report', async () => {
  const database = await createOperationsDatabase();
  const walletUserId = '55555555-5555-4555-8555-555555555555';
  const verifiedWallet = '11111111111111111111111111111111';

  try {
    await database.exec(`
      insert into auth.users (id) values ('${walletUserId}');
      insert into auth.identities (id, user_id, provider, provider_id, identity_data)
      values (
        'identity-rate-limit-wallet',
        '${walletUserId}',
        'web3',
        'web3:solana:${verifiedWallet}',
        '{"sub":"web3:solana:${verifiedWallet}"}'
      );
      update public.operations_intake_control
      set
        mode = 'wallet_staging',
        activation_reference = 'local database throttle runtime test';
      select set_config('request.jwt.claim.sub', '${walletUserId}', false);
      select set_config(
        'request.jwt.claims',
        '{"sub":"${walletUserId}","role":"authenticated"}',
        false
      );
      set role authenticated;
    `);

    for (let index = 0; index < 6; index += 1) {
      await database.exec(`
        insert into public.risk_reports (
          submitted_by,
          project_identifier,
          summary,
          reference_url,
          wallet_address,
          created_at
        ) values (
          '${walletUserId}',
          'Rate limit report ${index}',
          'A sufficiently detailed risk report used to validate the database throttle.',
          'https://example.com/rate-limit-${index}',
          '${verifiedWallet}',
          '2000-01-01T00:00:00Z'
        );
      `);
    }

    await assert.rejects(
      database.exec(`
        insert into public.risk_reports (
          submitted_by,
          project_identifier,
          summary,
          reference_url,
          wallet_address,
          created_at
        ) values (
          '${walletUserId}',
          'Rate limit report seven',
          'This sufficiently detailed report must be rejected by the database throttle.',
          'https://example.com/rate-limit-seven',
          '${verifiedWallet}',
          '2000-01-01T00:00:00Z'
        );
      `),
      /operations submission rate limit exceeded/,
    );
    await database.exec('reset role;');
  } finally {
    await database.close();
  }
});

test('runtime rejects published downgrade and allows moderator private discussion reads', async () => {
  const database = await createOperationsDatabase();
  const ownerId = '11111111-1111-4111-8111-111111111111';
  const moderatorId = '22222222-2222-4222-8222-222222222222';

  try {
    await database.exec(`
      insert into auth.users (id) values
        ('${ownerId}'),
        ('${moderatorId}');

      select set_config('request.jwt.claim.sub', '${ownerId}', false);
      select set_config(
        'request.jwt.claims',
        '{"sub":"${ownerId}","role":"authenticated"}',
        false
      );
      update public.operations_intake_control
      set
        mode = 'wallet_staging',
        activation_reference = 'local moderator runtime fixture';

      insert into public.community_tasks (
        title,
        summary,
        requirements,
        status,
        publication_status,
        published_at
      ) values (
        'Runtime staging task',
        'A sufficiently long staging summary for runtime validation.',
        'A sufficiently long staging requirement for runtime validation.',
        'closed',
        'published',
        timezone('utc', now())
      );

      insert into public.governance_discussions (
        submitted_by,
        topic,
        body,
        moderation_status
      ) values (
        '${ownerId}',
        'Runtime moderation',
        'A private discussion body that only its owner and moderators may read.',
        'pending'
      );
    `);

    await assert.rejects(
      database.exec(`
        update public.community_tasks
        set publication_status = 'draft'
        where title = 'Runtime staging task';
      `),
      /cannot be unpublished/,
    );

    await database.exec(`
      select set_config(
        'request.jwt.claims',
        '{"sub":"${moderatorId}","app_metadata":{"operations_role":"moderator"}}',
        false
      );
      set role authenticated;
    `);
    const moderatorRows = await database.query<{ topic: string }>(`
      select topic
      from public.governance_discussions
      where topic = 'Runtime moderation';
    `);
    assert.deepEqual(
      moderatorRows.rows.map((row) => row.topic),
      ['Runtime moderation'],
    );
    await database.exec('reset role;');
  } finally {
    await database.close();
  }
});

function extractCreateTable(table: string): string {
  const match = sql.match(
    new RegExp(`create table public\\.${table} \\([\\s\\S]*?\\n\\);`),
  );
  assert.ok(match, `missing create table statement for ${table}`);
  return match[0];
}

function extractPolicy(policy: string, source = sql): string {
  const match = source.match(
    new RegExp(`create policy ${policy}[\\s\\S]*?;`),
  );
  assert.ok(match, `missing policy ${policy}`);
  return match[0];
}

async function createOperationsDatabase(): Promise<PGlite> {
  const database = new PGlite();
  await database.exec(`
    create role anon nologin;
    create role authenticated nologin;
    create role service_role nologin bypassrls;
    create schema auth;
    create table auth.users (id uuid primary key);
    create table auth.identities (
      id text primary key,
      user_id uuid not null references auth.users(id),
      provider text not null,
      provider_id text not null,
      identity_data jsonb not null
    );

    create function auth.uid()
    returns uuid
    language sql
    stable
    as $$
      select nullif(current_setting('request.jwt.claim.sub', true), '')::uuid;
    $$;

    create function auth.jwt()
    returns jsonb
    language sql
    stable
    as $$
      select coalesce(
        nullif(current_setting('request.jwt.claims', true), ''),
        '{}'
      )::jsonb;
    $$;

    grant usage on schema auth to anon, authenticated;
    grant select on table auth.users to authenticated;
  `);

  await database.exec(
    foundationSql.replace(
      'create extension if not exists pgcrypto;',
      '-- pgcrypto is supplied by Supabase and omitted only in PGlite validation',
    ),
  );
  await database.exec(hardeningSql);
  await database.exec(cleanupPrivilegesSql);
  await database.exec(walletIntakeSql);
  await database.exec(identityCompatibilitySql);
  await database.exec(walletResolverLintCleanupSql);
  await database.exec(intakeGateAuditSql);
  await database.exec(taskModerationClosureSql);
  await database.exec(taskStagingE2ECleanupSql);
  return database;
}

async function resolveCurrentWallet(
  database: PGlite,
  userId: string,
): Promise<string | null> {
  await database.exec(`
    reset role;
    select set_config('request.jwt.claim.sub', '${userId}', false);
    select set_config(
      'request.jwt.claims',
      '{"sub":"${userId}","role":"authenticated"}',
      false
    );
    set role authenticated;
  `);
  const result = await database.query<{ wallet_address: string | null }>(`
    select public.current_verified_solana_wallet() as wallet_address;
  `);
  await database.exec('reset role;');
  return result.rows[0]?.wallet_address ?? null;
}

function quoteSql(value: string): string {
  return `'${value.replaceAll("'", "''")}'`;
}
