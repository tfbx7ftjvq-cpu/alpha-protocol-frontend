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
  MyOperationsAccess,
  OperationsStaffRole,
} from '../features/operations/domain';
import { loadMyOperationsAccess } from '../features/operations/repository';
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
  operationsAccessStatus: MyOperationsAccess['status'];
  operationsRoleExpiresAt: string | null;
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
  const [operationsAccessStatus, setOperationsAccessStatus] = useState<MyOperationsAccess['status']>(null);
  const [operationsRoleExpiresAt, setOperationsRoleExpiresAt] = useState<string | null>(null);
  const [intakeGateStatus, setIntakeGateStatus] = useState<OperationsIntakeGateStatus>(
    operationsBackendConfig.intakeEnabled ? 'checking' : 'unavailable',
  );
  const [error, setError] = useState<string | null>(
    operationsBackendConfig.intakeEnabled ? null : operationsBackendConfig.reason,
  );
  const [intakeGateError, setIntakeGateError] = useState<string | null>(null);

  const clearOperationsAccess = useCallback(() => {
    setOperationsRole(null);
    setOperationsAccessStatus(null);
    setOperationsRoleExpiresAt(null);
  }, []);

  const applyUser = useCallback(async (user: User | null) => {
    if (!operationsBackendConfig.intakeEnabled) {
      setStatus('disabled');
      setIntakeGateStatus('unavailable');
      setAuthenticatedWallet(null);
      clearOperationsAccess();
      setError(operationsBackendConfig.reason);
      setIntakeGateError(null);
      return;
    }

    if (!user) {
      setStatus('signed-out');
      setIntakeGateStatus('unavailable');
      setAuthenticatedWallet(null);
      clearOperationsAccess();
      setError(null);
      setIntakeGateError(null);
      return;
    }

    const identityWallet = getVerifiedSolanaWallet(user);
    if (!identityWallet) {
      setStatus('error');
      setIntakeGateStatus('unavailable');
      setAuthenticatedWallet(null);
      clearOperationsAccess();
      setError('Current session is not backed by a verified Solana Web3 wallet.');
      setIntakeGateError(null);
      return;
    }

    const client = getOperationsSupabase();
    if (!client) {
      setStatus('error');
      setIntakeGateStatus('error');
      setAuthenticatedWallet(null);
      clearOperationsAccess();
      setError('Operations backend is not configured.');
      setIntakeGateError('Could not read the intake gate state.');
      return;
    }

    if (!expectedWeb3Url || !isCurrentWeb3Page(expectedWeb3Url)) {
      setStatus('error');
      setIntakeGateStatus('unavailable');
      setAuthenticatedWallet(null);
      clearOperationsAccess();
      setError('Wallet sign-in is only allowed from the configured Web3 callback page.');
      setIntakeGateError(null);
      return;
    }

    const [intakeResult, walletResult] = await Promise.all([
      client.rpc('is_operations_wallet_intake_enabled'),
      client.rpc('current_verified_solana_wallet'),
    ]);
    if (intakeResult.error) {
      setIntakeGateStatus('error');
      setIntakeGateError('Could not confirm the intake gate state. New submissions stay locked.');
    } else if (intakeResult.data === true) {
      setIntakeGateStatus('enabled');
      setIntakeGateError(null);
    } else if (intakeResult.data === false) {
      setIntakeGateStatus('disabled');
      setIntakeGateError(null);
    } else {
      setIntakeGateStatus('error');
      setIntakeGateError('The intake gate returned an invalid state. New submissions stay locked.');
    }

    if (walletResult.error || walletResult.data !== identityWallet) {
      setStatus('error');
      setAuthenticatedWallet(null);
      clearOperationsAccess();
      setError('Database wallet verification did not match the current Web3 session.');
      return;
    }

    setAuthenticatedWallet(identityWallet);
    if (!connectedWallet || connectedWallet !== identityWallet) {
      clearOperationsAccess();
      setStatus('wallet-mismatch');
      setError('Connected wallet does not match the verified session wallet.');
      return;
    }

    const access = await loadMyOperationsAccess();
    setOperationsAccessStatus(access.status);
    setOperationsRoleExpiresAt(access.expiresAt);
    setStatus('authenticated');
    setOperationsRole(access.status === 'active' ? access.role : null);
    setError(null);
  }, [clearOperationsAccess, connectedWallet, expectedWeb3Url]);

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
      clearOperationsAccess();
      setError('Operations backend is not configured.');
      setIntakeGateError('Could not read the intake gate state.');
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
      clearOperationsAccess();
      setError('Could not validate the wallet-authenticated session.');
      setIntakeGateError(null);
      return;
    }

    await applyUser(userResult.data.user);
  }, [applyUser, clearOperationsAccess]);

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
          : 'Turnstile verification is invalid. Please try again.',
      );
      setIntakeGateError(null);
      return;
    }

    if (!wallet.publicKey || !wallet.signMessage || !connectedWallet) {
      setStatus('signed-out');
      setIntakeGateStatus('unavailable');
      setError('Connect a Solana wallet that supports message signing first.');
      setIntakeGateError(null);
      return;
    }

    const client = getOperationsSupabase();
    if (!client) {
      setStatus('error');
      setIntakeGateStatus('error');
      clearOperationsAccess();
      setError('Operations backend is not configured.');
      setIntakeGateError('Could not read the intake gate state.');
      return;
    }

    if (!expectedWeb3Url || !isCurrentWeb3Page(expectedWeb3Url)) {
      setStatus('error');
      setIntakeGateStatus('unavailable');
      setError('Wallet sign-in is only allowed from the configured Web3 callback page.');
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
      clearOperationsAccess();
      setError('Wallet sign-in failed. Check the Web3 provider, redirect URL, domain, and signature permission.');
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
      clearOperationsAccess();
      setError(authError instanceof Error ? authError.message : 'Wallet session verification failed.');
      setIntakeGateError(null);
    }
  }, [
    applyUser,
    clearOperationsAccess,
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
    clearOperationsAccess();
    setIntakeGateStatus('unavailable');
    setStatus(operationsBackendConfig.intakeEnabled ? 'signed-out' : 'disabled');
    setError(null);
    setIntakeGateError(null);
  }, [clearOperationsAccess]);

  return {
    status,
    intakeGateStatus,
    connectedWallet,
    authenticatedWallet,
    operationsRole,
    operationsAccessStatus,
    operationsRoleExpiresAt,
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
