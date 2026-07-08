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
export const DEFAULT_DEVNET_STAKING_POOL = new PublicKey("91PjLExu9FCLY6KQuvuisEhTEciQyWXJGW9fMKUEHW35");
export const DEFAULT_DEVNET_RPC_URL = "https://api.devnet.solana.com";
export const IDL_PATH = path.resolve(__dirname, "../../target/idl/my_first_solana_program.json");
export const BPF_LOADER_UPGRADEABLE_PROGRAM_ID = new PublicKey(
  "BPFLoaderUpgradeab1e11111111111111111111111",
);

const GREEN_LABEL_CONFIG_DISCRIMINATOR = anchorAccountDiscriminator("GreenLabelConfigV1");
const SECURITY_GOVERNANCE_CONFIG_V1_DISCRIMINATOR =
  anchorAccountDiscriminator("GovernanceConfigV1");
const TREASURY_CONFIG_V2_DISCRIMINATOR = anchorAccountDiscriminator("TreasuryConfigV2");
const TREASURY_USDC_STATE_V2_DISCRIMINATOR = anchorAccountDiscriminator("TreasuryUsdcStateV2");
const STAKING_POOL_V1_DISCRIMINATOR = anchorAccountDiscriminator("StakingPoolV1");

const TREASURY_CONFIG_V2_SEED = Buffer.from("treasury_config_v2");
const TREASURY_USDC_STATE_V2_SEED = Buffer.from("treasury_usdc_state_v2");
const RELIEF_USDC_VAULT_SEED = Buffer.from("relief_usdc_vault");
const BUYBACK_USDC_VAULT_SEED = Buffer.from("buyback_usdc_vault");
const BUILDERS_USDC_VAULT_SEED = Buffer.from("builders_usdc_vault");
const STAKING_USDC_VAULT_SEED = Buffer.from("staking_usdc_vault");
const VAULT_AUTHORITY_V2_SEED = Buffer.from("vault_authority_v2");

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

export type TreasuryConfigV2Decoded = {
  authority: PublicKey;
  usdcMint: PublicKey;
  alphaMint: PublicKey;
  bump: number;
};

export type TreasuryUsdcStateV2Decoded = {
  totalUsdcInflow: bigint;
  reliefUsdcTotal: bigint;
  buybackUsdcTotal: bigint;
  buildersUsdcTotal: bigint;
  stakingUsdcTotal: bigint;
  bump: number;
};

export type TreasuryV2CheckResult = {
  treasuryConfig: TreasuryConfigV2Decoded;
  treasuryUsdcState: TreasuryUsdcStateV2Decoded;
  treasuryConfigAddress: PublicKey;
  treasuryUsdcStateAddress: PublicKey;
  vaultAuthorityV2: PublicKey;
  reliefUsdcVault: PublicKey;
  buybackUsdcVault: PublicKey;
  buildersUsdcVault: PublicKey;
  stakingUsdcVault: PublicKey;
};

export type StakingPoolV1Decoded = {
  authority: PublicKey;
  alphaMint: PublicKey;
  usdcMint: PublicKey;
  alphaVault: PublicKey;
  alphaVaultAuthority: PublicKey;
  stakingUsdcVault: PublicKey;
  vaultAuthorityV2: PublicKey;
  totalStakedAlpha: bigint;
  totalEffectiveWeight: bigint;
  accUsdcPerWeight: bigint;
  lastRewardUpdateTs: bigint;
  lastObservedUsdcBalance: bigint;
  rewardReleaseBps: number;
  minClaimUsdc: bigint;
  vaultAuthorityV2Bump: number;
  alphaVaultAuthorityBump: number;
  bump: number;
};

export type RuntimeConfig = {
  cluster: Cluster;
  rpcUrl: string | null;
  expectedMode: ExpectedMode;
  programId: PublicKey;
  greenLabelConfig: PublicKey;
  treasuryUsdcStateOverride: PublicKey | null;
  stakingPool: PublicKey | null;
  stakingPoolSource: "env" | "default-devnet" | "missing";
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

type ParsedTokenAccountInfo = {
  mint: string;
  owner: string;
  amount: string;
  uiAmountString?: string;
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
  const treasuryUsdcStateOverride = readOptionalPublicKeyEnv(
    "TREASURY_USDC_STATE_V2",
    summary,
  );
  const { stakingPool, stakingPoolSource } = resolveStakingPoolAddress(
    cluster,
    expectedMode,
    summary,
  );

  return {
    cluster,
    rpcUrl,
    expectedMode,
    programId,
    greenLabelConfig,
    treasuryUsdcStateOverride,
    stakingPool,
    stakingPoolSource,
  };
}

export function printEnvironment(config: RuntimeConfig): void {
  console.log("=== Environment / Cluster ===");
  console.log("cluster:", config.cluster);
  console.log("RPC URL:", config.rpcUrl ?? "<missing>");
  console.log("expected mode:", config.expectedMode);
  console.log("program id:", config.programId.toBase58());
  console.log("green label config PDA:", config.greenLabelConfig.toBase58());
  console.log(
    "treasury_usdc_state_v2 source:",
    config.treasuryUsdcStateOverride
      ? "TREASURY_USDC_STATE_V2 env"
      : "GreenLabelConfig.treasury_usdc_state_v2 after decode",
  );
  console.log(
    "treasury_usdc_state_v2 override:",
    config.treasuryUsdcStateOverride?.toBase58() ?? "<none>",
  );
  console.log("staking_pool source:", config.stakingPoolSource);
  console.log("staking_pool:", config.stakingPool?.toBase58() ?? "<missing>");
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
  const tokenInfo = await readParsedTokenAccount(
    connection,
    label,
    vaultAddress,
    summary,
    "warn",
  );
  if (!tokenInfo) {
    return;
  }

  if (tokenInfo.mint === expectedMint.toBase58()) {
    addPass(summary, `${label} token mint matches GreenLabelConfig usdc_mint`);
  } else {
    addFail(
      summary,
      `${label} token mint mismatch. Expected ${expectedMint.toBase58()}, got ${tokenInfo.mint}`,
    );
  }
}

export async function checkTreasuryV2(
  connection: Connection,
  programId: PublicKey,
  greenLabelConfig: GreenLabelConfigSummary,
  treasuryUsdcStateOverride: PublicKey | null,
  expectedMode: ExpectedMode,
  summary: CheckSummary,
): Promise<TreasuryV2CheckResult | null> {
  console.log("");
  console.log("=== Treasury V2 ===");

  const treasuryConfigAddress = findPda(TREASURY_CONFIG_V2_SEED, programId);
  const expectedTreasuryUsdcStateAddress = findPda(TREASURY_USDC_STATE_V2_SEED, programId);
  const treasuryUsdcStateAddress =
    treasuryUsdcStateOverride ?? greenLabelConfig.treasuryUsdcStateV2;
  const treasuryUsdcStateSource = treasuryUsdcStateOverride
    ? "TREASURY_USDC_STATE_V2 env"
    : "GreenLabelConfig.treasury_usdc_state_v2";
  const vaultAuthorityV2 = findPda(VAULT_AUTHORITY_V2_SEED, programId);
  const reliefUsdcVault = findPda(RELIEF_USDC_VAULT_SEED, programId);
  const buybackUsdcVault = findPda(BUYBACK_USDC_VAULT_SEED, programId);
  const buildersUsdcVault = findPda(BUILDERS_USDC_VAULT_SEED, programId);
  const stakingUsdcVault = findPda(STAKING_USDC_VAULT_SEED, programId);

  console.log("treasury_config_v2 PDA:", treasuryConfigAddress.toBase58());
  console.log("treasury_usdc_state_v2:", treasuryUsdcStateAddress.toBase58());
  console.log("treasury_usdc_state_v2 source:", treasuryUsdcStateSource);
  console.log("expected treasury_usdc_state_v2 PDA:", expectedTreasuryUsdcStateAddress.toBase58());
  console.log("vault_authority_v2 PDA:", vaultAuthorityV2.toBase58());

  if (treasuryUsdcStateAddress.equals(expectedTreasuryUsdcStateAddress)) {
    addPass(summary, "Treasury USDC state matches expected PDA seed");
  } else {
    addWarn(summary, "Treasury USDC state does not match expected PDA seed");
  }

  const treasuryConfig = await readTreasuryConfigV2Account(
    connection,
    treasuryConfigAddress,
    programId,
    summary,
  );
  const treasuryUsdcState = await readTreasuryUsdcStateV2Account(
    connection,
    treasuryUsdcStateAddress,
    programId,
    summary,
  );

  if (!treasuryConfig || !treasuryUsdcState) {
    addFail(summary, "Treasury V2 decode unavailable; vault checks skipped");
    return null;
  }

  if (treasuryConfig.usdcMint.equals(greenLabelConfig.usdcMint)) {
    addPass(summary, "Treasury V2 usdc_mint matches GreenLabelConfig usdc_mint");
  } else {
    addFail(
      summary,
      `Treasury V2 usdc_mint mismatch. GreenLabelConfig=${greenLabelConfig.usdcMint.toBase58()} Treasury=${treasuryConfig.usdcMint.toBase58()}`,
    );
  }

  const result: TreasuryV2CheckResult = {
    treasuryConfig,
    treasuryUsdcState,
    treasuryConfigAddress,
    treasuryUsdcStateAddress,
    vaultAuthorityV2,
    reliefUsdcVault,
    buybackUsdcVault,
    buildersUsdcVault,
    stakingUsdcVault,
  };

  await checkTreasuryV2Vaults(connection, result, expectedMode, summary);
  checkTreasuryV2Policy(result, expectedMode, summary);

  return result;
}

export async function checkStakingPoolV1(
  connection: Connection,
  stakingPoolAddress: PublicKey | null,
  expectedMode: ExpectedMode,
  programId: PublicKey,
  summary: CheckSummary,
): Promise<StakingPoolV1Decoded | null> {
  console.log("");
  console.log("=== Staking Pool V1 ===");

  if (!stakingPoolAddress) {
    addFail(summary, "Staking pool address is missing");
    return null;
  }

  console.log("staking_pool:", stakingPoolAddress.toBase58());

  const info = await connection.getAccountInfo(stakingPoolAddress, "confirmed");
  if (!info) {
    addFail(summary, `StakingPoolV1 account does not exist: ${stakingPoolAddress.toBase58()}`);
    return null;
  }

  addPass(summary, "StakingPoolV1 account exists");
  console.log("owner:", info.owner.toBase58());
  console.log("lamports:", info.lamports);
  console.log("data length:", info.data.length);

  if (info.owner.equals(programId)) {
    addPass(summary, "StakingPoolV1 owner matches Program ID");
  } else {
    addFail(summary, `StakingPoolV1 owner mismatch: ${info.owner.toBase58()}`);
  }

  if (info.data.subarray(0, 8).equals(STAKING_POOL_V1_DISCRIMINATOR)) {
    addPass(summary, "StakingPoolV1 discriminator matches");
  } else {
    addFail(summary, "StakingPoolV1 discriminator mismatch");
    return null;
  }

  let pool: StakingPoolV1Decoded;
  try {
    pool = decodeStakingPoolV1(info.data);
  } catch (error) {
    addFail(summary, `StakingPoolV1 decode failed: ${formatError(error)}`);
    return null;
  }

  addPass(summary, "Staking pool decoded successfully");
  printStakingPoolV1(pool);

  await checkStakingVaults(connection, pool, expectedMode, summary);
  checkStakingPolicy(pool, expectedMode, summary);

  return pool;
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

function decodeTreasuryConfigV2(data: Buffer): TreasuryConfigV2Decoded {
  const minimumLength = 8 + 32 + 32 + 32 + 1;
  if (data.length < minimumLength) {
    throw new Error(`account data too short. Expected at least ${minimumLength}, got ${data.length}`);
  }

  let offset = 8;
  const authority = readPubkey(data, offset);
  offset += 32;
  const usdcMint = readPubkey(data, offset);
  offset += 32;
  const alphaMint = readPubkey(data, offset);
  offset += 32;
  const bump = data.readUInt8(offset);

  return {
    authority,
    usdcMint,
    alphaMint,
    bump,
  };
}

function decodeTreasuryUsdcStateV2(data: Buffer): TreasuryUsdcStateV2Decoded {
  const minimumLength = 8 + (8 * 5) + 1;
  if (data.length < minimumLength) {
    throw new Error(`account data too short. Expected at least ${minimumLength}, got ${data.length}`);
  }

  let offset = 8;
  const totalUsdcInflow = data.readBigUInt64LE(offset);
  offset += 8;
  const reliefUsdcTotal = data.readBigUInt64LE(offset);
  offset += 8;
  const buybackUsdcTotal = data.readBigUInt64LE(offset);
  offset += 8;
  const buildersUsdcTotal = data.readBigUInt64LE(offset);
  offset += 8;
  const stakingUsdcTotal = data.readBigUInt64LE(offset);
  offset += 8;
  const bump = data.readUInt8(offset);

  return {
    totalUsdcInflow,
    reliefUsdcTotal,
    buybackUsdcTotal,
    buildersUsdcTotal,
    stakingUsdcTotal,
    bump,
  };
}

function decodeStakingPoolV1(data: Buffer): StakingPoolV1Decoded {
  const minimumLength = 8 + (32 * 7) + 8 + 16 + 16 + 8 + 8 + 2 + 8 + 1 + 1 + 1;
  if (data.length < minimumLength) {
    throw new Error(`account data too short. Expected at least ${minimumLength}, got ${data.length}`);
  }

  let offset = 8;
  const authority = readPubkey(data, offset);
  offset += 32;
  const alphaMint = readPubkey(data, offset);
  offset += 32;
  const usdcMint = readPubkey(data, offset);
  offset += 32;
  const alphaVault = readPubkey(data, offset);
  offset += 32;
  const alphaVaultAuthority = readPubkey(data, offset);
  offset += 32;
  const stakingUsdcVault = readPubkey(data, offset);
  offset += 32;
  const vaultAuthorityV2 = readPubkey(data, offset);
  offset += 32;
  const totalStakedAlpha = data.readBigUInt64LE(offset);
  offset += 8;
  const totalEffectiveWeight = readBigUInt128LE(data, offset);
  offset += 16;
  const accUsdcPerWeight = readBigUInt128LE(data, offset);
  offset += 16;
  const lastRewardUpdateTs = data.readBigInt64LE(offset);
  offset += 8;
  const lastObservedUsdcBalance = data.readBigUInt64LE(offset);
  offset += 8;
  const rewardReleaseBps = data.readUInt16LE(offset);
  offset += 2;
  const minClaimUsdc = data.readBigUInt64LE(offset);
  offset += 8;
  const vaultAuthorityV2Bump = data.readUInt8(offset);
  offset += 1;
  const alphaVaultAuthorityBump = data.readUInt8(offset);
  offset += 1;
  const bump = data.readUInt8(offset);

  return {
    authority,
    alphaMint,
    usdcMint,
    alphaVault,
    alphaVaultAuthority,
    stakingUsdcVault,
    vaultAuthorityV2,
    totalStakedAlpha,
    totalEffectiveWeight,
    accUsdcPerWeight,
    lastRewardUpdateTs,
    lastObservedUsdcBalance,
    rewardReleaseBps,
    minClaimUsdc,
    vaultAuthorityV2Bump,
    alphaVaultAuthorityBump,
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

function printTreasuryConfigV2(config: TreasuryConfigV2Decoded): void {
  console.log("treasury_config.authority:", config.authority.toBase58());
  console.log("treasury_config.usdc_mint:", config.usdcMint.toBase58());
  console.log("treasury_config.alpha_mint:", config.alphaMint.toBase58());
  console.log("treasury_config.bump:", config.bump);
}

function printTreasuryUsdcStateV2(state: TreasuryUsdcStateV2Decoded): void {
  console.log("total_usdc_inflow:", `${formatUsdc(state.totalUsdcInflow)} USDC`);
  console.log("relief_usdc_total:", `${formatUsdc(state.reliefUsdcTotal)} USDC`);
  console.log("buyback_usdc_total:", `${formatUsdc(state.buybackUsdcTotal)} USDC`);
  console.log("builders_usdc_total:", `${formatUsdc(state.buildersUsdcTotal)} USDC`);
  console.log("staking_usdc_total:", `${formatUsdc(state.stakingUsdcTotal)} USDC`);
  console.log("treasury_usdc_state.bump:", state.bump);
}

function printStakingPoolV1(pool: StakingPoolV1Decoded): void {
  console.log("authority:", pool.authority.toBase58());
  console.log("alpha_mint:", pool.alphaMint.toBase58());
  console.log("usdc_mint:", pool.usdcMint.toBase58());
  console.log("alpha_vault:", pool.alphaVault.toBase58());
  console.log("alpha_vault_authority:", pool.alphaVaultAuthority.toBase58());
  console.log("staking_usdc_vault:", pool.stakingUsdcVault.toBase58());
  console.log("vault_authority_v2:", pool.vaultAuthorityV2.toBase58());
  console.log("total_staked_alpha:", pool.totalStakedAlpha.toString());
  console.log("total_effective_weight:", pool.totalEffectiveWeight.toString());
  console.log("acc_usdc_per_weight:", pool.accUsdcPerWeight.toString());
  console.log(
    "last_reward_update_ts:",
    `${pool.lastRewardUpdateTs.toString()} (${formatUnixTimestamp(pool.lastRewardUpdateTs)})`,
  );
  console.log(
    "last_observed_usdc_balance:",
    `${formatUsdc(pool.lastObservedUsdcBalance)} USDC`,
  );
  console.log("reward_release_bps:", pool.rewardReleaseBps);
  console.log("min_claim_usdc:", `${formatUsdc(pool.minClaimUsdc)} USDC`);
  console.log("vault_authority_v2_bump:", pool.vaultAuthorityV2Bump);
  console.log("alpha_vault_authority_bump:", pool.alphaVaultAuthorityBump);
  console.log("bump:", pool.bump);
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

async function readTreasuryConfigV2Account(
  connection: Connection,
  address: PublicKey,
  programId: PublicKey,
  summary: CheckSummary,
): Promise<TreasuryConfigV2Decoded | null> {
  console.log("");
  console.log("=== TreasuryConfigV2 ===");
  console.log("address:", address.toBase58());

  const info = await connection.getAccountInfo(address, "confirmed");
  if (!info) {
    addFail(summary, "TreasuryConfigV2 account does not exist");
    return null;
  }

  addPass(summary, "TreasuryConfigV2 account exists");
  console.log("owner:", info.owner.toBase58());
  console.log("lamports:", info.lamports);
  console.log("data length:", info.data.length);

  if (info.owner.equals(programId)) {
    addPass(summary, "TreasuryConfigV2 owner matches Program ID");
  } else {
    addFail(summary, `TreasuryConfigV2 owner mismatch: ${info.owner.toBase58()}`);
  }

  if (info.data.subarray(0, 8).equals(TREASURY_CONFIG_V2_DISCRIMINATOR)) {
    addPass(summary, "TreasuryConfigV2 discriminator matches");
  } else {
    addFail(summary, "TreasuryConfigV2 discriminator mismatch");
    return null;
  }

  try {
    const config = decodeTreasuryConfigV2(info.data);
    addPass(summary, "TreasuryConfigV2 decoded successfully");
    printTreasuryConfigV2(config);
    return config;
  } catch (error) {
    addFail(summary, `TreasuryConfigV2 decode failed: ${formatError(error)}`);
    return null;
  }
}

async function readTreasuryUsdcStateV2Account(
  connection: Connection,
  address: PublicKey,
  programId: PublicKey,
  summary: CheckSummary,
): Promise<TreasuryUsdcStateV2Decoded | null> {
  console.log("");
  console.log("=== TreasuryUsdcStateV2 ===");
  console.log("address:", address.toBase58());

  const info = await connection.getAccountInfo(address, "confirmed");
  if (!info) {
    addFail(summary, "TreasuryUsdcStateV2 account does not exist");
    return null;
  }

  addPass(summary, "TreasuryUsdcStateV2 account exists");
  console.log("owner:", info.owner.toBase58());
  console.log("lamports:", info.lamports);
  console.log("data length:", info.data.length);

  if (info.owner.equals(programId)) {
    addPass(summary, "TreasuryUsdcStateV2 owner matches Program ID");
  } else {
    addFail(summary, `TreasuryUsdcStateV2 owner mismatch: ${info.owner.toBase58()}`);
  }

  if (info.data.subarray(0, 8).equals(TREASURY_USDC_STATE_V2_DISCRIMINATOR)) {
    addPass(summary, "TreasuryUsdcStateV2 discriminator matches");
  } else {
    addFail(summary, "TreasuryUsdcStateV2 discriminator mismatch");
    return null;
  }

  try {
    const state = decodeTreasuryUsdcStateV2(info.data);
    addPass(summary, "Treasury V2 decoded successfully");
    printTreasuryUsdcStateV2(state);
    return state;
  } catch (error) {
    addFail(summary, `TreasuryUsdcStateV2 decode failed: ${formatError(error)}`);
    return null;
  }
}

async function checkTreasuryV2Vaults(
  connection: Connection,
  treasury: TreasuryV2CheckResult,
  expectedMode: ExpectedMode,
  summary: CheckSummary,
): Promise<void> {
  console.log("");
  console.log("=== Treasury V2 Vaults ===");

  const checks = [
    ["relief_usdc_vault", treasury.reliefUsdcVault],
    ["buyback_usdc_vault", treasury.buybackUsdcVault],
    ["builders_usdc_vault", treasury.buildersUsdcVault],
    ["staking_usdc_vault", treasury.stakingUsdcVault],
  ] as const;

  let matchingMintCount = 0;
  for (const [label, address] of checks) {
    const ok = await checkExpectedTokenVault(
      connection,
      label,
      address,
      treasury.treasuryConfig.usdcMint,
      treasury.vaultAuthorityV2,
      summary,
      "fail",
    );
    if (ok) {
      matchingMintCount += 1;
    }
  }

  if (matchingMintCount === checks.length) {
    addPass(summary, "Treasury vault mints match USDC mint");
  }

  if (expectedMode === "devnet-test") {
    addWarn(summary, "Devnet Treasury vault balances are test balances");
  }
}

function checkTreasuryV2Policy(
  treasury: TreasuryV2CheckResult,
  expectedMode: ExpectedMode,
  summary: CheckSummary,
): void {
  console.log("");
  console.log("=== Treasury V2 Policy ===");

  if (expectedMode === "mainnet-production") {
    addManualReview(
      summary,
      `MANUAL_REVIEW_REQUIRED: Confirm Mainnet USDC mint for Treasury V2: ${treasury.treasuryConfig.usdcMint.toBase58()}`,
    );
    addManualReview(
      summary,
      `MANUAL_REVIEW_REQUIRED: Confirm Treasury V2 vault authority is a PDA/governed authority: ${treasury.vaultAuthorityV2.toBase58()}`,
    );
    addManualReview(
      summary,
      `MANUAL_REVIEW_REQUIRED: Confirm Treasury V2 authority governance: ${treasury.treasuryConfig.authority.toBase58()}`,
    );
  } else {
    addWarn(summary, "Devnet Treasury V2 authority and mint are test-environment assumptions");
  }
}

async function checkStakingVaults(
  connection: Connection,
  pool: StakingPoolV1Decoded,
  expectedMode: ExpectedMode,
  summary: CheckSummary,
): Promise<void> {
  console.log("");
  console.log("=== Staking V1 Vaults ===");

  const alphaOk = await checkExpectedTokenVault(
    connection,
    "alpha_vault",
    pool.alphaVault,
    pool.alphaMint,
    pool.alphaVaultAuthority,
    summary,
    "fail",
  );
  const rewardsOk = await checkExpectedTokenVault(
    connection,
    "staking_usdc_vault",
    pool.stakingUsdcVault,
    pool.usdcMint,
    pool.vaultAuthorityV2,
    summary,
    "fail",
  );

  if (alphaOk && rewardsOk) {
    addPass(summary, "Staking vault mints match expected mints");
  }

  if (expectedMode === "devnet-test") {
    addWarn(summary, "Devnet Staking vault balances are test balances and may be zero");
  }
}

function checkStakingPolicy(
  pool: StakingPoolV1Decoded,
  expectedMode: ExpectedMode,
  summary: CheckSummary,
): void {
  console.log("");
  console.log("=== Staking V1 Policy ===");

  if (expectedMode === "mainnet-production") {
    addManualReview(
      summary,
      `MANUAL_REVIEW_REQUIRED: Confirm Mainnet ALPHA mint for Staking V1: ${pool.alphaMint.toBase58()}`,
    );
    addManualReview(
      summary,
      `MANUAL_REVIEW_REQUIRED: Confirm Staking V1 USDC rewards mint: ${pool.usdcMint.toBase58()}`,
    );
    addManualReview(
      summary,
      `MANUAL_REVIEW_REQUIRED: Confirm Staking V1 authority governance: ${pool.authority.toBase58()}`,
    );
    addManualReview(
      summary,
      `MANUAL_REVIEW_REQUIRED: Confirm Staking V1 vault authorities are expected PDAs: ${pool.alphaVaultAuthority.toBase58()} / ${pool.vaultAuthorityV2.toBase58()}`,
    );
  } else {
    addWarn(summary, "Devnet Staking V1 authority and mints are test-environment assumptions");
  }
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

function findPda(seed: Buffer, programId: PublicKey): PublicKey {
  const [address] = PublicKey.findProgramAddressSync([seed], programId);
  return address;
}

function readBigUInt128LE(data: Buffer, offset: number): bigint {
  const low = data.readBigUInt64LE(offset);
  const high = data.readBigUInt64LE(offset + 8);
  return low + (high << 64n);
}

async function checkExpectedTokenVault(
  connection: Connection,
  label: string,
  vaultAddress: PublicKey,
  expectedMint: PublicKey,
  expectedTokenOwner: PublicKey,
  summary: CheckSummary,
  missingSeverity: "warn" | "fail",
): Promise<boolean> {
  const tokenInfo = await readParsedTokenAccount(
    connection,
    label,
    vaultAddress,
    summary,
    missingSeverity,
  );
  if (!tokenInfo) {
    return false;
  }

  let ok = true;
  if (tokenInfo.mint === expectedMint.toBase58()) {
    addPass(summary, `${label} token mint matches expected mint`);
  } else {
    addFail(
      summary,
      `${label} token mint mismatch. Expected ${expectedMint.toBase58()}, got ${tokenInfo.mint}`,
    );
    ok = false;
  }

  if (tokenInfo.owner === expectedTokenOwner.toBase58()) {
    addPass(summary, `${label} token owner/authority matches expected vault authority`);
  } else {
    addFail(
      summary,
      `${label} token owner/authority mismatch. Expected ${expectedTokenOwner.toBase58()}, got ${tokenInfo.owner}`,
    );
    ok = false;
  }

  return ok;
}

async function readParsedTokenAccount(
  connection: Connection,
  label: string,
  vaultAddress: PublicKey,
  summary: CheckSummary,
  missingSeverity: "warn" | "fail",
): Promise<ParsedTokenAccountInfo | null> {
  console.log("");
  console.log(`=== Token Vault: ${label} ===`);
  console.log("address:", vaultAddress.toBase58());

  try {
    const account = await connection.getParsedAccountInfo(vaultAddress, "confirmed");
    if (!account.value) {
      addTokenAccountIssue(summary, missingSeverity, `${label} token account does not exist or could not be read`);
      return null;
    }

    console.log("owner program:", account.value.owner.toBase58());
    console.log("lamports:", account.value.lamports);

    const data = account.value.data;
    if (typeof data === "string" || !("parsed" in data)) {
      addTokenAccountIssue(summary, missingSeverity, `${label} is not a parsed token account`);
      return null;
    }

    const parsed = data as ParsedAccountData;
    const tokenInfo = readParsedTokenInfo(parsed);
    if (!tokenInfo) {
      addTokenAccountIssue(summary, missingSeverity, `${label} parsed token account shape was not recognized`);
      return null;
    }

    console.log("mint:", tokenInfo.mint);
    console.log("token owner/authority:", tokenInfo.owner);
    console.log("balance:", tokenInfo.amount, "raw /", tokenInfo.uiAmountString ?? "<unknown>", "UI");
    return tokenInfo;
  } catch (error) {
    addTokenAccountIssue(summary, missingSeverity, `${label} parsed token account read failed: ${formatError(error)}`);
    return null;
  }
}

function addTokenAccountIssue(
  summary: CheckSummary,
  severity: "warn" | "fail",
  message: string,
): void {
  if (severity === "fail") {
    addFail(summary, message);
  } else {
    addWarn(summary, message);
  }
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

function readOptionalPublicKeyEnv(name: string, summary: CheckSummary): PublicKey | null {
  const value = process.env[name];
  if (!value) {
    return null;
  }
  try {
    return new PublicKey(value);
  } catch {
    addFail(summary, `${name} is not a valid public key: ${value}`);
    return null;
  }
}

function resolveStakingPoolAddress(
  cluster: Cluster,
  expectedMode: ExpectedMode,
  summary: CheckSummary,
): {
  stakingPool: PublicKey | null;
  stakingPoolSource: RuntimeConfig["stakingPoolSource"];
} {
  const envValue = process.env.STAKING_POOL;
  if (envValue) {
    return {
      stakingPool: readPublicKeyEnv("STAKING_POOL", DEFAULT_DEVNET_STAKING_POOL, summary),
      stakingPoolSource: "env",
    };
  }

  if (cluster === "mainnet-beta" || expectedMode === "mainnet-production") {
    addFail(summary, "Mainnet requires explicit STAKING_POOL");
    return {
      stakingPool: null,
      stakingPoolSource: "missing",
    };
  }

  return {
    stakingPool: DEFAULT_DEVNET_STAKING_POOL,
    stakingPoolSource: "default-devnet",
  };
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

function readParsedTokenInfo(parsed: ParsedAccountData): ParsedTokenAccountInfo | null {
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

function formatUnixTimestamp(timestamp: bigint): string {
  if (timestamp <= 0n) {
    return "unset";
  }

  const milliseconds = Number(timestamp) * 1000;
  if (!Number.isSafeInteger(milliseconds)) {
    return "out of JavaScript Date range";
  }

  return new Date(milliseconds).toISOString();
}
