import { useCallback, useEffect, useState } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import type { User } from '@supabase/supabase-js';
import {
  assertWalletSessionMatch,
  buildOperationsWeb3AuthOptions,
  getVerifiedSolanaWallet,
  normalizeTurnstileToken,
  OPERATIONS_WALLET_SIGN_IN_STATEMENT,
} from '../features/operations/auth';
import {
  getOperationsSupabase,
  isExactOperationsWeb3Page,
  operationsBackendConfig,
} from '../lib/operationsSupabase';

export type OperationsAuthStatus =
  | 'disabled'
  | 'checking'
  | 'signed-out'
  | 'signing-in'
  | 'authenticated'
  | 'wallet-mismatch'
  | 'error';

export interface OperationsWalletAuthState {
  status: OperationsAuthStatus;
  connectedWallet: string | null;
  authenticatedWallet: string | null;
  error: string | null;
  signIn: (captchaToken: string) => Promise<void>;
  signOut: () => Promise<void>;
  refresh: () => Promise<void>;
}

export function useOperationsWalletAuth(): OperationsWalletAuthState {
  const wallet = useWallet();
  const connectedWallet = wallet.publicKey?.toBase58() ?? null;
  const expectedWeb3Url = operationsBackendConfig.web3Url;
  const [status, setStatus] = useState<OperationsAuthStatus>(
    operationsBackendConfig.intakeEnabled ? 'checking' : 'disabled',
  );
  const [authenticatedWallet, setAuthenticatedWallet] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(
    operationsBackendConfig.intakeEnabled ? null : operationsBackendConfig.reason,
  );

  const applyUser = useCallback(async (user: User | null) => {
    if (!operationsBackendConfig.intakeEnabled) {
      setStatus('disabled');
      setAuthenticatedWallet(null);
      setError(operationsBackendConfig.reason);
      return;
    }

    if (!user) {
      setStatus('signed-out');
      setAuthenticatedWallet(null);
      setError(null);
      return;
    }

    const identityWallet = getVerifiedSolanaWallet(user);
    if (!identityWallet) {
      setStatus('error');
      setAuthenticatedWallet(null);
      setError('当前会话不是唯一的 Solana Web3 钱包身份，请退出后重新认证');
      return;
    }

    const client = getOperationsSupabase();
    if (!client) {
      setStatus('error');
      setAuthenticatedWallet(null);
      setError('运营后端尚未配置');
      return;
    }

    if (!expectedWeb3Url || !isCurrentWeb3Page(expectedWeb3Url)) {
      setStatus('error');
      setAuthenticatedWallet(null);
      setError('当前页面与允许的钱包认证 URL 不一致，已拒绝签名');
      return;
    }

    const [intakeResult, walletResult] = await Promise.all([
      client.rpc('is_operations_wallet_intake_enabled'),
      client.rpc('current_verified_solana_wallet'),
    ]);
    if (intakeResult.error || intakeResult.data !== true) {
      setStatus('error');
      setAuthenticatedWallet(null);
      setError('数据库端钱包提交总闸门仍为关闭状态');
      return;
    }

    if (walletResult.error || walletResult.data !== identityWallet) {
      setStatus('error');
      setAuthenticatedWallet(null);
      setError('数据库未确认当前 Web3 钱包身份；钱包认证 migration 可能尚未应用');
      return;
    }

    setAuthenticatedWallet(identityWallet);
    if (!connectedWallet || connectedWallet !== identityWallet) {
      setStatus('wallet-mismatch');
      setError('连接钱包与已认证会话不一致；请切回原钱包或重新认证');
      return;
    }

    setStatus('authenticated');
    setError(null);
  }, [connectedWallet, expectedWeb3Url]);

  const refresh = useCallback(async () => {
    if (!operationsBackendConfig.intakeEnabled) {
      await applyUser(null);
      return;
    }

    const client = getOperationsSupabase();
    if (!client) {
      setStatus('error');
      setError('运营后端尚未配置');
      return;
    }

    setStatus('checking');
    setError(null);

    const sessionResult = await client.auth.getSession();
    if (sessionResult.error || !sessionResult.data.session) {
      await applyUser(null);
      return;
    }

    const userResult = await client.auth.getUser();
    if (userResult.error || !userResult.data.user) {
      setStatus('error');
      setAuthenticatedWallet(null);
      setError('无法验证钱包认证会话，请重新认证');
      return;
    }

    await applyUser(userResult.data.user);
  }, [applyUser]);

  useEffect(() => {
    void refresh();

    const client = getOperationsSupabase();
    if (!client || !operationsBackendConfig.intakeEnabled) {
      return;
    }

    const { data } = client.auth.onAuthStateChange((_event, session) => {
      setTimeout(() => {
        void applyUser(session?.user ?? null);
      }, 0);
    });

    return () => data.subscription.unsubscribe();
  }, [applyUser, refresh]);

  const signIn = useCallback(async (captchaToken: string) => {
    if (!operationsBackendConfig.intakeEnabled) {
      setStatus('disabled');
      setError(operationsBackendConfig.reason);
      return;
    }

    let verifiedCaptchaToken: string;
    try {
      verifiedCaptchaToken = normalizeTurnstileToken(captchaToken);
    } catch (captchaError) {
      setStatus('signed-out');
      setError(
        captchaError instanceof Error
          ? captchaError.message
          : 'Turnstile 验证无效，请重新验证',
      );
      return;
    }

    if (!wallet.publicKey || !wallet.signMessage || !connectedWallet) {
      setStatus('signed-out');
      setError('请先连接支持消息签名的 Solana 钱包');
      return;
    }

    const client = getOperationsSupabase();
    if (!client) {
      setStatus('error');
      setError('运营后端尚未配置');
      return;
    }

    if (!expectedWeb3Url || !isCurrentWeb3Page(expectedWeb3Url)) {
      setStatus('error');
      setError('当前页面与允许的钱包认证 URL 不一致，已拒绝签名');
      return;
    }

    setStatus('signing-in');
    setError(null);

    const existingSession = await client.auth.getSession();
    const existingWallet = getVerifiedSolanaWallet(existingSession.data.session?.user ?? null);
    if (existingWallet && existingWallet !== connectedWallet) {
      await client.auth.signOut({ scope: 'local' });
    }

    const signInResult = await client.auth.signInWithWeb3({
      chain: 'solana',
      statement: OPERATIONS_WALLET_SIGN_IN_STATEMENT,
      wallet: {
        publicKey: wallet.publicKey,
        signMessage: wallet.signMessage,
      },
      options: {
        ...buildOperationsWeb3AuthOptions(expectedWeb3Url, verifiedCaptchaToken),
      },
    });

    if (signInResult.error || !signInResult.data.user) {
      setStatus('error');
      setAuthenticatedWallet(null);
      setError('钱包认证失败；请重新完成 Turnstile，并检查 Web3 Provider、Redirect URL、域名和钱包签名权限');
      return;
    }

    try {
      assertWalletSessionMatch(signInResult.data.user, connectedWallet);
      await applyUser(signInResult.data.user);
    } catch (authError) {
      await client.auth.signOut({ scope: 'local' });
      setStatus('error');
      setAuthenticatedWallet(null);
      setError(authError instanceof Error ? authError.message : '钱包会话校验失败');
    }
  }, [
    applyUser,
    connectedWallet,
    expectedWeb3Url,
    wallet.publicKey,
    wallet.signMessage,
  ]);

  const signOut = useCallback(async () => {
    const client = getOperationsSupabase();
    if (client) {
      await client.auth.signOut({ scope: 'local' });
    }
    setAuthenticatedWallet(null);
    setStatus(operationsBackendConfig.intakeEnabled ? 'signed-out' : 'disabled');
    setError(null);
  }, []);

  return {
    status,
    connectedWallet,
    authenticatedWallet,
    error,
    signIn,
    signOut,
    refresh,
  };
}

function isCurrentWeb3Page(expectedUrl: string): boolean {
  if (typeof window === 'undefined') {
    return false;
  }

  return isExactOperationsWeb3Page(expectedUrl, window.location.href);
}
