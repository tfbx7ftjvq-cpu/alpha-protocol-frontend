import assert from 'node:assert/strict';
import test from 'node:test';
import {
  assertWalletSessionMatch,
  buildOperationsWeb3AuthOptions,
  getVerifiedSolanaWallet,
  normalizeTurnstileToken,
  parseSupabaseSolanaWeb3Subject,
  SUPABASE_SOLANA_WEB3_SUB_PREFIX,
  TURNSTILE_TOKEN_MAX_LENGTH,
} from '../src/features/operations/auth.ts';
import { resolveOperationsWalletAccess } from '../src/features/operations/walletAccess.ts';

const WALLET = '11111111111111111111111111111111';
const OTHER_WALLET = 'HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY';

test('a unique Supabase Web3 Solana identity resolves to its verified address', () => {
  assert.equal(
    getVerifiedSolanaWallet({
      identities: [{
        provider: 'web3',
        identity_data: {
          sub: `web3:solana:${WALLET}`,
        },
      }],
    }),
    WALLET,
  );
});

test('Supabase Solana Web3 subjects require the exact prefix and a 32-byte Base58 key', () => {
  assert.equal(
    parseSupabaseSolanaWeb3Subject(`${SUPABASE_SOLANA_WEB3_SUB_PREFIX}${WALLET}`),
    WALLET,
  );

  for (const subject of [
    null,
    WALLET,
    `web3:ethereum:${WALLET}`,
    `Web3:solana:${WALLET}`,
    `web3:solana:${WALLET} `,
    `web3:solana:${WALLET}:extra`,
    'web3:solana:not-a-solana-key',
  ]) {
    assert.equal(parseSupabaseSolanaWeb3Subject(subject), null);
  }
});

test('email, legacy, contradictory, malformed, and ambiguous identities fail closed', () => {
  for (const user of [
    null,
    { identities: null },
    {
      identities: [{
        provider: 'email',
        identity_data: { sub: `web3:solana:${WALLET}` },
      }],
    },
    {
      identities: [{
        provider: 'web3',
        identity_data: { chain: 'solana', address: WALLET },
      }],
    },
    {
      identities: [{
        provider: 'web3',
        identity_data: { sub: `web3:ethereum:${WALLET}` },
      }],
    },
    {
      identities: [{
        provider: 'web3',
        identity_data: { sub: 'web3:solana:not-a-solana-key' },
      }],
    },
    {
      identities: [{
        provider: 'web3',
        identity_data: {
          sub: `web3:solana:${WALLET}`,
          chain: 'ethereum',
        },
      }],
    },
    {
      identities: [{
        provider: 'web3',
        identity_data: {
          sub: `web3:solana:${WALLET}`,
          address: OTHER_WALLET,
        },
      }],
    },
    {
      identities: [
        {
          provider: 'web3',
          identity_data: { sub: `web3:solana:${WALLET}` },
        },
        {
          provider: 'web3',
          identity_data: { sub: `web3:solana:${OTHER_WALLET}` },
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
      identity_data: { sub: `web3:solana:${WALLET}` },
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

test('wallet session verification is independent from the database intake gate', () => {
  const locked = resolveOperationsWalletAccess(
    'authenticated',
    WALLET,
    WALLET,
    'disabled',
  );
  assert.deepEqual(locked, {
    sessionVerified: true,
    intakeEnabled: false,
  });

  const enabled = resolveOperationsWalletAccess(
    'authenticated',
    WALLET,
    WALLET,
    'enabled',
  );
  assert.deepEqual(enabled, {
    sessionVerified: true,
    intakeEnabled: true,
  });
});

test('intake stays locked for gate errors, signed-out sessions, and switched wallets', () => {
  for (const access of [
    resolveOperationsWalletAccess('authenticated', WALLET, WALLET, 'error'),
    resolveOperationsWalletAccess('signed-out', null, WALLET, 'enabled'),
    resolveOperationsWalletAccess('wallet-mismatch', WALLET, OTHER_WALLET, 'enabled'),
    resolveOperationsWalletAccess('authenticated', WALLET, OTHER_WALLET, 'enabled'),
  ]) {
    assert.equal(access.intakeEnabled, false);
  }
});
