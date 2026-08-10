import {
  assertStagingSecretIsolation,
  isMainModule,
  loadOperationsStagingEnvironment,
  resolveOperationsStagingConfig,
  sanitizeStagingError,
  type OperationsStagingConfig,
} from './common.ts';

const PUBLIC_READ_PROBES = [
  ['community_tasks', 'id'],
  ['risk_publications', 'id'],
  ['relief_public_updates', 'id'],
  ['governance_proposals', 'id'],
  ['governance_discussion_publications', 'id'],
  ['governance_decisions', 'id'],
  ['treasury_execution_public_registry', 'intent_public_id'],
] as const;

const PRIVATE_DENIAL_PROBES = [
  ['task_submissions', 'id'],
  ['risk_reports', 'id'],
  ['risk_evidence', 'id'],
  ['relief_applications', 'id'],
  ['governance_discussions', 'id'],
  ['treasury_execution_intents', 'id'],
  ['treasury_execution_receipts', 'id'],
] as const;

export interface OperationsStagingPreflightResult {
  projectRef: string;
  authHealth: 'reachable';
  publicReadTables: number;
  privateAnonTablesDenied: number;
}

export async function runOperationsStagingPreflight(
  config: OperationsStagingConfig,
): Promise<OperationsStagingPreflightResult> {
  if (config.mode !== 'preflight') {
    throw new Error('read-only preflight 必须使用 preflight 配置模式');
  }

  const authHealth = await fetch(`${config.supabaseUrl}/auth/v1/health`, {
    method: 'GET',
    headers: {
      apikey: config.publicKey,
    },
    redirect: 'error',
  });

  if (!authHealth.ok) {
    throw new Error(`Supabase Auth health check 失败（HTTP ${authHealth.status}）`);
  }

  for (const [table, column] of PUBLIC_READ_PROBES) {
    const response = await readTable(config, table, column);
    if (!response.ok) {
      throw new Error(
        `公开只读探针 ${table} 失败（HTTP ${response.status}）；请确认 migrations 已完整应用`,
      );
    }
  }

  for (const [table, column] of PRIVATE_DENIAL_PROBES) {
    const response = await readTable(config, table, column);
    if (response.ok) {
      throw new Error(`匿名 key 意外获得私有表 ${table} 的 SELECT 权限`);
    }

    if (![401, 403].includes(response.status)) {
      throw new Error(
        `私有表 ${table} 返回非预期 HTTP ${response.status}；无法证明 anon 被拒绝`,
      );
    }
  }

  return {
    projectRef: config.projectRef,
    authHealth: 'reachable',
    publicReadTables: PUBLIC_READ_PROBES.length,
    privateAnonTablesDenied: PRIVATE_DENIAL_PROBES.length,
  };
}

async function readTable(
  config: OperationsStagingConfig,
  table: string,
  column: string,
): Promise<Response> {
  const url = new URL(`${config.supabaseUrl}/rest/v1/${table}`);
  url.searchParams.set('select', column);
  url.searchParams.set('limit', '1');

  return fetch(url, {
    method: 'GET',
    headers: {
      apikey: config.publicKey,
      Authorization: `Bearer ${config.publicKey}`,
      Accept: 'application/json',
    },
    redirect: 'error',
  });
}

async function main(): Promise<void> {
  const loaded = loadOperationsStagingEnvironment();
  assertStagingSecretIsolation(loaded.projectDirectory, loaded.envFile);
  const config = resolveOperationsStagingConfig(loaded.env, 'preflight');

  try {
    const result = await runOperationsStagingPreflight(config);
    console.log('Operations staging read-only preflight passed.');
    console.log(`Project ref: ${result.projectRef}`);
    console.log(`Public tables readable: ${result.publicReadTables}`);
    console.log(`Private tables denied to anon: ${result.privateAnonTablesDenied}`);
    console.log('No rows were inserted, updated, or deleted.');
  } catch (error) {
    throw new Error(sanitizeStagingError(error, config));
  }
}

if (isMainModule(import.meta.url)) {
  main().catch((error: unknown) => {
    console.error(`Operations staging preflight failed: ${sanitizeStagingError(error)}`);
    process.exitCode = 1;
  });
}
