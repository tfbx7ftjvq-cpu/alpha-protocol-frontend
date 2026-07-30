import assert from 'node:assert/strict';
import { resolve } from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';
import {
  isMainModule,
  OPERATIONS_STAGING_CONFIRMATION,
  OperationsStagingConfigError,
  resolveOperationsStagingConfig,
  sanitizeStagingError,
} from '../scripts/operations-staging/common.ts';

const PROJECT_REF = 'abcdefghijklmnopqrst';
const PUBLIC_KEY = 'sb_publishable_staging_public_key_123456';
const SERVICE_KEY = [
  'sb',
  'secret',
  'staging-test-value-without-credentials',
].join('_');
const WEB3_URL = 'https://staging.alpha.example/operations';

function validEnvironment(): NodeJS.ProcessEnv {
  return {
    OPERATIONS_STAGING_PROJECT_REF: PROJECT_REF,
    OPERATIONS_STAGING_SUPABASE_URL: `https://${PROJECT_REF}.supabase.co`,
    OPERATIONS_STAGING_PUBLIC_KEY: PUBLIC_KEY,
  };
}

test('CLI entry detection is fail-closed and tolerates Windows path casing', () => {
  const entry = resolve('scripts/operations-staging/e2e.ts');
  const entryUrl = pathToFileURL(entry).href;
  const differentEntryUrl = pathToFileURL(
    resolve('scripts/operations-staging/preflight.ts'),
  ).href;
  const caseVariantUrl = entryUrl.replace(
    '/scripts/operations-staging/',
    '/SCRIPTS/OPERATIONS-STAGING/',
  );

  assert.equal(isMainModule(entryUrl, entry), true);
  assert.equal(isMainModule(differentEntryUrl, entry), false);
  if (process.platform === 'win32') {
    assert.equal(isMainModule(caseVariantUrl, entry), true);
  }
  assert.equal(isMainModule('not-a-file-url', entry), false);
  assert.equal(isMainModule(entryUrl, undefined), false);
});

test('staging preflight fails closed when required configuration is missing', () => {
  assert.throws(
    () => resolveOperationsStagingConfig({}, 'preflight'),
    OperationsStagingConfigError,
  );
});

test('staging URL must exactly match the explicit project reference', () => {
  assert.throws(
    () => resolveOperationsStagingConfig({
      ...validEnvironment(),
      OPERATIONS_STAGING_SUPABASE_URL: 'https://differentprojectref1.supabase.co',
    }, 'preflight'),
    /project ref 不一致/,
  );

  const config = resolveOperationsStagingConfig(validEnvironment(), 'preflight');
  assert.equal(config.projectRef, PROJECT_REF);
  assert.equal(config.supabaseUrl, `https://${PROJECT_REF}.supabase.co`);
});

test('public and service-role keys stay in separate configuration slots', () => {
  assert.throws(
    () => resolveOperationsStagingConfig({
      ...validEnvironment(),
      OPERATIONS_STAGING_PUBLIC_KEY: SERVICE_KEY,
    }, 'preflight'),
    /publishable\/anon key/,
  );

  assert.throws(
    () => resolveOperationsStagingConfig({
      ...validEnvironment(),
      OPERATIONS_STAGING_SERVICE_ROLE_KEY: PUBLIC_KEY,
      OPERATIONS_STAGING_WEB3_URL: WEB3_URL,
      CONFIRM_OPERATIONS_STAGING_E2E: OPERATIONS_STAGING_CONFIRMATION,
    }, 'e2e'),
    /service-role key/,
  );
});

test('mutating E2E requires an exact confirmation and rejects VITE secret exposure', () => {
  const e2eEnvironment = {
    ...validEnvironment(),
    OPERATIONS_STAGING_SERVICE_ROLE_KEY: SERVICE_KEY,
    OPERATIONS_STAGING_WEB3_URL: WEB3_URL,
  };

  assert.throws(
    () => resolveOperationsStagingConfig(e2eEnvironment, 'e2e'),
    /CONFIRM_OPERATIONS_STAGING_E2E=/,
  );

  assert.throws(
    () => resolveOperationsStagingConfig({
      ...e2eEnvironment,
      CONFIRM_OPERATIONS_STAGING_E2E: OPERATIONS_STAGING_CONFIRMATION,
      VITE_ACCIDENTAL_SERVICE_ROLE_KEY: SERVICE_KEY,
    }, 'e2e'),
    /浏览器环境变量/,
  );

  const config = resolveOperationsStagingConfig({
    ...e2eEnvironment,
    CONFIRM_OPERATIONS_STAGING_E2E: OPERATIONS_STAGING_CONFIRMATION,
  }, 'e2e');
  assert.equal(config.confirmedForWrites, true);
  assert.equal(config.web3Url, WEB3_URL);
});

test('wallet E2E URL must be an exact HTTPS page without credentials or query data', () => {
  for (const web3Url of [
    'http://staging.alpha.example/operations',
    'https://user:pass@staging.alpha.example/operations',
    'https://staging.alpha.example/operations?token=unsafe',
  ]) {
    assert.throws(
      () => resolveOperationsStagingConfig({
        ...validEnvironment(),
        OPERATIONS_STAGING_SERVICE_ROLE_KEY: SERVICE_KEY,
        OPERATIONS_STAGING_WEB3_URL: web3Url,
        CONFIRM_OPERATIONS_STAGING_E2E: OPERATIONS_STAGING_CONFIRMATION,
      }, 'e2e'),
      /OPERATIONS_STAGING_WEB3_URL/,
    );
  }
});

test('staging errors redact service keys and JWT-shaped credentials', () => {
  const config = resolveOperationsStagingConfig({
    ...validEnvironment(),
    OPERATIONS_STAGING_SERVICE_ROLE_KEY: SERVICE_KEY,
    OPERATIONS_STAGING_WEB3_URL: WEB3_URL,
    CONFIRM_OPERATIONS_STAGING_E2E: OPERATIONS_STAGING_CONFIRMATION,
  }, 'e2e');
  const jwt = [
    'eyJhbGciOiJIUzI1NiJ9',
    'eyJyb2xlIjoic2VydmljZV9yb2xlIn0',
    'signature_value',
  ].join('.');
  const sanitized = sanitizeStagingError(
    new Error(`request leaked ${SERVICE_KEY} and ${jwt}`),
    config,
  );

  assert.doesNotMatch(sanitized, /sb_secret_/);
  assert.doesNotMatch(sanitized, /eyJhbGci/);
  assert.match(sanitized, /\[REDACTED\]/);
});
