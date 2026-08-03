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
