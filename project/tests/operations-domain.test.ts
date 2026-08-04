import assert from 'node:assert/strict';
import test from 'node:test';
import {
  MAX_USDC_REQUEST,
  OperationsValidationError,
  isSolanaPublicKey,
  validateDiscussion,
  validateAuditReference,
  validateCommunityTaskPublication,
  validateHttpsUrl,
  validateReliefApplication,
  validateRiskReport,
  validateRiskEvidence,
  validateRiskReportReview,
  validateTaskSubmission,
  validateTaskSubmissionReview,
  validateUsdcAmount,
  validateOptionalUsdcBudget,
} from '../src/features/operations/domain.ts';

const VALID_WALLET = 'HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY';
const VALID_TASK_ID = '01912b5f-5f3e-7f8d-8da1-3dddb7567b84';

test('Solana public key validation accepts 32-byte base58 keys', () => {
  assert.equal(isSolanaPublicKey(VALID_WALLET), true);
  assert.equal(isSolanaPublicKey('11111111111111111111111111111111'), true);
});

test('Solana public key validation rejects wrong alphabet and byte length', () => {
  assert.equal(isSolanaPublicKey('O0Il-not-base58'), false);
  assert.equal(isSolanaPublicKey('1111111111111111111111111111111'), false);
  assert.equal(isSolanaPublicKey(`${VALID_WALLET}1`), false);
});

test('USDC amount validation accepts six decimals and the configured maximum', () => {
  assert.equal(validateUsdcAmount('0.000001'), '0.000001');
  assert.equal(validateUsdcAmount('42.500000'), '42.500000');
  assert.equal(validateUsdcAmount(String(MAX_USDC_REQUEST)), String(MAX_USDC_REQUEST));
});

test('USDC amount validation rejects zero, negative, excess precision, and oversized values', () => {
  for (const invalid of ['0', '-1', '1.0000001', '01', String(MAX_USDC_REQUEST + 1), 'NaN']) {
    assert.throws(() => validateUsdcAmount(invalid), OperationsValidationError);
  }
});

test('HTTPS evidence URLs reject credentials, localhost, and insecure protocols', () => {
  assert.equal(
    validateHttpsUrl('https://example.org/evidence?id=1', '证据', true),
    'https://example.org/evidence?id=1',
  );

  for (const invalid of [
    'http://example.org/evidence',
    'https://user:pass@example.org/evidence',
    'https://localhost/evidence',
    'not-a-url',
  ]) {
    assert.throws(
      () => validateHttpsUrl(invalid, '证据', true),
      OperationsValidationError,
    );
  }
});

test('task submission normalizes text and requires a valid task and wallet', () => {
  const result = validateTaskSubmission({
    taskId: VALID_TASK_ID,
    summary: '  This delivery contains enough detail to pass validation.  ',
    deliverableUrl: 'https://github.com/example/repository/pull/1',
    walletAddress: VALID_WALLET,
    publicResultConsent: true,
    publicWalletConsent: false,
  });

  assert.equal(result.summary, 'This delivery contains enough detail to pass validation.');
  assert.equal(result.taskId, VALID_TASK_ID);
  assert.equal(result.walletAddress, VALID_WALLET);

  assert.throws(
    () => validateTaskSubmission({
      taskId: 'not-a-uuid',
      summary: result.summary,
      deliverableUrl: result.deliverableUrl,
      walletAddress: result.walletAddress,
      publicResultConsent: true,
      publicWalletConsent: false,
    }),
    OperationsValidationError,
  );
});

test('task submission requires separate public-result and wallet consent', () => {
  assert.throws(
    () => validateTaskSubmission({
      taskId: VALID_TASK_ID,
      summary: 'A complete task result with enough information for private review.',
      deliverableUrl: 'https://example.org/task-result',
      walletAddress: VALID_WALLET,
      publicResultConsent: false,
      publicWalletConsent: true,
    }),
    /公开钱包需要先同意公开脱敏成果/,
  );
});

test('community task publication validates reward source, budget, and future deadline', () => {
  const result = validateCommunityTaskPublication({
    title: 'Review the public task workflow',
    summary: 'Review the complete task workflow and document evidence-backed findings.',
    requirements: 'Provide reproducible checks, a written result, and any known limitations.',
    rewardBudgetUsdc: '0.000001',
    rewardSource: 'builders_pool',
    submissionDeadline: '2099-01-01T00:00:00.000Z',
    auditReference: 'phase-4m-task-publication-001',
  });
  assert.equal(result.rewardBudgetUsdc, '0.000001');
  assert.equal(result.rewardSource, 'builders_pool');
  assert.throws(
    () => validateCommunityTaskPublication({
      ...result,
      rewardBudgetUsdc: '1.0000001',
      submissionDeadline: '',
    }),
    OperationsValidationError,
  );
});

test('optional task budget accepts empty and zero but rejects overflow', () => {
  assert.equal(validateOptionalUsdcBudget(''), null);
  assert.equal(validateOptionalUsdcBudget('0'), '0');
  assert.equal(validateOptionalUsdcBudget('1000000000.000000'), '1000000000.000000');
  assert.throws(() => validateOptionalUsdcBudget('1000000000.000001'), OperationsValidationError);
});

test('accepted task review requires public fields while rejection forbids them', () => {
  const accepted = validateTaskSubmissionReview({
    submissionId: VALID_TASK_ID,
    decision: 'accepted',
    reviewerNotes: 'The evidence was independently reproduced.',
    publicResultSummary: 'A sanitized public result that excludes private submission metadata.',
    publicDeliverableUrl: 'https://example.org/public-result',
    auditReference: 'phase-4m-review-accepted-001',
  });
  assert.equal(accepted.decision, 'accepted');
  assert.throws(
    () => validateTaskSubmissionReview({
      ...accepted,
      decision: 'rejected',
      publicResultSummary: 'This must not be published after rejection.',
      publicDeliverableUrl: '',
    }),
    /拒绝任务时不能发布公开成果/,
  );
});

test('audit references reject control characters without regex ambiguity', () => {
  assert.equal(
    validateAuditReference('phase-4m-audit-reference', '审计引用', 160),
    'phase-4m-audit-reference',
  );
  assert.throws(
    () => validateAuditReference('phase-4m\naudit-reference', '审计引用', 160),
    /控制字符/,
  );
});

test('risk report requires the authenticated wallet and evidence', () => {
  const result = validateRiskReport({
    projectIdentifier: 'Example Project',
    summary: 'Observed behavior is documented separately from unverified interpretation.',
    referenceUrl: 'https://example.org/risk-evidence',
    walletAddress: VALID_WALLET,
    publicReportConsent: true,
    publicReferenceConsent: false,
  });

  assert.equal(result.walletAddress, VALID_WALLET);
  assert.throws(
    () => validateRiskReport({
      projectIdentifier: 'Example Project',
      summary: result.summary,
      referenceUrl: '',
      walletAddress: VALID_WALLET,
      publicReportConsent: true,
      publicReferenceConsent: false,
    }),
    OperationsValidationError,
  );
  assert.throws(
    () => validateRiskReport({
      projectIdentifier: 'Example Project',
      summary: result.summary,
      referenceUrl: result.referenceUrl,
      walletAddress: '',
      publicReportConsent: true,
      publicReferenceConsent: false,
    }),
    OperationsValidationError,
  );
});

test('risk report requires separate consent before a public reference may be used', () => {
  assert.throws(
    () => validateRiskReport({
      projectIdentifier: 'Example Project',
      summary: 'Observed behavior is documented separately from unverified interpretation.',
      referenceUrl: 'https://example.org/private-risk-evidence',
      walletAddress: VALID_WALLET,
      publicReportConsent: false,
      publicReferenceConsent: true,
    }),
    /公开证据链接需要先同意公开脱敏风险记录/,
  );
});

test('risk evidence validates ownership fields, optional hash, and private evidence URL', () => {
  const result = validateRiskEvidence({
    riskReportId: VALID_TASK_ID,
    evidenceUrl: 'https://example.org/additional-risk-evidence',
    contentSha256: 'a'.repeat(64),
    summary: 'Additional evidence with enough context for independent private review.',
    walletAddress: VALID_WALLET,
  });
  assert.equal(result.contentSha256, 'a'.repeat(64));
  assert.equal(
    validateRiskEvidence({ ...result, contentSha256: 'A'.repeat(64) }).contentSha256,
    'a'.repeat(64),
  );
  assert.throws(
    () => validateRiskEvidence({ ...result, contentSha256: 'g'.repeat(64) }),
    OperationsValidationError,
  );
});

test('published risk review requires sanitized public fields while dismissal forbids them', () => {
  const published = validateRiskReportReview({
    riskReportId: VALID_TASK_ID,
    decision: 'published',
    reviewerNotes: 'The private evidence was independently reviewed and the conclusion was documented.',
    publicSummary: 'A sanitized public finding that excludes reporter identity and private evidence metadata.',
    publicReferenceUrl: 'https://example.org/safe-public-risk-reference',
    publicationBasis: 'Independent evidence review and documented governance criteria.',
    auditReference: 'phase-4n-risk-published-001',
  });
  assert.equal(published.decision, 'published');
  assert.throws(
    () => validateRiskReportReview({
      ...published,
      decision: 'dismissed',
      publicSummary: 'This public field is forbidden for a dismissed report.',
      publicReferenceUrl: '',
      publicationBasis: '',
    }),
    /驳回风险报告时不能创建公开记录/,
  );
});

test('relief application requires evidence, a valid wallet, and a positive amount', () => {
  const result = validateReliefApplication({
    incidentSummary: 'A sufficiently detailed incident account with transaction timing and mitigation steps.',
    requestedAmountUsdc: '125.25',
    evidenceUrl: 'https://example.org/relief-evidence',
    walletAddress: VALID_WALLET,
  });

  assert.equal(result.requestedAmountUsdc, '125.25');

  assert.throws(
    () => validateReliefApplication({
      incidentSummary: result.incidentSummary,
      requestedAmountUsdc: '0',
      evidenceUrl: result.evidenceUrl,
      walletAddress: result.walletAddress,
    }),
    OperationsValidationError,
  );
});

test('discussion validation requires the authenticated wallet and substantive content', () => {
  const result = validateDiscussion({
    topic: 'Builder compensation process',
    body: 'Publish acceptance evidence before a spending proposal is created.',
    walletAddress: VALID_WALLET,
  });

  assert.equal(result.walletAddress, VALID_WALLET);
  assert.throws(
    () => validateDiscussion({ topic: 'Tiny', body: 'too short', walletAddress: '' }),
    OperationsValidationError,
  );
});
