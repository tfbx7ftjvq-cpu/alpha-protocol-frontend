import { Connection, PublicKey } from '@solana/web3.js';

export const SECURITY_LAYER_DEVNET_RPC_ENDPOINT = 'https://api.devnet.solana.com';
export const SECURITY_LAYER_PROGRAM_ID = new PublicKey('HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY');
export const GOVERNANCE_CONFIG_PDA = new PublicKey('5np4fcpSP8eHVLD6dsgLHf7H11VLaGcYgdadxidt9ro3');

const GOVERNANCE_CONFIG_SEED = 'governance_config_v1';
const PROPOSAL_DECISION_SEED = 'proposal_decision_v1';
const EXECUTION_QUEUE_ITEM_SEED = 'execution_queue_item_v1';
const SOLANA_EXPLORER_DEVNET = 'https://explorer.solana.com';

const GOVERNANCE_CONFIG_V1_DISCRIMINATOR = [203, 20, 137, 216, 207, 141, 109, 16] as const;
const PROPOSAL_DECISION_V1_DISCRIMINATOR = [13, 13, 103, 236, 159, 187, 141, 237] as const;
const EXECUTION_QUEUE_ITEM_V1_DISCRIMINATOR = [27, 25, 66, 11, 60, 169, 202, 231] as const;

const GOVERNANCE_CONFIG_V1_MIN_SIZE = 8 + 32 + 8 + 8 + 32 + 1 + 1;
const PROPOSAL_DECISION_V1_MIN_SIZE = 8 + 8 + 1 + 32 + 1 + 8 + 8 + 8 + 8 + 8 + 1;
const EXECUTION_QUEUE_ITEM_V1_MIN_SIZE = 8 + 8 + 32 + 1 + 32 + 32 + 1 + 8 + 8 + 8 + 1 + 32 + 1;

const PROPOSAL_TYPE_NAMES = [
  'GreenLabelSlash',
  'GreenLabelRefund',
  'PayrollEmployeeImpeach',
  'PayrollPayout',
  'TreasuryParamChange',
  'EmergencyPause',
] as const;

const PROPOSAL_DECISION_NAMES = [
  'Pending',
  'Approved',
  'Rejected',
  'Partial',
] as const;

const EXECUTION_STATUS_NAMES = [
  'Queued',
  'Executed',
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

export type ProposalTypeName = typeof PROPOSAL_TYPE_NAMES[number];
export type ProposalDecisionName = typeof PROPOSAL_DECISION_NAMES[number];
export type ExecutionStatusName = typeof EXECUTION_STATUS_NAMES[number];
export type ActionTypeName = typeof ACTION_TYPE_NAMES[number];

export interface GovernanceConfigV1 {
  authority: string;
  minExecutionDelaySeconds: bigint;
  proposalCount: bigint;
  emergencyGuardian: string;
  isPaused: boolean;
  bump: number;
}

export interface ProposalDecisionV1 {
  account: string;
  proposalId: bigint;
  proposalType: ProposalTypeName;
  proposer: string;
  decision: ProposalDecisionName;
  yesWeight: bigint;
  noWeight: bigint;
  startTs: bigint;
  endTs: bigint;
  finalizedTs: bigint;
  bump: number;
}

export interface ExecutionQueueItemV1 {
  account: string;
  proposalId: bigint;
  proposer: string;
  actionType: ActionTypeName;
  targetProgram: string;
  targetAccount: string;
  decision: ProposalDecisionName;
  createdAt: bigint;
  executeAfter: bigint;
  executedAt: bigint;
  status: ExecutionStatusName;
  payloadHash: string;
  bump: number;
}

export interface SecurityGovernanceItemTarget {
  proposalId: number;
  expectedPathLabel: string;
  description: string;
}

export interface SecurityGovernanceItem {
  target: SecurityGovernanceItemTarget;
  proposalDecisionPda: string;
  executionQueueItemPda: string;
  proposalDecision: ProposalDecisionV1 | null;
  executionQueueItem: ExecutionQueueItemV1 | null;
  proposalError: string | null;
  queueError: string | null;
}

export const SECURITY_GOVERNANCE_ITEM_TARGETS: SecurityGovernanceItemTarget[] = [
  {
    proposalId: 1,
    expectedPathLabel: 'TreasuryParamChange happy path',
    description: 'queue + execute verified',
  },
  {
    proposalId: 3,
    expectedPathLabel: 'Cancel path verified',
    description: 'cancelled queue item cannot execute',
  },
  {
    proposalId: 4,
    expectedPathLabel: 'Green Label refund path verified',
    description: 'refund execution path linked to Security Layer',
  },
  {
    proposalId: 5,
    expectedPathLabel: 'Green Label slash path verified',
    description: 'slash execution path linked to Security Layer',
  },
];

const PROPOSAL_TYPE_LABELS: Record<ProposalTypeName, string> = {
  GreenLabelSlash: 'Green Label slash / 绿标罚没',
  GreenLabelRefund: 'Green Label refund / 绿标退款',
  PayrollEmployeeImpeach: 'Payroll employee impeach / 成员移除',
  PayrollPayout: 'Payroll payout / 薪酬支付',
  TreasuryParamChange: 'Treasury parameter change / 国库参数变更',
  EmergencyPause: 'Emergency pause / 紧急暂停',
};

const PROPOSAL_DECISION_LABELS: Record<ProposalDecisionName, string> = {
  Pending: 'Pending / 待裁决',
  Approved: 'Approved / 已通过',
  Rejected: 'Rejected / 已拒绝',
  Partial: 'Partial / 部分通过',
};

const EXECUTION_STATUS_LABELS: Record<ExecutionStatusName, string> = {
  Queued: 'Queued / 已排队',
  Executed: 'Executed / 已执行',
  Cancelled: 'Cancelled / 已取消',
};

const ACTION_TYPE_LABELS: Record<ActionTypeName, string> = {
  Noop: 'Noop / 空操作',
  GreenLabelSlash: 'Green Label slash / 绿标罚没',
  GreenLabelRefund: 'Green Label refund / 绿标退款',
  PayrollEmployeeImpeach: 'Payroll employee impeach / 成员移除',
  PayrollPayout: 'Payroll payout / 薪酬支付',
  TreasuryParamChange: 'Treasury parameter change / 国库参数变更',
  EmergencyPause: 'Emergency pause / 紧急暂停',
};

export async function fetchGovernanceConfigV1(connection: Connection): Promise<GovernanceConfigV1> {
  const accountInfo = await connection.getAccountInfo(GOVERNANCE_CONFIG_PDA, 'confirmed');

  if (!accountInfo) {
    throw new Error(`GovernanceConfigV1 account not found: ${GOVERNANCE_CONFIG_PDA.toBase58()}`);
  }

  if (!accountInfo.owner.equals(SECURITY_LAYER_PROGRAM_ID)) {
    throw new Error(`GovernanceConfigV1 owner mismatch: ${accountInfo.owner.toBase58()}`);
  }

  return decodeGovernanceConfigV1(accountInfo.data);
}

export async function fetchSecurityGovernanceItems(connection: Connection): Promise<SecurityGovernanceItem[]> {
  return Promise.all(
    SECURITY_GOVERNANCE_ITEM_TARGETS.map(async (target) => {
      const proposalDecisionPda = deriveProposalDecisionPda(target.proposalId);
      const executionQueueItemPda = deriveExecutionQueueItemPda(target.proposalId);
      const [proposalResult, queueResult] = await Promise.all([
        fetchOptionalProposalDecision(connection, proposalDecisionPda),
        fetchOptionalExecutionQueueItem(connection, executionQueueItemPda),
      ]);

      return {
        target,
        proposalDecisionPda: proposalDecisionPda.toBase58(),
        executionQueueItemPda: executionQueueItemPda.toBase58(),
        proposalDecision: proposalResult.account,
        executionQueueItem: queueResult.account,
        proposalError: proposalResult.error,
        queueError: queueResult.error,
      };
    }),
  );
}

export function decodeGovernanceConfigV1(data: Uint8Array): GovernanceConfigV1 {
  assertAccountLayout(data, GOVERNANCE_CONFIG_V1_MIN_SIZE, GOVERNANCE_CONFIG_V1_DISCRIMINATOR, 'GovernanceConfigV1');

  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  let offset = 8;

  const authority = readPubkey(data, offset);
  offset += 32;
  const minExecutionDelaySeconds = view.getBigInt64(offset, true);
  offset += 8;
  const proposalCount = view.getBigUint64(offset, true);
  offset += 8;
  const emergencyGuardian = readPubkey(data, offset);
  offset += 32;
  const isPaused = data[offset] === 1;
  offset += 1;
  const bump = data[offset];

  return {
    authority,
    minExecutionDelaySeconds,
    proposalCount,
    emergencyGuardian,
    isPaused,
    bump,
  };
}

export function decodeProposalDecisionV1(data: Uint8Array, account: string): ProposalDecisionV1 {
  assertAccountLayout(data, PROPOSAL_DECISION_V1_MIN_SIZE, PROPOSAL_DECISION_V1_DISCRIMINATOR, 'ProposalDecisionV1');

  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  let offset = 8;

  const proposalId = view.getBigUint64(offset, true);
  offset += 8;
  const proposalType = readEnum(data[offset], PROPOSAL_TYPE_NAMES, 'ProposalType');
  offset += 1;
  const proposer = readPubkey(data, offset);
  offset += 32;
  const decision = readEnum(data[offset], PROPOSAL_DECISION_NAMES, 'ProposalDecision');
  offset += 1;
  const yesWeight = view.getBigUint64(offset, true);
  offset += 8;
  const noWeight = view.getBigUint64(offset, true);
  offset += 8;
  const startTs = view.getBigInt64(offset, true);
  offset += 8;
  const endTs = view.getBigInt64(offset, true);
  offset += 8;
  const finalizedTs = view.getBigInt64(offset, true);
  offset += 8;
  const bump = data[offset];

  return {
    account,
    proposalId,
    proposalType,
    proposer,
    decision,
    yesWeight,
    noWeight,
    startTs,
    endTs,
    finalizedTs,
    bump,
  };
}

export function decodeExecutionQueueItemV1(data: Uint8Array, account: string): ExecutionQueueItemV1 {
  assertAccountLayout(data, EXECUTION_QUEUE_ITEM_V1_MIN_SIZE, EXECUTION_QUEUE_ITEM_V1_DISCRIMINATOR, 'ExecutionQueueItemV1');

  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  let offset = 8;

  const proposalId = view.getBigUint64(offset, true);
  offset += 8;
  const proposer = readPubkey(data, offset);
  offset += 32;
  const actionType = readEnum(data[offset], ACTION_TYPE_NAMES, 'ActionType');
  offset += 1;
  const targetProgram = readPubkey(data, offset);
  offset += 32;
  const targetAccount = readPubkey(data, offset);
  offset += 32;
  const decision = readEnum(data[offset], PROPOSAL_DECISION_NAMES, 'ProposalDecision');
  offset += 1;
  const createdAt = view.getBigInt64(offset, true);
  offset += 8;
  const executeAfter = view.getBigInt64(offset, true);
  offset += 8;
  const executedAt = view.getBigInt64(offset, true);
  offset += 8;
  const status = readEnum(data[offset], EXECUTION_STATUS_NAMES, 'ExecutionStatus');
  offset += 1;
  const payloadHash = readHashHex(data, offset);
  offset += 32;
  const bump = data[offset];

  return {
    account,
    proposalId,
    proposer,
    actionType,
    targetProgram,
    targetAccount,
    decision,
    createdAt,
    executeAfter,
    executedAt,
    status,
    payloadHash,
    bump,
  };
}

export function deriveGovernanceConfigPda(): PublicKey {
  return deriveSeedPda(GOVERNANCE_CONFIG_SEED);
}

export function deriveProposalDecisionPda(proposalId: number | bigint): PublicKey {
  return deriveSeedAndProposalIdPda(PROPOSAL_DECISION_SEED, proposalId);
}

export function deriveExecutionQueueItemPda(proposalId: number | bigint): PublicKey {
  return deriveSeedAndProposalIdPda(EXECUTION_QUEUE_ITEM_SEED, proposalId);
}

export function getProposalTypeLabel(value: ProposalTypeName): string {
  return PROPOSAL_TYPE_LABELS[value];
}

export function getProposalDecisionLabel(value: ProposalDecisionName): string {
  return PROPOSAL_DECISION_LABELS[value];
}

export function getExecutionStatusLabel(value: ExecutionStatusName): string {
  return EXECUTION_STATUS_LABELS[value];
}

export function getActionTypeLabel(value: ActionTypeName): string {
  return ACTION_TYPE_LABELS[value];
}

export function formatSecurityLayerDuration(seconds: bigint): string {
  if (seconds === 0n) return '0 seconds';

  const absSeconds = seconds < 0n ? -seconds : seconds;
  const sign = seconds < 0n ? '-' : '';

  if (absSeconds % 86_400n === 0n) {
    return `${sign}${(absSeconds / 86_400n).toString()} days`;
  }

  if (absSeconds % 3_600n === 0n) {
    return `${sign}${(absSeconds / 3_600n).toString()} hours`;
  }

  if (absSeconds % 60n === 0n) {
    return `${sign}${(absSeconds / 60n).toString()} minutes`;
  }

  return `${sign}${absSeconds.toString()} seconds`;
}

export function formatSecurityLayerTimestamp(timestamp: bigint): string {
  if (timestamp === 0n) {
    return 'unset';
  }

  const timestampMs = Number(timestamp) * 1000;

  if (!Number.isFinite(timestampMs)) {
    return 'out of display range';
  }

  return new Date(timestampMs).toLocaleString('zh-CN');
}

export function getSecurityExplorerAddressUrl(address: string): string {
  return `${SOLANA_EXPLORER_DEVNET}/address/${address}?cluster=devnet`;
}

async function fetchOptionalProposalDecision(
  connection: Connection,
  address: PublicKey,
): Promise<{ account: ProposalDecisionV1 | null; error: string | null }> {
  try {
    const accountInfo = await connection.getAccountInfo(address, 'confirmed');

    if (!accountInfo) {
      return { account: null, error: 'not found' };
    }

    if (!accountInfo.owner.equals(SECURITY_LAYER_PROGRAM_ID)) {
      return { account: null, error: `owner mismatch: ${accountInfo.owner.toBase58()}` };
    }

    return {
      account: decodeProposalDecisionV1(accountInfo.data, address.toBase58()),
      error: null,
    };
  } catch (error) {
    return {
      account: null,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

async function fetchOptionalExecutionQueueItem(
  connection: Connection,
  address: PublicKey,
): Promise<{ account: ExecutionQueueItemV1 | null; error: string | null }> {
  try {
    const accountInfo = await connection.getAccountInfo(address, 'confirmed');

    if (!accountInfo) {
      return { account: null, error: 'not found' };
    }

    if (!accountInfo.owner.equals(SECURITY_LAYER_PROGRAM_ID)) {
      return { account: null, error: `owner mismatch: ${accountInfo.owner.toBase58()}` };
    }

    return {
      account: decodeExecutionQueueItemV1(accountInfo.data, address.toBase58()),
      error: null,
    };
  } catch (error) {
    return {
      account: null,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

function deriveSeedPda(seed: string): PublicKey {
  const [pda] = PublicKey.findProgramAddressSync(
    [new TextEncoder().encode(seed)],
    SECURITY_LAYER_PROGRAM_ID,
  );

  return pda;
}

function deriveSeedAndProposalIdPda(seed: string, proposalId: number | bigint): PublicKey {
  const proposalIdBytes = new Uint8Array(8);
  const proposalIdView = new DataView(proposalIdBytes.buffer);
  proposalIdView.setBigUint64(0, BigInt(proposalId), true);

  const [pda] = PublicKey.findProgramAddressSync(
    [new TextEncoder().encode(seed), proposalIdBytes],
    SECURITY_LAYER_PROGRAM_ID,
  );

  return pda;
}

function readPubkey(data: Uint8Array, offset: number): string {
  return new PublicKey(data.slice(offset, offset + 32)).toBase58();
}

function readHashHex(data: Uint8Array, offset: number): string {
  return Array.from(data.slice(offset, offset + 32))
    .map((byte) => byte.toString(16).padStart(2, '0'))
    .join('');
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

function readEnum<T extends readonly string[]>(value: number, names: T, enumName: string): T[number] {
  const name = names[value];

  if (!name) {
    throw new Error(`${enumName} enum index out of range: ${value}`);
  }

  return name;
}
