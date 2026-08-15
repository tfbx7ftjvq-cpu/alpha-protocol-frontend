import {
  assertStagingSecretIsolation,
  isMainModule,
  loadOperationsStagingEnvironment,
  OperationsStagingConfigError,
  resolveOperationsStagingConfig,
  sanitizeStagingError,
  type OperationsStagingConfig,
} from './common.ts';
import { runOperationsIntakeGateAction } from './intake-gate.ts';
import { runOperationsStagingPreflight } from './preflight.ts';
import {
  assertReleaseManifest,
  type ReleaseManifest,
} from '../release-manifest.ts';

const COMMIT_SHA_PATTERN = /^[0-9a-f]{40}$/;

export const REQUIRED_PAGES_HEADERS = {
  'x-content-type-options': 'nosniff',
  'referrer-policy': 'strict-origin-when-cross-origin',
  'x-frame-options': 'DENY',
  'permissions-policy': 'camera=(), microphone=(), geolocation=()',
  'cross-origin-opener-policy': 'same-origin-allow-popups',
} as const;

export interface OperationsReleaseReadinessConfig {
  staging: OperationsStagingConfig;
  pagesUrl: string;
  expectedCommitSha: string;
}

export interface OperationsReleaseReadinessResult {
  projectRef: string;
  deployedCommitSha: string;
  gateMode: 'disabled' | 'wallet_staging';
  publicReadTables: number;
  privateAnonTablesDenied: number;
}

export function resolveOperationsReleaseReadinessConfig(
  environment: NodeJS.ProcessEnv,
): OperationsReleaseReadinessConfig {
  const staging = resolveOperationsStagingConfig(environment, 'release-inspect');
  const pagesUrl = normalizePagesUrl(environment.OPERATIONS_STAGING_PAGES_URL);
  const expectedCommitSha = normalizeExpectedCommit(
    environment.OPERATIONS_STAGING_EXPECTED_COMMIT_SHA,
  );
  return { staging, pagesUrl, expectedCommitSha };
}

export async function runOperationsReleaseReadiness(
  config: OperationsReleaseReadinessConfig,
): Promise<OperationsReleaseReadinessResult> {
  const homeResponse = await fetch(config.pagesUrl, {
    method: 'GET',
    redirect: 'error',
  });
  await assertPagesHomeResponse(homeResponse);
  assertRequiredPagesHeaders(homeResponse.headers);

  const releaseResponse = await fetch(new URL('release.json', config.pagesUrl), {
    method: 'GET',
    headers: { Accept: 'application/json' },
    redirect: 'error',
  });
  const manifest = await readReleaseManifestResponse(releaseResponse);
  assertReleaseCacheControl(releaseResponse.headers);
  assertExpectedDeployedCommit(manifest.commitSha, config.expectedCommitSha);

  const preflight = await runOperationsStagingPreflight({
    ...config.staging,
    mode: 'preflight',
  });
  const gate = await runOperationsIntakeGateAction({
    ...config.staging,
    mode: 'gate-inspect',
  }, 'inspect');

  return {
    projectRef: preflight.projectRef,
    deployedCommitSha: manifest.commitSha,
    gateMode: gate.snapshot.mode,
    publicReadTables: preflight.publicReadTables,
    privateAnonTablesDenied: preflight.privateAnonTablesDenied,
  };
}

export function normalizePagesUrl(value: string | undefined): string {
  if (!value?.trim()) {
    throw new OperationsStagingConfigError('Missing staging configuration: OPERATIONS_STAGING_PAGES_URL');
  }

  let parsed: URL;
  try {
    parsed = new URL(value.trim());
  } catch {
    throw new OperationsStagingConfigError('OPERATIONS_STAGING_PAGES_URL must be a valid URL');
  }
  if (
    parsed.protocol !== 'https:'
    || parsed.username
    || parsed.password
    || parsed.search
    || parsed.hash
  ) {
    throw new OperationsStagingConfigError(
      'OPERATIONS_STAGING_PAGES_URL must be an HTTPS page without credentials, query, or fragment',
    );
  }
  return parsed.href;
}

export function normalizeExpectedCommit(value: string | undefined): string {
  const commit = value?.trim() ?? '';
  if (!COMMIT_SHA_PATTERN.test(commit)) {
    throw new OperationsStagingConfigError(
      'OPERATIONS_STAGING_EXPECTED_COMMIT_SHA must be exactly 40 lowercase hexadecimal characters',
    );
  }
  return commit;
}

export async function assertPagesHomeResponse(response: Response): Promise<void> {
  if (response.status !== 200) {
    throw new Error(`Pages home check failed with HTTP ${response.status}`);
  }
  const contentType = response.headers.get('content-type') ?? '';
  if (!contentType.toLowerCase().includes('text/html')) {
    throw new Error('Pages home check did not return HTML');
  }
  const body = await response.clone().text();
  if (isCloudflareErrorHtml(body)) {
    throw new Error('Pages home check returned Cloudflare error HTML');
  }
}

export function assertRequiredPagesHeaders(headers: Headers): void {
  for (const [name, expected] of Object.entries(REQUIRED_PAGES_HEADERS)) {
    if (headers.get(name) !== expected) {
      throw new Error(`Pages response is missing required security header: ${name}`);
    }
  }
}

export async function readReleaseManifestResponse(response: Response): Promise<ReleaseManifest> {
  if (response.status !== 200) {
    throw new Error(`release.json check failed with HTTP ${response.status}`);
  }
  const contentType = response.headers.get('content-type') ?? '';
  if (!contentType.toLowerCase().includes('application/json')) {
    throw new Error('release.json did not return JSON');
  }

  let value: unknown;
  try {
    value = await response.json();
  } catch {
    throw new Error('release.json is not valid JSON');
  }
  assertReleaseManifest(value);
  if (!COMMIT_SHA_PATTERN.test(value.commitSha)) {
    throw new Error('release.json commitSha must be a deployed 40-character lowercase hexadecimal commit');
  }
  return value;
}

export function assertReleaseCacheControl(headers: Headers): void {
  if (headers.get('cache-control') !== 'no-store') {
    throw new Error('release.json is missing Cache-Control: no-store');
  }
}

export function assertExpectedDeployedCommit(
  deployedCommitSha: string,
  expectedCommitSha: string,
): void {
  if (deployedCommitSha !== expectedCommitSha) {
    throw new Error('deployed release commit does not match OPERATIONS_STAGING_EXPECTED_COMMIT_SHA');
  }
}

function isCloudflareErrorHtml(body: string): boolean {
  return /cloudflare/iu.test(body) && /(?:error\s+code\s+5\d\d|\b5(?:20|21|22|23|24|25|26|27)\b)/iu.test(body);
}

async function main(): Promise<void> {
  const loaded = loadOperationsStagingEnvironment();
  assertStagingSecretIsolation(loaded.projectDirectory, loaded.envFile);
  const config = resolveOperationsReleaseReadinessConfig(loaded.env);

  try {
    const result = await runOperationsReleaseReadiness(config);
    console.log('Operations release readiness inspection passed.');
    console.log(`Project ref: ${result.projectRef}`);
    console.log(`Deployed commit: ${result.deployedCommitSha}`);
    console.log(`Public tables readable: ${result.publicReadTables}`);
    console.log(`Private tables denied to anon: ${result.privateAnonTablesDenied}`);
    console.log(`Wallet intake gate mode: ${result.gateMode}`);
    console.log('No rows, users, roles, gate state, or Solana state were changed.');
  } catch (error) {
    throw new Error(sanitizeStagingError(error, config.staging));
  }
}

if (isMainModule(import.meta.url)) {
  main().catch((error: unknown) => {
    console.error(`Operations release readiness inspection failed: ${sanitizeStagingError(error)}`);
    process.exitCode = 1;
  });
}
