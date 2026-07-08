import { useCallback, useEffect, useMemo, useState } from 'react';
import { Connection } from '@solana/web3.js';
import {
  SECURITY_LAYER_DEVNET_RPC_ENDPOINT,
  fetchGovernanceConfigV1,
  type GovernanceConfigV1,
} from '../lib/securityLayer';

type SecurityGovernanceConfigStatus = 'idle' | 'loading' | 'ready' | 'error';

export function useSecurityGovernanceConfig() {
  const connection = useMemo(
    () => new Connection(SECURITY_LAYER_DEVNET_RPC_ENDPOINT, 'confirmed'),
    [],
  );

  const [status, setStatus] = useState<SecurityGovernanceConfigStatus>('idle');
  const [config, setConfig] = useState<GovernanceConfigV1 | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [lastLoadedAt, setLastLoadedAt] = useState<Date | null>(null);

  const refresh = useCallback(async () => {
    setStatus('loading');
    setError(null);

    try {
      const nextConfig = await fetchGovernanceConfigV1(connection);
      setConfig(nextConfig);
      setStatus('ready');
      setLastLoadedAt(new Date());
    } catch (err) {
      setStatus('error');
      setError(err instanceof Error ? err.message : String(err));
      setLastLoadedAt(new Date());
    }
  }, [connection]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return {
    config,
    error,
    lastLoadedAt,
    refresh,
    status,
  };
}
