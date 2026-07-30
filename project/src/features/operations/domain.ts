export const OPERATIONS_PUBLIC_RECORD_LIMIT = 24;
export const MAX_USDC_REQUEST = 1_000_000_000;

export type CommunityTaskStatus = 'open' | 'under_review';
export type PublicRiskStatus = 'published' | 'resolved' | 'dismissed';
export type PublicReliefOutcome = 'reviewing' | 'approved' | 'rejected' | 'paid' | 'cancelled';
export type GovernanceDecisionValue = 'approved' | 'rejected' | 'cancelled';
export type MyOperationsSubmissionKind = 'task' | 'risk' | 'relief' | 'discussion';

export interface CommunityTask {
  id: string;
  title: string;
  summary: string;
  requirements: string;
  rewardBudgetUsdc: string | null;
  status: CommunityTaskStatus;
  submissionDeadline: string | null;
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
}

export interface RiskReportInput {
  projectIdentifier: string;
  summary: string;
  referenceUrl: string;
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
}

export interface ValidatedRiskReport {
  projectIdentifier: string;
  summary: string;
  referenceUrl: string;
  walletAddress: string;
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
  return {
    taskId: validateUuid(input.taskId, '任务'),
    summary: validateText(input.summary, '成果说明', 20, 5_000),
    deliverableUrl: validateHttpsUrl(input.deliverableUrl, '成果链接', true),
    walletAddress: validateSolanaAddress(input.walletAddress, '收款钱包', true),
  };
}

export function validateRiskReport(input: RiskReportInput): ValidatedRiskReport {
  return {
    projectIdentifier: validateText(input.projectIdentifier, '项目标识', 2, 160),
    summary: validateText(input.summary, '风险说明', 30, 5_000),
    referenceUrl: validateHttpsUrl(input.referenceUrl, '证据链接', true),
    walletAddress: validateSolanaAddress(input.walletAddress, '认证提交钱包', true),
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
