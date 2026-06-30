import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey, SystemProgram, Transaction, TransactionInstruction } from "@solana/web3.js";
import { execFileSync } from "child_process";
import * as crypto from "crypto";
import * as fs from "fs";
import * as path from "path";
import {
  DEVNET_USDC_MINT,
  IDL_PATH,
  PROGRAM_ID,
  RuntimeIdl,
  anchorDiscriminator,
  buildInitializeGreenLabelConfigIx,
  deriveGovernanceConfig,
  deriveGreenLabelConfig,
  deriveTreasuryPdas,
  getInstructionDiscriminator,
  loadIdl,
  loadProgram,
  loadProvider,
  readPublicKeyEnv,
} from "./common";

type DiagnosticInstruction = {
  name: string;
  discriminator?: number[];
};

type DiagnosticIdl = {
  address?: string;
  metadata?: {
    address?: string;
  };
  instructions: DiagnosticInstruction[];
};

type SetupAccounts = {
  authority: PublicKey;
  usdcMint: PublicKey;
  treasuryUsdcStateV2: PublicKey;
  baseBondTreasuryVault: PublicKey;
  reliefOrRiskVault: PublicKey;
  vaultAuthorityV2: PublicKey;
  securityGovernanceConfig: PublicKey;
};

type AnchorMethodIxResult = {
  ix: TransactionInstruction | null;
  methodName: string | null;
  accountsStyle: string | null;
  errors: string[];
};

type SimulateResult = {
  label: string;
  err: unknown;
  logs: string[];
  unitsConsumed: number | null;
  first8: string;
  skippedReason?: string;
};

type Verdict =
  | "SETUP_SIMULATION_OK_CAN_RUN_REAL_SETUP"
  | "DISCRIMINATOR_BUG"
  | "SCRIPT_INSTRUCTION_BUILDER_MISMATCH"
  | "MANUAL_BUILDER_BUG"
  | "DEPLOYED_BINARY_OR_PROGRAM_MISMATCH"
  | "LOCAL_IDL_STALE"
  | "LOCAL_SO_STALE"
  | "PROGRAM_ID_MISMATCH"
  | "RPC_DUMP_FAILED_BUT_SIMULATION_DIAGNOSTIC_COMPLETE"
  | "DISCRIMINATOR_FIXED_NEXT_ACCOUNT_ERROR"
  | "UNKNOWN_NEEDS_MANUAL_REVIEW";

const REQUIRED_INSTRUCTIONS = [
  "initialize_green_label_config",
  "submit_green_label_application",
  "initialize_green_bond_vault",
  "lock_green_label_bond",
  "open_green_label_dispute",
  "mark_dispute_ready_for_decision",
  "link_green_label_security_decision",
  "execute_green_label_refund",
  "execute_green_label_slash",
  "create_proposal_decision",
  "queue_execution",
] as const;

const LOCAL_SO_PATH = path.resolve(__dirname, "../../../target/deploy/my_first_solana_program.so");
const LOCAL_KEYPAIR_PATH = path.resolve(
  __dirname,
  "../../../target/deploy/my_first_solana_program-keypair.json",
);
const DEVNET_URL = "https://api.devnet.solana.com";
const DUMP_PATH = "/tmp/devnet-my-first-solana-program.so";

function section(title: string): void {
  console.log("");
  console.log(`=== ${title} ===`);
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function existsSizeSha256(filePath: string): { exists: boolean; size: number | null; sha256: string | null } {
  if (!fs.existsSync(filePath)) {
    return { exists: false, size: null, sha256: null };
  }
  const file = fs.readFileSync(filePath);
  return {
    exists: true,
    size: file.length,
    sha256: crypto.createHash("sha256").update(file).digest("hex"),
  };
}

function readRawIdl(): DiagnosticIdl {
  return JSON.parse(fs.readFileSync(IDL_PATH, "utf8")) as DiagnosticIdl;
}

function discriminatorHex(idl: DiagnosticIdl, name: string): string | null {
  const instruction = idl.instructions.find((ix) => ix.name === name);
  if (!instruction?.discriminator) {
    return null;
  }
  return Buffer.from(instruction.discriminator).toString("hex");
}

function printInstruction(ix: TransactionInstruction, name: string, idl: RuntimeIdl): void {
  console.log(`${name}:`);
  console.log("  programId:", ix.programId.toBase58());
  console.log("  data length:", ix.data.length);
  console.log("  data first 8 bytes hex:", ix.data.subarray(0, 8).toString("hex"));
  console.log("  full data hex:", ix.data.toString("hex"));
  console.log(
    "  matches IDL discriminator:",
    ix.data.subarray(0, 8).equals(getInstructionDiscriminator(idl, "initialize_green_label_config")),
  );
  console.log("  accounts count:", ix.keys.length);
  ix.keys.forEach((account, index) => {
    console.log(
      `  account[${index}]: pubkey=${account.pubkey.toBase58()} isSigner=${account.isSigner} isWritable=${account.isWritable}`,
    );
  });
}

function accountSignature(ix: TransactionInstruction): string {
  return ix.keys
    .map((account) => `${account.pubkey.toBase58()}:${account.isSigner}:${account.isWritable}`)
    .join("|");
}

function printBuilderComparison(manualIx: TransactionInstruction, anchorIx: TransactionInstruction | null): boolean {
  if (!anchorIx) {
    console.log("builder comparison: Anchor program.methods instruction was not constructed.");
    return false;
  }

  const dataMatches = manualIx.data.equals(anchorIx.data);
  const accountsMatch = accountSignature(manualIx) === accountSignature(anchorIx);
  console.log("manual vs Anchor data matches:", dataMatches);
  console.log("manual vs Anchor accounts match:", accountsMatch);

  if (!dataMatches) {
    console.log("builder difference: data differs");
  }
  if (!accountsMatch) {
    console.log("builder difference: account metas/order/signers/writable flags differ");
    const max = Math.max(manualIx.keys.length, anchorIx.keys.length);
    for (let index = 0; index < max; index += 1) {
      const manual = manualIx.keys[index];
      const anchorMeta = anchorIx.keys[index];
      const manualText = manual
        ? `${manual.pubkey.toBase58()}:${manual.isSigner}:${manual.isWritable}`
        : "<missing>";
      const anchorText = anchorMeta
        ? `${anchorMeta.pubkey.toBase58()}:${anchorMeta.isSigner}:${anchorMeta.isWritable}`
        : "<missing>";
      if (manualText !== anchorText) {
        console.log(`  account[${index}] manual=${manualText} anchor=${anchorText}`);
      }
    }
  }

  return dataMatches && accountsMatch;
}

function readKeypairPubkey(): { pubkey: string | null; equalsProgramId: boolean | null; error: string | null } {
  if (!fs.existsSync(LOCAL_KEYPAIR_PATH)) {
    return { pubkey: null, equalsProgramId: null, error: "keypair file does not exist" };
  }

  try {
    const secret = Uint8Array.from(JSON.parse(fs.readFileSync(LOCAL_KEYPAIR_PATH, "utf8")) as number[]);
    const pubkey = Keypair.fromSecretKey(secret).publicKey;
    return {
      pubkey: pubkey.toBase58(),
      equalsProgramId: pubkey.equals(PROGRAM_ID),
      error: null,
    };
  } catch (error) {
    return { pubkey: null, equalsProgramId: null, error: errorMessage(error) };
  }
}

function parseProgramDataAddress(data: Buffer): PublicKey | null {
  if (data.length < 36 || data.readUInt32LE(0) !== 2) {
    return null;
  }
  return new PublicKey(data.subarray(4, 36));
}

function parseProgramData(data: Buffer): { slot: bigint | null; authority: PublicKey | null } {
  if (data.length < 16 || data.readUInt32LE(0) !== 3) {
    return { slot: null, authority: null };
  }

  const slot = data.readBigUInt64LE(4);
  const authorityOption = data.readUInt32LE(12);
  const authority = authorityOption === 1 && data.length >= 48 ? new PublicKey(data.subarray(16, 48)) : null;
  return { slot, authority };
}

function runSolanaCli(args: string[]): { ok: boolean; output: string } {
  try {
    return {
      ok: true,
      output: execFileSync("solana", args, {
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"],
      }),
    };
  } catch (error) {
    return { ok: false, output: errorMessage(error) };
  }
}

function camelAccounts(accounts: SetupAccounts): Record<string, PublicKey> {
  return {
    greenLabelConfig: deriveGreenLabelConfig(),
    authority: accounts.authority,
    usdcMint: accounts.usdcMint,
    treasuryUsdcStateV2: accounts.treasuryUsdcStateV2,
    baseBondTreasuryVault: accounts.baseBondTreasuryVault,
    reliefOrRiskVault: accounts.reliefOrRiskVault,
    vaultAuthorityV2: accounts.vaultAuthorityV2,
    securityGovernanceConfig: accounts.securityGovernanceConfig,
    systemProgram: SystemProgram.programId,
  };
}

function snakeAccounts(accounts: SetupAccounts): Record<string, PublicKey> {
  return {
    green_label_config: deriveGreenLabelConfig(),
    authority: accounts.authority,
    usdc_mint: accounts.usdcMint,
    treasury_usdc_state_v2: accounts.treasuryUsdcStateV2,
    base_bond_treasury_vault: accounts.baseBondTreasuryVault,
    relief_or_risk_vault: accounts.reliefOrRiskVault,
    vault_authority_v2: accounts.vaultAuthorityV2,
    security_governance_config: accounts.securityGovernanceConfig,
    system_program: SystemProgram.programId,
  };
}

async function buildAnchorMethodInstruction(
  provider: anchor.AnchorProvider,
  accounts: SetupAccounts,
): Promise<AnchorMethodIxResult> {
  const result: AnchorMethodIxResult = {
    ix: null,
    methodName: null,
    accountsStyle: null,
    errors: [],
  };

  let program: anchor.Program;
  try {
    program = loadProgram(provider);
  } catch (error) {
    result.errors.push(`loadProgram failed: ${errorMessage(error)}`);
    return result;
  }

  const methods = (program as unknown as { methods: Record<string, (...args: unknown[]) => unknown> }).methods;
  const methodName = ["initializeGreenLabelConfig", "initialize_green_label_config"].find(
    (name) => typeof methods[name] === "function",
  );

  if (!methodName) {
    result.errors.push("program.methods has neither initializeGreenLabelConfig nor initialize_green_label_config");
    return result;
  }

  result.methodName = methodName;

  const attempts: Array<{ style: string; accounts: Record<string, PublicKey> }> = [
    { style: "camelCase", accounts: camelAccounts(accounts) },
    { style: "snake_case", accounts: snakeAccounts(accounts) },
  ];

  for (const attempt of attempts) {
    try {
      const builder = methods[methodName]() as {
        accounts(accountMap: Record<string, PublicKey>): { instruction(): Promise<TransactionInstruction> };
      };
      result.ix = await builder.accounts(attempt.accounts).instruction();
      result.accountsStyle = attempt.style;
      return result;
    } catch (error) {
      result.errors.push(`${methodName} with ${attempt.style} accounts failed: ${errorMessage(error)}`);
    }
  }

  return result;
}

async function simulateInstruction(
  provider: anchor.AnchorProvider,
  label: string,
  ix: TransactionInstruction | null,
): Promise<SimulateResult> {
  if (!ix) {
    return {
      label,
      err: null,
      logs: [],
      unitsConsumed: null,
      first8: "<missing>",
      skippedReason: "instruction was not constructed",
    };
  }

  try {
    const transaction = new Transaction().add(ix);
    const latestBlockhash = await provider.connection.getLatestBlockhash("confirmed");
    transaction.recentBlockhash = latestBlockhash.blockhash;
    transaction.feePayer = provider.wallet.publicKey;
    const signedTransaction = await provider.wallet.signTransaction(transaction);
    const simulation = await provider.connection.simulateTransaction(signedTransaction);

    return {
      label,
      err: simulation.value.err,
      logs: simulation.value.logs ?? [],
      unitsConsumed: simulation.value.unitsConsumed ?? null,
      first8: ix.data.subarray(0, 8).toString("hex"),
    };
  } catch (error) {
    return {
      label,
      err: errorMessage(error),
      logs: [],
      unitsConsumed: null,
      first8: ix.data.subarray(0, 8).toString("hex"),
    };
  }
}

function printSimulation(result: SimulateResult): void {
  console.log(`${result.label} simulation:`);
  if (result.skippedReason) {
    console.log("  skipped:", result.skippedReason);
    return;
  }
  console.log("  err:", JSON.stringify(result.err));
  console.log("  unitsConsumed:", result.unitsConsumed);
  console.log("  first 8 bytes:", result.first8);
  console.log("  logs:");
  for (const log of result.logs) {
    console.log(`    ${log}`);
  }
}

function simulationText(result: SimulateResult): string {
  return `${JSON.stringify(result.err)}\n${result.logs.join("\n")}`;
}

function isFallbackFailure(result: SimulateResult): boolean {
  return /InstructionFallbackNotFound|Fallback functions are not supported/i.test(simulationText(result));
}

function isNextAccountError(result: SimulateResult): boolean {
  return /Constraint|account already in use|custom program error|GreenLabel|Error Code:/i.test(
    simulationText(result),
  );
}

function chooseVerdict(args: {
  programIdMismatch: boolean;
  localIdlStale: boolean;
  localSoStale: boolean;
  dumpFailed: boolean;
  builderMismatch: boolean;
  expectedDiscriminator: string;
  manualSimulation: SimulateResult | null;
  anchorSimulation: SimulateResult | null;
}): Verdict {
  if (args.programIdMismatch) {
    return "PROGRAM_ID_MISMATCH";
  }
  if (args.localIdlStale) {
    return "LOCAL_IDL_STALE";
  }
  if (args.builderMismatch) {
    return "SCRIPT_INSTRUCTION_BUILDER_MISMATCH";
  }

  const manual = args.manualSimulation;
  const anchorIx = args.anchorSimulation;
  if (manual && manual.first8 !== args.expectedDiscriminator) {
    return "DISCRIMINATOR_BUG";
  }
  if (anchorIx && !anchorIx.skippedReason && anchorIx.first8 !== args.expectedDiscriminator) {
    return "DISCRIMINATOR_BUG";
  }

  const manualFallback = manual ? isFallbackFailure(manual) : false;
  const anchorFallback = anchorIx ? isFallbackFailure(anchorIx) : false;
  if (manualFallback && anchorIx && !anchorIx.skippedReason && !anchorFallback) {
    return "MANUAL_BUILDER_BUG";
  }
  if (manualFallback && (!anchorIx || anchorFallback) && manual?.first8 === args.expectedDiscriminator) {
    return "DEPLOYED_BINARY_OR_PROGRAM_MISMATCH";
  }

  if (manual && !manual.err) {
    return "SETUP_SIMULATION_OK_CAN_RUN_REAL_SETUP";
  }
  if (manual && isNextAccountError(manual)) {
    return "DISCRIMINATOR_FIXED_NEXT_ACCOUNT_ERROR";
  }
  if (args.localSoStale) {
    return "LOCAL_SO_STALE";
  }
  if (args.dumpFailed) {
    return "RPC_DUMP_FAILED_BUT_SIMULATION_DIAGNOSTIC_COMPLETE";
  }
  return "UNKNOWN_NEEDS_MANUAL_REVIEW";
}

function nextActionFor(verdict: Verdict): string {
  switch (verdict) {
    case "SETUP_SIMULATION_OK_CAN_RUN_REAL_SETUP":
      return "Run the real setup only after confirming the printed accounts and environment are intended.";
    case "DISCRIMINATOR_BUG":
      return "Fix the script instruction discriminator construction before running setup.";
    case "SCRIPT_INSTRUCTION_BUILDER_MISMATCH":
      return "Compare manual builder vs Anchor methods output and update the script builder to match Anchor.";
    case "MANUAL_BUILDER_BUG":
      return "Switch setup to the Anchor-generated instruction or fix the manual builder mismatch.";
    case "DEPLOYED_BINARY_OR_PROGRAM_MISMATCH":
      return "Verify the deployed Devnet binary and Program ID; the local instruction bytes appear correct.";
    case "LOCAL_IDL_STALE":
      return "Refresh the local IDL used by scripts before trusting Anchor methods diagnostics.";
    case "LOCAL_SO_STALE":
      return "Rebuild or compare the local .so in a temp copy; do not deploy from this diagnostic.";
    case "PROGRAM_ID_MISMATCH":
      return "Fix local script environment/path mismatch without changing the deployed Program ID.";
    case "RPC_DUMP_FAILED_BUT_SIMULATION_DIAGNOSTIC_COMPLETE":
      return "Review simulation output; retry program dump separately if binary comparison is still needed.";
    case "DISCRIMINATOR_FIXED_NEXT_ACCOUNT_ERROR":
      return "Discriminator is likely fixed; investigate the next printed account/constraint error.";
    case "UNKNOWN_NEEDS_MANUAL_REVIEW":
      return "Review the full diagnostic output manually.";
  }
}

async function main(): Promise<void> {
  let verdict: Verdict = "UNKNOWN_NEEDS_MANUAL_REVIEW";
  let nextAction = "Review the full diagnostic output manually.";
  let provider: anchor.AnchorProvider | null = null;
  let dumpFailed = false;
  let localSoStale = false;
  let programIdMismatch = false;

  const rawIdl = readRawIdl();
  const runtimeIdl = loadIdl();
  const localSo = existsSizeSha256(LOCAL_SO_PATH);
  const keypairPubkey = readKeypairPubkey();
  const rawIdlAddressMismatch = Boolean(rawIdl.address && rawIdl.address !== PROGRAM_ID.toBase58());
  const rawIdlMetadataAddressMismatch = Boolean(
    rawIdl.metadata?.address && rawIdl.metadata.address !== PROGRAM_ID.toBase58(),
  );
  programIdMismatch =
    keypairPubkey.equalsProgramId === false || rawIdlAddressMismatch || rawIdlMetadataAddressMismatch;

  section("A. Environment");
  console.log("cwd:", process.cwd());
  console.log("node version:", process.version);
  console.log("package script cwd:", process.env.INIT_CWD ?? process.cwd());
  console.log("npm lifecycle event:", process.env.npm_lifecycle_event ?? "<not npm>");
  console.log("ANCHOR_PROVIDER_URL:", process.env.ANCHOR_PROVIDER_URL ?? "<unset>");
  console.log("ANCHOR_WALLET:", process.env.ANCHOR_WALLET ?? "<unset>");

  try {
    provider = loadProvider();
    console.log("provider publicKey:", provider.wallet.publicKey.toBase58());
    console.log(
      "solana RPC endpoint:",
      (provider.connection as unknown as { rpcEndpoint?: string }).rpcEndpoint ?? "<unknown>",
    );
  } catch (error) {
    console.log("provider load failed:", errorMessage(error));
  }

  console.log("Program ID:", PROGRAM_ID.toBase58());
  console.log("local IDL path:", IDL_PATH);
  console.log("local IDL address:", rawIdl.address ?? "<missing>");
  console.log("local IDL metadata address:", rawIdl.metadata?.address ?? "<missing>");
  console.log("local IDL address equals Program ID:", rawIdl.address ? !rawIdlAddressMismatch : "<missing>");
  console.log(
    "local IDL metadata address equals Program ID:",
    rawIdl.metadata?.address ? !rawIdlMetadataAddressMismatch : "<missing>",
  );
  console.log("target/deploy .so exists:", localSo.exists);
  console.log("target/deploy .so size:", localSo.size ?? "<missing>");
  console.log("target/deploy .so sha256:", localSo.sha256 ?? "<missing>");
  console.log("target/deploy keypair path:", LOCAL_KEYPAIR_PATH);
  console.log("target/deploy keypair pubkey:", keypairPubkey.pubkey ?? "<missing>");
  console.log("target/deploy keypair equals Program ID:", keypairPubkey.equalsProgramId ?? "<unknown>");
  if (keypairPubkey.error) {
    console.log("target/deploy keypair read error:", keypairPubkey.error);
  }

  section("B. IDL checks");
  const matchingNames = rawIdl.instructions
    .map((instruction) => instruction.name)
    .filter((name) => /green_label|greenlabel|proposal|queue/i.test(name));
  console.log("matching instruction names:", matchingNames.length > 0 ? matchingNames.join(", ") : "<none>");

  let localIdlStale = false;
  for (const name of REQUIRED_INSTRUCTIONS) {
    const instruction = rawIdl.instructions.find((ix) => ix.name === name);
    const idlHex = discriminatorHex(rawIdl, name);
    const fallbackHex = anchorDiscriminator(name).toString("hex");
    if (!instruction) {
      localIdlStale = true;
    }
    console.log(`${name}:`);
    console.log("  IDL exists:", Boolean(instruction));
    console.log("  IDL discriminator hex:", idlHex ?? "<missing>");
    console.log('  fallback sha256("global:" + name).slice(0,8) hex:', fallbackHex);
    console.log("  discriminator matches fallback:", idlHex ? idlHex === fallbackHex : false);
  }

  section("C. setup instruction construction");
  if (!provider) {
    console.log("Skipping instruction construction because provider failed to load.");
    verdict = localIdlStale ? "LOCAL_IDL_STALE" : "UNKNOWN_NEEDS_MANUAL_REVIEW";
  } else {
    const treasuryPdas = deriveTreasuryPdas();
    const accounts: SetupAccounts = {
      authority: provider.wallet.publicKey,
      usdcMint: readPublicKeyEnv("USDC_MINT", DEVNET_USDC_MINT),
      treasuryUsdcStateV2: treasuryPdas.treasuryUsdcStateV2,
      baseBondTreasuryVault: readPublicKeyEnv("BASE_BOND_TREASURY_VAULT", treasuryPdas.buildersUsdcVault),
      reliefOrRiskVault: readPublicKeyEnv("RELIEF_OR_RISK_VAULT", treasuryPdas.reliefUsdcVault),
      vaultAuthorityV2: treasuryPdas.vaultAuthorityV2,
      securityGovernanceConfig: deriveGovernanceConfig(),
    };
    const manualIx = buildInitializeGreenLabelConfigIx(accounts);
    const anchorResult = await buildAnchorMethodInstruction(provider, accounts);

    printInstruction(manualIx, "method 1 manual builder", runtimeIdl);
    if (anchorResult.methodName) {
      console.log("Anchor method name:", anchorResult.methodName);
    }
    if (anchorResult.accountsStyle) {
      console.log("Anchor accounts style:", anchorResult.accountsStyle);
    }
    for (const error of anchorResult.errors) {
      console.log("Anchor construction note:", error);
    }
    if (anchorResult.ix) {
      printInstruction(anchorResult.ix, "method 2 Anchor program.methods", runtimeIdl);
    }

    const buildersMatch = printBuilderComparison(manualIx, anchorResult.ix);
    const builderMismatch = Boolean(anchorResult.ix) && !buildersMatch;
    if (builderMismatch) {
      console.log("VERDICT: SCRIPT_INSTRUCTION_BUILDER_MISMATCH");
    }

    section("D. Devnet program state");
    const endpoint =
      (provider.connection as unknown as { rpcEndpoint?: string }).rpcEndpoint ??
      process.env.ANCHOR_PROVIDER_URL ??
      DEVNET_URL;
    const programInfo = await provider.connection.getAccountInfo(PROGRAM_ID);
    console.log("program account exists:", Boolean(programInfo));
    if (programInfo) {
      console.log("program executable:", programInfo.executable);
      console.log("program owner:", programInfo.owner.toBase58());
      console.log("program lamports:", programInfo.lamports);
      console.log("program data length:", programInfo.data.length);
      const programDataAddress = parseProgramDataAddress(programInfo.data);
      console.log("parsed ProgramData Address:", programDataAddress?.toBase58() ?? "<unavailable>");
      if (programDataAddress) {
        const programDataInfo = await provider.connection.getAccountInfo(programDataAddress);
        const programData = programDataInfo ? parseProgramData(programDataInfo.data) : null;
        console.log("parsed ProgramData Last Deployed Slot:", programData?.slot?.toString() ?? "<unavailable>");
        console.log("parsed ProgramData Authority:", programData?.authority?.toBase58() ?? "<none/unavailable>");
        console.log("parsed ProgramData Data Length:", programDataInfo?.data.length ?? "<unavailable>");
      }
    }

    const programShow = runSolanaCli(["program", "show", PROGRAM_ID.toBase58(), "--url", endpoint]);
    console.log("solana program show ok:", programShow.ok);
    console.log(programShow.output.trim() || "<empty>");

    section("E. on-chain .so comparison");
    try {
      fs.mkdirSync(path.dirname(DUMP_PATH), { recursive: true });
    } catch (error) {
      console.log("mkdir /tmp failed:", errorMessage(error));
    }
    const dump = runSolanaCli(["program", "dump", PROGRAM_ID.toBase58(), DUMP_PATH, "--url", endpoint]);
    console.log("solana program dump ok:", dump.ok);
    if (!dump.ok) {
      dumpFailed = true;
      console.log("PROGRAM_DUMP_FAILED:", dump.output);
    } else {
      console.log(dump.output.trim() || "<empty>");
      const dumpedSo = existsSizeSha256(DUMP_PATH);
      console.log("dump path:", DUMP_PATH);
      console.log("dump .so size:", dumpedSo.size ?? "<missing>");
      console.log("dump .so sha256:", dumpedSo.sha256 ?? "<missing>");
      console.log("local target/deploy .so sha256:", localSo.sha256 ?? "<missing>");
      localSoStale = Boolean(dumpedSo.sha256 && localSo.sha256 && dumpedSo.sha256 !== localSo.sha256);
      console.log("dump matches local .so:", !localSoStale);
    }

    section("F. read-only simulation");
    const manualSimulation = await simulateInstruction(provider, "method 1 manual builder", manualIx);
    const anchorSimulation = await simulateInstruction(provider, "method 2 Anchor program.methods", anchorResult.ix);
    printSimulation(manualSimulation);
    printSimulation(anchorSimulation);

    const expectedDiscriminator = anchorDiscriminator("initialize_green_label_config").toString("hex");
    verdict = chooseVerdict({
      programIdMismatch,
      localIdlStale,
      localSoStale,
      dumpFailed,
      builderMismatch,
      expectedDiscriminator,
      manualSimulation,
      anchorSimulation,
    });
  }

  nextAction = nextActionFor(verdict);
  section("G. Summary");
  console.log("VERDICT:", verdict);
  console.log("NEXT_ACTION:", nextAction);
  console.log("DO_NOT_DO: do not send transactions, deploy, run anchor keys sync, or modify Program ID/keypairs/target/deploy from this diagnostic.");
}

main().catch((error) => {
  console.error("diagnose-green-label-setup failed:");
  console.error(errorMessage(error));
  process.exit(1);
});
