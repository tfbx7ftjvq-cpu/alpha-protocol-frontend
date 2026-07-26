import { Transaction } from "@solana/web3.js";
import {
  DEVNET_USDC_MINT,
  MODULE_CODES,
  SYS,
  accountExists,
  buildIx,
  deriveGovernanceConfig,
  deriveProtocolModuleRegistry,
  deriveTreasuryPdas,
  deriveVictimReliefConfig,
  deriveVictimReliefPolicy,
  i64,
  loadDevnetContext,
  meta,
  readI64Default,
  readU16Default,
  readU32Default,
  readU64Default,
  sendOrPlan,
  u16,
  u32,
  u64,
} from "../devnet/alpha-v1/common";

const REQUIRED_INSTRUCTIONS = [
  "initialize_victim_relief_config_v1",
  "initialize_victim_relief_policy_v1",
  "initialize_protocol_module_registry_v1",
];

function buildPolicyPayload(): Buffer {
  return Buffer.concat([
    u64(readU64Default("MIN_CLAIM_AMOUNT_USDC", 1_000_000n)),
    u64(readU64Default("MAX_CLAIM_AMOUNT_USDC", 1_000_000_000n)),
    u64(readU64Default("MAX_PAYOUT_PER_CASE_USDC", 100_000_000n)),
    i64(readI64Default("EVIDENCE_WINDOW_SECONDS", 300n)),
    i64(readI64Default("REVIEW_WINDOW_SECONDS", 300n)),
    i64(readI64Default("APPEAL_WINDOW_SECONDS", 300n)),
    i64(readI64Default("SUBMISSION_COOLDOWN_SECONDS", 0n)),
    u32(readU32Default("MAX_EVIDENCE_ITEMS", 8)),
    u16(readU16Default("MAX_ACTIVE_CASES_PER_CLAIMANT", 2)),
  ]);
}

function buildVictimReliefModuleRegistryPayload(): Buffer {
  // Anchor/Borsh enum ordinal for ProtocolModuleIdV1::VictimRelief is 2.
  return Buffer.concat([Buffer.from([2]), u16(1)]);
}

async function main(): Promise<void> {
  const ctx = await loadDevnetContext({
    scriptName: "victim-relief-setup",
    sendsTransactions: true,
    requiredInstructions: REQUIRED_INSTRUCTIONS,
  });
  const { connection } = ctx.provider;
  const payer = ctx.wallet;
  const governanceConfig = deriveGovernanceConfig();
  const treasury = deriveTreasuryPdas();
  const victimReliefConfig = deriveVictimReliefConfig();
  const victimReliefPolicy = deriveVictimReliefPolicy(victimReliefConfig);
  const registry = deriveProtocolModuleRegistry(MODULE_CODES.VictimRelief);

  console.log("payer:", payer.toBase58());
  console.log("governance_config_v1:", governanceConfig.toBase58());
  console.log("treasury_config_v2:", treasury.treasuryConfigV2.toBase58());
  console.log("victim_relief_config_v1:", victimReliefConfig.toBase58());
  console.log("victim_relief_policy_v1:", victimReliefPolicy.toBase58());
  console.log("protocol_module_registry_v1:VictimRelief:", registry.toBase58());

  for (const [label, account] of [
    ["governance_config_v1", governanceConfig],
    ["treasury_config_v2", treasury.treasuryConfigV2],
    ["devnet_usdc_mint", DEVNET_USDC_MINT],
  ] as const) {
    if (!(await accountExists(connection, account))) {
      throw new Error(`${label} does not exist: ${account.toBase58()}`);
    }
  }

  const tx = new Transaction();
  if (!(await accountExists(connection, victimReliefConfig))) {
    tx.add(
      buildIx(ctx.idl, "initialize_victim_relief_config_v1", [
        meta(victimReliefConfig, false, true),
        meta(governanceConfig),
        meta(treasury.treasuryConfigV2),
        meta(DEVNET_USDC_MINT),
        meta(payer, true, true),
        meta(payer, true),
        meta(SYS),
      ]),
    );
  } else {
    console.log("VictimReliefConfigV1 already exists; skipping config init.");
  }

  if (!(await accountExists(connection, victimReliefPolicy))) {
    tx.add(
      buildIx(
        ctx.idl,
        "initialize_victim_relief_policy_v1",
        [
          meta(victimReliefConfig, false, true),
          meta(victimReliefPolicy, false, true),
          meta(payer, true, true),
          meta(payer, true),
          meta(SYS),
        ],
        buildPolicyPayload(),
      ),
    );
  } else {
    console.log("VictimReliefPolicyV1 already exists; skipping policy init.");
  }

  if (!(await accountExists(connection, registry))) {
    tx.add(
      buildIx(
        ctx.idl,
        "initialize_protocol_module_registry_v1",
        [
          meta(payer, true, true),
          meta(payer, true),
          meta(governanceConfig),
          meta(registry, false, true),
          meta(SYS),
        ],
        buildVictimReliefModuleRegistryPayload(),
      ),
    );
  } else {
    console.log("VictimRelief ProtocolModuleRegistryV1 already exists; skipping registry init.");
  }

  if (tx.instructions.length === 0) {
    console.log("Victim Relief setup is already initialized. No transaction needed.");
    return;
  }

  await sendOrPlan(ctx, "victim_relief_setup", tx);
}

main().catch((error) => {
  console.error("victim-relief setup failed:");
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});