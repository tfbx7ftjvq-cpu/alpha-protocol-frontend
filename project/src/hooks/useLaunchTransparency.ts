import { useCallback, useEffect, useMemo, useState } from 'react';
import { Connection } from '@solana/web3.js';
import {
  MAINNET_RPC_ENDPOINT,
  fetchMainnetRevenueSnapshot,
  readLaunchConfig,
  type MainnetRevenueSnapshot,
} from '../lib/launchTransparency';

type ReadStatus = 'unconfigured' | 'loading' | 'ready' | 'error';

export function useLaunchTransparency() {
  const config = useMemo(() => readLaunchConfig(), []);
  const connection = useMemo(() => new Connection(MAINNET_RPC_ENDPOINT, 'confirmed'), []);
  const [snapshot, setSnapshot] = useState<MainnetRevenueSnapshot | null>(null);
  const [status, setStatus] = useState<ReadStatus>(
    config.revenueWallet ? 'loading' : 'unconfigured',
  );
  const [error, setError] = useState<string | null>(null);
  const [lastLoadedAt, setLastLoadedAt] = useState<Date | null>(null);

  const refresh = useCallback(async () => {
    if (!config.revenueWallet) {
      setStatus('unconfigured');
      setSnapshot(null);
      return;
    }

    setStatus('loading');
    setError(null);
    try {
      setSnapshot(await fetchMainnetRevenueSnapshot(connection, config.revenueWallet));
      setStatus('ready');
    } catch (reason) {
      setSnapshot(null);
      setStatus('error');
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setLastLoadedAt(new Date());
    }
  }, [config.revenueWallet, connection]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return { config, error, lastLoadedAt, refresh, snapshot, status };
}
