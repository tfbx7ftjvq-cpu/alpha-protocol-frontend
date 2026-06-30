import * as anchor from "@coral-xyz/anchor";
import {
  createAssociatedTokenAccountInstruction,
  getAccount,
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  PublicKey,
  SYSVAR_RENT_PUBKEY,
  SystemProgram,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
import * as crypto from "crypto";
import * as fs from "fs";
import * as path from "path";

export const PROGRAM_ID = new PublicKey("HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY");
export const DEVNET_USDC_MINT = new PublicKey("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU");
export const DEFAULT_PUBLIC_KEY = new PublicKey("11111111111111111111111111111111");
export const GREEN_LABEL_USDC_DECIMALS = 6;
export const IDL_PATH = path.resolve(
  __dirname,
  "../../../target/idl/my_first_solana_program.json",
);
export const U64_MAX = (1n << 64n) - 1n;
export const I64_MIN = -(1n << 63n);
export const I64_MAX = (1n << 63n) - 1n;

export const GREEN_LABEL_STATUSES = [
  "PendingBondDeposit",
  "PendingObservation",
  "ActiveGreenLabel",
  "Disputed",
  "RefundQueued",
  "SlashQueued",
  "Refunded",
  "Slashed",
  "Cancelled",
] as const;

export const BOND_TIERS = ["Base", "Bronze", "Silver", "Gold", "Platinum", "Custom"] as const;

export const RUG_REASON_CODES = [
  "LiquidityRemoved",
  "DeveloperDump",
  "WebsiteOrCommunityAbandoned",
  "MintOrFreezeAuthorityAbuse",
  "TreasuryMisuse",
  "FalseDisclosure",
  "MaliciousContractUpgrade",
  "Other",
] as const;

export const DISPUTE_STATUSES = [
  "Open",
  "EvidencePeriod",
  "ProjectResponsePeriod",
  "ReadyForDecision",
  "DecisionQueued",
  "ResolvedRefund",
  "ResolvedSlash",
  "Rejected",
  "Cancelled",
] as const;

export const PROPOSAL_TYPES = [
  "GreenLabelSlash",
  "GreenLabelRefund",
  "PayrollEmployeeImpeach",
  "PayrollPayout",
  "TreasuryParamChange",
  "EmergencyPause",
] as const;

export const PROPOSAL_DECISIONS = ["Pending", "Approved", "Rejected", "Partial"] as const;

export const ACTION_TYPES = [
  "Noop",
  "GreenLabelSlash",
  "GreenLabelRefund",
  "PayrollEmployeeImpeach",
  "PayrollPayout",
  "TreasuryParamChange",
  "EmergencyPause",
] as const;

const SEEDS = {
  treasuryUsdcStateV2: "treasury_usdc_state_v2",
  reliefUsdcVault: "relief_usdc_vault",
  buybackUsdcVault: "buyback_usdc_vault",
  buildersUsdcVault: "builders_usdc_vault",
  stakingUsdcVault: "staking_usdc_vault",
  vaultAuthorityV2: "vault_authority_v2",
  greenLabelConfig: "green_label_config_v1",
  greenLabelProject: "green_label_project_v1",
  greenLabelDispute: "green_label_dispute_v1",
  greenBondVault: "green_bond_vault_v1",
  greenBondVaultAuthority: "green_bond_vault_authority_v1",
  governanceConfig: "governance_config_v1",
  proposalDecision: "proposal_decision_v1",
  executionQueueItem: "execution_queue_item_v1",
} as const;

export type RuntimeIdl = {
  address?: string;
  instructions: Array<{
    name: string;
    discriminator?: number[];
  }>;
};

export type EnumValue = {
  name: string;
  index: number;
};

export type GreenLabelConfigSummary = {
  authority: PublicKey;
  usdcMint: PublicKey;
  minBaseBondUsdc: bigint;
  baseRefundBps: number;
  baseTreasuryBps: number;
  observationPeriodSeconds: bigint;
  disputeWindowSeconds: bigint;
  responseWindowSeconds: bigint;
  projectCount: bigint;
  treasuryUsdcStateV2: PublicKey;
  baseBondTreasuryVault: PublicKey;
  reliefOrRiskVault: PublicKey;
  vaultAuthorityV2: PublicKey;
  securityGovernanceConfig: PublicKey;
  isPaused: boolean;
  bump: number;
};

export type GreenLabelProjectSummary = {
  projectId: bigint;
  projectOwner: PublicKey;
  projectNameHash: Buffer;
  projectUrlHash: Buffer;
  tokenMint: PublicKey;
  projectTreasuryWallet: PublicKey;
  baseBondAmount: bigint;
  extraBondAmount: bigint;
  totalBondAmount: bigint;
  bondVault: PublicKey;
  bondVaultAuthority: PublicKey;
  bondTier: string;
  status: string;
  submittedAt: bigint;
  observationStartTs: bigint;
  observationEndTs: bigint;
  disputeCount: bigint;
  activeDispute: PublicKey;
  approvedAt: bigint;
  refundedAt: bigint;
  slashedAt: bigint;
  riskScoreSnapshot: number;
  terminalProposalId: bigint;
  terminalProposalDecision: PublicKey;
  terminalExecutionQueueItem: PublicKey;
  terminalPayloadHash: Buffer;
  terminalActionType: string;
  bump: number;
};

export type GreenLabelDisputeSummary = {
  projectId: bigint;
  disputeId: bigint;
  project: PublicKey;
  disputer: PublicKey;
  reasonCode: string;
  evidenceHash: Buffer;
  status: string;
  openedAt: bigint;
  evidenceEndTs: bigint;
  responseEndTs: bigint;
  resolvedAt: bigint;
  proposalId: bigint;
  proposalDecision: PublicKey;
  executionQueueItem: PublicKey;
  payloadHash: Buffer;
  actionType: string;
  bump: number;
};

export type GovernanceConfigSummary = {
  authority: PublicKey;
  minExecutionDelaySeconds: bigint;
  proposalCount: bigint;
  emergencyGuardian: PublicKey;
  isPaused: boolean;
  bump: number;
};

export type ExecutionQueueSummary = {
  proposalId: bigint;
  proposer: PublicKey;
  actionType: string;
  targetProgram: PublicKey;
  targetAccount: PublicKey;
  decision: string;
  createdAt: bigint;
  executeAfter: bigint;
  executedAt: bigint;
  status: string;
  payloadHash: Buffer;
  bump: number;
};

export function loadProvider(): anchor.AnchorProvider {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  return provider;
}

export function loadIdl(): RuntimeIdl {
  const idl = JSON.parse(fs.readFileSync(IDL_PATH, "utf8")) as RuntimeIdl;
  idl.address = PROGRAM_ID.toBase58();
  return idl;
}

export function loadProgram(provider: anchor.AnchorProvider = loadProvider()): anchor.Program {
  const idl = loadIdl();
  return new anchor.Program(idl as anchor.Idl, provider);
}

export function printDevnetRiskBanner(scriptName: string): void {
  console.log(`Green Label V1 Devnet script: ${scriptName}`);
  console.log("WARNING: This script is for Devnet only.");
  console.log("WARNING: It can send on-chain transactions and move Devnet USDC.");
  console.log("WARNING: It does not bypass response windows or Security Layer timelocks.");
  console.log("WARNING: Do not use this script on Mainnet.");
  console.log("DRY_RUN:", String(isDryRun()));
}

export function isDryRun(): boolean {
  return (process.env.DRY_RUN ?? "false").toLowerCase() === "true";
}

export function anchorDiscriminator(name: string): Buffer {
  return crypto.createHash("sha256").update(`global:${name}`).digest().subarray(0, 8);
}

export function getInstructionDiscriminator(idl: RuntimeIdl, name: string): Buffer {
  const instruction = idl.instructions.find((ix) => ix.name === name);
  if (instruction?.discriminator) {
    const discriminator = Buffer.from(instruction.discriminator);
    if (discriminator.length !== 8) {
      throw new Error(
        `IDL discriminator for ${name} must be 8 bytes. Received ${discriminator.length}.`,
      );
    }
    return discriminator;
  }

  return anchorDiscriminator(name);
}

export function instructionDiscriminator(name: string): Buffer {
  return getInstructionDiscriminator(loadIdl(), name);
}

export function sha256Bytes(text: string): Buffer {
  return crypto.createHash("sha256").update(text).digest();
}

export function u64Buffer(value: bigint): Buffer {
  assertU64(value, "u64 value");
  const buffer = Buffer.alloc(8);
  buffer.writeBigUInt64LE(value);
  return buffer;
}

export function i64Buffer(value: bigint): Buffer {
  if (value < I64_MIN || value > I64_MAX) {
    throw new Error("i64 value must fit in a signed 64-bit integer");
  }
  const buffer = Buffer.alloc(8);
  buffer.writeBigInt64LE(value);
  return buffer;
}

export function assertU64(value: bigint, label: string): void {
  if (value < 0n || value > U64_MAX) {
    throw new Error(`${label} must fit in an unsigned 64-bit integer`);
  }
}

export function readU64Env(name: string, defaultValue: bigint): bigint {
  const raw = process.env[name] ?? defaultValue.toString();
  if (!/^\d+$/.test(raw)) {
    throw new Error(`${name} must be a non-negative integer. Received: ${raw}`);
  }
  const value = BigInt(raw);
  assertU64(value, name);
  return value;
}

export function readPublicKeyEnv(name: string, defaultValue: PublicKey): PublicKey {
  const raw = process.env[name];
  if (!raw) {
    return defaultValue;
  }
  try {
    return new PublicKey(raw);
  } catch {
    throw new Error(`Invalid public key in ${name}: ${raw}`);
  }
}

export function readRequiredPublicKeyEnv(name: string): PublicKey {
  const raw = process.env[name];
  if (!raw) {
    throw new Error(`Missing required environment variable: ${name}`);
  }
  return readPublicKeyEnv(name, DEFAULT_PUBLIC_KEY);
}

export function readEnumEnv(
  envName: string,
  defaultValue: string,
  variants: readonly string[],
): EnumValue {
  const raw = process.env[envName] ?? defaultValue;
  const index = variants.findIndex((variant) => variant.toLowerCase() === raw.toLowerCase());
  if (index < 0) {
    throw new Error(`${envName} must be one of: ${variants.join(", ")}. Received: ${raw}`);
  }
  return { name: variants[index], index };
}

export function readHashEnv(name: string, defaultText: string): Buffer {
  const raw = process.env[name];
  if (!raw) {
    return sha256Bytes(defaultText);
  }
  if (/^[0-9a-fA-F]{64}$/.test(raw)) {
    return Buffer.from(raw, "hex");
  }
  return sha256Bytes(raw);
}

export function readUsdcAmountEnv(name: string, defaultValue: string): bigint {
  const raw = process.env[name] ?? defaultValue;
  if (!/^\d+(\.\d{1,6})?$/.test(raw)) {
    throw new Error(`${name} must be a USDC amount with at most 6 decimals. Received: ${raw}`);
  }
  const [whole, fractional = ""] = raw.split(".");
  const paddedFractional = fractional.padEnd(GREEN_LABEL_USDC_DECIMALS, "0");
  const value = BigInt(whole) * 1_000_000n + BigInt(paddedFractional);
  assertU64(value, name);
  return value;
}

export function formatUsdc(amount: bigint): string {
  const sign = amount < 0n ? "-" : "";
  const absoluteAmount = amount < 0n ? -amount : amount;
  const whole = absoluteAmount / 1_000_000n;
  const fractional = (absoluteAmount % 1_000_000n).toString().padStart(6, "0");
  return `${sign}${whole}.${fractional}`;
}

export function explorerLink(signature: string): string {
  if (signature.startsWith("DRY_RUN")) {
    return signature;
  }
  return `https://explorer.solana.com/tx/${signature}?cluster=devnet`;
}

export async function sendAndConfirmLabeled(
  provider: anchor.AnchorProvider,
  label: string,
  tx: Transaction,
): Promise<string> {
  if (isDryRun()) {
    const dryRunSignature = `DRY_RUN_${label.replace(/[^a-z0-9]+/gi, "_").toUpperCase()}`;
    console.log(`[dry-run] Would send transaction: ${label}`);
    console.log(`[dry-run] Instruction count: ${tx.instructions.length}`);
    console.log("Transaction signature:", dryRunSignature);
    return dryRunSignature;
  }

  const signature = await provider.sendAndConfirm(tx, []);
  console.log(`${label} signature:`, signature);
  console.log("Transaction signature:", signature);
  console.log(`${label} explorer:`, explorerLink(signature));
  return signature;
}

export function derivePda(seed: string): PublicKey {
  return PublicKey.findProgramAddressSync([Buffer.from(seed)], PROGRAM_ID)[0];
}

export function deriveGreenLabelConfig(): PublicKey {
  return derivePda(SEEDS.greenLabelConfig);
}

export function deriveGreenLabelProject(projectId: bigint): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(SEEDS.greenLabelProject), u64Buffer(projectId)],
    PROGRAM_ID,
  )[0];
}

export function deriveGreenLabelDispute(project: PublicKey, disputeId: bigint): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(SEEDS.greenLabelDispute), project.toBuffer(), u64Buffer(disputeId)],
    PROGRAM_ID,
  )[0];
}

export function deriveGreenBondVault(project: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(SEEDS.greenBondVault), project.toBuffer()],
    PROGRAM_ID,
  )[0];
}

export function deriveGreenBondVaultAuthority(project: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(SEEDS.greenBondVaultAuthority), project.toBuffer()],
    PROGRAM_ID,
  )[0];
}

export function deriveGovernanceConfig(): PublicKey {
  return derivePda(SEEDS.governanceConfig);
}

export function deriveProposalDecision(proposalId: bigint): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(SEEDS.proposalDecision), u64Buffer(proposalId)],
    PROGRAM_ID,
  )[0];
}

export function deriveExecutionQueueItem(proposalId: bigint): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(SEEDS.executionQueueItem), u64Buffer(proposalId)],
    PROGRAM_ID,
  )[0];
}

export function deriveTreasuryPdas(): {
  treasuryUsdcStateV2: PublicKey;
  reliefUsdcVault: PublicKey;
  buybackUsdcVault: PublicKey;
  buildersUsdcVault: PublicKey;
  stakingUsdcVault: PublicKey;
  vaultAuthorityV2: PublicKey;
} {
  return {
    treasuryUsdcStateV2: derivePda(SEEDS.treasuryUsdcStateV2),
    reliefUsdcVault: derivePda(SEEDS.reliefUsdcVault),
    buybackUsdcVault: derivePda(SEEDS.buybackUsdcVault),
    buildersUsdcVault: derivePda(SEEDS.buildersUsdcVault),
    stakingUsdcVault: derivePda(SEEDS.stakingUsdcVault),
    vaultAuthorityV2: derivePda(SEEDS.vaultAuthorityV2),
  };
}

export async function requireAccountExists(
  provider: anchor.AnchorProvider,
  account: PublicKey,
  label: string,
): Promise<void> {
  const info = await provider.connection.getAccountInfo(account);
  if (!info) {
    throw new Error(`${label} does not exist: ${account.toBase58()}`);
  }
}

export async function getOrCreateAta(
  provider: anchor.AnchorProvider,
  mint: PublicKey,
  owner: PublicKey,
): Promise<PublicKey> {
  const ata = getAssociatedTokenAddressSync(mint, owner);
  const info = await provider.connection.getAccountInfo(ata);
  if (info) {
    return ata;
  }

  const ix = createAssociatedTokenAccountInstruction(
    provider.wallet.publicKey,
    ata,
    owner,
    mint,
  );
  await sendAndConfirmLabeled(provider, "create_project_owner_usdc_ata", new Transaction().add(ix));
  return ata;
}

export async function getTokenBalance(
  provider: anchor.AnchorProvider,
  tokenAccount: PublicKey,
): Promise<bigint> {
  const info = await provider.connection.getAccountInfo(tokenAccount);
  if (!info) {
    return 0n;
  }
  const account = await getAccount(provider.connection, tokenAccount);
  return account.amount;
}

export async function assertTokenBalanceAtLeast(
  provider: anchor.AnchorProvider,
  tokenAccount: PublicKey,
  minimumAmount: bigint,
  label: string,
): Promise<void> {
  const balance = await getTokenBalance(provider, tokenAccount);
  if (balance < minimumAmount) {
    throw new Error(
      `${label} has insufficient Devnet USDC. Required ${formatUsdc(
        minimumAmount,
      )}, found ${formatUsdc(balance)}. Fund the ATA first; this script does not mint USDC.`,
    );
  }
}

export async function sleepUntil(unixTimestamp: bigint, label: string): Promise<void> {
  const now = BigInt(Math.floor(Date.now() / 1000));
  if (now >= unixTimestamp) {
    return;
  }
  const waitMs = Number((unixTimestamp - now) * 1000n);
  console.log(`Waiting for ${label} until ${unixTimestamp.toString()} (${waitMs} ms)`);
  await new Promise((resolve) => setTimeout(resolve, waitMs));
}

export async function waitForTimelock(queue: ExecutionQueueSummary): Promise<void> {
  await sleepUntil(queue.executeAfter, "Security Layer timelock");
}

function readPubkey(data: Buffer, offset: number): PublicKey {
  return new PublicKey(data.subarray(offset, offset + 32));
}

function readHash(data: Buffer, offset: number): Buffer {
  return Buffer.from(data.subarray(offset, offset + 32));
}

function enumName(variants: readonly string[], index: number): string {
  return variants[index] ?? `Unknown(${index})`;
}

export function decodeGreenLabelConfig(data: Buffer): GreenLabelConfigSummary {
  let offset = 8;
  const authority = readPubkey(data, offset);
  offset += 32;
  const usdcMint = readPubkey(data, offset);
  offset += 32;
  const minBaseBondUsdc = data.readBigUInt64LE(offset);
  offset += 8;
  const baseRefundBps = data.readUInt16LE(offset);
  offset += 2;
  const baseTreasuryBps = data.readUInt16LE(offset);
  offset += 2;
  const observationPeriodSeconds = data.readBigInt64LE(offset);
  offset += 8;
  const disputeWindowSeconds = data.readBigInt64LE(offset);
  offset += 8;
  const responseWindowSeconds = data.readBigInt64LE(offset);
  offset += 8;
  const projectCount = data.readBigUInt64LE(offset);
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
  const isPaused = data.readUInt8(offset) !== 0;
  offset += 1;
  const bump = data.readUInt8(offset);

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

export function decodeGreenLabelProject(data: Buffer): GreenLabelProjectSummary {
  let offset = 8;
  const projectId = data.readBigUInt64LE(offset);
  offset += 8;
  const projectOwner = readPubkey(data, offset);
  offset += 32;
  const projectNameHash = readHash(data, offset);
  offset += 32;
  const projectUrlHash = readHash(data, offset);
  offset += 32;
  const tokenMint = readPubkey(data, offset);
  offset += 32;
  const projectTreasuryWallet = readPubkey(data, offset);
  offset += 32;
  const baseBondAmount = data.readBigUInt64LE(offset);
  offset += 8;
  const extraBondAmount = data.readBigUInt64LE(offset);
  offset += 8;
  const totalBondAmount = data.readBigUInt64LE(offset);
  offset += 8;
  const bondVault = readPubkey(data, offset);
  offset += 32;
  const bondVaultAuthority = readPubkey(data, offset);
  offset += 32;
  const bondTier = enumName(BOND_TIERS, data.readUInt8(offset));
  offset += 1;
  const status = enumName(GREEN_LABEL_STATUSES, data.readUInt8(offset));
  offset += 1;
  const submittedAt = data.readBigInt64LE(offset);
  offset += 8;
  const observationStartTs = data.readBigInt64LE(offset);
  offset += 8;
  const observationEndTs = data.readBigInt64LE(offset);
  offset += 8;
  const disputeCount = data.readBigUInt64LE(offset);
  offset += 8;
  const activeDispute = readPubkey(data, offset);
  offset += 32;
  const approvedAt = data.readBigInt64LE(offset);
  offset += 8;
  const refundedAt = data.readBigInt64LE(offset);
  offset += 8;
  const slashedAt = data.readBigInt64LE(offset);
  offset += 8;
  const riskScoreSnapshot = data.readUInt16LE(offset);
  offset += 2;
  const terminalProposalId = data.readBigUInt64LE(offset);
  offset += 8;
  const terminalProposalDecision = readPubkey(data, offset);
  offset += 32;
  const terminalExecutionQueueItem = readPubkey(data, offset);
  offset += 32;
  const terminalPayloadHash = readHash(data, offset);
  offset += 32;
  const terminalActionType = enumName(ACTION_TYPES, data.readUInt8(offset));
  offset += 1;
  const bump = data.readUInt8(offset);

  return {
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

export function decodeGreenLabelDispute(data: Buffer): GreenLabelDisputeSummary {
  let offset = 8;
  const projectId = data.readBigUInt64LE(offset);
  offset += 8;
  const disputeId = data.readBigUInt64LE(offset);
  offset += 8;
  const project = readPubkey(data, offset);
  offset += 32;
  const disputer = readPubkey(data, offset);
  offset += 32;
  const reasonCode = enumName(RUG_REASON_CODES, data.readUInt8(offset));
  offset += 1;
  const evidenceHash = readHash(data, offset);
  offset += 32;
  const status = enumName(DISPUTE_STATUSES, data.readUInt8(offset));
  offset += 1;
  const openedAt = data.readBigInt64LE(offset);
  offset += 8;
  const evidenceEndTs = data.readBigInt64LE(offset);
  offset += 8;
  const responseEndTs = data.readBigInt64LE(offset);
  offset += 8;
  const resolvedAt = data.readBigInt64LE(offset);
  offset += 8;
  const proposalId = data.readBigUInt64LE(offset);
  offset += 8;
  const proposalDecision = readPubkey(data, offset);
  offset += 32;
  const executionQueueItem = readPubkey(data, offset);
  offset += 32;
  const payloadHash = readHash(data, offset);
  offset += 32;
  const actionType = enumName(ACTION_TYPES, data.readUInt8(offset));
  offset += 1;
  const bump = data.readUInt8(offset);

  return {
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

export function decodeGovernanceConfig(data: Buffer): GovernanceConfigSummary {
  let offset = 8;
  const authority = readPubkey(data, offset);
  offset += 32;
  const minExecutionDelaySeconds = data.readBigInt64LE(offset);
  offset += 8;
  const proposalCount = data.readBigUInt64LE(offset);
  offset += 8;
  const emergencyGuardian = readPubkey(data, offset);
  offset += 32;
  const isPaused = data.readUInt8(offset) !== 0;
  offset += 1;
  const bump = data.readUInt8(offset);

  return {
    authority,
    minExecutionDelaySeconds,
    proposalCount,
    emergencyGuardian,
    isPaused,
    bump,
  };
}

export function decodeExecutionQueue(data: Buffer): ExecutionQueueSummary {
  let offset = 8;
  const proposalId = data.readBigUInt64LE(offset);
  offset += 8;
  const proposer = readPubkey(data, offset);
  offset += 32;
  const actionType = enumName(ACTION_TYPES, data.readUInt8(offset));
  offset += 1;
  const targetProgram = readPubkey(data, offset);
  offset += 32;
  const targetAccount = readPubkey(data, offset);
  offset += 32;
  const decision = enumName(PROPOSAL_DECISIONS, data.readUInt8(offset));
  offset += 1;
  const createdAt = data.readBigInt64LE(offset);
  offset += 8;
  const executeAfter = data.readBigInt64LE(offset);
  offset += 8;
  const executedAt = data.readBigInt64LE(offset);
  offset += 8;
  const status = enumName(["Queued", "Executed", "Cancelled"], data.readUInt8(offset));
  offset += 1;
  const payloadHash = readHash(data, offset);
  offset += 32;
  const bump = data.readUInt8(offset);

  return {
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

export async function fetchGreenLabelConfig(
  provider: anchor.AnchorProvider,
): Promise<GreenLabelConfigSummary | null> {
  const info = await provider.connection.getAccountInfo(deriveGreenLabelConfig());
  return info ? decodeGreenLabelConfig(info.data) : null;
}

export async function fetchGreenLabelProject(
  provider: anchor.AnchorProvider,
  projectId: bigint,
): Promise<GreenLabelProjectSummary | null> {
  const info = await provider.connection.getAccountInfo(deriveGreenLabelProject(projectId));
  return info ? decodeGreenLabelProject(info.data) : null;
}

export async function fetchGreenLabelDispute(
  provider: anchor.AnchorProvider,
  project: PublicKey,
  disputeId: bigint,
): Promise<GreenLabelDisputeSummary | null> {
  const info = await provider.connection.getAccountInfo(deriveGreenLabelDispute(project, disputeId));
  return info ? decodeGreenLabelDispute(info.data) : null;
}

export async function fetchGovernanceConfig(
  provider: anchor.AnchorProvider,
): Promise<GovernanceConfigSummary | null> {
  const info = await provider.connection.getAccountInfo(deriveGovernanceConfig());
  return info ? decodeGovernanceConfig(info.data) : null;
}

export async function fetchExecutionQueue(
  provider: anchor.AnchorProvider,
  proposalId: bigint,
): Promise<ExecutionQueueSummary | null> {
  const info = await provider.connection.getAccountInfo(deriveExecutionQueueItem(proposalId));
  return info ? decodeExecutionQueue(info.data) : null;
}

export function printGreenLabelConfig(config: GreenLabelConfigSummary): void {
  console.log("green_label_config:");
  console.log("  authority:", config.authority.toBase58());
  console.log("  usdc_mint:", config.usdcMint.toBase58());
  console.log("  min_base_bond_usdc:", formatUsdc(config.minBaseBondUsdc));
  console.log("  base_refund_bps:", config.baseRefundBps);
  console.log("  base_treasury_bps:", config.baseTreasuryBps);
  console.log("  observation_period_seconds:", config.observationPeriodSeconds.toString());
  console.log("  dispute_window_seconds:", config.disputeWindowSeconds.toString());
  console.log("  response_window_seconds:", config.responseWindowSeconds.toString());
  console.log("  project_count:", config.projectCount.toString());
  console.log("  treasury_usdc_state_v2:", config.treasuryUsdcStateV2.toBase58());
  console.log("  base_bond_treasury_vault:", config.baseBondTreasuryVault.toBase58());
  console.log("  relief_or_risk_vault:", config.reliefOrRiskVault.toBase58());
  console.log("  vault_authority_v2:", config.vaultAuthorityV2.toBase58());
  console.log("  security_governance_config:", config.securityGovernanceConfig.toBase58());
  console.log("  is_paused:", config.isPaused);
}

export function printGreenLabelProject(project: GreenLabelProjectSummary): void {
  console.log("green_label_project:");
  console.log("  project_id:", project.projectId.toString());
  console.log("  project_owner:", project.projectOwner.toBase58());
  console.log("  token_mint:", project.tokenMint.toBase58());
  console.log("  project_treasury_wallet:", project.projectTreasuryWallet.toBase58());
  console.log("  base_bond_amount:", formatUsdc(project.baseBondAmount));
  console.log("  extra_bond_amount:", formatUsdc(project.extraBondAmount));
  console.log("  total_bond_amount:", formatUsdc(project.totalBondAmount));
  console.log("  bond_vault:", project.bondVault.toBase58());
  console.log("  bond_vault_authority:", project.bondVaultAuthority.toBase58());
  console.log("  bond_tier:", project.bondTier);
  console.log("  status:", project.status);
  console.log("  observation_start_ts:", project.observationStartTs.toString());
  console.log("  observation_end_ts:", project.observationEndTs.toString());
  console.log("  dispute_count:", project.disputeCount.toString());
  console.log("  active_dispute:", project.activeDispute.toBase58());
  console.log("  refunded_at:", project.refundedAt.toString());
  console.log("  slashed_at:", project.slashedAt.toString());
  console.log("  terminal_proposal_id:", project.terminalProposalId.toString());
  console.log("  terminal_proposal_decision:", project.terminalProposalDecision.toBase58());
  console.log("  terminal_execution_queue_item:", project.terminalExecutionQueueItem.toBase58());
  console.log("  terminal_payload_hash:", project.terminalPayloadHash.toString("hex"));
  console.log("  terminal_action_type:", project.terminalActionType);
}

export function printGreenLabelDispute(dispute: GreenLabelDisputeSummary): void {
  console.log("green_label_dispute:");
  console.log("  project_id:", dispute.projectId.toString());
  console.log("  dispute_id:", dispute.disputeId.toString());
  console.log("  project:", dispute.project.toBase58());
  console.log("  disputer:", dispute.disputer.toBase58());
  console.log("  reason_code:", dispute.reasonCode);
  console.log("  status:", dispute.status);
  console.log("  opened_at:", dispute.openedAt.toString());
  console.log("  evidence_end_ts:", dispute.evidenceEndTs.toString());
  console.log("  response_end_ts:", dispute.responseEndTs.toString());
  console.log("  resolved_at:", dispute.resolvedAt.toString());
  console.log("  proposal_id:", dispute.proposalId.toString());
  console.log("  proposal_decision:", dispute.proposalDecision.toBase58());
  console.log("  execution_queue_item:", dispute.executionQueueItem.toBase58());
  console.log("  payload_hash:", dispute.payloadHash.toString("hex"));
  console.log("  action_type:", dispute.actionType);
}

export async function printVaultBalance(
  provider: anchor.AnchorProvider,
  label: string,
  tokenAccount: PublicKey,
): Promise<void> {
  const balance = await getTokenBalance(provider, tokenAccount);
  console.log(`${label}: ${formatUsdc(balance)} USDC (${balance.toString()} base units)`);
}

export function buildInitializeGreenLabelConfigIx(args: {
  authority: PublicKey;
  usdcMint: PublicKey;
  treasuryUsdcStateV2: PublicKey;
  baseBondTreasuryVault: PublicKey;
  reliefOrRiskVault: PublicKey;
  vaultAuthorityV2: PublicKey;
  securityGovernanceConfig: PublicKey;
}): TransactionInstruction {
  return new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: deriveGreenLabelConfig(), isSigner: false, isWritable: true },
      { pubkey: args.authority, isSigner: true, isWritable: true },
      { pubkey: args.usdcMint, isSigner: false, isWritable: false },
      { pubkey: args.treasuryUsdcStateV2, isSigner: false, isWritable: false },
      { pubkey: args.baseBondTreasuryVault, isSigner: false, isWritable: false },
      { pubkey: args.reliefOrRiskVault, isSigner: false, isWritable: false },
      { pubkey: args.vaultAuthorityV2, isSigner: false, isWritable: false },
      { pubkey: args.securityGovernanceConfig, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: instructionDiscriminator("initialize_green_label_config"),
  });
}

export function buildSubmitGreenLabelApplicationIx(args: {
  projectId: bigint;
  projectNameHash: Buffer;
  projectUrlHash: Buffer;
  projectTreasuryWallet: PublicKey;
  totalBondAmount: bigint;
  projectOwner: PublicKey;
  tokenMint: PublicKey;
}): TransactionInstruction {
  return new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: deriveGreenLabelConfig(), isSigner: false, isWritable: true },
      { pubkey: deriveGreenLabelProject(args.projectId), isSigner: false, isWritable: true },
      { pubkey: args.projectOwner, isSigner: true, isWritable: true },
      { pubkey: args.tokenMint, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: Buffer.concat([
      instructionDiscriminator("submit_green_label_application"),
      u64Buffer(args.projectId),
      args.projectNameHash,
      args.projectUrlHash,
      args.projectTreasuryWallet.toBuffer(),
      u64Buffer(args.totalBondAmount),
    ]),
  });
}

export function buildInitializeGreenBondVaultIx(args: {
  projectId: bigint;
  projectOwner: PublicKey;
  usdcMint: PublicKey;
}): TransactionInstruction {
  const project = deriveGreenLabelProject(args.projectId);
  return new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: deriveGreenLabelConfig(), isSigner: false, isWritable: false },
      { pubkey: project, isSigner: false, isWritable: true },
      { pubkey: deriveGreenBondVault(project), isSigner: false, isWritable: true },
      { pubkey: deriveGreenBondVaultAuthority(project), isSigner: false, isWritable: false },
      { pubkey: args.projectOwner, isSigner: true, isWritable: true },
      { pubkey: args.usdcMint, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
    ],
    data: instructionDiscriminator("initialize_green_bond_vault"),
  });
}

export function buildLockGreenLabelBondIx(args: {
  projectId: bigint;
  projectOwner: PublicKey;
  projectOwnerUsdcAta: PublicKey;
  usdcMint: PublicKey;
}): TransactionInstruction {
  const project = deriveGreenLabelProject(args.projectId);
  return new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: deriveGreenLabelConfig(), isSigner: false, isWritable: false },
      { pubkey: project, isSigner: false, isWritable: true },
      { pubkey: args.projectOwner, isSigner: true, isWritable: false },
      { pubkey: args.projectOwnerUsdcAta, isSigner: false, isWritable: true },
      { pubkey: deriveGreenBondVault(project), isSigner: false, isWritable: true },
      { pubkey: args.usdcMint, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data: instructionDiscriminator("lock_green_label_bond"),
  });
}

export function buildOpenGreenLabelDisputeIx(args: {
  projectId: bigint;
  disputeId: bigint;
  reasonCode: EnumValue;
  evidenceHash: Buffer;
  disputer: PublicKey;
}): TransactionInstruction {
  const project = deriveGreenLabelProject(args.projectId);
  return new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: deriveGreenLabelConfig(), isSigner: false, isWritable: false },
      { pubkey: project, isSigner: false, isWritable: true },
      { pubkey: deriveGreenLabelDispute(project, args.disputeId), isSigner: false, isWritable: true },
      { pubkey: args.disputer, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: Buffer.concat([
      instructionDiscriminator("open_green_label_dispute"),
      u64Buffer(args.disputeId),
      Buffer.from([args.reasonCode.index]),
      args.evidenceHash,
    ]),
  });
}

export function buildMarkDisputeReadyForDecisionIx(args: {
  projectId: bigint;
  disputeId: bigint;
  caller: PublicKey;
}): TransactionInstruction {
  const project = deriveGreenLabelProject(args.projectId);
  return new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: deriveGreenLabelConfig(), isSigner: false, isWritable: false },
      { pubkey: project, isSigner: false, isWritable: true },
      { pubkey: deriveGreenLabelDispute(project, args.disputeId), isSigner: false, isWritable: true },
      { pubkey: args.caller, isSigner: true, isWritable: false },
    ],
    data: instructionDiscriminator("mark_dispute_ready_for_decision"),
  });
}

export function buildCreateProposalDecisionIx(args: {
  proposalId: bigint;
  proposalType: EnumValue;
  decision: EnumValue;
  yesWeight: bigint;
  noWeight: bigint;
  startTs: bigint;
  endTs: bigint;
  authority: PublicKey;
}): TransactionInstruction {
  return new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: deriveGovernanceConfig(), isSigner: false, isWritable: true },
      { pubkey: deriveProposalDecision(args.proposalId), isSigner: false, isWritable: true },
      { pubkey: args.authority, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: Buffer.concat([
      instructionDiscriminator("create_proposal_decision"),
      u64Buffer(args.proposalId),
      Buffer.from([args.proposalType.index]),
      Buffer.from([args.decision.index]),
      u64Buffer(args.yesWeight),
      u64Buffer(args.noWeight),
      i64Buffer(args.startTs),
      i64Buffer(args.endTs),
    ]),
  });
}

export function buildQueueExecutionIx(args: {
  proposalId: bigint;
  actionType: EnumValue;
  targetProgram: PublicKey;
  targetAccount: PublicKey;
  payloadHash: Buffer;
  authority: PublicKey;
}): TransactionInstruction {
  return new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: deriveGovernanceConfig(), isSigner: false, isWritable: false },
      { pubkey: deriveProposalDecision(args.proposalId), isSigner: false, isWritable: false },
      { pubkey: deriveExecutionQueueItem(args.proposalId), isSigner: false, isWritable: true },
      { pubkey: args.authority, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: Buffer.concat([
      instructionDiscriminator("queue_execution"),
      u64Buffer(args.proposalId),
      Buffer.from([args.actionType.index]),
      args.targetProgram.toBuffer(),
      args.targetAccount.toBuffer(),
      args.payloadHash,
    ]),
  });
}

export function buildLinkGreenLabelSecurityDecisionIx(args: {
  projectId: bigint;
  disputeId: bigint;
  proposalId: bigint;
  actionType: EnumValue;
  payloadHash: Buffer;
  linker: PublicKey;
}): TransactionInstruction {
  const project = deriveGreenLabelProject(args.projectId);
  return new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: deriveGreenLabelConfig(), isSigner: false, isWritable: false },
      { pubkey: project, isSigner: false, isWritable: true },
      { pubkey: deriveGreenLabelDispute(project, args.disputeId), isSigner: false, isWritable: true },
      { pubkey: deriveProposalDecision(args.proposalId), isSigner: false, isWritable: false },
      { pubkey: deriveExecutionQueueItem(args.proposalId), isSigner: false, isWritable: false },
      { pubkey: args.linker, isSigner: true, isWritable: false },
    ],
    data: Buffer.concat([
      instructionDiscriminator("link_green_label_security_decision"),
      u64Buffer(args.proposalId),
      Buffer.from([args.actionType.index]),
      args.payloadHash,
    ]),
  });
}

export function buildExecuteGreenLabelRefundIx(args: {
  projectId: bigint;
  disputeId: bigint;
  executor: PublicKey;
  projectOwnerUsdcAta: PublicKey;
  baseBondTreasuryVault: PublicKey;
  usdcMint: PublicKey;
  proposalId: bigint;
}): TransactionInstruction {
  const project = deriveGreenLabelProject(args.projectId);
  return new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: deriveGreenLabelConfig(), isSigner: false, isWritable: false },
      { pubkey: project, isSigner: false, isWritable: true },
      { pubkey: deriveGreenLabelDispute(project, args.disputeId), isSigner: false, isWritable: true },
      { pubkey: deriveProposalDecision(args.proposalId), isSigner: false, isWritable: false },
      { pubkey: deriveExecutionQueueItem(args.proposalId), isSigner: false, isWritable: false },
      { pubkey: deriveGreenBondVault(project), isSigner: false, isWritable: true },
      { pubkey: deriveGreenBondVaultAuthority(project), isSigner: false, isWritable: false },
      { pubkey: args.projectOwnerUsdcAta, isSigner: false, isWritable: true },
      { pubkey: args.baseBondTreasuryVault, isSigner: false, isWritable: true },
      { pubkey: args.usdcMint, isSigner: false, isWritable: false },
      { pubkey: args.executor, isSigner: true, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data: instructionDiscriminator("execute_green_label_refund"),
  });
}

export function buildExecuteGreenLabelSlashIx(args: {
  projectId: bigint;
  disputeId: bigint;
  executor: PublicKey;
  reliefOrRiskVault: PublicKey;
  usdcMint: PublicKey;
  proposalId: bigint;
}): TransactionInstruction {
  const project = deriveGreenLabelProject(args.projectId);
  return new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: deriveGreenLabelConfig(), isSigner: false, isWritable: false },
      { pubkey: project, isSigner: false, isWritable: true },
      { pubkey: deriveGreenLabelDispute(project, args.disputeId), isSigner: false, isWritable: true },
      { pubkey: deriveProposalDecision(args.proposalId), isSigner: false, isWritable: false },
      { pubkey: deriveExecutionQueueItem(args.proposalId), isSigner: false, isWritable: false },
      { pubkey: deriveGreenBondVault(project), isSigner: false, isWritable: true },
      { pubkey: deriveGreenBondVaultAuthority(project), isSigner: false, isWritable: false },
      { pubkey: args.reliefOrRiskVault, isSigner: false, isWritable: true },
      { pubkey: args.usdcMint, isSigner: false, isWritable: false },
      { pubkey: args.executor, isSigner: true, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data: instructionDiscriminator("execute_green_label_slash"),
  });
}

export async function requireShortEnoughResponseWindow(
  dispute: GreenLabelDisputeSummary,
): Promise<void> {
  const now = BigInt(Math.floor(Date.now() / 1000));
  if (now >= dispute.responseEndTs) {
    return;
  }

  const maxWaitSeconds = readU64Env("MAX_RESPONSE_WAIT_SECONDS", 120n);
  const waitSeconds = dispute.responseEndTs - now;
  if (waitSeconds > maxWaitSeconds) {
    throw new Error(
      `Current config response window has not ended. Need to wait ${waitSeconds.toString()}s, ` +
        `which is above MAX_RESPONSE_WAIT_SECONDS=${maxWaitSeconds.toString()}. ` +
        "This script will not bypass the contract time rule. Re-run later or initialize a short-window Devnet config before E2E.",
    );
  }

  await sleepUntil(dispute.responseEndTs, "Green Label dispute response window");
}

export async function nextProposalId(
  provider: anchor.AnchorProvider,
  governance: GovernanceConfigSummary,
): Promise<bigint> {
  const envProposalId = process.env.PROPOSAL_ID;
  if (envProposalId) {
    return readU64Env("PROPOSAL_ID", 0n);
  }

  const nextId = governance.proposalCount + 1n;
  const queue = await provider.connection.getAccountInfo(deriveExecutionQueueItem(nextId));
  const decision = await provider.connection.getAccountInfo(deriveProposalDecision(nextId));
  if (queue || decision) {
    throw new Error(
      `Derived proposal id ${nextId.toString()} already exists. Set PROPOSAL_ID explicitly.`,
    );
  }
  return nextId;
}
