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

test('Phase 4M E2E grants audited roles and verifies sanitized public output', () => {
  assert.match(e2eSource, /grant_operations_role_v1/);
  assert.match(e2eSource, /get_my_operations_access_v1/);
  assert.match(e2eSource, /public_wallet_consent: false/);
  assert.match(e2eSource, /!\('submission_id' in publicResult\)/);
  assert.match(e2eSource, /!\('submitted_by' in publicResult\)/);
  assert.match(e2eSource, /!\('reviewed_by' in publicResult\)/);
  assert.match(e2eSource, /task workflow audit did not contain the exact four expected actions/);
});

test('Phase 4N E2E exercises independent risk review and sanitized publication', () => {
  assert.equal(
    (e2eSource.match(/rpc\(\s*'review_risk_report_v1'/g) ?? []).length,
    5,
  );
  assert.match(e2eSource, /const selfRiskReview = await ownerA\.client/);
  assert.match(e2eSource, /const unauthorizedRiskReview = await emailOnlyOwner\.client/);
  assert.match(e2eSource, /const directRiskRewrite = await operator\.client/);
  assert.match(e2eSource, /!\('submitted_by' in riskPublication\)/);
  assert.match(e2eSource, /!\('wallet_address' in riskPublication\)/);
  assert.match(e2eSource, /!\('reviewer_notes' in riskPublication\)/);
  assert.match(e2eSource, /dismissed report created a publication/);
  assert.match(e2eSource, /terminal risk review was replayed/);
});

test('Phase 4N E2E uses exact risk fixture cleanup', () => {
  assert.match(
    e2eSource,
    /rpc\(\s*'cleanup_operations_risk_staging_e2e_v1'/,
  );
  assert.match(e2eSource, /counts\.publicationsDeleted !== 1/);
  assert.match(e2eSource, /counts\.eventsDeleted !== 2/);
  assert.match(e2eSource, /counts\.evidenceDeleted !== 1/);
  assert.match(e2eSource, /counts\.reportsDeleted !== 2/);
  assert.match(e2eSource, /primaryError === null/);

});

test('Phase 4O E2E proves relief approval is not payment and cleans exact fixtures', () => {
  assert.equal(
    (e2eSource.match(/rpc\(\s*'review_relief_application_v1'/g) ?? []).length,
    5,
  );
  assert.match(e2eSource, /const selfReliefReview = await ownerA\.client/);
  assert.match(e2eSource, /const unauthorizedReliefReview = await emailOnlyOwner\.client/);
  assert.match(e2eSource, /const directReliefRewrite = await operator\.client/);
  assert.match(e2eSource, /!\('wallet_address' in publicRelief\)/);
  assert.match(e2eSource, /!\('requested_amount_usdc' in publicRelief\)/);
  assert.match(e2eSource, /relief review created a treasury intent or payment receipt/);
  assert.match(
    e2eSource,
    /rpc\(\s*'inspect_operations_relief_staging_e2e_payment_state_v1'/,
  );
  assert.doesNotMatch(
    e2eSource,
    /admin\s*\.from\(\s*'treasury_execution_intents'/,
  );
  assert.match(
    e2eSource,
    /rpc\(\s*'cleanup_operations_relief_staging_e2e_v1'/,
  );
  assert.match(e2eSource, /counts\.publicUpdatesDeleted !== 1/);
  assert.match(e2eSource, /counts\.eventsDeleted !== 2/);
  assert.match(e2eSource, /counts\.applicationsDeleted !== 2/);

  const increments = [...e2eSource.matchAll(/assertions \+= (\d+);/g)]
    .reduce((total, match) => total + Number(match[1]), 0);
  assert.equal(increments, 61);
});

test('governance discussion E2E uses an independent moderator and audited RPCs', () => {
  assert.match(e2eSource, /const moderator = await createActor\(admin, config, runId, 'moderator', 'moderator'\)/);
  assert.match(e2eSource, /ownerA\.client\.rpc\('submit_governance_discussion_v1'/);
  assert.match(e2eSource, /moderator\.client\.rpc\('review_governance_discussion_v1'/);
  assert.match(e2eSource, /operator read a moderator-only private discussion/);
  assert.doesNotMatch(e2eSource, /operator\.client[\s\S]{0,80}\.from\('governance_discussions'\)[\s\S]{0,120}\.update\(/);
});

test('governance discussion cleanup is exact RPC-only and owner-bound', () => {
  assert.match(e2eSource, /cleanup_governance_discussion_staging_e2e_v1/);
  assert.match(e2eSource, /p_owner_id: rows\.discussionOwnerId/);
  assert.match(e2eSource, /p_discussion_id: rows\.discussionId/);
  assert.doesNotMatch(e2eSource, /\.from\('governance_discussions'\)\s*\.delete\(/);
});
