import assert from 'node:assert/strict';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import test from 'node:test';
import {
  assertNoSecretLikeContent,
  verifyReleaseArtifacts,
} from '../scripts/release-artifact-verify.ts';
import {
  assertReleaseManifest,
  resolveReleaseManifest,
} from '../scripts/release-manifest.ts';
import {
  assertExpectedDeployedCommit,
  assertReleaseCacheControl,
  assertPagesHomeResponse,
  assertRequiredPagesHeaders,
  normalizeExpectedCommit,
  normalizePagesUrl,
  readReleaseManifestResponse,
  REQUIRED_PAGES_HEADERS,
} from '../scripts/operations-staging/release-readiness.ts';

const COMMIT = '0123456789abcdef0123456789abcdef01234567';
const RELEASE_MANIFEST = {
  schemaVersion: 1,
  commitSha: COMMIT,
  branch: 'main',
  buildContext: 'github-actions',
};

test('release manifest uses Cloudflare, GitHub, explicit local, then local fallback precedence', () => {
  assert.deepEqual(resolveReleaseManifest({
    CF_PAGES_COMMIT_SHA: COMMIT,
    GITHUB_SHA: 'abcdefabcdefabcdefabcdefabcdefabcdefabcd',
    RELEASE_COMMIT_SHA: 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
    CF_PAGES_BRANCH: 'production',
    GITHUB_REF_NAME: 'main',
  }), {
    schemaVersion: 1,
    commitSha: COMMIT,
    branch: 'production',
    buildContext: 'cloudflare-pages',
  });
  assert.deepEqual(resolveReleaseManifest({
    GITHUB_SHA: COMMIT,
    GITHUB_REF_NAME: 'main',
  }), RELEASE_MANIFEST);
  assert.deepEqual(resolveReleaseManifest({
    RELEASE_COMMIT_SHA: COMMIT,
  }), {
    schemaVersion: 1,
    commitSha: COMMIT,
    branch: 'local',
    buildContext: 'local-explicit',
  });
  assert.deepEqual(resolveReleaseManifest({}), {
    schemaVersion: 1,
    commitSha: 'local',
    branch: 'local',
    buildContext: 'local',
  });
});

test('release manifest fails closed for invalid commits and unsupported fields', () => {
  assert.throws(
    () => resolveReleaseManifest({ CF_PAGES_COMMIT_SHA: COMMIT.toUpperCase() }),
    /40 lowercase hexadecimal/,
  );
  assert.throws(
    () => resolveReleaseManifest({ GITHUB_SHA: 'too-short' }),
    /40 lowercase hexadecimal/,
  );
  assert.throws(
    () => assertReleaseManifest({ ...RELEASE_MANIFEST, serviceRoleKey: 'unsafe' }),
    /unsupported fields/,
  );
});

test('release readiness URL and expected commit validation are fail-closed', () => {
  assert.equal(
    normalizePagesUrl('https://alpha-protocol-frontend.pages.dev/'),
    'https://alpha-protocol-frontend.pages.dev/',
  );
  for (const url of [
    'http://alpha-protocol-frontend.pages.dev/',
    'https://user:pass@alpha-protocol-frontend.pages.dev/',
    'https://alpha-protocol-frontend.pages.dev/?unsafe=true',
    'https://alpha-protocol-frontend.pages.dev/#unsafe',
  ]) {
    assert.throws(() => normalizePagesUrl(url), /HTTPS page/);
  }
  assert.equal(normalizeExpectedCommit(COMMIT), COMMIT);
  assert.throws(() => normalizeExpectedCommit(COMMIT.toUpperCase()), /40 lowercase hexadecimal/);
  assert.doesNotThrow(() => assertExpectedDeployedCommit(COMMIT, COMMIT));
  assert.throws(
    () => assertExpectedDeployedCommit(COMMIT, 'abcdefabcdefabcdefabcdefabcdefabcdefabcd'),
    /does not match/,
  );
});

test('release readiness distinguishes healthy Pages HTML, Cloudflare errors, release schema, and headers', async () => {
  const headers = new Headers({
    'content-type': 'text/html; charset=utf-8',
    ...REQUIRED_PAGES_HEADERS,
  });
  await assertPagesHomeResponse(new Response('<html>Alpha</html>', { status: 200, headers }));
  assert.doesNotThrow(() => assertRequiredPagesHeaders(headers));
  await assert.rejects(
    () => assertPagesHomeResponse(new Response('Cloudflare error code 522', {
      status: 200,
      headers: { 'content-type': 'text/html' },
    })),
    /Cloudflare error HTML/,
  );
  assert.throws(
    () => assertRequiredPagesHeaders(new Headers()),
    /security header/,
  );
  assert.deepEqual(
    await readReleaseManifestResponse(new Response(JSON.stringify(RELEASE_MANIFEST), {
      status: 200,
      headers: { 'content-type': 'application/json', 'cache-control': 'no-store' },
    })),
    RELEASE_MANIFEST,
  );
  assert.doesNotThrow(() => assertReleaseCacheControl(new Headers({
    'cache-control': 'no-store',
  })));
  assert.throws(() => assertReleaseCacheControl(new Headers()), /Cache-Control/);
  await assert.rejects(
    () => readReleaseManifestResponse(new Response(JSON.stringify({ schemaVersion: 1 }), {
      status: 200,
      headers: { 'content-type': 'application/json' },
    })),
    /unsupported fields/,
  );
});

test('release artifact verifier requires both artifacts and rejects secret-like content', () => {
  const directory = mkdtempSync(resolve(tmpdir(), 'alpha-release-artifact-'));
  try {
    writeFileSync(resolve(directory, 'release.json'), `${JSON.stringify(RELEASE_MANIFEST)}\n`);
    assert.throws(() => verifyReleaseArtifacts(directory), /dist\/_headers/);
    writeFileSync(resolve(directory, '_headers'), '/*\n  X-Frame-Options: DENY\n');
    assert.equal(verifyReleaseArtifacts(directory).headersPresent, true);
    assert.throws(
      () => assertNoSecretLikeContent('sb_secret_example_value', 'test'),
      /secret-like/,
    );
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test('release readiness remains read-only and CI runs the full gate once without credentials', async () => {
  const readinessSource = await readFile(
    new URL('../scripts/operations-staging/release-readiness.ts', import.meta.url),
    'utf8',
  );
  const workflow = await readFile(
    new URL('../../.github/workflows/operations-release-verify.yml', import.meta.url),
    'utf8',
  );

  assert.doesNotMatch(readinessSource, /\.(?:insert|update|delete)\(/);
  assert.doesNotMatch(readinessSource, /createUser|deleteUser|grant_operations_role|revoke_operations_role|sendTransaction/i);
  assert.match(workflow, /permissions:\s*\n\s+contents: read/);
  assert.match(workflow, /node-version: 22/);
  assert.equal((workflow.match(/npm run release:verify/g) ?? []).length, 1);
  assert.doesNotMatch(workflow, /secrets:|operations:staging:(?:preflight|e2e|gate|roles)/);
});
