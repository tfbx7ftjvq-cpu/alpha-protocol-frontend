import assert from 'node:assert/strict';
import test from 'node:test';
import {
  cleanupOperationsRoles,
  isExactCleanupDeletion,
  readReliefPaymentStateCounts,
  readReliefWorkflowCleanupCounts,
  readRiskWorkflowCleanupCounts,
  readTaskWorkflowCleanupCounts,
} from '../scripts/operations-staging/e2e.ts';

function cleanupRoleAdmin(
  roleRowsByUserId: Record<string, unknown>,
): {
  calls: Array<{ name: string; params: Record<string, unknown> }>;
  admin: Parameters<typeof cleanupOperationsRoles>[0];
} {
  const calls: Array<{ name: string; params: Record<string, unknown> }> = [];
  const admin = {
    rpc: async (name: string, params: Record<string, unknown>) => {
      calls.push({ name, params });
      if (name === 'inspect_operations_role_v1') {
        return { data: roleRowsByUserId[String(params.p_user_id)], error: null };
      }
      return { data: null, error: null };
    },
  } as unknown as Parameters<typeof cleanupOperationsRoles>[0];
  return { calls, admin };
}

test('operations role cleanup skips users with no role assignment', async () => {
  const { admin, calls } = cleanupRoleAdmin({ unassigned: [] });
  const errors: string[] = [];

  await cleanupOperationsRoles(admin, ['unassigned'], 'run', errors);

  assert.deepEqual(errors, []);
  assert.deepEqual(calls, [{
    name: 'inspect_operations_role_v1',
    params: { p_user_id: 'unassigned' },
  }]);
});

test('operations role cleanup revokes one active assignment', async () => {
  const { admin, calls } = cleanupRoleAdmin({ active: [{ status: 'active' }] });
  const errors: string[] = [];

  await cleanupOperationsRoles(admin, ['active'], 'run', errors);

  assert.deepEqual(errors, []);
  assert.deepEqual(calls, [
    {
      name: 'inspect_operations_role_v1',
      params: { p_user_id: 'active' },
    },
    {
      name: 'revoke_operations_role_v1',
      params: {
        p_user_id: 'active',
        p_revoke_reference: 'run:cleanup:revoke:active',
      },
    },
  ]);
});

test('operations role cleanup fails closed for multiple assignments', async () => {
  const { admin, calls } = cleanupRoleAdmin({
    duplicated: [{ status: 'active' }, { status: 'active' }],
  });
  const errors: string[] = [];

  await cleanupOperationsRoles(admin, ['duplicated'], 'run', errors);

  assert.deepEqual(errors, ['operations role inspection cleanup returned multiple rows']);
  assert.deepEqual(calls, [{
    name: 'inspect_operations_role_v1',
    params: { p_user_id: 'duplicated' },
  }]);
});

test('operations role cleanup records one user failure and continues', async () => {
  const calls: string[] = [];
  const admin = {
    rpc: async (name: string, params: Record<string, unknown>) => {
      calls.push(`${name}:${params.p_user_id}`);
      if (name === 'inspect_operations_role_v1' && params.p_user_id === 'failed') {
        return { data: null, error: new Error('inspection failed') };
      }
      if (name === 'inspect_operations_role_v1') {
        return { data: [{ status: 'active' }], error: null };
      }
      return { data: null, error: null };
    },
  } as unknown as Parameters<typeof cleanupOperationsRoles>[0];
  const errors: string[] = [];

  await cleanupOperationsRoles(admin, ['active', 'failed'], 'run', errors);

  assert.deepEqual(errors, ['operations role inspection cleanup failed']);
  assert.deepEqual(calls, [
    'inspect_operations_role_v1:failed',
    'inspect_operations_role_v1:active',
    'revoke_operations_role_v1:active',
  ]);
});

test('relief payment inspection accepts one complete non-negative count receipt', () => {
  assert.deepEqual(
    readReliefPaymentStateCounts([{
      applications_matched: 2,
      treasury_intents_found: 0,
      payment_receipts_found: 0,
    }]),
    {
      applicationsMatched: 2,
      treasuryIntentsFound: 0,
      paymentReceiptsFound: 0,
    },
  );
  for (const invalid of [
    null,
    [],
    [{ applications_matched: 2 }],
    [{
      applications_matched: 2,
      treasury_intents_found: -1,
      payment_receipts_found: 0,
    }],
    [{
      applications_matched: 2,
      treasury_intents_found: 0,
      payment_receipts_found: 0.5,
    }],
  ]) {
    assert.equal(readReliefPaymentStateCounts(invalid), null);
  }
});

test('relief workflow cleanup accepts one complete non-negative count receipt', () => {
  assert.deepEqual(
    readReliefWorkflowCleanupCounts([{
      public_updates_deleted: 1,
      events_deleted: 2,
      applications_deleted: 2,
    }]),
    {
      publicUpdatesDeleted: 1,
      eventsDeleted: 2,
      applicationsDeleted: 2,
    },
  );
  for (const invalid of [
    null,
    [],
    [{ public_updates_deleted: 1 }],
    [{ public_updates_deleted: 1, events_deleted: -2, applications_deleted: 2 }],
    [{ public_updates_deleted: 1, events_deleted: 2, applications_deleted: 1.5 }],
  ]) {
    assert.equal(readReliefWorkflowCleanupCounts(invalid), null);
  }
});

test('staging cleanup only accepts one returned row with the expected id', () => {
  const expectedId = '11111111-1111-4111-8111-111111111111';

  assert.equal(isExactCleanupDeletion([{ id: expectedId }], expectedId), true);
  assert.equal(isExactCleanupDeletion([], expectedId), false);
  assert.equal(isExactCleanupDeletion(null, expectedId), false);
  assert.equal(
    isExactCleanupDeletion(
      [{ id: expectedId }, { id: expectedId }],
      expectedId,
    ),
    false,
  );
  assert.equal(
    isExactCleanupDeletion(
      [{ id: '22222222-2222-4222-8222-222222222222' }],
      expectedId,
    ),
    false,
  );
});

test('risk workflow cleanup accepts one complete non-negative count receipt', () => {
  assert.deepEqual(
    readRiskWorkflowCleanupCounts([{
      publications_deleted: 1,
      events_deleted: 2,
      evidence_deleted: 1,
      reports_deleted: 2,
    }]),
    {
      publicationsDeleted: 1,
      eventsDeleted: 2,
      evidenceDeleted: 1,
      reportsDeleted: 2,
    },
  );
  for (const invalid of [
    null,
    [],
    [{ publications_deleted: 1 }],
    [{
      publications_deleted: 1,
      events_deleted: -2,
      evidence_deleted: 1,
      reports_deleted: 2,
    }],
    [{
      publications_deleted: 1,
      events_deleted: 2,
      evidence_deleted: 1.5,
      reports_deleted: 2,
    }],
  ]) {
    assert.equal(readRiskWorkflowCleanupCounts(invalid), null);
  }
});

test('task workflow cleanup accepts one complete non-negative count receipt', () => {
  assert.deepEqual(
    readTaskWorkflowCleanupCounts([{
      publications_deleted: 1,
      events_deleted: 4,
      submissions_deleted: 2,
      tasks_deleted: 1,
    }]),
    {
      publicationsDeleted: 1,
      eventsDeleted: 4,
      submissionsDeleted: 2,
      tasksDeleted: 1,
    },
  );

  for (const invalid of [
    null,
    [],
    [{ publications_deleted: 1 }],
    [{
      publications_deleted: -1,
      events_deleted: 4,
      submissions_deleted: 2,
      tasks_deleted: 1,
    }],
    [{
      publications_deleted: 1,
      events_deleted: 4.5,
      submissions_deleted: 2,
      tasks_deleted: 1,
    }],
    [{
      publications_deleted: 1,
      events_deleted: 4,
      submissions_deleted: 2,
      tasks_deleted: 1,
    }, {
      publications_deleted: 0,
      events_deleted: 0,
      submissions_deleted: 0,
      tasks_deleted: 0,
    }],
  ]) {
    assert.equal(readTaskWorkflowCleanupCounts(invalid), null);
  }
});
