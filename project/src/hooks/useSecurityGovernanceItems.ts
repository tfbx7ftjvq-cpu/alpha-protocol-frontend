import { useCallback, useEffect, useMemo, useState } from 'react';
import { Connection } from '@solana/web3.js';
import {
  SECURITY_LAYER_DEVNET_RPC_ENDPOINT,
  fetchSecurityGovernanceItems,
  type SecurityGovernanceItem,
} from '../lib/securityLayer';

type SecurityGovernanceItemsStatus = 'idle' | 'loading' | 'ready' | 'error';

export function useSecurityGovernanceItems() {
  const connection = useMemo(
    () => new Connection(SECURITY_LAYER_DEVNET_RPC_ENDPOINT, 'confirmed'),
    [],
  );

  const [status, setStatus] = useState<SecurityGovernanceItemsStatus>('idle');
  const [items, setItems] = useState<SecurityGovernanceItem[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [lastLoadedAt, setLastLoadedAt] = useState<Date | null>(null);

  const refresh = useCallback(async () => {
    setStatus('loading');
    setError(null);

    try {
      const nextItems = await fetchSecurityGovernanceItems(connection);
      setItems(nextItems);
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
    items,
    lastLoadedAt,
    refresh,
    status,
  };
}
