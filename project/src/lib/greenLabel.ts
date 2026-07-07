import { Connection, PublicKey } from '@solana/web3.js';

export const GREEN_LABEL_DEVNET_RPC_ENDPOINT = 'https://api.devnet.solana.com';
export const GREEN_LABEL_PROGRAM_ID = new PublicKey('HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY');
export const GREEN_LABEL_CONFIG_PDA = new PublicKey('7hNAeoqZxqvp38giY9gZwfR5ai3ttYTrse63QNYrRBWS');
export const GREEN_LABEL_USDC_DECIMALS = 6;

const GREEN_LABEL_CONFIG_DISCRIMINATOR = [18, 20, 44, 233, 16, 255, 27, 58] as const;
const GREEN_LABEL_CONFIG_ACCOUNT_SIZE = 406;
const MAINNET_MIN_BASE_BOND_USDC_RAW = 299_000_000n;
const DEVNET_TEST_MIN_BASE_BOND_USDC_RAW = 1_000_000n;
const MAINNET_OBSERVATION_SECONDS = 2_592_000n;
const MAINNET_DISPUTE_SECONDS = 604_800n;
const MAINNET_RESPONSE_SECONDS = 259_200n;
const DEVNET_TEST_WINDOW_SECONDS = 30n;

export interface GreenLabelConfigV1 {
  authority: string;
  usdcMint: string;
  minBaseBondUsdc: bigint;
  baseRefundBps: number;
  baseTreasuryBps: number;
  observationPeriodSeconds: bigint;
  disputeWindowSeconds: bigint;
  responseWindowSeconds: bigint;
  projectCount: bigint;
  treasuryUsdcStateV2: string;
  baseBondTreasuryVault: string;
  reliefOrRiskVault: string;
  vaultAuthorityV2: string;
  securityGovernanceConfig: string;
  isPaused: boolean;
  bump: number;
}

export type GreenLabelParameterMode = 'devnet-test' | 'mainnet-like' | 'custom';

export async function fetchGreenLabelConfig(connection: Connection): Promise<GreenLabelConfigV1> {
  const accountInfo = await connection.getAccountInfo(GREEN_LABEL_CONFIG_PDA, 'confirmed');

  if (!accountInfo) {
    throw new Error(`GreenLabelConfigV1 account not found: ${GREEN_LABEL_CONFIG_PDA.toBase58()}`);
  }

  if (!accountInfo.owner.equals(GREEN_LABEL_PROGRAM_ID)) {
    throw new Error(`GreenLabelConfigV1 owner mismatch: ${accountInfo.owner.toBase58()}`);
  }

  return decodeGreenLabelConfig(accountInfo.data);
}

export function decodeGreenLabelConfig(data: Uint8Array): GreenLabelConfigV1 {
  if (data.length < GREEN_LABEL_CONFIG_ACCOUNT_SIZE) {
    throw new Error(`GreenLabelConfigV1 account too small: ${data.length} bytes`);
  }

  for (let index = 0; index < GREEN_LABEL_CONFIG_DISCRIMINATOR.length; index += 1) {
    if (data[index] !== GREEN_LABEL_CONFIG_DISCRIMINATOR[index]) {
      throw new Error('GreenLabelConfigV1 discriminator mismatch');
    }
  }

  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  let offset = 8;

  const authority = readPubkey(data, offset);
  offset += 32;
  const usdcMint = readPubkey(data, offset);
  offset += 32;
  const minBaseBondUsdc = view.getBigUint64(offset, true);
  offset += 8;
  const baseRefundBps = view.getUint16(offset, true);
  offset += 2;
  const baseTreasuryBps = view.getUint16(offset, true);
  offset += 2;
  const observationPeriodSeconds = view.getBigInt64(offset, true);
  offset += 8;
  const disputeWindowSeconds = view.getBigInt64(offset, true);
  offset += 8;
  const responseWindowSeconds = view.getBigInt64(offset, true);
  offset += 8;
  const projectCount = view.getBigUint64(offset, true);
  offset += 8;
  const treasuryUsdcStateV2 = readPubkey(data, offset);
  offset += 32;
  const baseBondTreasuryVault = readPubkey(data, offset);
  offset += 32;
  const reliefOrRiskVault = readPubkey(data, offset);
  offset += 32;
  const vaultAuthorityV2 = readPubkey(data, offset);
  offset += 32;
  const securityGovernanceConfig = readPubkey(data, offset);
  offset += 32;
  const isPaused = data[offset] === 1;
  offset += 1;
  const bump = data[offset];

  return {
    authority,
    usdcMint,
    minBaseBondUsdc,
    baseRefundBps,
    baseTreasuryBps,
    observationPeriodSeconds,
    disputeWindowSeconds,
    responseWindowSeconds,
    projectCount,
    treasuryUsdcStateV2,
    baseBondTreasuryVault,
    reliefOrRiskVault,
    vaultAuthorityV2,
    securityGovernanceConfig,
    isPaused,
    bump,
  };
}

export function formatUsdcAmount(rawAmount: bigint): string {
  const divisor = 10n ** BigInt(GREEN_LABEL_USDC_DECIMALS);
  const whole = rawAmount / divisor;
  const fraction = rawAmount % divisor;

  return `${whole.toString()}.${fraction.toString().padStart(GREEN_LABEL_USDC_DECIMALS, '0')} USDC`;
}

export function formatBps(bps: number): string {
  return `${(bps / 100).toFixed(2).replace(/\.00$/, '')}%`;
}

export function formatDuration(seconds: bigint): string {
  if (seconds === 0n) return '0 秒';

  const absSeconds = seconds < 0n ? -seconds : seconds;
  const sign = seconds < 0n ? '-' : '';

  if (absSeconds % 86_400n === 0n) {
    return `${sign}${(absSeconds / 86_400n).toString()} 天`;
  }

  if (absSeconds % 3_600n === 0n) {
    return `${sign}${(absSeconds / 3_600n).toString()} 小时`;
  }

  if (absSeconds % 60n === 0n) {
    return `${sign}${(absSeconds / 60n).toString()} 分钟`;
  }

  return `${sign}${absSeconds.toString()} 秒`;
}

export function getGreenLabelParameterMode(config: GreenLabelConfigV1): GreenLabelParameterMode {
  const hasDevnetTestParameters = config.minBaseBondUsdc === DEVNET_TEST_MIN_BASE_BOND_USDC_RAW
    || config.observationPeriodSeconds === DEVNET_TEST_WINDOW_SECONDS
    || config.disputeWindowSeconds === DEVNET_TEST_WINDOW_SECONDS
    || config.responseWindowSeconds === DEVNET_TEST_WINDOW_SECONDS;

  if (hasDevnetTestParameters) {
    return 'devnet-test';
  }

  const hasMainnetLikeParameters = config.minBaseBondUsdc === MAINNET_MIN_BASE_BOND_USDC_RAW
    && config.observationPeriodSeconds === MAINNET_OBSERVATION_SECONDS
    && config.disputeWindowSeconds === MAINNET_DISPUTE_SECONDS
    && config.responseWindowSeconds === MAINNET_RESPONSE_SECONDS;

  return hasMainnetLikeParameters ? 'mainnet-like' : 'custom';
}

function readPubkey(data: Uint8Array, offset: number): string {
  return new PublicKey(data.slice(offset, offset + 32)).toBase58();
}
