import assert from 'node:assert/strict';
import test from 'node:test';
import {
  isExactCleanupDeletion,
  readTaskWorkflowCleanupCounts,
} from '../scripts/operations-staging/e2e.ts';

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
