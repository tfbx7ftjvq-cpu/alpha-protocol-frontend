import { useCallback, useEffect, useState } from 'react';
import type {
  CommunityTaskPublicationInput,
  OperationsStaffRole,
  OperationsStaffWorkspace,
  TaskSubmissionReviewInput,
} from '../features/operations/domain';
import {
  loadOperationsStaffWorkspace,
  publishCommunityTask,
  reviewTaskSubmission,
} from '../features/operations/repository';

const EMPTY_WORKSPACE: OperationsStaffWorkspace = {
  submissions: [],
  events: [],
};

export interface OperationsStaffState {
  workspace: OperationsStaffWorkspace;
  loading: boolean;
  submitting: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  publishTask: (input: CommunityTaskPublicationInput) => Promise<string>;
  reviewSubmission: (input: TaskSubmissionReviewInput) => Promise<void>;
}

export function useOperationsStaff(
  role: OperationsStaffRole | null,
): OperationsStaffState {
  const [workspace, setWorkspace] = useState(EMPTY_WORKSPACE);
  const [loading, setLoading] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    if (!role) {
      setWorkspace(EMPTY_WORKSPACE);
      setError(null);
      return;
    }

    setLoading(true);
    setError(null);
    try {
      setWorkspace(await loadOperationsStaffWorkspace());
    } catch (loadError) {
      setWorkspace(EMPTY_WORKSPACE);
      setError(loadError instanceof Error ? loadError.message : '运营审核数据读取失败');
    } finally {
      setLoading(false);
    }
  }, [role]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const publishTask = useCallback(async (input: CommunityTaskPublicationInput) => {
    if (!role) {
      throw new Error('当前钱包会话没有运营权限');
    }
    setSubmitting(true);
    setError(null);
    try {
      const taskId = await publishCommunityTask(input);
      await refresh();
      return taskId;
    } catch (publishError) {
      const message = publishError instanceof Error ? publishError.message : '社区任务发布失败';
      setError(message);
      throw publishError;
    } finally {
      setSubmitting(false);
    }
  }, [refresh, role]);

  const reviewSubmission = useCallback(async (input: TaskSubmissionReviewInput) => {
    if (!role) {
      throw new Error('当前钱包会话没有运营权限');
    }
    setSubmitting(true);
    setError(null);
    try {
      await reviewTaskSubmission(input);
      await refresh();
    } catch (reviewError) {
      const message = reviewError instanceof Error ? reviewError.message : '任务审核失败';
      setError(message);
      throw reviewError;
    } finally {
      setSubmitting(false);
    }
  }, [refresh, role]);

  return {
    workspace,
    loading,
    submitting,
    error,
    refresh,
    publishTask,
    reviewSubmission,
  };
}
