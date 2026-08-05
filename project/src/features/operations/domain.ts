export const OPERATIONS_PUBLIC_RECORD_LIMIT = 24;
export const MAX_USDC_REQUEST = 1_000_000_000;

export type CommunityTaskStatus = 'open' | 'under_review';
export type PublicRiskStatus = 'published' | 'resolved' | 'dismissed';
export type PublicReliefOutcome = 'reviewing' | 'approved' | 'rejected' | 'paid' | 'cancelled';
export type GovernanceDecisionValue = 'approved' | 'rejected' | 'cancelled';
export type GovernanceProposalKind = 'task_acceptance' | 'risk_finding' | 'relief_recommendation' | 'builders_spend' | 'buyback_policy' | 'staking_policy' | 'protocol_parameter' | 'other';
export type MyOperationsSubmissionKind = 'task' | 'risk' | 'relief' | 'proposal' | 'discussion';
export type OperationsStaffRole = 'reviewer' | 'relief_reviewer' | 'operator' | 'moderator' | 'governance_admin';
export type TaskReviewDecision = 'accepted' | 'rejected';
export type RiskReviewDecision = 'published' | 'dismissed';
export type ReliefReviewDecision = 'approved' | 'rejected';
export type CommunityTaskRewardSource = 'builders_pool' | 'grant' | 'sponsor' | 'none';

export interface CommunityTask {
  id: string;
  title: string;
  summary: string;
  requirements: string;
  rewardBudgetUsdc: string | null;
  rewardSource: string | null;
  status: CommunityTaskStatus;
  submissionDeadline: string | null;
  publishedAt: string;
}

export interface PublicTaskResult {
  id: string;
  taskId: string;
  taskTitle: string;
  resultSummary: string;
  deliverableUrl: string;
  walletAddress: string | null;
  reviewReference: string;
  acceptedAt: string;
  publishedAt: string;
}

export interface PublicRiskReport {
  id: string;
  projectIdentifier: string;
  summary: string;
  referenceUrl: string | null;
  publicStatus: PublicRiskStatus;
  publishedAt: string;
}

export interface PublicReliefUpdate {
  id: string;
  caseReference: string;
  title: string;
  summary: string;
  outcome: PublicReliefOutcome;
  publishedAt: string;
}

export interface PublicDiscussion {
  id: string;
  topic: string;
  body: string;
  walletAddress: string | null;
  publishedAt: string;
}

export interface PublicGovernanceProposal {
  id: string;
  title: string;
  summary: string;
  proposalKind: GovernanceProposalKind;
  publicSourceReference: string | null;
  executionRequired: boolean;
  executionManifestUrl: string | null;
  status: string;
  publishedAt: string;
}

export interface PublicGovernanceDecision {
  id: string;
  proposalId: string;
  proposalTitle: string;
  decision: GovernanceDecisionValue;
  rationale: string;
  executionRequired: boolean;
  executionReference: string | null;
  executionManifestSha256: string | null;
  decisionHash: string;
  finalizationReference: string;
  decidedAt: string;
}

export interface OperationsOverview {
  tasks: CommunityTask[];
  taskResults: PublicTaskResult[];
  riskReports: PublicRiskReport[];
  reliefUpdates: PublicReliefUpdate[];
  discussions: PublicDiscussion[];
  governanceProposals: PublicGovernanceProposal[];
  governanceDecisions: PublicGovernanceDecision[];
}

export interface MyOperationsSubmission {
  id: string;
  kind: MyOperationsSubmissionKind;
  title: string;
  status: string;
  reviewerNotes: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface TaskSubmissionInput {
  taskId: string;
  summary: string;
  deliverableUrl: string;
  walletAddress: string;
  publicResultConsent: boolean;
  publicWalletConsent: boolean;
}

export interface RiskReportInput {
  projectIdentifier: string;
  summary: string;
  referenceUrl: string;
  walletAddress: string;
  publicReportConsent: boolean;
  publicReferenceConsent: boolean;
}

export interface RiskEvidenceInput {
  riskReportId: string;
  evidenceUrl: string;
  contentSha256: string;
  summary: string;
  walletAddress: string;
}

export interface ReliefApplicationInput {
  incidentSummary: string;
  requestedAmountUsdc: string;
  evidenceUrl: string;
  walletAddress: string;
  publicUpdateConsent: boolean;
}

export interface DiscussionInput {
  proposalId?: string;
  topic: string;
  body: string;
  walletAddress: string;
  publicBodyConsent?: boolean;
  publicWalletConsent?: boolean;
  submissionReference?: string;
}

export interface GovernanceProposalInput {
  title: string;
  privateSummary: string;
  proposalKind: GovernanceProposalKind;
  executionRequired: boolean;
  privateExecutionManifest: string;
  executionManifestSha256: string;
  publicProposalConsent: boolean;
  walletAddress: string;
  submissionReference: string;
}

export interface GovernanceProposalReviewInput {
  proposalSubmissionId: string;
  decision: 'published' | 'rejected';
  reviewerNotes: string;
  publicTitle: string;
  publicSummary: string;
  publicSourceReference: string;
  executionManifestUrl: string;
  executionManifestSha256: string;
  auditReference: string;
}

export interface GovernanceDiscussionReviewInput {
  discussionId: string;
  decision: 'published' | 'rejected';
  reviewerNotes: string;
  publicTopic: string;
  publicBody: string;
  publicationBasis: string;
  auditReference: string;
}

export interface GovernanceDecisionFinalizeInput {
  proposalId: string;
  decision: GovernanceDecisionValue;
  rationale: string;
  executionManifestSha256: string;
  finalizationReference: string;
}

export interface ValidatedTaskSubmission {
  taskId: string;
  summary: string;
  deliverableUrl: string;
  walletAddress: string;
  publicResultConsent: boolean;
  publicWalletConsent: boolean;
}

export interface CommunityTaskPublicationInput {
  title: string;
  summary: string;
  requirements: string;
  rewardBudgetUsdc: string;
  rewardSource: CommunityTaskRewardSource;
  submissionDeadline: string;
  auditReference: string;
}

export interface ValidatedCommunityTaskPublication {
  title: string;
  summary: string;
  requirements: string;
  rewardBudgetUsdc: string | null;
  rewardSource: CommunityTaskRewardSource;
  submissionDeadline: string | null;
  auditReference: string;
}

export interface TaskSubmissionReviewInput {
  submissionId: string;
  decision: TaskReviewDecision;
  reviewerNotes: string;
  publicResultSummary: string;
  publicDeliverableUrl: string;
  auditReference: string;
}

export interface ValidatedTaskSubmissionReview {
  submissionId: string;
  decision: TaskReviewDecision;
  reviewerNotes: string;
  publicResultSummary: string | null;
  publicDeliverableUrl: string | null;
  auditReference: string;
}

export interface StaffTaskSubmission {
  id: string;
  taskId: string;
  taskTitle: string;
  summary: string;
  deliverableUrl: string;
  walletAddress: string;
  publicResultConsent: boolean;
  publicWalletConsent: boolean;
  status: string;
  submittedBy: string;
  reviewerNotes: string | null;
  createdAt: string;
}

export interface TaskWorkflowEvent {
  eventId: string;
  entityType: 'community_task' | 'task_submission';
  entityReference: string;
  action: 'task_published' | 'submission_accepted' | 'submission_rejected' | 'result_published';
  actorRole: OperationsStaffRole;
  eventReference: string;
  createdAt: string;
}

export interface OperationsStaffWorkspace {
  submissions: StaffTaskSubmission[];
  events: TaskWorkflowEvent[];
  riskReports: StaffRiskReport[];
  riskEvents: RiskWorkflowEvent[];
  reliefApplications: StaffReliefApplication[];
  reliefEvents: ReliefWorkflowEvent[];
  proposalSubmissions: StaffGovernanceProposalSubmission[];
  discussions: StaffGovernanceDiscussion[];
  governanceEvents: GovernanceWorkflowEvent[];
}

export interface StaffGovernanceProposalSubmission {
  id: string;
  title: string;
  privateSummary: string;
  proposalKind: GovernanceProposalKind;
  executionRequired: boolean;
  executionManifestSha256: string | null;
  publicProposalConsent: boolean;
  submittedBy: string;
  reviewStatus: string;
  createdAt: string;
}

export interface StaffGovernanceDiscussion {
  id: string;
  proposalId: string | null;
  topic: string;
  body: string;
  walletAddress: string;
  publicBodyConsent: boolean;
  publicWalletConsent: boolean;
  submittedBy: string;
  moderationStatus: string;
  createdAt: string;
}

export interface GovernanceWorkflowEvent {
  eventId: string;
  action: string;
  actorRole: OperationsStaffRole | 'wallet_submitter';
  eventReference: string;
  createdAt: string;
}

export interface ValidatedRiskReport {
  projectIdentifier: string;
  summary: string;
  referenceUrl: string;
  walletAddress: string;
  publicReportConsent: boolean;
  publicReferenceConsent: boolean;
}

export interface ValidatedRiskEvidence {
  riskReportId: string;
  evidenceUrl: string;
  contentSha256: string | null;
  summary: string;
  walletAddress: string;
}

export interface StaffRiskReport {
  id: string;
  projectIdentifier: string;
  summary: string;
  referenceUrl: string;
  walletAddress: string;
  publicReportConsent: boolean;
  publicReferenceConsent: boolean;
  reviewStatus: string;
  submittedBy: string;
  reviewerNotes: string | null;
  evidenceCount: number;
  createdAt: string;
}

export interface RiskWorkflowEvent {
  eventId: string;
  riskReportId: string;
  action: 'report_published' | 'report_dismissed';
  actorRole: OperationsStaffRole;
  eventReference: string;
  createdAt: string;
}

export interface RiskReportReviewInput {
  riskReportId: string;
  decision: RiskReviewDecision;
  reviewerNotes: string;
  publicSummary: string;
  publicReferenceUrl: string;
  publicationBasis: string;
  auditReference: string;
}

export interface StaffReliefApplication {
  id: string;
  incidentSummary: string;
  requestedAmountUsdc: string;
  evidenceUrl: string;
  walletAddress: string;
  publicUpdateConsent: boolean;
  status: string;
  submittedBy: string;
  reviewerNotes: string | null;
  createdAt: string;
}

export interface ReliefWorkflowEvent {
  eventId: string;
  reliefApplicationId: string;
  action: 'application_approved' | 'application_rejected';
  actorRole: OperationsStaffRole;
  eventReference: string;
  createdAt: string;
}

export interface ReliefApplicationReviewInput {
  reliefApplicationId: string;
  decision: ReliefReviewDecision;
  reviewerNotes: string;
  publicTitle: string;
  publicSummary: string;
  publicationBasis: string;
  auditReference: string;
}

export interface ValidatedReliefApplicationReview {
  reliefApplicationId: string;
  decision: ReliefReviewDecision;
  reviewerNotes: string;
  publicTitle: string | null;
  publicSummary: string | null;
  publicationBasis: string | null;
  auditReference: string;
}

export interface ValidatedRiskReportReview {
  riskReportId: string;
  decision: RiskReviewDecision;
  reviewerNotes: string;
  publicSummary: string | null;
  publicReferenceUrl: string | null;
  publicationBasis: string | null;
  auditReference: string;
}

export interface ValidatedReliefApplication {
  incidentSummary: string;
  requestedAmountUsdc: string;
  evidenceUrl: string;
  walletAddress: string;
  publicUpdateConsent: boolean;
}

export interface ValidatedDiscussion {
  proposalId: string | null;
  topic: string;
  body: string;
  walletAddress: string;
  publicBodyConsent: boolean;
  publicWalletConsent: boolean;
  submissionReference: string;
}

export interface ValidatedGovernanceProposal extends Omit<GovernanceProposalInput, 'privateExecutionManifest'> {
  privateExecutionManifest: Record<string, unknown> | null;
}

export class OperationsValidationError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'OperationsValidationError';
  }
}

export function validateTaskSubmission(input: TaskSubmissionInput): ValidatedTaskSubmission {
  if (input.publicWalletConsent && !input.publicResultConsent) {
    throw new OperationsValidationError('公开钱包需要先同意公开脱敏成果');
  }

  return {
    taskId: validateUuid(input.taskId, '任务'),
    summary: validateText(input.summary, '成果说明', 20, 5_000),
    deliverableUrl: validateHttpsUrl(input.deliverableUrl, '成果链接', true),
    walletAddress: validateSolanaAddress(input.walletAddress, '收款钱包', true),
    publicResultConsent: input.publicResultConsent,
    publicWalletConsent: input.publicWalletConsent,
  };
}

export function validateCommunityTaskPublication(
  input: CommunityTaskPublicationInput,
): ValidatedCommunityTaskPublication {
  const rewardSources: CommunityTaskRewardSource[] = [
    'builders_pool',
    'grant',
    'sponsor',
    'none',
  ];
  if (!rewardSources.includes(input.rewardSource)) {
    throw new OperationsValidationError('任务奖励来源无效');
  }

  const deadline = input.submissionDeadline.trim();
  if (deadline && (!Number.isFinite(Date.parse(deadline)) || Date.parse(deadline) <= Date.now())) {
    throw new OperationsValidationError('任务截止时间必须是未来的有效时间');
  }

  return {
    title: validateText(input.title, '任务标题', 4, 160),
    summary: validateText(input.summary, '任务摘要', 20, 3_000),
    requirements: validateText(input.requirements, '任务要求', 20, 5_000),
    rewardBudgetUsdc: validateOptionalUsdcBudget(input.rewardBudgetUsdc),
    rewardSource: input.rewardSource,
    submissionDeadline: deadline ? new Date(deadline).toISOString() : null,
    auditReference: validateAuditReference(input.auditReference, '任务发布审计引用', 180),
  };
}

export function validateTaskSubmissionReview(
  input: TaskSubmissionReviewInput,
): ValidatedTaskSubmissionReview {
  if (input.decision !== 'accepted' && input.decision !== 'rejected') {
    throw new OperationsValidationError('审核决定必须是 accepted 或 rejected');
  }

  const publicSummary = input.publicResultSummary.trim();
  const publicUrl = input.publicDeliverableUrl.trim();
  if (input.decision === 'rejected' && (publicSummary || publicUrl)) {
    throw new OperationsValidationError('拒绝任务时不能发布公开成果');
  }

  return {
    submissionId: validateUuid(input.submissionId, '任务提交'),
    decision: input.decision,
    reviewerNotes: validateText(input.reviewerNotes, '审核说明', 1, 5_000),
    publicResultSummary: input.decision === 'accepted'
      ? validateText(publicSummary, '公开成果摘要', 20, 3_000)
      : null,
    publicDeliverableUrl: input.decision === 'accepted'
      ? validateHttpsUrl(publicUrl, '公开成果链接', true)
      : null,
    auditReference: validateAuditReference(input.auditReference, '任务审核审计引用', 160),
  };
}

export function validateRiskReport(input: RiskReportInput): ValidatedRiskReport {
  if (input.publicReferenceConsent && !input.publicReportConsent) {
    throw new OperationsValidationError('公开证据链接需要先同意公开脱敏风险记录');
  }

  return {
    projectIdentifier: validateText(input.projectIdentifier, '项目标识', 2, 160),
    summary: validateText(input.summary, '风险说明', 30, 5_000),
    referenceUrl: validateHttpsUrl(input.referenceUrl, '证据链接', true),
    walletAddress: validateSolanaAddress(input.walletAddress, '认证提交钱包', true),
    publicReportConsent: input.publicReportConsent,
    publicReferenceConsent: input.publicReferenceConsent,
  };
}

export function validateRiskEvidence(input: RiskEvidenceInput): ValidatedRiskEvidence {
  const hash = input.contentSha256.trim().toLowerCase();
  if (hash && !/^[0-9a-f]{64}$/.test(hash)) {
    throw new OperationsValidationError('证据 SHA-256 必须是 64 位小写十六进制');
  }

  return {
    riskReportId: validateUuid(input.riskReportId, '风险报告'),
    evidenceUrl: validateHttpsUrl(input.evidenceUrl, '追加证据链接', true),
    contentSha256: hash || null,
    summary: validateText(input.summary, '追加证据说明', 10, 2_000),
    walletAddress: validateSolanaAddress(input.walletAddress, '认证提交钱包', true),
  };
}

export function validateRiskReportReview(
  input: RiskReportReviewInput,
): ValidatedRiskReportReview {
  if (input.decision !== 'published' && input.decision !== 'dismissed') {
    throw new OperationsValidationError('风险审核决定必须是 published 或 dismissed');
  }

  const publicSummary = input.publicSummary.trim();
  const publicReferenceUrl = input.publicReferenceUrl.trim();
  const publicationBasis = input.publicationBasis.trim();
  if (
    input.decision === 'dismissed'
    && (publicSummary || publicReferenceUrl || publicationBasis)
  ) {
    throw new OperationsValidationError('驳回风险报告时不能创建公开记录');
  }

  return {
    riskReportId: validateUuid(input.riskReportId, '风险报告'),
    decision: input.decision,
    reviewerNotes: validateText(input.reviewerNotes, '风险审核说明', 1, 5_000),
    publicSummary: input.decision === 'published'
      ? validateText(publicSummary, '脱敏公开风险摘要', 30, 5_000)
      : null,
    publicReferenceUrl: input.decision === 'published' && publicReferenceUrl
      ? validateHttpsUrl(publicReferenceUrl, '公开依据链接', true)
      : null,
    publicationBasis: input.decision === 'published'
      ? validateText(publicationBasis, '公开依据说明', 10, 1_000)
      : null,
    auditReference: validateAuditReference(input.auditReference, '风险审核审计引用', 160),
  };
}

export function validateReliefApplication(input: ReliefApplicationInput): ValidatedReliefApplication {
  return {
    incidentSummary: validateText(input.incidentSummary, '事件说明', 50, 8_000),
    requestedAmountUsdc: validateUsdcAmount(input.requestedAmountUsdc),
    evidenceUrl: validateHttpsUrl(input.evidenceUrl, '证据链接', true),
    walletAddress: validateSolanaAddress(input.walletAddress, '拟收款钱包', true),
    publicUpdateConsent: input.publicUpdateConsent,
  };
}

export function validateReliefApplicationReview(
  input: ReliefApplicationReviewInput,
): ValidatedReliefApplicationReview {
  if (input.decision !== 'approved' && input.decision !== 'rejected') {
    throw new OperationsValidationError('救助审核决定必须是 approved 或 rejected');
  }

  const publicTitle = input.publicTitle.trim();
  const publicSummary = input.publicSummary.trim();
  const publicationBasis = input.publicationBasis.trim();
  const hasPublicUpdate = Boolean(publicTitle || publicSummary || publicationBasis);

  if (input.decision === 'rejected' && hasPublicUpdate) {
    throw new OperationsValidationError('拒绝救助申请时不能创建公开进度');
  }

  if (input.decision === 'approved' && hasPublicUpdate) {
    if (!publicTitle || !publicSummary || !publicationBasis) {
      throw new OperationsValidationError('公开救助进度的标题、摘要和依据必须同时填写');
    }
  }

  return {
    reliefApplicationId: validateUuid(input.reliefApplicationId, '救助申请'),
    decision: input.decision,
    reviewerNotes: validateText(input.reviewerNotes, '救助审核说明', 1, 5_000),
    publicTitle: hasPublicUpdate ? validateText(publicTitle, '公开进度标题', 4, 160) : null,
    publicSummary: hasPublicUpdate ? validateText(publicSummary, '脱敏公开进度', 20, 3_000) : null,
    publicationBasis: hasPublicUpdate ? validateText(publicationBasis, '公开依据', 10, 1_000) : null,
    auditReference: validateAuditReference(input.auditReference, '救助审核审计引用', 160),
  };
}

export function validateDiscussion(input: DiscussionInput): ValidatedDiscussion {
  const publicBodyConsent = input.publicBodyConsent ?? false;
  const publicWalletConsent = input.publicWalletConsent ?? false;
  if (publicWalletConsent && !publicBodyConsent) {
    throw new OperationsValidationError('公开发言钱包需要先同意公开脱敏讨论');
  }
  return {
    proposalId: input.proposalId?.trim()
      ? validateUuid(input.proposalId, '治理提案')
      : null,
    topic: validateText(input.topic, '讨论主题', 4, 160),
    body: validateText(input.body, '讨论内容', 20, 5_000),
    walletAddress: validateSolanaAddress(input.walletAddress, '认证发言钱包', true),
    publicBodyConsent,
    publicWalletConsent,
    submissionReference: validateAuditReference(
      input.submissionReference ?? 'legacy-discussion-submission',
      '讨论提交审计引用',
      200,
    ),
  };
}

export function validateGovernanceProposal(
  input: GovernanceProposalInput,
): ValidatedGovernanceProposal {
  const kinds: GovernanceProposalKind[] = [
    'task_acceptance', 'risk_finding', 'relief_recommendation', 'builders_spend',
    'buyback_policy', 'staking_policy', 'protocol_parameter', 'other',
  ];
  if (!kinds.includes(input.proposalKind)) {
    throw new OperationsValidationError('治理提案类型无效');
  }
  const hash = input.executionManifestSha256.trim().toLowerCase();
  let manifest: Record<string, unknown> | null = null;
  if (input.executionRequired) {
    if (!/^[0-9a-f]{64}$/.test(hash)) {
      throw new OperationsValidationError('执行 manifest SHA-256 必须是 64 位小写十六进制');
    }
    try {
      const parsed: unknown = JSON.parse(input.privateExecutionManifest);
      if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) throw new Error();
      manifest = parsed as Record<string, unknown>;
    } catch {
      throw new OperationsValidationError('私有执行 manifest 必须是 JSON 对象');
    }
  } else if (input.privateExecutionManifest.trim() || hash) {
    throw new OperationsValidationError('无需执行的提案不能携带 execution manifest');
  }
  return {
    title: validateText(input.title, '提案标题', 4, 160),
    privateSummary: validateText(input.privateSummary, '私有提案正文', 20, 8_000),
    proposalKind: input.proposalKind,
    executionRequired: input.executionRequired,
    privateExecutionManifest: manifest,
    executionManifestSha256: hash,
    publicProposalConsent: input.publicProposalConsent,
    walletAddress: validateSolanaAddress(input.walletAddress, '认证提案钱包', true),
    submissionReference: validateAuditReference(input.submissionReference, '提案提交审计引用', 200),
  };
}

export function validateText(
  rawValue: string,
  label: string,
  minLength: number,
  maxLength: number,
): string {
  const value = rawValue.trim().replace(/\r\n/g, '\n');

  if (value.length < minLength) {
    throw new OperationsValidationError(`${label}至少需要 ${minLength} 个字符`);
  }

  if (value.length > maxLength) {
    throw new OperationsValidationError(`${label}不能超过 ${maxLength} 个字符`);
  }

  return value;
}

export function validateHttpsUrl(rawValue: string, label: string, required: true): string;
export function validateHttpsUrl(rawValue: string, label: string, required: false): string | null;
export function validateHttpsUrl(rawValue: string, label: string, required: boolean): string | null {
  const value = rawValue.trim();

  if (!value) {
    if (required) {
      throw new OperationsValidationError(`${label}不能为空`);
    }

    return null;
  }

  let parsed: URL;
  try {
    parsed = new URL(value);
  } catch {
    throw new OperationsValidationError(`${label}必须是有效的 HTTPS URL`);
  }

  if (parsed.protocol !== 'https:' || parsed.username || parsed.password) {
    throw new OperationsValidationError(`${label}必须是无内嵌凭据的 HTTPS URL`);
  }

  if (parsed.hostname === 'localhost' || parsed.hostname.endsWith('.local')) {
    throw new OperationsValidationError(`${label}不能指向本地地址`);
  }

  return parsed.toString();
}

export function validateUsdcAmount(rawValue: string): string {
  const value = rawValue.trim();

  if (!/^(?:0|[1-9]\d*)(?:\.\d{1,6})?$/.test(value)) {
    throw new OperationsValidationError('申请金额必须是最多 6 位小数的非负 USDC 数字');
  }

  const amount = Number(value);
  if (!Number.isFinite(amount) || amount <= 0) {
    throw new OperationsValidationError('申请金额必须大于 0 USDC');
  }

  if (amount > MAX_USDC_REQUEST) {
    throw new OperationsValidationError(`申请金额不能超过 ${MAX_USDC_REQUEST} USDC`);
  }

  return value;
}

export function validateOptionalUsdcBudget(rawValue: string): string | null {
  const value = rawValue.trim();
  if (!value) {
    return null;
  }

  if (!/^(?:0|[1-9]\d*)(?:\.\d{1,6})?$/.test(value)) {
    throw new OperationsValidationError('任务预算必须是最多 6 位小数的非负 USDC 数字');
  }

  const amount = Number(value);
  if (!Number.isFinite(amount) || amount > MAX_USDC_REQUEST) {
    throw new OperationsValidationError(`任务预算不能超过 ${MAX_USDC_REQUEST} USDC`);
  }

  return value;
}

export function validateAuditReference(
  rawValue: string,
  label: string,
  maxLength: number,
): string {
  const value = rawValue.trim();
  if (value.length < 10 || value.length > maxLength) {
    throw new OperationsValidationError(`${label}必须为 10 到 ${maxLength} 个字符`);
  }

  for (const character of value) {
    const codePoint = character.codePointAt(0) ?? 0;
    if (codePoint < 32 || codePoint === 127) {
      throw new OperationsValidationError(`${label}不能包含控制字符`);
    }
  }

  return value;
}

export function validateSolanaAddress(
  rawValue: string,
  label: string,
  required: true,
): string;
export function validateSolanaAddress(
  rawValue: string,
  label: string,
  required: false,
): string | null;
export function validateSolanaAddress(
  rawValue: string,
  label: string,
  required: boolean,
): string | null {
  const value = rawValue.trim();

  if (!value) {
    if (required) {
      throw new OperationsValidationError(`${label}不能为空`);
    }

    return null;
  }

  if (!isSolanaPublicKey(value)) {
    throw new OperationsValidationError(`${label}不是有效的 Solana 地址`);
  }

  return value;
}

export function isSolanaPublicKey(value: string): boolean {
  const alphabet = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
  let decoded = 0n;

  if (value.length < 32 || value.length > 44) {
    return false;
  }

  for (const character of value) {
    const digit = alphabet.indexOf(character);
    if (digit < 0) {
      return false;
    }

    decoded = decoded * 58n + BigInt(digit);
  }

  let nonZeroBytes = 0;
  while (decoded > 0n) {
    decoded >>= 8n;
    nonZeroBytes += 1;
  }

  let leadingZeroBytes = 0;
  while (leadingZeroBytes < value.length && value[leadingZeroBytes] === '1') {
    leadingZeroBytes += 1;
  }

  return leadingZeroBytes + nonZeroBytes === 32;
}

function validateUuid(rawValue: string, label: string): string {
  const value = rawValue.trim().toLowerCase();

  if (!/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(value)) {
    throw new OperationsValidationError(`${label}标识无效`);
  }

  return value;
}
