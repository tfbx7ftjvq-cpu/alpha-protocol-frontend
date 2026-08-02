import { existsSync, readFileSync, realpathSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

export const OPERATIONS_STAGING_CONFIRMATION =
  'I_UNDERSTAND_THIS_CREATES_AND_DELETES_STAGING_TEST_DATA';
export const OPERATIONS_STAGING_GATE_ACTIVATION_CONFIRMATION =
  'I_UNDERSTAND_THIS_ENABLES_WALLET_AUTHENTICATED_STAGING_WRITES';
export const OPERATIONS_STAGING_GATE_DISABLE_CONFIRMATION =
  'I_UNDERSTAND_THIS_DISABLES_WALLET_AUTHENTICATED_STAGING_WRITES';

export type OperationsStagingMode =
  | 'preflight'
  | 'e2e'
  | 'gate-inspect'
  | 'gate-activate'
  | 'gate-disable';

export interface OperationsStagingConfig {
  mode: OperationsStagingMode;
  projectRef: string;
  supabaseUrl: string;
  publicKey: string;
  serviceRoleKey: string | null;
  web3Url: string | null;
  confirmedForWrites: boolean;
  confirmedForGateChange: boolean;
  gateChangeReference: string | null;
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

  if (missing.length > 0) {
    throw new OperationsStagingConfigError(
      `缺少 staging 配置：${missing.join(', ')}`,
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

  if (!PROJECT_REF_PATTERN.test(projectRef)) {
    throw new OperationsStagingConfigError(
      'OPERATIONS_STAGING_PROJECT_REF 必须是 20 位小写字母或数字',
    );
  }

  const expectedOrigin = `https://${projectRef}.supabase.co`;
  if (supabaseUrl !== expectedOrigin) {
    throw new OperationsStagingConfigError(
      'Supabase URL 与明确指定的 staging project ref 不一致',
    );
  }

  if (!isSupabasePublicKey(publicKey)) {
    throw new OperationsStagingConfigError(
      'OPERATIONS_STAGING_PUBLIC_KEY 必须是 publishable/anon key，不能是 secret/service-role key',
    );
  }

  if (serviceRoleKey && !isSupabaseServiceRoleKey(serviceRoleKey)) {
    throw new OperationsStagingConfigError(
      'OPERATIONS_STAGING_SERVICE_ROLE_KEY 不是可识别的 secret/service-role key',
    );
  }

  if (serviceRoleKey === publicKey) {
    throw new OperationsStagingConfigError('公开 key 与 service-role key 不得相同');
  }

  assertNoBrowserSecretExposure(env, serviceRoleKey);

  const confirmedForWrites =
    env.CONFIRM_OPERATIONS_STAGING_E2E === OPERATIONS_STAGING_CONFIRMATION;
  const isGateMutation = mode === 'gate-activate' || mode === 'gate-disable';
  const requiredGateConfirmation = mode === 'gate-activate'
    ? OPERATIONS_STAGING_GATE_ACTIVATION_CONFIRMATION
    : OPERATIONS_STAGING_GATE_DISABLE_CONFIRMATION;
  const gateConfirmationName = mode === 'gate-activate'
    ? 'CONFIRM_OPERATIONS_STAGING_GATE_ACTIVATION'
    : 'CONFIRM_OPERATIONS_STAGING_GATE_DISABLE';
  const confirmedForGateChange = isGateMutation
    && env[gateConfirmationName] === requiredGateConfirmation;
  const gateChangeReference = isGateMutation
    ? normalizeGateChangeReference(env.OPERATIONS_STAGING_GATE_CHANGE_REFERENCE)
    : null;

  if (mode === 'e2e' && !confirmedForWrites) {
    throw new OperationsStagingConfigError(
      `执行真实 staging 写入前必须设置 CONFIRM_OPERATIONS_STAGING_E2E=${OPERATIONS_STAGING_CONFIRMATION}`,
    );
  }

  if (isGateMutation && !confirmedForGateChange) {
    throw new OperationsStagingConfigError(
      `执行数据库总闸门变更前必须设置 ${gateConfirmationName}=${requiredGateConfirmation}`,
    );
  }

  return {
    mode,
    projectRef,
    supabaseUrl,
    publicKey,
    serviceRoleKey,
    web3Url,
    confirmedForWrites,
    confirmedForGateChange,
    gateChangeReference,
  };
}

export function assertStagingSecretIsolation(
  projectDirectory: string,
  envFile: string,
): void {
  const gitignorePath = resolve(projectDirectory, '.gitignore');
  if (!existsSync(gitignorePath)) {
    throw new OperationsStagingConfigError('project/.gitignore 不存在，拒绝加载 staging secret');
  }

  const ignoredEntries = readFileSync(gitignorePath, 'utf8')
    .split(/\r?\n/)
    .map((line) => line.trim());

  if (!ignoredEntries.includes('.env.operations-staging')) {
    throw new OperationsStagingConfigError(
      '.env.operations-staging 未在 project/.gitignore 中精确忽略',
    );
  }

  if (!existsSync(envFile)) {
    return;
  }

  const examplePath = resolve(projectDirectory, '.env.operations-staging.example');
  if (!existsSync(examplePath)) {
    throw new OperationsStagingConfigError('staging 环境变量模板缺失');
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
    '请从仓库根目录或 project 目录运行 operations staging 工具',
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
        `.env.operations-staging 第 ${index + 1} 行格式无效`,
      );
    }

    const name = normalized.slice(0, separator).trim();
    let value = normalized.slice(separator + 1).trim();
    if (!/^[A-Z][A-Z0-9_]*$/.test(name)) {
      throw new OperationsStagingConfigError(
        `.env.operations-staging 第 ${index + 1} 行变量名无效`,
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
    throw new OperationsStagingConfigError('OPERATIONS_STAGING_SUPABASE_URL 不是有效 URL');
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
      'Staging Supabase URL 必须是无凭据、无路径参数的 HTTPS origin',
    );
  }

  return parsed.origin;
}

function normalizeWeb3Url(rawUrl: string): string {
  let parsed: URL;
  try {
    parsed = new URL(rawUrl.trim());
  } catch {
    throw new OperationsStagingConfigError('OPERATIONS_STAGING_WEB3_URL 不是有效 URL');
  }

  if (
    parsed.protocol !== 'https:'
    || parsed.username
    || parsed.password
    || parsed.search
    || parsed.hash
  ) {
    throw new OperationsStagingConfigError(
      'OPERATIONS_STAGING_WEB3_URL 必须是无凭据、无 query/hash 的 HTTPS URL',
    );
  }

  return parsed.href;
}

function normalizeGateChangeReference(rawReference: string | undefined): string {
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
      'OPERATIONS_STAGING_GATE_CHANGE_REFERENCE 必须是 10–200 字符且不含控制字符的审计引用',
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
        `检测到浏览器环境变量 ${name} 暴露 staging secret，已拒绝继续`,
      );
    }
  }
}

function isSupabasePublicKey(value: string): boolean {
  if (value.startsWith('sb_secret_') || value.toLowerCase().includes('service_role')) {
    return false;
  }

  if (value.startsWith('sb_publishable_')) {
    return value.length >= 24;
  }

  return decodeJwtRole(value) === 'anon';
}

function isSupabaseServiceRoleKey(value: string): boolean {
  if (value.startsWith('sb_secret_')) {
    return value.length >= 20;
  }

  return decodeJwtRole(value) === 'service_role';
}

function decodeJwtRole(value: string): string | null {
  const parts = value.split('.');
  if (parts.length !== 3) {
    return null;
  }

  try {
    const payload = JSON.parse(
      Buffer.from(parts[1], 'base64url').toString('utf8'),
    ) as { role?: unknown };
    return typeof payload.role === 'string' ? payload.role : null;
  } catch {
    return null;
  }
}
