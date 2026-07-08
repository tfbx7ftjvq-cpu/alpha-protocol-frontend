import { Connection, PublicKey } from '@solana/web3.js';
import {
  DEVNET_RPC_ENDPOINT,
  USDC_MINT,
  USDC_TREASURY_V2_VAULTS,
  getDevnetExplorerAddressUrl,
  type TreasuryV2VaultKey,
} from './devnetTreasuryV2';

export const TREASURY_V2_DEVNET_RPC_ENDPOINT = DEVNET_RPC_ENDPOINT;
export const TREASURY_V2_PROGRAM_ID = new PublicKey('HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY');
export const TREASURY_USDC_STATE_V2_PDA = new PublicKey('5e7eyC5ViwH9GBn73cY6so7J6KpRCX6XsbxozHabk2fE');
export const TREASURY_V2_USDC_MINT = USDC_MINT;
export const TREASURY_V2_VAULT_AUTHORITY = PublicKey.findProgramAddressSync(
  [new TextEncoder().encode('vault_authority_v2')],
  TREASURY_V2_PROGRAM_ID,
)[0];

const TREASURY_USDC_STATE_V2_DISCRIMINATOR = [61, 37, 248, 219, 123, 241, 97, 142] as const;
const TREASURY_USDC_STATE_V2_ACCOUNT_SIZE = 8 + (8 * 5) + 1;

export interface TreasuryUsdcStateV2 {
  totalUsdcInflow: bigint;
  reliefUsdcTotal: bigint;
  buybackUsdcTotal: bigint;
  buildersUsdcTotal: bigint;
  stakingUsdcTotal: bigint;
  bump: number;
}

export interface TreasuryV2VaultRead {
  key: TreasuryV2VaultKey;
  label: string;
  address: string;
  balanceRaw: string | null;
  balanceUi: string | null;
  decimals: number | null;
  explorerUrl: string;
  error: string | null;
}

export interface TreasuryV2Overview {
  treasuryUsdcState: string;
  usdcMint: string;
  vaultAuthority: string;
  state: TreasuryUsdcStateV2 | null;
  stateError: string | null;
  vaults: TreasuryV2VaultRead[];
}

export async function fetchTreasuryV2Overview(connection: Connection): Promise<TreasuryV2Overview> {
  const [stateAccount, vaults] = await Promise.all([
    readTreasuryUsdcStateV2(connection),
    readTreasuryVaults(connection),
  ]);

  return {
    treasuryUsdcState: TREASURY_USDC_STATE_V2_PDA.toBase58(),
    usdcMint: TREASURY_V2_USDC_MINT.toBase58(),
    vaultAuthority: TREASURY_V2_VAULT_AUTHORITY.toBase58(),
    state: stateAccount.state,
    stateError: stateAccount.error,
    vaults,
  };
}

export function formatTreasuryUsdcAmount(amount: bigint | null | undefined): string {
  if (amount === null || amount === undefined) {
    return 'unavailable';
  }

  return `${formatFixedAmount(amount, 6)} USDC`;
}

export function getTreasuryV2ExplorerAddressUrl(address: PublicKey | string): string {
  return getDevnetExplorerAddressUrl(address);
}

async function readTreasuryUsdcStateV2(
  connection: Connection,
): Promise<{ state: TreasuryUsdcStateV2 | null; error: string | null }> {
  try {
    const account = await connection.getAccountInfo(TREASURY_USDC_STATE_V2_PDA, 'confirmed');

    if (!account) {
      return { state: null, error: 'Treasury USDC State V2 account was not found.' };
    }

    if (!account.owner.equals(TREASURY_V2_PROGRAM_ID)) {
      return {
        state: null,
        error: `Treasury USDC State V2 owner mismatch: ${account.owner.toBase58()}`,
      };
    }

    return {
      state: decodeTreasuryUsdcStateV2(account.data),
      error: null,
    };
  } catch (err) {
    return {
      state: null,
      error: getReadableErrorMessage(err),
    };
  }
}

function decodeTreasuryUsdcStateV2(data: Buffer): TreasuryUsdcStateV2 {
  assertAccountLayout(data, TREASURY_USDC_STATE_V2_ACCOUNT_SIZE, TREASURY_USDC_STATE_V2_DISCRIMINATOR);

  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  let offset = 8;
  const totalUsdcInflow = view.getBigUint64(offset, true);
  offset += 8;
  const reliefUsdcTotal = view.getBigUint64(offset, true);
  offset += 8;
  const buybackUsdcTotal = view.getBigUint64(offset, true);
  offset += 8;
  const buildersUsdcTotal = view.getBigUint64(offset, true);
  offset += 8;
  const stakingUsdcTotal = view.getBigUint64(offset, true);
  offset += 8;
  const bump = view.getUint8(offset);

  return {
    totalUsdcInflow,
    reliefUsdcTotal,
    buybackUsdcTotal,
    buildersUsdcTotal,
    stakingUsdcTotal,
    bump,
  };
}

async function readTreasuryVaults(connection: Connection): Promise<TreasuryV2VaultRead[]> {
  const entries = Object.entries(USDC_TREASURY_V2_VAULTS) as Array<[
    TreasuryV2VaultKey,
    typeof USDC_TREASURY_V2_VAULTS[TreasuryV2VaultKey],
  ]>;

  return Promise.all(entries.map(([key, vault]) => readTreasuryVault(connection, key, vault)));
}

async function readTreasuryVault(
  connection: Connection,
  key: TreasuryV2VaultKey,
  vault: typeof USDC_TREASURY_V2_VAULTS[TreasuryV2VaultKey],
): Promise<TreasuryV2VaultRead> {
  const address = vault.address.toBase58();

  try {
    const balance = await connection.getTokenAccountBalance(vault.address, 'confirmed');
    const balanceUi = balance.value.uiAmountString
      ?? formatFixedAmount(BigInt(balance.value.amount), balance.value.decimals);

    return {
      key,
      label: vault.label,
      address,
      balanceRaw: balance.value.amount,
      balanceUi,
      decimals: balance.value.decimals,
      explorerUrl: getTreasuryV2ExplorerAddressUrl(vault.address),
      error: null,
    };
  } catch (err) {
    return {
      key,
      label: vault.label,
      address,
      balanceRaw: null,
      balanceUi: null,
      decimals: null,
      explorerUrl: getTreasuryV2ExplorerAddressUrl(vault.address),
      error: getReadableErrorMessage(err),
    };
  }
}

function assertAccountLayout(
  data: Buffer,
  expectedSize: number,
  discriminator: readonly number[],
): void {
  if (data.length < expectedSize) {
    throw new Error(`TreasuryUsdcStateV2 data too short: ${data.length} bytes`);
  }

  for (let index = 0; index < discriminator.length; index += 1) {
    if (data[index] !== discriminator[index]) {
      throw new Error('TreasuryUsdcStateV2 discriminator mismatch');
    }
  }
}

function formatFixedAmount(amount: bigint, decimals: number): string {
  const divisor = 10n ** BigInt(decimals);
  const whole = amount / divisor;
  const fraction = amount % divisor;

  if (fraction === 0n) {
    return `${whole.toString()}.${'0'.repeat(decimals)}`;
  }

  return `${whole.toString()}.${fraction.toString().padStart(decimals, '0')}`;
}

function getReadableErrorMessage(err: unknown): string {
  if (err instanceof Error) {
    return err.message;
  }

  return String(err);
}
