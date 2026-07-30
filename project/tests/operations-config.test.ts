import assert from 'node:assert/strict';
import test from 'node:test';
import {
  isExactOperationsWeb3Page,
  resolveOperationsBackendConfig,
} from '../src/lib/operationsSupabase.ts';

const PUBLIC_KEY = 'sb_publishable_public_browser_key_example_123456';
const PROJECT_REF = 'abcdefghijklmnopqrst';
const PROJECT_URL = `https://${PROJECT_REF}.supabase.co`;
const WEB3_URL = 'https://staging.alpha.example/operations';

test('operations backend is fail-closed when public configuration is missing', () => {
  const config = resolveOperationsBackendConfig(
    '',
    '',
    'wallet-staging',
    PROJECT_REF,
    WEB3_URL,
  );

  assert.equal(config.configured, false);
  assert.equal(config.publicReadEnabled, false);
  assert.equal(config.intakeEnabled, false);
});

test('valid public configuration enables reads but keeps intake disabled by default', () => {
  const config = resolveOperationsBackendConfig(
    'https://example.supabase.co',
    PUBLIC_KEY,
    'disabled',
  );

  assert.equal(config.configured, true);
  assert.equal(config.publicReadEnabled, true);
  assert.equal(config.intakeEnabled, false);
});

test('wallet staging intake requires exact mode and project binding', () => {
  const enabled = resolveOperationsBackendConfig(
    PROJECT_URL,
    PUBLIC_KEY,
    'wallet-staging',
    PROJECT_REF,
    WEB3_URL,
  );
  const typo = resolveOperationsBackendConfig(
    PROJECT_URL,
    PUBLIC_KEY,
    'Wallet-Staging',
    PROJECT_REF,
    WEB3_URL,
  );
  const mismatchedProject = resolveOperationsBackendConfig(
    PROJECT_URL,
    PUBLIC_KEY,
    'wallet-staging',
    'differentprojectref1',
    WEB3_URL,
  );
  const oldAnonymousMode = resolveOperationsBackendConfig(
    PROJECT_URL,
    PUBLIC_KEY,
    'anonymous',
    PROJECT_REF,
    WEB3_URL,
  );

  assert.equal(enabled.intakeEnabled, true);
  assert.equal(typo.intakeEnabled, false);
  assert.equal(mismatchedProject.publicReadEnabled, true);
  assert.equal(mismatchedProject.intakeEnabled, false);
  assert.equal(oldAnonymousMode.intakeEnabled, false);
});

test('wallet staging intake requires an exact HTTPS Web3 page binding', () => {
  for (const web3Url of [
    '',
    'http://staging.alpha.example/operations',
    'https://user:pass@staging.alpha.example/operations',
    'https://staging.alpha.example/operations?token=unsafe',
    'not-a-url',
  ]) {
    const config = resolveOperationsBackendConfig(
      PROJECT_URL,
      PUBLIC_KEY,
      'wallet-staging',
      PROJECT_REF,
      web3Url,
    );
    assert.equal(config.publicReadEnabled, true);
    assert.equal(config.intakeEnabled, false);
  }

  const enabled = resolveOperationsBackendConfig(
    PROJECT_URL,
    PUBLIC_KEY,
    'wallet-staging',
    PROJECT_REF,
    WEB3_URL,
  );
  assert.equal(enabled.web3Url, WEB3_URL);
  assert.equal(enabled.intakeEnabled, true);
});

test('wallet signing page stays bound to the configured origin and path', () => {
  assert.equal(isExactOperationsWeb3Page(
    WEB3_URL,
    `${WEB3_URL}?tab=tasks#intake`,
  ), true);
  assert.equal(isExactOperationsWeb3Page(
    WEB3_URL,
    'https://staging.alpha.example/other',
  ), false);
  assert.equal(isExactOperationsWeb3Page(
    WEB3_URL,
    'https://lookalike.example/operations',
  ), false);
  assert.equal(isExactOperationsWeb3Page(WEB3_URL, 'not-a-url'), false);
});

test('browser configuration rejects secret and service-role keys', () => {
  const secret = resolveOperationsBackendConfig(
    'https://example.supabase.co',
    ['sb', 'secret', 'test-only-value-without-credentials'].join('_'),
    'wallet-staging',
    PROJECT_REF,
    WEB3_URL,
  );
  const serviceRoleJwt = [
    base64Url({ alg: 'HS256', typ: 'JWT' }),
    base64Url({ role: 'service_role' }),
    'signature',
  ].join('.');
  const serviceRole = resolveOperationsBackendConfig(
    'https://example.supabase.co',
    serviceRoleJwt,
    'wallet-staging',
    PROJECT_REF,
    WEB3_URL,
  );

  assert.equal(secret.configured, false);
  assert.equal(serviceRole.configured, false);
  assert.equal(secret.intakeEnabled, false);
  assert.equal(serviceRole.intakeEnabled, false);
});

test('backend URL must be HTTPS and contain no credentials', () => {
  for (const url of [
    'http://example.supabase.co',
    'https://user:pass@example.supabase.co',
    'not-a-url',
  ]) {
    const config = resolveOperationsBackendConfig(
      url,
      PUBLIC_KEY,
      'wallet-staging',
      PROJECT_REF,
      WEB3_URL,
    );
    assert.equal(config.configured, false);
  }
});

function base64Url(value: object): string {
  return Buffer.from(JSON.stringify(value))
    .toString('base64url');
}
