import { Connection, PublicKey } from '@solana/web3.js';

export const DEVNET_RPC_ENDPOINT = 'https://api.devnet.solana.com';
export const USDC_MINT = new PublicKey('4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU');

export const USDC_TREASURY_V2_VAULTS = {
  relief: {
    label: '赔付池 / Relief Pool',
    address: new PublicKey('GQSK91eQ5zwzGfYchunVqrPtxe3WLokxY88JbzTVcuRM'),
  },
  buyback: {
    label: '回购销毁池 / Buyback & Burn Pool',
    address: new PublicKey('D9M74v2tW78EbyPZgngsrB7DGxF8RMTpejiEyugGgoiR'),
  },
  builders: {
    label: 'DAO 建设者池 / Builders Pool',
    address: new PublicKey('5XXaoWVSxVzyupzSs5NGXx6c8JMPD26QE7oZNmnUBAt8'),
  },
  staking: {
    label: '质押奖励池 / Staking Rewards Pool',
    address: new PublicKey('9nAUb7QG3mALgEUQZ26fHRa4p9BkfvKV5xGp6NFXA8wQ'),
  },
} as const;

export type TreasuryV2VaultKey = keyof typeof USDC_TREASURY_V2_VAULTS;

export interface TreasuryV2VaultBalance {
  key: TreasuryV2VaultKey;
  label: string;
  address: string;
  amount: string;
  decimals: number;
  uiAmountString: string;
}

export interface TreasuryV2Balances {
  relief: TreasuryV2VaultBalance;
  buyback: TreasuryV2VaultBalance;
  builders: TreasuryV2VaultBalance;
  staking: TreasuryV2VaultBalance;
  totalAmount: string;
  totalUiAmountString: string;
  decimals: number;
}

export async function readTreasuryV2Balances(connection: Connection): Promise<TreasuryV2Balances> {
  const [relief, buyback, builders, staking] = await Promise.all([
    readVaultBalance(connection, 'relief'),
    readVaultBalance(connection, 'buyback'),
    readVaultBalance(connection, 'builders'),
    readVaultBalance(connection, 'staking'),
  ]);

  const totalAmount = [relief, buyback, builders, staking]
    .reduce((sum, balance) => sum + BigInt(balance.amount), 0n)
    .toString();
  const decimals = relief.decimals;

  return {
    relief,
    buyback,
    builders,
    staking,
    totalAmount,
    totalUiAmountString: formatTokenAmount(totalAmount, decimals),
    decimals,
  };
}

async function readVaultBalance(
  connection: Connection,
  key: TreasuryV2VaultKey,
): Promise<TreasuryV2VaultBalance> {
  const vault = USDC_TREASURY_V2_VAULTS[key];
  const balance = await connection.getTokenAccountBalance(vault.address);
  const uiAmountString = balance.value.uiAmountString
    ?? formatTokenAmount(balance.value.amount, balance.value.decimals);

  return {
    key,
    label: vault.label,
    address: vault.address.toBase58(),
    amount: balance.value.amount,
    decimals: balance.value.decimals,
    uiAmountString,
  };
}

function formatTokenAmount(amount: string, decimals: number): string {
  const rawAmount = BigInt(amount);
  const divisor = 10n ** BigInt(decimals);
  const whole = rawAmount / divisor;
  const fraction = rawAmount % divisor;

  if (fraction === 0n) {
    return whole.toString();
  }

  const trimmedFraction = fraction.toString().padStart(decimals, '0').replace(/0+$/, '');
  return `${whole.toString()}.${trimmedFraction}`;
}
