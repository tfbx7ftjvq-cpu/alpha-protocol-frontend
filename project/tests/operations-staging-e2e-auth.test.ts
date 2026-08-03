import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

const e2eSource = readFileSync(
  new URL('../scripts/operations-staging/e2e.ts', import.meta.url),
  'utf8',
);

test('CAPTCHA-compatible E2E uses one-time admin links for non-wallet roles', () => {
  assert.match(e2eSource, /admin\.auth\.admin\.generateLink\(\{/);
  assert.match(e2eSource, /type: 'magiclink'/);
  assert.match(e2eSource, /client\.auth\.verifyOtp\(\{/);
  assert.match(e2eSource, /token_hash: hashedToken/);
  assert.doesNotMatch(e2eSource, /signInWithPassword/);
});

test('CAPTCHA-compatible E2E consumes one transient token for one Web3 actor', () => {
  assert.equal(
    (e2eSource.match(/await createWalletActor\(admin, config\)/g) ?? []).length,
    1,
  );
  assert.match(e2eSource, /captchaToken: config\.e2eCaptchaToken/);
  assert.match(
    e2eSource,
    /const switchedWalletAddress = Keypair\.generate\(\)\.publicKey\.toBase58\(\)/,
  );
  assert.match(e2eSource, /const crossUserRead = await emailOnlyOwner\.client/);
  assert.match(
    e2eSource,
    /const unauthorizedModeration = await emailOnlyOwner\.client/,
  );
  assert.doesNotMatch(e2eSource, /ownerB/);
});

test('Phase 4M E2E exercises audited task RPCs and controlled fixture cleanup', () => {
  assert.match(e2eSource, /rpc\(\s*'publish_community_task_v1'/);
  assert.equal(
    (e2eSource.match(/rpc\(\s*'review_task_submission_v1'/g) ?? []).length,
    5,
  );
  assert.match(
    e2eSource,
    /rpc\(\s*'cleanup_operations_task_staging_e2e_v1'/,
  );
  assert.match(e2eSource, /const directTaskInsert = await operator\.client/);
  assert.match(
    e2eSource,
    /operator bypassed the audited task publication RPC/,
  );
  assert.match(e2eSource, /primaryError === null/);
  assert.match(e2eSource, /counts\.publicationsDeleted !== 1/);
  assert.match(e2eSource, /counts\.eventsDeleted !== 4/);
  assert.match(e2eSource, /counts\.submissionsDeleted !== 2/);
  assert.match(e2eSource, /counts\.tasksDeleted !== 1/);
});

test('Phase 4M E2E refreshes role claims and verifies sanitized public output', () => {
  assert.match(e2eSource, /admin\.auth\.admin\.updateUserById/);
  assert.match(e2eSource, /actor\.client\.auth\.refreshSession\(\)/);
  assert.match(e2eSource, /public_wallet_consent: false/);
  assert.match(e2eSource, /!\('submission_id' in publicResult\)/);
  assert.match(e2eSource, /!\('submitted_by' in publicResult\)/);
  assert.match(e2eSource, /!\('reviewed_by' in publicResult\)/);
  assert.match(e2eSource, /task workflow audit did not contain the exact four expected actions/);
});
