import assert from 'node:assert/strict';
import test from 'node:test';
import {
  isExactCleanupDeletion,
  readReliefPaymentStateCounts,
  readReliefWorkflowCleanupCounts,
  readRiskWorkflowCleanupCounts,
  readTaskWorkflowCleanupCounts,
} from '../scripts/operations-staging/e2e.ts';

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
