import assert from 'node:assert/strict';
import { resolve } from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';
import {
  assertNoPersistedE2ECaptchaToken,
  isMainModule,
  OPERATIONS_STAGING_CONFIRMATION,
  OPERATIONS_STAGING_GATE_ACTIVATION_CONFIRMATION,
  OPERATIONS_STAGING_GATE_DISABLE_CONFIRMATION,
  OPERATIONS_STAGING_ROLE_GRANT_CONFIRMATION,
  OPERATIONS_STAGING_ROLE_REVOKE_CONFIRMATION,
  OperationsStagingConfigError,
  resolveOperationsStagingConfig,
  sanitizeStagingError,
} from '../scripts/operations-staging/common.ts';
import { runOperationsStagingPreflight } from '../scripts/operations-staging/preflight.ts';

const PROJECT_REF = 'abcdefghijklmnopqrst';
const PUBLIC_KEY = 'sb_publishable_staging_public_key_123456';
const SERVICE_KEY = [
  'sb',
  'secret',
  'staging-test-value-without-credentials',
].join('_');
const WEB3_URL = 'https://staging.alpha.example/operations';
const CAPTCHA_TOKEN = 'turnstile-response-token-for-staging-e2e';

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
    /project ref|staging project reference/i,
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
      OPERATIONS_STAGING_E2E_CAPTCHA_TOKEN: CAPTCHA_TOKEN,
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
    OPERATIONS_STAGING_E2E_CAPTCHA_TOKEN: CAPTCHA_TOKEN,
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
    /browser environment variable|secret exposure/i,
  );

  const config = resolveOperationsStagingConfig({
    ...e2eEnvironment,
    CONFIRM_OPERATIONS_STAGING_E2E: OPERATIONS_STAGING_CONFIRMATION,
  }, 'e2e');
  assert.equal(config.confirmedForWrites, true);
  assert.equal(config.web3Url, WEB3_URL);
  assert.equal(config.e2eCaptchaToken, CAPTCHA_TOKEN);
});

test('mutating E2E requires a fresh process-only CAPTCHA token', () => {
  const baseEnvironment = {
    ...validEnvironment(),
    OPERATIONS_STAGING_SERVICE_ROLE_KEY: SERVICE_KEY,
    OPERATIONS_STAGING_WEB3_URL: WEB3_URL,
    CONFIRM_OPERATIONS_STAGING_E2E: OPERATIONS_STAGING_CONFIRMATION,
  };

  assert.throws(
    () => resolveOperationsStagingConfig(baseEnvironment, 'e2e'),
    /OPERATIONS_STAGING_E2E_CAPTCHA_TOKEN/,
  );
  assert.throws(
    () => resolveOperationsStagingConfig({
      ...baseEnvironment,
      OPERATIONS_STAGING_E2E_CAPTCHA_TOKEN: 'too-short',
    }, 'e2e'),
    /CAPTCHA_TOKEN|is invalid/i,
  );

  const config = resolveOperationsStagingConfig({
    ...baseEnvironment,
    OPERATIONS_STAGING_E2E_CAPTCHA_TOKEN: CAPTCHA_TOKEN,
  }, 'e2e');
  assert.equal(config.e2eCaptchaToken, CAPTCHA_TOKEN);

  assert.throws(
    () => assertNoPersistedE2ECaptchaToken(
      `OPERATIONS_STAGING_E2E_CAPTCHA_TOKEN=${CAPTCHA_TOKEN}`,
    ),
    /process-only|must not be written/i,
  );
  assert.doesNotThrow(() => assertNoPersistedE2ECaptchaToken(
    '# OPERATIONS_STAGING_E2E_CAPTCHA_TOKEN must remain process-only',
  ));
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
        OPERATIONS_STAGING_E2E_CAPTCHA_TOKEN: CAPTCHA_TOKEN,
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
    OPERATIONS_STAGING_E2E_CAPTCHA_TOKEN: CAPTCHA_TOKEN,
    CONFIRM_OPERATIONS_STAGING_E2E: OPERATIONS_STAGING_CONFIRMATION,
  }, 'e2e');
  const jwt = [
    'eyJhbGciOiJIUzI1NiJ9',
    'eyJyb2xlIjoic2VydmljZV9yb2xlIn0',
    'signature_value',
  ].join('.');
  const sanitized = sanitizeStagingError(
    new Error(`request leaked ${SERVICE_KEY}, ${CAPTCHA_TOKEN}, and ${jwt}`),
    config,
  );

  assert.doesNotMatch(sanitized, /sb_secret_/);
  assert.doesNotMatch(sanitized, new RegExp(CAPTCHA_TOKEN));
  assert.doesNotMatch(sanitized, /eyJhbGci/);
  assert.match(sanitized, /\[REDACTED\]/);
});

test('gate inspection requires the isolated service-role credential but no mutation confirmation', () => {
  assert.throws(
    () => resolveOperationsStagingConfig(validEnvironment(), 'gate-inspect'),
    /OPERATIONS_STAGING_SERVICE_ROLE_KEY/,
  );

  const config = resolveOperationsStagingConfig({
    ...validEnvironment(),
    OPERATIONS_STAGING_SERVICE_ROLE_KEY: SERVICE_KEY,
  }, 'gate-inspect');
  assert.equal(config.confirmedForGateChange, false);
  assert.equal(config.gateChangeReference, null);
});

test('gate activation and emergency disable require distinct exact confirmations', () => {
  const baseGateEnvironment = {
    ...validEnvironment(),
    OPERATIONS_STAGING_SERVICE_ROLE_KEY: SERVICE_KEY,
    OPERATIONS_STAGING_GATE_CHANGE_REFERENCE: 'phase-4l reviewed staging change',
  };

  assert.throws(
    () => resolveOperationsStagingConfig(baseGateEnvironment, 'gate-activate'),
    /CONFIRM_OPERATIONS_STAGING_GATE_ACTIVATION=/,
  );
  assert.throws(
    () => resolveOperationsStagingConfig({
      ...baseGateEnvironment,
      CONFIRM_OPERATIONS_STAGING_GATE_ACTIVATION:
        OPERATIONS_STAGING_GATE_DISABLE_CONFIRMATION,
    }, 'gate-activate'),
    /CONFIRM_OPERATIONS_STAGING_GATE_ACTIVATION=/,
  );

  const activation = resolveOperationsStagingConfig({
    ...baseGateEnvironment,
    CONFIRM_OPERATIONS_STAGING_GATE_ACTIVATION:
      OPERATIONS_STAGING_GATE_ACTIVATION_CONFIRMATION,
  }, 'gate-activate');
  assert.equal(activation.confirmedForGateChange, true);
  assert.equal(
    activation.gateChangeReference,
    'phase-4l reviewed staging change',
  );

  const disable = resolveOperationsStagingConfig({
    ...baseGateEnvironment,
    CONFIRM_OPERATIONS_STAGING_GATE_DISABLE:
      OPERATIONS_STAGING_GATE_DISABLE_CONFIRMATION,
  }, 'gate-disable');
  assert.equal(disable.confirmedForGateChange, true);
});

test('roles inspection requires service-role without mutation confirmation', () => {
  assert.throws(
    () => resolveOperationsStagingConfig(validEnvironment(), 'roles-inspect'),
    /OPERATIONS_STAGING_SERVICE_ROLE_KEY/,
  );

  const config = resolveOperationsStagingConfig({
    ...validEnvironment(),
    OPERATIONS_STAGING_SERVICE_ROLE_KEY: SERVICE_KEY,
  }, 'roles-inspect');
  assert.equal(config.confirmedForRoleGrant, false);
  assert.equal(config.confirmedForRoleRevoke, false);
  assert.equal(config.roleChangeReference, null);
});

test('roles grant and revoke require distinct exact confirmations and audit references', () => {
  const baseEnvironment = {
    ...validEnvironment(),
    OPERATIONS_STAGING_SERVICE_ROLE_KEY: SERVICE_KEY,
    OPERATIONS_STAGING_ROLE_CHANGE_REFERENCE:
      'phase-2e-6e audited staging role change',
  };

  assert.throws(
    () => resolveOperationsStagingConfig(baseEnvironment, 'roles-grant'),
    /CONFIRM_OPERATIONS_STAGING_ROLE_GRANT=/,
  );
  assert.throws(
    () => resolveOperationsStagingConfig({
      ...baseEnvironment,
      CONFIRM_OPERATIONS_STAGING_ROLE_GRANT:
        OPERATIONS_STAGING_ROLE_REVOKE_CONFIRMATION,
    }, 'roles-grant'),
    /CONFIRM_OPERATIONS_STAGING_ROLE_GRANT=/,
  );
  assert.throws(
    () => resolveOperationsStagingConfig({
      ...baseEnvironment,
      CONFIRM_OPERATIONS_STAGING_ROLE_GRANT:
        OPERATIONS_STAGING_ROLE_GRANT_CONFIRMATION,
      OPERATIONS_STAGING_ROLE_CHANGE_REFERENCE: 'short',
    }, 'roles-grant'),
    /OPERATIONS_STAGING_GATE_CHANGE_REFERENCE|OPERATIONS_STAGING_ROLE_CHANGE_REFERENCE|10/,
  );

  const grant = resolveOperationsStagingConfig({
    ...baseEnvironment,
    CONFIRM_OPERATIONS_STAGING_ROLE_GRANT:
      OPERATIONS_STAGING_ROLE_GRANT_CONFIRMATION,
  }, 'roles-grant');
  assert.equal(grant.confirmedForRoleGrant, true);
  assert.equal(
    grant.roleChangeReference,
    'phase-2e-6e audited staging role change',
  );

  const revoke = resolveOperationsStagingConfig({
    ...baseEnvironment,
    CONFIRM_OPERATIONS_STAGING_ROLE_REVOKE:
      OPERATIONS_STAGING_ROLE_REVOKE_CONFIRMATION,
  }, 'roles-revoke');
  assert.equal(revoke.confirmedForRoleRevoke, true);
});

test('gate changes reject missing, short, oversized, and control-character audit references', () => {
  for (const reference of [
    undefined,
    'too-short',
    'x'.repeat(201),
    'reviewed change\nsecond line',
  ]) {
    assert.throws(
      () => resolveOperationsStagingConfig({
        ...validEnvironment(),
        OPERATIONS_STAGING_SERVICE_ROLE_KEY: SERVICE_KEY,
        OPERATIONS_STAGING_GATE_CHANGE_REFERENCE: reference,
        CONFIRM_OPERATIONS_STAGING_GATE_ACTIVATION:
          OPERATIONS_STAGING_GATE_ACTIVATION_CONFIRMATION,
      }, 'gate-activate'),
      /GATE_CHANGE_REFERENCE/,
    );
  }
});

test('staging preflight reads the public treasury registry and denies anon receipt reads', async () => {
  const config = resolveOperationsStagingConfig(validEnvironment(), 'preflight');
  const originalFetch = globalThis.fetch;
  const calls: string[] = [];
  const privateTables = new Set([
    'task_submissions',
    'risk_reports',
    'risk_evidence',
    'relief_applications',
    'governance_discussions',
    'treasury_execution_intents',
    'treasury_execution_receipts',
  ]);

  globalThis.fetch = async (input) => {
    const url = typeof input === 'string'
      ? input
      : input instanceof URL
      ? input.href
      : input.url;
    calls.push(url);

    if (url.endsWith('/auth/v1/health')) {
      return new Response(null, { status: 200 });
    }

    const parsed = new URL(url);
    const table = parsed.pathname.split('/').at(-1);
    if (table && privateTables.has(table)) {
      return new Response('{"message":"permission denied"}', { status: 403 });
    }

    return new Response('[]', { status: 200 });
  };

  try {
    const result = await runOperationsStagingPreflight(config);
    assert.equal(result.publicReadTables, 7);
    assert.equal(result.privateAnonTablesDenied, 7);
    assert.ok(
      calls.some((url) =>
        url.includes('/rest/v1/treasury_execution_public_registry')
        && url.includes('select=intent_public_id'),
      ),
    );
    assert.ok(
      calls.some((url) => url.includes('/rest/v1/treasury_execution_receipts')),
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});
