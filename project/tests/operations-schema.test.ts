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
const foundationSql = readFileSync(foundationMigrationUrl, 'utf8');
const hardeningSql = readFileSync(hardeningMigrationUrl, 'utf8');
const cleanupPrivilegesSql = readFileSync(cleanupPrivilegesMigrationUrl, 'utf8');
const walletIntakeSql = readFileSync(walletIntakeMigrationUrl, 'utf8');
const identityCompatibilitySql = readFileSync(identityCompatibilityMigrationUrl, 'utf8');
const walletResolverLintCleanupSql = readFileSync(
  walletResolverLintCleanupMigrationUrl,
  'utf8',
);
const sql = [
  foundationSql,
  hardeningSql,
  cleanupPrivilegesSql,
  walletIntakeSql,
  identityCompatibilitySql,
  walletResolverLintCleanupSql,
].join('\n');

const expectedTables = [
  'operations_intake_control',
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
    'operations_intake_control',
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
      { table_name: 'community_tasks', privilege_type: 'DELETE' },
      { table_name: 'community_tasks', privilege_type: 'SELECT' },
      { table_name: 'governance_discussions', privilege_type: 'DELETE' },
      { table_name: 'governance_discussions', privilege_type: 'SELECT' },
      { table_name: 'task_submissions', privilege_type: 'DELETE' },
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

    assert.equal(tableCount.rows[0]?.count, 14);
    assert.equal(policyCount.rows[0]?.count, 37);
    assert.equal(triggerCount.rows[0]?.count, 34);
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
