export const OPERATIONS_PUBLIC_RECORD_LIMIT = 24;
export const MAX_USDC_REQUEST = 1_000_000_000;

export type CommunityTaskStatus = 'open' | 'under_review';
export type PublicRiskStatus = 'published' | 'resolved' | 'dismissed';
export type PublicReliefOutcome = 'reviewing' | 'approved' | 'rejected' | 'paid' | 'cancelled';
export type GovernanceDecisionValue = 'approved' | 'rejected' | 'cancelled';
export type MyOperationsSubmissionKind = 'task' | 'risk' | 'relief' | 'discussion';
export type OperationsStaffRole = 'reviewer' | 'operator' | 'governance_admin';
export type TaskReviewDecision = 'accepted' | 'rejected';
export type RiskReviewDecision = 'published' | 'dismissed';
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

export interface PublicGovernanceDecision {
  id: string;
  proposalId: string;
  proposalTitle: string;
  decision: GovernanceDecisionValue;
  rationale: string;
  executionRequired: boolean;
  executionReference: string | null;
  decidedAt: string;
}

export interface OperationsOverview {
  tasks: CommunityTask[];
  taskResults: PublicTaskResult[];
  riskReports: PublicRiskReport[];
  reliefUpdates: PublicReliefUpdate[];
  discussions: PublicDiscussion[];
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
}

export interface DiscussionInput {
  topic: string;
  body: string;
  walletAddress: string;
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
}

export interface ValidatedDiscussion {
  topic: string;
  body: string;
  walletAddress: string;
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
  };
}

export function validateDiscussion(input: DiscussionInput): ValidatedDiscussion {
  return {
    topic: validateText(input.topic, '讨论主题', 4, 160),
    body: validateText(input.body, '讨论内容', 20, 5_000),
    walletAddress: validateSolanaAddress(input.walletAddress, '认证发言钱包', true),
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
