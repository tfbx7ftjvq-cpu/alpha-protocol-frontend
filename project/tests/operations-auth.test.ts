import assert from 'node:assert/strict';
import test from 'node:test';
import {
  assertWalletSessionMatch,
  getVerifiedSolanaWallet,
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
