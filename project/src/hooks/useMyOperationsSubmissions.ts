import { useCallback, useEffect, useState } from 'react';
import type { MyOperationsSubmission } from '../features/operations/domain';
import { loadMyOperationsSubmissions } from '../features/operations/repository';

export type MyOperationsLoadStatus = 'locked' | 'loading' | 'ready' | 'error';

export interface MyOperationsSubmissionsState {
  status: MyOperationsLoadStatus;
  submissions: MyOperationsSubmission[];
  error: string | null;
  refresh: () => Promise<void>;
}

export function useMyOperationsSubmissions(
  authenticatedWallet: string | null,
): MyOperationsSubmissionsState {
  const [status, setStatus] = useState<MyOperationsLoadStatus>(
    authenticatedWallet ? 'loading' : 'locked',
  );
  const [submissions, setSubmissions] = useState<MyOperationsSubmission[]>([]);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    if (!authenticatedWallet) {
      setStatus('locked');
      setSubmissions([]);
      setError(null);
      return;
    }

    setStatus('loading');
    setError(null);

    try {
      setSubmissions(await loadMyOperationsSubmissions(authenticatedWallet));
      setStatus('ready');
    } catch (loadError) {
      setSubmissions([]);
      setStatus('error');
      setError(loadError instanceof Error ? loadError.message : '我的提交记录读取失败');
    }
  }, [authenticatedWallet]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return {
    status,
    submissions,
    error,
    refresh,
  };
}
