import { useCallback, useEffect, useMemo, useState } from 'react';
import { Connection } from '@solana/web3.js';
import {
  GREEN_LABEL_DEVNET_RPC_ENDPOINT,
  fetchGreenLabelE2EResults,
  type GreenLabelE2EResult,
} from '../lib/greenLabel';

type GreenLabelE2EResultsStatus = 'idle' | 'loading' | 'ready' | 'error';

export function useGreenLabelE2EResults() {
  const connection = useMemo(
    () => new Connection(GREEN_LABEL_DEVNET_RPC_ENDPOINT, 'confirmed'),
    [],
  );

  const [status, setStatus] = useState<GreenLabelE2EResultsStatus>('idle');
  const [results, setResults] = useState<GreenLabelE2EResult[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [lastLoadedAt, setLastLoadedAt] = useState<Date | null>(null);

  const refresh = useCallback(async () => {
    setStatus('loading');
    setError(null);

    try {
      const nextResults = await fetchGreenLabelE2EResults(connection);
      setResults(nextResults);
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
    error,
    lastLoadedAt,
    refresh,
    results,
    status,
  };
}
