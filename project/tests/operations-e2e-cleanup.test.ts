import assert from 'node:assert/strict';
import test from 'node:test';
import { isExactCleanupDeletion } from '../scripts/operations-staging/e2e.ts';

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
