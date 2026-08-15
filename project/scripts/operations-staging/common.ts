import { existsSync, readFileSync, realpathSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { normalizeTurnstileToken } from '../../src/features/operations/auth.ts';

export const OPERATIONS_STAGING_CONFIRMATION =
  'I_UNDERSTAND_THIS_CREATES_AND_DELETES_STAGING_TEST_DATA';
export const OPERATIONS_STAGING_GATE_ACTIVATION_CONFIRMATION =
  'I_UNDERSTAND_THIS_ENABLES_WALLET_AUTHENTICATED_STAGING_WRITES';
export const OPERATIONS_STAGING_GATE_DISABLE_CONFIRMATION =
  'I_UNDERSTAND_THIS_DISABLES_WALLET_AUTHENTICATED_STAGING_WRITES';
export const OPERATIONS_STAGING_ROLE_GRANT_CONFIRMATION =
  'I_UNDERSTAND_THIS_GRANTS_AUDITED_OPERATIONS_ACCESS_ON_STAGING';
export const OPERATIONS_STAGING_ROLE_REVOKE_CONFIRMATION =
  'I_UNDERSTAND_THIS_REVOKES_AUDITED_OPERATIONS_ACCESS_ON_STAGING';

export type OperationsStagingMode =
  | 'preflight'
  | 'release-inspect'
  | 'e2e'
  | 'gate-inspect'
  | 'gate-activate'
  | 'gate-disable'
  | 'roles-inspect'
  | 'roles-grant'
  | 'roles-revoke';

export interface OperationsStagingConfig {
  mode: OperationsStagingMode;
  projectRef: string;
  supabaseUrl: string;
  publicKey: string;
  serviceRoleKey: string | null;
  web3Url: string | null;
  e2eCaptchaToken: string | null;
  confirmedForWrites: boolean;
  confirmedForGateChange: boolean;
  gateChangeReference: string | null;
  confirmedForRoleGrant: boolean;
  confirmedForRoleRevoke: boolean;
  roleChangeReference: string | null;
}

export interface LoadedStagingEnvironment {
  projectDirectory: string;
  envFile: string;
  env: NodeJS.ProcessEnv;
}

export class OperationsStagingConfigError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'OperationsStagingConfigError';
  }
}

const PROJECT_REF_PATTERN = /^[a-z0-9]{20}$/;
const REQUIRED_PREFLIGHT_NAMES = [
  'OPERATIONS_STAGING_PROJECT_REF',
  'OPERATIONS_STAGING_SUPABASE_URL',
  'OPERATIONS_STAGING_PUBLIC_KEY',
] as const;

export function loadOperationsStagingEnvironment(
  cwd = process.cwd(),
  baseEnvironment: NodeJS.ProcessEnv = process.env,
): LoadedStagingEnvironment {
  const projectDirectory = resolveProjectDirectory(cwd);
  const envFile = resolve(projectDirectory, '.env.operations-staging');
  const env = { ...baseEnvironment };

  if (existsSync(envFile)) {
    const parsed = parseEnvironmentFile(readFileSync(envFile, 'utf8'));
    for (const [name, value] of Object.entries(parsed)) {
      if (env[name] === undefined) {
        env[name] = value;
      }
    }
  }

  return { projectDirectory, envFile, env };
}

export function resolveOperationsStagingConfig(
  env: NodeJS.ProcessEnv,
  mode: OperationsStagingMode,
): OperationsStagingConfig {
  const missing: string[] = REQUIRED_PREFLIGHT_NAMES.filter(
    (name) => !env[name]?.trim(),
  );
  const requiresServiceRole = mode !== 'preflight';
  if (requiresServiceRole && !env.OPERATIONS_STAGING_SERVICE_ROLE_KEY?.trim()) {
    missing.push('OPERATIONS_STAGING_SERVICE_ROLE_KEY');
  }
  if (mode === 'e2e' && !env.OPERATIONS_STAGING_WEB3_URL?.trim()) {
    missing.push('OPERATIONS_STAGING_WEB3_URL');
  }
  if (mode === 'e2e' && !env.OPERATIONS_STAGING_E2E_CAPTCHA_TOKEN?.trim()) {
    missing.push('OPERATIONS_STAGING_E2E_CAPTCHA_TOKEN');
  }
  if ((mode === 'gate-activate' || mode === 'gate-disable')
    && !env.OPERATIONS_STAGING_GATE_CHANGE_REFERENCE?.trim()) {
    missing.push('OPERATIONS_STAGING_GATE_CHANGE_REFERENCE');
  }
  if ((mode === 'roles-grant' || mode === 'roles-revoke')
    && !env.OPERATIONS_STAGING_ROLE_CHANGE_REFERENCE?.trim()) {
    missing.push('OPERATIONS_STAGING_ROLE_CHANGE_REFERENCE');
  }

  if (missing.length > 0) {
    throw new OperationsStagingConfigError(
      `Missing staging configuration: ${missing.join(', ')}`,
    );
  }

  const projectRef = env.OPERATIONS_STAGING_PROJECT_REF!.trim();
  const supabaseUrl = normalizeStagingUrl(env.OPERATIONS_STAGING_SUPABASE_URL!);
  const publicKey = env.OPERATIONS_STAGING_PUBLIC_KEY!.trim();
  const rawServiceKey = env.OPERATIONS_STAGING_SERVICE_ROLE_KEY?.trim() ?? '';
  const serviceRoleKey = rawServiceKey || null;
  const web3Url = env.OPERATIONS_STAGING_WEB3_URL
    ? normalizeWeb3Url(env.OPERATIONS_STAGING_WEB3_URL)
    : null;
  const e2eCaptchaToken = mode === 'e2e'
    ? normalizeE2ECaptchaToken(env.OPERATIONS_STAGING_E2E_CAPTCHA_TOKEN!)
    : null;

  if (!PROJECT_REF_PATTERN.test(projectRef)) {
    throw new OperationsStagingConfigError(
      'OPERATIONS_STAGING_PROJECT_REF must be exactly 20 lowercase letters or digits',
    );
  }

  const expectedOrigin = `https://${projectRef}.supabase.co`;
  if (supabaseUrl !== expectedOrigin) {
    throw new OperationsStagingConfigError(
      'Supabase URL must exactly match the explicit staging project reference',
    );
  }

  if (!isSupabasePublicKey(publicKey)) {
    throw new OperationsStagingConfigError(
      'OPERATIONS_STAGING_PUBLIC_KEY must be a publishable/anon key',
    );
  }

  if (serviceRoleKey && !isSupabaseServiceRoleKey(serviceRoleKey)) {
    throw new OperationsStagingConfigError(
      'OPERATIONS_STAGING_SERVICE_ROLE_KEY is not a recognized service-role key',
    );
  }

  if (serviceRoleKey === publicKey) {
    throw new OperationsStagingConfigError(
      'public and service-role keys must be kept in separate configuration slots',
    );
  }

  assertNoBrowserSecretExposure(env, serviceRoleKey);

  const confirmedForWrites =
    env.CONFIRM_OPERATIONS_STAGING_E2E === OPERATIONS_STAGING_CONFIRMATION;
  const isGateMutation = mode === 'gate-activate' || mode === 'gate-disable';
  const isRoleGrant = mode === 'roles-grant';
  const isRoleRevoke = mode === 'roles-revoke';
  const isRoleMutation = isRoleGrant || isRoleRevoke;
  const requiredGateConfirmation = mode === 'gate-activate'
    ? OPERATIONS_STAGING_GATE_ACTIVATION_CONFIRMATION
    : OPERATIONS_STAGING_GATE_DISABLE_CONFIRMATION;
  const gateConfirmationName = mode === 'gate-activate'
    ? 'CONFIRM_OPERATIONS_STAGING_GATE_ACTIVATION'
    : 'CONFIRM_OPERATIONS_STAGING_GATE_DISABLE';
  const confirmedForGateChange = isGateMutation
    && env[gateConfirmationName] === requiredGateConfirmation;
  const gateChangeReference = isGateMutation
    ? normalizeAuditReference(
      env.OPERATIONS_STAGING_GATE_CHANGE_REFERENCE,
      'OPERATIONS_STAGING_GATE_CHANGE_REFERENCE',
    )
    : null;
  const confirmedForRoleGrant = isRoleGrant
    && env.CONFIRM_OPERATIONS_STAGING_ROLE_GRANT
      === OPERATIONS_STAGING_ROLE_GRANT_CONFIRMATION;
  const confirmedForRoleRevoke = isRoleRevoke
    && env.CONFIRM_OPERATIONS_STAGING_ROLE_REVOKE
      === OPERATIONS_STAGING_ROLE_REVOKE_CONFIRMATION;
  const roleChangeReference = isRoleMutation
    ? normalizeAuditReference(
      env.OPERATIONS_STAGING_ROLE_CHANGE_REFERENCE,
      'OPERATIONS_STAGING_ROLE_CHANGE_REFERENCE',
    )
    : null;

  if (mode === 'e2e' && !confirmedForWrites) {
    throw new OperationsStagingConfigError(
      `Set CONFIRM_OPERATIONS_STAGING_E2E=${OPERATIONS_STAGING_CONFIRMATION} before mutating staging data`,
    );
  }

  if (isGateMutation && !confirmedForGateChange) {
    throw new OperationsStagingConfigError(
      `Set ${gateConfirmationName}=${requiredGateConfirmation} before changing the intake gate`,
    );
  }

  if (isRoleGrant && !confirmedForRoleGrant) {
    throw new OperationsStagingConfigError(
      `Set CONFIRM_OPERATIONS_STAGING_ROLE_GRANT=${OPERATIONS_STAGING_ROLE_GRANT_CONFIRMATION} before granting audited operations access`,
    );
  }

  if (isRoleRevoke && !confirmedForRoleRevoke) {
    throw new OperationsStagingConfigError(
      `Set CONFIRM_OPERATIONS_STAGING_ROLE_REVOKE=${OPERATIONS_STAGING_ROLE_REVOKE_CONFIRMATION} before revoking audited operations access`,
    );
  }

  return {
    mode,
    projectRef,
    supabaseUrl,
    publicKey,
    serviceRoleKey,
    web3Url,
    e2eCaptchaToken,
    confirmedForWrites,
    confirmedForGateChange,
    gateChangeReference,
    confirmedForRoleGrant,
    confirmedForRoleRevoke,
    roleChangeReference,
  };
}

export function assertStagingSecretIsolation(
  projectDirectory: string,
  envFile: string,
): void {
  const gitignorePath = resolve(projectDirectory, '.gitignore');
  if (!existsSync(gitignorePath)) {
    throw new OperationsStagingConfigError(
      'project/.gitignore is required before loading staging secrets',
    );
  }

  const ignoredEntries = readFileSync(gitignorePath, 'utf8')
    .split(/\r?\n/)
    .map((line) => line.trim());
  if (!ignoredEntries.includes('.env.operations-staging')) {
    throw new OperationsStagingConfigError(
      '.env.operations-staging must be ignored by project/.gitignore',
    );
  }

  if (!existsSync(envFile)) {
    return;
  }

  assertNoPersistedE2ECaptchaToken(readFileSync(envFile, 'utf8'));

  const examplePath = resolve(projectDirectory, '.env.operations-staging.example');
  if (!existsSync(examplePath)) {
    throw new OperationsStagingConfigError(
      'Missing .env.operations-staging.example template',
    );
  }
}

export function assertNoPersistedE2ECaptchaToken(contents: string): void {
  if (
    /^\s*(?:export\s+)?OPERATIONS_STAGING_E2E_CAPTCHA_TOKEN\s*=/mu.test(
      contents,
    )
  ) {
    throw new OperationsStagingConfigError(
      'OPERATIONS_STAGING_E2E_CAPTCHA_TOKEN must stay process-only and must not be written to .env.operations-staging',
    );
  }
}

export function sanitizeStagingError(
  error: unknown,
  config?: OperationsStagingConfig,
): string {
  let message = error instanceof Error ? error.message : String(error);
  const secrets = [
    config?.publicKey,
    config?.serviceRoleKey,
    config?.e2eCaptchaToken,
  ].filter((value): value is string => Boolean(value));

  for (const secret of secrets) {
    message = message.split(secret).join('[REDACTED]');
  }

  message = message
    .replace(/\bsb_secret_[A-Za-z0-9_-]+\b/g, '[REDACTED]')
    .replace(
      /\beyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\b/g,
      '[REDACTED_JWT]',
    );

  return message;
}

export function isMainModule(
  metaUrl: string,
  entry = process.argv[1],
): boolean {
  if (!entry) {
    return false;
  }

  let modulePath: string;
  try {
    modulePath = canonicalPath(fileURLToPath(metaUrl));
  } catch {
    return false;
  }

  const entryPath = canonicalPath(entry);
  if (process.platform === 'win32') {
    return modulePath.toLowerCase() === entryPath.toLowerCase();
  }

  return modulePath === entryPath;
}

function canonicalPath(value: string): string {
  const resolvedPath = resolve(value);
  try {
    return realpathSync.native(resolvedPath);
  } catch {
    return resolvedPath;
  }
}

function resolveProjectDirectory(cwd: string): string {
  const directPackage = resolve(cwd, 'package.json');
  if (existsSync(directPackage)) {
    return resolve(cwd);
  }

  const nestedProject = resolve(cwd, 'project');
  if (existsSync(resolve(nestedProject, 'package.json'))) {
    return nestedProject;
  }

  throw new OperationsStagingConfigError(
    'Run operations staging tools from the repository root or the project directory',
  );
}

function parseEnvironmentFile(contents: string): Record<string, string> {
  const parsed: Record<string, string> = {};

  for (const [index, rawLine] of contents.split(/\r?\n/).entries()) {
    const line = rawLine.trim();
    if (!line || line.startsWith('#')) {
      continue;
    }

    const normalized = line.startsWith('export ') ? line.slice(7).trim() : line;
    const separator = normalized.indexOf('=');
    if (separator <= 0) {
      throw new OperationsStagingConfigError(
        `.env.operations-staging line ${index + 1} is invalid`,
      );
    }

    const name = normalized.slice(0, separator).trim();
    let value = normalized.slice(separator + 1).trim();
    if (!/^[A-Z][A-Z0-9_]*$/.test(name)) {
      throw new OperationsStagingConfigError(
        `.env.operations-staging variable name on line ${index + 1} is invalid`,
      );
    }

    if (
      (value.startsWith('"') && value.endsWith('"'))
      || (value.startsWith("'") && value.endsWith("'"))
    ) {
      value = value.slice(1, -1);
    }

    parsed[name] = value;
  }

  return parsed;
}

function normalizeStagingUrl(rawUrl: string): string {
  let parsed: URL;
  try {
    parsed = new URL(rawUrl.trim());
  } catch {
    throw new OperationsStagingConfigError(
      'OPERATIONS_STAGING_SUPABASE_URL must be a valid URL',
    );
  }

  if (
    parsed.protocol !== 'https:'
    || parsed.username
    || parsed.password
    || parsed.search
    || parsed.hash
    || (parsed.pathname !== '/' && parsed.pathname !== '')
  ) {
    throw new OperationsStagingConfigError(
      'Staging Supabase URL must be a clean HTTPS origin without credentials, path, query, or hash',
    );
  }

  return parsed.origin;
}

function normalizeWeb3Url(rawUrl: string): string {
  let parsed: URL;
  try {
    parsed = new URL(rawUrl.trim());
  } catch {
    throw new OperationsStagingConfigError(
      'OPERATIONS_STAGING_WEB3_URL must be a valid URL',
    );
  }

  if (
    parsed.protocol !== 'https:'
    || parsed.username
    || parsed.password
    || parsed.search
    || parsed.hash
  ) {
    throw new OperationsStagingConfigError(
      'OPERATIONS_STAGING_WEB3_URL must be an HTTPS page without credentials, query, or hash',
    );
  }

  return parsed.href;
}

function normalizeE2ECaptchaToken(rawToken: string): string {
  try {
    return normalizeTurnstileToken(rawToken);
  } catch {
    throw new OperationsStagingConfigError(
      'OPERATIONS_STAGING_E2E_CAPTCHA_TOKEN is invalid',
    );
  }
}

function normalizeAuditReference(
  rawReference: string | undefined,
  label: string,
): string {
  const reference = rawReference?.trim() ?? '';
  if (
    reference.length < 10
    || reference.length > 200
    || [...reference].some((character) => {
      const codePoint = character.codePointAt(0) ?? 0;
      return codePoint <= 31 || codePoint === 127;
    })
  ) {
    throw new OperationsStagingConfigError(
      `${label} must be 10 to 200 characters without control characters`,
    );
  }

  return reference;
}

function assertNoBrowserSecretExposure(
  env: NodeJS.ProcessEnv,
  serviceRoleKey: string | null,
): void {
  for (const [name, value] of Object.entries(env)) {
    if (!name.startsWith('VITE_') || !value) {
      continue;
    }

    if (
      value === serviceRoleKey
      || isSupabaseServiceRoleKey(value)
      || name.includes('SERVICE_ROLE')
      || name.includes('SECRET')
    ) {
      throw new OperationsStagingConfigError(
        `Detected staging secret exposure in browser environment variable ${name}`,
      );
    }
  }
}

function isSupabasePublicKey(value: string): boolean {
  const trimmed = value.trim();
  if (trimmed.startsWith('sb_secret_')) {
    return false;
  }

  if (trimmed.startsWith('sb_publishable_')) {
    return trimmed.length >= 24;
  }

  return readSupabaseJwtRole(trimmed) === 'anon';
}

function isSupabaseServiceRoleKey(value: string): boolean {
  const trimmed = value.trim();
  if (trimmed.startsWith('sb_publishable_')) {
    return false;
  }

  if (trimmed.startsWith('sb_secret_')) {
    return trimmed.length >= 16;
  }

  return readSupabaseJwtRole(trimmed) === 'service_role';
}

function readSupabaseJwtRole(value: string): string | null {
  const parts = value.split('.');
  if (
    parts.length !== 3
    || parts.some((part) => !/^[A-Za-z0-9_-]+$/.test(part))
  ) {
    return null;
  }

  let payload: unknown;
  try {
    payload = JSON.parse(Buffer.from(parts[1], 'base64url').toString('utf8'));
  } catch {
    return null;
  }

  if (!payload || typeof payload !== 'object' || Array.isArray(payload)) {
    return null;
  }

  const role = (payload as { role?: unknown }).role;
  return typeof role === 'string' ? role : null;
}
