import { useCallback, useEffect, useMemo, useState } from 'react';
import { Connection } from '@solana/web3.js';
import {
  TREASURY_V2_DEVNET_RPC_ENDPOINT,
  fetchTreasuryV2Overview,
  type TreasuryV2Overview,
} from '../lib/treasuryV2';

type TreasuryV2ReadStatus = 'idle' | 'loading' | 'ready' | 'error';

export function useTreasuryV2() {
  const connection = useMemo(
    () => new Connection(TREASURY_V2_DEVNET_RPC_ENDPOINT, 'confirmed'),
    [],
  );
  const [overview, setOverview] = useState<TreasuryV2Overview | null>(null);
  const [status, setStatus] = useState<TreasuryV2ReadStatus>('idle');
  const [error, setError] = useState<string | null>(null);
  const [lastLoadedAt, setLastLoadedAt] = useState<Date | null>(null);

  const refresh = useCallback(async () => {
    setStatus('loading');
    setError(null);

    try {
      const nextOverview = await fetchTreasuryV2Overview(connection);
      setOverview(nextOverview);
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
    overview,
    refresh,
    status,
  };
}
