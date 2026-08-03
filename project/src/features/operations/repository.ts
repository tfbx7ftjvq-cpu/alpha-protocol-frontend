import type { SupabaseClient, User } from '@supabase/supabase-js';
import {
  type MyOperationsSubmission,
  type CommunityTaskPublicationInput,
  type OperationsStaffRole,
  type OperationsStaffWorkspace,
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
  type PublicTaskResult,
  type ReliefApplicationInput,
  type RiskReportInput,
  type TaskSubmissionInput,
  type TaskSubmissionReviewInput,
  validateCommunityTaskPublication,
  validateDiscussion,
  validateReliefApplication,
  validateRiskReport,
  validateTaskSubmission,
  validateTaskSubmissionReview,
} from './domain';
import { assertWalletSessionMatch } from './auth';
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
    taskPublicationResult,
    riskResult,
    reliefResult,
    discussionResult,
    proposalResult,
    decisionResult,
  ] = await Promise.all([
    client
      .from('community_tasks')
      .select('id,title,summary,requirements,reward_budget_usdc,reward_source,status,submission_deadline,published_at')
      .eq('publication_status', 'published')
      .in('status', ['open', 'under_review'])
      .order('published_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('task_submission_publications')
      .select('id,task_id,task_title,result_summary,deliverable_url,wallet_address,review_reference,accepted_at,published_at')
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
  assertNoQueryError(taskPublicationResult.error, '公开任务成果');
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
    taskResults: (taskPublicationResult.data ?? []).map(mapTaskResult),
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
  const { client, user } = await requireIntakeSession(payload.walletAddress);
  const { error } = await client.from('task_submissions').insert({
    task_id: payload.taskId,
    submitted_by: user.id,
    summary: payload.summary,
    deliverable_url: payload.deliverableUrl,
    wallet_address: payload.walletAddress,
    wallet_verified: false,
    public_result_consent: payload.publicResultConsent,
    public_wallet_consent: payload.publicWalletConsent,
    status: 'submitted',
  });

  assertNoMutationError(error, '任务成果');
}

export function resolveOperationsStaffRole(user: User | null): OperationsStaffRole | null {
  const role = user?.app_metadata?.operations_role;
  return role === 'reviewer' || role === 'operator' || role === 'governance_admin'
    ? role
    : null;
}

export async function loadOperationsStaffWorkspace(): Promise<OperationsStaffWorkspace> {
  const { client } = await requireStaffSession();
  const [submissionResult, taskResult, eventResult] = await Promise.all([
    client
      .from('task_submissions')
      .select('id,task_id,submitted_by,summary,deliverable_url,wallet_address,public_result_consent,public_wallet_consent,status,reviewer_notes,created_at')
      .in('status', ['submitted', 'in_review'])
      .order('created_at', { ascending: true })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('community_tasks')
      .select('id,title')
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('operations_task_workflow_events')
      .select('event_id,entity_type,entity_reference,action,actor_role,event_reference,created_at')
      .order('created_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
  ]);

  assertNoQueryError(submissionResult.error, '待审核任务成果');
  assertNoQueryError(taskResult.error, '任务标题');
  assertNoQueryError(eventResult.error, '任务工作流审计记录');
  const taskTitles = new Map((taskResult.data ?? []).map((row) => [row.id, row.title]));

  return {
    submissions: (submissionResult.data ?? []).map((row) => ({
      id: row.id,
      taskId: row.task_id,
      taskTitle: taskTitles.get(row.task_id) ?? '未找到任务标题',
      summary: row.summary,
      deliverableUrl: row.deliverable_url,
      walletAddress: row.wallet_address,
      publicResultConsent: row.public_result_consent,
      publicWalletConsent: row.public_wallet_consent,
      status: row.status,
      submittedBy: row.submitted_by,
      reviewerNotes: row.reviewer_notes,
      createdAt: row.created_at,
    })),
    events: (eventResult.data ?? []).map((row) => ({
      eventId: String(row.event_id),
      entityType: row.entity_type,
      entityReference: row.entity_reference,
      action: row.action,
      actorRole: row.actor_role,
      eventReference: row.event_reference,
      createdAt: row.created_at,
    })),
  };
}

export async function publishCommunityTask(
  input: CommunityTaskPublicationInput,
): Promise<string> {
  const payload = validateCommunityTaskPublication(input);
  const { client, role } = await requireStaffSession();
  if (role === 'reviewer') {
    throw new OperationsBackendError('reviewer 不能发布社区任务');
  }

  const result = await client.rpc('publish_community_task_v1', {
    p_title: payload.title,
    p_summary: payload.summary,
    p_requirements: payload.requirements,
    p_reward_budget_usdc: payload.rewardBudgetUsdc,
    p_reward_source: payload.rewardSource,
    p_submission_deadline: payload.submissionDeadline,
    p_audit_reference: payload.auditReference,
  });
  assertNoMutationError(result.error, '社区任务发布');
  if (typeof result.data !== 'string') {
    throw new OperationsBackendError('社区任务发布结果缺少任务标识');
  }
  return result.data;
}

export async function reviewTaskSubmission(input: TaskSubmissionReviewInput): Promise<void> {
  const payload = validateTaskSubmissionReview(input);
  const { client } = await requireStaffSession();
  const result = await client.rpc('review_task_submission_v1', {
    p_submission_id: payload.submissionId,
    p_decision: payload.decision,
    p_reviewer_notes: payload.reviewerNotes,
    p_public_result_summary: payload.publicResultSummary,
    p_public_deliverable_url: payload.publicDeliverableUrl,
    p_audit_reference: payload.auditReference,
  });
  assertNoMutationError(result.error, '任务成果审核');
}

export async function submitRiskReport(input: RiskReportInput): Promise<void> {
  const payload = validateRiskReport(input);
  const { client, user } = await requireIntakeSession(payload.walletAddress);
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
  const { client, user } = await requireIntakeSession(payload.walletAddress);
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
  const { client, user } = await requireIntakeSession(payload.walletAddress);
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

export async function loadMyOperationsSubmissions(
  connectedWallet: string,
): Promise<MyOperationsSubmission[]> {
  const { client } = await requireIntakeSession(connectedWallet);
  const [taskResult, riskResult, reliefResult, discussionResult] = await Promise.all([
    client
      .from('task_submissions')
      .select('id,summary,status,reviewer_notes,created_at,updated_at')
      .order('created_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('risk_reports')
      .select('id,project_identifier,review_status,reviewer_notes,created_at,updated_at')
      .order('created_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('relief_applications')
      .select('id,incident_summary,status,reviewer_notes,created_at,updated_at')
      .order('created_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('governance_discussions')
      .select('id,topic,moderation_status,created_at,updated_at')
      .order('created_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
  ]);

  assertNoQueryError(taskResult.error, '我的任务提交');
  assertNoQueryError(riskResult.error, '我的风险报告');
  assertNoQueryError(reliefResult.error, '我的救助申请');
  assertNoQueryError(discussionResult.error, '我的治理讨论');

  return [
    ...(taskResult.data ?? []).map((row) => ({
      id: row.id,
      kind: 'task' as const,
      title: summarizePrivateText(row.summary, '任务成果'),
      status: row.status,
      reviewerNotes: row.reviewer_notes,
      createdAt: row.created_at,
      updatedAt: row.updated_at,
    })),
    ...(riskResult.data ?? []).map((row) => ({
      id: row.id,
      kind: 'risk' as const,
      title: row.project_identifier,
      status: row.review_status,
      reviewerNotes: row.reviewer_notes,
      createdAt: row.created_at,
      updatedAt: row.updated_at,
    })),
    ...(reliefResult.data ?? []).map((row) => ({
      id: row.id,
      kind: 'relief' as const,
      title: summarizePrivateText(row.incident_summary, '救助申请'),
      status: row.status,
      reviewerNotes: row.reviewer_notes,
      createdAt: row.created_at,
      updatedAt: row.updated_at,
    })),
    ...(discussionResult.data ?? []).map((row) => ({
      id: row.id,
      kind: 'discussion' as const,
      title: row.topic,
      status: row.moderation_status,
      reviewerNotes: null,
      createdAt: row.created_at,
      updatedAt: row.updated_at,
    })),
  ]
    .sort((left, right) => Date.parse(right.createdAt) - Date.parse(left.createdAt))
    .slice(0, OPERATIONS_PUBLIC_RECORD_LIMIT);
}

async function requireIntakeSession(
  connectedWallet: string,
): Promise<{ client: SupabaseClient; user: User }> {
  if (!operationsBackendConfig.intakeEnabled) {
    throw new OperationsBackendError(
      operationsBackendConfig.reason ?? '社区提交入口尚未启用',
    );
  }

  const client = getOperationsSupabase();
  if (!client) {
    throw new OperationsBackendError('运营后端尚未配置');
  }

  const userResult = await client.auth.getUser();
  if (userResult.error || !userResult.data.user) {
    throw new OperationsBackendError('缺少有效的钱包认证会话，请先签名认证');
  }

  try {
    assertWalletSessionMatch(userResult.data.user, connectedWallet);
  } catch (error) {
    throw new OperationsBackendError(
      error instanceof Error ? error.message : '钱包认证会话不匹配',
    );
  }

  const [intakeResult, walletResult] = await Promise.all([
    client.rpc('is_operations_wallet_intake_enabled'),
    client.rpc('current_verified_solana_wallet'),
  ]);
  if (intakeResult.error || intakeResult.data !== true) {
    throw new OperationsBackendError('数据库端钱包提交总闸门仍为关闭状态');
  }

  if (walletResult.error || walletResult.data !== connectedWallet) {
    throw new OperationsBackendError('数据库未确认当前 Web3 钱包身份，提交已中止');
  }

  return { client, user: userResult.data.user };
}

async function requireStaffSession(): Promise<{
  client: SupabaseClient;
  user: User;
  role: OperationsStaffRole;
}> {
  const client = getOperationsSupabase();
  if (!client) {
    throw new OperationsBackendError('运营后端尚未配置');
  }

  const userResult = await client.auth.getUser();
  const user = userResult.data.user;
  const role = resolveOperationsStaffRole(user);
  if (userResult.error || !user || !role) {
    throw new OperationsBackendError('当前会话没有运营审核权限');
  }

  return { client, user, role };
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

function summarizePrivateText(value: string, fallback: string): string {
  const normalized = value.trim().replace(/\s+/g, ' ');
  if (!normalized) {
    return fallback;
  }

  return normalized.length > 80 ? `${normalized.slice(0, 77)}…` : normalized;
}

function mapTask(row: {
  id: string;
  title: string;
  summary: string;
  requirements: string;
  reward_budget_usdc: string | number | null;
  reward_source: string | null;
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
    rewardSource: row.reward_source,
    status: row.status,
    submissionDeadline: row.submission_deadline,
    publishedAt: row.published_at,
  };
}

function mapTaskResult(row: {
  id: string;
  task_id: string;
  task_title: string;
  result_summary: string;
  deliverable_url: string;
  wallet_address: string | null;
  review_reference: string;
  accepted_at: string;
  published_at: string;
}): PublicTaskResult {
  return {
    id: row.id,
    taskId: row.task_id,
    taskTitle: row.task_title,
    resultSummary: row.result_summary,
    deliverableUrl: row.deliverable_url,
    walletAddress: row.wallet_address,
    reviewReference: row.review_reference,
    acceptedAt: row.accepted_at,
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
