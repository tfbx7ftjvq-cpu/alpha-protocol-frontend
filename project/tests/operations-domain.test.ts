import assert from 'node:assert/strict';
import test from 'node:test';
import {
  MAX_USDC_REQUEST,
  OperationsValidationError,
  isSolanaPublicKey,
  validateDiscussion,
  validateHttpsUrl,
  validateReliefApplication,
  validateRiskReport,
  validateTaskSubmission,
  validateUsdcAmount,
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
    }),
    OperationsValidationError,
  );
});

test('risk report requires the authenticated wallet and evidence', () => {
  const result = validateRiskReport({
    projectIdentifier: 'Example Project',
    summary: 'Observed behavior is documented separately from unverified interpretation.',
    referenceUrl: 'https://example.org/risk-evidence',
    walletAddress: VALID_WALLET,
  });

  assert.equal(result.walletAddress, VALID_WALLET);
  assert.throws(
    () => validateRiskReport({
      projectIdentifier: 'Example Project',
      summary: result.summary,
      referenceUrl: '',
      walletAddress: VALID_WALLET,
    }),
    OperationsValidationError,
  );
  assert.throws(
    () => validateRiskReport({
      projectIdentifier: 'Example Project',
      summary: result.summary,
      referenceUrl: result.referenceUrl,
      walletAddress: '',
    }),
    OperationsValidationError,
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
