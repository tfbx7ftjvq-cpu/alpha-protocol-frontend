import { createClient, type SupabaseClient } from '@supabase/supabase-js';

export type OperationsIntakeMode = 'disabled' | 'anonymous';

export interface OperationsBackendConfig {
  configured: boolean;
  intakeMode: OperationsIntakeMode;
  publicReadEnabled: boolean;
  intakeEnabled: boolean;
  reason: string | null;
}

const rawUrl = import.meta.env?.VITE_SUPABASE_URL?.trim() ?? '';
const rawPublicKey = import.meta.env?.VITE_SUPABASE_ANON_KEY?.trim() ?? '';
const rawIntakeMode = import.meta.env?.VITE_OPERATIONS_INTAKE_MODE?.trim() ?? 'disabled';

let client: SupabaseClient | null = null;

export const operationsBackendConfig = resolveOperationsBackendConfig(
  rawUrl,
  rawPublicKey,
  rawIntakeMode,
);

export function getOperationsSupabase(): SupabaseClient | null {
  if (!operationsBackendConfig.configured) {
    return null;
  }

  if (!client) {
    client = createClient(rawUrl, rawPublicKey, {
      auth: {
        autoRefreshToken: true,
        detectSessionInUrl: false,
        persistSession: true,
      },
      global: {
        headers: {
          'X-Client-Info': 'alpha-protocol-public-operations/1.0',
        },
      },
    });
  }

  return client;
}

export function resolveOperationsBackendConfig(
  url: string,
  publicKey: string,
  intakeModeValue: string,
): OperationsBackendConfig {
  if (!url || !publicKey) {
    return disabledConfig('Supabase URL 或公开 anon/publishable key 尚未配置');
  }

  let parsedUrl: URL;
  try {
    parsedUrl = new URL(url);
  } catch {
    return disabledConfig('Supabase URL 无效');
  }

  if (parsedUrl.protocol !== 'https:' || parsedUrl.username || parsedUrl.password) {
    return disabledConfig('Supabase URL 必须是无内嵌凭据的 HTTPS URL');
  }

  if (isBrowserForbiddenKey(publicKey)) {
    return disabledConfig('检测到疑似 service-role/secret key，已拒绝在浏览器中加载');
  }

  if (publicKey.length < 20) {
    return disabledConfig('Supabase 公开 key 格式无效');
  }

  const intakeMode: OperationsIntakeMode = intakeModeValue === 'anonymous' ? 'anonymous' : 'disabled';

  return {
    configured: true,
    intakeMode,
    publicReadEnabled: true,
    intakeEnabled: intakeMode === 'anonymous',
    reason: intakeMode === 'anonymous'
      ? null
      : '公开读取已启用；用户提交保持关闭，直到显式设置 anonymous intake',
  };
}

function disabledConfig(reason: string): OperationsBackendConfig {
  return {
    configured: false,
    intakeMode: 'disabled',
    publicReadEnabled: false,
    intakeEnabled: false,
    reason,
  };
}

function isBrowserForbiddenKey(key: string): boolean {
  if (key.startsWith('sb_secret_') || key.toLowerCase().includes('service_role')) {
    return true;
  }

  const parts = key.split('.');
  if (parts.length !== 3) {
    return false;
  }

  try {
    const normalizedPayload = parts[1].replace(/-/g, '+').replace(/_/g, '/');
    const paddedPayload = normalizedPayload.padEnd(Math.ceil(normalizedPayload.length / 4) * 4, '=');
    const payload = JSON.parse(atob(paddedPayload)) as { role?: unknown };
    return payload.role === 'service_role';
  } catch {
    return false;
  }
}
