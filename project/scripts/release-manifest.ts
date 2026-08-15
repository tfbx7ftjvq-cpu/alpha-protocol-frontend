export const RELEASE_SCHEMA_VERSION = 1;

export type ReleaseBuildContext =
  | 'cloudflare-pages'
  | 'github-actions'
  | 'local-explicit'
  | 'local';

export interface ReleaseManifest {
  schemaVersion: typeof RELEASE_SCHEMA_VERSION;
  commitSha: string;
  branch: string;
  buildContext: ReleaseBuildContext;
}

const COMMIT_SHA_PATTERN = /^[0-9a-f]{40}$/;
const BRANCH_PATTERN = /^[A-Za-z0-9][A-Za-z0-9._/-]{0,127}$/;

export function resolveReleaseManifest(
  environment: NodeJS.ProcessEnv = process.env,
): ReleaseManifest {
  const commit = resolveCommit(environment);
  const branch = resolveBranch(environment);

  return {
    schemaVersion: RELEASE_SCHEMA_VERSION,
    commitSha: commit.value,
    branch,
    buildContext: commit.context,
  };
}

export function assertReleaseManifest(value: unknown): asserts value is ReleaseManifest {
  if (!isPlainObject(value)) {
    throw new Error('release manifest must be a JSON object');
  }

  const keys = Object.keys(value).sort();
  const expectedKeys = ['branch', 'buildContext', 'commitSha', 'schemaVersion'];
  if (keys.length !== expectedKeys.length || keys.some((key, index) => key !== expectedKeys[index])) {
    throw new Error('release manifest contains unsupported fields');
  }
  if (value.schemaVersion !== RELEASE_SCHEMA_VERSION) {
    throw new Error('release manifest schemaVersion is invalid');
  }
  if (!isReleaseCommit(value.commitSha)) {
    throw new Error('release manifest commitSha is invalid');
  }
  if (typeof value.branch !== 'string' || !isReleaseBranch(value.branch)) {
    throw new Error('release manifest branch is invalid');
  }
  if (!isReleaseBuildContext(value.buildContext)) {
    throw new Error('release manifest buildContext is invalid');
  }
}

export function isReleaseCommit(value: unknown): value is string {
  return value === 'local' || (typeof value === 'string' && COMMIT_SHA_PATTERN.test(value));
}

export function isReleaseBranch(value: string): boolean {
  return value === 'local' || BRANCH_PATTERN.test(value);
}

function resolveCommit(environment: NodeJS.ProcessEnv): {
  value: string;
  context: ReleaseBuildContext;
} {
  const candidates: Array<{ value: string | undefined; context: ReleaseBuildContext }> = [
    { value: environment.CF_PAGES_COMMIT_SHA, context: 'cloudflare-pages' },
    { value: environment.GITHUB_SHA, context: 'github-actions' },
    { value: environment.RELEASE_COMMIT_SHA, context: 'local-explicit' },
  ];

  for (const candidate of candidates) {
    const value = candidate.value?.trim();
    if (!value) {
      continue;
    }
    if (!COMMIT_SHA_PATTERN.test(value)) {
      throw new Error(`${candidate.context} commit SHA must be 40 lowercase hexadecimal characters`);
    }
    return { value, context: candidate.context };
  }

  return { value: 'local', context: 'local' };
}

function resolveBranch(environment: NodeJS.ProcessEnv): string {
  const branch = environment.CF_PAGES_BRANCH?.trim()
    || environment.GITHUB_REF_NAME?.trim()
    || 'local';
  if (!isReleaseBranch(branch)) {
    throw new Error('release branch must be a safe non-secret branch name');
  }
  return branch;
}

function isReleaseBuildContext(value: unknown): value is ReleaseBuildContext {
  return value === 'cloudflare-pages'
    || value === 'github-actions'
    || value === 'local-explicit'
    || value === 'local';
}

function isPlainObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}
