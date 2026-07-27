import assert from 'node:assert/strict';
import test from 'node:test';
import { resolveOperationsBackendConfig } from '../src/lib/operationsSupabase.ts';

const PUBLIC_KEY = 'sb_publishable_public_browser_key_example_123456';

test('operations backend is fail-closed when public configuration is missing', () => {
  const config = resolveOperationsBackendConfig('', '', 'anonymous');

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

test('anonymous intake requires an exact explicit mode', () => {
  const enabled = resolveOperationsBackendConfig(
    'https://example.supabase.co',
    PUBLIC_KEY,
    'anonymous',
  );
  const typo = resolveOperationsBackendConfig(
    'https://example.supabase.co',
    PUBLIC_KEY,
    'Anonymous',
  );

  assert.equal(enabled.intakeEnabled, true);
  assert.equal(typo.intakeEnabled, false);
});

test('browser configuration rejects secret and service-role keys', () => {
  const secret = resolveOperationsBackendConfig(
    'https://example.supabase.co',
    ['sb', 'secret', 'test-only-value-without-credentials'].join('_'),
    'anonymous',
  );
  const serviceRoleJwt = [
    base64Url({ alg: 'HS256', typ: 'JWT' }),
    base64Url({ role: 'service_role' }),
    'signature',
  ].join('.');
  const serviceRole = resolveOperationsBackendConfig(
    'https://example.supabase.co',
    serviceRoleJwt,
    'anonymous',
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
    const config = resolveOperationsBackendConfig(url, PUBLIC_KEY, 'anonymous');
    assert.equal(config.configured, false);
  }
});

function base64Url(value: object): string {
  return Buffer.from(JSON.stringify(value))
    .toString('base64url');
}
