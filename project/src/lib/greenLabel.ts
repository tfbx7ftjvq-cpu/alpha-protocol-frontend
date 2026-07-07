import { Connection, PublicKey } from '@solana/web3.js';

export const GREEN_LABEL_DEVNET_RPC_ENDPOINT = 'https://api.devnet.solana.com';
export const GREEN_LABEL_PROGRAM_ID = new PublicKey('HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY');
export const GREEN_LABEL_CONFIG_PDA = new PublicKey('7hNAeoqZxqvp38giY9gZwfR5ai3ttYTrse63QNYrRBWS');
export const GREEN_LABEL_USDC_DECIMALS = 6;

const GREEN_LABEL_CONFIG_DISCRIMINATOR = [18, 20, 44, 233, 16, 255, 27, 58] as const;
const GREEN_LABEL_PROJECT_DISCRIMINATOR = [235, 175, 107, 126, 165, 114, 174, 130] as const;
const GREEN_LABEL_DISPUTE_DISCRIMINATOR = [248, 233, 171, 233, 77, 59, 92, 89] as const;
const GREEN_LABEL_CONFIG_ACCOUNT_SIZE = 406;
const GREEN_LABEL_PROJECT_ACCOUNT_SIZE = 622;
const GREEN_LABEL_DISPUTE_ACCOUNT_SIZE = 388;
const MAINNET_MIN_BASE_BOND_USDC_RAW = 299_000_000n;
const DEVNET_TEST_MIN_BASE_BOND_USDC_RAW = 1_000_000n;
const MAINNET_OBSERVATION_SECONDS = 2_592_000n;
const MAINNET_DISPUTE_SECONDS = 604_800n;
const MAINNET_RESPONSE_SECONDS = 259_200n;
const DEVNET_TEST_WINDOW_SECONDS = 30n;
const SOLANA_EXPLORER_DEVNET = 'https://explorer.solana.com';

export const GREEN_LABEL_E2E_TARGETS = [
  {
    key: 'refund',
    title: 'Refund Project #2',
    projectId: 2,
    projectAccount: '8TzeHWGWw2rWtQQgggBzim6Wz6PU81VZgb8nmt8uXRJ',
    disputeAccount: '3PtfxqksFXQUMKWm9eGgcG76uXgAMfDDE3Q5DqWT6Rea',
    expectedProjectStatus: 'Refunded',
    expectedDisputeStatus: 'ResolvedRefund',
  },
  {
    key: 'slash',
    title: 'Slash Project #3',
    projectId: 3,
    projectAccount: '87CQ4qqFnGbkuvm6udp8MNULdrNsWvLSdqXfU38narRp',
    disputeAccount: '5HVNT7f58MTf7gXWXqrenSdzbTj1mGLJQHYes1AU3Mw6',
    expectedProjectStatus: 'Slashed',
    expectedDisputeStatus: 'ResolvedSlash',
  },
] as const;

const GREEN_LABEL_STATUS_NAMES = [
  'PendingBondDeposit',
  'PendingObservation',
  'ActiveGreenLabel',
  'Disputed',
  'RefundQueued',
  'SlashQueued',
  'Refunded',
  'Slashed',
  'Cancelled',
] as const;

const BOND_TIER_NAMES = [
  'Base',
  'Bronze',
  'Silver',
  'Gold',
  'Platinum',
  'Custom',
] as const;

const RUG_REASON_CODE_NAMES = [
  'LiquidityRemoved',
  'DeveloperDump',
  'WebsiteOrCommunityAbandoned',
  'MintOrFreezeAuthorityAbuse',
  'TreasuryMisuse',
  'FalseDisclosure',
  'MaliciousContractUpgrade',
  'Other',
] as const;

const DISPUTE_STATUS_NAMES = [
  'Open',
  'EvidencePeriod',
  'ProjectResponsePeriod',
  'ReadyForDecision',
  'DecisionQueued',
  'ResolvedRefund',
  'ResolvedSlash',
  'Rejected',
  'Cancelled',
] as const;

const ACTION_TYPE_NAMES = [
  'Noop',
  'GreenLabelSlash',
  'GreenLabelRefund',
  'PayrollEmployeeImpeach',
  'PayrollPayout',
  'TreasuryParamChange',
  'EmergencyPause',
] as const;

export type GreenLabelStatusName = typeof GREEN_LABEL_STATUS_NAMES[number];
export type BondTierName = typeof BOND_TIER_NAMES[number];
export type RugReasonCodeName = typeof RUG_REASON_CODE_NAMES[number];
export type DisputeStatusName = typeof DISPUTE_STATUS_NAMES[number];
export type ActionTypeName = typeof ACTION_TYPE_NAMES[number];
export type GreenLabelE2EKey = typeof GREEN_LABEL_E2E_TARGETS[number]['key'];

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

export interface GreenLabelProjectV1 {
  account: string;
  projectId: bigint;
  projectOwner: string;
  projectNameHash: string;
  projectUrlHash: string;
  tokenMint: string;
  projectTreasuryWallet: string;
  baseBondAmount: bigint;
  extraBondAmount: bigint;
  totalBondAmount: bigint;
  bondVault: string;
  bondVaultAuthority: string;
  bondTier: BondTierName;
  status: GreenLabelStatusName;
  submittedAt: bigint;
  observationStartTs: bigint;
  observationEndTs: bigint;
  disputeCount: bigint;
  activeDispute: string;
  approvedAt: bigint;
  refundedAt: bigint;
  slashedAt: bigint;
  riskScoreSnapshot: number;
  terminalProposalId: bigint;
  terminalProposalDecision: string;
  terminalExecutionQueueItem: string;
  terminalPayloadHash: string;
  terminalActionType: ActionTypeName;
  bump: number;
}

export interface GreenLabelDisputeV1 {
  account: string;
  projectId: bigint;
  disputeId: bigint;
  project: string;
  disputer: string;
  reasonCode: RugReasonCodeName;
  evidenceHash: string;
  status: DisputeStatusName;
  openedAt: bigint;
  evidenceEndTs: bigint;
  responseEndTs: bigint;
  resolvedAt: bigint;
  proposalId: bigint;
  proposalDecision: string;
  executionQueueItem: string;
  payloadHash: string;
  actionType: ActionTypeName;
  bump: number;
}

export interface GreenLabelE2EResult {
  key: GreenLabelE2EKey;
  title: string;
  projectId: number;
  projectAccount: string;
  disputeAccount: string;
  expectedProjectStatus: GreenLabelStatusName;
  expectedDisputeStatus: DisputeStatusName;
  project: GreenLabelProjectV1;
  dispute: GreenLabelDisputeV1;
}

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

export async function fetchGreenLabelE2EResults(connection: Connection): Promise<GreenLabelE2EResult[]> {
  return Promise.all(
    GREEN_LABEL_E2E_TARGETS.map(async (target) => {
      const projectAddress = new PublicKey(target.projectAccount);
      const disputeAddress = new PublicKey(target.disputeAccount);
      const [projectAccountInfo, disputeAccountInfo] = await Promise.all([
        connection.getAccountInfo(projectAddress, 'confirmed'),
        connection.getAccountInfo(disputeAddress, 'confirmed'),
      ]);

      if (!projectAccountInfo) {
        throw new Error(`GreenLabelProjectV1 account not found: ${target.projectAccount}`);
      }

      if (!disputeAccountInfo) {
        throw new Error(`GreenLabelDisputeV1 account not found: ${target.disputeAccount}`);
      }

      if (!projectAccountInfo.owner.equals(GREEN_LABEL_PROGRAM_ID)) {
        throw new Error(`GreenLabelProjectV1 owner mismatch: ${projectAccountInfo.owner.toBase58()}`);
      }

      if (!disputeAccountInfo.owner.equals(GREEN_LABEL_PROGRAM_ID)) {
        throw new Error(`GreenLabelDisputeV1 owner mismatch: ${disputeAccountInfo.owner.toBase58()}`);
      }

      return {
        ...target,
        project: decodeGreenLabelProject(projectAccountInfo.data, target.projectAccount),
        dispute: decodeGreenLabelDispute(disputeAccountInfo.data, target.disputeAccount),
      };
    }),
  );
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

export function decodeGreenLabelProject(data: Uint8Array, account: string): GreenLabelProjectV1 {
  assertAccountLayout(data, GREEN_LABEL_PROJECT_ACCOUNT_SIZE, GREEN_LABEL_PROJECT_DISCRIMINATOR, 'GreenLabelProjectV1');

  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  let offset = 8;

  const projectId = view.getBigUint64(offset, true);
  offset += 8;
  const projectOwner = readPubkey(data, offset);
  offset += 32;
  const projectNameHash = readHashHex(data, offset);
  offset += 32;
  const projectUrlHash = readHashHex(data, offset);
  offset += 32;
  const tokenMint = readPubkey(data, offset);
  offset += 32;
  const projectTreasuryWallet = readPubkey(data, offset);
  offset += 32;
  const baseBondAmount = view.getBigUint64(offset, true);
  offset += 8;
  const extraBondAmount = view.getBigUint64(offset, true);
  offset += 8;
  const totalBondAmount = view.getBigUint64(offset, true);
  offset += 8;
  const bondVault = readPubkey(data, offset);
  offset += 32;
  const bondVaultAuthority = readPubkey(data, offset);
  offset += 32;
  const bondTier = readEnum(data[offset], BOND_TIER_NAMES, 'BondTier');
  offset += 1;
  const status = readEnum(data[offset], GREEN_LABEL_STATUS_NAMES, 'GreenLabelStatus');
  offset += 1;
  const submittedAt = view.getBigInt64(offset, true);
  offset += 8;
  const observationStartTs = view.getBigInt64(offset, true);
  offset += 8;
  const observationEndTs = view.getBigInt64(offset, true);
  offset += 8;
  const disputeCount = view.getBigUint64(offset, true);
  offset += 8;
  const activeDispute = readPubkey(data, offset);
  offset += 32;
  const approvedAt = view.getBigInt64(offset, true);
  offset += 8;
  const refundedAt = view.getBigInt64(offset, true);
  offset += 8;
  const slashedAt = view.getBigInt64(offset, true);
  offset += 8;
  const riskScoreSnapshot = view.getUint16(offset, true);
  offset += 2;
  const terminalProposalId = view.getBigUint64(offset, true);
  offset += 8;
  const terminalProposalDecision = readPubkey(data, offset);
  offset += 32;
  const terminalExecutionQueueItem = readPubkey(data, offset);
  offset += 32;
  const terminalPayloadHash = readHashHex(data, offset);
  offset += 32;
  const terminalActionType = readEnum(data[offset], ACTION_TYPE_NAMES, 'ActionType');
  offset += 1;
  const bump = data[offset];

  return {
    account,
    projectId,
    projectOwner,
    projectNameHash,
    projectUrlHash,
    tokenMint,
    projectTreasuryWallet,
    baseBondAmount,
    extraBondAmount,
    totalBondAmount,
    bondVault,
    bondVaultAuthority,
    bondTier,
    status,
    submittedAt,
    observationStartTs,
    observationEndTs,
    disputeCount,
    activeDispute,
    approvedAt,
    refundedAt,
    slashedAt,
    riskScoreSnapshot,
    terminalProposalId,
    terminalProposalDecision,
    terminalExecutionQueueItem,
    terminalPayloadHash,
    terminalActionType,
    bump,
  };
}

export function decodeGreenLabelDispute(data: Uint8Array, account: string): GreenLabelDisputeV1 {
  assertAccountLayout(data, GREEN_LABEL_DISPUTE_ACCOUNT_SIZE, GREEN_LABEL_DISPUTE_DISCRIMINATOR, 'GreenLabelDisputeV1');

  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  let offset = 8;

  const projectId = view.getBigUint64(offset, true);
  offset += 8;
  const disputeId = view.getBigUint64(offset, true);
  offset += 8;
  const project = readPubkey(data, offset);
  offset += 32;
  const disputer = readPubkey(data, offset);
  offset += 32;
  const reasonCode = readEnum(data[offset], RUG_REASON_CODE_NAMES, 'RugReasonCode');
  offset += 1;
  const evidenceHash = readHashHex(data, offset);
  offset += 32;
  const status = readEnum(data[offset], DISPUTE_STATUS_NAMES, 'DisputeStatus');
  offset += 1;
  const openedAt = view.getBigInt64(offset, true);
  offset += 8;
  const evidenceEndTs = view.getBigInt64(offset, true);
  offset += 8;
  const responseEndTs = view.getBigInt64(offset, true);
  offset += 8;
  const resolvedAt = view.getBigInt64(offset, true);
  offset += 8;
  const proposalId = view.getBigUint64(offset, true);
  offset += 8;
  const proposalDecision = readPubkey(data, offset);
  offset += 32;
  const executionQueueItem = readPubkey(data, offset);
  offset += 32;
  const payloadHash = readHashHex(data, offset);
  offset += 32;
  const actionType = readEnum(data[offset], ACTION_TYPE_NAMES, 'ActionType');
  offset += 1;
  const bump = data[offset];

  return {
    account,
    projectId,
    disputeId,
    project,
    disputer,
    reasonCode,
    evidenceHash,
    status,
    openedAt,
    evidenceEndTs,
    responseEndTs,
    resolvedAt,
    proposalId,
    proposalDecision,
    executionQueueItem,
    payloadHash,
    actionType,
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

export function formatUnixTimestamp(timestamp: bigint): string {
  if (timestamp === 0n) {
    return '未设置';
  }

  const timestampMs = Number(timestamp) * 1000;

  if (!Number.isFinite(timestampMs)) {
    return '时间戳超出前端可显示范围';
  }

  return new Date(timestampMs).toLocaleString('zh-CN');
}

export function getGreenLabelExplorerAddressUrl(address: string): string {
  return `${SOLANA_EXPLORER_DEVNET}/address/${address}?cluster=devnet`;
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

function assertAccountLayout(
  data: Uint8Array,
  expectedSize: number,
  discriminator: readonly number[],
  accountName: string,
) {
  if (data.length < expectedSize) {
    throw new Error(`${accountName} account too small: ${data.length} bytes`);
  }

  for (let index = 0; index < discriminator.length; index += 1) {
    if (data[index] !== discriminator[index]) {
      throw new Error(`${accountName} discriminator mismatch`);
    }
  }
}

function readHashHex(data: Uint8Array, offset: number): string {
  return Array.from(data.slice(offset, offset + 32))
    .map((byte) => byte.toString(16).padStart(2, '0'))
    .join('');
}

function readEnum<T extends readonly string[]>(value: number, names: T, enumName: string): T[number] {
  const name = names[value];

  if (!name) {
    throw new Error(`${enumName} enum index out of range: ${value}`);
  }

  return name;
}
