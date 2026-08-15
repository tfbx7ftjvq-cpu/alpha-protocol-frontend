import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import {
  assertReleaseManifest,
  type ReleaseManifest,
} from './release-manifest.ts';
import { isMainModule } from './operations-staging/common.ts';

const SECRET_LIKE_CONTENT = /\b(?:sb_secret_[A-Za-z0-9_-]+|eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+|service[_-]?role|turnstile[^\s"']{0,80}|private[_-]?key|mnemonic|seed\s+phrase)\b/i;

export interface ReleaseArtifactVerificationResult {
  manifest: ReleaseManifest;
  headersPresent: true;
}

export function verifyReleaseArtifacts(
  distDirectory = resolve(process.cwd(), 'dist'),
): ReleaseArtifactVerificationResult {
  const manifestPath = resolve(distDirectory, 'release.json');
  const headersPath = resolve(distDirectory, '_headers');
  if (!existsSync(manifestPath)) {
    throw new Error('release artifact is missing dist/release.json');
  }
  if (!existsSync(headersPath)) {
    throw new Error('release artifact is missing dist/_headers');
  }

  const manifestContents = readFileSync(manifestPath, 'utf8');
  const headersContents = readFileSync(headersPath, 'utf8');
  assertNoSecretLikeContent(manifestContents, 'release manifest');
  assertNoSecretLikeContent(headersContents, 'headers artifact');

  let manifestValue: unknown;
  try {
    manifestValue = JSON.parse(manifestContents);
  } catch {
    throw new Error('release artifact is not valid JSON');
  }
  assertReleaseManifest(manifestValue);

  return { manifest: manifestValue, headersPresent: true };
}

export function assertNoSecretLikeContent(contents: string, label: string): void {
  if (SECRET_LIKE_CONTENT.test(contents)) {
    throw new Error(`${label} contains secret-like content`);
  }
}

function main(): void {
  const result = verifyReleaseArtifacts();
  console.log('Release artifacts verified.');
  console.log(`Commit: ${result.manifest.commitSha}`);
  console.log(`Build context: ${result.manifest.buildContext}`);
}

if (isMainModule(import.meta.url)) {
  main();
}
