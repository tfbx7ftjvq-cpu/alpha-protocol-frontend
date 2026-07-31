import assert from 'node:assert/strict';
import test from 'node:test';
import {
  assertWalletSessionMatch,
  buildOperationsWeb3AuthOptions,
  getVerifiedSolanaWallet,
  normalizeTurnstileToken,
  TURNSTILE_TOKEN_MAX_LENGTH,
} from '../src/features/operations/auth.ts';

const WALLET = '11111111111111111111111111111111';
const OTHER_WALLET = 'HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY';

test('a unique Supabase Web3 Solana identity resolves to its verified address', () => {
  assert.equal(
    getVerifiedSolanaWallet({
      identities: [{
        provider: 'web3',
        identity_data: {
          sub: `web3:solana:${WALLET}`,
          chain: 'solana',
          address: WALLET,
        },
      }],
    }),
    WALLET,
  );
});

test('email, malformed, non-Solana, and ambiguous identities fail closed', () => {
  for (const user of [
    null,
    { identities: null },
    {
      identities: [{
        provider: 'email',
        identity_data: { chain: 'solana', address: WALLET },
      }],
    },
    {
      identities: [{
        provider: 'web3',
        identity_data: { chain: 'ethereum', address: WALLET },
      }],
    },
    {
      identities: [{
        provider: 'web3',
        identity_data: { chain: 'solana', address: 'not-a-solana-key' },
      }],
    },
    {
      identities: [
        {
          provider: 'web3',
          identity_data: { chain: 'solana', address: WALLET },
        },
        {
          provider: 'web3',
          identity_data: { chain: 'solana', address: OTHER_WALLET },
        },
      ],
    },
  ]) {
    assert.equal(getVerifiedSolanaWallet(user), null);
  }
});

test('wallet session matching rejects disconnected and switched wallets', () => {
  const user = {
    identities: [{
      provider: 'web3',
      identity_data: { chain: 'solana', address: WALLET },
    }],
  };

  assert.equal(assertWalletSessionMatch(user, WALLET), WALLET);
  assert.throws(() => assertWalletSessionMatch(user, null), /连接 Solana 钱包/);
  assert.throws(() => assertWalletSessionMatch(user, OTHER_WALLET), /不一致/);
});

test('Turnstile token validation accepts boundaries and rejects unsafe values', () => {
  const minimumToken = 'a'.repeat(20);
  const maximumToken = 'b'.repeat(TURNSTILE_TOKEN_MAX_LENGTH);

  assert.equal(normalizeTurnstileToken(minimumToken), minimumToken);
  assert.equal(normalizeTurnstileToken(maximumToken), maximumToken);

  for (const token of [
    '',
    'a'.repeat(19),
    ` ${minimumToken}`,
    `${minimumToken}\n`,
    `valid-token ${minimumToken}`,
    'x'.repeat(TURNSTILE_TOKEN_MAX_LENGTH + 1),
  ]) {
    assert.throws(() => normalizeTurnstileToken(token), /Turnstile/);
  }
});

test('Web3 auth options bind the exact page and forward the CAPTCHA token', () => {
  const url = 'https://alpha.example/operations';
  const captchaToken = 'turnstile-token-for-one-auth-attempt';

  assert.deepEqual(
    buildOperationsWeb3AuthOptions(url, captchaToken),
    { url, captchaToken },
  );
});
