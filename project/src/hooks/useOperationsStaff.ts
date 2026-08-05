import { useCallback, useEffect, useState } from 'react';
import type {
  CommunityTaskPublicationInput,
  OperationsStaffRole,
  OperationsStaffWorkspace,
  RiskReportReviewInput,
  ReliefApplicationReviewInput,
  GovernanceProposalReviewInput,
  GovernanceDiscussionReviewInput,
  GovernanceDecisionFinalizeInput,
  TaskSubmissionReviewInput,
} from '../features/operations/domain';
import {
  loadOperationsStaffWorkspace,
  publishCommunityTask,
  reviewTaskSubmission,
  reviewRiskReport,
  reviewReliefApplication,
  reviewGovernanceProposal,
  reviewGovernanceDiscussion,
  finalizeGovernanceDecision,
} from '../features/operations/repository';

const EMPTY_WORKSPACE: OperationsStaffWorkspace = {
  submissions: [],
  events: [],
  riskReports: [],
  riskEvents: [],
  reliefApplications: [],
  reliefEvents: [],
  proposalSubmissions: [],
  discussions: [],
  governanceEvents: [],
};

export interface OperationsStaffState {
  workspace: OperationsStaffWorkspace;
  loading: boolean;
  submitting: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  publishTask: (input: CommunityTaskPublicationInput) => Promise<string>;
  reviewSubmission: (input: TaskSubmissionReviewInput) => Promise<void>;
  reviewRisk: (input: RiskReportReviewInput) => Promise<void>;
  reviewRelief: (input: ReliefApplicationReviewInput) => Promise<void>;
  reviewProposal: (input: GovernanceProposalReviewInput) => Promise<void>;
  reviewDiscussion: (input: GovernanceDiscussionReviewInput) => Promise<void>;
  finalizeDecision: (input: GovernanceDecisionFinalizeInput) => Promise<void>;
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

  const reviewRisk = useCallback(async (input: RiskReportReviewInput) => {
    if (!role) {
      throw new Error('当前钱包会话没有运营权限');
    }
    setSubmitting(true);
    setError(null);
    try {
      await reviewRiskReport(input);
      await refresh();
    } catch (reviewError) {
      const message = reviewError instanceof Error ? reviewError.message : '风险审核失败';
      setError(message);
      throw reviewError;
    } finally {
      setSubmitting(false);
    }
  }, [refresh, role]);

  const reviewRelief = useCallback(async (input: ReliefApplicationReviewInput) => {
    if (!role) {
      throw new Error('当前钱包会话没有运营权限');
    }
    setSubmitting(true);
    setError(null);
    try {
      await reviewReliefApplication(input);
      await refresh();
    } catch (reviewError) {
      const message = reviewError instanceof Error ? reviewError.message : '救助审核失败';
      setError(message);
      throw reviewError;
    } finally {
      setSubmitting(false);
    }
  }, [refresh, role]);

  const reviewProposal = useCallback(async (input: GovernanceProposalReviewInput) => {
    if (!role) throw new Error('当前钱包会话没有治理提案审核权限');
    setSubmitting(true); setError(null);
    try { await reviewGovernanceProposal(input); await refresh(); }
    catch (reviewError) { setError(reviewError instanceof Error ? reviewError.message : '治理提案审核失败'); throw reviewError; }
    finally { setSubmitting(false); }
  }, [refresh, role]);

  const reviewDiscussion = useCallback(async (input: GovernanceDiscussionReviewInput) => {
    if (!role) throw new Error('当前钱包会话没有治理讨论审核权限');
    setSubmitting(true); setError(null);
    try { await reviewGovernanceDiscussion(input); await refresh(); }
    catch (reviewError) { setError(reviewError instanceof Error ? reviewError.message : '治理讨论审核失败'); throw reviewError; }
    finally { setSubmitting(false); }
  }, [refresh, role]);

  const finalizeDecision = useCallback(async (input: GovernanceDecisionFinalizeInput) => {
    if (!role) throw new Error('当前钱包会话没有治理决定终局权限');
    setSubmitting(true); setError(null);
    try { await finalizeGovernanceDecision(input); await refresh(); }
    catch (finalizeError) { setError(finalizeError instanceof Error ? finalizeError.message : '治理决定终局失败'); throw finalizeError; }
    finally { setSubmitting(false); }
  }, [refresh, role]);

  return {
    workspace,
    loading,
    submitting,
    error,
    refresh,
    publishTask,
    reviewSubmission,
    reviewRisk,
    reviewRelief,
    reviewProposal,
    reviewDiscussion,
    finalizeDecision,
  };
}
