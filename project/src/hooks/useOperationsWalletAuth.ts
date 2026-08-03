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
import type {
  OperationsStaffRole,
} from '../features/operations/domain';
import {
  resolveOperationsStaffRole,
} from '../features/operations/repository';
import type {
  OperationsAuthStatus,
  OperationsIntakeGateStatus,
} from '../features/operations/walletAccess';
import {
  getOperationsSupabase,
  isExactOperationsWeb3Page,
  operationsBackendConfig,
} from '../lib/operationsSupabase';

export interface OperationsWalletAuthState {
  status: OperationsAuthStatus;
  intakeGateStatus: OperationsIntakeGateStatus;
  connectedWallet: string | null;
  authenticatedWallet: string | null;
  operationsRole: OperationsStaffRole | null;
  error: string | null;
  intakeGateError: string | null;
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
  const [operationsRole, setOperationsRole] = useState<OperationsStaffRole | null>(null);
  const [intakeGateStatus, setIntakeGateStatus] = useState<OperationsIntakeGateStatus>(
    operationsBackendConfig.intakeEnabled ? 'checking' : 'unavailable',
  );
  const [error, setError] = useState<string | null>(
    operationsBackendConfig.intakeEnabled ? null : operationsBackendConfig.reason,
  );
  const [intakeGateError, setIntakeGateError] = useState<string | null>(null);

  const applyUser = useCallback(async (user: User | null) => {
    if (!operationsBackendConfig.intakeEnabled) {
      setStatus('disabled');
      setIntakeGateStatus('unavailable');
      setAuthenticatedWallet(null);
      setOperationsRole(null);
      setError(operationsBackendConfig.reason);
      setIntakeGateError(null);
      return;
    }

    if (!user) {
      setStatus('signed-out');
      setIntakeGateStatus('unavailable');
      setAuthenticatedWallet(null);
      setOperationsRole(null);
      setError(null);
      setIntakeGateError(null);
      return;
    }

    const identityWallet = getVerifiedSolanaWallet(user);
    if (!identityWallet) {
      setStatus('error');
      setIntakeGateStatus('unavailable');
      setAuthenticatedWallet(null);
      setOperationsRole(null);
      setError('当前会话不是唯一的 Solana Web3 钱包身份，请退出后重新认证');
      setIntakeGateError(null);
      return;
    }

    const client = getOperationsSupabase();
    if (!client) {
      setStatus('error');
      setIntakeGateStatus('error');
      setAuthenticatedWallet(null);
      setOperationsRole(null);
      setError('运营后端尚未配置');
      setIntakeGateError('无法读取数据库写入总闸门');
      return;
    }

    if (!expectedWeb3Url || !isCurrentWeb3Page(expectedWeb3Url)) {
      setStatus('error');
      setIntakeGateStatus('unavailable');
      setAuthenticatedWallet(null);
      setOperationsRole(null);
      setError('当前页面与允许的钱包认证 URL 不一致，已拒绝签名');
      setIntakeGateError(null);
      return;
    }

    const [intakeResult, walletResult] = await Promise.all([
      client.rpc('is_operations_wallet_intake_enabled'),
      client.rpc('current_verified_solana_wallet'),
    ]);
    if (intakeResult.error) {
      setIntakeGateStatus('error');
      setIntakeGateError('无法确认数据库写入总闸门；所有提交继续锁定');
    } else if (intakeResult.data === true) {
      setIntakeGateStatus('enabled');
      setIntakeGateError(null);
    } else if (intakeResult.data === false) {
      setIntakeGateStatus('disabled');
      setIntakeGateError(null);
    } else {
      setIntakeGateStatus('error');
      setIntakeGateError('数据库写入总闸门返回了无效状态；所有提交继续锁定');
    }

    if (walletResult.error || walletResult.data !== identityWallet) {
      setStatus('error');
      setAuthenticatedWallet(null);
      setOperationsRole(null);
      setError('数据库未确认当前 Web3 钱包身份；钱包认证 migration 可能尚未应用');
      return;
    }

    setAuthenticatedWallet(identityWallet);
    if (!connectedWallet || connectedWallet !== identityWallet) {
      setOperationsRole(null);
      setStatus('wallet-mismatch');
      setError('连接钱包与已认证会话不一致；请切回原钱包或重新认证');
      return;
    }

    setStatus('authenticated');
    setOperationsRole(resolveOperationsStaffRole(user));
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
      setIntakeGateStatus('error');
      setAuthenticatedWallet(null);
      setOperationsRole(null);
      setError('运营后端尚未配置');
      setIntakeGateError('无法读取数据库写入总闸门');
      return;
    }

    setStatus('checking');
    setIntakeGateStatus('checking');
    setError(null);
    setIntakeGateError(null);

    const sessionResult = await client.auth.getSession();
    if (sessionResult.error || !sessionResult.data.session) {
      await applyUser(null);
      return;
    }

    const userResult = await client.auth.getUser();
    if (userResult.error || !userResult.data.user) {
      setStatus('error');
      setIntakeGateStatus('unavailable');
      setAuthenticatedWallet(null);
      setOperationsRole(null);
      setError('无法验证钱包认证会话，请重新认证');
      setIntakeGateError(null);
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
      setIntakeGateStatus('unavailable');
      setError(operationsBackendConfig.reason);
      setIntakeGateError(null);
      return;
    }

    let verifiedCaptchaToken: string;
    try {
      verifiedCaptchaToken = normalizeTurnstileToken(captchaToken);
    } catch (captchaError) {
      setStatus('signed-out');
      setIntakeGateStatus('unavailable');
      setError(
        captchaError instanceof Error
          ? captchaError.message
          : 'Turnstile 验证无效，请重新验证',
      );
      setIntakeGateError(null);
      return;
    }

    if (!wallet.publicKey || !wallet.signMessage || !connectedWallet) {
      setStatus('signed-out');
      setIntakeGateStatus('unavailable');
      setError('请先连接支持消息签名的 Solana 钱包');
      setIntakeGateError(null);
      return;
    }

    const client = getOperationsSupabase();
    if (!client) {
      setStatus('error');
      setIntakeGateStatus('error');
      setError('运营后端尚未配置');
      setIntakeGateError('无法读取数据库写入总闸门');
      return;
    }

    if (!expectedWeb3Url || !isCurrentWeb3Page(expectedWeb3Url)) {
      setStatus('error');
      setIntakeGateStatus('unavailable');
      setError('当前页面与允许的钱包认证 URL 不一致，已拒绝签名');
      setIntakeGateError(null);
      return;
    }

    setStatus('signing-in');
    setError(null);
    setIntakeGateStatus('checking');
    setIntakeGateError(null);

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
      setIntakeGateStatus('unavailable');
      setAuthenticatedWallet(null);
      setOperationsRole(null);
      setError('钱包认证失败；请重新完成 Turnstile，并检查 Web3 Provider、Redirect URL、域名和钱包签名权限');
      setIntakeGateError(null);
      return;
    }

    try {
      assertWalletSessionMatch(signInResult.data.user, connectedWallet);
      await applyUser(signInResult.data.user);
    } catch (authError) {
      await client.auth.signOut({ scope: 'local' });
      setStatus('error');
      setIntakeGateStatus('unavailable');
      setAuthenticatedWallet(null);
      setOperationsRole(null);
      setError(authError instanceof Error ? authError.message : '钱包会话校验失败');
      setIntakeGateError(null);
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
    setOperationsRole(null);
    setIntakeGateStatus('unavailable');
    setStatus(operationsBackendConfig.intakeEnabled ? 'signed-out' : 'disabled');
    setError(null);
    setIntakeGateError(null);
  }, []);

  return {
    status,
    intakeGateStatus,
    connectedWallet,
    authenticatedWallet,
    operationsRole,
    error,
    intakeGateError,
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
