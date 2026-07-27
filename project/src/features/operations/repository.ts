import type { SupabaseClient, User } from '@supabase/supabase-js';
import {
  OPERATIONS_PUBLIC_RECORD_LIMIT,
  type CommunityTask,
  type DiscussionInput,
  type GovernanceDecisionValue,
  type OperationsOverview,
  type PublicDiscussion,
  type PublicGovernanceDecision,
  type PublicReliefOutcome,
  type PublicReliefUpdate,
  type PublicRiskReport,
  type ReliefApplicationInput,
  type RiskReportInput,
  type TaskSubmissionInput,
  validateDiscussion,
  validateReliefApplication,
  validateRiskReport,
  validateTaskSubmission,
} from './domain';
import {
  getOperationsSupabase,
  operationsBackendConfig,
} from '../../lib/operationsSupabase';

export class OperationsBackendError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'OperationsBackendError';
  }
}

export async function loadOperationsOverview(): Promise<OperationsOverview> {
  const client = requirePublicReadClient();

  const [
    taskResult,
    riskResult,
    reliefResult,
    discussionResult,
    proposalResult,
    decisionResult,
  ] = await Promise.all([
    client
      .from('community_tasks')
      .select('id,title,summary,requirements,reward_budget_usdc,status,submission_deadline,published_at')
      .eq('publication_status', 'published')
      .in('status', ['open', 'under_review'])
      .order('published_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('risk_publications')
      .select('id,project_identifier,summary,reference_url,public_status,published_at')
      .order('published_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('relief_public_updates')
      .select('id,case_reference,title,summary,outcome,published_at')
      .order('published_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('governance_discussion_publications')
      .select('id,topic,body,wallet_address,published_at')
      .order('published_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('governance_proposals')
      .select('id,title')
      .eq('publication_status', 'published')
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('governance_decisions')
      .select('id,proposal_id,decision,rationale,execution_required,execution_reference,decided_at')
      .eq('publication_status', 'published')
      .order('decided_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
  ]);

  assertNoQueryError(taskResult.error, '社区任务');
  assertNoQueryError(riskResult.error, '风险报告');
  assertNoQueryError(reliefResult.error, '救助公开进度');
  assertNoQueryError(discussionResult.error, '治理讨论');
  assertNoQueryError(proposalResult.error, '治理提案');
  assertNoQueryError(decisionResult.error, '治理决定');

  const proposals = new Map(
    (proposalResult.data ?? []).map((row) => [row.id, row.title]),
  );

  return {
    tasks: (taskResult.data ?? []).map(mapTask),
    riskReports: (riskResult.data ?? []).map(mapRiskReport),
    reliefUpdates: (reliefResult.data ?? []).map(mapReliefUpdate),
    discussions: (discussionResult.data ?? []).map(mapDiscussion),
    governanceDecisions: (decisionResult.data ?? []).map((row) => mapDecision(
      row,
      proposals.get(row.proposal_id) ?? '未公开提案标题',
    )),
  };
}

export async function submitTaskResult(input: TaskSubmissionInput): Promise<void> {
  const payload = validateTaskSubmission(input);
  const { client, user } = await requireIntakeSession();
  const { error } = await client.from('task_submissions').insert({
    task_id: payload.taskId,
    submitted_by: user.id,
    summary: payload.summary,
    deliverable_url: payload.deliverableUrl,
    wallet_address: payload.walletAddress,
    wallet_verified: false,
    status: 'submitted',
  });

  assertNoMutationError(error, '任务成果');
}

export async function submitRiskReport(input: RiskReportInput): Promise<void> {
  const payload = validateRiskReport(input);
  const { client, user } = await requireIntakeSession();
  const { error } = await client.from('risk_reports').insert({
    submitted_by: user.id,
    project_identifier: payload.projectIdentifier,
    summary: payload.summary,
    reference_url: payload.referenceUrl,
    wallet_address: payload.walletAddress,
    wallet_verified: false,
    review_status: 'submitted',
    publication_status: 'private',
  });

  assertNoMutationError(error, '风险报告');
}

export async function submitReliefApplication(input: ReliefApplicationInput): Promise<void> {
  const payload = validateReliefApplication(input);
  const { client, user } = await requireIntakeSession();
  const { error } = await client.from('relief_applications').insert({
    submitted_by: user.id,
    incident_summary: payload.incidentSummary,
    requested_amount_usdc: payload.requestedAmountUsdc,
    evidence_url: payload.evidenceUrl,
    wallet_address: payload.walletAddress,
    wallet_verified: false,
    status: 'submitted',
  });

  assertNoMutationError(error, '救助申请');
}

export async function submitDiscussion(input: DiscussionInput): Promise<void> {
  const payload = validateDiscussion(input);
  const { client, user } = await requireIntakeSession();
  const { error } = await client.from('governance_discussions').insert({
    submitted_by: user.id,
    topic: payload.topic,
    body: payload.body,
    wallet_address: payload.walletAddress,
    wallet_verified: false,
    moderation_status: 'pending',
  });

  assertNoMutationError(error, '治理讨论');
}

async function requireIntakeSession(): Promise<{ client: SupabaseClient; user: User }> {
  if (!operationsBackendConfig.intakeEnabled) {
    throw new OperationsBackendError(
      operationsBackendConfig.reason ?? '社区提交入口尚未启用',
    );
  }

  const client = getOperationsSupabase();
  if (!client) {
    throw new OperationsBackendError('运营后端尚未配置');
  }

  const sessionResult = await client.auth.getSession();
  if (sessionResult.error) {
    throw new OperationsBackendError('无法读取匿名提交会话');
  }

  if (sessionResult.data.session?.user) {
    return { client, user: sessionResult.data.session.user };
  }

  const signInResult = await client.auth.signInAnonymously();
  if (signInResult.error || !signInResult.data.user) {
    throw new OperationsBackendError('匿名提交会话创建失败，请确认 Supabase Anonymous Sign-Ins 已开启');
  }

  return { client, user: signInResult.data.user };
}

function requirePublicReadClient(): SupabaseClient {
  if (!operationsBackendConfig.publicReadEnabled) {
    throw new OperationsBackendError(
      operationsBackendConfig.reason ?? '运营后端尚未配置',
    );
  }

  const client = getOperationsSupabase();
  if (!client) {
    throw new OperationsBackendError('运营后端尚未配置');
  }

  return client;
}

function assertNoQueryError(error: { message: string } | null, label: string): void {
  if (error) {
    throw new OperationsBackendError(`${label}读取失败；未展示缓存或模拟数据`);
  }
}

function assertNoMutationError(error: { message: string } | null, label: string): void {
  if (error) {
    throw new OperationsBackendError(`${label}未提交成功，请检查权限或稍后重试`);
  }
}

function mapTask(row: {
  id: string;
  title: string;
  summary: string;
  requirements: string;
  reward_budget_usdc: string | number | null;
  status: CommunityTask['status'];
  submission_deadline: string | null;
  published_at: string;
}): CommunityTask {
  return {
    id: row.id,
    title: row.title,
    summary: row.summary,
    requirements: row.requirements,
    rewardBudgetUsdc: row.reward_budget_usdc === null ? null : String(row.reward_budget_usdc),
    status: row.status,
    submissionDeadline: row.submission_deadline,
    publishedAt: row.published_at,
  };
}

function mapRiskReport(row: {
  id: string;
  project_identifier: string;
  summary: string;
  reference_url: string | null;
  public_status: PublicRiskReport['publicStatus'];
  published_at: string;
}): PublicRiskReport {
  return {
    id: row.id,
    projectIdentifier: row.project_identifier,
    summary: row.summary,
    referenceUrl: row.reference_url,
    publicStatus: row.public_status,
    publishedAt: row.published_at,
  };
}

function mapReliefUpdate(row: {
  id: string;
  case_reference: string;
  title: string;
  summary: string;
  outcome: PublicReliefOutcome;
  published_at: string;
}): PublicReliefUpdate {
  return {
    id: row.id,
    caseReference: row.case_reference,
    title: row.title,
    summary: row.summary,
    outcome: row.outcome,
    publishedAt: row.published_at,
  };
}

function mapDiscussion(row: {
  id: string;
  topic: string;
  body: string;
  wallet_address: string | null;
  published_at: string;
}): PublicDiscussion {
  return {
    id: row.id,
    topic: row.topic,
    body: row.body,
    walletAddress: row.wallet_address,
    publishedAt: row.published_at,
  };
}

function mapDecision(
  row: {
    id: string;
    proposal_id: string;
    decision: GovernanceDecisionValue;
    rationale: string;
    execution_required: boolean;
    execution_reference: string | null;
    decided_at: string;
  },
  proposalTitle: string,
): PublicGovernanceDecision {
  return {
    id: row.id,
    proposalId: row.proposal_id,
    proposalTitle,
    decision: row.decision,
    rationale: row.rationale,
    executionRequired: row.execution_required,
    executionReference: row.execution_reference,
    decidedAt: row.decided_at,
  };
}
