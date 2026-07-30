import { createClient, type SupabaseClient } from '@supabase/supabase-js';

export type OperationsIntakeMode = 'disabled' | 'wallet-staging';

export interface OperationsBackendConfig {
  configured: boolean;
  intakeMode: OperationsIntakeMode;
  publicReadEnabled: boolean;
  intakeEnabled: boolean;
  projectRef: string | null;
  web3Url: string | null;
  reason: string | null;
}

const rawUrl = import.meta.env?.VITE_SUPABASE_URL?.trim() ?? '';
const rawPublicKey = import.meta.env?.VITE_SUPABASE_ANON_KEY?.trim() ?? '';
const rawIntakeMode = import.meta.env?.VITE_OPERATIONS_INTAKE_MODE?.trim() ?? 'disabled';
const rawProjectRef = import.meta.env?.VITE_OPERATIONS_PROJECT_REF?.trim() ?? '';
const rawWeb3Url = import.meta.env?.VITE_OPERATIONS_WEB3_URL?.trim() ?? '';

let client: SupabaseClient | null = null;

export const operationsBackendConfig = resolveOperationsBackendConfig(
  rawUrl,
  rawPublicKey,
  rawIntakeMode,
  rawProjectRef,
  rawWeb3Url,
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
  expectedProjectRef = '',
  expectedWeb3Url = '',
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

  const intakeMode: OperationsIntakeMode = intakeModeValue === 'wallet-staging'
    ? 'wallet-staging'
    : 'disabled';
  const projectRef = extractSupabaseProjectRef(parsedUrl);
  const web3Url = normalizeWeb3Url(expectedWeb3Url);

  if (intakeMode === 'wallet-staging') {
    if (!/^[a-z0-9]{20}$/.test(expectedProjectRef)) {
      return readOnlyConfig(
        projectRef,
        web3Url,
        '公开读取已启用；钱包提交保持关闭，因为缺少有效的 VITE_OPERATIONS_PROJECT_REF',
      );
    }

    if (projectRef !== expectedProjectRef) {
      return readOnlyConfig(
        projectRef,
        web3Url,
        '公开读取已启用；钱包提交保持关闭，因为 Supabase URL 与预期 project ref 不一致',
      );
    }

    if (!web3Url) {
      return readOnlyConfig(
        projectRef,
        null,
        '公开读取已启用；钱包提交保持关闭，因为缺少有效的 VITE_OPERATIONS_WEB3_URL',
      );
    }
  }

  return {
    configured: true,
    intakeMode,
    publicReadEnabled: true,
    intakeEnabled: intakeMode === 'wallet-staging',
    projectRef,
    web3Url,
    reason: intakeMode === 'wallet-staging'
      ? null
      : '公开读取已启用；用户提交保持关闭，直到显式设置 wallet-staging intake',
  };
}

export function isExactOperationsWeb3Page(
  expectedUrl: string,
  currentUrlValue: string,
): boolean {
  const normalizedExpected = normalizeWeb3Url(expectedUrl);
  if (!normalizedExpected) {
    return false;
  }

  try {
    const currentUrl = new URL(currentUrlValue);
    currentUrl.search = '';
    currentUrl.hash = '';
    return currentUrl.href === normalizedExpected;
  } catch {
    return false;
  }
}

function disabledConfig(reason: string): OperationsBackendConfig {
  return {
    configured: false,
    intakeMode: 'disabled',
    publicReadEnabled: false,
    intakeEnabled: false,
    projectRef: null,
    web3Url: null,
    reason,
  };
}

function readOnlyConfig(
  projectRef: string | null,
  web3Url: string | null,
  reason: string,
): OperationsBackendConfig {
  return {
    configured: true,
    intakeMode: 'disabled',
    publicReadEnabled: true,
    intakeEnabled: false,
    projectRef,
    web3Url,
    reason,
  };
}

function extractSupabaseProjectRef(url: URL): string | null {
  const match = url.hostname.match(/^([a-z0-9]{20})\.supabase\.co$/);
  return match?.[1] ?? null;
}

function normalizeWeb3Url(value: string): string | null {
  if (!value) {
    return null;
  }

  try {
    const url = new URL(value);
    if (
      url.protocol !== 'https:'
      || url.username
      || url.password
      || url.search
      || url.hash
    ) {
      return null;
    }
    return url.href;
  } catch {
    return null;
  }
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
