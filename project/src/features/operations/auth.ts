import type { User } from '@supabase/supabase-js';
import { isSolanaPublicKey } from './domain.ts';

export const OPERATIONS_WALLET_SIGN_IN_STATEMENT = [
  'Authenticate to Alpha Protocol staging operations.',
  'This signature creates only an off-chain session and does not authorize',
  'a Solana transaction, token approval, or transfer of funds.',
].join(' ');

export const TURNSTILE_TOKEN_MAX_LENGTH = 2048;
const TURNSTILE_TOKEN_MIN_LENGTH = 20;

type AuthIdentity = NonNullable<User['identities']>[number];

export interface OperationsAuthUser {
  identities?: Pick<AuthIdentity, 'provider' | 'identity_data'>[] | null;
}

export function normalizeTurnstileToken(value: string): string {
  if (
    value.length < TURNSTILE_TOKEN_MIN_LENGTH
    || value.length > TURNSTILE_TOKEN_MAX_LENGTH
    || value.trim() !== value
    || /\s/.test(value)
  ) {
    throw new Error('Turnstile 验证已失效或格式无效，请重新完成安全验证');
  }

  return value;
}

export function buildOperationsWeb3AuthOptions(
  url: string,
  captchaToken: string,
): { url: string; captchaToken: string } {
  return {
    url,
    captchaToken: normalizeTurnstileToken(captchaToken),
  };
}

export function getVerifiedSolanaWallet(user: OperationsAuthUser | null): string | null {
  if (!user?.identities) {
    return null;
  }

  const addresses = new Set<string>();

  for (const identity of user.identities) {
    if (identity.provider !== 'web3' || !isRecord(identity.identity_data)) {
      continue;
    }

    const chain = identity.identity_data.chain;
    const address = identity.identity_data.address;
    if (chain === 'solana' && typeof address === 'string' && isSolanaPublicKey(address)) {
      addresses.add(address);
    }
  }

  return addresses.size === 1 ? [...addresses][0] : null;
}

export function assertWalletSessionMatch(
  user: OperationsAuthUser | null,
  connectedWallet: string | null,
): string {
  if (!connectedWallet) {
    throw new Error('请先连接 Solana 钱包');
  }

  const verifiedWallet = getVerifiedSolanaWallet(user);
  if (!verifiedWallet) {
    throw new Error('当前 Supabase 会话没有唯一、已验证的 Solana Web3 身份');
  }

  if (verifiedWallet !== connectedWallet) {
    throw new Error('连接钱包与已认证的钱包会话不一致，请重新签名认证');
  }

  return verifiedWallet;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}
