import {
  DEVNET_USDC_MINT,
  EXPECTED_RELIEF_USDC_VAULT,
  EXPECTED_VAULT_AUTHORITY_V2,
  MODULE_CODES,
  deriveGovernanceConfig,
  deriveProtocolModuleRegistry,
  deriveTreasuryPdas,
  deriveVictimReliefConfig,
  deriveVictimReliefPolicy,
  loadDevnetContext,
  printAccount,
  printTokenAccount,
  readProgramDeployment,
} from "../devnet/alpha-v1/common";

async function main(): Promise<void> {
  const ctx = await loadDevnetContext({
    scriptName: "victim-relief-inspect",
    sendsTransactions: false,
  });
  const { connection } = ctx.provider;
  const treasury = deriveTreasuryPdas();
  const governanceConfig = deriveGovernanceConfig();
  const victimReliefConfig = deriveVictimReliefConfig();
  const victimReliefPolicy = deriveVictimReliefPolicy(victimReliefConfig);
  const victimReliefRegistry = deriveProtocolModuleRegistry(MODULE_CODES.VictimRelief);
  const deployment = await readProgramDeployment(connection);

  console.log("=== Program Deployment ===");
  console.log("program_data:", deployment.programData?.toBase58() ?? "<missing>");
  console.log("upgrade_authority:", deployment.upgradeAuthority?.toBase58() ?? "<none>");
  console.log("program_data_length:", deployment.dataLength);

  console.log("=== Core PDAs ===");
  await printAccount(connection, "governance_config_v1", governanceConfig);
  await printAccount(connection, "treasury_config_v2", treasury.treasuryConfigV2);
  await printAccount(connection, "victim_relief_config_v1", victimReliefConfig);
  await printAccount(connection, "victim_relief_policy_v1", victimReliefPolicy);
  await printAccount(connection, "protocol_module_registry_v1:VictimRelief", victimReliefRegistry);

  console.log("=== Treasury / Vaults ===");
  await printTokenAccount(connection, "relief_usdc_vault", treasury.reliefUsdcVault);
  await printAccount(connection, "vault_authority_v2", treasury.vaultAuthorityV2);
  console.log("expected_devnet_usdc_mint:", DEVNET_USDC_MINT.toBase58());
  console.log("expected_relief_usdc_vault:", EXPECTED_RELIEF_USDC_VAULT.toBase58());
  console.log("expected_vault_authority_v2:", EXPECTED_VAULT_AUTHORITY_V2.toBase58());

  console.log("=== IDL Freshness ===");
  console.log("instruction_count:", ctx.idl.instructions.length);
  console.log(
    "has_victim_relief_config_init:",
    ctx.idl.instructions.some((ix) => ix.name === "initialize_victim_relief_config_v1"),
  );
  console.log(
    "has_victim_relief_payout:",
    ctx.idl.instructions.some((ix) => ix.name === "execute_victim_relief_approved_payout_v1"),
  );
}

main().catch((error) => {
  console.error("victim-relief inspect failed:");
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});