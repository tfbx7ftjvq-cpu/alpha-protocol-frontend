import { useCallback, useEffect, useMemo, useState } from 'react';
import { Connection } from '@solana/web3.js';
import {
  GREEN_LABEL_DEVNET_RPC_ENDPOINT,
  fetchGreenLabelConfig,
  type GreenLabelConfigV1,
} from '../lib/greenLabel';

type GreenLabelConfigStatus = 'idle' | 'loading' | 'ready' | 'error';

export function useGreenLabelConfig() {
  const connection = useMemo(
    () => new Connection(GREEN_LABEL_DEVNET_RPC_ENDPOINT, 'confirmed'),
    [],
  );

  const [status, setStatus] = useState<GreenLabelConfigStatus>('idle');
  const [config, setConfig] = useState<GreenLabelConfigV1 | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [lastLoadedAt, setLastLoadedAt] = useState<Date | null>(null);

  const refresh = useCallback(async () => {
    setStatus('loading');
    setError(null);

    try {
      const nextConfig = await fetchGreenLabelConfig(connection);
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
