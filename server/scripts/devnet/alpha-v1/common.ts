import * as anchor from "@coral-xyz/anchor";
import { getAccount, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
  AccountMeta,
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
import * as crypto from "crypto";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";

export const PROGRAM_ID = new PublicKey("HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY");
export const EXPECTED_UPGRADE_AUTHORITY = new PublicKey("CqSs2yq6Jo3gYwXBq7fGRqohcxXS7HFJNYypykZTEGa8");
export const DEVNET_USDC_MINT = new PublicKey("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU");
export const EXPECTED_RELIEF_USDC_VAULT = new PublicKey("GQSK91eQ5zwzGfYchunVqrPtxe3WLokxY88JbzTVcuRM");
export const EXPECTED_VAULT_AUTHORITY_V2 = new PublicKey("FovfcDDZzc8ff2Z2uxNZ1fTjpuVoLkRTPUPTLvXL8TEK");
export const DEVNET_GENESIS_HASH = "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG";
export const DEFAULT_RPC_URL = "https://api.devnet.solana.com";
export const BPF_LOADER_UPGRADEABLE_PROGRAM_ID = new PublicKey("BPFLoaderUpgradeab1e11111111111111111111111");
export const IDL_PATH = path.resolve(__dirname, "../../../target/idl/my_first_solana_program.json");

export const SEEDS = {
  governanceConfig: "governance_config_v1",
  governanceProposal: "governance_proposal_v1",
  governanceProposalAction: "governance_proposal_action_v1",
  universalGovernanceDecisionAdapter: "universal_governance_decision_adapter_v1",
  proposalDecision: "proposal_decision_v1",
  executionQueueItem: "execution_queue_item_v1",
  protocolModuleRegistry: "protocol_module_registry_v1",
  protocolAuthorityControl: "protocol_authority_control_v1",
  protocolDaoControlActivationRecord: "protocol_dao_control_rcpt_v1",
  protocolSecurityUnpauseRecord: "protocol_unpause_rcpt_v1",
  treasuryConfigV2: "treasury_config_v2",
  treasuryUsdcStateV2: "treasury_usdc_state_v2",
  reliefUsdcVault: "relief_usdc_vault",
  buybackUsdcVault: "buyback_usdc_vault",
  buildersUsdcVault: "builders_usdc_vault",
  stakingUsdcVault: "staking_usdc_vault",
  vaultAuthorityV2: "vault_authority_v2",
  victimReliefConfig: "victim_relief_config_v1",
  victimReliefPolicy: "victim_relief_policy_v1",
  victimReliefClaimantState: "victim_relief_claimant_state_v1",
  victimReliefCase: "victim_relief_case_v1",
  victimReliefEvidenceSnapshot: "victim_relief_evidence_snap_v1",
  victimReliefDecisionRecord: "victim_relief_decision_rcpt_v1",
  victimReliefAppeal: "victim_relief_appeal_v1",
  victimReliefAppealRecord: "victim_relief_appeal_rcpt_v1",
  victimReliefPauseRecord: "victim_relief_pause_rcpt_v1",
  reliefPayoutRequest: "relief_payout_request_v1",
  reliefPayoutRecord: "relief_payout_rcpt_v1",
  reliefPayoutCancelRecord: "relief_payout_cancel_rcpt_v1",
} as const;

export const MODULE_CODES = {
  Treasury: 1,
  GreenLabel: 2,
  VictimRelief: 3,
  ScamRegistry: 4,
  Contributor: 5,
  Protocol: 6,
} as const;

export type RuntimeIdl = anchor.Idl & {
  address?: string;
  metadata?: { address?: string };
  instructions: Array<{ name: string; discriminator?: number[] }>;
  accounts?: Array<{ name: string }>;
};

export type DevnetContext = {
  provider: anchor.AnchorProvider;
  wallet: PublicKey;
  rpcUrl: string;
  idl: RuntimeIdl;
};

export function isDryRun(): boolean {
  return (process.env.DRY_RUN ?? "false").toLowerCase() === "true";
}

export function allowDevnetTx(): boolean {
  return (process.env.CONFIRM_DEVNET_TX ?? "false").toLowerCase() === "true";
}

export function resolveRpcUrl(): string {
  return process.env.ANCHOR_PROVIDER_URL || process.env.RPC_URL || DEFAULT_RPC_URL;
}

function expandHome(filePath: string): string {
  if (filePath === "~") {
    return os.homedir();
  }
  if (filePath.startsWith("~/")) {
    return path.join(os.homedir(), filePath.slice(2));
  }
  return filePath;
}

export function resolveWalletPath(): string {
  return expandHome(process.env.ANCHOR_WALLET || path.join(os.homedir(), ".config/solana/id.json"));
}

export function loadWallet(): anchor.Wallet {
  const walletPath = resolveWalletPath();
  if (!fs.existsSync(walletPath)) {
    throw new Error(`Wallet keypair not found: ${walletPath}`);
  }
  const secret = JSON.parse(fs.readFileSync(walletPath, "utf8")) as number[];
  return new anchor.Wallet(Keypair.fromSecretKey(Uint8Array.from(secret)));
}

export function loadIdl(): RuntimeIdl {
  if (!fs.existsSync(IDL_PATH)) {
    throw new Error(`IDL not found: ${IDL_PATH}. Run anchor build --ignore-keys before Devnet scripts.`);
  }
  const idl = JSON.parse(fs.readFileSync(IDL_PATH, "utf8")) as RuntimeIdl;
  const address = idl.address || idl.metadata?.address;
  if (address && address !== PROGRAM_ID.toBase58()) {
    throw new Error(`IDL program id mismatch. Expected ${PROGRAM_ID.toBase58()}, got ${address}`);
  }
  return idl;
}

export function idlInstructionNames(idl: RuntimeIdl): string[] {
  return idl.instructions.map((instruction) => instruction.name);
}

export function requireFreshIdl(idl: RuntimeIdl, requiredInstructions: string[]): void {
  const available = new Set(idlInstructionNames(idl));
  const missing = requiredInstructions.filter((name) => !available.has(name));
  if (missing.length > 0) {
    throw new Error(
      `Generated IDL is stale or incomplete. Missing instructions: ${missing.join(", ")}. ` +
        "Run anchor build --ignore-keys before transaction scripts.",
    );
  }
}

export function instructionDiscriminator(idl: RuntimeIdl, name: string): Buffer {
  const instruction = idl.instructions.find((ix) => ix.name === name);
  if (!instruction) {
    throw new Error(`Instruction ${name} is missing from the generated IDL.`);
  }
  if (instruction.discriminator) {
    const discriminator = Buffer.from(instruction.discriminator);
    if (discriminator.length !== 8) {
      throw new Error(`IDL discriminator for ${name} must be 8 bytes.`);
    }
    return discriminator;
  }
  return crypto.createHash("sha256").update(`global:${name}`).digest().subarray(0, 8);
}

export function u16(value: number): Buffer {
  const buffer = Buffer.alloc(2);
  buffer.writeUInt16LE(value);
  return buffer;
}

export function u32(value: number): Buffer {
  const buffer = Buffer.alloc(4);
  buffer.writeUInt32LE(value);
  return buffer;
}

export function u64(value: bigint): Buffer {
  const buffer = Buffer.alloc(8);
  buffer.writeBigUInt64LE(value);
  return buffer;
}

export function i64(value: bigint): Buffer {
  const buffer = Buffer.alloc(8);
  buffer.writeBigInt64LE(value);
  return buffer;
}

export function pk(value: string | undefined, name: string): PublicKey {
  if (!value) {
    throw new Error(`Missing required environment variable: ${name}`);
  }
  return new PublicKey(value);
}

export function optionalPk(value: string | undefined): PublicKey | null {
  return value ? new PublicKey(value) : null;
}

export function readU64(name: string): bigint {
  const raw = process.env[name];
  if (!raw || !/^\d+$/.test(raw)) {
    throw new Error(`${name} must be a positive integer string.`);
  }
  return BigInt(raw);
}

export function readU64Default(name: string, defaultValue: bigint): bigint {
  const raw = process.env[name];
  if (!raw) {
    return defaultValue;
  }
  if (!/^\d+$/.test(raw)) {
    throw new Error(`${name} must be a non-negative integer string.`);
  }
  return BigInt(raw);
}

export function readI64Default(name: string, defaultValue: bigint): bigint {
  const raw = process.env[name];
  if (!raw) {
    return defaultValue;
  }
  if (!/^-?\d+$/.test(raw)) {
    throw new Error(`${name} must be an integer string.`);
  }
  return BigInt(raw);
}

export function readU32Default(name: string, defaultValue: number): number {
  const raw = process.env[name];
  if (!raw) {
    return defaultValue;
  }
  const value = Number(raw);
  if (!Number.isInteger(value) || value < 0 || value > 0xffffffff) {
    throw new Error(`${name} must fit in u32.`);
  }
  return value;
}

export function readU16Default(name: string, defaultValue: number): number {
  const raw = process.env[name];
  if (!raw) {
    return defaultValue;
  }
  const value = Number(raw);
  if (!Number.isInteger(value) || value < 0 || value > 0xffff) {
    throw new Error(`${name} must fit in u16.`);
  }
  return value;
}

export function pda(seed: string): PublicKey {
  return PublicKey.findProgramAddressSync([Buffer.from(seed)], PROGRAM_ID)[0];
}

export function pdaWithSeeds(seeds: Array<Buffer | Uint8Array>): PublicKey {
  return PublicKey.findProgramAddressSync(seeds, PROGRAM_ID)[0];
}

export function deriveGovernanceConfig(): PublicKey {
  return pda(SEEDS.governanceConfig);
}

export function deriveTreasuryConfigV2(): PublicKey {
  return pda(SEEDS.treasuryConfigV2);
}

export function deriveVictimReliefConfig(): PublicKey {
  return pda(SEEDS.victimReliefConfig);
}

export function deriveVictimReliefPolicy(config: PublicKey, policyVersion = 1n): PublicKey {
  return pdaWithSeeds([Buffer.from(SEEDS.victimReliefPolicy), config.toBuffer(), u64(policyVersion)]);
}

export function deriveVictimReliefCase(config: PublicKey, caseId: bigint): PublicKey {
  return pdaWithSeeds([Buffer.from(SEEDS.victimReliefCase), config.toBuffer(), u64(caseId)]);
}

export function deriveVictimReliefClaimantState(config: PublicKey, claimant: PublicKey): PublicKey {
  return pdaWithSeeds([Buffer.from(SEEDS.victimReliefClaimantState), config.toBuffer(), claimant.toBuffer()]);
}

export function deriveEvidenceSnapshot(caseKey: PublicKey): PublicKey {
  return pdaWithSeeds([Buffer.from(SEEDS.victimReliefEvidenceSnapshot), caseKey.toBuffer()]);
}

export function deriveAppeal(caseKey: PublicKey): PublicKey {
  return pdaWithSeeds([Buffer.from(SEEDS.victimReliefAppeal), caseKey.toBuffer()]);
}

export function deriveReliefPayoutRequest(caseKey: PublicKey): PublicKey {
  return pdaWithSeeds([Buffer.from(SEEDS.reliefPayoutRequest), caseKey.toBuffer()]);
}

export function derivePayoutRecord(request: PublicKey): PublicKey {
  return pdaWithSeeds([Buffer.from(SEEDS.reliefPayoutRecord), request.toBuffer()]);
}

export function derivePayoutCancelRecord(request: PublicKey): PublicKey {
  return pdaWithSeeds([Buffer.from(SEEDS.reliefPayoutCancelRecord), request.toBuffer()]);
}

export function deriveGovernanceProposal(proposalId: bigint): PublicKey {
  return pdaWithSeeds([Buffer.from(SEEDS.governanceProposal), u64(proposalId)]);
}

export function deriveGovernanceProposalAction(proposal: PublicKey): PublicKey {
  return pdaWithSeeds([Buffer.from(SEEDS.governanceProposalAction), proposal.toBuffer()]);
}

export function deriveGovernanceAdapter(proposal: PublicKey): PublicKey {
  return pdaWithSeeds([Buffer.from(SEEDS.universalGovernanceDecisionAdapter), proposal.toBuffer()]);
}

export function deriveProposalDecision(proposalId: bigint): PublicKey {
  return pdaWithSeeds([Buffer.from(SEEDS.proposalDecision), u64(proposalId)]);
}

export function deriveExecutionQueueItem(proposalId: bigint): PublicKey {
  return pdaWithSeeds([Buffer.from(SEEDS.executionQueueItem), u64(proposalId)]);
}

export function deriveDecisionRecord(queue: PublicKey): PublicKey {
  return pdaWithSeeds([Buffer.from(SEEDS.victimReliefDecisionRecord), queue.toBuffer()]);
}

export function deriveAppealRecord(queue: PublicKey): PublicKey {
  return pdaWithSeeds([Buffer.from(SEEDS.victimReliefAppealRecord), queue.toBuffer()]);
}

export function derivePauseRecord(queue: PublicKey): PublicKey {
  return pdaWithSeeds([Buffer.from(SEEDS.victimReliefPauseRecord), queue.toBuffer()]);
}

export function deriveProtocolModuleRegistry(moduleCode: number): PublicKey {
  return pdaWithSeeds([Buffer.from(SEEDS.protocolModuleRegistry), Buffer.from([moduleCode])]);
}

export function deriveProtocolAuthorityControl(governanceConfig = deriveGovernanceConfig()): PublicKey {
  return pdaWithSeeds([Buffer.from(SEEDS.protocolAuthorityControl), governanceConfig.toBuffer()]);
}

export function deriveProtocolActivationRecord(authorityControl: PublicKey): PublicKey {
  return pdaWithSeeds([Buffer.from(SEEDS.protocolDaoControlActivationRecord), authorityControl.toBuffer()]);
}

export function deriveProtocolUnpauseRecord(queue: PublicKey): PublicKey {
  return pdaWithSeeds([Buffer.from(SEEDS.protocolSecurityUnpauseRecord), queue.toBuffer()]);
}

export function deriveTreasuryPdas(): Record<string, PublicKey> {
  return {
    treasuryConfigV2: deriveTreasuryConfigV2(),
    treasuryUsdcStateV2: pda(SEEDS.treasuryUsdcStateV2),
    reliefUsdcVault: pda(SEEDS.reliefUsdcVault),
    buybackUsdcVault: pda(SEEDS.buybackUsdcVault),
    buildersUsdcVault: pda(SEEDS.buildersUsdcVault),
    stakingUsdcVault: pda(SEEDS.stakingUsdcVault),
    vaultAuthorityV2: pda(SEEDS.vaultAuthorityV2),
  };
}

export function meta(pubkey: PublicKey, isSigner = false, isWritable = false): AccountMeta {
  return { pubkey, isSigner, isWritable };
}

export async function assertDevnet(connection: Connection): Promise<string> {
  const endpoint = connection.rpcEndpoint;
  const lower = endpoint.toLowerCase();
  if (lower.includes("mainnet")) {
    throw new Error(`Refusing Mainnet endpoint: ${endpoint}`);
  }
  if (!lower.includes("devnet") && process.env.ALLOW_CUSTOM_DEVNET_RPC !== "true") {
    throw new Error(`Refusing non-devnet RPC URL without ALLOW_CUSTOM_DEVNET_RPC=true: ${endpoint}`);
  }
  const genesis = await connection.getGenesisHash();
  if (genesis !== DEVNET_GENESIS_HASH) {
    throw new Error(`Refusing non-Devnet genesis hash: ${genesis}`);
  }
  return genesis;
}

export async function loadDevnetContext(args: {
  scriptName: string;
  sendsTransactions: boolean;
  requiredInstructions?: string[];
}): Promise<DevnetContext> {
  const rpcUrl = resolveRpcUrl();
  const connection = new Connection(rpcUrl, "confirmed");
  const wallet = loadWallet();
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: "confirmed",
    preflightCommitment: "confirmed",
  });
  anchor.setProvider(provider);
  const idl = loadIdl();
  if (args.requiredInstructions?.length) {
    requireFreshIdl(idl, args.requiredInstructions);
  }
  const genesis = await assertDevnet(connection);
  const programInfo = await connection.getAccountInfo(PROGRAM_ID);
  if (!programInfo?.executable) {
    throw new Error(`Program is not executable on Devnet: ${PROGRAM_ID.toBase58()}`);
  }
  console.log(`Alpha Protocol Devnet script: ${args.scriptName}`);
  console.log("cluster:", "devnet");
  console.log("RPC URL:", rpcUrl);
  console.log("genesis:", genesis);
  console.log("Program ID:", PROGRAM_ID.toBase58());
  console.log("wallet:", wallet.publicKey.toBase58());
  console.log("DRY_RUN:", String(isDryRun()));
  console.log(
    "will send transactions:",
    args.sendsTransactions
      ? isDryRun() || !allowDevnetTx()
        ? "no (DRY_RUN=true or CONFIRM_DEVNET_TX not true)"
        : "yes (Devnet only)"
      : "no (read-only)",
  );
  if (args.sendsTransactions) {
    console.log("WARNING: transaction scripts are Devnet-only and require CONFIRM_DEVNET_TX=true.");
  }
  return { provider, wallet: wallet.publicKey, rpcUrl, idl };
}

export async function sendOrPlan(ctx: DevnetContext, label: string, tx: Transaction): Promise<string> {
  await assertDevnet(ctx.provider.connection);
  console.log("operation:", label);
  console.log("instruction_count:", tx.instructions.length);
  if (isDryRun() || !allowDevnetTx()) {
    const sig = `DRY_RUN_${label.replace(/[^a-z0-9]+/gi, "_").toUpperCase()}`;
    console.log("DRY_RUN_OR_LOCKED:", sig);
    console.log("Set DRY_RUN=false and CONFIRM_DEVNET_TX=true to send this Devnet transaction.");
    return sig;
  }
  const sig = await ctx.provider.sendAndConfirm(tx, []);
  console.log("signature:", sig);
  console.log("confirmed:", true);
  console.log("explorer:", `https://explorer.solana.com/tx/${sig}?cluster=devnet`);
  return sig;
}

export function buildIx(idl: RuntimeIdl, name: string, keys: AccountMeta[], payload: Uint8Array = Buffer.alloc(0)): TransactionInstruction {
  return new TransactionInstruction({
    programId: PROGRAM_ID,
    keys,
    data: Buffer.concat([instructionDiscriminator(idl, name), Buffer.from(payload)]),
  });
}

export async function accountExists(connection: Connection, pubkey: PublicKey): Promise<boolean> {
  return (await connection.getAccountInfo(pubkey)) !== null;
}

export async function printAccount(connection: Connection, label: string, pubkey: PublicKey): Promise<void> {
  const info = await connection.getAccountInfo(pubkey);
  console.log(`${label}:`, pubkey.toBase58(), info ? `exists owner=${info.owner.toBase58()} lamports=${info.lamports} bytes=${info.data.length}` : "missing");
}

export async function printTokenAccount(connection: Connection, label: string, pubkey: PublicKey): Promise<void> {
  const info = await connection.getAccountInfo(pubkey);
  if (!info) {
    console.log(`${label}:`, pubkey.toBase58(), "missing");
    return;
  }
  try {
    const token = await getAccount(connection, pubkey);
    console.log(`${label}:`, pubkey.toBase58(), `mint=${token.mint.toBase58()} owner=${token.owner.toBase58()} amount=${token.amount.toString()}`);
  } catch {
    console.log(`${label}:`, pubkey.toBase58(), `exists owner=${info.owner.toBase58()} bytes=${info.data.length}`);
  }
}

export async function readProgramDeployment(connection: Connection): Promise<{
  programData: PublicKey | null;
  upgradeAuthority: PublicKey | null;
  dataLength: number;
}> {
  const info = await connection.getAccountInfo(PROGRAM_ID);
  if (!info || !info.owner.equals(BPF_LOADER_UPGRADEABLE_PROGRAM_ID) || info.data.length < 36) {
    return { programData: null, upgradeAuthority: null, dataLength: info?.data.length ?? 0 };
  }
  const programData = new PublicKey(info.data.subarray(4, 36));
  const programDataInfo = await connection.getAccountInfo(programData);
  if (!programDataInfo || programDataInfo.data.length < 45) {
    return { programData, upgradeAuthority: null, dataLength: programDataInfo?.data.length ?? 0 };
  }
  const upgradeAuthority = programDataInfo.data[12] === 1 ? new PublicKey(programDataInfo.data.subarray(13, 45)) : null;
  return { programData, upgradeAuthority, dataLength: programDataInfo.data.length };
}

export function printRequiredEnv(names: string[]): void {
  for (const name of names) {
    console.log(`${name}:`, process.env[name] ?? "<missing>");
  }
}

export function requiredScenarioBasics(): {
  caseId: bigint;
  claimant: PublicKey;
  recipient: PublicKey | null;
} {
  return {
    caseId: readU64("CASE_ID"),
    claimant: pk(process.env.CLAIMANT, "CLAIMANT"),
    recipient: optionalPk(process.env.RECIPIENT_USDC_TOKEN_ACCOUNT),
  };
}

export const SYS = SystemProgram.programId;
export const TOKEN = TOKEN_PROGRAM_ID;