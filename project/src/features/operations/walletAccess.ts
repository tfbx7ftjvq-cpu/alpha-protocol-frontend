export type OperationsAuthStatus =
  | 'disabled'
  | 'checking'
  | 'signed-out'
  | 'signing-in'
  | 'authenticated'
  | 'wallet-mismatch'
  | 'error';

export type OperationsIntakeGateStatus =
  | 'unavailable'
  | 'checking'
  | 'enabled'
  | 'disabled'
  | 'error';

export interface OperationsWalletAccess {
  sessionVerified: boolean;
  intakeEnabled: boolean;
}

export function resolveOperationsWalletAccess(
  authStatus: OperationsAuthStatus,
  authenticatedWallet: string | null,
  connectedWallet: string | null,
  intakeGateStatus: OperationsIntakeGateStatus,
): OperationsWalletAccess {
  const sessionVerified = authStatus === 'authenticated'
    && Boolean(authenticatedWallet)
    && authenticatedWallet === connectedWallet;

  return {
    sessionVerified,
    intakeEnabled: sessionVerified && intakeGateStatus === 'enabled',
  };
}
