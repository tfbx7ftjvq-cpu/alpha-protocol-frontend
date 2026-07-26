import { Connection, PublicKey } from '@solana/web3.js';

export const MAINNET_RPC_ENDPOINT =
  import.meta.env.VITE_MAINNET_RPC_ENDPOINT?.trim() || 'https://api.mainnet-beta.solana.com';

const USDC_MAINNET_MINT = new PublicKey('EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v');
const TOKEN_PROGRAM_ID = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA');
const ASSOCIATED_TOKEN_PROGRAM_ID = new PublicKey('ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL');

export const REVENUE_SPLIT = {
  relief: 50,
  buyback: 20,
  builders: 20,
  staking: 10,
} as const;

export type LaunchStatus = 'prelaunch' | 'live';

export interface LaunchConfig {
  status: LaunchStatus;
  alphaMint: PublicKey | null;
  revenueWallet: PublicKey | null;
  pumpUrl: string | null;
  distributionThresholdUsdc: number;
}

export interface MainnetRevenueSnapshot {
  revenueWallet: string;
  solBalance: number;
  usdcBalance: number;
  usdcTokenAccount: string;
}

export function readLaunchConfig(): LaunchConfig {
  const status = import.meta.env.VITE_ALPHA_LAUNCH_STATUS === 'live' ? 'live' : 'prelaunch';

  return {
    status,
    alphaMint: readOptionalPublicKey('VITE_ALPHA_MAINNET_MINT'),
    revenueWallet: readOptionalPublicKey('VITE_REVENUE_WALLET'),
    pumpUrl: readOptionalUrl('VITE_ALPHA_PUMP_URL'),
    distributionThresholdUsdc: readPositiveNumber('VITE_REVENUE_DISTRIBUTION_THRESHOLD_USDC', 100),
  };
}

export async function fetchMainnetRevenueSnapshot(
  connection: Connection,
  revenueWallet: PublicKey,
): Promise<MainnetRevenueSnapshot> {
  const usdcTokenAccount = getAssociatedTokenAddress(revenueWallet, USDC_MAINNET_MINT);
  const [lamports, usdcBalance] = await Promise.all([
    connection.getBalance(revenueWallet, 'confirmed'),
    connection.getTokenAccountBalance(usdcTokenAccount, 'confirmed').catch(() => null),
  ]);

  return {
    revenueWallet: revenueWallet.toBase58(),
    solBalance: lamports / 1_000_000_000,
    usdcBalance: usdcBalance?.value.uiAmount ?? 0,
    usdcTokenAccount: usdcTokenAccount.toBase58(),
  };
}

function getAssociatedTokenAddress(owner: PublicKey, mint: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [owner.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
    ASSOCIATED_TOKEN_PROGRAM_ID,
  )[0];
}

export function getMainnetExplorerAddressUrl(address: PublicKey | string): string {
  const value = typeof address === 'string' ? address : address.toBase58();
  return `https://explorer.solana.com/address/${value}`;
}

function readOptionalPublicKey(name: string): PublicKey | null {
  const value = import.meta.env[name]?.trim();
  if (!value) return null;

  try {
    return new PublicKey(value);
  } catch {
    throw new Error(`${name} is not a valid Solana public key.`);
  }
}

function readOptionalUrl(name: string): string | null {
  const value = import.meta.env[name]?.trim();
  if (!value) return null;

  try {
    const url = new URL(value);
    if (url.protocol !== 'https:') throw new Error();
    return url.toString();
  } catch {
    throw new Error(`${name} must be a valid HTTPS URL.`);
  }
}

function readPositiveNumber(name: string, fallback: number): number {
  const value = import.meta.env[name]?.trim();
  if (!value) return fallback;
  const parsed = Number(value);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    throw new Error(`${name} must be a positive number.`);
  }
  return parsed;
}
