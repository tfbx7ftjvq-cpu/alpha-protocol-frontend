import {
  createClient,
  type SupabaseClient,
} from '@supabase/supabase-js';
import {
  assertStagingSecretIsolation,
  isMainModule,
  loadOperationsStagingEnvironment,
  resolveOperationsStagingConfig,
  sanitizeStagingError,
  type OperationsStagingConfig,
} from './common.ts';

export type OperationsRoleAction = 'inspect' | 'grant' | 'revoke';
export const OPERATIONS_ROLE_NAMES = [
  'reviewer',
  'relief_reviewer',
  'operator',
  'moderator',
  'governance_admin',
  'treasury_preparer',
  'treasury_authorizer',
  'executor',
  'treasury_reconciler',
] as const;

export type OperationsRoleName =
  typeof OPERATIONS_ROLE_NAMES[number];

export interface OperationsRoleSnapshot {
  userId: string;
  roleName: string | null;
  status: string | null;
  expiresAt: string | null;
}

export interface OperationsRoleActionResult {
  projectRef: string;
  action: OperationsRoleAction;
  changed: boolean;
  snapshot: OperationsRoleSnapshot;
}

export async function runOperationsRoleAction(
  config: OperationsStagingConfig,
  action: OperationsRoleAction,
  userId: string,
  roleName?: OperationsRoleName,
): Promise<OperationsRoleActionResult> {
  if (!config.serviceRoleKey) {
    throw new Error('service-role key is required for operations role tooling');
  }

  const client = createClient(config.supabaseUrl, config.serviceRoleKey, {
    auth: {
      autoRefreshToken: false,
      detectSessionInUrl: false,
      persistSession: false,
    },
  });

  let changed = false;
  if (action === 'grant') {
    if (!config.confirmedForRoleGrant || !config.roleChangeReference || !roleName) {
      throw new Error('grant requires an exact confirmation, role name, and change reference');
    }
    const result = await client.rpc('grant_operations_role_v1', {
      p_user_id: userId,
      p_role_name: roleName,
      p_grant_reference: config.roleChangeReference,
    });
    if (result.error) {
      throw new Error(`grant_operations_role_v1 failed: ${result.error.message}`);
    }
    changed = true;
  }

  if (action === 'revoke') {
    if (!config.confirmedForRoleRevoke || !config.roleChangeReference) {
      throw new Error('revoke requires an exact confirmation and change reference');
    }
    const result = await client.rpc('revoke_operations_role_v1', {
      p_user_id: userId,
      p_revoke_reference: config.roleChangeReference,
    });
    if (result.error) {
      throw new Error(`revoke_operations_role_v1 failed: ${result.error.message}`);
    }
    changed = true;
  }

  const snapshot = await inspectOperationsRole(client, userId);
  return {
    projectRef: config.projectRef,
    action,
    changed,
    snapshot,
  };
}

async function inspectOperationsRole(
  client: SupabaseClient,
  userId: string,
): Promise<OperationsRoleSnapshot> {
  const result = await client.rpc('inspect_operations_role_v1', {
    p_user_id: userId,
  });
  if (result.error) {
    throw new Error(`inspect_operations_role_v1 failed: ${result.error.message}`);
  }
  if (!Array.isArray(result.data)) {
    throw new Error('inspect_operations_role_v1 must return an array');
  }
  if (result.data.length === 0) {
    return {
      userId,
      roleName: null,
      status: null,
      expiresAt: null,
    };
  }

  const row = result.data[0];
  if (typeof row !== 'object' || row === null || Array.isArray(row)) {
    throw new Error('inspect_operations_role_v1 returned an invalid row');
  }

  const record = row as Record<string, unknown>;
  return {
    userId: readString(record.user_id, 'user_id'),
    roleName: readNullableString(record.role_name, 'role_name'),
    status: readNullableString(record.status, 'status'),
    expiresAt: readNullableString(record.expires_at, 'expires_at'),
  };
}

function readString(value: unknown, label: string): string {
  if (typeof value !== 'string' || value.length === 0) {
    throw new Error(`${label} is invalid`);
  }
  return value;
}

function readNullableString(value: unknown, label: string): string | null {
  if (value === null) {
    return null;
  }
  return readString(value, label);
}

function modeForAction(action: OperationsRoleAction): OperationsStagingConfig['mode'] {
  return action === 'inspect'
    ? 'roles-inspect'
    : action === 'grant'
      ? 'roles-grant'
      : 'roles-revoke';
}

async function main(): Promise<void> {
  const action = parseAction(process.argv[2]);
  const userId = readRequiredArgument(process.argv[3], 'user_id');
  const roleName = action === 'grant'
    ? parseRoleName(process.argv[4])
    : undefined;
  const environment = loadOperationsStagingEnvironment();
  assertStagingSecretIsolation(environment.projectDirectory, environment.envFile);
  const config = resolveOperationsStagingConfig(
    environment.env,
    modeForAction(action),
  );
  const result = await runOperationsRoleAction(config, action, userId, roleName);
  process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
}

function parseAction(raw: string | undefined): OperationsRoleAction {
  if (raw === 'inspect' || raw === 'grant' || raw === 'revoke') {
    return raw;
  }
  throw new Error('usage: operations-roles.ts <inspect|grant|revoke> <user_id> [role_name]');
}

function parseRoleName(raw: string | undefined): OperationsRoleName {
  if (OPERATIONS_ROLE_NAMES.includes(raw as OperationsRoleName)) {
    return raw as OperationsRoleName;
  }
  throw new Error('grant requires a supported audited operations role name');
}

function readRequiredArgument(raw: string | undefined, label: string): string {
  if (!raw || !raw.trim()) {
    throw new Error(`${label} is required`);
  }
  return raw.trim();
}

if (isMainModule(import.meta.url)) {
  main().catch((error: unknown) => {
    const environment = loadOperationsStagingEnvironment(process.cwd(), {});
    let config: OperationsStagingConfig | undefined;
    try {
      const action = parseAction(process.argv[2]);
      config = resolveOperationsStagingConfig(
        environment.env,
        modeForAction(action),
      );
    } catch {
      config = undefined;
    }

    process.stderr.write(`${sanitizeStagingError(error, config)}\n`);
    process.exitCode = 1;
  });
}
