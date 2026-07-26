import {
  EXPECTED_UPGRADE_AUTHORITY,
  MODULE_CODES,
  deriveGovernanceConfig,
  deriveProtocolActivationRecord,
  deriveProtocolAuthorityControl,
  deriveProtocolModuleRegistry,
  loadDevnetContext,
  printAccount,
  readProgramDeployment,
} from "../devnet/alpha-v1/common";

async function main(): Promise<void> {
  const ctx = await loadDevnetContext({ scriptName: "protocol-authority-inspect", sendsTransactions: false });
  const { connection } = ctx.provider;
  const governanceConfig = deriveGovernanceConfig();
  const authorityControl = deriveProtocolAuthorityControl(governanceConfig);
  const protocolRegistry = deriveProtocolModuleRegistry(MODULE_CODES.Protocol);
  const activationRecord = deriveProtocolActivationRecord(authorityControl);
  const deployment = await readProgramDeployment(connection);

  console.log("=== Program Authority ===");
  console.log("program_data:", deployment.programData?.toBase58() ?? "<missing>");
  console.log("upgrade_authority:", deployment.upgradeAuthority?.toBase58() ?? "<none>");
  console.log("expected_upgrade_authority:", EXPECTED_UPGRADE_AUTHORITY.toBase58());
  console.log("wallet_matches_upgrade_authority:", deployment.upgradeAuthority?.equals(ctx.wallet) ?? false);

  console.log("=== Protocol Authority PDAs ===");
  await printAccount(connection, "governance_config_v1", governanceConfig);
  await printAccount(connection, "protocol_authority_control_v1", authorityControl);
  await printAccount(connection, "protocol_module_registry_v1:Protocol", protocolRegistry);
  await printAccount(connection, "protocol_dao_control_activation_record_v1", activationRecord);
}

main().catch((error) => {
  console.error("protocol-authority inspect failed:");
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});