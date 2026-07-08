import {
  Connection,
  PublicKey,
  type ParsedAccountData,
} from "@solana/web3.js";
import * as crypto from "crypto";
import * as fs from "fs";
import * as path from "path";

export type Cluster = "devnet" | "mainnet-beta";
export type ExpectedMode = "devnet-test" | "mainnet-production";

export const DEFAULT_PROGRAM_ID = new PublicKey("HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY");
export const DEFAULT_GREEN_LABEL_CONFIG = new PublicKey("7hNAeoqZxqvp38giY9gZwfR5ai3ttYTrse63QNYrRBWS");
export const DEFAULT_DEVNET_RPC_URL = "https://api.devnet.solana.com";
export const IDL_PATH = path.resolve(__dirname, "../../target/idl/my_first_solana_program.json");
export const BPF_LOADER_UPGRADEABLE_PROGRAM_ID = new PublicKey(
  "BPFLoaderUpgradeab1e11111111111111111111111",
);

const GREEN_LABEL_CONFIG_DISCRIMINATOR = anchorAccountDiscriminator("GreenLabelConfigV1");
const SECURITY_GOVERNANCE_CONFIG_V1_DISCRIMINATOR =
  anchorAccountDiscriminator("GovernanceConfigV1");

const REQUIRED_GREEN_LABEL_INSTRUCTIONS = [
  "initialize_green_label_config",
  "submit_green_label_application",
  "initialize_green_bond_vault",
  "lock_green_label_bond",
  "open_green_label_dispute",
  "mark_dispute_ready_for_decision",
  "link_green_label_security_decision",
  "execute_green_label_refund",
  "execute_green_label_slash",
  "update_green_label_windows",
  "update_green_label_min_base_bond",
] as const;

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

export type SecurityGovernanceConfigV1Decoded = {
  authority: PublicKey;
  minExecutionDelaySeconds: bigint;
  proposalCount: bigint;
  emergencyGuardian: PublicKey;
  isPaused: boolean;
  bump: number;
};

export type RuntimeConfig = {
  cluster: Cluster;
  rpcUrl: string | null;
  expectedMode: ExpectedMode;
  programId: PublicKey;
  greenLabelConfig: PublicKey;
};

export type CheckSummary = {
  pass: string[];
  warn: string[];
  fail: string[];
  manualReview: string[];
};

export type RuntimeIdl = {
  address?: string;
  metadata?: {
    address?: string;
  };
  instructions?: Array<{
    name: string;
    discriminator?: number[];
  }>;
};

export function createSummary(): CheckSummary {
  return {
    pass: [],
    warn: [],
    fail: [],
    manualReview: [],
  };
}

export function addPass(summary: CheckSummary, message: string): void {
  summary.pass.push(message);
  console.log(`PASS: ${message}`);
}

export function addWarn(summary: CheckSummary, message: string): void {
  summary.warn.push(message);
  console.log(`WARN: ${message}`);
}

export function addFail(summary: CheckSummary, message: string): void {
  summary.fail.push(message);
  console.log(`FAIL: ${message}`);
}

export function addManualReview(summary: CheckSummary, message: string): void {
  summary.manualReview.push(message);
  console.log(`MANUAL_REVIEW: ${message}`);
}

export function printSummary(summary: CheckSummary): void {
  console.log("");
  console.log("=== Summary ===");
  printSummaryGroup("PASS", summary.pass);
  printSummaryGroup("WARN", summary.warn);
  printSummaryGroup("FAIL", summary.fail);
  printSummaryGroup("MANUAL_REVIEW", summary.manualReview);
}

export function resolveRuntimeConfig(summary: CheckSummary): RuntimeConfig {
  const lifecycleEvent = process.env.npm_lifecycle_event ?? "";
  const inferredCluster: Cluster = lifecycleEvent.includes("mainnet")
    ? "mainnet-beta"
    : "devnet";
  const cluster = readClusterEnv(process.env.CLUSTER, inferredCluster, summary);
  const expectedMode = readExpectedModeEnv(
    process.env.EXPECTED_MODE,
    cluster === "mainnet-beta" ? "mainnet-production" : "devnet-test",
    summary,
  );
  const rpcUrl = resolveRpcUrl(cluster, summary);
  const programId = readPublicKeyEnv("PROGRAM_ID", DEFAULT_PROGRAM_ID, summary);
  const greenLabelConfig = readPublicKeyEnv(
    "GREEN_LABEL_CONFIG",
    DEFAULT_GREEN_LABEL_CONFIG,
    summary,
  );

  return {
    cluster,
    rpcUrl,
    expectedMode,
    programId,
    greenLabelConfig,
  };
}

export function printEnvironment(config: RuntimeConfig): void {
  console.log("=== Environment / Cluster ===");
  console.log("cluster:", config.cluster);
  console.log("RPC URL:", config.rpcUrl ?? "<missing>");
  console.log("expected mode:", config.expectedMode);
  console.log("program id:", config.programId.toBase58());
  console.log("green label config PDA:", config.greenLabelConfig.toBase58());
  console.log("local IDL path:", IDL_PATH);
  console.log("current time:", new Date().toISOString());
}

export function checkClusterEndpoint(config: RuntimeConfig, summary: CheckSummary): void {
  if (!config.rpcUrl) {
    return;
  }

  const normalized = config.rpcUrl.toLowerCase();
  const usesDevnet = normalized.includes("devnet");
  const usesMainnet = normalized.includes("mainnet");

  if (config.cluster === "mainnet-beta" && usesDevnet) {
    addFail(summary, "mainnet sanity check is using a devnet RPC endpoint");
    return;
  }

  if (config.cluster === "devnet" && usesMainnet) {
    addFail(summary, "devnet sanity check is using a mainnet RPC endpoint");
    return;
  }

  addPass(summary, "cluster and RPC endpoint do not conflict");
}

export function checkLocalIdl(programId: PublicKey, summary: CheckSummary): RuntimeIdl | null {
  console.log("");
  console.log("=== Local IDL ===");

  if (!fs.existsSync(IDL_PATH)) {
    addFail(summary, `IDL file does not exist: ${IDL_PATH}`);
    return null;
  }

  addPass(summary, `IDL file exists: ${IDL_PATH}`);

  let idl: RuntimeIdl;
  try {
    idl = JSON.parse(fs.readFileSync(IDL_PATH, "utf8")) as RuntimeIdl;
  } catch (error) {
    addFail(summary, `IDL JSON parse failed: ${formatError(error)}`);
    return null;
  }

  const idlAddress = idl.address ?? idl.metadata?.address ?? null;
  console.log("IDL address:", idlAddress ?? "<missing>");
  if (idlAddress === programId.toBase58()) {
    addPass(summary, "IDL address matches Program ID");
  } else {
    addFail(
      summary,
      `IDL address mismatch. Expected ${programId.toBase58()}, got ${idlAddress ?? "<missing>"}`,
    );
  }

  const instructions = idl.instructions ?? [];
  for (const name of REQUIRED_GREEN_LABEL_INSTRUCTIONS) {
    const exists = instructions.some((instruction) => instruction.name === name);
    if (exists) {
      addPass(summary, `IDL instruction exists: ${name}`);
    } else {
      addFail(summary, `IDL missing required Green Label instruction: ${name}`);
    }
  }

  return idl;
}

export async function checkProgramAccount(
  connection: Connection,
  programId: PublicKey,
  summary: CheckSummary,
): Promise<void> {
  console.log("");
  console.log("=== Program Account ===");

  const info = await connection.getAccountInfo(programId, "confirmed");
  if (!info) {
    addFail(summary, `Program account does not exist: ${programId.toBase58()}`);
    return;
  }

  addPass(summary, "Program account exists");
  console.log("executable:", info.executable);
  console.log("owner:", info.owner.toBase58());
  console.log("lamports:", info.lamports);
  console.log("data length:", info.data.length);

  if (info.executable) {
    addPass(summary, "Program account is executable");
  } else {
    addFail(summary, "Program account is not executable");
  }

  if (info.owner.equals(BPF_LOADER_UPGRADEABLE_PROGRAM_ID)) {
    addPass(summary, "Program owner is BPFLoaderUpgradeable");
  } else {
    addFail(
      summary,
      `Program owner is not BPFLoaderUpgradeable: ${info.owner.toBase58()}`,
    );
  }

  const programDataAddress = parseProgramDataAddress(info.data);
  console.log("ProgramData address:", programDataAddress?.toBase58() ?? "<unparsed>");
  if (programDataAddress) {
    addPass(summary, "ProgramData address parsed from Program account");
  } else {
    addWarn(summary, "ProgramData address could not be parsed from Program account");
  }
}

export async function checkGreenLabelConfigAccount(
  connection: Connection,
  configAddress: PublicKey,
  programId: PublicKey,
  summary: CheckSummary,
): Promise<GreenLabelConfigSummary | null> {
  console.log("");
  console.log("=== GreenLabelConfigV1 ===");

  const info = await connection.getAccountInfo(configAddress, "confirmed");
  if (!info) {
    addFail(summary, `GreenLabelConfigV1 does not exist: ${configAddress.toBase58()}`);
    return null;
  }

  addPass(summary, "GreenLabelConfigV1 account exists");
  console.log("owner:", info.owner.toBase58());
  console.log("lamports:", info.lamports);
  console.log("data length:", info.data.length);

  if (info.owner.equals(programId)) {
    addPass(summary, "GreenLabelConfigV1 owner matches Program ID");
  } else {
    addFail(
      summary,
      `GreenLabelConfigV1 owner mismatch. Expected ${programId.toBase58()}, got ${info.owner.toBase58()}`,
    );
  }

  if (info.data.subarray(0, 8).equals(GREEN_LABEL_CONFIG_DISCRIMINATOR)) {
    addPass(summary, "GreenLabelConfigV1 Anchor discriminator matches");
  } else {
    addFail(summary, "GreenLabelConfigV1 Anchor discriminator mismatch");
    return null;
  }

  let config: GreenLabelConfigSummary;
  try {
    config = decodeGreenLabelConfig(info.data);
  } catch (error) {
    addFail(summary, `GreenLabelConfigV1 decode failed: ${formatError(error)}`);
    return null;
  }

  printGreenLabelConfig(config);
  return config;
}

export function checkParameterPolicy(
  config: GreenLabelConfigSummary,
  expectedMode: ExpectedMode,
  summary: CheckSummary,
): void {
  console.log("");
  console.log("=== Parameter Policy ===");

  const isDevnet1Usdc = config.minBaseBondUsdc === 1_000_000n;
  const isDevnet30s =
    config.observationPeriodSeconds === 30n &&
    config.disputeWindowSeconds === 30n &&
    config.responseWindowSeconds === 30n;
  const isDevnet60s =
    config.observationPeriodSeconds === 60n &&
    config.disputeWindowSeconds === 60n &&
    config.responseWindowSeconds === 60n;
  const isMainnetProduction =
    config.minBaseBondUsdc === 299_000_000n &&
    config.observationPeriodSeconds === 2_592_000n &&
    config.disputeWindowSeconds === 604_800n &&
    config.responseWindowSeconds === 259_200n;

  if (expectedMode === "devnet-test") {
    if (isDevnet1Usdc && (isDevnet30s || isDevnet60s)) {
      addPass(summary, "Devnet test parameters match allowed 1 USDC + 30/60 second windows");
    } else {
      addWarn(summary, "Devnet test parameters differ from the expected 1 USDC + 30/60 second windows");
    }
    addWarn(summary, "Devnet test params are not Mainnet production params");
    return;
  }

  if (isMainnetProduction && !config.isPaused) {
    addPass(summary, "Mainnet production parameters match 299 USDC / 30d / 7d / 3d and config is unpaused");
  } else {
    addFail(
      summary,
      "Mainnet production parameters must be 299 USDC / 30d / 7d / 3d and is_paused=false",
    );
  }

  if (isDevnet1Usdc || isDevnet30s || isDevnet60s) {
    addFail(summary, "Mainnet mode detected Devnet test parameters");
  }
}

export async function checkTokenVault(
  connection: Connection,
  label: string,
  vaultAddress: PublicKey,
  expectedMint: PublicKey,
  summary: CheckSummary,
): Promise<void> {
  console.log("");
  console.log(`=== Token Vault: ${label} ===`);
  console.log("address:", vaultAddress.toBase58());

  try {
    const account = await connection.getParsedAccountInfo(vaultAddress, "confirmed");
    if (!account.value) {
      addWarn(summary, `${label} token account does not exist or could not be read`);
      return;
    }

    console.log("owner program:", account.value.owner.toBase58());
    console.log("lamports:", account.value.lamports);

    const data = account.value.data;
    if (typeof data === "string" || !("parsed" in data)) {
      addWarn(summary, `${label} is not a parsed token account`);
      return;
    }

    const parsed = data as ParsedAccountData;
    const tokenInfo = readParsedTokenInfo(parsed);
    if (!tokenInfo) {
      addWarn(summary, `${label} parsed token account shape was not recognized`);
      return;
    }

    console.log("mint:", tokenInfo.mint);
    console.log("token owner/authority:", tokenInfo.owner);
    console.log("balance:", tokenInfo.amount, "raw /", tokenInfo.uiAmountString ?? "<unknown>", "UI");

    if (tokenInfo.mint === expectedMint.toBase58()) {
      addPass(summary, `${label} token mint matches GreenLabelConfig usdc_mint`);
    } else {
      addFail(
        summary,
        `${label} token mint mismatch. Expected ${expectedMint.toBase58()}, got ${tokenInfo.mint}`,
      );
    }
  } catch (error) {
    addWarn(summary, `${label} parsed token account read failed: ${formatError(error)}`);
  }
}

export async function checkSecurityGovernanceConfig(
  connection: Connection,
  governanceConfig: PublicKey,
  programId: PublicKey,
  expectedMode: ExpectedMode,
  summary: CheckSummary,
): Promise<void> {
  console.log("");
  console.log("=== Security Governance Config ===");
  console.log("address:", governanceConfig.toBase58());

  const info = await connection.getAccountInfo(governanceConfig, "confirmed");
  if (!info) {
    addFail(summary, "Security governance config account does not exist");
    return;
  }

  addPass(summary, "Security governance config account exists");
  console.log("owner:", info.owner.toBase58());
  console.log("lamports:", info.lamports);
  console.log("data length:", info.data.length);

  if (info.owner.equals(programId)) {
    addPass(summary, "Security governance config owner matches Program ID");
  } else {
    addFail(summary, `Security governance config owner mismatch: ${info.owner.toBase58()}`);
  }

  if (info.data.subarray(0, 8).equals(SECURITY_GOVERNANCE_CONFIG_V1_DISCRIMINATOR)) {
    addPass(summary, "Security governance config discriminator matches");
  } else {
    addFail(summary, "Security governance config discriminator mismatch");
    return;
  }

  let config: SecurityGovernanceConfigV1Decoded;
  try {
    config = decodeSecurityGovernanceConfigV1(info.data);
  } catch (error) {
    addFail(summary, `Security governance config decode failed: ${formatError(error)}`);
    return;
  }

  addPass(summary, "Security governance config decoded successfully");
  printSecurityGovernanceConfig(config);
  checkSecurityGovernancePolicy(config, expectedMode, summary);
}

export function checkAuthorityPolicy(
  config: GreenLabelConfigSummary,
  expectedMode: ExpectedMode,
  summary: CheckSummary,
): void {
  console.log("");
  console.log("=== Authority Policy ===");
  console.log("GreenLabelConfig authority:", config.authority.toBase58());

  if (expectedMode === "mainnet-production") {
    addManualReview(
      summary,
      "MANUAL_REVIEW_REQUIRED: GreenLabelConfig authority must be multisig/governance/timelock before Mainnet production",
    );
  } else {
    addWarn(summary, "Devnet authority may be a test wallet; do not carry this assumption into Mainnet");
  }
}

export function formatUsdc(amount: bigint): string {
  const whole = amount / 1_000_000n;
  const fractional = (amount % 1_000_000n).toString().padStart(6, "0");
  return `${whole}.${fractional}`;
}

function anchorAccountDiscriminator(accountName: string): Buffer {
  return crypto
    .createHash("sha256")
    .update(`account:${accountName}`)
    .digest()
    .subarray(0, 8);
}

function decodeGreenLabelConfig(data: Buffer): GreenLabelConfigSummary {
  const minimumLength = 8 + 32 + 32 + 8 + 2 + 2 + 8 + 8 + 8 + 8 + 32 + 32 + 32 + 32 + 32 + 1 + 1;
  if (data.length < minimumLength) {
    throw new Error(`account data too short. Expected at least ${minimumLength}, got ${data.length}`);
  }

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

function decodeSecurityGovernanceConfigV1(data: Buffer): SecurityGovernanceConfigV1Decoded {
  const minimumLength = 8 + 32 + 8 + 8 + 32 + 1 + 1;
  if (data.length < minimumLength) {
    throw new Error(`account data too short. Expected at least ${minimumLength}, got ${data.length}`);
  }

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

function readPubkey(data: Buffer, offset: number): PublicKey {
  return new PublicKey(data.subarray(offset, offset + 32));
}

function printGreenLabelConfig(config: GreenLabelConfigSummary): void {
  console.log("authority:", config.authority.toBase58());
  console.log("usdc_mint:", config.usdcMint.toBase58());
  console.log("min_base_bond_usdc:", `${formatUsdc(config.minBaseBondUsdc)} USDC`);
  console.log("base_refund_bps:", config.baseRefundBps);
  console.log("base_treasury_bps:", config.baseTreasuryBps);
  console.log("observation_period_seconds:", config.observationPeriodSeconds.toString());
  console.log("dispute_window_seconds:", config.disputeWindowSeconds.toString());
  console.log("response_window_seconds:", config.responseWindowSeconds.toString());
  console.log("project_count:", config.projectCount.toString());
  console.log("treasury_usdc_state_v2:", config.treasuryUsdcStateV2.toBase58());
  console.log("base_bond_treasury_vault:", config.baseBondTreasuryVault.toBase58());
  console.log("relief_or_risk_vault:", config.reliefOrRiskVault.toBase58());
  console.log("vault_authority_v2:", config.vaultAuthorityV2.toBase58());
  console.log("security_governance_config:", config.securityGovernanceConfig.toBase58());
  console.log("is_paused:", config.isPaused);
  console.log("bump:", config.bump);
}

function printSecurityGovernanceConfig(config: SecurityGovernanceConfigV1Decoded): void {
  console.log("authority:", config.authority.toBase58());
  console.log(
    "min_execution_delay_seconds:",
    formatSecondsWithReadable(config.minExecutionDelaySeconds),
  );
  console.log("proposal_count:", config.proposalCount.toString());
  console.log("emergency_guardian:", config.emergencyGuardian.toBase58());
  console.log("is_paused:", config.isPaused);
  console.log("bump:", config.bump);
}

function checkSecurityGovernancePolicy(
  config: SecurityGovernanceConfigV1Decoded,
  expectedMode: ExpectedMode,
  summary: CheckSummary,
): void {
  console.log("");
  console.log("=== Security Governance Policy ===");

  if (expectedMode === "devnet-test") {
    if (config.isPaused) {
      addWarn(summary, "Security governance config is paused in Devnet test mode");
    } else {
      addPass(summary, "Security governance config is not paused in Devnet test mode");
    }

    if (config.minExecutionDelaySeconds <= 0n) {
      addWarn(summary, "Security governance timelock delay is zero or negative in Devnet test mode");
    } else {
      addPass(summary, "Security governance timelock delay is nonzero in Devnet test mode");
    }

    addWarn(
      summary,
      "Devnet Security governance authority and emergency guardian may be test wallets; review before Mainnet",
    );
    return;
  }

  if (config.isPaused) {
    addFail(summary, "Security governance config must not be paused for Mainnet production");
  } else {
    addPass(summary, "Security governance config is not paused for Mainnet production");
  }

  if (config.minExecutionDelaySeconds <= 0n) {
    addFail(summary, "Security governance timelock delay must be greater than zero for Mainnet");
  } else {
    addPass(summary, "Security governance timelock delay is nonzero for Mainnet");
  }

  addManualReview(
    summary,
    "MANUAL_REVIEW_REQUIRED: Confirm Security governance authority is multisig/governance/timelock before Mainnet",
  );
  addManualReview(
    summary,
    "MANUAL_REVIEW_REQUIRED: Confirm emergency guardian permissions are pause-only before Mainnet",
  );
}

function formatSecondsWithReadable(seconds: bigint): string {
  const sign = seconds < 0n ? "-" : "";
  const absolute = seconds < 0n ? -seconds : seconds;

  if (absolute === 0n) {
    return `${seconds.toString()} (0 seconds)`;
  }

  if (absolute % 86_400n === 0n) {
    return `${seconds.toString()} (${sign}${(absolute / 86_400n).toString()} days)`;
  }

  if (absolute % 3_600n === 0n) {
    return `${seconds.toString()} (${sign}${(absolute / 3_600n).toString()} hours)`;
  }

  if (absolute % 60n === 0n) {
    return `${seconds.toString()} (${sign}${(absolute / 60n).toString()} minutes)`;
  }

  return `${seconds.toString()} (${sign}${absolute.toString()} seconds)`;
}

function parseProgramDataAddress(data: Buffer): PublicKey | null {
  if (data.length < 36 || data.readUInt32LE(0) !== 2) {
    return null;
  }
  return new PublicKey(data.subarray(4, 36));
}

function readClusterEnv(value: string | undefined, fallback: Cluster, summary: CheckSummary): Cluster {
  if (!value) {
    return fallback;
  }
  if (value === "devnet" || value === "mainnet-beta") {
    return value;
  }
  addFail(summary, `CLUSTER must be devnet or mainnet-beta. Received: ${value}`);
  return fallback;
}

function readExpectedModeEnv(
  value: string | undefined,
  fallback: ExpectedMode,
  summary: CheckSummary,
): ExpectedMode {
  if (!value) {
    return fallback;
  }
  if (value === "devnet-test" || value === "mainnet-production") {
    return value;
  }
  addFail(summary, `EXPECTED_MODE must be devnet-test or mainnet-production. Received: ${value}`);
  return fallback;
}

function resolveRpcUrl(cluster: Cluster, summary: CheckSummary): string | null {
  if (process.env.RPC_URL) {
    return process.env.RPC_URL;
  }

  if (cluster === "devnet") {
    return DEFAULT_DEVNET_RPC_URL;
  }

  addFail(summary, "RPC_URL is required for mainnet-beta sanity check");
  return null;
}

function readPublicKeyEnv(name: string, fallback: PublicKey, summary: CheckSummary): PublicKey {
  const value = process.env[name];
  if (!value) {
    return fallback;
  }
  try {
    return new PublicKey(value);
  } catch {
    addFail(summary, `${name} is not a valid public key: ${value}`);
    return fallback;
  }
}

function printSummaryGroup(title: string, items: string[]): void {
  console.log(`${title}:`);
  if (items.length === 0) {
    console.log("- <none>");
    return;
  }
  for (const item of items) {
    console.log(`- ${item}`);
  }
}

function readParsedTokenInfo(parsed: ParsedAccountData): {
  mint: string;
  owner: string;
  amount: string;
  uiAmountString?: string;
} | null {
  const info = parsed.parsed?.info as unknown;
  if (!info || typeof info !== "object") {
    return null;
  }

  const tokenInfo = info as {
    mint?: unknown;
    owner?: unknown;
    tokenAmount?: {
      amount?: unknown;
      uiAmountString?: unknown;
    };
  };
  if (
    typeof tokenInfo.mint !== "string" ||
    typeof tokenInfo.owner !== "string" ||
    typeof tokenInfo.tokenAmount?.amount !== "string"
  ) {
    return null;
  }

  return {
    mint: tokenInfo.mint,
    owner: tokenInfo.owner,
    amount: tokenInfo.tokenAmount.amount,
    uiAmountString:
      typeof tokenInfo.tokenAmount.uiAmountString === "string"
        ? tokenInfo.tokenAmount.uiAmountString
        : undefined,
  };
}

function formatError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
