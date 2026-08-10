import { useCallback, useEffect, useState } from 'react';
import type { OperationsOverview } from '../features/operations/domain';
import { loadOperationsOverview } from '../features/operations/repository';
import { operationsBackendConfig } from '../lib/operationsSupabase';

export type OperationsLoadStatus = 'unconfigured' | 'loading' | 'ready' | 'error';

const EMPTY_OVERVIEW: OperationsOverview = {
  tasks: [],
  taskResults: [],
  riskReports: [],
  reliefUpdates: [],
  discussions: [],
  governanceProposals: [],
  governanceDecisions: [],
  treasuryExecutions: [],
};

export interface OperationsState {
  status: OperationsLoadStatus;
  overview: OperationsOverview;
  error: string | null;
  lastLoadedAt: Date | null;
  refresh: () => Promise<void>;
}

export function useOperations(): OperationsState {
  const [status, setStatus] = useState<OperationsLoadStatus>(
    operationsBackendConfig.publicReadEnabled ? 'loading' : 'unconfigured',
  );
  const [overview, setOverview] = useState<OperationsOverview>(EMPTY_OVERVIEW);
  const [error, setError] = useState<string | null>(
    operationsBackendConfig.publicReadEnabled ? null : operationsBackendConfig.reason,
  );
  const [lastLoadedAt, setLastLoadedAt] = useState<Date | null>(null);

  const refresh = useCallback(async () => {
    if (!operationsBackendConfig.publicReadEnabled) {
      setStatus('unconfigured');
      setError(operationsBackendConfig.reason);
      return;
    }

    setStatus('loading');
    setError(null);

    try {
      const nextOverview = await loadOperationsOverview();
      setOverview(nextOverview);
      setLastLoadedAt(new Date());
      setStatus('ready');
    } catch (loadError) {
      setOverview(EMPTY_OVERVIEW);
      setStatus('error');
      setError(loadError instanceof Error ? loadError.message : '运营数据读取失败');
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return {
    status,
    overview,
    error,
    lastLoadedAt,
    refresh,
  };
}
