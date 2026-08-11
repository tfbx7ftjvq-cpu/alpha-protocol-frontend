import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { PGlite } from '@electric-sql/pglite';

const foundationSql = readMigration('202607270001_offchain_operations_foundation.sql');
const hardeningSql = readMigration('202607270002_operations_staging_hardening.sql');
const cleanupPrivilegesSql = readMigration('202607290001_operations_staging_e2e_cleanup_privileges.sql');
const walletIntakeSql = readMigration('202607300001_wallet_authenticated_operations_intake.sql');
const identityCompatibilitySql = readMigration('202607310001_web3_solana_identity_subject_compatibility.sql');
const walletResolverLintCleanupSql = readMigration('202608020001_web3_solana_wallet_resolver_lint_cleanup.sql');
const intakeGateAuditSql = readMigration('202608020002_operations_wallet_intake_gate_audit.sql');
const taskModerationClosureSql = readMigration('202608030001_operations_task_moderation_closure.sql');
const taskCleanupSql = readMigration('202608040001_operations_task_staging_e2e_cleanup.sql');
const riskModerationClosureSql = readMigration('202608050001_operations_risk_moderation_closure.sql');
const riskCleanupSql = readMigration('202608050002_operations_risk_staging_e2e_cleanup.sql');
const reliefModerationClosureSql = readMigration('202608060001_operations_relief_moderation_closure.sql');
const reliefCleanupSql = readMigration('202608060002_operations_relief_staging_e2e_cleanup.sql');
const reliefPaymentGuardSql = readMigration('202608060003_operations_relief_staging_e2e_payment_guard.sql');
const governanceOperationsSql = readMigration('202608070001_governance_operations_audited_execution_preparation.sql');
const treasuryRegistrySql = readMigration('202608080001_audited_treasury_execution_registry_and_reconciliation.sql');
const treasuryBase58LintCleanupSql = readMigration('202608080002_treasury_execution_base58_lint_cleanup.sql');
const auditedAccessControlSql = readMigration('202608110001_audited_operations_access_control_and_role_lifecycle.sql');

test('migration fails closed when legacy operations_role claims exist', async () => {
  const database = await createOperationsDatabase(false);
  const userId = '70000000-0000-4000-8000-000000000001';

  try {
    await database.exec(`
      insert into auth.users (id, raw_app_meta_data)
      values ('${userId}', '{"operations_role":"operator"}'::jsonb);
    `);

    await assert.rejects(
      database.exec(auditedAccessControlSql),
      /legacy operations_role JWT claims detected/i,
    );
  } finally {
    await database.close();
  }
});

test('JWT claims never authorize operations access without an audited database assignment', async () => {
  const database = await createOperationsDatabase();
  const userId = '70000000-0000-4000-8000-000000000002';

  try {
    await database.exec(`
      insert into auth.users (id) values ('${userId}');
    `);

    await assumeAuthenticated(database, userId, 'operator');
    const roleResult = await database.query<{ role_name: string | null }>(`
      select public.current_operations_role_v1() as role_name;
    `);
    assert.equal(roleResult.rows[0]?.role_name ?? null, null);

    const accessResult = await database.query<{
      role_name: string;
      status: string;
      expires_at: string | null;
    }>(`
      select * from public.get_my_operations_access_v1();
    `);
    assert.deepEqual(accessResult.rows, []);

    await assert.rejects(
      database.query(`
        select public.publish_community_task_v1(
          'Claim-only task',
          'JWT claims alone must not authorize operations publication.',
          'This call must fail closed because the database has no audited assignment.',
          0,
          'none',
          null,
          'phase-2e-6e-claim-only-denied'
        );
      `),
      /operations role is not authorized to publish tasks/i,
    );
  } finally {
    await database.close();
  }
});

test('active audited assignments authorize access and revoked or expired assignments fail closed immediately', async () => {
  const database = await createOperationsDatabase();
  const operatorId = '70000000-0000-4000-8000-000000000003';
  const reviewerId = '70000000-0000-4000-8000-000000000004';
  const expiringId = '70000000-0000-4000-8000-000000000005';

  try {
    await database.exec(`
      insert into auth.users (id) values
        ('${operatorId}'),
        ('${reviewerId}'),
        ('${expiringId}');
    `);

    await grantOperationsRole(database, operatorId, 'operator', 'phase-2e-6e-grant-operator');
    await assumeAuthenticated(database, operatorId);
    const published = await database.query<{ task_id: string }>(`
      select public.publish_community_task_v1(
        'Audited operator task',
        'A database-assigned operator can publish this task.',
        'This verifies that operations authorization resolves from audited assignments.',
        0,
        'none',
        null,
        'phase-2e-6e-audited-operator'
      ) as task_id;
    `);
    assert.ok(published.rows[0]?.task_id);

    const accessResult = await database.query<{
      role_name: string;
      status: string;
      expires_at: string | null;
    }>(`
      select * from public.get_my_operations_access_v1();
    `);
    assert.deepEqual(accessResult.rows, [{
      role_name: 'operator',
      status: 'active',
      expires_at: null,
    }]);

    await grantOperationsRole(database, reviewerId, 'reviewer', 'phase-2e-6e-grant-reviewer');
    await revokeOperationsRole(database, reviewerId, 'phase-2e-6e-revoke-reviewer');
    await assumeAuthenticated(database, reviewerId, 'reviewer');
    const revokedRole = await database.query<{ role_name: string | null }>(`
      select public.current_operations_role_v1() as role_name;
    `);
    assert.equal(revokedRole.rows[0]?.role_name ?? null, null);
    await assert.rejects(
      database.query(`
        select * from public.review_task_submission_v1(
          '70000000-0000-4000-8000-000000000099',
          'rejected',
          'Revoked reviewers must not retain access.',
          null,
          null,
          'phase-2e-6e-revoked-denied'
        );
      `),
      /operations role is not authorized to review task submissions/i,
    );

    await grantOperationsRole(
      database,
      expiringId,
      'reviewer',
      'phase-2e-6e-grant-expiring',
      '2099-01-01T00:00:00Z',
    );
    await database.exec(`
      update public.operations_role_assignments
      set status = 'expired'
      where user_id = '${expiringId}';
    `);
    await assumeAuthenticated(database, expiringId);
    const expiredRole = await database.query<{ role_name: string | null }>(`
      select public.current_operations_role_v1() as role_name;
    `);
    assert.equal(expiredRole.rows[0]?.role_name ?? null, null);
  } finally {
    await database.close();
  }
});

test('grant and revoke RPCs are service-role-only, emit append-only audit events, and tables reject direct mutation', async () => {
  const database = await createOperationsDatabase();
  const userId = '70000000-0000-4000-8000-000000000006';

  try {
    await database.exec(`
      insert into auth.users (id) values ('${userId}');
    `);

    await assumeAuthenticated(database, userId);
    await assert.rejects(
      database.query(`
        select * from public.grant_operations_role_v1(
          '${userId}',
          'operator',
          'phase-2e-6e-authenticated-grant-denied'
        );
      `),
      /permission denied|granting operations roles requires the service_role credential/i,
    );
    await database.exec('reset role;');

    await grantOperationsRole(database, userId, 'operator', 'phase-2e-6e-service-grant');
    await revokeOperationsRole(database, userId, 'phase-2e-6e-service-revoke');

    const events = await database.query<{
      event_type: string;
      previous_status: string | null;
      new_status: string;
      actor_type: string;
      actor_user_id: string | null;
    }>(`
      select event_type, previous_status, new_status, actor_type, actor_user_id
      from public.operations_role_assignment_events
      where user_id = '${userId}'
      order by event_id asc;
    `);
    assert.deepEqual(events.rows, [
      {
        event_type: 'granted',
        previous_status: null,
        new_status: 'active',
        actor_type: 'service_role',
        actor_user_id: null,
      },
      {
        event_type: 'revoked',
        previous_status: 'active',
        new_status: 'revoked',
        actor_type: 'service_role',
        actor_user_id: null,
      },
    ]);

    await database.exec('set role service_role;');
    await assert.rejects(
      database.exec(`
        insert into public.operations_role_assignments (
          user_id,
          role_name,
          status,
          grant_reference
        ) values (
          '${userId}',
          'operator',
          'active',
          'phase-2e-6e-direct-table-write'
        );
      `),
      /permission denied/i,
    );
    await assert.rejects(
      database.exec(`
        update public.operations_role_assignment_events
        set change_reference = 'phase-2e-6e-mutated'
        where user_id = '${userId}';
      `),
      /permission denied|append-only|immutable/i,
    );
    await database.exec('reset role;');
  } finally {
    await database.close();
  }
});

test('source files no longer use JWT app_metadata.operations_role as final authorization', () => {
  const repositorySource = readFileSync(
    new URL('../src/features/operations/repository.ts', import.meta.url),
    'utf8',
  );
  const walletHookSource = readFileSync(
    new URL('../src/hooks/useOperationsWalletAuth.ts', import.meta.url),
    'utf8',
  );
  const rolesToolSource = readFileSync(
    new URL('../scripts/operations-staging/operations-roles.ts', import.meta.url),
    'utf8',
  );
  const stagingCommonSource = readFileSync(
    new URL('../scripts/operations-staging/common.ts', import.meta.url),
    'utf8',
  );
  const stagingE2ESource = readFileSync(
    new URL('../scripts/operations-staging/e2e.ts', import.meta.url),
    'utf8',
  );

  assert.match(repositorySource, /client\.rpc\('get_my_operations_access_v1'\)/);
  assert.doesNotMatch(repositorySource, /app_metadata\s*\.\s*operations_role/);
  assert.match(walletHookSource, /loadMyOperationsAccess/);
  assert.doesNotMatch(walletHookSource, /app_metadata\s*\.\s*operations_role/);
  assert.match(stagingCommonSource, /OPERATIONS_STAGING_SERVICE_ROLE_KEY/);
  assert.match(rolesToolSource, /sanitizeStagingError/);
  assert.match(stagingE2ESource, /grant_operations_role_v1/);
  assert.match(stagingE2ESource, /revoke_operations_role_v1/);
  assert.doesNotMatch(stagingE2ESource, /updateUserById/);
});

test('audited access migration contains no HTTP, signer, or Solana transaction sender paths', () => {
  assert.doesNotMatch(
    auditedAccessControlSql,
    /http_post|http_get|fetch\(|axios\.|sendgrid|resend|net\.http/i,
  );
  assert.doesNotMatch(auditedAccessControlSql, /signTransaction|sendTransaction|solana rpc|connection\./i);
  assert.match(auditedAccessControlSql, /create table public\.operations_role_assignments/i);
  assert.match(auditedAccessControlSql, /create table public\.operations_role_assignment_events/i);
  assert.match(auditedAccessControlSql, /create or replace function public\.current_operations_role_v1\(\)/i);
  assert.match(auditedAccessControlSql, /create or replace function public\.grant_operations_role_v1\(/i);
});

async function createOperationsDatabase(
  includeAuditedAccessControl = true,
): Promise<PGlite> {
  const database = new PGlite();
  await database.exec(`
    create role anon nologin;
    create role authenticated nologin;
    create role service_role nologin bypassrls;
    create schema auth;
    create table auth.users (
      id uuid primary key,
      raw_app_meta_data jsonb not null default '{}'::jsonb
    );
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

    grant usage on schema auth to anon, authenticated, service_role;
    grant select on table auth.users to authenticated, service_role;
  `);

  await database.exec(
    foundationSql.replace(
      'create extension if not exists pgcrypto;',
      '-- pgcrypto is omitted in PGlite validation',
    ),
  );
  await database.exec(hardeningSql);
  await database.exec(cleanupPrivilegesSql);
  await database.exec(walletIntakeSql);
  await database.exec(identityCompatibilitySql);
  await database.exec(walletResolverLintCleanupSql);
  await database.exec(intakeGateAuditSql);
  await database.exec(taskModerationClosureSql);
  await database.exec(taskCleanupSql);
  await database.exec(riskModerationClosureSql);
  await database.exec(riskCleanupSql);
  await database.exec(reliefModerationClosureSql);
  await database.exec(reliefCleanupSql);
  await database.exec(reliefPaymentGuardSql);
  await database.exec(governanceOperationsSql);
  await database.exec(treasuryRegistrySql);
  await database.exec(treasuryBase58LintCleanupSql);
  if (includeAuditedAccessControl) {
    await database.exec(auditedAccessControlSql);
  }
  return database;
}

async function assumeAuthenticated(
  database: PGlite,
  userId: string,
  legacyClaimRole?: string,
): Promise<void> {
  await database.exec('reset role;');
  await database.exec(`
    select set_config('request.jwt.claim.sub', '${userId}', false);
    select set_config(
      'request.jwt.claims',
      '{"sub":"${userId}","role":"authenticated"${legacyClaimRole ? `,"app_metadata":{"operations_role":"${legacyClaimRole}"}` : ''}}',
      false
    );
    set role authenticated;
  `);
}

async function grantOperationsRole(
  database: PGlite,
  userId: string,
  roleName: string,
  grantReference: string,
  expiresAt: string | null = null,
): Promise<void> {
  await database.exec('reset role;');
  await database.exec(`
    select set_config(
      'request.jwt.claims',
      '{"role":"service_role"}',
      false
    );
    set role service_role;
  `);
  const expiresArgument = expiresAt ? `'${expiresAt}'::timestamptz` : 'null';
  await database.query(`
    select * from public.grant_operations_role_v1(
      '${userId}'::uuid,
      '${roleName}',
      '${grantReference}',
      ${expiresArgument}
    );
  `);
  await database.exec('reset role;');
}

async function revokeOperationsRole(
  database: PGlite,
  userId: string,
  revokeReference: string,
): Promise<void> {
  await database.exec('reset role;');
  await database.exec(`
    select set_config(
      'request.jwt.claims',
      '{"role":"service_role"}',
      false
    );
    set role service_role;
  `);
  await database.query(`
    select * from public.revoke_operations_role_v1(
      '${userId}'::uuid,
      '${revokeReference}'
    );
  `);
  await database.exec('reset role;');
}

function readMigration(name: string): string {
  return readFileSync(
    new URL(`../../supabase/migrations/${name}`, import.meta.url),
    'utf8',
  );
}
